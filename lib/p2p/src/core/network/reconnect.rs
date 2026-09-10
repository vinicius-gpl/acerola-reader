//! Reconexão proativa com peers conhecidos depois que o node é reconstruído — troca de relay
//! (`apply_relay_settings`) ou o botão manual "Reiniciar" (`NetworkServiceApi::restart` no
//! Desktop, `P2PNode::restart` no Android, os dois chamadores esperados deste módulo).
//!
//! Sem isso, um node recém-reconstruído sobe com identidade/storage antigos mas ZERO conexões
//! ativas (`NetworkState` começa vazio a cada `AcerolaP2pBuilder::build()` — ver
//! `core/network/state.rs`) e fica parado esperando: nada tenta falar de novo com nenhum peer
//! pareado até o usuário disparar alguma ação manual na UI (browse library, sync, etc). Achado
//! ao vivo: trocar de relay silenciosamente corta o alcance por relay pra qualquer peer que não
//! esteja no mesmo relay novo, e ninguém era avisado nem uma tentativa de reconexão acontecia
//! sozinha.

use futures::future::join_all;

use crate::{api::AcerolaP2p, infra::peer::PeerAddr};

/// ALPN usado só pra sondar reachability — o mesmo handshake usado no pareamento
/// (`RpcClientHandler`/`RpcServerHandler`, registrado incondicionalmente por
/// `AcerolaP2pBuilder::build` pra QUALQUER node, não é específico de nenhum app). É a operação
/// mais barata que existe (troca PING/PONG/DeviceInfo, sem nenhuma transferência de dado real),
/// e seu sucesso já é reportado pelos eventos que o protocolo já emite
/// (`rpc:device_info_received`/`rpc:device_info_exchanged`) — este módulo não precisa saber nada
/// sobre esse protocolo além do nome do ALPN.
pub const RECONNECT_PROBE_ALPN: &[u8] = b"acerola/handshake/1";

/// Dispara uma tentativa de handshake pra CADA peer em `known_peers`, em paralelo — chamado logo
/// depois que um node novo é construído (restart). `AcerolaP2p::connect` só enfileira o pedido
/// pro `NetworkManager` (que sozinho já faz retry com backoff exponencial — 5 tentativas, ver
/// `handle_connect_command`) e retorna na hora: esta função não espera nenhuma confirmação de
/// sucesso, só garante que a TENTATIVA aconteça pra todo peer que já foi pareado antes, sem
/// depender do usuário abrir uma tela e clicar em algo primeiro. O resultado real (conectou ou
/// não) chega pelos MESMOS eventos que qualquer outro `connect()` já emite hoje.
///
/// Peers cujo `PeerAddr::addrs` está vazio são pulados — sem coordenadas pra discar, `connect`
/// cairia no fallback de endereço cacheado do TRANSPORTE (não do storage do app que chama esta
/// função), que um node recém-nascido nunca tem ainda, então a tentativa seria descartada de
/// qualquer forma (ver `IrohTransport::open_bi`); melhor nem enfileirar o comando.
pub async fn reconnect_known_peers(node: &AcerolaP2p, known_peers: Vec<PeerAddr>) {
    let attempts = known_peers.into_iter().filter(|addr| !addr.addrs.is_empty()).map(|addr| {
        let peer_id = addr.id.clone();
        async move {
            if let Err(error) = node.connect(addr, RECONNECT_PROBE_ALPN).await {
                tracing::warn!(
                    peer = %peer_id,
                    ?error,
                    "failed to queue reconnect attempt after restart"
                );
            }
        }
    });

    join_all(attempts).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        api::{identity::DeviceInfo, transport::IrohTransportBuilder},
        infra::peer::PeerId,
    };

    fn no_op_emitter() -> crate::api::protocol::EventEmitter {
        std::sync::Arc::new(|_event: &str, _payload: String| {})
    }

    fn test_device_info() -> DeviceInfo {
        DeviceInfo {
            name: "test-device".to_string(),
            os: "test-os".to_string(),
            version: "0.0.0".to_string(),
        }
    }

    async fn build_node() -> AcerolaP2p {
        AcerolaP2p::builder(no_op_emitter(), IrohTransportBuilder::default(), test_device_info())
            .build()
            .await
            .unwrap()
    }

    /// Regressão central: um node A que nunca discou pra ninguém nesta sessão, ao receber a
    /// lista de peers conhecidos (que inclui um node B real), deve conseguir se tornar
    /// "reachable" pra B SOZINHO — sem nenhuma ação adicional do chamador além de invocar esta
    /// função. Prova que a reconexão de fato acontece de ponta a ponta, não só que um comando
    /// foi enfileirado.
    #[tokio::test]
    async fn reconnects_to_a_real_known_peer_without_any_further_action() {
        let node_a = build_node().await;
        let node_b = build_node().await;

        let addr_b = node_b.local_addr().unwrap();
        let id_b = PeerId {
            id: node_b.local_id().to_string(),
            device_id: node_b.local_device_id().map(|s| s.to_string()),
        };

        // Handshake nunca aconteceu nesta sessão — exatamente o estado de um node recém-saído
        // de um restart.
        assert!(!node_a.is_peer_reachable(&id_b).await);

        reconnect_known_peers(&node_a, vec![addr_b]).await;

        let mut reachable = false;
        for _ in 0..50 {
            if node_a.is_peer_reachable(&id_b).await {
                reachable = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        assert!(reachable, "reconnect_known_peers deveria ter restabelecido a conexão sozinho");
    }

    /// Sem isso, `connect()` seria enfileirado pra um peer cujo endereço não tem NENHUMA
    /// coordenada de rede — `open_bi` cairia no fallback de endereço cacheado do transporte
    /// (vazio, num node recém-nascido) e falharia igual, só que depois de gastar uma tentativa
    /// inteira de retry. Filtrar aqui evita esse desperdício.
    #[tokio::test]
    async fn skips_peers_with_no_cached_address_without_panicking() {
        let node_a = build_node().await;
        let unreachable_peer = PeerAddr {
            id: PeerId { id: "ghost-peer".to_string(), device_id: None },
            addrs: vec![],
        };

        reconnect_known_peers(&node_a, vec![unreachable_peer]).await;
    }

    /// `reconnect_known_peers` recebendo uma lista vazia (node sem nenhum peer pareado ainda)
    /// não deve travar nem entrar em pânico — só não faz nada.
    #[tokio::test]
    async fn empty_known_peers_list_is_a_no_op() {
        let node_a = build_node().await;
        reconnect_known_peers(&node_a, vec![]).await;
    }

    /// Caminho triste real: um peer pareado pode ter saído da rede entre sessões (desligou o
    /// app, perdeu conexão) — isso não pode impedir a reconexão com os DEMAIS peers do MESMO
    /// lote. Cada tentativa roda numa task própria dentro do `NetworkManager`
    /// (`handle_connect_command`), então uma falhando (5 tentativas com backoff até desistir)
    /// não deveria atrasar nem travar as outras — só um teste que force os dois casos no MESMO
    /// lote prova isso de verdade, em vez de assumir pela arquitetura.
    #[tokio::test]
    async fn reconnects_to_the_reachable_peer_even_when_another_in_the_same_batch_is_offline() {
        let node_a = build_node().await;
        let node_b = build_node().await;

        let addr_b = node_b.local_addr().unwrap();
        let id_b = PeerId {
            id: node_b.local_id().to_string(),
            device_id: node_b.local_device_id().map(|s| s.to_string()),
        };

        // Endereço REAL e válido (deserializa normalmente), mas o node por trás dele foi
        // desligado ANTES da tentativa — simula um peer pareado que saiu da rede, não um
        // endereço corrompido/nunca existente (já coberto pelo teste de "sem coordenada").
        let offline_node = build_node().await;
        let offline_addr = offline_node.local_addr().unwrap();
        offline_node.shutdown().await.expect("shutdown should succeed");

        reconnect_known_peers(&node_a, vec![offline_addr, addr_b]).await;

        let mut reachable = false;
        for _ in 0..50 {
            if node_a.is_peer_reachable(&id_b).await {
                reachable = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        assert!(
            reachable,
            "o peer OFFLINE no mesmo lote não pode impedir a reconexão com o peer alcançável"
        );
    }

    /// Se o node já estiver desligado (canal de comando pro `NetworkManager` fechado),
    /// `AcerolaP2p::connect` retorna `Err` NA HORA (antes de qualquer tentativa de rede) pra
    /// QUALQUER peer da lista. `reconnect_known_peers` precisa engolir esse erro peer por peer
    /// (só loga um aviso) em vez de propagar ou entrar em pânico — e continuar processando o
    /// resto do lote em vez de abortar no primeiro.
    #[tokio::test]
    async fn does_not_panic_when_connect_fails_immediately_for_every_peer() {
        let node_a = build_node().await;
        // Qualquer endereço não-vazio serve — o que importa aqui é que o CANAL de comando já
        // está fechado, não o conteúdo do endereço.
        let some_valid_addrs = node_a.local_addr().unwrap().addrs;
        node_a.shutdown().await.expect("shutdown should succeed");

        let peer_one = PeerAddr {
            id: PeerId { id: "peer-one".to_string(), device_id: None },
            addrs: some_valid_addrs.clone(),
        };
        let peer_two = PeerAddr {
            id: PeerId { id: "peer-two".to_string(), device_id: None },
            addrs: some_valid_addrs,
        };

        reconnect_known_peers(&node_a, vec![peer_one, peer_two]).await;
    }
}
