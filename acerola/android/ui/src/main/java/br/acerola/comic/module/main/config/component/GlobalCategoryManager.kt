package br.acerola.comic.module.main.config.component

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Add
import androidx.compose.material.icons.rounded.Bookmark
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material3.Icon
import androidx.compose.material3.InputChip
import androidx.compose.material3.InputChipDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.Dialog
import br.acerola.comic.common.ux.component.DialogButton
import br.acerola.comic.common.ux.component.HeroButton
import br.acerola.comic.dto.metadata.category.CategoryDto
import br.acerola.comic.module.main.Main
import br.acerola.comic.ui.R
import androidx.compose.ui.tooling.preview.Preview
import android.content.res.Configuration
import br.acerola.comic.common.ux.theme.AcerolaTheme

// Paleta de cores é só desta tela: `Category.color` (Room) é um Int sem validação —
// `CategoryManager.createCategory` persiste o que a UI mandar, sem checar contra esta lista.
val categoryColors =
    listOf(
        0xFFF44336,
        0xFFE91E63,
        0xFF9C27B0,
        0xFF673AB7,
        0xFF3F51B5,
        0xFF2196F3,
        0xFF03A9F4,
        0xFF00BCD4,
        0xFF009688,
        0xFF4CAF50,
        0xFF8BC34A,
        0xFFCDDC39,
        0xFFFFEB3B,
        0xFFFFC107,
        0xFFFF9800,
        0xFFFF5722,
        0xFF795548,
        0xFF9E9E9E,
        0xFF607D8B,
    )

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun Main.Config.Component.GlobalCategoryManager(
    categories: List<CategoryDto>,
    onCreateCategory: (String, Int) -> Unit,
    onDeleteCategory: (Long) -> Unit,
    modifier: Modifier = Modifier,
) {
    var showCreateDialog by remember { mutableStateOf(false) }

    Acerola.Component.HeroButton(
        title = stringResource(id = R.string.action_add_category),
        description =
            if (categories.isEmpty()) {
                stringResource(id = R.string.description_config_categories)
            } else {
                null
            },
        icon = Icons.Rounded.Add,
        iconTint = MaterialTheme.colorScheme.onPrimaryContainer,
        iconBackground = MaterialTheme.colorScheme.primaryContainer,
        onClick = { showCreateDialog = true },
        modifier = modifier,
        action = {
            if (categories.isNotEmpty()) {
                Text(
                    text = "${categories.size}",
                    style = MaterialTheme.typography.labelLarge,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.primary,
                )
            }
        },
        bottomContent =
            if (categories.isNotEmpty()) {
                {
                    FlowRow(
                        modifier =
                            Modifier
                                .fillMaxWidth()
                                .padding(horizontal = 16.dp, vertical = 12.dp),
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalArrangement = Arrangement.spacedBy(4.dp),
                    ) {
                        categories.forEach { category ->
                            InputChip(
                                onClick = { },
                                label = { Text(text = category.name) },
                                selected = true,
                                leadingIcon = {
                                    Icon(
                                        imageVector = Icons.Rounded.Bookmark,
                                        contentDescription = null,
                                        tint = Color(category.color),
                                        modifier = Modifier.size(16.dp),
                                    )
                                },
                                trailingIcon = {
                                    Icon(
                                        imageVector = Icons.Rounded.Close,
                                        contentDescription = stringResource(id = R.string.action_delete_category),
                                        modifier =
                                            Modifier
                                                .size(14.dp)
                                                .clickable { onDeleteCategory(category.id) },
                                    )
                                },
                                colors =
                                    InputChipDefaults.inputChipColors(
                                        selectedContainerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.6f),
                                        selectedLabelColor = MaterialTheme.colorScheme.onSurface,
                                    ),
                            )
                        }
                    }
                }
            } else {
                null
            },
    )

    if (showCreateDialog) {
        CreateCategoryDialog(
            onDismiss = { showCreateDialog = false },
            onConfirm = { name, color ->
                onCreateCategory(name, color)
                showCreateDialog = false
            },
        )
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun CreateCategoryDialog(
    onDismiss: () -> Unit,
    onConfirm: (String, Int) -> Unit,
) {
    var name by remember { mutableStateOf("") }
    var selectedColor by remember { mutableLongStateOf(categoryColors.first()) }

    Acerola.Component.Dialog(
        show = true,
        onDismiss = onDismiss,
        title = stringResource(id = R.string.action_add_category),
        confirmButtonContent = {
            Acerola.Component.DialogButton(
                text = stringResource(id = android.R.string.ok),
                onClick = { if (name.isNotBlank()) onConfirm(name, selectedColor.toInt()) },
                containerColor = MaterialTheme.colorScheme.primary,
                contentColor = MaterialTheme.colorScheme.onPrimary,
                fontWeight = FontWeight.Bold,
            )
        },
        dismissButtonContent = {
            Acerola.Component.DialogButton(
                text = stringResource(id = android.R.string.cancel),
                onClick = onDismiss,
                contentColor = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        },
        content = {
            Column(
                modifier = Modifier.fillMaxWidth(),
                verticalArrangement = Arrangement.spacedBy(16.dp),
            ) {
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text(stringResource(id = R.string.label_category_name)) },
                    placeholder = { Text(stringResource(id = R.string.placeholder_category_name)) },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                )

                Text(
                    text = stringResource(id = R.string.label_category_color),
                    style = MaterialTheme.typography.labelMedium,
                )

                FlowRow(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    categoryColors.forEach { color ->
                        Box(
                            modifier =
                                Modifier
                                    .size(32.dp)
                                    .clip(CircleShape)
                                    .background(Color(color))
                                    .border(
                                        width = if (selectedColor == color) 2.dp else 0.dp,
                                        color = if (selectedColor == color) MaterialTheme.colorScheme.primary else Color.Transparent,
                                        shape = CircleShape,
                                    ).clickable { selectedColor = color },
                        )
                    }
                }
            }
        },
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun GlobalCategoryManagerPreview() {
    AcerolaTheme {
        Main.Config.Component.GlobalCategoryManager(
            categories = emptyList(),
            onCreateCategory = { _, _ -> },
            onDeleteCategory = {},
        )
    }
}
