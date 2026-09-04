package br.acerola.comic.module.main.home.component

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.CornerSize
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.rounded.AutoStories
import androidx.compose.material.icons.rounded.Star
import androidx.compose.material.icons.rounded.VisibilityOff
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.BookmarkRibbon
import br.acerola.comic.common.ux.component.ImageCard
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SemanticColors
import br.acerola.comic.common.ux.tokens.ShapeTokens
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.dto.ComicDto
import br.acerola.comic.dto.archive.ComicDirectoryDto
import br.acerola.comic.dto.history.ReadingHistoryDto
import br.acerola.comic.module.main.Main
import br.acerola.comic.pattern.metadata.MetadataSource
import br.acerola.comic.ui.R
import coil.compose.rememberAsyncImagePainter
import coil.request.ImageRequest
import coil.size.Size
import coil.size.SizeResolver

@Composable
fun Main.Home.Component.ComicGridItem(
    comic: ComicDto,
    history: ReadingHistoryDto? = null,
    chapterCount: Int = 0,
    isSelected: Boolean = false,
    isSelectionMode: Boolean = false,
    onLongClick: (() -> Unit)? = null,
    onShowActions: () -> Unit = {},
    onClick: () -> Unit,
) {
    val context = LocalContext.current
    val density = LocalDensity.current

    val coverUri = comic.directory.coverUri ?: comic.directory.bannerUri
    val title = comic.remoteInfo?.title ?: comic.directory.name

    val imageSize: Size =
        with(receiver = density) {
            Size(
                width = SizeTokens.ComicCardWidth.toPx().toInt(),
                height = SizeTokens.ComicCardHeight.toPx().toInt(),
            )
        }

    val placeholderPainter =
        rememberAsyncImagePainter(
            model =
                ImageRequest
                    .Builder(context)
                    .data(data = R.raw.placeholder_comic)
                    .size(resolver = SizeResolver(imageSize))
                    .build(),
        )

    val coverPainter =
        rememberAsyncImagePainter(
            placeholder = placeholderPainter,
            fallback = placeholderPainter,
            error = placeholderPainter,
            model =
                ImageRequest
                    .Builder(context)
                    .data(data = coverUri)
                    .memoryCacheKey("${coverUri}_${comic.directory.lastModified}")
                    .diskCacheKey("${coverUri}_${comic.directory.lastModified}")
                    .size(resolver = SizeResolver(imageSize))
                    .build(),
        )

    val categoryColor = comic.category?.color
    val score =
        comic.remoteInfo
            ?.sources
            ?.anilist
            ?.averageScore
            ?.let { it / 10f }

    Column(
        modifier =
            Modifier
                .padding(all = SpacingTokens.ExtraSmall)
                .width(width = SizeTokens.ComicCardWidth),
        horizontalAlignment = Alignment.Start,
    ) {
        Box(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .aspectRatio(ratio = 2f / 3f),
        ) {
            Acerola.Component.ImageCard(
                onClick = onClick,
                onLongClick = onLongClick,
                image = coverPainter,
                modifier =
                    Modifier
                        .fillMaxSize()
                        .padding(top = SpacingTokens.Small),
            )

            Box(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .height(40.dp)
                        .align(Alignment.BottomCenter)
                        .clip(ShapeTokens.Medium.copy(topStart = CornerSize(0.dp), topEnd = CornerSize(0.dp)))
                        .background(
                            Brush.verticalGradient(
                                colors = listOf(Color.Transparent, Color.Black.copy(alpha = 0.5f)),
                            ),
                        ),
            )

            Box(
                modifier =
                    Modifier
                        .fillMaxSize()
                        .padding(bottom = SpacingTokens.Small, end = SpacingTokens.Small),
                contentAlignment = Alignment.BottomEnd,
            ) {
                val sourceIcon =
                    when (comic.remoteInfo?.syncSource) {
                        MetadataSource.MANGADEX -> R.drawable.mangadex_v2
                        MetadataSource.ANILIST -> R.drawable.anilist
                        else -> null
                    }
                if (sourceIcon != null) {
                    Icon(
                        painter = painterResource(id = sourceIcon),
                        contentDescription = null,
                        tint = Color.Unspecified,
                        modifier = Modifier.size(SizeTokens.IconExtraSmall),
                    )
                }
            }

            if (categoryColor != null) {
                BookmarkRibbon(
                    color = Color(categoryColor),
                    modifier =
                        Modifier
                            .align(Alignment.TopStart)
                            .padding(start = SpacingTokens.Medium)
                            .width(18.dp)
                            .height(SpacingTokens.Giant),
                )
            }

            if (comic.directory.hidden) {
                Icon(
                    imageVector = Icons.Rounded.VisibilityOff,
                    contentDescription = stringResource(id = R.string.description_icon_comic_hidden),
                    tint = Color.White,
                    modifier =
                        Modifier
                            .align(Alignment.BottomStart)
                            .padding(bottom = SpacingTokens.Small, start = SpacingTokens.Small)
                            .size(SizeTokens.IconExtraSmall)
                            .background(
                                color = Color.Black.copy(alpha = 0.5f),
                                shape = ShapeTokens.ExtraSmall,
                            ).padding(2.dp),
                )
            }

            if (isSelectionMode) {
                if (isSelected) {
                    Box(
                        modifier =
                            Modifier
                                .fillMaxSize()
                                .padding(top = SpacingTokens.Small)
                                .clip(ShapeTokens.Medium)
                                .background(MaterialTheme.colorScheme.primary.copy(alpha = 0.35f)),
                        contentAlignment = Alignment.Center,
                    ) {
                        Box(
                            modifier =
                                Modifier
                                    .size(44.dp)
                                    .background(
                                        color = MaterialTheme.colorScheme.primary,
                                        shape = CircleShape,
                                    ),
                            contentAlignment = Alignment.Center,
                        ) {
                            Icon(
                                imageVector = Icons.Default.Check,
                                contentDescription = null,
                                tint = MaterialTheme.colorScheme.onPrimary,
                                modifier = Modifier.size(28.dp),
                            )
                        }
                    }
                } else {
                    Box(
                        modifier =
                            Modifier
                                .fillMaxSize()
                                .padding(top = SpacingTokens.Small)
                                .clip(ShapeTokens.Medium)
                                .background(MaterialTheme.colorScheme.surface.copy(alpha = 0.45f)),
                        contentAlignment = Alignment.Center,
                    ) {
                        Box(
                            modifier =
                                Modifier
                                    .size(36.dp)
                                    .background(
                                        color = Color.Black.copy(alpha = 0.3f),
                                        shape = CircleShape,
                                    ).border(
                                        width = 2.dp,
                                        color = Color.White.copy(alpha = 0.8f),
                                        shape = CircleShape,
                                    ),
                        )
                    }
                }
            }
        }

        Spacer(modifier = Modifier.height(SpacingTokens.Small))

        Text(
            text = title,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.padding(horizontal = SpacingTokens.ExtraSmall),
            color = MaterialTheme.colorScheme.onBackground,
            style = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.Bold),
        )

        Spacer(modifier = Modifier.height(SpacingTokens.ExtraSmall))

        Row(
            modifier = Modifier.fillMaxWidth().padding(horizontal = SpacingTokens.ExtraSmall),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpacingTokens.ExtraSmall)) {
                if (score != null) {
                    Icon(
                        imageVector = Icons.Rounded.Star,
                        contentDescription = null,
                        tint = SemanticColors.Rating,
                        modifier = Modifier.size(14.dp),
                    )
                    Text(
                        text = score.toString(),
                        style = MaterialTheme.typography.labelMedium.copy(fontWeight = FontWeight.Bold),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }

                if (chapterCount > 0) {
                    Icon(
                        imageVector = Icons.Rounded.AutoStories,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.size(14.dp),
                    )
                    Text(
                        text = chapterCount.toString(),
                        style = MaterialTheme.typography.labelMedium.copy(fontWeight = FontWeight.Bold),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }

                if (history?.isCompleted == true) {
                    if (score != null || chapterCount > 0) {
                        Text(
                            text = "•",
                            style = MaterialTheme.typography.labelMedium,
                            color = MaterialTheme.colorScheme.outline,
                        )
                    }
                    Text(
                        text = "100%",
                        style = MaterialTheme.typography.labelMedium.copy(fontWeight = FontWeight.Bold),
                        color = MaterialTheme.colorScheme.primary,
                    )
                }
            }

            IconButton(
                onClick = onShowActions,
                modifier = Modifier.size(SizeTokens.ClickTargetSmall),
            ) {
                Icon(
                    imageVector = Icons.Default.MoreVert,
                    contentDescription = stringResource(id = R.string.description_icon_chapter_more_options),
                    modifier = Modifier.size(SizeTokens.IconSmall),
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ComicGridItemPreview() {
    AcerolaTheme {
        Main.Home.Component.ComicGridItem(
            comic =
                ComicDto(
                    directory =
                        ComicDirectoryDto(
                            id = 1L,
                            name = "Sample Comic",
                            path = "/path",
                            coverUri = null,
                            bannerUri = null,
                            lastModified = 0L,
                            archiveTemplateFk = null,
                        ),
                    category = null,
                    remoteInfo = null,
                ),
            onClick = {},
        )
    }
}
