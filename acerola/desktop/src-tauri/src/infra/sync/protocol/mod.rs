pub mod comic_handler;
pub mod comic_sync_registry;
pub mod cover_browse_handler;
pub mod cover_request_registry;
pub mod file_handler;
pub mod file_session_guard;
pub mod history_entry_handler;
pub mod history_entry_registry;
pub mod history_handler;
pub mod library_browse_handler;
pub mod transfer;

pub const HISTORY_SYNC_ALPN: &[u8] = b"acerola/sync-history/1";
/// Push individual de UMA entrada de histórico (progresso de UM quadrinho), sem trocar o
/// manifesto da biblioteca inteira — ver `history_entry_handler.rs`. Complementar a
/// `HISTORY_SYNC_ALPN`, não substitui: aquele continua sendo o sync completo (botão "Sincronizar
/// histórico"), este é o "mandar só esse capítulo" pontual.
pub const HISTORY_ENTRY_SYNC_ALPN: &[u8] = b"acerola/sync-history-entry/1";
pub const FILE_SYNC_ALPN: &[u8] = b"acerola/sync-files/1";
/// Sync individual de UM quadrinho (push ou pull, ver `comic_handler.rs`) — reaproveita o
/// mesmo `FileSyncSessionGuard` de `FILE_SYNC_ALPN`.
pub const COMIC_SYNC_ALPN: &[u8] = b"acerola/sync-comic/1";
/// Lista/busca a biblioteca remota antes de decidir um sync individual — round-trip único,
/// sem guard (ver `library_browse_handler.rs`). String e formato idênticos ao Android
/// (`protocol/library_browse/mod.rs::LIBRARY_BROWSE_ALPN`) — os dois lados não compartilham
/// código, então precisa bater exatamente ou o transporte (que roteia por ALPN exato) nunca
/// encontra handler do outro lado.
pub const LIBRARY_BROWSE_ALPN: &[u8] = b"acerola/browse-library/1";
/// Busca a capa (thumbnail) de UM quadrinho remoto por vez, complementar ao `browse-library` —
/// ver `cover_browse_handler.rs`. Sem sessão/guard: round-trip curto, não corre o mesmo risco de
/// I/O pesado duplicado que `sync-files`/`sync-comic` correm. String idêntica ao Android
/// (`protocol/cover_browse/mod.rs::COVER_BROWSE_ALPN`).
pub const COVER_BROWSE_ALPN: &[u8] = b"acerola/browse-cover/1";
