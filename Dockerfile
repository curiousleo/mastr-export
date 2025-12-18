# Run with:
#   podman run --rm --network=none --transient-store \
#     --userns=keep-id:uid=65535,gid=65535 \
#     --volume /path/to/Gesamtdatenexport_YYYYMMDD_XX.Y.zip:/mnt/in/mastr.zip:ro,Z,U \
#     --volume ./schema/:/mnt/in/schema:ro,Z,U \
#     --volume ./out/:/mnt/out/:rw,Z,U \
#     mastr-export:latest <download|extract|initdb> [options]...

FROM ghcr.io/rust-cross/rust-musl-cross:x86_64-musl AS build-env
WORKDIR /home/rust/
COPY Cargo.lock ./
COPY Cargo.toml ./
COPY src/       ./src/
COPY schema/    ./schema/
RUN cargo build --release
RUN musl-strip ./target/x86_64-unknown-linux-musl/release/mastr-export

# This is the `latest` image on 2025-12-18, to get DuckDB.
FROM alpine@sha256:865b95f46d98cf867a156fe4a135ad3fe50d2056aa3f25ed31662dff6da4eb62
RUN echo '@testing https://dl-cdn.alpinelinux.org/alpine/edge/testing' >> /etc/apk/repositories
RUN apk add --no-cache axel bash curl duckdb@testing icu parallel unzip
COPY --from=build-env /home/rust/target/x86_64-unknown-linux-musl/release/mastr-export /usr/bin/mastr-export
COPY contrib/getoptions /usr/bin/getoptions
COPY download /usr/bin/download
COPY extract /usr/bin/extract
COPY initdb /usr/bin/initdb
VOLUME /mnt/mastr.zip
VOLUME /mnt/schema
VOLUME /mnt/parquet

ENV USER_ID=65535
ENV GROUP_ID=65535
ENV USER_NAME=u
ENV GROUP_NAME=u
RUN addgroup -g $GROUP_ID $GROUP_NAME \
    && adduser --shell /sbin/nologin --disabled-password \
    --uid $USER_ID --ingroup $GROUP_NAME $USER_NAME
USER $USER_NAME
RUN duckdb -c 'INSTALL ducklake;'
