import { describe, expect, it } from 'vitest';
import { cn } from './utils';

describe('cn', () => {
	it('joins plain class name strings', () => {
		expect(cn('a', 'b', 'c')).toBe('a b c');
	});

	it('drops falsy values', () => {
		expect(cn('a', false, undefined, null, 0, '', 'b')).toBe('a b');
	});

	it('resolves conflicting tailwind classes, keeping the last one', () => {
		expect(cn('px-2', 'px-4')).toBe('px-4');
		expect(cn('text-sm', 'text-lg')).toBe('text-lg');
	});

	it('merges non-conflicting classes from objects and arrays', () => {
		expect(cn(['a', 'b'], { c: true, d: false })).toBe('a b c');
	});

	it('lets a later conflicting class from an object win', () => {
		expect(cn('p-2', { 'p-4': true })).toBe('p-4');
	});
});
