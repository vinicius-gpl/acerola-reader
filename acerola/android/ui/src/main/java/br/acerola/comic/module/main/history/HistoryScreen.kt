package br.acerola.comic.module.main.history

import android.content.Intent
import android.content.res.Configuration
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Sync
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import br.acerola.comic.common.state.LocalSnackbarHostState
import br.acerola.comic.common.state.SyncActionVisualState
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.SnackbarVariant
import br.acerola.comic.common.ux.component.SyncActionIcon
import br.acerola.comic.common.ux.component.showSnackbar
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.dto.ComicDto
import br.acerola.comic.dto.archive.ComicDirectoryDto
import br.acerola.comic.dto.history.ReadingHistoryWithChapterDto
import br.acerola.comic.module.comic.ComicActivity
import br.acerola.comic.module.main.Main
import br.acerola.comic.module.main.common.component.ComicListItem
import br.acerola.comic.module.main.common.component.PeerPickerSheet
import br.acerola.comic.module.main.history.component.HistoryHeroCard
import br.acerola.comic.module.main.history.state.HistoryAction
import br.acerola.comic.module.main.history.state.HistoryItemState
import br.acerola.comic.module.main.history.state.HistoryUiState
import br.acerola.comic.module.reader.ReaderActivity
import br.acerola.comic.ui.R
import kotlinx.coroutines.delay
import kotlin.time.Duration.Companion.milliseconds

@Composable
fun Main.History.Template.Screen(viewModel: HistoryViewModel = hiltViewModel()) {
    val context = LocalContext.current
    val historyItems by viewModel.historyItems.collectAsState()
    val pairedPeers by viewModel.pairedPeers.collectAsState()
    val isSyncingWithPeer by viewModel.isSyncingWithPeer.collectAsState()
    val snackbarHostState = LocalSnackbarHostState.current

    LaunchedEffect(Unit) {
        viewModel.uiEvents.collect { message ->
            snackbarHostState.showSnackbar(message.uiMessage.asString(context), SnackbarVariant.Error)
        }
    }

    // Mesma máquina de "flash de sucesso" do `ComicScreen` pro sync com peer — um único botão
    // (não um por peer), então não dá pra reaproveitar o rastreio genérico por instância usado
    // pros outros syncs desta tela.
    var syncWithPeerSuccess by remember { mutableStateOf(false) }
    var wasSyncingWithPeer by remember { mutableStateOf(false) }

    LaunchedEffect(isSyncingWithPeer) {
        if (isSyncingWithPeer) {
            wasSyncingWithPeer = true
            return@LaunchedEffect
        }

        if (wasSyncingWithPeer) {
            wasSyncingWithPeer = false
            syncWithPeerSuccess = true
            delay(1800.milliseconds)
            if (syncWithPeerSuccess) syncWithPeerSuccess = false
        }
    }

    val historySyncState =
        when {
            isSyncingWithPeer -> SyncActionVisualState.LOADING
            syncWithPeerSuccess -> SyncActionVisualState.SUCCESS
            else -> SyncActionVisualState.IDLE
        }

    val uiState = HistoryUiState(items = historyItems, pairedPeers = pairedPeers)

    val onAction: (HistoryAction) -> Unit = { action ->
        when (action) {
            is HistoryAction.ClickManga -> {
                val intent =
                    Intent(context, ComicActivity::class.java).apply {
                        putExtra(ComicActivity.ChapterExtra.COMIC, action.comic)
                    }
                context.startActivity(intent)
            }
            is HistoryAction.ClickContinue -> {
                val intent =
                    Intent(context, ReaderActivity::class.java).apply {
                        putExtra(ReaderActivity.PageExtra.MANGA_ID, action.comic.directory.id)
                        putExtra(ReaderActivity.PageExtra.CHAPTER_ID, action.history.chapterArchiveId)
                        putExtra(ReaderActivity.PageExtra.CHAPTER_SORT, action.history.chapterSort)
                        putExtra(ReaderActivity.PageExtra.INITIAL_PAGE, action.history.lastPage)
                    }
                context.startActivity(intent)
            }
            HistoryAction.LoadPairedPeersForSync -> viewModel.loadPairedPeers()
            is HistoryAction.SyncHistoryWithPeer -> viewModel.syncHistoryWithPeer(action.peerId)
        }
    }

    HistoryScreenContent(uiState = uiState, onAction = onAction, historySyncState = historySyncState)
}

@Composable
fun HistoryScreenContent(
    uiState: HistoryUiState,
    onAction: (HistoryAction) -> Unit,
    historySyncState: SyncActionVisualState = SyncActionVisualState.IDLE,
) {
    var showPeerPicker by remember { mutableStateOf(false) }

    // `MainActivity` renders `BottomBar` as a sibling outside this screen's own Scaffold
    // (`applyScaffoldPadding = false` in `BaseActivity`, on purpose — screens draw edge-to-edge
    // behind the translucent/blurred bar), so this inner Scaffold's automatic FAB placement has
    // no idea that 64dp bar exists and only clears the raw system nav-bar inset. Same fixed
    // height as `BottomBar.kt`'s `Modifier.height(64.dp)` / `HomeScreen.kt`'s
    // `mainBottomBarHeight` — hidden in landscape too, where `SideBar` replaces it instead.
    val isLandscape = LocalConfiguration.current.orientation == Configuration.ORIENTATION_LANDSCAPE
    val bottomBarClearance = if (isLandscape) 0.dp else 64.dp

    Scaffold(
        modifier = Modifier.fillMaxSize(),
        containerColor = MaterialTheme.colorScheme.background,
        floatingActionButton = {
            Acerola.Component.SyncActionIcon(
                state = historySyncState,
                modifier =
                    Modifier
                        .padding(bottom = bottomBarClearance)
                        .clickable(enabled = historySyncState != SyncActionVisualState.LOADING) {
                            onAction(HistoryAction.LoadPairedPeersForSync)
                            showPeerPicker = true
                        },
            ) {
                Icon(
                    imageVector = Icons.Default.Sync,
                    contentDescription = stringResource(id = R.string.description_icon_history_sync_with_peer),
                    tint = MaterialTheme.colorScheme.onPrimaryContainer,
                )
            }
        },
    ) { paddingValues ->
        Column(
            modifier =
                Modifier
                    .fillMaxSize()
                    .padding(paddingValues)
                    .padding(horizontal = 16.dp),
        ) {
            if (uiState.items.isEmpty()) {
                Column(
                    modifier = Modifier.fillMaxSize(),
                    verticalArrangement = Arrangement.Center,
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    Text(
                        text = stringResource(id = R.string.description_history_empty_state),
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            } else {
                LazyColumn(
                    contentPadding = PaddingValues(bottom = 16.dp + 64.dp),
                    verticalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    item {
                        val firstItem = uiState.items.first()
                        Main.History.Component.HistoryHeroCard(
                            comic = firstItem.comic,
                            onClick = { onAction(HistoryAction.ClickManga(firstItem.comic)) },
                            onContinueClick = { onAction(HistoryAction.ClickContinue(firstItem.comic, firstItem.history)) },
                        )

                        if (uiState.items.size > 1) {
                            Spacer(modifier = Modifier.height(12.dp))
                        }
                    }

                    items(uiState.items.drop(1), key = { it.comic.directory.id }) { item ->
                        val chapterInfo =
                            item.history.chapterName ?: stringResource(
                                id = R.string.label_chapter_unknown,
                            )

                        val progressInfo =
                            stringResource(
                                id = R.string.label_history_chapter_progress,
                                chapterInfo,
                                item.history.lastPage + 1,
                            )

                        Main.Common.Component.ComicListItem(
                            comic = item.comic,
                            subtitle = progressInfo,
                            chapterCount = item.chapterCount,
                            isCompleted = item.history.isCompleted,
                            onPlayClick = { onAction(HistoryAction.ClickContinue(item.comic, item.history)) },
                            onClick = { onAction(HistoryAction.ClickManga(item.comic)) },
                        )
                    }
                }
            }
        }
    }

    if (showPeerPicker) {
        Main.Common.Component.PeerPickerSheet(
            peers = uiState.pairedPeers,
            onSelect = { peerId ->
                showPeerPicker = false
                onAction(HistoryAction.SyncHistoryWithPeer(peerId))
            },
            onDismiss = { showPeerPicker = false },
        )
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ScreenPreview() {
    AcerolaTheme {
        HistoryScreenContent(
            uiState =
                HistoryUiState(
                    items =
                        listOf(
                            HistoryItemState(
                                comic =
                                    ComicDto(
                                        directory =
                                            ComicDirectoryDto(
                                                id = 1L,
                                                name = "Cyberpunk 2077",
                                                path = "/path",
                                                coverUri = null,
                                                bannerUri = null,
                                                lastModified = 0L,
                                                archiveTemplateFk = null,
                                            ),
                                        category = null,
                                        remoteInfo = null,
                                    ),
                                history =
                                    ReadingHistoryWithChapterDto(
                                        comicDirectoryId = 1L,
                                        chapterArchiveId = 10L,
                                        chapterSort = "0001",
                                        lastPage = 5,
                                        updatedAt = 123456L,
                                        chapterName = "Capítulo 1",
                                        isCompleted = false,
                                    ),
                                chapterCount = 12,
                            ),
                        ),
                ),
            onAction = {},
        )
    }
}
