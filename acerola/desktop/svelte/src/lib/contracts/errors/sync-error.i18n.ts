import { m } from '$lib/paraglide/messages';

// Contrato com o `code` opcional que os protocolos de sync P2P (sync-files, sync-comic,
// sync-history, browse-library, browse-cover) anexam ao payload de `*:error` quando a causa
// tem uma tradução estável (ver `classify_sync_error`/`SyncErrorCode` em `transfer.rs`) — o
// `message` cru continua em inglês/técnico (log/wire, às vezes texto de baixo nível do QUIC
// tipo "stream reset by peer"), então sem isso esses erros apareceriam sem tradução na UI
// mesmo com o app todo em pt-BR. Código não reconhecido (`classify_sync_error` retornou
// `None`) cai no fallback natural de `translateSyncMessage` — mostra o `message` cru, mesmo
// padrão de `errors.i18n.ts` pros erros de `ComicError`. Compartilhado por todos os hooks que
// escutam eventos `*:error` desses protocolos (`use-network-sync.svelte.ts`,
// `use-remote-library.svelte.ts`) — mesmo `code` em todos, um lugar só pra traduzir.
const SYNC_ERROR_MESSAGES: Record<string, () => string> = {
	busy: m['tauri_errors.sync.session_busy.label'],
	timeout: m['tauri_errors.sync.timed_out.label'],
	connection_lost: m['tauri_errors.sync.connection_lost.label'],
	peer_not_found: m['tauri_errors.sync.peer_not_found.label'],
	auth_denied: m['tauri_errors.sync.auth_denied.label'],
	shutdown: m['tauri_errors.sync.shutdown.label'],
	incompatible_version: m['tauri_errors.sync.incompatible_version.label'],
	startup_failed: m['tauri_errors.sync.startup_failed.label'],
	blob_not_found: m['tauri_errors.sync.blob_not_found.label'],
	missing_blob_hash: m['tauri_errors.sync.missing_blob_hash.label'],
	blob_fetch_failed: m['tauri_errors.sync.blob_fetch_failed.label'],
	checksum_mismatch: m['tauri_errors.sync.checksum_mismatch.label'],
	comic_directory_unavailable: m['tauri_errors.sync.comic_directory_unavailable.label'],
	persist_failed: m['tauri_errors.sync.persist_failed.label'],
	no_pending_request: m['tauri_errors.sync.no_pending_request.label'],
	partial_sync: m['tauri_errors.sync.partial_sync.label']
};

export function translateSyncMessage(code: string | undefined, message: string): string {
	return (code && SYNC_ERROR_MESSAGES[code]?.()) ?? message;
}
