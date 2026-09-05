use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager, Runtime, State};

use crate::{
    bios::scopes::read_relay_settings,
    cmd::events::network::{DeviceInfoPayload, NetworkStatusPayload, PairedPeerPayload, RelayInfo},
    core::services::network::NetworkServiceApi,
    data::{models::sync::SyncHistoryLogEntry, repositories::sync::SyncHistoryLogRepository},
    infra::{
        security::MasterKeySource,
        sync::{
            messages::SyncDirection,
            protocol::{
                comic_sync_registry::PendingComicSyncRegistry,
                cover_request_registry::PendingCoverRequestRegistry, COMIC_SYNC_ALPN,
                COVER_BROWSE_ALPN, FILE_SYNC_ALPN, HISTORY_SYNC_ALPN, LIBRARY_BROWSE_ALPN,
            },
        },
    },
};

const SYNC_HISTORY_LOG_LIMIT: i64 = 200;

#[tauri::command]
pub async fn get_network_status<R: Runtime>(
    app: AppHandle<R>, service: State<'_, Arc<dyn NetworkServiceApi>>,
) -> Result<(), String> {
    let mode = service.mode().await?;
    let peers = service.connected_peers_with_info().await?;

    app.emit("network:status", NetworkStatusPayload::from(mode, peers)).unwrap();

    Ok(())
}

#[tauri::command]
pub async fn switch_to_local(service: State<'_, Arc<dyn NetworkServiceApi>>) -> Result<(), String> {
    service.switch_to_local().await?;
    Ok(())
}

#[tauri::command]
pub async fn switch_to_relay(service: State<'_, Arc<dyn NetworkServiceApi>>) -> Result<(), String> {
    service.switch_to_relay().await?;
    Ok(())
}

#[tauri::command]
pub async fn get_local_id(
    service: State<'_, Arc<dyn NetworkServiceApi>>,
) -> Result<String, String> {
    service.local_id()
}

/// Endereço completo pra gerar o código/QR de pareamento (ver [`NetworkServiceApi::local_addr`]).
#[tauri::command]
pub async fn get_local_addr(
    service: State<'_, Arc<dyn NetworkServiceApi>>,
) -> Result<acerola_p2p::api::peer::PeerAddr, String> {
    service.local_addr()
}

/// Nome/OS/versão deste dispositivo, pra exibir algo legível na tela de Rede em vez do
/// peer id cru.
#[tauri::command]
pub async fn get_local_device_info(
    service: State<'_, Arc<dyn NetworkServiceApi>>,
) -> Result<DeviceInfoPayload, String> {
    Ok(DeviceInfoPayload::from(service.local_device_info().await?))
}

/// Define um apelido customizado pro dispositivo local (estilo LocalSend, em vez do hostname
/// automático) — vale a partir do próximo handshake, sem precisar reiniciar o app. Persistência
/// entre reinícios é responsabilidade do frontend (`settings.json`, chave `device_alias`), que
/// `bios::network::setup_network` relê no próximo boot.
#[tauri::command]
pub async fn set_local_device_name(
    service: State<'_, Arc<dyn NetworkServiceApi>>, name: String,
) -> Result<(), String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("device name cannot be empty".to_string());
    }

    service.set_local_device_name(trimmed.to_string()).await
}

/// Retorna a configuração de relay combinável atual (relay do Acerola / próprio(s) /
/// Iroh / rede pública Iroh), lida de `settings.json`.
#[tauri::command]
pub async fn get_relay_info<R: Runtime>(
    app: AppHandle<R>, service: State<'_, Arc<dyn NetworkServiceApi>>,
) -> Result<RelayInfo, String> {
    let app_data_directory = app.path().app_data_dir().map_err(|error| error.to_string())?;
    let has_ticket = service.has_iroh_services_ticket().await?;
    Ok(RelayInfo::new(read_relay_settings(&app_data_directory), has_ticket))
}

/// Salva o ticket da conta do usuário em `services.iroh.computer` (colado por ele na tela de
/// configuração de rede) no cofre criptografado — nunca em `settings.json`. Valida o formato
/// antes de persistir. Não aplica sozinho a config nova — o frontend chama
/// [`apply_relay_settings`] logo em seguida, mesmo fluxo de qualquer outra mudança de relay.
#[tauri::command]
pub async fn set_iroh_services_ticket(
    service: State<'_, Arc<dyn NetworkServiceApi>>, ticket: String,
) -> Result<(), String> {
    service.set_iroh_services_ticket(ticket).await
}

/// Remove o ticket salvo — usado quando o usuário desliga a fonte ou substitui por um novo.
#[tauri::command]
pub async fn clear_iroh_services_ticket(
    service: State<'_, Arc<dyn NetworkServiceApi>>,
) -> Result<(), String> {
    service.clear_iroh_services_ticket().await
}

/// Relê `settings.json` + o ticket do cofre e aplica a config de relay resolvida ao node JÁ
/// VIVO, sem precisar reiniciar o app — ver `NetworkServiceApi::apply_relay_settings`. O
/// frontend chama isso depois de QUALQUER mudança nas fontes de relay (toggle do
/// Acerola/Iroh, add/remove de URL própria, salvar/remover ticket).
#[tauri::command]
pub async fn apply_relay_settings(
    service: State<'_, Arc<dyn NetworkServiceApi>>,
) -> Result<(), String> {
    service.apply_relay_settings().await
}

#[tauri::command]
pub async fn connect_to_peer(
    service: State<'_, Arc<dyn NetworkServiceApi>>, peer_id: String, addrs: Vec<u8>, alpn: String,
) -> Result<(), String> {
    use acerola_p2p::api::peer::{PeerAddr, PeerIdentity};

    let peer_identity = PeerIdentity { id: peer_id, device_id: None };
    let peer_addr = PeerAddr { id: peer_identity, addrs };
    service.connect(peer_addr, alpn.into_bytes()).await?;
    Ok(())
}

/// Todo peer já pareado alguma vez, com o último endereço conhecido — sobrevive a restart e
/// independe de conexão ativa agora (ver [`NetworkServiceApi::paired_peers`]). É essa lista,
/// não `get_network_status`, que deve alimentar "disparar sync com X" na UI.
#[tauri::command]
pub async fn get_paired_peers(
    service: State<'_, Arc<dyn NetworkServiceApi>>,
) -> Result<Vec<PairedPeerPayload>, String> {
    Ok(service.paired_peers().await?.into_iter().map(PairedPeerPayload::from).collect())
}

/// Desempareia um peer — some da lista de pareados e da confiança (TOFU). Se ele tentar se
/// conectar de novo depois, passa pelo mesmo fluxo de confirmação de um dispositivo nunca
/// visto (ver [`NetworkServiceApi::remove_peer`]).
#[tauri::command]
pub async fn remove_paired_peer(
    service: State<'_, Arc<dyn NetworkServiceApi>>, peer_id: String,
) -> Result<(), String> {
    service.remove_peer(peer_id).await
}

/// Dispara uma sessão de sync de histórico com um peer já pareado. O progresso e a
/// conclusão chegam pro frontend via os eventos `sync:history:*`, emitidos direto pelo
/// protocolo (ver `infra::sync::protocol::history_handler`) assim que a conexão é aceita.
#[tauri::command]
pub async fn sync_history(
    service: State<'_, Arc<dyn NetworkServiceApi>>, peer_id: String, addrs: Vec<u8>,
) -> Result<(), String> {
    use acerola_p2p::api::peer::{PeerAddr, PeerIdentity};

    let peer_addr = PeerAddr { id: PeerIdentity { id: peer_id, device_id: None }, addrs };
    service.connect(peer_addr, HISTORY_SYNC_ALPN.to_vec()).await?;
    Ok(())
}

/// Dispara uma sessão de sync de arquivos com um peer já pareado. Progresso via os
/// eventos `sync:files:*`.
#[tauri::command]
pub async fn sync_files(
    service: State<'_, Arc<dyn NetworkServiceApi>>, peer_id: String, addrs: Vec<u8>,
) -> Result<(), String> {
    use acerola_p2p::api::peer::{PeerAddr, PeerIdentity};

    let peer_addr = PeerAddr { id: PeerIdentity { id: peer_id, device_id: None }, addrs };
    service.connect(peer_addr, FILE_SYNC_ALPN.to_vec()).await?;
    Ok(())
}

/// Dispara histórico e arquivos em sequência contra o mesmo peer.
#[tauri::command]
pub async fn sync_all(
    service: State<'_, Arc<dyn NetworkServiceApi>>, peer_id: String, addrs: Vec<u8>,
) -> Result<(), String> {
    use acerola_p2p::api::peer::{PeerAddr, PeerIdentity};

    let peer_identity = PeerIdentity { id: peer_id, device_id: None };

    let history_addr = PeerAddr { id: peer_identity.clone(), addrs: addrs.clone() };
    service.connect(history_addr, HISTORY_SYNC_ALPN.to_vec()).await?;

    let files_addr = PeerAddr { id: peer_identity, addrs };
    service.connect(files_addr, FILE_SYNC_ALPN.to_vec()).await?;

    Ok(())
}

/// Dispara uma sessão de sync individual de UM quadrinho com um peer já pareado — funciona
/// tanto pra "empurrar" (eu tenho, o peer não) quanto pra "puxar" (o peer tem, eu não), ver o
/// comentário de design em `infra::sync::protocol::comic_handler`. Registra o `comic_name` no
/// `PendingComicSyncRegistry` antes de conectar, porque o `Handler` (`ComicSyncOutbound`) é um
/// singleton do boot e não recebe esse parâmetro por chamada de `connect()`. Progresso via os
/// eventos `sync:comic:*`.
#[tauri::command]
pub async fn sync_comic(
    service: State<'_, Arc<dyn NetworkServiceApi>>,
    registry: State<'_, Arc<PendingComicSyncRegistry>>, peer_id: String, addrs: Vec<u8>,
    comic_name: String, direction: SyncDirection,
) -> Result<(), String> {
    use acerola_p2p::api::peer::{PeerAddr, PeerIdentity};

    registry.set(peer_id.clone(), comic_name, direction);

    let peer_addr = PeerAddr { id: PeerIdentity { id: peer_id, device_id: None }, addrs };
    service.connect(peer_addr, COMIC_SYNC_ALPN.to_vec()).await?;
    Ok(())
}

/// Consulta a biblioteca remota de um peer já pareado (só títulos + contagem de capítulos, sem
/// transferir nada) — pré-requisito pra escolher um quadrinho pra puxar (`sync_comic`). O
/// resultado chega pro frontend via o evento `library:query:result`.
#[tauri::command]
pub async fn query_remote_library(
    service: State<'_, Arc<dyn NetworkServiceApi>>, peer_id: String, addrs: Vec<u8>,
) -> Result<(), String> {
    use acerola_p2p::api::peer::{PeerAddr, PeerIdentity};

    let peer_addr = PeerAddr { id: PeerIdentity { id: peer_id, device_id: None }, addrs };
    service.connect(peer_addr, LIBRARY_BROWSE_ALPN.to_vec()).await?;
    Ok(())
}

/// Busca a capa (thumbnail) de UM quadrinho remoto — `known_version` é a versão já cacheada
/// localmente (`None` se nunca buscou essa capa antes). Mesmo padrão fire-and-forget de
/// `sync_comic`: enfileira `(comic_name, known_version)` em `PendingCoverRequestRegistry` antes
/// de conectar, porque o `Handler` (`CoverBrowseOutbound`) é um singleton do boot e não recebe
/// esse parâmetro por chamada de `connect()`. `use-remote-library.svelte.ts::fetchCoversFor`
/// chama este comando em paralelo (um por quadrinho da lista) pro mesmo peer — por isso o
/// registry é uma fila por peer (`push`/`take` FIFO), não um slot único que uma chamada
/// sobrescreveria a da outra (bug reportado/corrigido em 22/08/2026, ver doc do registry).
/// Resultado via `browse:cover:result`/`browse:cover:error`.
#[tauri::command]
pub async fn query_remote_cover(
    service: State<'_, Arc<dyn NetworkServiceApi>>,
    registry: State<'_, Arc<PendingCoverRequestRegistry>>, peer_id: String, addrs: Vec<u8>,
    comic_name: String, known_version: Option<i64>,
) -> Result<(), String> {
    use acerola_p2p::api::peer::{PeerAddr, PeerIdentity};

    registry.push(peer_id.clone(), comic_name, known_version);

    let peer_addr = PeerAddr { id: PeerIdentity { id: peer_id, device_id: None }, addrs };
    service.connect(peer_addr, COVER_BROWSE_ALPN.to_vec()).await?;
    Ok(())
}

/// Últimas sessões de sync (histórico e arquivos) persistidas — sobrevive a restart,
/// diferente do log ao vivo em memória do frontend (`use-network-sync.svelte.ts`), que só
/// tem os eventos da sessão atual do app.
#[tauri::command]
pub async fn get_sync_history_log(
    repo: State<'_, SyncHistoryLogRepository>,
) -> Result<Vec<SyncHistoryLogEntry>, String> {
    repo.find_recent(SYNC_HISTORY_LOG_LIMIT).await.map_err(|error| error.to_string())
}

/// Se `true`, a chave mestra que criptografa identidade/peers/confiança caiu pro fallback
/// em arquivo local por falta de um keyring do SO utilizável — ver
/// `infra::security::get_or_create_master_key`. Consultado sob demanda (em vez de só
/// confiar no evento `security:keyring_unavailable`) porque `setup_network` roda numa task
/// separada que pode terminar antes do frontend montar e começar a ouvir eventos.
#[tauri::command]
pub async fn get_security_status(source: State<'_, MasterKeySource>) -> Result<bool, String> {
    Ok(*source == MasterKeySource::FallbackFile)
}
