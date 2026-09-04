package br.acerola.comic.config.preference

import android.content.Context
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.core.stringSetPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import br.acerola.comic.config.preference.types.VolumeViewType
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.first
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

/**
 * Configuração de relay combinável, espelhando `RelaySettings`/`RelaySettings::resolve` do
 * Desktop (`bios/scopes.rs`) — relay do Acerola, rede pública Iroh e listas de relays próprios/
 * Iroh podem ser combinados entre si (exceto a rede pública Iroh, exclusiva com as demais).
 * Mudar qualquer fonte só tem efeito no próximo início do app: `P2pService`/`NetworkCaseModule`
 * só lê isso na construção do `P2PNode`, a lib não suporta trocar relay em runtime.
 */
object RelayPreference {
    const val DEFAULT_ACEROLA_RELAY_URL = "https://relay.acerola-comic.com"

    private val Context.dataStore by preferencesDataStore(name = "relay_prefs")
    private val USE_ACEROLA_RELAY = booleanPreferencesKey(name = "use_acerola_relay")
    private val USE_IROH_PUBLIC_NETWORK = booleanPreferencesKey(name = "use_iroh_public_network")
    private val CUSTOM_RELAY_URLS = stringSetPreferencesKey(name = "custom_relay_urls")
    private val IROH_RELAY_URLS = stringSetPreferencesKey(name = "iroh_relay_urls")

    // Chave legada (de antes desta feature, URL única) — só lida quando nenhuma das chaves
    // novas acima existe ainda, pra migrar pra dentro de `customRelayUrls`. Sem isso, quem já
    // tinha configurado um relay próprio perderia essa escolha silenciosamente no primeiro boot
    // após o update (mesmo cuidado do Desktop com `relay_url` em `settings.json`).
    private val LEGACY_RELAY_URL_OVERRIDE = stringPreferencesKey(name = "relay_url_override")

    data class RelaySettings(
        val useAcerolaRelay: Boolean = true,
        val useIrohPublicNetwork: Boolean = false,
        val customRelayUrls: List<String> = emptyList(),
        val irohRelayUrls: List<String> = emptyList(),
    )

    suspend fun setUseAcerolaRelay(
        context: Context,
        value: Boolean,
    ) {
        context.dataStore.edit { prefs -> prefs[USE_ACEROLA_RELAY] = value }
    }

    suspend fun setUseIrohPublicNetwork(
        context: Context,
        value: Boolean,
    ) {
        context.dataStore.edit { prefs -> prefs[USE_IROH_PUBLIC_NETWORK] = value }
    }

    suspend fun addCustomRelayUrl(
        context: Context,
        url: String,
    ) {
        context.dataStore.edit { prefs -> prefs[CUSTOM_RELAY_URLS] = (prefs[CUSTOM_RELAY_URLS] ?: emptySet()) + url }
    }

    suspend fun removeCustomRelayUrl(
        context: Context,
        url: String,
    ) {
        context.dataStore.edit { prefs -> prefs[CUSTOM_RELAY_URLS] = (prefs[CUSTOM_RELAY_URLS] ?: emptySet()) - url }
    }

    suspend fun addIrohRelayUrl(
        context: Context,
        url: String,
    ) {
        context.dataStore.edit { prefs -> prefs[IROH_RELAY_URLS] = (prefs[IROH_RELAY_URLS] ?: emptySet()) + url }
    }

    suspend fun removeIrohRelayUrl(
        context: Context,
        url: String,
    ) {
        context.dataStore.edit { prefs -> prefs[IROH_RELAY_URLS] = (prefs[IROH_RELAY_URLS] ?: emptySet()) - url }
    }

    fun relaySettingsFlow(context: Context): Flow<RelaySettings> = context.dataStore.data.map(::toRelaySettings)

    /** Leitura pontual (sem coletar o `Flow`) — usada na inicialização síncrona do `P2PNode`
     *  (ver `NetworkCaseModule`), mesmo padrão do antigo `relayUrlOverrideFlow(...).first()`. */
    suspend fun currentRelaySettings(context: Context): RelaySettings = toRelaySettings(context.dataStore.data.first())

    private fun toRelaySettings(prefs: Preferences): RelaySettings {
        val hasAnyNewKey =
            prefs.contains(USE_ACEROLA_RELAY) ||
                prefs.contains(USE_IROH_PUBLIC_NETWORK) ||
                prefs.contains(CUSTOM_RELAY_URLS) ||
                prefs.contains(IROH_RELAY_URLS)

        if (!hasAnyNewKey) {
            val legacyUrl = prefs[LEGACY_RELAY_URL_OVERRIDE]?.trim()
            if (!legacyUrl.isNullOrEmpty()) {
                return RelaySettings(customRelayUrls = listOf(legacyUrl))
            }
            return RelaySettings()
        }

        return RelaySettings(
            useAcerolaRelay = prefs[USE_ACEROLA_RELAY] ?: true,
            useIrohPublicNetwork = prefs[USE_IROH_PUBLIC_NETWORK] ?: false,
            customRelayUrls = (prefs[CUSTOM_RELAY_URLS] ?: emptySet()).toList(),
            irohRelayUrls = (prefs[IROH_RELAY_URLS] ?: emptySet()).toList(),
        )
    }
}
