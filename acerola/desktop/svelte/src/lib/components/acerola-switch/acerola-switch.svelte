<script module lang="ts">
	import { Switch as SwitchPrimitive } from 'bits-ui';
	import type { WithoutChildrenOrChild } from '$lib/utils/cn.utils';

	export type AcerolaSwitchProps = {
		state?: {
			checked?: boolean;
		};
		events?: {
			onCheckedChange?: (checked: boolean) => void;
		};
		ui?: Omit<WithoutChildrenOrChild<SwitchPrimitive.RootProps>, 'checked' | 'class'> & {
			class?: string;
			size?: 'sm' | 'default';
		};
	};
</script>

<script lang="ts">
	import { cn } from '$lib/utils/cn.utils';

	let { events, state: control, ui }: AcerolaSwitchProps = $props();

	let localChecked = $state(false);
	const isControlled = $derived(control?.checked !== undefined);
	const currentChecked = $derived(isControlled ? Boolean(control?.checked) : localChecked);

	const restProps = $derived.by(() => {
		const { class: _class, size: _size, ...rest } = ui ?? {};
		return rest;
	});

	function handleCheckedChange(nextChecked: boolean) {
		if (!isControlled) {
			localChecked = nextChecked;
		}
		events?.onCheckedChange?.(nextChecked);
	}
</script>

<SwitchPrimitive.Root
	checked={currentChecked}
	onCheckedChange={handleCheckedChange}
	data-slot="switch"
	data-size={ui?.size ?? 'default'}
	class={cn(
		'peer group/switch relative inline-flex shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-primary data-[state=unchecked]:bg-input',
		(ui?.size ?? 'default') === 'default' ? 'h-8 w-14' : 'h-6 w-11',
		ui?.class
	)}
	{...restProps}
>
	<SwitchPrimitive.Thumb
		class={cn(
			'pointer-events-none block rounded-full bg-background shadow-lg ring-0 transition-transform data-[state=checked]:translate-x-full data-[state=unchecked]:translate-x-0',
			(ui?.size ?? 'default') === 'default' ? 'h-6 w-6' : 'h-5 w-5'
		)}
	/>
</SwitchPrimitive.Root>
