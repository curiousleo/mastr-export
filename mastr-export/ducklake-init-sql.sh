#!/usr/bin/env bash
set -Eeuo pipefail
shopt -s extglob

get_tables() {
    local parquet_dir="$1"

    find "$parquet_dir" -type f -name "*.parquet" -exec basename {} \; | while read -r file; do
        echo "${file%%?(_*).parquet}"
    done | sort --unique
}

get_parquet_files() {
    local parquet_dir="$1"
    local table_name="$2"

    find "$parquet_dir" -type f \( -name "${table_name}_*.parquet" -o -name "${table_name}.parquet" \) | sort --version-sort
}

create_table_statement() {
    local db_name="$1"
    local table_name="$2"
    local representative_parquet_file="$3"

    echo "CREATE TABLE $db_name.$table_name AS SELECT * FROM read_parquet('$representative_parquet_file') WITH NO DATA;"
}

add_data_statement() {
    local db_name="$1"
    local table_name="$2"
    local parquet_file="$3"

    echo "CALL ducklake_add_data_files('$db_name', '$table_name', '$parquet_file');"
}

usage() {
    echo "Usage: $(basename "$0") --db-name <db_name> --parquet-dir <parquet_dir> --data-url <data_url>"
}

main() {
    local db_name parquet_dir data_url

    local args
    local valid
    args=$(getopt -n "$(basename "$0")" -o h --long help,db-name:,parquet-dir:,data-url: -- "$@")
    valid=$?

    if [ $valid -ne 0 ]; then
        usage
        exit $valid
    fi

    eval set -- "$args"
    while true; do
        case "$1" in
            --db-name)
                db_name="$2"
                shift 2
                ;;
            --parquet-dir)
                parquet_dir="$2"
                shift 2
                ;;
            --data-url)
                data_url="$2"
                shift 2
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            --)
                shift
                break
                ;;
            *)
                echo "Invalid option: $1" >&2
                exit 2
                ;;
        esac
    done

    # Normalise paths by removing trailing slashes
    parquet_dir="${parquet_dir%/}"
    data_url="${data_url%/}"

    cat <<-.
.bail on
.echo on
LOAD ducklake;
ATTACH 'ducklake:catalog.ducklake' AS $db_name (DATA_PATH 'tmp_always_empty');
.

    local tables
    tables=$(get_tables "$parquet_dir")

    for table in $tables; do
        create_table_statement "$db_name" "$table" "$(get_parquet_files "$parquet_dir" "$table" | head -n1)"
    done

    for table in $tables; do
        get_parquet_files "$parquet_dir" "$table" | while read -r parquet_file; do
            add_data_statement "$db_name" "$table" "$parquet_file"
        done
    done

    cat <<-.
DETACH $db_name;
ATTACH 'catalog.ducklake' AS catalog;
UPDATE catalog.ducklake_data_file
   SET path = replace(path, '$parquet_dir', '$data_url')
 WHERE path LIKE '$parquet_dir/%';
DETACH catalog;
.
}

main "$@"
