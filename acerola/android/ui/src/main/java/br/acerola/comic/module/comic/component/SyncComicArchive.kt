package br.acerola.comic.module.comic.component

import android.content.res.Configuration
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.AutoAwesomeMotion
import androidx.compose.material.icons.filled.Collections
import androidx.compose.material.icons.filled.ImageSearch
import androidx.compose.material.icons.filled.SyncAlt
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.state.SyncActionVisualState
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.HeroButton
import br.acerola.comic.common.ux.component.SyncActionIcon
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.module.comic.Comic
import br.acerola.comic.ui.R

@Composable
fun Comic.Component.SyncMangaArchive(
    onSyncChapters: () -> Unit,
    onRescanCover: () -> Unit,
    onExtractFirstPageAsCover: () -> Unit,
    onExtractVolumeCovers: (() -> Unit)? = null,
    syncChaptersState: SyncActionVisualState = SyncActionVisualState.IDLE,
    rescanCoverState: SyncActionVisualState = SyncActionVisualState.IDLE,
    extractFirstPageState: SyncActionVisualState = SyncActionVisualState.IDLE,
    extractVolumeCoversState: SyncActionVisualState = SyncActionVisualState.IDLE,
    modifier: Modifier = Modifier,
) {
    val anyLoading =
        syncChaptersState == SyncActionVisualState.LOADING ||
            rescanCoverState == SyncActionVisualState.LOADING ||
            extractFirstPageState == SyncActionVisualState.LOADING ||
            extractVolumeCoversState == SyncActionVisualState.LOADING

    Column(modifier = modifier.fillMaxWidth()) {
        Acerola.Component.HeroButton(
            title = stringResource(id = R.string.title_sync_chapters),
            description = stringResource(id = R.string.description_sync_chapters_local),
            iconBackground = MaterialTheme.colorScheme.secondaryContainer,
            onClick = if (anyLoading) null else onSyncChapters,
            icon = {
                Acerola.Component.SyncActionIcon(
                    state = syncChaptersState,
                    defaultBackground = MaterialTheme.colorScheme.secondaryContainer,
                ) {
                    Icon(
                        imageVector = Icons.Default.SyncAlt,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onSecondaryContainer,
                        modifier = Modifier.size(SizeTokens.IconMedium),
                    )
                }
            },
        )

        Spacer(modifier = Modifier.height(8.dp))

        Acerola.Component.HeroButton(
            title = stringResource(id = R.string.title_sync_cover_banner),
            description = stringResource(id = R.string.description_sync_cover_banner),
            iconBackground = MaterialTheme.colorScheme.secondaryContainer,
            onClick = if (anyLoading) null else onRescanCover,
            icon = {
                Acerola.Component.SyncActionIcon(
                    state = rescanCoverState,
                    defaultBackground = MaterialTheme.colorScheme.secondaryContainer,
                ) {
                    Icon(
                        imageVector = Icons.Default.Collections,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onSecondaryContainer,
                        modifier = Modifier.size(SizeTokens.IconMedium),
                    )
                }
            },
        )

        Spacer(modifier = Modifier.height(8.dp))

        Acerola.Component.HeroButton(
            title = stringResource(id = R.string.title_extract_first_page_as_cover),
            description = stringResource(id = R.string.description_extract_first_page_as_cover),
            iconBackground = MaterialTheme.colorScheme.secondaryContainer,
            onClick = if (anyLoading) null else onExtractFirstPageAsCover,
            icon = {
                Acerola.Component.SyncActionIcon(
                    state = extractFirstPageState,
                    defaultBackground = MaterialTheme.colorScheme.secondaryContainer,
                ) {
                    Icon(
                        imageVector = Icons.Default.ImageSearch,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onSecondaryContainer,
                        modifier = Modifier.size(SizeTokens.IconMedium),
                    )
                }
            },
        )

        if (onExtractVolumeCovers != null) {
            Spacer(modifier = Modifier.height(8.dp))

            Acerola.Component.HeroButton(
                title = stringResource(id = R.string.title_extract_volume_covers),
                description = stringResource(id = R.string.description_extract_volume_covers),
                iconBackground = MaterialTheme.colorScheme.secondaryContainer,
                onClick = if (anyLoading) null else onExtractVolumeCovers,
                icon = {
                    Acerola.Component.SyncActionIcon(
                        state = extractVolumeCoversState,
                        defaultBackground = MaterialTheme.colorScheme.secondaryContainer,
                    ) {
                        Icon(
                            imageVector = Icons.Default.AutoAwesomeMotion,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.onSecondaryContainer,
                            modifier = Modifier.size(SizeTokens.IconMedium),
                        )
                    }
                },
            )
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SyncMangaArchivePreview() {
    AcerolaTheme {
        Comic.Component.SyncMangaArchive(
            onSyncChapters = {},
            onRescanCover = {},
            onExtractFirstPageAsCover = {},
        )
    }
}
