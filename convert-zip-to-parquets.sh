#!/usr/bin/env bash
set -Eeuo pipefail

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
    uconv -f UTF-16LE -t UTF-8 # | sed "s@encoding='UTF-16'@encoding='UTF-8'@"
}

table_name_of_file_name() {
    local file="$1"

    shopt -s extglob
    echo "${file%%?(_*).*}"
}

process_xml_file() {
    local zip_file="$1"
    local schema_dir="$2"
    local parquet_dir="$3"
    local xml_file="$4"

    local table_name
    table_name=$(table_name_of_file_name "$xml_file")
    extract_as_utf8 "$zip_file" "$xml_file" |
    ./target/release/mastr-export --schema "$schema_dir/$table_name.json" --output "$parquet_dir/${xml_file%%.xml}.parquet"
}

# Required so `parallel` can run these functions.
export -f process_xml_file
export -f extract_as_utf8
export -f table_name_of_file_name

main() {
    local zip_file="$1"
    local schema_dir="$2"
    local parquet_dir="$3"

    mkdir -p "$parquet_dir"
    list_xml_files "$zip_file" | parallel --jobs 50% --eta --progress process_xml_file "$zip_file" "$schema_dir" "$parquet_dir" {}
}

main "$@"
