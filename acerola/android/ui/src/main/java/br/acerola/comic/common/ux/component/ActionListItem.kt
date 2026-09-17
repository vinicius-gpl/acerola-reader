package br.acerola.comic.common.ux.component

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.ListItem
import androidx.compose.material3.ListItemDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.tokens.ShapeTokens
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens

/**
 * Linha de ação reutilizada pelos menus de "3 pontinhos" (do quadrinho e do capítulo) — um
 * `ListItem` clicável com fundo transparente pra ficar dentro de um `Surface` agrupador, e um
 * divisor recuado abaixo (exceto no último item do grupo). Quando `iconBackground` é informado,
 * o ícone ganha um chip colorido (estilo "contorno/heroicon", igual ao HeroButton) em vez de
 * ficar cru — usado pra padronizar com o botão de ação equivalente no Desktop.
 */
@Composable
fun Acerola.Component.ActionListItem(
    icon: ImageVector,
    title: String,
    onClick: () -> Unit,
    subtitle: String? = null,
    tint: Color = MaterialTheme.colorScheme.onSurfaceVariant,
    iconBackground: Color? = null,
    isLast: Boolean = false,
) {
    // Quando há `iconBackground`, `tint` é a cor de contraste do chip (ex.: onAccent) — serve só
    // pro ícone, nunca pro texto, senão o título/subtítulo herda uma cor pensada pra ficar em cima
    // do chip colorido e fica ilegível no fundo neutro do ListItem.
    val titleColor =
        when {
            iconBackground != null -> MaterialTheme.colorScheme.onSurface
            tint == MaterialTheme.colorScheme.onSurfaceVariant -> MaterialTheme.colorScheme.onSurface
            else -> tint
        }
    val subtitleColor = if (iconBackground != null) MaterialTheme.colorScheme.onSurfaceVariant else tint

    ListItem(
        leadingContent = {
            if (iconBackground != null) {
                Surface(shape = ShapeTokens.Medium, color = iconBackground, modifier = Modifier.size(SizeTokens.ClickTargetSmall)) {
                    Box(contentAlignment = Alignment.Center) {
                        Icon(imageVector = icon, contentDescription = null, tint = tint)
                    }
                }
            } else {
                Icon(imageVector = icon, contentDescription = null, tint = tint)
            }
        },
        headlineContent = { Text(text = title, color = titleColor) },
        supportingContent = subtitle?.let { { Text(text = it, color = subtitleColor) } },
        colors = ListItemDefaults.colors(containerColor = Color.Transparent),
        modifier = Modifier.clickable(onClick = onClick),
    )

    if (!isLast) {
        HorizontalDivider(
            modifier = Modifier.padding(start = SpacingTokens.ExtraGiant),
            color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.3f),
        )
    }
}
