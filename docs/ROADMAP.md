# Dev CLI — Roadmap

## Epic 1 — Fundação

### DEV-001

Criar crate único com módulos (decisão: crate único, **não** workspace — ver "Estrutura" em ARCHITECTURE.md)

### DEV-002

Configurar Clap, incluindo as flags globais: `--dry-run`, `--yes`, `-v`/`-q`, `--config`

### DEV-003

Criar estrutura inicial de módulos: `cli/`, `core/`, `providers/`, `os/`, `config/`, `errors/` (sem `utils/` nem `commands/` — ver "Estrutura" em ARCHITECTURE.md)

### DEV-004

Criar sistema de logging

### DEV-005

Implementar a estratégia de erros definida em ARCHITECTURE.md (taxonomia de 4 categorias com códigos de saída 0/2/3/4/70, `thiserror` nos módulos + `anyhow`/`miette` na borda do CLI, `main` mapeando categoria → código de saída)

### DEV-006

Criar parser de configuração TOML

---

## Epic 2 — Core

### DEV-010

**Removido.** O clap já faz o dispatch dos subcomandos; os handlers chamam o Provider Registry diretamente (ver ARCHITECTURE.md). Substituído pelo wiring dos handlers em `cli/`.

### DEV-011

Criar Registry de Providers: registrar, localizar e disponibilizar Providers aos handlers, acionando o carregamento e a validação dos manifests (DEV-013)

### DEV-012

Criar trait Provider — `name`, `manifest`, `install`, `doctor` e `init`, todos recebendo o `InstallContext` (DEV-016); `uninstall`/`update` são pós-MVP (ver ARCHITECTURE.md)

### DEV-013

Implementar loader do `manifest.toml` (schema v1): parsing TOML, validação da versão do schema e rejeição de chaves desconhecidas com fail fast (ver "Manifest (schema v1)" em ARCHITECTURE.md)

### DEV-014

Criar sistema de configuração do usuário com resolução da config efetiva de cada Provider: a config do usuário sobrescreve os defaults do manifest e as chaves ausentes usam o default (precedência config ↔ manifest — ver ARCHITECTURE.md)

### DEV-015

Testes do Core e do Registry com fakes da trait Provider

### DEV-016

Criar `InstallContext`: pacote de dependências que o Core entrega aos Providers — config efetiva já resolvida + handle do `OsAdapter` + handle do `CommandRunner` (injeção explícita, sem estado global)

---

## Epic 3 — Sistema Operacional

Escopo do MVP: **Homebrew (macOS) + apt (Debian/Ubuntu)**.

### DEV-020

Criar abstração `CommandRunner`: único ponto do código autorizado a spawnar processos (`execute` herdando stdio e `capture` capturando saída); modos real, dry-run (flag `--dry-run`) e recording (fake de testes)

### DEV-021

Implementar adapter apt (Linux) — MVP

Incluir o modo de simulação nativo (`apt-get -s`) para o `--dry-run`: em vez de apenas registrar o comando, o adapter troca pelo equivalente simulado, que resolve dependências de verdade e valida se o pacote existe (ver "Pendente para o Epic 3" em ARCHITECTURE.md).

### DEV-022

Implementar Windows Adapter (winget/scoop/choco) — pós-MVP

### DEV-023

Implementar adapter Homebrew (macOS) — MVP

Incluir o modo de simulação nativo (`brew install -n`) para o `--dry-run`, pelo mesmo motivo do DEV-021.

### DEV-024

Detectar sistema operacional

### DEV-025

Detectar gerenciador de pacotes (MVP: apt e brew)

### DEV-026

Testes dos adapters com CommandRunner fake de gravação (recording)

### DEV-027

Adapters pós-MVP: pacman, dnf, zypper, apk (Linux). A trait permite adição aditiva, sem alterar Providers ou Core.

### DEV-028

Criar trait `OsAdapter`: monta os comandos de cada gerenciador (ex.: `apt install -y python3`) e delega a execução ao `CommandRunner` recebido na construção — sem método `execute` próprio (ver "OS Adapter" em ARCHITECTURE.md)

---

## Milestone — Vertical Slice: `dev doctor` de ponta a ponta

Logo após o Epic 3 (antes do Epic 4), entregar `dev doctor` funcionando de ponta a ponta: CLI → handler → Registry → Provider → OS Adapter → CommandRunner.

**Por quê:** é o menor comando que atravessa todas as costuras da arquitetura. Validar o fluxo completo agora expõe problemas de design enquanto ainda é barato mudar — antes de multiplicar Providers nos Epics 4 e 5.

---

## Epic 4 — Python

### DEV-030

Criar PythonProvider

### DEV-031

Criar `manifest.toml` do PythonProvider (schema v1: `[tools]`, `[packages]` tabelados por gerenciador, `[commands]`)

### DEV-032

Implementar instalação do Python (idempotente: verificar o estado atual antes de agir — ver "Idempotência" em ARCHITECTURE.md)

### DEV-033

Implementar instalação do uv

### DEV-034

Implementar init com uv

### DEV-035

Implementar doctor

### DEV-036

Implementar update — pós-MVP

### DEV-037

Testes unitários do PythonProvider (fakes de CommandRunner e OsAdapter)

---

## Epic 5 — Rust

### DEV-040

Criar RustProvider

### DEV-041

Implementar rustup

### DEV-042

Implementar cargo init

### DEV-043

Implementar doctor

### DEV-044

Testes unitários do RustProvider (fakes de CommandRunner e OsAdapter)

### DEV-045

Criar `manifest.toml` do RustProvider (schema v1)

---

## Epic 6 — Configuração

### DEV-050

Definir formato e localização do `config.toml` do usuário (`~/.config/dev/config.toml`)

### DEV-051

Implementar leitura das preferências

### DEV-052

Implementar validação das opções (chave desconhecida = erro de usuário — ver ARCHITECTURE.md)

### DEV-053

Implementar `dev config get`

### DEV-054

Implementar `dev config set`

---

## Epic 7 — Templates

### DEV-060

Criar empacotamento dos recursos dos Providers (templates, scripts e assets) embutidos no binário via `rust-embed` (ver "Recursos do Provider" em ARCHITECTURE.md)

### DEV-061

Adicionar template Python

### DEV-062

Adicionar template Rust

### DEV-063

Adicionar template Node — pós-MVP (NodeProvider não está no MVP)

---

## Epic 8 — CLI

### DEV-070

Implementar `dev install <linguagem>`: o handler resolve o Provider no Registry e o chama com o `InstallContext`

### DEV-071

Implementar `dev uninstall` — pós-MVP

### DEV-072

Implementar `dev update` — pós-MVP

### DEV-073

Implementar `dev doctor` — entregue no milestone de vertical slice (após o Epic 3)

### DEV-074

Implementar `dev init <linguagem>` (usa os templates embutidos do Provider — ver Epic 7)

### DEV-075

Implementar `dev search` — pós-MVP

---

## Epic 9 — Testes de Integração e E2E

Testes unitários foram movidos para dentro de cada epic (DEV-015, DEV-026, DEV-037, DEV-044): testar junto da feature evita uma fase de testes em cascata no final do projeto.

### DEV-080

**Movido para os Epics 1–2** (testes unitários junto de cada feature).

### DEV-081

**Movido para os Epics 4–5.**

### DEV-082

**Movido para o Epic 3.**

### DEV-083

Testes de integração

### DEV-084

Testes end-to-end

---

## Epic 10 — Distribuição

### DEV-090

Criar GitHub Actions

### DEV-091

Gerar binários Linux

### DEV-092

Gerar binários Windows

### DEV-093

Gerar binários macOS

### DEV-094

Publicar Releases

### DEV-095

Criar instalador

### DEV-096

Escrever documentação

### DEV-097

Criar exemplos de uso

### DEV-098

Preparar MVP v0.1.0
