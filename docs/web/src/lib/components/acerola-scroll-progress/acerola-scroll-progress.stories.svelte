<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaScrollProgress from './acerola-scroll-progress.svelte';

	const { Story } = defineMeta({
		title: 'Primitivos/AcerolaScrollProgress',
		component: AcerolaScrollProgress,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Barra de progresso de rolagem / leitura fixa no topo da tela, com scrub suave via GSAP ScrollTrigger e gradiente adaptável ao tema ativo.'
				}
			}
		}
	});
</script>

<Story
	name="Default"
	parameters={{
		docs: {
			description: {
				story: 'Barra de progresso de leitura fixa no topo.'
			}
		}
	}}
/>
