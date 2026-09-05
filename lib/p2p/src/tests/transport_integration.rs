#[cfg(all(test, feature = "iroh"))]
mod run_in_isolation {
    use std::sync::Arc;

    use async_trait::async_trait;
    use tokio::{
        io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
        sync::Mutex,
        time::{sleep, Duration},
    };

    use crate::{
        api::{identity::DeviceInfo, AcerolaP2p},
        core::transport::iroh::IrohTransportBuilder,
        data::protocol::{EventEmitter, ProtocolHandler},
        infra::{error::ConnectionError, peer::PeerId},
    };

    fn emitter() -> EventEmitter {
        Arc::new(|_: &str, _: String| {})
    }

    fn device(name: &str) -> DeviceInfo {
        DeviceInfo { name: name.into(), os: "linux".into(), version: "0.0.1".into() }
    }

    async fn build_node(name: &str) -> AcerolaP2p {
        AcerolaP2p::builder(emitter(), IrohTransportBuilder::default(), device(name))
            .build()
            .await
            .unwrap()
    }

    struct SenderHandler {
        payload: Vec<u8>,
    }

    #[async_trait]
    impl ProtocolHandler for SenderHandler {
        async fn handle(
            &self, _peer: &PeerId, mut send: Box<dyn AsyncWrite + Send + Unpin>,
            mut recv: Box<dyn AsyncRead + Send + Unpin>,
        ) -> Result<(), ConnectionError> {
            send.write_all(&self.payload)
                .await
                .map_err(|e| ConnectionError::StreamFailed(e.to_string()))?;
            send.shutdown().await.map_err(|e| ConnectionError::StreamFailed(e.to_string()))?;
            let mut dummy = Vec::new();
            let _ = recv.read_to_end(&mut dummy).await;
            Ok(())
        }
    }

    struct ReceiverHandler {
        received: Arc<Mutex<Vec<u8>>>,
    }

    #[async_trait]
    impl ProtocolHandler for ReceiverHandler {
        async fn handle(
            &self, _peer: &PeerId, mut send: Box<dyn AsyncWrite + Send + Unpin>,
            mut recv: Box<dyn AsyncRead + Send + Unpin>,
        ) -> Result<(), ConnectionError> {
            let mut buf = Vec::new();
            recv.read_to_end(&mut buf)
                .await
                .map_err(|e| ConnectionError::StreamFailed(e.to_string()))?;
            *self.received.lock().await = buf;
            let _ = send.shutdown().await;
            Ok(())
        }
    }

    #[tokio::test]
    async fn two_nodes_exchange_data_on_custom_alpn() {
        crate::tests::init_tracing();
        let received = Arc::new(Mutex::new(Vec::new()));
        let payload = b"hello acerola".to_vec();

        let node_a: AcerolaP2p =
            AcerolaP2p::builder(emitter(), IrohTransportBuilder::default(), device("a"))
                .outbound(b"test/echo", Arc::new(SenderHandler { payload: payload.clone() }))
                .build()
                .await
                .unwrap();

        let node_b: AcerolaP2p =
            AcerolaP2p::builder(emitter(), IrohTransportBuilder::default(), device("b"))
                .inbound(
                    b"test/echo",
                    Arc::new(ReceiverHandler { received: Arc::clone(&received) }),
                )
                .build()
                .await
                .unwrap();

        tracing::debug!(node_a_id = %node_a.local_id(), node_b_id = %node_b.local_id(), "initiating test connection");
        let connect_result = node_a.connect(node_b.local_addr().unwrap(), b"test/echo").await;
        tracing::debug!(result = ?connect_result, "connect completed");
        sleep(Duration::from_millis(1500)).await;

        let received_data = received.lock().await.clone();
        tracing::debug!(len = received_data.len(), "data received");
        assert_eq!(received_data, payload);
    }

    #[tokio::test]
    async fn data_arrives_intact_with_large_payload() {
        let received = Arc::new(Mutex::new(Vec::new()));
        let payload = vec![0xABu8; 64 * 1024];

        let node_a: AcerolaP2p =
            AcerolaP2p::builder(emitter(), IrohTransportBuilder::default(), device("a"))
                .outbound(b"test/bulk", Arc::new(SenderHandler { payload: payload.clone() }))
                .build()
                .await
                .unwrap();

        let node_b: AcerolaP2p =
            AcerolaP2p::builder(emitter(), IrohTransportBuilder::default(), device("b"))
                .inbound(
                    b"test/bulk",
                    Arc::new(ReceiverHandler { received: Arc::clone(&received) }),
                )
                .build()
                .await
                .unwrap();

        node_a.connect(node_b.local_addr().unwrap(), b"test/bulk").await.unwrap();
        sleep(Duration::from_millis(2000)).await;

        assert_eq!(*received.lock().await, payload);
    }

    #[tokio::test]
    async fn peer_appears_in_node_b_known_peers_after_real_connection() {
        // O handshake (`acerola/handshake/1`) é uma troca pontual — o handler retorna assim que
        // termina, então `connected_peers()` (sessões de protocolo ativas) já está vazio de novo
        // logo em seguida. `known_peers()` é quem deve continuar mostrando o peer: é o cache de
        // "quem eu já vi", sobrevivendo ao fim da sessão de handshake em si.
        let node_a = build_node("a").await;
        let node_b = build_node("b").await;

        let id_a = node_a.local_id().to_string();
        node_a.connect(node_b.local_addr().unwrap(), b"acerola/handshake/1").await.unwrap();
        sleep(Duration::from_millis(500)).await;

        let known = node_b.known_peers().await;
        assert!(
            known.iter().any(|(p, _, _)| p.id == id_a),
            "node A did not appear in node B's known peers"
        );
    }

    #[tokio::test]
    async fn alpn_without_handler_on_receiver_does_not_break_sender() {
        let node_a: AcerolaP2p =
            AcerolaP2p::builder(emitter(), IrohTransportBuilder::default(), device("a"))
                .outbound(b"test/ghost", Arc::new(SenderHandler { payload: b"ignored".to_vec() }))
                .build()
                .await
                .unwrap();

        let node_b = build_node("b").await;

        let result: Result<(), ConnectionError> =
            node_a.connect(node_b.local_addr().unwrap(), b"test/ghost").await;
        sleep(Duration::from_millis(300)).await;

        assert!(result.is_ok(), "connect should not fail on sender");
        assert!(
            node_b.connected_peers().await.is_empty(),
            "node B should not have connected peers"
        );
    }

    #[tokio::test]
    async fn real_transfer_completes_and_status_reflects_it_right_after() {
        crate::tests::init_tracing();
        let received = Arc::new(Mutex::new(Vec::new()));
        let payload = b"acerola sync payload".to_vec();

        let node_a: AcerolaP2p =
            AcerolaP2p::builder(emitter(), IrohTransportBuilder::default(), device("a"))
                .outbound(
                    b"test/status-check",
                    Arc::new(SenderHandler { payload: payload.clone() }),
                )
                .build()
                .await
                .unwrap();

        let node_b: AcerolaP2p =
            AcerolaP2p::builder(emitter(), IrohTransportBuilder::default(), device("b"))
                .inbound(
                    b"test/status-check",
                    Arc::new(ReceiverHandler { received: Arc::clone(&received) }),
                )
                .build()
                .await
                .unwrap();

        let id_a = node_a.local_id().to_string();

        node_a.connect(node_b.local_addr().unwrap(), b"test/status-check").await.unwrap();

        // Espera de verdade a transferência acontecer (poll no buffer recebido, com timeout) —
        // um sleep fixo arbitrário mascararia uma transferência que trava no meio.
        let start = std::time::Instant::now();
        loop {
            if *received.lock().await == payload {
                break;
            }
            assert!(start.elapsed() < Duration::from_secs(5), "transfer did not complete in time");
            sleep(Duration::from_millis(20)).await;
        }

        // Checa o status LOGO depois da transferência confirmada. A sessão desse ALPN é
        // one-shot por design (o manager derruba a conexão assim que o handler termina —
        // ver `NetworkManager::handle_connect_command`), então o peer NÃO deve continuar
        // "conectado" pra sempre depois disso — se aparecer, é vazamento de estado.
        let peers_on_b = node_b.connected_peers().await;
        assert!(
            !peers_on_b.keys().any(|p| p.id == id_a),
            "peer should not remain connected after a one-shot ALPN session completes"
        );

        let _ = node_a.shutdown().await;
        let _ = node_b.shutdown().await;
    }

    #[tokio::test]
    async fn simultaneous_bidirectional_connect_same_alpn_does_not_silently_drop_either_side() {
        crate::tests::init_tracing();
        let received_a = Arc::new(Mutex::new(Vec::new()));
        let received_b = Arc::new(Mutex::new(Vec::new()));

        // Os dois nós registram handler outbound E inbound pro MESMO alpn, pra poder discar
        // um pro outro ao mesmo tempo — simula dois devices dando "Sync History" quase
        // simultaneamente, o cenário mais plausível pra explicar uma conexão fechando sem
        // nenhuma troca visível no log (glare: os dois lados abrindo stream um pro outro
        // pro mesmo par (peer, alpn) na mesma janela de tempo).
        let node_a: AcerolaP2p =
            AcerolaP2p::builder(emitter(), IrohTransportBuilder::default(), device("a"))
                .outbound(b"test/glare", Arc::new(SenderHandler { payload: b"from-a".to_vec() }))
                .inbound(
                    b"test/glare",
                    Arc::new(ReceiverHandler { received: Arc::clone(&received_a) }),
                )
                .build()
                .await
                .unwrap();

        let node_b: AcerolaP2p =
            AcerolaP2p::builder(emitter(), IrohTransportBuilder::default(), device("b"))
                .outbound(b"test/glare", Arc::new(SenderHandler { payload: b"from-b".to_vec() }))
                .inbound(
                    b"test/glare",
                    Arc::new(ReceiverHandler { received: Arc::clone(&received_b) }),
                )
                .build()
                .await
                .unwrap();

        let addr_a = node_a.local_addr().unwrap();
        let addr_b = node_b.local_addr().unwrap();

        // Dispara os dois connect()s o mais simultâneo possível.
        let (result_a, result_b) = tokio::join!(
            node_a.connect(addr_b, b"test/glare"),
            node_b.connect(addr_a, b"test/glare"),
        );
        assert!(result_a.is_ok(), "node_a.connect() failed: {result_a:?}");
        assert!(result_b.is_ok(), "node_b.connect() failed: {result_b:?}");

        let start = std::time::Instant::now();
        loop {
            let a_done = *received_a.lock().await == b"from-b".to_vec();
            let b_done = *received_b.lock().await == b"from-a".to_vec();
            if a_done && b_done {
                break;
            }
            assert!(
                start.elapsed() < Duration::from_secs(5),
                "glare: at least one side did not receive data (a_done={a_done}, b_done={b_done})"
            );
            sleep(Duration::from_millis(20)).await;
        }

        let _ = node_a.shutdown().await;
        let _ = node_b.shutdown().await;
    }

    #[tokio::test]
    async fn real_iroh_nodes_latency_monitoring_integration() {
        crate::tests::init_tracing();

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let emitter_a: EventEmitter = Arc::new(move |event: &str, payload: String| {
            let _ = tx.send((event.to_string(), payload));
        });

        let node_a = AcerolaP2p::builder(emitter_a, IrohTransportBuilder::default(), device("a"))
            .build()
            .await
            .unwrap();

        let node_b = build_node("b").await;
        let id_b = node_b.local_id().to_string();

        node_a.connect(node_b.local_addr().unwrap(), b"acerola/handshake/1").await.unwrap();
        sleep(Duration::from_millis(1500)).await;

        // O handshake é pontual (ver comentário em `peer_appears_in_node_b_known_peers_...`) —
        // `known_peers()` é quem reflete "já vi esse peer", não `connected_peers()`.
        let known = node_a.known_peers().await;
        assert!(
            known.iter().any(|(p, _, _)| p.id == id_b),
            "node B did not appear in node A known peers"
        );

        let mut events = Vec::new();
        while let Ok(evt) = rx.try_recv() {
            events.push(evt);
        }

        // Verifica que eventos RPC do handshake foram emitidos na conexão real
        assert!(events.iter().any(|(ev, _)| ev == "rpc:ping_sent" || ev == "rpc:device_info_sent"));

        let _ = node_a.shutdown().await;
        let _ = node_b.shutdown().await;
    }
}
