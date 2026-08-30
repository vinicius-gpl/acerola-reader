<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaFaultyTerminal from './acerola-faulty-terminal.svelte';

	const { Story } = defineMeta({
		title: 'Primitivos/AcerolaFaultyTerminal',
		component: AcerolaFaultyTerminal,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Fundo animado em WebGL (via `ogl`) que simula um terminal com falhas: grade de digitos, scanlines, glitch e reacao ao mouse. Usado como plano de fundo decorativo da landing page.'
				}
			}
		},
		argTypes: {
			scale: { description: 'Escala da grade de digitos.', control: 'number' },
			digitSize: { description: 'Tamanho de cada digito dentro da celula.', control: 'number' },
			timeScale: { description: 'Velocidade da animacao ao longo do tempo.', control: 'number' },
			pause: { description: 'Congela a animacao no frame atual.', control: 'boolean' },
			scanlineIntensity: {
				description: 'Intensidade das linhas de varredura horizontais.',
				control: 'number'
			},
			glitchAmount: {
				description: 'Intensidade do deslocamento horizontal (glitch).',
				control: 'number'
			},
			flickerAmount: { description: 'Intensidade da cintilacao das linhas.', control: 'number' },
			noiseAmp: { description: 'Amplitude do ruido usado no padrao de fundo.', control: 'number' },
			chromaticAberration: {
				description: 'Deslocamento cromatico (RGB) em pixels.',
				control: 'number'
			},
			dither: {
				description: 'Intensidade do dithering aplicado ao resultado final.',
				control: 'number'
			},
			curvature: { description: 'Curvatura tipo CRT aplicada ao UV.', control: 'number' },
			tint: { description: 'Cor do efeito, em hexadecimal.', control: 'color' },
			mouseReact: { description: 'Se o efeito reage a posicao do mouse.', control: 'boolean' },
			mouseStrength: {
				description: 'Forca da influencia do mouse sobre o padrao.',
				control: 'number'
			},
			brightness: {
				description: 'Multiplicador de brilho aplicado a cor final.',
				control: 'number'
			},
			pageLoadAnimation: {
				description: 'Anima uma revelacao gradual das celulas ao montar o componente.',
				control: 'boolean'
			},
			class: { description: 'Classes extras aplicadas ao container.', control: 'text' }
		}
	});
</script>

<Story
	name="Default"
	args={{ tint: '#cba6f7' }}
	parameters={{
		docs: {
			description: {
				story: 'Configuracao padrao, reagindo ao mouse e com animacao de carregamento.'
			}
		}
	}}
/>

<Story
	name="Static"
	args={{ tint: '#cba6f7', mouseReact: false, pageLoadAnimation: false, glitchAmount: 0.2 }}
	parameters={{
		docs: {
			description: {
				story: 'Sem reacao ao mouse e com glitch reduzido, para um fundo mais discreto.'
			}
		}
	}}
/>
