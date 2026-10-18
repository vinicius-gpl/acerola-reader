import { browser } from '$app/environment';
import { MODE_STORAGE_KEY } from '$lib/constants/themes';

export type ThemeMode = 'light' | 'dark';
export type ThemeModeOption = ThemeMode | 'system';

const prefersDark = browser ? window.matchMedia('(prefers-color-scheme: dark)') : null;

let mode = $state<ThemeModeOption>('dark');
let resolved = $state<ThemeMode>('dark');

function resolveMode(modeOption: ThemeModeOption): ThemeMode {
	switch (modeOption) {
		case 'system':
			return prefersDark?.matches ? 'dark' : 'light';
		case 'light':
		case 'dark':
			return modeOption;
	}
}

function applyTheme() {
	if (!browser) return;

	resolved = resolveMode(mode);

	const root = document.documentElement;
	root.setAttribute('data-theme', resolved);
	root.classList.toggle('dark', resolved === 'dark');
}

if (browser) {
	const savedMode = localStorage.getItem(MODE_STORAGE_KEY) as ThemeModeOption | null;

	if (savedMode) mode = savedMode;

	applyTheme();

	prefersDark?.addEventListener('change', () => {
		if (mode === 'system') applyTheme();
	});
}

export function useTheme() {
	function setMode(newMode: ThemeModeOption) {
		mode = newMode;
		localStorage.setItem(MODE_STORAGE_KEY, newMode);
		applyTheme();
	}

	return {
		setMode,
		get resolved() {
			return resolved;
		},
		get mode() {
			return mode;
		}
	};
}
