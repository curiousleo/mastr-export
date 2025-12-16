#!/usr/bin/env bash
set -Eeuo pipefail

URL="https://www.marktstammdatenregister.de/MaStR/Datendownload"

get_download_url() {
    local download_url
    download_url=$(curl -s "$URL" | \
        grep -oP 'https://download\.marktstammdatenregister\.de/Gesamtdatenexport_\d{8}_\d+\.\d+\.zip' | \
        head -n 1)
    echo "$download_url"
}

get_download_url
