//! Adapter `IrohBlobStore: P2pBlobStore`, atrelado ao motor real do `iroh-blobs`.
//!
//! Atrelado deliberadamente ao transporte `iroh` (feature `iroh-blobs-adapter` ativa `iroh`
//! junto, ver `Cargo.toml`): `IrohTransport` já é o único dono do `Endpoint` (`core/transport/iroh/transport.rs`),
//! e é ele quem constrói este adapter internamente e o expõe via `P2pTransport::blobs()`. Nada
//! aqui é público além do que `core::blobs::P2pBlobStore` já expõe.

mod config;
mod gc;

use std::{collections::HashMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use iroh::{endpoint::Connection, Endpoint, EndpointAddr, EndpointId};
use iroh_blobs::{api::Store, Hash, HashAndFormat};
use tokio::{
    io::AsyncRead,
    sync::{Mutex, RwLock},
};

pub use self::config::IrohBlobsConfig;
use super::{hash::BlobHash, store::P2pBlobStore};
use crate::infra::{error::ConnectionError, peer::PeerAddr};

/// Teto pra quanto tempo `fetch()` espera por uma conexão/path silenciosamente morto — mesma
/// classe de bug já documentada e corrigida para conexões via pool em
/// `core/transport/iroh/transport.rs::invalidate_forces_fresh_dial_on_next_open_bi`; aqui a
/// conexão vem de `IrohBlobStore::dial_or_reuse` (próprio pool, ALPN `iroh_blobs::ALPN`,
/// separado do de `IrohTransport` porque atende um protocolo diferente). Sem timeout, o
/// `.complete()` do iroh-blobs pode ficar esperando indefinidamente por um path que nunca vai
/// responder de novo, sem nunca resolver a `Err` — o que impede até o mecanismo genérico de
/// retry do `NetworkManager` (que só reage a um handler retornando `Err`) de sequer entrar em
/// ação. Timeout aqui garante que uma conexão/path morto sempre termina em erro, deixando quem
/// chama (`ChapterTransfer::fetch_reader` e o resto do protocolo de sync) livre pra tratar
/// como qualquer outra falha de rede — pular o item, tentar de novo mais tarde, etc.
const BLOB_FETCH_TIMEOUT: Duration = Duration::from_secs(15);

fn to_iroh_hash(hash: &BlobHash) -> Hash {
    Hash::from_bytes(*hash.as_bytes())
}

fn to_blob_hash(hash: Hash) -> BlobHash {
    BlobHash::from_bytes(*hash.as_bytes())
}

fn parse_endpoint_addr(peer: &PeerAddr) -> Result<EndpointAddr, ConnectionError> {
    serde_json::from_slice(&peer.addrs).map_err(|_| ConnectionError::PeerNotFound(peer.id.clone()))
}

/// Store de blobs content-addressed apoiado no `iroh-blobs`, com transferência real via QUIC.
pub struct IrohBlobStore {
    store: Store,
    endpoint: Endpoint,
    /// Sempre `BLOB_FETCH_TIMEOUT` em produção — campo existe (em vez do `const` direto em
    /// `fetch()`) só pra um teste específico poder injetar um valor curto e simular "peer
    /// inalcançável" sem afetar o timeout de verdade usado pelos outros testes deste módulo
    /// (transferência de blob grande, fetch concorrente sob GC), que precisam do valor de
    /// produção pra não estourar por serem legitimamente mais lentos que um timeout de teste
    /// artificialmente curto.
    fetch_timeout: Duration,
    /// Conexão física reaproveitada entre fetches sucessivos pro MESMO peer — sem isso, cada
    /// `fetch()` (um por capítulo/blob, discado direto aqui fora do pool de `IrohTransport`)
    /// abria uma conexão QUIC nova, reproduzindo em sequência rápida (um fetch atrás do outro
    /// na sincronização de uma pasta) o mesmo padrão que já colidia contra relay
    /// (`MultipathNotNegotiated`, handshake falhando) documentado e corrigido pro pool de
    /// `IrohTransport::open_bi` — aqui nunca tinha ganhado o mesmo tratamento.
    connections: Arc<RwLock<HashMap<EndpointId, Connection>>>,
    /// Serializa `check cache → dial → insert cache` por peer — mesmo motivo do
    /// `dial_locks` em `transport.rs`.
    dial_locks: Mutex<HashMap<EndpointId, Arc<tokio::sync::Mutex<()>>>>,
}

impl IrohBlobStore {
    /// Constrói o adapter a partir da configuração de storage e do `Endpoint` já vinculado do
    /// `IrohTransport` — chamado internamente por `IrohTransportBuilder::build`.
    pub(crate) async fn new(
        config: &IrohBlobsConfig, endpoint: Endpoint,
    ) -> Result<Self, ConnectionError> {
        let store = config.build_store().await?;
        Ok(Self {
            store,
            endpoint,
            fetch_timeout: BLOB_FETCH_TIMEOUT,
            connections: Arc::new(RwLock::new(HashMap::new())),
            dial_locks: Mutex::new(HashMap::new()),
        })
    }

    /// O `Store` interno, usado por `IrohTransport` para montar o `BlobsProtocol` no lado
    /// inbound. Não faz parte de `P2pBlobStore` — é um detalhe de wiring entre os dois adapters.
    pub(crate) fn inner_store(&self) -> &Store {
        &self.store
    }
}

#[async_trait]
impl P2pBlobStore for IrohBlobStore {
    async fn put(&self, data: Vec<u8>) -> Result<BlobHash, ConnectionError> {
        let tag = self.store.blobs().add_bytes(data).await?;
        Ok(to_blob_hash(tag.hash))
    }

    async fn get(
        &self, hash: &BlobHash,
    ) -> Result<Box<dyn AsyncRead + Send + Unpin>, ConnectionError> {
        let iroh_hash = to_iroh_hash(hash);
        if !self.store.blobs().has(iroh_hash).await? {
            return Err(ConnectionError::BlobNotFound(hash.to_string()));
        }
        Ok(Box::new(self.store.blobs().reader(iroh_hash)))
    }

    async fn has(&self, hash: &BlobHash) -> Result<bool, ConnectionError> {
        Ok(self.store.blobs().has(to_iroh_hash(hash)).await?)
    }

    async fn remove(&self, hash: &BlobHash) -> Result<(), ConnectionError> {
        gc::untag(&self.store, to_iroh_hash(hash)).await
    }

    async fn fetch(&self, hash: &BlobHash, from: &PeerAddr) -> Result<(), ConnectionError> {
        let addr = parse_endpoint_addr(from)?;
        let iroh_hash = to_iroh_hash(hash);

        match tokio::time::timeout(self.fetch_timeout, self.dial_and_fetch(addr, iroh_hash)).await {
            Ok(result) => result,
            Err(_elapsed) => {
                // Mesma limpeza do branch de erro abaixo — `untag` é seguro mesmo se o timeout
                // bateu antes da tag chegar a ser criada (nada casa em `tags_for_hash`, vira
                // no-op). Sem isso, um timeout durante a fase de tag-já-criada-mas-fetch-preso
                // deixaria uma tag permanente órfã pra sempre.
                let _ = gc::untag(&self.store, iroh_hash).await;
                // `ConnectionError::Timeout` (não um `StreamFailed(String)` próprio) pra quem
                // consome poder classificar por variante de enum, igual a qualquer outro
                // timeout de conexão (`classify_connection_error`) — o texto detalhado (segundos
                // configurados, doc de referência) só importa pro log técnico, não precisa
                // sobreviver na variante retornada pra continuar sendo classificável depois de
                // atravessar `From<GetError>`/`From<IrohConnectionError>` (ver `iroh_blobs.rs`).
                tracing::warn!(
                    layer = "iroh_blobs",
                    timeout_secs = self.fetch_timeout.as_secs(),
                    "blob fetch timed out — peer connection or path is likely stale"
                );
                Err(ConnectionError::Timeout)
            },
        }
    }
}

impl IrohBlobStore {
    /// Disca + publica a tag de proteção + baixa o blob — corpo de `fetch()` isolado numa
    /// função própria só pra poder envolver a chamada inteira (não só o `.complete()`) num
    /// único `tokio::time::timeout` em `fetch()`, sem duplicar a lógica de limpeza de tag em
    /// dois lugares (timeout vs. erro normal).
    async fn dial_and_fetch(
        &self, addr: EndpointAddr, iroh_hash: Hash,
    ) -> Result<(), ConnectionError> {
        let peer_id = addr.id;
        let conn = self.dial_or_reuse(addr).await?;

        // `remote().fetch()` não usa nenhuma proteção interna — confirmado lendo o código-fonte
        // do iroh-blobs (`api::remote::execute_get_sink`, que escreve os chunks direto no store)
        // e o próprio teste `gc_file_delete` da lib, que mostra um arquivo PARCIAL sendo apagado
        // por um `gc_run_once` no meio do download. Um fetch em andamento fica visível pra
        // `store.blobs().list()` (o que o GC varre) sem nada impedindo a varredura de deletar
        // ele antes mesmo do download terminar.
        //
        // Uma primeira versão deste fix criava um `temp_tag()` antes do fetch e só o soltava
        // DEPOIS de criar a tag permanente, pensando que a sobreposição entre os dois eliminava
        // qualquer janela. Não elimina: `gc_mark_task` (iroh-blobs) lê tags permanentes
        // (`tags().list()`) e temporárias (`tags().list_temp_tags()`) em DUAS chamadas separadas
        // e não-atômicas. Se a proteção de um hash migra de "só temp tag" pra "só tag permanente"
        // bem no meio dessas duas leituras — exatamente o que a troca "cria permanente, solta
        // temp" fazia — o mark não vê nenhuma das duas, mesmo o hash nunca tendo ficado
        // desprotegido do ponto de vista do nosso código. Sob 1 fetch por vez isso quase nunca
        // acontecia; sob ~20 fetches concorrentes (o caso ao vivo de `browse-cover`), a
        // probabilidade de pelo menos um cair nessa janela sobe rápido — reproduzido de forma
        // determinística em teste com fetches concorrentes mesmo com payload pequeno (download
        // quase instantâneo), provando que não é sobre duração do download, é sobre a transição
        // em si. `TempTag::new` (iroh-blobs) já documenta essa responsabilidade do chamador:
        // "make sure that temp tags created between a mark phase and a sweep phase are
        // protected" — a troca por baixo do pano viola exatamente essa garantia.
        //
        // Fix: cria a tag permanente ANTES do fetch começar, não depois. Sem transição nenhuma
        // pra qualquer varredura de GC observar de forma inconsistente — a tag existe
        // continuamente antes da primeira chamada de `list()` que algum dia vier a enxergá-la.
        // Se o fetch falhar, desfaz a tag (senão fica uma tag permanente órfã apontando pra um
        // hash sem dado nenhum).
        self.store.tags().create(HashAndFormat::raw(iroh_hash)).await?;

        if let Err(err) = self.store.remote().fetch(conn, iroh_hash).complete().await {
            let _ = gc::untag(&self.store, iroh_hash).await;
            // A conexão cacheada pode ter morrido no meio deste fetch — descarta pra garantir
            // que o PRÓXIMO fetch pro mesmo peer disque uma conexão física nova em vez de
            // reaproveitar a mesma que acabou de falhar.
            self.invalidate(&peer_id).await;
            return Err(err.into());
        }

        Ok(())
    }

    /// Disca (ou reaproveita) a conexão física pro peer — mesmo padrão de
    /// `IrohTransport::dial_or_reuse` (`transport.rs`): sem isso, cada `fetch()` (um por
    /// capítulo/blob) discava do zero, batendo na mesma colisão de handshake concorrente já
    /// corrigida lá.
    async fn dial_or_reuse(&self, addr: EndpointAddr) -> Result<Connection, ConnectionError> {
        let peer_id = addr.id;
        let key_lock = {
            let mut locks = self.dial_locks.lock().await;
            Arc::clone(locks.entry(peer_id).or_insert_with(|| Arc::new(tokio::sync::Mutex::new(()))))
        };
        let _dial_guard = key_lock.lock().await;

        if let Some(conn) = self.connections.read().await.get(&peer_id).cloned() {
            if conn.close_reason().is_none() {
                return Ok(conn);
            }
        }

        let conn = self.endpoint.connect(addr, iroh_blobs::ALPN).await?;
        self.connections.write().await.insert(peer_id, conn.clone());
        Ok(conn)
    }

    /// Remove a conexão cacheada pro peer — chamado quando um fetch falha, pra garantir que a
    /// PRÓXIMA chamada disque uma conexão física nova em vez de reaproveitar uma morta.
    async fn invalidate(&self, peer_id: &EndpointId) {
        self.connections.write().await.remove(peer_id);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::io::AsyncReadExt;

    use super::*;
    use crate::infra::peer::PeerId;

    /// Endpoint mínimo só para satisfazer o construtor — estes testes exercitam apenas
    /// operações locais do store (`put`/`get`/`has`/`remove`), que nunca tocam a rede.
    async fn unbound_endpoint() -> Endpoint {
        Endpoint::builder(iroh::endpoint::presets::N0)
            .relay_mode(iroh::RelayMode::Disabled)
            .bind()
            .await
            .unwrap()
    }

    async fn mem_store_with_gc_interval(gc_interval: Duration) -> IrohBlobStore {
        let config = IrohBlobsConfig::Mem { gc_interval };
        IrohBlobStore::new(&config, unbound_endpoint().await).await.unwrap()
    }

    async fn mem_store() -> IrohBlobStore {
        mem_store_with_gc_interval(Duration::from_secs(30)).await
    }

    /// Endpoint capaz de discar via endereço explícito sem depender de discovery nenhum —
    /// mesmo padrão comprovado em `IrohTransportBuilder::default()`/`transport.rs`
    /// (`presets::Minimal`, só o `crypto_provider` obrigatório). Diferente de
    /// `unbound_endpoint()` acima (`presets::N0`, registra DNS/pkarr que depende de rede real
    /// e nunca foi pensado pra round-trip de verdade entre dois nodes — só pra satisfazer o
    /// construtor em testes que nunca tocam a rede).
    async fn connectable_endpoint() -> Endpoint {
        Endpoint::builder(iroh::endpoint::presets::Minimal)
            .address_lookup(iroh_mdns_address_lookup::MdnsAddressLookup::builder())
            .relay_mode(iroh::RelayMode::Disabled)
            .alpns(vec![iroh_blobs::ALPN.to_vec()])
            .bind()
            .await
            .unwrap()
    }

    async fn connectable_mem_store() -> IrohBlobStore {
        let config = IrohBlobsConfig::Mem { gc_interval: Duration::from_secs(30) };
        IrohBlobStore::new(&config, connectable_endpoint().await).await.unwrap()
    }

    /// `IrohBlobStore` sozinho (fora do `IrohTransport`) não tem nada chamando
    /// `endpoint.accept()` — em produção quem faz isso é `blobs_bridge.rs::try_accept`, chamado
    /// de dentro do loop de accept do `IrohTransport`. Testes que precisam de um round-trip de
    /// verdade (peer que ACEITA, não só disca) precisam desse mesmo dispatch manualmente.
    fn spawn_blob_accept_loop(store: Arc<IrohBlobStore>) {
        use iroh::protocol::ProtocolHandler as _;

        tokio::spawn(async move {
            loop {
                let Some(incoming) = store.endpoint.accept().await else { break };
                let Ok(conn) = incoming.await else { continue };
                if conn.alpn() != iroh_blobs::ALPN {
                    continue;
                }
                let protocol = iroh_blobs::BlobsProtocol::new(store.inner_store(), None);
                tokio::spawn(async move {
                    let _ = protocol.accept(conn).await;
                });
            }
        });
    }

    // Bug conhecido e ja documentado do iroh-blobs 0.103.0: `FsStore::load_with_opts`
    // (src/store/fs.rs) trava. O metodo cria seu PROPRIO runtime tokio multi-thread por dentro
    // (`tokio::runtime::Builder::new_multi_thread()...build()`) e spawna `Actor::new(...)` nele
    // via `handle.spawn(...).await` -- esse spawn nunca chega a rodar (confirmado: a pasta do
    // teste fica vazia, nem o arquivo do redb chega a ser criado). Reproduz de forma consistente
    // aqui, local, com ou sem instrumentacao de cobertura.
    //
    // Isso ja bateu em produção antes desse teste existir -- ver a mitigacao identica em
    // `acerola/desktop/src-tauri/src/bios/network.rs` e `acerola/android/native/rust/src/api.rs`
    // (ambos usam `IrohBlobsConfig::mem()` em vez de `Fs` por causa exatamente desse hang; store
    // em disco nao persiste blobs entre reinicios do app enquanto isso nao for corrigido
    // upstream). Ignorado ate o iroh-blobs corrigir ou a gente isolar/reportar o bug --
    // rode com `cargo test -- --ignored` pra checar se ja foi corrigido numa versao nova.
    #[ignore = "iroh-blobs 0.103.0: FsStore::load_with_opts trava (bug upstream, ja mitigado em produção com IrohBlobsConfig::mem())"]
    #[tokio::test(flavor = "multi_thread")]
    async fn fs_store_load_does_not_hang() {
        let dir =
            std::env::temp_dir().join(format!("acerola-p2p-fs-store-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let config = IrohBlobsConfig::fs(&dir);
        let result = tokio::time::timeout(
            Duration::from_secs(15),
            IrohBlobStore::new(&config, unbound_endpoint().await),
        )
        .await;
        assert!(result.is_ok(), "IrohBlobStore::new with Fs config timed out after 15s");
        let store = result.unwrap().unwrap();

        let hash = store.put(b"hello fs store".to_vec()).await.unwrap();
        let mut reader = store.get(&hash).await.unwrap();
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf, b"hello fs store");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn put_then_get_returns_same_bytes() {
        let store = mem_store().await;
        let hash = store.put(b"hello iroh-blobs".to_vec()).await.unwrap();

        let mut reader = store.get(&hash).await.unwrap();
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await.unwrap();

        assert_eq!(buf, b"hello iroh-blobs");
    }

    #[tokio::test]
    async fn has_reflects_presence_before_and_after_put() {
        let store = mem_store().await;
        let data = b"presence check".to_vec();
        let hash = to_blob_hash(Hash::new(&data));

        assert!(!store.has(&hash).await.unwrap());
        store.put(data).await.unwrap();
        assert!(store.has(&hash).await.unwrap());
    }

    #[tokio::test]
    async fn get_unknown_hash_returns_blob_not_found() {
        let store = mem_store().await;
        let unknown = BlobHash::from_bytes([0x99; 32]);

        let result = store.get(&unknown).await;

        assert!(matches!(result, Err(ConnectionError::BlobNotFound(_))));
    }

    #[tokio::test]
    async fn remove_eventually_makes_has_return_false() {
        // GC roda em ciclo, então a reclamação física não é instantânea — configuramos um
        // intervalo curto e fazemos polling, mesmo padrão já usado nos testes de latência real
        // de `core/transport/iroh/transport.rs`.
        let store = mem_store_with_gc_interval(Duration::from_millis(20)).await;
        let hash = store.put(b"to be removed".to_vec()).await.unwrap();

        store.remove(&hash).await.unwrap();

        let mut reclaimed = false;
        for _ in 0..50 {
            if !store.has(&hash).await.unwrap() {
                reclaimed = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }

        assert!(reclaimed, "blob should be physically reclaimed after remove + gc cycle");
    }

    #[tokio::test]
    async fn put_is_idempotent_for_identical_content() {
        let store = mem_store().await;
        let hash_a = store.put(b"same content".to_vec()).await.unwrap();
        let hash_b = store.put(b"same content".to_vec()).await.unwrap();

        assert_eq!(hash_a, hash_b);
    }

    /// Regressão do bug relatado em produção (Android): `sync-comic` puxando capa/banner via
    /// `fetch()` ficava preso pra sempre quando o path pro peer estava silenciosamente morto.
    /// Sem timeout, nada nunca resolve a `Err`, então nem o retry genérico do `NetworkManager`
    /// (que só reage a um handler retornando `Err`) entra em ação — o usuário via o botão de
    /// sync preso até fechar e reabrir o app.
    ///
    /// Simula "path morto" com um endpoint alvo que foi fechado antes da tentativa de conexão
    /// — `fetch()` precisa retornar `Err` num tempo limitado, não travar esperando o handshake
    /// que nunca vai completar. Usa um `fetch_timeout` curto só nesta instância (em vez do
    /// `BLOB_FETCH_TIMEOUT` de produção, 15s) só pra o teste não demorar — os outros testes
    /// deste módulo (`mem_store()`/`mem_store_with_gc_interval()`) continuam no timeout real,
    /// já que precisam de tempo de verdade pra transferências/GC concorrentes legítimas.
    #[tokio::test]
    async fn fetch_times_out_instead_of_hanging_when_peer_is_unreachable() {
        let mut store = mem_store().await;
        store.fetch_timeout = Duration::from_millis(300);

        let dead_endpoint = unbound_endpoint().await;
        let dead_addr = dead_endpoint.addr();
        let dead_peer = PeerAddr {
            id: PeerId { id: dead_endpoint.id().to_string(), device_id: None },
            addrs: serde_json::to_vec(&dead_addr).unwrap(),
        };
        dead_endpoint.close().await;

        let unknown_hash = BlobHash::from_bytes([0x42; 32]);

        let started = std::time::Instant::now();
        let result = store.fetch(&unknown_hash, &dead_peer).await;
        let elapsed = started.elapsed();

        assert!(result.is_err(), "fetch pra um peer inalcançável deveria retornar Err, não travar");
        assert!(
            elapsed < Duration::from_secs(5),
            "fetch deveria ter estourado o BLOB_FETCH_TIMEOUT de teste rapidamente, levou {elapsed:?}"
        );
    }

    /// Regressão do bug relatado em produção: sincronizar vários capítulos disparava um
    /// `fetch()` por capítulo, cada um discando uma conexão QUIC nova pro MESMO peer em
    /// sequência rápida — o mesmo padrão que colide contra relay (`MultipathNotNegotiated`,
    /// handshake falhando) já documentado e corrigido pro pool de `IrohTransport::open_bi`
    /// (`transport.rs::open_bi_reuses_cached_connection_across_calls`). Prova que um segundo
    /// `fetch()` pro mesmo peer reaproveita a MESMA conexão física da primeira chamada, em vez
    /// de discar outra.
    #[tokio::test]
    async fn fetch_reuses_cached_connection_across_calls() {
        let store_a = Arc::new(connectable_mem_store().await);
        let store_b = connectable_mem_store().await;
        spawn_blob_accept_loop(Arc::clone(&store_a));

        let payload = b"reuse test payload".to_vec();
        let hash = store_a.put(payload.clone()).await.unwrap();

        let addr_a = store_a.endpoint.addr();
        let peer_a = PeerAddr {
            id: PeerId {
                id: store_a.endpoint.id().to_string(),
                device_id: None,
            },
            addrs: serde_json::to_vec(&addr_a).unwrap(),
        };
        let peer_id = addr_a.id;

        store_b.fetch(&hash, &peer_a).await.unwrap();
        let first_stable_id =
            store_b.connections.read().await.get(&peer_id).map(|conn| conn.stable_id());
        assert!(first_stable_id.is_some(), "conexão deveria estar cacheada após o primeiro fetch");

        store_b.fetch(&hash, &peer_a).await.unwrap();
        let second_stable_id =
            store_b.connections.read().await.get(&peer_id).map(|conn| conn.stable_id());

        assert_eq!(
            first_stable_id, second_stable_id,
            "segundo fetch pro mesmo peer deveria reaproveitar a conexão cacheada, não discar outra"
        );
    }
}
