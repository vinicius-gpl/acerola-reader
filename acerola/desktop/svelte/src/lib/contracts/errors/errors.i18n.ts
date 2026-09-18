import * as m from '$lib/paraglide/messages';
import type { ErrorPayload } from '$lib/contracts/shared/shared.payloads';

// Contrato entre os tipos de erro do Rust (ComicError) e as mensagens do Paraglide.
// A chave é o valor exato do campo `errorType` vindo do payload do evento Tauri.
export const COMIC_ERROR_MESSAGES: Record<string, () => string> = {
	AlreadyExists: m['tauri_errors.comic.already_exists.label'],
	SyncInProgress: m['tauri_errors.comic.sync_in_progress.label'],
	NotFound: m['tauri_errors.comic.not_found.label'],
	InvalidRequest: m['tauri_errors.comic.invalid_request.label'],
	IntegrityViolation: m['tauri_errors.comic.integrity_violation.label'],
	SystemFailure: m['tauri_errors.comic.system_failure.label'],
	Io: m['tauri_errors.comic.io_error.label']
};

// Resolve a mensagem traduzida a partir do payload de erro.
// Usa o `message` técnico do Rust como fallback se o tipo não estiver mapeado.
export function resolveErrorMessage(payload: ErrorPayload): string {
	return COMIC_ERROR_MESSAGES[payload.errorType]?.() ?? payload.message;
}
