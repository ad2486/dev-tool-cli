**English** | [Português](rust.pt-BR.md) · [Back to README](../../README.md)

# Rust

`dev` manages Rust through [rustup](https://rustup.rs/), the official toolchain installer. Every check and every `cargo` command goes through rustup, so a `rustc` installed some other way does not count.

## What gets installed

| Tool | How | Where |
|---|---|---|
| rustup | rustup's official script: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh -s -- -y` | `~/.cargo/bin` |
| Rust (`rustc`, `cargo`) | The `stable` toolchain, installed by the script above | `~/.rustup/toolchains` |

The same method runs on macOS and Linux, with no `sudo`. The `rustc` in apt is often several versions behind, too old for edition 2024.

## `dev install rust`

1. **rustup:** skipped if `rustup` is already on your `PATH`.
2. **Toolchain:** skipped if rustup has an active toolchain (`rustup show active-toolchain`). Otherwise, `rustup default stable`.

### Why a `rustc` outside rustup does not count

A machine can hold two compilers, for example one from rustup and one from Homebrew, and the one that runs depends on your `PATH`. `dev` asks rustup which toolchain is active and runs `rustc` and `cargo` through it (`rustup run <toolchain> ...`), so it always uses the same compiler.

## `dev init rust`

In the current directory:

1. Runs `dev install rust`.
2. `cargo init --edition <edition>`, through the active toolchain: creates `Cargo.toml`, `src/main.rs`, `.gitignore`, and a git repository.
3. Records `rust` in `dev.toml`, with the versions of `rustup` and `rustc`.

If `Cargo.toml` already exists, step 2 is skipped and your project is left as it is.

## `dev doctor`

| Check | `ok` when |
|---|---|
| `rustup` | `rustup` is found on the `PATH` or in `~/.cargo/bin` |
| `rust` | rustup has an active toolchain |

If `dev.toml` recorded a version, a different major.minor version is reported, for example `1.90.0 (project uses 1.98.0)`.

## Configuration

| Key | Default | Used by |
|---|---|---|
| `rust.edition` | `2024` | `dev init`, passed to `cargo init --edition` |

```bash
dev config set rust.edition 2021
```

Changing a key affects projects created afterwards, not existing ones.

## `PATH`

The rustup installer adds `~/.cargo/bin` to your shell's startup file, which only affects **new** terminals. `dev` finds rustup there even in the same terminal it was installed from. To run `cargo` yourself, open a new terminal.
