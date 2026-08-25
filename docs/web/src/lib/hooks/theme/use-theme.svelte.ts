import { browser } from '$app/environment';
import { MODE_STORAGE_KEY, THEME_STORAGE_KEY, THEMES } from '$lib/constants/themes';

export type ThemeColor = keyof typeof THEMES;
export type ThemeMode = keyof (typeof THEMES)[keyof typeof THEMES];
export type ThemeModeOption = ThemeMode | 'system';

const prefersDark = browser ? window.matchMedia('(prefers-color-scheme: dark)') : null;

let theme = $state<ThemeColor>('catppuccin');
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
	root.setAttribute('data-theme', THEMES[theme][resolved]);
	root.classList.toggle('dark', resolved === 'dark');
}

if (browser) {
	const savedTheme = localStorage.getItem(THEME_STORAGE_KEY) as ThemeColor | null;
	const savedMode = localStorage.getItem(MODE_STORAGE_KEY) as ThemeModeOption | null;

	if (savedTheme && savedTheme in THEMES) theme = savedTheme;
	if (savedMode) mode = savedMode;

	applyTheme();

	prefersDark?.addEventListener('change', () => {
		if (mode === 'system') applyTheme();
	});
}

export function useTheme() {
	function setTheme(newTheme: ThemeColor) {
		theme = newTheme;
		localStorage.setItem(THEME_STORAGE_KEY, newTheme);
		applyTheme();
	}

	function setMode(newMode: ThemeModeOption) {
		mode = newMode;
		localStorage.setItem(MODE_STORAGE_KEY, newMode);
		applyTheme();
	}

	return {
		setMode,
		setTheme,
		get resolved() {
			return resolved;
		},
		get theme() {
			return theme;
		},
		get mode() {
			return mode;
		}
	};
}
