#!/usr/bin/env -S deno run --allow-net --allow-read --allow-write --allow-run --allow-env

import { parseArgs } from "jsr:@std/cli@1/parse-args";
import { join, basename } from "jsr:@std/path@1";

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

const args = parseArgs(Deno.args, {
  string: [
    "state-dir",
    "scratch-dir",
    "output-dir",
    "clickhouse-path",
    "clickhouse-url",
    "clickhouse-db",
    "clickhouse-user",
    "clickhouse-password",
  ],
  boolean: ["dry-run", "help"],
  default: { "dry-run": false, "clickhouse-db": "mastr" },
});

if (
  args.help ||
  !args["state-dir"] ||
  !args["scratch-dir"] ||
  !args["output-dir"] ||
  !args["clickhouse-url"]
) {
  console.log(
    `Usage: mastr-export.ts
  --state-dir DIR             State directory for tracking processed exports
  --scratch-dir DIR           Temporary working directory
  --output-dir DIR            Output directory for Parquet files (mounted into ClickHouse)
  --clickhouse-url URL        ClickHouse HTTP endpoint (e.g. http://localhost:8123)
  [--clickhouse-path PATH]    Path to output-dir as seen by ClickHouse (default: output-dir)
  [--clickhouse-db NAME]      Database name (default: mastr)
  [--clickhouse-user USER]    ClickHouse username
  [--clickhouse-password PWD] ClickHouse password
  [--dry-run]`,
  );
  Deno.exit(args.help ? 0 : 1);
}

const STATE_DIR = args["state-dir"];
const SCRATCH_DIR = args["scratch-dir"];
const OUTPUT_DIR = args["output-dir"]!;
const CLICKHOUSE_URL = args["clickhouse-url"];
const CLICKHOUSE_PATH = (args["clickhouse-path"] ?? OUTPUT_DIR).replace(
  /\/$/,
  "",
);
const CLICKHOUSE_DB = args["clickhouse-db"]!;
const CLICKHOUSE_USER = args["clickhouse-user"];
const CLICKHOUSE_PASSWORD =
  args["clickhouse-password"] ?? Deno.env.get("CLICKHOUSE_PASSWORD");
const DRY_RUN = args["dry-run"];

function clickhouseHeaders(): Record<string, string> {
  const headers: Record<string, string> = {};
  if (CLICKHOUSE_USER) headers["X-ClickHouse-User"] = CLICKHOUSE_USER;
  if (CLICKHOUSE_PASSWORD) headers["X-ClickHouse-Key"] = CLICKHOUSE_PASSWORD;
  return headers;
}

const SCHEMA_DIR = join(import.meta.dirname!, "schema");

// ---------------------------------------------------------------------------
// Runner – thin abstraction to keep dry-run out of pipeline logic
// ---------------------------------------------------------------------------

interface ExecResult {
  stdout: string;
  stderr: string;
  success: boolean;
}

class Runner {
  constructor(private dryRun: boolean) {}

  /** Run a command. In dry-run mode, prints and returns a stub. */
  async exec(
    cmd: string[],
    opts?: { stdin?: string; cwd?: string },
  ): Promise<ExecResult> {
    if (this.dryRun) {
      console.log(`[dry-run] ${cmd.join(" ")}`);
      return { stdout: "", stderr: "", success: true };
    }
    const p = new Deno.Command(cmd[0], {
      args: cmd.slice(1),
      cwd: opts?.cwd,
      stdin: opts?.stdin !== undefined ? "piped" : "null",
      stdout: "piped",
      stderr: "piped",
    });
    const child = p.spawn();
    if (opts?.stdin !== undefined) {
      const w = child.stdin.getWriter();
      await w.write(new TextEncoder().encode(opts.stdin));
      await w.close();
    }
    const out = await child.output();
    const stdout = new TextDecoder().decode(out.stdout);
    const stderr = new TextDecoder().decode(out.stderr);
    if (!out.success) {
      const detail = [stdout, stderr].filter(Boolean).join("\n");
      throw new Error(
        `Command failed: ${cmd.join(" ")}${detail ? `\n${detail}` : ""}`,
      );
    }
    return { stdout, stderr, success: true };
  }

  /** Run a shell pipeline via bash -c. */
  async shell(script: string): Promise<ExecResult> {
    return this.exec(["bash", "-c", script]);
  }

  /** Always executes, even in dry-run – for read-only queries. */
  async query(cmd: string[]): Promise<ExecResult> {
    const p = new Deno.Command(cmd[0], {
      args: cmd.slice(1),
      stdout: "piped",
      stderr: "piped",
    });
    const out = await p.output();
    return {
      stdout: new TextDecoder().decode(out.stdout),
      stderr: new TextDecoder().decode(out.stderr),
      success: out.success,
    };
  }

  async mkdir(dir: string): Promise<void> {
    if (this.dryRun) {
      console.log(`[dry-run] mkdir -p ${dir}`);
      return;
    }
    await Deno.mkdir(dir, { recursive: true });
  }

  async writeText(path: string, content: string): Promise<void> {
    if (this.dryRun) {
      console.log(`[dry-run] write ${path}:`);
      console.log(content);
      return;
    }
    await Deno.writeTextFile(path, content);
  }

  async removeDir(dir: string): Promise<void> {
    if (this.dryRun) {
      console.log(`[dry-run] rm -rf ${dir}`);
      return;
    }
    await Deno.remove(dir, { recursive: true }).catch(() => {});
  }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

interface State {
  lastProcessedZip: string;
}

async function readState(stateDir: string): Promise<State | null> {
  try {
    const text = await Deno.readTextFile(join(stateDir, "state.json"));
    return JSON.parse(text);
  } catch {
    return null;
  }
}

// ---------------------------------------------------------------------------
// Step 1: Discover download URL
// ---------------------------------------------------------------------------

const MASTR_URL = "https://www.marktstammdatenregister.de/MaStR/Datendownload";
const ZIP_RE =
  /https:\/\/download\.marktstammdatenregister\.de\/Gesamtdatenexport_[0-9]{8}_[0-9]+\.[0-9]+\.zip/;

async function getDownloadUrl(): Promise<string> {
  const resp = await fetch(MASTR_URL);
  const html = await resp.text();
  const m = html.match(ZIP_RE);
  if (!m) throw new Error("Could not find download URL on MaStR page");
  return m[0];
}

// ---------------------------------------------------------------------------
// Step 3: Extract XML → Parquet
// ---------------------------------------------------------------------------

const XML_FILE_RE = /^.*\s([A-Za-z0-9_]+\.xml)$/;

async function listXmlFiles(
  runner: Runner,
  zipFile: string,
): Promise<string[]> {
  const { stdout, success } = await runner.query(["unzip", "-l", zipFile]);
  if (!success) return [];
  const files: string[] = [];
  for (const line of stdout.split("\n")) {
    const m = line.match(XML_FILE_RE);
    if (m) files.push(m[1]);
  }
  files.sort();
  return files;
}

function tableNameOf(file: string): string {
  return file.replace(/(_\d+)?\.(xml|parquet)$/, "");
}

async function extractAll(
  runner: Runner,
  zipFile: string,
  parquetDir: string,
): Promise<void> {
  await runner.mkdir(parquetDir);
  const xmlFiles = await listXmlFiles(runner, zipFile);
  if (xmlFiles.length === 0 && !DRY_RUN) {
    throw new Error(`No XML files found in ${zipFile}`);
  }
  const concurrency = Math.max(
    1,
    Math.floor(navigator.hardwareConcurrency / 2),
  );
  let running = 0;
  let idx = 0;

  await new Promise<void>((resolve, reject) => {
    function next() {
      while (running < concurrency && idx < xmlFiles.length) {
        const xmlFile = xmlFiles[idx++];
        const table = tableNameOf(xmlFile);
        const schema = join(SCHEMA_DIR, `${table}.json`);
        const parquet = join(parquetDir, xmlFile.replace(/\.xml$/, ".parquet"));
        const script =
          `unzip -p '${zipFile}' '${xmlFile}'` +
          ` | uconv -f UTF-16LE -t UTF-8` +
          ` | mastr-extract --schema '${schema}' --output '${parquet}'`;
        running++;
        runner
          .shell(script)
          .then((r) => {
            if (!r.success && !runner["dryRun"]) {
              const detail = [r.stdout, r.stderr].filter(Boolean).join("\n");
              reject(
                new Error(
                  `Failed to process ${xmlFile}${detail ? `: ${detail}` : ""}`,
                ),
              );
              return;
            }
            running--;
            next();
          })
          .catch(reject);
      }
      if (running === 0 && idx >= xmlFiles.length) resolve();
    }
    next();
  });
}

// ---------------------------------------------------------------------------
// Step 5: ClickHouse
// ---------------------------------------------------------------------------

async function clickhouseQuery(
  url: string,
  query: string,
  dryRun: boolean,
): Promise<void> {
  console.log(`    ${query}`);
  if (dryRun) {
    return;
  }
  const resp = await fetch(url, {
    method: "POST",
    body: query,
    headers: clickhouseHeaders(),
  });
  if (!resp.ok) {
    const body = await resp.text();
    throw new Error(`ClickHouse error: ${body.trim()}`);
  }
  await resp.body?.cancel();
}

async function listParquetFiles(parquetDir: string): Promise<string[]> {
  const files: string[] = [];
  try {
    for await (const entry of Deno.readDir(parquetDir)) {
      if (entry.isFile && entry.name.endsWith(".parquet")) {
        files.push(entry.name);
      }
    }
  } catch (e) {
    if (!DRY_RUN) throw e;
  }
  return files.sort();
}

async function initDictsAndViewsSql(schema_dir: string): Promise<string> {
  interface Field {
    name: string;
    xsd?: string;
  }

  interface Schema {
    root: string;
    element: string;
    fields: Field[];
  }

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
      const text = await Deno.readTextFile(join(schema_dir, `${table}.json`));
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

  // Emit SQL.
  const lines: string[] = [];

  lines.push(
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
  return lines.join("\n");
}

async function initClickHouse(
  parquetDir: string,
  chPath: string,
  zipName: string,
  chUrl: string,
  db: string,
): Promise<void> {
  const staging = `${db}_staging`;
  const old = `${db}_old`;

  const ch = (query: string) => clickhouseQuery(chUrl, query, DRY_RUN);

  // Clean up any previous failed staging
  await ch(`DROP DATABASE IF EXISTS ${staging}`);
  await ch(`CREATE DATABASE ${staging}`);

  // Derive table names from parquet files
  const allFiles = await listParquetFiles(parquetDir);
  const tables = [...new Set(allFiles.map((f) => tableNameOf(f)))].sort();

  // Create each table from local files
  for (const table of tables) {
    const filePattern = `${chPath}/${zipName}/${table}*.parquet`;
    console.log(`  Creating ${staging}.${table}...`);
    await ch(
      `CREATE TABLE ${staging}.${table}` +
        ` ENGINE = MergeTree ORDER BY tuple()` +
        ` AS SELECT * FROM file('${filePattern}', Parquet)` +
        ` SETTINGS date_time_overflow_behavior = 'saturate'`,
    );
  }

  // Atomic swap
  await ch(`DROP DATABASE IF EXISTS ${old}`);

  // Check if main db exists (first run won't have it)
  const checkResp = await (DRY_RUN
    ? Promise.resolve("1")
    : fetch(chUrl, {
        method: "POST",
        body: `SELECT count() FROM system.databases WHERE name = '${db}'`,
        headers: clickhouseHeaders(),
      }).then((r) => r.text()));

  const initDictsAndViews = await initDictsAndViewsSql(SCHEMA_DIR);

  if (checkResp.trim() === "1") {
    // ClickHouse doesn't seem to support renaming databases atomically, so we
    // drop all dictionaries and views before renaming.
    await ch(`DROP VIEW ${db}.Einheiten`);
    await ch(`DROP DICTIONARY ${db}.KatalogwerteDict`);
    await ch(`RENAME DATABASE ${db} TO ${old}`);
  }
  await ch(`RENAME DATABASE ${staging} TO ${db}`);
  await ch(initDictsAndViews);
  await ch(`DROP DATABASE IF EXISTS ${old}`);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main() {
  const runner = new Runner(DRY_RUN);

  // 1. Discover URL
  const url = await getDownloadUrl();
  const zipName = basename(url, ".zip");
  console.log(`Latest export: ${zipName} (${url})`);

  // 2. Check state
  const state = await readState(STATE_DIR);
  if (state && zipName <= state.lastProcessedZip) {
    console.log(`Already processed ${state.lastProcessedZip}, nothing to do.`);
    return;
  }

  // 3. Download
  const downloadDir = join(SCRATCH_DIR, "download");
  const zipFile = join(downloadDir, `${zipName}.zip`);
  await runner.mkdir(downloadDir);
  await runner.exec([
    "axel",
    "--num-connections=5",
    `--output=${zipFile}`,
    url,
  ]);
  console.log(`Downloaded ${zipName}`);

  // 4. Extract XML → Parquet
  const parquetDir = join(OUTPUT_DIR, zipName);
  console.log(`Extracting XML files to Parquet in ${parquetDir}...`);
  await extractAll(runner, zipFile, parquetDir);
  console.log("Extraction complete.");

  // 5. Create ClickHouse tables + atomic swap
  console.log("Creating ClickHouse tables...");
  await initClickHouse(
    parquetDir,
    CLICKHOUSE_PATH,
    zipName,
    CLICKHOUSE_URL,
    CLICKHOUSE_DB,
  );
  console.log("ClickHouse database ready.");

  // 7. Clean up scratch
  await runner.removeDir(SCRATCH_DIR);

  // 8. Update state
  await runner.mkdir(STATE_DIR);
  const newState: State = { lastProcessedZip: zipName };
  await runner.writeText(
    join(STATE_DIR, "state.json"),
    JSON.stringify(newState, null, 2) + "\n",
  );
  console.log(`State updated: lastProcessedZip = ${zipName}`);
}

main();
