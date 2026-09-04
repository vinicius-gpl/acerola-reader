export const STORE_FILE = 'settings.json';

export const STORE_KEYS = {
	mode: 'mode',
	theme: 'theme',
	libraryPath: 'library_path',
	comicInfoPreference: 'comic_info_preference',
	volumeViewMode: 'volume_view_mode',
	readerMode: 'reader_mode',
	onboardingCompleted: 'onboarding_completed',
	metadataLanguage: 'metadata_language',
	relayUrl: 'relay_url',
	relayUseAcerola: 'relay_use_acerola',
	relayUseIrohPublic: 'relay_use_iroh_public',
	relayCustomUrls: 'relay_custom_urls',
	relayIrohUrls: 'relay_iroh_urls',
	deviceAlias: 'device_alias'
} as const;
