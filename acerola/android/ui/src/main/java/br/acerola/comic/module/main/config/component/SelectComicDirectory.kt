package br.acerola.comic.module.main.config.component

import android.content.res.Configuration
import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowForward
import androidx.compose.material.icons.filled.Folder
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.HeroButton
import br.acerola.comic.common.ux.component.IconButton
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.ShapeTokens
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.module.main.Main
import br.acerola.comic.ui.R

@Composable
fun Main.Config.Component.SelectComicDirectory(
    folderName: String?,
    onFolderSelected: (Uri?) -> Unit,
    modifier: Modifier = Modifier,
) {
    val launcher =
        rememberLauncherForActivityResult(
            contract = ActivityResultContracts.OpenDocumentTree(),
            onResult = { uri ->
                onFolderSelected(uri)
            },
        )

    val description =
        if (folderName != null) {
            stringResource(id = R.string.description_text_selected_comic_directory, folderName)
        } else {
            stringResource(id = R.string.description_text_config_select_path_comic)
        }

    Acerola.Component.HeroButton(
        title = stringResource(id = R.string.title_text_config_select_path_comic),
        description = description,
        icon = Icons.Filled.Folder,
        iconTint = MaterialTheme.colorScheme.onPrimaryContainer,
        iconBackground = MaterialTheme.colorScheme.primaryContainer,
        modifier = modifier,
        onClick = { launcher.launch(input = null) },
        action = {
            Acerola.Component.IconButton(
                onClick = { launcher.launch(input = null) },
                icon = {
                    Box(
                        contentAlignment = Alignment.Center,
                        modifier =
                            Modifier
                                .size(size = SpacingTokens.Giant)
                                .clip(ShapeTokens.Full)
                                .background(color = MaterialTheme.colorScheme.primary),
                    ) {
                        Icon(
                            imageVector = Icons.AutoMirrored.Filled.ArrowForward,
                            contentDescription = stringResource(R.string.description_icon_select_folder_comics),
                            tint = MaterialTheme.colorScheme.onPrimary,
                            modifier =
                                Modifier
                                    .size(size = SizeTokens.IconLarge)
                                    .padding(all = SpacingTokens.ExtraSmall),
                        )
                    }
                },
            )
        },
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SelectComicDirectoryPreview() {
    AcerolaTheme {
        Main.Config.Component.SelectComicDirectory(
            folderName = "Comics",
            onFolderSelected = {},
        )
    }
}
