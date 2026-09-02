package br.acerola.comic.module.comic
import androidx.compose.ui.tooling.preview.Preview
import android.content.res.Configuration
import br.acerola.comic.common.ux.theme.AcerolaTheme
import androidx.compose.foundation.layout.Column
import br.acerola.comic.dto.archive.ComicDirectoryDto

import android.content.Intent
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.DoneAll
import androidx.compose.material.icons.filled.FilterList
import androidx.compose.material.icons.filled.RemoveDone
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.zIndex
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import br.acerola.comic.common.state.LocalSnackbarHostState
import br.acerola.comic.common.state.SyncActionVisualState
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.GlassButton
import br.acerola.comic.common.ux.component.SelectionAction
import br.acerola.comic.common.ux.component.SelectionActionDock
import br.acerola.comic.common.ux.component.SelectionTopBar
import br.acerola.comic.common.ux.component.SnackbarVariant
import br.acerola.comic.common.ux.component.TopBar
import br.acerola.comic.common.ux.component.showSnackbar
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.common.viewmodel.library.archive.ChapterArchiveViewModel
import br.acerola.comic.common.viewmodel.library.archive.ComicDirectoryViewModel
import br.acerola.comic.common.viewmodel.library.metadata.ComicMetadataViewModel
import br.acerola.comic.dto.ComicDto
import br.acerola.comic.module.comic.component.ChapterSortSheet
import br.acerola.comic.module.comic.state.ComicAction
import br.acerola.comic.module.comic.state.ComicChapterAction
import br.acerola.comic.module.comic.state.ComicSyncAction
import br.acerola.comic.module.comic.state.ComicUiState
import br.acerola.comic.module.comic.state.MainTab
import br.acerola.comic.module.comic.template.Header
import br.acerola.comic.module.comic.template.Tabs
import br.acerola.comic.module.comic.template.chapterSection
import br.acerola.comic.module.comic.template.configSection
import br.acerola.comic.module.reader.ReaderActivity
import br.acerola.comic.ui.R
import br.acerola.comic.worker.sync.MetadataSyncWorker
import kotlinx.collections.immutable.toPersistentList
import kotlinx.collections.immutable.toPersistentSet
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlin.time.Duration.Companion.milliseconds

@Composable
fun ComicScreen(
    comic: ComicDto,
    onBackClick: () -> Unit,
    comicViewModel: ComicViewModel = hiltViewModel(),
    comicMetadataViewModel: ComicMetadataViewModel = hiltViewModel(),
    chapterArchiveViewModel: ChapterArchiveViewModel = hiltViewModel(),
    comicDirectoryViewModel: ComicDirectoryViewModel = hiltViewModel(),
) {
    val context = LocalContext.current
    val snackbarHostState = LocalSnackbarHostState.current

    LaunchedEffect(key1 = comic.directory.id) {
        comicViewModel.init(comicId = comic.remoteInfo?.id, folderId = comic.directory.id)
    }

    // Coleta de eventos de UI para snackbars
    LaunchedEffect(Unit) {
        launch {
            comicViewModel.uiEvents.collect { message ->
                snackbarHostState.showSnackbar(message.uiMessage.asString(context), SnackbarVariant.Error)
            }
        }
        launch {
            comicDirectoryViewModel.uiEvents.collect { message ->
                snackbarHostState.showSnackbar(message.uiMessage.asString(context), SnackbarVariant.Error)
            }
        }
        launch {
            chapterArchiveViewModel.uiEvents.collect { message ->
                snackbarHostState.showSnackbar(message.uiMessage.asString(context), SnackbarVariant.Error)
            }
        }
        launch {
            comicMetadataViewModel.uiEvents.collect { message ->
                snackbarHostState.showSnackbar(message.uiMessage.asString(context), SnackbarVariant.Error)
            }
        }
    }

    var selectedTab by remember { mutableStateOf(value = MainTab.CHAPTERS) }

    val comicState by comicViewModel.comic.collectAsStateWithLifecycle()
    val chapterDto by comicViewModel.chapters.collectAsStateWithLifecycle()
    val allChapters by comicViewModel.allChapters.collectAsStateWithLifecycle()
    val history by comicViewModel.history.collectAsStateWithLifecycle()
    val readChapters by comicViewModel.readChapters.collectAsStateWithLifecycle()
    val selectedChapterSorts by comicViewModel.selectedChapterSorts.collectAsStateWithLifecycle()
    val selectedChapterPerPage by comicViewModel.selectedChapterPerPage.collectAsStateWithLifecycle()
    val chapterSortSettings by comicViewModel.chapterSortSettings.collectAsStateWithLifecycle()
    val allCategories by comicMetadataViewModel.allCategories.collectAsStateWithLifecycle()
    val volumeViewMode by comicViewModel.volumeViewMode.collectAsStateWithLifecycle()
    val activeVolumeId by comicViewModel.activeVolumeId.collectAsStateWithLifecycle()

    val isChapterArchiveIndexing by chapterArchiveViewModel.isIndexing.collectAsStateWithLifecycle(false)
    val isComicDirectoryIndexing by comicDirectoryViewModel.isIndexing.collectAsStateWithLifecycle(false)
    val isComicMetadataIndexing by comicMetadataViewModel.isIndexing.collectAsStateWithLifecycle(false)

    val isExtractingVolumeCovers by comicViewModel.isExtractingVolumeCovers.collectAsStateWithLifecycle(false)
    val isSyncingWithPeer by comicViewModel.isSyncingWithPeer.collectAsStateWithLifecycle(false)
    val pairedPeers by comicViewModel.pairedPeers.collectAsStateWithLifecycle()

    val activeLibrarySyncType by comicDirectoryViewModel.activeSyncType.collectAsStateWithLifecycle(null)
    val activeMetadataSource by comicMetadataViewModel.activeSyncSource.collectAsStateWithLifecycle(null)

    var activeSyncAction by remember { mutableStateOf<ComicSyncAction?>(null) }
    var successSyncAction by remember { mutableStateOf<ComicSyncAction?>(null) }
    var isCurrentlyIndexing by remember { mutableStateOf(false) }

    val isAnyIndexing = isChapterArchiveIndexing || isComicDirectoryIndexing || isComicMetadataIndexing || isExtractingVolumeCovers

    LaunchedEffect(isAnyIndexing) {
        if (isAnyIndexing) {
            isCurrentlyIndexing = true
            return@LaunchedEffect
        }

        if (isCurrentlyIndexing && activeSyncAction != null) {
            val finishedAction = activeSyncAction
            activeSyncAction = null
            isCurrentlyIndexing = false
            successSyncAction = finishedAction

            delay(1800.milliseconds)

            if (successSyncAction == finishedAction) {
                successSyncAction = null
            }
        } else {
            isCurrentlyIndexing = false
            activeSyncAction = null
        }
    }

    fun getSyncActionVisualState(action: ComicSyncAction): SyncActionVisualState =
        when {
            activeSyncAction == action -> SyncActionVisualState.LOADING
            action == ComicSyncAction.RescanComic && activeLibrarySyncType != null -> SyncActionVisualState.LOADING
            action == ComicSyncAction.SyncMangadexInfo && activeMetadataSource == MetadataSyncWorker.SOURCE_MANGADEX -> SyncActionVisualState.LOADING
            action == ComicSyncAction.SyncAnilistInfo && activeMetadataSource == MetadataSyncWorker.SOURCE_ANILIST -> SyncActionVisualState.LOADING
            action == ComicSyncAction.SyncComicInfo && activeMetadataSource == MetadataSyncWorker.SOURCE_COMICINFO -> SyncActionVisualState.LOADING
            successSyncAction == action -> SyncActionVisualState.SUCCESS
            else -> SyncActionVisualState.IDLE
        }

    // `ComicSyncAction.SyncWithPeer` carries a `peerId` chosen only after the peer picker
    // closes, so it can't reuse the generic per-instance-equality tracking above (there's a
    // single button, not one per peer) — a small dedicated state machine mirroring the same
    // "flash success for a bit" shape instead.
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

    val syncWithPeerState =
        when {
            isSyncingWithPeer -> SyncActionVisualState.LOADING
            syncWithPeerSuccess -> SyncActionVisualState.SUCCESS
            else -> SyncActionVisualState.IDLE
        }

    val currentManga = comicState ?: comic
    val totalChapters = chapterDto?.archive?.total ?: 0
    val currentPage = chapterDto?.archive?.page ?: 0

    val totalPages =
        remember(key1 = chapterDto?.archive?.total, key2 = chapterDto?.archive?.pageSize) {
            val size = chapterDto?.archive?.pageSize ?: 1
            val total = chapterDto?.archive?.total ?: 0
            if (total == 0) 0 else kotlin.math.ceil(x = total.toDouble() / size).toInt()
        }

    val showVolumeHeaders =
        remember(key1 = chapterDto?.showVolumeHeaders) {
            chapterDto?.showVolumeHeaders ?: false
        }

    val uiState =
        ComicUiState(
            comic = currentManga,
            chapters = chapterDto,
            selectedTab = selectedTab,
            history = history,
            readChapters = readChapters.toPersistentSet(),
            totalChapters = totalChapters,
            currentPage = currentPage,
            totalPages = totalPages,
            selectedChapterPerPage = selectedChapterPerPage,
            chapterSortSettings = chapterSortSettings,
            allCategories = allCategories.toPersistentList(),
            showVolumeHeaders = showVolumeHeaders,
            volumeViewMode = chapterDto?.effectiveViewMode ?: volumeViewMode,
            activeVolumeId = activeVolumeId,
            hasVolumeStructure = chapterDto?.hasVolumeStructure ?: false,
        )

    val listState = rememberLazyListState()
    val haptic = LocalHapticFeedback.current

    val isChapterSelectionMode = selectedChapterSorts.isNotEmpty()
    val allChapterSorts =
        remember(allChapters) {
            allChapters.map { it.chapterSort }
        }
    val areAllSelectedRead =
        remember(readChapters, selectedChapterSorts) {
            selectedChapterSorts.isNotEmpty() && selectedChapterSorts.all { readChapters.contains(it) }
        }

    val onChapterLongPress: (String) -> Unit = { chapterSort ->
        haptic.performHapticFeedback(HapticFeedbackType.LongPress)
        comicViewModel.toggleChapterSelection(chapterSort)
    }

    var showSortSheet by remember { mutableStateOf(false) }

    val onChapterAction: (ComicChapterAction) -> Unit = { action ->
        when (action) {
            is ComicChapterAction.ChangePage -> {
                comicViewModel.loadPageAsync(action.page)
            }
            is ComicChapterAction.ClickChapter -> {
                val intent =
                    Intent(context, ReaderActivity::class.java).apply {
                        putExtra(ReaderActivity.PageExtra.PAGE, action.chapter)
                        putExtra(ReaderActivity.PageExtra.MANGA_ID, uiState.comic.directory.id)
                        putExtra(ReaderActivity.PageExtra.INITIAL_PAGE, action.initialPage)
                    }
                context.startActivity(intent)
            }
            is ComicChapterAction.ClickContinue -> {
                val targetChapter =
                    if (action.chapterId == -1L) {
                        allChapters.firstOrNull()
                    } else {
                        allChapters.find { it.id == action.chapterId }
                    }

                targetChapter?.let {
                    val intent =
                        Intent(context, ReaderActivity::class.java).apply {
                            putExtra(ReaderActivity.PageExtra.PAGE, it)
                            putExtra(ReaderActivity.PageExtra.MANGA_ID, uiState.comic.directory.id)
                            putExtra(ReaderActivity.PageExtra.INITIAL_PAGE, action.lastPage)
                        }
                    context.startActivity(intent)
                }
            }
            is ComicChapterAction.ToggleReadStatus -> {
                comicViewModel.toggleChapterReadStatus(action.chapterSort)
            }
        }
    }

    val onSyncAction: (ComicSyncAction) -> Unit = { action ->
        activeSyncAction = action
        when (action) {
            ComicSyncAction.SyncChaptersLocal -> chapterArchiveViewModel.syncChaptersByMangaDirectory(uiState.comic.directory.id)
            ComicSyncAction.RescanComic -> comicDirectoryViewModel.rescanMangaByManga(uiState.comic.directory.id)
            ComicSyncAction.SyncMangadexInfo -> comicMetadataViewModel.syncFromMangadex(uiState.comic.directory.id)
            ComicSyncAction.SyncComicInfo -> comicMetadataViewModel.syncFromComicInfo(uiState.comic.directory.id)
            ComicSyncAction.SyncAnilistInfo -> comicMetadataViewModel.syncFromAnilist(uiState.comic.directory.id)
            ComicSyncAction.ExtractFirstPageAsCover -> comicDirectoryViewModel.extractCoverFromChapter(uiState.comic.directory.id)
            ComicSyncAction.ExtractVolumeCovers -> comicViewModel.extractAllVolumeCovers()
            is ComicSyncAction.SyncWithPeer -> comicViewModel.syncWithPeer(action.peerId, action.direction)
        }
    }

    val onAction: (ComicAction) -> Unit = { action ->
        when (action) {
            ComicAction.NavigateBack -> onBackClick()
            is ComicAction.SelectTab -> {
                selectedTab = action.tab
                if (action.tab != MainTab.CHAPTERS) comicViewModel.clearChapterSelection()
            }
            is ComicAction.UpdatePageSize -> comicViewModel.updateChapterPerPage(action.size)
            is ComicAction.UpdateCategory ->
                comicMetadataViewModel.updateMangaCategory(
                    uiState.comic.directory.id,
                    action.categoryId,
                )
            is ComicAction.ToggleExternalSync ->
                comicDirectoryViewModel.updateExternalSyncEnabled(
                    uiState.comic.directory.id,
                    action.enabled,
                )
            is ComicAction.UpdateVolumeView -> comicViewModel.updateVolumeViewMode(action.mode)
            ComicAction.ClearMetadata -> comicMetadataViewModel.clearMetadata(uiState.comic.directory.id)
        }
    }

    Box(modifier = Modifier.fillMaxSize()) {
        Scaffold(
            modifier = Modifier.fillMaxSize(),
            containerColor = MaterialTheme.colorScheme.background,
            contentColor = MaterialTheme.colorScheme.onBackground,
        ) { paddingValues ->
            LazyColumn(
                state = listState,
                modifier =
                    Modifier
                        .fillMaxSize()
                        .padding(bottom = paddingValues.calculateBottomPadding()),
            ) {
                item(
                    key = "comic_header",
                    contentType = "header",
                ) {
                    Comic.Template.Header(
                        comic = uiState.comic,
                        history = uiState.history,
                        onContinueClick = { id, page -> onChapterAction(ComicChapterAction.ClickContinue(id, page)) },
                    )
                }

                item(
                    key = "comic_tabs",
                    contentType = "tabs",
                ) {
                    Comic.Template.Tabs(
                        totalChapters = uiState.totalChapters,
                        activeTab = uiState.selectedTab,
                        onTabSelected = { onAction(ComicAction.SelectTab(it)) },
                    )
                }

                // TODO: Fazer eles não só trocarem quando clicados, mas com o arrastar do dedo uma troca entre abas com arrastar de dedos.
                when (uiState.selectedTab) {
                    MainTab.CHAPTERS -> {
                        uiState.chapters?.let {
                            Comic.Template.chapterSection(
                                scope = this,
                                chapters = it,
                                currentPage = uiState.currentPage,
                                totalPages = uiState.totalPages,
                                onPageChange = { page -> onChapterAction(ComicChapterAction.ChangePage(page)) },
                                readChapters = uiState.readChapters.toList(),
                                onToggleRead = { id -> onChapterAction(ComicChapterAction.ToggleReadStatus(id)) },
                                onChapterClick = { chapter -> onChapterAction(ComicChapterAction.ClickChapter(chapter, 0)) },
                                selectedChapterSorts = selectedChapterSorts,
                                isSelectionMode = isChapterSelectionMode,
                                onToggleSelection = comicViewModel::toggleChapterSelection,
                                onLongPressChapter = onChapterLongPress,
                                volumeViewMode = uiState.volumeViewMode,
                                activeVolumeId = uiState.activeVolumeId,
                                onSetActiveVolume = comicViewModel::setActiveVolume,
                                onLoadVolumeChaptersPage = comicViewModel::loadVolumeChaptersPage,
                                onExtractVolumeCover = comicViewModel::extractVolumeCover,
                            )
                        }
                    }

                    MainTab.SETTINGS -> {
                        Comic.Template.configSection(
                            scope = this,
                            uiState = uiState,
                            getSyncActionVisualState = ::getSyncActionVisualState,
                            pairedPeers = pairedPeers,
                            syncWithPeerState = syncWithPeerState,
                            onLoadPairedPeers = comicViewModel::loadPairedPeers,
                            onAction = onAction,
                            onSyncAction = onSyncAction,
                        )
                    }
                }

                item(
                    key = "comic_bottom_spacer",
                    contentType = "tabs",
                ) {
                    Spacer(modifier = Modifier.height(height = 24.dp))
                }
            }
        }

        if (isChapterSelectionMode && uiState.selectedTab == MainTab.CHAPTERS) {
            val isAllChaptersSelected = selectedChapterSorts.size == allChapterSorts.size && allChapterSorts.isNotEmpty()
            val statusBarHeight = WindowInsets.statusBars.asPaddingValues().calculateTopPadding()

            Box(
                modifier =
                    Modifier
                        .align(Alignment.TopCenter)
                        .zIndex(2f)
                        .fillMaxWidth()
                        .padding(
                            start = SpacingTokens.Small,
                            end = SpacingTokens.Small,
                            top = statusBarHeight + SpacingTokens.ExtraSmall,
                        ),
            ) {
                Acerola.Component.SelectionTopBar(
                    selectedCount = selectedChapterSorts.size,
                    isAllSelected = isAllChaptersSelected,
                    onClear = { comicViewModel.clearChapterSelection() },
                    onToggleSelectAll = {
                        if (isAllChaptersSelected) {
                            comicViewModel.clearChapterSelection()
                        } else {
                            comicViewModel.selectAllChapters(allChapterSorts)
                        }
                    },
                )
            }
        } else {
            Acerola.Component.TopBar(
                navigationIcon = {
                    Acerola.Component.GlassButton(
                        onClick = { onAction(ComicAction.NavigateBack) },
                        icon = {
                            Icon(
                                tint = MaterialTheme.colorScheme.onSurface,
                                imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                                contentDescription = stringResource(id = R.string.description_icon_navigation_back),
                            )
                        },
                    )
                },
                actions = {
                    Acerola.Component.GlassButton(
                        onClick = { showSortSheet = true },
                        icon = {
                            Icon(
                                tint = MaterialTheme.colorScheme.onSurface,
                                imageVector = Icons.Default.FilterList,
                                contentDescription = stringResource(id = R.string.description_icon_home_filter),
                            )
                        },
                    )
                },
            )
        }

        AnimatedVisibility(
            visible = isChapterSelectionMode && uiState.selectedTab == MainTab.CHAPTERS,
            enter = slideInVertically { it } + fadeIn(),
            exit = slideOutVertically { it } + fadeOut(),
            modifier =
                Modifier
                    .align(Alignment.BottomCenter)
                    .zIndex(3f)
                    .padding(bottom = SpacingTokens.Large),
        ) {
            Acerola.Component.SelectionActionDock(
                actions =
                    listOf(
                        SelectionAction(
                            icon = if (areAllSelectedRead) Icons.Default.RemoveDone else Icons.Default.DoneAll,
                            label =
                                stringResource(
                                    id = if (areAllSelectedRead) R.string.action_mark_as_unread else R.string.action_mark_as_read,
                                ),
                            onClick = { comicViewModel.markSelectedChaptersReadStatus(!areAllSelectedRead) },
                        ),
                    ),
                modifier =
                    Modifier
                        .widthIn(max = 440.dp)
                        .padding(horizontal = SpacingTokens.Medium),
            )
        }

        if (showSortSheet) {
            Comic.Component.ChapterSortSheet(
                sortSettings = uiState.chapterSortSettings,
                onSortChange = { comicViewModel.updateChapterSort(it) },
                onDismiss = { showSortSheet = false },
            )
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ComicScreenPreview() {
    AcerolaTheme {
        Column(modifier = Modifier.fillMaxSize()) {
            Comic.Template.Header(
                comic = ComicDto(
                    directory = ComicDirectoryDto(id = 1L, name = "Sample Comic", path = "/path", coverUri = null, bannerUri = null, lastModified = 0L, archiveTemplateFk = null),
                    category = null,
                    remoteInfo = null,
                ),
                history = null,
                onContinueClick = { _, _ -> },
            )
            Comic.Template.Tabs(
                totalChapters = 12,
                activeTab = br.acerola.comic.module.comic.state.MainTab.CHAPTERS,
                onTabSelected = {},
            )
        }
    }
}
