package br.acerola.comic.module.main.sync

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Error
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.Dialog
import br.acerola.comic.common.ux.component.DialogButton
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.ShapeTokens
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.module.main.Main
import br.acerola.comic.module.main.sync.state.LogState
import br.acerola.comic.module.main.sync.state.TransferLogEntry
import br.acerola.comic.ui.R

/**
 * Tela cheia de histórico de transferências P2P — mesmo formato do localSend: barra própria
 * com seta de voltar, botões de "Atualizar"/"Limpar" logo abaixo, depois a lista. Uma tela de
 * verdade (não um dialog/sheet/accordion espremido na tela de Rede) deixa o scroll óbvio (o
 * próximo item cortado na borda inferior já avisa que tem mais) e nunca compete por espaço com
 * o resto da UI da aba de Rede.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun Main.Sync.Template.TransferLogScreen(
    onBack: () -> Unit,
    viewModel: TransferLogViewModel = hiltViewModel(),
) {
    val uiState by viewModel.uiState.collectAsState()
    var showClearDialog by remember { mutableStateOf(false) }

    Scaffold(
        modifier = Modifier.fillMaxSize(),
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text = stringResource(id = R.string.title_sync_activity_log),
                        style = MaterialTheme.typography.titleMedium,
                        fontWeight = FontWeight.Bold,
                    )
                },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(
                            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                            contentDescription = stringResource(id = R.string.description_icon_navigation_back),
                        )
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(containerColor = Color.Transparent),
            )
        },
    ) { paddingValues ->
        Column(modifier = Modifier.fillMaxSize().padding(paddingValues)) {
            Row(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .padding(horizontal = SpacingTokens.Large, vertical = SpacingTokens.Small),
                horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
            ) {
                FilledTonalButton(onClick = { viewModel.refresh() }) {
                    Icon(
                        imageVector = Icons.Default.Refresh,
                        contentDescription = null,
                        modifier = Modifier.size(SizeTokens.IconExtraSmall),
                    )
                    Text(
                        text = stringResource(id = R.string.action_sync_activity_log_refresh),
                        modifier = Modifier.padding(start = SpacingTokens.ExtraSmall),
                    )
                }

                if (uiState.entries.isNotEmpty()) {
                    FilledTonalButton(onClick = { showClearDialog = true }) {
                        Icon(
                            imageVector = Icons.Default.Delete,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.error,
                            modifier = Modifier.size(SizeTokens.IconExtraSmall),
                        )
                        Text(
                            text = stringResource(id = R.string.action_sync_activity_log_clear),
                            color = MaterialTheme.colorScheme.error,
                            modifier = Modifier.padding(start = SpacingTokens.ExtraSmall),
                        )
                    }
                }
            }

            TransferLogList(entries = uiState.entries, modifier = Modifier.fillMaxSize())
        }
    }

    Acerola.Component.Dialog(
        show = showClearDialog,
        onDismiss = { showClearDialog = false },
        title = stringResource(id = R.string.title_sync_activity_log_clear_confirm),
        confirmButtonContent = {
            Acerola.Component.DialogButton(
                text = stringResource(id = R.string.action_sync_activity_log_clear_confirm),
                contentColor = MaterialTheme.colorScheme.error,
                onClick = {
                    viewModel.clear()
                    showClearDialog = false
                },
            )
        },
        dismissButtonContent = {
            Acerola.Component.DialogButton(
                text = stringResource(id = R.string.action_cancel),
                onClick = { showClearDialog = false },
            )
        },
    ) {
        Text(text = stringResource(id = R.string.description_sync_activity_log_clear_confirm))
    }
}

@Composable
private fun TransferLogList(
    entries: List<TransferLogEntry>,
    modifier: Modifier = Modifier,
) {
    if (entries.isEmpty()) {
        Box(modifier = modifier, contentAlignment = Alignment.Center) {
            Text(
                text = stringResource(id = R.string.label_sync_activity_log_empty),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    } else {
        LazyColumn(
            modifier = modifier,
            contentPadding = PaddingValues(horizontal = SpacingTokens.Large, vertical = SpacingTokens.Small),
            verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
        ) {
            items(items = entries, key = { it.id }) { entry ->
                TransferLogCard(entry = entry)
            }
        }
    }
}

/** Mini-card estilo localSend: avatar quadrado com o ícone de status + título/subtítulo em
 *  duas linhas, em vez de ícone pequeno + texto + data jogados numa linha só (formato antigo
 *  do accordion, ilegível numa tela dedicada a isso). */
@Composable
private fun TransferLogCard(entry: TransferLogEntry) {
    val containerColor: Color
    val contentColor: Color
    when (entry.state) {
        LogState.SUCCESS -> {
            containerColor = MaterialTheme.colorScheme.primaryContainer
            contentColor = MaterialTheme.colorScheme.onPrimaryContainer
        }
        LogState.ERROR -> {
            containerColor = MaterialTheme.colorScheme.errorContainer
            contentColor = MaterialTheme.colorScheme.onErrorContainer
        }
        LogState.IN_PROGRESS -> {
            containerColor = MaterialTheme.colorScheme.surfaceVariant
            contentColor = MaterialTheme.colorScheme.onSurfaceVariant
        }
    }

    Surface(
        shape = ShapeTokens.Large,
        color = MaterialTheme.colorScheme.surfaceContainer,
        modifier = Modifier.fillMaxWidth(),
    ) {
        Row(
            modifier = Modifier.padding(SpacingTokens.Medium),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Surface(
                shape = ShapeTokens.Medium,
                color = containerColor,
                modifier = Modifier.size(SizeTokens.ClickTarget),
            ) {
                Box(contentAlignment = Alignment.Center) {
                    when (entry.state) {
                        LogState.IN_PROGRESS ->
                            CircularProgressIndicator(
                                modifier = Modifier.size(SizeTokens.IconSmall),
                                strokeWidth = 2.dp,
                                color = contentColor,
                            )
                        LogState.SUCCESS ->
                            Icon(
                                imageVector = Icons.Default.CheckCircle,
                                contentDescription = null,
                                tint = contentColor,
                                modifier = Modifier.size(SizeTokens.IconMedium),
                            )
                        LogState.ERROR ->
                            Icon(
                                imageVector = Icons.Default.Error,
                                contentDescription = null,
                                tint = contentColor,
                                modifier = Modifier.size(SizeTokens.IconMedium),
                            )
                    }
                }
            }

            Spacer(modifier = Modifier.width(SpacingTokens.Medium))

            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = describeEntry(entry),
                    style = MaterialTheme.typography.bodyMedium,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.onSurface,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )
                Spacer(modifier = Modifier.height(SpacingTokens.ExtraSmall))
                Text(
                    text = formatLogTimestamp(entry.timestamp),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun TransferLogListPreview() {
    AcerolaTheme {
        TransferLogList(
            entries =
                listOf(
                    TransferLogEntry(id = 1, kind = "files", status = "complete", state = LogState.SUCCESS),
                    TransferLogEntry(
                        id = 2,
                        kind = "history",
                        status = "error",
                        state = LogState.ERROR,
                        message = "conexão perdida",
                    ),
                ),
        )
    }
}

@Preview(name = "Empty", showBackground = true)
@Composable
private fun TransferLogListEmptyPreview() {
    AcerolaTheme {
        TransferLogList(entries = emptyList())
    }
}
