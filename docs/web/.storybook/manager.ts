import { addons } from 'storybook/manager-api';
import { create } from 'storybook/theming';

addons.setConfig({
	theme: create({
		base: 'dark',
		brandTitle: 'Acerola Docs',
		// Logo real do site (docs/web/static/logo.svg) — o favicon.ico real da aba é
		// setado à parte em manager-head.html, não por aqui.
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
