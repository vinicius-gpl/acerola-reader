export const LIBRARY_COMMANDS = {
	selectFolder: 'select_folder',
	comicInfoPreference: 'comic_info_preference'
} as const;

export const DIRECTORY_SCAN_COMMANDS = {
	refreshLibrary: 'refresh_library',
	incrementalScan: 'incremental_scan',
	rebuildLibrary: 'rebuild_library'
} as const;

export type DirectoryScanCommand =
	(typeof DIRECTORY_SCAN_COMMANDS)[keyof typeof DIRECTORY_SCAN_COMMANDS];
