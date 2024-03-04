#!/usr/bin/env bash
set -Eeuox pipefail

axel --num-connections=10 --quiet \
    --output=/mnt/out/Gesamtdatenexport.zip \
    "$(python -m mastr-export print-export-url)"
python -m mastr-export extract-to-sqlite \
    --export /mnt/out/Gesamtdatenexport.zip \
    --sqlite /mnt/out/bnetza.duckdb
