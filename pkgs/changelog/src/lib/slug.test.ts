import { describe, expect, it } from 'vitest';
import { readingMinutes, slugify } from './slug';

describe('slugify', () => {
	it('lowercases, strips accents and joins with hyphens', () => {
		expect(slugify('  Héllo, World! 2026 ')).toBe('hello-world-2026');
	});
	it('collapses punctuation runs', () => {
		expect(slugify('a -- b __ c')).toBe('a-b-c');
	});
});

describe('readingMinutes', () => {
	it('is at least one minute', () => {
		expect(readingMinutes('')).toBe(1);
	});
	it('rounds up at 200 words per minute', () => {
		expect(readingMinutes('word '.repeat(401))).toBe(3);
	});
});
