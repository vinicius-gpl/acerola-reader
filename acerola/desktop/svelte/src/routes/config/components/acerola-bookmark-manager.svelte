<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { useBookmarks } from '$lib/hooks/store/use-bookmarks.svelte';
	import { m } from '$lib/paraglide/messages';
	import { autoAnimateList } from '$lib/utils/auto-animate.utils';

	import Trash2Icon from '@lucide/svelte/icons/trash-2';
	import PlusIcon from '@lucide/svelte/icons/plus';

	const bookmarkStore = useBookmarks();

	const CATEGORY_COLORS = [
		0xfff44336, 0xffe91e63, 0xff9c27b0, 0xff673ab7, 0xff3f51b5, 0xff2196f3, 0xff03a9f4, 0xff00bcd4,
		0xff009688, 0xff4caf50, 0xff8bc34a, 0xffcddc39, 0xffffeb3b, 0xffffc107, 0xffff9800, 0xffff5722,
		0xff795548, 0xff9e9e9e, 0xff607d8b
	];

	let newBookmarkName = $state('');
	let newBookmarkColor = $state<number>(CATEGORY_COLORS[0]);

	onMount(() => {
		bookmarkStore.loadBookmarks();
	});
</script>

<div class="space-y-4">
	<div class="grid gap-4">
		<div class="rounded-2xl border border-border/40 bg-card/50 p-6 backdrop-blur-sm">
			<p class="mb-4 text-sm text-muted-foreground">{m['pages.config.bookmarks.desc']()}</p>

			<div class="mb-6 space-y-6">
				<!-- Row 1: Name and Button -->
				<div class="flex items-end gap-4">
					<div class="flex-1 space-y-1">
						<label for="bookmarkName" class="text-xs font-semibold"
							>{m['pages.config.bookmarks.name']()}</label
						>
						<Input
							id="bookmarkName"
							placeholder={m['pages.config.bookmarks.name']()}
							bind:value={newBookmarkName}
							class="h-10 bg-background text-foreground"
						/>
					</div>
					<Button
						disabled={!newBookmarkName.trim() || bookmarkStore.isLoading}
						onclick={async () => {
							await bookmarkStore.createBookmark(newBookmarkName, newBookmarkColor);
							newBookmarkName = '';
						}}
						class="h-10 gap-2 px-6"
					>
						<PlusIcon size={16} />
						{m['pages.config.bookmarks.create']()}
					</Button>
				</div>

				<!-- Row 2: Colors -->
				<div class="space-y-2">
					<span class="block text-xs font-semibold">{m['pages.config.bookmarks.color']()}</span>
					<div class="flex flex-wrap gap-2">
						{#each CATEGORY_COLORS as hexColor}
							{@const hexLabel = '#' + (hexColor & 0xffffff).toString(16).padStart(6, '0')}
							<button
								type="button"
								class="relative h-8 w-8 cursor-pointer rounded-full transition-transform hover:scale-110"
								style="background-color: {hexLabel}"
								onclick={() => (newBookmarkColor = hexColor)}
								aria-label={m['pages.config.bookmarks.color_option']({ hex: hexLabel })}
								aria-pressed={newBookmarkColor === hexColor}
							>
								{#if newBookmarkColor === hexColor}
									<div
										class="absolute inset-0 rounded-full border-2 border-primary ring-2 ring-background"
									></div>
								{/if}
							</button>
						{/each}
					</div>
				</div>
			</div>

			<div class="space-y-2" use:autoAnimateList>
				{#if bookmarkStore.bookmarks.length === 0}
					<div
						class="rounded-xl border border-dashed border-border/40 p-8 text-center text-sm text-muted-foreground"
					>
						{m['pages.config.bookmarks.empty']()}
					</div>
				{:else}
					{#each bookmarkStore.bookmarks as bookmark (bookmark.id)}
						<div
							class="flex items-center justify-between rounded-xl border border-border/40 bg-background/50 p-3 transition-colors hover:bg-muted/50"
						>
							<div class="flex items-center gap-3">
								<div
									class="h-6 w-6 rounded-full shadow-inner"
									style="background-color: #{(bookmark.color & 0xffffff)
										.toString(16)
										.padStart(6, '0')}"
								></div>
								<span class="font-medium">{bookmark.name}</span>
							</div>
							<Button
								variant="ghost"
								size="icon"
								class="h-8 w-8 text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
								onclick={() => bookmarkStore.deleteBookmark(bookmark.id)}
							>
								<Trash2Icon size={16} />
							</Button>
						</div>
					{/each}
				{/if}
			</div>
		</div>
	</div>
</div>
