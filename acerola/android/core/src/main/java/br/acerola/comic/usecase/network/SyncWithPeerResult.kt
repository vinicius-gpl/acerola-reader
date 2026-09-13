package br.acerola.comic.usecase.network

/**
 * Resultado de [SyncComicWithPeerUseCase]/[SyncHistoryEntryWithPeerUseCase]/
 * [SyncHistoryWithPeerUseCase] — três estados em vez de um `Boolean` porque o chamador precisa
 * distinguir "não fez nada porque deu erro" (mostra um snackbar) de "não fez nada porque o
 * usuário recusou no diálogo de dados móveis" (não é erro nenhum, não mostra nada extra — o
 * próprio diálogo já comunicou a escolha).
 */
enum class SyncWithPeerResult {
    /** `peerId` não está mais pareado — nada foi disparado. */
    NOT_PAIRED,

    /** Usuário recusou usar dados móveis no diálogo (ver `MobileDataSyncGate`) — nada foi
     *  disparado, e quem chamou não deve marcar nenhum estado de "sincronizando" nem mostrar
     *  erro. */
    DECLINED_MOBILE_DATA,

    /** Sessão disparada de verdade — quem chamou pode marcar o spinner de "sincronizando" e
     *  esperar o [br.acerola.comic.service.network.P2pEvent] correspondente. */
    STARTED,
}
