# Acerola Android — TODO

> Lista de features já implementadas removida daqui — histórico completo no `git log`. Este
> arquivo lista só o que está realmente em aberto, com estado confirmado em auditoria de
> código (11/09/2026).

## Crítico

- [ ] **`FsStore` do iroh-blobs trava e vira ANR (mitigado, causa raiz aberta)** — Mitigado
  trocando `.blobs(IrohBlobsConfig::fs(...))` por `.mem()` em `native/rust/src/api.rs` —
  funciona, mas blobs não persistem entre reinícios do app. Causa raiz é do `iroh-blobs` em
  si, rastreada em [`lib/p2p/TODO.md`](../../lib/p2p/TODO.md).

## Alta

- [ ] **`P2PNode::restart` não reconecta com peers pareados** — Confirmado: `restart()`
  (`native/rust/src/api.rs:414-431`) reconstrói o node do zero e nunca chama
  `reconnect_known_peers` (zero referências no crate). A função já existe pronta, testada e
  exportada do `acerola-p2p` (`api::network::reconnect_known_peers`) — o Desktop já chama ela
  no próprio `restart()`. Fix aqui é só a chamada + passar `paired_peers()` do storage.
- [ ] **Validar encerramento de conexões/blobs — sessões só voltam ao fechar o app** — Log ao
  vivo: `browse:library:error -> "stream failed: timed out reading library summary"`, sem
  recuperação até reabrir o app. Suspeita: o Desktop inicia uma sessão
  `acerola/browse-cover/1` e não a finaliza corretamente do lado dele. Ainda sem fix.
- [ ] **`browse-library` — fix aplicado, aguardando confirmação ao vivo** — Causa raiz já
  corrigida no código (`LibraryBrowseOutbound`/`run_outbound` agora escreve um marcador `{}`
  antes de esperar resposta, respeitando a regra do quinn de `open_bi()`/`accept_bi()`).
  Pendente: rebuild + reinstall no celular pra confirmar em produção.
- [ ] **`BlobNotFound` esporádico em sync ("Omoide Emanon" não trouxe todos os capítulos")** —
  Causa raiz corrigida no `acerola-p2p` compartilhado (tag permanente do blob criada antes do
  fetch, protegendo contra GC concorrente) e `cargo update -p acerola-p2p` já rodado.
  Pendente: confirmar ao vivo que resolveu de vez.
- [ ] **Trocar pra um relay que um peer pareado não compartilha corta o alcance sem aviso** —
  Mesma limitação de desenho já documentada no Desktop: `RelayModeConfig::resolve` monta um
  `RelayMap` fechado, então trocar de relay próprio deixa peers que não usam esse relay
  inalcançáveis até ambos convergirem. Falta algum aviso na UI antes de trocar.

## Média

- [ ] **Sync individual: UI ainda não dispara `syncHistoryEntry`** — O protocolo de sync de
  uma única entrada de histórico (`acerola/sync-history-entry/1`) já está pronto e testado nos
  dois lados, mas nenhuma tela chama ele ainda — falta decidir o gatilho de UX (automático ao
  terminar de ler, ou botão manual).
- [ ] **Melhorar busca/visualização da biblioteca remota** — Hoje está ruim de ver o conteúdo
  que o outro dispositivo tem.
- [ ] **"Reescanear quadrinho completo": unificar entre os dois apps** — Desktop tem, Android
  não. Preferência: remover do Desktop em vez de adicionar aqui.
- [ ] **Botão flutuante da Home: buscar quadrinhos no outro dispositivo + sincronizar tudo**
  *(talvez)* — Dois botões: um pra buscar/puxar quadrinhos específicos de um peer, outro numa
  sheet maior pra "sincronizar tudo que o peer tem" — esse último precisa de confirmação
  explícita antes de disparar.
- [ ] **Link estável de download do APK** — CI (`android-release.yml`) já sobe o APK pro
  Cloudflare R2 quando o canal é `prod`, mas nenhuma tag até hoje usou `prod` (só `alpha`) —
  esse caminho nunca rodou de verdade. Falta: confirmar o dry-run, e depois resolver o nome de
  arquivo feio (`Content-Disposition` correto ou rota de redirect no `docs/web`).

## Baixa

- [ ] **Botão de limpar histórico na tela de histórico**
- [ ] **Botão de sync de histórico na tela de histórico** *(talvez)*

## Referência: itens que o doc dizia em aberto mas já estão corrigidos (auditoria 11/09/2026)

- `Protocolo de sync de arquivos não leva o quadrinho 100%` — **já corrigido**
  (`FileComicInfo` com cover/banner/ComicInfo.xml, `send_extras`/`receive_extras` em
  `exchange.rs`, commit `d0bcf7f1`). Não é mais um item em aberto.
- `Conflito de sync (quadrinho existente nos dois lados) está quebrado` — **detecção/relato
  corrigidos**: `missing_from` (`protocol/files/exchange.rs`) agora distingue "nunca vi esse
  capítulo" de "já tenho, checksum diferente" (conflito de verdade), e `FileSyncComplete`
  carrega `conflictsCount` até a notificação final de sync (`P2pSyncCoordinator`) — um total
  por sessão, sem toast, sem notificação por capítulo. Continua sobrescrevendo com a versão do
  peer (comportamento inalterado); resolução de conflito de verdade (escolher lado vencedor)
  fica pra depois.
