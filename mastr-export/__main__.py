from .download import get_latest_url
from .spec import Specs
from .parser import Parser

import argparse
import os.path
import polars as pl
from tqdm.auto import tqdm
from tqdm.utils import CallbackIOWrapper
import zipfile


def cli():
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(required=True)

    get_export_url = subparsers.add_parser("get-export-url")
    get_export_url.set_defaults(func=lambda _args: print(get_latest_url()))

    extract = subparsers.add_parser("extract")
    extract.add_argument(
        "--spec",
        required=True,
        help="Path to the YAML file containing the list of specs",
    )
    extract.add_argument(
        "--export",
        required=True,
        help="Path to the Marktstammdatenregister export ZIP file",
    )
    extract.add_argument(
        "--parquet-dir",
        help="Where to write Parquet files (and DuckDB SQL file to load them)",
    )
    extract.add_argument(
        "--csv-dir", help="Where to write CSV files (and SQLite file to load them)"
    )
    extract.add_argument(
        "--show-per-file-progress",
        default=False,
        action="store_true",
        help="Show a progress bar for each individual file?",
    )
    extract.set_defaults(
        func=lambda args: run(
            args.spec,
            args.export,
            args.parquet_dir,
            args.csv_dir,
            args.show_per_file_progress,
        )
    )

    args = parser.parse_args()
    args.func(args)


def run(spec, export, parquet_dir, csv_dir, show_per_file_progress):
    if parquet_dir is None and csv_dir is None:
        raise Exception("You must pass at least one of --parquet-dir or --csv-dir")
    for dir in [dir for dir in [parquet_dir, csv_dir] if dir is not None]:
        os.makedirs(dir, exist_ok=True)

    specs = Specs.load(spec)
    with zipfile.ZipFile(export) as z:
        # Sanity check: do we know how to handle all the files in the export?
        spec_to_xml_files = {}
        for i in z.infolist():
            d = specs.for_file(i.filename)
            if not i.filename.endswith(".xml"):
                raise Exception(f"Expected only XML files, got {i.filename}")
            spec_to_xml_files[d.element] = spec_to_xml_files.get(d.element, []) + [i]

        # Assemble the list of files in the order of their specs.
        xml_files = []
        for d in specs:
            for i in spec_to_xml_files[d.element]:
                xml_files.append((i, d))

        # Convert XML to DataFrames
        xml_files_progress = tqdm(xml_files, desc="Files")
        for i, d in xml_files_progress:
            with tqdm(
                total=i.file_size,
                desc=i.filename,
                unit="B",
                unit_scale=True,
                unit_divisor=1024,
                leave=False,
                disable=(not show_per_file_progress),
            ) as xml_progress:
                if not show_per_file_progress:
                    xml_files_progress.set_description(i.filename)
                with z.open(i) as f:
                    f = CallbackIOWrapper(xml_progress.update, f)
                    data = Parser(d).parse(f, i.filename)

            df = pl.DataFrame(
                data=data,
                schema=dict(
                    (name, field.polars_type) for name, field in d.fields.items()
                ),
            )
            if parquet_dir is not None:
                df.write_parquet(
                    os.path.join(parquet_dir, i.filename.replace(".xml", ".parquet"))
                )
            if csv_dir is not None:
                df.write_csv(os.path.join(csv_dir, i.filename.replace(".xml", ".csv")))

    # Output SQL to import Parquet and CSV
    sqlite_load_name = os.path.join(csv_dir, "sqlite.sql")
    sqlite_load = open(sqlite_load_name, "w") if csv_dir is not None else None
    duckdb_load_name = os.path.join(parquet_dir, "duckdb.sql")
    duckdb_load = open(duckdb_load_name, "w") if parquet_dir is not None else None

    if sqlite_load is not None:
        sqlite_load.write(
            """-- Make SQLite go brrrr
pragma journal_mode=off;
pragma synchronous=off;
pragma page_size=16384;

"""
        )

    for d in specs:
        if duckdb_load is not None:
            parquet_files = [
                i.filename.replace(".xml", ".parquet")
                for i in spec_to_xml_files[d.element]
            ]
            duckdb_load.write(d.duckdb_schema())
            duckdb_load.writelines(
                f"""insert into {d.element} select * from read_parquet('{f}');\n"""
                for f in parquet_files
            )

        if sqlite_load is not None:
            csv_files = [
                i.filename.replace(".xml", ".csv") for i in spec_to_xml_files[d.element]
            ]
            sqlite_load.write(d.sqlite_schema())
            sqlite_load.writelines(
                f""".import "{f}" "{d.element}" --csv --skip 1\n""" for f in csv_files
            )
            sqlite_load.write("\n\n")

    if duckdb_load is not None:
        duckdb_load.close()

    if sqlite_load is not None:
        sqlite_load.write("vacuum;\n")
        sqlite_load.close()

    if duckdb_load is not None:
        print(
            f"""Parquet export finished. You can import the Parquet files into a DuckDB file
called 'bnetza.duckdb' with the following command:

$ cd '{csv_dir}'; duckdb 'bnetza.duckdb' -init '{duckdb_load_name}' -bail -batch -echo -no-stdin
"""
        )

    if sqlite_load is not None:
        print(
            f"""CSV export finished. You can import the CSV files into a SQLite file
called 'bnetza.sqlite3' with the following command:

$ cd '{parquet_dir}'; sqlite3 -init '{duckdb_load_name}' -echo 'bnetza.sqlite3'
"""
        )


cli()
