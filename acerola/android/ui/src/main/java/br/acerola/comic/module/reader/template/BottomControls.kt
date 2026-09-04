package br.acerola.comic.module.reader.template

import android.content.res.Configuration
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowLeft
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.automirrored.filled.NavigateBefore
import androidx.compose.material.icons.automirrored.filled.NavigateNext
import androidx.compose.material3.Icon
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.GlassButton
import br.acerola.comic.common.ux.modifier.glass
import br.acerola.comic.common.ux.modifier.glassContainer
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.ShapeTokens
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.logging.AcerolaLogger
import br.acerola.comic.logging.LogSource
import br.acerola.comic.module.reader.Reader
import br.acerola.comic.ui.R

private const val TAG = "ReaderBottomControls"

@Composable
fun Reader.Template.BottomControls(
    pageCount: Int,
    currentPage: Int,
    onPrevClick: () -> Unit,
    onNextClick: () -> Unit,
    onNextChapterClick: () -> Unit,
    onPreviousChapterClick: () -> Unit,
    isChapterRead: Boolean = false,
    hasNextChapter: Boolean = false,
    hasPreviousChapter: Boolean = false,
    enableNavigation: Boolean = true,
    isLoading: Boolean = false,
) {
    AcerolaLogger.d(
        TAG,
        "Recomposed: isChapterRead=$isChapterRead, hasNextChapter=$hasNextChapter, page=$currentPage/$pageCount",
        LogSource.UI,
    )

    val glassColor = MaterialTheme.colorScheme.surface.copy(alpha = 0.85f)
    val borderColor = MaterialTheme.colorScheme.outline.copy(alpha = 0.15f)
    val shape = ShapeTokens.Giant

    Box(
        modifier =
            Modifier
                .fillMaxWidth()
                .navigationBarsPadding()
                .padding(SpacingTokens.Large),
        contentAlignment = Alignment.Center,
    ) {
        Box(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .glassContainer(shape),
        ) {
            Box(
                modifier =
                    Modifier
                        .matchParentSize()
                        .glass(shape, glassColor, borderColor),
            )

            Column(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .animateContentSize()
                        .padding(horizontal = SpacingTokens.Huge, vertical = SpacingTokens.Large),
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                PageIndicator(currentPage = currentPage, pageCount = pageCount)

                Spacer(modifier = Modifier.height(SpacingTokens.Large))

                ChapterNavigation(
                    isLoading = isLoading,
                    enableNavigation = enableNavigation,
                    currentPage = currentPage,
                    pageCount = pageCount,
                    hasPreviousChapter = hasPreviousChapter,
                    hasNextChapter = hasNextChapter,
                    isChapterRead = isChapterRead,
                    onPreviousChapterClick = onPreviousChapterClick,
                    onPrevClick = onPrevClick,
                    onNextClick = onNextClick,
                    onNextChapterClick = onNextChapterClick,
                )
            }
        }
    }
}

@Composable
private fun PageIndicator(
    currentPage: Int,
    pageCount: Int,
) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.Bottom,
        ) {
            Text(
                text = stringResource(id = R.string.label_reader_pages_prefix),
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.7f),
            )
            Text(
                text = "${currentPage + 1} / $pageCount",
                style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.Bold),
                color = MaterialTheme.colorScheme.primary,
            )
        }

        Spacer(modifier = Modifier.height(SpacingTokens.Small))

        val progress = if (pageCount > 0) (currentPage + 1).toFloat() / pageCount.toFloat() else 0f
        LinearProgressIndicator(
            progress = { progress },
            modifier =
                Modifier
                    .fillMaxWidth()
                    .height(6.dp)
                    .clip(RoundedCornerShape(3.dp)),
            color = MaterialTheme.colorScheme.primary,
            trackColor = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.1f),
        )
    }
}

@Composable
private fun ChapterNavigation(
    isLoading: Boolean,
    enableNavigation: Boolean,
    currentPage: Int,
    pageCount: Int,
    hasPreviousChapter: Boolean,
    hasNextChapter: Boolean,
    isChapterRead: Boolean,
    onPreviousChapterClick: () -> Unit,
    onPrevClick: () -> Unit,
    onNextClick: () -> Unit,
    onNextChapterClick: () -> Unit,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(modifier = Modifier.weight(1f), contentAlignment = Alignment.CenterStart) {
            PreviousChapterButton(
                visible = hasPreviousChapter,
                isLoading = isLoading,
                onClick = onPreviousChapterClick,
            )
        }

        if (enableNavigation) {
            PageNavigationControls(
                currentPage = currentPage,
                pageCount = pageCount,
                onPrevClick = onPrevClick,
                onNextClick = onNextClick,
            )
        }

        Box(modifier = Modifier.weight(1f), contentAlignment = Alignment.CenterEnd) {
            NextChapterButton(
                visible = hasNextChapter,
                isLoading = isLoading,
                onClick = onNextChapterClick,
            )
        }
    }
}

@Composable
private fun PreviousChapterButton(
    visible: Boolean,
    isLoading: Boolean,
    onClick: () -> Unit,
) {
    if (visible) {
        Box(
            modifier =
                Modifier
                    .clip(ShapeTokens.ExtraLarge)
                    .background(
                        MaterialTheme.colorScheme.onSurface.copy(alpha = if (isLoading) 0.05f else 0.1f),
                    ).clickable(enabled = !isLoading, onClick = onClick)
                    .padding(horizontal = SpacingTokens.Medium, vertical = SpacingTokens.Small),
            contentAlignment = Alignment.Center,
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Icon(
                    imageVector = Icons.AutoMirrored.Filled.NavigateBefore,
                    contentDescription = null,
                    tint =
                        if (isLoading) {
                            MaterialTheme.colorScheme.onSurface.copy(
                                alpha = 0.3f,
                            )
                        } else {
                            MaterialTheme.colorScheme.onSurface
                        },
                    modifier = Modifier.size(SizeTokens.IconSmall),
                )
                Text(
                    text = stringResource(id = R.string.label_reader_previous_chapter),
                    style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Bold),
                    color =
                        if (isLoading) {
                            MaterialTheme.colorScheme.onSurface.copy(
                                alpha = 0.3f,
                            )
                        } else {
                            MaterialTheme.colorScheme.onSurface
                        },
                    maxLines = 1,
                    modifier = Modifier.padding(start = SpacingTokens.ExtraSmall),
                )
            }
        }
    }
}

@Composable
private fun NextChapterButton(
    visible: Boolean,
    isLoading: Boolean,
    onClick: () -> Unit,
) {
    if (visible) {
        Box(
            modifier =
                Modifier
                    .clip(ShapeTokens.ExtraLarge)
                    .background(
                        if (isLoading) {
                            MaterialTheme.colorScheme.primary.copy(
                                alpha = 0.5f,
                            )
                        } else {
                            MaterialTheme.colorScheme.primary
                        },
                    ).clickable(enabled = !isLoading, onClick = onClick)
                    .padding(horizontal = SpacingTokens.Medium, vertical = SpacingTokens.Small),
            contentAlignment = Alignment.Center,
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = stringResource(id = R.string.label_reader_next_chapter),
                    style =
                        MaterialTheme.typography.labelSmall.copy(
                            fontWeight = FontWeight.ExtraBold,
                        ),
                    color =
                        if (isLoading) {
                            MaterialTheme.colorScheme.onPrimary.copy(
                                alpha = 0.5f,
                            )
                        } else {
                            MaterialTheme.colorScheme.onPrimary
                        },
                    maxLines = 1,
                    modifier = Modifier.padding(end = SpacingTokens.ExtraSmall),
                )
                Icon(
                    imageVector = Icons.AutoMirrored.Filled.NavigateNext,
                    contentDescription = null,
                    tint =
                        if (isLoading) {
                            MaterialTheme.colorScheme.onPrimary.copy(
                                alpha = 0.5f,
                            )
                        } else {
                            MaterialTheme.colorScheme.onPrimary
                        },
                    modifier = Modifier.size(SizeTokens.IconSmall),
                )
            }
        }
    }
}

@Composable
private fun PageNavigationControls(
    currentPage: Int,
    pageCount: Int,
    onPrevClick: () -> Unit,
    onNextClick: () -> Unit,
) {
    Row(
        modifier = Modifier.padding(horizontal = SpacingTokens.Small),
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Acerola.Component.GlassButton(
            modifier = Modifier.size(SizeTokens.ClickTargetSmall),
            onClick = onPrevClick,
            icon = {
                Icon(
                    imageVector = Icons.AutoMirrored.Filled.KeyboardArrowLeft,
                    contentDescription =
                        stringResource(
                            id = R.string.description_icon_pagination_previous,
                        ),
                    tint =
                        if (currentPage > 0) {
                            MaterialTheme.colorScheme.onSurface
                        } else {
                            MaterialTheme.colorScheme.onSurface.copy(
                                alpha = 0.3f,
                            )
                        },
                )
            },
        )

        Spacer(modifier = Modifier.width(SpacingTokens.Medium))

        Acerola.Component.GlassButton(
            modifier = Modifier.size(SizeTokens.ClickTargetSmall),
            onClick = onNextClick,
            icon = {
                Icon(
                    imageVector = Icons.AutoMirrored.Filled.KeyboardArrowRight,
                    contentDescription =
                        stringResource(
                            id = R.string.description_icon_pagination_next,
                        ),
                    tint =
                        if (currentPage <
                            pageCount - 1
                        ) {
                            MaterialTheme.colorScheme.onSurface
                        } else {
                            MaterialTheme.colorScheme.onSurface.copy(
                                alpha = 0.3f,
                            )
                        },
                )
            },
        )
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun BottomControlsPreview() {
    AcerolaTheme {
        Reader.Template.BottomControls(
            pageCount = 10,
            currentPage = 1,
            onPrevClick = {},
            onNextClick = {},
            onNextChapterClick = {},
            onPreviousChapterClick = {},
        )
    }
}
