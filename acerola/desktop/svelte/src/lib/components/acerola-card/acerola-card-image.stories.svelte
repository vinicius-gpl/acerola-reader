<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';

	import MoreVerticalIcon from '@lucide/svelte/icons/more-vertical';
	import BookOpenIcon from '@lucide/svelte/icons/book-open';

	import AcerolaCardImage from './acerola-card-image.svelte';

	const { Story } = defineMeta({
		title: 'Primitivos/AcerolaCardImage',
		component: AcerolaCardImage,
		tags: ['autodocs'],

		parameters: {
			docs: {
				description: {
					component: 'Card de quadrinho com imagem de capa.'
				}
			}
		}
	});
</script>

<Story
	name="Sem capa"
	args={{
		data: {
			title: 'Berserk'
		}
	}}
/>

<Story name="Com progresso" asChild>
	<AcerolaCardImage
		data={{
			title: 'Berserk',
			description: 'Descrição de teste',
			progress: 50
		}}
	>
		{#snippet floatingBadge()}
			<div
				class="absolute -top-2 left-2 flex h-6 w-6 items-center justify-center rounded-full bg-red-500 text-xs font-bold text-white shadow-md"
			>
				N
			</div>
		{/snippet}
	</AcerolaCardImage>
</Story>

<Story
	name="Com descrição no overlay"
	args={{
		data: {
			title: 'Berserk',
			description: 'Uma história sombria de espada e feitiçaria.'
		}
	}}
/>

<Story name="Com footer e action" asChild>
	<AcerolaCardImage
		data={{
			title: 'Berserk'
		}}
	>
		{#snippet footer()}
			<span
				class="flex items-center gap-1 text-[10px] font-black tracking-wider text-muted-foreground uppercase"
			>
				<BookOpenIcon size={10} />
				374 Caps
			</span>
		{/snippet}

		{#snippet action()}
			<button type="button" class="p-1 text-muted-foreground transition-colors hover:text-primary">
				<MoreVerticalIcon size={16} />
			</button>
		{/snippet}
	</AcerolaCardImage>
</Story>

<Story name="Com overlay customizado" asChild>
	<AcerolaCardImage
		data={{
			title: 'Vagabond'
		}}
	>
		{#snippet overlay()}
			<button
				type="button"
				class="w-full rounded-2xl bg-primary py-2 text-xs font-black text-primary-foreground"
			>
				LER
			</button>
		{/snippet}
	</AcerolaCardImage>
</Story>
