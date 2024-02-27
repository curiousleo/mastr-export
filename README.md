# mastr-export

Python library and tool for extracting data from the German grid agency's data
export (Gesamtdatenexport der Bundesnetzagentur).

## Usage

First, download the latest export file. Using a download accelerator like
[`axel`](https://github.com/axel-download-accelerator/axel) is recommended, but
`curl` or `wget` instead of `axel` will work too:

```
$ axel --output=Gesamtdatenexport.zip "$(python -m mastr-export print-export-url)"
```

Then, extract the export. At least one of `--parquet-dir` or `csv-dir` must be
passed:

```
$ python -m mastr-export extract \
    Gesamtdatenexport.zip \
    --parquet-dir out/parquet \
    --csv-dir out/csv
```

Optionally, you can now package the data up in databases: the Parquet files can
be assembled into a DuckDB database file, and the CSV files into a SQLite
database file.

The `extract` command will output the appropriate instructions. If you passed
`--parquet-dir`, you'll get:

```
Parquet export finished. You can import the Parquet files into a DuckDB file
called 'bnetza.duckdb' with the following command:

$ duckdb 'bnetza.duckdb' -init 'out/parquet/import-duckdb.sql' -bail -batch -echo -no-stdin
```

... and if you passed `--csv-dir`, it will print:

```
CSV export finished. You can import the CSV files into a SQLite file
called 'bnetza.sqlite3' with the following command:

$ sqlite3 -bail 'bnetza.sqlite3' <'out/csv/import-sqlite.sql' 

You can optionally add common indices with this command:

$ sqlite3 -bail 'bnetza.sqlite3' <'out/csv/index-sqlite.sql'
```