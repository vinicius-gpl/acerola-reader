package br.acerola.comic.module.reader.component

import android.content.res.Configuration
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.PagerState
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
fun Reader.Component.HorizontalPagedReader(
    comicId: Long,
    chapterId: Long?,
    pagerState: PagerState,
    onUiToggle: () -> Unit,
    onPrevClick: () -> Unit,
    onNextClick: () -> Unit,
    onZoomChange: (Boolean) -> Unit,
) {
    var isZoomed by remember { mutableStateOf(false) }

    HorizontalPager(
        state = pagerState,
        modifier = Modifier.fillMaxSize(),
        key = { it },
        userScrollEnabled = !isZoomed,
    ) { index ->
        Reader.Gesture.ZoomablePageImage(
            comicId = comicId,
            chapterId = chapterId,
            pageIndex = index,
            orientation = ReadingMode.HORIZONTAL,
            onZoomStatusChange = { zoomed ->
                onZoomChange(zoomed)
            },
            onAreaTap = { area ->
                when (area) {
                    TapArea.LEFT -> onPrevClick()
                    TapArea.RIGHT -> onNextClick()
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
private fun HorizontalPagedReaderPreview() {
    AcerolaTheme {
        val pagerState = rememberPagerState(pageCount = { 5 })
        Reader.Component.HorizontalPagedReader(
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
