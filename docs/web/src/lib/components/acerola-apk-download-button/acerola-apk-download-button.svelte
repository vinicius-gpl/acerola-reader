<script lang="ts">
	import { onMount } from 'svelte';
	import AndroidIcon from '$lib/assets/icons/android.svg?component';
	import TriangleAlertIcon from '@lucide/svelte/icons/triangle-alert';
	import { APK_URL, GITHUB_RELEASES_URL } from '$lib/constants/site';

	let {
		label = 'Get the',
		fallbackLabel = 'Unavailable — try',
		fallbackTitle = 'Direct download is temporarily unavailable — opening the latest GitHub release instead.'
	}: { label?: string; fallbackLabel?: string; fallbackTitle?: string } = $props();

	// Otimista: renderiza o link normal de cara (inclusive no SSR, onde este check nunca
	// roda) e só troca pro fallback se a checagem no cliente confirmar que o objeto não
	// está acessível — evita bloquear o botão atrás de uma chamada de rede toda vez.
	let available = $state(true);

	onMount(async () => {
		try {
			const response = await fetch('/api/apk-status');
			const data: { available: boolean } = await response.json();
			available = data.available;
		} catch {
			// Se a própria checagem falhar (ex.: offline), mantém o link direto — um
			// problema no check não deveria impedir o usuário de tentar o download real.
		}
	});
</script>

{#if available}
	<a
		href={APK_URL}
		class="group/badge flex w-full items-center justify-center gap-3 rounded-2xl bg-neutral-900 px-5 py-3 text-white ring-1 ring-white/10 transition-all hover:scale-[1.02] hover:shadow-lg active:scale-[0.98]"
	>
		<AndroidIcon class="h-6 w-auto shrink-0" />
		<span class="flex flex-col items-start leading-tight">
			<span class="text-[10px] font-medium tracking-wide text-white/60 uppercase">
				{label}
			</span>
			<span class="text-sm font-bold text-white">APK</span>
		</span>
	</a>
{:else}
	<a
		href={GITHUB_RELEASES_URL}
		target="_blank"
		rel="noopener noreferrer"
		title={fallbackTitle}
		class="group/badge flex w-full items-center justify-center gap-3 rounded-2xl bg-neutral-900 px-5 py-3 text-white ring-1 ring-amber-500/50 transition-all hover:scale-[1.02] hover:shadow-lg active:scale-[0.98]"
	>
		<TriangleAlertIcon class="h-5 w-5 shrink-0 text-amber-400" />
		<span class="flex flex-col items-start leading-tight">
			<span class="text-[10px] font-medium tracking-wide text-amber-400/80 uppercase">
				{fallbackLabel}
			</span>
			<span class="text-sm font-bold text-white">GitHub Releases</span>
		</span>
	</a>
{/if}
