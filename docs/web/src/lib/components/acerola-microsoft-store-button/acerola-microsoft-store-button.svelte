<script lang="ts">
	import MicrosoftStoreIcon from '$lib/assets/icons/microsoft-store.svg?component';

	let { label = 'Get it from' }: { label?: string } = $props();

	const STORE_URL = 'https://apps.microsoft.com/store/detail/9N9SL3P7F8GC?cid=DevShareMCLPCS';
</script>

<a
	href={STORE_URL}
	target="_blank"
	rel="noopener noreferrer"
	class="group/badge flex w-full items-center justify-center gap-3 rounded-2xl bg-neutral-900 px-5 py-3 text-white ring-1 ring-white/10 transition-all hover:scale-[1.02] hover:shadow-lg active:scale-[0.98]"
>
	<MicrosoftStoreIcon class="h-7 w-7 shrink-0" />
	<span class="flex flex-col items-start leading-tight">
		<span class="text-[10px] font-medium tracking-wide text-white/60 uppercase">
			{label}
		</span>
		<span class="text-sm font-bold text-white">Microsoft Store</span>
	</span>
</a>
