import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createNotifications } from './notification.svelte';

describe('createNotifications', () => {
	beforeEach(() => {
		vi.useFakeTimers();
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	it('adds notification by variant and returns id', () => {
		const store = createNotifications(['success', 'error'] as const);

		const id = store.notify.success('Scan concluído', { duration: 0 });

		expect(id).toBe(0);
		expect(store.notifications).toHaveLength(1);
		expect(store.notifications[0]).toMatchObject({
			id,
			message: 'Scan concluído',
			variant: 'success',
			duration: 0
		});
	});

	it('assigns increasing ids to successive notifications', () => {
		const store = createNotifications(['info'] as const);

		const firstId = store.notify.info('Primeira', { duration: 0 });
		const secondId = store.notify.info('Segunda', { duration: 0 });

		expect(secondId).toBeGreaterThan(firstId);
	});

	it('a duration: 0 notification does not get auto-removed by a timer', () => {
		const store = createNotifications(['info'] as const);
		store.notify.info('Permanente', { duration: 0 });

		vi.advanceTimersByTime(0);
		vi.advanceTimersByTime(60_000);

		expect(store.notifications).toHaveLength(1);
	});

	it('removes notification by id', () => {
		const store = createNotifications(['info'] as const);
		const id = store.notify.info('Em andamento', { duration: 0 });

		store.pop(id);

		expect(store.notifications).toEqual([]);
	});

	it('maintains list when removing non-existent id', () => {
		const store = createNotifications(['info'] as const);
		store.notify.info('Em andamento', { duration: 0 });

		store.pop(999);

		expect(store.notifications).toHaveLength(1);
	});

	it('clears all notifications', () => {
		const store = createNotifications(['info', 'error'] as const);
		store.notify.info('Primeira', { duration: 0 });
		store.notify.error('Segunda', { duration: 0 });

		store.clearAll();

		expect(store.notifications).toEqual([]);
	});

	// Nota: não mata os mutantes de notification.svelte.ts:28 (`options?.variant ?? variants[0]`
	// -> `options.variant ?? ...` / `... && variants[0]`) — são mutantes equivalentes de
	// verdade (ver comentário `Stryker disable` no source). add() é privada e só é chamada por
	// notify.<variant>(...), que sempre injeta `variant` no options antes de add() rodar, e o
	// `...options` logo depois sobrescreve o campo de qualquer jeito. Este teste continua útil
	// como rede de segurança: garante que chamar sem nenhum objeto de options não lança e ainda
	// resulta no variant correto.
	it('assigns default variant when called without an options object at all', () => {
		const store = createNotifications(['info', 'error'] as const);

		expect(() => store.notify.error('Sem opções')).not.toThrow();

		expect(store.notifications).toHaveLength(1);
		expect(store.notifications[0].variant).toBe('error');
	});

	it('removes automatically after configured duration', () => {
		const store = createNotifications(['info'] as const);
		store.notify.info('Temporária', { duration: 1000 });

		expect(store.notifications).toHaveLength(1);

		vi.advanceTimersByTime(1000);

		expect(store.notifications).toEqual([]);
	});
});
