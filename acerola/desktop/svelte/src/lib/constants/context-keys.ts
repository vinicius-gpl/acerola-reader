export const CONTEXT_KEYS = {
	activeComic: Symbol('active-comic'),
	/// Compartilha a MESMA instância de `useNetworkSync()` entre `+layout.svelte` (nunca
	/// desmonta) e `routes/network/+page.svelte` — sem isso, cada um tinha sua própria
	/// instância com seus próprios listeners, e `syncComic()` (que só resolve via um listener
	/// da instância que o chamou) virava uma promise cancelada à força sempre que o usuário
	/// navegava pra outra tela com um sync ainda em andamento, mesmo o sync de verdade
	/// continuando (e terminando bem) no backend.
	networkSync: Symbol('network-sync')
} as const;
