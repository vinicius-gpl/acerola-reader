package br.acerola.comic.common.ux.component
import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.ShapeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.ui.R

private val selectionActionButtonHeight = 54.dp

data class SelectionAction(
    val icon: ImageVector,
    val label: String,
    val onClick: () -> Unit,
    val isError: Boolean = false,
)

@Composable
fun Acerola.Component.SelectionTopBar(
    selectedCount: Int,
    isAllSelected: Boolean,
    onClear: () -> Unit,
    onToggleSelectAll: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(
        shape = ShapeTokens.Large,
        color = MaterialTheme.colorScheme.surfaceContainerHigh,
        tonalElevation = 6.dp,
        shadowElevation = 8.dp,
        modifier = modifier.fillMaxWidth(),
    ) {
        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .height(56.dp)
                    .padding(horizontal = SpacingTokens.Small),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                IconButton(onClick = onClear) {
                    Icon(
                        imageVector = Icons.Default.Close,
                        contentDescription = stringResource(id = R.string.action_cancel),
                    )
                }
                Spacer(modifier = Modifier.width(SpacingTokens.Small))
                Text(
                    text = stringResource(id = R.string.label_selection_count, selectedCount),
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.onSurface,
                )
            }

            TextButton(onClick = onToggleSelectAll) {
                Text(
                    text =
                        stringResource(
                            id = if (isAllSelected) R.string.action_deselect_all else R.string.action_select_all,
                        ),
                    fontWeight = FontWeight.SemiBold,
                )
            }
        }
    }
}

@Composable
fun Acerola.Component.SelectionActionDock(
    actions: List<SelectionAction>,
    modifier: Modifier = Modifier,
) {
    Surface(
        shape = ShapeTokens.Large,
        color = MaterialTheme.colorScheme.surfaceContainerHighest,
        tonalElevation = 8.dp,
        shadowElevation = 12.dp,
        modifier = modifier,
    ) {
        Row(
            modifier = Modifier.padding(all = SpacingTokens.Small),
            horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            actions.forEach { action ->
                SelectionActionButton(
                    icon = action.icon,
                    label = action.label,
                    onClick = action.onClick,
                    isError = action.isError,
                    modifier = Modifier.weight(1f),
                )
            }
        }
    }
}

@Composable
private fun SelectionActionButton(
    icon: ImageVector,
    label: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    isError: Boolean = false,
) {
    val containerColor =
        if (isError) {
            MaterialTheme.colorScheme.errorContainer.copy(alpha = 0.5f)
        } else {
            MaterialTheme.colorScheme.surfaceContainerHigh
        }
    val contentColor =
        if (isError) {
            MaterialTheme.colorScheme.error
        } else {
            MaterialTheme.colorScheme.onSurface
        }

    Surface(
        onClick = onClick,
        shape = ShapeTokens.Medium,
        color = containerColor,
        contentColor = contentColor,
        modifier = modifier.height(selectionActionButtonHeight),
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center,
            modifier = Modifier.padding(vertical = 4.dp, horizontal = 4.dp),
        ) {
            Icon(
                imageVector = icon,
                contentDescription = null,
                modifier = Modifier.size(20.dp),
            )
            Spacer(modifier = Modifier.height(2.dp))
            Text(
                text = label,
                style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Bold),
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SelectionTopBarPreview() {
    AcerolaTheme {
        Acerola.Component.SelectionTopBar(
            selectedCount = 3,
            isAllSelected = false,
            onClear = {},
            onToggleSelectAll = {},
        )
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SelectionActionDockPreview() {
    AcerolaTheme {
        Acerola.Component.SelectionActionDock(
            actions =
                listOf(
                    SelectionAction(Icons.Default.Close, "Action", {}),
                    SelectionAction(Icons.Default.Close, "Action", {}, isError = true),
                ),
        )
    }
}
