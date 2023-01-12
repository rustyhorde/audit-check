FROM alpine:latest
COPY binary/audit-check /audit-check
ENTRYPOINT ["/audit-check"]
