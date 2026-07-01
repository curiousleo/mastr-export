#!/usr/bin/env -S deno run --allow-read --allow-write

// Transforms MaStR XSD schemas into the compact JSON schemas consumed by
// mastr-extract. For each <xsd-dir>/<Name>.xsd it produces <json-dir>/<Name>.json.
//
// The XSD is canonical; re-run this whenever the XSD files change to keep the
// JSON in sync. Pass schema names to transform a subset, or --check to verify
// the JSON is up to date without writing (exit 1 if not).
//
// To confine filesystem access, scope Deno's permissions to the two dirs:
//   deno run --allow-read=<xsd-dir> --allow-write=<json-dir> \
//     xsd-to-json.ts --xsd-dir <xsd-dir> --json-dir <json-dir>
// Under --check (compare mode) grant read on both and no write at all:
//   deno run --allow-read=<xsd-dir>,<json-dir> \
//     xsd-to-json.ts --xsd-dir <xsd-dir> --json-dir <json-dir> --check

import { parseArgs } from "jsr:@std/cli@1/parse-args";
import { basename, join } from "jsr:@std/path@1";
import { XMLParser } from "npm:fast-xml-parser@4";

// ---------------------------------------------------------------------------
// Schema shape
// ---------------------------------------------------------------------------

interface Field {
  name: string;
  /** XSD type without the "xs:" prefix; omitted for plain strings. */
  xsd?: string;
}

interface Schema {
  root: string;
  element: string;
  fields: Field[];
}

// ---------------------------------------------------------------------------
// XSD → Schema
// ---------------------------------------------------------------------------

// `xs:` prefixes are stripped from tag names; `element` and `enumeration`
// are forced to arrays so single-child cases behave like the many-child ones.
const parser = new XMLParser({
  ignoreAttributes: false,
  attributeNamePrefix: "@_",
  removeNSPrefix: true,
  isArray: (name) => name === "element" || name === "enumeration",
});

// deno-lint-ignore no-explicit-any
type Node = Record<string, any>;

const stripNs = (t: string): string => t.replace(/^[^:]+:/, "");

/** Map an <xs:element> field to its JSON type, or null for a plain string. */
function fieldType(el: Node): string | null {
  const type = el["@_type"];
  if (type) {
    const t = stripNs(type);
    return t === "string" ? null : t;
  }

  const restriction = el.simpleType?.restriction;
  if (restriction) {
    const base = stripNs(restriction["@_base"]);
    const values = new Set<string>(
      (restriction.enumeration ?? []).map((e: Node) => String(e["@_value"])),
    );
    // A byte enumerated over exactly {0, 1} is a boolean flag.
    if (base === "byte" && values.size === 2 && values.has("0") && values.has("1")) {
      return "boolean";
    }
    return base === "string" ? null : base;
  }

  return null;
}

function xsdToSchema(xsd: string, source: string): Schema {
  const doc = parser.parse(xsd);

  const rootEl: Node | undefined = doc.schema?.element?.[0];
  const recordEl: Node | undefined =
    rootEl?.complexType?.sequence?.element?.[0];
  const inner: Node | undefined = recordEl?.complexType;
  const container: Node | undefined = inner?.sequence ?? inner?.choice;
  const fieldEls: Node[] | undefined = container?.element;

  if (!rootEl?.["@_name"] || !recordEl?.["@_name"] || !fieldEls) {
    throw new Error(`Unexpected XSD structure in ${source}`);
  }

  const fields: Field[] = fieldEls.map((el) => {
    const name = el["@_name"];
    if (!name) throw new Error(`Field without name in ${source}`);
    const xsd = fieldType(el);
    return xsd === null ? { name } : { name, xsd };
  });

  return { root: rootEl["@_name"], element: recordEl["@_name"], fields };
}

const serialize = (schema: Schema): string =>
  JSON.stringify(schema, null, 2) + "\n";

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

async function xsdNames(xsdDir: string): Promise<string[]> {
  const names: string[] = [];
  for await (const entry of Deno.readDir(xsdDir)) {
    if (entry.isFile && entry.name.endsWith(".xsd")) {
      names.push(basename(entry.name, ".xsd"));
    }
  }
  return names.sort();
}

function usage(): never {
  console.log(
    `Usage: xsd-to-json.ts --xsd-dir DIR (--json-dir DIR | --check) [SCHEMA...]

Transforms <xsd-dir>/<Name>.xsd into <json-dir>/<Name>.json.
With no SCHEMA arguments, processes every .xsd file in --xsd-dir.

  --xsd-dir DIR    Source directory of .xsd files (read only)          [required]
  --json-dir DIR   Target directory for .json files (written, or the
                   files compared against under --check)
  --check          Verify without writing; exit 1 if any JSON is stale.
                   Without --json-dir, only checks that the XSDs transform.
  --help

One of --json-dir or --check is required.`,
  );
  Deno.exit(0);
}

async function main() {
  const args = parseArgs(Deno.args, {
    string: ["xsd-dir", "json-dir"],
    boolean: ["check", "help"],
  });

  if (args.help) usage();

  const xsdDir = args["xsd-dir"];
  const jsonDir = args["json-dir"];
  if (!xsdDir) {
    console.error("Error: --xsd-dir is required.\n");
    usage();
  }
  if (!jsonDir && !args.check) {
    console.error("Error: one of --json-dir or --check is required.\n");
    usage();
  }

  const names = args._.length > 0
    ? args._.map(String)
    : await xsdNames(xsdDir);

  let stale = 0;
  for (const name of names) {
    const xsd = await Deno.readTextFile(join(xsdDir, `${name}.xsd`));
    const output = serialize(xsdToSchema(xsd, `${name}.xsd`));

    if (args.check) {
      // Compare against the existing JSON when a directory is given;
      // otherwise the transform above is itself the validation.
      if (jsonDir) {
        const current = await Deno.readTextFile(join(jsonDir, `${name}.json`))
          .catch(() => null);
        if (current !== output) {
          console.error(`stale: ${name}.json`);
          stale++;
        }
      }
    } else {
      await Deno.writeTextFile(join(jsonDir!, `${name}.json`), output);
      console.log(`wrote ${name}.json`);
    }
  }

  if (args.check) {
    if (stale > 0) {
      console.error(`\n${stale} schema(s) out of date. Re-run without --check.`);
      Deno.exit(1);
    }
    console.log(
      jsonDir
        ? `${names.length} schema(s) up to date.`
        : `${names.length} schema(s) transform cleanly.`,
    );
  }
}

main();
