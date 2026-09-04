package br.acerola.comic.common.ux.component
import android.content.res.Configuration
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CornerSize
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalInspectionMode
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.ShapeTokens
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.ui.R

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun Acerola.Component.AdaptiveSheet(
    onDismissRequest: () -> Unit,
    isScrollable: Boolean = true,
    containerColor: Color = MaterialTheme.colorScheme.surfaceContainerLow,
    contentColor: Color = MaterialTheme.colorScheme.onSurface,
    content: @Composable ColumnScope.() -> Unit,
) {
    val configuration = LocalConfiguration.current
    val isLandscape = configuration.orientation == Configuration.ORIENTATION_LANDSCAPE
    val isPreview = LocalInspectionMode.current

    val scrollModifier =
        if (isScrollable) {
            Modifier.verticalScroll(rememberScrollState())
        } else {
            Modifier
        }

    if (isPreview) {
        Box(modifier = Modifier.fillMaxSize()) {
            Surface(
                modifier =
                    if (isLandscape) {
                        Modifier
                            .width(SizeTokens.SideSheetWidth)
                            .fillMaxHeight()
                            .align(Alignment.CenterEnd)
                    } else {
                        Modifier
                            .fillMaxWidth()
                            .align(Alignment.BottomCenter)
                    },
                color = containerColor,
                contentColor = contentColor,
                shape =
                    if (isLandscape) {
                        ShapeTokens.Large.copy(topEnd = CornerSize(0.dp), bottomEnd = CornerSize(0.dp))
                    } else {
                        ShapeTokens.Large.copy(bottomStart = CornerSize(0.dp), bottomEnd = CornerSize(0.dp))
                    },
            ) {
                Column(
                    modifier =
                        Modifier
                            .then(if (isLandscape) Modifier.fillMaxSize() else Modifier.fillMaxWidth())
                            .then(if (!isLandscape) Modifier.heightIn(max = configuration.screenHeightDp.dp) else Modifier)
                            .then(scrollModifier),
                ) {
                    content()
                }
            }
        }
    } else if (isLandscape) {
        Dialog(
            onDismissRequest = onDismissRequest,
            properties = DialogProperties(usePlatformDefaultWidth = false),
        ) {
            Box(modifier = Modifier.fillMaxSize()) {
                Box(
                    modifier =
                        Modifier
                            .fillMaxSize()
                            .clickable(
                                indication = null,
                                interactionSource = remember { MutableInteractionSource() },
                            ) { onDismissRequest() },
                )
                Surface(
                    modifier =
                        Modifier
                            .width(SizeTokens.SideSheetWidth)
                            .fillMaxHeight()
                            .align(Alignment.CenterEnd),
                    color = containerColor,
                    contentColor = contentColor,
                    shape = ShapeTokens.Large.copy(topEnd = CornerSize(0.dp), bottomEnd = CornerSize(0.dp)),
                ) {
                    Column(
                        modifier =
                            Modifier
                                .fillMaxSize()
                                .then(scrollModifier),
                    ) {
                        content()
                    }
                }
            }
        }
    } else {
        ModalBottomSheet(
            onDismissRequest = onDismissRequest,
            sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
            containerColor = containerColor,
            contentColor = contentColor,
        ) {
            Column(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .heightIn(max = configuration.screenHeightDp.dp)
                        .then(scrollModifier),
            ) {
                content()
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun AdaptiveSheetPreview() {
    AcerolaTheme {
        Acerola.Component.AdaptiveSheet(onDismissRequest = {}) {
            Column(modifier = Modifier.padding(24.dp)) {
                Text(
                    text = stringResource(R.string.title_home_filter_sheet),
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                Text(
                    text = stringResource(R.string.description_text_home_empty_subtitle),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}
