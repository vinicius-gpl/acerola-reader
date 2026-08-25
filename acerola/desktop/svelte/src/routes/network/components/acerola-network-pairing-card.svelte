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
	import CopyIcon from '@lucide/svelte/icons/copy';
	import CheckIcon from '@lucide/svelte/icons/check';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import ShieldAlertIcon from '@lucide/svelte/icons/shield-alert';
	import AcerolaButton from '$lib/components/acerola-button/acerola-button.svelte';
	import { m } from '$lib/paraglide/messages';

	let { data }: NetworkPairingCardProps = $props();

	let qrDataUrl = $state<string | undefined>(undefined);
	let qrError = $state(false);
	let showRawCode = $state(false);
	// Feedback visual do botão de copiar — além do toast, o próprio ícone vira um check
	// por um instante, pra quem não pegar o toast a tempo perceber que a cópia aconteceu.
	let codeJustCopied = $state(false);

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

	async function copyCode() {
		if (!data?.code) return;
		await navigator.clipboard.writeText(data.code);
		toast.success(m['pages.network.copied']());
		codeJustCopied = true;
		setTimeout(() => (codeJustCopied = false), 2000);
	}
</script>

<section class="space-y-4">
	<div
		class="flex items-center gap-3 text-xs font-bold tracking-widest text-muted-foreground uppercase"
	>
		<QrCodeIcon size={16} />
		{m['pages.network.pairing.title']()}
	</div>

	<div class="rounded-2xl border border-border/40 bg-card/50 p-6 backdrop-blur-sm">
		<p class="mb-4 text-sm text-muted-foreground">{m['pages.network.pairing.desc']()}</p>

		<div class="flex flex-col items-center gap-4">
			<div class="flex h-72 w-72 shrink-0 items-center justify-center rounded-xl bg-white p-3">
				{#if qrDataUrl}
					<img src={qrDataUrl} alt={m['pages.network.pairing.desc']()} class="h-full w-full" />
				{:else if qrError}
					<span class="px-4 text-center text-xs text-destructive">
						{m['pages.network.pairing.qr_error']()}
					</span>
				{:else}
					<span class="text-xs text-muted-foreground">{m['pages.network.pairing.loading']()}</span
					>
				{/if}
			</div>

			<AcerolaButton
				events={{ onClick: copyCode }}
				ui={{ variant: 'outline', size: 'sm', class: 'gap-2', disabled: !data?.code }}
			>
				{#if codeJustCopied}
					<CheckIcon size={14} class="text-chart-3" />
				{:else}
					<CopyIcon size={14} />
				{/if}
				{m['pages.network.pairing.copy']()}
			</AcerolaButton>

			<button
				type="button"
				class="flex items-center gap-1 text-xs text-muted-foreground transition-colors hover:text-foreground"
				onclick={() => (showRawCode = !showRawCode)}
				aria-expanded={showRawCode}
				aria-controls="pairing-raw-code"
			>
				{showRawCode ? m['pages.network.pairing.hide_code']() : m['pages.network.pairing.show_code']()}
				<ChevronDownIcon size={12} class="transition-transform {showRawCode ? 'rotate-180' : ''}" />
			</button>

			{#if showRawCode}
				<textarea
					id="pairing-raw-code"
					readonly
					aria-label={m['pages.network.pairing.title']()}
					value={data?.code ?? ''}
					class="h-24 w-full resize-none rounded-xl border border-border/60 bg-background/70 p-3 font-mono text-xs break-all text-muted-foreground"
				></textarea>
			{/if}
		</div>

		<div class="mt-6 flex items-start gap-2 rounded-xl border border-chart-4/20 bg-chart-4/5 p-3">
			<ShieldAlertIcon size={16} class="mt-0.5 shrink-0 text-chart-4" />
			<p class="text-xs text-muted-foreground">{m['pages.network.pairing.security_note']()}</p>
		</div>
	</div>
</section>
