# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`audit-check` is a GitHub Action that runs `cargo audit` inside a Docker container and optionally creates a GitHub issue when RustSec advisories are found. The binary is built for `x86_64-unknown-linux-musl`, copied into `binary/audit-check`, and bundled with a pre-built `cargo-audit` binary into an Alpine-based Docker image.

## Commands

```sh
# Build
cargo build

# Run tests
cargo test

# Run a single test
cargo test <test_name>

# Lint (nightly required for full lint set)
cargo clippy

# Check with nightly (activates the full deny list in main.rs)
cargo +nightly check
```

The build script (`build.rs`) uses `rustversion` to set a `nightly` cfg flag; the massive `deny(...)` lint list in `main.rs` only activates on nightly.

## Architecture

### Execution flow

`main` → `runtime::run()`:
1. Reads config from environment variables (`INPUT_TOKEN`, `INPUT_DENY`, `INPUT_LEVEL`, `INPUT_CREATE_ISSUE`, `GITHUB_REPOSITORY`).
2. Checks rustc meets MSRV (1.57.0) via `check::rustc`.
3. Verifies `cargo audit` is installed via `check::installed`.
4. Spawns `cargo audit -D{deny}` as a subprocess (`audit::audit`), fanning stdout/stderr/exit-code out over three `std::sync::mpsc` channels to three reader threads.
5. If exit code is non-zero and `create_issue` is true, builds a Tokio runtime, calls `create_issue` async fn which POSTs to the GitHub Issues API.

### Key modules

| Module | Role |
|---|---|
| `config` | Reads all inputs from env vars; the only required var is `INPUT_TOKEN` |
| `audit` | Spawns `cargo audit` subprocess, streams stdout/stderr line-by-line |
| `runtime` | Orchestrates threads, parses RustSec output with regexes, calls GitHub API |
| `check::rustc` | MSRV gate |
| `check::installed` | Verifies `cargo audit --version` exits 0 |
| `log` | Initializes `tracing-subscriber` with ISO 8601 timestamps |
| `error` | `AuditCheckError` via `thiserror` |

### RustSec parsing

`runtime::parse` splits `cargo audit` stdout on double-newlines, then applies per-field regexes (`CRATE_REGEX`, `VERSION_REGEX`, etc.) to extract structured `Rustsec` values. Results are collected into a `BTreeMap<String, (String, Rustsec)>` keyed by advisory ID.

### Docker / deployment

The action runs via Docker (see `action.yml`). Pre-built musl binaries are committed to `binary/`. `Dockerfile` copies `binary/cargo-audit` and `binary/audit-check` into the Alpine image — no Rust toolchain is installed in the image itself.

### Git submodule

`rustsec/` is a submodule pointing to `https://github.com/rustsec/rustsec.git`. It is reference material, not compiled as part of this crate.

## Updating binaries

When bumping `cargo-audit` or rebuilding the action binary, replace the files under `binary/` with new musl-linked builds, then update the Docker image tag in `action.yml` if applicable.
