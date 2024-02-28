FROM docker.io/library/python:3.11-slim-bookworm

RUN set -eux; \
	apt-get update; \
	apt-get install -y --no-install-recommends \
		axel \
        sqlite3 \
        unzip \
	; \
	rm -rf /var/lib/apt/lists/*

WORKDIR /tmp

ARG TARGETARCH
RUN set -eux; \
    if [ "$TARGETARCH" = "arm64" ]; then \
    export TARGETARCH=aarch64; \
    fi; \
    axel --quiet --output=duckdb.zip https://github.com/duckdb/duckdb/releases/download/v0.10.0/duckdb_cli-linux-${TARGETARCH}.zip ; \
    unzip duckdb.zip; \
    mkdir -p /opt/duckdb/bin; \
    mv duckdb /opt/duckdb/bin/; \
    rm duckdb.zip

ENV PATH="${PATH}:/opt/duckdb/bin"

WORKDIR /app
COPY . .

RUN set -eux; \
    pip install --no-cache-dir .; \
    ;

CMD ["bash", "export-all-formats.sh"]