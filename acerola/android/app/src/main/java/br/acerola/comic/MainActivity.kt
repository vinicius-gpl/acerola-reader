package br.acerola.comic

import android.content.Context
import android.os.Bundle
import androidx.compose.animation.AnimatedVisibilityScope
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.produceState
import androidx.compose.ui.Modifier
import androidx.navigation.NavBackStackEntry
import androidx.navigation.NavGraphBuilder
import androidx.navigation.NavHostController
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import br.acerola.comic.common.activity.BaseActivity
import br.acerola.comic.common.navigation.Destination
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.BottomBar
import br.acerola.comic.common.ux.component.SideBar
import br.acerola.comic.config.preference.OnboardingPreference
import br.acerola.comic.module.main.Main
import br.acerola.comic.module.main.config.Screen
import br.acerola.comic.module.main.history.Screen
import br.acerola.comic.module.main.home.Screen
import br.acerola.comic.module.main.pattern.FilePatternScreen
import br.acerola.comic.module.main.sync.Screen
import br.acerola.comic.module.main.tutorial.Screen
import br.acerola.comic.ui.R
import dagger.hilt.android.AndroidEntryPoint
import dev.chrisbanes.haze.HazeState

@AndroidEntryPoint
class MainActivity(
    override val startDestinationRes: Int = R.string.navigation_launcher,
) : BaseActivity() {
    override val applyScaffoldPadding: Boolean = false

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.TIRAMISU) {
            requestPermissions(arrayOf(android.Manifest.permission.POST_NOTIFICATIONS), 0)
        }
    }

    override fun NavGraphBuilder.setupNavGraph(
        context: Context,
        navController: NavHostController,
    ) {
        composable(route = context.getString(R.string.navigation_launcher)) {
            LauncherScreen(context = context, navController = navController)
        }
        defaultComposable(context, Destination.TUTORIAL) {
            Main.Tutorial.Template.Screen(
                onNavigateToHome = {
                    navController.navigate(context.getString(Destination.HOME.route)) {
                        popUpTo(context.getString(R.string.navigation_launcher)) { inclusive = true }
                    }
                },
            )
        }
        defaultComposable(context, Destination.HOME) {
            Main.Home.Template.Screen(
                onNavigateToConfig = {
                    navController.navigate(context.getString(Destination.CONFIG.route))
                },
            )
        }
        defaultComposable(context, Destination.HISTORY) {
            Main.History.Template.Screen()
        }
        defaultComposable(context, Destination.CONFIG) {
            Main.Config.Template.Screen(
                onNavigateToTemplates = {
                    navController.navigate(context.getString(Destination.PATTERN.route))
                },
            )
        }
        defaultComposable(context, Destination.PATTERN) {
            Main.Pattern.Template.FilePatternScreen(
                onBack = { navController.popBackStack() },
            )
        }
        defaultComposable(context, Destination.SYNC) {
            Main.Sync.Template.Screen()
        }
    }

    @Composable
    override fun TopBar(navController: NavHostController) {
    }

    @Composable
    override fun BottomBar(
        navController: NavHostController,
        hazeState: HazeState,
    ) {
        val currentEntry by navController.currentBackStackEntryAsState()
        val currentRoute = currentEntry?.destination?.route
        val hiddenRoutes =
            setOf(
                getString(R.string.navigation_launcher),
                getString(Destination.TUTORIAL.route),
                getString(Destination.PATTERN.route),
            )
        if (currentRoute !in hiddenRoutes) {
            Acerola.Component.BottomBar(navController, hazeState)
        }
    }

    @Composable
    override fun SideBar(navController: NavHostController) {
        val currentEntry by navController.currentBackStackEntryAsState()
        val currentRoute = currentEntry?.destination?.route
        val hiddenRoutes =
            setOf(
                getString(R.string.navigation_launcher),
                getString(Destination.TUTORIAL.route),
                getString(Destination.PATTERN.route),
            )
        if (currentRoute !in hiddenRoutes) {
            Acerola.Component.SideBar(navController)
        }
    }

    @Composable
    private fun LauncherScreen(
        context: Context,
        navController: NavHostController,
    ) {
        val isCompleted by produceState<Boolean?>(initialValue = null) {
            OnboardingPreference.isCompletedFlow(context).collect { value = it }
        }

        LaunchedEffect(isCompleted) {
            val completed = isCompleted ?: return@LaunchedEffect
            val destination =
                if (completed) {
                    context.getString(Destination.HOME.route)
                } else {
                    context.getString(Destination.TUTORIAL.route)
                }
            navController.navigate(destination) {
                popUpTo(context.getString(R.string.navigation_launcher)) { inclusive = true }
            }
        }

        Box(modifier = Modifier.fillMaxSize())
    }

    private fun NavGraphBuilder.defaultComposable(
        context: Context,
        destination: Destination,
        content: @Composable AnimatedVisibilityScope.(NavBackStackEntry) -> Unit,
    ) {
        composable(
            route = context.getString(destination.route),
            enterTransition = {
                scaleIn(
                    initialScale = 0.8f,
                    animationSpec = tween(durationMillis = 300),
                ) + fadeIn(animationSpec = tween(durationMillis = 300))
            },
            exitTransition = {
                scaleOut(
                    targetScale = 0.8f,
                    animationSpec = tween(durationMillis = 300),
                ) + fadeOut(animationSpec = tween(durationMillis = 300))
            },
            popEnterTransition = {
                scaleIn(
                    initialScale = 1.2f,
                    animationSpec = tween(durationMillis = 300),
                ) + fadeIn(animationSpec = tween(durationMillis = 300))
            },
            popExitTransition = {
                scaleOut(
                    targetScale = 1.2f,
                    animationSpec = tween(durationMillis = 300),
                ) + fadeOut(animationSpec = tween(durationMillis = 300))
            },
            content = content,
        )
    }
}
