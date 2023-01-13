FROM alpine:latest
RUN apk add --no-cache gcc musl-dev rust cargo
ADD advisory-db /root/.cargo/
COPY binary/cargo-audit /root/.cargo/bin/
COPY binary/audit-check /audit-check
ENTRYPOINT ["/audit-check"]
