import sqlite3
from .spec import Specs
from .parser import Parser

from . import download
from . import spec_data

import argparse
import datetime
import duckdb
import importlib.resources
import polars as pl
import time
from tqdm.auto import tqdm
from tqdm.utils import CallbackIOWrapper
import zipfile


def cli():
    parser = argparse.ArgumentParser(prog="mastr-export")
    subparsers = parser.add_subparsers(required=True)

    print_export_url = subparsers.add_parser("print-export-url")
    print_export_url.set_defaults(func=lambda _args: print(download.print_export_url()))

    duckdb_extract = subparsers.add_parser("extract-to-duckdb")
    duckdb_extract.add_argument(
        "--export",
        required=True,
        help="(input) path to the Marktstammdatenregister export ZIP file",
    )
    duckdb_extract.add_argument(
        "--spec",
        default=(importlib.resources.files(spec_data) / "Gesamtdatenexport.yaml"),
        help="(input) path to the YAML file containing the list of specs",
    )
    duckdb_extract.add_argument(
        "--duckdb",
        required=True,
        help="(outpu) DuckDB database file path",
    )
    duckdb_extract.add_argument(
        "--show-per-file-progress",
        default=False,
        action="store_true",
        help="show a progress bar for each individual file?",
    )
    duckdb_extract.set_defaults(
        func=lambda args: extract_to_duckdb(
            args.spec,
            args.export,
            args.duckdb,
            args.show_per_file_progress,
        )
    )

    sqlite_extract = subparsers.add_parser("extract-to-sqlite")
    sqlite_extract.add_argument(
        "--export",
        required=True,
        help="(input) path to the Marktstammdatenregister export ZIP file",
    )
    sqlite_extract.add_argument(
        "--spec",
        default=(importlib.resources.files(spec_data) / "Gesamtdatenexport.yaml"),
        help="(input) path to the YAML file containing the list of specs",
    )
    sqlite_extract.add_argument(
        "--sqlite",
        required=True,
        help="(output) SQLite database file path",
    )
    sqlite_extract.add_argument(
        "--show-per-file-progress",
        default=False,
        action="store_true",
        help="show a progress bar for each individual file?",
    )
    sqlite_extract.set_defaults(
        func=lambda args: extract_to_sqlite(
            args.spec,
            args.export,
            args.sqlite,
            args.show_per_file_progress,
        )
    )

    export = subparsers.add_parser("export-from-duckdb")
    export.add_argument(
        "--duckdb",
        required=True,
        help="DuckDB database file path",
    )
    export.add_argument(
        "--sqlite",
        help="SQLite database file path",
    )
    export.add_argument(
        "--csv-dir",
        help="CSV directory",
    )
    export.add_argument(
        "--parquet-dir",
        help="Parquet directory",
    )
    export.set_defaults(
        func=lambda args: export_from_duckdb(
            args.duckdb,
            args.sqlite,
            args.csv_dir,
            args.parquet_dir,
        )
    )

    args = parser.parse_args()
    args.func(args)


def print_runtime(message, action, arg):
    print(message, end=" ", flush=True)
    start_ns = time.perf_counter_ns()
    action(arg)
    delta_ns = time.perf_counter_ns() - start_ns
    delta = datetime.timedelta(microseconds=delta_ns // 1_000)
    print(f"took {str(delta)}")


def extract(specs: Specs, export, show_per_file_progress):
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
            yield d, df


def extract_to_duckdb(spec, export, duckdb_file, show_per_file_progress):
    specs = Specs.load(spec)
    with duckdb.connect(duckdb_file) as duckdb_con:
        for spec in specs.specs:
            duckdb_con.sql(spec.duckdb_schema())

    for d, df in extract(specs, export, show_per_file_progress):
        with duckdb.connect(duckdb_file) as duckdb_con:
            duckdb_con.sql(f"""INSERT INTO "{d.element}" SELECT * FROM df""")

    with duckdb.connect(duckdb_file) as duckdb_con:
        duckdb_con.sql("vacuum analyze")


def extract_to_sqlite(spec, export, sqlite_file, show_per_file_progress):
    con = sqlite3.connect(sqlite_file)
    specs = Specs.load(spec)
    with con:
        for spec in specs.specs:
            con.execute(spec.sqlite_schema())

    for d, df in extract(specs, export, show_per_file_progress):
        with con:
            columns = ", ".join(f"'{name}'" for name in df.columns)
            values = ", ".join("?" for _ in range(len(df.columns)))
            stmt = f"""insert into "{d.element}" ({columns}) values ({values})"""
            con.executemany(stmt, df.iter_rows())

    with con:
        con.execute("analyze")
        con.execute("vacuum")


def export_from_duckdb(duckdb_file, sqlite_file, csv_dir, parquet_dir):
    if csv_dir is not None:
        with duckdb.connect(duckdb_file, read_only=True) as duckdb_con:
            print_runtime(
                f"Exporting CSV files to {csv_dir} ...",
                action=duckdb_con.sql,
                arg=f"""export database '{csv_dir}' (format csv)""",
            )

    if parquet_dir is not None:
        with duckdb.connect(duckdb_file, read_only=True) as duckdb_con:
            print_runtime(
                f"Exporting Parquet files to {parquet_dir} ...",
                action=duckdb_con.sql,
                arg=f"""export database '{parquet_dir}' (format parquet)""",
            )

    if sqlite_file is not None:
        print_runtime(
            f"Exporting SQLite database to {sqlite_file} ...",
            action=duckdb.sql,
            arg=f"""
attach '{duckdb_file}' as source (read_only);
attach '{sqlite_file}' as target (type sqlite);
copy from database source to target;
""",
        )


cli()
