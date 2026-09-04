package br.acerola.comic.common.ux.component
import android.content.res.Configuration
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.sp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.modifier.glass
import br.acerola.comic.common.ux.modifier.glassContainer
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.ShapeTokens
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens

@Composable
fun Acerola.Component.TopBar(
    title: String? = null,
    modifier: Modifier = Modifier,
    navigationIcon: @Composable (() -> Unit)? = null,
    actions: @Composable (() -> Unit)? = null,
) {
    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .windowInsetsPadding(WindowInsets.statusBars)
                .padding(horizontal = SpacingTokens.Large, vertical = SpacingTokens.Small),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            modifier = Modifier.size(SizeTokens.ClickTarget),
            contentAlignment = Alignment.CenterStart,
        ) {
            navigationIcon?.invoke()
        }

        Box(
            modifier =
                Modifier
                    .weight(1f)
                    .padding(horizontal = SpacingTokens.Medium),
            contentAlignment = Alignment.Center,
        ) {
            if (title != null) {
                TitleCapsule(text = title)
            }
        }

        Box(
            modifier = Modifier.size(SizeTokens.ClickTarget),
            contentAlignment = Alignment.CenterEnd,
        ) {
            actions?.invoke()
        }
    }
}

@Composable
fun Acerola.Component.TitleCapsule(
    text: String,
    modifier: Modifier = Modifier,
) {
    val borderColor = MaterialTheme.colorScheme.outline.copy(alpha = 0.15f)
    val glassColor = MaterialTheme.colorScheme.surface.copy(alpha = 0.65f)
    val shape = ShapeTokens.Huge

    Box(
        modifier =
            modifier
                .fillMaxWidth()
                .wrapContentWidth(Alignment.CenterHorizontally)
                .glassContainer(shape),
    ) {
        Box(
            modifier =
                Modifier
                    .matchParentSize()
                    .glass(shape, glassColor, borderColor),
        )

        Text(
            text = text,
            modifier = Modifier.padding(horizontal = SpacingTokens.Large, vertical = SpacingTokens.Small),
            style =
                MaterialTheme.typography.titleMedium.copy(
                    fontWeight = FontWeight.SemiBold,
                    letterSpacing = 0.5.sp,
                    color = MaterialTheme.colorScheme.onSurface,
                ),
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun TopBarPreview() {
    AcerolaTheme {
        Acerola.Component.TopBar(title = "Title")
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun TitleCapsulePreview() {
    AcerolaTheme {
        Acerola.Component.TitleCapsule(text = "Capsule")
    }
}
