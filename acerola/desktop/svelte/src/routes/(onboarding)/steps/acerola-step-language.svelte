<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { Button } from '$lib/components/ui/button';
	import * as Select from '$lib/components/ui/select';
	import LanguagesIcon from '@lucide/svelte/icons/languages';
	import { locales } from '$lib/paraglide/runtime';
	import { setLocale, getLocale } from '$lib/paraglide/runtime';
	import type { Locale } from '$lib/paraglide/runtime';

	let selectedLocale = $state<Locale>(getLocale());

	const localeOptions = locales.map((locale) => ({
		value: locale,
		label:
			locale === 'pt-br'
				? 'Português (BR)'
				: locale === 'en'
					? 'English'
					: (locale as string).toUpperCase()
	}));

	function handleLocaleChange(locale: string) {
		selectedLocale = locale as Locale;
		setLocale(selectedLocale);
	}

	let { onNext, onPrev } = $props<{ onNext: () => void; onPrev: () => void }>();
</script>

<div class="flex h-full w-full items-center justify-center p-8">
	<div class="w-full max-w-xl space-y-8">
		<div class="flex items-center justify-center gap-3">
			<LanguagesIcon class="text-chart-3" size={32} />
			<h1 class="text-3xl font-bold text-foreground">{m['onboarding.language.title']()}</h1>
		</div>

		<p class="text-center text-muted-foreground">{m['onboarding.language.desc']()}</p>

		<div class="rounded-2xl border border-border/40 bg-card/50 p-6 backdrop-blur-sm">
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

		<div class="flex justify-between">
			<Button onclick={onPrev} variant="outline" class="rounded-xl">
				{m['onboarding.back']()}
			</Button>
			<Button onclick={onNext} class="rounded-xl">
				{m['onboarding.next']()}
			</Button>
		</div>
	</div>
</div>
