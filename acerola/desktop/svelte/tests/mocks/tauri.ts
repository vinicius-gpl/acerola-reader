import { vi } from 'vitest';

/**
 * Cria um mock do LazyStore com valores iniciais customizados.
 * Use em testes que precisam simular um estado salvo no store.
 *
 * @example
 * vi.mock("@tauri-apps/plugin-store", () => ({
 *   LazyStore: mockLazyStore({ mode: "dark" }),
 * }));
 */
export function mockLazyStore(savedValues: Record<string, unknown> = {}) {
	return vi.fn().mockImplementation(function () {
		return {
			get: vi.fn().mockImplementation((key: string) => Promise.resolve(savedValues[key] ?? null)),
			set: vi.fn().mockResolvedValue(undefined)
		};
	});
}
