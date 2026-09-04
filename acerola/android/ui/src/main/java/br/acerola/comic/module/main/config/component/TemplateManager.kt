package br.acerola.comic.module.main.config.component

import android.content.res.Configuration
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.SettingsSuggest
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.HeroButton
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.module.main.Main
import br.acerola.comic.ui.R

@Composable
fun Main.Config.Component.TemplateManager(
    onManageTemplates: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Acerola.Component.HeroButton(
        title = stringResource(id = R.string.title_chapter_naming_templates),
        description = stringResource(id = R.string.description_template_config_activity),
        icon = Icons.Default.SettingsSuggest,
        iconTint = MaterialTheme.colorScheme.onPrimaryContainer,
        iconBackground = MaterialTheme.colorScheme.primaryContainer,
        onClick = onManageTemplates,
        modifier = modifier,
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun TemplateManagerPreview() {
    AcerolaTheme {
        Main.Config.Component.TemplateManager(
            onManageTemplates = {},
        )
    }
}
