package br.acerola.comic.module.main.tutorial
import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.AutoAwesome
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.FolderZip
import androidx.compose.material.icons.filled.Layers
import androidx.compose.material.icons.filled.PictureAsPdf
import androidx.compose.material3.Button
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedCard
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.ShapeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.common.viewmodel.archive.FileSystemAccessViewModel
import br.acerola.comic.common.viewmodel.theme.ThemeViewModel
import br.acerola.comic.module.main.Main
import br.acerola.comic.module.main.config.component.SelectComicDirectory
import br.acerola.comic.module.main.config.component.ThemeSettings
import br.acerola.comic.module.main.tutorial.state.TutorialPage
import br.acerola.comic.ui.R
import kotlinx.coroutines.launch

@Composable
fun Main.Tutorial.Template.Screen(
    viewModel: TutorialViewModel = hiltViewModel(),
    themeViewModel: ThemeViewModel = hiltViewModel(),
    fileSystemAccessViewModel: FileSystemAccessViewModel = hiltViewModel(),
    onNavigateToHome: () -> Unit,
) {
    val pages = TutorialPage.entries
    val pagerState = rememberPagerState(pageCount = { pages.size })
    val scope = rememberCoroutineScope()

    val isFirstPage = pagerState.currentPage == 0
    val isLastPage = pagerState.currentPage == pages.lastIndex

    val folderName by fileSystemAccessViewModel.folderName.collectAsState()
    val canProceedSettings = !folderName.isNullOrEmpty()

    fun complete() {
        viewModel.markOnboardingCompleted()
        onNavigateToHome()
    }

    Column(modifier = Modifier.fillMaxSize()) {
        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(vertical = SpacingTokens.Large, horizontal = SpacingTokens.Large),
            horizontalArrangement = Arrangement.Center,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            pages.forEachIndexed { index, _ ->
                val isSelected = index == pagerState.currentPage
                val isPast = index < pagerState.currentPage

                val bgColor = if (isSelected || isPast) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.surfaceVariant
                val textColor = if (isSelected || isPast) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurfaceVariant

                Box(
                    modifier =
                        Modifier
                            .size(32.dp)
                            .clip(ShapeTokens.Full)
                            .background(bgColor),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(text = "${index + 1}", color = textColor, fontWeight = FontWeight.Bold)
                }

                if (index < pages.lastIndex) {
                    Box(
                        modifier =
                            Modifier
                                .width(24.dp)
                                .height(2.dp)
                                .background(
                                    if (isSelected ||
                                        isPast
                                    ) {
                                        MaterialTheme.colorScheme.primary
                                    } else {
                                        MaterialTheme.colorScheme.surfaceVariant
                                    },
                                ),
                    )
                }
            }
        }

        HorizontalPager(
            state = pagerState,
            modifier = Modifier.weight(1f),
            userScrollEnabled = false,
        ) { pageIndex ->
            when (pageIndex) {
                0 -> WelcomeSlide()
                1 -> FormatsSlide()
                2 -> SettingsSlide(themeViewModel, fileSystemAccessViewModel, folderName)
                3 -> CompleteSlide()
            }
        }

        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(horizontal = SpacingTokens.Large)
                    .padding(bottom = SpacingTokens.Large),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            if (!isFirstPage) {
                OutlinedButton(
                    onClick = {
                        scope.launch {
                            pagerState.animateScrollToPage(pagerState.currentPage - 1)
                        }
                    },
                ) {
                    Text(text = stringResource(id = R.string.tutorial_action_previous))
                }
            } else {
                Spacer(modifier = Modifier.weight(1f))
            }

            if (isLastPage) {
                Button(onClick = ::complete) {
                    Text(text = stringResource(id = R.string.tutorial_action_finish))
                }
            } else {
                Button(
                    onClick = {
                        scope.launch {
                            pagerState.animateScrollToPage(pagerState.currentPage + 1)
                        }
                    },
                    enabled = if (pagerState.currentPage == 2) canProceedSettings else true,
                ) {
                    Text(text = stringResource(id = R.string.tutorial_action_next))
                }
            }
        }
    }
}

@Composable
fun WelcomeSlide() {
    Column(
        modifier =
            Modifier
                .fillMaxSize()
                .padding(horizontal = SpacingTokens.Giant),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Text(
            text = "Acerola",
            style = MaterialTheme.typography.displayMedium,
            fontWeight = FontWeight.Bold,
            color = MaterialTheme.colorScheme.primary,
        )
        Spacer(modifier = Modifier.height(SpacingTokens.Medium))
        Text(
            text = stringResource(id = R.string.tutorial_desc_welcome),
            style = MaterialTheme.typography.bodyLarge,
            textAlign = TextAlign.Center,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Composable
fun FormatsSlide() {
    val scrollState = rememberScrollState()

    Column(
        modifier =
            Modifier
                .fillMaxSize()
                .verticalScroll(scrollState)
                .padding(horizontal = SpacingTokens.Large, vertical = SpacingTokens.Small),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Box(
            contentAlignment = Alignment.Center,
            modifier =
                Modifier
                    .size(56.dp)
                    .clip(ShapeTokens.Large)
                    .background(MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.6f)),
        ) {
            Icon(
                imageVector = Icons.Default.FolderZip,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.primary,
                modifier = Modifier.size(30.dp),
            )
        }

        Spacer(modifier = Modifier.height(SpacingTokens.Small))

        Text(
            text = stringResource(id = R.string.tutorial_formats_title),
            style = MaterialTheme.typography.headlineMedium,
            fontWeight = FontWeight.Bold,
            textAlign = TextAlign.Center,
            color = MaterialTheme.colorScheme.onSurface,
        )

        Spacer(modifier = Modifier.height(SpacingTokens.ExtraSmall))

        Text(
            text = stringResource(id = R.string.tutorial_formats_subtitle),
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
        )

        Spacer(modifier = Modifier.height(SpacingTokens.Medium))

        FormatCard(
            extension = "CBZ",
            name = stringResource(id = R.string.tutorial_formats_cbz_title),
            badge = stringResource(id = R.string.tutorial_formats_cbz_badge),
            description = stringResource(id = R.string.tutorial_formats_cbz_desc),
            features =
                listOf(
                    stringResource(id = R.string.tutorial_formats_cbz_feat_1),
                    stringResource(id = R.string.tutorial_formats_cbz_feat_2),
                ),
            icon = Icons.Default.FolderZip,
            isPrimary = true,
        )

        Spacer(modifier = Modifier.height(SpacingTokens.Small))

        FormatCard(
            extension = "CBR",
            name = stringResource(id = R.string.tutorial_formats_cbr_title),
            badge = stringResource(id = R.string.tutorial_formats_cbr_badge),
            description = stringResource(id = R.string.tutorial_formats_cbr_desc),
            features =
                listOf(
                    stringResource(id = R.string.tutorial_formats_cbr_feat_1),
                    stringResource(id = R.string.tutorial_formats_cbr_feat_2),
                ),
            icon = Icons.Default.Layers,
            isPrimary = false,
        )

        Spacer(modifier = Modifier.height(SpacingTokens.Small))

        FormatCard(
            extension = "PDF",
            name = stringResource(id = R.string.tutorial_formats_pdf_title),
            badge = stringResource(id = R.string.tutorial_formats_pdf_badge),
            description = stringResource(id = R.string.tutorial_formats_pdf_desc),
            features =
                listOf(
                    stringResource(id = R.string.tutorial_formats_pdf_feat_1),
                    stringResource(id = R.string.tutorial_formats_pdf_feat_2),
                ),
            icon = Icons.Default.PictureAsPdf,
            isPrimary = false,
        )

        Spacer(modifier = Modifier.height(SpacingTokens.Medium))

        OutlinedCard(
            modifier = Modifier.fillMaxWidth(),
            colors =
                CardDefaults.outlinedCardColors(
                    containerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.4f),
                ),
            shape = ShapeTokens.Medium,
        ) {
            Row(
                modifier = Modifier.padding(SpacingTokens.Medium),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Box(
                    contentAlignment = Alignment.Center,
                    modifier =
                        Modifier
                            .size(36.dp)
                            .clip(ShapeTokens.Medium)
                            .background(MaterialTheme.colorScheme.primary.copy(alpha = 0.12f)),
                ) {
                    Icon(
                        imageVector = Icons.Default.AutoAwesome,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.primary,
                        modifier = Modifier.size(18.dp),
                    )
                }

                Spacer(modifier = Modifier.width(SpacingTokens.Medium))

                val annotatedText =
                    buildAnnotatedString {
                        withStyle(style = SpanStyle(fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurface)) {
                            append(stringResource(id = R.string.tutorial_formats_sync_note_title))
                        }
                        append(" ")
                        append(stringResource(id = R.string.tutorial_formats_sync_note_content))
                    }

                Text(
                    text = annotatedText,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    lineHeight = MaterialTheme.typography.bodySmall.lineHeight,
                )
            }
        }
    }
}

@Composable
private fun FormatCard(
    extension: String,
    name: String,
    badge: String,
    description: String,
    features: List<String>,
    icon: ImageVector,
    isPrimary: Boolean,
) {
    val borderColor = if (isPrimary) MaterialTheme.colorScheme.primary.copy(alpha = 0.5f) else MaterialTheme.colorScheme.outlineVariant
    val containerBg =
        if (isPrimary) {
            MaterialTheme.colorScheme.primaryContainer.copy(
                alpha = 0.15f,
            )
        } else {
            MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.3f)
        }
    val badgeBg =
        if (isPrimary) {
            MaterialTheme.colorScheme.primary.copy(
                alpha = 0.15f,
            )
        } else {
            MaterialTheme.colorScheme.secondaryContainer.copy(alpha = 0.5f)
        }
    val badgeTextColor = if (isPrimary) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSecondaryContainer

    OutlinedCard(
        modifier = Modifier.fillMaxWidth(),
        shape = ShapeTokens.Medium,
        colors = CardDefaults.outlinedCardColors(containerColor = containerBg),
        border = androidx.compose.foundation.BorderStroke(1.dp, borderColor),
    ) {
        Column(modifier = Modifier.padding(SpacingTokens.Medium)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Box(
                    contentAlignment = Alignment.Center,
                    modifier =
                        Modifier
                            .size(40.dp)
                            .clip(ShapeTokens.Medium)
                            .background(badgeBg),
                ) {
                    Icon(
                        imageVector = icon,
                        contentDescription = null,
                        tint = badgeTextColor,
                        modifier = Modifier.size(20.dp),
                    )
                }

                Surface(
                    shape = ShapeTokens.Full,
                    color = badgeBg,
                    border = androidx.compose.foundation.BorderStroke(1.dp, badgeTextColor.copy(alpha = 0.3f)),
                ) {
                    Text(
                        text = badge.uppercase(),
                        style = MaterialTheme.typography.labelSmall,
                        fontWeight = FontWeight.Bold,
                        color = badgeTextColor,
                        modifier = Modifier.padding(horizontal = SpacingTokens.Small, vertical = 2.dp),
                    )
                }
            }

            Spacer(modifier = Modifier.height(SpacingTokens.Small))

            Text(
                text = extension,
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.Black,
                color = MaterialTheme.colorScheme.onSurface,
            )
            Text(
                text = name.uppercase(),
                style = MaterialTheme.typography.labelSmall,
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )

            Spacer(modifier = Modifier.height(SpacingTokens.ExtraSmall))

            Text(
                text = description,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )

            Spacer(modifier = Modifier.height(SpacingTokens.Small))

            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.5f))

            Spacer(modifier = Modifier.height(SpacingTokens.ExtraSmall))

            features.forEach { feature ->
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier.padding(vertical = 2.dp),
                ) {
                    Icon(
                        imageVector = Icons.Default.CheckCircle,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.primary,
                        modifier = Modifier.size(14.dp),
                    )
                    Spacer(modifier = Modifier.width(SpacingTokens.ExtraSmall))
                    Text(
                        text = feature,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurface,
                    )
                }
            }
        }
    }
}

@Composable
fun SettingsSlide(
    themeViewModel: ThemeViewModel,
    fileSystemAccessViewModel: FileSystemAccessViewModel,
    folderName: String?,
) {
    val selectedTheme by themeViewModel.currentTheme.collectAsState()

    Column(
        modifier =
            Modifier
                .fillMaxSize()
                .padding(horizontal = SpacingTokens.Large),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Text(
            text = stringResource(id = R.string.tutorial_title_settings),
            style = MaterialTheme.typography.headlineMedium,
            textAlign = TextAlign.Center,
            color = MaterialTheme.colorScheme.onSurface,
        )
        Spacer(modifier = Modifier.height(SpacingTokens.Large))

        Main.Config.Component.ThemeSettings(
            currentTheme = selectedTheme,
            onThemeChange = { themeViewModel.setTheme(it) },
        )

        Spacer(modifier = Modifier.height(SpacingTokens.Large))

        Main.Config.Component.SelectComicDirectory(
            folderName = folderName,
            onFolderSelected = { fileSystemAccessViewModel.saveFolderUri(it) },
            modifier = Modifier.fillMaxWidth(),
        )
    }
}

@Composable
fun CompleteSlide() {
    Column(
        modifier =
            Modifier
                .fillMaxSize()
                .padding(horizontal = SpacingTokens.Giant),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Icon(
            imageVector = Icons.Default.CheckCircle,
            contentDescription = null,
            modifier = Modifier.size(64.dp),
            tint = MaterialTheme.colorScheme.primary,
        )
        Spacer(modifier = Modifier.height(SpacingTokens.Large))
        Text(
            text = stringResource(id = R.string.tutorial_title_complete),
            style = MaterialTheme.typography.headlineMedium,
            textAlign = TextAlign.Center,
            color = MaterialTheme.colorScheme.onSurface,
        )
        Spacer(modifier = Modifier.height(SpacingTokens.Medium))
        Text(
            text = stringResource(id = R.string.tutorial_desc_complete),
            style = MaterialTheme.typography.bodyLarge,
            textAlign = TextAlign.Center,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(modifier = Modifier.height(SpacingTokens.Large))
        Text(
            text = stringResource(id = R.string.tutorial_note_complete),
            style = MaterialTheme.typography.bodySmall,
            textAlign = TextAlign.Center,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ScreenPreview() {
    AcerolaTheme {
        WelcomeSlide()
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun WelcomeSlidePreview() {
    AcerolaTheme {
        WelcomeSlide()
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun FormatsSlidePreview() {
    AcerolaTheme {
        FormatsSlide()
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun CompleteSlidePreview() {
    AcerolaTheme {
        CompleteSlide()
    }
}
