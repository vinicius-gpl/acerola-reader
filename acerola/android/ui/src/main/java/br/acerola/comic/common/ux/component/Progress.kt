package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.ExperimentalAnimationApi
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.CheckCircle
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.ShapeTokens
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.ui.R
import kotlinx.coroutines.delay

@OptIn(ExperimentalAnimationApi::class)
@Composable
fun Acerola.Component.Progress(
    modifier: Modifier = Modifier,
    progress: Float? = null,
    isLoading: Boolean,
) {
    var showIndicator by remember { mutableStateOf(false) }
    var isFinished by remember { mutableStateOf(false) }

    LaunchedEffect(isLoading) {
        if (isLoading) {
            isFinished = false
            showIndicator = true
        } else {
            if (showIndicator) {
                isFinished = true
                delay(2000)
                showIndicator = false
                isFinished = false
            }
        }
    }

    AnimatedVisibility(
        visible = showIndicator,
        enter = fadeIn() + slideInVertically { it / 2 },
        exit = fadeOut() + slideOutVertically { it / 2 },
        modifier = modifier,
    ) {
        Box(
            contentAlignment = Alignment.BottomStart,
        ) {
            Card(
                elevation = CardDefaults.elevatedCardElevation(defaultElevation = SpacingTokens.Small),
                colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
                shape = ShapeTokens.Large,
            ) {
                Row(
                    modifier = Modifier.padding(horizontal = SpacingTokens.ExtraLarge, vertical = SpacingTokens.Medium),
                    horizontalArrangement = Arrangement.spacedBy(space = SpacingTokens.Large),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    AnimatedContent(
                        targetState = isFinished,
                        transitionSpec = {
                            fadeIn(animationSpec = tween(300)) togetherWith
                                fadeOut(animationSpec = tween(300))
                        },
                        label = "SyncStatusIcon",
                    ) { finished ->
                        if (finished) {
                            Icon(
                                imageVector = Icons.Rounded.CheckCircle,
                                contentDescription = null,
                                tint = MaterialTheme.colorScheme.primary,
                                modifier = Modifier.size(SizeTokens.IconMedium),
                            )
                        } else {
                            if (progress == null) {
                                CircularProgressIndicator(
                                    modifier = Modifier.size(size = SizeTokens.IconMedium),
                                    color = MaterialTheme.colorScheme.primary,
                                    strokeWidth = 3.dp,
                                )
                            } else {
                                CircularProgressIndicator(
                                    progress = { progress.coerceIn(0f, 1f) },
                                    modifier = Modifier.size(size = SizeTokens.IconMedium),
                                    color = MaterialTheme.colorScheme.primary,
                                    strokeWidth = 3.dp,
                                )
                            }
                        }
                    }

                    AnimatedContent(
                        targetState = isFinished,
                        label = "SyncStatusText",
                        transitionSpec = {
                            fadeIn(animationSpec = tween(300)) togetherWith
                                fadeOut(animationSpec = tween(300))
                        },
                    ) { finished ->
                        val text =
                            if (finished) {
                                stringResource(id = R.string.label_sync_complete)
                            } else {
                                if (progress != null) {
                                    stringResource(
                                        id = R.string.label_sync_progress_percent,
                                        (progress.coerceIn(0f, 1f) * 100).toInt(),
                                    )
                                } else {
                                    stringResource(id = R.string.label_sync_progress)
                                }
                            }

                        Text(
                            text = text,
                            style = MaterialTheme.typography.bodyMedium,
                            color = if (finished) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurface,
                        )
                    }
                }
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ProgressPreview() {
    AcerolaTheme {
        Acerola.Component.Progress(isLoading = true)
    }
}
