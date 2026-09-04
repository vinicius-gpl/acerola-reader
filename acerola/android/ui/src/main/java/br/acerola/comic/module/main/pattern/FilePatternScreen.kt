package br.acerola.comic.module.main.pattern
import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Add
import androidx.compose.material3.CenterAlignedTopAppBar
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import br.acerola.comic.common.state.LocalSnackbarHostState
import br.acerola.comic.common.ux.component.SnackbarVariant
import br.acerola.comic.common.ux.component.showSnackbar
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.dto.archive.ArchiveTemplateDto
import br.acerola.comic.module.main.Main
import br.acerola.comic.module.main.pattern.component.AddTemplateDialog
import br.acerola.comic.module.main.pattern.component.TemplateItem
import br.acerola.comic.module.main.pattern.state.FilePatternAction
import br.acerola.comic.module.main.pattern.state.FilePatternUiState
import br.acerola.comic.ui.R
import br.acerola.comic.util.sort.SortType

@Composable
fun Main.Pattern.Template.FilePatternScreen(
    viewModel: FilePatternViewModel = hiltViewModel(),
    onBack: () -> Unit,
) {
    val context = LocalContext.current
    val snackbarHostState = LocalSnackbarHostState.current
    val uiState by viewModel.uiState.collectAsState()

    LaunchedEffect(Unit) {
        viewModel.uiEvents.collect { message ->
            snackbarHostState.showSnackbar(message.uiMessage.asString(context), SnackbarVariant.Error)
        }
    }

    FilePatternLayout(
        uiState = uiState,
        onAction = viewModel::onAction,
        onBack = onBack,
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun FilePatternLayout(
    uiState: FilePatternUiState,
    onAction: (FilePatternAction) -> Unit,
    onBack: () -> Unit,
) {
    var showAddDialog by remember { mutableStateOf(false) }
    var editingTemplate by remember { mutableStateOf<ArchiveTemplateDto?>(null) }

    Scaffold(
        modifier = Modifier.fillMaxSize(),
        topBar = {
            CenterAlignedTopAppBar(
                title = {
                    Text(
                        text = stringResource(id = R.string.label_template_config_activity),
                        style = MaterialTheme.typography.titleMedium,
                        fontWeight = FontWeight.Bold,
                    )
                },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(
                            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                            tint = MaterialTheme.colorScheme.onSurface,
                            contentDescription = stringResource(id = R.string.description_icon_navigation_back),
                        )
                    }
                },
                colors =
                    TopAppBarDefaults.topAppBarColors(
                        containerColor = Color.Transparent,
                    ),
            )
        },
        floatingActionButton = {
            FloatingActionButton(
                onClick = { showAddDialog = true },
                containerColor = MaterialTheme.colorScheme.primary,
                contentColor = MaterialTheme.colorScheme.onPrimary,
            ) {
                Icon(
                    imageVector = Icons.Default.Add,
                    contentDescription = stringResource(id = R.string.action_add_template),
                )
            }
        },
    ) { paddingValues ->
        Box(
            modifier =
                Modifier
                    .fillMaxSize()
                    .padding(paddingValues),
        ) {
            if (uiState.templates.isEmpty()) {
                Column(
                    modifier =
                        Modifier
                            .fillMaxSize()
                            .padding(SpacingTokens.Large),
                    verticalArrangement = Arrangement.Center,
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    Text(
                        text = stringResource(id = R.string.label_template_empty_state),
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            } else {
                val groupedTemplates = uiState.templates.groupBy { it.type }

                LazyColumn(
                    modifier = Modifier.fillMaxSize(),
                    contentPadding =
                        PaddingValues(
                            start = SpacingTokens.Large,
                            top = SpacingTokens.None,
                            end = SpacingTokens.Large,
                            bottom = SpacingTokens.SuperGiant,
                        ),
                ) {
                    groupedTemplates.forEach { (type, templates) ->
                        item {
                            val headerText =
                                when (type) {
                                    SortType.CHAPTER -> stringResource(id = R.string.label_sort_type_chapter)
                                    SortType.VOLUME -> stringResource(id = R.string.label_sort_type_volume)
                                }
                            Text(
                                text = headerText,
                                style = MaterialTheme.typography.titleMedium,
                                fontWeight = FontWeight.Bold,
                                modifier = Modifier.padding(top = SpacingTokens.Medium, bottom = SpacingTokens.Small),
                            )
                        }
                        items(
                            items = templates,
                            key = { it.id },
                        ) { template ->
                            Main.Pattern.Component.TemplateItem(
                                template = template,
                                onEdit = { editingTemplate = template },
                                onDelete = { onAction(FilePatternAction.DeleteTemplate(template.id)) },
                            )
                        }
                    }
                }
            }
        }
    }

    if (showAddDialog) {
        Main.Pattern.Component.AddTemplateDialog(
            onDismiss = { showAddDialog = false },
            onConfirm = { label, pattern, type ->
                onAction(FilePatternAction.AddTemplate(label, pattern, type))
                showAddDialog = false
            },
        )
    }

    editingTemplate?.let { template ->
        Main.Pattern.Component.AddTemplateDialog(
            isEditMode = true,
            initialLabel = template.label,
            initialPattern = template.pattern,
            initialType = template.type,
            onDismiss = { editingTemplate = null },
            onConfirm = { label, pattern, type ->
                onAction(FilePatternAction.EditTemplate(template.id, label, pattern, type))
                editingTemplate = null
            },
        )
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun FilePatternScreenPreview() {
    AcerolaTheme {
        FilePatternLayout(
            uiState =
                FilePatternUiState(
                    templates =
                        listOf(
                            ArchiveTemplateDto(
                                id = 1L,
                                label = "Padrao Manga",
                                pattern = "{title} - {chapter}",
                                type = SortType.CHAPTER,
                                isDefault = true,
                            ),
                        ),
                ),
            onAction = {},
            onBack = {},
        )
    }
}
