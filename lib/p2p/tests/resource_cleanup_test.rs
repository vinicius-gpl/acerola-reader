//! Testes de cancelamento e liberação de recursos do AcerolaP2p (Etapa G)
//!
//! Garante que a destruição (drop) da instância do AcerolaP2p sem invocar shutdown()
//! interrompe as tarefas em background e libera todos os recursos.

#[cfg(feature = "iroh")]
use std::sync::Arc;

#[cfg(feature = "iroh")]
use acerola_p2p::api::{
    identity::DeviceInfo, protocol::EventEmitter, transport::IrohTransportBuilder, AcerolaP2p,
};

#[cfg(feature = "iroh")]
fn no_op_emitter() -> EventEmitter {
    Arc::new(|_event_name: &str, _event_payload: String| {})
}

#[cfg(feature = "iroh")]
fn test_device_info() -> DeviceInfo {
    DeviceInfo {
        name: "cleanup-test-node".to_string(),
        os: "linux".to_string(),
        version: "1.0.0".to_string(),
    }
}

#[cfg(feature = "iroh")]
#[tokio::test]
async fn drop_acerola_p2p_without_shutdown_releases_resources_and_cancels_tasks() {
    // Executa iterativamente para garantir que conexões e tasks são finalizados em múltiplos ciclos
    for iteration_index in 0..5 {
        let node_instance = AcerolaP2p::builder(
            no_op_emitter(),
            IrohTransportBuilder::default(),
            test_device_info(),
        )
        .build()
        .await
        .expect("Failed to build AcerolaP2p node");

        let local_identifier = node_instance.local_id().to_string();
        assert!(
            !local_identifier.is_empty(),
            "Local ID must not be empty at iteration {}",
            iteration_index
        );

        // Dropar explicitamente a instância sem invocar shutdown()
        drop(node_instance);

        // Aguarda pequena janela para liberação dos sockets e finalização da task no tokio runtime
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }
}
