#!/usr/bin/env fish
cargo fmt --all -- --check; and \
cargo clippy --all-targets --all-features -- -Dwarnings; and \
cargo build-all-features; and \
cargo test-all-features; and \
docker run -v cargo-cache:/root/.cargo/registry -v (pwd):/volume -v ~/.gitconfig:/root/.gitconfig:ro --rm -t clux/muslrust:nightly cargo build --release; and \
sudo chown -R $USER:$USER target/; and \
cp target/x86_64-unknown-linux-musl/release/audit-check binary/; and \
cd rustsec; and \
docker run -v cargo-cache:/root/.cargo/registry -v (pwd):/volume -v ~/.gitconfig:/root/.gitconfig:ro --rm -t clux/muslrust:nightly cargo build -p cargo-audit --release; and \
sudo chown -R $USER:$USER target/; and \
cp target/x86_64-unknown-linux-musl/release/cargo-audit ../binary/; and \
cd ..; and \
sudo chown -R $USER:$USER target/; and \
cp target/x86_64-unknown-linux-musl/release/audit-check binary/; and \
docker build -t ozias/audit-check:latest .; and \
docker run -v cargo-cache:/root/.cargo/registry -v (pwd):/volume -w=/volume --rm -t ozias/audit-check:latest