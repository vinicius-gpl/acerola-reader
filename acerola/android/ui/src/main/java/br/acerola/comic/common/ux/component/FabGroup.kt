package br.acerola.comic.common.ux.component
import android.content.res.Configuration
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Star
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.FloatingActionButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.zIndex
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.ShapeTokens
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens

data class FabGroupItem(
    val icon: @Composable () -> Unit,
    val label: String? = null,
    val onClick: () -> Unit,
)

@Composable
fun Acerola.Component.FabGroup(
    icon: @Composable () -> Unit,
    items: List<FabGroupItem>,
    modifier: Modifier = Modifier,
    paddingFromEdges: Dp = SpacingTokens.Large,
    spacingBetweenItems: Dp = SpacingTokens.MediumLarge,
) {
    var expanded by remember { mutableStateOf(false) }

    Box(
        modifier = modifier.fillMaxSize(),
        contentAlignment = Alignment.BottomEnd,
    ) {
        Column(
            horizontalAlignment = Alignment.End,
            modifier = Modifier.padding(paddingFromEdges),
            verticalArrangement = Arrangement.spacedBy(spacingBetweenItems),
        ) {
            LazyColumn(
                reverseLayout = true,
                contentPadding = PaddingValues(vertical = SpacingTokens.ExtraSmall),
                verticalArrangement = Arrangement.spacedBy(spacingBetweenItems),
                horizontalAlignment = Alignment.End,
                modifier =
                    Modifier
                        .heightIn(max = 400.dp)
                        .graphicsLayer { clip = false }
                        .padding(end = SpacingTokens.ExtraSmall),
            ) {
                itemsIndexed(
                    items = items.reversed(),
                    key = { _, item -> item.label ?: item.hashCode().toString() },
                ) { index, item ->

                    val enterDelay = index * 50
                    val exitDelay = (items.size - 1 - index) * 30
                    val itemZIndex = (items.size - index).toFloat()

                    AnimatedVisibility(
                        visible = expanded,
                        modifier =
                            Modifier
                                .zIndex(itemZIndex)
                                .graphicsLayer { clip = false },
                        enter =
                            slideInHorizontally(
                                animationSpec = tween(delayMillis = enterDelay),
                                initialOffsetX = { it / 2 },
                            ) +
                                fadeIn(
                                    animationSpec = tween(delayMillis = enterDelay),
                                ),
                        exit =
                            slideOutHorizontally(
                                animationSpec = tween(delayMillis = exitDelay),
                                targetOffsetX = { it / 2 },
                            ) +
                                fadeOut(
                                    animationSpec = tween(delayMillis = exitDelay),
                                ),
                    ) {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
                            modifier = Modifier.graphicsLayer { clip = false },
                        ) {
                            item.label?.let { label ->
                                Surface(
                                    shape = ShapeTokens.Small,
                                    color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.9f),
                                    tonalElevation = 2.dp,
                                    modifier =
                                        Modifier.clickable {
                                            item.onClick()
                                            expanded = false
                                        },
                                ) {
                                    Text(
                                        text = label,
                                        style = MaterialTheme.typography.labelLarge,
                                        modifier =
                                            Modifier.padding(
                                                horizontal = SpacingTokens.Small,
                                                vertical = SpacingTokens.ExtraSmall,
                                            ),
                                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                                    )
                                }
                            }

                            FloatingActionButton(
                                modifier = Modifier.size(SizeTokens.ClickTarget),
                                containerColor = MaterialTheme.colorScheme.secondary,
                                elevation =
                                    FloatingActionButtonDefaults.elevation(
                                        defaultElevation = 0.dp,
                                        pressedElevation = 0.dp,
                                        focusedElevation = 0.dp,
                                        hoveredElevation = 0.dp,
                                    ),
                                onClick = {
                                    item.onClick()
                                    expanded = false
                                },
                            ) {
                                item.icon()
                            }
                        }
                    }
                }
            }

            FloatingActionButton(
                modifier = Modifier.size(SizeTokens.FabDefault),
                containerColor = MaterialTheme.colorScheme.primary,
                elevation =
                    FloatingActionButtonDefaults.elevation(
                        defaultElevation = 4.dp,
                        pressedElevation = 8.dp,
                    ),
                onClick = { expanded = !expanded },
            ) {
                icon()
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun FabGroupPreview() {
    AcerolaTheme {
        Acerola.Component.FabGroup(
            icon = { Icon(Icons.Default.Add, contentDescription = null) },
            items =
                listOf(
                    FabGroupItem(
                        icon = { Icon(Icons.Default.Star, contentDescription = null) },
                        label = "Star",
                        onClick = {},
                    ),
                ),
        )
    }
}
