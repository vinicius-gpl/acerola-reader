// TODO: `brandImage` abaixo troca a LOGO da sidebar (usa favicon.png como logo), mas
// nunca customiza o favicon.ico real da aba do navegador (fica o padrão do Storybook).
// Em docs/web/.storybook/main.ts acontece o inverso: staticDirs serve favicon.svg na
// raiz e o navegador pega isso como favicon da aba sozinho, mas a logo da sidebar nunca
// foi customizada (sem manager.ts lá). Os dois projetos deviam ter os dois consistentes.
import { addons } from 'storybook/manager-api';
import { create } from 'storybook/theming';

addons.setConfig({
	theme: create({
		base: 'dark',
		brandTitle: 'Acerola',
		brandImage: 'favicon.png',
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
