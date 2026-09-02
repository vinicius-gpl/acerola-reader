package br.acerola.comic.module.comic.component

import android.content.res.Configuration
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.CloudDownload
import androidx.compose.material.icons.rounded.CloudUpload
import androidx.compose.material.icons.rounded.PhoneAndroid
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
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
import br.acerola.comic.module.comic.Comic
import br.acerola.comic.ui.R

/**
 * Single entry point for `acerola/sync-comic/1`, shared by the Comic Detail's config section
 * and (via [br.acerola.comic.module.main.common.component.PeerPickerSheet]) the Home's
 * `ComicActionsSheet`. Two explicit actions — push (send this comic to a device) and pull
 * (fetch it from a device) — each open the peer picker on click; the actual `syncComic` call
 * only fires once a peer is chosen, with the direction already decided by which icon started
 * the flow.
 */
@Composable
fun Comic.Component.SyncWithPeerAction(
    state: SyncActionVisualState,
    onPush: () -> Unit,
    onPull: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Acerola.Component.HeroButton(
        title = stringResource(id = R.string.action_sync_comic_with_peer),
        description = stringResource(id = R.string.description_sync_comic_with_peer),
        iconBackground = MaterialTheme.colorScheme.primaryContainer,
        icon = {
            Acerola.Component.SyncActionIcon(
                state = state,
                defaultBackground = MaterialTheme.colorScheme.primaryContainer,
            ) {
                Icon(
                    imageVector = Icons.Rounded.PhoneAndroid,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.onPrimaryContainer,
                    modifier = Modifier.size(24.dp),
                )
            }
        },
        action = {
            if (state != SyncActionVisualState.LOADING) {
                Row {
                    IconButton(onClick = onPush) {
                        Icon(
                            imageVector = Icons.Rounded.CloudUpload,
                            contentDescription = stringResource(id = R.string.action_sync_comic_push),
                        )
                    }
                    IconButton(onClick = onPull) {
                        Icon(
                            imageVector = Icons.Rounded.CloudDownload,
                            contentDescription = stringResource(id = R.string.action_sync_comic_pull),
                        )
                    }
                }
            }
        },
        modifier = modifier,
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SyncWithPeerActionPreview() {
    AcerolaTheme {
        Comic.Component.SyncWithPeerAction(
            state = SyncActionVisualState.IDLE,
            onPush = {},
            onPull = {},
        )
    }
}
