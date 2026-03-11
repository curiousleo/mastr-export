#!/usr/bin/env -S deno run --allow-read
// Generate Einheiten.sql — a CREATE VIEW that unions all electricity
// generation/storage tables on their common columns.
//
// The table list is defined here; the common columns are derived from the
// schema JSON files.
//
// Catalog-valued columns (xsd: short/byte) are resolved to human-readable
// labels via the KatalogwerteDict dictionary.

import { join, dirname, fromFileUrl } from "https://deno.land/std/path/mod.ts";

interface Field {
  name: string;
  xsd?: string;
}

interface Schema {
  root: string;
  element: string;
  fields: Field[];
}

const SCRIPT_DIR = dirname(fromFileUrl(import.meta.url));
const SCHEMA_DIR = join(SCRIPT_DIR, "..", "schema");

// Quelle label -> table name (schema file is schema/${table}.json)
const tables: [string, string][] = [
  ["Biomasse", "EinheitenBiomasse"],
  [
    "GeothermieGrubengasDruckentspannung",
    "EinheitenGeothermieGrubengasDruckentspannung",
  ],
  ["Kernkraft", "EinheitenKernkraft"],
  ["Solar", "EinheitenSolar"],
  ["Verbrennung", "EinheitenVerbrennung"],
  ["Wasser", "EinheitenWasser"],
  ["Wind", "EinheitenWind"],
];

// Load all schemas.
const schemas: Schema[] = await Promise.all(
  tables.map(async ([, table]) => {
    const text = await Deno.readTextFile(join(SCHEMA_DIR, `${table}.json`));
    return JSON.parse(text) as Schema;
  }),
);

// Find columns common to all schemas (name appears in every file).
const fieldCounts = new Map<string, number>();
for (const schema of schemas) {
  for (const field of schema.fields) {
    fieldCounts.set(field.name, (fieldCounts.get(field.name) ?? 0) + 1);
  }
}

const n = schemas.length;
const columns = [...fieldCounts.entries()]
  .filter(([, count]) => count === n)
  .map(([name]) => name)
  .sort();

// Identify catalog-valued columns (xsd: short or byte) from the first schema.
const catalogColumns = new Set<string>();
const firstSchema = schemas[0];
for (const field of firstSchema.fields) {
  if (
    columns.includes(field.name) &&
    (field.xsd === "short" || field.xsd === "byte")
  ) {
    catalogColumns.add(field.name);
  }
}

console.error(
  `-- Common columns (${columns.length}) across ${n} tables, ${catalogColumns.size} catalog-resolved`,
);

// Build the column list for a SELECT.
function buildColList(): string {
  return columns
    .map((col, i) => {
      const comma = i < columns.length - 1 ? "," : "";
      if (catalogColumns.has(col)) {
        return `    IF(${col} IS NULL, NULL, dictGet('KatalogwerteDict', 'Wert', toUInt64(${col}))) AS ${col}${comma}`;
      }
      return `    ${col}${comma}`;
    })
    .join("\n");
}

const colList = buildColList();

// Emit SQL.
const lines: string[] = [];

// TODO(leo): Use LAYOUT(FLAT()), this requires importing Katalogwerte with non-nullable Id.
lines.push(`CREATE OR REPLACE DICTIONARY KatalogwerteDict
(
    Id Nullable(UInt64),
    Wert Nullable(String)
)
PRIMARY KEY Id
SOURCE(CLICKHOUSE(TABLE 'Katalogwerte' USER 'mastr' PASSWORD 'mastr'))
LAYOUT(HASHED())
LIFETIME(0);
`);

lines.push("CREATE OR REPLACE VIEW Einheiten AS");

for (let i = 0; i < tables.length; i++) {
  const [quelle, table] = tables[i];
  if (i > 0) lines.push("UNION ALL");
  lines.push(`SELECT '${quelle}' AS Quelle,\n${colList}\nFROM ${table}`);
}

lines.push(";");

console.log(lines.join("\n"));
