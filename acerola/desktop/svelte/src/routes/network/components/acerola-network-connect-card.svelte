<script module lang="ts">
	export type NetworkConnectCardProps = {
		data: {
			connecting: boolean;
		};
		events: {
			onConnect: (code: string) => Promise<void>;
		};
	};
</script>

<script lang="ts">
	import { toast } from 'svelte-sonner';
	import PlugIcon from '@lucide/svelte/icons/plug';
	import AlertCircleIcon from '@lucide/svelte/icons/alert-circle';
	import AcerolaButton from '$lib/components/acerola-button/acerola-button.svelte';
	import AcerolaInput from '$lib/components/acerola-input/acerola-input.svelte';
	import { m } from '$lib/paraglide/messages';
	import { InvalidConnectionCodeError } from '$lib/utils/connection-code.utils';

	let { data, events }: NetworkConnectCardProps = $props();

	let pasteValue = $state('');
	let connectError = $state<string | undefined>(undefined);

	async function handleConnect() {
		connectError = undefined;
		try {
			await events.onConnect(pasteValue);
			pasteValue = '';
			toast.success(m['pages.network.connect.success']());
		} catch (err) {
			connectError =
				err instanceof InvalidConnectionCodeError
					? m['pages.network.connect.invalid_code']()
					: m['pages.network.connect.error']();
		}
	}
</script>

<section class="space-y-4">
	<div
		class="flex items-center gap-3 text-xs font-bold tracking-widest text-muted-foreground uppercase"
	>
		<PlugIcon size={16} />
		{m['pages.network.connect.title']()}
	</div>

	<div class="rounded-2xl border border-border/40 bg-card/50 p-6 backdrop-blur-sm">
		<div class="flex flex-col gap-3 sm:flex-row">
			<AcerolaInput
				state={{ value: pasteValue }}
				events={{ onValueChange: (value) => (pasteValue = value) }}
				ui={{
					placeholder: m['pages.network.connect.placeholder'](),
					class: 'h-10 flex-1 bg-background font-mono text-xs'
				}}
			/>
			<AcerolaButton
				events={{ onClick: handleConnect }}
				ui={{ class: 'h-10 gap-2 px-6', disabled: !pasteValue.trim() || data.connecting }}
			>
				<PlugIcon size={16} />
				{m['pages.network.connect.button']()}
			</AcerolaButton>
		</div>

		{#if connectError}
			<p class="mt-3 flex items-center gap-2 text-sm text-destructive">
				<AlertCircleIcon size={14} />
				{connectError}
			</p>
		{/if}
	</div>
</section>
