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
	let progressPercent = $derived(((currentStepIndex) / (STEPS.length - 1)) * 100);
</script>

<div class="relative flex h-full w-full flex-col overflow-hidden bg-background">
	<!-- Ambient Glow Background -->
	<div
		class="pointer-events-none absolute inset-0 opacity-30 transition-opacity duration-700 ease-out"
		style="background: radial-gradient(circle 800px at 50% -10%, var(--color-primary) 0%, transparent 70%);"
	></div>

	<!-- Top Stepper Navigation (VibePrompts inspired) -->
	<div class="relative z-10 border-b border-border/40 bg-background/80 px-6 py-4 backdrop-blur-xl">
		<div class="mx-auto flex max-w-4xl flex-col gap-3">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-2">
					<span class="text-xs font-semibold tracking-wider text-muted-foreground uppercase">
						{m['onboarding.setup_title']()}
					</span>
				</div>
				<span class="text-xs font-medium text-muted-foreground">
					{currentStepIndex + 1} / {STEPS.length} ({Math.round(progressPercent)}%)
				</span>
			</div>

			<!-- Stepper progress pills -->
			<div class="relative flex items-center justify-between gap-2">
				{#each STEPS as step, it}
					{@const isCompleted = it < currentStepIndex}
					{@const isCurrent = it === currentStepIndex}
					<div
						class="flex flex-1 items-center gap-2 rounded-xl border px-3 py-2 text-xs font-medium transition-all duration-300 {isCurrent
							? 'border-primary/50 bg-primary/10 text-primary shadow-[0_0_16px_rgba(var(--color-primary-rgb),0.15)] ring-1 ring-primary/30'
							: isCompleted
								? 'border-border/60 bg-muted/40 text-foreground'
								: 'border-transparent bg-transparent text-muted-foreground/60'}"
					>
						<span
							class="flex size-5 shrink-0 items-center justify-center rounded-full text-[11px] font-bold transition-all duration-300 {isCurrent
								? 'bg-primary text-primary-foreground shadow-xs'
								: isCompleted
									? 'bg-primary/20 text-primary'
									: 'bg-muted/60 text-muted-foreground'}"
						>
							{it + 1}
						</span>
						<span class="truncate font-medium max-sm:hidden">{step.label()}</span>
					</div>
				{/each}
			</div>

			<!-- Progress track bar -->
			<div class="h-1 w-full overflow-hidden rounded-full bg-muted/40">
				<div
					class="h-full bg-linear-to-r from-primary/80 via-primary to-chart-1 transition-all duration-500 ease-out"
					style="width: {progressPercent}%;"
				></div>
			</div>
		</div>
	</div>

	<!-- Step Content Container with dynamic fly animation -->
	<div class="relative flex-1 overflow-hidden">
		{#key currentStepIndex}
			<div
				class="absolute inset-0 overflow-y-auto"
				in:fly={{ duration: 300, x: direction * 100 }}
			>
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
