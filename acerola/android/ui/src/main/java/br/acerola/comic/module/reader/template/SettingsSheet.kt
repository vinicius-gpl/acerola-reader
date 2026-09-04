package br.acerola.comic.module.reader.template

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.AutoStories
import androidx.compose.material.icons.filled.VerticalAlignBottom
import androidx.compose.material.icons.filled.ViewHeadline
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.AdaptiveSheet
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.ShapeTokens
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.config.preference.types.ReadingMode
import br.acerola.comic.module.reader.Reader
import br.acerola.comic.ui.R

@Composable
fun Reader.Template.SettingsSheet(
    onDismissRequest: () -> Unit,
    currentMode: ReadingMode,
    onModeSelected: (ReadingMode) -> Unit,
) {
    Acerola.Component.AdaptiveSheet(
        onDismissRequest = onDismissRequest,
        isScrollable = false,
        containerColor = MaterialTheme.colorScheme.surface,
        contentColor = MaterialTheme.colorScheme.onSurface,
    ) {
        Column(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(horizontal = SpacingTokens.Huge)
                    .verticalScroll(rememberScrollState())
                    .padding(bottom = SpacingTokens.Giant),
        ) {
            Text(
                text = stringResource(id = R.string.label_reader_config_title),
                style = MaterialTheme.typography.headlineSmall.copy(fontWeight = FontWeight.Bold),
                color = MaterialTheme.colorScheme.onSurface,
                modifier = Modifier.padding(vertical = SpacingTokens.Huge),
            )

            Text(
                text = "Layout de Leitura",
                style = MaterialTheme.typography.labelLarge.copy(color = MaterialTheme.colorScheme.primary),
                modifier = Modifier.padding(bottom = SpacingTokens.Medium),
            )

            ReadingModeItem(
                title = stringResource(id = R.string.label_reader_mode_horizontal),
                icon = Icons.Default.AutoStories,
                isSelected = currentMode == ReadingMode.HORIZONTAL,
                onClick = { onModeSelected(ReadingMode.HORIZONTAL) },
            )

            ReadingModeItem(
                title = stringResource(id = R.string.label_reader_mode_vertical),
                icon = Icons.Default.ViewHeadline,
                isSelected = currentMode == ReadingMode.VERTICAL,
                onClick = { onModeSelected(ReadingMode.VERTICAL) },
            )

            ReadingModeItem(
                title = stringResource(id = R.string.label_reader_mode_webtoon),
                icon = Icons.Default.VerticalAlignBottom,
                isSelected = currentMode == ReadingMode.WEBTOON,
                onClick = { onModeSelected(ReadingMode.WEBTOON) },
            )
        }
    }
}

@Composable
private fun ReadingModeItem(
    title: String,
    icon: ImageVector,
    isSelected: Boolean,
    onClick: () -> Unit,
) {
    val backgroundColor =
        if (isSelected) {
            MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.5f)
        } else {
            Color.Transparent
        }

    val contentColor =
        if (isSelected) {
            MaterialTheme.colorScheme.primary
        } else {
            MaterialTheme.colorScheme.onSurface
        }

    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier =
            Modifier
                .fillMaxWidth()
                .padding(vertical = SpacingTokens.ExtraSmall)
                .clip(ShapeTokens.Large)
                .background(backgroundColor)
                .clickable(onClick = onClick)
                .padding(SpacingTokens.Large),
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = contentColor,
            modifier = Modifier.size(SizeTokens.IconMedium),
        )

        Spacer(modifier = Modifier.width(SpacingTokens.Large))

        Text(
            text = title,
            color = contentColor,
            modifier = Modifier.weight(1f),
            style =
                MaterialTheme.typography.bodyLarge.copy(
                    fontWeight = if (isSelected) FontWeight.Bold else FontWeight.Medium,
                ),
        )

        if (isSelected) {
            Box(
                modifier =
                    Modifier
                        .size(SpacingTokens.Small)
                        .clip(ShapeTokens.Full)
                        .background(MaterialTheme.colorScheme.primary),
            )
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SettingsSheetPreview() {
    AcerolaTheme {
        Reader.Template.SettingsSheet(
            onDismissRequest = {},
            currentMode = ReadingMode.VERTICAL,
            onModeSelected = {},
        )
    }
}
