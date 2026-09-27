# AGENTS.md — Guia para agentes de código

Instruções para agentes (e contribuidores) trabalharem neste repositório sem desviar do design estabelecido.

## Visão geral

`dev` é um CLI em Rust para configurar ambientes de desenvolvimento de forma consistente e reproduzível: instala toolchains de linguagens (`dev install python`), cria projetos a partir de templates (`dev init python`) e diagnostica o ambiente (`dev doctor`). O produto é um binário único (`[[bin]] name = "dev"`), sem runtime externo.

## Status atual

**MVP completo, preparando a v0.1.0.** Todos os comandos do MVP funcionam para Python e Rust em macOS e Linux, cobertos por testes unitários, de integração (`tests/cli.rs`) e E2E no CI. A distribuição existe (`release.yml` + `install.sh`); falta publicar a primeira tag. Templates (Epic 7) foram adiados para o pós-MVP. A documentação de usuário está em `README.md` e `docs/languages/` (inglês principal, par `.pt-BR.md`); ao mudar o comportamento de um Provider, atualize a página da linguagem nos dois idiomas.

## Stack técnica

- **Rust, edition 2024** (definido em `Cargo.toml`).
- Dependências instaladas: `clap` (derive), `thiserror`, `anyhow`, `log` + `env_logger`, `toml`, `toml_edit` (escrita do config preservando comentários), `serde` (derive), `directories` (caminho idiomático do config por SO).
- Dependências **planejadas**: `rust-embed` (recursos embutidos no binário, Epic 7 — adiado para pós-MVP); `miette` segue como alternativa ao `anyhow` na borda do CLI.

## Estrutura do projeto

**Crate único com módulos, sem workspace** (ARCHITECTURE.md, seção "Estrutura"). Módulos no estilo moderno — `arquivo.rs` como raiz do módulo e um diretório de mesmo nome para os submódulos, **sem `mod.rs`**:

```
src/
├── main.rs              # declara os módulos, inicializa o logging, mapeia erro → código de saída
├── cli.rs               # definição dos comandos (clap)
├── core.rs              # Registry de Providers
├── config.rs            # config do usuário: caminho, carga e resolução da config efetiva
├── errors.rs            # Category, ErrorCategory e códigos de saída
├── providers.rs         # trait Provider, InstallContext
├── providers/
│   └── manifest.rs      # parser do manifest.toml (schema v1)
├── os.rs                # traits CommandRunner e OsAdapter; runners real, dry-run e recording
└── os/
    └── brew.rs          # adapter Homebrew
```

Sem módulo `utils/` e sem camada `commands/` (o clap faz o dispatch; os handlers viverão em `cli/`).

## Convenções

- **Idioma:** toda a documentação é escrita em português (pt-BR). Código, identificadores e mensagens de commit seguem o padrão do repositório.
- **Disciplina de escopo MVP:** implementar apenas `install`, `init`, `doctor` e `config get|set` (+ flags globais `--dry-run`, `--yes`/`-y`, `-v`/`-q`, `--config`). `uninstall`, `update`, `search` e `template` são pós-MVP — não implementar. Providers do MVP: Python e Rust. Adapters do MVP: Homebrew e apt.
- **Arquitetura:** lógica de linguagem vive em Providers híbridos (struct Rust + `manifest.toml`); Providers nunca executam processos diretamente — toda execução passa pelo `CommandRunner`. O que varia por SO ou gerenciador de pacotes passa antes pelo `OsAdapter`; comandos idênticos em todos os sistemas (`uv init`, `cargo init`) vão direto ao Runner (ver "OS Adapter" em ARCHITECTURE.md). Recursos de Provider (templates, pós-MVP) serão embutidos no binário via `rust-embed`.
- **Decisões de design:** `docs/ARCHITECTURE.md` é a **fonte de verdade**. Qualquer decisão arquitetural nova (ou mudança de decisão existente) deve ser registrada lá **com a justificativa (o porquê)**, seguindo o padrão das seções "Por quê" já existentes.

## Testes

- Cada epic do ROADMAP inclui suas próprias tarefas de teste — testes são entregues junto da feature, não em uma fase separada.
- O crate é só binário (sem `[lib]`), então um diretório `tests/` não enxerga o código de `src/`: teste de função é **unitário, inline** no arquivo que testa, em `#[cfg(test)] mod tests { use super::*; }`.
- **Testes de integração** (`tests/cli.rs`) executam o binário compilado (`env!("CARGO_BIN_EXE_dev")`) numa pasta temporária, sempre com `--config` apontando para um arquivo do sandbox — nunca a config real do usuário. Cobrem só o que não modifica o sistema: códigos de saída, mensagens, `dev config` e `--dry-run`.
- **Testes E2E** existem **só no CI** (job `e2e` em `.github/workflows/ci.yml`): em máquinas descartáveis Ubuntu e macOS, o workflow remove o Rust da imagem, roda `dev init <linguagem>` de verdade e falha se o `dev doctor` apontar `missing` ou versão divergente. Nunca rodar instalação real na máquina de desenvolvimento.
- Testes unitários usam **fakes das traits** (`Provider`, `OsAdapter`, `CommandRunner`).
- Adapters são testados com o **`RecordingRunner`**, que grava a sequência de comandos (`commands()`) e devolve respostas programadas (`push_response()`; fila vazia → sucesso com código 0).
- **Exceção ao uso de fakes:** smoke tests que só *leem* o sistema (ex.: o lookup de binário no PATH) podem usar o `RealRunner`, marcados com `#[cfg(unix)]` — são o único jeito de provar que um comando montado de fato funciona, o que o fake de gravação não prova. Nunca um teste que modifica o sistema.
- Comandos que modificam o sistema devem ser **idempotentes** (ver "Idempotência" em ARCHITECTURE.md).

## Tratamento de erros

Seguir a estratégia definida em ARCHITECTURE.md, seção **"Tratamento de Erros"**: taxonomia de quatro categorias com códigos de saída (usuário = 2, ambiente = 3, falha de ferramenta = 4, interno = 70), `thiserror` para erros tipados nos módulos e `anyhow`/`miette` na borda do CLI, onde o `main` mapeia categoria → código de saída.

## Comandos de build e verificação

```bash
cargo build      # compilar
cargo test       # rodar testes
cargo clippy     # lints
cargo fmt        # formatação
```

## Referências

- `docs/PRD.md` — o que o produto deve fazer (e o que é não-objetivo)
- `docs/ARCHITECTURE.md` — como é estruturado e por quê (fonte de verdade do design)
- `docs/ROADMAP.md` — epics, tarefas (DEV-xxx) e o que é MVP vs. pós-MVP
