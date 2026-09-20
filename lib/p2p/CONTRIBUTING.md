# Guia de Contribuição — acerola-p2p

Primeiramente, obrigado por considerar contribuir para a **acerola-p2p**! O acerola-p2p é a biblioteca P2P central do projeto **Acerola**, construída em Rust sobre [iroh](https://github.com/n0-computer/iroh) (QUIC) e [tokio](https://tokio.rs), projetada para ser compartilhada entre aplicações Desktop (Windows/Linux/macOS) e Mobile (Android/iOS).

Este documento orienta sobre o fluxo de desenvolvimento, padrões de código, execução de testes e processo de envio de Pull Requests.

---

## 📋 Sumário

- [Código de Conduta](#-código-de-conduta)
- [Como Posso Contribuir?](#-como-posso-contribuir)
  - [Relatando Bugs](#relatando-bugs)
  - [Sugerindo Funcionalidades](#sugerindo-funcionalidades)
  - [Enviando Pull Requests](#enviando-pull-requests)
- [Configuração do Ambiente de Desenvolvimento](#-configuração-do-ambiente-de-desenvolvimento)
  - [Pré-requisitos](#pré-requisitos)
  - [Passo a Passo](#passo-a-passo)
- [Comandos de Build, Linter e Testes](#-comandos-de-build-linter-e-testes)
- [Padrões de Código e Arquitetura](#-padrões-de-código-e-arquitetura)
  - [Estilo Rust e Formatação](#estilo-rust-e-formatação)
  - [Diretrizes de Arquitetura](#diretrizes-de-arquitetura)
  - [Convenção de Commits](#convenção-de-commits)
- [Licença](#-licença)

---

## 🤝 Código de Conduta

Esperamos que todos os colaboradores mantenham um ambiente respeitoso, inclusivo e construtivo. Trate todos com consideração e profissionalismo ao abrir issues, comentar em Pull Requests ou debater decisões de arquitetura.

---

## 🚀 Como Posso Contribuir?

### Relatando Bugs

Se você encontrou um bug ou comportamento inesperado:
1. Verifique se o problema já não foi relatado nas **Issues** do repositório.
2. Abra uma nova **Issue** detalhando:
   - **Descrição clara do problema**.
   - **Passos para reproduzir**.
   - **Sistema Operacional** e arquitetura.
   - **Logs de erro** ou tracebacks relevantes (se aplicável, com `tracing`).

### Sugerindo Funcionalidades

Ideias para melhorar a performance, adicionar suporte a novos transports ou estender os protocolos internos são bem-vindas:
1. Abra uma **Issue** do tipo *Feature Request*.
2. Explique o caso de uso e a utilidade da funcionalidade proposta.
3. Alinhe com o mapa de desenvolvimento (`../../TODO.md`, centralizado na raiz do monorepo) para garantir consonância com os objetivos do projeto.

### Enviando Pull Requests

1. **Fork & Branch**: Crie um fork do repositório e crie sua branch a partir de `develop` (ex: `git checkout -b feature/minha-feature` ou `fix/corrigir-bug`).
2. **Manutenção da Qualidade**: Certifique-se de que todo o código passe nos testes (`cargo make test`), linters (`cargo make lint`) e formatação (`cargo make format`).
3. **Escopo Pequeno e Focado**: Prefira PRs concisos e com escopo bem definido.
4. **Descrição Detalhada**: Explique o que foi alterado e a motivação do PR.

---

## 🛠 Configuração do Ambiente de Desenvolvimento

### Pré-requisitos

- **Rust**: Versão Stable mais recente (Edição 2021). [Instalar Rust](https://www.rust-lang.org/tools/install).
- **cargo-make**: Ferramenta de automação de tarefas em Rust.
  ```bash
  cargo install cargo-make
  ```
- **cargo-nextest** (Recomendado): Runner de testes de alta performance.
  ```bash
  cargo install cargo-nextest
  ```
- **Android NDK** (Opcional, apenas para compilação cruzada mobile).

### Passo a Passo

```bash
# 1. Clone o repositório
git clone https://github.com/Vinicius-Gabriel-P-Leitao/acerola-p2p.git
cd acerola-p2p

# 2. Verifique a compilação do workspace
cargo make check
```

---

## ⚡ Comandos de Build, Linter e Testes

O projeto utiliza `cargo-make` para gerenciar todas as rotinas de desenvolvimento e CI.

| Comando | Descrição |
|---|---|
| `cargo make check` | Verifica se o código e os testes compilam |
| `cargo make build` | Compila o projeto em modo debug |
| `cargo make build-release` | Compila a biblioteca otimizada para produção |
| `cargo make format` | Aplica formatação de código com regras customizadas do `rustfmt.toml` |
| `cargo make lint` | Executa o `clippy` com `-D warnings` |
| `cargo make test` | Executa a suíte de testes unitários e de integração |
| `cargo make test-verbose` | Executa testes sem capturar a saída padrão (`--no-capture`) |
| `cargo make test-stress` | Executa o teste de estresse de transporte (`transport_validation`) |
| `cargo bench` | Executa os benchmarks formais de throughput (Mock & Iroh) via Criterion |
| `cargo llvm-cov` | Gera o relatório de cobertura de linhas de código (`--all-features`) |
| `cargo mutants` | Executa a análise de testes de mutação no pacote (`--package acerola-p2p`) |
| `cargo make ci` | Executa a pipeline completa de CI (check + lint + test-ci) |
| `cargo make build-android-all` | Cross-compilação para alvos Android (ARM64, ARMv7, x86_64) |

---

## 📐 Padrões de Código e Arquitetura

### Estilo Rust e Formatação

- Execute `cargo make format` antes de abrir qualquer Pull Request. As configurações do `rustfmt.toml` aplicam:
  - `max_width = 100`
  - Reordenação de `imports` agrupados (`StdExternalCrate`).
- Todo o código deve passar pelo Clippy sem nenhum aviso (`cargo make lint`).

### Diretrizes de Arquitetura

1. **Abstração de Transporte (`TransportP2pBuilder` & `P2pTransport`)**:
   - A biblioteca desacopla a rede da lógica de aplicação. O Iroh QUIC é o transporte padrão (`feature = "iroh"`), mas novas implementações devem respeitar as traits de transporte.
2. **Padrão Builder & Injeção de Dependências**:
   - Instâncias do `AcerolaP2p` são construídas via `AcerolaP2pBuilder`.
   - Modificadores (Guards, Handlers, EventEmitter, DeviceInfo) são injetáveis.
3. **Guards e Segurança (TOFU Guard)**:
   - Validações de peer usam conexões seguras sob QUIC/TLS 1.3. O `TofuGuard` fornece a política Trust-On-First-Use padrão.
4. **Controle de Erros**:
   - Erros da infraestrutura e transporte utilizam `thiserror` (exportados como `P2pError`). Evite utilizar `.unwrap()` ou `.expect()` em código de produção.

### Convenção de Commits

Utilizamos mensagens de commit estruturadas no padrão `[tag]: Descrição.`:

- `[feat]:` Adição de nova funcionalidade.
- `[fixed]:` ou `[fix]:` Correção de bug.
- `[docs]:` Alterações em documentação ou comentários.
- `[refactor]:` Reformatação ou refatoração sem alterar funcionalidades.
- `[test]:` Adição ou correção de testes.
- `[chore]:` Tarefas de manutenção, formatação ou atualizações gerais.
- `[ci]:` Alterações nos fluxos de integração contínua.
- `[merge]:` Mensagens de merge entre branches.

*Exemplo:* `[feat]: Adiciona suporte a persistência no TofuGuard.`

---

## 📄 Licença

Ao contribuir para o **acerola-p2p**, você concorda que suas contribuições serão licenciadas sob os termos da **Mozilla Public License 2.0 (MPL-2.0)**.
