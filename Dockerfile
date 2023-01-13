FROM alpine:latest
RUN apk add --no-cache gcc musl-dev rust cargo
ENV CARGO_HOME /root/.cargo
COPY binary/cargo-audit /root/.cargo/bin/
COPY binary/audit-check /audit-check
ENTRYPOINT ["/audit-check"]
