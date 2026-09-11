import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { toast } from 'svelte-sonner';
import * as m from '$lib/paraglide/messages';
import { LIBRARY_EVENTS } from '$lib/contracts/library/library.events';
import { type DirectoryScanCommand } from '$lib/contracts/library/library.commands';
import type { ErrorPayload } from '$lib/contracts/shared/shared.payloads';
import { resolveErrorMessage } from '$lib/contracts/errors/errors.i18n';
import { notificationStore } from '$lib/components/acerola-notification/acerola-notification.svelte';

const { notify, pop } = notificationStore;

export function useLibraryScanner(
	command: DirectoryScanCommand,
	getPath: () => string | undefined
) {
	let progressId: number | undefined;
	// Toast espelhando o mesmo progresso — atualizado in-place (mesmo `id`, estilo
	// `toastAsync`) em vez de empilhar um toast por evento. Sem isso, um scan que roda em
	// background só aparecia no sininho de notificações: quem não estivesse de olho nele
	// não tinha como saber que o scan tinha começado/terminado/falhado.
	let toastId: string | number | undefined;
	let scanning = $state(false);

	async function start() {
		const path = getPath();

		if (!path) {
			toast.error(m['hooks.comic_scanner.no_folder']());
			return;
		}

		scanning = true;

		const unlistenProgress = await listen(LIBRARY_EVENTS.scanProgress, () => {
			if (progressId === undefined) {
				progressId = notify.info(m['hooks.comic_scanner.in_progress'](), { duration: 0 });
			}
			if (toastId === undefined) {
				toastId = toast.loading(m['hooks.comic_scanner.in_progress']());
			}
		});

		const unlistenConverting = await listen<string>(LIBRARY_EVENTS.scanConverting, (event) => {
			const msg = event.payload || m['hooks.comic_scanner.converting']();

			if (progressId !== undefined) {
				pop(progressId);
			}

			progressId = notify.info(msg, { duration: 0 });
			toastId = toast.loading(msg, { id: toastId });
		});

		const unlisten = await listen(LIBRARY_EVENTS.scanComplete, () => {
			if (progressId !== undefined) {
				pop(progressId);
				progressId = undefined;
			}

			notify.success(m['hooks.comic_scanner.success'](), { duration: 0 });
			toast.success(m['hooks.comic_scanner.success'](), { id: toastId });
			toastId = undefined;

			scanning = false;

			unlisten();
			unlistenErr();
			unlistenProgress();
			unlistenConverting();
		});

		const unlistenErr = await listen<ErrorPayload>(LIBRARY_EVENTS.scanError, (it) => {
			if (progressId !== undefined) {
				pop(progressId);
				progressId = undefined;
			}

			const description = resolveErrorMessage(it.payload);
			notify.error(m['hooks.comic_scanner.error.title'](), {
				description,
				duration: 0
			});
			toast.error(m['hooks.comic_scanner.error.title'](), {
				description,
				id: toastId
			});
			toastId = undefined;

			scanning = false;

			unlisten();
			unlistenErr();
			unlistenProgress();
			unlistenConverting();
		});

		await invoke(command, { path });
	}

	return {
		start,
		get scanning() {
			return scanning;
		}
	};
}
