# TODO — Acerola (índice)

Monorepo com 4 componentes versionados independentemente (`lib/p2p`, `acerola/android`,
`acerola/desktop`, `docs/web`). Cada um mantém seu próprio `TODO.md` com os itens em aberto
específicos dele — este arquivo é só o índice, mais os bloqueios que afetam mais de um
componente ou o lançamento como um todo.

## TODOs por componente

- [`lib/p2p/TODO.md`](lib/p2p/TODO.md) — troca dinâmica de relay, gaps de mutation testing, hang do FsStore
- [`acerola/android/TODO.md`](acerola/android/TODO.md) — bugs de sync P2P, features pendentes
- [`acerola/desktop/TODO.md`](acerola/desktop/TODO.md) — bugs de sync P2P, features pendentes
- [`docs/web/TODO.md`](docs/web/TODO.md) — conteúdo da página de instalação

## Bloqueios principais antes do beta v1.0.0

Levantado em auditoria de release (11/09/2026), cruzando os três `TODO.md` antigos com o
código real.

- [x] **Conflito de sync (quadrinho existente nos dois lados) — detecção/relato corrigidos** —
  Android e Desktop agora distinguem "capítulo ausente" de "capítulo já existe com checksum
  diferente" (conflito de verdade) e reportam a contagem na notificação/log de transferências
  da sessão, sem toast e sem uma notificação por capítulo. Decisão de escopo: continua
  sobrescrevendo com a versão do peer (comportamento inalterado) — resolução de conflito de
  verdade (escolher qual lado vence) fica pra depois, isso só resolve o falso-negativo
  relatado (sessão que termina bem mas mostra erro).
- [x] **Página de instalação dos docs é placeholder** — **corrigido**: conteúdo real (EN +
  PT-BR) com requisitos por plataforma e aviso sobre SmartScreen/instalação de fonte
  desconhecida, ver [`docs/web/TODO.md`](docs/web/TODO.md).
- [ ] **README.md desatualizado** — ainda diz "docs/web (ainda vazio)"; não é mais verdade
  (docs já em produção, v1.0.13).
- [ ] **Canais `beta`/`prod` da CI nunca foram exercitados** — toda tag até hoje usou o canal
  `alpha` (`git tag -l`). O upload pro Cloudflare R2 (Android, só em `prod`) e o
  `prerelease=false` nunca rodaram de verdade. Fazer um dry-run antes de depender disso pro
  lançamento em si.
- [ ] **`p2p-release.yml` não publica em lugar nenhum de verdade** — só sobe um
  `actions/upload-artifact` de CI, não `cargo publish` nem GitHub Release. Confirmar se isso é
  intencional (lib consumida só via path/git dependency) antes do beta.
- [ ] **Sem convenção documentada de versão entre os 4 componentes** — hoje cada um versiona
  independente (`p2p` 1.0.3, `desktop` 1.0.0, `android` 1.0.1, `docs` 1.0.13). Decidir o que
  "v1.0.0 beta" significa como selo conjunto antes de anunciar publicamente.
- [ ] **`FsStore` do `iroh-blobs` trava ao abrir store em disco** — mitigado em Android e
  Desktop com blob store em memória (blobs não persistem entre reinícios). Causa raiz vive no
  `lib/p2p` — ver [`lib/p2p/TODO.md`](lib/p2p/TODO.md).
