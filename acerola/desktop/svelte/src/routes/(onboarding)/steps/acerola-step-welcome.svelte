<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { Button } from '$lib/components/ui/button';
	import BookOpenIcon from '@lucide/svelte/icons/book-open';

	let { onNext } = $props<{ onNext: () => void }>();
</script>

<div class="flex h-full w-full items-center justify-center">
	<div class="max-w-md space-y-8 text-center">
		<div class="flex justify-center">
			<div
				class="flex size-20 items-center justify-center rounded-2xl bg-primary text-primary-foreground"
			>
				<BookOpenIcon size={48} />
			</div>
		</div>

		<div class="space-y-2">
			<h1 class="text-4xl font-bold text-foreground">Acerola</h1>
			<p class="text-lg text-muted-foreground">{m['onboarding.welcome']()}</p>
		</div>

		<Button onclick={onNext} size="lg" class="rounded-xl px-8">
			{m['onboarding.start']()}
		</Button>
	</div>
</div>
