package br.acerola.comic.module.comic.component

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Bookmark
import androidx.compose.material3.AssistChip
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.dto.archive.VolumeArchiveDto
import br.acerola.comic.module.comic.Comic
import br.acerola.comic.ui.R

@Composable
fun Comic.Component.VolumeHeader(
    volume: VolumeArchiveDto,
    chapterCount: Int,
    modifier: Modifier = Modifier,
) {
    Box(
        modifier =
            modifier
                .fillMaxWidth()
                .background(MaterialTheme.colorScheme.surface)
                .padding(vertical = 8.dp),
    ) {
        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .background(
                        color = MaterialTheme.colorScheme.secondaryContainer,
                        shape = MaterialTheme.shapes.small,
                    ).padding(horizontal = 16.dp, vertical = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(
                imageVector = Icons.Default.Bookmark,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onSecondaryContainer,
                modifier = Modifier.size(20.dp),
            )
            Column(
                modifier =
                    Modifier
                        .weight(1f)
                        .padding(start = 8.dp),
            ) {
                Text(
                    text = volume.name.ifBlank { stringResource(id = R.string.label_volume_header_unknown) },
                    style = MaterialTheme.typography.titleSmall,
                    color = MaterialTheme.colorScheme.onSecondaryContainer,
                    fontWeight = FontWeight.Bold,
                )
                Text(
                    text = pluralStringResource(id = R.plurals.label_volume_header_chapter_count, chapterCount, chapterCount),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSecondaryContainer,
                )
            }
            if (volume.isSpecial) {
                AssistChip(
                    onClick = {},
                    enabled = false,
                    label = {
                        Text(text = stringResource(id = R.string.label_volume_header_special))
                    },
                )
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun VolumeHeaderPreview() {
    AcerolaTheme {
        Comic.Component.VolumeHeader(
            volume = VolumeArchiveDto(id = 1L, name = "Vol 1", volumeSort = "0001", isSpecial = false),
            chapterCount = 5,
        )
    }
}
