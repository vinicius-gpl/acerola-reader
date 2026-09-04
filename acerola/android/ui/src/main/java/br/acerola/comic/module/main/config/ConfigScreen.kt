package br.acerola.comic.module.main.config
import androidx.compose.ui.tooling.preview.Preview
import android.content.res.Configuration
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.config.preference.types.AppTheme

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Folder
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.Palette
import androidx.compose.material.icons.filled.Public
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Sync
import androidx.compose.material.icons.rounded.Bookmark
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import br.acerola.comic.common.state.LocalSnackbarHostState
import br.acerola.comic.common.state.SyncActionVisualState
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.AccordionCard
import br.acerola.comic.common.ux.component.HeroButton
import br.acerola.comic.common.ux.component.SnackbarVariant
import br.acerola.comic.common.ux.component.showSnackbar
import br.acerola.comic.common.ux.tokens.ShapeTokens
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.common.viewmodel.archive.FileSystemAccessViewModel
import br.acerola.comic.common.viewmodel.library.archive.ComicDirectoryViewModel
import br.acerola.comic.common.viewmodel.library.metadata.ComicMetadataViewModel
import br.acerola.comic.common.viewmodel.metadata.MetadataSettingsViewModel
import br.acerola.comic.common.viewmodel.theme.ThemeViewModel
import br.acerola.comic.module.main.Main
import br.acerola.comic.module.main.config.component.GlobalCategoryManager
import br.acerola.comic.module.main.config.component.LanguageSettings
import br.acerola.comic.module.main.config.component.MetadataExportSettings
import br.acerola.comic.module.main.config.component.SelectComicDirectory
import br.acerola.comic.module.main.config.component.SyncAnilistData
import br.acerola.comic.module.main.config.component.SyncLibraryArchive
import br.acerola.comic.module.main.config.component.SyncMangadexData
import br.acerola.comic.module.main.config.component.TemplateManager
import br.acerola.comic.module.main.config.component.ThemeSettings
import br.acerola.comic.module.main.config.state.ConfigAction
import br.acerola.comic.module.main.config.state.ConfigUiState
import br.acerola.comic.ui.R
import br.acerola.comic.worker.sync.LibrarySyncWorker
import br.acerola.comic.worker.sync.MetadataSyncWorker
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlin.time.Duration.Companion.milliseconds

@Composable
fun Main.Config.Template.Screen(
    metadataSettingsViewModel: MetadataSettingsViewModel = hiltViewModel(),
    fileSystemAccessViewModel: FileSystemAccessViewModel = hiltViewModel(),
    comicDirectoryViewModel: ComicDirectoryViewModel = hiltViewModel(),
    comicDexViewModel: ComicMetadataViewModel = hiltViewModel(),
    themeViewModel: ThemeViewModel = hiltViewModel(),
    onNavigateToTemplates: () -> Unit,
    onNavigateToSync: () -> Unit,
) {
    val context = LocalContext.current
    val snackbarHostState = LocalSnackbarHostState.current
    val scrollState = rememberScrollState()

    LaunchedEffect(Unit) {
        launch {
            fileSystemAccessViewModel.uiEvents.collect { message ->
                snackbarHostState.showSnackbar(message.uiMessage.asString(context), SnackbarVariant.Error)
            }
        }
        launch {
            comicDirectoryViewModel.uiEvents.collect { message ->
                snackbarHostState.showSnackbar(message.uiMessage.asString(context), SnackbarVariant.Error)
            }
        }
        launch {
            comicDexViewModel.uiEvents.collect { message ->
                snackbarHostState.showSnackbar(message.uiMessage.asString(context), SnackbarVariant.Error)
            }
        }
        launch {
            metadataSettingsViewModel.uiEvents.collect { message ->
                snackbarHostState.showSnackbar(message.uiMessage.asString(context), SnackbarVariant.Error)
            }
        }
        launch {
            themeViewModel.uiEvents.collect { message ->
                snackbarHostState.showSnackbar(message.uiMessage.asString(context), SnackbarVariant.Error)
            }
        }
    }

    val selectedTheme by themeViewModel.currentTheme.collectAsState()
    val generateComicInfo by metadataSettingsViewModel.generateComicInfo.collectAsState()
    val metadataLanguage by metadataSettingsViewModel.metadataLanguage.collectAsState()
    val allCategories by comicDexViewModel.allCategories.collectAsState()
    val folderName by fileSystemAccessViewModel.folderName.collectAsState()
    val tutorialShown by fileSystemAccessViewModel.tutorialShown.collectAsState()
    val isLibraryIndexing by comicDirectoryViewModel.isIndexing.collectAsStateWithLifecycle(false)
    val isMetadataIndexing by comicDexViewModel.isIndexing.collectAsStateWithLifecycle(false)
    val activeLibrarySyncType by comicDirectoryViewModel.activeSyncType.collectAsStateWithLifecycle(null)
    val activeMetadataSource by comicDexViewModel.activeSyncSource.collectAsStateWithLifecycle(null)

    var activeSyncAction by remember { mutableStateOf<ConfigAction?>(null) }
    var successSyncAction by remember { mutableStateOf<ConfigAction?>(null) }
    var isCurrentlyIndexing by remember { mutableStateOf(false) }
    val isAnyIndexing = isLibraryIndexing || isMetadataIndexing

    // Categorias colapsam/expandem inline na própria lista, mesmo padrão da tela de config
    // do desktop (ver acerola-accordion-card.svelte) — mais de uma pode ficar aberta ao
    // mesmo tempo, por isso é um Set em vez de uma categoria única selecionada.
    var expandedCategories by remember { mutableStateOf(setOf<String>()) }

    fun toggleCategory(id: String) {
        expandedCategories = if (id in expandedCategories) expandedCategories - id else expandedCategories + id
    }

    LaunchedEffect(isAnyIndexing) {
        if (isAnyIndexing) {
            isCurrentlyIndexing = true
            return@LaunchedEffect
        }

        if (isCurrentlyIndexing && activeSyncAction != null) {
            val finishedAction = activeSyncAction
            activeSyncAction = null
            isCurrentlyIndexing = false
            successSyncAction = finishedAction

            delay(1800.milliseconds)

            if (successSyncAction == finishedAction) {
                successSyncAction = null
            }
        } else {
            isCurrentlyIndexing = false
            activeSyncAction = null
        }
    }

    fun getSyncActionVisualState(action: ConfigAction): SyncActionVisualState =
        when {
            activeSyncAction == action -> SyncActionVisualState.LOADING
            action == ConfigAction.QuickSyncLibrary && activeLibrarySyncType == LibrarySyncWorker.SYNC_TYPE_INCREMENTAL -> SyncActionVisualState.LOADING
            action == ConfigAction.DeepScanLibrary && activeLibrarySyncType == LibrarySyncWorker.SYNC_TYPE_REBUILD -> SyncActionVisualState.LOADING
            action == ConfigAction.SyncMangadexMetadata && activeMetadataSource == MetadataSyncWorker.SOURCE_MANGADEX -> SyncActionVisualState.LOADING
            action == ConfigAction.SyncAnilistMetadata && activeMetadataSource == MetadataSyncWorker.SOURCE_ANILIST -> SyncActionVisualState.LOADING
            successSyncAction == action -> SyncActionVisualState.SUCCESS
            else -> SyncActionVisualState.IDLE
        }

    val uiState =
        ConfigUiState(
            selectedTheme = selectedTheme,
            folderUri = fileSystemAccessViewModel.folderUri,
            folderName = folderName,
            generateComicInfo = generateComicInfo,
            metadataLanguage = metadataLanguage,
        )

    val onAction: (ConfigAction) -> Unit = { action ->
        when (action) {
            is ConfigAction.UpdateTheme -> themeViewModel.setTheme(action.theme)
            is ConfigAction.SelectFolder -> fileSystemAccessViewModel.saveFolderUri(action.uri)
            is ConfigAction.UpdateGenerateComicInfo -> metadataSettingsViewModel.setGenerateComicInfo(action.enabled)
            is ConfigAction.UpdateMetadataLanguage -> metadataSettingsViewModel.setMetadataLanguage(action.language)
            ConfigAction.DeepScanLibrary -> {
                activeSyncAction = ConfigAction.DeepScanLibrary
                comicDirectoryViewModel.deepScanLibrary()
            }
            ConfigAction.QuickSyncLibrary -> {
                activeSyncAction = ConfigAction.QuickSyncLibrary
                comicDirectoryViewModel.syncLibrary()
            }
            ConfigAction.SyncMangadexMetadata -> {
                activeSyncAction = ConfigAction.SyncMangadexMetadata
                comicDexViewModel.rescanMangas()
            }
            ConfigAction.SyncAnilistMetadata -> {
                activeSyncAction = ConfigAction.SyncAnilistMetadata
                comicDexViewModel.rescanAnilistMangas()
            }
            is ConfigAction.CreateCategory -> comicDexViewModel.createCategory(action.name, action.color)
            is ConfigAction.DeleteCategory -> comicDexViewModel.deleteCategory(action.id)
            ConfigAction.NavigateToTemplateConfig -> onNavigateToTemplates()
            ConfigAction.NavigateToSync -> onNavigateToSync()
        }
    }

    Scaffold(
        containerColor = MaterialTheme.colorScheme.background,
        modifier = Modifier.fillMaxSize(),
    ) { paddingValues ->
        Box(modifier = Modifier.fillMaxSize()) {
            Column(
                modifier =
                    Modifier
                        .padding(paddingValues)
                        .padding(bottom = 64.dp)
                        .fillMaxSize()
                        .verticalScroll(scrollState),
            ) {
                if (!tutorialShown) {
                    OnboardingGuideCard()
                }

                val categoryModifier = Modifier.padding(horizontal = SpacingTokens.Large, vertical = SpacingTokens.Small)

                // NOTE: Arquivos Locais
                Acerola.Component.AccordionCard(
                    title = stringResource(id = R.string.title_text_archive_configs_in_app),
                    icon = Icons.Default.Folder,
                    accentColor = MaterialTheme.colorScheme.primary,
                    expanded = "files" in expandedCategories,
                    onToggleExpanded = { toggleCategory("files") },
                    modifier = categoryModifier,
                ) {
                    Main.Config.Component.SelectComicDirectory(
                        folderName = uiState.folderName,
                        onFolderSelected = { onAction(ConfigAction.SelectFolder(it)) },
                    )

                    Main.Config.Component.MetadataExportSettings(
                        enabled = uiState.generateComicInfo,
                        onCheckedChange = { onAction(ConfigAction.UpdateGenerateComicInfo(it)) },
                    )

                    Main.Config.Component.TemplateManager(
                        onManageTemplates = { onAction(ConfigAction.NavigateToTemplateConfig) },
                    )
                }

                // NOTE: Biblioteca
                Acerola.Component.AccordionCard(
                    title = stringResource(id = R.string.label_library_context),
                    icon = Icons.Default.Refresh,
                    accentColor = MaterialTheme.colorScheme.secondary,
                    expanded = "library" in expandedCategories,
                    onToggleExpanded = { toggleCategory("library") },
                    modifier = categoryModifier,
                ) {
                    Main.Config.Component.SyncLibraryArchive(
                        onDeepScan = { onAction(ConfigAction.DeepScanLibrary) },
                        onQuickSync = { onAction(ConfigAction.QuickSyncLibrary) },
                        deepScanState = getSyncActionVisualState(ConfigAction.DeepScanLibrary),
                        quickSyncState = getSyncActionVisualState(ConfigAction.QuickSyncLibrary),
                    )
                }

                // NOTE: Aparência
                Acerola.Component.AccordionCard(
                    title = stringResource(id = R.string.title_settings_appearance),
                    icon = Icons.Default.Palette,
                    accentColor = MaterialTheme.colorScheme.tertiary,
                    expanded = "appearance" in expandedCategories,
                    onToggleExpanded = { toggleCategory("appearance") },
                    modifier = categoryModifier,
                ) {
                    Main.Config.Component.ThemeSettings(
                        currentTheme = uiState.selectedTheme,
                        onThemeChange = { onAction(ConfigAction.UpdateTheme(it)) },
                    )
                }

                // NOTE: Categorias
                Acerola.Component.AccordionCard(
                    title = stringResource(id = R.string.title_config_categories),
                    icon = Icons.Rounded.Bookmark,
                    accentColor = MaterialTheme.colorScheme.primary,
                    expanded = "categories" in expandedCategories,
                    onToggleExpanded = { toggleCategory("categories") },
                    modifier = categoryModifier,
                ) {
                    Main.Config.Component.GlobalCategoryManager(
                        categories = allCategories,
                        onCreateCategory = { name, color -> onAction(ConfigAction.CreateCategory(name, color)) },
                        onDeleteCategory = { id -> onAction(ConfigAction.DeleteCategory(id)) },
                    )
                }

                // NOTE: Metadados
                Acerola.Component.AccordionCard(
                    title = stringResource(id = R.string.label_sync_group),
                    icon = Icons.Default.Public,
                    accentColor = MaterialTheme.colorScheme.secondary,
                    expanded = "metadata" in expandedCategories,
                    onToggleExpanded = { toggleCategory("metadata") },
                    modifier = categoryModifier,
                ) {
                    Main.Config.Component.LanguageSettings(
                        selectedLanguage = uiState.metadataLanguage,
                        onLanguageSelected = { onAction(ConfigAction.UpdateMetadataLanguage(it)) },
                    )

                    Main.Config.Component.SyncMangadexData(
                        onRescan = { onAction(ConfigAction.SyncMangadexMetadata) },
                        state = getSyncActionVisualState(ConfigAction.SyncMangadexMetadata),
                    )

                    Main.Config.Component.SyncAnilistData(
                        onRescan = { onAction(ConfigAction.SyncAnilistMetadata) },
                        state = getSyncActionVisualState(ConfigAction.SyncAnilistMetadata),
                    )
                }

                // NOTE: Sincronização P2P
                Acerola.Component.AccordionCard(
                    title = stringResource(id = R.string.label_sync_activity),
                    icon = Icons.Default.Sync,
                    accentColor = MaterialTheme.colorScheme.tertiary,
                    expanded = "p2p" in expandedCategories,
                    onToggleExpanded = { toggleCategory("p2p") },
                    modifier = categoryModifier,
                ) {
                    Acerola.Component.HeroButton(
                        title = stringResource(id = R.string.label_sync_activity),
                        description = stringResource(id = R.string.description_sync_activity),
                        icon = Icons.Default.Sync,
                        iconTint = MaterialTheme.colorScheme.onPrimaryContainer,
                        iconBackground = MaterialTheme.colorScheme.primaryContainer,
                        onClick = { onAction(ConfigAction.NavigateToSync) },
                    )
                }

                Spacer(modifier = Modifier.height(SizeTokens.ClickTarget))
            }
        }
    }
}

@Composable
private fun OnboardingGuideCard() {
    Card(
        modifier =
            Modifier
                .padding(SpacingTokens.Large)
                .fillMaxWidth(),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
        shape = ShapeTokens.Medium,
    ) {
        Column(modifier = Modifier.padding(SpacingTokens.Large)) {
            Icon(imageVector = Icons.Default.Info, contentDescription = null, tint = MaterialTheme.colorScheme.onPrimaryContainer)
            Spacer(modifier = Modifier.height(SpacingTokens.Small))
            Text(
                text = stringResource(id = R.string.title_tutorial_setup),
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onPrimaryContainer,
            )
            Spacer(modifier = Modifier.height(SpacingTokens.ExtraSmall))
            Text(
                text = "1. " + stringResource(id = R.string.description_tutorial_folder_select),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onPrimaryContainer,
            )
            Spacer(modifier = Modifier.height(SpacingTokens.ExtraSmall))
            Text(
                text = "2. " + stringResource(id = R.string.description_tutorial_sync_deep),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onPrimaryContainer,
            )
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ScreenPreview() {
    AcerolaTheme {
        Main.Config.Component.ThemeSettings(
            currentTheme = AppTheme.CATPPUCCIN,
            onThemeChange = {},
        )
    }
}
