# Dev CLI — Architecture

# Visão Geral

A aplicação seguirá uma arquitetura modular baseada em **Providers**.

O **Core** é responsável apenas por coordenar a execução da aplicação. Toda lógica específica de linguagens fica encapsulada em seus respectivos Providers.

```
CLI (clap)

↓

Handler do subcomando

↓

Core
├── Config Manager
└── Provider Registry

↓

Provider

↓

OS Adapter

↓

CommandRunner

↓

Package Manager / Ferramentas do Sistema
```

**Por quê não existe Command Dispatcher:** o clap já faz o parsing dos argumentos e o dispatch de cada subcomando para seu handler. Um dispatcher próprio sobre o clap seria duplo despacho — duas camadas com a mesma responsabilidade e mais código para manter e testar, sem ganho. Os handlers (em `cli/`) resolvem o Provider no Registry e o chamam diretamente.

---

# Estrutura

O projeto é um **crate único com módulos**, não um workspace.

```
src/
├── cli/          # definição dos comandos (clap) e seus handlers
├── core/         # inicialização e orquestração
├── providers/    # trait Provider + implementações (+ recursos embutidos)
├── os/           # OsAdapter, CommandRunner, detecção de SO/gerenciador
├── config/       # configuração do usuário
└── errors/       # taxonomia de erros e códigos de saída
```

**Por quê crate único:** o produto é um único binário (`[[bin]] name = "dev"` no Cargo.toml). Um workspace multi-crate adicionaria overhead de versionamento, dependências internas e tempo de compilação sem benefício neste tamanho de projeto. Se algum módulo precisar virar biblioteca reutilizável no futuro, a extração é simples.

**Por quê sem `utils/`:** módulos `utils` viram gaveta sem dono — qualquer código sem lugar claro vai parar ali, criando acoplamento difuso e dificultando descobrir onde as coisas vivem. Cada utilidade deve viver no módulo do conceito a que pertence.

**Por quê sem `commands/`:** sem Command Dispatcher, não há camada de comandos separada; os handlers vivem em `cli/`, ao lado da definição clap que os dispara.

---

# Core

O Core é o ponto central da aplicação.

Responsabilidades:

* inicializar a aplicação
* registrar componentes (Providers, Adapters, CommandRunner)
* carregar a configuração
* tratamento global de erros (mapear categoria → código de saída)

O Core **não conhece detalhes de nenhuma linguagem**.

---

# Config Manager

Responsável por gerenciar a configuração do usuário.

Funções:

* localizar `config.toml`
* carregar preferências
* validar configurações
* resolver a configuração efetiva de cada Provider (config do usuário + defaults do manifest)
* disponibilizar configurações aos Providers (via `InstallContext`)

Exemplo:

```
~/.config/dev/config.toml
```

```toml
[python]
formatter = "black"   # sobrescreve o default do manifest ("ruff")

[rust]
edition = "2024"
```

Esse arquivo contém apenas preferências do usuário e nunca modifica os Providers.

## Precedência config ↔ manifest

1. **A config do usuário vence**: qualquer chave definida em `config.toml` sobrescreve o default do `manifest.toml` do Provider. No exemplo acima, o Python usará `black` como formatter, e não o `ruff` definido como padrão no manifest.
2. **O manifest fornece os defaults**: chaves ausentes na config do usuário usam o valor do manifest.
3. **Chave desconhecida = erro de validação**: uma chave fora do schema do Provider (ex.: `formattr = "black"`) é erro de usuário (código de saída 2), reportado na validação — nunca ignorada silenciosamente.

**Por quê:** config silenciosamente ignorada (um typo que "passa" mas não tem efeito) é uma das falhas mais frustrantes em CLIs; falhar rápido torna o erro óbvio. Manter os defaults no manifest, e não na config, permite evoluir Providers sem exigir migração das configs existentes.

---

# Provider Registry

Responsável por registrar e disponibilizar todos os Providers da aplicação.

Funções:

* registrar Providers
* localizar Providers
* disponibilizar Providers aos handlers de comando

Isso permite adicionar novas linguagens sem alterar o Core.

## Manifest Loader

Componente acionado pelo Registry na inicialização, responsável pelo ciclo de vida do `manifest.toml` de cada Provider:

* fazer o parsing do manifest (embutido no binário com os demais recursos)
* validar a versão do `schema` antes de interpretar o arquivo
* validar o conteúdo (chaves desconhecidas = erro, fail fast)
* entregar a estrutura `Manifest` pronta para o Provider e para o Config Manager (resolução de precedência)

**Por quê um componente separado:** o Registry gerencia o catálogo de Providers; parsing e validação de manifest são uma responsabilidade distinta, com regras próprias (schema versionado). Separar permite testar o loader isoladamente e evoluir o schema sem tocar no Registry.

---

# Providers

Cada linguagem é implementada como um **Provider**, responsável por toda a lógica relacionada àquela linguagem.

Exemplos:

* PythonProvider
* RustProvider
* NodeProvider
* GoProvider

Providers são **híbridos**: uma struct Rust contém o comportamento, e um `manifest.toml` contém os dados declarativos.

**Por quê híbrido:** manifest puro (só dados) não expressa lógica condicional de instalação (ex.: instalar Rust via rustup, mas uv via gerenciador de pacotes); código puro (só Rust) obrigaria editar código e recompilar para trocar um nome de pacote ou uma ferramenta padrão. O manifest guarda o que muda com frequência; a struct guarda o que é comportamento.

## Trait Provider

A trait `Provider` é a peça central da arquitetura:

```rust
trait Provider {
    fn name(&self) -> &str;

    fn manifest(&self) -> &Manifest;

    fn install(&self, ctx: &InstallContext) -> Result<()>;

    fn doctor(&self, ctx: &InstallContext) -> Result<Report>;

    fn init(&self, ctx: &InstallContext, opts: InitOpts) -> Result<()>;

    // uninstall / update: pós-MVP
}
```

`InstallContext` é o pacote de dependências que o Core entrega ao Provider:

* a configuração efetiva do Provider (config do usuário + defaults do manifest, já resolvida)
* um handle do `OsAdapter` (operações de SO e gerenciador de pacotes)
* um handle do `CommandRunner` (execução de processos)

**Por quê um contexto explícito:** Providers não acessam estado global nem executam processos diretamente; toda dependência chega por injeção. Isso permite testar Providers unitariamente com fakes (CommandRunner que grava comandos, OsAdapter em memória) sem tocar no sistema real.

## Recursos do Provider (embutidos no binário)

Cada Provider possui um pacote próprio de recursos:

```
providers/
└── python/
    ├── manifest.toml
    ├── templates/
    ├── scripts/
    └── assets/
```

Esses recursos são **embutidos no binário em tempo de compilação** com `rust-embed` (alternativa: `include_dir`), não lidos do disco em runtime.

**Por quê:** pastas soltas ao lado do executável contradizem o RNF "Binário único" — o usuário teria que carregar um diretório de recursos junto com o `dev`. Com embedding, o binário continua autocontido e os recursos ficam versionados junto com o código do Provider. `rust-embed` é a escolha preferida porque em builds de debug lê do disco (loop de desenvolvimento rápido, sem recompilar a cada edição de template) e só embute de fato em builds de release.

## Manifest (schema v1)

O `manifest.toml` descreve os dados do Provider:

```toml
schema = 1
name = "python"

[tools]            # ferramentas padrão (a config do usuário pode sobrescrever)
formatter = "ruff"
linter = "ruff"
tester = "pytest"

[packages]         # nome do pacote por gerenciador de pacotes
apt = "python3"
brew = "python"

[packages.uv]      # pacotes auxiliares também são tabelados por gerenciador
apt = "uv"
brew = "uv"

[commands]
init = "uv init"
```

Regras:

* `[packages]` mapeia gerenciador de pacotes → nome do pacote (ex.: `python3` no apt vs. `python` no brew), evitando `if/else` de plataforma espalhados pelo código do Provider.
* chaves desconhecidas no manifest = erro de validação ao carregar (fail fast).
* o manifest define **defaults**; a config do usuário **sobrescreve** (ver Config Manager).

**Por quê schema versionado:** o campo `schema` permite evoluir o formato (v2, v3...) sem quebrar manifests antigos — o carregador valida a versão antes de interpretar o arquivo.

## Templates

A pasta `templates/` contém estruturas de projeto utilizadas pelo comando `dev init`.

Exemplo:

```
templates/
├── default/
├── api/
├── library/
└── cli/
```

Cada template pode possuir sua própria estrutura de arquivos. São embutidos no binário (ver "Recursos do Provider").

## Scripts

A pasta `scripts/` contém scripts auxiliares utilizados pelo Provider quando uma simples execução de comandos não for suficiente.

Exemplos:

* instalações complexas
* configurações específicas
* migrações
* inicializações avançadas

Podem ser separados por sistema operacional:

```
scripts/
├── linux/
├── windows/
└── macos/
```

Scripts também são executados exclusivamente através do `CommandRunner`.

## Assets

A pasta `assets/` contém arquivos estáticos utilizados pelo Provider.

Exemplos:

* `.gitignore`
* `README.md`
* arquivos de configuração
* licenças
* arquivos auxiliares

## Benefícios

* isolamento entre linguagens
* facilidade para adicionar novas linguagens
* evolução independente dos Providers
* suporte a templates específicos
* Providers testáveis unitariamente com fakes
* arquitetura preparada para plugins futuros

---

# OS Adapter

Camada responsável por abstrair diferenças entre sistemas operacionais e gerenciadores de pacotes.

O Adapter **monta** os comandos de cada gerenciador (ex.: `apt install -y python3`) e delega a **execução** ao `CommandRunner`, recebido na construção do Adapter.

Interface conceitual:

```rust
trait OsAdapter {
    fn install(...);

    fn uninstall(...);

    fn update(...);

    fn command_exists(...);

    fn detect_package_manager(...);
}
```

O método `execute` não existe mais aqui: executar processos é responsabilidade exclusiva do `CommandRunner` (ver seção CommandRunner).

Escopo:

* **MVP:** Homebrew (macOS) e apt (Debian/Ubuntu).
* **Previstos (pós-MVP):** pacman, dnf, zypper, apk (Linux); winget, scoop, chocolatey (Windows).

**Por quê o recorte:** Homebrew + apt cobrem os dois ambientes de desenvolvimento mais comuns do público-alvo com a menor superfície possível. A trait foi desenhada para que adicionar um gerenciador seja **aditivo** — nova implementação + uma entrada na detecção — sem tocar em Providers ou Core.

Os Providers nunca executam comandos diretamente; toda interação com o sistema operacional passa pelo OS Adapter.

---

# CommandRunner

O `CommandRunner` é a costura (seam) de mais baixo nível da arquitetura: o **único** ponto do código autorizado a spawnar processos.

Interface conceitual:

```rust
trait CommandRunner {
    fn execute(&self, cmd: &Command) -> Result<ExitStatus>;  // roda herdando stdio

    fn capture(&self, cmd: &Command) -> Result<Output>;      // roda capturando stdout/stderr
}
```

Modos de operação:

* **real:** executa o processo no sistema.
* **dry-run:** não executa; registra e imprime o comando que seria executado (flag global `--dry-run`).
* **recording (testes):** fake que grava a sequência de comandos recebidos e retorna respostas programadas.

**Por quê:** concentrar a execução de processos em uma única abstração entrega três recursos de uma vez só — (1) testes unitários com fakes, sem tocar no sistema real; (2) `--dry-run` global implementado em um único lugar, valendo para todos os comandos; (3) logging centralizado de tudo o que o `dev` executa. Sem essa costura, cada um desses recursos teria que ser reimplementado em cada Provider e Adapter.

---

# Tratamento de Erros

## Taxonomia

Todo erro do `dev` se classifica em uma de quatro categorias:

| Categoria | Significado | Exemplos | Código de saída |
|-----------|-------------|----------|:---:|
| Erro de usuário | entrada ou configuração inválida | `dev install pythn`; chave desconhecida no config.toml | 2 |
| Erro de ambiente | pré-requisito ausente no sistema | SO não suportado; gerenciador de pacotes não encontrado | 3 |
| Falha de ferramenta | comando externo executou e falhou | `apt install` retornou erro; download corrompido | 4 |
| Erro interno | bug ou invariante violada no `dev` | estado impossível; invariante quebrada | 70 |

Sucesso = código 0.

**Por quê esses códigos:** 2 é o mesmo código que o clap usa para erro de uso, mantendo semântica consistente para scripts; 70 segue a convenção `sysexits.h` (EX_SOFTWARE) para falhas internas. Categorias distintas permitem que automação decida se vale tentar de novo (falha de ferramenta) ou não (erro de usuário).

## Estratégia

* **`thiserror` nos módulos**: cada módulo (`config`, `providers`, `os`) define seu próprio enum de erro tipado, com variantes precisas e testáveis.
* **`anyhow` (ou `miette`) na borda do CLI**: os handlers agregam os erros tipados com contexto ("ao instalar python: ...") e o `main` mapeia a categoria para o código de saída e a mensagem final.

**Por quê dois níveis:** `thiserror` dá erros tipados que permitem match exaustivo e testes dentro da aplicação; `anyhow`/`miette` dá ergonomia e cadeia de contexto na fronteira, onde o erro vira texto para o usuário. Usar só `anyhow` em tudo esconderia as categorias (impossibilitando o mapeamento para códigos de saída); usar só `thiserror` em tudo tornaria a borda verbosa.

## Idempotência

Todo comando que modifica o sistema **deve ser idempotente**: rodar `dev install python` duas vezes seguidas produz o mesmo resultado, sem erro na segunda vez. Para isso, Providers verificam o estado atual (via `command_exists`, versão instalada) antes de agir.

**Por quê:** ambientes são reconfigurados com frequência e scripts de bootstrap executam o `dev` repetidamente; um comando não-idempotente forçaria o usuário a tratar erros espúrios. A idempotência também é o que torna o `--dry-run` confiável: as mesmas verificações rodam sem efeitos colaterais.

---

# Fluxo de Execução

```
CLI

↓

Parser (clap)

↓

Handler do subcomando (clap)

↓

Provider Registry

↓

Provider

↓

OS Adapter

↓

CommandRunner

↓

Package Manager / Sistema Operacional
```

---

# Princípios

* Separação de responsabilidades (SRP)
* Arquitetura modular
* Configuração sobre código
* Providers independentes
* Core desacoplado
* Fácil extensibilidade
* Testabilidade (costuras com fakes: Provider, OsAdapter, CommandRunner)
* Comandos idempotentes
* Multiplataforma
* Open/Closed Principle (OCP)

---

# Evolução Futura

A arquitetura foi projetada para suportar recursos futuros sem alterações significativas no Core, como:

* comandos pós-MVP: `uninstall`, `update`, `search`, `template`
* adapters de gerenciadores adicionais (adição aditiva)
* sistema de plugins
* marketplace de templates
* providers externos
* sincronização de configurações
* instalação de stacks completas (ex.: Backend Python, Frontend React, Data Science)
