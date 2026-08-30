package br.acerola.comic.config.preference

import android.content.Context
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import br.acerola.comic.config.preference.types.VolumeViewType
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

object VolumeViewPreference {
    private val Context.dataStore by preferencesDataStore(name = "volume_view_prefs")

    private val VOLUME_VIEW_KEY = stringPreferencesKey(name = "volume_view_type")

    suspend fun saveVolumeView(
        context: Context,
        mode: VolumeViewType,
    ) {
        context.dataStore.edit { prefs ->
            prefs[VOLUME_VIEW_KEY] = mode.key
        }
    }

    fun volumeViewFlow(context: Context): Flow<VolumeViewType> =
        context.dataStore.data.map { prefs ->
            val saved = VolumeViewType.fromKey(prefs[VOLUME_VIEW_KEY])
            if (saved == VolumeViewType.CHAPTER) VolumeViewType.VOLUME else saved
        }
}

object MetadataPreference {
    private val Context.dataStore by preferencesDataStore(name = "metadata_prefs")
    private val GENERATE_COMIC_INFO_KEY = booleanPreferencesKey(name = "generate_comic_info_by_default")
    private val METADATA_LANGUAGE_KEY = stringPreferencesKey(name = "metadata_language")

    suspend fun saveGenerateComicInfo(
        context: Context,
        generate: Boolean,
    ) {
        context.dataStore.edit { prefs ->
            prefs[GENERATE_COMIC_INFO_KEY] = generate
        }
    }

    suspend fun saveMetadataLanguage(
        context: Context,
        language: String,
    ) {
        context.dataStore.edit { prefs ->
            prefs[METADATA_LANGUAGE_KEY] = language
        }
    }

    fun generateComicInfoFlow(context: Context): Flow<Boolean> =
        context.dataStore.data.map { prefs ->
            prefs[GENERATE_COMIC_INFO_KEY] ?: true
        }

    fun metadataLanguageFlow(context: Context): Flow<String?> =
        context.dataStore.data.map { prefs ->
            prefs[METADATA_LANGUAGE_KEY]
        }
}

object ComicDirectoryPreference {
    private val Context.dataStore by preferencesDataStore(name = "folder_prefs")
    private val FOLDER_URI = stringPreferencesKey(name = "folder_uri")
    private val TUTORIAL_SHOWN = booleanPreferencesKey(name = "tutorial_shown")

    suspend fun saveFolderUri(
        context: Context,
        uri: String,
    ) {
        context.dataStore.edit { prefs ->
            prefs[FOLDER_URI] = uri
        }
    }

    suspend fun clearFolderUri(context: Context) {
        context.dataStore.edit { prefs ->
            prefs.remove(key = FOLDER_URI)
        }
    }

    fun folderUriFlow(context: Context): Flow<String?> = context.dataStore.data.map { prefs -> prefs[FOLDER_URI] }

    suspend fun setTutorialShown(
        context: Context,
        shown: Boolean,
    ) {
        context.dataStore.edit { prefs ->
            prefs[TUTORIAL_SHOWN] = shown
        }
    }

    fun tutorialShownFlow(context: Context): Flow<Boolean> = context.dataStore.data.map { prefs -> prefs[TUTORIAL_SHOWN] ?: false }
}

object DeviceAliasPreference {
    private val Context.dataStore by preferencesDataStore(name = "device_alias_prefs")
    private val DEVICE_ALIAS = stringPreferencesKey(name = "device_alias")

    suspend fun saveAlias(
        context: Context,
        name: String,
    ) {
        context.dataStore.edit { prefs ->
            prefs[DEVICE_ALIAS] = name
        }
    }

    /** `null` quando o usuário nunca definiu um apelido — quem chama deve cair pro nome
     *  automático (`Build.MODEL`). */
    fun deviceAliasFlow(context: Context): Flow<String?> = context.dataStore.data.map { prefs -> prefs[DEVICE_ALIAS] }
}

object RelayPreference {
    const val DEFAULT_RELAY_URL = "https://relay.acerola-comic.com/"

    private val Context.dataStore by preferencesDataStore(name = "relay_prefs")
    private val RELAY_URL_OVERRIDE = stringPreferencesKey(name = "relay_url_override")

    suspend fun saveOverride(
        context: Context,
        url: String,
    ) {
        context.dataStore.edit { prefs ->
            prefs[RELAY_URL_OVERRIDE] = url
        }
    }

    suspend fun clearOverride(context: Context) {
        context.dataStore.edit { prefs ->
            prefs.remove(key = RELAY_URL_OVERRIDE)
        }
    }

    fun relayUrlOverrideFlow(context: Context): Flow<String?> = context.dataStore.data.map { prefs -> prefs[RELAY_URL_OVERRIDE] }
}
