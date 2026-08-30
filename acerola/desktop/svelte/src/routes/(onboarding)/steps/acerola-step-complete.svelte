<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { Button } from '$lib/components/ui/button';
	import CheckCircleIcon from '@lucide/svelte/icons/check-circle';

	let { onPrev, onComplete } = $props<{ onPrev: () => void; onComplete: () => void }>();
</script>

<div class="flex h-full w-full items-center justify-center">
	<div class="max-w-md space-y-8 text-center">
		<div class="flex justify-center">
			<div class="flex size-20 items-center justify-center rounded-full bg-primary/10 text-primary">
				<CheckCircleIcon size={48} />
			</div>
		</div>

		<div class="space-y-2">
			<h1 class="text-4xl font-bold text-foreground">{m['onboarding.complete.title']()}</h1>
			<p class="text-lg text-muted-foreground">{m['onboarding.complete.desc']()}</p>
		</div>

		<div class="rounded-2xl border border-border/40 bg-card/50 p-4 text-left backdrop-blur-sm">
			<p class="text-sm text-muted-foreground">{m['onboarding.complete.note']()}</p>
		</div>

		<div class="flex justify-between">
			<Button onclick={onPrev} variant="outline" class="rounded-xl">
				{m['onboarding.back']()}
			</Button>
			<Button onclick={onComplete} size="lg" class="rounded-xl px-8">
				{m['onboarding.complete.button']()}
			</Button>
		</div>
	</div>
</div>
