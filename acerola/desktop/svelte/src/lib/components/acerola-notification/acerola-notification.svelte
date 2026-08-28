<script lang="ts" module>
	import { createNotifications } from '$lib/state/notification.svelte';
	import { tv } from 'tailwind-variants';

	import AlertTriangleIcon from '@lucide/svelte/icons/alert-triangle';
	import BellIcon from '@lucide/svelte/icons/bell';
	import CheckCircle2Icon from '@lucide/svelte/icons/check-circle-2';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import InfoIcon from '@lucide/svelte/icons/info';
	import XIcon from '@lucide/svelte/icons/x';
	import XCircleIcon from '@lucide/svelte/icons/x-circle';

	function keysOf<T extends object>(obj: T): Array<keyof T> {
		return Object.keys(obj) as Array<keyof T>;
	}

	export const notifyVariants = tv({
		variants: {
			variant: {
				warning: 'text-yellow-500 bg-yellow-500/5 border-yellow-500',
				success: 'text-green-500 bg-green-500/5 border-green-500',
				error: 'text-destructive bg-destructive/5 border-destructive',
				info: 'text-blue-500 bg-blue-500/5 border-blue-500'
			}
		}
	});

	const variantIcon = {
		warning: AlertTriangleIcon,
		success: CheckCircle2Icon,
		error: XCircleIcon,
		info: InfoIcon
	};

	export const notificationStore = createNotifications(keysOf(notifyVariants.variants.variant));
</script>

<script lang="ts">
	import AcerolaButtonIcon from '$lib/components/acerola-button/acerola-button-icon.svelte';
	import AcerolaPopover from '$lib/components/acerola-popover/acerola-popover.svelte';
	import { Button } from '$lib/components/ui/button';
	import { fly, fade } from 'svelte/transition';
	import { flip } from 'svelte/animate';
	import { cubicOut } from 'svelte/easing';

	import { m } from '$lib/paraglide/messages';

	const { notifications, pop, clearAll } = notificationStore;

	let scrollContainer = $state<HTMLDivElement | null>(null);
	let showScrollIndicator = $state(false);

	function checkScroll() {
		if (!scrollContainer) return;
		const { scrollTop, scrollHeight, clientHeight } = scrollContainer;
		// Show indicator if there is more to scroll and we aren't at the bottom
		showScrollIndicator =
			scrollHeight > clientHeight && scrollTop + clientHeight < scrollHeight - 10;
	}

	function scrollToBottom() {
		scrollContainer?.scrollTo({
			top: scrollContainer.scrollHeight,
			behavior: 'smooth'
		});
	}

	$effect(() => {
		// Re-check when notifications change
		notifications.length;
		checkScroll();
	});
</script>

<AcerolaPopover
	ui={{
		align: 'end',
		sideOffset: 8,
		collisionPadding: 16,
		contentClass:
			'p-0 w-84 overflow-hidden rounded-2xl border-border/40 bg-card/95 backdrop-blur-md shadow-2xl'
	}}
>
	{#snippet trigger()}
		<AcerolaButtonIcon ui={{ class: 'relative hover:bg-muted/80' }}>
			<BellIcon size={20} />
			{#if notifications.length > 0}
				<span
					class="absolute -top-1 -right-1 flex h-[18px] min-w-[18px] items-center justify-center rounded-full border-2 border-background bg-primary px-1 text-[10px] font-bold text-primary-foreground shadow-sm"
				>
					{notifications.length}
				</span>
			{/if}
		</AcerolaButtonIcon>
	{/snippet}

	{#snippet content()}
		<div class="flex flex-col">
			<div
				class="flex items-center justify-between border-b border-border/40 bg-muted/20 px-4 py-3"
			>
				<div class="flex items-center gap-2">
					<h3 class="text-xs font-bold tracking-widest text-foreground/80 uppercase">
						{m['components.notification.title']()}
					</h3>
					{#if notifications.length > 0}
						<span
							class="rounded-full bg-primary/10 px-1.5 py-0.5 text-[10px] font-bold text-primary"
						>
							{notifications.length}
						</span>
					{/if}
				</div>
				{#if notifications.length > 0}
					<Button
						variant="ghost"
						size="xs"
						class="h-7 px-2 text-[10px] font-semibold hover:bg-destructive/10 hover:text-destructive"
						onclick={clearAll}
					>
						{m['components.notification.clear.all']()}
					</Button>
				{/if}
			</div>

			<div class="relative flex flex-col">
				<div
					bind:this={scrollContainer}
					onscroll={checkScroll}
					class="flex max-h-[400px] [scrollbar-width:none] flex-col gap-1 overflow-y-auto p-2 [&::-webkit-scrollbar]:hidden"
				>
					{#if notifications.length === 0}
						<div class="flex flex-col items-center justify-center py-12 text-center">
							<div class="relative mb-4">
								<div class="absolute inset-0 animate-ping rounded-full bg-primary/10"></div>
								<div class="relative rounded-full bg-muted/30 p-4">
									<BellIcon size={28} class="text-muted-foreground/30" />
								</div>
							</div>
							<p class="text-sm font-semibold text-foreground/60">
								{m['components.notification.empty']()}
							</p>
							<p class="mt-1 text-xs text-muted-foreground/60">Suas novidades aparecerão aqui</p>
						</div>
					{:else}
						{#each notifications as notify (notify.id)}
							{@const Icon = variantIcon[notify.variant]}

							<div
								animate:flip={{ duration: 200 }}
								in:fly={{ y: 10, duration: 250, easing: cubicOut }}
								out:fade={{ duration: 150 }}
								class="group relative flex flex-col gap-2 rounded-xl border border-transparent p-3 transition-all hover:bg-muted/40"
							>
								<!-- Indicator Bar -->
								<div
									class="absolute top-3 bottom-3 left-0 w-1 rounded-full {notifyVariants({
										variant: notify.variant
									})
										.split(' ')
										.pop()}"
								></div>

								<div class="flex items-start justify-between gap-2 pl-2">
									<div class="flex items-start gap-3">
										<div
											class="mt-0.5 rounded-lg p-1.5 {notifyVariants({ variant: notify.variant })
												.split(' ')
												.slice(0, 2)
												.join(' ')}"
										>
											<Icon size={14} />
										</div>
										<div class="min-w-0 flex-1">
											<p class="text-sm leading-snug font-semibold text-foreground">
												{notify.message}
											</p>
											{#if notify.description}
												<p
													class="mt-1 line-clamp-2 text-xs leading-relaxed text-muted-foreground/80"
												>
													{notify.description}
												</p>
											{/if}
										</div>
									</div>

									<button
										class="flex size-6 shrink-0 items-center justify-center rounded-lg text-muted-foreground/40 opacity-0 transition-all group-hover:opacity-100 hover:bg-muted hover:text-foreground"
										onclick={() => pop(notify.id)}
									>
										<XIcon size={14} />
									</button>
								</div>

								{#if notify.action}
									<div class="pl-11">
										<Button
											variant="secondary"
											size="xs"
											class="h-7 rounded-lg border border-border/50 px-3 text-[10px] font-bold shadow-sm"
											onclick={() => {
												notify.action?.onClick();
												pop(notify.id);
											}}
										>
											{notify.action.label}
										</Button>
									</div>
								{/if}
							</div>
						{/each}
					{/if}
				</div>

				<!-- Scroll Indicators -->
				{#if showScrollIndicator}
					<div
						transition:fade={{ duration: 200 }}
						class="pointer-events-none absolute right-0 bottom-0 left-0 h-16 bg-gradient-to-t from-card/80 to-transparent"
					></div>
					<button
						onclick={scrollToBottom}
						transition:fly={{ y: 10, duration: 200 }}
						class="absolute bottom-4 left-1/2 flex size-8 -translate-x-1/2 items-center justify-center rounded-full border border-border/40 bg-card/90 text-foreground shadow-lg backdrop-blur-sm transition-all hover:scale-110 hover:bg-accent active:scale-95"
						aria-label={m['components.notification.scroll_to_bottom']()}
					>
						<ChevronDownIcon size={16} />
					</button>
				{/if}
			</div>
		</div>
	{/snippet}
</AcerolaPopover>
