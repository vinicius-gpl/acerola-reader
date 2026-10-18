<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { Button } from '$lib/components/ui/button';
	import { cn } from '$lib/utils/cn.utils';
	import FileTextIcon from '@lucide/svelte/icons/file-text';
	import FileArchiveIcon from '@lucide/svelte/icons/file-archive';
	import LayersIcon from '@lucide/svelte/icons/layers';
	import SparklesIcon from '@lucide/svelte/icons/sparkles';
	import CheckCircle2Icon from '@lucide/svelte/icons/check-circle-2';

	let { onNext, onPrev } = $props<{ onNext: () => void; onPrev: () => void }>();

	const formats = $derived([
		{
			extension: 'CBZ',
			name: m['onboarding.formats.cbz.title'](),
			badge: m['onboarding.formats.cbz.badge'](),
			badgeClass: 'bg-primary/10 text-primary border-primary/30',
			cardClass: 'border-primary/40 bg-gradient-to-b from-primary/5 to-card/50 shadow-md',
			icon: FileArchiveIcon,
			iconColor: 'text-primary bg-primary/10',
			description: m['onboarding.formats.cbz.desc'](),
			features: [m['onboarding.formats.cbz.feature_1'](), m['onboarding.formats.cbz.feature_2']()]
		},
		{
			extension: 'CBR',
			name: m['onboarding.formats.cbr.title'](),
			badge: m['onboarding.formats.cbr.badge'](),
			badgeClass: 'bg-chart-3/10 text-chart-3 border-chart-3/30',
			cardClass: 'border-border/40 bg-card/50 hover:border-chart-3/40 transition-all',
			icon: LayersIcon,
			iconColor: 'text-chart-3 bg-chart-3/10',
			description: m['onboarding.formats.cbr.desc'](),
			features: [m['onboarding.formats.cbr.feature_1'](), m['onboarding.formats.cbr.feature_2']()]
		},
		{
			extension: 'PDF',
			name: m['onboarding.formats.pdf.title'](),
			badge: m['onboarding.formats.pdf.badge'](),
			badgeClass: 'bg-chart-1/10 text-chart-1 border-chart-1/30',
			cardClass: 'border-border/40 bg-card/50 hover:border-chart-1/40 transition-all',
			icon: FileTextIcon,
			iconColor: 'text-chart-1 bg-chart-1/10',
			description: m['onboarding.formats.pdf.desc'](),
			features: [m['onboarding.formats.pdf.feature_1'](), m['onboarding.formats.pdf.feature_2']()]
		}
	]);
</script>

<div class="flex h-full w-full items-center justify-center overflow-y-auto p-6 sm:p-8">
	<div class="w-full max-w-3xl animate-in space-y-8 duration-300 fade-in-50">
		<!-- Header -->
		<div class="space-y-2 text-center">
			<div
				class="mb-2 inline-flex size-14 items-center justify-center rounded-2xl bg-chart-5/10 text-chart-5 shadow-inner"
			>
				<FileArchiveIcon size={30} />
			</div>
			<h1 class="text-3xl font-bold tracking-tight text-foreground">
				{m['onboarding.formats.title']()}
			</h1>
			<p class="mx-auto max-w-md text-sm text-muted-foreground">{m['onboarding.formats.desc']()}</p>
		</div>

		<!-- Grid Formatos -->
		<div class="grid grid-cols-1 gap-4.5 md:grid-cols-3">
			{#each formats as item}
				<div
					class={cn(
						'group relative flex flex-col justify-between rounded-3xl border p-6 backdrop-blur-md transition-all duration-300 hover:-translate-y-1.5 hover:shadow-2xl',
						'shadow-[0_1px_0_rgba(255,255,255,0.1)_inset,0_4px_16px_rgba(0,0,0,0.04)] dark:shadow-[0_1px_0_rgba(255,255,255,0.06)_inset,0_8px_24px_rgba(0,0,0,0.4)]',
						item.cardClass
					)}
				>
					<div>
						<!-- Top row: Icon & Badge -->
						<div class="mb-5 flex items-center justify-between">
							<div
								class={cn(
									'flex size-12 items-center justify-center rounded-2xl p-3 shadow-inner',
									item.iconColor
								)}
							>
								<item.icon size={24} />
							</div>
							<span
								class={cn(
									'rounded-full border px-3 py-1 text-[11px] font-bold tracking-wider uppercase shadow-xs',
									item.badgeClass
								)}
							>
								{item.badge}
							</span>
						</div>

						<!-- Title & Extension -->
						<h3 class="mb-1 text-3xl font-black tracking-tight text-foreground">
							{item.extension}
						</h3>
						<p class="mb-3 text-xs font-semibold tracking-wider text-muted-foreground uppercase">
							{item.name}
						</p>

						<!-- Description -->
						<p class="text-xs leading-relaxed text-muted-foreground/90">{item.description}</p>
					</div>

					<!-- Features list -->
					<div class="mt-5 space-y-2 border-t border-border/20 pt-4">
						{#each item.features as feat}
							<div class="flex items-center gap-2 text-xs font-medium text-foreground/80">
								<CheckCircle2Icon size={14} class="shrink-0 text-primary" />
								<span>{feat}</span>
							</div>
						{/each}
					</div>
				</div>
			{/each}
		</div>

		<!-- Bottom callout -->
		<div
			class="flex items-center gap-3.5 rounded-2xl border border-border/50 bg-card/40 p-4.5 text-xs text-muted-foreground backdrop-blur-md shadow-xs"
		>
			<div
				class="flex size-9 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-primary shadow-inner"
			>
				<SparklesIcon size={20} />
			</div>
			<p class="leading-normal">
				<strong class="font-semibold text-foreground"
					>{m['onboarding.formats.sync_note.title']()}</strong
				>
				{m['onboarding.formats.sync_note.content']()}
			</p>
		</div>

		<!-- Actions -->
		<div class="flex items-center justify-between pt-2">
			<Button onclick={onPrev} variant="outline" class="rounded-xl px-6">
				{m['onboarding.back']()}
			</Button>
			<Button
				onclick={onNext}
				class="rounded-xl px-8 font-semibold shadow-md transition-all hover:shadow-lg"
			>
				{m['onboarding.next']()}
			</Button>
		</div>
	</div>
</div>
