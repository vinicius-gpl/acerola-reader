import { render, screen } from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import AcerolaToc from './acerola-toc.svelte';

// jsdom doesn't implement IntersectionObserver — Toc's $effect creates one unconditionally
// whenever it finds a matching container, so a minimal stub is required here (scoped to this
// file only, not added to the shared tests/setup.ts).
class FakeIntersectionObserver {
	observe() {}
	unobserve() {}
	disconnect() {}
}

// Same reasoning for ResizeObserver — the active-heading indicator (use:slidingIndicator)
// observes the <ul> container for size changes.
class FakeResizeObserver {
	observe() {}
	unobserve() {}
	disconnect() {}
}

beforeEach(() => {
	vi.stubGlobal('IntersectionObserver', FakeIntersectionObserver);
	vi.stubGlobal('ResizeObserver', FakeResizeObserver);
});

afterEach(() => {
	vi.unstubAllGlobals();
	document.body.innerHTML = '';
});

describe('AcerolaToc', () => {
	it('renders nothing when the content container is not found in the document', () => {
		const { container } = render(AcerolaToc);

		expect(container.querySelector('nav')).toBeNull();
	});

	it('lists the h2/h3 headings found inside the container, in document order', async () => {
		const content = document.createElement('div');
		content.className = 'doc-content';
		content.innerHTML = `
			<h2 id="intro">Introdução</h2>
			<h3 id="details">Detalhes</h3>
			<h2 id="no-id"></h2>
		`;
		document.body.appendChild(content);

		render(AcerolaToc);

		// Query by link role, not text: the source heading and the generated ToC entry share
		// the same text ("Introdução"), and only the generated entry is a link.
		expect(await screen.findByRole('link', { name: 'Introdução' })).toHaveAttribute(
			'href',
			'#intro'
		);
		expect(screen.getByRole('link', { name: 'Detalhes' })).toHaveAttribute('href', '#details');
	});

	it('ignores headings without an id', async () => {
		const content = document.createElement('div');
		content.className = 'doc-content';
		content.innerHTML = `<h2 id="intro">Introdução</h2><h2>Sem id</h2>`;
		document.body.appendChild(content);

		render(AcerolaToc);

		await screen.findByRole('link', { name: 'Introdução' });
		// The "Sem id" heading itself is still present in the source content — what must be
		// absent is a ToC entry (link) for it.
		expect(screen.queryByRole('link', { name: 'Sem id' })).not.toBeInTheDocument();
	});

	it('supports a custom containerSelector', async () => {
		const content = document.createElement('div');
		content.id = 'custom-container';
		content.innerHTML = `<h2 id="intro">Introdução</h2>`;
		document.body.appendChild(content);

		render(AcerolaToc, { props: { containerSelector: '#custom-container' } });

		expect(await screen.findByRole('link', { name: 'Introdução' })).toHaveAttribute(
			'href',
			'#intro'
		);
	});
});
