import { browser } from '$app/environment';
import { STORE_FILE, STORE_KEYS } from '$lib/constants/store-plugin';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { LazyStore } from '@tauri-apps/plugin-store';

export type ThemeMode = 'light' | 'dark';
export type ThemeModeOption = ThemeMode | 'system';

const store = new LazyStore(STORE_FILE);

let mode = $state<ThemeModeOption>('dark');
let resolved = $state<ThemeMode>('dark');

async function applyTheme(modeOption: ThemeModeOption) {
	if (!browser) return;

	const window = getCurrentWindow();
	let newResolvedMode: ThemeMode = modeOption as ThemeMode;

	if (modeOption === 'system') {
		newResolvedMode = (await window.theme()) ?? 'light';
	}

	resolved = newResolvedMode;

	const root = document.documentElement;
	root.setAttribute('data-theme', resolved);
	root.classList.toggle('dark', resolved === 'dark');

	await window.setTheme(resolved);
}

// prettier-ignore
browser && (async () => {
    const savedMode = await store.get<ThemeModeOption>(STORE_KEYS.mode);

    if (savedMode) mode = savedMode;

    applyTheme(mode);

    const window = getCurrentWindow();
    window.onThemeChanged(async () => {
      if (mode === "system") {
        applyTheme("system");
      }
    });
  })();

export function useTheme() {
	async function setMode(newMode: ThemeModeOption) {
		mode = newMode;
		await store.set(STORE_KEYS.mode, newMode);
		applyTheme(mode);
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
