package br.acerola.comic.module.main.config.state
import android.net.Uri

data class ConfigUiState(
    val folderUri: Uri? = null,
    val folderName: String? = null,
    val generateComicInfo: Boolean = true,
    val metadataLanguage: String? = null,
)
