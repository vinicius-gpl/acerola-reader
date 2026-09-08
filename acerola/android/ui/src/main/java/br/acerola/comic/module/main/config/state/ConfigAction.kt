package br.acerola.comic.module.main.config.state
import android.net.Uri
import br.acerola.comic.config.preference.types.AppTheme

sealed interface ConfigAction {
    data class UpdateTheme(
        val theme: AppTheme,
    ) : ConfigAction

    data class SelectFolder(
        val uri: Uri?,
    ) : ConfigAction

    data class UpdateGenerateComicInfo(
        val enabled: Boolean,
    ) : ConfigAction

    data class UpdateMetadataLanguage(
        val language: String,
    ) : ConfigAction

    data object DeepScanLibrary : ConfigAction

    data object QuickSyncLibrary : ConfigAction

    data object SyncMangadexMetadata : ConfigAction

    data object SyncAnilistMetadata : ConfigAction

    data class CreateCategory(
        val name: String,
        val color: Int,
    ) : ConfigAction

    data class DeleteCategory(
        val id: Long,
    ) : ConfigAction

    data object NavigateToTemplateConfig : ConfigAction
}
