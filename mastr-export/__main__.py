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

    extract = subparsers.add_parser("extract")
    extract.add_argument(
        "export_file",
        nargs=1,
        help="path to the Marktstammdatenregister export ZIP file",
    )
    extract.add_argument(
        "--spec",
        default=(importlib.resources.files(spec_data) / "Gesamtdatenexport.yaml"),
        help="path to the YAML file containing the list of specs",
    )
    extract.add_argument(
        "--duckdb",
        required=True,
        help="DuckDB database file path",
    )
    extract.add_argument(
        "--sqlite",
        help="SQLite database file path",
    )
    extract.add_argument(
        "--csv-dir",
        help="CSV directory",
    )
    extract.add_argument(
        "--parquet-dir",
        help="Parquet directory",
    )
    extract.add_argument(
        "--show-per-file-progress",
        default=False,
        action="store_true",
        help="show a progress bar for each individual file?",
    )
    extract.set_defaults(
        func=lambda args: run(
            args.spec,
            args.export_file[0],
            args.duckdb,
            args.sqlite,
            args.csv_dir,
            args.parquet_dir,
            args.show_per_file_progress,
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


def run(
    spec, export, duckdb_file, sqlite_file, csv_dir, parquet_dir, show_per_file_progress
):
    with duckdb.connect(duckdb_file) as duckdb_con:
        specs = Specs.load(spec)
        extract(export, specs, duckdb_con, show_per_file_progress)

        if csv_dir is not None:
            print_runtime(
                f"Exporting CSV files to {csv_dir} ...",
                action=duckdb_con.sql,
                arg=f"""export database '{csv_dir}' (format csv)""",
            )

        if parquet_dir is not None:
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


def extract(
    export,
    specs: Specs,
    duckdb_con: duckdb.DuckDBPyConnection,
    show_per_file_progress,
):
    for spec in specs.specs:
        duckdb_con.sql(spec.duckdb_schema())

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
            duckdb_con.sql(f"""INSERT INTO "{d.element}" SELECT * FROM df""")

    duckdb_con.sql("vacuum analyze")
    return spec_to_xml_files


cli()
