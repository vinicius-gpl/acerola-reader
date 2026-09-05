//! Testes de estresse e validação do transporte (Mock e Iroh).
//!
//! Garante a estabilidade da camada de transporte sob alta concorrência,
//! cargas pesadas e validação estrita de integridade de dados usando Blake3.

use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

use async_trait::async_trait;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    sync::Mutex,
    time::{sleep, Duration},
};

use crate::{
    api::identity::DeviceInfo,
    core::{guard::open::OpenGuard, network::manager::NetworkManager},
    data::protocol::{EventEmitter, ProtocolHandler},
    infra::{error::ConnectionError, peer::PeerId},
    tests::mock_transport::mock_transport,
};
#[cfg(feature = "iroh")]
use crate::{api::AcerolaP2p, core::transport::iroh::IrohTransportBuilder};

#[allow(dead_code)]
fn no_op_emitter() -> EventEmitter {
    Arc::new(|_: &str, _: String| {})
}

#[allow(dead_code)]
fn create_device(device_name: &str) -> DeviceInfo {
    DeviceInfo {
        name: device_name.to_string(),
        os: "linux".to_string(),
        version: "1.0.0".to_string(),
    }
}

/// Manipulador de protocolo genérico para envio de dados com encerramento gracioso de stream.
#[allow(dead_code)]
struct BulkSenderHandler {
    payload: Vec<u8>,
}

#[async_trait]
impl ProtocolHandler for BulkSenderHandler {
    async fn handle(
        &self, _peer: &PeerId, mut send: Box<dyn AsyncWrite + Send + Unpin>,
        mut recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), ConnectionError> {
        send.write_all(&self.payload)
            .await
            .map_err(|err| ConnectionError::StreamFailed(err.to_string()))?;
        send.shutdown().await.map_err(|err| ConnectionError::StreamFailed(err.to_string()))?;

        // Aguarda o sinal de fechamento remoto para garantir o esvaziamento completo da stream.
        let mut drain_sink = Vec::new();
        let _ = recv.read_to_end(&mut drain_sink).await;
        Ok(())
    }
}

/// Manipulador de protocolo genérico para recebimento de dados e armazenamento do buffer consumido.
#[allow(dead_code)]
struct BulkReceiverHandler {
    received_buffer: Arc<Mutex<Vec<u8>>>,
    completed_counter: Arc<AtomicUsize>,
}

#[async_trait]
impl ProtocolHandler for BulkReceiverHandler {
    async fn handle(
        &self, _peer: &PeerId, mut send: Box<dyn AsyncWrite + Send + Unpin>,
        mut recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), ConnectionError> {
        let mut buffer = Vec::new();
        recv.read_to_end(&mut buffer)
            .await
            .map_err(|err| ConnectionError::StreamFailed(err.to_string()))?;

        let mut locked_buffer = self.received_buffer.lock().await;
        locked_buffer.extend_from_slice(&buffer);
        self.completed_counter.fetch_add(1, Ordering::SeqCst);

        let _ = send.shutdown().await;
        Ok(())
    }
}

// ============================================================================
// Testes do MockTransport (Validação sem driver físico de rede)
// ============================================================================

#[tokio::test]
async fn mock_transport_high_concurrency_stress() {
    crate::tests::init_tracing();

    let (transport, handle) = mock_transport();
    let (mut manager, _command_tx, _state) =
        NetworkManager::new(Arc::new(transport), OpenGuard::into_validator(), no_op_emitter());

    let completed_counter = Arc::new(AtomicUsize::new(0));
    let buffer = Arc::new(Mutex::new(Vec::new()));

    let handler = Arc::new(BulkReceiverHandler {
        received_buffer: Arc::clone(&buffer),
        completed_counter: Arc::clone(&completed_counter),
    });

    manager.register_inbound(b"test/stress", handler);
    tokio::spawn(manager.run());

    let concurrent_connections = 50;
    tracing::info!(
        count = concurrent_connections,
        "dispatching concurrent connections on mock transport"
    );

    for connection_index in 0..concurrent_connections {
        let peer = PeerId { id: format!("mock-peer-{}", connection_index), device_id: None };
        let (mut client_tx, client_rx) = tokio::io::duplex(2048);
        let (server_tx, _server_rx) = tokio::io::duplex(2048);

        handle.inject(b"test/stress", peer, client_rx, server_tx);

        tokio::spawn(async move {
            let _ = client_tx.write_all(b"stress-payload").await;
            let _ = client_tx.shutdown().await;
        });
    }

    sleep(Duration::from_millis(300)).await;

    let processed_count = completed_counter.load(Ordering::SeqCst);
    tracing::info!(
        processed = processed_count,
        "mock transport stress connections successfully processed"
    );
    assert_eq!(processed_count, concurrent_connections, "All mock connections should be served");
}

#[tokio::test]
async fn mock_transport_blake3_data_integrity_validation() {
    crate::tests::init_tracing();

    let (transport, handle) = mock_transport();
    let (mut manager, _command_tx, _state) =
        NetworkManager::new(Arc::new(transport), OpenGuard::into_validator(), no_op_emitter());

    let received_buffer = Arc::new(Mutex::new(Vec::new()));
    let completed_counter = Arc::new(AtomicUsize::new(0));

    let handler = Arc::new(BulkReceiverHandler {
        received_buffer: Arc::clone(&received_buffer),
        completed_counter: Arc::clone(&completed_counter),
    });

    manager.register_inbound(b"test/blake3", handler);
    tokio::spawn(manager.run());

    // Carga útil reproduzível de 100KB
    let payload: Vec<u8> = (0..102_400).map(|byte_index| (byte_index % 256) as u8).collect();
    let expected_hash = blake3::hash(&payload);

    let (mut client_tx, client_rx) = tokio::io::duplex(65_536);
    let (server_tx, _server_rx) = tokio::io::duplex(65_536);

    let peer = PeerId { id: "mock-blake3-peer".to_string(), device_id: None };

    handle.inject(b"test/blake3", peer, client_rx, server_tx);

    tokio::spawn(async move {
        let _ = client_tx.write_all(&payload).await;
        let _ = client_tx.shutdown().await;
    });

    sleep(Duration::from_millis(200)).await;

    let received = received_buffer.lock().await;
    let actual_hash = blake3::hash(&received);

    tracing::info!(
        hash = %actual_hash.to_hex(),
        "data integrity verified via blake3 hash"
    );

    assert_eq!(actual_hash, expected_hash, "Blake3 hash of received data must match expected hash");
}

// ============================================================================
// Testes de estresse com Iroh Transport (Driver QUIC real)
// ============================================================================

#[cfg(feature = "iroh")]
#[tokio::test]
async fn iroh_transport_throughput_and_large_payload_integrity() {
    crate::tests::init_tracing();

    let payload_size = 2 * 1024 * 1024; // 2MB
    let payload: Vec<u8> = (0..payload_size).map(|byte_index| (byte_index % 251) as u8).collect();
    let expected_hash = blake3::hash(&payload);

    let received_buffer = Arc::new(Mutex::new(Vec::new()));
    let completed_counter = Arc::new(AtomicUsize::new(0));

    let node_a = AcerolaP2p::builder(
        no_op_emitter(),
        IrohTransportBuilder::default(),
        create_device("node-a-sender"),
    )
    .outbound(b"test/throughput", Arc::new(BulkSenderHandler { payload: payload.clone() }))
    .build()
    .await
    .unwrap();

    let node_b = AcerolaP2p::builder(
        no_op_emitter(),
        IrohTransportBuilder::default(),
        create_device("node-b-receiver"),
    )
    .inbound(
        b"test/throughput",
        Arc::new(BulkReceiverHandler {
            received_buffer: Arc::clone(&received_buffer),
            completed_counter: Arc::clone(&completed_counter),
        }),
    )
    .build()
    .await
    .unwrap();

    let start_time = Instant::now();

    node_a.connect(node_b.local_addr().unwrap(), b"test/throughput").await.unwrap();

    // Aguarda a conclusão da transferência
    let elapsed_secs = loop {
        if completed_counter.load(Ordering::SeqCst) > 0 {
            break start_time.elapsed().as_secs_f64();
        }
        if start_time.elapsed() > Duration::from_secs(10) {
            panic!("Timeout waiting for 2MB transfer on Iroh transport");
        }
        sleep(Duration::from_millis(50)).await;
    };

    let received = received_buffer.lock().await;
    let actual_hash = blake3::hash(&received);

    let throughput_mbps = (payload_size as f64 / (1024.0 * 1024.0)) / elapsed_secs;

    tracing::info!(
        elapsed_seconds = elapsed_secs,
        throughput_mb_s = throughput_mbps,
        "2MB payload transfer completed"
    );
    tracing::info!(
        hash = %actual_hash.to_hex(),
        "blake3 integrity validation"
    );

    assert_eq!(received.len(), payload_size, "Received data size must match payload size");
    assert_eq!(
        actual_hash, expected_hash,
        "Blake3 hash mismatch! Data corrupted during transport."
    );

    node_a.shutdown().await.unwrap();
    node_b.shutdown().await.unwrap();
}

#[cfg(feature = "iroh")]
#[tokio::test]
async fn iroh_transport_multiple_concurrent_streams_stress() {
    crate::tests::init_tracing();

    let stream_count = 10;
    let payload_per_stream = 64 * 1024; // 64KB por stream
    let payload: Vec<u8> = vec![0x37u8; payload_per_stream];

    let received_buffer = Arc::new(Mutex::new(Vec::new()));
    let completed_counter = Arc::new(AtomicUsize::new(0));

    let node_a = AcerolaP2p::builder(
        no_op_emitter(),
        IrohTransportBuilder::default(),
        create_device("node-a-multi"),
    )
    .outbound(b"test/multi-stream", Arc::new(BulkSenderHandler { payload: payload.clone() }))
    .build()
    .await
    .unwrap();

    let node_b = AcerolaP2p::builder(
        no_op_emitter(),
        IrohTransportBuilder::default(),
        create_device("node-b-multi"),
    )
    .inbound(
        b"test/multi-stream",
        Arc::new(BulkReceiverHandler {
            received_buffer: Arc::clone(&received_buffer),
            completed_counter: Arc::clone(&completed_counter),
        }),
    )
    .build()
    .await
    .unwrap();

    tracing::info!(streams = stream_count, "initiating concurrent dispatch of streams");

    for _ in 0..stream_count {
        node_a.connect(node_b.local_addr().unwrap(), b"test/multi-stream").await.unwrap();
    }

    let start_time = Instant::now();
    while completed_counter.load(Ordering::SeqCst) < stream_count {
        if start_time.elapsed() > Duration::from_secs(10) {
            panic!(
                "Timeout: only {} of {} streams were completed",
                completed_counter.load(Ordering::SeqCst),
                stream_count
            );
        }
        sleep(Duration::from_millis(50)).await;
    }

    let total_received = received_buffer.lock().await.len();
    let expected_total = stream_count * payload_per_stream;

    tracing::info!(
        received_streams = stream_count,
        total_bytes = total_received,
        "concurrent streams processed successfully"
    );

    assert_eq!(total_received, expected_total, "Total accumulated bytes must match expected total");

    node_a.shutdown().await.unwrap();
    node_b.shutdown().await.unwrap();
}
