import { toast } from 'svelte-sonner';

type ToastAsyncMessages<T> = {
	loading: string;
	success: string | ((result: T) => string);
	error: string | ((err: unknown) => string);
	/// Tempo mínimo (ms) que o toast fica no estado "loading" antes de virar sucesso/erro —
	/// sem isso, uma operação local quase instantânea pula direto pro resultado final e a
	/// animação de transição do ícone (loading -> check) nunca chega a ser percebida.
	minDurationMs?: number;
};

const DEFAULT_MIN_DURATION_MS = 450;

function waitRemaining(startedAt: number, minDurationMs: number): Promise<void> {
	const remaining = minDurationMs - (performance.now() - startedAt);
	if (remaining <= 0) return Promise.resolve();
	return new Promise((resolve) => setTimeout(resolve, remaining));
}

/// Dispara um toast de loading, roda `action`, e atualiza o MESMO toast (mesmo `id`) pra
/// sucesso ou erro quando termina — o svelte-sonner troca o ícone in-place nesse caso (ver
/// `toastIconEnter` em `toast-icon-motion.utils.ts`), em vez de empilhar um segundo toast
/// independente como `toast.info(...)` seguido de `toast.success(...)` fazia antes.
export async function toastAsync<T>(
	action: () => Promise<T>,
	messages: ToastAsyncMessages<T>
): Promise<T> {
	const { loading, success, error, minDurationMs = DEFAULT_MIN_DURATION_MS } = messages;
	const id = toast.loading(loading);
	const startedAt = performance.now();

	try {
		const result = await action();
		await waitRemaining(startedAt, minDurationMs);
		toast.success(typeof success === 'function' ? success(result) : success, { id });
		return result;
	} catch (err) {
		await waitRemaining(startedAt, minDurationMs);
		toast.error(typeof error === 'function' ? error(err) : error, { id });
		throw err;
	}
}
