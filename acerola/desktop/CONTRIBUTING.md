# Contribuindo com o Projeto Acerola Desktop

Este guia descreve a estrutura do projeto, boas práticas e os padrões arquiteturais que devem ser seguidos ao contribuir com o Acerola Desktop.

## Visão Geral e Organização do Código

O projeto adota uma arquitetura em duas camadas principais: um **Frontend** reativo baseado em Svelte 5 (SvelteKit) e um **Backend** performático e seguro em Rust usando Tauri.

### 1. Backend (Rust & Tauri) - `src-tauri/src/`

A camada Rust é fortemente estruturada utilizando princípios de Clean Architecture.

- `cmd/`: Agrupa todos os `#[tauri::command]`. É a porta de entrada para o Frontend. Esses comandos nunca devem possuir regras de negócio ou executar SQL; eles apenas chamam os serviços correspondentes em `core/services`.
- `core/services/`: Contém a regra de negócio do aplicativo. Os serviços agrupam as chamadas de repositórios, consolidam dados (ex.: juntar metadados com as contagens de capítulos) e retornam estruturas seguras para o Frontend.
- `data/`: Subdividido em:
  - `models/`: Estruturas de dados (structs) mapeadas diretamente para as tabelas ou Views do banco (ex.: `ComicSummaryView`).
  - `repositories/`: Implementam os acessos diretos ao SQLite (via `sqlx`). Todas as consultas SQL devem residir **somente** nesta camada. Há o uso massivo de Traits como `Entity` e `Bindable` para padronizar operações CRUD comuns de repositórios base.
- `infra/`: Abriga configurações de banco de dados (`infra/db/migrations/`), gerenciamento de sistema de arquivos e abstrações de erros (`infra/error/` como `DbError`, `ComicError`).
- `tests/`: Funções auxiliares para configuração de banco de dados em memória (`setup_test_db`) usado para testes unitários nos repositórios.

### 2. Frontend (Svelte 5) - `svelte/src/`

- `routes/`: Páginas da aplicação SvelteKit (ex.: `/home`, `/comic/[slug]`, `/reader`). Cada diretório define uma rota que usa os componentes e stores.
- `lib/components/`: Componentes visuais reusáveis utilizando um design system customizado (identificados pelo prefixo `acerola-`, ex.: `acerola-button`, `acerola-input`, `acerola-card`).
- `lib/hooks/store/`: Gerenciadores de estado reativos baseados no `$state` e `$effect` do Svelte 5 (Runes). Ex: `use-comic-summary.svelte.ts` cuida do controle de requisições de biblioteca e cache de dados no Frontend.
- `lib/contracts/`: Tipagens, payloads e definições estritas de constantes Tauri (ex.: nomes de eventos e comandos de IPC) usados para não haver hardcode entre backend e frontend.
- `lib/paraglide/`: Estrutura de internacionalização (i18n). Os textos da UI vêm daqui (ex.: `m['pages.home.loading']()`).

### 3. Testes de Frontend (Svelte 5) - `svelte/`

O projeto Vitest (`vitest.config.ts`) roda 3 "projects" diferentes, cada um com um propósito:

- **`unit`** (`npm run test:unit`): jsdom + `@testing-library/svelte`. A grande maioria dos testes vive aqui — componentes, hooks, stores, utils. Arquivos `*.test.ts` colocados junto do arquivo testado.
- **`integration`** (roda junto com `npm run test`): browser real (Chromium via Playwright) para os poucos casos que dependem de comportamento que jsdom não reproduz fielmente (ex.: scroll real). Arquivos nomeados `*.browser.test.ts`.
- **`storybook`** (`npm run test:storybook`): executa toda story como teste via `@storybook/addon-vitest`, também em browser real. Garante que nenhuma story quebra silenciosamente.

**Convenção de colocation**: todo componente em `lib/components/` ou `routes/**/components/` deve ter, na mesma pasta, `<nome>.svelte` + `<nome>.test.ts` (Vitest, caminho feliz e triste quando fizer sentido) + `<nome>.stories.svelte` (Storybook CSF via `@storybook/addon-svelte-csf`, usando `defineMeta`). Os componentes vendorizados do shadcn-svelte em `lib/components/ui/` são exceção deliberada — não recebem teste/story próprios, só são exercitados indiretamente pelos componentes `acerola-*` que os usam.

**Testando hooks baseados em runes (`lib/hooks/`, `lib/state/`)**: se o hook só usa `$state`/`$derived` (sem `onMount`/`onDestroy`/`$effect`), teste chamando a função exportada direto no corpo do teste — sem `render()`, sem harness (ver `lib/state/notification.svelte.ts` + `notification.test.ts`). Se o hook depende de lifecycle real (`onMount`/`$effect`), ele precisa rodar dentro de um componente montado de verdade: use os wrappers em `tests/harness/hooks/*.svelte` (ex.: `reader-store.svelte`), montados via `@testing-library/svelte`'s `render()`. Hooks com estado singleton em nível de módulo expõem um `_reset*State()` só para uso em teste (ver `use-archive-templates.svelte.ts`) — sempre chame o reset no `beforeEach` pra não vazar estado entre arquivos de teste.

**E2E**: `tests/e2e/*.spec.ts` (Playwright, roda contra o `vite dev` real, rápido — hoje faz parte do CI) mocka o IPC do Tauri inteiro via `svelte/tests/mocks/tauri-playwright.ts` (`installTauriMocks`), que já cobre por padrão o boilerplate do layout raiz (bookmarks, identidade do pacote, status de rede) e pula o onboarding — só é preciso mockar os comandos específicos do fluxo testado. `tests/wdio/*.spec.ts` (WebdriverIO, dirige o binário Tauri compilado de verdade) continua manual/local apenas — exige build release + driver nativo por SO (ver `tests/wdio/README.md`), não roda no CI.

**Mutation testing** (`npm run test:mutation`, via Stryker): roda semanalmente/manual no CI, não bloqueia PR (é lento). Escopo hoje é só lógica pura em `.ts`/`.svelte.ts` (`lib/hooks/`, `lib/state/`, `lib/services/`, `lib/utils/`) — mutação de `.svelte` está fora de escopo por decisão (suporte do Stryker a SFC Svelte ainda é imaturo).

## Padrões Adotados (Regras da Aplicação)

Ao contribuir, é estritamente proibido desobedecer às seguintes restrições:

1. **SQL Apenas em Repositórios**:
   Nunca escreva SQL nas camadas `cmd/` ou `core/services/`. Se um novo filtro for adicionado, deve-se modificar os métodos no `Repository` específico para construir a query em `sqlx`.
2. **Uso das Abstrações Já Feitas**:
   Não reinvente a roda. Use a View `comic_summary_view` se precisar iterar na Home. Não crie novos models que desrespeitem o Trait `Entity`.

3. **Restrições de Migrations**:
   Para manter a base de dados determinística para usuários finais do desktop, não crie alterações abruptas nem altere migrações existentes. Utilize o arquivo consolidado correspondente ou construa Views caso precise extrair dados relacionais complexos, evitando tabelas espelho.

4. **Idioma de Desenvolvimento**:
   **Inglês** é obrigatório para nomes de variáveis, classes, structs, funções e arquivos.
   **Português (pt-BR)** é permitido **exclusivamente** para:
   - Comentários no código explicando motivações.
   - Nomes de funções nos testes (ex.: `#[tokio::test] async fn teste_busca_por_titulo_lower_e_upper()`).

5. **Testes Unitários Obrigatórios**:
   Toda nova funcionalidade implementada no backend deve vir acompanhada do seu próprio teste unitário no mesmo arquivo (no sub-módulo `mod tests`), garantindo a robustez das queries SQLite e o tratamento de caso (ex.: lower e uppercase em buscas com `LIKE`).

## Principais Bibliotecas Utilizadas

- **Rust**: `sqlx` (Query Builder / DB Async), `tauri` (Framework Desktop), `tokio` (Async runtime).
- **Frontend**: Svelte 5 (Runes), TailwindCSS (Estilização via utilitários atrelados aos componentes locais `acerola-`), `lucide-svelte` (Ícones).

## Como Executar e Testar Localmente

1. Suba o App normalmente através do Tauri CLI: `npm run tauri dev`
2. Rode os testes do Rust isoladamente para validar mudanças nos Repositórios/Services:
   `cargo test -p acerola` ou especificamente: `cargo test -p acerola --lib data::repositories::views::tests`
3. Frontend: `npm run test` (unit + integration + storybook), `npm run test:coverage:unit` (com cobertura), `npm run test:e2e` (Playwright), `npm run storybook` (dev server do Storybook), `npm run test:mutation` (Stryker, local — pode demorar).

Siga essa base de regras estritas para submeter Pull Requests maduras e prontas para ambiente de produção!
