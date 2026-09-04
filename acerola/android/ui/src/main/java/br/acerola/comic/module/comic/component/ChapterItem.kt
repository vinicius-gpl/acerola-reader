package br.acerola.comic.module.comic.component

import android.content.res.Configuration
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.MenuBook
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.Dialog
import br.acerola.comic.common.ux.component.DialogButton
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.ShapeTokens
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.dto.archive.ChapterFileDto
import br.acerola.comic.module.comic.Comic
import br.acerola.comic.ui.R

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun Comic.Component.ChapterItem(
    chapterFileDto: ChapterFileDto,
    modifier: Modifier = Modifier,
    onToggleRead: () -> Unit = {},
    isRead: Boolean = false,
    hasConflict: Boolean = false,
    isSelected: Boolean = false,
    isSelectionMode: Boolean = false,
    onLongClick: () -> Unit = {},
    onClick: () -> Unit,
) {
    var showDetails by remember { mutableStateOf(value = false) }

    val mainTitle = stringResource(id = R.string.title_chapter_item_chapter_number, chapterFileDto.chapterSort)
    val subtitle = chapterFileDto.name

    val surfaceColor =
        when {
            hasConflict -> MaterialTheme.colorScheme.error.copy(alpha = 0.08f)
            isRead -> MaterialTheme.colorScheme.primary.copy(alpha = 0.08f)
            else -> Color.Transparent
        }

    Surface(
        color = surfaceColor,
        modifier =
            modifier
                .fillMaxWidth()
                .combinedClickable(
                    onClick = onClick,
                    onLongClick = onLongClick,
                ),
    ) {
        Row(
            modifier = Modifier.padding(horizontal = SpacingTokens.ExtraLarge, vertical = SpacingTokens.Medium),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            when {
                isSelectionMode && isSelected ->
                    ChapterLeadingIcon(
                        icon = Icons.Default.Check,
                        iconTint = MaterialTheme.colorScheme.onPrimary,
                        circleColor = MaterialTheme.colorScheme.primary,
                    )
                isSelectionMode ->
                    Box(
                        modifier =
                            Modifier
                                .size(SizeTokens.ClickTargetSmall)
                                .background(
                                    color = MaterialTheme.colorScheme.surfaceVariant,
                                    shape = CircleShape,
                                ).border(
                                    width = 2.dp,
                                    color = MaterialTheme.colorScheme.outline,
                                    shape = CircleShape,
                                ),
                    )
                // Mesma ideia do círculo de "lido" (ícone principal + cor diferente), mas com
                // preenchimento tintado (alpha) em vez de sólido: testei `onError` sobre `error`
                // sólido (o par que seria o equivalente direto de onPrimary/primary) e ele falha
                // contraste em dois temas — Dracula (2.95:1) e Alucard (1.01:1, praticamente sem
                // distinção). Tintado, o ícone continua na cor `error` pura (não `onError`), que
                // já passa em todos os temas contra o fundo por trás.
                hasConflict ->
                    ChapterLeadingIcon(
                        icon = Icons.Default.Warning,
                        iconTint = MaterialTheme.colorScheme.error,
                        circleColor = MaterialTheme.colorScheme.error.copy(alpha = 0.15f),
                    )
                isRead ->
                    ChapterLeadingIcon(
                        icon = Icons.Default.Check,
                        iconTint = MaterialTheme.colorScheme.onPrimary,
                        circleColor = MaterialTheme.colorScheme.primary,
                    )
                else ->
                    ChapterLeadingIcon(
                        icon = Icons.Default.MenuBook,
                        iconTint = MaterialTheme.colorScheme.primary,
                    )
            }

            Spacer(modifier = Modifier.width(SpacingTokens.Large))

            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = mainTitle,
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.Bold,
                    color = if (isRead) MaterialTheme.colorScheme.onSurfaceVariant else MaterialTheme.colorScheme.onSurface,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = subtitle,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }

            // Mesma estrutura do pill de "Lido" (só texto, sem ícone repetido — o ícone já
            // aparece no círculo principal à esquerda). `error` como texto falha 4.5:1 em
            // Nord (3.03~3.55:1), por isso o texto usa `onSurfaceVariant` (par já confiável
            // em todo o app) — só o círculo à esquerda carrega a cor de alerta.
            if (hasConflict) {
                Surface(
                    shape = ShapeTokens.Full,
                    color = MaterialTheme.colorScheme.surfaceVariant,
                ) {
                    Text(
                        text = stringResource(id = R.string.label_comic_status_conflict),
                        style = MaterialTheme.typography.labelSmall,
                        fontWeight = FontWeight.Black,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier =
                            Modifier.padding(
                                horizontal = SpacingTokens.Medium,
                                vertical = SpacingTokens.ExtraSmall,
                            ),
                    )
                }
                Spacer(modifier = Modifier.width(SpacingTokens.Small))
            }

            if (isRead) {
                Surface(
                    shape = ShapeTokens.Full,
                    color = MaterialTheme.colorScheme.primary.copy(alpha = 0.1f),
                ) {
                    Text(
                        text = stringResource(id = R.string.label_comic_status_read),
                        style = MaterialTheme.typography.labelSmall,
                        fontWeight = FontWeight.Black,
                        color = MaterialTheme.colorScheme.primary,
                        modifier =
                            Modifier.padding(
                                horizontal = SpacingTokens.Medium,
                                vertical = SpacingTokens.ExtraSmall,
                            ),
                    )
                }
            }

            IconButton(onClick = { if (isSelectionMode) onClick() else showDetails = true }) {
                Icon(
                    imageVector = Icons.Default.MoreVert,
                    contentDescription = stringResource(id = R.string.description_icon_chapter_more_options),
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }

    if (showDetails) {
        Acerola.Component.Dialog(
            show = true,
            title = mainTitle,
            onDismiss = { showDetails = false },
            confirmButtonContent = {
                Acerola.Component.DialogButton(
                    text = stringResource(id = R.string.label_dialog_close),
                    onClick = { showDetails = false },
                )
            },
            content = {
                Column(modifier = Modifier.fillMaxWidth()) {
                    DetailRow(
                        label = stringResource(id = R.string.label_chapter_detail_file),
                        value = chapterFileDto.name,
                    )

                    Spacer(modifier = Modifier.height(SpacingTokens.Large))
                    HorizontalDivider(color = MaterialTheme.colorScheme.outline.copy(alpha = 0.2f))
                    Spacer(modifier = Modifier.height(SpacingTokens.Small))

                    TextButton(
                        modifier = Modifier.fillMaxWidth(),
                        onClick = { onToggleRead() },
                        colors =
                            ButtonDefaults.textButtonColors(
                                contentColor = if (isRead) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.primary,
                            ),
                    ) {
                        Text(
                            text =
                                if (isRead) {
                                    stringResource(id = R.string.action_mark_as_unread)
                                } else {
                                    stringResource(id = R.string.action_mark_as_read)
                                },
                            fontWeight = FontWeight.SemiBold,
                        )
                    }
                }
            },
        )
    }
}

@Composable
private fun ChapterLeadingIcon(
    icon: ImageVector,
    iconTint: Color,
    circleColor: Color = Color.Transparent,
) {
    Box(
        modifier =
            Modifier
                .size(SizeTokens.ClickTargetSmall)
                .background(color = circleColor, shape = CircleShape),
        contentAlignment = Alignment.Center,
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = iconTint,
            modifier = Modifier.size(SizeTokens.IconSmall),
        )
    }
}

@Composable
private fun DetailRow(
    label: String,
    value: String,
) {
    if (value.isBlank()) return
    Column(modifier = Modifier.padding(bottom = SpacingTokens.Medium)) {
        Text(
            text = label,
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.primary,
            fontWeight = FontWeight.Bold,
        )
        Text(
            text = value,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurface,
        )
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ChapterItemPreview() {
    AcerolaTheme {
        Comic.Component.ChapterItem(
            chapterFileDto = ChapterFileDto(id = 1L, name = "Capítulo 1", path = "/path/1", chapterSort = "0001"),
            onClick = {},
        )
    }
}

@Preview(name = "Read - Light", showBackground = true)
@Preview(name = "Read - Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ChapterItemReadPreview() {
    AcerolaTheme {
        Comic.Component.ChapterItem(
            chapterFileDto = ChapterFileDto(id = 1L, name = "Capítulo 1", path = "/path/1", chapterSort = "0001"),
            isRead = true,
            onClick = {},
        )
    }
}

@Preview(name = "Conflict - Light", showBackground = true)
@Preview(name = "Conflict - Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ChapterItemConflictPreview() {
    AcerolaTheme {
        Comic.Component.ChapterItem(
            chapterFileDto = ChapterFileDto(id = 1L, name = "Capítulo 5 (conflito-peer1)", path = "/path/5", chapterSort = "0005"),
            hasConflict = true,
            onClick = {},
        )
    }
}
