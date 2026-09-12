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
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.staticCompositionLocalOf
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

// Alguns temas (Nord, Dracula/Alucard, TokyoNight) não definem uma cor de "container" própria
// (erro ou sucesso) na paleta original. Em vez de inventar um hex sem poder validar visualmente,
// derivamos essas cores a partir do restante da paleta do próprio tema, mantendo os papéis
// completos do Material3 (error/errorContainer/onErrorContainer) e o papel extra de sucesso
// (successContainer/onSuccessContainer) para todos os temas.
private fun deriveContainer(
    background: Color,
    accent: Color,
): Color = lerp(start = background, stop = accent, fraction = 0.35f)

private fun deriveOnContainer(
    foreground: Color,
    accent: Color,
): Color = lerp(start = foreground, stop = accent, fraction = 0.55f)

// Material3 não tem papéis de "sucesso" nativos — cada tema expõe os seus via
// LocalAcerolaExtraColors/AcerolaExtendedTheme, usando o próprio verde da paleta do tema em vez
// de um verde fixo igual pra todo mundo (o que destoava de temas como Dracula ou TokyoNight).
data class AcerolaExtraColors(
    val successContainer: Color,
    val onSuccessContainer: Color,
)

private fun successExtraColors(
    background: Color,
    foreground: Color,
    green: Color,
): AcerolaExtraColors =
    AcerolaExtraColors(
        successContainer = deriveContainer(background, green),
        onSuccessContainer = deriveOnContainer(foreground, green),
    )

// Usado só quando o tema é o Dynamic Color real do Material You (Android 12+), que não tem
// um "verde" próprio pra derivar a partir da paleta do usuário.
private val DynamicSuccessGreen = Color(0xFF2E7D32)

private val LocalAcerolaExtraColors =
    staticCompositionLocalOf<AcerolaExtraColors> {
        error("AcerolaExtraColors não fornecido — use dentro de AcerolaTheme")
    }

object AcerolaExtendedTheme {
    val colors: AcerolaExtraColors
        @Composable get() = LocalAcerolaExtraColors.current
}

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

private val CatppuccinDarkExtraColors =
    successExtraColors(CatppuccinMocha.Base, CatppuccinMocha.Text, CatppuccinMocha.Green)

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

private val CatppuccinLightExtraColors =
    successExtraColors(CatppuccinLatte.Base, CatppuccinLatte.Text, CatppuccinLatte.Green)

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
        errorContainer = deriveContainer(NordDark.Background, NordDark.Error),
        onErrorContainer = deriveOnContainer(NordDark.Text, NordDark.Error),
    )

private val NordDarkExtraColors =
    successExtraColors(NordDark.Background, NordDark.Text, NordDark.Green)

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
        errorContainer = deriveContainer(NordLight.Background, NordLight.Error),
        onErrorContainer = deriveOnContainer(NordLight.Text, NordLight.Error),
    )

private val NordLightExtraColors =
    successExtraColors(NordLight.Background, NordLight.Text, NordLight.Green)

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
        errorContainer = deriveContainer(Dracula.Background, Dracula.Red),
        onErrorContainer = deriveOnContainer(Dracula.Foreground, Dracula.Red),
    )

private val DraculaExtraColors =
    successExtraColors(Dracula.Background, Dracula.Foreground, Dracula.Green)

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
        errorContainer = deriveContainer(Alucard.Background, Alucard.Red),
        onErrorContainer = deriveOnContainer(Alucard.Foreground, Alucard.Red),
    )

private val AlucardExtraColors =
    successExtraColors(Alucard.Background, Alucard.Foreground, Alucard.Green)

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
        errorContainer = deriveContainer(TokyoNightDark.Background, TokyoNightDark.Red),
        onErrorContainer = deriveOnContainer(TokyoNightDark.Foreground, TokyoNightDark.Red),
    )

private val TokyoNightDarkExtraColors =
    successExtraColors(TokyoNightDark.Background, TokyoNightDark.Foreground, TokyoNightDark.Green)

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
        errorContainer = deriveContainer(TokyoNightDay.Background, TokyoNightDay.Red),
        onErrorContainer = deriveOnContainer(TokyoNightDay.Foreground, TokyoNightDay.Red),
    )

private val TokyoNightLightExtraColors =
    successExtraColors(TokyoNightDay.Background, TokyoNightDay.Foreground, TokyoNightDay.Green)

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

    val extraColors =
        when (theme) {
            AppTheme.DYNAMIC -> {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                    successExtraColors(colorScheme.background, colorScheme.onBackground, DynamicSuccessGreen)
                } else {
                    if (darkTheme) CatppuccinDarkExtraColors else CatppuccinLightExtraColors
                }
            }
            AppTheme.NORD -> if (darkTheme) NordDarkExtraColors else NordLightExtraColors
            AppTheme.DRACULA -> if (darkTheme) DraculaExtraColors else AlucardExtraColors
            AppTheme.CATPPUCCIN -> if (darkTheme) CatppuccinDarkExtraColors else CatppuccinLightExtraColors
            AppTheme.TOKYO_NIGHT -> if (darkTheme) TokyoNightDarkExtraColors else TokyoNightLightExtraColors
        }

    CompositionLocalProvider(LocalAcerolaExtraColors provides extraColors) {
        MaterialTheme(
            colorScheme = colorScheme,
            typography = Typography,
            content = content,
        )
    }
}
