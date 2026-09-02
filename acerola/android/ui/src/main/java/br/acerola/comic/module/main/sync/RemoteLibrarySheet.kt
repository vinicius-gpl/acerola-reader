package br.acerola.comic.module.main.sync

import android.content.res.Configuration
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.AutoStories
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.ListItem
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.AdaptiveSheet
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.error.message.SyncProtocolError
import br.acerola.comic.service.network.ComicSummary
import br.acerola.comic.ui.R
import coil.compose.AsyncImage

/**
 * Lists a peer's library (name + chapter count only, via `acerola/browse-library/1`) and lets
 * the user pick a comic to pull with [br.acerola.comic.module.main.sync.state.SyncAction.SyncComic]
 * — the "pull" half of individual comic sync, complementing the push entry points on Home and
 * Comic Detail.
 */
@Composable
fun RemoteLibrarySheet(
    peerDisplayName: String,
    comics: List<ComicSummary>,
    isLoading: Boolean,
    errorMessage: String?,
    errorType: SyncProtocolError? = null,
    onSelectComic: (comicName: String) -> Unit,
    onDismiss: () -> Unit,
    /** Caminho local (`file://...`) da capa já baixada via `acerola/browse-cover/1`, se
     *  houver — `null` mostra o ícone padrão em vez de uma imagem. */
    coverPathFor: (comicName: String) -> String? = { null },
) {
    Acerola.Component.AdaptiveSheet(
        onDismissRequest = onDismiss,
        isScrollable = false,
    ) {
        Text(
            text = stringResource(id = R.string.title_sync_remote_library, peerDisplayName),
            style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.Bold),
            modifier = Modifier.padding(horizontal = SpacingTokens.ExtraLarge, vertical = SpacingTokens.Medium),
        )

        HorizontalDivider()

        when {
            isLoading ->
                ListItem(
                    leadingContent = { CircularProgressIndicator(modifier = Modifier.size(20.dp)) },
                    headlineContent = { Text(text = stringResource(id = R.string.label_sync_remote_library_loading)) },
                )

            errorMessage != null ->
                ListItem(
                    headlineContent = {
                        Text(
                            text =
                                errorType?.uiMessage?.asString()
                                    ?: stringResource(id = R.string.error_sync_remote_library_failed, errorMessage),
                            color = MaterialTheme.colorScheme.error,
                        )
                    },
                )

            comics.isEmpty() ->
                ListItem(
                    headlineContent = { Text(text = stringResource(id = R.string.label_sync_remote_library_empty)) },
                )

            else ->
                LazyColumn {
                    items(items = comics, key = { it.comicName }) { comic ->
                        ListItem(
                            leadingContent = {
                                val coverPath = coverPathFor(comic.comicName)
                                if (coverPath != null) {
                                    AsyncImage(
                                        model = coverPath,
                                        contentDescription = null,
                                        contentScale = ContentScale.Crop,
                                        modifier =
                                            Modifier
                                                .size(40.dp)
                                                .clip(RoundedCornerShape(SpacingTokens.Small)),
                                    )
                                } else {
                                    Icon(imageVector = Icons.Rounded.AutoStories, contentDescription = null)
                                }
                            },
                            headlineContent = { Text(text = comic.comicName) },
                            supportingContent = {
                                Text(
                                    text =
                                        stringResource(
                                            id = R.string.label_sync_remote_library_chapter_count,
                                            comic.chapterCount,
                                        ),
                                )
                            },
                            modifier =
                                Modifier
                                    .fillMaxWidth()
                                    .clickable { onSelectComic(comic.comicName) },
                        )
                    }
                }
        }

        Spacer(modifier = Modifier.navigationBarsPadding())
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun RemoteLibrarySheetPreview() {
    AcerolaTheme {
        RemoteLibrarySheet(
            peerDisplayName = "Pixel 8",
            comics =
                listOf(
                    ComicSummary(comicName = "Berserk", chapterCount = 42, coverVersion = 0),
                    ComicSummary(comicName = "Solo Leveling", chapterCount = 12, coverVersion = 0),
                ),
            isLoading = false,
            errorMessage = null,
            onSelectComic = {},
            onDismiss = {},
        )
    }
}
