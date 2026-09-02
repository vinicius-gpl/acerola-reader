package br.acerola.comic.module.main.remotelibrary

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.CenterAlignedTopAppBar
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import br.acerola.comic.common.state.LocalSnackbarHostState
import br.acerola.comic.common.state.SyncActionVisualState
import br.acerola.comic.common.ux.component.SnackbarVariant
import br.acerola.comic.common.ux.component.showSnackbar
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.module.main.Main
import br.acerola.comic.module.main.remotelibrary.component.RemoteComicGridItem
import br.acerola.comic.module.main.remotelibrary.state.RemoteLibraryUiState
import br.acerola.comic.service.network.ComicSummary
import br.acerola.comic.ui.R
import kotlinx.coroutines.delay
import kotlin.time.Duration.Companion.milliseconds

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun Main.RemoteLibrary.Template.Screen(
    peerId: String,
    peerDisplayName: String,
    onBack: () -> Unit,
    viewModel: RemoteLibraryViewModel = hiltViewModel(),
) {
    val context = LocalContext.current
    val snackbarHostState = LocalSnackbarHostState.current
    val uiState by viewModel.uiState.collectAsState()

    LaunchedEffect(peerId) {
        viewModel.init(peerId, peerDisplayName)
    }

    LaunchedEffect(Unit) {
        viewModel.uiEvents.collect { message ->
            snackbarHostState.showSnackbar(message.uiMessage.asString(context), SnackbarVariant.Error)
        }
    }

    // Flash de sucesso por um instante quando um pull termina — mesma ideia do `ComicScreen`
    // pro sync com peer, já que `syncingComicName` só sabe dizer "em andamento ou não", não
    // "acabou de terminar com sucesso".
    var justSyncedComicName by remember { mutableStateOf<String?>(null) }
    var lastSyncingComicName by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(uiState.syncingComicName) {
        val previous = lastSyncingComicName
        lastSyncingComicName = uiState.syncingComicName

        if (uiState.syncingComicName == null && previous != null) {
            justSyncedComicName = previous
            delay(1800.milliseconds)
            if (justSyncedComicName == previous) justSyncedComicName = null
        }
    }

    Scaffold(
        modifier = Modifier.fillMaxSize(),
        topBar = {
            CenterAlignedTopAppBar(
                title = {
                    Text(
                        text = stringResource(id = R.string.title_sync_remote_library, uiState.peerDisplayName),
                        style = MaterialTheme.typography.titleMedium,
                        fontWeight = FontWeight.Bold,
                        maxLines = 1,
                    )
                },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(
                            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                            contentDescription = stringResource(id = R.string.description_icon_navigation_back),
                        )
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(containerColor = Color.Transparent),
            )
        },
    ) { paddingValues ->
        RemoteLibraryContent(
            uiState = uiState,
            justSyncedComicName = justSyncedComicName,
            onSelectComic = viewModel::syncComic,
            modifier = Modifier.padding(paddingValues),
        )
    }
}

@Composable
private fun RemoteLibraryContent(
    uiState: RemoteLibraryUiState,
    justSyncedComicName: String?,
    onSelectComic: (comicName: String) -> Unit,
    modifier: Modifier = Modifier,
) {
    Box(modifier = modifier.fillMaxSize()) {
        when {
            !uiState.loaded ->
                Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    CircularProgressIndicator()
                }

            uiState.errorMessage != null ->
                Box(
                    modifier = Modifier.fillMaxSize().padding(SpacingTokens.Large),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        text =
                            uiState.errorType?.uiMessage?.asString()
                                ?: stringResource(id = R.string.error_sync_remote_library_failed, uiState.errorMessage),
                        color = MaterialTheme.colorScheme.error,
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }

            uiState.comics.isEmpty() ->
                Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    Text(
                        text = stringResource(id = R.string.label_sync_remote_library_empty),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }

            else ->
                LazyVerticalGrid(
                    columns = GridCells.Adaptive(minSize = SizeTokens.ComicGridMinSize),
                    contentPadding = PaddingValues(all = SpacingTokens.Small),
                    verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
                    horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
                ) {
                    items(items = uiState.comics, key = { it.comicName }) { comic ->
                        val syncState =
                            when (comic.comicName) {
                                uiState.syncingComicName -> SyncActionVisualState.LOADING
                                justSyncedComicName -> SyncActionVisualState.SUCCESS
                                else -> SyncActionVisualState.IDLE
                            }

                        Main.RemoteLibrary.Component.RemoteComicGridItem(
                            comicName = comic.comicName,
                            chapterCount = comic.chapterCount,
                            coverPath = uiState.coverPaths[comic.comicName],
                            syncState = syncState,
                            onClick = { onSelectComic(comic.comicName) },
                        )
                    }
                }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun RemoteLibraryContentPreview() {
    AcerolaTheme {
        RemoteLibraryContent(
            uiState =
                RemoteLibraryUiState(
                    peerDisplayName = "Pixel 8",
                    loaded = true,
                    comics =
                        listOf(
                            ComicSummary(comicName = "Berserk", chapterCount = 42, coverVersion = 0),
                            ComicSummary(comicName = "Solo Leveling", chapterCount = 12, coverVersion = 0),
                            ComicSummary(comicName = "Vinland Saga", chapterCount = 24, coverVersion = 0),
                        ),
                ),
            justSyncedComicName = null,
            onSelectComic = {},
        )
    }
}

@Preview(name = "Empty state", showBackground = true)
@Composable
private fun RemoteLibraryContentEmptyPreview() {
    AcerolaTheme {
        RemoteLibraryContent(
            uiState = RemoteLibraryUiState(peerDisplayName = "Pixel 8", loaded = true),
            justSyncedComicName = null,
            onSelectComic = {},
        )
    }
}
