package br.acerola.comic.common.navigation

import androidx.annotation.StringRes
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.MenuBook
import androidx.compose.material.icons.filled.Book
import androidx.compose.material.icons.filled.History
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.filled.Sync
import androidx.compose.ui.graphics.vector.ImageVector
import br.acerola.comic.ui.R

enum class Destination(
    val icon: ImageVector,
    @param:StringRes val label: Int,
    @param:StringRes val route: Int,
    @param:StringRes val contentDescriptionRes: Int,
) {
    COMIC(
        icon = Icons.AutoMirrored.Filled.MenuBook,
        label = R.string.label_chapters_activity,
        route = R.string.navigation_chapters_activity,
        contentDescriptionRes = R.string.description_chapters_activity,
    ),
    READER(
        icon = Icons.Default.Book,
        label = R.string.label_reader_activity,
        route = R.string.navigation_reader_activity,
        contentDescriptionRes = R.string.description_reader_activity,
    ),
    HOME(
        icon = Icons.Default.Home,
        label = R.string.label_home_activity,
        route = R.string.navigation_home_activity,
        contentDescriptionRes = R.string.description_home_activity,
    ),
    HISTORY(
        icon = Icons.Default.History,
        label = R.string.label_history_activity,
        route = R.string.navigation_history_activity,
        contentDescriptionRes = R.string.description_history_activity,
    ),
    CONFIG(
        icon = Icons.Default.Settings,
        label = R.string.label_config_activity,
        route = R.string.navigation_config_activity,
        contentDescriptionRes = R.string.description_config_activity,
    ),
    PATTERN(
        icon = Icons.Default.Settings,
        label = R.string.label_template_config_activity,
        route = R.string.navigation_template_config_activity,
        contentDescriptionRes = R.string.description_template_config_activity,
    ),
    TUTORIAL(
        icon = Icons.Default.Book,
        label = R.string.label_tutorial_activity,
        route = R.string.navigation_tutorial_activity,
        contentDescriptionRes = R.string.description_tutorial_activity,
    ),
    SYNC(
        icon = Icons.Default.Sync,
        label = R.string.label_sync_activity,
        route = R.string.navigation_sync_activity,
        contentDescriptionRes = R.string.description_sync_activity,
    ),
    REMOTE_LIBRARY(
        icon = Icons.Default.Sync,
        label = R.string.label_remote_library_activity,
        route = R.string.navigation_remote_library_activity,
        contentDescriptionRes = R.string.description_remote_library_activity,
    ),
}
