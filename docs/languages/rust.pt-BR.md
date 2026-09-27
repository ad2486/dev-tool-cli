[English](rust.md) | **Português** · [Voltar ao README](../../README.pt-BR.md)

# Rust

O `dev` gerencia o Rust pelo [rustup](https://rustup.rs/), o instalador oficial de toolchains. Toda verificação e todo comando `cargo` passam pelo rustup, então um `rustc` instalado de outro jeito não conta.

## O que é instalado

| Ferramenta | Como | Onde |
|---|---|---|
| rustup | Script oficial do rustup: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh -s -- -y` | `~/.cargo/bin` |
| Rust (`rustc`, `cargo`) | A toolchain `stable`, instalada pelo script acima | `~/.rustup/toolchains` |

O mesmo método roda no macOS e no Linux, sem `sudo`. O `rustc` do apt costuma estar várias versões atrás, antigo demais para a edition 2024.

## `dev install rust`

1. **rustup:** pulado se o `rustup` já está no seu `PATH`.
2. **Toolchain:** pulada se o rustup tem uma toolchain ativa (`rustup show active-toolchain`). Senão, `rustup default stable`.

### Por que um `rustc` fora do rustup não conta

Uma máquina pode ter dois compiladores, por exemplo um do rustup e outro do Homebrew, e o que roda depende do seu `PATH`. O `dev` pergunta ao rustup qual toolchain está ativa e roda o `rustc` e o `cargo` por ela (`rustup run <toolchain> ...`), então usa sempre o mesmo compilador.

## `dev init rust`

Na pasta atual:

1. Roda o `dev install rust`.
2. `cargo init --edition <edition>`, pela toolchain ativa: cria o `Cargo.toml`, o `src/main.rs`, o `.gitignore` e um repositório git.
3. Registra `rust` no `dev.toml`, com as versões do `rustup` e do `rustc`.

Se o `Cargo.toml` já existe, o passo 2 é pulado e seu projeto fica como está.

## `dev doctor`

| Verificação | `ok` quando |
|---|---|
| `rustup` | o `rustup` é encontrado no `PATH` ou em `~/.cargo/bin` |
| `rust` | o rustup tem uma toolchain ativa |

Se o `dev.toml` registrou uma versão, uma versão major.minor diferente é apontada, por exemplo `1.90.0 (project uses 1.98.0)`.

## Configuração

| Chave | Padrão | Usada por |
|---|---|---|
| `rust.edition` | `2024` | `dev init`, passada ao `cargo init --edition` |

```bash
dev config set rust.edition 2021
```

Mudar uma chave afeta os projetos criados depois, não os que já existem.

## `PATH`

O instalador do rustup acrescenta `~/.cargo/bin` ao arquivo de inicialização do seu shell, o que só vale para terminais **novos**. O `dev` encontra o rustup lá mesmo no terminal em que ele foi instalado. Para rodar o `cargo` você mesmo, abra um terminal novo.
