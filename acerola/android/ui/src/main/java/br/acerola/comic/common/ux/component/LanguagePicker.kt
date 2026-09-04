package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Button
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.ListItem
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.mapper.LanguageMapper
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.ui.R

@Composable
fun Acerola.Component.LanguagePicker(
    selectedLanguage: String?,
    onLanguageSelected: (String) -> Unit,
    trigger: @Composable (onClick: () -> Unit) -> Unit,
) {
    var showSheet by remember { mutableStateOf(false) }

    trigger { showSheet = true }

    if (showSheet) {
        val configuration = LocalConfiguration.current
        val maxHeight = (configuration.screenHeightDp * 0.55f).dp

        Acerola.Component.AdaptiveSheet(
            onDismissRequest = { showSheet = false },
            isScrollable = false,
        ) {
            Text(
                text = stringResource(id = R.string.title_settings_metadata_language),
                style = MaterialTheme.typography.titleLarge,
                modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
            )

            HorizontalDivider(modifier = Modifier.padding(vertical = 8.dp))

            LazyColumn(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .heightIn(max = maxHeight)
                        .padding(bottom = 32.dp),
            ) {
                items(LanguageMapper.getAllCodes()) { code ->
                    ListItem(
                        headlineContent = { Text(stringResource(id = LanguageMapper.getLabelRes(code))) },
                        leadingContent = {
                            RadioButton(
                                selected = code == selectedLanguage,
                                onClick = null,
                            )
                        },
                        modifier =
                            Modifier.clickable {
                                onLanguageSelected(code)
                                showSheet = false
                            },
                    )
                }
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun LanguagePickerPreview() {
    AcerolaTheme {
        Acerola.Component.LanguagePicker(
            selectedLanguage = "pt-BR",
            onLanguageSelected = {},
            trigger = { onClick -> Button(onClick = onClick) { Text("Language") } },
        )
    }
}
