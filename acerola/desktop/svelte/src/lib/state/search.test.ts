import { describe, expect, it } from 'vitest';
import { globalSearch, SearchState } from './search.svelte';

describe('SearchState / globalSearch', () => {
	it('starts with an empty query', () => {
		expect(globalSearch.query).toBe('');
	});

	it('allows reading and writing the query reactively', () => {
		globalSearch.query = 'one piece';

		expect(globalSearch.query).toBe('one piece');

		globalSearch.query = '';
	});

	it('creates independent instances with their own state', () => {
		const other = new SearchState();
		other.query = 'naruto';

		expect(other.query).toBe('naruto');
		expect(globalSearch.query).toBe('');
	});
});
