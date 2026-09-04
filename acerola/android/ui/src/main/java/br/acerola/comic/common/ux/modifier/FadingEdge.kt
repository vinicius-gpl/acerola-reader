package br.acerola.comic.common.ux.modifier

import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

// Sinaliza visualmente que uma LazyRow continua além da borda visível — sem isso, se o
// item couber um número exato de vezes na largura da tela (caso do carrossel de temas em
// ThemeSettings.kt), a lista parece uma grade fechada em vez de algo pra arrastar.
// `edgeColor` deve ser a cor de fundo por trás da lista, pra dissolver nela.
fun Modifier.horizontalScrollFade(
    state: LazyListState,
    edgeColor: Color,
    edgeWidth: Dp = 32.dp,
): Modifier =
    this.drawWithContent {
        drawContent()

        val fadeWidthPx = edgeWidth.toPx()

        if (state.canScrollBackward) {
            drawRect(
                brush = Brush.horizontalGradient(colors = listOf(edgeColor, Color.Transparent), endX = fadeWidthPx),
            )
        }

        if (state.canScrollForward) {
            drawRect(
                brush =
                    Brush.horizontalGradient(
                        colors = listOf(Color.Transparent, edgeColor),
                        startX = size.width - fadeWidthPx,
                    ),
            )
        }
    }
