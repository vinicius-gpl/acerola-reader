<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaDotField from './acerola-dot-field.svelte';

	const { Story } = defineMeta({
		title: 'Backgrounds/AcerolaDotField',
		component: AcerolaDotField,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Componente de fundo pontilhado interativo (Dot Field do SvelteBits) com deformação ao passar o cursor, brilho e ondas.'
				}
			}
		},
		argTypes: {
			dotRadius: { control: 'number', description: 'Raio dos pontos' },
			dotSpacing: { control: 'number', description: 'Espaçamento entre os pontos' },
			cursorRadius: { control: 'number', description: 'Raio de alcance do cursor' },
			bulgeStrength: { control: 'number', description: 'Intensidade de repulsão dos pontos' },
			glowRadius: { control: 'number', description: 'Raio do brilho radial sob o cursor' },
			sparkle: { control: 'boolean', description: 'Efeito de cintilação aleatória' },
			waveAmplitude: { control: 'number', description: 'Amplitude da animação de onda' },
			gradientFrom: { control: 'color', description: 'Cor inicial do gradiente' },
			gradientTo: { control: 'color', description: 'Cor final do gradiente' },
			glowColor: { control: 'color', description: 'Cor do brilho do cursor' }
		}
	});
</script>

<Story name="Default" asChild>
	<div class="relative h-96 w-full overflow-hidden rounded-2xl bg-[#120F17]">
		<AcerolaDotField />
	</div>
</Story>

<Story name="Tema Acerola (Catppuccin)" asChild>
	<div class="relative h-96 w-full overflow-hidden rounded-2xl bg-[#1e1e2e]">
		<AcerolaDotField
			gradientFrom="rgba(203, 166, 247, 0.45)"
			gradientTo="rgba(245, 194, 231, 0.3)"
			glowColor="rgba(203, 166, 247, 0.15)"
		/>
	</div>
</Story>

<Story name="Com Onda e Sparkle" asChild>
	<div class="relative h-96 w-full overflow-hidden rounded-2xl bg-[#120F17]">
		<AcerolaDotField waveAmplitude={4} sparkle={true} />
	</div>
</Story>
