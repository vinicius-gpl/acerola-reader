<script module lang="ts">
	export type NetworkMyDeviceCardProps = {
		data: {
			deviceName: string | null | undefined;
			localId: string | null | undefined;
			mode: 'relay' | 'local' | undefined;
			activeRelay: string | null | undefined;
			isRelayOverridden: boolean;
		};
	};
</script>

<script lang="ts">
	import { toast } from 'svelte-sonner';
	import Share2Icon from '@lucide/svelte/icons/share-2';
	import MonitorIcon from '@lucide/svelte/icons/monitor';
	import CopyIcon from '@lucide/svelte/icons/copy';
	import CheckIcon from '@lucide/svelte/icons/check';
	import AcerolaHeroButton from '$lib/components/acerola-hero-button/acerola-hero-button.svelte';
	import AcerolaButton from '$lib/components/acerola-button/acerola-button.svelte';
	import { m } from '$lib/paraglide/messages';
	import { shortId } from '$lib/utils/connection-code.utils';

	let { data }: NetworkMyDeviceCardProps = $props();

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
</script>

<section class="space-y-4">
	<div
		class="flex items-center gap-3 text-xs font-bold tracking-widest text-muted-foreground uppercase"
	>
		<Share2Icon size={16} />
		{m['pages.network.my_device.title']()}
	</div>

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
