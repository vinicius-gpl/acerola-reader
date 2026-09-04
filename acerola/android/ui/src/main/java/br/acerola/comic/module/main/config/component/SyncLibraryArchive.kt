package br.acerola.comic.module.main.config.component

import android.content.res.Configuration
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Folder
import androidx.compose.material.icons.filled.Sync
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
import br.acerola.comic.module.main.Main
import br.acerola.comic.ui.R

@Composable
fun Main.Config.Component.SyncLibraryArchive(
    onDeepScan: () -> Unit,
    onQuickSync: () -> Unit,
    deepScanState: SyncActionVisualState = SyncActionVisualState.IDLE,
    quickSyncState: SyncActionVisualState = SyncActionVisualState.IDLE,
    modifier: Modifier = Modifier,
) {
    val anyLoading = deepScanState == SyncActionVisualState.LOADING || quickSyncState == SyncActionVisualState.LOADING

    Column(modifier = modifier.fillMaxWidth()) {
        Acerola.Component.HeroButton(
            title = stringResource(id = R.string.description_text_home_deep_sync),
            description = stringResource(id = R.string.description_text_home_deep_sync_supporting),
            iconBackground = MaterialTheme.colorScheme.primaryContainer,
            onClick = if (anyLoading) null else onDeepScan,
            icon = {
                Acerola.Component.SyncActionIcon(
                    state = deepScanState,
                    defaultBackground = MaterialTheme.colorScheme.primaryContainer,
                ) {
                    Icon(
                        imageVector = Icons.Default.Folder,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onPrimaryContainer,
                        modifier = Modifier.size(SizeTokens.IconMedium),
                    )
                }
            },
        )
        Spacer(modifier = Modifier.height(8.dp))
        Acerola.Component.HeroButton(
            title = stringResource(id = R.string.description_text_home_quick_sync),
            description = stringResource(id = R.string.description_text_home_quick_sync_supporting),
            iconBackground = MaterialTheme.colorScheme.primaryContainer,
            onClick = if (anyLoading) null else onQuickSync,
            icon = {
                Acerola.Component.SyncActionIcon(
                    state = quickSyncState,
                    defaultBackground = MaterialTheme.colorScheme.primaryContainer,
                ) {
                    Icon(
                        imageVector = Icons.Default.Sync,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onPrimaryContainer,
                        modifier = Modifier.size(SizeTokens.IconMedium),
                    )
                }
            },
        )
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SyncLibraryArchivePreview() {
    AcerolaTheme {
        Main.Config.Component.SyncLibraryArchive(
            onDeepScan = {},
            onQuickSync = {},
        )
    }
}
