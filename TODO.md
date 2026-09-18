# TODO — Acerola

Monorepo com 4 componentes versionados independentemente (`lib/p2p`, `acerola/android`,
`acerola/desktop`, `docs/web`). Este arquivo é o **TODO único e centralizado** do monorepo —
até 17/09/2026 cada componente tinha seu próprio `TODO.md`, mas manter 4 arquivos sincronizados
ficou ruim de manipular; tudo foi migrado pra cá, organizado por componente.

> **18/09/2026 — decisões de escopo pro lançamento:**
> - Persistência de blobs entre reinícios (mitigação `IrohBlobsConfig::mem()`) **fica de fora
>   do escopo do beta** — não é mais tratado como bloqueio de prod, é comportamento aceito.
>   Causa raiz no `iroh-blobs` continua não resolvida, mas deixou de ser prioridade.
> - Não existe (e não existiu) pedido de convenção de versão conjunta entre os 4 componentes —
>   cada um segue versionando independente, sem selo unificado. Item removido daqui.
> - Auditoria de testes/cobertura (`wdio`, e2e, emulador Android, gaps de unit test) **adiada
>   de propósito** pra depois da leva de bugfixes/melhorias funcionais, pra poder revisar
>   testes de forma limpa sem código funcional mudando por baixo ao mesmo tempo. Os itens
>   continuam documentados nas seções "Testes" de cada componente, só não fazem parte da leva
>   de tasks atual.

## Bloqueios principais antes do beta v1.0.0

Levantado em auditoria de release (11/09/2026), revisado em 18/09/2026.

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
- [x] **README.md desatualizado** — **já não é mais verdade**: conferido em 18/09/2026, o
  README atual não menciona mais "docs/web ainda vazio". Item fechado.
- [ ] **Canal `prod` da CI nunca foi exercitado** — toda tag até hoje usou `alpha` ou `beta`
  (`git tag -l`; `beta` só passou a existir em 12/09/2026). O upload pro Cloudflare R2
  (Android, só em `prod`) e o `prerelease=false` nunca rodaram de verdade. Fazer um dry-run
  antes de depender disso pro lançamento em si.
- [ ] **`p2p-release.yml` não publica em lugar nenhum de verdade** — só sobe um
  `actions/upload-artifact` de CI, não `cargo publish` nem GitHub Release. Confirmar se isso é
  intencional (lib consumida só via path/git dependency) antes do beta.
- [ ] **Android está fraco de testes — FFI, código em geral e testes instrumentados/emulador**
  — ver "Android > Testes" abaixo. **Auditoria adiada pra depois da leva funcional atual**
  (decisão de 18/09/2026), mas é o maior gap de qualidade entre os 4 componentes.
- [ ] **Sistema de cobertura (Codecov) não recebe `wdio`, e2e (Playwright) nem testes
  instrumentados/emulador do Android — só unit tests** — detalhes em "Android > Testes" e
  "Desktop > Testes" abaixo. **Auditoria adiada**, mesma decisão acima.

---

## lib/p2p

### Crítico

- [ ] **`FsStore` do iroh-blobs trava ao abrir store em disco (mitigado; persistência fora de
  escopo)** — Causa raiz não encontrada em `FsStore::load_with_opts` (crate `iroh-blobs`).
  Reproduzido isolado em `core/blobs/iroh/mod.rs::tests::fs_store_load_does_not_hang`.
  Mitigado nos apps consumidores (Android e Desktop) com `.blobs(IrohBlobsConfig::mem())` —
  blobs não persistem entre reinícios. **Decisão (18/09/2026): persistência entre reinícios
  fica de fora do escopo do beta**, a mitigação em memória é o comportamento aceito pra prod;
  a causa raiz no `iroh-blobs` continua rastreada aqui só como referência, sem ser bloqueio.

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
  - Nota: enquanto isso não existe, Android e Desktop mitigam o sintoma (perda de alcance ao
    trocar de relay) com um aviso na UI antes de trocar — ver itens correspondentes abaixo.
    Essa feature resolveria o problema na raiz (troca sem reiniciar o node/perder alcance).

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

- [x] **`FsStore` do iroh-blobs trava e vira ANR — mitigado, aceito como final** — Mitigado
  trocando `.blobs(IrohBlobsConfig::fs(...))` por `.mem()` em `native/rust/src/api.rs` —
  funciona, o ANR está resolvido. A não-persistência de blobs entre reinícios que isso causa
  **não é mais tratada como bloqueio** (decisão 18/09/2026, ver topo do arquivo) — causa raiz
  segue rastreada em "lib/p2p > Crítico" só como referência.

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
  de reprodução ao vivo com tracing na camada de conexão do iroh. **Não é um fix de código
  isolável hoje — precisa de sessão de reprodução ao vivo antes de virar task.**
- [ ] **`browse-library` — fix aplicado, aguardando confirmação ao vivo** — Causa raiz já
  corrigida no código (`LibraryBrowseOutbound`/`run_outbound` agora escreve um marcador `{}`
  antes de esperar resposta, respeitando a regra do quinn de `open_bi()`/`accept_bi()`).
  Pendente: rebuild + reinstall no celular pra confirmar em produção. **QA manual, não código.**
- [ ] **`BlobNotFound` esporádico em sync ("Omoide Emanon" não trouxe todos os capítulos")** —
  Causa raiz corrigida no `acerola-p2p` compartilhado (tag permanente do blob criada antes do
  fetch, protegendo contra GC concorrente) e `cargo update -p acerola-p2p` já rodado.
  Pendente: confirmar ao vivo que resolveu de vez. **QA manual, não código.**
- [ ] **Trocar pra um relay que um peer pareado não compartilha corta o alcance sem aviso** —
  Mesma limitação de desenho já documentada no Desktop: `RelayModeConfig::resolve` monta um
  `RelayMap` fechado, então trocar de relay próprio deixa peers que não usam esse relay
  inalcançáveis até ambos convergirem. Falta aviso na UI antes de trocar.
  **Mapeado (18/09/2026):** tela `RelaySettingsCard` em
  `ui/src/main/java/br/acerola/comic/module/main/sync/SyncScreen.kt` (toggles "Usar relay do
  Acerola" / "Relays próprios" / "Usar rede pública Iroh") dispara direto via
  `SyncViewModel.onAction()` → `RelayPreference.set*` (DataStore) → o `collect` do `init`
  (`ui/.../sync/SyncViewModel.kt:98-125`) reage e chama `applyCurrentRelaySettingsLive()`
  (linha 123) → `p2pUseCase.applyRelaySettings(...)` (linha 325-342), aplicado **ao vivo** no
  node já rodando, sem diálogo intermediário nenhum. O botão manual "Reiniciar"
  (`SyncAction.RestartP2p`, linha 348-380) também não tem aviso. É aqui que entra a
  confirmação/aviso.

### Média

- [ ] **Sync individual: UI ainda não dispara `syncHistoryEntry`** — O protocolo de sync de
  uma única entrada de histórico (`acerola/sync-history-entry/1`) já está pronto e testado nos
  dois lados, mas nenhuma tela chama ele ainda — falta decidir o gatilho de UX (automático ao
  terminar de ler, ou botão manual).
  **Mapeado (18/09/2026):** toda a cadeia já existe e funciona, só não é chamada por nenhuma
  tela: FFI em `native/src/main/java/br/acerola/comic/p2p/acerola.kt:4083/4507`, service em
  `native/.../service/P2pService.kt:221-233`, use case de baixo nível em
  `core/.../usecase/network/P2pUseCase.kt:71-84`, e use case pronto pra UI
  `core/.../usecase/network/SyncHistoryEntryWithPeerUseCase.kt` (já testado, resolve
  `NOT_PAIRED`/`DECLINED_MOBILE_DATA`/`STARTED`). `ComicViewModel.kt:84` já injeta esse use
  case mas nunca chama (`syncHistoryEntryWithPeerUseCase(...)` não aparece fora de
  teste/comentário). Candidato de gatilho mais óbvio: um botão adicional ao lado do fluxo
  existente de "enviar capítulos pro peer" (`ChapterItem`/`ComicActionsSheet`, que hoje só
  dispara `syncComicWithPeerUseCase`, o protocolo de arquivo) — "sincronizar progresso de
  leitura" como ação separada de "sincronizar capítulos".
- [ ] **Melhorar busca/visualização da biblioteca remota** — Hoje está ruim de ver o conteúdo
  que o outro dispositivo tem.
- [x] **"Reescanear quadrinho completo": unificar entre os dois apps** — Desktop tem, Android
  não. **Decisão confirmada (18/09/2026): remover do Desktop**, não adicionar aqui — ver
  "Desktop > Alta/Referência" pros pontos exatos a remover.
- [ ] **Botão flutuante da Home: buscar quadrinhos no outro dispositivo + sincronizar tudo**
  *(talvez)* — Dois botões: um pra buscar/puxar quadrinhos específicos de um peer, outro numa
  sheet maior pra "sincronizar tudo que o peer tem" — esse último precisa de confirmação
  explícita antes de disparar.
  **Mapeado (18/09/2026):** o `FabGroup` da Home (`HomeScreen.kt:546-604`) **já tem** o 3º
  item de "buscar no outro dispositivo" (`Icons.Rounded.PhoneAndroid`, abre
  `PeerPickerSheet` → biblioteca remota) — essa parte já está feita. Falta só o botão de
  "sincronizar tudo", reaproveitando a lógica que já existe em `SyncScreen.kt:1153`
  (`SyncAction.SyncAll(peer.peerId)`, hoje sem confirmação lá também) e adicionando a
  confirmação explícita nos dois lugares.
- [ ] **Melhorar responsividade mobile de alguns componentes** — Alguns componentes parecem ter
  puxado o formato/layout do Desktop mais ou menos direto. **Disclaimer:** cuidado ao copiar
  1:1 — o Desktop é pensado pra tela horizontal (landscape/wide), Android é majoritariamente
  vertical (portrait), então o que funciona bem lá pode ficar ruim aqui sem adaptação.
  **Investigado (18/09/2026), inconclusivo via busca estática:** todos os componentes
  candidatos que têm origem no padrão do Desktop (`AccordionCard`, `ToggleCard`, `RadioGroup`,
  `RelaySettingsCard`/`RelayUrlListEditor`, `ConfigSection`) já têm adaptação deliberada e
  comentada pra toque/portrait. Único ponto de risco latente (não confirmado como bug real):
  `common/ux/component/RadioGroup.kt:27-48` (usado em `PaginationPreference`/
  `VolumeStylePreference`) é um `Row` sem `horizontalScroll`/wrap — pode estourar largura com
  fontes grandes de acessibilidade ou traduções longas, mas hoje só tem 2-3 opções curtas.
  Esse item precisa de inspeção visual/manual em dispositivo real, não dá pra fechar só com
  grep — fica como item de auditoria de UX, não uma task de código isolada.
- [ ] **Link estável de download do APK** — CI (`android-release.yml`) já sobe o APK pro
  Cloudflare R2 quando o canal é `prod`, mas nenhuma tag até hoje usou `prod` (só `alpha`) —
  esse caminho nunca rodou de verdade. `android/latest/acerola-{version}.apk` +
  `android/releases/acerola-{version}.apk` (arquivo permanente) e a rota
  `/api/apk-latest` no `docs/web` (resolve a maior versão via `R2Bucket.list`) já resolvem o
  nome de arquivo feio e dão histórico versionado. Falta só: confirmar o dry-run contra um
  tag `prod` de verdade. **Depende do dry-run de release, não é código isolado.**

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
  `ComicActionsSheet` (Bookmark/Hide/Sync Push/Sync Pull) + Clear Metadata em vermelho
  (obs.: no piloto, Clear Metadata/Delete usam só `tint`, sem `iconBackground` — conferir se é
  intencional antes de replicar o padrão). Falta o resto, mapeado em 18/09/2026:
  - `module/comic/component/ChapterItem.kt:337` (marcar lido/não lido) e `:356` (enviar
    capítulos pro peer) — sem `iconBackground`.
  - `module/main/transferlog/TransferLogScreen.kt:127-152` — botões refresh/limpar log são
    `FilledTonalButton` com ícone+texto, sem chip, equivalente direto ao item já citado no
    Desktop (`acerola-network-transfers-log.svelte`).
  - `common/ux/component/ActionIcon.kt` — sempre tem bg, mas fixo em `surfaceVariant`, não no
    token novo `accent`; usado em `ComicListItem.kt:392`, `ChapterSortSheet.kt:79`,
    `HomeFilterSheet.kt`, `HistoryScreen.kt`, `RemoteLibraryScreen.kt:107`,
    `TransferLogScreen.kt:109` — decidir se migra pro token de accent ou fica de fora por ser
    um componente genérico (não semântico).
  - `FabGroup.kt:147` (mini-FABs da Home) usa `containerColor = colorScheme.secondary` fixo —
    decidir se FAB entra no escopo de "ícone de ação" ou fica fora por ser outro tipo de
    componente (M3 FAB vs. lista de ação).
  - `ComicListItem.kt:372-388` — botão "continuar lendo" já tem bg (`secondary`), mas hardcoded
    via `Modifier.background(...)` em vez do componente padrão.

### Testes _(auditoria adiada — ver nota no topo do arquivo)_

- [ ] **Android está fraco de testes de forma geral — FFI e resto do código** — o gap não é só
  o fluxo P2P (item abaixo): é estrutural. Duas frentes concretas:
  - [ ] **Camada de FFI (`native/rust`) sem harness de teste nenhum** — `P2PNode`
    (`native/rust/src/api.rs`) é glue ligada a `uniffi::Object`/`Runtime` e não tem nenhum
    teste próprio hoje (nem unitário, nem de integração). A lógica de negócio por trás (ex.:
    `reconnect_known_peers`) já é testada no `lib/p2p`, mas o marshalling da FFI em si —
    conversão de tipos, ciclo de vida do `Object`, comportamento em erro do lado Kotlin/JNI —
    não tem cobertura alguma. Precisa de um harness (mesmo que só smoke tests via JNI/JVM).
  - [ ] **Cobertura de teste desigual no resto do código, não só no fluxo P2P** — foi
    exatamente por isso que o gate de dados móveis (13/09/2026) só cobriu a tela de Sync na
    primeira tentativa — antes dessa sessão, `SyncViewModel`, os métodos de sync-com-peer de
    `HomeViewModel`/`HistoryViewModel` e o `RemoteLibraryViewModel` inteiro não tinham teste
    nenhum. Parte já coberta (`SyncViewModelTest`/`RemoteLibraryViewModelTest` novos; casos de
    recusa — `DECLINED_MOBILE_DATA` — em
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
  (`androidTest`) não existem, enquanto todas as outras telas principais têm.
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

- [x] **`FsStore` do iroh-blobs trava ao abrir store em disco — mitigado, aceito como final** —
  Mitigado com `.blobs(IrohBlobsConfig::mem())` em `bios/network.rs` — blobs não persistem
  entre reinícios. **Decisão (18/09/2026): fora do escopo do beta**, ver nota no topo do
  arquivo. Causa raiz rastreada em "lib/p2p > Crítico" só como referência.

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
      de reprodução ao vivo com tracing na camada de conexão do iroh. **Não é um fix de código
      isolável hoje — precisa de sessão de reprodução ao vivo antes de virar task.**
- [ ] **`browse-library` — fix aplicado no Android, aguardando confirmação ao vivo** — O lado
      Desktop (inbound) já estava correto; o fix foi só no Android (outbound). Pendente:
      rebuild+reinstall lá e confirmar. **QA manual, não código.**
- [ ] **`BlobNotFound` esporádico em transferências** — Causa raiz corrigida no `acerola-p2p`
      compartilhado (tag permanente antes do fetch) e `cargo update -p acerola-p2p` já rodado nos
      dois apps. Pendente: confirmar ao vivo. **QA manual, não código.**
- [ ] **Trocar pra um relay que um peer não compartilha corta o alcance sem aviso** —
      `RelayModeConfig::resolve` monta um `RelayMap` fechado; trocar de relay próprio deixa peers
      que não usam esse relay inalcançáveis até convergirem. Falta aviso na UI antes de trocar.
      **Mapeado (18/09/2026):** tela `routes/network/components/acerola-network-relay-settings-card.svelte`
      (props `onToggleAcerolaRelay`/`onToggleIrohPublicNetwork`/`onAddCustomRelayUrl`/
      `onRemoveCustomRelayUrl`/`onRestart`) → `toggleAcerolaRelay`/`toggleIrohPublicNetwork`
      (linhas 143-149) chamam `runRestartingAction(...)` (linhas 76-87, já tem trava contra
      reentrância + feedback visual, mas nenhum aviso preventivo) → hook
      `lib/hooks/preferences/use-relay-settings.svelte.ts` persiste e chama
      `invoke(NETWORK_COMMANDS.applyRelaySettings)` → comando Tauri `apply_relay_settings`
      (`cmd/features/network/mod.rs:126-129` → `NetworkServiceApi::apply_relay_settings`,
      `core/services/network/mod.rs:286`), que **reinicia o node P2P inteiro**. É no card
      Svelte que entra a confirmação/aviso.
- [ ] **[To Fix] Recompilação do app quebra o handshake P2P (mitigado)** — Suspeito
      identificado: `shutdown()` não era chamado no encerramento normal do app. Mitigado (`lib.rs`
      agora chama `bios::shutdown_network` em `RunEvent::Exit`). Pendente: confirmar ao vivo que
      isso elimina o sintoma; se persistir, investigar o timeout de keep-alive do relay. **QA
      manual, não código.**
- [x] **"Reescanear quadrinho completo": unificar entre os dois apps** — **Decisão confirmada
      (18/09/2026): remover daqui** (Android não ganha a feature). Pontos exatos a remover,
      mapeados:
      - Rust: comando `deep_rescan_comic` (`cmd/features/comic/mod.rs:130-145`, registrado em
        `bios/mod.rs:94`), `ComicService::deep_rescan_comic`
        (`core/services/comic/mod.rs:50-60`), motor
        `ComicScannerService::deep_rescan_comic` (`core/services/archive/comic_scanner_engine.rs:327-336`)
        + testes (`comic_scanner_engine.rs:1117-1148`).
      - Svelte: contrato `HOME_COMMANDS.deepRescanComic` (`contracts/home/home.commands.ts:8`),
        handler `handleDeepRescanComic` (`routes/comic/[folderName]/+page.svelte:367-382`,
        estado `deepRescanning` linha 95), UI/diálogo em
        `routes/comic/[folderName]/components/acerola-comic-preferences.svelte` (prop
        `deepRescanning`/evento `onDeepRescanComic` linhas 27/39, botão linhas 378-394, diálogo
        de confirmação linhas 648-655).
      - i18n: mensagens Paraglide sob `pages.comic.preferences.file_sync.deep_rescan.*` e
        `pages.comic.toast.*deep_rescan*`.
      - **Cuidado:** `rescan_comic` (não-deep) é uma feature *separada*, definida em paralelo
        nos mesmos arquivos — não é o alvo, não remover.

### Média

- [ ] **Editar template** — Update das tabelas locais relacionadas via Tauri Invoke.
      **Mapeado (18/09/2026): não existe update em nenhuma camada hoje**, só create+delete —
      UI (`routes/config/templates/+page.svelte`, sem affordance de editar na listagem), hook
      (`hooks/store/use-archive-templates.svelte.ts`, comentário explícito "only ... created or
      deleted"), contrato (`contracts/archive/archive-template.commands.ts`, sem
      `updateArchiveTemplate`), comandos Rust (`cmd/features/archive/mod.rs`, sem update),
      serviço (`core/services/archive/archive_template_service.rs`, sem `update_template`).
      A vagueza do item original ("tabelas locais relacionadas") é a FK
      `comic_directory.archive_template_fk → archive_template.id`
      (`infra/db/migrations/models/archive/002_create_comic_directory.sql:8,11`) — editar o
      `pattern` de um template muda a regra de extração de capítulo/volume, então qualquer
      `comic_directory` que já usa esse template provavelmente precisa de um re-scan
      (`rescan_comic`/`deep_rescan_comic`, mas `deep_rescan_comic` está sendo removido, ver
      item acima) pra refletir o novo padrão. **Precisa de decisão de produto** (rescan
      automático em cascata vs. só avisar o usuário) antes de implementar — feature nova a
      construir do zero em todas as camadas, não bloqueia prod.
- [x] **Sync MangaDex/AniList: corrida entre sync individual e "sync all" causa falso erro
      "already exists"** — **já corrigido** (commit `64ed93d9`, 13/09/2026, antes desta
      auditoria). `MetadataService` (`core/services/metadata/mod.rs:80-91`) tem um
      `Mutex<HashSet<i64>>` (`syncing`, chaveado por `comic_directory_fk`) com RAII guard
      (`SyncGuard`), adquirido no topo de `sync_comic_mangadex`/`sync_comic_anilist`/
      `parse_and_sync_comic_info` — a segunda chamada concorrente recebe
      `ComicError::SyncInProgress` (já mapeado em `COMIC_ERROR_MESSAGES`, traduz certo) em vez
      de bater na constraint UNIQUE e virar `AlreadyExists`. Resta só um refinamento
      cosmético opcional, não bloqueia prod: `use-metadata-sync.svelte.ts` não sabe que um
      lote está rodando, então o botão individual continua clicável e o usuário vê o toast de
      `SyncInProgress` em vez do botão já vir desabilitado.
- [ ] **Toasts de erro mostram o texto cru em inglês em vez de traduzir** — **Atualizado
      (18/09/2026): a causa original (nenhum toast usando `resolveErrorMessage`) já foi
      corrigida no commit `ab78f9ff` (13/09/2026) — `extractErrorMessage`
      (`lib/utils/error.utils.ts:19-26`) já delega pra `resolveErrorMessage` quando o payload
      tem `errorType`+`message`. Mas sobraram dois bugs reais que ainda vazam texto cru:**
      1. `ErrorPayload::from(&ComicError)` (`cmd/events/shared/mod.rs:12-15`) usa
         `format!("{:?}", err)` (Debug) como `error_type` em vez do nome exato da variante.
         Pra variantes com dados (`InvalidRequest(String)`, `SystemFailure(String)`,
         `Io(std::io::Error)`) isso produz algo como `InvalidRequest("mensagem")`, que nunca
         bate com a chave exata `"InvalidRequest"` esperada em `COMIC_ERROR_MESSAGES`
         (`errors.i18n.ts`, lookup é match exato) — cai no fallback e mostra o texto cru.
         Além disso a chave mapeada no i18n é `"IoError"`, mas o variante real do enum
         (`infra/error/mod.rs:14-38`) é `Io` — essa chave **nunca bate**, nem sem dados.
      2. O listener do evento `metadata:sync_all:error`
         (`routes/config/+page.svelte:146-151`) pega `event.payload?.message` direto, sem
         passar por `extractErrorMessage`/`resolveErrorMessage` — sempre mostra o texto cru do
         Rust nesse toast específico (erro do sync em lote de MangaDex/AniList).

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
- [ ] **Automatizar submissão do MSIX na Microsoft Store via CD** — Publicação no Partner
      Center ainda é manual. Adicionar a `msstore` CLI no `desktop-release.yml` pra
      preparar/atualizar a submissão a cada release, parando antes do envio pra certificação.
      **Infra/CD, não bloqueia o beta em si.**
- [ ] **App conseguir ficar em segundo plano com ícone escondido** — Poder colapsar em
      segundo plano/bandeja do sistema pra tarefas demoradas sem precisar deixar a janela aberta.

### Testes _(auditoria adiada — ver nota no topo do arquivo)_

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
