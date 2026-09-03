<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaApkDownloadButton from './acerola-apk-download-button.svelte';

	const { Story } = defineMeta({
		title: 'Primitivos/AcerolaApkDownloadButton',
		component: AcerolaApkDownloadButton,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Badge de download do APK do Android — mesmo chip escuro fixo do badge da Microsoft Store, com o logo oficial do Android, pra manter os dois botões de plataforma consistentes lado a lado.'
				}
			}
		},
		argTypes: {
			label: {
				description: 'Legenda pequena acima de "APK".',
				control: 'text'
			}
		}
	});
</script>

<Story name="Default" args={{}} />

<Story name="CustomLabel" args={{ label: 'Baixe o' }} />
