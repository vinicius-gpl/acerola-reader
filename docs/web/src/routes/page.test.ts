import { render, screen } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import LandingPage from './+page.svelte';

// FaultyTerminal usa a lib `ogl` pra desenhar um shader WebGL no hero — jsdom não tem WebGL,
// e o próprio time já documentou (ver faulty-terminal.svelte) instabilidade desse efeito sob
// GPU virtual/software mesmo no Chromium headless do Playwright. Mockamos a lib inteira aqui:
// o objetivo deste teste é a composição da landing page, não o shader em si.
vi.mock('ogl', () => {
	class FakeRenderer {
		gl = {
			clearColor: () => {},
			canvas: document.createElement('canvas'),
			getExtension: () => ({ loseContext: () => {} })
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

describe('landing +page', () => {
	beforeEach(() => {
		globalThis.ResizeObserver = class {
			observe = vi.fn();
			unobserve = vi.fn();
			disconnect = vi.fn();
		} as unknown as typeof ResizeObserver;
	});

	it('renders the hero title and the get started / github CTAs', () => {
		render(LandingPage);

		expect(screen.getByRole('link', { name: /get started|começar/i })).toHaveAttribute(
			'href',
			'/docs/getting-started'
		);
		expect(screen.getByRole('link', { name: /github/i })).toHaveAttribute(
			'href',
			expect.stringContaining('github.com')
		);
	});

	it('renders a platform card for each supported platform', () => {
		render(LandingPage);

		expect(screen.getAllByText(/android|desktop|relay|p2p/i).length).toBeGreaterThan(0);
	});
});
