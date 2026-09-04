package br.acerola.comic.module.main.common.component

import android.content.res.Configuration
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Bookmark
import androidx.compose.material.icons.rounded.BookmarkBorder
import androidx.compose.material.icons.rounded.CloudDownload
import androidx.compose.material.icons.rounded.CloudUpload
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.LayersClear
import androidx.compose.material.icons.rounded.Visibility
import androidx.compose.material.icons.rounded.VisibilityOff
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.ListItem
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Text
import androidx.compose.material3.TriStateCheckbox
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.state.ToggleableState
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.AdaptiveSheet
import br.acerola.comic.common.ux.component.Dialog
import br.acerola.comic.common.ux.component.DialogButton
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.dto.ComicDto
import br.acerola.comic.dto.archive.ComicDirectoryDto
import br.acerola.comic.dto.metadata.category.CategoryDto
import br.acerola.comic.module.main.Main
import br.acerola.comic.module.main.sync.state.PairedPeer
import br.acerola.comic.service.SyncDirection
import br.acerola.comic.ui.R
import coil.compose.AsyncImage
import coil.request.ImageRequest

@Composable
fun Main.Common.Component.ComicActionsSheet(
    comic: ComicDto,
    categories: List<CategoryDto>,
    onHide: () -> Unit,
    onDelete: () -> Unit,
    onClearMetadata: () -> Unit,
    onBookmark: (categoryId: Long?) -> Unit,
    onDismiss: () -> Unit,
    pairedPeers: List<PairedPeer> = emptyList(),
    onLoadPairedPeers: () -> Unit = {},
    onSyncWithPeer: (peerId: String, direction: SyncDirection) -> Unit = { _, _ -> },
) {
    var showCategorySheet by remember { mutableStateOf(false) }
    var showHideDialog by remember { mutableStateOf(false) }
    var showDeleteDialog by remember { mutableStateOf(false) }
    var showClearMetadataDialog by remember { mutableStateOf(false) }
    var showPeerPicker by remember { mutableStateOf(false) }
    var pendingDirection by remember { mutableStateOf(SyncDirection.PUSH) }

    val title = comic.remoteInfo?.title ?: comic.directory.name
    val currentCategoryName = comic.category?.name

    val context = LocalContext.current
    val coverUri = comic.directory.coverUri ?: comic.directory.bannerUri

    Acerola.Component.AdaptiveSheet(
        onDismissRequest = onDismiss,
    ) {
        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(horizontal = SpacingTokens.ExtraLarge, vertical = SpacingTokens.Medium),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            AsyncImage(
                model =
                    ImageRequest
                        .Builder(context)
                        .data(data = coverUri)
                        .memoryCacheKey("${coverUri}_${comic.directory.lastModified}")
                        .diskCacheKey("${coverUri}_${comic.directory.lastModified}")
                        .build(),
                contentDescription = null,
                modifier =
                    Modifier
                        .width(56.dp)
                        .height(84.dp),
            )

            Spacer(modifier = Modifier.width(SpacingTokens.Large))

            Column {
                Text(
                    text = title,
                    style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.Bold),
                    color = MaterialTheme.colorScheme.onSurface,
                    maxLines = 2,
                )
                if (currentCategoryName != null) {
                    Text(
                        text = currentCategoryName,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        }

        HorizontalDivider()

        ListItem(
            leadingContent = {
                Icon(
                    imageVector = if (comic.category != null) Icons.Rounded.Bookmark else Icons.Rounded.BookmarkBorder,
                    contentDescription = null,
                )
            },
            headlineContent = { Text(text = stringResource(id = R.string.action_bookmark)) },
            supportingContent = {
                Text(
                    text = currentCategoryName ?: stringResource(id = R.string.label_no_bookmark),
                )
            },
            modifier = Modifier.clickable { showCategorySheet = true },
        )

        ListItem(
            leadingContent = {
                Icon(
                    imageVector = if (comic.directory.hidden) Icons.Rounded.Visibility else Icons.Rounded.VisibilityOff,
                    contentDescription = null,
                )
            },
            headlineContent = {
                Text(
                    text =
                        stringResource(
                            id = if (comic.directory.hidden) R.string.action_unhide else R.string.action_hide,
                        ),
                )
            },
            supportingContent = {
                Text(
                    text =
                        stringResource(
                            id = if (comic.directory.hidden) R.string.description_unhide else R.string.description_hide,
                        ),
                )
            },
            modifier = Modifier.clickable { showHideDialog = true },
        )

        ListItem(
            leadingContent = {
                Icon(imageVector = Icons.Rounded.CloudUpload, contentDescription = null)
            },
            headlineContent = { Text(text = stringResource(id = R.string.action_sync_comic_push)) },
            supportingContent = { Text(text = stringResource(id = R.string.description_sync_comic_with_peer)) },
            modifier =
                Modifier.clickable {
                    onLoadPairedPeers()
                    pendingDirection = SyncDirection.PUSH
                    showPeerPicker = true
                },
        )

        ListItem(
            leadingContent = {
                Icon(imageVector = Icons.Rounded.CloudDownload, contentDescription = null)
            },
            headlineContent = { Text(text = stringResource(id = R.string.action_sync_comic_pull)) },
            supportingContent = { Text(text = stringResource(id = R.string.description_sync_comic_with_peer)) },
            modifier =
                Modifier.clickable {
                    onLoadPairedPeers()
                    pendingDirection = SyncDirection.PULL
                    showPeerPicker = true
                },
        )

        ListItem(
            leadingContent = {
                Icon(imageVector = Icons.Rounded.LayersClear, contentDescription = null)
            },
            headlineContent = { Text(text = stringResource(id = R.string.action_clear_metadata)) },
            supportingContent = { Text(text = stringResource(id = R.string.description_clear_metadata)) },
            modifier = Modifier.clickable { showClearMetadataDialog = true },
        )

        ListItem(
            leadingContent = {
                Icon(
                    imageVector = Icons.Rounded.Delete,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.error,
                )
            },
            headlineContent = {
                Text(
                    text = stringResource(id = R.string.action_delete),
                    color = MaterialTheme.colorScheme.error,
                )
            },
            supportingContent = {
                Text(
                    text = stringResource(id = R.string.description_delete),
                    color = MaterialTheme.colorScheme.error,
                )
            },
            modifier = Modifier.clickable { showDeleteDialog = true },
        )

        Spacer(modifier = Modifier.navigationBarsPadding())
    }

    if (showCategorySheet) {
        ComicCategorySheet(
            categories = categories,
            selectedCategoryId = comic.category?.id,
            onSelect = { categoryId ->
                onBookmark(categoryId)
                showCategorySheet = false
                onDismiss()
            },
            onDismiss = { showCategorySheet = false },
        )
    }

    if (showHideDialog) {
        Acerola.Component.Dialog(
            show = true,
            onDismiss = { showHideDialog = false },
            title =
                stringResource(
                    id = if (comic.directory.hidden) R.string.dialog_unhide_title else R.string.dialog_hide_title,
                ),
            confirmButtonContent = {
                Acerola.Component.DialogButton(
                    text =
                        stringResource(
                            id = if (comic.directory.hidden) R.string.action_unhide else R.string.action_hide,
                        ),
                    onClick = {
                        showHideDialog = false
                        onHide()
                        onDismiss()
                    },
                    containerColor = MaterialTheme.colorScheme.primary,
                    contentColor = MaterialTheme.colorScheme.onPrimary,
                    fontWeight = FontWeight.Bold,
                )
            },
            dismissButtonContent = {
                Acerola.Component.DialogButton(
                    text = stringResource(id = R.string.action_cancel),
                    onClick = { showHideDialog = false },
                    contentColor = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            },
            content = {
                Text(
                    text =
                        stringResource(
                            id = if (comic.directory.hidden) R.string.dialog_unhide_message else R.string.dialog_hide_message,
                        ),
                )
            },
        )
    }

    if (showDeleteDialog) {
        Acerola.Component.Dialog(
            show = true,
            onDismiss = { showDeleteDialog = false },
            title = stringResource(id = R.string.dialog_delete_title),
            confirmButtonContent = {
                Acerola.Component.DialogButton(
                    text = stringResource(id = R.string.action_delete),
                    onClick = {
                        onDelete()
                        showDeleteDialog = false
                        onDismiss()
                    },
                    containerColor = MaterialTheme.colorScheme.error,
                    contentColor = MaterialTheme.colorScheme.onError,
                    fontWeight = FontWeight.Bold,
                )
            },
            dismissButtonContent = {
                Acerola.Component.DialogButton(
                    text = stringResource(id = R.string.action_cancel),
                    onClick = { showDeleteDialog = false },
                    contentColor = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            },
            content = { Text(text = stringResource(id = R.string.dialog_delete_message)) },
        )
    }

    if (showClearMetadataDialog) {
        Acerola.Component.Dialog(
            show = true,
            onDismiss = { showClearMetadataDialog = false },
            title = stringResource(id = R.string.dialog_clear_metadata_title),
            confirmButtonContent = {
                Acerola.Component.DialogButton(
                    text = stringResource(id = R.string.action_clear_metadata),
                    onClick = {
                        onClearMetadata()
                        showClearMetadataDialog = false
                        onDismiss()
                    },
                    containerColor = MaterialTheme.colorScheme.error,
                    contentColor = MaterialTheme.colorScheme.onError,
                    fontWeight = FontWeight.Bold,
                )
            },
            dismissButtonContent = {
                Acerola.Component.DialogButton(
                    text = stringResource(id = R.string.action_cancel),
                    onClick = { showClearMetadataDialog = false },
                    contentColor = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            },
            content = { Text(text = stringResource(id = R.string.dialog_clear_metadata_message)) },
        )
    }

    if (showPeerPicker) {
        Main.Common.Component.PeerPickerSheet(
            peers = pairedPeers,
            onSelect = { peerId ->
                showPeerPicker = false
                onSyncWithPeer(peerId, pendingDirection)
                onDismiss()
            },
            onDismiss = { showPeerPicker = false },
        )
    }
}

@Composable
fun Main.Common.Component.ComicCategorySheet(
    categories: List<CategoryDto>,
    selectedCategoryId: Long?,
    onSelect: (categoryId: Long?) -> Unit,
    onDismiss: () -> Unit,
) {
    Acerola.Component.AdaptiveSheet(
        onDismissRequest = onDismiss,
        isScrollable = false,
    ) {
        Text(
            text = stringResource(id = R.string.action_bookmark),
            style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.Bold),
            modifier = Modifier.padding(horizontal = SpacingTokens.ExtraLarge, vertical = SpacingTokens.Medium),
        )

        HorizontalDivider()

        LazyColumn {
            item {
                ListItem(
                    leadingContent = {
                        RadioButton(
                            selected = selectedCategoryId == null,
                            onClick = { onSelect(null) },
                        )
                    },
                    headlineContent = { Text(text = stringResource(id = R.string.action_remove_bookmark)) },
                    modifier = Modifier.clickable { onSelect(null) },
                )
            }

            items(items = categories) { category ->
                ListItem(
                    leadingContent = {
                        RadioButton(
                            selected = selectedCategoryId == category.id,
                            onClick = { onSelect(category.id) },
                        )
                    },
                    headlineContent = { Text(text = category.name) },
                    trailingContent = {
                        Spacer(
                            modifier =
                                Modifier
                                    .size(SizeTokens.IconSmall)
                                    .drawBehind {
                                        drawCircle(color = Color(category.color))
                                    },
                        )
                    },
                    modifier = Modifier.clickable { onSelect(category.id) },
                )
            }
        }

        Spacer(modifier = Modifier.navigationBarsPadding())
    }
}

@Composable
fun Main.Common.Component.BatchComicCategorySheet(
    categories: List<CategoryDto>,
    categoryCounts: Map<Long, Int>,
    totalSelectedCount: Int,
    onSelectCategory: (categoryId: Long) -> Unit,
    onRemoveCategory: () -> Unit,
    onDismiss: () -> Unit,
) {
    Acerola.Component.AdaptiveSheet(
        onDismissRequest = onDismiss,
        isScrollable = false,
    ) {
        Text(
            text = stringResource(id = R.string.action_bookmark),
            style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.Bold),
            modifier = Modifier.padding(horizontal = SpacingTokens.ExtraLarge, vertical = SpacingTokens.Medium),
        )

        HorizontalDivider()

        LazyColumn {
            item {
                ListItem(
                    leadingContent = {
                        Icon(
                            imageVector = Icons.Rounded.BookmarkBorder,
                            contentDescription = null,
                        )
                    },
                    headlineContent = { Text(text = stringResource(id = R.string.action_remove_bookmark)) },
                    modifier =
                        Modifier.clickable {
                            onRemoveCategory()
                        },
                )
            }

            items(items = categories) { category ->
                val count = categoryCounts[category.id] ?: 0
                val state =
                    when {
                        totalSelectedCount > 0 && count == totalSelectedCount -> ToggleableState.On
                        count > 0 -> ToggleableState.Indeterminate
                        else -> ToggleableState.Off
                    }

                val stateDescription =
                    when (state) {
                        ToggleableState.On -> stringResource(id = R.string.description_batch_category_all)
                        ToggleableState.Indeterminate -> stringResource(id = R.string.description_batch_category_mixed)
                        ToggleableState.Off -> stringResource(id = R.string.description_batch_category_none)
                    }

                ListItem(
                    leadingContent = {
                        TriStateCheckbox(
                            state = state,
                            onClick = {
                                if (state == ToggleableState.On) {
                                    onRemoveCategory()
                                } else {
                                    onSelectCategory(category.id)
                                }
                            },
                        )
                    },
                    headlineContent = { Text(text = category.name) },
                    supportingContent = {
                        if (state == ToggleableState.Indeterminate) {
                            Text(
                                text = stringResource(id = R.string.description_batch_category_mixed),
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                    },
                    trailingContent = {
                        Spacer(
                            modifier =
                                Modifier
                                    .size(SizeTokens.IconSmall)
                                    .drawBehind {
                                        drawCircle(color = Color(category.color))
                                    },
                        )
                    },
                    modifier =
                        Modifier
                            .semantics {
                                this.stateDescription = stateDescription
                            }.clickable {
                                if (state == ToggleableState.On) {
                                    onRemoveCategory()
                                } else {
                                    onSelectCategory(category.id)
                                }
                            },
                )
            }
        }

        Spacer(modifier = Modifier.navigationBarsPadding())
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ComicActionsSheetPreview() {
    AcerolaTheme {
        Main.Common.Component.ComicActionsSheet(
            comic =
                ComicDto(
                    directory =
                        ComicDirectoryDto(
                            id = 1L,
                            name = "Sample Comic",
                            path = "/path",
                            coverUri = null,
                            bannerUri = null,
                            lastModified = 0L,
                            archiveTemplateFk = null,
                        ),
                    category = null,
                    remoteInfo = null,
                ),
            categories = emptyList(),
            onHide = {},
            onDelete = {},
            onClearMetadata = {},
            onBookmark = {},
            onDismiss = {},
        )
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ComicCategorySheetPreview() {
    AcerolaTheme {
        Main.Common.Component.ComicCategorySheet(
            categories = emptyList(),
            selectedCategoryId = null,
            onSelect = {},
            onDismiss = {},
        )
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun BatchComicCategorySheetPreview() {
    AcerolaTheme {
        Main.Common.Component.BatchComicCategorySheet(
            categories = emptyList(),
            categoryCounts = emptyMap(),
            totalSelectedCount = 0,
            onSelectCategory = {},
            onRemoveCategory = {},
            onDismiss = {},
        )
    }
}
