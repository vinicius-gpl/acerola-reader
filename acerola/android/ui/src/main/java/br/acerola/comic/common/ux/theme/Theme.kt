package br.acerola.comic.common.ux.theme

import android.content.Context
import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.lerp
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.theme.color.Alucard
import br.acerola.comic.common.ux.theme.color.CatppuccinLatte
import br.acerola.comic.common.ux.theme.color.CatppuccinMocha
import br.acerola.comic.common.ux.theme.color.Dracula
import br.acerola.comic.common.ux.theme.color.NordDark
import br.acerola.comic.common.ux.theme.color.NordLight
import br.acerola.comic.common.ux.theme.color.TokyoNightDark
import br.acerola.comic.common.ux.theme.color.TokyoNightDay
import br.acerola.comic.config.preference.types.AppTheme

// Alguns temas (Nord, Dracula/Alucard, TokyoNight) não definem uma cor de "container de erro"
// própria na paleta original. Em vez de inventar um hex sem poder validar visualmente,
// derivamos essas cores a partir do restante da paleta do próprio tema, mantendo os papéis
// completos do Material3 (error / errorContainer / onErrorContainer) para todos os temas.
private fun deriveErrorContainer(
    background: Color,
    error: Color,
): Color = lerp(start = background, stop = error, fraction = 0.35f)

private fun deriveOnErrorContainer(
    foreground: Color,
    error: Color,
): Color = lerp(start = foreground, stop = error, fraction = 0.55f)

private val CatppuccinDarkColorScheme =
    darkColorScheme(
        primary = CatppuccinMocha.Mauve,
        onPrimary = CatppuccinMocha.Base,
        primaryContainer = CatppuccinMocha.Surface0,
        onPrimaryContainer = CatppuccinMocha.Mauve,
        secondary = CatppuccinMocha.Pink,
        onSecondary = CatppuccinMocha.Base,
        secondaryContainer = CatppuccinMocha.Surface0,
        onSecondaryContainer = CatppuccinMocha.Pink,
        tertiary = CatppuccinMocha.Sky,
        onTertiary = CatppuccinMocha.Base,
        tertiaryContainer = CatppuccinMocha.Surface0,
        onTertiaryContainer = CatppuccinMocha.Sky,
        background = CatppuccinMocha.Base,
        onBackground = CatppuccinMocha.Text,
        surface = CatppuccinMocha.Base,
        onSurface = CatppuccinMocha.Text,
        surfaceVariant = CatppuccinMocha.Surface1,
        onSurfaceVariant = CatppuccinMocha.Subtext1,
        surfaceContainerLowest = CatppuccinMocha.Mantle,
        surfaceContainerLow = CatppuccinMocha.Base,
        surfaceContainer = CatppuccinMocha.Surface0,
        surfaceContainerHigh = CatppuccinMocha.Surface1,
        surfaceContainerHighest = CatppuccinMocha.Surface2,
        outline = CatppuccinMocha.Overlay0,
        error = CatppuccinMocha.Red,
        onError = CatppuccinMocha.Base,
        errorContainer = CatppuccinMocha.Maroon,
        onErrorContainer = CatppuccinMocha.Text,
    )

private val CatppuccinLightColorScheme =
    lightColorScheme(
        primary = CatppuccinLatte.Mauve,
        onPrimary = CatppuccinLatte.Base,
        primaryContainer = CatppuccinLatte.Surface0,
        onPrimaryContainer = CatppuccinLatte.Mauve,
        secondary = CatppuccinLatte.Pink,
        onSecondary = CatppuccinLatte.Base,
        secondaryContainer = CatppuccinLatte.Surface0,
        onSecondaryContainer = CatppuccinLatte.Pink,
        tertiary = CatppuccinLatte.Sky,
        onTertiary = CatppuccinLatte.Base,
        tertiaryContainer = CatppuccinLatte.Surface0,
        onTertiaryContainer = CatppuccinLatte.Sky,
        background = CatppuccinLatte.Base,
        onBackground = CatppuccinLatte.Text,
        surface = CatppuccinLatte.Base,
        onSurface = CatppuccinLatte.Text,
        surfaceVariant = CatppuccinLatte.Surface1,
        onSurfaceVariant = CatppuccinLatte.Subtext1,
        surfaceContainerLowest = CatppuccinLatte.Base,
        surfaceContainerLow = CatppuccinLatte.Base,
        surfaceContainer = CatppuccinLatte.Mantle,
        surfaceContainerHigh = CatppuccinLatte.Crust,
        surfaceContainerHighest = CatppuccinLatte.Surface0,
        outline = CatppuccinLatte.Overlay0,
        error = CatppuccinLatte.Red,
        onError = CatppuccinLatte.Base,
        errorContainer = CatppuccinLatte.Maroon,
        onErrorContainer = CatppuccinLatte.Text,
    )

private val NordDarkColorScheme =
    darkColorScheme(
        primary = NordDark.Primary,
        onPrimary = NordDark.Background,
        primaryContainer = NordDark.Surface,
        onPrimaryContainer = NordDark.Primary,
        secondary = NordDark.Secondary,
        onSecondary = NordDark.Background,
        secondaryContainer = NordDark.Surface,
        onSecondaryContainer = NordDark.Secondary,
        tertiary = NordDark.Tertiary,
        onTertiary = NordDark.Background,
        tertiaryContainer = NordDark.Surface,
        onTertiaryContainer = NordDark.Tertiary,
        background = NordDark.Background,
        onBackground = NordDark.Text,
        surface = NordDark.Background,
        onSurface = NordDark.Text,
        surfaceVariant = NordDark.Surface,
        onSurfaceVariant = NordDark.Subtext,
        surfaceContainerLowest = NordDark.Background,
        surfaceContainerLow = NordDark.Background,
        surfaceContainer = NordDark.Surface,
        surfaceContainerHigh = NordDark.SurfaceVariant,
        surfaceContainerHighest = NordDark.Outline,
        outline = NordDark.Outline,
        error = NordDark.Error,
        onError = NordDark.Text,
        errorContainer = deriveErrorContainer(NordDark.Background, NordDark.Error),
        onErrorContainer = deriveOnErrorContainer(NordDark.Text, NordDark.Error),
    )

private val NordLightColorScheme =
    lightColorScheme(
        primary = NordLight.Primary,
        onPrimary = NordLight.Background,
        primaryContainer = NordLight.Surface,
        onPrimaryContainer = NordLight.Primary,
        secondary = NordLight.Secondary,
        onSecondary = NordLight.Background,
        secondaryContainer = NordLight.Surface,
        onSecondaryContainer = NordLight.Secondary,
        tertiary = NordLight.Tertiary,
        onTertiary = NordLight.Background,
        tertiaryContainer = NordLight.Surface,
        onTertiaryContainer = NordLight.Tertiary,
        background = NordLight.Background,
        onBackground = NordLight.Text,
        surface = NordLight.Background,
        onSurface = NordLight.Text,
        surfaceVariant = NordLight.Surface,
        onSurfaceVariant = NordLight.Subtext,
        surfaceContainerLowest = NordLight.Background,
        surfaceContainerLow = NordLight.Background,
        surfaceContainer = NordLight.Surface,
        surfaceContainerHigh = NordLight.SurfaceVariant,
        surfaceContainerHighest = NordLight.Outline,
        outline = NordLight.Outline,
        error = NordLight.Error,
        onError = NordLight.Background,
        errorContainer = deriveErrorContainer(NordLight.Background, NordLight.Error),
        onErrorContainer = deriveOnErrorContainer(NordLight.Text, NordLight.Error),
    )

private val DraculaColorScheme =
    darkColorScheme(
        primary = Dracula.Purple,
        onPrimary = Dracula.Background,
        primaryContainer = Dracula.CurrentLine,
        onPrimaryContainer = Dracula.Purple,
        secondary = Dracula.Pink,
        onSecondary = Dracula.Background,
        secondaryContainer = Dracula.CurrentLine,
        onSecondaryContainer = Dracula.Pink,
        tertiary = Dracula.Cyan,
        onTertiary = Dracula.Background,
        tertiaryContainer = Dracula.CurrentLine,
        onTertiaryContainer = Dracula.Cyan,
        background = Dracula.Background,
        onBackground = Dracula.Foreground,
        surface = Dracula.Background,
        onSurface = Dracula.Foreground,
        surfaceVariant = Dracula.CurrentLine,
        onSurfaceVariant = Dracula.Foreground,
        surfaceContainerLowest = Dracula.Background,
        surfaceContainerLow = Dracula.Background,
        surfaceContainer = Dracula.CurrentLine,
        surfaceContainerHigh = Dracula.Selection,
        surfaceContainerHighest = Dracula.Comment,
        outline = Dracula.Comment,
        error = Dracula.Red,
        onError = Dracula.Foreground,
        errorContainer = deriveErrorContainer(Dracula.Background, Dracula.Red),
        onErrorContainer = deriveOnErrorContainer(Dracula.Foreground, Dracula.Red),
    )

private val AlucardColorScheme =
    lightColorScheme(
        primary = Alucard.Purple,
        onPrimary = Alucard.Background,
        primaryContainer = Alucard.CurrentLine,
        onPrimaryContainer = Alucard.Purple,
        secondary = Alucard.Pink,
        onSecondary = Alucard.Background,
        secondaryContainer = Alucard.CurrentLine,
        onSecondaryContainer = Alucard.Pink,
        tertiary = Alucard.Cyan,
        onTertiary = Alucard.Background,
        tertiaryContainer = Alucard.CurrentLine,
        onTertiaryContainer = Alucard.Cyan,
        background = Alucard.Background,
        onBackground = Alucard.Foreground,
        surface = Alucard.Background,
        onSurface = Alucard.Foreground,
        surfaceVariant = Alucard.CurrentLine,
        onSurfaceVariant = Alucard.Foreground,
        surfaceContainerLowest = Alucard.Background,
        surfaceContainerLow = Alucard.Background,
        surfaceContainer = Alucard.CurrentLine,
        surfaceContainerHigh = Alucard.SurfaceHigh,
        surfaceContainerHighest = Alucard.SurfaceHighest,
        outline = Alucard.Comment,
        error = Alucard.Red,
        onError = Alucard.Foreground,
        errorContainer = deriveErrorContainer(Alucard.Background, Alucard.Red),
        onErrorContainer = deriveOnErrorContainer(Alucard.Foreground, Alucard.Red),
    )

private val TokyoNightDarkColorScheme =
    darkColorScheme(
        primary = TokyoNightDark.Blue,
        onPrimary = TokyoNightDark.Background,
        primaryContainer = TokyoNightDark.Surface,
        onPrimaryContainer = TokyoNightDark.Blue,
        secondary = TokyoNightDark.Cyan,
        onSecondary = TokyoNightDark.Background,
        secondaryContainer = TokyoNightDark.Surface,
        onSecondaryContainer = TokyoNightDark.Cyan,
        tertiary = TokyoNightDark.Purple,
        onTertiary = TokyoNightDark.Background,
        tertiaryContainer = TokyoNightDark.Surface,
        onTertiaryContainer = TokyoNightDark.Purple,
        background = TokyoNightDark.Background,
        onBackground = TokyoNightDark.Foreground,
        surface = TokyoNightDark.Background,
        onSurface = TokyoNightDark.Foreground,
        surfaceVariant = TokyoNightDark.Surface,
        onSurfaceVariant = TokyoNightDark.Comment,
        surfaceContainerLowest = TokyoNightDark.Background,
        surfaceContainerLow = TokyoNightDark.Background,
        surfaceContainer = TokyoNightDark.Surface,
        surfaceContainerHigh = TokyoNightDark.SurfaceHigh,
        surfaceContainerHighest = TokyoNightDark.SurfaceHighest,
        outline = TokyoNightDark.Comment,
        error = TokyoNightDark.Red,
        onError = TokyoNightDark.Foreground,
        errorContainer = deriveErrorContainer(TokyoNightDark.Background, TokyoNightDark.Red),
        onErrorContainer = deriveOnErrorContainer(TokyoNightDark.Foreground, TokyoNightDark.Red),
    )

private val TokyoNightLightColorScheme =
    lightColorScheme(
        primary = TokyoNightDay.Blue,
        onPrimary = TokyoNightDay.Background,
        primaryContainer = TokyoNightDay.Surface,
        onPrimaryContainer = TokyoNightDay.Blue,
        secondary = TokyoNightDay.Cyan,
        onSecondary = TokyoNightDay.Background,
        secondaryContainer = TokyoNightDay.Surface,
        onSecondaryContainer = TokyoNightDay.Cyan,
        tertiary = TokyoNightDay.Purple,
        onTertiary = TokyoNightDay.Background,
        tertiaryContainer = TokyoNightDay.Surface,
        onTertiaryContainer = TokyoNightDay.Purple,
        background = TokyoNightDay.Background,
        onBackground = TokyoNightDay.Foreground,
        surface = TokyoNightDay.Background,
        onSurface = TokyoNightDay.Foreground,
        surfaceVariant = TokyoNightDay.Surface,
        onSurfaceVariant = TokyoNightDay.Comment,
        surfaceContainerLowest = TokyoNightDay.Background,
        surfaceContainerLow = TokyoNightDay.Background,
        surfaceContainer = TokyoNightDay.Surface,
        surfaceContainerHigh = TokyoNightDay.SurfaceHigh,
        surfaceContainerHighest = TokyoNightDay.SurfaceHighest,
        outline = TokyoNightDay.Comment,
        error = TokyoNightDay.Red,
        onError = TokyoNightDay.Background,
        errorContainer = deriveErrorContainer(TokyoNightDay.Background, TokyoNightDay.Red),
        onErrorContainer = deriveOnErrorContainer(TokyoNightDay.Foreground, TokyoNightDay.Red),
    )

// Escala tipográfica completa do Material3, explícita para toda a hierarquia (antes só
// bodyLarge era definido e o restante herdava o default do Material3 sem decisão consciente).
// Ter os 13 estilos aqui, num único lugar, é o que permite trocar fontFamily/pesos do app
// inteiro futuramente sem caçar overrides espalhados pelos componentes.
val Typography =
    Typography(
        displayLarge =
            TextStyle(
                fontFamily = FontFamily.Default,
                fontWeight = FontWeight.Normal,
                fontSize = 57.sp,
                lineHeight = 64.sp,
                letterSpacing = (-0.25).sp,
            ),
        displayMedium =
            TextStyle(fontFamily = FontFamily.Default, fontWeight = FontWeight.Normal, fontSize = 45.sp, lineHeight = 52.sp, letterSpacing = 0.sp),
        displaySmall =
            TextStyle(fontFamily = FontFamily.Default, fontWeight = FontWeight.Normal, fontSize = 36.sp, lineHeight = 44.sp, letterSpacing = 0.sp),
        headlineLarge =
            TextStyle(fontFamily = FontFamily.Default, fontWeight = FontWeight.Normal, fontSize = 32.sp, lineHeight = 40.sp, letterSpacing = 0.sp),
        headlineMedium =
            TextStyle(fontFamily = FontFamily.Default, fontWeight = FontWeight.Normal, fontSize = 28.sp, lineHeight = 36.sp, letterSpacing = 0.sp),
        headlineSmall =
            TextStyle(fontFamily = FontFamily.Default, fontWeight = FontWeight.Normal, fontSize = 24.sp, lineHeight = 32.sp, letterSpacing = 0.sp),
        titleLarge =
            TextStyle(fontFamily = FontFamily.Default, fontWeight = FontWeight.Normal, fontSize = 22.sp, lineHeight = 28.sp, letterSpacing = 0.sp),
        titleMedium =
            TextStyle(fontFamily = FontFamily.Default, fontWeight = FontWeight.Medium, fontSize = 16.sp, lineHeight = 24.sp, letterSpacing = 0.15.sp),
        titleSmall =
            TextStyle(fontFamily = FontFamily.Default, fontWeight = FontWeight.Medium, fontSize = 14.sp, lineHeight = 20.sp, letterSpacing = 0.1.sp),
        bodyLarge =
            TextStyle(
                fontFamily = FontFamily.Default,
                fontWeight = FontWeight.Normal,
                fontSize = 16.sp,
                lineHeight = 24.sp,
                letterSpacing = 0.5.sp,
            ),
        bodyMedium =
            TextStyle(fontFamily = FontFamily.Default, fontWeight = FontWeight.Normal, fontSize = 14.sp, lineHeight = 20.sp, letterSpacing = 0.25.sp),
        bodySmall =
            TextStyle(fontFamily = FontFamily.Default, fontWeight = FontWeight.Normal, fontSize = 12.sp, lineHeight = 16.sp, letterSpacing = 0.4.sp),
        labelLarge =
            TextStyle(fontFamily = FontFamily.Default, fontWeight = FontWeight.Medium, fontSize = 14.sp, lineHeight = 20.sp, letterSpacing = 0.1.sp),
        labelMedium =
            TextStyle(fontFamily = FontFamily.Default, fontWeight = FontWeight.Medium, fontSize = 12.sp, lineHeight = 16.sp, letterSpacing = 0.5.sp),
        labelSmall =
            TextStyle(fontFamily = FontFamily.Default, fontWeight = FontWeight.Medium, fontSize = 11.sp, lineHeight = 16.sp, letterSpacing = 0.5.sp),
    )

@Composable
fun AcerolaTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    theme: AppTheme = AppTheme.CATPPUCCIN,
    content: @Composable () -> Unit,
) {
    val colorScheme =
        when (theme) {
            AppTheme.DYNAMIC -> {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                    val context: Context = LocalContext.current
                    if (darkTheme) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
                } else {
                    if (darkTheme) CatppuccinDarkColorScheme else CatppuccinLightColorScheme
                }
            }
            AppTheme.NORD -> if (darkTheme) NordDarkColorScheme else NordLightColorScheme
            AppTheme.DRACULA -> if (darkTheme) DraculaColorScheme else AlucardColorScheme
            AppTheme.CATPPUCCIN -> if (darkTheme) CatppuccinDarkColorScheme else CatppuccinLightColorScheme
            AppTheme.TOKYO_NIGHT -> if (darkTheme) TokyoNightDarkColorScheme else TokyoNightLightColorScheme
        }

    MaterialTheme(
        colorScheme = colorScheme,
        typography = Typography,
        content = content,
    )
}
