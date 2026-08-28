import { addons } from 'storybook/manager-api';
import { create } from 'storybook/theming';

addons.setConfig({
	theme: create({
		base: 'dark',
		brandTitle: 'Acerola',
		// Logo de verdade do app (cópia de svelte/src/lib/assets/icons/acerola.svg em
		// svelte/static/logo.svg — precisa ser um arquivo estático simples, a manager UI
		// do Storybook não processa imports do Vite tipo `?component`). O favicon.ico real
		// da aba é setado à parte em manager-head.html, não por aqui.
		brandImage: 'logo.svg',
		brandTarget: '_self'
	})
});

const style = document.createElement('style');
style.textContent = `
  .sidebar-header a img {
    max-height: 34px;
    width: auto;
  }
`;
document.head.appendChild(style);
