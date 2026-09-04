package br.acerola.comic.module.reader.component
import android.content.res.Configuration
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.calculatePan
import androidx.compose.foundation.gestures.calculateZoom
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.input.pointer.positionChanged
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.module.reader.Reader
import coil.compose.AsyncImage
import coil.request.ImageRequest

@Composable
fun Reader.Component.WebtoonReader(
    pageCount: Int,
    comicId: Long,
    chapterId: Long?,
    onUiToggle: () -> Unit,
    listState: LazyListState,
    onPageRequest: (Int) -> Unit,
    onZoomChange: (Boolean) -> Unit,
) {
    var scale by remember { mutableFloatStateOf(value = 1f) }
    var offset by remember { mutableStateOf(Offset.Zero) }

    LaunchedEffect(key1 = scale) {
        onZoomChange(scale > 1.0f)
    }

    LaunchedEffect(listState) {
        snapshotFlow { listState.firstVisibleItemIndex }
            .collect { index -> onPageRequest(index) }
    }

    Box(
        modifier =
            Modifier
                .fillMaxSize()
                .pointerInput(key1 = Unit) {
                    awaitEachGesture {
                        awaitFirstDown(requireUnconsumed = false)
                        do {
                            val isZoomed = scale > 1f
                            val event = awaitPointerEvent()
                            val isMultiTouch = event.changes.size > 1

                            if (isMultiTouch || isZoomed) {
                                val zoom = event.calculateZoom()
                                val pan = event.calculatePan()

                                if (zoom != 1f || pan != Offset.Zero) {
                                    val newScale = (scale * zoom).coerceIn(1f, 3f)

                                    val maxTranslateX = (newScale - 1) * size.width / 2
                                    val maxTranslateY = (newScale - 1) * size.height / 2

                                    val newOffset =
                                        if (newScale > 1f) {
                                            Offset(
                                                x = (offset.x + pan.x * newScale).coerceIn(-maxTranslateX, maxTranslateX),
                                                y = (offset.y + pan.y * newScale).coerceIn(-maxTranslateY, maxTranslateY),
                                            )
                                        } else {
                                            Offset.Zero
                                        }

                                    scale = newScale
                                    offset = newOffset

                                    event.changes.forEach {
                                        if (it.positionChanged()) it.consume()
                                    }
                                }
                            }
                        } while (event.changes.any { it.pressed })
                    }
                },
    ) {
        LazyColumn(
            state = listState,
            userScrollEnabled = scale == 1f,
            modifier =
                Modifier
                    .fillMaxSize()
                    .graphicsLayer(
                        scaleX = scale,
                        scaleY = scale,
                        translationX = offset.x,
                        translationY = offset.y,
                    ),
        ) {
            items(pageCount) { index ->
                AsyncImage(
                    model =
                        ImageRequest
                            .Builder(LocalContext.current)
                            .data("acerola://page/$comicId/$chapterId/$index")
                            .build(),
                    contentDescription = null,
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .wrapContentHeight()
                            .testTag("webtoon_page_$index")
                            .pointerInput(key1 = Unit) {
                                detectTapGestures(
                                    onTap = {
                                        onUiToggle()
                                    },
                                    onDoubleTap = {
                                        if (scale > 1f) {
                                            scale = 1f
                                            offset = Offset.Zero
                                        } else {
                                            scale = 2f
                                        }
                                    },
                                )
                            },
                    contentScale = ContentScale.FillWidth,
                )
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun WebtoonReaderPreview() {
    AcerolaTheme {
        val listState = rememberLazyListState()
        Reader.Component.WebtoonReader(
            pageCount = 5,
            comicId = 1L,
            chapterId = 1L,
            onUiToggle = {},
            listState = listState,
            onPageRequest = {},
            onZoomChange = {},
        )
    }
}
