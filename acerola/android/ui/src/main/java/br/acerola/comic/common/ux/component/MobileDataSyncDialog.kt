package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.ui.R

/**
 * Diálogo compartilhado por QUALQUER tela que dispare uma sincronização P2P enquanto o
 * dispositivo está em dados móveis (ver `MobileDataSyncGate`/`MobileDataSyncViewModel`) — montado
 * uma única vez na raiz do app (`br.acerola.comic.common.activity.BaseActivity`), não dentro de
 * uma tela específica, já que Home/Histórico/Quadrinho/Biblioteca remota/Sync podem disparar essa
 * confirmação cada uma pelo seu próprio caminho.
 */
@Composable
fun Acerola.Component.MobileDataSyncDialog(
    onConfirm: (remember: Boolean) -> Unit,
    onCancel: () -> Unit,
) {
    var rememberChoice by remember { mutableStateOf(false) }

    Acerola.Component.Dialog(
        show = true,
        onDismiss = onCancel,
        title = stringResource(id = R.string.title_mobile_data_sync_confirm),
        confirmButtonContent = {
            Acerola.Component.DialogButton(
                text = stringResource(id = R.string.action_mobile_data_sync_confirm),
                onClick = { onConfirm(rememberChoice) },
                contentColor = MaterialTheme.colorScheme.tertiary,
                fontWeight = FontWeight.Bold,
            )
        },
        dismissButtonContent = {
            Acerola.Component.DialogButton(
                text = stringResource(id = R.string.action_cancel),
                onClick = onCancel,
            )
        },
    ) {
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Text(text = stringResource(id = R.string.description_mobile_data_sync_confirm))

            Spacer(modifier = Modifier.height(SpacingTokens.Medium))

            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
            ) {
                Text(
                    text = stringResource(id = R.string.label_mobile_data_sync_remember),
                    style = MaterialTheme.typography.labelMedium,
                )
                Switch(checked = rememberChoice, onCheckedChange = { rememberChoice = it })
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun MobileDataSyncDialogPreview() {
    AcerolaTheme {
        Acerola.Component.MobileDataSyncDialog(onConfirm = {}, onCancel = {})
    }
}
