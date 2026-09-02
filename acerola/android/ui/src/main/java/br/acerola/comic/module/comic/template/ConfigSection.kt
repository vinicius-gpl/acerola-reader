package br.acerola.comic.module.comic.template

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.LayersClear
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.state.SyncActionVisualState
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.Dialog
import br.acerola.comic.common.ux.component.DialogButton
import br.acerola.comic.common.ux.component.HeroButton
import br.acerola.comic.config.preference.types.VolumeViewType
import br.acerola.comic.module.comic.Comic
import br.acerola.comic.module.comic.component.ComicCategorySelector
import br.acerola.comic.module.comic.component.ComicExternalSyncToggle
import br.acerola.comic.module.comic.component.PaginationPreference
import br.acerola.comic.module.comic.component.SyncMangaArchive
import br.acerola.comic.module.comic.component.SyncMetadata
import br.acerola.comic.module.comic.component.SyncWithPeerAction
import br.acerola.comic.module.comic.component.VolumeStylePreference
import br.acerola.comic.module.comic.state.ComicAction
import br.acerola.comic.module.comic.state.ComicSyncAction
import br.acerola.comic.module.comic.state.ComicUiState
import br.acerola.comic.module.main.Main
import br.acerola.comic.module.main.common.component.PeerPickerSheet
import br.acerola.comic.module.main.sync.state.PairedPeer
import br.acerola.comic.service.SyncDirection
import br.acerola.comic.ui.R

private val itemModifier = Modifier.padding(horizontal = 16.dp)

@Composable
private fun SectionHeader(title: String) {
    Text(
        text = title.uppercase(),
        style = MaterialTheme.typography.labelMedium,
        fontWeight = FontWeight.Bold,
        color = MaterialTheme.colorScheme.secondary,
        modifier = Modifier.padding(start = 24.dp, top = 24.dp, bottom = 8.dp),
    )
}

fun Comic.Template.configSection(
    scope: LazyListScope,
    uiState: ComicUiState,
    getSyncActionVisualState: (ComicSyncAction) -> SyncActionVisualState = { SyncActionVisualState.IDLE },
    pairedPeers: List<PairedPeer> = emptyList(),
    syncWithPeerState: SyncActionVisualState = SyncActionVisualState.IDLE,
    onLoadPairedPeers: () -> Unit = {},
    onAction: (ComicAction) -> Unit,
    onSyncAction: (ComicSyncAction) -> Unit,
) {
    scope.item { Spacer(modifier = Modifier.height(16.dp)) }

    // NOTE: Configurações de Exibição
    scope.item {
        SectionHeader(stringResource(id = R.string.title_settings_display_config))
    }

    scope.item {
        Comic.Component.PaginationPreference(
            selected = uiState.selectedChapterPerPage,
            onSelect = { onAction(ComicAction.UpdatePageSize(it)) },
            modifier = itemModifier,
        )
    }

    scope.item { Spacer(modifier = Modifier.height(8.dp)) }

    if (uiState.hasVolumeStructure) {
        scope.item {
            Comic.Component.VolumeStylePreference(
                selected = uiState.volumeViewMode,
                onSelect = { mode -> onAction(ComicAction.UpdateVolumeView(mode)) },
                modifier = itemModifier,
            )
        }
    }

    scope.item {
        HorizontalDivider(
            modifier =
                Modifier
                    .padding(horizontal = 24.dp, vertical = 8.dp)
                    .alpha(0.3f),
        )
    }

    // NOTE: Categorias
    scope.item {
        SectionHeader(stringResource(id = R.string.title_config_categories))
    }

    scope.item {
        Comic.Component.ComicCategorySelector(
            selectedCategory = uiState.comic.category,
            allCategories = uiState.allCategories,
            onUpdateMangaCategory = { id -> onAction(ComicAction.UpdateCategory(id)) },
            modifier = itemModifier,
        )
    }

    scope.item {
        HorizontalDivider(
            modifier =
                Modifier
                    .padding(horizontal = 24.dp, vertical = 8.dp)
                    .alpha(0.3f),
        )
    }

    // NOTE: Arquivos Locais
    scope.item {
        SectionHeader(stringResource(id = R.string.title_text_archive_configs_in_app))
    }

    scope.item {
        Comic.Component.SyncMangaArchive(
            onSyncChapters = { onSyncAction(ComicSyncAction.SyncChaptersLocal) },
            onRescanCover = { onSyncAction(ComicSyncAction.RescanComic) },
            onExtractFirstPageAsCover = { onSyncAction(ComicSyncAction.ExtractFirstPageAsCover) },
            onExtractVolumeCovers =
                if (uiState.volumeViewMode == VolumeViewType.COVER_VOLUME) {
                    { onSyncAction(ComicSyncAction.ExtractVolumeCovers) }
                } else {
                    null
                },
            syncChaptersState = getSyncActionVisualState(ComicSyncAction.SyncChaptersLocal),
            rescanCoverState = getSyncActionVisualState(ComicSyncAction.RescanComic),
            extractFirstPageState = getSyncActionVisualState(ComicSyncAction.ExtractFirstPageAsCover),
            extractVolumeCoversState = getSyncActionVisualState(ComicSyncAction.ExtractVolumeCovers),
            modifier = itemModifier,
        )
    }

    scope.item {
        HorizontalDivider(
            modifier =
                Modifier
                    .padding(horizontal = 24.dp, vertical = 8.dp)
                    .alpha(0.3f),
        )
    }

    // NOTE: Metadados Externos
    scope.item {
        SectionHeader(stringResource(id = R.string.title_sync_external_metadata))
    }

    scope.item {
        Comic.Component.ComicExternalSyncToggle(
            enabled = uiState.comic.directory.externalSyncEnabled,
            onToggle = { onAction(ComicAction.ToggleExternalSync(it)) },
            modifier = itemModifier,
        )
    }

    scope.item { Spacer(modifier = Modifier.height(8.dp)) }

    scope.item {
        Comic.Component.SyncMetadata(
            remoteInfo = uiState.comic.remoteInfo,
            externalSyncEnabled = uiState.comic.directory.externalSyncEnabled,
            onSyncMangadexInfo = { onSyncAction(ComicSyncAction.SyncMangadexInfo) },
            onSyncComicInfo = { onSyncAction(ComicSyncAction.SyncComicInfo) },
            onSyncAnilistInfo = { onSyncAction(ComicSyncAction.SyncAnilistInfo) },
            mangadexInfoState = getSyncActionVisualState(ComicSyncAction.SyncMangadexInfo),
            anilistInfoState = getSyncActionVisualState(ComicSyncAction.SyncAnilistInfo),
            comicInfoState = getSyncActionVisualState(ComicSyncAction.SyncComicInfo),
            modifier = itemModifier,
        )
    }

    scope.item { Spacer(modifier = Modifier.height(8.dp)) }

    scope.item {
        var showClearMetadataDialog by remember { mutableStateOf(false) }

        Acerola.Component.HeroButton(
            title = stringResource(id = R.string.action_clear_metadata),
            description = stringResource(id = R.string.description_clear_metadata),
            icon = Icons.Rounded.LayersClear,
            iconTint = MaterialTheme.colorScheme.onErrorContainer,
            iconBackground = MaterialTheme.colorScheme.errorContainer,
            onClick = { showClearMetadataDialog = true },
            modifier = itemModifier,
        )

        if (showClearMetadataDialog) {
            Acerola.Component.Dialog(
                show = true,
                onDismiss = { showClearMetadataDialog = false },
                title = stringResource(id = R.string.dialog_clear_metadata_title),
                confirmButtonContent = {
                    Acerola.Component.DialogButton(
                        text = stringResource(id = R.string.action_clear_metadata),
                        onClick = {
                            showClearMetadataDialog = false
                            onAction(ComicAction.ClearMetadata)
                        },
                        containerColor = MaterialTheme.colorScheme.error,
                        contentColor = MaterialTheme.colorScheme.onError,
                        fontWeight = FontWeight.Bold,
                    )
                },
                dismissButtonContent = {
                    Acerola.Component.DialogButton(
                        text = stringResource(id = R.string.action_cancel),
                        onClick = { showClearMetadataDialog = false },
                        contentColor = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                },
                content = { Text(text = stringResource(id = R.string.dialog_clear_metadata_message)) },
            )
        }
    }

    scope.item {
        HorizontalDivider(
            modifier =
                Modifier
                    .padding(horizontal = 24.dp, vertical = 8.dp)
                    .alpha(0.3f),
        )
    }

    // NOTE: Sincronização entre Dispositivos (acerola/sync-comic/1)
    scope.item {
        SectionHeader(stringResource(id = R.string.title_config_sync_devices))
    }

    scope.item {
        var showPeerPicker by remember { mutableStateOf(false) }
        var pendingDirection by remember { mutableStateOf(SyncDirection.PUSH) }

        Comic.Component.SyncWithPeerAction(
            state = syncWithPeerState,
            onPush = {
                onLoadPairedPeers()
                pendingDirection = SyncDirection.PUSH
                showPeerPicker = true
            },
            onPull = {
                onLoadPairedPeers()
                pendingDirection = SyncDirection.PULL
                showPeerPicker = true
            },
            modifier = itemModifier,
        )

        if (showPeerPicker) {
            Main.Common.Component.PeerPickerSheet(
                peers = pairedPeers,
                onSelect = { peerId ->
                    showPeerPicker = false
                    onSyncAction(ComicSyncAction.SyncWithPeer(peerId, pendingDirection))
                },
                onDismiss = { showPeerPicker = false },
            )
        }
    }

    scope.item { Spacer(modifier = Modifier.height(32.dp)) }
}
