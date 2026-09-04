package br.acerola.comic.common.ux.component
import android.content.res.Configuration
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Snackbar
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.SnackbarVisuals
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaTheme

enum class SnackbarVariant { Error, Success, Warn }

private data class AcerolaSnackbarVisuals(
    val variant: SnackbarVariant,
    override val message: String,
    override val actionLabel: String? = null,
    override val withDismissAction: Boolean = false,
    override val duration: SnackbarDuration = SnackbarDuration.Short,
) : SnackbarVisuals

internal fun resolveSnackbarVariant(visuals: SnackbarVisuals): SnackbarVariant =
    (visuals as? AcerolaSnackbarVisuals)?.variant ?: SnackbarVariant.Error

suspend fun SnackbarHostState.showSnackbar(
    message: String,
    variant: SnackbarVariant,
) {
    showSnackbar(
        AcerolaSnackbarVisuals(
            message = message,
            variant = variant,
        ),
    )
}

@Composable
fun Acerola.Component.SnackbarError(
    message: String,
    modifier: Modifier = Modifier,
) {
    Snackbar(
        modifier = modifier.padding(horizontal = 16.dp, vertical = 8.dp),
        containerColor = MaterialTheme.colorScheme.errorContainer,
        contentColor = MaterialTheme.colorScheme.onErrorContainer,
        content = { Text(text = message) },
    )
}

@Composable
fun Acerola.Component.SnackbarSuccess(
    message: String,
    modifier: Modifier = Modifier,
) {
    Snackbar(
        modifier = modifier.padding(horizontal = 16.dp, vertical = 8.dp),
        containerColor = MaterialTheme.colorScheme.secondaryContainer,
        contentColor = MaterialTheme.colorScheme.onSecondaryContainer,
        content = { Text(text = message) },
    )
}

@Composable
fun Acerola.Component.SnackbarWarn(
    message: String,
    modifier: Modifier = Modifier,
) {
    Snackbar(
        modifier = modifier.padding(horizontal = 16.dp, vertical = 8.dp),
        containerColor = MaterialTheme.colorScheme.tertiaryContainer,
        contentColor = MaterialTheme.colorScheme.onTertiaryContainer,
        content = { Text(text = message) },
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SnackbarErrorPreview() {
    AcerolaTheme {
        Acerola.Component.SnackbarError(message = "Error message")
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SnackbarSuccessPreview() {
    AcerolaTheme {
        Acerola.Component.SnackbarSuccess(message = "Success message")
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SnackbarWarnPreview() {
    AcerolaTheme {
        Acerola.Component.SnackbarWarn(message = "Warning message")
    }
}
