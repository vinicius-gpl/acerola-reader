<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { Button } from '$lib/components/ui/button';
	import * as Select from '$lib/components/ui/select';
	import LanguagesIcon from '@lucide/svelte/icons/languages';
	import { locales } from '$lib/paraglide/runtime';
	import { setLocale, getLocale } from '$lib/paraglide/runtime';
	import type { Locale } from '$lib/paraglide/runtime';

	import CheckCircle2Icon from '@lucide/svelte/icons/check-circle-2';
	import GlobeIcon from '@lucide/svelte/icons/globe';

	let selectedLocale = $state<Locale>(getLocale());

	const localeOptions = $derived(
		locales.map((locale) => ({
			value: locale,
			label:
				locale === 'pt-br'
					? 'Português (BR)'
					: locale === 'en'
						? 'English'
						: (locale as string).toUpperCase(),
			flag: locale === 'pt-br' ? '🇧🇷' : '🇺🇸',
			nativeName: locale === 'pt-br' ? 'Português' : 'English',
			sub:
				locale === 'pt-br'
					? m['onboarding.language.country_br']()
					: m['onboarding.language.country_us']()
		}))
	);

	function handleLocaleChange(locale: string) {
		selectedLocale = locale as Locale;
		setLocale(selectedLocale);
	}

	let { onNext, onPrev } = $props<{ onNext: () => void; onPrev: () => void }>();
</script>

<div class="flex h-full w-full items-center justify-center p-6 sm:p-8">
	<div class="w-full max-w-xl space-y-8 text-center sm:text-left">
		<!-- Header -->
		<div class="flex flex-col items-center gap-3 sm:flex-row sm:items-start">
			<div
				class="flex size-14 items-center justify-center rounded-2xl bg-chart-3/10 text-chart-3 shadow-inner"
			>
				<LanguagesIcon size={28} />
			</div>
			<div>
				<h1 class="text-3xl font-bold tracking-tight text-foreground sm:text-4xl">
					{m['onboarding.language.title']()}
				</h1>
				<p class="mt-1 text-sm text-muted-foreground">{m['onboarding.language.desc']()}</p>
			</div>
		</div>

		<!-- Visual Language Cards (VibePrompts Role/Selection Picker inspired) -->
		<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
			{#each localeOptions as opt}
				{@const isSelected = selectedLocale === opt.value}
				<button
					type="button"
					onclick={() => handleLocaleChange(opt.value)}
					class="group relative flex cursor-pointer items-start justify-between rounded-2xl border p-5 text-left backdrop-blur-sm transition-all duration-200 {isSelected
						? 'border-primary/60 bg-linear-to-b from-primary/10 via-card/70 to-card/90 shadow-[0_0_24px_rgba(var(--color-primary-rgb),0.2)] ring-2 ring-primary/40'
						: 'border-border/40 bg-card/40 hover:border-border hover:bg-card/70'}"
				>
					<div class="space-y-2">
						<span class="text-3xl">{opt.flag}</span>
						<div>
							<h3 class="text-lg font-bold text-foreground">{opt.nativeName}</h3>
							<p class="text-xs text-muted-foreground">{opt.sub}</p>
						</div>
					</div>

					<div
						class="flex size-6 items-center justify-center rounded-full transition-all {isSelected
							? 'bg-primary text-primary-foreground shadow-xs'
							: 'border border-border/80 bg-background/50 text-transparent'}"
					>
						<CheckCircle2Icon size={16} strokeWidth={2.5} />
					</div>
				</button>
			{/each}
		</div>

		<!-- Dropdown Alternative -->
		<div class="rounded-2xl border border-border/40 bg-card/30 p-4 backdrop-blur-sm">
			<div class="mb-2 flex items-center gap-2 text-xs font-medium text-muted-foreground">
				<GlobeIcon size={14} />
				<span>{m['onboarding.language.quick_select']()}</span>
			</div>
			<Select.Root type="single" value={selectedLocale} onValueChange={handleLocaleChange}>
				<Select.Trigger class="w-full rounded-xl">
					{localeOptions.find((l) => l.value === selectedLocale)?.label || selectedLocale}
				</Select.Trigger>
				<Select.Content>
					{#each localeOptions as option}
						<Select.Item value={option.value} label={option.label}>
							{option.label}
						</Select.Item>
					{/each}
				</Select.Content>
			</Select.Root>
		</div>

		<!-- Action Controls -->
		<div class="flex items-center justify-between pt-2">
			<Button onclick={onPrev} variant="outline" class="rounded-xl px-6">
				{m['onboarding.back']()}
			</Button>
			<Button onclick={onNext} class="rounded-xl px-8 font-semibold shadow-md transition-all hover:shadow-lg">
				{m['onboarding.next']()}
			</Button>
		</div>
	</div>
</div>
