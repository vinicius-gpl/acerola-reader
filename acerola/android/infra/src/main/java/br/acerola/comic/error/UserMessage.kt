package br.acerola.comic.error

import br.acerola.comic.type.UiText

interface UserMessage {
    val uiMessage: UiText

    // Todo tipo de erro (ChapterError, NetworkError, etc.) é UserMessage e é sempre falha —
    // só Raw precisa poder representar uma mensagem de sucesso, por isso o default aqui
    // cobre o resto sem exigir que cada sealed interface de erro implemente isso.
    val isSuccess: Boolean get() = false

    data class Raw(
        override val uiMessage: UiText,
        override val isSuccess: Boolean = false,
    ) : UserMessage {
        constructor(message: String) : this(UiText.DynamicString(message))
    }
}
