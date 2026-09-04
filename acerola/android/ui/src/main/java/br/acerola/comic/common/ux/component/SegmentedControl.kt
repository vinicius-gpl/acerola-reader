package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.animation.animateColorAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.ShapeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens

/** Pílula de N opções mutuamente exclusivas — para trocar de aba dentro de um mesmo
 *  contexto (ex.: "Meu código" / "Conectar" no sheet de pareamento), diferente de
 *  [Acerola.Component.RadioGroup], que representa a seleção de uma entre várias opções
 *  de uma lista de configuração. */
@Composable
fun Acerola.Component.SegmentedControl(
    options: List<String>,
    selectedIndex: Int,
    onSelect: (Int) -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(
        shape = ShapeTokens.Full,
        color = MaterialTheme.colorScheme.surfaceContainerHigh,
        modifier = modifier.fillMaxWidth(),
    ) {
        Row(modifier = Modifier.padding(SpacingTokens.ExtraSmall)) {
            options.forEachIndexed { index, label ->
                val selected = index == selectedIndex
                val containerColor by animateColorAsState(
                    targetValue = if (selected) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.surfaceContainerHigh,
                    label = "segmentedControlBg",
                )
                val contentColor = if (selected) MaterialTheme.colorScheme.onPrimaryContainer else MaterialTheme.colorScheme.onSurfaceVariant

                Box(
                    contentAlignment = Alignment.Center,
                    modifier =
                        Modifier
                            .weight(1f)
                            .clip(ShapeTokens.Full)
                            .background(containerColor)
                            .clickable(enabled = !selected) { onSelect(index) }
                            .padding(vertical = SpacingTokens.Small),
                ) {
                    Text(
                        text = label,
                        style = MaterialTheme.typography.labelLarge,
                        fontWeight = if (selected) FontWeight.Bold else FontWeight.Normal,
                        color = contentColor,
                        textAlign = TextAlign.Center,
                    )
                }
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SegmentedControlPreview() {
    AcerolaTheme {
        Acerola.Component.SegmentedControl(
            options = listOf("Meu código", "Conectar"),
            selectedIndex = 0,
            onSelect = {},
            modifier = Modifier.padding(SpacingTokens.Large),
        )
    }
}
