package br.acerola.comic.module.comic.component

import android.content.res.Configuration
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.LibraryBooks
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.HeroButton
import br.acerola.comic.common.ux.component.RadioGroup
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.config.preference.types.VolumeViewType
import br.acerola.comic.module.comic.Comic
import br.acerola.comic.ui.R

@Composable
fun Comic.Component.VolumeStylePreference(
    selected: VolumeViewType,
    onSelect: (VolumeViewType) -> Unit,
    modifier: Modifier = Modifier,
) {
    val options = listOf(VolumeViewType.VOLUME, VolumeViewType.COVER_VOLUME)
    val selectedIndex = options.indexOf(selected).takeIf { it >= 0 } ?: 0

    Acerola.Component.HeroButton(
        title = stringResource(id = R.string.title_settings_volume_style),
        description = volumeStyleLabel(selected),
        icon = Icons.Default.LibraryBooks,
        iconTint = MaterialTheme.colorScheme.onTertiaryContainer,
        iconBackground = MaterialTheme.colorScheme.tertiaryContainer,
        modifier = modifier,
        bottomContent = {
            Acerola.Component.RadioGroup(
                selectedIndex = selectedIndex,
                options = options.map { volumeStyleLabel(it) },
                onSelect = { index -> onSelect(options[index]) },
                modifier = Modifier.padding(horizontal = 20.dp, vertical = 12.dp),
            )
        },
    )
}

@Composable
private fun volumeStyleLabel(viewType: VolumeViewType): String =
    when (viewType) {
        VolumeViewType.COVER_VOLUME -> stringResource(R.string.label_volume_style_cover)
        else -> stringResource(R.string.label_volume_style_normal)
    }

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun VolumeStylePreferencePreview() {
    AcerolaTheme {
        Comic.Component.VolumeStylePreference(
            selected = VolumeViewType.VOLUME,
            onSelect = {},
        )
    }
}
