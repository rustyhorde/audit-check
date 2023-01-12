FROM alpine:latest
COPY target/x86_64-unknown-linux-musl/release/audit-check /audit-check
ENTRYPOINT ["/audit-check"]
