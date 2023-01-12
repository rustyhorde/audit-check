FROM alpine:latest
RUN apk add --no-cache gcc musl-dev && apk add --no-cache rust cargo
COPY binary/audit-check /audit-check
ENTRYPOINT ["/audit-check"]
