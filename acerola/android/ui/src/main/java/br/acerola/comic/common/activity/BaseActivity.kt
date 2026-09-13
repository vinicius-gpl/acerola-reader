package br.acerola.comic.common.activity

import android.content.Context
import android.content.res.Configuration
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.viewModels
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.unit.dp
import androidx.navigation.NavGraphBuilder
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.rememberNavController
import br.acerola.comic.common.state.LocalSnackbarHostState
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.MobileDataSyncDialog
import br.acerola.comic.common.ux.component.Scaffold
import br.acerola.comic.common.ux.component.SnackbarError
import br.acerola.comic.common.ux.component.SnackbarSuccess
import br.acerola.comic.common.ux.component.SnackbarVariant
import br.acerola.comic.common.ux.component.SnackbarWarn
import br.acerola.comic.common.ux.component.resolveSnackbarVariant
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.viewmodel.network.MobileDataSyncViewModel
import br.acerola.comic.common.viewmodel.theme.ThemeViewModel
import dagger.hilt.android.AndroidEntryPoint
import dev.chrisbanes.haze.HazeState
import dev.chrisbanes.haze.hazeSource
import dev.chrisbanes.haze.rememberHazeState

@AndroidEntryPoint
abstract class BaseActivity : ComponentActivity() {
    abstract val startDestinationRes: Int

    open val applyScaffoldPadding: Boolean = true

    private val themeViewModel: ThemeViewModel by viewModels()
    private val mobileDataSyncViewModel: MobileDataSyncViewModel by viewModels()

    open fun NavGraphBuilder.setupNavGraph(
        context: Context,
        navController: NavHostController,
    ) {
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        enableEdgeToEdge()

        setContent {
            val currentTheme by themeViewModel.currentTheme.collectAsState()

            AcerolaTheme(theme = currentTheme) {
                val navController = rememberNavController()
                val startDestination = getString(startDestinationRes)
                val snackbarHostState = remember { SnackbarHostState() }
                val configuration = LocalConfiguration.current
                val isLandscape = configuration.orientation == Configuration.ORIENTATION_LANDSCAPE

                CompositionLocalProvider(
                    value = LocalSnackbarHostState provides snackbarHostState,
                ) {
                    // Montado uma única vez aqui (raiz do app) — não em nenhuma tela específica
                    // — porque Home/Histórico/Quadrinho/Biblioteca remota/Sync podem disparar
                    // uma sincronização P2P cada uma pelo seu próprio caminho (ver
                    // `MobileDataSyncGate`). Um diálogo só na tela de Sync não cobriria os
                    // outros quatro.
                    val awaitingMobileDataConfirmation by mobileDataSyncViewModel.awaitingConfirmation.collectAsState()
                    if (awaitingMobileDataConfirmation) {
                        Acerola.Component.MobileDataSyncDialog(
                            onConfirm = { remember -> mobileDataSyncViewModel.confirm(remember) },
                            onCancel = { mobileDataSyncViewModel.cancel() },
                        )
                    }

                    if (isLandscape) {
                        // Em modo landscape: sidebar fora do Scaffold para ocupar toda a altura
                        // (status bar + conteúdo + nav bar), igual ao Spotify
                        // applyStatusBarPadding=false para que a sidebar gerencie seus próprios insets
                        Acerola.Component.Scaffold(applyStatusBarPadding = false) {
                            Row(modifier = Modifier.fillMaxHeight()) {
                                SideBar(navController)
                                Scaffold(
                                    modifier = Modifier.weight(1f),
                                    topBar = { TopBar(navController) },
                                    snackbarHost = {
                                        SnackbarHost(hostState = snackbarHostState) { snackbarData ->
                                            val message = snackbarData.visuals.message
                                            when (resolveSnackbarVariant(snackbarData.visuals)) {
                                                SnackbarVariant.Error -> Acerola.Component.SnackbarError(message)
                                                SnackbarVariant.Success -> Acerola.Component.SnackbarSuccess(message)
                                                SnackbarVariant.Warn -> Acerola.Component.SnackbarWarn(message)
                                            }
                                        }
                                    },
                                    // Em landscape, o Scaffold M3 deve respeitar apenas os insets
                                    // do topo e da direita (a sidebar já cuida do lado esquerdo)
                                    contentWindowInsets =
                                        WindowInsets.safeDrawing.only(
                                            WindowInsetsSides.Top + WindowInsetsSides.End + WindowInsetsSides.Bottom,
                                        ),
                                ) { padding ->
                                    val contentPadding = if (applyScaffoldPadding) padding else PaddingValues(all = 0.dp)
                                    Box(modifier = Modifier.padding(paddingValues = contentPadding)) {
                                        NavHost(navController, startDestination) { setupNavGraph(context = this@BaseActivity, navController) }
                                    }
                                }
                            }
                        }
                    } else {
                        // Em modo portrait: layout original com bottom bar
                        Acerola.Component.Scaffold {
                            val hazeState = rememberHazeState()
                            Scaffold(
                                topBar = { TopBar(navController) },
                                snackbarHost = {
                                    SnackbarHost(hostState = snackbarHostState) { snackbarData ->
                                        val message = snackbarData.visuals.message
                                        when (resolveSnackbarVariant(snackbarData.visuals)) {
                                            SnackbarVariant.Error -> Acerola.Component.SnackbarError(message)
                                            SnackbarVariant.Success -> Acerola.Component.SnackbarSuccess(message)
                                            SnackbarVariant.Warn -> Acerola.Component.SnackbarWarn(message)
                                        }
                                    }
                                },
                                bottomBar = { BottomBar(navController, hazeState) },
                            ) { padding ->
                                val contentPadding = if (applyScaffoldPadding) padding else PaddingValues(all = 0.dp)
                                Box(
                                    modifier =
                                        Modifier
                                            .padding(paddingValues = contentPadding)
                                            .hazeSource(hazeState),
                                ) {
                                    NavHost(navController, startDestination) { setupNavGraph(context = this@BaseActivity, navController) }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    @Composable
    open fun TopBar(navController: NavHostController) {
    }

    @Composable
    open fun BottomBar(
        navController: NavHostController,
        hazeState: HazeState,
    ) {
    }

    @Composable
    open fun SideBar(navController: NavHostController) {
    }
}
