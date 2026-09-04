package br.acerola.comic.module.reader.component
import android.content.res.Configuration
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.pager.PagerState
import androidx.compose.foundation.pager.VerticalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.config.preference.types.ReadingMode
import br.acerola.comic.module.reader.Reader
import br.acerola.comic.module.reader.gesture.ZoomablePageImage
import br.acerola.comic.module.reader.state.TapArea

@Composable
fun Reader.Component.VerticalPagedReader(
    comicId: Long,
    chapterId: Long?,
    pagerState: PagerState,
    onUiToggle: () -> Unit,
    onPrevClick: () -> Unit,
    onNextClick: () -> Unit,
    onZoomChange: (Boolean) -> Unit,
) {
    var isZoomed by remember { mutableStateOf(false) }

    VerticalPager(
        key = { it },
        state = pagerState,
        modifier = Modifier.fillMaxSize(),
        userScrollEnabled = !isZoomed,
    ) { index ->
        Reader.Gesture.ZoomablePageImage(
            comicId = comicId,
            chapterId = chapterId,
            pageIndex = index,
            orientation = ReadingMode.VERTICAL,
            onZoomStatusChange = { zoomed ->
                onZoomChange(zoomed)
            },
            onAreaTap = { area ->
                when (area) {
                    TapArea.TOP -> onPrevClick()
                    TapArea.BOTTOM -> onNextClick()
                    TapArea.CENTER -> onUiToggle()
                    else -> {}
                }
            },
        )
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun VerticalPagedReaderPreview() {
    AcerolaTheme {
        val pagerState = rememberPagerState(pageCount = { 5 })
        Reader.Component.VerticalPagedReader(
            comicId = 1L,
            chapterId = 1L,
            pagerState = pagerState,
            onUiToggle = {},
            onPrevClick = {},
            onNextClick = {},
            onZoomChange = {},
        )
    }
}
