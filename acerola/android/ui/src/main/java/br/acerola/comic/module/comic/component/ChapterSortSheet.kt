package br.acerola.comic.module.comic.component

import android.content.res.Configuration
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowDownward
import androidx.compose.material.icons.filled.ArrowUpward
import androidx.compose.material.icons.filled.Check
import androidx.compose.material3.Divider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.AdaptiveSheet
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.config.preference.types.ChapterSortPreferenceData
import br.acerola.comic.config.preference.types.ChapterSortType
import br.acerola.comic.config.preference.types.SortDirection
import br.acerola.comic.module.comic.Comic
import br.acerola.comic.ui.R

@Composable
fun Comic.Component.ChapterSortSheet(
    sortSettings: ChapterSortPreferenceData,
    onSortChange: (ChapterSortPreferenceData) -> Unit,
    onDismiss: () -> Unit,
) {
    Acerola.Component.AdaptiveSheet(
        onDismissRequest = onDismiss,
        containerColor = MaterialTheme.colorScheme.surface,
        contentColor = MaterialTheme.colorScheme.onSurface,
    ) {
        Column(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(bottom = 32.dp),
        ) {
            Text(
                text = stringResource(id = R.string.title_chapter_sort_sheet),
                style = MaterialTheme.typography.titleLarge,
                modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
            )

            Divider(modifier = Modifier.padding(vertical = 8.dp))

            ChapterSortType.entries.forEach { type ->
                val label =
                    when (type) {
                        ChapterSortType.NUMBER -> stringResource(id = R.string.label_sort_number)
                        ChapterSortType.LAST_UPDATE -> stringResource(id = R.string.label_sort_last_update)
                    }
                Row(
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .clickable { onSortChange(sortSettings.copy(type = type)) }
                            .padding(horizontal = 16.dp, vertical = 12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(
                        text = label,
                        modifier = Modifier.weight(1f),
                        style = MaterialTheme.typography.bodyLarge,
                    )
                    if (sortSettings.type == type) {
                        IconButton(onClick = {
                            onSortChange(
                                sortSettings.copy(
                                    direction =
                                        if (sortSettings.direction == SortDirection.ASCENDING) {
                                            SortDirection.DESCENDING
                                        } else {
                                            SortDirection.ASCENDING
                                        },
                                ),
                            )
                        }) {
                            Icon(
                                imageVector =
                                    if (sortSettings.direction == SortDirection.ASCENDING) {
                                        Icons.Default.ArrowUpward
                                    } else {
                                        Icons.Default.ArrowDownward
                                    },
                                contentDescription = null,
                            )
                        }
                        Icon(
                            imageVector = Icons.Default.Check,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.primary,
                        )
                    }
                }
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ChapterSortSheetPreview() {
    AcerolaTheme {
        Comic.Component.ChapterSortSheet(
            sortSettings =
                ChapterSortPreferenceData(
                    type = ChapterSortType.NUMBER,
                    direction = SortDirection.ASCENDING,
                ),
            onSortChange = {},
            onDismiss = {},
        )
    }
}
