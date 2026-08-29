import { render } from '@testing-library/svelte';
import { tick } from 'svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { load, LazyStore } from '@tauri-apps/plugin-store';
import { STORE_KEYS } from '$lib/constants/store-plugin';
import HookHarness from '../../../../tests/harness/hooks/rune-wrapper.svelte';

const mockStoreMethods = vi.hoisted(() => ({
	get: vi.fn((_key: string): Promise<boolean | null> => Promise.resolve(null)),
	set: vi.fn((_key: string, _value: unknown) => Promise.resolve()),
	save: vi.fn(() => Promise.resolve())
}));

vi.mock('@tauri-apps/plugin-store', () => ({
	load: vi.fn(),
	LazyStore: vi.fn().mockImplementation(function () {
		return mockStoreMethods;
	})
}));

import { useOnboarding } from './use-onboarding.svelte';

const loadMock = vi.mocked(load);

async function renderHook() {
	let hook: ReturnType<typeof useOnboarding> | undefined;

	render(HookHarness, {
		props: {
			create: useOnboarding,
			onReady: (value) => {
				hook = value as ReturnType<typeof useOnboarding>;
			}
		}
	});

	await tick();
	await Promise.resolve();

	return hook!;
}

describe('useOnboarding', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		useOnboarding().setStep(0);
	});

	it('resolves the onboarding status at module load (checkStatus already ran)', async () => {
		// `checkStatus()` roda uma única vez no import do módulo, com `store.get()`
		// resolvendo `null` por padrão (mock global) — por isso `isCompleted` cai no
		// fallback `?? false`, e `isLoading` termina em `false` depois de resolver.
		const hook = await renderHook();

		expect(hook.isLoading).toBe(false);
		expect(hook.isCompleted).toBe(false);
	});

	it('starts with isLoading=true and isCompleted=false before checkStatus resolves', async () => {
		// Faz `store.get()` retornar uma Promise que só resolve quando mandarmos,
		// segurando `checkStatus()` no meio do `await`. Com `vi.resetModules()` +
		// import dinâmico, pegamos uma instância nova do módulo (o state de
		// `isLoading`/`isCompleted` é module-level), então lemos os valores
		// iniciais do hook antes de qualquer microtask de `checkStatus` rodar.
		let resolveGet!: (value: boolean | null) => void;
		mockStoreMethods.get.mockImplementationOnce(
			() =>
				new Promise<boolean | null>((resolve) => {
					resolveGet = resolve;
				})
		);

		vi.resetModules();
		const { useOnboarding: freshUseOnboarding } = await import('./use-onboarding.svelte');
		const hook = freshUseOnboarding();

		expect(hook.isLoading).toBe(true);
		expect(hook.isCompleted).toBe(false);

		resolveGet(true);
		await Promise.resolve();
		await Promise.resolve();
		await Promise.resolve();

		expect(hook.isLoading).toBe(false);
		expect(hook.isCompleted).toBe(true);
	});

	it('starts with step 0 and manages step forward and backward', async () => {
		const hook = await renderHook();

		expect(hook.currentStep).toBe(0);

		hook.nextStep();
		expect(hook.currentStep).toBe(1);

		hook.nextStep();
		expect(hook.currentStep).toBe(2);

		hook.prevStep();
		expect(hook.currentStep).toBe(1);

		hook.setStep(4);
		expect(hook.currentStep).toBe(4);
	});

	it('marks onboarding as completed and saves to store', async () => {
		const hook = await renderHook();

		await hook.complete();

		expect(mockStoreMethods.set).toHaveBeenCalledWith(STORE_KEYS.onboardingCompleted, true);
		expect(mockStoreMethods.save).toHaveBeenCalledOnce();
		expect(hook.isCompleted).toBe(true);
	});
});
