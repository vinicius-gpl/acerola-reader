package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Star
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.ShapeTokens
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens

private val HeroShape = ShapeTokens.Huge
private val IconShape = ShapeTokens.Large

@Composable
fun Acerola.Component.HeroButton(
    title: String,
    icon: ImageVector,
    modifier: Modifier = Modifier,
    description: String? = null,
    iconTint: Color = MaterialTheme.colorScheme.onPrimaryContainer,
    iconBackground: Color = MaterialTheme.colorScheme.primaryContainer,
    onClick: (() -> Unit)? = null,
    onLongClick: (() -> Unit)? = null,
    action: @Composable (() -> Unit)? = null,
    bottomContent: @Composable (() -> Unit)? = null,
) {
    Acerola.Component.HeroButton(
        title = title,
        modifier = modifier,
        description = description,
        iconBackground = iconBackground,
        onClick = onClick,
        onLongClick = onLongClick,
        action = action,
        bottomContent = bottomContent,
        icon = {
            Icon(
                imageVector = icon,
                contentDescription = null,
                tint = iconTint,
                modifier = Modifier.size(SizeTokens.IconMedium),
            )
        },
    )
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun Acerola.Component.HeroButton(
    title: String,
    modifier: Modifier = Modifier,
    description: String? = null,
    iconBackground: Color = MaterialTheme.colorScheme.primaryContainer,
    iconModifier: Modifier = Modifier.size(SizeTokens.ClickTarget),
    onClick: (() -> Unit)? = null,
    onLongClick: (() -> Unit)? = null,
    action: @Composable (() -> Unit)? = null,
    bottomContent: @Composable (() -> Unit)? = null,
    icon: @Composable () -> Unit,
) {
    val clickableModifier =
        if (onClick != null || onLongClick != null) {
            Modifier.combinedClickable(
                onClick = onClick ?: {},
                onLongClick = onLongClick,
            )
        } else {
            Modifier
        }

    Surface(
        shape = HeroShape,
        border = BorderStroke(SizeTokens.BorderThin, MaterialTheme.colorScheme.outlineVariant),
        color = MaterialTheme.colorScheme.surface,
        modifier =
            modifier
                .fillMaxWidth()
                .clip(HeroShape)
                .then(clickableModifier),
    ) {
        HeroButtonContent(title, description, iconBackground, iconModifier, action, bottomContent, icon)
    }
}

@Composable
fun Acerola.Component.HeroNestedButton(
    title: String,
    modifier: Modifier = Modifier,
    description: String? = null,
    iconBackground: Color = MaterialTheme.colorScheme.primaryContainer,
    onClick: () -> Unit,
    icon: @Composable () -> Unit,
) {
    Surface(
        onClick = onClick,
        color = Color.Transparent,
        modifier = modifier.fillMaxWidth(),
    ) {
        Row(
            modifier = Modifier.padding(horizontal = SpacingTokens.ExtraLarge, vertical = SpacingTokens.Medium),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Surface(
                shape = ShapeTokens.MediumLarge,
                color = iconBackground,
                modifier = Modifier.size(40.dp),
            ) {
                Box(contentAlignment = Alignment.Center) {
                    icon()
                }
            }
            Spacer(modifier = Modifier.width(SpacingTokens.Large))
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = title,
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.Medium,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                if (description != null) {
                    Text(
                        text = description,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        }
    }
}

@Composable
fun Acerola.Component.HeroNestedButton(
    title: String,
    icon: ImageVector,
    modifier: Modifier = Modifier,
    description: String? = null,
    iconTint: Color = MaterialTheme.colorScheme.onPrimaryContainer,
    iconBackground: Color = MaterialTheme.colorScheme.primaryContainer,
    onClick: () -> Unit,
) {
    Acerola.Component.HeroNestedButton(
        title = title,
        description = description,
        modifier = modifier,
        iconBackground = iconBackground,
        onClick = onClick,
        icon = {
            Icon(
                imageVector = icon,
                contentDescription = null,
                tint = iconTint,
                modifier = Modifier.size(SizeTokens.IconSmall),
            )
        },
    )
}

@Composable
fun Acerola.Component.GroupedHeroButton(
    title: String,
    modifier: Modifier = Modifier,
    description: String? = null,
    iconBackground: Color = MaterialTheme.colorScheme.primaryContainer,
    iconModifier: Modifier = Modifier.size(SizeTokens.ClickTarget),
    onClick: (() -> Unit)? = null,
    onLongClick: (() -> Unit)? = null,
    action: @Composable (() -> Unit)? = null,
    nestedItem: @Composable (() -> Unit)? = null,
    icon: @Composable () -> Unit,
) {
    Acerola.Component.HeroButton(
        title = title,
        modifier = modifier,
        description = description,
        iconBackground = iconBackground,
        iconModifier = iconModifier,
        onClick = onClick,
        onLongClick = onLongClick,
        action = action,
        bottomContent = nestedItem,
        icon = icon,
    )
}

@Composable
fun Acerola.Component.GroupedHeroButton(
    title: String,
    icon: ImageVector,
    modifier: Modifier = Modifier,
    description: String? = null,
    iconTint: Color = MaterialTheme.colorScheme.onPrimaryContainer,
    iconBackground: Color = MaterialTheme.colorScheme.primaryContainer,
    onClick: (() -> Unit)? = null,
    onLongClick: (() -> Unit)? = null,
    action: @Composable (() -> Unit)? = null,
    nestedItem: @Composable (() -> Unit)? = null,
) {
    Acerola.Component.HeroButton(
        title = title,
        icon = icon,
        modifier = modifier,
        description = description,
        iconTint = iconTint,
        iconBackground = iconBackground,
        onClick = onClick,
        onLongClick = onLongClick,
        action = action,
        bottomContent = nestedItem,
    )
}

@Composable
private fun HeroButtonContent(
    title: String,
    description: String?,
    iconBackground: Color,
    iconModifier: Modifier,
    action: @Composable (() -> Unit)?,
    bottomContent: @Composable (() -> Unit)?,
    icon: @Composable () -> Unit,
) {
    Column {
        Row(
            modifier = Modifier.padding(horizontal = SpacingTokens.ExtraLarge, vertical = SpacingTokens.Large),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Surface(
                shape = IconShape,
                color = iconBackground,
                modifier = iconModifier,
            ) {
                Box(contentAlignment = Alignment.Center) {
                    icon()
                }
            }

            Spacer(modifier = Modifier.width(SpacingTokens.Large))

            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = title,
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                if (description != null) {
                    Text(
                        text = description,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }

            if (action != null) {
                Spacer(modifier = Modifier.width(SpacingTokens.Medium))
                action()
            }
        }

        if (bottomContent != null) {
            HorizontalDivider(
                modifier =
                    Modifier
                        .padding(horizontal = SpacingTokens.ExtraLarge)
                        .alpha(0.4f),
            )
            bottomContent()
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun HeroButtonWithVectorPreview() {
    AcerolaTheme {
        Acerola.Component.HeroButton(
            title = "Hero Title",
            icon = Icons.Default.Star,
        )
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun HeroButtonWithComposablePreview() {
    AcerolaTheme {
        Acerola.Component.HeroButton(
            title = "Hero Title",
            icon = { Icon(Icons.Default.Star, contentDescription = null) },
        )
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun HeroNestedButtonWithComposablePreview() {
    AcerolaTheme {
        Acerola.Component.HeroNestedButton(
            title = "Nested Title",
            onClick = {},
            icon = { Icon(Icons.Default.Star, contentDescription = null) },
        )
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun HeroNestedButtonWithVectorPreview() {
    AcerolaTheme {
        Acerola.Component.HeroNestedButton(
            title = "Nested Title",
            icon = Icons.Default.Star,
            onClick = {},
        )
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun GroupedHeroButtonWithComposablePreview() {
    AcerolaTheme {
        Acerola.Component.GroupedHeroButton(
            title = "Grouped Title",
            icon = { Icon(Icons.Default.Star, contentDescription = null) },
        )
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun GroupedHeroButtonWithVectorPreview() {
    AcerolaTheme {
        Acerola.Component.GroupedHeroButton(
            title = "Grouped Title",
            icon = Icons.Default.Star,
        )
    }
}
