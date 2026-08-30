<script module lang="ts">
	export type NetworkMyDeviceCardProps = {
		data: {
			deviceName: string | null | undefined;
			localId: string | null | undefined;
			mode: 'relay' | 'local' | undefined;
			activeRelay: string | null | undefined;
			isRelayOverridden: boolean;
		};
		events?: {
			onRenameDevice?: (name: string) => Promise<void>;
		};
	};
</script>

<script lang="ts">
	import { toast } from 'svelte-sonner';
	import Share2Icon from '@lucide/svelte/icons/share-2';
	import MonitorIcon from '@lucide/svelte/icons/monitor';
	import CopyIcon from '@lucide/svelte/icons/copy';
	import CheckIcon from '@lucide/svelte/icons/check';
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import XIcon from '@lucide/svelte/icons/x';
	import AcerolaHeroButton from '$lib/components/acerola-hero-button/acerola-hero-button.svelte';
	import AcerolaButton from '$lib/components/acerola-button/acerola-button.svelte';
	import AcerolaButtonIcon from '$lib/components/acerola-button/acerola-button-icon.svelte';
	import AcerolaInput from '$lib/components/acerola-input/acerola-input.svelte';
	import { m } from '$lib/paraglide/messages';
	import { shortId } from '$lib/utils/connection-code.utils';

	let { data, events }: NetworkMyDeviceCardProps = $props();

	// No teardown da story (Storybook + vitest browser mode), o efeito reativo deste
	// template roda mais uma vez com `data` já undefined antes do componente ser
	// destruído de fato — sem o fallback aqui isso vaza como unhandled error e derruba
	// a suíte mesmo com todos os asserts passando.
	let safeData = $derived(
		data ?? {
			deviceName: undefined,
			localId: undefined,
			mode: undefined,
			activeRelay: undefined,
			isRelayOverridden: false
		}
	);

	// Feedback visual do botão de copiar — além do toast, o próprio ícone vira um check
	// por um instante, pra quem não pegar o toast a tempo perceber que a cópia aconteceu.
	let idJustCopied = $state(false);

	async function copyLocalId() {
		if (!safeData.localId) return;
		await navigator.clipboard.writeText(safeData.localId);
		toast.success(m['pages.network.copied']());
		idJustCopied = true;
		setTimeout(() => (idJustCopied = false), 2000);
	}

	// Apelido custom estilo LocalSend — edição inline em vez de um dialog separado, já que é
	// um campo único e a tela de Rede já é o lugar natural pra mexer nisso.
	let editingName = $state(false);
	let nameDraft = $state('');
	let savingName = $state(false);

	function startEditingName() {
		nameDraft = safeData.deviceName ?? '';
		editingName = true;
	}

	function cancelEditingName() {
		editingName = false;
	}

	async function confirmEditingName() {
		const trimmed = nameDraft.trim();
		if (!trimmed || savingName || !events?.onRenameDevice) return;

		savingName = true;
		try {
			await events.onRenameDevice(trimmed);
			toast.success(m['pages.network.my_device.rename.success']());
			editingName = false;
		} catch (err) {
			toast.error(m['pages.network.my_device.rename.error']({ msg: String(err) }));
		} finally {
			savingName = false;
		}
	}
</script>

<section class="space-y-4">
	<div
		class="flex items-center gap-3 text-xs font-bold tracking-widest text-muted-foreground uppercase"
	>
		<Share2Icon size={16} />
		{m['pages.network.my_device.title']()}
	</div>

	{#if editingName}
		<div class="flex items-center gap-3 rounded-3xl border border-border bg-card p-6">
			<AcerolaInput
				state={{ value: nameDraft }}
				events={{ onValueChange: (value) => (nameDraft = value) }}
				ui={{
					placeholder: m['pages.network.my_device.rename.placeholder'](),
					class: 'flex-1',
					disabled: savingName
				}}
			/>
			<AcerolaButton
				events={{ onClick: confirmEditingName }}
				ui={{ size: 'sm', disabled: !nameDraft.trim() || savingName }}
			>
				<CheckIcon size={14} />
				{m['pages.network.my_device.rename.save']()}
			</AcerolaButton>
			<AcerolaButtonIcon
				events={{ onClick: cancelEditingName }}
				ui={{
					variant: 'ghost',
					class: 'size-10',
					disabled: savingName,
					'aria-label': m['pages.network.my_device.rename.cancel']()
				}}
			>
				<XIcon size={16} />
			</AcerolaButtonIcon>
		</div>
	{:else}
		<AcerolaHeroButton
			data={{
				title: safeData.deviceName ?? '...',
				description: safeData.localId ? shortId(safeData.localId) : '...'
			}}
		>
			{#snippet icon()}
				<MonitorIcon size={22} />
			{/snippet}

			{#snippet action()}
				{#if events?.onRenameDevice}
					<AcerolaButtonIcon
						events={{ onClick: startEditingName }}
						ui={{
							variant: 'outline',
							class: 'size-10',
							'aria-label': m['pages.network.my_device.rename.action']()
						}}
					>
						<PencilIcon size={14} />
					</AcerolaButtonIcon>
				{/if}
				<AcerolaButton
					events={{ onClick: copyLocalId }}
					ui={{ variant: 'outline', size: 'sm', class: 'gap-2', disabled: !safeData.localId }}
				>
					{#if idJustCopied}
						<CheckIcon size={14} class="text-chart-3" />
					{:else}
						<CopyIcon size={14} />
					{/if}
					{m['pages.network.my_device.copy_id']()}
				</AcerolaButton>
			{/snippet}
		</AcerolaHeroButton>
	{/if}

	<p class="px-2 text-xs text-muted-foreground">
		{safeData.mode === 'relay'
			? m['pages.network.my_device.mode_relay']()
			: m['pages.network.my_device.mode_local']()}
		· {m['pages.network.my_device.relay_label']()}: {safeData.activeRelay ?? '...'}
		{#if safeData.isRelayOverridden}
			<span class="text-chart-4">({m['pages.network.my_device.relay_custom']()})</span>
		{:else}
			<span>({m['pages.network.my_device.relay_default']()})</span>
		{/if}
	</p>
</section>
