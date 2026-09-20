package br.acerola.comic.common.ux.component

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.tokens.ShapeTokens
import br.acerola.comic.common.ux.tokens.SizeTokens

@Composable
fun Acerola.Component.ActionIcon(
    icon: ImageVector,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    contentDescription: String? = null,
    iconTint: Color = MaterialTheme.colorScheme.onSurfaceVariant,
    iconBackground: Color = MaterialTheme.colorScheme.surfaceVariant,
) {
    val resolvedBackground = if (enabled) iconBackground else iconBackground.copy(alpha = 0.4f)
    val resolvedTint = if (enabled) iconTint else iconTint.copy(alpha = 0.4f)

    Surface(
        shape = ShapeTokens.Medium,
        color = resolvedBackground,
        modifier =
            modifier
                .size(SizeTokens.ClickTargetSmall)
                .clip(ShapeTokens.Medium)
                .clickable(enabled = enabled, onClick = onClick),
    ) {
        Box(contentAlignment = Alignment.Center) {
            Icon(imageVector = icon, contentDescription = contentDescription, tint = resolvedTint)
        }
    }
}
