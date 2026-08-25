// prettier-ignore
export const THEMES = {
  catppuccin:    { light: "catppuccin-latte", dark: "catppuccin-mocha" },
  nord:          { light: "nord-light",       dark: "nord-dark"        },
  dracula:       { light: "alucard",          dark: "dracula"          },
  'tokyo-night': { light: "tokyo-night-day",  dark: "tokyo-night-storm"},
} as const;

export const THEME_STORAGE_KEY = 'acerola-docs:theme';
export const MODE_STORAGE_KEY = 'acerola-docs:mode';
