package br.acerola.comic.module.main.config.component

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Language
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.mapper.LanguageMapper
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.HeroButton
import br.acerola.comic.common.ux.component.LanguagePicker
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.ShapeTokens
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.module.main.Main
import br.acerola.comic.type.Language
import br.acerola.comic.ui.R

@Composable
fun Main.Config.Component.LanguageSettings(
    selectedLanguage: String?,
    onLanguageSelected: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val resolvedLanguage = selectedLanguage ?: Language.PT_BR.code
    val languageLabel = stringResource(id = LanguageMapper.getLabelRes(resolvedLanguage))

    Acerola.Component.LanguagePicker(
        selectedLanguage = resolvedLanguage,
        onLanguageSelected = onLanguageSelected,
        trigger = { onClick ->
            Acerola.Component.HeroButton(
                title = stringResource(id = R.string.title_settings_metadata_language),
                description = languageLabel,
                icon = Icons.Filled.Language,
                iconTint = MaterialTheme.colorScheme.onPrimaryContainer,
                iconBackground = MaterialTheme.colorScheme.primaryContainer,
                modifier = modifier,
                action = {
                    IconButton(onClick = onClick) {
                        Box(
                            contentAlignment = Alignment.Center,
                            modifier =
                                Modifier
                                    .size(size = SpacingTokens.Giant)
                                    .clip(ShapeTokens.Full)
                                    .background(color = MaterialTheme.colorScheme.primary),
                        ) {
                            Icon(
                                imageVector = Icons.Filled.Add,
                                contentDescription = stringResource(id = R.string.label_select_language),
                                tint = MaterialTheme.colorScheme.onPrimary,
                                modifier =
                                    Modifier
                                        .size(size = SizeTokens.IconLarge)
                                        .padding(all = SpacingTokens.ExtraSmall),
                            )
                        }
                    }
                },
            )
        },
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun LanguageSettingsPreview() {
    AcerolaTheme {
        Main.Config.Component.LanguageSettings(
            selectedLanguage = "pt-BR",
            onLanguageSelected = {},
        )
    }
}
