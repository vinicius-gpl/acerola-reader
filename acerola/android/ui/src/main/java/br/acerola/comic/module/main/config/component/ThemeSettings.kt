package br.acerola.comic.module.main.config.component

import android.content.Context
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.isSystemInDarkTheme
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
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedCard
import androidx.compose.material3.Text
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import br.acerola.comic.common.ux.theme.color.CatppuccinLatte
import br.acerola.comic.common.ux.theme.color.CatppuccinMocha
import br.acerola.comic.common.ux.theme.color.Dracula
import br.acerola.comic.common.ux.theme.color.NordDark
import br.acerola.comic.common.ux.theme.color.TokyoNightDark
import br.acerola.comic.common.ux.theme.color.TokyoNightDay
import br.acerola.comic.common.ux.tokens.ShapeTokens
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.config.preference.types.AppTheme
import br.acerola.comic.module.main.Main
import br.acerola.comic.ui.R
import androidx.compose.ui.tooling.preview.Preview
import android.content.res.Configuration
import br.acerola.comic.common.ux.theme.AcerolaTheme

// Sem cabeçalho próprio: quando aninhado no Acerola.Component.AccordionCard da tela de
// config, o título/descrição já vêm do card — duplicar aqui repetiria o mesmo texto.
@Composable
fun Main.Config.Component.ThemeSettings(
    currentTheme: AppTheme,
    onThemeChange: (AppTheme) -> Unit,
) {
    val context = LocalContext.current
    val isDark = isSystemInDarkTheme()
    val themes = AppTheme.entries

    LazyRow(
        modifier = Modifier.fillMaxWidth(),
        contentPadding = PaddingValues(horizontal = SpacingTokens.Large, vertical = SpacingTokens.Small),
        horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Medium),
    ) {
        items(themes) { theme ->
            ThemeCard(
                modifier = Modifier.width(150.dp),
                title = getThemeTitle(theme, isDark),
                subtitle = getThemeSubtitle(theme, isDark),
                selected = currentTheme == theme,
                colors = getThemeColors(theme, isDark, context),
                onClick = { onThemeChange(theme) },
            )
        }
    }
}

@Composable
private fun getThemeTitle(
    theme: AppTheme,
    isDark: Boolean,
): String =
    when (theme) {
        AppTheme.CATPPUCCIN -> stringResource(R.string.title_settings_catppuccin_theme)
        AppTheme.NORD -> stringResource(R.string.title_settings_nord_theme)
        AppTheme.DRACULA ->
            if (isDark) {
                stringResource(
                    R.string.title_settings_dracula_theme,
                )
            } else {
                stringResource(R.string.title_settings_alucard_theme)
            }
        AppTheme.DYNAMIC -> stringResource(R.string.title_settings_dynamic_color)
        AppTheme.TOKYO_NIGHT -> stringResource(R.string.title_settings_tokyo_night_theme)
    }

@Composable
private fun getThemeSubtitle(
    theme: AppTheme,
    isDark: Boolean,
): String =
    when (theme) {
        AppTheme.CATPPUCCIN ->
            if (isDark) {
                stringResource(
                    R.string.subtitle_settings_mocha_theme,
                )
            } else {
                stringResource(R.string.subtitle_settings_latte_theme)
            }
        AppTheme.NORD ->
            if (isDark) {
                stringResource(
                    R.string.subtitle_settings_nord_dark_theme,
                )
            } else {
                stringResource(R.string.subtitle_settings_nord_light_theme)
            }
        AppTheme.DRACULA ->
            if (isDark) {
                stringResource(
                    R.string.subtitle_settings_vampire_theme,
                )
            } else {
                stringResource(R.string.subtitle_settings_dracula_theme)
            }
        AppTheme.DYNAMIC -> stringResource(R.string.subtitle_settings_dynamic_color)
        AppTheme.TOKYO_NIGHT ->
            if (isDark) {
                stringResource(R.string.subtitle_settings_tokyo_night_dark_theme)
            } else {
                stringResource(R.string.subtitle_settings_tokyo_night_light_theme)
            }
    }

@Composable
private fun getThemeColors(
    theme: AppTheme,
    isDark: Boolean,
    context: Context,
): List<Color> =
    when (theme) {
        AppTheme.DYNAMIC -> dynamicColorsFromContext(context, isDark)
        AppTheme.CATPPUCCIN ->
            if (isDark) {
                listOf(CatppuccinMocha.Mauve, CatppuccinMocha.Pink, CatppuccinMocha.Sky)
            } else {
                listOf(CatppuccinLatte.Mauve, CatppuccinLatte.Pink, CatppuccinLatte.Sky)
            }
        AppTheme.DRACULA ->
            if (isDark) {
                listOf(Dracula.Purple, Dracula.Pink, Dracula.Cyan)
            } else {
                listOf(Color(0xFF6272A4), Color(0xFFFF79C6), Color(0xFF8BE9FD))
            }
        AppTheme.NORD ->
            if (isDark) {
                listOf(NordDark.Primary, NordDark.Secondary, NordDark.Tertiary)
            } else {
                listOf(Color(0xFF88C0D0), Color(0xFF81A1C1), Color(0xFF8FBCBB))
            }
        AppTheme.TOKYO_NIGHT ->
            if (isDark) {
                listOf(TokyoNightDark.Blue, TokyoNightDark.Purple, TokyoNightDark.Cyan)
            } else {
                listOf(TokyoNightDay.Blue, TokyoNightDay.Purple, TokyoNightDay.Cyan)
            }
    }

@Composable
private fun dynamicColorsFromContext(
    context: Context,
    isDark: Boolean,
): List<Color> =
    if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.S) {
        val scheme = if (isDark) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)

        listOf(scheme.primaryContainer, scheme.secondaryContainer, scheme.tertiaryContainer)
    } else {
        listOf(
            MaterialTheme.colorScheme.primaryContainer,
            MaterialTheme.colorScheme.secondaryContainer,
            MaterialTheme.colorScheme.tertiaryContainer,
        )
    }

@Composable
private fun ThemeCard(
    modifier: Modifier = Modifier,
    title: String,
    subtitle: String,
    selected: Boolean,
    colors: List<Color>,
    onClick: () -> Unit,
) {
    val borderColor = if (selected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.outlineVariant
    val containerColor =
        if (selected) MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.3f) else MaterialTheme.colorScheme.surface

    OutlinedCard(
        onClick = onClick,
        modifier = modifier,
        shape = ShapeTokens.Large,
        colors = CardDefaults.outlinedCardColors(containerColor = containerColor),
        border = CardDefaults.outlinedCardBorder().copy(brush = SolidColor(borderColor)),
    ) {
        Column(
            modifier = Modifier.padding(SpacingTokens.Medium),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Box(
                modifier =
                    Modifier
                        .size(60.dp)
                        .padding(SpacingTokens.ExtraSmall),
                contentAlignment = Alignment.Center,
            ) {
                Box(
                    modifier =
                        Modifier
                            .fillMaxSize()
                            .background(
                                brush = Brush.linearGradient(colors),
                                shape = ShapeTokens.Full,
                            ),
                )

                Row(
                    horizontalArrangement = Arrangement.spacedBy((-12).dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    val dotBorderColor = if (isSystemInDarkTheme()) Color.White.copy(alpha = 0.2f) else Color.Black.copy(alpha = 0.1f)
                    colors.forEach { color ->
                        Box(
                            modifier =
                                Modifier
                                    .size(SpacingTokens.ExtraLarge)
                                    .background(color, ShapeTokens.Full)
                                    .border(1.5.dp, dotBorderColor, ShapeTokens.Full),
                        )
                    }
                }
            }

            Spacer(modifier = Modifier.height(SpacingTokens.Small))

            Text(
                text = title,
                style = MaterialTheme.typography.labelLarge,
                fontWeight = FontWeight.Bold,
                color = if (selected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurface,
                maxLines = 1,
            )

            Text(
                text = subtitle,
                style = MaterialTheme.typography.bodySmall,
                fontSize = 10.sp,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
            )

            if (selected) {
                Icon(
                    imageVector = Icons.Filled.CheckCircle,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.primary,
                    modifier =
                        Modifier
                            .padding(top = SpacingTokens.ExtraSmall)
                            .size(SizeTokens.IconExtraSmall),
                )
            } else {
                Spacer(modifier = Modifier.height(SpacingTokens.ExtraLarge))
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ThemeSettingsPreview() {
    AcerolaTheme {
        Main.Config.Component.ThemeSettings(
            currentTheme = AppTheme.CATPPUCCIN,
            onThemeChange = {},
        )
    }
}
