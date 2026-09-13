# Acerola Desktop — TODO

> Lista de features já implementadas removida daqui — histórico completo no `git log`. Este
> arquivo lista só o que está realmente em aberto, com estado confirmado em auditoria de
> código (11/09/2026).

## Crítico

- [ ] **`FsStore` do iroh-blobs trava ao abrir store em disco (mitigado)** — Mitigado com
      `.blobs(IrohBlobsConfig::mem())` em `bios/network.rs` — blobs não persistem entre
      reinícios. Causa raiz rastreada em [`lib/p2p/TODO.md`](../../lib/p2p/TODO.md).

## Alta

- [ ] **Validar encerramento de conexões/blobs — sessões só voltam ao fechar o app (lado
      Android)** — Log do Android: `timed out reading library summary`, sem recuperação até
      reabrir o app. Suspeita original: este lado (Desktop) inicia uma sessão
      `acerola/browse-cover/1` e não a finaliza corretamente, deixando o Android preso
      esperando.
      **Investigado (11/09/2026), teoria descartada:** a suspeita de "stream nunca fechado"
      não se sustenta — nenhum `Handler` (`cover_browse_handler.rs`, `library_browse_handler.rs`,
      etc.) chama `finish()`/`shutdown()` explícito no `SendStream`, mas isso não é o problema:
      `quinn::SendStream::drop` (`quinn-0.11.9/src/send_stream.rs:344`) já chama `finish()`
      automaticamente ao ser descartado (só cai pra `reset()` se o peer já tinha mandado
      `STOP_SENDING`). Também descartada a hipótese de um handler travado bloquear os outros:
      `NetworkManager::handle_incoming` (`lib/p2p/src/core/network/manager.rs:238-289`) roda
      cada conexão aceita em uma `tokio::spawn` própria — uma sessão presa não impede novas
      conexões/streams de serem aceitas e despachadas. Causa raiz continua desconhecida; precisa
      de reprodução ao vivo com tracing na camada de conexão do iroh (não dá pra ver daqui se é
      exaustão de `max_concurrent_bidi_streams`, um lock específico do app, ou outra coisa).
- [ ] **`browse-library` — fix aplicado no Android, aguardando confirmação ao vivo** — O lado
      Desktop (inbound) já estava correto; o fix foi só no Android (outbound). Pendente:
      rebuild+reinstall lá e confirmar.
- [ ] **`BlobNotFound` esporádico em transferências** — Causa raiz corrigida no `acerola-p2p`
      compartilhado (tag permanente antes do fetch) e `cargo update -p acerola-p2p` já rodado nos
      dois apps. Pendente: confirmar ao vivo.
- [ ] **Trocar pra um relay que um peer não compartilha corta o alcance sem aviso** —
      `RelayModeConfig::resolve` monta um `RelayMap` fechado; trocar de relay próprio deixa peers
      que não usam esse relay inalcançáveis até convergirem. Falta aviso na UI antes de trocar.
- [ ] **[To Fix] Recompilação do app quebra o handshake P2P (mitigado)** — Suspeito
      identificado: `shutdown()` não era chamado no encerramento normal do app. Mitigado (`lib.rs`
      agora chama `bios::shutdown_network` em `RunEvent::Exit`). Pendente: confirmar ao vivo que
      isso elimina o sintoma; se persistir, investigar o timeout de keep-alive do relay.

## Média

- [ ] **Editar template** — Update das tabelas locais relacionadas via Tauri Invoke.
- [ ] **"Reescanear quadrinho completo": unificar entre os dois apps** — Preferência: remover
      daqui em vez de adicionar no Android.
- [ ] **Automatizar submissão do MSIX na Microsoft Store via CD** — Publicação no Partner
      Center ainda é manual. Adicionar a `msstore` CLI no `desktop-release.yml` pra
      preparar/atualizar a submissão a cada release, parando antes do envio pra certificação.
- [ ] **App conseguir ficar em segundo plano com ícone escondido** — Poder colapsar em
      segundo plano/bandeja do sistema pra tarefas demoradas sem precisar deixar a janela aberta.
- [ ] **Sync MangaDex/AniList: corrida entre sync individual e "sync all" causa falso erro
      "already exists"** — `sync_all_metadata_mangadex` roda como `tokio::spawn` solto (Config),
      sem lock por quadrinho, e não compartilha estado com o sync individual de um quadrinho
      (`use-metadata-sync.svelte.ts`, que só desabilita o próprio botão). Se o sync em lote
      ainda tá processando um quadrinho e o usuário dispara o sync individual dele, os dois
      fazem check-then-insert em `upsert_metadata` sem coordenação — o segundo bate na
      constraint UNIQUE (`DbError::UniqueViolation`) e vira `ComicError::AlreadyExists`
      (`infra/error/comic.rs:6-9`), a mesma mensagem genérica usada pra "quadrinho duplicado",
      um caso bem diferente. Falta lock por `comic_directory_fk` (ou desabilitar sync individual
      enquanto um lote tiver rodando).
- [ ] **Toasts de erro mostram o texto cru em inglês em vez de traduzir** — Existe um
      mapeamento certo `errorType → Paraglide` (`COMIC_ERROR_MESSAGES`/`resolveErrorMessage()`
      em `lib/contracts/errors/errors.i18n.ts`), mas nenhum toast usa essa função. Os toasts de
      `comic/[folderName]/+page.svelte` (MangaDex, AniList, P2P, send-to-peer) usam
      `extractErrorMessage(err)` (`lib/utils/error.utils.ts:7`), que devolve `payload.message`
      cru (o `Display` em inglês do Rust) e ignora o `errorType` tipado que já vem no mesmo
      payload. Trocar essas chamadas por `resolveErrorMessage`.

## UI/UX

- [ ] **Animações em botões de sync e afins** — Hoje o feedback de clique/loading é estático
      (spinner parado), sem transição nenhuma.
- [ ] **Melhorar os ícones do Desktop de forma geral**
- [ ] **Trazer o conceito de "hero button" do Android pro Desktop** — No Android, um botão
      marcado/ativo ganha contorno + ícone de destaque; hoje o estado "selecionado" no Desktop é
      mais discreto que isso.
- [ ] **Padronizar TODOS os ícones-de-ação com bg "contorno" — sem exceção (compartilhado com
      Android)** — Escopo ampliado (13/09/2026): não é só "ícone rosa", é a mistura de estilos
      que existe hoje — alguns ícones já têm bg colorido fixo, outros (ex.: `AcerolaButtonIcon`
      de refresh/trash em `acerola-network-transfers-log.svelte`, e vários outros pela tela de
      Rede/Config/header) não têm bg nenhum em repouso, só ganham cor no hover. Regra: **todo**
      ícone de ação (`AcerolaButtonIcon`, ícone do `AcerolaHeroButton`, etc) precisa de chip com
      bg colorido por padrão, não só no hover. Ícones semânticos (destructive = vermelho, status
      do log de transferências = cor do status) usam a própria cor semântica como bg+ícone;
      ícones sem semântica própria (refresh, editar, ações neutras) usam o token de accent novo
      (`--accent-hero`, já adicionado em `feature/ui-ux-polish` nos 4 temas). Piloto já feito
      nessa branch: ícone de Bookmark do `AcerolaComicActionDialog`. Falta o resto:
      `acerola-network-transfers-log`, `acerola-network-peer-list`,
      `acerola-network-relay-settings-card`, header (`+layout.svelte`), toolbar da tela de
      Comic, Config, etc. Mesmo pedido no Android — ver
      [`acerola/android/TODO.md`](../android/TODO.md).

## Baixa

- [ ] **Otimizar busca/navegação da biblioteca remota** — É webview, dá pra fazer melhor
      (paginação/virtualização, layout mais claro do que o outro dispositivo tem).
- [ ] **Botão de sync de histórico na tela de histórico** _(talvez)_

## Referência: itens que o doc dizia em aberto mas já estão corrigidos (auditoria 11/09/2026)

- `Tela de rede: remover dispositivo pareado` — **já implementado**
  (`acerola-network-peer-list.svelte`, ação "Remove" com `removePeer` →
  `NETWORK_COMMANDS.removePairedPeer` → `NetworkServiceApi::remove_peer`).
- `Protocolo de sync de arquivos não leva o quadrinho 100%` — **já corrigido**
  (`build_manifest`/`build_manifest_for_comic` já incluem cover/banner/ComicInfo.xml,
  `restrict_manifest_to_chapters` nunca filtra esses extras).
- `Conflito de sync (quadrinho existente nos dois lados) está quebrado` — **detecção/relato
  corrigidos**: `FileSyncService::diff_wanted` agora distingue "nunca vi esse capítulo" de "já
  tenho, checksum diferente" (conflito de verdade), e `sync:files:complete`/
  `sync:comic:complete` carregam `conflicts` até o log de transferências da tela de Rede — um
  total por sessão, sem toast novo. Continua sobrescrevendo com a versão do peer (comportamento
  inalterado); resolução de conflito de verdade (escolher lado vencedor) fica pra depois.
