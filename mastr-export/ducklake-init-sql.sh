#!/usr/bin/env bash
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

main() {
    local db_name="$1"
    local parquet_dir="$2"

    cat <<-.
.bail on
.echo on
LOAD ducklake;
ATTACH 'ducklake:$db_name.ducklake' AS $db_name (DATA_PATH 'tmp_always_empty');
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
}

main "$@"
