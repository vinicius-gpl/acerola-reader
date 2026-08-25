package br.acerola.comic.module.main.remotelibrary.component

import android.content.res.Configuration
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
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Sync
import androidx.compose.material.icons.rounded.AutoStories
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.state.SyncActionVisualState
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.ImageCard
import br.acerola.comic.common.ux.component.SyncActionIcon
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.module.main.Main
import br.acerola.comic.ui.R
import coil.compose.rememberAsyncImagePainter
import coil.request.ImageRequest
import coil.size.Size
import coil.size.SizeResolver

/**
 * Item de grid pra um quadrinho da biblioteca remota — mesma composição visual de
 * [br.acerola.comic.module.main.home.component.ComicGridItem] (capa 2:3 grande +
 * título/contagem de capítulos abaixo), mas sem nada de [br.acerola.comic.dto.ComicDto]
 * (categoria, marcador, nota, histórico) já que um [br.acerola.comic.service.network.ComicSummary]
 * remoto não carrega nenhum desses campos. O card inteiro é o alvo de clique — pede o pull
 * (`acerola/sync-comic/1`) do quadrinho, sem uma ação separada.
 */
@Composable
fun Main.RemoteLibrary.Component.RemoteComicGridItem(
    comicName: String,
    chapterCount: Int,
    coverPath: String?,
    syncState: SyncActionVisualState,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val context = LocalContext.current
    val density = LocalDensity.current

    val imageSize =
        with(density) {
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
                    .data(data = coverPath)
                    .memoryCacheKey(coverPath)
                    .diskCacheKey(coverPath)
                    .size(resolver = SizeResolver(imageSize))
                    .build(),
        )

    Column(
        modifier = modifier.width(SizeTokens.ComicCardWidth),
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
                image = coverPainter,
                modifier = Modifier.fillMaxSize(),
            )
        }

        Spacer(modifier = Modifier.height(SpacingTokens.Small))

        Text(
            text = comicName,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.padding(horizontal = SpacingTokens.ExtraSmall),
            color = MaterialTheme.colorScheme.onBackground,
            style = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.Bold),
        )

        Spacer(modifier = Modifier.height(SpacingTokens.ExtraSmall))

        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(horizontal = SpacingTokens.ExtraSmall),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(SpacingTokens.ExtraSmall),
            ) {
                Icon(
                    imageVector = Icons.Rounded.AutoStories,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.size(SizeTokens.IconExtraSmall),
                )
                Text(
                    text = chapterCount.toString(),
                    style = MaterialTheme.typography.labelMedium.copy(fontWeight = FontWeight.Bold),
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }

            Acerola.Component.SyncActionIcon(
                state = syncState,
                containerSize = SizeTokens.ClickTargetSmall,
                iconSize = SizeTokens.IconSmall,
                defaultBackground = MaterialTheme.colorScheme.surfaceVariant,
            ) {
                Icon(
                    imageVector = Icons.Default.Sync,
                    contentDescription = stringResource(id = R.string.action_sync_comic_with_peer),
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun RemoteComicGridItemPreview() {
    AcerolaTheme {
        Row {
            Main.RemoteLibrary.Component.RemoteComicGridItem(
                comicName = "Berserk",
                chapterCount = 42,
                coverPath = null,
                syncState = SyncActionVisualState.IDLE,
                onClick = {},
            )
            Main.RemoteLibrary.Component.RemoteComicGridItem(
                comicName = "Vinland Saga",
                chapterCount = 24,
                coverPath = null,
                syncState = SyncActionVisualState.LOADING,
                onClick = {},
            )
        }
    }
}
