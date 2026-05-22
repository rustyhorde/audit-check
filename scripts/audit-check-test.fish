#!/usr/bin/env fish
set tag (if set -q argv[1]; echo $argv[1]; else; echo latest; end)

if set -q argv[2]
    set -x GITHUB_REPOSITORY $argv[2]
end

if not set -q INPUT_TOKEN
    echo "ERROR: INPUT_TOKEN is not set"
    exit 1
end

if not set -q GITHUB_REPOSITORY
    echo "ERROR: GITHUB_REPOSITORY is not set (pass as 2nd argument or set the env var)"
    exit 1
end

docker pull ghcr.io/rustyhorde/audit-check:$tag; and \
docker run -e INPUT_TOKEN -e GITHUB_REPOSITORY \
    -v cargo-cache:/root/.cargo/registry -v (pwd):/volume \
    -w=/volume --rm -t ghcr.io/rustyhorde/audit-check:$tag
