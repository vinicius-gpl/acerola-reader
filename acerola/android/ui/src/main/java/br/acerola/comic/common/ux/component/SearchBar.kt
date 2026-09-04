package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.LinearOutSlowInEasing
import androidx.compose.animation.core.animateDp
import androidx.compose.animation.core.tween
import androidx.compose.animation.core.updateTransition
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SearchBar
import androidx.compose.material3.SearchBarDefaults
import androidx.compose.material3.SearchBarDefaults.InputField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.ui.R

@Composable
@OptIn(ExperimentalMaterial3Api::class)
fun <T> Acerola.Component.SearchBar(
    query: String,
    onQueryChange: (String) -> Unit,
    onSearch: (String) -> Unit,
    expanded: Boolean,
    onExpandedChange: (Boolean) -> Unit,
    onBackClick: (() -> Unit)? = null,
    isLoading: Boolean = false,
    items: List<T>,
    placeholder: String,
    itemKey: (T) -> Any,
    modifier: Modifier = Modifier,
    contentPadding: PaddingValues = rememberSearchBarContentPadding(),
    itemContent: @Composable (T) -> Unit,
) {
    val internalBackClick = onBackClick ?: { onExpandedChange(false) }

    val animatedShape = rememberSearchBarShape(expanded)

    SearchBar(
        modifier = modifier.animateContentSize(),
        inputField = {
            InputField(
                query = query,
                onQueryChange = onQueryChange,
                onSearch = onSearch,
                expanded = expanded,
                onExpandedChange = onExpandedChange,
                placeholder = {
                    Text(
                        text = placeholder,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                },
                leadingIcon = {
                    if (expanded) {
                        IconButton(onClick = internalBackClick) {
                            Icon(
                                imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                                contentDescription = stringResource(R.string.common_back),
                                tint = MaterialTheme.colorScheme.onSurface,
                            )
                        }
                    } else {
                        Icon(
                            imageVector = Icons.Default.Search,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                },
                trailingIcon = {
                    if (expanded && query.isNotEmpty()) {
                        IconButton(onClick = { onQueryChange("") }) {
                            Icon(
                                imageVector = Icons.Default.Close,
                                contentDescription = stringResource(R.string.common_clear),
                                tint = MaterialTheme.colorScheme.onSurface,
                            )
                        }
                    }
                },
            )
        },
        expanded = expanded,
        onExpandedChange = onExpandedChange,
        shape = animatedShape,
        colors =
            SearchBarDefaults.colors(
                containerColor =
                    if (expanded) {
                        MaterialTheme.colorScheme.surfaceContainerHigh
                    } else {
                        MaterialTheme.colorScheme.surfaceContainer
                    },
            ),
        tonalElevation = if (expanded) 0.dp else 3.dp,
        shadowElevation = 0.dp,
        windowInsets =
            androidx.compose.foundation.layout
                .WindowInsets(0.dp),
    ) {
        AnimatedVisibility(
            visible = expanded,
            enter = fadeIn(tween(150)) + expandVertically(),
            exit = fadeOut(tween(220)) + shrinkVertically(),
        ) {
            Surface(
                shape = RoundedCornerShape(0.dp),
                modifier = Modifier.fillMaxSize(),
                color = MaterialTheme.colorScheme.surfaceContainerHigh,
            ) {
                Column {
                    if (isLoading) {
                        LinearProgressIndicator(
                            modifier = Modifier.fillMaxWidth(),
                            color = MaterialTheme.colorScheme.primary,
                            trackColor = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.2f),
                        )
                    } else {
                        HorizontalDivider(
                            color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.5f),
                        )
                    }

                    if (items.isEmpty() && !isLoading && query.isNotEmpty()) {
                        Box(
                            contentAlignment = Alignment.Center,
                            modifier =
                                Modifier
                                    .fillMaxWidth()
                                    .padding(32.dp),
                        ) {
                            Text(
                                text = stringResource(R.string.common_no_results),
                                style = MaterialTheme.typography.bodyLarge,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                                textAlign = TextAlign.Center,
                            )
                        }
                    } else {
                        LazyColumn(contentPadding = contentPadding) {
                            items(
                                items = items,
                                key = { item -> itemKey(item) },
                            ) { item ->
                                itemContent(item)
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun rememberSearchBarShape(expanded: Boolean): RoundedCornerShape {
    val transition =
        updateTransition(
            targetState = expanded,
            label = stringResource(R.string.common_search_transition),
        )

    val collapsedRadius = 28.dp

    val cornerRadius by transition.animateDp(
        transitionSpec = {
            if (targetState) {
                tween(
                    durationMillis = 200,
                    easing = FastOutSlowInEasing,
                )
            } else {
                tween(
                    durationMillis = 300,
                    easing = LinearOutSlowInEasing,
                )
            }
        },
        label = stringResource(R.string.common_search_corner_radius),
    ) { isExpanded ->
        if (isExpanded) 12.dp else collapsedRadius
    }

    return remember(cornerRadius) {
        RoundedCornerShape(cornerRadius)
    }
}

@Composable
fun rememberSearchBarContentPadding(additionalBottomPadding: Dp = 16.dp): PaddingValues {
    val configuration = LocalConfiguration.current
    val isLandscape = configuration.orientation == Configuration.ORIENTATION_LANDSCAPE
    val bottomInset = WindowInsets.navigationBars.asPaddingValues().calculateBottomPadding()
    val bottomBarHeight = if (isLandscape) 0.dp else 64.dp

    return PaddingValues(
        start = 0.dp,
        top = 0.dp,
        end = 0.dp,
        bottom = bottomBarHeight + bottomInset + additionalBottomPadding,
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SearchBarPreview() {
    AcerolaTheme {
        Acerola.Component.SearchBar<String>(
            query = "One Piece",
            onQueryChange = {},
            onSearch = {},
            expanded = true,
            onExpandedChange = {},
            items = listOf("One Piece - Vol. 1", "One Piece - Vol. 2", "One Piece - Film Red"),
            placeholder = stringResource(R.string.label_home_search_placeholder),
            itemKey = { it },
            itemContent = { title ->
                Text(
                    text = title,
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .padding(horizontal = 16.dp, vertical = 12.dp),
                    style = MaterialTheme.typography.bodyMedium,
                )
            },
        )
    }
}
