# audit-check

[![CI](https://github.com/rustyhorde/audit-check/actions/workflows/cargo-matrix.yml/badge.svg?branch=master)](https://github.com/rustyhorde/audit-check/actions/workflows/cargo-matrix.yml)
[![crates.io](https://img.shields.io/crates/v/audit-check)](https://crates.io/crates/audit-check)
[![license](https://img.shields.io/crates/l/audit-check)](LICENSE-MIT)

A GitHub Action that runs [`cargo audit`](https://github.com/rustsec/rustsec/tree/main/cargo-audit) on your Rust project and optionally opens a GitHub issue when RustSec advisories are found.

## Inputs

| Input | Required | Default | Description |
|-------|----------|---------|-------------|
| `token` | yes | — | GitHub token (`${{ secrets.GITHUB_TOKEN }}`) |
| `deny` | no | `warnings` | Fail on: `warnings` (any), `unmaintained`, `unsound`, `yanked` |
| `level` | no | `INFO` | Log level: `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR` |
| `create_issue` | no | `false` | Open a GitHub issue when advisories are found |

## Usage

### GitHub Action (recommended)

Minimal — add to any workflow step:

```yaml
- uses: rustyhorde/audit-check@v1
  with:
    token: ${{ secrets.GITHUB_TOKEN }}
```

Full example with all options:

```yaml
- uses: rustyhorde/audit-check@v1
  with:
    token: ${{ secrets.GITHUB_TOKEN }}
    deny: warnings      # warnings | unmaintained | unsound | yanked
    level: INFO         # TRACE | DEBUG | INFO | WARN | ERROR
    create_issue: false # true | false
```

A complete scheduled audit workflow:

```yaml
name: Security Audit

on:
  push:
    branches: [master]
  schedule:
    - cron: '0 0 * * 0'  # weekly on Sunday

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustyhorde/audit-check@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
          create_issue: true
```

### Docker

The image is published to [GitHub Container Registry](https://github.com/rustyhorde/audit-check/pkgs/container/audit-check).
Run it locally against a Rust project:

```sh
docker pull ghcr.io/rustyhorde/audit-check:latest

docker run \
  -e INPUT_TOKEN=<github-token> \
  -e GITHUB_REPOSITORY=owner/repo \
  -v "$(pwd):/volume" \
  -w /volume \
  --rm \
  ghcr.io/rustyhorde/audit-check:latest
```

Optional env vars:

```sh
-e INPUT_DENY=warnings       # warnings | unmaintained | unsound | yanked
-e INPUT_LEVEL=INFO          # TRACE | DEBUG | INFO | WARN | ERROR
-e INPUT_CREATE_ISSUE=false  # true | false
```

### Standalone CLI

Install the binary and run it directly. `cargo audit` must also be installed.

**Install:**

```sh
# Pre-built binary via cargo-binstall (fastest)
cargo binstall audit-check

# Or build from source
cargo install audit-check

# cargo-audit is a required runtime dependency
cargo install cargo-audit
```

**Run:**

```sh
export INPUT_TOKEN=<github-token>
export GITHUB_REPOSITORY=owner/repo  # e.g. rustyhorde/audit-check

# Optional
export INPUT_DENY=warnings
export INPUT_LEVEL=INFO
export INPUT_CREATE_ISSUE=false

audit-check
```

> `INPUT_TOKEN` is always required even when `create_issue` is `false`. A classic PAT with
> `public_repo` scope (or `repo` for private repositories) is sufficient.
