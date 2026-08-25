package br.acerola.comic.module.main.remotelibrary.state

import br.acerola.comic.service.network.ComicSummary

data class RemoteLibraryUiState(
    val peerDisplayName: String = "",
    val comics: List<ComicSummary> = emptyList(),
    /** Distingue "ainda esperando `browse:library:result`" de "peer realmente tem biblioteca
     *  vazia" — [comics] sozinho não diferencia os dois assim que fica vazio. */
    val loaded: Boolean = false,
    val errorMessage: String? = null,
    /** `comicName` -> caminho local (`file://...`) da capa já baixada via
     *  `acerola/browse-cover/1`, pronto pro Coil carregar. */
    val coverPaths: Map<String, String> = emptyMap(),
    /** `comicName` da sessão `acerola/sync-comic/1` em andamento, ou `null` se nenhuma — só uma
     *  por vez, mesma restrição de protocolo que
     *  [br.acerola.comic.module.comic.ComicViewModel]'s `_syncingPeerId`. */
    val syncingComicName: String? = null,
)
