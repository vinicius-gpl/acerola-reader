<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { Button } from '$lib/components/ui/button';
	import CheckCircleIcon from '@lucide/svelte/icons/check-circle';
	import CheckCircle2Icon from '@lucide/svelte/icons/check-circle-2';
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import SparklesIcon from '@lucide/svelte/icons/sparkles';

	let { onPrev, onComplete } = $props<{ onPrev: () => void; onComplete: () => void }>();
</script>

<div class="flex h-full w-full items-center justify-center p-6 sm:p-8">
	<div class="flex max-w-lg flex-col items-center space-y-8 text-center">
		<!-- Celebration Glowing Icon -->
		<div class="relative flex items-center justify-center">
			<div
				class="absolute -inset-3 rounded-full bg-linear-to-tr from-emerald-500/30 to-teal-500/30 opacity-70 blur-xl filter animate-pulse"
			></div>
			<div
				class="relative flex size-24 items-center justify-center rounded-3xl bg-linear-to-br from-emerald-500 via-emerald-600 to-teal-700 text-white shadow-[0_8px_32px_rgba(16,185,129,0.35)] ring-1 ring-white/20"
			>
				<CheckCircleIcon size={48} />
			</div>
		</div>

		<!-- Title & Subtitle -->
		<div class="space-y-2">
			<h1 class="text-4xl font-black tracking-tight text-foreground sm:text-5xl">
				{m['onboarding.complete.title']()}
			</h1>
			<p class="mx-auto max-w-md text-base leading-relaxed text-muted-foreground sm:text-lg">
				{m['onboarding.complete.desc']()}
			</p>
		</div>

		<!-- Checklist Card (VibePrompts Onboarding Checklist pattern) -->
		<div
			class="w-full space-y-3 rounded-3xl border border-border/50 bg-card/60 p-5 text-left shadow-xs backdrop-blur-md"
		>
			<div class="flex items-center gap-2 border-b border-border/30 pb-3 text-xs font-semibold text-foreground">
				<SparklesIcon size={14} class="text-emerald-500" />
				<span>{m['onboarding.complete.checklist_title']()}</span>
			</div>

			<div class="space-y-2 text-xs">
				<div class="flex items-center gap-2.5 text-foreground/90">
					<CheckCircle2Icon size={15} class="text-emerald-500 shrink-0" />
					<span>{m['onboarding.complete.checklist_item_1']()}</span>
				</div>
				<div class="flex items-center gap-2.5 text-foreground/90">
					<CheckCircle2Icon size={15} class="text-emerald-500 shrink-0" />
					<span>{m['onboarding.complete.checklist_item_2']()}</span>
				</div>
				<div class="flex items-center gap-2.5 text-foreground/90">
					<CheckCircle2Icon size={15} class="text-emerald-500 shrink-0" />
					<span>{m['onboarding.complete.checklist_item_3']()}</span>
				</div>
			</div>
		</div>

		<!-- Note Box -->
		<div class="w-full rounded-2xl border border-border/40 bg-card/40 p-4 text-left backdrop-blur-sm">
			<p class="text-xs text-muted-foreground leading-relaxed">{m['onboarding.complete.note']()}</p>
		</div>

		<!-- Action Controls -->
		<div class="flex w-full items-center justify-between pt-2">
			<Button onclick={onPrev} variant="outline" class="rounded-xl px-6">
				{m['onboarding.back']()}
			</Button>
			<Button
				onclick={onComplete}
				size="lg"
				class="group relative inline-flex items-center gap-2 rounded-2xl bg-primary px-8 py-6 text-base font-semibold text-primary-foreground shadow-[0_0_24px_rgba(var(--color-primary-rgb),0.35)] transition-all duration-300 hover:scale-[1.02] hover:shadow-[0_0_32px_rgba(var(--color-primary-rgb),0.55)] active:scale-[0.98]"
			>
				<span>{m['onboarding.complete.button']()}</span>
				<ArrowRightIcon size={18} class="transition-transform duration-200 group-hover:translate-x-1" />
			</Button>
		</div>
	</div>
</div>
