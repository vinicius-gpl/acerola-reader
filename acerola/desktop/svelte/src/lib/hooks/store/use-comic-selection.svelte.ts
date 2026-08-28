import { error } from '@tauri-apps/plugin-log';

export type ComicId = string | number;

let selectedIds = $state<Set<ComicId>>(new Set());
let isSelectionMode = $state(false);

// ONLY FOR TESTING
export function _resetComicSelectionState() {
	selectedIds = new Set();
	isSelectionMode = false;
}

/**
 * Hook para gerenciar seleção múltipla de quadrinhos.
 * Permite selecionar um ou vários quadrinhos para ações em batch.
 */
export function useComicSelection() {
	function toggleSelection(id: ComicId): void {
		const next = new Set(selectedIds);
		if (next.has(id)) {
			next.delete(id);
		} else {
			next.add(id);
		}
		selectedIds = next;
		isSelectionMode = selectedIds.size > 0;
	}

	function selectSingle(id: ComicId): void {
		selectedIds = new Set([id]);
		isSelectionMode = true;
	}

	function selectAll(ids: ComicId[]): void {
		selectedIds = new Set(ids);
		isSelectionMode = ids.length > 0;
	}

	function deselectAll(): void {
		selectedIds = new Set();
		isSelectionMode = false;
	}

	function enterSelectionMode(): void {
		isSelectionMode = true;
	}

	function exitSelectionMode(): void {
		selectedIds = new Set();
		isSelectionMode = false;
	}

	function isSelected(id: ComicId): boolean {
		return selectedIds.has(id);
	}

	return {
		toggleSelection,
		selectSingle,
		selectAll,
		deselectAll,
		enterSelectionMode,
		exitSelectionMode,
		isSelected,
		get selectedIds() {
			return selectedIds;
		},
		get selectedCount() {
			return selectedIds.size;
		},
		get isSelectionMode() {
			return isSelectionMode;
		},
		get selectedIdsArray() {
			return Array.from(selectedIds);
		}
	};
}
