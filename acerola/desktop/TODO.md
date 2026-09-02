# Acerola Desktop TODO de Features

---

## Home

- [x] **Exibir biblioteca em grade (grid)** - Cards com capa retornada pelo backend (Rust). O Svelte carrega os assets do disco local.
- [x] **Buscar quadrinhos por título** - Filtro reativo no frontend (Svelte store) comparando com os itens em memória.
- [x] **Ordenar biblioteca** - Menu na UI que permite ordenar os itens do frontend por: título (A-Z / Z-A), contagem ou data de atualização.
- [x] **Filtrar biblioteca** - Filtros aplicados no state manager (MangaDex, AniList, Local, lidos/não lidos).
- [x] **(validar se faz sentido nesse caso) Menu de ações por quadrinho (multi-select / hover menu / botão direito)** - Menu para: favoritar com categoria, ocultar/mostrar e deletar. (Para Desktop o long press não se aplica).
- [x] **Favoritar quadrinho com categoria** - Comando Tauri vincula um `category_id` ao quadrinho no banco de dados SQLite local.
- [x] **Ocultar / mostrar quadrinho** - Comando Tauri atualiza o campo de status do quadrinho. O frontend apenas remove/adiciona visualmente.
- [x] **Deletar quadrinho** - Comando Tauri deleta o registro do banco local, sem afetar o arquivo físico (com dialog de confirmação no Svelte).

---

## Tela do Quadrinho

- [x] **Exibir capa, título, autor e status** - O componente do header exibe as strings baseadas no retorno de metadados do backend em Rust.
- [x] **Exibir badge da fonte de metadados** - Badge indicando a origem dos metadados processados no Rust (MangaDex, AniList, Local).
- [x] **Exibir gêneros / tags como badges** - Chips das tags extraídas e formatadas.
- [x] **Exibir sinopse expansível** - Truncate via CSS/Svelte que permite expansão para leitura do texto completo.
- [x] **Iniciar / Continuar / Reler** - O Svelte consulta `history_get_comic` via Tauri e decide a string e para qual página a rota `/reader` vai navegar.
- [x] **Listar capítulos com status de leitura** - Listagem renderizada com os `readChapters` marcados visualmente.
- [x] **Marcar capítulo como lido / não lido** - Ação explícita na UI (menu do capítulo + seleção múltipla em lote) chamando um evento no backend (Tauri) para inserir ou deletar row no histórico.
- [x] **Ordenar capítulos** - Menu com 4 opções: número (crescente/decrescente) e última modificação (crescente/decrescente). Aplica-se a chapters e volumes.
- [x] **Agrupar capítulos por volume** - Agrupamento e renderização dos dados que possuem `volumeId` formatados no Rust.
- [x] **Trocar estilo de exibição de capítulos/volumes** - O frontend muda o layout de list para volume baseado nas preferências da store do Svelte.
- [x] **Configurar paginação da lista de capítulos** - Parametrização customizável gravada nas preferências que quebra requisições grandes pro Rust.
- [x] **Atribuir categoria ao quadrinho** - Modal/Dropdown no Svelte que dispara o update do quadrinho no banco.
- [x] **Ativar/desativar sync externo por quadrinho** - Toggle na UI repassado para o backend que anula ou permite metadados online específicos.
- [x] **Sincronizar capítulos locais (rescaneamento manual/folder watch)** - Chamada de comando Tauri que instrui o Rust a recarregar apenas essa pasta pontual do File System.
- [x] **Reescanear quadrinho completo** - Invalida metadados atuais do banco e extrai tudo de novo daquele subdiretório.
- [x] **Sincronizar metadados pelo MangaDex** - Endpoint no Rust com um HTTP Client para buscar cover/banner/tags da API externa.
- [x] **Sincronizar metadados pelo AniList** - Endpoint no Rust utilizando queries GraphQL pro serviço.
- [x] **Sincronizar metadados pelo ComicInfo.xml** - O Rust decodifica e carrega o arquivo XML para popular o banco interno.
- [x] **Sincronizar capítulos pelo ComicInfo.xml** - O Rust associa as `Pages` e informações estruturais de capítulo via parse do XML.
- [x] **(validar se faz sentido nesse caso) Extrair primeira página de capítulo como capa do quadrinho** - Rust abre o `cbz/rar`, processa a page 0 e salva em disco como miniatura persistente.
- [x] **(validar se faz sentido nesse caso) Extrair capa do volume a partir do primeiro capítulo do volume** - O Rust resolve o primeiro item do volume local e exporta uma thumb isolada na pasta correspondente.

---

## Leitor

- [x] **Ler arquivos .cbz** - Tauri/Rust lidam com zip/cbz em disco, retornando uma lista de bytes serializada ou lendo via protocolo de assets (asset://).
- [x] **Ler arquivos .cbr** - Backend Rust trata a descompressão do rar no SO e entrega as páginas no front.
- [x] **(validar se faz sentido nesse caso) Converter .pdf para .cbz antes de ler** - O backend converte o PDF internamente num zip de bitmaps para servir ao leitor.
- [x] **(validar se faz sentido nesse caso) Pré-carregar páginas adjacentes (prefetch)** - O store do leitor (Svelte) pode fazer um fetch em background das N próximas imagens, ou o Tauri as deixa em buffer de RAM cache (LRU).
- [x] **Modo de leitura horizontal paginado** - Componente web lidando com interações de next/prev horizontalmente (mouse, teclado, etc).
- [x] **Modo de leitura vertical paginado** - Scroll restrito p/ cada página por vez.
- [x] **Modo Webtoon (scroll vertical contínuo)** - Lista CSS (`flex-col`) contínua para mangás compridos com intersection observers reportando a página.
- [x] **Zoom nas páginas** - Pan & Zoom controlado via Svelte/CSS (wheel do mouse, pinch).
- [x] **Alternar modo de leitura dentro do leitor** - Menu rápido (Command Palette ou Header) atualiza instantaneamente as stores de user preference (Svelte).
- [x] **Mostrar/ocultar controles do leitor** - Clique simples alterna classes de visibilidade dos toolbars.
- [x] **Salvar progresso de leitura automaticamente** - O Svelte observa a página ativa com Intersection Observer e invoca no Tauri o save do timestamp + page info.
- [x] **Navegar para próximo/capítulo anterior** - Shortcut de teclado ou botão na UI manda requisição pro Rust carregar as rotas do capítulo adjacente.

---

## Metadados

- [x] **Sincronizar metadados de toda a biblioteca pelo MangaDex** - Task de background no Rust processa recursivamente toda biblioteca em lotes (pool) buscando atualizações.
- [x] **Sincronizar metadados de toda a biblioteca pelo AniList** - Rotina similar ao anterior no backend usando GraphQL batch queries.
- [x] **(validar se faz sentido nesse caso) Salvar capa na pasta do quadrinho** - Rust realiza a operação de FS:I/O na mesma pasta do arquivo original com uma thumb otimizada (ex: resize jpeg).
- [x] **(validar se faz sentido nesse caso) Salvar banner na pasta do quadrinho** - Similar à escrita local, focado na imagem `banner.jpg` extraída ou da web.
- [x] **Exportar metadados como ComicInfo.xml** - Rotina do Rust que agrupa as tabelas do SQLite num formatador XML e injeta na pasta raíz caso configurado.
- [x] **Ler metadados de ComicInfo.xml** - Parser Rust que intercepta o ComicInfo durante o rescan incremental e sobrescreve as propriedades.

---

## Configurações

- [x] **Selecionar pasta raiz da coleção** - Abre o window picker padrão do Tauri e persiste a configuração pro Svelte state e backend db.
- [x] **Scan incremental da biblioteca** - Botão dispara evento Tauri (e.g. `refresh_library`) que usa filesystem watchers ou diff entre db e arquivos.
- [x] **Scan profundo (rebuild) da biblioteca** - O comando `rebuild_library` dropa informações antigas no Rust e itera pesadamente em todos arquivos de novo.
- [x] **Selecionar idioma global de metadados** - Svelte envia o código `pt-br`, `en`, para parametrizar APIs do backend.
- [x] **Ativar/desativar geração de ComicInfo.xml** - Estado guardado que liga rotinas passivas do Rust.
- [x] **Criar categoria** - Envio dos params (Label, HexColor) do modal do frontend para insert no SQLite via Tauri.
- [x] **Deletar categoria** - Deleção propagada em cascata pelo Rust com as devidas confirmações no Svelte.
- [x] **Selecionar tema do app** - Comportamento nativo SvelteKit pra injetar class CSS da cor (Ex: Catppuccin/Dracula).
- [x] **Navegar para configuração de templates** - Rota isolada (`/config/templates`) listando as macros e regras.

---

## Templates de Nomenclatura

- [x] **Criar template** - Frontend expõe os macros (ex: `{chapter}`, `{decimal}`) e grava numa entidade Rust de Parsing.
- [ ] **Editar template** - Update das tabelas locais relacionadas via Tauri Invoke.
- [x] **Deletar template** - Delete row, bloqueado para templates padrão (`is_default`).
- [x] **Listar templates** - O backend lista os templates ordenados que o usuário fez pra parser.
- [x] **Detecção automática de template no scan** - O parser de texto nativo no Rust intercepta arquivos de nome que não possuem um padrão pré-descrito, associando as strings corretas.

---

## Histórico

- [x] **Exibir leituras recentes** - Rota renderizada agrupando items a partir de uma call SQL (Tauri) listando as leituras e capas ativas.
- [x] **Continuar pelo histórico** - Payload de clique empacota state via router Svelte abrindo direto a page do `reader`.
- [ ] **[Alta] Corrigir bug no histórico** - Quando um capítulo é marcado como concluído o app não atualiza o histórico para otimizar o histórico.

---

## Onboarding

- [x] **Tutorial de primeira abertura** - Rota vazia caso DB esteja cru guiando o user pela seleção da primeira pasta no Tauri FS open API.

---

## Distribuição

- [ ] **[Média] Automatizar submissão do MSIX na Microsoft Store via CD** - Hoje a publicação no Partner Center é manual. Adicionar a `msstore` CLI no `desktop-release.yml` pra preparar/atualizar a submissão a cada release automaticamente, parando antes do envio pra certificação — esse último clique continua manual, como revisão final.

---

## Pendente

- [x] **Marcar quadrinho / capítulo como concluído manualmente** - Ação direta pro Tauri alterar a prop bool no banco.
- [x] **Seleção múltipla de quadrinhos e capítulos (multi-select)** - Manter Set Array/Map ativo na memória do Svelte UI pra realizar highlights com shift/ctrl cliques (mouse interaction).
- [x] **Ações em lote sobre seleção múltipla** - Loopar actions de API e passar Listas para queries batch do SQLite (Tauri) otimizando deletes.

---

## P2P / Sincronização

- [ ] **[Alta] `browse-library` — causa raiz encontrada, aguardando confirmação ao vivo** - O timeout de 15s/30s era o outbound do Android nunca escrevendo nada no stream QUIC antes de esperar resposta (regra do quinn: quem chama `open_bi()` precisa escrever antes do lado que aceita conseguir `accept_bi()`) — o inbound deste repo (Desktop) já estava correto (não lê nenhum marcador antes de responder, pra não travar em NAT traversal lento). Corrigido no Android (`LibraryBrowseOutbound`/`run_outbound` passou a escrever um marcador `{}` antes de esperar a resposta); commit local no Android, ainda não buildado/instalado no celular. **Pendente:** confirmar ao vivo depois do rebuild+reinstall.
- [ ] **[Alta] `BlobNotFound` esporádico em transferências (capítulos/capas)** - Causa raiz em duas camadas, ambas no `acerola-p2p` compartilhado: (1) `fetch()` não criava nenhuma tag protegendo o blob baixado do GC periódico do store — corrigido criando a tag permanente ANTES do fetch começar (não só depois), eliminando a janela onde uma varredura podia reciclar o blob no meio do download; (2) uma versão intermediária do fix (`temp_tag` antes + tag permanente depois + solta o `temp_tag`) ainda falhava sob concorrência real (~20 fetches simultâneos, como o burst de `browse-cover` ao navegar uma biblioteca remota) porque o `gc_mark_task` do `iroh-blobs` lê tags permanentes e temporárias em duas chamadas não-atômicas — se a proteção migra de temp pra permanente bem no meio dessas duas leituras, nenhuma pega. Reproduzido em teste com 24 fetches concorrentes (mesmo padrão do burst ao vivo), 10/10 execuções limpas depois do fix; suite inteira do `acerola-p2p` (177 testes) passando. Publicado (`acerola-p2p` main) e `cargo update -p acerola-p2p` já rodado nos dois apps. **Pendente:** confirmar ao vivo.
- [ ] **[Crítica] `FsStore` do `iroh-blobs` trava indefinidamente ao abrir store em disco (mitigado, não corrigido)** - Descoberto em 22/08/2026, depois de habilitar `.blobs(IrohBlobsConfig::fs(...))`: `AcerolaP2p::builder(...).build()` passou a travar 100% das vezes, estourando o timeout de 10s em `setup_network` (`[Bios::Network] Timeout waiting for AcerolaP2p::build(): Elapsed(())`) — como resultado, `network_service` nunca era `.manage()`do, e TODO comando Tauri que depende dele (`get_local_id`, QR code de pareamento, tudo) quebrava com `state not managed`. Confirmado isolado, fora do app: um teste mínimo em `acerola-p2p` (`core/blobs/iroh/mod.rs::tests::fs_store_load_does_not_hang`) chamando só `IrohBlobStore::new` com config `Fs` num diretório limpo trava e estoura 15s sozinho. **Mitigação aplicada:** trocado `.blobs(IrohBlobsConfig::fs(app_data_directory.join("blobs")))` por `.blobs(IrohBlobsConfig::mem())` em `bios/network.rs` — destrava o app, mas blobs deixam de persistir entre reinícios do app. **Pendente:** achar a causa raiz do hang no `FsStore::load_with_opts` (`iroh-blobs`) e voltar pra `.fs(...)` depois.
- [ ] **[Média] Callback visual de quando um sync ocorre ou alguém pede dados** - O comando Tauri `sync_comic` resolve assim que a CONEXÃO é estabelecida (fire-and-forget) — a tela do quadrinho mostra "Sucesso!" nesse instante, não quando o sync de fato termina. O resultado real (`sync:comic:started/progress/complete/error`) só é visível no log da tela `/network`, nunca onde a ação foi disparada. Parar de mostrar sucesso prematuro e refletir o resultado real (sucesso/erro/progresso) onde o usuário disparou a ação.
- [ ] **[Média] Tela de rede: remover dispositivo pareado** - Adicionar a mesma ação que o Android já tem (desparear um peer da lista).
- [ ] **[Baixa] Otimizar tela de busca/navegação de biblioteca remota** - É webview, dá pra fazer melhor que o atual (paginação/virtualização, layout mais claro do que o outro dispositivo tem).
- [x] **[Baixa] Tirar mensagem "Metadados sincronizados!" ao dar sync num capítulo** - Sync de capítulo não mexe em metadados, só em arquivo — a mensagem não faz sentido nesse contexto.
- [ ] **[Média] Lista de capítulos não atualiza depois de um sync** - Depois de um `sync_comic` completar, a tela do quadrinho não reflete os capítulos novos — precisa dar F5 ou sair/voltar pra tela. A lista/LRU de capítulos tem que reagir ao evento `sync:comic:complete` e se reconstruir.
- [ ] **[Baixa] Botão de sync de histórico na tela de histórico** _(talvez)_
- [ ] **[Alta] Conflito (quadrinho existente nos dois lados): callbacks de erro e sucesso quebrados** - Testado ao vivo com um quadrinho que deveria dar conflito e trazer capítulos novos: os capítulos novos chegaram, mas nenhum conflito real foi detectado/reportado (um lado provavelmente ignorou ou sobrescreveu) — e mesmo assim o Android mostrou "Erro ao sincronizar quadrinho: stream failed: stream error: timeout waiting for frame", um falso negativo (não houve erro nenhum na sessão). Investigar por que o timeout aparece mesmo numa sessão que completou, e implementar a lógica de conflito de verdade (ver item combinado nos dois apps).
- [x] **[Média] Sync individual por quadrinho: push/pull explícito** - Hoje não existe uma forma clara de, dentro da tela de um quadrinho específico, escolher "puxar dele" ou "mandar pra ele" pra um peer — precisa ficar explícito, não só implícito pela direção que o manifest calcula.
- [ ] **[Média] "Reescanear quadrinho completo": unificar entre os dois apps** - Desktop tem essa função, Android não. Preferência: remover do Desktop em vez de adicionar no Android.
- [ ] **[Alta] Protocolo de sync de arquivos não leva o quadrinho 100%** - Hoje não inclui `ComicInfo.xml` + cover + banner junto com os capítulos (sem metadados de fontes externas tipo MangaDex/AniList, só os arquivos locais) — corrigir pra levar o quadrinho completo numa sincronização.
- [ ] **[Alta] Validar encerramento correto de conexões/blobs — sessões que só voltam ao fechar o app** - Log real do Android (só volta a funcionar fechando e reabrindo o app):
  ```
  browse:library:error -> "stream failed: timed out reading library summary"
  outbound handler failed error=StreamFailed("timed out reading library summary")
  outbound connection closed
  ```
  Suspeita: o Desktop inicia uma sessão `acerola/browse-cover/1` e não a finaliza corretamente do lado dele, deixando o Android preso esperando. Investigar mais fundo como o `iroh` notifica o encerramento internamente (parece só notar quando finaliza normalmente, não quando trava).

---

## Arquitetura & Infraestrutura (Rust)

- [x] **Gerar seed dinâmico para nó P2P** - (validar se é a melhor forma) Substituir o seed hardcoded por geração de 32 bytes aleatórios persistidos em arquivo local (.key) ou SQLite para cada instalação ter sua identidade P2P isolada.
- [x] **Tratamento gracioso de erro na inicialização assíncrona do Rust** - Substituir o uso de `panic!` na inicialização de serviços assíncronos (banco de dados SQLite, nó de rede P2P) por retornos de `Result` e exibição de alerta gráfico ao usuário.
- [x] **Otimizar e dinamizar o gerenciamento de escopos do File System (fs_scope)** - Substituir a leitura crua do settings.json via std::fs pelo plugin tauri-plugin-store e atualizar dinamicamente as permissões do fs_scope quando o usuário alterar a pasta da biblioteca em runtime.
- [ ] **[Média] Fazer o app conseguir ficar em segundo plano com ícone escondido** - Fazer o app poder ficar colapsado em segundo plano e na barra de tarefas do sistema para quando o usuário executar algo demorado ele poder deixar fechado enquanto roda.
