package br.acerola.comic.common.ux.component

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.ListItem
import androidx.compose.material3.ListItemDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.tokens.SpacingTokens

/**
 * Linha de ação reutilizada pelos menus de "3 pontinhos" (do quadrinho e do capítulo) — um
 * `ListItem` clicável com fundo transparente pra ficar dentro de um `Surface` agrupador, e um
 * divisor recuado abaixo (exceto no último item do grupo).
 */
@Composable
fun Acerola.Component.ActionListItem(
    icon: ImageVector,
    title: String,
    onClick: () -> Unit,
    subtitle: String? = null,
    tint: Color = MaterialTheme.colorScheme.onSurfaceVariant,
    isLast: Boolean = false,
) {
    val titleColor = if (tint == MaterialTheme.colorScheme.onSurfaceVariant) MaterialTheme.colorScheme.onSurface else tint

    ListItem(
        leadingContent = { Icon(imageVector = icon, contentDescription = null, tint = tint) },
        headlineContent = { Text(text = title, color = titleColor) },
        supportingContent = subtitle?.let { { Text(text = it, color = tint) } },
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
