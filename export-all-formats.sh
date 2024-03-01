#!/usr/bin/env bash
set -Eeuox pipefail

axel --num-connections=10 --quiet \
    --output=/mnt/out/Gesamtdatenexport.zip \
    "$(python -m mastr-export print-export-url)"
python -m mastr-export extract \
    --duckdb /mnt/out/bnetza.duckdb \
    --sqlite /mnt/out/bnetza.sqlite \
    --parquet-dir /mnt/out/parquet \
    --csv-dir /mnt/out/csv \
    /mnt/out/Gesamtdatenexport.zip
