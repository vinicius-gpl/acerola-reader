package br.acerola.comic.module.comic.component

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Bookmark
import androidx.compose.material.icons.rounded.BookmarkBorder
import androidx.compose.material3.FilterChip
import androidx.compose.material3.FilterChipDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.HeroButton
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.dto.metadata.category.CategoryDto
import br.acerola.comic.module.comic.Comic
import br.acerola.comic.ui.R

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun Comic.Component.ComicCategorySelector(
    selectedCategory: CategoryDto?,
    allCategories: List<CategoryDto>,
    onUpdateMangaCategory: (Long?) -> Unit,
    modifier: Modifier = Modifier,
) {
    Acerola.Component.HeroButton(
        title = stringResource(id = R.string.title_comic_category),
        description = selectedCategory?.name ?: stringResource(id = R.string.label_category_none_selected),
        icon = if (selectedCategory != null) Icons.Rounded.Bookmark else Icons.Rounded.BookmarkBorder,
        iconTint = if (selectedCategory != null) Color(selectedCategory.color) else MaterialTheme.colorScheme.onPrimaryContainer,
        iconBackground = MaterialTheme.colorScheme.primaryContainer,
        modifier = modifier,
        bottomContent =
            if (allCategories.isNotEmpty()) {
                {
                    FlowRow(
                        modifier =
                            Modifier
                                .fillMaxWidth()
                                .padding(horizontal = 16.dp, vertical = 12.dp),
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalArrangement = Arrangement.spacedBy(4.dp),
                    ) {
                        allCategories.forEach { category ->
                            val isSelected = selectedCategory?.id == category.id
                            FilterChip(
                                selected = isSelected,
                                onClick = {
                                    if (isSelected) {
                                        onUpdateMangaCategory(null)
                                    } else {
                                        onUpdateMangaCategory(category.id)
                                    }
                                },
                                label = { Text(text = category.name) },
                                leadingIcon = {
                                    Icon(
                                        imageVector = Icons.Rounded.Bookmark,
                                        contentDescription = null,
                                        tint = Color(category.color),
                                        modifier = Modifier.size(16.dp),
                                    )
                                },
                                colors =
                                    FilterChipDefaults.filterChipColors(
                                        selectedContainerColor = MaterialTheme.colorScheme.primaryContainer,
                                        selectedLabelColor = MaterialTheme.colorScheme.onPrimaryContainer,
                                        selectedLeadingIconColor = Color(category.color),
                                    ),
                            )
                        }
                    }
                }
            } else {
                null
            },
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ComicCategorySelectorPreview() {
    AcerolaTheme {
        Comic.Component.ComicCategorySelector(
            selectedCategory = null,
            allCategories = emptyList(),
            onUpdateMangaCategory = {},
        )
    }
}
