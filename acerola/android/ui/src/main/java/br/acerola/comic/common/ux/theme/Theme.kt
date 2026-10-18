package br.acerola.comic.common.ux.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp
import br.acerola.comic.common.ux.theme.color.CatppuccinLatte
import br.acerola.comic.common.ux.theme.color.CatppuccinMocha

// Ícones de ação seguem o mesmo formato do desktop: fundo SÓLIDO na cor semântica/de destaque +
// ícone numa cor neutra por cima (o fundo "contorna" o ícone, nunca o contrário) — por isso
// success/accent aqui são a cor cheia da paleta, não um container translúcido. onSuccess/onAccent
// recebem o `--primary-foreground`, do mesmo jeito que o desktop reaproveita esse token pros
// `--accent-hero-foreground`/`--destructive-foreground`. Material3 não tem papéis nativos de
// "sucesso" nem de "accent vibrante", então eles são expostos via
// LocalAcerolaExtraColors/AcerolaExtendedTheme. successContainer/onSuccessContainer são a exceção:
// só o Snackbar de sucesso usa (banner cheio, não ícone-em-quadrado), com o par `--success-soft`
// + `--success` da paleta — o mesmo tom suave do Error (errorContainer = `--destructive-soft`).
data class AcerolaExtraColors(
    val success: Color,
    val onSuccess: Color,
    val accent: Color,
    val onAccent: Color,
    val successContainer: Color,
    val onSuccessContainer: Color,
)

private val LocalAcerolaExtraColors =
    staticCompositionLocalOf<AcerolaExtraColors> {
        error("AcerolaExtraColors não fornecido — use dentro de AcerolaTheme")
    }

object AcerolaExtendedTheme {
    val colors: AcerolaExtraColors
        @Composable get() = LocalAcerolaExtraColors.current
}

// Tema único do app: só existe a troca claro/escuro (seguindo o sistema). Os papéis do
// Material3 são preenchidos com os tokens da paleta (ver color/Catppuccin.kt) seguindo o mesmo
// papel que cada token tem no CSS: `background`/`card`/`surface` pros níveis de superfície,
// `border` pras bordas, `*-soft` pros containers.
private val DarkColorScheme =
    darkColorScheme(
        primary = CatppuccinMocha.Primary,
        onPrimary = CatppuccinMocha.PrimaryForeground,
        primaryContainer = CatppuccinMocha.PrimarySoft,
        onPrimaryContainer = CatppuccinMocha.Primary,
        secondary = CatppuccinMocha.AccentHero,
        onSecondary = CatppuccinMocha.PrimaryForeground,
        secondaryContainer = CatppuccinMocha.Secondary,
        onSecondaryContainer = CatppuccinMocha.AccentHero,
        tertiary = CatppuccinMocha.Chart5,
        onTertiary = CatppuccinMocha.PrimaryForeground,
        tertiaryContainer = CatppuccinMocha.Secondary,
        onTertiaryContainer = CatppuccinMocha.Chart5,
        background = CatppuccinMocha.Background,
        onBackground = CatppuccinMocha.Foreground,
        surface = CatppuccinMocha.Background,
        onSurface = CatppuccinMocha.Foreground,
        surfaceVariant = CatppuccinMocha.Surface,
        onSurfaceVariant = CatppuccinMocha.MutedForeground,
        surfaceContainerLowest = CatppuccinMocha.Crust,
        surfaceContainerLow = CatppuccinMocha.Mantle,
        surfaceContainer = CatppuccinMocha.Card,
        surfaceContainerHigh = CatppuccinMocha.Surface,
        surfaceContainerHighest = CatppuccinMocha.Border,
        outline = CatppuccinMocha.Border,
        outlineVariant = CatppuccinMocha.Border,
        error = CatppuccinMocha.Destructive,
        onError = CatppuccinMocha.PrimaryForeground,
        errorContainer = CatppuccinMocha.DestructiveSoft,
        onErrorContainer = CatppuccinMocha.Destructive,
    )

private val DarkExtraColors =
    AcerolaExtraColors(
        success = CatppuccinMocha.Success,
        onSuccess = CatppuccinMocha.PrimaryForeground,
        accent = CatppuccinMocha.AccentHero,
        onAccent = CatppuccinMocha.PrimaryForeground,
        successContainer = CatppuccinMocha.SuccessSoft,
        onSuccessContainer = CatppuccinMocha.Success,
    )

private val LightColorScheme =
    lightColorScheme(
        primary = CatppuccinLatte.Primary,
        onPrimary = CatppuccinLatte.PrimaryForeground,
        primaryContainer = CatppuccinLatte.PrimarySoft,
        onPrimaryContainer = CatppuccinLatte.Primary,
        secondary = CatppuccinLatte.AccentHero,
        onSecondary = CatppuccinLatte.PrimaryForeground,
        secondaryContainer = CatppuccinLatte.Secondary,
        onSecondaryContainer = CatppuccinLatte.AccentHero,
        tertiary = CatppuccinLatte.Chart5,
        onTertiary = CatppuccinLatte.PrimaryForeground,
        tertiaryContainer = CatppuccinLatte.Secondary,
        onTertiaryContainer = CatppuccinLatte.Chart5,
        background = CatppuccinLatte.Background,
        onBackground = CatppuccinLatte.Foreground,
        surface = CatppuccinLatte.Background,
        onSurface = CatppuccinLatte.Foreground,
        surfaceVariant = CatppuccinLatte.Surface,
        onSurfaceVariant = CatppuccinLatte.MutedForeground,
        surfaceContainerLowest = CatppuccinLatte.Card,
        surfaceContainerLow = CatppuccinLatte.Base,
        surfaceContainer = CatppuccinLatte.Mantle,
        surfaceContainerHigh = CatppuccinLatte.Surface,
        surfaceContainerHighest = CatppuccinLatte.Crust,
        outline = CatppuccinLatte.Border,
        outlineVariant = CatppuccinLatte.Border,
        error = CatppuccinLatte.Destructive,
        onError = CatppuccinLatte.PrimaryForeground,
        errorContainer = CatppuccinLatte.DestructiveSoft,
        onErrorContainer = CatppuccinLatte.Destructive,
    )

private val LightExtraColors =
    AcerolaExtraColors(
        success = CatppuccinLatte.Success,
        onSuccess = CatppuccinLatte.PrimaryForeground,
        accent = CatppuccinLatte.AccentHero,
        onAccent = CatppuccinLatte.PrimaryForeground,
        successContainer = CatppuccinLatte.SuccessSoft,
        onSuccessContainer = CatppuccinLatte.Success,
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
    content: @Composable () -> Unit,
) {
    val colorScheme = if (darkTheme) DarkColorScheme else LightColorScheme
    val extraColors = if (darkTheme) DarkExtraColors else LightExtraColors

    CompositionLocalProvider(LocalAcerolaExtraColors provides extraColors) {
        MaterialTheme(
            colorScheme = colorScheme,
            typography = Typography,
            content = content,
        )
    }
}
