package br.acerola.comic.common.ux.component
import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaTheme

@Composable
fun Acerola.Component.Scaffold(
    modifier: Modifier = Modifier,
    containerColor: Color = MaterialTheme.colorScheme.background,
    applyStatusBarPadding: Boolean = true,
    content: @Composable BoxScope.() -> Unit,
) {
    Box(
        modifier =
            modifier
                .fillMaxSize()
                .background(containerColor),
    ) {
        Scrim(color = containerColor)
        Box(
            modifier =
                Modifier
                    .fillMaxSize()
                    .then(if (applyStatusBarPadding) Modifier.statusBarsPadding() else Modifier),
        ) {
            content()
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ScaffoldPreview() {
    AcerolaTheme {
        Acerola.Component.Scaffold {
            Text("Scaffold Content")
        }
    }
}
