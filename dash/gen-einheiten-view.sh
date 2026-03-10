#!/bin/bash
# Generate Einheiten.sql — a CREATE VIEW that unions all electricity
# generation/storage tables on their common columns.
#
# The table list is defined here; the common columns are derived from the
# schema JSON files using jq.
#
# Catalog-valued columns (xsd: short/byte) are resolved to human-readable
# labels via the KatalogwerteDict dictionary.

set -euo pipefail

SCHEMA_DIR="$(dirname "$0")/../schema"

# Quelle label -> table name (and schema file is schema/${table}.json)
tables=(
    "Biomasse:EinheitenBiomasse"
    "GeothermieGrubengasDruckentspannung:EinheitenGeothermieGrubengasDruckentspannung"
    "Kernkraft:EinheitenKernkraft"
    "Solar:EinheitenSolar"
    "StromSpeicher:EinheitenStromSpeicher"
    "Verbrennung:EinheitenVerbrennung"
    "Wasser:EinheitenWasser"
    "Wind:EinheitenWind"
)

# Collect schema files for these tables.
schema_files=()
for entry in "${tables[@]}"; do
    table="${entry#*:}"
    schema_files+=("$SCHEMA_DIR/$table.json")
done

# Find columns common to all schemas (same name appears in every file).
n=${#schema_files[@]}
mapfile -t columns < <(
    jq -r '.fields[].name' "${schema_files[@]}" \
        | sort | uniq -c | awk -v n="$n" '$1 == n { print $2 }'
)

# Identify which common columns are catalog-valued (xsd: short or byte).
# We check against the first schema file — types are consistent across tables.
first_schema="${schema_files[0]}"
declare -A catalog_columns
while IFS=$'\t' read -r name type; do
    catalog_columns["$name"]=1
done < <(
    jq -r --argjson cols "$(printf '%s\n' "${columns[@]}" | jq -R . | jq -s .)" \
        '.fields[] | select(.name as $n | $cols | index($n)) | select(.xsd == "short" or .xsd == "byte") | .name + "\t" + .xsd' \
        "$first_schema"
)

echo "-- Common columns (${#columns[@]}) across $n tables, ${#catalog_columns[@]} catalog-resolved" >&2

# Build the column list for a SELECT. Catalog columns get dictGet resolution.
build_col_list() {
    local last_idx=$(( ${#columns[@]} - 1 ))
    local result=""
    for i in "${!columns[@]}"; do
        local col="${columns[$i]}"
        local comma=","
        if (( i == last_idx )); then comma=""; fi

        if [[ -n "${catalog_columns[$col]:-}" ]]; then
            result+="    dictGet('KatalogwerteDict', 'Wert', toUInt64($col)) AS $col$comma"$'\n'
        else
            result+="    $col$comma"$'\n'
        fi
    done
    # Trim trailing newline
    result="${result%$'\n'}"
    echo "$result"
}

col_list=$(build_col_list)

cat <<'SQL'
CREATE OR REPLACE DICTIONARY KatalogwerteDict
(
    Id UInt64,
    Wert String
)
PRIMARY KEY Id
SOURCE(CLICKHOUSE(TABLE 'Katalogwerte'))
LAYOUT(FLAT())
LIFETIME(0);

SQL

echo "CREATE OR REPLACE VIEW Einheiten AS"

first=true
for entry in "${tables[@]}"; do
    quelle="${entry%%:*}"
    table="${entry#*:}"
    if $first; then
        first=false
    else
        echo "UNION ALL"
    fi
    printf "SELECT '%s' AS Quelle,\n%s\nFROM %s\n" "$quelle" "$col_list" "$table"
done
echo ";"
