[English](README.md) | **Português**

# dev

Configure um ambiente de desenvolvimento com um comando, do mesmo jeito em qualquer máquina.

O `dev` instala toolchains de linguagens, cria projetos que as usam e verifica se a máquina tem as dependências de um projeto. É um binário único, sem runtime para instalar antes.

```bash
dev init python     # instala o uv e o Python, cria o projeto, registra as versões
dev doctor          # verifica se esta máquina tem o que o projeto precisa
```

## Linguagens suportadas

| Linguagem | Toolchain | Documentação |
|---|---|---|
| Python | [uv](https://docs.astral.sh/uv/) | [docs/languages/python.pt-BR.md](docs/languages/python.pt-BR.md) |
| Rust | [rustup](https://rustup.rs/) | [docs/languages/rust.pt-BR.md](docs/languages/rust.pt-BR.md) |

Sistemas suportados: **macOS** (Apple Silicon e Intel) e **Linux** (x86_64 e arm64).

## Instalação

```bash
curl -fsSL https://raw.githubusercontent.com/ad2486/dev-tool-cli/main/install.sh | sh
```

O script escolhe o binário do seu sistema, confere o checksum SHA-256 e instala em `~/.local/bin`. Ele nunca usa `sudo`. Se essa pasta não estiver no seu `PATH`, o script mostra como adicionar.

| Variável | Padrão | Para quê |
|---|---|---|
| `DEV_INSTALL_DIR` | `~/.local/bin` | Onde colocar o binário |
| `DEV_VERSION` | `latest` | Uma versão específica, ex.: `v0.1.0` |

Também dá para baixar o binário na página de [Releases](https://github.com/ad2486/dev-tool-cli/releases). Para desinstalar, apague o binário: `rm ~/.local/bin/dev`.

## Uso

### `dev install <linguagem>`

Instala a toolchain da linguagem. O que já está instalado é pulado, então rodar duas vezes é seguro.

### `dev init <linguagem>`

Roda o `install`, cria um projeto **na pasta atual** e o registra no `dev.toml`. Rode de novo com outra linguagem para acrescentar um segundo componente ao mesmo projeto.

```toml
# dev.toml
schema = 1
components = ["python"]

[dependencies]
python = "3.12.1"
uv = "0.12.17"
```

Faça commit do `dev.toml` junto com o projeto: é ele que o `dev doctor` lê nas outras máquinas.

### `dev doctor`

Verifica os componentes listados no `dev.toml` e imprime uma linha por dependência:

```
python:
  uv           ok
  python       3.13.0 (project uses 3.12.1)
```

Cada dependência aparece como `ok`, `missing` ou instalada numa versão **major.minor** diferente da que o projeto registrou. Diferenças só de patch são ignoradas. O `dev doctor` sai com 0 mesmo quando encontra problemas: o relatório é o resultado. Fora de um projeto, sai com 2 e sugere o `dev init`.

### `dev config get|set`

Lê e grava suas preferências, como qual formatter um projeto Python novo recebe. As chaves são `<linguagem>.<chave>`:

```bash
dev config get                          # todas as chaves, com o valor efetivo
dev config get python.formatter         # python.formatter = ruff (default)
dev config set python.formatter black
```

`(default)` marca valores que vêm do próprio `dev`, e não da sua config. A página de cada linguagem lista as chaves dela. Chave desconhecida é erro, e nada é gravado.

O arquivo de config fica em `~/Library/Application Support/dev/config.toml` no macOS e em `~/.config/dev/config.toml` no Linux. O `set` preserva seus comentários e a formatação.

### Opções globais

| Opção | Efeito |
|---|---|
| `--dry-run` | Mostra os comandos que modificariam o sistema, em vez de rodá-los. As verificações de leitura rodam normalmente, então o plano corresponde à máquina |
| `--config <arquivo>` | Usa outro arquivo de config |
| `-v`, `-vv` | Mais log |
| `-q` | Nenhum log (resultados de comandos, como o relatório do doctor, continuam aparecendo) |
| `-y`, `--yes` | Reservado para confirmações; o `dev` ainda não pergunta nada |

### Códigos de saída

| Código | Significado |
|---|---|
| 0 | Sucesso |
| 2 | Erro de uso: linguagem ou chave desconhecida, config inválida, pasta que não é um projeto `dev` |
| 3 | Problema no ambiente: nenhum gerenciador de pacotes suportado, arquivo ilegível |
| 4 | Uma ferramenta que o `dev` rodou falhou |
| 70 | Bug no `dev` |

## Compilar do código-fonte

Requer Rust 1.85 ou mais novo (edition 2024).

```bash
cargo build --release     # binário em target/release/dev
cargo test
```

## Contribuindo

Os documentos de design:

- [docs/PRD.md](docs/PRD.md): objetivos, requisitos e o que está fora do escopo
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): como o `dev` é construído, e o porquê de cada decisão
- [docs/ROADMAP.md](docs/ROADMAP.md): trabalho planejado

## Licença

[MIT](LICENSE)
