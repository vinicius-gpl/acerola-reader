package br.acerola.comic.module.main.sync

import android.content.res.Configuration
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.ContentCopy
import androidx.compose.material.icons.filled.ContentPaste
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Description
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.Error
import androidx.compose.material.icons.filled.ExpandLess
import androidx.compose.material.icons.filled.ExpandMore
import androidx.compose.material.icons.filled.History
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.filled.PersonRemove
import androidx.compose.material.icons.filled.PhoneAndroid
import androidx.compose.material.icons.filled.QrCodeScanner
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.Sync
import androidx.compose.material.icons.filled.Wifi
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CenterAlignedTopAppBar
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import br.acerola.comic.common.state.LocalSnackbarHostState
import br.acerola.comic.common.state.SyncActionVisualState
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.AccordionCard
import br.acerola.comic.common.ux.component.AdaptiveSheet
import br.acerola.comic.common.ux.component.Dialog
import br.acerola.comic.common.ux.component.DialogButton
import br.acerola.comic.common.ux.component.HeroButton
import br.acerola.comic.common.ux.component.SegmentedControl
import br.acerola.comic.common.ux.component.SnackbarVariant
import br.acerola.comic.common.ux.component.SyncActionIcon
import br.acerola.comic.common.ux.component.showSnackbar
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.ShapeTokens
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.config.preference.RelayPreference
import br.acerola.comic.logging.AcerolaLogger
import br.acerola.comic.logging.LogSource
import br.acerola.comic.module.main.Main
import br.acerola.comic.module.main.sync.state.ConnectError
import br.acerola.comic.module.main.sync.state.LogState
import br.acerola.comic.module.main.sync.state.PairedPeer
import br.acerola.comic.module.main.sync.state.RelaySettingsUiState
import br.acerola.comic.module.main.sync.state.SyncAction
import br.acerola.comic.module.main.sync.state.SyncResult
import br.acerola.comic.module.main.sync.state.SyncUiState
import br.acerola.comic.module.main.sync.state.TransferLogEntry
import br.acerola.comic.service.NetworkMode
import br.acerola.comic.ui.R
import br.acerola.comic.util.p2p.PairingCode
import br.acerola.comic.util.p2p.QrBitmapGenerator
import com.google.mlkit.vision.barcode.common.Barcode
import com.google.mlkit.vision.codescanner.GmsBarcodeScannerOptions
import com.google.mlkit.vision.codescanner.GmsBarcodeScanning
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

@Composable
fun Main.Sync.Template.Screen(
    viewModel: SyncViewModel = hiltViewModel(),
    onBack: () -> Unit,
) {
    val uiState by viewModel.uiState.collectAsState()

    SyncLayout(
        uiState = uiState,
        onAction = viewModel::onAction,
        onBack = onBack,
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun SyncLayout(
    uiState: SyncUiState,
    onAction: (SyncAction) -> Unit,
    onBack: () -> Unit,
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val snackbarHostState = LocalSnackbarHostState.current
    val scanFailedMessage = stringResource(id = R.string.error_sync_scan_failed)
    val scanEmptyMessage = stringResource(id = R.string.error_sync_scan_empty)
    var pasteValue by remember { mutableStateOf("") }
    var showAddDeviceSheet by remember { mutableStateOf(false) }
    var addDeviceTab by remember { mutableStateOf(0) }
    var peerPendingRemoval by remember { mutableStateOf<PairedPeer?>(null) }

    fun startScan(onResult: (String) -> Unit) {
        val options = GmsBarcodeScannerOptions.Builder().setBarcodeFormats(Barcode.FORMAT_QR_CODE).build()
        GmsBarcodeScanning
            .getClient(context, options)
            .startScan()
            .addOnSuccessListener { barcode ->
                val value = barcode.rawValue
                if (value != null) {
                    onResult(value)
                } else {
                    AcerolaLogger.w("SyncScreen", "QR scan succeeded but rawValue was null", LogSource.UI)
                    scope.launch { snackbarHostState.showSnackbar(scanEmptyMessage, SnackbarVariant.Error) }
                }
            }.addOnFailureListener { error ->
                AcerolaLogger.e("SyncScreen", "QR scan failed", LogSource.UI, error)
                scope.launch { snackbarHostState.showSnackbar(scanFailedMessage, SnackbarVariant.Error) }
            }
    }

    LaunchedEffect(uiState.connectError) {
        val message =
            when (uiState.connectError) {
                ConnectError.INVALID_CODE -> context.getString(R.string.error_sync_connect_invalid_code)
                ConnectError.CONNECTION_FAILED -> context.getString(R.string.error_sync_connect_failed)
                null -> return@LaunchedEffect
            }
        snackbarHostState.showSnackbar(message, SnackbarVariant.Error)
    }

    // Um código válido já resolve o propósito do sheet (o diálogo de confirmação assume
    // o resto do fluxo) — sem isso, o sheet ficava aberto atrás do diálogo sem necessidade.
    LaunchedEffect(uiState.pendingConnect) {
        if (uiState.pendingConnect != null) {
            showAddDeviceSheet = false
        }
    }

    Scaffold(
        modifier = Modifier.fillMaxSize(),
        topBar = {
            CenterAlignedTopAppBar(
                title = {
                    Text(
                        text = stringResource(id = R.string.label_sync_activity),
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
                colors = TopAppBarDefaults.topAppBarColors(containerColor = Color.Transparent),
            )
        },
    ) { paddingValues ->
        Column(
            modifier =
                Modifier
                    .padding(paddingValues)
                    .fillMaxSize()
                    .verticalScroll(rememberScrollState())
                    .padding(SpacingTokens.Large),
            verticalArrangement = Arrangement.spacedBy(SpacingTokens.Large),
        ) {
            Text(
                text = stringResource(id = R.string.description_sync_activity),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )

            ThisDeviceSection(uiState = uiState, onAction = onAction)

            RelaySettingsCard(
                relaySettings = uiState.relaySettings,
                irohServicesTicketError = uiState.irohServicesTicketError,
                onAction = onAction,
            )

            PeersSection(
                uiState = uiState,
                onAction = onAction,
                onAddDeviceClick = { showAddDeviceSheet = true },
                onRemoveClick = { peerPendingRemoval = it },
            )

            ActivityLogCard(uiState = uiState)
        }

        if (uiState.pendingConnect != null) {
            ConfirmConnectDialog(
                peerId = uiState.pendingConnect.peerId,
                onConfirm = { onAction(SyncAction.ConfirmConnect) },
                onCancel = { onAction(SyncAction.CancelConnect) },
            )
        }

        if (uiState.trustedPeerDialogPeerId != null) {
            TofuDialog(
                peerId = uiState.trustedPeerDialogPeerId,
                onDismiss = { onAction(SyncAction.DismissTrustDialog) },
            )
        }

        peerPendingRemoval?.let { peer ->
            RemovePeerDialog(
                peer = peer,
                onConfirm = {
                    onAction(SyncAction.RemovePeer(peer.peerId))
                    peerPendingRemoval = null
                },
                onCancel = { peerPendingRemoval = null },
            )
        }

        if (showAddDeviceSheet) {
            Acerola.Component.AdaptiveSheet(onDismissRequest = { showAddDeviceSheet = false }) {
                AddDeviceSheetContent(
                    uiState = uiState,
                    selectedTab = addDeviceTab,
                    onTabSelected = { addDeviceTab = it },
                    pasteValue = pasteValue,
                    onPasteValueChange = { pasteValue = it },
                    onConnect = { onAction(SyncAction.ProposeConnect(pasteValue)) },
                    onScan = { startScan { value -> onAction(SyncAction.ProposeConnect(value)) } },
                    onDismissError = { onAction(SyncAction.DismissConnectError) },
                )
            }
        }

        uiState.browsingPeerId?.let { peerId ->
            val peerDisplayName =
                uiState.pairedPeers.find { it.peerId == peerId }?.deviceName ?: PairingCode.shortId(peerId)
            RemoteLibrarySheet(
                peerDisplayName = peerDisplayName,
                comics = uiState.remoteLibrary,
                isLoading = !uiState.remoteLibraryLoaded && uiState.browseLibraryError == null,
                errorMessage = uiState.browseLibraryError,
                errorType = uiState.browseLibraryErrorType,
                onSelectComic = { comicName -> onAction(SyncAction.SyncComic(peerId, comicName)) },
                onDismiss = { onAction(SyncAction.DismissLibraryBrowse) },
                coverPathFor = { comicName -> uiState.remoteCoverPaths[coverKey(peerId, comicName)] },
            )
        }
    }
}

@Composable
private fun ThisDeviceSection(
    uiState: SyncUiState,
    onAction: (SyncAction) -> Unit,
) {
    val clipboardManager = LocalClipboardManager.current
    val scope = rememberCoroutineScope()
    val snackbarHostState = LocalSnackbarHostState.current
    val copiedMessage = stringResource(id = R.string.label_sync_copied)
    val renamedMessage = stringResource(id = R.string.label_sync_rename_success)

    // Apelido custom estilo LocalSend — edição inline em vez de um dialog separado, já que é
    // um campo único e essa tela já é o lugar natural pra mexer nisso (mesma abordagem do
    // lado desktop, ver `acerola-network-my-device-card.svelte`).
    var editingName by remember { mutableStateOf(false) }
    var nameDraft by remember { mutableStateOf("") }

    if (editingName) {
        Card(shape = ShapeTokens.Huge, modifier = Modifier.fillMaxWidth()) {
            Row(
                modifier = Modifier.padding(SpacingTokens.Large),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
            ) {
                OutlinedTextField(
                    value = nameDraft,
                    onValueChange = { nameDraft = it },
                    placeholder = { Text(text = stringResource(id = R.string.hint_sync_rename_device)) },
                    singleLine = true,
                    modifier = Modifier.weight(1f),
                )
                IconButton(
                    enabled = nameDraft.isNotBlank(),
                    onClick = {
                        onAction(SyncAction.RenameDevice(nameDraft))
                        scope.launch { snackbarHostState.showSnackbar(renamedMessage, SnackbarVariant.Success) }
                        editingName = false
                    },
                ) {
                    Icon(
                        imageVector = Icons.Default.Check,
                        contentDescription = stringResource(id = R.string.action_sync_rename_save),
                    )
                }
                IconButton(onClick = { editingName = false }) {
                    Icon(imageVector = Icons.Default.Close, contentDescription = stringResource(id = R.string.action_cancel))
                }
            }
        }
        return
    }

    Acerola.Component.HeroButton(
        title = uiState.localDeviceName,
        description = uiState.localId,
        modifier = Modifier.fillMaxWidth(),
        icon = {
            Icon(
                imageVector = Icons.Default.PhoneAndroid,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onPrimaryContainer,
                modifier = Modifier.size(SizeTokens.IconMedium),
            )
        },
        action = {
            Row(verticalAlignment = Alignment.CenterVertically) {
                IconButton(onClick = {
                    nameDraft = uiState.localDeviceName
                    editingName = true
                }) {
                    Icon(
                        imageVector = Icons.Default.Edit,
                        contentDescription = stringResource(id = R.string.action_sync_rename_device),
                    )
                }
                OutlinedButton(onClick = {
                    clipboardManager.setText(AnnotatedString(uiState.localId))
                    scope.launch { snackbarHostState.showSnackbar(copiedMessage, SnackbarVariant.Success) }
                }) {
                    Icon(
                        imageVector = Icons.Default.ContentCopy,
                        contentDescription = null,
                        modifier = Modifier.size(SizeTokens.IconExtraSmall),
                    )
                    Spacer(modifier = Modifier.width(SpacingTokens.ExtraSmall))
                    Text(text = stringResource(id = R.string.action_sync_copy_id))
                }
            }
        },
        bottomContent = {
            Row(
                modifier = Modifier.padding(horizontal = SpacingTokens.ExtraLarge, vertical = SpacingTokens.Small),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text =
                        stringResource(
                            id =
                                when (uiState.mode) {
                                    NetworkMode.LOCAL -> R.string.label_sync_mode_local
                                    NetworkMode.RELAY -> R.string.label_sync_mode_relay
                                },
                        ),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        },
    )
}

@Composable
private fun RelaySettingsCard(
    relaySettings: RelaySettingsUiState,
    irohServicesTicketError: Boolean,
    onAction: (SyncAction) -> Unit,
) {
    var expanded by remember { mutableStateOf(false) }

    val activeSourceCount =
        (if (relaySettings.useAcerolaRelay) 1 else 0) +
            relaySettings.customRelayUrls.size +
            relaySettings.irohRelayUrls.size

    val summary =
        when {
            relaySettings.useIrohPublicNetwork -> stringResource(id = R.string.label_relay_settings_summary_iroh_public)
            activeSourceCount == 0 -> stringResource(id = R.string.label_relay_settings_summary_mdns_only)
            else -> stringResource(id = R.string.label_relay_settings_summary_active, activeSourceCount)
        }

    Acerola.Component.AccordionCard(
        title = stringResource(id = R.string.title_relay_settings),
        description = summary,
        icon = Icons.Default.Wifi,
        accentColor = MaterialTheme.colorScheme.secondary,
        expanded = expanded,
        onToggleExpanded = { expanded = !expanded },
        modifier = Modifier.fillMaxWidth(),
    ) {
        RelaySwitchRow(
            title = stringResource(id = R.string.label_relay_settings_use_acerola_relay),
            description = stringResource(id = R.string.label_relay_settings_use_acerola_relay_desc, RelayPreference.DEFAULT_ACEROLA_RELAY_URL),
            checked = relaySettings.useAcerolaRelay,
            enabled = !relaySettings.useIrohPublicNetwork,
            onCheckedChange = { onAction(SyncAction.ToggleUseAcerolaRelay(it)) },
        )

        RelaySwitchRow(
            title = stringResource(id = R.string.label_relay_settings_use_iroh_public_network),
            description = stringResource(id = R.string.label_relay_settings_use_iroh_public_network_desc),
            checked = relaySettings.useIrohPublicNetwork,
            enabled = relaySettings.hasIrohServicesTicket,
            onCheckedChange = { onAction(SyncAction.ToggleUseIrohPublicNetwork(it)) },
        )

        // Sem isso, o switch cinza acima não explica por conta própria por que está travado —
        // o usuário precisa saber que a solução é colar um ticket na seção logo abaixo.
        if (!relaySettings.hasIrohServicesTicket) {
            Text(
                text = stringResource(id = R.string.label_relay_settings_iroh_services_ticket_required_hint),
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                fontStyle = FontStyle.Italic,
                modifier = Modifier.padding(horizontal = SpacingTokens.Small),
            )
        }

        if (relaySettings.useIrohPublicNetwork) {
            Text(
                text = stringResource(id = R.string.label_relay_settings_exclusive_note),
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(horizontal = SpacingTokens.Small),
            )
        }

        IrohServicesTicketSection(
            hasTicket = relaySettings.hasIrohServicesTicket,
            hasError = irohServicesTicketError,
            onSave = { onAction(SyncAction.SetIrohServicesTicket(it)) },
            onRemove = { onAction(SyncAction.ClearIrohServicesTicket) },
            onDismissError = { onAction(SyncAction.DismissIrohServicesTicketError) },
        )

        RelayUrlListEditor(
            title = stringResource(id = R.string.title_relay_settings_custom_relays),
            urls = relaySettings.customRelayUrls,
            placeholder = stringResource(id = R.string.hint_relay_settings_custom_relay_add),
            emptyLabel = stringResource(id = R.string.label_relay_settings_custom_relays_empty),
            removeContentDescription = stringResource(id = R.string.action_relay_settings_custom_relay_remove),
            enabled = !relaySettings.useIrohPublicNetwork,
            onAdd = { onAction(SyncAction.AddCustomRelayUrl(it)) },
            onRemove = { onAction(SyncAction.RemoveCustomRelayUrl(it)) },
        )

        RelayUrlListEditor(
            title = stringResource(id = R.string.title_relay_settings_iroh_relays),
            urls = relaySettings.irohRelayUrls,
            placeholder = stringResource(id = R.string.hint_relay_settings_iroh_relay_add),
            emptyLabel = stringResource(id = R.string.label_relay_settings_iroh_relays_empty),
            removeContentDescription = stringResource(id = R.string.action_relay_settings_iroh_relay_remove),
            enabled = !relaySettings.useIrohPublicNetwork,
            onAdd = { onAction(SyncAction.AddIrohRelayUrl(it)) },
            onRemove = { onAction(SyncAction.RemoveIrohRelayUrl(it)) },
        )
    }
}

/** Ticket da conta do PRÓPRIO usuário em `services.iroh.computer` — nunca um secret de projeto
 *  embutido no build (ver `RelayModeConfig::IrohDefault` do lado Rust). O valor em si nunca é
 *  exibido de volta (é uma credencial real, guardada no cofre criptografado do node) — só
 *  [hasTicket] chega aqui. */
@Composable
private fun IrohServicesTicketSection(
    hasTicket: Boolean,
    hasError: Boolean,
    onSave: (String) -> Unit,
    onRemove: () -> Unit,
    onDismissError: () -> Unit,
) {
    var draft by remember { mutableStateOf("") }

    Column(
        modifier =
            Modifier
                .fillMaxWidth()
                .clip(ShapeTokens.Medium)
                .background(MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.4f))
                .padding(SpacingTokens.Medium),
        verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
    ) {
        Text(
            text = stringResource(id = R.string.title_relay_settings_iroh_services_ticket),
            style = MaterialTheme.typography.labelSmall,
            fontWeight = FontWeight.Bold,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Text(
            text =
                stringResource(
                    id =
                        if (hasTicket) {
                            R.string.label_relay_settings_iroh_services_ticket_configured
                        } else {
                            R.string.label_relay_settings_iroh_services_ticket_not_configured
                        },
                ),
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )

        // Campo em linha própria, largura cheia — na versão anterior ele dividia a linha com o
        // botão "Salvar ticket", o que espremia o campo a ponto do placeholder (bem mais longo
        // que qualquer URL de relay) quebrar em 4 linhas e inflar a caixa inteira.
        OutlinedTextField(
            value = draft,
            onValueChange = {
                draft = it
                if (hasError) onDismissError()
            },
            placeholder = {
                Text(
                    text = stringResource(id = R.string.hint_relay_settings_iroh_services_ticket),
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            },
            singleLine = true,
            isError = hasError,
            visualTransformation = PasswordVisualTransformation(),
            modifier = Modifier.fillMaxWidth(),
        )

        if (hasError) {
            Text(
                text = stringResource(id = R.string.error_relay_settings_iroh_services_ticket_invalid),
                style = MaterialTheme.typography.labelSmall,
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.error,
            )
        }

        // Ações numa linha própria, alinhadas à direita — evita competir por espaço com o campo
        // acima em telas estreitas.
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small, Alignment.End),
        ) {
            if (hasTicket) {
                TextButton(onClick = onRemove) {
                    Text(
                        text = stringResource(id = R.string.action_relay_settings_iroh_services_ticket_remove),
                        color = MaterialTheme.colorScheme.error,
                    )
                }
            }
            Button(
                onClick = {
                    val trimmed = draft.trim()
                    if (trimmed.isBlank()) return@Button
                    onSave(trimmed)
                    draft = ""
                },
                enabled = draft.isNotBlank(),
            ) {
                Text(
                    text =
                        stringResource(
                            id =
                                if (hasTicket) {
                                    R.string.action_relay_settings_iroh_services_ticket_replace
                                } else {
                                    R.string.action_relay_settings_iroh_services_ticket_save
                                },
                        ),
                )
            }
        }

        Text(
            text = stringResource(id = R.string.label_relay_settings_iroh_services_ticket_help),
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Composable
private fun RelaySwitchRow(
    title: String,
    description: String,
    checked: Boolean,
    enabled: Boolean,
    onCheckedChange: (Boolean) -> Unit,
) {
    Row(
        modifier =
            Modifier
                .fillMaxWidth()
                .padding(horizontal = SpacingTokens.Small),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = title,
                style = MaterialTheme.typography.bodyMedium,
                fontWeight = FontWeight.SemiBold,
            )
            Text(
                text = description,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        Spacer(modifier = Modifier.width(SpacingTokens.Small))
        Switch(checked = checked, enabled = enabled, onCheckedChange = onCheckedChange)
    }
}

/** Espelha o comportamento das listas de relay próprio/Iroh do Desktop
 *  (`acerola-network-relay-settings-card.svelte`): valida a URL (http/https) antes de aceitar,
 *  desabilita edição enquanto a rede pública do Iroh estiver ativa (mutuamente exclusiva). */
@Composable
private fun RelayUrlListEditor(
    title: String,
    urls: List<String>,
    placeholder: String,
    emptyLabel: String,
    removeContentDescription: String,
    enabled: Boolean,
    onAdd: (String) -> Unit,
    onRemove: (String) -> Unit,
) {
    var draft by remember { mutableStateOf("") }
    var showError by remember { mutableStateOf(false) }

    Column(
        modifier =
            Modifier
                .fillMaxWidth()
                .padding(horizontal = SpacingTokens.Small),
        verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
    ) {
        Text(
            text = title,
            style = MaterialTheme.typography.labelSmall,
            fontWeight = FontWeight.Bold,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )

        if (urls.isEmpty()) {
            Text(
                text = emptyLabel,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }

        urls.forEach { url ->
            Row(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .clip(ShapeTokens.Medium)
                        .background(MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.4f))
                        .padding(start = SpacingTokens.Medium, end = SpacingTokens.ExtraSmall),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = url,
                    style = MaterialTheme.typography.bodySmall,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.weight(1f),
                )
                IconButton(enabled = enabled, onClick = { onRemove(url) }) {
                    Icon(
                        imageVector = Icons.Default.Delete,
                        contentDescription = removeContentDescription,
                        tint = MaterialTheme.colorScheme.error,
                        modifier = Modifier.size(SizeTokens.IconSmall),
                    )
                }
            }
        }

        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
        ) {
            OutlinedTextField(
                value = draft,
                onValueChange = {
                    draft = it
                    showError = false
                },
                placeholder = { Text(text = placeholder) },
                singleLine = true,
                enabled = enabled,
                modifier = Modifier.weight(1f),
            )
            IconButton(
                enabled = enabled && draft.isNotBlank(),
                onClick = {
                    val trimmed = draft.trim()
                    if (isValidRelayUrl(trimmed)) {
                        onAdd(trimmed)
                        draft = ""
                        showError = false
                    } else {
                        showError = true
                    }
                },
            ) {
                Icon(imageVector = Icons.Default.Add, contentDescription = null)
            }
        }

        if (showError) {
            Text(
                text = stringResource(id = R.string.error_relay_settings_invalid_url),
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.error,
            )
        }
    }
}

private fun isValidRelayUrl(value: String): Boolean =
    try {
        val scheme = java.net.URI(value).scheme
        scheme == "http" || scheme == "https"
    } catch (error: java.net.URISyntaxException) {
        false
    }

@Composable
private fun PeersSection(
    uiState: SyncUiState,
    onAction: (SyncAction) -> Unit,
    onAddDeviceClick: () -> Unit,
    onRemoveClick: (PairedPeer) -> Unit,
) {
    Column(modifier = Modifier.fillMaxWidth()) {
        SectionHeader(title = stringResource(id = R.string.title_sync_peers))
        Spacer(modifier = Modifier.height(SpacingTokens.Small))

        if (uiState.pairedPeers.isEmpty()) {
            Acerola.Component.HeroButton(
                title = stringResource(id = R.string.action_sync_add_device),
                description = stringResource(id = R.string.label_sync_no_peers),
                icon = Icons.Default.Add,
                onClick = onAddDeviceClick,
            )
        } else {
            Column(verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small)) {
                uiState.pairedPeers.forEach { peer ->
                    PeerRow(
                        peer = peer,
                        isOnline = peer.peerId in uiState.connectedPeerIds,
                        syncingKeys = uiState.syncingKeys,
                        lastSyncedAt = uiState.lastSyncedByPeer[peer.peerId],
                        lastResult = uiState.lastSyncResultByPeer[peer.peerId],
                        onAction = onAction,
                        onRemoveClick = { onRemoveClick(peer) },
                    )
                }
            }

            Spacer(modifier = Modifier.height(SpacingTokens.Small))

            OutlinedButton(onClick = onAddDeviceClick, modifier = Modifier.fillMaxWidth()) {
                Icon(imageVector = Icons.Default.Add, contentDescription = null, modifier = Modifier.size(SizeTokens.IconExtraSmall))
                Spacer(modifier = Modifier.width(SpacingTokens.ExtraSmall))
                Text(text = stringResource(id = R.string.action_sync_add_device))
            }
        }
    }
}

@Composable
private fun PeerRow(
    peer: PairedPeer,
    isOnline: Boolean,
    syncingKeys: Set<String>,
    lastSyncedAt: Long?,
    lastResult: SyncResult?,
    onAction: (SyncAction) -> Unit,
    onRemoveClick: () -> Unit,
) {
    val historySyncing = syncKey(peer.peerId, SYNC_KIND_HISTORY) in syncingKeys
    val filesSyncing = syncKey(peer.peerId, SYNC_KIND_FILES) in syncingKeys
    val anySyncing = historySyncing || filesSyncing
    var menuExpanded by remember { mutableStateOf(false) }

    // Pisca o ícone de sucesso por um instante quando uma sessão termina bem — mesma ideia
    // do SyncActionIcon usado em SyncLibraryArchive — em vez de voltar direto pro ícone
    // ocioso sem nenhum feedback de que a sync deu certo.
    var showSuccess by remember(peer.peerId) { mutableStateOf(false) }
    LaunchedEffect(lastResult) {
        if (lastResult?.state == LogState.SUCCESS) {
            showSuccess = true
            delay(2000)
            showSuccess = false
        }
    }

    val isErrorIdle = !anySyncing && !showSuccess && lastResult?.state == LogState.ERROR
    val syncIconState =
        when {
            anySyncing -> SyncActionVisualState.LOADING
            showSuccess -> SyncActionVisualState.SUCCESS
            else -> SyncActionVisualState.IDLE
        }
    val description =
        when {
            isOnline -> stringResource(id = R.string.label_sync_peer_online)
            lastSyncedAt != null -> stringResource(id = R.string.label_sync_last_synced, formatLogTimestamp(lastSyncedAt))
            else -> stringResource(id = R.string.label_sync_never_synced)
        }

    Acerola.Component.HeroButton(
        title = peer.deviceName ?: PairingCode.shortId(peer.peerId),
        description = description,
        modifier = Modifier.fillMaxWidth(),
        bottomContent =
            if (isErrorIdle) {
                {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        modifier = Modifier.padding(horizontal = SpacingTokens.ExtraLarge, vertical = SpacingTokens.Small),
                    ) {
                        Icon(
                            imageVector = Icons.Default.Error,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.error,
                            modifier = Modifier.size(SizeTokens.IconExtraSmall),
                        )
                        Spacer(modifier = Modifier.width(SpacingTokens.ExtraSmall))
                        Text(
                            text =
                                lastResult?.errorType?.uiMessage?.asString()
                                    ?: lastResult?.message?.let {
                                        stringResource(id = R.string.error_sync_last_attempt_failed, it)
                                    }
                                    ?: stringResource(id = R.string.error_sync_connect_failed),
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.error,
                        )
                    }
                }
            } else {
                null
            },
        action = {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Box {
                    IconButton(onClick = { menuExpanded = true }) {
                        Icon(
                            imageVector = Icons.Default.MoreVert,
                            contentDescription = stringResource(id = R.string.description_icon_sync_peer_more_actions),
                        )
                    }
                    DropdownMenu(expanded = menuExpanded, onDismissRequest = { menuExpanded = false }) {
                        DropdownMenuItem(
                            text = { Text(stringResource(id = R.string.action_sync_history)) },
                            leadingIcon = { Icon(imageVector = Icons.Default.History, contentDescription = null) },
                            enabled = !historySyncing,
                            onClick = {
                                menuExpanded = false
                                onAction(SyncAction.SyncHistory(peer.peerId))
                            },
                        )
                        DropdownMenuItem(
                            text = { Text(stringResource(id = R.string.action_sync_files)) },
                            leadingIcon = { Icon(imageVector = Icons.Default.Description, contentDescription = null) },
                            enabled = !filesSyncing,
                            onClick = {
                                menuExpanded = false
                                onAction(SyncAction.SyncFiles(peer.peerId))
                            },
                        )
                        DropdownMenuItem(
                            text = { Text(stringResource(id = R.string.action_sync_browse_library)) },
                            leadingIcon = { Icon(imageVector = Icons.Default.Search, contentDescription = null) },
                            enabled = !filesSyncing,
                            onClick = {
                                menuExpanded = false
                                onAction(SyncAction.BrowseLibrary(peer.peerId))
                            },
                        )
                        HorizontalDivider()
                        DropdownMenuItem(
                            text = {
                                Text(
                                    text = stringResource(id = R.string.action_sync_remove_peer),
                                    color = MaterialTheme.colorScheme.error,
                                )
                            },
                            leadingIcon = {
                                Icon(
                                    imageVector = Icons.Default.PersonRemove,
                                    contentDescription = null,
                                    tint = MaterialTheme.colorScheme.error,
                                )
                            },
                            onClick = {
                                menuExpanded = false
                                onRemoveClick()
                            },
                        )
                    }
                }
                Spacer(modifier = Modifier.width(SpacingTokens.Small))
                Acerola.Component.SyncActionIcon(
                    state = syncIconState,
                    defaultBackground =
                        if (isErrorIdle) MaterialTheme.colorScheme.errorContainer else MaterialTheme.colorScheme.primaryContainer,
                    modifier = Modifier.clickable(enabled = !anySyncing) { onAction(SyncAction.SyncAll(peer.peerId)) },
                ) {
                    Icon(
                        imageVector = if (isErrorIdle) Icons.Default.Error else Icons.Default.Sync,
                        contentDescription = stringResource(id = R.string.action_sync_all),
                        tint = if (isErrorIdle) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onPrimaryContainer,
                        modifier = Modifier.size(SizeTokens.IconMedium),
                    )
                }
            }
        },
        icon = {
            Box(modifier = Modifier.fillMaxSize()) {
                Icon(
                    imageVector = Icons.Default.PhoneAndroid,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.onPrimaryContainer,
                    modifier = Modifier.align(Alignment.Center).size(SizeTokens.IconMedium),
                )
                Box(
                    modifier =
                        Modifier
                            .align(Alignment.BottomEnd)
                            // O ícone fica dentro de uma Surface arredondada (ShapeTokens.Large,
                            // 16dp) que recorta o próprio conteúdo — encostada em (0, 0) do
                            // BottomEnd, a bolinha cai fora do arco do canto e é cortada. Esse
                            // deslocamento pra dentro é o suficiente pra ela ficar inteira dentro
                            // da curva.
                            .offset(x = (-4).dp, y = (-4).dp)
                            .size(12.dp)
                            .background(MaterialTheme.colorScheme.surface, CircleShape)
                            .padding(2.dp)
                            .background(
                                if (isOnline) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.outlineVariant,
                                CircleShape,
                            ),
                )
            }
        },
    )
}

@Composable
private fun AddDeviceSheetContent(
    uiState: SyncUiState,
    selectedTab: Int,
    onTabSelected: (Int) -> Unit,
    pasteValue: String,
    onPasteValueChange: (String) -> Unit,
    onConnect: () -> Unit,
    onScan: () -> Unit,
    onDismissError: () -> Unit,
) {
    Column(modifier = Modifier.padding(SpacingTokens.Large)) {
        Text(
            text = stringResource(id = R.string.action_sync_add_device),
            style = MaterialTheme.typography.titleMedium,
            fontWeight = FontWeight.Bold,
        )

        Spacer(modifier = Modifier.height(SpacingTokens.Medium))

        Acerola.Component.SegmentedControl(
            options = listOf(stringResource(id = R.string.tab_sync_my_code), stringResource(id = R.string.tab_sync_connect)),
            selectedIndex = selectedTab,
            onSelect = onTabSelected,
        )

        Spacer(modifier = Modifier.height(SpacingTokens.Large))

        when (selectedTab) {
            0 -> MyCodeTabContent(uiState = uiState)
            else ->
                ConnectTabContent(
                    uiState = uiState,
                    pasteValue = pasteValue,
                    onPasteValueChange = onPasteValueChange,
                    onConnect = onConnect,
                    onScan = onScan,
                    onDismissError = onDismissError,
                )
        }

        Spacer(modifier = Modifier.height(SpacingTokens.Large))
    }
}

@Composable
private fun MyCodeTabContent(uiState: SyncUiState) {
    val clipboardManager = LocalClipboardManager.current
    val scope = rememberCoroutineScope()
    val snackbarHostState = LocalSnackbarHostState.current
    val copiedMessage = stringResource(id = R.string.label_sync_copied)
    var showRawCode by remember { mutableStateOf(false) }

    Text(
        text = stringResource(id = R.string.description_sync_pairing),
        style = MaterialTheme.typography.bodySmall,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
    )

    Spacer(modifier = Modifier.height(SpacingTokens.Medium))

    Column(horizontalAlignment = Alignment.CenterHorizontally, modifier = Modifier.fillMaxWidth()) {
        val code = uiState.pairingCode
        if (code != null) {
            val bitmap = remember(code) { QrBitmapGenerator.generate(code) }
            Image(
                bitmap = bitmap.asImageBitmap(),
                contentDescription = stringResource(id = R.string.title_sync_pairing),
                modifier = Modifier.size(220.dp),
            )
        } else {
            Box(modifier = Modifier.size(220.dp), contentAlignment = Alignment.Center) {
                CircularProgressIndicator()
            }
        }

        Spacer(modifier = Modifier.height(SpacingTokens.Medium))

        OutlinedButton(
            onClick = {
                code?.let {
                    clipboardManager.setText(AnnotatedString(it))
                    scope.launch { snackbarHostState.showSnackbar(copiedMessage, SnackbarVariant.Success) }
                }
            },
            enabled = code != null,
        ) {
            Icon(imageVector = Icons.Default.ContentCopy, contentDescription = null, modifier = Modifier.size(SizeTokens.IconExtraSmall))
            Spacer(modifier = Modifier.width(SpacingTokens.ExtraSmall))
            Text(text = stringResource(id = R.string.action_sync_copy_code))
        }

        Spacer(modifier = Modifier.height(SpacingTokens.Small))

        TextButton(onClick = { showRawCode = !showRawCode }) {
            Text(
                text =
                    stringResource(
                        id = if (showRawCode) R.string.action_sync_hide_code else R.string.action_sync_show_code,
                    ),
                style = MaterialTheme.typography.labelSmall,
            )
            Icon(
                imageVector = if (showRawCode) Icons.Default.ExpandLess else Icons.Default.ExpandMore,
                contentDescription = null,
                modifier = Modifier.size(SizeTokens.IconExtraSmall),
            )
        }

        if (showRawCode && code != null) {
            Text(
                text = code,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .background(MaterialTheme.colorScheme.surfaceContainerHigh, ShapeTokens.Small)
                        .padding(SpacingTokens.Small),
            )
        }
    }

    Spacer(modifier = Modifier.height(SpacingTokens.Medium))

    SecurityNote()
}

@Composable
private fun ConnectTabContent(
    uiState: SyncUiState,
    pasteValue: String,
    onPasteValueChange: (String) -> Unit,
    onConnect: () -> Unit,
    onScan: () -> Unit,
    onDismissError: () -> Unit,
) {
    val clipboardManager = LocalClipboardManager.current

    Text(
        text = stringResource(id = R.string.title_sync_connect),
        style = MaterialTheme.typography.titleSmall,
        fontWeight = FontWeight.Bold,
    )

    Spacer(modifier = Modifier.height(SpacingTokens.Medium))

    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small)) {
        OutlinedTextField(
            value = pasteValue,
            onValueChange = {
                onPasteValueChange(it)
                if (uiState.connectError != null) onDismissError()
            },
            placeholder = { Text(text = stringResource(id = R.string.hint_sync_connect_code)) },
            singleLine = true,
            modifier = Modifier.weight(1f),
        )

        IconButton(onClick = onScan) {
            Icon(imageVector = Icons.Default.QrCodeScanner, contentDescription = stringResource(id = R.string.action_sync_scan_code))
        }
    }

    Spacer(modifier = Modifier.height(SpacingTokens.Small))

    TextButton(onClick = {
        val clipboardText = clipboardManager.getText()?.text.orEmpty()
        onPasteValueChange(clipboardText)
        if (uiState.connectError != null) onDismissError()
    }) {
        Icon(imageVector = Icons.Default.ContentPaste, contentDescription = null, modifier = Modifier.size(SizeTokens.IconExtraSmall))
        Spacer(modifier = Modifier.width(SpacingTokens.ExtraSmall))
        Text(text = stringResource(id = R.string.action_sync_paste), style = MaterialTheme.typography.labelSmall)
    }

    Spacer(modifier = Modifier.height(SpacingTokens.Small))

    Button(
        onClick = onConnect,
        enabled = pasteValue.isNotBlank() && !uiState.connecting,
        modifier = Modifier.fillMaxWidth(),
    ) {
        if (uiState.connecting) {
            CircularProgressIndicator(
                modifier = Modifier.size(SizeTokens.IconExtraSmall),
                strokeWidth = 2.dp,
                color = MaterialTheme.colorScheme.onPrimary,
            )
        } else {
            Text(text = stringResource(id = R.string.action_sync_connect))
        }
    }

    if (uiState.connectError != null) {
        Spacer(modifier = Modifier.height(SpacingTokens.Small))
        Text(
            text =
                stringResource(
                    id =
                        when (uiState.connectError) {
                            ConnectError.INVALID_CODE -> R.string.error_sync_connect_invalid_code
                            ConnectError.CONNECTION_FAILED -> R.string.error_sync_connect_failed
                        },
                ),
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.error,
        )
    }
}

@Composable
private fun SecurityNote() {
    Row(
        modifier =
            Modifier
                .fillMaxWidth()
                .background(MaterialTheme.colorScheme.tertiaryContainer.copy(alpha = 0.5f), ShapeTokens.Small)
                .padding(SpacingTokens.Small),
    ) {
        Icon(
            imageVector = Icons.Default.Info,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.tertiary,
            modifier = Modifier.size(SizeTokens.IconExtraSmall),
        )
        Spacer(modifier = Modifier.width(SpacingTokens.Small))
        Text(
            text = stringResource(id = R.string.description_sync_security_note),
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Composable
private fun ActivityLogCard(uiState: SyncUiState) {
    SectionCard(title = stringResource(id = R.string.title_sync_activity_log)) {
        if (uiState.transferLog.isEmpty()) {
            Text(
                text = stringResource(id = R.string.label_sync_activity_log_empty),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        } else {
            uiState.transferLog.forEach { entry ->
                LogRow(entry = entry)
            }
        }
    }
}

/** Turns a raw [TransferLogEntry] into display text — the only place doing that resolution,
 *  so [SyncViewModel] never needs an Android [android.content.Context]-flavored dependency
 *  just to pre-render a string. */
@Composable
private fun describeEntry(entry: TransferLogEntry): String =
    when ("${entry.kind}:${entry.status}") {
        "history:started" -> stringResource(id = R.string.log_sync_history_started)
        "history:complete" -> stringResource(id = R.string.log_sync_history_complete)
        "history:error" -> stringResource(id = R.string.log_sync_history_error, entry.message.orEmpty())
        "files:started" -> stringResource(id = R.string.log_sync_files_started)
        "files:progress" ->
            stringResource(id = R.string.log_sync_files_progress, entry.comicName.orEmpty(), entry.chapter.orEmpty())
        "files:chapterFailed" ->
            stringResource(
                id = R.string.log_sync_files_chapter_failed,
                entry.comicName.orEmpty(),
                entry.chapter.orEmpty(),
            )
        "files:error" -> stringResource(id = R.string.log_sync_files_error, entry.message.orEmpty())
        "files:complete" -> stringResource(id = R.string.log_sync_files_complete)
        else -> entry.message ?: entry.status
    }

@Composable
private fun LogRow(entry: TransferLogEntry) {
    Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.padding(vertical = SpacingTokens.ExtraSmall)) {
        when (entry.state) {
            LogState.IN_PROGRESS ->
                CircularProgressIndicator(modifier = Modifier.size(SizeTokens.IconExtraSmall), strokeWidth = 2.dp)
            LogState.SUCCESS ->
                Icon(
                    imageVector = Icons.Default.CheckCircle,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.primary,
                    modifier = Modifier.size(SizeTokens.IconExtraSmall),
                )
            LogState.ERROR ->
                Icon(
                    imageVector = Icons.Default.Error,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.error,
                    modifier = Modifier.size(SizeTokens.IconExtraSmall),
                )
        }
        Spacer(modifier = Modifier.width(SpacingTokens.Small))
        Text(text = describeEntry(entry), style = MaterialTheme.typography.bodySmall, modifier = Modifier.weight(1f))
        Spacer(modifier = Modifier.width(SpacingTokens.Small))
        Text(
            text = formatLogTimestamp(entry.timestamp),
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Composable
private fun ConfirmConnectDialog(
    peerId: String,
    onConfirm: () -> Unit,
    onCancel: () -> Unit,
) {
    Acerola.Component.Dialog(
        show = true,
        onDismiss = onCancel,
        title = stringResource(id = R.string.title_sync_confirm_connect),
        confirmButtonContent = {
            Acerola.Component.DialogButton(
                text = stringResource(id = R.string.action_sync_connect),
                onClick = onConfirm,
            )
        },
        dismissButtonContent = {
            Acerola.Component.DialogButton(
                text = stringResource(id = R.string.action_cancel),
                onClick = onCancel,
            )
        },
    ) {
        Text(text = stringResource(id = R.string.description_sync_confirm_connect, PairingCode.shortId(peerId)))
    }
}

@Composable
private fun TofuDialog(
    peerId: String,
    onDismiss: () -> Unit,
) {
    Acerola.Component.Dialog(
        show = true,
        onDismiss = onDismiss,
        title = stringResource(id = R.string.title_sync_trust_dialog),
        confirmButtonContent = {
            Acerola.Component.DialogButton(
                text = stringResource(id = R.string.action_confirm),
                onClick = onDismiss,
            )
        },
    ) {
        Text(text = stringResource(id = R.string.description_sync_trust_dialog, peerId))
    }
}

@Composable
private fun RemovePeerDialog(
    peer: PairedPeer,
    onConfirm: () -> Unit,
    onCancel: () -> Unit,
) {
    val peerLabel = peer.deviceName ?: PairingCode.shortId(peer.peerId)
    Acerola.Component.Dialog(
        show = true,
        onDismiss = onCancel,
        title = stringResource(id = R.string.title_sync_remove_peer_confirm, peerLabel),
        confirmButtonContent = {
            Acerola.Component.DialogButton(
                text = stringResource(id = R.string.action_sync_remove_peer),
                onClick = onConfirm,
            )
        },
        dismissButtonContent = {
            Acerola.Component.DialogButton(
                text = stringResource(id = R.string.action_cancel),
                onClick = onCancel,
            )
        },
    ) {
        Text(text = stringResource(id = R.string.description_sync_remove_peer_confirm))
    }
}

/** Same formatting used both in the activity log and in "last synced" per peer. */
private fun formatLogTimestamp(timestampMillis: Long): String = SimpleDateFormat("dd/MM HH:mm", Locale.getDefault()).format(Date(timestampMillis))

@Composable
private fun SectionHeader(title: String) {
    Text(
        text = title.uppercase(),
        style = MaterialTheme.typography.labelMedium,
        fontWeight = FontWeight.Bold,
        color = MaterialTheme.colorScheme.secondary,
    )
}

@Composable
private fun SectionCard(
    title: String,
    content: @Composable ColumnScope.() -> Unit,
) {
    Card(shape = ShapeTokens.Medium, modifier = Modifier.fillMaxWidth()) {
        Column(modifier = Modifier.padding(SpacingTokens.Large)) {
            SectionHeader(title = title)
            Spacer(modifier = Modifier.height(SpacingTokens.Small))
            content()
        }
    }
}

private fun previewUiState() =
    SyncUiState(
        localId = "z6MkfriRLZTX4GC93z2XFqEmaXbXPPnvVQpEEeCQBDGGeMSw",
        localDeviceName = "Pixel 8",
        pairingCode = "acerola1:eyJpIjoiZGVtbyJ9",
        mode = NetworkMode.LOCAL,
        relaySettings = RelaySettingsUiState(),
        pairedPeers =
            listOf(
                PairedPeer(peerId = "z6Mkabc123def456ghi789", deviceName = "Desktop-Vinicius"),
                PairedPeer(peerId = "z6Mkxyz789ghi012jkl345", deviceName = null),
                PairedPeer(peerId = "z6Mkerr000fail111peer2", deviceName = "Notebook-Trabalho"),
            ),
        syncingKeys = setOf(syncKey("z6Mkabc123def456ghi789", SYNC_KIND_HISTORY)),
        lastSyncedByPeer = mapOf("z6Mkxyz789ghi012jkl345" to System.currentTimeMillis()),
        connectedPeerIds = setOf("z6Mkabc123def456ghi789"),
        lastSyncResultByPeer =
            mapOf(
                "z6Mkerr000fail111peer2" to
                    SyncResult(
                        kind = SYNC_KIND_FILES,
                        state = LogState.ERROR,
                        message = "conexão perdida",
                        timestamp = System.currentTimeMillis(),
                    ),
            ),
        transferLog =
            listOf(
                TransferLogEntry(
                    id = 3,
                    kind = SYNC_KIND_FILES,
                    status = "progress",
                    state = LogState.IN_PROGRESS,
                    comicName = "Berserk",
                    chapter = "42",
                ),
                TransferLogEntry(id = 2, kind = SYNC_KIND_FILES, status = "error", state = LogState.ERROR, message = "connection reset"),
                TransferLogEntry(id = 1, kind = SYNC_KIND_HISTORY, status = "complete", state = LogState.SUCCESS),
            ),
    )

/** [SyncLayout] (e as telas que ele monta, como [MyCodeTabContent]) lê [LocalSnackbarHostState]
 *  incondicionalmente, que não tem default e lança se não for provido — todo preview precisa
 *  de um, mesmo os que nunca mostram de fato um snackbar. */
@Composable
private fun PreviewSyncLayout(uiState: SyncUiState) {
    AcerolaTheme {
        CompositionLocalProvider(LocalSnackbarHostState provides remember { SnackbarHostState() }) {
            SyncLayout(uiState = uiState, onAction = {}, onBack = {})
        }
    }
}

@Composable
private fun PreviewAddDeviceSheetContent(
    uiState: SyncUiState,
    selectedTab: Int,
) {
    AcerolaTheme {
        CompositionLocalProvider(LocalSnackbarHostState provides remember { SnackbarHostState() }) {
            Column(modifier = Modifier.width(360.dp)) {
                AddDeviceSheetContent(
                    uiState = uiState,
                    selectedTab = selectedTab,
                    onTabSelected = {},
                    pasteValue = "",
                    onPasteValueChange = {},
                    onConnect = {},
                    onScan = {},
                    onDismissError = {},
                )
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SyncLayoutPreview() {
    PreviewSyncLayout(uiState = previewUiState())
}

@Preview(name = "Empty state", showBackground = true)
@Composable
private fun SyncLayoutEmptyPreview() {
    PreviewSyncLayout(uiState = SyncUiState())
}

@Preview(name = "Add device - Meu código", showBackground = true)
@Composable
private fun AddDeviceSheetMyCodePreview() {
    PreviewAddDeviceSheetContent(uiState = previewUiState(), selectedTab = 0)
}

@Preview(name = "Add device - Conectar (erro)", showBackground = true)
@Composable
private fun AddDeviceSheetConnectErrorPreview() {
    PreviewAddDeviceSheetContent(uiState = previewUiState().copy(connectError = ConnectError.INVALID_CODE), selectedTab = 1)
}
