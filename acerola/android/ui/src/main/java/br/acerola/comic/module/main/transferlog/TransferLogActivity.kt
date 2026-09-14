package br.acerola.comic.module.main.transferlog

import android.content.Context
import androidx.compose.runtime.Composable
import androidx.navigation.NavGraphBuilder
import androidx.navigation.NavHostController
import androidx.navigation.compose.composable
import br.acerola.comic.common.activity.BaseActivity
import br.acerola.comic.common.navigation.Destination
import br.acerola.comic.module.main.Main
import dagger.hilt.android.AndroidEntryPoint

/**
 * Tela cheia de histórico de transferências P2P, aberta a partir do hero button da aba de
 * Rede — mesmo padrão de [br.acerola.comic.module.main.remotelibrary.RemoteLibraryActivity]
 * (Activity própria, não uma rota dentro do NavHost do `MainActivity`). Sem dados via Intent:
 * a própria [TransferLogViewModel] busca o histórico persistido sozinha.
 */
@AndroidEntryPoint
class TransferLogActivity(
    override val startDestinationRes: Int = Destination.TRANSFER_LOG.route,
) : BaseActivity() {
    override fun NavGraphBuilder.setupNavGraph(
        context: Context,
        navController: NavHostController,
    ) {
        composable(route = context.getString(Destination.TRANSFER_LOG.route)) {
            Main.TransferLog.Template.Screen(onBack = { finish() })
        }
    }

    @Composable
    override fun TopBar(navController: NavHostController) = Unit
}
