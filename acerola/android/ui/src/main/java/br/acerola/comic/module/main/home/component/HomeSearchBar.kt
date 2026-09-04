package br.acerola.comic.module.main.home.component

import android.content.res.Configuration
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.SearchBar
import br.acerola.comic.common.ux.component.rememberSearchBarContentPadding
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.dto.ComicDto
import br.acerola.comic.dto.history.ReadingHistoryDto
import br.acerola.comic.module.main.Main
import br.acerola.comic.ui.R

@Composable
fun Main.Home.Component.HomeSearchBar(
    query: String,
    onQueryChange: (String) -> Unit,
    onSearch: (String) -> Unit,
    expanded: Boolean,
    onExpandedChange: (Boolean) -> Unit,
    comics: List<Triple<ComicDto, ReadingHistoryDto?, Int>>,
    onComicClick: (ComicDto) -> Unit,
    modifier: Modifier = Modifier,
    contentPadding: PaddingValues = rememberSearchBarContentPadding(),
) {
    Acerola.Component.SearchBar(
        query = query,
        onQueryChange = onQueryChange,
        onSearch = onSearch,
        expanded = expanded,
        onExpandedChange = onExpandedChange,
        items = comics,
        placeholder = stringResource(R.string.label_home_search_placeholder),
        itemKey = { it.first.directory.id },
        modifier = modifier,
        contentPadding = contentPadding,
        itemContent = { (comic, history, chapterCount) ->
            Main.Home.Component.ComicSearchItem(
                comic = comic,
                chapterCount = chapterCount,
                onClick = { onComicClick(comic) },
            )
        },
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun HomeSearchBarPreview() {
    AcerolaTheme {
        Main.Home.Component.HomeSearchBar(
            query = "",
            onQueryChange = {},
            onSearch = {},
            expanded = false,
            onExpandedChange = {},
            comics = emptyList(),
            onComicClick = {},
        )
    }
}
