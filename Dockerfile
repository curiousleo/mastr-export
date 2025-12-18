# Run with:
#   podman run --rm --network=none --transient-store \
#     --volume /path/to/Gesamtdatenexport_YYYYMMDD_XX.Y.zip:/mnt/mastr.zip:ro,Z,U \
#     --volume ./schema/:/mnt/schema:ro,Z,U \
#     --volume ./parquet/:/mnt/parquet:rw,Z,U \
#     mastr-export:latest <download|extract|initdb> [options]...

FROM ghcr.io/rust-cross/rust-musl-cross:x86_64-musl as build-env
WORKDIR /home/rust/
COPY Cargo.lock ./
COPY Cargo.toml ./
COPY src/       ./src/
COPY schema/    ./schema/
RUN cargo build --release
RUN ls -lah ./target/x86_64-unknown-linux-musl/release/mastr-export
RUN musl-strip ./target/x86_64-unknown-linux-musl/release/mastr-export
RUN ls -lah ./target/x86_64-unknown-linux-musl/release/mastr-export

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
RUN mkdir -p /home/guest && chown -R guest /home/guest
USER guest
ENV HOME=/home/guest
RUN duckdb -c 'INSTALL ducklake;'
