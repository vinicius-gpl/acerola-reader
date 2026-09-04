package br.acerola.comic.module.main.home.component

import android.content.res.Configuration
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.AutoStories
import androidx.compose.material.icons.rounded.Star
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.ImageCard
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.dto.ComicDto
import br.acerola.comic.dto.archive.ComicDirectoryDto
import br.acerola.comic.module.main.Main
import br.acerola.comic.pattern.metadata.MetadataSource
import br.acerola.comic.ui.R
import coil.compose.rememberAsyncImagePainter
import coil.request.ImageRequest
import coil.size.Size
import coil.size.SizeResolver

@Composable
fun Main.Home.Component.ComicSearchItem(
    comic: ComicDto,
    chapterCount: Int,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val context = LocalContext.current
    val density = LocalDensity.current

    val coverUri = comic.directory.coverUri ?: comic.directory.bannerUri
    val title = comic.remoteInfo?.title ?: comic.directory.name

    val score =
        comic.remoteInfo
            ?.sources
            ?.anilist
            ?.averageScore
            ?.let { it / 10f }
    val status =
        comic.remoteInfo
            ?.status
            ?.lowercase()
            ?.replaceFirstChar { it.uppercase() }

    val imageSize =
        with(density) {
            Size(width = 64.dp.toPx().toInt(), height = 96.dp.toPx().toInt())
        }

    val placeholderPainter =
        rememberAsyncImagePainter(
            model =
                ImageRequest
                    .Builder(context)
                    .data(R.raw.placeholder_comic)
                    .size(SizeResolver(imageSize))
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
                    .data(coverUri)
                    .memoryCacheKey("${coverUri}_${comic.directory.lastModified}")
                    .diskCacheKey("${coverUri}_${comic.directory.lastModified}")
                    .size(SizeResolver(imageSize))
                    .build(),
        )

    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .height(110.dp)
                .padding(horizontal = 12.dp, vertical = 6.dp)
                .clip(MaterialTheme.shapes.medium)
                .clickable(onClick = onClick),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Acerola.Component.ImageCard(
            onClick = onClick,
            image = coverPainter,
            modifier =
                Modifier
                    .width(64.dp)
                    .height(96.dp)
                    .clip(MaterialTheme.shapes.small),
        )

        Spacer(modifier = Modifier.width(12.dp))

        Column(
            modifier =
                Modifier
                    .weight(1f)
                    .fillMaxHeight()
                    .padding(vertical = 8.dp),
            verticalArrangement = Arrangement.SpaceBetween,
        ) {
            Column(
                verticalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                Text(
                    text = title,
                    style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.SemiBold),
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                    color = MaterialTheme.colorScheme.onSurface,
                )

                if (status != null) {
                    Text(
                        text = status,
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }

            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(16.dp),
            ) {
                if (score != null) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(4.dp),
                    ) {
                        Icon(
                            imageVector = Icons.Rounded.Star,
                            contentDescription = null,
                            tint = Color(0xFFFFC107),
                            modifier = Modifier.width(18.dp).height(18.dp),
                        )
                        Text(
                            text = score.toString(),
                            style = MaterialTheme.typography.labelMedium.copy(fontWeight = FontWeight.Medium),
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }

                if (chapterCount > 0) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(4.dp),
                    ) {
                        Icon(
                            imageVector = Icons.Rounded.AutoStories,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.onSurfaceVariant,
                            modifier = Modifier.width(18.dp).height(18.dp),
                        )
                        Text(
                            text = "$chapterCount",
                            style = MaterialTheme.typography.labelMedium.copy(fontWeight = FontWeight.Medium),
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }

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
                        modifier = Modifier.width(18.dp).height(18.dp),
                    )
                }
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ComicSearchItemPreview() {
    AcerolaTheme {
        Main.Home.Component.ComicSearchItem(
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
            chapterCount = 10,
            onClick = {},
        )
    }
}
