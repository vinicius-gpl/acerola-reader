package br.acerola.comic.module.main.history.component

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
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.sp
import br.acerola.comic.common.ux.tokens.ShapeTokens
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.dto.ComicDto
import br.acerola.comic.module.main.Main
import br.acerola.comic.ui.R
import coil.compose.AsyncImage
import coil.request.ImageRequest
import androidx.compose.ui.tooling.preview.Preview
import android.content.res.Configuration
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.dto.archive.ComicDirectoryDto

@Composable
fun Main.History.Component.HistoryHeroCard(
    comic: ComicDto,
    onClick: () -> Unit,
    onContinueClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val context = LocalContext.current
    val bannerUri = comic.directory.bannerUri ?: comic.directory.coverUri ?: comic.remoteInfo?.banner?.url
    val title = comic.remoteInfo?.title ?: comic.directory.name

    Card(
        onClick = onClick,
        shape = ShapeTokens.Huge,
        modifier =
            modifier
                .fillMaxWidth()
                .height(SizeTokens.HistoryHeroHeight),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant),
    ) {
        Box(modifier = Modifier.fillMaxSize()) {
            AsyncImage(
                model =
                    ImageRequest
                        .Builder(context)
                        .data(bannerUri)
                        .memoryCacheKey("${bannerUri}_${comic.directory.lastModified}")
                        .diskCacheKey("${bannerUri}_${comic.directory.lastModified}")
                        .crossfade(true)
                        .build(),
                contentDescription = null,
                contentScale = ContentScale.Crop,
                modifier = Modifier.fillMaxSize(),
            )

            Box(
                modifier =
                    Modifier
                        .fillMaxSize()
                        .background(
                            Brush.verticalGradient(
                                colors =
                                    listOf(
                                        Color.Transparent,
                                        Color.Black.copy(alpha = 0.4f),
                                        Color.Black.copy(alpha = 0.9f),
                                    ),
                                startY = 0f,
                            ),
                        ),
            )

            Column(
                modifier =
                    Modifier
                        .fillMaxSize()
                        .padding(SpacingTokens.ExtraLarge),
                verticalArrangement = Arrangement.Bottom,
            ) {
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
                ) {
                    Box(
                        modifier =
                            Modifier
                                .background(
                                    color = Color.White.copy(alpha = 0.2f),
                                    shape = ShapeTokens.Large,
                                ).padding(horizontal = SpacingTokens.Small, vertical = SpacingTokens.ExtraSmall),
                    ) {
                        Text(
                            text = stringResource(id = R.string.label_history_hero_most_recent),
                            style =
                                MaterialTheme.typography.labelSmall.copy(
                                    fontWeight = FontWeight.Bold,
                                    letterSpacing = 1.sp,
                                ),
                            color = Color.White,
                        )
                    }
                }

                Spacer(modifier = Modifier.height(SpacingTokens.Medium))

                Text(
                    text = title,
                    style =
                        MaterialTheme.typography.headlineSmall.copy(
                            fontWeight = FontWeight.ExtraBold,
                            color = Color.White,
                        ),
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )

                Spacer(modifier = Modifier.height(SpacingTokens.ExtraLarge))

                Button(
                    onClick = onContinueClick,
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .height(SizeTokens.FabDefault),
                    shape = ShapeTokens.Large,
                    colors =
                        ButtonDefaults.buttonColors(
                            containerColor = MaterialTheme.colorScheme.primary,
                            contentColor = MaterialTheme.colorScheme.onPrimary,
                        ),
                ) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.Center,
                    ) {
                        Text(
                            text = stringResource(id = R.string.label_history_hero_continue),
                            style =
                                MaterialTheme.typography.labelLarge.copy(
                                    fontWeight = FontWeight.ExtraBold,
                                    letterSpacing = 1.2.sp,
                                ),
                        )
                        Spacer(modifier = Modifier.width(SpacingTokens.Small))
                        Icon(
                            imageVector = Icons.Filled.PlayArrow,
                            contentDescription = null,
                            modifier = Modifier.size(SizeTokens.IconSmall),
                        )
                    }
                }
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun HistoryHeroCardPreview() {
    AcerolaTheme {
        Main.History.Component.HistoryHeroCard(
            comic = ComicDto(directory = ComicDirectoryDto(id = 1L, name = "Sample Comic", path = "/path", coverUri = null, bannerUri = null, lastModified = 0L, archiveTemplateFk = null), category = null, remoteInfo = null),
            onClick = {},
            onContinueClick = {},
        )
    }
}
