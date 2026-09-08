package br.acerola.comic.module.comic.template

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Collections
import androidx.compose.material.icons.filled.Folder
import androidx.compose.material.icons.filled.PhoneAndroid
import androidx.compose.material.icons.filled.Public
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.filled.Sync
import androidx.compose.material.icons.rounded.LayersClear
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import br.acerola.comic.common.state.SyncActionVisualState
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.AccordionCard
import br.acerola.comic.common.ux.component.Dialog
import br.acerola.comic.common.ux.component.DialogButton
import br.acerola.comic.common.ux.component.HeroButton
import br.acerola.comic.config.preference.types.VolumeViewType
import br.acerola.comic.module.comic.Comic
import br.acerola.comic.module.comic.component.ComicCategorySelector
import br.acerola.comic.module.comic.component.ComicExternalSyncToggle
import br.acerola.comic.module.comic.component.ComicInfoSync
import br.acerola.comic.module.comic.component.ExternalMetadataSync
import br.acerola.comic.module.comic.component.PaginationPreference
import br.acerola.comic.module.comic.component.SyncMangaArchive
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

private val categoryModifier = Modifier.padding(horizontal = 16.dp, vertical = 4.dp)

// Mesmas 3 categorias do desktop (Leitura/Sincronização/Avançado, ver
// acerola-comic-preferences.svelte): Sincronização é a única flat (sempre aberta, é a parte
// mais usada — metadados, arquivos e dispositivos pareados); Leitura e Avançado colapsam.
// `expandedCategories`/`onToggleCategory` vêm do `ComicScreen` porque este arquivo não é
// `@Composable` (só monta `scope.item {}`), então não pode ter seu próprio `remember`.
fun Comic.Template.configSection(
    scope: LazyListScope,
    uiState: ComicUiState,
    expandedCategories: Set<String>,
    onToggleCategory: (String) -> Unit,
    getSyncActionVisualState: (ComicSyncAction) -> SyncActionVisualState = { SyncActionVisualState.IDLE },
    pairedPeers: List<PairedPeer> = emptyList(),
    syncWithPeerState: SyncActionVisualState = SyncActionVisualState.IDLE,
    onLoadPairedPeers: () -> Unit = {},
    onAction: (ComicAction) -> Unit,
    onSyncAction: (ComicSyncAction) -> Unit,
) {
    scope.item { Spacer(modifier = Modifier.height(16.dp)) }

    // NOTE: Leitura
    scope.item {
        Acerola.Component.AccordionCard(
            title = stringResource(id = R.string.title_comic_preferences_reading),
            description = stringResource(id = R.string.description_comic_preferences_reading),
            icon = Icons.Default.Settings,
            accentColor = MaterialTheme.colorScheme.primary,
            expanded = "reading" in expandedCategories,
            onToggleExpanded = { onToggleCategory("reading") },
            modifier = categoryModifier,
        ) {
            if (uiState.hasVolumeStructure) {
                Comic.Component.VolumeStylePreference(
                    selected = uiState.volumeViewMode,
                    onSelect = { mode -> onAction(ComicAction.UpdateVolumeView(mode)) },
                )
            }

            Comic.Component.PaginationPreference(
                selected = uiState.selectedChapterPerPage,
                onSelect = { onAction(ComicAction.UpdatePageSize(it)) },
            )

            Comic.Component.ComicCategorySelector(
                selectedCategory = uiState.comic.category,
                allCategories = uiState.allCategories,
                onUpdateMangaCategory = { id -> onAction(ComicAction.UpdateCategory(id)) },
            )
        }
    }

    // NOTE: Sincronização — fica sempre aberta (flat): metadados, arquivos e dispositivos
    // pareados são a parte mais usada dessa tela, não vale esconder atrás de um clique.
    scope.item {
        Acerola.Component.AccordionCard(
            title = stringResource(id = R.string.title_comic_preferences_sync),
            description = stringResource(id = R.string.description_comic_preferences_sync),
            icon = Icons.Default.Sync,
            accentColor = MaterialTheme.colorScheme.secondary,
            expanded = true,
            onToggleExpanded = {},
            collapsible = false,
            modifier = categoryModifier,
        ) {
            SectionLabel(icon = Icons.Default.Public, text = stringResource(id = R.string.label_comic_preferences_section_metadata))

            Comic.Component.ComicExternalSyncToggle(
                enabled = uiState.comic.directory.externalSyncEnabled,
                onToggle = { onAction(ComicAction.ToggleExternalSync(it)) },
            ) {
                Comic.Component.ExternalMetadataSync(
                    remoteInfo = uiState.comic.remoteInfo,
                    onSyncMangadexInfo = { onSyncAction(ComicSyncAction.SyncMangadexInfo) },
                    onSyncAnilistInfo = { onSyncAction(ComicSyncAction.SyncAnilistInfo) },
                    mangadexInfoState = getSyncActionVisualState(ComicSyncAction.SyncMangadexInfo),
                    anilistInfoState = getSyncActionVisualState(ComicSyncAction.SyncAnilistInfo),
                )
            }

            Comic.Component.ComicInfoSync(
                remoteInfo = uiState.comic.remoteInfo,
                onSyncComicInfo = { onSyncAction(ComicSyncAction.SyncComicInfo) },
                comicInfoState = getSyncActionVisualState(ComicSyncAction.SyncComicInfo),
            )

            SectionLabel(icon = Icons.Default.Folder, text = stringResource(id = R.string.label_comic_preferences_section_files))

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
            )

            SectionLabel(icon = Icons.Default.PhoneAndroid, text = stringResource(id = R.string.label_comic_preferences_section_devices))

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
    }

    // NOTE: Avançado
    scope.item {
        Acerola.Component.AccordionCard(
            title = stringResource(id = R.string.title_comic_preferences_advanced),
            description = stringResource(id = R.string.description_comic_preferences_advanced),
            icon = Icons.Default.Collections,
            accentColor = MaterialTheme.colorScheme.tertiary,
            expanded = "advanced" in expandedCategories,
            onToggleExpanded = { onToggleCategory("advanced") },
            modifier = categoryModifier,
        ) {
            SectionLabel(
                icon = Icons.Rounded.LayersClear,
                text = stringResource(id = R.string.label_comic_preferences_section_danger_zone),
            )

            var showClearMetadataDialog by remember { mutableStateOf(false) }

            Acerola.Component.HeroButton(
                title = stringResource(id = R.string.action_clear_metadata),
                description = stringResource(id = R.string.description_clear_metadata),
                icon = Icons.Rounded.LayersClear,
                iconTint = MaterialTheme.colorScheme.onErrorContainer,
                iconBackground = MaterialTheme.colorScheme.errorContainer,
                onClick = { showClearMetadataDialog = true },
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
    }

    scope.item { Spacer(modifier = Modifier.height(32.dp)) }
}

// Rótulo de subgrupo dentro de uma categoria flat (ícone 16dp + texto uppercase) — mesmo
// padrão visual das `<section>` do desktop dentro de "Sincronização"/"Avançado".
@Composable
private fun SectionLabel(
    icon: ImageVector,
    text: String,
) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.size(16.dp),
        )
        Text(
            text = text.uppercase(),
            style = MaterialTheme.typography.labelSmall,
            fontWeight = FontWeight.Bold,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            letterSpacing = 1.sp,
        )
    }
}
