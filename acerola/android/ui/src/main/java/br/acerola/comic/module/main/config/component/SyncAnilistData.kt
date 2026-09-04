package br.acerola.comic.module.main.config.component

import android.content.res.Configuration
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.state.SyncActionVisualState
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.HeroButton
import br.acerola.comic.common.ux.component.SyncActionIcon
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.module.main.Main
import br.acerola.comic.ui.R

@Composable
fun Main.Config.Component.SyncAnilistData(
    onRescan: () -> Unit,
    state: SyncActionVisualState = SyncActionVisualState.IDLE,
    modifier: Modifier = Modifier,
) {
    Acerola.Component.HeroButton(
        title = stringResource(id = R.string.title_sync_anilist_remote_info),
        description = stringResource(id = R.string.description_sync_anilist_remote_info),
        iconBackground = MaterialTheme.colorScheme.tertiaryContainer,
        onClick = if (state == SyncActionVisualState.LOADING) null else onRescan,
        modifier = modifier,
        icon = {
            Acerola.Component.SyncActionIcon(
                state = state,
                defaultBackground = MaterialTheme.colorScheme.tertiaryContainer,
            ) {
                Image(
                    painter = painterResource(id = R.drawable.anilist),
                    contentDescription = null,
                    modifier = Modifier.size(28.dp),
                )
            }
        },
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SyncAnilistDataPreview() {
    AcerolaTheme {
        Main.Config.Component.SyncAnilistData(
            onRescan = {},
        )
    }
}
