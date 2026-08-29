type NotificationOptions = {
	action?: { label: string; onClick: () => void };
	description?: string;
	duration?: number;
};

type Notification<V extends string> = {
	message: string;
	variant: V;
	id: number;
} & NotificationOptions;

type NotifyMethods<V extends string> = {
	[K in V]: (message: string, options?: NotificationOptions) => number;
};

export function createNotifications<V extends string>(variants: readonly V[]) {
	let notifications = $state<Notification<V>[]>([]);
	let _id = 0;

	function add(message: string, options?: NotificationOptions & { variant: V }): number {
		const id = _id++;

		const notify = {
			id,
			message,
			duration: 5000,
			// Stryker disable next-line OptionalChaining,LogicalOperator: fallback comprovadamente
			// morto — add() é privada, só é chamada por notify.<variant>(...) (ver linha ~51), que
			// SEMPRE injeta `variant` explícito no options antes de chamar add(). Logo `options`
			// nunca chega undefined aqui, e o `...options` alguns campos abaixo sobrescreve
			// incondicionalmente este valor de qualquer forma. Verificado empiricamente aplicando
			// os dois mutantes manualmente: os testes públicos (incluindo chamada sem options
			// nenhum) passam do mesmo jeito, pois é impossível observar a diferença via a API
			// exportada. Ver notification.test.ts.
			variant: options?.variant ?? variants[0],
			...options
		};

		notifications.push(notify);
		if (notify.duration > 0) setTimeout(() => pop(id), notify.duration);

		return id;
	}

	function pop(id: number) {
		const index = notifications.findIndex((it) => it.id === id);
		if (index !== -1) notifications.splice(index, 1);
	}

	function clearAll() {
		notifications.splice(0, notifications.length);
	}

	// deriva notify.* das chaves passadas no bootstrap
	const notify = Object.fromEntries(
		variants.map((variant) => [
			variant,
			(message: string, options?: NotificationOptions) => add(message, { ...options, variant })
		])
	) as NotifyMethods<V>;

	return {
		pop,
		notify,
		clearAll,
		get notifications() {
			return notifications;
		}
	};
}
