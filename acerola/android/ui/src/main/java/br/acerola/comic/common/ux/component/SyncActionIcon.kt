package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.state.SyncActionVisualState
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.ShapeTokens
import br.acerola.comic.common.ux.tokens.SizeTokens

@Composable
fun Acerola.Component.SyncActionIcon(
    state: SyncActionVisualState,
    modifier: Modifier = Modifier,
    containerSize: Dp = SizeTokens.ClickTarget,
    iconSize: Dp = SizeTokens.IconMedium,
    defaultBackground: Color = MaterialTheme.colorScheme.primaryContainer,
    successBackground: Color = Color(0xFF1B5E20),
    icon: @Composable () -> Unit,
) {
    val backgroundColor by animateColorAsState(
        targetValue =
            when (state) {
                SyncActionVisualState.SUCCESS -> successBackground
                else -> defaultBackground
            },
        animationSpec = tween(durationMillis = 400, easing = FastOutSlowInEasing),
        label = "syncIconBgColor",
    )

    Surface(
        shape = ShapeTokens.Large,
        color = backgroundColor,
        modifier = modifier.size(containerSize),
    ) {
        Box(contentAlignment = Alignment.Center) {
            AnimatedContent(
                targetState = state,
                transitionSpec = {
                    (
                        scaleIn(
                            animationSpec =
                                spring(
                                    stiffness = Spring.StiffnessMediumLow,
                                    dampingRatio = Spring.DampingRatioMediumBouncy,
                                ),
                        ) + fadeIn()
                    ).togetherWith(scaleOut() + fadeOut())
                },
                label = "syncIconTransition",
            ) { visualState ->
                when (visualState) {
                    SyncActionVisualState.LOADING -> {
                        CircularProgressIndicator(
                            modifier = Modifier.size(iconSize),
                            color = MaterialTheme.colorScheme.primary,
                            strokeWidth = 2.5.dp,
                        )
                    }
                    SyncActionVisualState.SUCCESS -> {
                        Icon(
                            imageVector = Icons.Rounded.Check,
                            contentDescription = null,
                            tint = Color.White,
                            modifier = Modifier.size(iconSize),
                        )
                    }
                    SyncActionVisualState.IDLE -> {
                        icon()
                    }
                }
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SyncActionIconPreview() {
    AcerolaTheme {
        Acerola.Component.SyncActionIcon(
            state = SyncActionVisualState.IDLE,
            icon = {},
        )
    }
}
