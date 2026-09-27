**English** | [Português](README.pt-BR.md)

# dev

Set up a development environment in one command, the same way on every machine.

`dev` installs language toolchains, creates projects that use them, and checks that a project's dependencies are present on the machine. It is a single binary with no runtime to install first.

```bash
dev init python     # installs uv and Python, creates the project, records its versions
dev doctor          # checks that this machine has what the project needs
```

## Supported languages

| Language | Toolchain | Docs |
|---|---|---|
| Python | [uv](https://docs.astral.sh/uv/) | [docs/languages/python.md](docs/languages/python.md) |
| Rust | [rustup](https://rustup.rs/) | [docs/languages/rust.md](docs/languages/rust.md) |

Supported systems: **macOS** (Apple Silicon and Intel) and **Linux** (x86_64 and arm64).

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/ad2486/dev-tool-cli/main/install.sh | sh
```

The script picks the binary for your system, verifies its SHA-256 checksum, and installs it to `~/.local/bin`. It never uses `sudo`. If that directory is not on your `PATH`, the script tells you how to add it.

| Variable | Default | Purpose |
|---|---|---|
| `DEV_INSTALL_DIR` | `~/.local/bin` | Where to put the binary |
| `DEV_VERSION` | `latest` | A specific release, e.g. `v0.1.0` |

You can also download a binary from the [Releases](https://github.com/ad2486/dev-tool-cli/releases) page. To uninstall, delete the binary: `rm ~/.local/bin/dev`.

## Usage

### `dev install <language>`

Installs the language's toolchain. Anything already installed is skipped, so running it twice is safe.

### `dev init <language>`

Runs `install`, then creates a project **in the current directory** and records it in `dev.toml`. Run it again with another language to add a second component to the same project.

```toml
# dev.toml
schema = 1
components = ["python"]

[dependencies]
python = "3.12.1"
uv = "0.12.17"
```

Commit `dev.toml` with your project: it is what `dev doctor` reads on other machines.

### `dev doctor`

Checks the components listed in `dev.toml` and prints one line per dependency:

```
python:
  uv           ok
  python       3.13.0 (project uses 3.12.1)
```

A dependency is `ok`, `missing`, or installed at a different **major.minor** version than the project recorded. Patch differences are ignored. `dev doctor` exits with 0 even when it finds problems: the report is the result. Outside a project, it exits with 2 and suggests `dev init`.

### `dev config get|set`

Reads and writes your preferences, such as which formatter a new Python project gets. Keys are `<language>.<key>`:

```bash
dev config get                          # every key, with its effective value
dev config get python.formatter         # python.formatter = ruff (default)
dev config set python.formatter black
```

`(default)` marks values that come from `dev` itself rather than your config. Each language's page lists its keys. An unknown key is an error, and nothing is written.

The config file lives at `~/Library/Application Support/dev/config.toml` on macOS and `~/.config/dev/config.toml` on Linux. `set` keeps your comments and formatting.

### Global options

| Option | Effect |
|---|---|
| `--dry-run` | Print the commands that would modify the system instead of running them. Read-only checks still run, so the plan matches the machine |
| `--config <file>` | Use another config file |
| `-v`, `-vv` | More log output |
| `-q` | No log output (command results, such as the doctor report, still print) |
| `-y`, `--yes` | Reserved for confirmation prompts; `dev` does not prompt yet |

### Exit codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 2 | Usage error: unknown language or key, invalid config, not a `dev` project |
| 3 | Environment problem: no supported package manager, unreadable file |
| 4 | A tool that `dev` ran failed |
| 70 | Bug in `dev` |

## Build from source

Requires Rust 1.85 or later (edition 2024).

```bash
cargo build --release     # binary at target/release/dev
cargo test
```

## Contributing

The design documents are in Portuguese:

- [docs/PRD.md](docs/PRD.md): goals, requirements, and what is out of scope
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): how `dev` is built, and why each decision was made
- [docs/ROADMAP.md](docs/ROADMAP.md): planned work

## License

[MIT](LICENSE)
