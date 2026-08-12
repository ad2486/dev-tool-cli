# AGENTS.md — Guia para agentes de código

Instruções para agentes (e contribuidores) trabalharem neste repositório sem desviar do design estabelecido.

## Visão geral

`dev` é um CLI em Rust para configurar ambientes de desenvolvimento de forma consistente e reproduzível: instala toolchains de linguagens (`dev install python`), cria projetos a partir de templates (`dev init python`) e diagnostica o ambiente (`dev doctor`). O produto é um binário único (`[[bin]] name = "dev"`), sem runtime externo.

## Status atual

**Pré-implementação.** A fase de design está concluída (ver `docs/`); o código é apenas um `src/main.rs` hello-world e o `Cargo.toml` ainda não tem dependências. Toda funcionalidade descrita nos docs é planejada, não existente.

## Stack técnica

- **Rust, edition 2024** (definido em `Cargo.toml`).
- Dependências **planejadas** (não instaladas ainda): `clap` (parsing de CLI), `thiserror` (erros tipados nos módulos), `anyhow` ou `miette` (borda do CLI), `rust-embed` (recursos embutidos no binário), `toml` (config e manifests).

## Estrutura do projeto

Estado atual:

```
docs/    # PRD.md, ARCHITECTURE.md, ROADMAP.md — design completo
src/     # apenas main.rs (hello-world)
Cargo.toml
```

Estrutura alvo (definida em ARCHITECTURE.md, seção "Estrutura") — **crate único com módulos, sem workspace**: `cli/`, `core/`, `providers/`, `os/`, `config/`, `errors/` sob `src/`. Sem módulo `utils/` e sem camada `commands/` (o clap faz o dispatch; os handlers vivem em `cli/`).

## Convenções

- **Idioma:** toda a documentação é escrita em português (pt-BR). Código, identificadores e mensagens de commit seguem o padrão do repositório.
- **Disciplina de escopo MVP:** implementar apenas `install`, `init`, `doctor` e `config get|set` (+ flags globais `--dry-run`, `--yes`/`-y`, `-v`/`-q`, `--config`). `uninstall`, `update`, `search` e `template` são pós-MVP — não implementar. Providers do MVP: Python e Rust. Adapters do MVP: Homebrew e apt.
- **Arquitetura:** lógica de linguagem vive em Providers híbridos (struct Rust + `manifest.toml`); Providers nunca executam processos diretamente — toda execução passa pelo `CommandRunner`, e operações de SO pelo `OsAdapter`. Recursos de Provider são embutidos no binário via `rust-embed`.
- **Decisões de design:** `docs/ARCHITECTURE.md` é a **fonte de verdade**. Qualquer decisão arquitetural nova (ou mudança de decisão existente) deve ser registrada lá **com a justificativa (o porquê)**, seguindo o padrão das seções "Por quê" já existentes.

## Testes

- Cada epic do ROADMAP inclui suas próprias tarefas de teste — testes são entregues junto da feature, não em uma fase separada.
- Testes unitários usam **fakes das traits** (`Provider`, `OsAdapter`, `CommandRunner`), nunca o sistema real.
- Adapters são testados com um **CommandRunner fake de gravação (recording)**, que registra a sequência de comandos e retorna respostas programadas.
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
