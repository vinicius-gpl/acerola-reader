import { beforeEach, describe, expect, it, vi } from 'vitest';
import { convertFileSrc } from '@tauri-apps/api/core';
import { resolveArtworkPath, resolveBanner, resolveCover } from './artwork.utils';

vi.mock('@tauri-apps/api/core', () => ({
	convertFileSrc: vi.fn((path: string) => `asset://${path}`)
}));

const convertFileSrcMock = vi.mocked(convertFileSrc);

describe('resolveArtworkPath', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('returns null for a null path', () => {
		expect(resolveArtworkPath(null)).toBeNull();
		expect(convertFileSrcMock).not.toHaveBeenCalled();
	});

	it('returns null for an undefined path', () => {
		expect(resolveArtworkPath(undefined)).toBeNull();
		expect(convertFileSrcMock).not.toHaveBeenCalled();
	});

	it('returns null for an empty string path', () => {
		expect(resolveArtworkPath('')).toBeNull();
		expect(convertFileSrcMock).not.toHaveBeenCalled();
	});

	it('normalizes Windows backslashes to forward slashes before converting', () => {
		resolveArtworkPath('C:\\comics\\cover.jpg');

		expect(convertFileSrcMock).toHaveBeenCalledWith('C:/comics/cover.jpg');
	});

	it('returns the converted src for an already-normalized path', () => {
		const result = resolveArtworkPath('/comics/cover.jpg');

		expect(result).toBe('asset:///comics/cover.jpg');
	});
});

describe('resolveCover', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('prioritizes cover over banner', () => {
		resolveCover({ cover: 'cover.jpg', banner: 'banner.jpg' });

		expect(convertFileSrcMock).toHaveBeenCalledWith('cover.jpg');
	});

	it('falls back to banner when cover is missing', () => {
		resolveCover({ cover: null, banner: 'banner.jpg' });

		expect(convertFileSrcMock).toHaveBeenCalledWith('banner.jpg');
	});

	it('returns null when neither cover nor banner is present', () => {
		expect(resolveCover({})).toBeNull();
		expect(convertFileSrcMock).not.toHaveBeenCalled();
	});
});

describe('resolveBanner', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('prioritizes banner over cover', () => {
		resolveBanner({ cover: 'cover.jpg', banner: 'banner.jpg' });

		expect(convertFileSrcMock).toHaveBeenCalledWith('banner.jpg');
	});

	it('falls back to cover when banner is missing', () => {
		resolveBanner({ cover: 'cover.jpg', banner: null });

		expect(convertFileSrcMock).toHaveBeenCalledWith('cover.jpg');
	});

	it('returns null when neither banner nor cover is present', () => {
		expect(resolveBanner({})).toBeNull();
		expect(convertFileSrcMock).not.toHaveBeenCalled();
	});
});
