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
      reabrir o app. Suspeita: este lado (Desktop) inicia uma sessão `acerola/browse-cover/1` e
      não a finaliza corretamente, deixando o Android preso esperando.
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
