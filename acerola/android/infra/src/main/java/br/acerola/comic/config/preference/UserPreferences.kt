package br.acerola.comic.config.preference

import android.content.Context
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import br.acerola.comic.config.preference.types.ChapterPageSizeType
import br.acerola.comic.config.preference.types.ChapterSortPreferenceData
import br.acerola.comic.config.preference.types.ChapterSortType
import br.acerola.comic.config.preference.types.ComicSortType
import br.acerola.comic.config.preference.types.HomeLayoutType
import br.acerola.comic.config.preference.types.HomeSortPreference
import br.acerola.comic.config.preference.types.ReadingMode
import br.acerola.comic.config.preference.types.SortDirection
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

object ReadingModePreference {
    private val Context.dataStore by preferencesDataStore(name = "reading_mode_prefs")
    private val READING_MODE_KEY = stringPreferencesKey(name = "reading_mode_type")

    suspend fun saveReadingMode(
        context: Context,
        mode: ReadingMode,
    ) {
        context.dataStore.edit { prefs ->
            prefs[READING_MODE_KEY] = mode.key
        }
    }

    fun readingModeFlow(context: Context): Flow<ReadingMode> =
        context.dataStore.data.map { prefs ->
            ReadingMode.Companion.fromKey(key = prefs[READING_MODE_KEY])
        }
}

object HomeLayoutPreference {
    private val Context.dataStore by preferencesDataStore(name = "home_layout_prefs")

    private val HOME_LAYOUT_KEY = stringPreferencesKey(name = "home_layout_type")

    suspend fun saveLayout(
        context: Context,
        layout: HomeLayoutType,
    ) {
        context.dataStore.edit { prefs ->
            prefs[HOME_LAYOUT_KEY] = layout.key
        }
    }

    fun layoutFlow(context: Context): Flow<HomeLayoutType> =
        context.dataStore.data.map { prefs ->
            HomeLayoutType.Companion.fromKey(key = prefs[HOME_LAYOUT_KEY])
        }
}

object ComicSortPreference {
    private val Context.dataStore by preferencesDataStore(name = "comic_sort_prefs")

    private val MANGA_SORT_TYPE_KEY = stringPreferencesKey(name = "comic_sort_type")
    private val MANGA_SORT_DIRECTION_KEY = stringPreferencesKey(name = "comic_sort_direction")

    suspend fun saveSort(
        context: Context,
        sort: HomeSortPreference,
    ) {
        context.dataStore.edit { prefs ->
            prefs[MANGA_SORT_TYPE_KEY] = sort.type.key
            prefs[MANGA_SORT_DIRECTION_KEY] = sort.direction.key
        }
    }

    fun sortFlow(context: Context): Flow<HomeSortPreference> =
        context.dataStore.data.map { prefs ->
            HomeSortPreference(
                type = ComicSortType.Companion.fromKey(key = prefs[MANGA_SORT_TYPE_KEY]),
                direction = SortDirection.Companion.fromKey(key = prefs[MANGA_SORT_DIRECTION_KEY]),
            )
        }
}

object HomeFilterPreference {
    private val Context.dataStore by preferencesDataStore(name = "home_filter_prefs")

    private val SHOW_HIDDEN_KEY = booleanPreferencesKey(name = "show_hidden")

    suspend fun saveShowHidden(
        context: Context,
        showHidden: Boolean,
    ) {
        context.dataStore.edit { prefs ->
            prefs[SHOW_HIDDEN_KEY] = showHidden
        }
    }

    fun showHiddenFlow(context: Context): Flow<Boolean> =
        context.dataStore.data.map { prefs ->
            prefs[SHOW_HIDDEN_KEY] ?: false
        }
}

object ChapterPerPagePreference {
    private val Context.dataStore by preferencesDataStore(name = "chapter_per_page_prefs")

    private val CHAPTER_PER_PAGE_KEY = stringPreferencesKey(name = "chapter_per_page_type")

    suspend fun saveChapterPerPage(
        context: Context,
        size: ChapterPageSizeType,
    ) {
        context.dataStore.edit { prefs ->
            prefs[CHAPTER_PER_PAGE_KEY] = size.key
        }
    }

    fun chapterPerPageFlow(context: Context): Flow<ChapterPageSizeType> =
        context.dataStore.data.map { prefs ->
            ChapterPageSizeType.Companion.fromKey(key = prefs[CHAPTER_PER_PAGE_KEY])
        }
}

object OnboardingPreference {
    private val Context.dataStore by preferencesDataStore(name = "onboarding_prefs")
    private val ONBOARDING_COMPLETED_KEY = booleanPreferencesKey(name = "onboarding_completed")

    suspend fun markCompleted(context: Context) {
        context.dataStore.edit { prefs ->
            prefs[ONBOARDING_COMPLETED_KEY] = true
        }
    }

    fun isCompletedFlow(context: Context): Flow<Boolean> =
        context.dataStore.data.map { prefs ->
            prefs[ONBOARDING_COMPLETED_KEY] ?: false
        }
}

object ChapterSortPreference {
    private val Context.dataStore by preferencesDataStore(name = "chapter_sort_prefs")

    private val CHAPTER_SORT_TYPE_KEY = stringPreferencesKey(name = "chapter_sort_type")
    private val CHAPTER_SORT_DIRECTION_KEY = stringPreferencesKey(name = "chapter_sort_direction")

    suspend fun saveSort(
        context: Context,
        sort: ChapterSortPreferenceData,
    ) {
        context.dataStore.edit { prefs ->
            prefs[CHAPTER_SORT_TYPE_KEY] = sort.type.key
            prefs[CHAPTER_SORT_DIRECTION_KEY] = sort.direction.key
        }
    }

    fun sortFlow(context: Context): Flow<ChapterSortPreferenceData> =
        context.dataStore.data.map { prefs ->
            ChapterSortPreferenceData(
                type = ChapterSortType.fromKey(key = prefs[CHAPTER_SORT_TYPE_KEY]),
                direction = SortDirection.fromKey(key = prefs[CHAPTER_SORT_DIRECTION_KEY]),
            )
        }
}
