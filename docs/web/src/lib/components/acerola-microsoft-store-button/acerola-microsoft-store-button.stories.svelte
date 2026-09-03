<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaMicrosoftStoreButton from './acerola-microsoft-store-button.svelte';

	const { Story } = defineMeta({
		title: 'Primitivos/AcerolaMicrosoftStoreButton',
		component: AcerolaMicrosoftStoreButton,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Badge de download pra Microsoft Store — sempre um chip escuro fixo (independe do tema claro/escuro do site), com o logo oficial da loja e uma legenda curta acima do nome da loja, igual às convenções usuais de badge de app store.'
				}
			}
		},
		argTypes: {
			label: {
				description: 'Legenda pequena acima de "Microsoft Store".',
				control: 'text'
			}
		}
	});
</script>

<Story name="Default" args={{}} />

<Story name="CustomLabel" args={{ label: 'Baixe na' }} />
