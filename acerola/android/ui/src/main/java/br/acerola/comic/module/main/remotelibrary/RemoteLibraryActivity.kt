package br.acerola.comic.module.main.remotelibrary

import android.content.Context
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.navigation.NavGraphBuilder
import androidx.navigation.NavHostController
import androidx.navigation.compose.composable
import br.acerola.comic.common.activity.BaseActivity
import br.acerola.comic.common.navigation.Destination
import br.acerola.comic.module.main.Main
import dagger.hilt.android.AndroidEntryPoint

/**
 * Tela cheia de biblioteca remota de UM peer, aberta a partir do FAB da Home — mesmo padrão de
 * [br.acerola.comic.module.comic.ComicActivity]/[br.acerola.comic.module.reader.ReaderActivity]
 * (Activity própria + dados via Intent extra, não NavArgs, já que o `peerId` é escolhido antes
 * de navegar, via [br.acerola.comic.module.main.common.component.PeerPickerSheet]).
 */
@AndroidEntryPoint
class RemoteLibraryActivity(
    override val startDestinationRes: Int = Destination.REMOTE_LIBRARY.route,
) : BaseActivity() {
    object PeerExtra {
        const val PEER_ID = "PEER_ID"
        const val PEER_DISPLAY_NAME = "PEER_DISPLAY_NAME"
    }

    private val peerId: String? by lazy { intent?.getStringExtra(PeerExtra.PEER_ID) }
    private val peerDisplayName: String? by lazy { intent?.getStringExtra(PeerExtra.PEER_DISPLAY_NAME) }

    override fun NavGraphBuilder.setupNavGraph(
        context: Context,
        navController: NavHostController,
    ) {
        composable(route = context.getString(Destination.REMOTE_LIBRARY.route)) {
            val id = peerId
            if (id != null) {
                Main.RemoteLibrary.Template.Screen(
                    peerId = id,
                    peerDisplayName = peerDisplayName ?: id,
                    onBack = { finish() },
                )
            } else {
                LaunchedEffect(Unit) {
                    finish()
                }
            }
        }
    }

    @Composable
    override fun TopBar(navController: NavHostController) = Unit
}
