<script module lang="ts">
	import type { PairedPeerPayload } from '$lib/contracts/network/network.payloads';

	export type AcerolaPeerPickerProps = {
		state: {
			open: boolean;
		};
		data: {
			peers: PairedPeerPayload[];
		};
		events: {
			onOpenChange: (open: boolean) => void;
			onSelect: (peer: PairedPeerPayload) => void;
		};
	};
</script>

<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import AcerolaDialog from '$lib/components/acerola-dialog/acerola-dialog.svelte';
	import { shortId } from '$lib/utils/connection-code.utils';
	import MonitorIcon from '@lucide/svelte/icons/monitor';

	let { state, data, events }: AcerolaPeerPickerProps = $props();

	// No teardown da story (Storybook + vitest browser mode), o efeito reativo deste
	// template roda mais uma vez com `events` já undefined antes do componente ser
	// destruído de fato — sem o fallback aqui isso vaza como unhandled error e derruba
	// a suíte mesmo com todos os asserts passando (mesmo padrão de
	// acerola-network-my-device-card.svelte).
	let safeEvents = $derived(events ?? { onOpenChange: () => {}, onSelect: () => {} });
	let safeData = $derived(data ?? { peers: [] });
</script>

<AcerolaDialog
	state={{ open: state?.open ?? false }}
	data={{ title: m['pages.network.peer_picker.title']() }}
	events={{ onOpenChange: safeEvents.onOpenChange }}
	ui={{
		contentClass:
			'w-full max-w-[calc(100vw-2rem)] sm:max-w-sm border-border/60 bg-background/95 backdrop-blur-xl shadow-2xl p-6 rounded-3xl overflow-hidden'
	}}
>
	{#if safeData.peers.length === 0}
		<p class="p-4 text-center text-sm text-muted-foreground">
			{m['pages.network.peers.empty']()}
		</p>
	{:else}
		<div class="flex flex-col gap-1 py-1">
			{#each safeData.peers as peer (peer.peerId)}
				<button
					type="button"
					class="flex items-center gap-3 rounded-xl px-3 py-2.5 text-left transition-colors hover:bg-muted"
					onclick={() => safeEvents.onSelect(peer)}
				>
					<MonitorIcon size={18} class="shrink-0 text-muted-foreground" />
					<span class="truncate text-sm font-medium text-foreground">
						{peer.deviceName ?? shortId(peer.peerId)}
					</span>
				</button>
			{/each}
		</div>
	{/if}
</AcerolaDialog>
