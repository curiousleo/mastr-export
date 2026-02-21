#!/usr/bin/env -S deno run --allow-net=www.marktstammdatenregister.de,download.marktstammdatenregister.de --allow-read --allow-write --allow-run --allow-env

import { parseArgs } from "jsr:@std/cli@1/parse-args";
import { join, basename } from "jsr:@std/path@1";

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

const args = parseArgs(Deno.args, {
  string: ["state-dir", "scratch-dir", "output-dir"],
  boolean: ["dry-run", "help"],
  default: { "dry-run": false },
});

if (
  args.help ||
  !args["state-dir"] ||
  !args["scratch-dir"] ||
  !args["output-dir"]
) {
  console.log(
    `Usage: mastr-export.ts --state-dir DIR --scratch-dir DIR --output-dir DIR [--dry-run]`,
  );
  Deno.exit(args.help ? 0 : 1);
}

const STATE_DIR = args["state-dir"];
const SCRATCH_DIR = args["scratch-dir"];
const OUTPUT_DIR = args["output-dir"];
const DRY_RUN = args["dry-run"];
const SCHEMA_DIR = join(import.meta.dirname!, "schema");

// ---------------------------------------------------------------------------
// Runner – thin abstraction to keep dry-run out of pipeline logic
// ---------------------------------------------------------------------------

interface ExecResult {
  stdout: string;
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
      return { stdout: "", success: true };
    }
    const p = new Deno.Command(cmd[0], {
      args: cmd.slice(1),
      cwd: opts?.cwd,
      stdin: opts?.stdin !== undefined ? "piped" : "null",
      stdout: "piped",
      stderr: "inherit",
    });
    const child = p.spawn();
    if (opts?.stdin !== undefined) {
      const w = child.stdin.getWriter();
      await w.write(new TextEncoder().encode(opts.stdin));
      await w.close();
    }
    const out = await child.output();
    return {
      stdout: new TextDecoder().decode(out.stdout),
      success: out.success,
    };
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
      stderr: "inherit",
    });
    const out = await p.output();
    return {
      stdout: new TextDecoder().decode(out.stdout),
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

  async moveDir(src: string, dst: string): Promise<void> {
    if (this.dryRun) {
      console.log(`[dry-run] mv ${src} ${dst}`);
      return;
    }
    // Remove destination if it exists, then rename.
    await Deno.remove(dst, { recursive: true }).catch(() => {});
    await Deno.rename(src, dst);
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

function tableNameOf(xmlFile: string): string {
  return xmlFile.replace(/(_\d+)?\.xml$/, "");
}

async function extractAll(
  runner: Runner,
  zipFile: string,
  parquetDir: string,
): Promise<void> {
  await runner.mkdir(parquetDir);
  const xmlFiles = await listXmlFiles(runner, zipFile);
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
          ` | mastr-export --schema '${schema}' --output '${parquet}'`;
        running++;
        runner
          .shell(script)
          .then((r) => {
            if (!r.success && !runner["dryRun"]) {
              reject(new Error(`Failed to process ${xmlFile}`));
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
// Step 4: Init Ducklake DB
// ---------------------------------------------------------------------------

function getParquetFiles(
  parquetDir: string,
  table: string,
  allFiles: string[],
): string[] {
  return allFiles
    .filter((f) => f === `${table}.parquet` || f.startsWith(`${table}_`))
    .map((f) => join(parquetDir, f))
    .sort();
}

async function buildInitSql(
  dbName: string,
  parquetDir: string,
  dataDir: string,
  dbDir: string,
): Promise<string> {
  const dbFile = join(dbDir, "catalog.ducklake");
  const dataPath = join(dbDir, "tmp_always_empty");

  const allFiles: string[] = [];
  try {
    for await (const entry of Deno.readDir(parquetDir)) {
      if (entry.isFile && entry.name.endsWith(".parquet")) {
        allFiles.push(entry.name);
      }
    }
  } catch (e) {
    if (!DRY_RUN) throw e;
  }
  allFiles.sort();

  const tables = [...new Set(allFiles.map((f) => tableNameOf(f)))].sort();
  const lines: string[] = [];

  lines.push(".bail on");
  lines.push(".echo on");
  lines.push(".mode list");
  lines.push("LOAD ducklake;");
  lines.push(
    `ATTACH 'ducklake:${dbFile}' AS ${dbName} (DATA_PATH '${dataPath}');`,
  );

  for (const table of tables) {
    const files = getParquetFiles(dataDir, table, allFiles);
    lines.push(
      `CREATE TABLE ${dbName}.${table} AS SELECT * FROM read_parquet('${files[0]}') WITH NO DATA;`,
    );
  }

  for (const table of tables) {
    const files = getParquetFiles(dataDir, table, allFiles);
    for (const f of files) {
      lines.push(
        `CALL ducklake_add_data_files('${dbName}', '${table}', '${f}');`,
      );
    }
  }

  lines.push(`DETACH ${dbName};`);
  return lines.join("\n") + "\n";
}

async function initDb(
  runner: Runner,
  parquetDir: string,
  outputDir: string,
): Promise<void> {
  const dbDir = join(SCRATCH_DIR, "db");
  await runner.mkdir(dbDir);

  const dataDir = join(outputDir, "data");
  const sql = await buildInitSql("mastr", parquetDir, dataDir, dbDir);

  if (DRY_RUN) {
    console.log("[dry-run] duckdb <<SQL");
    console.log(sql);
    console.log("SQL");
    return;
  }

  await runner.exec(["duckdb"], { stdin: sql });
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main() {
  const runner = new Runner(DRY_RUN);

  // 1. Discover URL
  const url = await getDownloadUrl();
  const zipName = basename(url);
  console.log(`Latest export: ${zipName} (${url})`);

  // 2. Check state
  const state = await readState(STATE_DIR);
  if (state && zipName <= state.lastProcessedZip) {
    console.log(`Already processed ${state.lastProcessedZip}, nothing to do.`);
    return;
  }

  // 3. Download
  const downloadDir = join(SCRATCH_DIR, "download");
  const zipFile = join(downloadDir, zipName);
  await runner.mkdir(downloadDir);
  await runner.exec([
    "axel",
    "--num-connections=5",
    `--output=${zipFile}`,
    url,
  ]);
  console.log(`Downloaded ${zipName}`);

  // 4. Extract
  const parquetDir = join(SCRATCH_DIR, "parquet");
  console.log("Extracting XML files to Parquet...");
  await extractAll(runner, zipFile, parquetDir);
  console.log("Extraction complete.");

  // 5. Init DB
  console.log("Building Ducklake database...");
  await initDb(runner, parquetDir, OUTPUT_DIR);
  console.log("Database ready.");

  // 6. Assemble output
  const outputDataDir = join(OUTPUT_DIR, "data");
  await runner.moveDir(parquetDir, outputDataDir);
  await runner.moveDir(
    join(SCRATCH_DIR, "db", "catalog.ducklake"),
    join(OUTPUT_DIR, "catalog.ducklake"),
  );
  console.log("Output assembled.");

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
