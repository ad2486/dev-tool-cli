**English** | [Português](python.pt-BR.md) · [Back to README](../../README.md)

# Python

`dev` manages Python through [uv](https://docs.astral.sh/uv/). uv installs Python itself, creates the project, and manages its dependencies. `dev` never uses the system's `python3`.

## What gets installed

| Tool | How | Where |
|---|---|---|
| uv | uv's official script: `curl -LsSf https://astral.sh/uv/install.sh \| sh` | `~/.local/bin` |
| Python | `uv python install` | Managed by uv (`~/.local/share/uv/python`) |

The same method runs on macOS and Linux. It needs no `sudo`, and it gets current versions, which system package managers often lag behind.

## `dev install python`

1. **uv:** skipped if `uv` is already on your `PATH`, however it got there (Homebrew included).
2. **Python:** skipped if uv already manages a Python (`uv python list --managed-python --only-installed` is not empty).

### Why the system `python3` does not count

macOS ships a `python3`, and so do Homebrew and most Linux distributions. Which one runs depends on your `PATH`, and its version is whatever the system chose. A project set up by `dev` uses a Python that uv manages, so every check asks uv, not the `PATH`.

## `dev init python`

In the current directory:

1. Runs `dev install python`.
2. `uv init`: creates `pyproject.toml`, a starter `main.py`, `.gitignore`, and a git repository.
3. `uv add --dev <tools>`: adds your formatter, linter, and tester as development dependencies. A tool used for two roles is added once, so the defaults become `uv add --dev ruff pytest`.
4. Records `python` in `dev.toml`, with the versions of `uv` and Python.

If `pyproject.toml` already exists, steps 2 and 3 are skipped and your project is left as it is.

## `dev doctor`

| Check | `ok` when |
|---|---|
| `uv` | `uv` is found on the `PATH` or in `~/.local/bin` |
| `python` | uv manages at least one Python |

If `dev.toml` recorded a version, a different major.minor version is reported, for example `3.13.0 (project uses 3.12.1)`.

## Configuration

| Key | Default | Used by |
|---|---|---|
| `python.formatter` | `ruff` | `dev init`, as a dev dependency |
| `python.linter` | `ruff` | `dev init`, as a dev dependency |
| `python.tester` | `pytest` | `dev init`, as a dev dependency |

Values are package names passed to `uv add`, so any package on PyPI works:

```bash
dev config set python.formatter black
dev config set python.linter flake8
```

Changing a key affects projects created afterwards, not existing ones.

## `PATH`

The uv installer adds `~/.local/bin` to your shell's startup file, which only affects **new** terminals. `dev` finds uv there even in the same terminal it was installed from. To run `uv` yourself, open a new terminal.
