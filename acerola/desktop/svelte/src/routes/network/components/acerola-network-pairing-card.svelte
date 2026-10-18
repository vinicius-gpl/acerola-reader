<script module lang="ts">
	export type NetworkPairingCardProps = {
		data: {
			code: string | undefined;
		};
	};
</script>

<script lang="ts">
	import { toast } from 'svelte-sonner';
	import { error } from '@tauri-apps/plugin-log';
	import QRCode from 'qrcode';
	import QrCodeIcon from '@lucide/svelte/icons/qr-code';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import ShieldAlertIcon from '@lucide/svelte/icons/shield-alert';
	import AcerolaCopyButton from '$lib/components/acerola-button/acerola-copy-button.svelte';
	import { m } from '$lib/paraglide/messages';
	import { cn } from '$lib/utils/cn.utils';

	let { data }: NetworkPairingCardProps = $props();

	let qrDataUrl = $state<string | undefined>(undefined);
	let qrError = $state(false);
	let showRawCode = $state(false);

	$effect(() => {
		// No teardown da story (Storybook + vitest browser mode), este efeito roda mais
		// uma vez com `data` já undefined antes do componente ser destruído de fato —
		// sem o optional chaining aqui isso vaza como unhandled error e derruba a suíte
		// mesmo com todos os asserts passando.
		const code = data?.code;
		if (!code) {
			qrDataUrl = undefined;
			qrError = false;
			return;
		}
		QRCode.toDataURL(code, { margin: 4, width: 280, errorCorrectionLevel: 'L' })
			.then((url) => {
				qrDataUrl = url;
				qrError = false;
			})
			.catch((err) => {
				error(`failed to generate QR code: ${err}`);
				qrDataUrl = undefined;
				qrError = true;
			});
	});

	function handleCopied() {
		toast.success(m['pages.network.copied']());
	}
</script>

<section class="space-y-4">
	<div
		class="flex items-center gap-3 text-xs font-bold tracking-widest text-muted-foreground uppercase"
	>
		<QrCodeIcon size={16} />
		{m['pages.network.pairing.title']()}
	</div>

	<div class="relative overflow-hidden rounded-3xl border border-border/50 bg-card/60 p-6 shadow-sm backdrop-blur-md">
		<div class="pointer-events-none absolute -right-16 -top-16 size-48 rounded-full bg-primary/5 blur-3xl"></div>

		<p class="mb-5 text-sm text-muted-foreground">{m['pages.network.pairing.desc']()}</p>

		<div class="flex flex-col items-center gap-5">
			<div class="group relative flex h-72 w-72 shrink-0 items-center justify-center rounded-2xl border border-border/40 bg-white p-3 shadow-md transition-transform duration-300 hover:scale-[1.01]">
				{#if qrDataUrl}
					<img src={qrDataUrl} alt={m['pages.network.pairing.desc']()} class="h-full w-full rounded-lg" />
				{:else if qrError}
					<span class="px-4 text-center text-xs font-medium text-destructive">
						{m['pages.network.pairing.qr_error']()}
					</span>
				{:else}
					<span class="text-xs font-medium text-muted-foreground">{m['pages.network.pairing.loading']()}</span>
				{/if}
			</div>

			<AcerolaCopyButton
				value={data?.code}
				label={m['pages.network.pairing.copy']()}
				copiedLabel={m['pages.network.copied']()}
				disabled={!data?.code}
				onCopy={handleCopied}
				variant="outline"
				size="sm"
				class="gap-2 rounded-xl px-4 py-2 shadow-xs transition-all hover:border-primary/50"
			/>

			<button
				type="button"
				class="flex items-center gap-1.5 text-xs font-medium text-muted-foreground transition-colors hover:text-foreground"
				onclick={() => (showRawCode = !showRawCode)}
				aria-expanded={showRawCode}
				aria-controls="pairing-raw-code"
			>
				{showRawCode
					? m['pages.network.pairing.hide_code']()
					: m['pages.network.pairing.show_code']()}
				<ChevronDownIcon
					size={12}
					class={cn('transition-transform duration-200', showRawCode && 'rotate-180')}
				/>
			</button>

			{#if showRawCode}
				<textarea
					id="pairing-raw-code"
					readonly
					aria-label={m['pages.network.pairing.title']()}
					value={data?.code ?? ''}
					class="h-24 w-full resize-none rounded-xl border border-border/60 bg-background/80 p-3 font-mono text-xs break-all text-muted-foreground shadow-inner focus:outline-hidden"
				></textarea>
			{/if}
		</div>

		<div class="mt-6 flex items-start gap-3 rounded-2xl border border-chart-4/20 bg-chart-4/10 p-3.5 backdrop-blur-xs">
			<ShieldAlertIcon size={16} class="mt-0.5 shrink-0 text-chart-4" />
			<p class="text-xs leading-relaxed text-muted-foreground">{m['pages.network.pairing.security_note']()}</p>
		</div>
	</div>
</section>
