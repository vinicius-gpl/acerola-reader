package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.spring
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Star
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.ShapeTokens
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens

private val ToggleCardShape = ShapeTokens.Large

// Card clicável com estado ativo/inativo (borda+fundo tintados, selo de check animado) e
// conteúdo expansível opcional — port do AcerolaToggleCard do desktop
// (acerola-toggle-card.svelte). Reaproveita a mesma fórmula visual já usada em `ThemeCard`
// (module/main/config/component/ThemeSettings.kt), só generalizada como componente de design
// system em vez de ficar privada a um único arquivo.
@Composable
fun Acerola.Component.ToggleCard(
    title: String,
    active: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    subtitle: String? = null,
    enabled: Boolean = true,
    expanded: Boolean = false,
    icon: @Composable (() -> Unit)? = null,
    content: (@Composable ColumnScope.() -> Unit)? = null,
) {
    val borderColor = if (active) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.outlineVariant
    val containerColor =
        if (active) MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.3f) else MaterialTheme.colorScheme.surface

    Surface(
        shape = ToggleCardShape,
        color = containerColor,
        border = BorderStroke(SizeTokens.BorderMedium, borderColor),
        modifier = modifier.fillMaxWidth().clip(ToggleCardShape),
    ) {
        Column {
            Row(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .alpha(if (enabled) 1f else 0.5f)
                        .clickable(enabled = enabled, onClick = onClick)
                        .padding(SpacingTokens.Large),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                if (icon != null) {
                    Surface(
                        shape = ShapeTokens.Large,
                        color = MaterialTheme.colorScheme.surfaceVariant,
                        modifier = Modifier.size(SizeTokens.ClickTarget),
                    ) {
                        Box(contentAlignment = Alignment.Center) { icon() }
                    }
                    Spacer(modifier = Modifier.width(SpacingTokens.Large))
                }

                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = title,
                        style = MaterialTheme.typography.titleSmall,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onSurface,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    if (subtitle != null) {
                        Text(
                            text = subtitle,
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            maxLines = 2,
                            overflow = TextOverflow.Ellipsis,
                        )
                    }
                }

                AnimatedVisibility(
                    visible = active,
                    enter =
                        scaleIn(
                            animationSpec =
                                spring(
                                    stiffness = Spring.StiffnessMediumLow,
                                    dampingRatio = Spring.DampingRatioMediumBouncy,
                                ),
                        ) + fadeIn(),
                    exit = scaleOut() + fadeOut(),
                ) {
                    Surface(
                        shape = ShapeTokens.Full,
                        color = MaterialTheme.colorScheme.primary,
                        modifier = Modifier.size(SizeTokens.IconSmall),
                    ) {
                        Box(contentAlignment = Alignment.Center) {
                            Icon(
                                imageVector = Icons.Default.Check,
                                contentDescription = null,
                                tint = MaterialTheme.colorScheme.onPrimary,
                                modifier = Modifier.size(SizeTokens.IconExtraSmall),
                            )
                        }
                    }
                }
            }

            if (content != null) {
                AnimatedVisibility(
                    visible = expanded,
                    enter = expandVertically() + fadeIn(),
                    exit = shrinkVertically() + fadeOut(),
                ) {
                    Column(
                        modifier = Modifier.padding(SpacingTokens.Medium),
                        verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
                        content = content,
                    )
                }
            }
        }
    }
}

@Preview(name = "Ativo - Light", showBackground = true)
@Preview(name = "Ativo - Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ToggleCardActivePreview() {
    AcerolaTheme {
        Acerola.Component.ToggleCard(
            title = "Usar o relay do Acerola",
            subtitle = "Gerenciado automaticamente (relay.acerola-comic.com).",
            active = true,
            onClick = {},
            icon = { Icon(Icons.Default.Star, contentDescription = null) },
        )
    }
}

@Preview(name = "Inativo - Light", showBackground = true)
@Preview(name = "Inativo - Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ToggleCardInactivePreview() {
    AcerolaTheme {
        Acerola.Component.ToggleCard(
            title = "Usar a Iroh Services",
            subtitle = "Configure um ticket abaixo para ativar essa fonte.",
            active = false,
            enabled = false,
            onClick = {},
            icon = { Icon(Icons.Default.Star, contentDescription = null) },
        )
    }
}

@Preview(name = "Expandido - Light", showBackground = true)
@Composable
private fun ToggleCardExpandedPreview() {
    AcerolaTheme {
        Acerola.Component.ToggleCard(
            title = "Relays próprios",
            subtitle = "2 configurado(s)",
            active = true,
            expanded = true,
            onClick = {},
            icon = { Icon(Icons.Default.Star, contentDescription = null) },
            content = {
                Text(text = "https://relay-a.exemplo.com")
                Text(text = "https://relay-b.exemplo.com")
            },
        )
    }
}
