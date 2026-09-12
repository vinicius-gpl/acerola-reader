# AcerolaP2P — TODO

> Lista de features já implementadas removida daqui — histórico completo no `git log`. Este
> arquivo lista só o que está realmente em aberto, com estado confirmado em auditoria de
> código (11/09/2026). As notas de design que explicavam desvios do rascunho original da
> abstração de blobs foram removidas daqui — se ainda forem úteis, migrar para o rustdoc/README
> em vez de manter num TODO.

## Crítico

- [ ] **`FsStore` do iroh-blobs trava ao abrir store em disco** — Causa raiz não encontrada em
  `FsStore::load_with_opts` (crate `iroh-blobs`). Reproduzido isolado em
  `core/blobs/iroh/mod.rs::tests::fs_store_load_does_not_hang`. Mitigado nos apps consumidores
  (Android e Desktop) com `.blobs(IrohBlobsConfig::mem())` — blobs não persistem entre
  reinícios enquanto isso não for resolvido.

## Alta

- [ ] **Troca dinâmica de relay em runtime** — Hoje `RelayModeConfig` (4 variantes:
  `MdnsOnly`, `Custom`, `AcerolaOwn`, `IrohDefault`) só é configurável no builder, antes do
  node subir (`IrohTransportBuilder::relay_mode`, `core/transport/iroh/relay_mode.rs`). Falta:
  - [ ] `NetworkCommand::SwitchRelay` no `NetworkManager`
  - [ ] `node.switch_relay(mode).await` público em `AcerolaP2p`, preservando identidade e
    `known_peers`
  - [ ] Persistência da preferência de relay no `P2PStorage`
  - [ ] Evento `"network:relay_changed"` via `EventEmitter`
  - [ ] Testes de troca dinâmica + persistência

## Média

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

## Referência: itens que o doc dizia em aberto mas já estão implementados (auditoria 11/09/2026)

- `Interface de configuração de relay (modo estático)` — **já implementado**, com design
  diferente do rascunho original: `RelayModeConfig` (`core/transport/iroh/relay_mode.rs`),
  `.relay_mode()` no `IrohTransportBuilder`, exportado em `api::mod`. Só falta a troca
  *dinâmica* em runtime (ver "Alta" acima).
- `Abstração de blobs` — **concluída**: `P2pBlobStore`, `IrohBlobStore`, `InMemoryBlobStore`,
  transferência real via `fetch`/`put`/`get`, suíte de testes (unitários, round-trip real,
  estresse, mutation testing).
- `Race de GC causando BlobNotFound esporádico` — **corrigida**: tag permanente do blob criada
  antes do fetch começar, não depois.
