package br.acerola.comic.module.reader.gesture

import android.content.res.Configuration
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.calculatePan
import androidx.compose.foundation.gestures.calculateZoom
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.input.pointer.positionChanged
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.config.preference.types.ReadingMode
import br.acerola.comic.module.reader.Reader
import br.acerola.comic.module.reader.state.TapArea
import coil.compose.AsyncImage
import coil.request.ImageRequest

@Composable
fun Reader.Gesture.ZoomablePageImage(
    comicId: Long,
    chapterId: Long?,
    pageIndex: Int,
    onAreaTap: (TapArea) -> Unit,
    onZoomStatusChange: (Boolean) -> Unit,
    orientation: ReadingMode = ReadingMode.VERTICAL,
) {
    var scale by remember { mutableFloatStateOf(1f) }
    var offset by remember { mutableStateOf(Offset.Zero) }

    LaunchedEffect(scale) {
        onZoomStatusChange(scale > 1f)
    }

    Box(
        modifier =
            Modifier
                .fillMaxSize()
                .onSizeChanged { }
                .pointerInput(Unit) {
                    awaitEachGesture {
                        awaitFirstDown(requireUnconsumed = false)
                        do {
                            val event = awaitPointerEvent()
                            val isZoomed = scale > 1f
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
        AsyncImage(
            model =
                ImageRequest
                    .Builder(LocalContext.current)
                    .data("acerola://page/$comicId/$chapterId/$pageIndex")
                    .build(),
            contentDescription = null,
            contentScale = ContentScale.Fit,
            modifier =
                Modifier
                    .fillMaxSize()
                    .testTag("zoomable_image")
                    .graphicsLayer(
                        scaleX = scale,
                        scaleY = scale,
                        translationX = offset.x,
                        translationY = offset.y,
                    ).pointerInput(orientation) {
                        detectTapGestures(
                            onTap = { tapOffset ->
                                val area =
                                    if (orientation == ReadingMode.HORIZONTAL) {
                                        when {
                                            tapOffset.x < size.width / 3f -> TapArea.LEFT
                                            tapOffset.x > size.width * 2f / 3f -> TapArea.RIGHT
                                            else -> TapArea.CENTER
                                        }
                                    } else {
                                        when {
                                            tapOffset.y < size.height / 3f -> TapArea.TOP
                                            tapOffset.y > size.height * 2f / 3f -> TapArea.BOTTOM
                                            else -> TapArea.CENTER
                                        }
                                    }
                                onAreaTap(area)
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
        )
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ZoomablePageImagePreview() {
    AcerolaTheme {
        Reader.Gesture.ZoomablePageImage(
            comicId = 1L,
            chapterId = 1L,
            pageIndex = 0,
            onAreaTap = {},
            onZoomStatusChange = {},
        )
    }
}
