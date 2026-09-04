package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.BasicAlertDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ProvideTextStyle
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.ui.R

@Composable
fun Acerola.Component.DialogButton(
    text: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    containerColor: Color = Color.Transparent,
    contentColor: Color = MaterialTheme.colorScheme.primary,
    fontWeight: FontWeight = FontWeight.Normal,
) {
    Surface(
        onClick = onClick,
        modifier = modifier.fillMaxSize(),
        color = containerColor,
        contentColor = contentColor,
    ) {
        Box(
            modifier =
                Modifier
                    .fillMaxSize()
                    .padding(vertical = 16.dp),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                text = text,
                style = MaterialTheme.typography.labelLarge.copy(fontSize = 15.sp),
                fontWeight = fontWeight,
                textAlign = TextAlign.Center,
            )
        }
    }
}

@Composable
@OptIn(ExperimentalMaterial3Api::class)
fun Acerola.Component.Dialog(
    show: Boolean,
    onDismiss: () -> Unit,
    title: String? = null,
    confirmButtonContent: (@Composable () -> Unit)? = null,
    dismissButtonContent: (@Composable () -> Unit)? = null,
    content: @Composable () -> Unit,
) {
    if (show) {
        BasicAlertDialog(
            onDismissRequest = onDismiss,
            modifier = Modifier.clip(MaterialTheme.shapes.extraLarge),
        ) {
            Surface(
                color = MaterialTheme.colorScheme.surfaceContainerHigh,
                shape = MaterialTheme.shapes.extraLarge,
                tonalElevation = 6.dp,
            ) {
                Column(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    Column(
                        modifier =
                            Modifier
                                .padding(24.dp)
                                .fillMaxWidth(),
                        horizontalAlignment = Alignment.CenterHorizontally,
                    ) {
                        title?.let {
                            Text(
                                text = it,
                                style = MaterialTheme.typography.headlineSmall,
                                fontWeight = FontWeight.Bold,
                                textAlign = TextAlign.Center,
                                color = MaterialTheme.colorScheme.onSurface,
                            )
                            Spacer(modifier = Modifier.height(12.dp))
                        }

                        Box(
                            modifier = Modifier.fillMaxWidth(),
                            contentAlignment = Alignment.Center,
                        ) {
                            CompositionLocalProvider(
                                LocalContentColor provides MaterialTheme.colorScheme.onSurfaceVariant,
                            ) {
                                ProvideTextStyle(value = MaterialTheme.typography.bodyMedium.copy(textAlign = TextAlign.Center)) {
                                    content()
                                }
                            }
                        }
                    }

                    HorizontalDivider(
                        modifier = Modifier.fillMaxWidth(),
                        thickness = 0.5.dp,
                        color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.4f),
                    )

                    Row(
                        modifier =
                            Modifier
                                .fillMaxWidth()
                                .height(IntrinsicSize.Min),
                    ) {
                        if (dismissButtonContent != null) {
                            Box(
                                modifier =
                                    Modifier
                                        .weight(1f)
                                        .fillMaxHeight(),
                                contentAlignment = Alignment.Center,
                            ) {
                                dismissButtonContent()
                            }
                        }

                        if (dismissButtonContent != null && confirmButtonContent != null) {
                            VerticalDivider(
                                modifier = Modifier.fillMaxHeight(),
                                thickness = 0.5.dp,
                                color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.4f),
                            )
                        }

                        if (confirmButtonContent != null) {
                            Box(
                                modifier =
                                    Modifier
                                        .weight(1f)
                                        .fillMaxHeight(),
                                contentAlignment = Alignment.Center,
                            ) {
                                confirmButtonContent()
                            }
                        }
                    }
                }
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun DialogButtonPreview() {
    AcerolaTheme {
        Acerola.Component.DialogButton(
            text = stringResource(R.string.action_confirm),
            onClick = {},
        )
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun DialogPreview() {
    AcerolaTheme {
        Acerola.Component.Dialog(
            show = true,
            onDismiss = {},
            title = stringResource(R.string.dialog_delete_title),
            confirmButtonContent = {
                Acerola.Component.DialogButton(
                    text = stringResource(R.string.action_delete),
                    onClick = {},
                    contentColor = MaterialTheme.colorScheme.error,
                )
            },
            dismissButtonContent = {
                Acerola.Component.DialogButton(
                    text = stringResource(R.string.action_cancel),
                    onClick = {},
                )
            },
        ) {
            Text(
                text = stringResource(R.string.dialog_delete_message),
                style = MaterialTheme.typography.bodyMedium,
            )
        }
    }
}
