<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { goto } from '$app/navigation';
	import { fly, fade } from 'svelte/transition';

	import AcerolaAlertDialog from '$lib/components/acerola-alert-dialog/acerola-alert-dialog.svelte';
	import AcerolaButton from '$lib/components/acerola-button/acerola-button.svelte';
	import AcerolaButtonIcon from '$lib/components/acerola-button/acerola-button-icon.svelte';
	import AcerolaCard from '$lib/components/acerola-card/acerola-card.svelte';
	import AcerolaInput from '$lib/components/acerola-input/acerola-input.svelte';
	import AcerolaSettingsHeader from '$lib/components/acerola-settings-header/acerola-settings-header.svelte';
	import AcerolaToggleGroup from '$lib/components/acerola-toggle-group/acerola-toggle-group.svelte';
	import * as ToggleGroup from '$lib/components/ui/toggle-group/index.js';
	import { extractErrorMessage } from '$lib/utils/error.utils';
	import { autoAnimateList } from '$lib/utils/auto-animate.utils';
	import { m } from '$lib/paraglide/messages';

	import { useArchiveTemplates } from '$lib/hooks/store/use-archive-templates.svelte';
	import type {
		ArchiveTemplate,
		ArchiveTemplateSortType
	} from '$lib/contracts/archive/archive-template.payloads';

	import Trash2Icon from '@lucide/svelte/icons/trash-2';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import LockIcon from '@lucide/svelte/icons/lock';

	const templateStore = useArchiveTemplates();

	// Macros e exemplos são sintaxe técnica, não texto de idioma — ficam fixos aqui
	// em vez de nas mensagens i18n, para não colidir com a interpolação do paraglide.
	const CHAPTER_EXAMPLE = 'Ch. {chapter}{decimal}.*.{extension}';
	const VOLUME_EXAMPLE = 'Vol. {volume}{decimal}';
	// {extension} nunca é digitado pelo usuário: é sempre o último token de um
	// template de capítulo, então o formulário anexa automaticamente na hora de criar.
	const CHAPTER_BODY_PLACEHOLDER = 'Ch. {chapter}{decimal}.*.';

	let newLabel = $state('');
	let newPattern = $state('');
	let newSortType = $state<ArchiveTemplateSortType>('Chapter');
	let isCreating = $state(false);

	let templateToDelete = $state<ArchiveTemplate | null>(null);

	const chapterTemplates = $derived(
		templateStore.templates.filter((template) => template.sort_type === 'Chapter')
	);
	const volumeTemplates = $derived(
		templateStore.templates.filter((template) => template.sort_type === 'Volume')
	);

	onMount(() => {
		templateStore.loadTemplates().catch((err: unknown) => {
			toast.error(m['pages.config.templates.toast.load_error']({ msg: extractErrorMessage(err) }));
		});
	});

	function selectSortType(value: ArchiveTemplateSortType) {
		newSortType = value;
		// Troca de tipo limpa o padrão: uma macro de capítulo dentro de um template
		// de volume (ou vice-versa) não é barrada pelo parser e gera extração errada.
		newPattern = '';
	}

	function insertMacro(token: string) {
		newPattern += token;
	}

	async function handleCreate() {
		if (!newLabel.trim() || !newPattern.trim() || isCreating) return;

		isCreating = true;
		try {
			const fullPattern = newSortType === 'Chapter' ? `${newPattern}{extension}` : newPattern;
			const created = await templateStore.createTemplate(newLabel, fullPattern, newSortType);
			toast.success(m['pages.config.templates.toast.create_success']({ label: created.label }));
			newLabel = '';
			newPattern = '';
		} catch (err: unknown) {
			toast.error(
				m['pages.config.templates.toast.create_error']({ msg: extractErrorMessage(err) })
			);
		} finally {
			isCreating = false;
		}
	}

	async function handleDelete() {
		if (!templateToDelete) return;

		try {
			await templateStore.deleteTemplate(templateToDelete.id);
			toast.success(m['pages.config.templates.toast.delete_success']());
		} catch (err: unknown) {
			toast.error(
				m['pages.config.templates.toast.delete_error']({ msg: extractErrorMessage(err) })
			);
		} finally {
			templateToDelete = null;
		}
	}
</script>

<div
	in:fly={{ x: 16, duration: 180 }}
	out:fade={{ duration: 100 }}
	class="mx-auto w-full max-w-5xl space-y-10 p-8"
>
	<div class="space-y-1">
		<AcerolaSettingsHeader
			data={{ title: m['pages.config.templates.title']() }}
			events={{ onBack: () => goto('/config') }}
		/>
		<p class="text-muted-foreground">
			{m['pages.config.templates.desc']()}
		</p>
	</div>

	<!-- Referência: como montar um template -->
	<AcerolaCard
		data={{
			title: m['pages.config.templates.help.title'](),
			description: m['pages.config.templates.help.desc']()
		}}
	>
		<div class="grid gap-3 sm:grid-cols-2">
			<div class="rounded-xl border border-border/40 bg-background/50 p-3">
				<code class="text-sm font-bold text-primary">{'{chapter}'}</code>
				<p class="mt-1 text-xs text-muted-foreground">
					{m['pages.config.templates.help.macro_chapter_desc']()}
				</p>
			</div>
			<div class="rounded-xl border border-border/40 bg-background/50 p-3">
				<code class="text-sm font-bold text-primary">{'{volume}'}</code>
				<p class="mt-1 text-xs text-muted-foreground">
					{m['pages.config.templates.help.macro_volume_desc']()}
				</p>
			</div>
			<div class="rounded-xl border border-border/40 bg-background/50 p-3">
				<code class="text-sm font-bold text-primary">{'{decimal}'}</code>
				<p class="mt-1 text-xs text-muted-foreground">
					{m['pages.config.templates.help.macro_decimal_desc']()}
				</p>
			</div>
			<div class="rounded-xl border border-border/40 bg-background/50 p-3">
				<code class="text-sm font-bold text-primary">{'{extension}'}</code>
				<p class="mt-1 text-xs text-muted-foreground">
					{m['pages.config.templates.help.macro_extension_desc']()}
				</p>
			</div>
			<div class="rounded-xl border border-border/40 bg-background/50 p-3 sm:col-span-2">
				<code class="text-sm font-bold text-primary">*</code>
				<p class="mt-1 text-xs text-muted-foreground">
					{m['pages.config.templates.help.wildcard_desc']()}
				</p>
			</div>
		</div>

		<div class="mt-5 grid gap-3 sm:grid-cols-2">
			<div class="rounded-xl bg-muted/30 p-3">
				<p class="text-xs font-semibold text-foreground">
					{m['pages.config.templates.help.example_chapter_label']()}
				</p>
				<code class="mt-1 block truncate text-xs text-muted-foreground">{CHAPTER_EXAMPLE}</code>
				<p class="mt-1 text-xs text-muted-foreground/80">
					{m['pages.config.templates.help.example_chapter_match']()}
				</p>
			</div>
			<div class="rounded-xl bg-muted/30 p-3">
				<p class="text-xs font-semibold text-foreground">
					{m['pages.config.templates.help.example_volume_label']()}
				</p>
				<code class="mt-1 block truncate text-xs text-muted-foreground">{VOLUME_EXAMPLE}</code>
				<p class="mt-1 text-xs text-muted-foreground/80">
					{m['pages.config.templates.help.example_volume_match']()}
				</p>
			</div>
		</div>
	</AcerolaCard>

	<!-- Criar novo template -->
	<AcerolaCard data={{ title: m['pages.config.templates.form.title']() }}>
		<div class="space-y-4">
			<div class="space-y-1">
				<span class="text-xs font-semibold">{m['pages.config.templates.form.sort_type']()}</span>
				<AcerolaToggleGroup
					state={{ value: newSortType }}
					events={{
						onValueChange: (value) =>
							selectSortType((value as ArchiveTemplateSortType) ?? 'Chapter')
					}}
					ui={{ variant: 'outline' }}
				>
					<ToggleGroup.Item value="Chapter" class="px-4">
						{m['pages.config.templates.form.sort_type_chapter']()}
					</ToggleGroup.Item>
					<ToggleGroup.Item value="Volume" class="px-4">
						{m['pages.config.templates.form.sort_type_volume']()}
					</ToggleGroup.Item>
				</AcerolaToggleGroup>
			</div>

			<div class="grid gap-4 sm:grid-cols-2">
				<div class="space-y-1">
					<label for="templateLabel" class="text-xs font-semibold"
						>{m['pages.config.templates.form.label']()}</label
					>
					<AcerolaInput
						state={{ value: newLabel }}
						events={{ onValueChange: (value) => (newLabel = value) }}
						ui={{
							id: 'templateLabel',
							placeholder: m['pages.config.templates.form.label_placeholder'](),
							class: 'w-full bg-background'
						}}
					/>
				</div>

				<div class="space-y-1">
					<label for="templatePattern" class="text-xs font-semibold"
						>{m['pages.config.templates.form.pattern']()}</label
					>

					<div class="relative">
						<AcerolaInput
							state={{ value: newPattern }}
							events={{ onValueChange: (value) => (newPattern = value) }}
							ui={{
								id: 'templatePattern',
								placeholder: newSortType === 'Chapter' ? CHAPTER_BODY_PLACEHOLDER : VOLUME_EXAMPLE,
								class:
									newSortType === 'Chapter'
										? 'w-full bg-background font-mono pr-24'
										: 'w-full bg-background font-mono'
							}}
						/>
						{#if newSortType === 'Chapter'}
							<span
								class="pointer-events-none absolute inset-y-0 right-3 flex items-center font-mono text-xs font-semibold text-primary"
							>
								{'{extension}'}
							</span>
						{/if}
					</div>

					<div class="flex flex-wrap items-center gap-1.5 pt-0.5">
						{#if newSortType === 'Chapter'}
							<AcerolaButton
								events={{ onClick: () => insertMacro('{chapter}') }}
								ui={{ variant: 'outline', size: 'xs', class: 'rounded-full font-mono' }}
							>
								+ {'{chapter}'}
							</AcerolaButton>
							<AcerolaButton
								events={{ onClick: () => insertMacro('{decimal}') }}
								ui={{ variant: 'outline', size: 'xs', class: 'rounded-full font-mono' }}
							>
								+ {'{decimal}'}
							</AcerolaButton>
						{:else}
							<AcerolaButton
								events={{ onClick: () => insertMacro('{volume}') }}
								ui={{ variant: 'outline', size: 'xs', class: 'rounded-full font-mono' }}
							>
								+ {'{volume}'}
							</AcerolaButton>
							<AcerolaButton
								events={{ onClick: () => insertMacro('{decimal}') }}
								ui={{ variant: 'outline', size: 'xs', class: 'rounded-full font-mono' }}
							>
								+ {'{decimal}'}
							</AcerolaButton>
						{/if}
					</div>

					{#if newSortType === 'Chapter'}
						<p class="text-xs text-muted-foreground/80">
							{m['pages.config.templates.form.extension_auto_hint']()}
						</p>
					{/if}
				</div>
			</div>

			<div class="flex justify-end">
				<AcerolaButton
					events={{ onClick: handleCreate }}
					ui={{
						disabled: !newLabel.trim() || !newPattern.trim() || isCreating,
						class: 'h-10 gap-2 px-6'
					}}
				>
					<PlusIcon size={16} />
					{isCreating
						? m['pages.config.templates.form.creating']()
						: m['pages.config.templates.form.create']()}
				</AcerolaButton>
			</div>
		</div>
	</AcerolaCard>

	<!-- Listagem -->
	<section class="grid gap-8 sm:grid-cols-2">
		{#each [{ title: m['pages.config.templates.list.chapter_title'](), items: chapterTemplates, empty: m['pages.config.templates.list.empty_chapter']() }, { title: m['pages.config.templates.list.volume_title'](), items: volumeTemplates, empty: m['pages.config.templates.list.empty_volume']() }] as group (group.title)}
			<div class="space-y-3">
				<h2 class="text-xs font-bold tracking-widest text-muted-foreground uppercase">
					{group.title}
				</h2>

				{#if group.items.length === 0}
					<div
						class="rounded-xl border border-dashed border-border/40 p-6 text-center text-sm text-muted-foreground"
					>
						{group.empty}
					</div>
				{:else}
					<div class="space-y-2" use:autoAnimateList>
						{#each group.items as template (template.id)}
							<div
								class="flex items-center justify-between gap-3 rounded-xl border border-border/40 bg-background/50 p-3 transition-colors hover:bg-muted/50"
							>
								<div class="min-w-0">
									<div class="flex items-center gap-2">
										<span class="truncate font-medium">{template.label}</span>
										{#if template.is_default}
											<span
												title={m['pages.config.templates.list.default_hint']()}
												class="flex shrink-0 items-center gap-1 rounded-full bg-primary/10 px-2 py-0.5 text-[10px] font-bold tracking-wide text-primary uppercase"
											>
												<LockIcon size={10} />
												{m['pages.config.templates.list.default_badge']()}
											</span>
										{/if}
									</div>
									<code class="block truncate text-xs text-muted-foreground"
										>{template.pattern}</code
									>
								</div>

								{#if !template.is_default}
									<AcerolaButtonIcon
										events={{ onClick: () => (templateToDelete = template) }}
										ui={{
											variant: 'ghost',
											class:
												'size-8 shrink-0 text-muted-foreground hover:bg-destructive/10 hover:text-destructive'
										}}
									>
										<Trash2Icon size={16} />
									</AcerolaButtonIcon>
								{/if}
							</div>
						{/each}
					</div>
				{/if}
			</div>
		{/each}
	</section>
</div>

<!-- Delete Confirmation Alert Dialog -->
<AcerolaAlertDialog
	state={{ open: templateToDelete !== null }}
	data={{
		title: m['pages.config.templates.delete_confirm.title'](),
		description: templateToDelete
			? m['pages.config.templates.delete_confirm.desc']({ label: templateToDelete.label })
			: '',
		cancelText: m['pages.config.templates.delete_confirm.cancel'](),
		actionText: m['pages.config.templates.delete_confirm.action']()
	}}
	events={{
		onAction: handleDelete,
		onCancel: () => (templateToDelete = null)
	}}
	ui={{ variant: 'destructive' }}
/>
