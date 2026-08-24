//! Adapter `IrohBlobStore: P2pBlobStore`, atrelado ao motor real do `iroh-blobs`.
//!
//! Atrelado deliberadamente ao transporte `iroh` (feature `iroh-blobs-adapter` ativa `iroh`
//! junto, ver `Cargo.toml`): `IrohTransport` já é o único dono do `Endpoint` (`core/transport/iroh/transport.rs`),
//! e é ele quem constrói este adapter internamente e o expõe via `P2pTransport::blobs()`. Nada
//! aqui é público além do que `core::blobs::P2pBlobStore` já expõe.

mod config;
mod gc;

use async_trait::async_trait;
use iroh::{Endpoint, EndpointAddr};
use iroh_blobs::{api::Store, Hash, HashAndFormat};
use tokio::io::AsyncRead;

pub use self::config::IrohBlobsConfig;
use super::{hash::BlobHash, store::P2pBlobStore};
use crate::infra::{error::ConnectionError, peer::PeerAddr};

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
}

impl IrohBlobStore {
    /// Constrói o adapter a partir da configuração de storage e do `Endpoint` já vinculado do
    /// `IrohTransport` — chamado internamente por `IrohTransportBuilder::build`.
    pub(crate) async fn new(
        config: &IrohBlobsConfig, endpoint: Endpoint,
    ) -> Result<Self, ConnectionError> {
        let store = config.build_store().await?;
        Ok(Self { store, endpoint })
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
        let conn = self.endpoint.connect(addr, iroh_blobs::ALPN).await?;

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
            return Err(err.into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::io::AsyncReadExt;

    use super::*;

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
        let dir = std::env::temp_dir().join(format!("acerola-p2p-fs-store-test-{}", std::process::id()));
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
}
