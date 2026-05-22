#!/usr/bin/env fish
docker pull ghcr.io/rustyhorde/audit-check:latest; and \
docker run -e INPUT_TOKEN -e GITHUB_REPOSITORY \
    -v cargo-cache:/root/.cargo/registry -v (pwd):/volume \
    -w=/volume --rm -t ghcr.io/rustyhorde/audit-check:latest
