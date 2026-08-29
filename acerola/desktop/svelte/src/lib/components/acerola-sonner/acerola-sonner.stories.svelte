<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import { toast } from 'svelte-sonner';
	import AcerolaSonner from './acerola-sonner.svelte';

	const { Story } = defineMeta({
		title: 'Primitivos/AcerolaSonner',
		component: AcerolaSonner,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Wrapper do sistema de notificacoes da aplicacao baseado em Sonner. Este componente deve ser montado uma vez na arvore para permitir a exibicao de toasts globais de sucesso, erro, aviso e informacao disparados de qualquer ponto da interface.'
				}
			}
		}
	});
</script>

{#snippet template()}
	<div class="flex flex-col items-start gap-4">
		<AcerolaSonner />
		<button
			type="button"
			class="rounded bg-primary px-4 py-2 text-primary-foreground"
			onclick={() => toast.success('Toast disparado!')}
		>
			Disparar Toast
		</button>
	</div>
{/snippet}

<Story name="Default" {template} />

<Story
	name="Docs"
	parameters={{
		docs: {
			description: {
				story:
					'Este exemplo monta o toaster e expoe um botao para disparar um toast de sucesso. Em uso real, o `AcerolaSonner` deve permanecer montado em um layout ou shell principal, enquanto os toasts podem ser chamados com `toast.success(...)`, `toast.error(...)` e variacoes semelhantes em eventos de acao, feedback de formulario e operacoes assincronas.'
			}
		}
	}}
	{template}
/>
