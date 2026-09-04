package br.acerola.comic.common.ux.component
import android.content.res.Configuration
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaTheme

@Composable
fun Acerola.Component.RadioGroup(
    selectedIndex: Int,
    options: List<String>,
    onSelect: (Int) -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = modifier.fillMaxWidth(),
    ) {
        options.forEachIndexed { index, label ->
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier =
                    Modifier
                        .padding(end = 12.dp)
                        .clickable { onSelect(index) },
            ) {
                RadioButton(
                    selected = (selectedIndex == index),
                    onClick = { onSelect(index) },
                    modifier = Modifier.testTag("radio_button_$label"),
                )
                Spacer(modifier = Modifier.width(width = 8.dp))
                Text(text = label)
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun RadioGroupPreview() {
    AcerolaTheme {
        Acerola.Component.RadioGroup(
            selectedIndex = 0,
            options = listOf("Option 1", "Option 2"),
            onSelect = {},
        )
    }
}
