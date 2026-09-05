//! Testes de estresse e validação da camada de transporte (Etapa 3.2)
//!
//! Valida a integridade dos dados, throughput básico e estabilidade
//! sob condições de estresse para os transportes Iroh e Mock.

use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

#[cfg(feature = "iroh")]
use acerola_p2p::api::transport::IrohTransportBuilder;
use acerola_p2p::api::{
    error::P2pError,
    identity::DeviceInfo,
    peer::PeerIdentity,
    protocol::{EventEmitter, Handler},
    AcerolaP2p,
};
use async_trait::async_trait;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    sync::Mutex,
    time::{sleep, Duration},
};

fn no_op_emitter() -> EventEmitter {
    Arc::new(|_: &str, _: String| {})
}

fn create_device_info(device_name: &str) -> DeviceInfo {
    DeviceInfo {
        name: device_name.to_string(),
        os: "linux".to_string(),
        version: "1.0.0".to_string(),
    }
}

struct StreamSender {
    payload: Vec<u8>,
}

#[async_trait]
impl Handler for StreamSender {
    async fn handle(
        &self, _peer: &PeerIdentity, mut send: Box<dyn AsyncWrite + Send + Unpin>,
        mut recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        send.write_all(&self.payload)
            .await
            .map_err(|err| P2pError::StreamFailed(err.to_string()))?;
        send.shutdown().await.map_err(|err| P2pError::StreamFailed(err.to_string()))?;

        let mut drain_sink = Vec::new();
        let _ = recv.read_to_end(&mut drain_sink).await;
        Ok(())
    }
}

struct StreamReceiver {
    buffer: Arc<Mutex<Vec<u8>>>,
    counter: Arc<AtomicUsize>,
}

#[async_trait]
impl Handler for StreamReceiver {
    async fn handle(
        &self, _peer: &PeerIdentity, mut send: Box<dyn AsyncWrite + Send + Unpin>,
        mut recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        let mut buffer = Vec::new();
        recv.read_to_end(&mut buffer)
            .await
            .map_err(|err| P2pError::StreamFailed(err.to_string()))?;

        self.buffer.lock().await.extend_from_slice(&buffer);
        self.counter.fetch_add(1, Ordering::SeqCst);
        let _ = send.shutdown().await;
        Ok(())
    }
}

#[cfg(feature = "iroh")]
#[tokio::test]
async fn validate_iroh_data_integrity_and_throughput() {
    let payload_size = 1024 * 1024; // 1MB
    let payload: Vec<u8> = (0..payload_size).map(|byte_index| (byte_index % 256) as u8).collect();
    let expected_hash = blake3::hash(&payload);

    let buffer = Arc::new(Mutex::new(Vec::new()));
    let counter = Arc::new(AtomicUsize::new(0));

    let node_a = AcerolaP2p::builder(
        no_op_emitter(),
        IrohTransportBuilder::default(),
        create_device_info("stress-node-a"),
    )
    .outbound(b"validation/bulk", Arc::new(StreamSender { payload: payload.clone() }))
    .build()
    .await
    .expect("Failed to build node A");

    let node_b = AcerolaP2p::builder(
        no_op_emitter(),
        IrohTransportBuilder::default(),
        create_device_info("stress-node-b"),
    )
    .inbound(
        b"validation/bulk",
        Arc::new(StreamReceiver { buffer: Arc::clone(&buffer), counter: Arc::clone(&counter) }),
    )
    .build()
    .await
    .expect("Failed to build node B");

    let start_time = Instant::now();

    node_a
        .connect(node_b.local_addr().unwrap(), b"validation/bulk")
        .await
        .expect("Failed to connect on validation ALPN");

    while counter.load(Ordering::SeqCst) == 0 {
        if start_time.elapsed() > Duration::from_secs(10) {
            panic!("Timeout waiting for validation transfer on Iroh transport");
        }
        sleep(Duration::from_millis(50)).await;
    }

    let elapsed_secs = start_time.elapsed().as_secs_f64();
    let throughput_mbps = (payload_size as f64 / (1024.0 * 1024.0)) / elapsed_secs;

    let received = buffer.lock().await.clone();
    let actual_hash = blake3::hash(&received);

    tracing::info!(
        elapsed_seconds = elapsed_secs,
        throughput_mb_s = throughput_mbps,
        "1MB validation transfer completed"
    );
    tracing::info!(
        hash = %actual_hash.to_hex(),
        "blake3 checksum validation"
    );

    assert_eq!(
        received.len(),
        payload_size,
        "Transferred data size differs from expected payload size"
    );
    assert_eq!(actual_hash, expected_hash, "Blake3 checksum mismatch - data corrupted");

    node_a.shutdown().await.unwrap();
    node_b.shutdown().await.unwrap();
}

#[cfg(feature = "iroh")]
#[tokio::test]
async fn validate_iroh_concurrent_transport_stress() {
    let stream_count = 5;
    let payload = vec![0x99u8; 32 * 1024]; // 32KB por stream

    let buffer = Arc::new(Mutex::new(Vec::new()));
    let counter = Arc::new(AtomicUsize::new(0));

    let node_a = AcerolaP2p::builder(
        no_op_emitter(),
        IrohTransportBuilder::default(),
        create_device_info("multi-node-a"),
    )
    .outbound(b"validation/multi", Arc::new(StreamSender { payload: payload.clone() }))
    .build()
    .await
    .unwrap();

    let node_b = AcerolaP2p::builder(
        no_op_emitter(),
        IrohTransportBuilder::default(),
        create_device_info("multi-node-b"),
    )
    .inbound(
        b"validation/multi",
        Arc::new(StreamReceiver { buffer: Arc::clone(&buffer), counter: Arc::clone(&counter) }),
    )
    .build()
    .await
    .unwrap();

    tracing::info!(requests = stream_count, "dispatching concurrent requests");

    for _ in 0..stream_count {
        node_a.connect(node_b.local_addr().unwrap(), b"validation/multi").await.unwrap();
    }

    let start_time = Instant::now();
    while counter.load(Ordering::SeqCst) < stream_count {
        if start_time.elapsed() > Duration::from_secs(10) {
            panic!("Timeout during concurrent connection stress test");
        }
        sleep(Duration::from_millis(50)).await;
    }

    let processed_count = counter.load(Ordering::SeqCst);
    tracing::info!(
        processed_streams = processed_count,
        "concurrent streams processed successfully"
    );

    assert_eq!(processed_count, stream_count, "All concurrent streams must be completed");

    node_a.shutdown().await.unwrap();
    node_b.shutdown().await.unwrap();
}
