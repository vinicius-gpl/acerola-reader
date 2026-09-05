//! Benchmarks formais de throughput para os transportes Mock e Iroh (Etapa G)
//!
//! Mede a taxa de conexões por segundo para o Mock transport e a taxa de transferência
//! em MB/s para o Iroh transport gerando relatórios HTML via Criterion.

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use acerola_p2p::{
    api::{
        error::P2pError,
        identity::DeviceInfo,
        peer::PeerIdentity,
        protocol::{EventEmitter, Handler},
        transport::IrohTransportBuilder,
        AcerolaP2p,
    },
    core::network::manager::{NetworkCommand, NetworkManager},
    tests::mock_transport::mock_transport,
};
use async_trait::async_trait;
use criterion::{criterion_group, criterion_main, Bencher, Criterion, Throughput};
use tokio::{
    io::{duplex, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    runtime::Runtime,
    sync::Mutex,
    time::{sleep, Duration, Instant},
};

fn no_op_emitter() -> EventEmitter {
    Arc::new(|_event_name: &str, _event_payload: String| {})
}

fn create_device_info(device_name: &str) -> DeviceInfo {
    DeviceInfo {
        name: device_name.to_string(),
        os: "linux".to_string(),
        version: "1.0.0".to_string(),
    }
}

struct BenchmarkStreamSender {
    payload: Vec<u8>,
}

#[async_trait]
impl Handler for BenchmarkStreamSender {
    async fn handle(
        &self, _peer_identity: &PeerIdentity,
        mut stream_writer: Box<dyn AsyncWrite + Send + Unpin>,
        mut stream_reader: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        stream_writer
            .write_all(&self.payload)
            .await
            .map_err(|error_val| P2pError::StreamFailed(error_val.to_string()))?;
        stream_writer
            .shutdown()
            .await
            .map_err(|error_val| P2pError::StreamFailed(error_val.to_string()))?;

        let mut drain_buffer = Vec::new();
        let _ = stream_reader.read_to_end(&mut drain_buffer).await;
        Ok(())
    }
}

struct BenchmarkStreamReceiver {
    buffer_sink: Arc<Mutex<Vec<u8>>>,
    transfer_counter: Arc<AtomicUsize>,
}

#[async_trait]
impl Handler for BenchmarkStreamReceiver {
    async fn handle(
        &self, _peer_identity: &PeerIdentity,
        mut stream_writer: Box<dyn AsyncWrite + Send + Unpin>,
        mut stream_reader: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        let mut received_bytes = Vec::new();
        stream_reader
            .read_to_end(&mut received_bytes)
            .await
            .map_err(|error_val| P2pError::StreamFailed(error_val.to_string()))?;

        self.buffer_sink.lock().await.extend_from_slice(&received_bytes);
        self.transfer_counter.fetch_add(1, Ordering::SeqCst);
        let _ = stream_writer.shutdown().await;
        Ok(())
    }
}

/// Benchmark de conexões/s usando o Mock transport.
///
/// Mede o número de conexões aceitas e processadas por segundo (baseline: >=100 conexões/s).
fn bench_mock_transport_throughput(criterion_instance: &mut Criterion) {
    let tokio_runtime = Runtime::new().expect("Failed to initialize Tokio Runtime");

    let mut benchmark_group = criterion_instance.benchmark_group("mock_transport_connection_rate");
    let batch_size = 100;
    benchmark_group.throughput(Throughput::Elements(batch_size as u64));

    benchmark_group.bench_function("connection_throughput", |bencher: &mut Bencher| {
        bencher.iter(|| {
            tokio_runtime.block_on(async {
                let (mock_transport_instance, mock_handle) = mock_transport();

                let (manager_instance, _command_sender, network_state) = NetworkManager::new(
                    Arc::new(mock_transport_instance),
                    Box::new(|_context| Box::pin(async { Ok(()) })),
                    no_op_emitter(),
                );

                let manager_task_handle = tokio::spawn(manager_instance.run());

                for connection_index in 0..batch_size {
                    let (client_stream, server_stream) = duplex(1024);
                    let target_peer = PeerIdentity {
                        id: format!("mock-peer-{}", connection_index),
                        device_id: None,
                    };

                    mock_handle.inject(
                        b"acerola/handshake/1",
                        target_peer,
                        client_stream,
                        server_stream,
                    );
                }

                sleep(Duration::from_millis(10)).await;

                let _connected_count = network_state.read().await.peers().len();
                let _ = _command_sender.send(NetworkCommand::Shutdown).await;
                let _ = manager_task_handle.await;
            });
        });
    });

    benchmark_group.finish();
}

/// Benchmark de throughput de dados (MB/s) usando o Iroh transport.
///
/// Mede a taxa de transferência de carga útil de 1 MB (baseline: >=10 MB/s).
fn bench_iroh_transport_throughput(criterion_instance: &mut Criterion) {
    let tokio_runtime = Runtime::new().expect("Failed to initialize Tokio Runtime");

    let payload_size_bytes = 1024 * 1024; // Carga útil de 1 MB
    let test_payload: Vec<u8> =
        (0..payload_size_bytes).map(|byte_index| (byte_index % 256) as u8).collect();

    let buffer_sink = Arc::new(Mutex::new(Vec::new()));
    let transfer_counter = Arc::new(AtomicUsize::new(0));

    let receiver_handler: Arc<dyn Handler> = Arc::new(BenchmarkStreamReceiver {
        buffer_sink: Arc::clone(&buffer_sink),
        transfer_counter: Arc::clone(&transfer_counter),
    });

    let sender_handler: Arc<dyn Handler> =
        Arc::new(BenchmarkStreamSender { payload: test_payload.clone() });

    // Inicializa o par de nós Iroh uma única vez na fase de setup do benchmark
    let (node_sender, node_receiver) = tokio_runtime.block_on(async {
        let node_sender = AcerolaP2p::builder(
            no_op_emitter(),
            IrohTransportBuilder::default(),
            create_device_info("bench-sender-node"),
        )
        .outbound(b"bench/throughput", sender_handler)
        .build()
        .await
        .expect("Failed to create sender node in setup");

        let node_receiver = AcerolaP2p::builder(
            no_op_emitter(),
            IrohTransportBuilder::default(),
            create_device_info("bench-receiver-node"),
        )
        .inbound(b"bench/throughput", receiver_handler)
        .build()
        .await
        .expect("Failed to create receiver node in setup");

        // Aguarda mDNS descobrir o endereço do peer receptor antes da medição
        sleep(Duration::from_millis(1500)).await;
        (node_sender, node_receiver)
    });

    let mut benchmark_group = criterion_instance.benchmark_group("iroh_transport_data_throughput");
    benchmark_group.throughput(Throughput::Bytes(payload_size_bytes as u64));

    let target_addr = node_receiver.local_addr().unwrap();

    benchmark_group.bench_function("data_transfer_1mb", |bencher: &mut Bencher| {
        bencher.iter(|| {
            tokio_runtime.block_on(async {
                let initial_counter = transfer_counter.load(Ordering::SeqCst);
                if node_sender.connect(target_addr.clone(), b"bench/throughput").await.is_ok() {
                    let start_wait = Instant::now();
                    while transfer_counter.load(Ordering::SeqCst) == initial_counter {
                        if start_wait.elapsed() > Duration::from_secs(5) {
                            break;
                        }
                        sleep(Duration::from_millis(1)).await;
                    }
                }
            });
        });
    });

    tokio_runtime.block_on(async {
        let _ = node_sender.shutdown().await;
        let _ = node_receiver.shutdown().await;
    });

    benchmark_group.finish();
}

criterion_group!(benches, bench_mock_transport_throughput, bench_iroh_transport_throughput);
criterion_main!(benches);
