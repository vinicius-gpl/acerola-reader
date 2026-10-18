<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { Button } from '$lib/components/ui/button';
	import BookOpenIcon from '@lucide/svelte/icons/book-open';
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import ZapIcon from '@lucide/svelte/icons/zap';
	import Share2Icon from '@lucide/svelte/icons/share-2';
	import ShieldCheckIcon from '@lucide/svelte/icons/shield-check';

	let { onNext } = $props<{ onNext: () => void }>();
</script>

<div class="flex h-full w-full items-center justify-center p-6 sm:p-8">
	<div class="flex max-w-lg flex-col items-center space-y-8 text-center">
		<!-- Glowing Brand Icon (Halo effect) -->
		<div class="relative flex items-center justify-center">
			<div
				class="absolute -inset-3 rounded-full bg-linear-to-tr from-primary/30 to-chart-1/30 opacity-70 blur-xl filter animate-pulse"
			></div>
			<div
				class="relative flex size-24 items-center justify-center rounded-3xl bg-linear-to-br from-primary via-primary/95 to-chart-1 text-primary-foreground shadow-[0_8px_32px_rgba(var(--color-primary-rgb),0.35)] ring-1 ring-white/20"
			>
				<BookOpenIcon size={44} strokeWidth={2.2} />
			</div>
		</div>

		<!-- Title & Subtitle -->
		<div class="space-y-3">
			<h1 class="text-4xl font-black tracking-tight text-foreground sm:text-5xl">
				Acerola
			</h1>
			<p class="mx-auto max-w-md text-base leading-relaxed text-muted-foreground sm:text-lg">
				{m['onboarding.welcome']()}
			</p>
		</div>

		<!-- Feature Highlight Chips -->
		<div class="grid w-full grid-cols-1 gap-2.5 sm:grid-cols-3">
			<div
				class="flex items-center gap-2.5 rounded-2xl border border-border/50 bg-card/60 p-3 text-left shadow-xs backdrop-blur-sm transition-all hover:border-primary/40"
			>
				<div class="flex size-8 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-primary">
					<ZapIcon size={16} />
				</div>
				<div class="min-w-0">
					<p class="text-xs font-semibold text-foreground">{m['onboarding.welcome_badge_fast_title']()}</p>
					<p class="truncate text-[10px] text-muted-foreground">{m['onboarding.welcome_badge_fast_desc']()}</p>
				</div>
			</div>

			<div
				class="flex items-center gap-2.5 rounded-2xl border border-border/50 bg-card/60 p-3 text-left shadow-xs backdrop-blur-sm transition-all hover:border-chart-3/40"
			>
				<div class="flex size-8 shrink-0 items-center justify-center rounded-xl bg-chart-3/10 text-chart-3">
					<Share2Icon size={16} />
				</div>
				<div class="min-w-0">
					<p class="text-xs font-semibold text-foreground">{m['onboarding.welcome_badge_p2p_title']()}</p>
					<p class="truncate text-[10px] text-muted-foreground">{m['onboarding.welcome_badge_p2p_desc']()}</p>
				</div>
			</div>

			<div
				class="flex items-center gap-2.5 rounded-2xl border border-border/50 bg-card/60 p-3 text-left shadow-xs backdrop-blur-sm transition-all hover:border-chart-2/40"
			>
				<div class="flex size-8 shrink-0 items-center justify-center rounded-xl bg-chart-2/10 text-chart-2">
					<ShieldCheckIcon size={16} />
				</div>
				<div class="min-w-0">
					<p class="text-xs font-semibold text-foreground">{m['onboarding.welcome_badge_local_title']()}</p>
					<p class="truncate text-[10px] text-muted-foreground">{m['onboarding.welcome_badge_local_desc']()}</p>
				</div>
			</div>
		</div>

		<!-- Action Button -->
		<Button
			onclick={onNext}
			size="lg"
			class="group relative inline-flex items-center gap-2 rounded-2xl bg-primary px-9 py-6 text-base font-semibold text-primary-foreground shadow-[0_0_24px_rgba(var(--color-primary-rgb),0.35)] transition-all duration-300 hover:scale-[1.02] hover:shadow-[0_0_32px_rgba(var(--color-primary-rgb),0.55)] active:scale-[0.98]"
		>
			<span>{m['onboarding.start']()}</span>
			<ArrowRightIcon size={18} class="transition-transform duration-200 group-hover:translate-x-1" />
		</Button>
	</div>
</div>
