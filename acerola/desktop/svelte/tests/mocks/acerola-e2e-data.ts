import type {
	ComicSummaryItem,
	ComicSummaryPayload
} from '../../src/lib/contracts/home/home.payloads';
import type {
	ChapterDto,
	ChapterFilePayload
} from '../../src/lib/contracts/library/chapter.payloads';
import type {
	ReaderPagePayload,
	ReaderSessionPayload
} from '../../src/lib/contracts/reader/reader.payloads';
import type {
	DeviceInfoPayload,
	PairedPeerPayload,
	RelayInfo
} from '../../src/lib/contracts/network/network.payloads';
import type { LocalPeerAddr } from '../../src/lib/utils/connection-code.utils';

export const e2eComic: ComicSummaryItem = {
	relations: { directoryId: 'dir-1', metadataId: null },
	filesystem: { folderName: 'Acerola' },
	metadata: {
		title: 'Acerola',
		externalSync: false,
		activeSource: 'LOCAL',
		chapterCount: 3
	},
	artwork: {
		cover: 'C:\\Comics\\Acerola\\cover.png',
		banner: 'C:\\Comics\\Acerola\\banner.png'
	}
};

export const e2eChapters: ChapterFilePayload[] = [
	{
		id: 'chapter-1',
		name: 'Chapter 1',
		path: 'C:\\Comics\\Acerola\\chapter-1.cbz',
		chapterSort: '001',
		volumeId: null,
		volumeName: null,
		isSpecial: false,
		lastModified: 1
	},
	{
		id: 'chapter-2',
		name: 'Chapter 2',
		path: 'C:\\Comics\\Acerola\\chapter-2.cbz',
		chapterSort: '002',
		volumeId: null,
		volumeName: null,
		isSpecial: false,
		lastModified: 2
	},
	{
		id: 'chapter-3',
		name: 'Chapter 3',
		path: 'C:\\Comics\\Acerola\\chapter-3.cbz',
		chapterSort: '003',
		volumeId: null,
		volumeName: null,
		isSpecial: false,
		lastModified: 3
	}
];

export function e2eComicSummary(comics: ComicSummaryItem[] = [e2eComic]): ComicSummaryPayload {
	return {
		comics,
		total: comics.length,
		fetchedAt: '2026-06-08T00:00:00.000Z'
	};
}

export function e2eComicChapters(): ChapterDto {
	return {
		archive: {
			items: e2eChapters,
			volumes: [],
			pageSize: 25,
			page: 0,
			total: e2eChapters.length,
			volumeSections: []
		},
		showVolumeHeaders: false,
		hasVolumeStructure: false,
		effectiveViewMode: 'CHAPTER'
	};
}

export function e2eReaderSession(): ReaderSessionPayload {
	return {
		chapter: e2eChapters[0],
		pageCount: 3,
		currentPage: 0,
		cacheCapacity: 7
	};
}

export const e2eLocalPeerId = 'e2e-local-peer-id';

export const e2eLocalAddr: LocalPeerAddr = {
	id: { id: e2eLocalPeerId, device_id: null },
	addrs: [1, 2, 3, 4]
};

export const e2eLocalDeviceInfo: DeviceInfoPayload = {
	name: 'Meu PC',
	os: 'windows',
	version: '1.0.0'
};

export const e2eRelayInfo: RelayInfo = {
	defaultRelay: 'relay.acerola-comic.com',
	activeRelay: 'relay.acerola-comic.com'
};

export const e2ePairedPeer: PairedPeerPayload = {
	peerId: 'e2e-paired-peer-id',
	addrs: [5, 6, 7, 8],
	deviceName: 'Meu Celular'
};

export function e2eReaderPage(index: number): ReaderPagePayload {
	const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="320" height="480" viewBox="0 0 320 480"><rect width="320" height="480" fill="#202020"/><text x="160" y="240" fill="#fff" font-family="Arial" font-size="32" text-anchor="middle">Page ${index + 1}</text></svg>`;

	return {
		chapterId: e2eChapters[0].id,
		index,
		total: 3,
		mimeType: 'image/svg+xml',
		bytes: Array.from(new TextEncoder().encode(svg)),
		cacheHit: false
	};
}
