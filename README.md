# dev

`dev` é uma ferramenta de linha de comando para configurar ambientes de desenvolvimento de maneira consistente e reproduzível.

A ideia: preparar um computador novo com poucos comandos, sem pesquisar a documentação de instalação de cada linguagem. O `dev` instala toolchains, cria projetos a partir de templates e diagnostica o ambiente — tudo em um binário único, sem runtime externo.

## Status

**Fase de design concluída; implementação do MVP iniciando.** O binário hoje é apenas um esqueleto — nenhum comando está implementado ainda. O design completo (produto, arquitetura e plano de execução) está em [`docs/`](docs/).

## Comandos planejados (MVP)

Nada disso existe ainda — é a superfície que o MVP vai entregar:

```bash
dev install python      # instala a linguagem e suas ferramentas padrão
dev init python         # cria um projeto a partir de template embutido
dev doctor              # diagnostica o ambiente (git, linguagens, PATH, ferramentas)
dev config get|set      # lê e altera preferências do usuário
```

Flags globais planejadas: `--dry-run` (mostra os comandos sem executar), `--yes`/`-y` (sem confirmação interativa), `-v`/`-q` (verbosidade) e `--config <arquivo>` (configuração alternativa).

Providers de linguagem no MVP: **Python** e **Rust**. Node.js, Go e Java estão previstos para depois. Adapters de sistema no MVP: **Homebrew** (macOS) e **apt** (Debian/Ubuntu).

## Como é por dentro (resumo)

Arquitetura modular baseada em **Providers**: cada linguagem é uma struct Rust + um `manifest.toml` declarativo. Diferenças de SO ficam atrás de um `OsAdapter`; toda execução de processo passa por um `CommandRunner` (o que viabiliza testes com fakes e o `--dry-run` global). Templates e recursos são embutidos no binário com `rust-embed`. Detalhes e justificativas em [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

## Documentação

- [`docs/PRD.md`](docs/PRD.md) — visão, problema, requisitos e casos de uso
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — design da arquitetura e decisões (com o *porquê* de cada uma)
- [`docs/ROADMAP.md`](docs/ROADMAP.md) — epics e tarefas do MVP

## Como compilar e executar

Requer Rust (edition 2024). Com a toolchain instalada:

```bash
cargo build              # compila
cargo run --bin dev      # executa o binário "dev"
cargo test               # roda os testes
```

## Licença

A definir.
