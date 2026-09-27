[English](python.md) | **Português** · [Voltar ao README](../../README.pt-BR.md)

# Python

O `dev` gerencia o Python pelo [uv](https://docs.astral.sh/uv/). O uv instala o próprio Python, cria o projeto e gerencia as dependências dele. O `dev` nunca usa o `python3` do sistema.

## O que é instalado

| Ferramenta | Como | Onde |
|---|---|---|
| uv | Script oficial do uv: `curl -LsSf https://astral.sh/uv/install.sh \| sh` | `~/.local/bin` |
| Python | `uv python install` | Gerenciado pelo uv (`~/.local/share/uv/python`) |

O mesmo método roda no macOS e no Linux. Não precisa de `sudo` e traz versões atuais, que os gerenciadores de pacotes do sistema costumam demorar para oferecer.

## `dev install python`

1. **uv:** pulado se o `uv` já está no seu `PATH`, não importa como chegou lá (Homebrew incluso).
2. **Python:** pulado se o uv já gerencia algum Python (`uv python list --managed-python --only-installed` não está vazio).

### Por que o `python3` do sistema não conta

O macOS vem com um `python3`, e o Homebrew e a maioria das distribuições Linux também. Qual deles roda depende do seu `PATH`, e a versão é a que o sistema escolheu. Um projeto criado pelo `dev` usa um Python gerenciado pelo uv, então toda verificação pergunta ao uv, e não ao `PATH`.

## `dev init python`

Na pasta atual:

1. Roda o `dev install python`.
2. `uv init`: cria o `pyproject.toml`, um `main.py` inicial, o `.gitignore` e um repositório git.
3. `uv add --dev <ferramentas>`: adiciona seu formatter, linter e tester como dependências de desenvolvimento. Uma ferramenta usada em dois papéis entra uma vez só, então os defaults viram `uv add --dev ruff pytest`.
4. Registra `python` no `dev.toml`, com as versões do `uv` e do Python.

Se o `pyproject.toml` já existe, os passos 2 e 3 são pulados e seu projeto fica como está.

## `dev doctor`

| Verificação | `ok` quando |
|---|---|
| `uv` | o `uv` é encontrado no `PATH` ou em `~/.local/bin` |
| `python` | o uv gerencia pelo menos um Python |

Se o `dev.toml` registrou uma versão, uma versão major.minor diferente é apontada, por exemplo `3.13.0 (project uses 3.12.1)`.

## Configuração

| Chave | Padrão | Usada por |
|---|---|---|
| `python.formatter` | `ruff` | `dev init`, como dependência de desenvolvimento |
| `python.linter` | `ruff` | `dev init`, como dependência de desenvolvimento |
| `python.tester` | `pytest` | `dev init`, como dependência de desenvolvimento |

Os valores são nomes de pacotes passados ao `uv add`, então qualquer pacote do PyPI funciona:

```bash
dev config set python.formatter black
dev config set python.linter flake8
```

Mudar uma chave afeta os projetos criados depois, não os que já existem.

## `PATH`

O instalador do uv acrescenta `~/.local/bin` ao arquivo de inicialização do seu shell, o que só vale para terminais **novos**. O `dev` encontra o uv lá mesmo no terminal em que ele foi instalado. Para rodar o `uv` você mesmo, abra um terminal novo.
