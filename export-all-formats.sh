#!/usr/bin/env bash
set -Eeuox pipefail

axel --num-connections=10 --quiet \
    --output=/mnt/out/Gesamtdatenexport.zip \
    "$(python -m mastr-export print-export-url)"
python -m mastr-export extract \
    /mnt/out/Gesamtdatenexport.zip \
    --parquet-dir /mnt/out/parquet \
    --csv-dir /mnt/out/csv

pushd /mnt/out/parquet
duckdb '/mnt/out/bnetza.duckdb' -init 'import-duckdb.sql' -bail -batch -echo -no-stdin
popd

pushd /mnt/out/csv
sqlite3 -bail '/mnt/out/bnetza.sqlite3' <'import-sqlite.sql'
popd