#!/usr/bin/env bash
set -Eeuo pipefail
shopt -s extglob

set -x

list_xml_files() {
    local zip_file="$1"

    unzip -l "$zip_file" |
    sed --silent 's@^.* \([A-Za-z0-9_]\+\.xml\)$@\1@p' |
    sort --version-sort
}

extract_as_utf8() {
    local zip_file="$1"
    local xml_file="$2"

    unzip -p "$zip_file" "$xml_file" |
    iconv -f UTF-16 -t UTF-8 |
    sed "s@encoding='UTF-16'@encoding='UTF-8'@"
}

table_name_of_file_name() {
    local file="$1"

    echo "${file%%?(_*).*}"
}

main() {
    local zip_file="$1"
    local schema_dir="$2"
    local parquet_dir="$3"

    list_xml_files "$zip_file" | while read -r xml_file; do
        local table_name=$(table_name_of_file_name "$xml_file")
        extract_as_utf8 "$zip_file" "$xml_file" |
        ./target/release/mastr-export.exe --schema "$schema_dir/$table_name.json" --output "$parquet_dir/${xml_file%%.xml}.parquet"
    done
}

main "$@"
