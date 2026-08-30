<script module lang="ts">
	import type { ReaderMode } from '../hooks/use-reader-zoom.svelte';

	export type ReaderCommandPaletteProps = {
		data: {
			zoomMode: boolean;
		};
		state: {
			open: boolean;
			value: string;
			readingMode: ReaderMode;
		};
		events: {
			onOpenChange: (open: boolean) => void;
			onValueChange: (value: string) => void;
			onReadingModeChange: (mode: ReaderMode) => void;
			onToggleZoomMode: () => void;
			onZoomIn: () => void;
			onZoomOut: () => void;
			onResetZoom: () => void;
		};
	};
</script>

<script lang="ts">
	import AcerolaCommand from '$lib/components/acerola-command/acerola-command.svelte';
	import * as Command from '$lib/components/ui/command';
	import { m } from '$lib/paraglide/messages';
	import Columns2 from '@lucide/svelte/icons/columns-2';
	import Rows2 from '@lucide/svelte/icons/rows-2';
	import ScrollText from '@lucide/svelte/icons/scroll-text';
	import ZoomIn from '@lucide/svelte/icons/zoom-in';
	import ZoomOut from '@lucide/svelte/icons/zoom-out';

	let { data, events, state }: ReaderCommandPaletteProps = $props();

	function closeCommand() {
		events.onOpenChange(false);
		events.onValueChange('');
	}

	function runCommand(action: () => void) {
		action();
		closeCommand();
	}
</script>

{#if state.open}
	<div class="absolute inset-0 z-40">
		<button
			type="button"
			aria-label={m['pages.reader.actions.close_commands']()}
			class="absolute inset-0 bg-base/20 backdrop-blur-[2px]"
			onclick={closeCommand}
		></button>

		<div
			class="absolute top-1/2 left-1/2 w-[min(34rem,calc(100vw-2rem))] -translate-x-1/2 -translate-y-1/2 overflow-hidden rounded-2xl border border-surface/50 bg-base/95 shadow-2xl shadow-crust/60"
		>
			<AcerolaCommand
				state={{ value: state.value }}
				events={{ onValueChange: events.onValueChange }}
			>
				{#snippet children()}
					<Command.Input placeholder={m['pages.reader.command.placeholder']()} autofocus />

					<Command.List class="p-1">
						<Command.Group heading={m['pages.reader.command.zoom_group']()}>
							<Command.Item
								value={m['pages.reader.command.search.toggle_zoom']()}
								class="cursor-pointer"
								onSelect={() => runCommand(events.onToggleZoomMode)}
							>
								<ZoomIn size={16} />
								<span>
									{data.zoomMode
										? m['pages.reader.command.mode.disable_zoom']()
										: m['pages.reader.command.mode.enable_zoom']()}
								</span>
								<Command.Shortcut>Z</Command.Shortcut>
							</Command.Item>

							<Command.Item
								value={m['pages.reader.command.search.zoom_in']()}
								class="cursor-pointer"
								onSelect={() => runCommand(events.onZoomIn)}
							>
								<ZoomIn size={16} />
								<span>{m['pages.reader.command.zoom.in']()}</span>
								<Command.Shortcut>Ctrl +</Command.Shortcut>
							</Command.Item>

							<Command.Item
								value={m['pages.reader.command.search.zoom_out']()}
								class="cursor-pointer"
								onSelect={() => runCommand(events.onZoomOut)}
							>
								<ZoomOut size={16} />
								<span>{m['pages.reader.command.zoom.out']()}</span>
								<Command.Shortcut>Ctrl -</Command.Shortcut>
							</Command.Item>

							<Command.Item
								value={m['pages.reader.command.search.reset_zoom']()}
								class="cursor-pointer"
								onSelect={() => runCommand(events.onResetZoom)}
							>
								<ZoomOut size={16} />
								<span>{m['pages.reader.command.zoom.reset']()}</span>
								<Command.Shortcut>Ctrl 0</Command.Shortcut>
							</Command.Item>
						</Command.Group>

						<Command.Group heading={m['pages.reader.command.reading_group']()}>
							<Command.Item
								value={m['pages.reader.command.search.vertical']()}
								class="cursor-pointer"
								onSelect={() => runCommand(() => events.onReadingModeChange('vertical'))}
							>
								<Rows2 size={16} />
								<span>{m['pages.reader.modes.vertical.full']()}</span>
							</Command.Item>

							<Command.Item
								value={m['pages.reader.command.search.horizontal']()}
								class="cursor-pointer"
								onSelect={() => runCommand(() => events.onReadingModeChange('horizontal'))}
							>
								<Columns2 size={16} />
								<span>{m['pages.reader.modes.horizontal.full']()}</span>
							</Command.Item>

							<Command.Item
								value={m['pages.reader.command.search.webtoon']()}
								class="cursor-pointer"
								onSelect={() => runCommand(() => events.onReadingModeChange('webtoon'))}
							>
								<ScrollText size={16} />
								<span>{m['pages.reader.modes.webtoon']()}</span>
							</Command.Item>
						</Command.Group>
					</Command.List>
				{/snippet}
			</AcerolaCommand>
		</div>
	</div>
{/if}
