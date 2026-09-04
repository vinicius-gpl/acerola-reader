package br.acerola.comic.module.reader
import android.content.res.Configuration
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import br.acerola.comic.common.state.LocalSnackbarHostState
import br.acerola.comic.common.ux.component.SnackbarVariant
import br.acerola.comic.common.ux.component.showSnackbar
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.config.preference.types.ReadingMode
import br.acerola.comic.dto.archive.ChapterFileDto
import br.acerola.comic.module.reader.state.ReaderAction
import br.acerola.comic.module.reader.template.BottomControls
import br.acerola.comic.module.reader.template.PageContent
import br.acerola.comic.module.reader.template.SettingsSheet
import br.acerola.comic.module.reader.template.TopBar
import br.acerola.comic.ui.R
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.distinctUntilChanged

@Composable
fun ReaderScreen(
    chapter: ChapterFileDto?,
    chapterId: Long = -1L,
    chapterSort: String = "",
    initialPage: Int,
    comicId: Long,
    onBackClick: () -> Unit,
    viewModel: ReaderViewModel = hiltViewModel(),
) {
    val context = LocalContext.current

    val state by viewModel.uiState.collectAsState()
    val snackbarHostState = LocalSnackbarHostState.current

    val activeChapter = state.currentChapter ?: chapter

    LaunchedEffect(chapter, chapterId, chapterSort, comicId) {
        if (chapter != null) {
            viewModel.openChapter(comicId, chapter, initialPage)
            return@LaunchedEffect
        }

        if (chapterId != -1L || chapterSort.isNotEmpty()) {
            viewModel.loadAndOpenChapter(
                comicId = comicId,
                chapterId = if (chapterId == -1L) null else chapterId,
                chapterSort = chapterSort,
                initialPage = initialPage,
            )
        }
    }

    LaunchedEffect(Unit) {
        viewModel.uiEvents.collect { message ->
            snackbarHostState.showSnackbar(message.uiMessage.asString(context), SnackbarVariant.Error)
        }
    }

    val onAction: (ReaderAction) -> Unit = { action ->
        when (action) {
            ReaderAction.NavigateBack -> onBackClick()
            ReaderAction.ToggleUi -> viewModel.toggleUiVisibility()
            is ReaderAction.UpdateReadingMode -> viewModel.updateReadingMode(action.mode)
            is ReaderAction.ChangePage -> viewModel.onSliderChanged(action.index)
            ReaderAction.LoadNextChapter -> viewModel.loadNextChapter(comicId)
            ReaderAction.LoadPreviousChapter -> viewModel.loadPreviousChapter(comicId)
            is ReaderAction.PageVisible ->
                viewModel.onPageVisible(
                    comicId,
                    state.currentChapter?.chapterSort ?: chapterSort,
                    state.currentChapter?.id ?: (if (chapterId == -1L) null else chapterId),
                    action.index,
                )
            is ReaderAction.CurrentPageChanged ->
                viewModel.onCurrentPageChanged(
                    comicId,
                    state.currentChapter?.chapterSort ?: chapterSort,
                    state.currentChapter?.id ?: (if (chapterId == -1L) null else chapterId),
                    action.index,
                )
        }
    }

    if (state.isLoading || state.pageCount == 0) {
        Box(
            modifier = Modifier.fillMaxSize(),
            contentAlignment = Alignment.Center,
        ) {
            CircularProgressIndicator(
                color = MaterialTheme.colorScheme.primary,
            )
        }
        return
    }

    val pagerState =
        rememberPagerState(
            initialPage = initialPage.coerceIn(0, (state.pageCount - 1).coerceAtLeast(0)),
            pageCount = { state.pageCount },
        )
    val listState =
        rememberLazyListState(
            initialFirstVisibleItemIndex = initialPage.coerceIn(0, (state.pageCount - 1).coerceAtLeast(0)),
        )

    LaunchedEffect(pagerState, listState, state.readingMode, comicId, chapter, chapterId) {
        snapshotFlow {
            if (state.readingMode == ReadingMode.WEBTOON) {
                listState.firstVisibleItemIndex
            } else {
                pagerState.currentPage
            }
        }.distinctUntilChanged().collectLatest { index ->
            onAction(ReaderAction.CurrentPageChanged(index))
        }
    }

    LaunchedEffect(key1 = state.currentPage) {
        if (state.readingMode == ReadingMode.WEBTOON) {
            if (listState.firstVisibleItemIndex != state.currentPage) {
                listState.scrollToItem(index = state.currentPage)
            }
        } else {
            if (pagerState.currentPage != state.currentPage) {
                pagerState.animateScrollToPage(state.currentPage)
            }
        }
    }

    var showSettings by remember { mutableStateOf(value = false) }

    Box(modifier = Modifier.fillMaxSize()) {
        Reader.Template.PageContent(
            comicId = comicId,
            chapterId = state.currentChapter?.id ?: (if (chapterId == -1L) null else chapterId),
            listState = listState,
            pagerState = pagerState,
            pageCount = state.pageCount,
            readingMode = state.readingMode,
            onUiToggle = { onAction(ReaderAction.ToggleUi) },
            onPageRequest = { index -> onAction(ReaderAction.PageVisible(index)) },
            onPrevClick = { onAction(ReaderAction.ChangePage(state.currentPage - 1)) },
            onNextClick = { onAction(ReaderAction.ChangePage(state.currentPage + 1)) },
            onZoomChange = {},
        )

        // UI Overlay
        Reader.Template.TopBar(
            title = activeChapter?.name ?: stringResource(id = R.string.label_reader_activity),
            subtitle = stringResource(id = R.string.label_reader_chapter_order, activeChapter?.chapterSort ?: "-"),
            isVisible = state.isUiVisible,
            onBackClick = { onAction(ReaderAction.NavigateBack) },
            onSettingsClick = { showSettings = true },
        )

        AnimatedVisibility(
            visible = state.isUiVisible,
            enter = slideInVertically { it },
            exit = slideOutVertically { it },
            modifier = Modifier.fillMaxSize(),
        ) {
            Box(contentAlignment = Alignment.BottomCenter) {
                Reader.Template.BottomControls(
                    isLoading = state.isLoading,
                    pageCount = state.pageCount,
                    currentPage = state.currentPage,
                    isChapterRead = state.isChapterRead,
                    hasNextChapter = state.nextChapterId != null,
                    hasPreviousChapter = state.previousChapterId != null,
                    enableNavigation = state.readingMode != ReadingMode.WEBTOON,
                    onNextChapterClick = { onAction(ReaderAction.LoadNextChapter) },
                    onPrevClick = { onAction(ReaderAction.ChangePage(state.currentPage - 1)) },
                    onNextClick = { onAction(ReaderAction.ChangePage(state.currentPage + 1)) },
                    onPreviousChapterClick = { onAction(ReaderAction.LoadPreviousChapter) },
                )
            }
        }

        if (showSettings) {
            Reader.Template.SettingsSheet(
                onDismissRequest = { showSettings = false },
                currentMode = state.readingMode,
                onModeSelected = { mode ->
                    onAction(ReaderAction.UpdateReadingMode(mode))
                    showSettings = false
                },
            )
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ReaderScreenPreview() {
    AcerolaTheme {
        Column(modifier = Modifier.fillMaxSize()) {
            Reader.Template.TopBar(
                title = "Capítulo 01",
                subtitle = "Sample Comic",
                isVisible = true,
                onBackClick = {},
                onSettingsClick = {},
            )
            Spacer(modifier = Modifier.weight(1f))
            Reader.Template.BottomControls(
                pageCount = 20,
                currentPage = 5,
                onPrevClick = {},
                onNextClick = {},
                onNextChapterClick = {},
                onPreviousChapterClick = {},
            )
        }
    }
}
