# TODO — Acerola

Monorepo com 4 componentes versionados independentemente (`lib/p2p`, `acerola/android`,
`acerola/desktop`, `docs/web`). Este arquivo é o **TODO único e centralizado** do monorepo —
até 17/09/2026 cada componente tinha seu próprio `TODO.md`, mas manter 4 arquivos sincronizados
ficou ruim de manipular; tudo foi migrado pra cá, organizado por componente.

## Bloqueios principais antes do beta v1.0.0

Levantado em auditoria de release (11/09/2026), atualizado em 17/09/2026.

- [x] **Conflito de sync (quadrinho existente nos dois lados) — detecção/relato corrigidos** —
  Android e Desktop agora distinguem "capítulo ausente" de "capítulo já existe com checksum
  diferente" (conflito de verdade) e reportam a contagem na notificação/log de transferências
  da sessão, sem toast e sem uma notificação por capítulo. Decisão de escopo: continua
  sobrescrevendo com a versão do peer (comportamento inalterado) — resolução de conflito de
  verdade (escolher qual lado vence) fica pra depois, isso só resolve o falso-negativo
  relatado (sessão que termina bem mas mostra erro).
- [x] **Página de instalação dos docs é placeholder** — **corrigido**: conteúdo real (EN +
  PT-BR) com requisitos por plataforma e aviso sobre SmartScreen/instalação de fonte
  desconhecida, ver seção "Docs (web)" abaixo.
- [ ] **README.md desatualizado** — ainda diz "docs/web (ainda vazio)"; não é mais verdade
  (docs já em produção, v1.0.18).
- [ ] **Canal `prod` da CI nunca foi exercitado** — toda tag até hoje usou `alpha` ou `beta`
  (`git tag -l`; `beta` só passou a existir em 12/09/2026). O upload pro Cloudflare R2
  (Android, só em `prod`) e o `prerelease=false` nunca rodaram de verdade. Fazer um dry-run
  antes de depender disso pro lançamento em si.
- [ ] **`p2p-release.yml` não publica em lugar nenhum de verdade** — só sobe um
  `actions/upload-artifact` de CI, não `cargo publish` nem GitHub Release. Confirmar se isso é
  intencional (lib consumida só via path/git dependency) antes do beta.
- [ ] **Sem convenção documentada de versão entre os 4 componentes** — hoje cada um versiona
  independente (`p2p` 1.0.4, `desktop` 1.0.1, `android` 1.0.2, `docs` 1.0.18). Decidir o que
  "v1.0.0 beta" significa como selo conjunto antes de anunciar publicamente.
- [ ] **`FsStore` do `iroh-blobs` trava ao abrir store em disco** — mitigado em Android e
  Desktop com blob store em memória (blobs não persistem entre reinícios). Causa raiz vive no
  `lib/p2p` — ver seção "lib/p2p" abaixo.
- [ ] **Sistema de cobertura (Codecov) não recebe `wdio`, e2e (Playwright) nem testes
  instrumentados/emulador do Android — só unit tests** — `codecov.yml` só tem flags pra
  Kover (Android unit), `llvm-cov` (Desktop Rust) e `vitest` (Desktop/Docs Svelte). Nenhum dos
  três tipos de teste "de fora pra dentro" alimenta a cobertura hoje. Detalhes em
  "Android > Testes" e "Desktop > Testes" abaixo.
- [ ] **Android está fraco de testes — FFI, código em geral e testes instrumentados/emulador**
  — ver "Android > Testes" abaixo; é o maior gap de qualidade entre os 4 componentes hoje.

---

## lib/p2p

### Crítico

- [ ] **`FsStore` do iroh-blobs trava ao abrir store em disco** — Causa raiz não encontrada em
  `FsStore::load_with_opts` (crate `iroh-blobs`). Reproduzido isolado em
  `core/blobs/iroh/mod.rs::tests::fs_store_load_does_not_hang`. Mitigado nos apps consumidores
  (Android e Desktop) com `.blobs(IrohBlobsConfig::mem())` — blobs não persistem entre
  reinícios enquanto isso não for resolvido.

### Alta

- [ ] **Troca dinâmica de relay em runtime** — Hoje `RelayModeConfig` (4 variantes:
  `MdnsOnly`, `Custom`, `AcerolaOwn`, `IrohDefault`) só é configurável no builder, antes do
  node subir (`IrohTransportBuilder::relay_mode`, `core/transport/iroh/relay_mode.rs`). Falta:
  - [ ] `NetworkCommand::SwitchRelay` no `NetworkManager`
  - [ ] `node.switch_relay(mode).await` público em `AcerolaP2p`, preservando identidade e
    `known_peers`
  - [ ] Persistência da preferência de relay no `P2PStorage`
  - [ ] Evento `"network:relay_changed"` via `EventEmitter`
  - [ ] Testes de troca dinâmica + persistência

### Média

- [ ] **Fechar 51 gaps de mutation testing** — `cargo mutants --package acerola-p2p` (job
  semanal/manual, não bloqueia merge). Por arquivo:
  - `core/blobs/iroh/mod.rs` — `put`/`get`/`has`/`remove`/`fetch`, `parse_endpoint_addr`
  - `core/blobs/iroh/config.rs` — `mem`/`fs`/`build_store`, `gc_config`
  - `core/blobs/iroh/gc.rs` — `untag`/`tags_for_hash`
  - `core/transport/iroh/blobs_bridge.rs` — `wants_alpn`/`configure`/`as_capability`/`try_accept`
  - `core/transport/iroh/transport.rs` — `open_bi`, `latency`, `blobs()`, `is_connected`
  - `core/transport/iroh/builder.rs` — `blobs()`
  - `api/acerola_p2p.rs` — `connected_peers`/`blobs`
  - `infra/error/iroh_blobs.rs` — conversões `From<...> for ConnectionError`
  - `src/tests/mock_transport.rs`, `api/acerola_builder.rs` — mutantes sobrevivendo em código
    auxiliar de teste

### Referência: itens que o doc dizia em aberto mas já estão implementados (auditoria 11/09/2026)

- `Interface de configuração de relay (modo estático)` — **já implementado**, com design
  diferente do rascunho original: `RelayModeConfig` (`core/transport/iroh/relay_mode.rs`),
  `.relay_mode()` no `IrohTransportBuilder`, exportado em `api::mod`. Só falta a troca
  *dinâmica* em runtime (ver "Alta" acima).
- `Abstração de blobs` — **concluída**: `P2pBlobStore`, `IrohBlobStore`, `InMemoryBlobStore`,
  transferência real via `fetch`/`put`/`get`, suíte de testes (unitários, round-trip real,
  estresse, mutation testing).
- `Race de GC causando BlobNotFound esporádico` — **corrigida**: tag permanente do blob criada
  antes do fetch começar, não depois.

---

## Android

### Crítico

- [ ] **`FsStore` do iroh-blobs trava e vira ANR (mitigado, causa raiz aberta)** — Mitigado
  trocando `.blobs(IrohBlobsConfig::fs(...))` por `.mem()` em `native/rust/src/api.rs` —
  funciona, mas blobs não persistem entre reinícios do app. Causa raiz é do `iroh-blobs` em
  si, rastreada em "lib/p2p > Crítico" acima.

### Alta

- [ ] **Validar encerramento de conexões/blobs — sessões só voltam ao fechar o app** — Log ao
  vivo: `browse:library:error -> "stream failed: timed out reading library summary"`, sem
  recuperação até reabrir o app. Suspeita original: o Desktop inicia uma sessão
  `acerola/browse-cover/1` e não a finaliza corretamente do lado dele.
  **Investigado (11/09/2026), teoria descartada:** nenhum `Handler` chama `finish()`/
  `shutdown()` explícito no `SendStream`, mas `quinn::SendStream::drop` já faz isso sozinho
  (só cai pra `reset()` se o peer já tinha mandado `STOP_SENDING`) — não é a causa. Também
  descartada a hipótese de um handler travado bloquear os outros: `NetworkManager::handle_incoming`
  roda cada conexão aceita numa `tokio::spawn` própria. Causa raiz continua desconhecida; precisa
  de reprodução ao vivo com tracing na camada de conexão do iroh.
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

### Média

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
- [ ] **Melhorar responsividade mobile de alguns componentes** — Alguns componentes parecem ter
  puxado o formato/layout do Desktop mais ou menos direto. **Disclaimer:** cuidado ao copiar
  1:1 — o Desktop é pensado pra tela horizontal (landscape/wide), Android é majoritariamente
  vertical (portrait), então o que funciona bem lá pode ficar ruim aqui sem adaptação. Falta
  levantar quais componentes especificamente precisam de revisão.
- [ ] **Link estável de download do APK** — CI (`android-release.yml`) já sobe o APK pro
  Cloudflare R2 quando o canal é `prod`, mas nenhuma tag até hoje usou `prod` (só `alpha`/
  `beta`) — esse caminho nunca rodou de verdade. `android/latest/acerola-{version}.apk` +
  `android/releases/acerola-{version}.apk` (arquivo permanente) e a rota
  `/api/apk-latest` no `docs/web` (resolve a maior versão via `R2Bucket.list`) já resolvem o
  nome de arquivo feio e dão histórico versionado. Falta só: confirmar o dry-run contra um
  tag `prod` de verdade.

### Baixa

- [ ] **Botão de limpar histórico na tela de histórico**
- [ ] **Botão de sync de histórico na tela de histórico** *(talvez)*

### UI/UX

- [ ] **Componente de Input/TextInput com visual próprio** — Hoje usa o estilo default do
  Material; quero um formato diferente/mais autoral (mesmo esforço de padronização visual do
  item de ícones abaixo).
- [x] **Capítulo: trocar o dialog de metadados por `AdaptiveSheet`** — feito: menu de "mais
  opções" do capítulo alinhado ao padrão visual do `ComicActionsSheet` da Home.
- [x] **Home: abrir quadrinho no `AdaptiveSheet` com mais metadados** — feito: bottom sheet da
  Home agora mostra autor/ano/status/gêneros do quadrinho.
- [ ] **Padronizar TODOS os ícones-de-ação com bg "contorno" — sem exceção (compartilhado com
  Desktop)** — Escopo ampliado (13/09/2026): não é só "ícone rosa", é a mistura de estilos que
  já existe hoje entre telas — `HeroButton` (bg colorido + contorno) só é usado no Settings;
  fora dele, ícones de ação usam `Icon` cru ou o wrapper `ActionListItem` sem `iconBackground`
  (sem bg nenhum). Regra: **todo** ícone de ação precisa de chip com bg colorido por padrão.
  Ícones semânticos (destructive = vermelho, status = cor do status) usam a própria cor
  semântica como bg+ícone; ícones sem semântica própria (refresh, editar, ações neutras) usam o
  token de accent novo (`accentContainer`/`onAccentContainer`, já adicionado em
  `feature/ui-ux-polish` nos 4 temas). Piloto já feito nessa branch: ações neutras do
  `ComicActionsSheet` (Bookmark/Hide/Sync Push/Sync Pull) + Clear Metadata em vermelho. Falta o
  resto: FabGroup da Home, ícones soltos em telas fora do Settings, etc. Mesmo pedido no
  Desktop — ver "Desktop > UI/UX" abaixo.

### Testes

- [ ] **Android está fraco de testes de forma geral — FFI e resto do código** — o gap não é só
  o fluxo P2P (item abaixo): é estrutural. Duas frentes concretas:
  - [ ] **Camada de FFI (`native/rust`) sem harness de teste nenhum** — `P2PNode`
    (`native/rust/src/api.rs`) é glue ligada a `uniffi::Object`/`Runtime` e não tem nenhum
    teste próprio hoje (nem unitário, nem de integração). A lógica de negócio por trás (ex.:
    `reconnect_known_peers`) já é testada no `lib/p2p`, mas o marshalling da FFI em si —
    conversão de tipos, ciclo de vida do `Object`, comportamento em erro do lado Kotlin/JNI —
    não tem cobertura alguma. Precisa de um harness (mesmo que só smoke tests via JNI/JVM) antes
    do beta.
  - [ ] **Cobertura de teste desigual no resto do código, não só no fluxo P2P** — foi
    exatamente por isso que o gate de dados móveis (13/09/2026) só cobriu a tela de Sync na
    primeira tentativa — antes dessa sessão, `SyncViewModel`, os métodos de sync-com-peer de
    `HomeViewModel`/`HistoryViewModel` e o `RemoteLibraryViewModel` inteiro não tinham teste
    nenhum; ninguém percebeu em revisão que Home/Histórico/Quadrinho/Biblioteca remota disparam
    P2P por caminhos próprios. Parte já coberta na sessão (`SyncViewModelTest`/
    `RemoteLibraryViewModelTest` novos; casos de recusa — `DECLINED_MOBILE_DATA` — em
    `ComicViewModelTest`/`HomeViewModelTest`/`HistoryViewModelTest`). Ainda em aberto:
    - `MobileDataSyncViewModelTest`, `TransferLogViewModelTest`, `TutorialViewModelTest` —
      únicas ViewModels sem teste unitário.
    - `P2pSyncCoordinatorTest` (`core`) — singleton que persiste resultado de sync e dispara
      notificação/foreground service independente de qualquer tela estar aberta; hoje só é
      validado manualmente.
- [ ] **Testes instrumentados/emulador nunca rodam no CI** — `android-tests.yml` só executa
  `./gradlew test` (JVM puro); não existe job `connectedAndroidTest`/emulador no pipeline hoje.
  Isso combina com a ausência total de testes de UI instrumentados nas telas de rede —
  `SyncScreenTest`/`RemoteLibraryScreenTest`/`TransferLogScreenTest`/`TutorialScreenTest`
  (`androidTest`) não existem, enquanto todas as outras telas principais têm. Ou seja: mesmo que
  esses testes existissem hoje, não haveria onde rodá-los — falta o job de emulador antes de
  fazer sentido escrever os testes em si.
- [ ] **Cobertura (Codecov) não recebe nada de instrumentado/emulador** — consequência direta do
  item acima: `koverXmlReport` (usado no CI) só agrega os testes JVM de `./gradlew test`; sem
  job de emulador, não existe relatório de instrumentado pra sequer tentar subir.

### Referência: itens que o doc dizia em aberto mas já estão corrigidos (auditoria 11/09/2026)

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
- `P2PNode::restart não reconecta com peers pareados` — **corrigido**: `restart()`
  (`native/rust/src/api.rs`) agora chama `acerola_p2p::api::network::reconnect_known_peers`
  com `storage.load_peers()`, igual ao Desktop. Sem teste dedicado (ver "Testes" acima — camada
  de FFI sem harness) — a função reconectada em si já é testada no `lib/p2p`.

---

## Desktop

### Crítico

- [ ] **`FsStore` do iroh-blobs trava ao abrir store em disco (mitigado)** — Mitigado com
  `.blobs(IrohBlobsConfig::mem())` em `bios/network.rs` — blobs não persistem entre
  reinícios. Causa raiz rastreada em "lib/p2p > Crítico" acima.

### Alta

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

### Média

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

### UI/UX

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
      Comic, Config, etc. Mesmo pedido no Android — ver "Android > UI/UX" acima.

### Baixa

- [ ] **Otimizar busca/navegação da biblioteca remota** — É webview, dá pra fazer melhor
      (paginação/virtualização, layout mais claro do que o outro dispositivo tem).
- [ ] **Botão de sync de histórico na tela de histórico** _(talvez)_

### Testes

- [ ] **Suíte `wdio` (e2e real contra o app Tauri empacotado) nunca roda no CI** — existe suíte
      completa em `tests/wdio/` (specs de navegação do reader, PDF, etc.) e o script
      `npm run test:wdio` (`package.json`), mas nenhum job de `desktop-tests.yml` chama isso.
      Hoje só roda manualmente, quando alguém lembra.
- [ ] **Cobertura de e2e (Playwright) e `wdio` nunca chega no Codecov** — os jobs `e2e` de
      `desktop-tests.yml`/`docs-web-tests.yml` sobem o *relatório* do Playwright
      (`playwright-report`) como artifact, mas isso é resultado de teste, não cobertura de
      código — não há instrumentação de coverage nesses runs nem upload pro Codecov. Mesma
      lacuna vale pro `wdio` (que nem roda no CI ainda, ver item acima). `codecov.yml` só tem
      flags pra unit tests (`desktop-svelte`, `desktop-rust`, `docs-web`).

### Referência: itens que o doc dizia em aberto mas já estão corrigidos (auditoria 11/09/2026)

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

---

## Docs (web)

### Referência: itens já corrigidos (auditoria 11/09/2026)

- `Conteúdo real da página de instalação` — **corrigido**: removido o aviso de placeholder de
  `src/content/docs/{en,pt-br}/getting-started.md`, com requisitos por plataforma (Windows
  10/11, Android 8.0+), aviso sobre SmartScreen/instalação de fonte desconhecida fora das lojas
  oficiais (consistente com o README), e link pra `contributing-android` no trecho de build a
  partir do código-fonte. Validado com `npm run build` (23 páginas indexadas pelo pagefind, sem
  erro).
