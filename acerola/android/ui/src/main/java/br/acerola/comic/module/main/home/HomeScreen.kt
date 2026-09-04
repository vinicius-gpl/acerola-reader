package br.acerola.comic.module.main.home
import android.content.Intent
import android.content.res.Configuration
import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ViewList
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.FilterList
import androidx.compose.material.icons.filled.FolderOpen
import androidx.compose.material.icons.filled.GridView
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.rounded.Bookmark
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.LayersClear
import androidx.compose.material.icons.rounded.PhoneAndroid
import androidx.compose.material.icons.rounded.Visibility
import androidx.compose.material.icons.rounded.VisibilityOff
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.zIndex
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import br.acerola.comic.common.state.LocalSnackbarHostState
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.Dialog
import br.acerola.comic.common.ux.component.DialogButton
import br.acerola.comic.common.ux.component.FabGroup
import br.acerola.comic.common.ux.component.FabGroupItem
import br.acerola.comic.common.ux.component.SelectionAction
import br.acerola.comic.common.ux.component.SelectionActionDock
import br.acerola.comic.common.ux.component.SelectionTopBar
import br.acerola.comic.common.ux.component.SnackbarVariant
import br.acerola.comic.common.ux.component.showSnackbar
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.ShapeTokens
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.common.viewmodel.archive.FileSystemAccessViewModel
import br.acerola.comic.common.viewmodel.library.archive.ComicDirectoryViewModel
import br.acerola.comic.config.preference.types.HomeLayoutType
import br.acerola.comic.dto.ComicDto
import br.acerola.comic.module.comic.ComicActivity
import br.acerola.comic.module.main.Main
import br.acerola.comic.module.main.common.component.BatchComicCategorySheet
import br.acerola.comic.module.main.common.component.ComicActionsSheet
import br.acerola.comic.module.main.common.component.ComicListItem
import br.acerola.comic.module.main.common.component.PeerPickerSheet
import br.acerola.comic.module.main.home.component.ComicGridItem
import br.acerola.comic.module.main.home.component.HomeContinueBanner
import br.acerola.comic.module.main.home.component.HomeFilterSheet
import br.acerola.comic.module.main.home.component.HomeSearchBar
import br.acerola.comic.module.main.home.state.HomeAction
import br.acerola.comic.module.main.home.state.HomeUiState
import br.acerola.comic.module.main.remotelibrary.RemoteLibraryActivity
import br.acerola.comic.module.reader.ReaderActivity
import br.acerola.comic.ui.R
import br.acerola.comic.util.p2p.PairingCode
import kotlinx.coroutines.launch

private val homeTopOverlayHeight = 56.dp
private val homeTopContentOffset = 6.dp
private val mainBottomBarHeight = 64.dp
private val selectionActionButtonHeight = 54.dp

@Composable
fun Main.Home.Template.Screen(
    homeViewModel: HomeViewModel = hiltViewModel(),
    fileSystemAccessViewModel: FileSystemAccessViewModel = hiltViewModel(),
    comicDirectoryViewModel: ComicDirectoryViewModel = hiltViewModel(),
    onNavigateToConfig: () -> Unit,
) {
    val context = LocalContext.current
    val snackbarHostState = LocalSnackbarHostState.current
    val haptic = LocalHapticFeedback.current

    LaunchedEffect(Unit) {
        launch {
            homeViewModel.uiEvents.collect { message ->
                snackbarHostState.showSnackbar(message.uiMessage.asString(context), SnackbarVariant.Error)
            }
        }
        launch {
            fileSystemAccessViewModel.uiEvents.collect { message ->
                snackbarHostState.showSnackbar(message.uiMessage.asString(context), SnackbarVariant.Error)
            }
        }
        launch {
            comicDirectoryViewModel.uiEvents.collect { message ->
                snackbarHostState.showSnackbar(message.uiMessage.asString(context), SnackbarVariant.Error)
            }
        }
    }

    val layout by homeViewModel.selectedHomeLayout.collectAsStateWithLifecycle()
    val isIndexing by homeViewModel.isIndexing.collectAsStateWithLifecycle()
    val comics by homeViewModel.comics.collectAsStateWithLifecycle()
    val allCategories by homeViewModel.allCategories.collectAsStateWithLifecycle()
    val sortSettings by homeViewModel.sortSettings.collectAsStateWithLifecycle()
    val filterSettings by homeViewModel.filterSettings.collectAsStateWithLifecycle()
    val searchQuery by homeViewModel.searchQuery.collectAsStateWithLifecycle()
    val isSearchExpanded by homeViewModel.isSearchExpanded.collectAsStateWithLifecycle()
    val selectedComicIds by homeViewModel.selectedComicIds.collectAsStateWithLifecycle()
    val pairedPeers by homeViewModel.pairedPeers.collectAsStateWithLifecycle()

    val isSelectionMode = selectedComicIds.isNotEmpty()

    val selectedComics =
        remember(comics, selectedComicIds) {
            comics?.filter { selectedComicIds.contains(it.first.directory.id) } ?: emptyList()
        }

    val areAllSelectedHidden =
        remember(selectedComics) {
            selectedComics.isNotEmpty() && selectedComics.all { it.first.directory.hidden }
        }

    val selectedCategoryCounts =
        remember(selectedComics) {
            val counts = mutableMapOf<Long, Int>()
            selectedComics.forEach { item ->
                item.first.category?.id?.let { catId ->
                    counts[catId] = (counts[catId] ?: 0) + 1
                }
            }
            counts
        }

    val lastRead by remember(comics) {
        derivedStateOf {
            comics?.filter { it.second != null }?.maxByOrNull { it.second?.updatedAt ?: 0L }
        }
    }

    val uiState =
        HomeUiState(
            layout = layout,
            isIndexing = isIndexing,
            comics = comics,
            sortType = sortSettings.type,
            sortDirection = sortSettings.direction,
            filter = filterSettings,
        )

    var showFilterSheet by remember { mutableStateOf(false) }
    var selectedMangaForActions by remember { mutableStateOf<ComicDto?>(null) }
    var isBannerExpanded by remember { mutableStateOf(true) }
    var showRemoteLibraryPeerPicker by remember { mutableStateOf(false) }

    var showBatchCategorySheet by remember { mutableStateOf(false) }
    var showBatchHideDialog by remember { mutableStateOf(false) }
    var showBatchDeleteDialog by remember { mutableStateOf(false) }
    var showBatchClearMetadataDialog by remember { mutableStateOf(false) }

    val onAction: (HomeAction) -> Unit = { action ->
        when (action) {
            is HomeAction.UpdateLayout -> homeViewModel.updateHomeLayout(action.layout)
            is HomeAction.ClickManga -> {
                val intent =
                    Intent(context, ComicActivity::class.java).apply {
                        putExtra(ComicActivity.ChapterExtra.COMIC, action.comic)
                    }
                context.startActivity(intent)
            }

            is HomeAction.ClickContinue -> {
                val intent =
                    Intent(context, ReaderActivity::class.java).apply {
                        putExtra(ReaderActivity.PageExtra.INITIAL_PAGE, action.history.lastPage)
                        putExtra(ReaderActivity.PageExtra.MANGA_ID, action.comic.directory.id)
                        putExtra(ReaderActivity.PageExtra.CHAPTER_ID, action.history.chapterArchiveId)
                        putExtra(ReaderActivity.PageExtra.CHAPTER_SORT, action.history.chapterSort)
                    }
                context.startActivity(intent)
            }
        }
    }

    Box(modifier = Modifier.fillMaxSize()) {
        val comicList = uiState.comics
        val configuration = LocalConfiguration.current
        val isLandscape = configuration.orientation == Configuration.ORIENTATION_LANDSCAPE

        val statusBarHeight = WindowInsets.statusBars.asPaddingValues().calculateTopPadding()
        val topOverlayTopPadding =
            if (isLandscape) {
                statusBarHeight + SpacingTokens.Small
            } else {
                SpacingTokens.ExtraSmall
            }
        val searchBarTopPadding =
            if (isSearchExpanded && !isLandscape) {
                0.dp
            } else {
                topOverlayTopPadding
            }
        val gridTopPadding = topOverlayTopPadding + homeTopOverlayHeight + SpacingTokens.Medium + homeTopContentOffset
        val navBarHeight = WindowInsets.navigationBars.asPaddingValues().calculateBottomPadding()
        val bottomNavigationHeight = if (isLandscape) 0.dp else mainBottomBarHeight
        val bottomSafeAreaPadding = maxOf(navBarHeight, bottomNavigationHeight)
        val selectionDockHeight = selectionActionButtonHeight + SpacingTokens.Small + SpacingTokens.Small
        val selectionDockBottomPadding = bottomSafeAreaPadding + SpacingTokens.Small
        val selectionDockGridPadding =
            if (isSelectionMode) {
                selectionDockHeight + SpacingTokens.Small
            } else {
                0.dp
            }
        val fabGridPadding = 88.dp
        val gridBottomPadding =
            bottomSafeAreaPadding +
                if (isSelectionMode) {
                    selectionDockGridPadding + SpacingTokens.Medium
                } else {
                    fabGridPadding
                }

        when {
            comicList == null -> Unit
            comicList.isEmpty() ->
                EmptyState(
                    isIndexing = uiState.isIndexing,
                    onQuickSync = { comicDirectoryViewModel.syncLibrary() },
                    onFolderSelected = { uri ->
                        if (uri != null) {
                            fileSystemAccessViewModel.saveFolderUri(uri)
                            comicDirectoryViewModel.syncLibrary()
                        }
                    },
                )
            else -> {
                Box(modifier = Modifier.fillMaxSize()) {
                    if (!isSelectionMode) {
                        Main.Home.Component.HomeSearchBar(
                            query = searchQuery,
                            onQueryChange = { homeViewModel.updateSearchQuery(it) },
                            onSearch = { homeViewModel.updateSearchQuery(it) },
                            expanded = isSearchExpanded,
                            onExpandedChange = { homeViewModel.setSearchExpanded(it) },
                            comics = comicList,
                            onComicClick = { comic ->
                                homeViewModel.setSearchExpanded(false)
                                onAction(HomeAction.ClickManga(comic))
                            },
                            modifier =
                                Modifier
                                    .align(Alignment.TopCenter)
                                    .zIndex(1f)
                                    .fillMaxWidth(if (isLandscape) 0.7f else 1f)
                                    .padding(
                                        start = if (isLandscape) SizeTokens.LandscapeSidePadding else SpacingTokens.Small,
                                        end = if (isLandscape) SizeTokens.LandscapeSidePadding else SpacingTokens.Small,
                                        top = searchBarTopPadding,
                                    ),
                        )
                    }

                    AnimatedVisibility(
                        visible = isSelectionMode,
                        enter = fadeIn() + slideInVertically { -it },
                        exit = fadeOut() + slideOutVertically { -it },
                        modifier =
                            Modifier
                                .align(Alignment.TopCenter)
                                .zIndex(2f)
                                .fillMaxWidth(if (isLandscape) 0.7f else 1f)
                                .padding(
                                    start = if (isLandscape) SizeTokens.LandscapeSidePadding else SpacingTokens.Small,
                                    end = if (isLandscape) SizeTokens.LandscapeSidePadding else SpacingTokens.Small,
                                    top = topOverlayTopPadding,
                                ),
                    ) {
                        val allIds = comicList.map { it.first.directory.id }
                        val isAllSelected = selectedComicIds.size == allIds.size && allIds.isNotEmpty()

                        Acerola.Component.SelectionTopBar(
                            selectedCount = selectedComicIds.size,
                            isAllSelected = isAllSelected,
                            onClear = { homeViewModel.clearComicSelection() },
                            onToggleSelectAll = {
                                if (isAllSelected) {
                                    homeViewModel.clearComicSelection()
                                } else {
                                    homeViewModel.selectAllComics(allIds)
                                }
                            },
                            modifier = Modifier.fillMaxWidth(),
                        )
                    }

                    val (lastComic, lastHistory, _) = lastRead ?: Triple(null, null, 0)

                    val gridCells =
                        when (uiState.layout) {
                            HomeLayoutType.GRID -> GridCells.Adaptive(minSize = SizeTokens.ComicGridMinSize)
                            HomeLayoutType.LIST -> GridCells.Fixed(count = 1)
                        }

                    LazyVerticalGrid(
                        columns = gridCells,
                        verticalArrangement = Arrangement.spacedBy(space = SpacingTokens.Small),
                        horizontalArrangement = Arrangement.spacedBy(space = SpacingTokens.Small),
                        contentPadding =
                            PaddingValues(
                                start = SpacingTokens.Small,
                                top = gridTopPadding,
                                end = SpacingTokens.Small,
                                bottom = gridBottomPadding,
                            ),
                    ) {
                        if (lastComic != null && lastHistory != null && !isSelectionMode) {
                            item(
                                key = "continue_banner",
                                span = { GridItemSpan(maxLineSpan) },
                            ) {
                                Main.Home.Component.HomeContinueBanner(
                                    comic = lastComic,
                                    history = lastHistory,
                                    isExpanded = isBannerExpanded && !isLandscape,
                                    onExpandedChange = { isBannerExpanded = it },
                                    onContinueClick = { onAction(HomeAction.ClickContinue(lastComic, lastHistory)) },
                                    onComicClick = { onAction(HomeAction.ClickManga(lastComic)) },
                                    isLandscape = isLandscape,
                                    modifier = Modifier.padding(bottom = SpacingTokens.Small),
                                )
                            }
                        }

                        items(
                            items = comicList,
                            key = { (comic, _, _) -> "manga_${comic.directory.id}" },
                        ) { (comic, history, chapterCount) ->
                            val isSelected = selectedComicIds.contains(comic.directory.id)

                            val onItemClick = {
                                if (isSelectionMode) {
                                    homeViewModel.toggleComicSelection(comic.directory.id)
                                } else {
                                    onAction(HomeAction.ClickManga(comic))
                                }
                            }

                            val onItemLongClick = {
                                haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                                homeViewModel.toggleComicSelection(comic.directory.id)
                            }

                            val onShowItemActions = {
                                if (isSelectionMode) {
                                    homeViewModel.toggleComicSelection(comic.directory.id)
                                } else {
                                    selectedMangaForActions = comic
                                }
                            }
                            val onPlayClick =
                                if (!isSelectionMode && history != null) {
                                    { onAction(HomeAction.ClickContinue(comic, history)) }
                                } else {
                                    null
                                }

                            when (uiState.layout) {
                                HomeLayoutType.GRID ->
                                    Main.Home.Component.ComicGridItem(
                                        comic = comic,
                                        history = history,
                                        chapterCount = chapterCount,
                                        isSelected = isSelected,
                                        isSelectionMode = isSelectionMode,
                                        onLongClick = onItemLongClick,
                                        onShowActions = onShowItemActions,
                                        onClick = onItemClick,
                                    )

                                HomeLayoutType.LIST ->
                                    Main.Common.Component.ComicListItem(
                                        comic = comic,
                                        chapterCount = chapterCount,
                                        subtitle = comic.remoteInfo?.authors?.name,
                                        isSelected = isSelected,
                                        isSelectionMode = isSelectionMode,
                                        onLongClick = onItemLongClick,
                                        onClick = onItemClick,
                                        onPlayClick = onPlayClick,
                                        onShowActions = onShowItemActions,
                                    )
                            }
                        }
                    }
                }
            }
        }

        if (!isSelectionMode) {
            Acerola.Component.FabGroup(
                modifier = Modifier.padding(bottom = bottomSafeAreaPadding),
                icon = {
                    Icon(
                        imageVector = Icons.Default.Edit,
                        contentDescription = stringResource(id = R.string.description_icon_home_floating_tool_hub),
                    )
                },
                items =
                    listOf(
                        FabGroupItem(
                            onClick = {
                                onAction(
                                    HomeAction.UpdateLayout(
                                        layout =
                                            when (uiState.layout) {
                                                HomeLayoutType.LIST -> HomeLayoutType.GRID
                                                HomeLayoutType.GRID -> HomeLayoutType.LIST
                                            },
                                    ),
                                )
                            },
                            icon = {
                                Icon(
                                    imageVector =
                                        if (uiState.layout == HomeLayoutType.GRID) {
                                            Icons.AutoMirrored.Filled.ViewList
                                        } else {
                                            Icons.Default.GridView
                                        },
                                    contentDescription = stringResource(id = R.string.description_icon_home_change_layout),
                                )
                            },
                        ),
                        FabGroupItem(
                            icon = {
                                Icon(
                                    imageVector = Icons.Default.FilterList,
                                    contentDescription = stringResource(id = R.string.description_icon_home_filter),
                                )
                            },
                            onClick = { showFilterSheet = true },
                        ),
                        FabGroupItem(
                            icon = {
                                Icon(
                                    imageVector = Icons.Rounded.PhoneAndroid,
                                    contentDescription = stringResource(id = R.string.description_icon_home_browse_remote_library),
                                )
                            },
                            onClick = {
                                homeViewModel.loadPairedPeers()
                                showRemoteLibraryPeerPicker = true
                            },
                        ),
                    ),
            )
        }

        if (showRemoteLibraryPeerPicker) {
            Main.Common.Component.PeerPickerSheet(
                peers = pairedPeers,
                onSelect = { peerId ->
                    showRemoteLibraryPeerPicker = false
                    val peerDisplayName = pairedPeers.find { it.peerId == peerId }?.deviceName ?: PairingCode.shortId(peerId)
                    val intent =
                        Intent(context, RemoteLibraryActivity::class.java).apply {
                            putExtra(RemoteLibraryActivity.PeerExtra.PEER_ID, peerId)
                            putExtra(RemoteLibraryActivity.PeerExtra.PEER_DISPLAY_NAME, peerDisplayName)
                        }
                    context.startActivity(intent)
                },
                onDismiss = { showRemoteLibraryPeerPicker = false },
            )
        }

        AnimatedVisibility(
            visible = isSelectionMode,
            enter = slideInVertically { it } + fadeIn(),
            exit = slideOutVertically { it } + fadeOut(),
            modifier =
                Modifier
                    .align(Alignment.BottomCenter)
                    .zIndex(3f)
                    .padding(bottom = selectionDockBottomPadding),
        ) {
            Acerola.Component.SelectionActionDock(
                actions =
                    listOf(
                        SelectionAction(
                            icon = Icons.Rounded.Bookmark,
                            label = stringResource(id = R.string.action_bookmark),
                            onClick = { showBatchCategorySheet = true },
                        ),
                        SelectionAction(
                            icon = if (areAllSelectedHidden) Icons.Rounded.Visibility else Icons.Rounded.VisibilityOff,
                            label = stringResource(id = if (areAllSelectedHidden) R.string.action_unhide else R.string.action_hide),
                            onClick = { showBatchHideDialog = true },
                        ),
                        SelectionAction(
                            icon = Icons.Rounded.LayersClear,
                            label = stringResource(id = R.string.action_clear_metadata),
                            onClick = { showBatchClearMetadataDialog = true },
                        ),
                        SelectionAction(
                            icon = Icons.Rounded.Delete,
                            label = stringResource(id = R.string.action_delete),
                            onClick = { showBatchDeleteDialog = true },
                            isError = true,
                        ),
                    ),
                modifier =
                    Modifier
                        .fillMaxWidth(if (isLandscape) 0.6f else 1f)
                        .widthIn(max = SizeTokens.MaxContentWidth)
                        .padding(horizontal = SpacingTokens.Medium),
            )
        }

        val activeManga = selectedMangaForActions
        if (activeManga != null) {
            Main.Common.Component.ComicActionsSheet(
                comic = activeManga,
                categories = allCategories,
                onHide = { homeViewModel.hideManga(activeManga.directory.id) },
                onDelete = { homeViewModel.deleteComic(activeManga.directory.id) },
                onClearMetadata = { homeViewModel.clearMetadata(activeManga.directory.id) },
                onBookmark = { categoryId -> homeViewModel.setMangaCategory(activeManga.directory.id, categoryId) },
                onDismiss = { selectedMangaForActions = null },
                pairedPeers = pairedPeers,
                onLoadPairedPeers = homeViewModel::loadPairedPeers,
                onSyncWithPeer = { peerId, direction ->
                    homeViewModel.syncComicWithPeer(peerId, activeManga.directory.name, direction)
                },
            )
        }

        if (showBatchCategorySheet) {
            Main.Common.Component.BatchComicCategorySheet(
                categories = allCategories,
                categoryCounts = selectedCategoryCounts,
                totalSelectedCount = selectedComicIds.size,
                onSelectCategory = { categoryId ->
                    homeViewModel.setSelectedComicsCategory(categoryId)
                    showBatchCategorySheet = false
                },
                onRemoveCategory = {
                    homeViewModel.setSelectedComicsCategory(null)
                    showBatchCategorySheet = false
                },
                onDismiss = { showBatchCategorySheet = false },
            )
        }

        if (showBatchHideDialog) {
            Acerola.Component.Dialog(
                show = true,
                onDismiss = { showBatchHideDialog = false },
                title =
                    stringResource(
                        id = if (areAllSelectedHidden) R.string.dialog_batch_unhide_title else R.string.dialog_batch_hide_title,
                        selectedComicIds.size,
                    ),
                confirmButtonContent = {
                    Acerola.Component.DialogButton(
                        text = stringResource(id = if (areAllSelectedHidden) R.string.action_unhide else R.string.action_hide),
                        onClick = {
                            showBatchHideDialog = false
                            if (areAllSelectedHidden) {
                                homeViewModel.unhideSelectedComics()
                            } else {
                                homeViewModel.hideSelectedComics()
                            }
                        },
                        containerColor = MaterialTheme.colorScheme.primary,
                        contentColor = MaterialTheme.colorScheme.onPrimary,
                        fontWeight = FontWeight.Bold,
                    )
                },
                dismissButtonContent = {
                    Acerola.Component.DialogButton(
                        text = stringResource(id = R.string.action_cancel),
                        onClick = { showBatchHideDialog = false },
                        contentColor = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                },
                content = {
                    Text(
                        text =
                            stringResource(
                                id = if (areAllSelectedHidden) R.string.dialog_batch_unhide_message else R.string.dialog_batch_hide_message,
                            ),
                    )
                },
            )
        }

        if (showBatchDeleteDialog) {
            Acerola.Component.Dialog(
                show = true,
                onDismiss = { showBatchDeleteDialog = false },
                title = stringResource(id = R.string.dialog_batch_delete_title, selectedComicIds.size),
                confirmButtonContent = {
                    Acerola.Component.DialogButton(
                        text = stringResource(id = R.string.action_delete),
                        onClick = {
                            showBatchDeleteDialog = false
                            homeViewModel.deleteSelectedComics()
                        },
                        containerColor = MaterialTheme.colorScheme.error,
                        contentColor = MaterialTheme.colorScheme.onError,
                        fontWeight = FontWeight.Bold,
                    )
                },
                dismissButtonContent = {
                    Acerola.Component.DialogButton(
                        text = stringResource(id = R.string.action_cancel),
                        onClick = { showBatchDeleteDialog = false },
                        contentColor = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                },
                content = { Text(text = stringResource(id = R.string.dialog_batch_delete_message, selectedComicIds.size)) },
            )
        }

        if (showBatchClearMetadataDialog) {
            Acerola.Component.Dialog(
                show = true,
                onDismiss = { showBatchClearMetadataDialog = false },
                title = stringResource(id = R.string.dialog_batch_clear_metadata_title, selectedComicIds.size),
                confirmButtonContent = {
                    Acerola.Component.DialogButton(
                        text = stringResource(id = R.string.action_clear_metadata),
                        onClick = {
                            showBatchClearMetadataDialog = false
                            homeViewModel.clearMetadataForSelectedComics()
                        },
                        containerColor = MaterialTheme.colorScheme.error,
                        contentColor = MaterialTheme.colorScheme.onError,
                        fontWeight = FontWeight.Bold,
                    )
                },
                dismissButtonContent = {
                    Acerola.Component.DialogButton(
                        text = stringResource(id = R.string.action_cancel),
                        onClick = { showBatchClearMetadataDialog = false },
                        contentColor = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                },
                content = { Text(text = stringResource(id = R.string.dialog_batch_clear_metadata_message, selectedComicIds.size)) },
            )
        }

        if (showFilterSheet) {
            Main.Home.Component.HomeFilterSheet(
                sortSettings = sortSettings,
                filterSettings = filterSettings,
                categories = allCategories,
                onDismiss = { showFilterSheet = false },
                onSortChange = { homeViewModel.updateSortSettings(it) },
                onFilterChange = { homeViewModel.updateFilterSettings(it) },
            )
        }
    }
}

@Composable
private fun EmptyState(
    isIndexing: Boolean,
    onQuickSync: () -> Unit,
    onFolderSelected: (Uri?) -> Unit,
) {
    val launcher =
        rememberLauncherForActivityResult(
            contract = ActivityResultContracts.OpenDocumentTree(),
            onResult = { uri ->
                onFolderSelected(uri)
            },
        )

    val infiniteTransition = rememberInfiniteTransition(label = "sync_spin")
    val angle by infiniteTransition.animateFloat(
        initialValue = 0f,
        targetValue = 360f,
        animationSpec =
            infiniteRepeatable(
                animation = tween(durationMillis = 1000, easing = LinearEasing),
                repeatMode = RepeatMode.Restart,
            ),
        label = "spin_angle",
    )

    val scrollState = rememberScrollState()

    BoxWithConstraints(
        modifier =
            Modifier
                .fillMaxSize()
                .verticalScroll(scrollState)
                .padding(SpacingTokens.Large),
        contentAlignment = Alignment.Center,
    ) {
        val isNarrow = maxWidth < 400.dp

        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center,
            modifier =
                Modifier
                    .widthIn(max = SizeTokens.MaxContentWidth)
                    .fillMaxWidth(),
        ) {
            Box(
                contentAlignment = Alignment.Center,
                modifier =
                    Modifier
                        .size(64.dp)
                        .clip(ShapeTokens.Large)
                        .background(MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.6f)),
            ) {
                Icon(
                    imageVector = Icons.Default.FolderOpen,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.primary,
                    modifier = Modifier.size(32.dp),
                )
            }

            Spacer(modifier = Modifier.height(SpacingTokens.Medium))

            Text(
                text = stringResource(id = R.string.description_text_home_empty_state),
                style = MaterialTheme.typography.titleLarge,
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.onBackground,
                textAlign = TextAlign.Center,
            )

            Spacer(modifier = Modifier.height(SpacingTokens.ExtraSmall))

            Text(
                text = stringResource(id = R.string.description_text_home_empty_subtitle),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                textAlign = TextAlign.Center,
                modifier = Modifier.padding(horizontal = SpacingTokens.Small),
            )

            Spacer(modifier = Modifier.height(SpacingTokens.Large))

            if (isNarrow) {
                Column(
                    modifier = Modifier.fillMaxWidth(),
                    verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    Button(
                        onClick = onQuickSync,
                        shape = ShapeTokens.Medium,
                        modifier =
                            Modifier
                                .fillMaxWidth()
                                .heightIn(min = 44.dp),
                    ) {
                        Icon(
                            imageVector = Icons.Default.Refresh,
                            contentDescription = null,
                            modifier =
                                Modifier
                                    .size(18.dp)
                                    .graphicsLayer { rotationZ = if (isIndexing) angle else 0f },
                        )
                        Spacer(modifier = Modifier.width(SpacingTokens.Small))
                        Text(
                            text = stringResource(id = R.string.action_home_quick_sync),
                            fontWeight = FontWeight.SemiBold,
                        )
                    }

                    OutlinedButton(
                        onClick = { launcher.launch(null) },
                        shape = ShapeTokens.Medium,
                        modifier =
                            Modifier
                                .fillMaxWidth()
                                .heightIn(min = 44.dp),
                    ) {
                        Icon(
                            imageVector = Icons.Default.FolderOpen,
                            contentDescription = null,
                            modifier = Modifier.size(18.dp),
                        )
                        Spacer(modifier = Modifier.width(SpacingTokens.Small))
                        Text(
                            text = stringResource(id = R.string.action_home_select_folder),
                            fontWeight = FontWeight.Medium,
                        )
                    }
                }
            } else {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small, Alignment.CenterHorizontally),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Button(
                        onClick = onQuickSync,
                        shape = ShapeTokens.Medium,
                        modifier = Modifier.heightIn(min = 44.dp),
                    ) {
                        Icon(
                            imageVector = Icons.Default.Refresh,
                            contentDescription = null,
                            modifier =
                                Modifier
                                    .size(18.dp)
                                    .graphicsLayer { rotationZ = if (isIndexing) angle else 0f },
                        )
                        Spacer(modifier = Modifier.width(SpacingTokens.Small))
                        Text(
                            text = stringResource(id = R.string.action_home_quick_sync),
                            fontWeight = FontWeight.SemiBold,
                        )
                    }

                    OutlinedButton(
                        onClick = { launcher.launch(null) },
                        shape = ShapeTokens.Medium,
                        modifier = Modifier.heightIn(min = 44.dp),
                    ) {
                        Icon(
                            imageVector = Icons.Default.FolderOpen,
                            contentDescription = null,
                            modifier = Modifier.size(18.dp),
                        )
                        Spacer(modifier = Modifier.width(SpacingTokens.Small))
                        Text(
                            text = stringResource(id = R.string.action_home_select_folder),
                            fontWeight = FontWeight.Medium,
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
private fun ScreenPreview() {
    AcerolaTheme {
        EmptyState(
            isIndexing = false,
            onQuickSync = {},
            onFolderSelected = {},
        )
    }
}
