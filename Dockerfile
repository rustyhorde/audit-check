FROM alpine:latest
RUN echo "http://dl-cdn.alpinelinux.org/alpine/edge/main" >> /etc/apk/repositories && \
    echo "http://dl-cdn.alpinelinux.org/alpine/edge/community" >> /etc/apk/repositories
RUN apk add --no-cache rust cargo
ENV CARGO_HOME=/root/.cargo
COPY binary/cargo-audit /root/.cargo/bin/
COPY binary/audit-check /audit-check
ENTRYPOINT ["/audit-check"]
