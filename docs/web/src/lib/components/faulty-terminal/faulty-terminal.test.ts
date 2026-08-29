import { render } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import FaultyTerminal from './faulty-terminal.svelte';

// Mesma justificativa do teste da landing page (routes/page.test.ts): jsdom não tem WebGL, e
// o próprio componente documenta instabilidade desse shader sob GPU virtual/software mesmo no
// Chromium headless. Mockamos a lib `ogl` inteira pra testar só o ciclo de vida do componente
// (monta o canvas, limpa no unmount), não o shader em si.
const loseContext = vi.fn();

vi.mock('ogl', () => {
	class FakeRenderer {
		gl = {
			clearColor: () => {},
			canvas: document.createElement('canvas'),
			getExtension: () => ({ loseContext })
		};
		setSize() {}
		render() {}
	}
	class FakeProgram {
		uniforms: Record<string, { value: unknown }>;
		constructor(_gl: unknown, options: { uniforms: Record<string, { value: unknown }> }) {
			this.uniforms = options.uniforms;
		}
	}
	class FakeMesh {}
	class FakeTriangle {}
	class FakeColor {
		constructor(...args: number[]) {
			Object.assign(this, args);
		}
	}

	return {
		Renderer: FakeRenderer,
		Program: FakeProgram,
		Mesh: FakeMesh,
		Triangle: FakeTriangle,
		Color: FakeColor
	};
});

describe('FaultyTerminal', () => {
	beforeEach(() => {
		loseContext.mockClear();
		globalThis.ResizeObserver = class {
			observe = vi.fn();
			unobserve = vi.fn();
			disconnect = vi.fn();
		} as unknown as typeof ResizeObserver;
	});

	it('mounts and appends the WebGL canvas to its container', () => {
		const { container } = render(FaultyTerminal, { props: { tint: '#cba6f7' } });

		expect(container.querySelector('canvas')).toBeInTheDocument();
	});

	it('releases the WebGL context on unmount', () => {
		const { unmount, container } = render(FaultyTerminal);
		expect(container.querySelector('canvas')).toBeInTheDocument();

		unmount();

		expect(loseContext).toHaveBeenCalled();
	});

	it('accepts a hex tint shorthand without throwing', () => {
		expect(() => render(FaultyTerminal, { props: { tint: '#fff' } })).not.toThrow();
	});
});
