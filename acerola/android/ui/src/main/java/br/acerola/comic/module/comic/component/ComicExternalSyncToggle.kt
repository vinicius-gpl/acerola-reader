package br.acerola.comic.module.comic.component

import android.content.res.Configuration
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Sync
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.ToggleCard
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.module.comic.Comic
import br.acerola.comic.ui.R

// Card ativado no lugar do HeroButton+Switch cru — mesmo formato do card "Sincronização
// externa" do desktop (acerola-comic-preferences.svelte): clicar liga/desliga, e os botões de
// sync (Comic.Component.SyncMetadata, renderizado como irmão logo abaixo) já se auto-escondem
// quando desativado, sem precisar do `content` expansível do ToggleCard aqui.
@Composable
fun Comic.Component.ComicExternalSyncToggle(
    enabled: Boolean,
    onToggle: (Boolean) -> Unit,
    modifier: Modifier = Modifier,
) {
    Acerola.Component.ToggleCard(
        title = stringResource(id = R.string.label_config_external_sync),
        subtitle = stringResource(id = R.string.description_config_external_sync),
        active = enabled,
        onClick = { onToggle(!enabled) },
        modifier = modifier,
        icon = { Icon(imageVector = Icons.Default.Sync, contentDescription = null) },
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ComicExternalSyncTogglePreview() {
    AcerolaTheme {
        Comic.Component.ComicExternalSyncToggle(
            enabled = true,
            onToggle = {},
        )
    }
}
