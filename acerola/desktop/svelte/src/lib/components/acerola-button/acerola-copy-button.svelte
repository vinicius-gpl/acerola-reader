<script module lang="ts">
	export type AcerolaCopyButtonProps = {
		value?: string | null;
		label?: string;
		copiedLabel?: string;
		variant?: 'default' | 'outline' | 'ghost' | 'secondary';
		size?: 'default' | 'sm' | 'icon' | 'icon-sm';
		class?: string;
		disabled?: boolean;
		onCopy?: () => void;
	};
</script>

<script lang="ts">
	import CopyIcon from '@lucide/svelte/icons/copy';
	import CheckIcon from '@lucide/svelte/icons/check';
	import { Button } from '$lib/components/ui/button';
	import { cn } from '$lib/utils/cn.utils';
	import { m } from '$lib/paraglide/messages';

	let {
		value,
		label,
		copiedLabel,
		variant = 'outline',
		size = 'sm',
		class: className,
		disabled = false,
		onCopy
	}: AcerolaCopyButtonProps = $props();

	let defaultLabel = $derived(label ?? m['components.copy_button.copy']());
	let defaultCopiedLabel = $derived(copiedLabel ?? m['components.copy_button.copied']());

	let copied = $state(false);
	let timeoutId: ReturnType<typeof setTimeout> | undefined;

	async function handleCopy(e: MouseEvent) {
		e.stopPropagation();
		if (!value || copied) return;

		try {
			if (navigator.clipboard?.writeText) {
				await navigator.clipboard.writeText(value);
			} else {
				const textArea = document.createElement('textarea');
				textArea.value = value;
				document.body.appendChild(textArea);
				textArea.select();
				document.execCommand('copy');
				document.body.removeChild(textArea);
			}
			copied = true;
			onCopy?.();
			clearTimeout(timeoutId);
			timeoutId = setTimeout(() => {
				copied = false;
			}, 2000);
		} catch (err) {
			console.error('Failed to copy', err);
		}
	}
</script>

<Button
	type="button"
	{variant}
	size={size === 'icon-sm' ? 'icon' : size}
	disabled={disabled || !value}
	onclick={handleCopy}
	class={cn(
		'relative transition-all duration-200 active:scale-95',
		size === 'icon-sm' && 'size-8 rounded-lg',
		copied && 'border-emerald-500/50 bg-emerald-500/10 text-emerald-600 dark:text-emerald-400',
		className
	)}
	aria-label={copied ? defaultCopiedLabel : defaultLabel}
>
	{#if copied}
		<CheckIcon size={14} class="animate-in zoom-in-50 duration-200" strokeWidth={2.5} />
		{#if copiedLabel}
			<span class="ml-1.5 text-xs font-semibold">{defaultCopiedLabel}</span>
		{:else if label}
			<span class="ml-1.5 text-xs font-semibold">{defaultLabel}</span>
		{/if}
	{:else}
		<CopyIcon size={14} class="transition-transform group-hover:scale-105" />
		{#if label}
			<span class="ml-1.5 text-xs font-semibold">{defaultLabel}</span>
		{/if}
	{/if}
</Button>
