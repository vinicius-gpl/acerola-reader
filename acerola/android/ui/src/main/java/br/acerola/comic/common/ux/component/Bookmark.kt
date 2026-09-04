package br.acerola.comic.common.ux.component
import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Outline
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.LayoutDirection
import br.acerola.comic.common.ux.theme.AcerolaTheme

@Composable
fun BookmarkRibbon(
    color: Color,
    modifier: Modifier = Modifier,
) {
    val ribbonShape =
        remember {
            object : Shape {
                override fun createOutline(
                    size: androidx.compose.ui.geometry.Size,
                    layoutDirection: LayoutDirection,
                    density: androidx.compose.ui.unit.Density,
                ): Outline {
                    val path =
                        Path().apply {
                            moveTo(0f, 0f)
                            lineTo(size.width, 0f)
                            lineTo(size.width, size.height)
                            lineTo(size.width / 2f, size.height * 0.75f)
                            lineTo(0f, size.height)
                            close()
                        }
                    return Outline.Generic(path)
                }
            }
        }

    Box(
        modifier =
            modifier.background(
                color = color,
                shape = ribbonShape,
            ),
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun BookmarkRibbonPreview() {
    AcerolaTheme {
        BookmarkRibbon(color = MaterialTheme.colorScheme.primary)
    }
}
