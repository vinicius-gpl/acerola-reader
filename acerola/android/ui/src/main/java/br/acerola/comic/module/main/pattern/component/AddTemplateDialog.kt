package br.acerola.comic.module.main.pattern.component

import android.content.res.Configuration
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Info
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.AdaptiveSheet
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.module.main.Main
import br.acerola.comic.ui.R
import br.acerola.comic.util.sort.SortType

@Composable
fun Main.Pattern.Component.AddTemplateDialog(
    onDismiss: () -> Unit,
    onConfirm: (String, String, SortType) -> Unit,
    initialLabel: String = "",
    initialPattern: String = "",
    initialType: SortType = SortType.CHAPTER,
    isEditMode: Boolean = false,
) {
    var label by remember { mutableStateOf(initialLabel) }
    var pattern by remember { mutableStateOf(initialPattern) }
    var type by remember { mutableStateOf(initialType) }

    Acerola.Component.AdaptiveSheet(onDismissRequest = onDismiss) {
        Column(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(horizontal = SpacingTokens.Huge, vertical = SpacingTokens.Small),
        ) {
            Text(
                text =
                    stringResource(
                        id =
                            if (isEditMode) {
                                R.string.title_dialog_edit_template
                            } else {
                                R.string.title_dialog_new_template
                            },
                    ),
                style = MaterialTheme.typography.titleLarge,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(bottom = SpacingTokens.Large),
            )

            SingleChoiceSegmentedButtonRow(
                modifier = Modifier.fillMaxWidth(),
            ) {
                SortType.entries.forEachIndexed { index, sortType ->
                    SegmentedButton(
                        selected = sortType == type,
                        onClick = { type = sortType },
                        shape = SegmentedButtonDefaults.itemShape(index = index, count = SortType.entries.size),
                    ) {
                        Text(
                            text =
                                when (sortType) {
                                    SortType.CHAPTER -> stringResource(id = R.string.label_sort_type_chapter)
                                    SortType.VOLUME -> stringResource(id = R.string.label_sort_type_volume)
                                },
                        )
                    }
                }
            }

            Spacer(modifier = Modifier.height(SpacingTokens.Small))

            OutlinedTextField(
                value = label,
                onValueChange = { label = it },
                label = { Text(stringResource(id = R.string.label_template_label)) },
                modifier = Modifier.fillMaxWidth(),
            )
            Spacer(modifier = Modifier.height(SpacingTokens.Small))
            OutlinedTextField(
                value = pattern,
                onValueChange = { pattern = it },
                label = { Text(stringResource(id = R.string.label_template_pattern)) },
                placeholder = { Text(stringResource(id = R.string.placeholder_template_pattern)) },
                modifier = Modifier.fillMaxWidth(),
            )

            Card(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .padding(top = SpacingTokens.Medium),
                colors =
                    CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.secondaryContainer.copy(alpha = 0.4f),
                    ),
            ) {
                Row(
                    modifier = Modifier.padding(SpacingTokens.Medium),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Icon(
                        Icons.Default.Info,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.primary,
                        modifier = Modifier.size(SizeTokens.IconSmall),
                    )
                    Spacer(modifier = Modifier.width(SpacingTokens.Medium))
                    Text(
                        text = stringResource(id = R.string.description_template_macros),
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.onSecondaryContainer,
                    )
                }
            }

            Spacer(modifier = Modifier.height(SpacingTokens.Huge))

            Row(modifier = Modifier.fillMaxWidth()) {
                OutlinedButton(
                    onClick = onDismiss,
                    modifier = Modifier.weight(1f),
                ) {
                    Text(stringResource(id = R.string.action_cancel))
                }
                Spacer(modifier = Modifier.width(SpacingTokens.Medium))
                Button(
                    onClick = { if (label.isNotBlank() && pattern.isNotBlank()) onConfirm(label, pattern, type) },
                    modifier = Modifier.weight(1f),
                ) {
                    Text(
                        text =
                            stringResource(
                                id = if (isEditMode) R.string.action_save else R.string.action_add,
                            ),
                        fontWeight = FontWeight.Bold,
                    )
                }
            }

            Spacer(modifier = Modifier.height(SpacingTokens.Large))
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun AddTemplateDialogPreview() {
    AcerolaTheme {
        Main.Pattern.Component.AddTemplateDialog(
            onDismiss = {},
            onConfirm = { _, _, _ -> },
        )
    }
}
