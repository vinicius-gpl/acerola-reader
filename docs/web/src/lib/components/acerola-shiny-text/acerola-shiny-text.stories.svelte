<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaShinyText from './acerola-shiny-text.svelte';

	const { Story } = defineMeta({
		title: 'Primitivos/AcerolaShinyText',
		component: AcerolaShinyText,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Texto com um brilho que varre da esquerda para a direita continuamente, como um leve efeito de destaque. Respeita `prefers-reduced-motion`.'
				}
			}
		},
		argTypes: {
			text: { description: 'Texto exibido.', control: 'text' },
			speed: { description: 'Duracao de uma volta da animacao, em segundos.', control: 'number' },
			class: { description: 'Classes extras aplicadas ao elemento.', control: 'text' }
		}
	});
</script>

<Story
	name="Default"
	args={{ text: 'Acerola Reader' }}
	parameters={{
		docs: {
			description: {
				story: 'Velocidade padrao de 6 segundos por ciclo.'
			}
		}
	}}
/>

<Story
	name="Fast"
	args={{ text: 'Acerola Reader', speed: 2 }}
	parameters={{
		docs: {
			description: {
				story: 'Velocidade customizada, util para chamar mais atencao em destaques curtos.'
			}
		}
	}}
/>
