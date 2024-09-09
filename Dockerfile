FROM alpine:latest
RUN apk add --no-cache rust cargo
ENV CARGO_HOME=/root/.cargo
COPY binary/cargo-audit /root/.cargo/bin/
COPY binary/audit-check /audit-check
ENTRYPOINT ["/audit-check"]
