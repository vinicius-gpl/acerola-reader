<script lang="ts">
	import CheckIcon from '@lucide/svelte/icons/check';
	import ClipboardCopyIcon from '@lucide/svelte/icons/clipboard-copy';
	import XIcon from '@lucide/svelte/icons/x';
	import { Button } from '$lib/components/ui/button/index.js';

	let {
		raw,
		url,
		label = 'Copy page as Markdown',
		copiedLabel = 'Copied',
		failedLabel = "Couldn't copy"
	}: {
		raw: string;
		url: string;
		label?: string;
		copiedLabel?: string;
		failedLabel?: string;
	} = $props();

	type Status = 'idle' | 'copied' | 'failed';
	let status = $state<Status>('idle');
	let resetTimeout: ReturnType<typeof setTimeout> | undefined;

	function resetAfter(next: Status, ms: number) {
		status = next;
		clearTimeout(resetTimeout);
		resetTimeout = setTimeout(() => (status = 'idle'), ms);
	}

	async function copy() {
		try {
			await navigator.clipboard.writeText(`<!-- URL: ${url} -->\n\n${raw}`);
			resetAfter('copied', 2000);
		} catch (error) {
			// navigator.clipboard.writeText rejeita em contexto não seguro ou sem
			// permissão — sem esse catch a promise falha em silêncio e o botão
			// nunca dá nenhum feedback de erro pro usuário.
			console.error('Failed to copy page as Markdown', error);
			resetAfter('failed', 3000);
		}
	}

	const views: Record<Status, { icon: typeof ClipboardCopyIcon; text: string; class: string }> =
		$derived({
			idle: { icon: ClipboardCopyIcon, text: label, class: 'text-muted-foreground' },
			copied: { icon: CheckIcon, text: copiedLabel, class: 'text-primary' },
			failed: { icon: XIcon, text: failedLabel, class: 'text-destructive' }
		});
	const view = $derived(views[status]);
</script>

<Button
	variant="outline"
	size="icon-lg"
	class={view.class}
	onclick={copy}
	aria-label={view.text}
	title={view.text}
>
	{@const Icon = view.icon}
	<Icon class="size-5" />
</Button>
