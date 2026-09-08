# AcerolaP2P — TODO: Interface de Configuração e Troca Dinâmica de Relay

Este documento detalha o planejamento para expor uma API intuitiva que permite aos aplicativos (Desktop, Android, CLI, etc.) configurar e alternar entre **4 modos de relay**, permitindo a construção de interfaces visuais e controle total da conectividade pelo usuário final.

---

## 🎯 Visão Geral dos 4 Modos de Relay

A camada de rede do Acerola deve expor um enum/interface de alto nível representando as 4 opções:

1. **Relay Oficial do Acerola (`RelayModeConfig::AcerolaDefault`)**:
   - Utiliza os servidores de relay oficiais mantidos pelo ecossistema Acerola.
   - Ideal para uso *out-of-the-box* pelos usuários finais dos apps.

2. **Relay Self-Hosted / Próprio (`RelayModeConfig::Custom(String)`)**:
   - Permite ao usuário informar uma URL HTTPS customizada de seu próprio servidor de relay Iroh (ex: `https://meu-relay.exemplo.com`).
   - Garante privacidade e autonomia total de infraestrutura para usuários avançados ou corporativos.

3. **Relay Público do Iroh (`RelayModeConfig::IrohDefault`)**:
   - Utiliza a infraestrutura pública global padrão do projeto Iroh (`iroh::RelayMode::Default`).
   - Opção de fallback ou uso com a rede pública aberta do ecossistema Iroh.

4. **Apenas mDNS / Rede Local (`RelayModeConfig::MdnsOnly`)**:
   - Desativa completamente o tráfego e discagem por relays remotos (`iroh::RelayMode::Disabled`).
   - Opera exclusivamente na rede local (LAN) via mDNS (zero saída para a internet, ideal para modo offline / alta privacidade).

---

## 📋 Tarefas de Implementação

### 1. Definição dos Tipos e Abstrações da API (`api/network/` e `core/transport/`)

- [ ] **Criar enum de alto nível `RelayModeConfig`** no módulo `api::network`:
  ```rust
  #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
  pub enum RelayModeConfig {
      /// Relay oficial gerenciado pelo projeto Acerola
      AcerolaDefault,
      /// Servidor de relay próprio informado pelo usuário
      Custom(String),
      /// Servidores públicos padrão da rede Iroh
      IrohDefault,
      /// Sem relay remoto — apenas descoberta mDNS local (offline/LAN)
      MdnsOnly,
  }
  ```
- [ ] **Adicionar constante com a URL do Relay Oficial do Acerola** (ex: `const ACEROLA_DEFAULT_RELAY_URL: &str = "...";`).
- [ ] **Implementar método de conversão/resolução** que transforma `RelayModeConfig` na configuração concreta correspondente do Iroh (`iroh::RelayMode`).
- [ ] **Validar formato de URL** para o modo `Custom(String)` com erro descritivo (`ConnectionError::StartupFailed("invalid relay URL")`).

### 2. Configuração Inicial no Builder (`AcerolaP2pBuilder` & `IrohTransportBuilder`)

- [ ] **Adicionar método `.relay_mode(RelayModeConfig)` no `IrohTransportBuilder`** e na trait `TransportP2pBuilder`.
- [ ] **Adicionar método `.relay_mode(RelayModeConfig)` diretamente no `AcerolaP2pBuilder`** para acesso direto via fachada pública.
- [ ] **Persistência da escolha do usuário**:
  - Salvar a preferência do modo de relay no `P2PStorage` (se configurado).
  - Na inicialização, carregar o último modo selecionado pelo usuário como padrão.

### 3. Troca Dinâmica em Tempo de Execução (`AcerolaP2p` & `NetworkManager`)

- [ ] **Adicionar comando `NetworkCommand::SwitchRelay`** no `NetworkManager`:
  ```rust
  NetworkCommand::SwitchRelay {
      mode: RelayModeConfig,
      response: tokio::sync::oneshot::Sender<Result<(), ConnectionError>>,
  }
  ```
- [ ] **Implementar método público assíncrono `node.switch_relay(mode: RelayModeConfig).await`** no `AcerolaP2p`:
  - Envia o comando para o `NetworkManager`.
  - Atualiza o `Endpoint` do Iroh / reconecta ou reconfigura o `RelayMap` dinamicamente.
  - Mantém intactos o `PeerId`, chave mestra e tabela de peers conhecidos (`known_peers`).
- [ ] **Emitir evento via `EventEmitter`**:
  - Disparar `"network:relay_changed"` contendo o novo modo em JSON para que as interfaces gráficas (Tauri/Flutter/Kotlin) atualizem o status visual instantaneamente.

### 4. Testes Automatizados e Validação

- [ ] **Testes unitários dos 4 modos de relay** no `IrohTransportBuilder`:
  - `AcerolaDefault`: inicializa com a URL oficial.
  - `Custom`: aceita URL válida e rejeita URL malformada com erro.
  - `IrohDefault`: inicializa com o modo padrão do Iroh.
  - `MdnsOnly`: inicializa com `RelayMode::Disabled`.
- [ ] **Teste de integração da troca dinâmica (`switch_relay`)**:
  - Iniciar nó em `MdnsOnly` e chavear em runtime para `AcerolaDefault` / `Custom`.
  - Verificar que o evento `"network:relay_changed"` é emitido.
  - Garantir que a identidade do nó e peers conhecidos sobrevivem à troca.
- [ ] **Teste de persistência de preferência de relay no `P2PStorage`**.

### 5. Documentação e Exemplos de Integração com UI

- [ ] **Atualizar o `README.md`**:
  - Adicionar seção dedicada explicando os 4 modos e como construir um componente de seleção visual de Relay (Dropdown / Radio buttons).
  - Incluir exemplo de código consumindo `node.switch_relay(...)` e escutando `"network:relay_changed"`.

---

# TODO: Abstração de Blobs (transferência de dados content-addressed)

> ✅ **Concluído** (branch `release/blobs-interface`). A abstração e o adapter `iroh-blobs` estão implementados, testados e documentados no `README.md`. O desenho final divergiu do rascunho original em alguns pontos — ver "Notas de implementação" no fim desta seção antes de usar este documento como referência de API.

O `acerola-p2p` já trata o `iroh` como **adapter**, não como infraestrutura: `P2pTransport` é a trait genérica (`core/transport/mod.rs`), `IrohTransport` é só a implementação concreta escolhida hoje, ativada por um feature flag (`iroh`) que não é o default estrutural — é o default de *conveniência*. Trocar de motor de transporte no futuro (outro backend QUIC, ou nenhum) não deveria vazar pra API pública nem pros apps consumidores.

Hoje isso **não vale** pra transferência de arquivo: cada protocolo de aplicação (`sync-files`, `sync-comic`, e o `browse-cover` planejado nos TODOs do Desktop/Android) reinventa a própria máquina de chunking + checksum na mão, em cada app, em vez de existir uma abstração de "blob" na lib. `iroh-blobs` (a crate irmã do `iroh`, com chunking, verificação BLAKE3 e resume automáticos) nem é dependência do `acerola-p2p` hoje — foi checado e confirmado que não tem nenhuma referência a `iroh_blobs`/`iroh-blobs` no `Cargo.lock` nem no código-fonte.

A tarefa aqui não é "adicionar `iroh-blobs`" direto — é dar a ele o mesmo tratamento de adapter que o `iroh` já tem pro transporte, pra um blob store diferente (ou nenhum) poder ser plugado sem quebrar quem consome a API.

> A migração de fato dos protocolos de app (`sync-files`, `sync-comic`, `browse-cover`) pra consumir essa abstração é trabalho de cada app consumidor, já anotado nos `TODO.md` do Desktop e do Android — não é tarefa desta lib nem deste documento.

## Tarefas de Implementação

### 1. Definição da abstração (`core/blobs/`)

- [x] **Criar trait `P2pBlobStore`** (mesmo espírito de `P2pTransport`) — `put(Vec<u8>) -> BlobHash`, `get(&hash) -> Box<dyn AsyncRead + Send + Unpin>`, `has(&hash) -> bool`, `remove(&hash)`, e mais um método além do rascunho original: `fetch(&hash, &PeerAddr)`, que baixa de um peer remoto específico pro store local (ver notas abaixo).
- [x] **Tipo `BlobHash` genérico na lib** (`core/blobs/hash.rs`) — wrapper opaco sobre `[u8; 32]`, com `Display`/`FromStr` em hex e `Serialize`/`Deserialize`, sem amarra ao tipo `Hash` do `iroh-blobs`.
- [x] **`InMemoryBlobStore`** como implementação de referência (mesmo padrão de `InMemoryStorage`), usada em testes e como store local sem capacidade de rede.
- [ ] ~~Registrar blob store no builder via `.blobs(impl P2pBlobStore)` no `AcerolaP2pBuilder`~~ — **não foi assim que ficou**, ver "Notas de implementação".

### 2. Adapter `iroh-blobs` (feature flag próprio)

- [x] **`iroh-blobs` como dependência opcional**, atrás da feature `iroh-blobs-adapter` (junto com `irpc` e `n0-error`, exigidas pra conversão de erro e opções de GC do store).
- [x] **`IrohBlobStore: P2pBlobStore`** em `core/blobs/iroh/` (`mod.rs`, `config.rs`, `gc.rs`), com conversão de erro própria em `infra/error/iroh_blobs.rs`, espelhando a separação que `core/transport/iroh/` já tem.
- [x] **Transferência real de blob entre peers**, mas **não** via um ALPN registrado pelo mecanismo genérico `inbound`/`outbound`/`ProtocolHandler` — ver "Notas de implementação".

### 3. Testes

- [x] **Testes unitários de `InMemoryBlobStore`** (`core/blobs/mem.rs`) e de **`IrohBlobStore` contra `MemStore`** (`core/blobs/iroh/mod.rs`) — `put`→`get` bytes idênticos, hash desconhecido retorna `ConnectionError::BlobNotFound`, `has()` não baixa o blob, `remove()` reflete em `has()` (com polling, já que a reclamação é assíncrona via GC do store).
- [x] **Teste de round-trip real** (`src/tests/blob_transfer.rs::run_in_isolation::real_two_node_blob_round_trip_via_fetch`) — dois nós de verdade, `put` em A, `fetch`+`get` em B, comparação BLAKE3.
- [x] **Testes de estresse**: puts/gets concorrentes de 50 blobs (`concurrent_puts_and_gets_of_multiple_blobs`) e blob de 16 MiB medindo throughput (`large_blob_throughput_and_integrity`), ambos em `run_in_isolation`.
- [x] **Testes de caminho triste** (não previstos no rascunho original, adicionados após revisão): `fetch` de hash inexistente num peer online, e `fetch` contra peer genuinamente inalcançável (falha em ~30s — timeout padrão de handshake QUIC do iroh, sem timeout menor configurado) seguido de recuperação contra um peer real.
- [x] **Mutation testing com `cargo-mutants`**: rodado de fato via `[p2p] Tests` (`workflow_dispatch`, job `Cargo Mutants`) — 298 mutantes testados em 53min: 116 pegos, 128 inviáveis, 3 timeout, **51 perdidos** (nenhum teste detecta a mudança). Os 51 gaps ficam como item novo abaixo, pra fechar aos poucos — CI não bloqueia merge por causa deles (job só roda semanal/manual).

### 6. Fechar os 51 gaps de mutation testing encontrados

> Cada `MISSED` é um lugar onde a suíte atual não distingue o comportamento real de uma versão quebrada — precisa de teste novo, um por um. Agrupado por arquivo. Reprodução: `cargo mutants --package acerola-p2p --output mutants-out` (demora ~1h).

- [ ] **`core/blobs/iroh/mod.rs`** — `IrohBlobStore::put/get/has/remove/fetch` (linhas 58, 65, 66, 73, 77, 81) e `parse_endpoint_addr` (linha 29): nenhum teste falha se essas operações virarem no-op/`Ok(Default::default())`. Provavelmente os testes existentes (`put`→`get`) usam sempre o "caminho feliz" com asserções fracas — precisa de assert mais específico por operação, não só round-trip.
- [ ] **`core/blobs/iroh/config.rs`** — `IrohBlobsConfig::mem`/`fs`/`build_store` (linhas 37, 42, 47) e `gc_config` (linha 65): nada verifica que a config resultante realmente é `Mem` vs `Fs`, nem que o `GcConfig` reflete o `gc_interval` passado.
- [ ] **`core/blobs/iroh/gc.rs`** — `untag`/`tags_for_hash` (linhas 16, 23, 28): nenhum teste verifica o conteúdo real das tags retornadas nem a lógica de comparação em `tags_for_hash`.
- [ ] **`core/transport/iroh/blobs_bridge.rs`** — `wants_alpn`/`configure`/`as_capability`/`try_accept` nas variantes `enabled`/`disabled` (linhas 33, 43, 50, 57, 58, 86, 96): a ponte entre feature ligada/desligada não tem teste que force os dois branches a se comportarem diferente de verdade.
- [ ] **`core/transport/iroh/transport.rs`** — `open_bi` (guards de `close_reason`/`retried`, linhas 179, 198), `latency` (linha 220), `blobs()` (linha 242), `is_connected` (linha 250): lógica de retry/estado de conexão sem teste que force os dois lados de cada condição.
- [ ] **`core/transport/iroh/builder.rs`** — `IrohTransportBuilder::blobs` (linha 39): nenhum teste confirma que configurar blobs no builder realmente muda o resultado.
- [ ] **`api/acerola_p2p.rs`** — `connected_peers`/`blobs` (linhas 81, 118): getters sem teste que force retorno não-vazio/`Some`.
- [ ] **`api/acerola_builder.rs`** — `FailingIdentityLoadStorage::save_identity` (linha 520, é código de teste auxiliar): mutante sobrevive dentro do próprio helper de teste, revisar se o helper é usado do jeito que deveria.
- [ ] **`infra/error/iroh_blobs.rs`** — todos os `From<...> for ConnectionError` (linhas 8, 15, 24, 33, 43): nenhum teste verifica que a conversão de erro preserva informação (viraria `Default::default()` sem quebrar nada).
- [ ] **`src/tests/mock_transport.rs`** — `MockTransportHandle::expect_open` (linha 91, código de teste): mutante sobrevive no próprio mock — revisar se o mock realmente valida o que os testes que o usam esperam validar.

3 timeouts (`drive_incoming_connections`, `drive_incoming_streams`, `disabled::BlobsIntegration::try_accept`) não entram nessa lista — são esperados pra loops que dirigem conexão assíncrona, não indicam gap de teste real.

## Notas de implementação (desvios do rascunho original)

Discutido e decidido durante a implementação — registrado aqui pra quem for consumir isso não estranhar o desenho:

- **Blobs não são configurados no `AcerolaP2pBuilder`, e sim no `IrohTransportBuilder`** (`.blobs(IrohBlobsConfig::mem())` / `.fs(path)`), e acessados em runtime via `node.blobs() -> Option<Arc<dyn P2pBlobStore>>`. A trait `P2pTransport` ganhou um método `blobs()` com default `None` (mesmo padrão de `is_connected`) — qualquer adapter pode expor essa capacidade opcionalmente, sem mexer em `IncomingConnection`.
- **A feature `iroh-blobs-adapter` depende de `iroh`** (`iroh-blobs-adapter = ["dep:iroh-blobs", ..., "iroh"]`), ao contrário do que o rascunho original sugeria. Motivo: o adapter monta o motor de transferência real do `iroh-blobs` (BLAKE3, dedup, download verificado) direto sobre o `Endpoint` que o `IrohTransport` já possui — não faz sentido sem o transporte iroh.
- **Não existe ALPN de blob passando pelo `inbound`/`outbound`/`ProtocolHandler` genérico.** O `BlobsProtocol` real do `iroh-blobs` precisa da `Connection` inteira (não de um stream já isolado), então a detecção do ALPN de blob acontece direto em `drive_incoming_connections` (`core/transport/iroh/transport.rs`), delegada pra uma ponte dedicada (`core/transport/iroh/blobs_bridge.rs`) que vira "null object" quando a feature está desligada.
- **`remove()` é "lógico", não imediato**: o `iroh-blobs` mantém a deleção física de blob restrita a garbage collection interno de propósito (a API pública não expõe deleção síncrona). `remove()` apaga a tag associada ao hash; a reclamação física acontece no próximo ciclo de GC do store (intervalo configurável em `IrohBlobsConfig`).
- ~~**Achado incidental, fora de escopo desta feature**: `AcerolaP2p::shutdown()`/`drop()` não fecha o `Endpoint` do iroh de verdade hoje~~ — **resolvido**: `NetworkManager::run()` agora chama `transport.shutdown()` ao processar `NetworkCommand::Shutdown` (`core/network/manager.rs`), e `AcerolaP2p::shutdown()` só retorna depois que a task de `run()` de fato terminou (`run_handle` guardado e esperado, `api/acerola_p2p.rs`). `IrohTransport::shutdown()` chama `endpoint.close()`, que também encerra `drive_incoming_connections` sozinha (`endpoint.accept()` devolve `None` num endpoint fechado) — não precisou de um handle/cancelamento dedicado pra essa task. Coberto por `shutdown_command_shuts_down_the_transport` e `shutdown_can_be_called_more_than_once_without_hanging`.
