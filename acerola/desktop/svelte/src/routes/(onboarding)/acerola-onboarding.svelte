<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { useOnboarding } from '$lib/hooks/onboarding/use-onboarding.svelte';
	import { useLibraryScanner } from '$lib/hooks/store/use-comic-scanner.svelte';
	import { DIRECTORY_SCAN_COMMANDS } from '$lib/contracts/library/library.commands';
	import { useSelectFolder } from '$lib/hooks/store/use-select-folder.svelte';
	import StepWelcome from './steps/acerola-step-welcome.svelte';
	import StepLanguage from './steps/acerola-step-language.svelte';
	import StepFormats from './steps/acerola-step-formats.svelte';
	import StepSettings from './steps/acerola-step-settings.svelte';
	import StepComplete from './steps/acerola-step-complete.svelte';
	import { goto } from '$app/navigation';
	import { fly } from 'svelte/transition';

	const onboarding = useOnboarding();
	const folder = useSelectFolder();

	const scanner = useLibraryScanner(
		DIRECTORY_SCAN_COMMANDS.incrementalScan,
		() => folder.folderPath
	);

	const STEPS = [
		{ label: m['onboarding.step.welcome'] },
		{ label: m['onboarding.step.language'] },
		{ label: m['onboarding.step.formats'] },
		{ label: m['onboarding.step.settings'] },
		{ label: m['onboarding.step.complete'] }
	];

	let direction = $state(1);
	let previousStep = $state(0);

	$effect(() => {
		if (onboarding.currentStep !== previousStep) {
			direction = onboarding.currentStep > previousStep ? 1 : -1;
			previousStep = onboarding.currentStep;
		}
	});

	function handleNext() {
		direction = 1;
		onboarding.nextStep();
	}

	function handlePrev() {
		direction = -1;
		onboarding.prevStep();
	}

	async function handleComplete() {
		await onboarding.complete();
		if (folder.folderPath) {
			scanner.start();
		}
		await goto('/home');
	}

	let currentStepIndex = $derived(onboarding.currentStep);
</script>

<div class="flex h-full w-full flex-col">
	<div class="border-b border-border/30 bg-background/95 px-8 py-4">
		<div class="flex items-center justify-center gap-2">
			{#each STEPS as step, it}
				<div
					class="flex items-center gap-2 rounded-full px-4 py-2 text-xs font-medium transition-all duration-300
						{it === onboarding.currentStep
						? 'bg-primary text-primary-foreground'
						: it < onboarding.currentStep
							? 'bg-muted text-muted-foreground'
							: 'bg-transparent text-muted-foreground/50'}"
				>
					<span
						class="flex size-5 items-center justify-center rounded-full
						{it <= onboarding.currentStep ? 'bg-background/20' : 'bg-muted/30'}"
					>
						{it + 1}
					</span>
					<span class="hidden sm:inline">{step.label()}</span>
				</div>
				{#if it < STEPS.length - 1}
					<div class="h-px w-8 bg-border/30"></div>
				{/if}
			{/each}
		</div>
	</div>

	<div class="relative flex-1 overflow-hidden">
		{#key currentStepIndex}
			<div class="absolute inset-0 overflow-y-auto" in:fly={{ duration: 300, x: direction * 100 }}>
				{#if currentStepIndex === 0}
					<StepWelcome onNext={handleNext} />
				{:else if currentStepIndex === 1}
					<StepLanguage onNext={handleNext} onPrev={handlePrev} />
				{:else if currentStepIndex === 2}
					<StepFormats onNext={handleNext} onPrev={handlePrev} />
				{:else if currentStepIndex === 3}
					<StepSettings onNext={handleNext} onPrev={handlePrev} />
				{:else if currentStepIndex === 4}
					<StepComplete onPrev={handlePrev} onComplete={handleComplete} />
				{/if}
			</div>
		{/key}
	</div>
</div>
