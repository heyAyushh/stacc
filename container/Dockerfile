FROM docker.io/library/node:22.22.0-alpine3.22 AS node
FROM docker.io/koalaman/shellcheck-alpine:v0.11.0 AS shellcheck

FROM docker.io/library/rust:1.92.0-alpine3.22

ENV NEXT_TELEMETRY_DISABLED=1

COPY --from=node /usr/local/ /usr/local/
COPY --from=shellcheck /bin/shellcheck /usr/local/bin/shellcheck

RUN rustup component add clippy rustfmt \
    && apk add --no-cache \
        bash \
        git

WORKDIR /workspace

COPY entrypoint.sh /usr/local/bin/stacc-container-entrypoint
RUN chmod +x /usr/local/bin/stacc-container-entrypoint

ENTRYPOINT ["stacc-container-entrypoint"]
CMD ["cargo", "run", "--", "check"]
