//! Testes de integração da transferência real de blobs entre dois nós (adapter `iroh-blobs`).

#![cfg(feature = "iroh-blobs-adapter")]

use std::{sync::Arc, time::Duration};

use tokio::time::sleep;

use crate::{
    api::{
        blobs::{BlobHash, IrohBlobsConfig},
        identity::DeviceInfo,
        peer::{PeerAddr, PeerIdentity},
        transport::IrohTransportBuilder,
        AcerolaP2p,
    },
    data::protocol::EventEmitter,
};

fn no_op_emitter() -> EventEmitter {
    Arc::new(|_: &str, _: String| {})
}

fn test_device_info() -> DeviceInfo {
    DeviceInfo {
        name: "blob-test-node".to_string(),
        os: "linux".to_string(),
        version: "1.0.0".to_string(),
    }
}

async fn build_node_with_blobs() -> AcerolaP2p {
    AcerolaP2p::builder(
        no_op_emitter(),
        IrohTransportBuilder::default().blobs(IrohBlobsConfig::mem()),
        test_device_info(),
    )
    .build()
    .await
    .unwrap()
}

/// Espera mDNS descobrir o peer, mesmo padrão já usado em `acerola_builder.rs`/`acerola_p2p.rs`.
async fn wait_for_mdns() {
    sleep(Duration::from_millis(1500)).await;
}

/// Endereço sintético de um nó que nunca existiu — chave pública válida (o handshake QUIC
/// exige uma), mas apontando pra uma porta local que nada escuta. Diferente de derrubar um
/// `AcerolaP2p` de verdade (o `Endpoint` do iroh não fecha o socket enquanto qualquer clone dele
/// — inclusive o da task de fundo `drive_incoming_connections` — continuar vivo, então "derrubar"
/// um nó não garante inalcançabilidade hoje), isso testa unreachability real e determinística.
fn unreachable_peer_addr() -> PeerAddr {
    let node_id = iroh::SecretKey::generate().public();
    let endpoint_addr = iroh::EndpointAddr::new(node_id)
        .with_ip_addr(std::net::SocketAddr::from(([127, 0, 0, 1], 1)));
    let addrs = serde_json::to_vec(&endpoint_addr).unwrap();
    PeerAddr { id: PeerIdentity::from_public_key(node_id.to_string(), node_id.as_bytes()), addrs }
}

#[tokio::test]
async fn blobs_capability_is_none_without_blobs_config() {
    let node =
        AcerolaP2p::builder(no_op_emitter(), IrohTransportBuilder::default(), test_device_info())
            .build()
            .await
            .unwrap();

    assert!(node.blobs().await.is_none());
}

#[tokio::test]
async fn blobs_capability_is_some_when_configured() {
    let node = build_node_with_blobs().await;
    assert!(node.blobs().await.is_some());
}

mod run_in_isolation {
    use tokio::io::AsyncReadExt;

    use super::*;

    #[tokio::test]
    async fn real_two_node_blob_round_trip_via_fetch() {
        let node_a = build_node_with_blobs().await;
        let node_b = build_node_with_blobs().await;

        let addr_a = node_a.local_addr().unwrap();
        let payload = b"acerola blob round trip payload".to_vec();

        let blobs_a = node_a.blobs().await.unwrap();
        let hash = blobs_a.put(payload.clone()).await.unwrap();

        wait_for_mdns().await;

        let blobs_b = node_b.blobs().await.unwrap();
        blobs_b.fetch(&hash, &addr_a).await.unwrap();

        let mut reader = blobs_b.get(&hash).await.unwrap();
        let mut received = Vec::new();
        reader.read_to_end(&mut received).await.unwrap();

        assert_eq!(received, payload);
        assert_eq!(blake3::hash(&received), blake3::hash(&payload));
    }

    #[tokio::test]
    async fn concurrent_puts_and_gets_of_multiple_blobs() {
        let node = build_node_with_blobs().await;
        let blobs = node.blobs().await.unwrap();

        const BLOB_COUNT: usize = 50;
        let mut handles = Vec::with_capacity(BLOB_COUNT);

        for index in 0..BLOB_COUNT {
            let blobs = Arc::clone(&blobs);
            handles.push(tokio::spawn(async move {
                let payload = format!("blob-payload-{index}").into_bytes();
                let hash = blobs.put(payload.clone()).await.unwrap();

                let mut reader = blobs.get(&hash).await.unwrap();
                let mut received = Vec::new();
                reader.read_to_end(&mut received).await.unwrap();

                assert_eq!(received, payload);
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn large_blob_throughput_and_integrity() {
        let node_a = build_node_with_blobs().await;
        let node_b = build_node_with_blobs().await;

        let addr_a = node_a.local_addr().unwrap();

        const PAYLOAD_SIZE: usize = 16 * 1024 * 1024; // 16 MiB
        let payload: Vec<u8> = (0..PAYLOAD_SIZE).map(|i| (i % 251) as u8).collect();
        let expected_hash = blake3::hash(&payload);

        let blobs_a = node_a.blobs().await.unwrap();
        let hash = blobs_a.put(payload).await.unwrap();

        wait_for_mdns().await;

        let blobs_b = node_b.blobs().await.unwrap();

        let start = std::time::Instant::now();
        blobs_b.fetch(&hash, &addr_a).await.unwrap();
        let elapsed = start.elapsed();

        let mut reader = blobs_b.get(&hash).await.unwrap();
        let mut received = Vec::new();
        reader.read_to_end(&mut received).await.unwrap();

        assert_eq!(blake3::hash(&received), expected_hash);
        tracing::info!(
            bytes = PAYLOAD_SIZE,
            elapsed_ms = elapsed.as_millis(),
            "large blob transfer throughput"
        );
    }

    /// Limite de segurança pra nenhum teste de caminho triste travar o suite pra sempre caso a
    /// premissa de "falha rápido, sem relay" pare de valer numa versão futura do iroh.
    const SAD_PATH_TIMEOUT: Duration = Duration::from_secs(10);

    /// Regressão do bug reportado/reproduzido ao vivo em 22-23/08/2026: `IrohBlobStore::fetch()`
    /// baixava o blob no store local sem criar nenhuma tag de proteção (diferente de `put()`,
    /// que sai de `AddProgress::with_tag()` já com uma tag permanente) — deixando uma janela
    /// real entre o `fetch()` retornar e o `get()` seguinte do chamador onde a varredura
    /// periódica de GC do store (`gc_interval`) podia reciclar o blob recém-baixado antes de
    /// qualquer um lê-lo, resultando em `BlobNotFound` esporádico. Usa um `gc_interval` bem
    /// curto (bem menor que o default de produção) pra tornar essa janela determinística sem
    /// deixar o teste lento: sem a tag, esperar mais que o intervalo antes do `get()` reproduz a
    /// falha de forma confiável; com a tag, o blob sobrevive indefinidamente.
    #[tokio::test]
    async fn fetched_blob_survives_a_gc_sweep_before_being_read() {
        use crate::api::blobs::IrohBlobsConfig;

        let gc_interval = Duration::from_millis(200);

        let node_a = AcerolaP2p::builder(
            no_op_emitter(),
            IrohTransportBuilder::default().blobs(IrohBlobsConfig::Mem { gc_interval }),
            test_device_info(),
        )
        .build()
        .await
        .unwrap();
        let node_b = AcerolaP2p::builder(
            no_op_emitter(),
            IrohTransportBuilder::default().blobs(IrohBlobsConfig::Mem { gc_interval }),
            test_device_info(),
        )
        .build()
        .await
        .unwrap();

        let addr_a = node_a.local_addr().unwrap();
        let payload = b"acerola gc survival payload".to_vec();

        let blobs_a = node_a.blobs().await.unwrap();
        let hash = blobs_a.put(payload.clone()).await.unwrap();

        wait_for_mdns().await;

        let blobs_b = node_b.blobs().await.unwrap();
        blobs_b.fetch(&hash, &addr_a).await.unwrap();

        // Deixa pelo menos uma varredura de GC completa acontecer antes de ler o blob de volta —
        // sem a tag de proteção, essa espera por si só já é o suficiente pra reproduzir o bug.
        sleep(gc_interval * 3).await;

        let mut reader = blobs_b
            .get(&hash)
            .await
            .expect("fetched blob must survive a GC sweep until explicitly removed");
        let mut received = Vec::new();
        reader.read_to_end(&mut received).await.unwrap();

        assert_eq!(received, payload);
    }

    /// Mesma classe de bug de `fetched_blob_survives_a_gc_sweep_before_being_read`, mas
    /// testando a janela mais estreita e mais perigosa: uma varredura de GC acontecendo
    /// DURANTE o download em si, não só depois dele terminar. Reproduzido ao vivo em
    /// 23/08/2026 sob carga real (dezenas de `browse-cover` disparados de uma vez): o fix
    /// anterior só criava a tag DEPOIS do `fetch()` retornar, então uma varredura que caísse
    /// durante o download (não depois) continuava sem nenhuma proteção — confirmado lendo
    /// `api::remote::execute_get_sink` do iroh-blobs (escreve os chunks direto no store sem
    /// usar `temp_tag()`) e o teste `gc_file_delete` da própria lib, que mostra um arquivo
    /// parcial sendo apagado por `gc_run_once` no meio do download. Payload grande + intervalo
    /// de GC minúsculo garante várias varreduras acontecendo enquanto os bytes ainda estão
    /// chegando, tornando essa janela determinística sem precisar de carga concorrente real.
    #[tokio::test]
    async fn fetch_survives_gc_sweeps_happening_mid_download() {
        use crate::api::blobs::IrohBlobsConfig;

        let gc_interval = Duration::from_millis(5);

        let node_a = AcerolaP2p::builder(
            no_op_emitter(),
            IrohTransportBuilder::default().blobs(IrohBlobsConfig::Mem { gc_interval }),
            test_device_info(),
        )
        .build()
        .await
        .unwrap();
        let node_b = AcerolaP2p::builder(
            no_op_emitter(),
            IrohTransportBuilder::default().blobs(IrohBlobsConfig::Mem { gc_interval }),
            test_device_info(),
        )
        .build()
        .await
        .unwrap();

        let addr_a = node_a.local_addr().unwrap();

        // Grande o bastante pra um download real cruzar várias varreduras de GC de 5ms
        // enquanto ainda está em andamento.
        const PAYLOAD_SIZE: usize = 32 * 1024 * 1024; // 32 MiB
        let payload: Vec<u8> = (0..PAYLOAD_SIZE).map(|i| (i % 251) as u8).collect();
        let expected_hash = blake3::hash(&payload);

        let blobs_a = node_a.blobs().await.unwrap();
        let hash = blobs_a.put(payload).await.unwrap();

        wait_for_mdns().await;

        let blobs_b = node_b.blobs().await.unwrap();
        blobs_b.fetch(&hash, &addr_a).await.expect("fetch must survive GC sweeps mid-download");

        let mut reader =
            blobs_b.get(&hash).await.expect("blob must be present immediately after fetch");
        let mut received = Vec::new();
        reader.read_to_end(&mut received).await.unwrap();

        assert_eq!(blake3::hash(&received), expected_hash);
    }

    /// Reproduz o cenário real que expôs o bug (o teste acima isola o mecanismo com um único
    /// fetch artificialmente lento) — não uma transferência lenta isolada, mas VÁRIAS
    /// transferências concorrentes pro mesmo peer competindo por banda/CPU, o padrão ao vivo
    /// de verdade (~20 buscas de capa disparadas de uma vez, `browse-cover` sem nenhum limite
    /// de concorrência). Cada fetch concorrente aumenta a chance de algum deles cruzar uma
    /// varredura de GC enquanto ainda em andamento; sem o `temp_tag`, isso derrubava uma
    /// fração deles com `BlobNotFound` de forma esporádica — exatamente o padrão observado
    /// (2 de ~22 falharam numa sessão real).
    #[tokio::test]
    async fn concurrent_fetches_from_the_same_peer_all_survive_gc_sweeps() {
        use crate::api::blobs::IrohBlobsConfig;

        let gc_interval = Duration::from_millis(20);

        let node_a = AcerolaP2p::builder(
            no_op_emitter(),
            IrohTransportBuilder::default().blobs(IrohBlobsConfig::Mem { gc_interval }),
            test_device_info(),
        )
        .build()
        .await
        .unwrap();
        let node_b = AcerolaP2p::builder(
            no_op_emitter(),
            IrohTransportBuilder::default().blobs(IrohBlobsConfig::Mem { gc_interval }),
            test_device_info(),
        )
        .build()
        .await
        .unwrap();

        let addr_a = node_a.local_addr().unwrap();

        // Próximo do burst real visto ao vivo (~20-22 `browse-cover` simultâneos pro mesmo peer).
        const CONCURRENT_FETCHES: usize = 24;
        const PAYLOAD_SIZE: usize = 2 * 1024 * 1024; // 2 MiB cada — tamanho realista de uma capa

        let blobs_a = node_a.blobs().await.unwrap();
        let mut payloads = Vec::with_capacity(CONCURRENT_FETCHES);
        let mut hashes = Vec::with_capacity(CONCURRENT_FETCHES);
        for index in 0..CONCURRENT_FETCHES {
            let payload: Vec<u8> =
                (0..PAYLOAD_SIZE).map(|byte| ((byte + index) % 251) as u8).collect();
            let hash = blobs_a.put(payload.clone()).await.unwrap();
            payloads.push(payload);
            hashes.push(hash);
        }

        wait_for_mdns().await;

        let blobs_b = node_b.blobs().await.unwrap();
        let mut handles = Vec::with_capacity(CONCURRENT_FETCHES);
        for hash in hashes {
            let blobs_b = Arc::clone(&blobs_b);
            let addr_a = addr_a.clone();
            handles.push(tokio::spawn(async move {
                blobs_b
                    .fetch(&hash, &addr_a)
                    .await
                    .expect("fetch must survive GC sweeps happening under concurrent load");
                let mut reader = blobs_b
                    .get(&hash)
                    .await
                    .expect("blob must be present immediately after a concurrent fetch");
                let mut received = Vec::new();
                reader.read_to_end(&mut received).await.unwrap();
                received
            }));
        }

        for (handle, expected) in handles.into_iter().zip(payloads) {
            let received = handle.await.unwrap();
            assert_eq!(received, expected);
        }
    }

    #[tokio::test]
    async fn fetch_fails_when_remote_peer_does_not_have_the_requested_hash() {
        let node_a = build_node_with_blobs().await;
        let node_b = build_node_with_blobs().await;

        let addr_a = node_a.local_addr().unwrap();

        // Hash que não corresponde a nenhum conteúdo real que node_a tenha armazenado —
        // simula pedir um blob errado a um peer que está online mas não o possui.
        let unknown_hash = BlobHash::from_bytes([0x7a; 32]);

        wait_for_mdns().await;

        let blobs_b = node_b.blobs().await.unwrap();
        let result =
            tokio::time::timeout(SAD_PATH_TIMEOUT, blobs_b.fetch(&unknown_hash, &addr_a)).await;

        assert!(result.is_ok(), "fetch must fail cleanly instead of hanging forever");
        assert!(
            result.unwrap().is_err(),
            "fetch must return an error when the remote peer doesn't have the hash"
        );
        assert!(
            !blobs_b.has(&unknown_hash).await.unwrap(),
            "a failed fetch must not leave a phantom/partial entry in the local store"
        );
    }

    #[tokio::test]
    async fn fetch_fails_cleanly_when_peer_is_unreachable_then_succeeds_against_a_real_peer() {
        let node_b = build_node_with_blobs().await;
        let payload = b"acerola resilience payload".to_vec();

        let blobs_b = node_b.blobs().await.unwrap();
        let dead_addr = unreachable_peer_addr();

        // Hash arbitrário — o peer nem existe, então não há blob real associado a ele; o que
        // importa aqui é só a inalcançabilidade do endereço, não o conteúdo.
        let target_hash = BlobHash::from_bytes(*blake3::hash(&payload).as_bytes());

        // Medido empiricamente: um peer genuinamente inalcançável (sem resposta nenhuma, ao
        // contrário de "online mas sem o blob") só desiste após o timeout padrão de handshake
        // QUIC do iroh, ~30s — bem maior que o SAD_PATH_TIMEOUT usado nos outros casos, que
        // assumem um peer que responde ativamente.
        let dead_peer_result =
            tokio::time::timeout(Duration::from_secs(40), blobs_b.fetch(&target_hash, &dead_addr))
                .await;

        assert!(
            dead_peer_result.is_ok(),
            "fetch against an unreachable peer must not hang forever"
        );
        assert!(
            dead_peer_result.unwrap().is_err(),
            "fetch must return an error when the peer is unreachable"
        );
        assert!(
            !blobs_b.has(&target_hash).await.unwrap(),
            "a failed fetch against a dead peer must not leave a phantom entry in the local store"
        );

        // Depois da falha, uma tentativa contra um peer de verdade, online, com o mesmo conteúdo
        // (mesmo hash, content-addressed) — confirma que a falha anterior não deixou nenhum
        // estado corrompido em blobs_b que impeça uma recuperação normal.
        let node_c = build_node_with_blobs().await;
        let blobs_c = node_c.blobs().await.unwrap();
        let hash_c = blobs_c.put(payload.clone()).await.unwrap();
        assert_eq!(target_hash, hash_c, "same content must be content-addressed to the same hash");

        wait_for_mdns().await;

        blobs_b.fetch(&target_hash, &node_c.local_addr().unwrap()).await.unwrap();

        let mut reader = blobs_b.get(&target_hash).await.unwrap();
        let mut received = Vec::new();
        reader.read_to_end(&mut received).await.unwrap();
        assert_eq!(received, payload);
    }
}
