package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.animateFloatAsState
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
import androidx.compose.material.icons.filled.ChevronRight
import androidx.compose.material.icons.filled.Folder
import androidx.compose.material.icons.filled.Star
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.ShapeTokens
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens

private val AccordionShape = ShapeTokens.Huge

// Card colapsável que agrupa vários HeroButton por categoria — mesmo comportamento do
// AcerolaAccordionCard do desktop (ver acerola-accordion-card.svelte): cabeçalho clicável
// com chevron que gira, corpo com a lista de HeroButton some/aparece, e mais de um card
// pode ficar expandido ao mesmo tempo (quem controla isso é o chamador via `expanded`).
@Composable
fun Acerola.Component.AccordionCard(
    title: String,
    expanded: Boolean,
    onToggleExpanded: () -> Unit,
    modifier: Modifier = Modifier,
    description: String? = null,
    // Cor de identidade da categoria — mesma ideia do `CATEGORY_HOVER_BORDER` (chart-N) do
    // desktop, só que sempre visível em vez de só no hover (não existe hover em touch): tinge
    // a borda do card e o fundo do ícone. Papéis do MaterialTheme em vez de hex fixos porque
    // são os únicos tons garantidos por todos os temas do app (Catppuccin, Dracula, Nord...).
    accentColor: Color = MaterialTheme.colorScheme.outline,
    icon: @Composable (() -> Unit)? = null,
    content: @Composable ColumnScope.() -> Unit,
) {
    val chevronRotation by animateFloatAsState(targetValue = if (expanded) 90f else 0f, label = "accordion_chevron_rotation")

    Surface(
        shape = AccordionShape,
        border = BorderStroke(SizeTokens.BorderThin, accentColor.copy(alpha = 0.5f)),
        color = MaterialTheme.colorScheme.surface,
        modifier =
            modifier
                .fillMaxWidth()
                .clip(AccordionShape),
    ) {
        Column(modifier = Modifier.animateContentSize()) {
            Row(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .clickable(onClick = onToggleExpanded)
                        .padding(SpacingTokens.ExtraLarge),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                if (icon != null) {
                    Surface(
                        shape = ShapeTokens.Large,
                        color = accentColor.copy(alpha = 0.12f),
                        modifier = Modifier.size(SizeTokens.ClickTarget),
                    ) {
                        Box(contentAlignment = Alignment.Center) {
                            icon()
                        }
                    }

                    Spacer(modifier = Modifier.width(SpacingTokens.Large))
                }

                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = title,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        style = MaterialTheme.typography.titleMedium,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onSurface,
                    )

                    if (description != null) {
                        Text(
                            text = description,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }

                Spacer(modifier = Modifier.width(SpacingTokens.Small))

                Icon(
                    imageVector = Icons.Default.ChevronRight,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier =
                        Modifier
                            .size(SizeTokens.IconMedium)
                            .graphicsLayer { rotationZ = chevronRotation },
                )
            }

            if (expanded) {
                HorizontalDivider(
                    modifier =
                        Modifier
                            .padding(horizontal = SpacingTokens.ExtraLarge)
                            .alpha(0.4f),
                )

                Column(
                    modifier = Modifier.padding(SpacingTokens.Medium),
                    verticalArrangement = Arrangement.spacedBy(SpacingTokens.Medium),
                    content = content,
                )
            }
        }
    }
}

@Composable
fun Acerola.Component.AccordionCard(
    title: String,
    icon: ImageVector,
    expanded: Boolean,
    onToggleExpanded: () -> Unit,
    modifier: Modifier = Modifier,
    description: String? = null,
    accentColor: Color = MaterialTheme.colorScheme.outline,
    content: @Composable ColumnScope.() -> Unit,
) {
    Acerola.Component.AccordionCard(
        title = title,
        expanded = expanded,
        onToggleExpanded = onToggleExpanded,
        modifier = modifier,
        description = description,
        accentColor = accentColor,
        icon = {
            Icon(
                imageVector = icon,
                contentDescription = null,
                tint = accentColor,
                modifier = Modifier.size(SizeTokens.IconMedium),
            )
        },
        content = content,
    )
}

@Preview(name = "Colapsado - Light", showBackground = true)
@Preview(name = "Colapsado - Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun AccordionCardCollapsedPreview() {
    AcerolaTheme {
        Acerola.Component.AccordionCard(
            title = "Arquivos",
            icon = Icons.Default.Folder,
            accentColor = MaterialTheme.colorScheme.primary,
            expanded = false,
            onToggleExpanded = {},
        ) {
            Acerola.Component.HeroButton(title = "Selecionar pasta", icon = Icons.Default.Folder)
        }
    }
}

@Preview(name = "Expandido - Light", showBackground = true)
@Preview(name = "Expandido - Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun AccordionCardExpandedPreview() {
    AcerolaTheme {
        Acerola.Component.AccordionCard(
            title = "Aparência",
            description = "Tema e cores do app",
            icon = Icons.Default.Star,
            accentColor = MaterialTheme.colorScheme.tertiary,
            expanded = true,
            onToggleExpanded = {},
        ) {
            Acerola.Component.HeroButton(title = "Tema escuro", icon = Icons.Default.Star)
            Acerola.Component.HeroButton(title = "Cor de destaque", icon = Icons.Default.Star)
        }
    }
}
