package br.acerola.comic.module.reader.template
import android.content.res.Configuration
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.pager.PagerState
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.config.preference.types.ReadingMode
import br.acerola.comic.module.reader.Reader
import br.acerola.comic.module.reader.component.HorizontalPagedReader
import br.acerola.comic.module.reader.component.VerticalPagedReader
import br.acerola.comic.module.reader.component.WebtoonReader

@Composable
fun Reader.Template.PageContent(
    pageCount: Int,
    pagerState: PagerState,
    onUiToggle: () -> Unit,
    onPrevClick: () -> Unit,
    onNextClick: () -> Unit,
    readingMode: ReadingMode,
    listState: LazyListState,
    comicId: Long,
    chapterId: Long?,
    onPageRequest: (Int) -> Unit,
    onZoomChange: (Boolean) -> Unit,
) {
    Box(modifier = Modifier.fillMaxSize()) {
        when (readingMode) {
            ReadingMode.HORIZONTAL -> {
                Reader.Component.HorizontalPagedReader(
                    comicId = comicId,
                    chapterId = chapterId,
                    onUiToggle = onUiToggle,
                    pagerState = pagerState,
                    onPrevClick = onPrevClick,
                    onNextClick = onNextClick,
                    onZoomChange = onZoomChange,
                )
            }

            ReadingMode.VERTICAL -> {
                Reader.Component.VerticalPagedReader(
                    comicId = comicId,
                    chapterId = chapterId,
                    onUiToggle = onUiToggle,
                    pagerState = pagerState,
                    onPrevClick = onPrevClick,
                    onNextClick = onNextClick,
                    onZoomChange = onZoomChange,
                )
            }

            ReadingMode.WEBTOON -> {
                Reader.Component.WebtoonReader(
                    pageCount = pageCount,
                    comicId = comicId,
                    chapterId = chapterId,
                    listState = listState,
                    onPageRequest = onPageRequest,
                    onUiToggle = onUiToggle,
                    onZoomChange = onZoomChange,
                )
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun PageContentPreview() {
    AcerolaTheme {
        val pagerState = rememberPagerState(pageCount = { 5 })
        val listState = rememberLazyListState()
        Reader.Template.PageContent(
            pageCount = 5,
            pagerState = pagerState,
            onUiToggle = {},
            onPrevClick = {},
            onNextClick = {},
            readingMode = ReadingMode.VERTICAL,
            listState = listState,
            comicId = 1L,
            chapterId = 1L,
            onPageRequest = {},
            onZoomChange = {},
        )
    }
}
