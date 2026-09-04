package br.acerola.comic.module.comic.component

import android.content.res.Configuration
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.LibraryBooks
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.ShapeTokens
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.dto.archive.ChapterFileDto
import br.acerola.comic.dto.archive.VolumeArchiveDto
import br.acerola.comic.dto.archive.VolumeChapterGroupDto
import br.acerola.comic.module.comic.Comic
import br.acerola.comic.ui.R
import coil.compose.rememberAsyncImagePainter
import coil.request.ImageRequest
import coil.size.Size
import coil.size.SizeResolver

private val IdentitySquareSize = 44.dp
private val IdentityCoverWidth = 44.dp
private val IdentityCoverHeight = 64.dp

// Cabeçalho de seção, não card: sem borda/fundo próprios, para refletir que ele abre uma
// lista de capítulos que segue *fora* dele, em vez de prometer contê-la (ver HeroButton).
@OptIn(ExperimentalFoundationApi::class)
@Composable
fun Comic.Component.VolumeSectionHeader(
    title: String,
    chapterSummary: String,
    expanded: Boolean,
    onToggleExpanded: () -> Unit,
    modifier: Modifier = Modifier,
    isSpecial: Boolean = false,
    onLongClick: (() -> Unit)? = null,
    identity: @Composable () -> Unit,
) {
    val chevronRotation by animateFloatAsState(targetValue = if (expanded) 180f else 0f, label = "volume_chevron_rotation")

    Column(modifier = modifier.fillMaxWidth()) {
        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .heightIn(min = SizeTokens.ClickTarget)
                    .combinedClickable(onClick = onToggleExpanded, onLongClick = onLongClick)
                    .padding(horizontal = SpacingTokens.ExtraLarge, vertical = SpacingTokens.Small),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            identity()

            Spacer(modifier = Modifier.width(SpacingTokens.Medium))

            Column(modifier = Modifier.weight(1f)) {
                Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                    Text(
                        text = title,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        style = MaterialTheme.typography.titleSmall,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onSurface,
                        modifier = Modifier.weight(1f, fill = false),
                    )

                    if (isSpecial) {
                        Spacer(modifier = Modifier.width(SpacingTokens.ExtraSmall))
                        SpecialPill()
                    }
                }

                Text(
                    text = chapterSummary,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }

            Spacer(modifier = Modifier.width(SpacingTokens.Small))

            Icon(
                imageVector = Icons.Default.KeyboardArrowDown,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier =
                    Modifier
                        .size(SizeTokens.IconMedium)
                        .graphicsLayer { rotationZ = chevronRotation },
            )
        }

        if (!expanded) {
            HorizontalDivider(
                thickness = SizeTokens.BorderThin,
                color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.5f),
            )
        }
    }
}

@Composable
fun Comic.Component.VolumeSectionHeader(
    group: VolumeChapterGroupDto,
    expanded: Boolean,
    onToggleExpanded: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Comic.Component.VolumeSectionHeader(
        title = group.volume.name,
        chapterSummary = stringResource(id = R.string.label_volume_card_description, group.loadedCount, group.totalChapters),
        expanded = expanded,
        onToggleExpanded = onToggleExpanded,
        isSpecial = group.volume.isSpecial,
        modifier = modifier,
        identity = { VolumeNumberBadge(label = volumeNumberLabel(group.volume.volumeSort)) },
    )
}

@Composable
fun Comic.Component.CoverVolumeSectionHeader(
    group: VolumeChapterGroupDto,
    expanded: Boolean,
    onToggleExpanded: () -> Unit,
    onExtractCover: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Comic.Component.VolumeSectionHeader(
        title = group.volume.name,
        chapterSummary =
            pluralStringResource(
                id = R.plurals.label_volume_header_chapter_count,
                count = group.totalChapters,
                group.totalChapters,
            ),
        expanded = expanded,
        onToggleExpanded = onToggleExpanded,
        isSpecial = group.volume.isSpecial,
        onLongClick = onExtractCover,
        modifier = modifier,
        identity = {
            VolumeCoverIdentity(
                coverUri = group.volume.coverUri,
                volumeId = group.volume.id,
                lastModified = group.volume.lastModified,
            )
        },
    )
}

@Composable
private fun VolumeNumberBadge(
    label: String,
    modifier: Modifier = Modifier,
) {
    Box(
        modifier =
            modifier
                .size(IdentitySquareSize)
                .clip(ShapeTokens.Medium)
                .background(MaterialTheme.colorScheme.tertiaryContainer),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = label,
            style = MaterialTheme.typography.titleMedium,
            fontWeight = FontWeight.ExtraBold,
            color = MaterialTheme.colorScheme.onTertiaryContainer,
        )
    }
}

@Composable
private fun VolumeCoverIdentity(
    coverUri: String?,
    volumeId: Long,
    lastModified: Long,
    modifier: Modifier = Modifier,
) {
    Box(
        modifier =
            modifier
                .width(IdentityCoverWidth)
                .height(IdentityCoverHeight)
                .clip(ShapeTokens.Small),
    ) {
        if (coverUri != null) {
            val context = LocalContext.current
            val density = LocalDensity.current
            val imageSize =
                with(receiver = density) {
                    Size(width = IdentityCoverWidth.toPx().toInt(), height = IdentityCoverHeight.toPx().toInt())
                }
            val coverPainter =
                rememberAsyncImagePainter(
                    model =
                        ImageRequest
                            .Builder(context)
                            .data(data = coverUri)
                            .memoryCacheKey("volume_header_${volumeId}_$lastModified")
                            .diskCacheKey("volume_header_${volumeId}_$lastModified")
                            .size(resolver = SizeResolver(imageSize))
                            .build(),
                )

            Image(
                painter = coverPainter,
                contentDescription = null,
                contentScale = ContentScale.Crop,
                modifier = Modifier.fillMaxSize(),
            )
        } else {
            Box(
                modifier =
                    Modifier
                        .fillMaxSize()
                        .background(MaterialTheme.colorScheme.tertiaryContainer),
                contentAlignment = Alignment.Center,
            ) {
                Icon(
                    imageVector = Icons.Default.LibraryBooks,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.onTertiaryContainer,
                    modifier = Modifier.size(SizeTokens.IconMedium),
                )
            }
        }
    }
}

@Composable
private fun SpecialPill(modifier: Modifier = Modifier) {
    Box(
        modifier =
            modifier
                .clip(ShapeTokens.Full)
                .background(MaterialTheme.colorScheme.secondaryContainer)
                .padding(horizontal = SpacingTokens.Small, vertical = 2.dp),
    ) {
        Text(
            text = stringResource(id = R.string.label_volume_header_special),
            style = MaterialTheme.typography.labelSmall,
            fontWeight = FontWeight.Bold,
            color = MaterialTheme.colorScheme.onSecondaryContainer,
        )
    }
}

private fun volumeNumberLabel(volumeSort: String): String = volumeSort.trimStart('0').ifEmpty { "0" }

private val previewVolume1 = VolumeArchiveDto(id = 1L, name = "Volume 1", volumeSort = "0001", isSpecial = false)
private val previewVolume2Special = VolumeArchiveDto(id = 2L, name = "Extra - Artbook", volumeSort = "0000", isSpecial = true)
private val previewVolume3 = VolumeArchiveDto(id = 3L, name = "Volume 3", volumeSort = "0003", isSpecial = false)
private val previewVolume4 = VolumeArchiveDto(id = 4L, name = "Volume 4", volumeSort = "0004", isSpecial = false)

private val previewGroup1 =
    VolumeChapterGroupDto(volume = previewVolume1, items = emptyList(), totalChapters = 8, loadedCount = 8, hasMore = false)
private val previewGroup2Special =
    VolumeChapterGroupDto(volume = previewVolume2Special, items = emptyList(), totalChapters = 1, loadedCount = 1, hasMore = false)
private val previewGroup3 =
    VolumeChapterGroupDto(volume = previewVolume3, items = emptyList(), totalChapters = 6, loadedCount = 6, hasMore = false)
private val previewGroup4 =
    VolumeChapterGroupDto(volume = previewVolume4, items = emptyList(), totalChapters = 5, loadedCount = 3, hasMore = true)

@Preview(name = "Colapsado - Light", showBackground = true)
@Preview(name = "Colapsado - Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun VolumeSectionHeaderCollapsedPreview() {
    AcerolaTheme {
        Comic.Component.VolumeSectionHeader(group = previewGroup1, expanded = false, onToggleExpanded = {})
    }
}

@Preview(name = "Expandido - Light", showBackground = true)
@Preview(name = "Expandido - Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun VolumeSectionHeaderExpandedPreview() {
    AcerolaTheme {
        Comic.Component.VolumeSectionHeader(group = previewGroup1, expanded = true, onToggleExpanded = {})
    }
}

@Preview(name = "Especial (colapsado) - Light", showBackground = true)
@Preview(name = "Especial (colapsado) - Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun VolumeSectionHeaderSpecialPreview() {
    AcerolaTheme {
        Comic.Component.VolumeSectionHeader(group = previewGroup2Special, expanded = false, onToggleExpanded = {})
    }
}

@Preview(name = "Modo capa (sem capa) - Light", showBackground = true)
@Preview(name = "Modo capa (sem capa) - Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun CoverVolumeSectionHeaderPreview() {
    AcerolaTheme {
        Comic.Component.CoverVolumeSectionHeader(
            group = previewGroup1,
            expanded = false,
            onToggleExpanded = {},
            onExtractCover = {},
        )
    }
}

@Preview(name = "Lista completa - Light", showBackground = true, heightDp = 480)
@Preview(name = "Lista completa - Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES, heightDp = 480)
@Composable
private fun VolumeSectionHeaderListPreview() {
    AcerolaTheme {
        Column(modifier = Modifier.fillMaxWidth()) {
            Comic.Component.VolumeSectionHeader(group = previewGroup1, expanded = false, onToggleExpanded = {})
            Comic.Component.VolumeSectionHeader(group = previewGroup2Special, expanded = false, onToggleExpanded = {})
            Comic.Component.VolumeSectionHeader(group = previewGroup3, expanded = true, onToggleExpanded = {})
            Comic.Component.ChapterItem(
                chapterFileDto = ChapterFileDto(id = 30L, name = "Capítulo 18", path = "", chapterSort = "018"),
                isRead = true,
                onClick = {},
            )
            Comic.Component.ChapterItem(
                chapterFileDto = ChapterFileDto(id = 31L, name = "Capítulo 17", path = "", chapterSort = "017"),
                isRead = false,
                onClick = {},
            )
            Comic.Component.VolumeSectionHeader(group = previewGroup4, expanded = false, onToggleExpanded = {})
        }
    }
}
