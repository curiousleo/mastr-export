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

Then, extract the export to DuckDB or SQLite:

```
$ python -m mastr-export extract-to-duckdb --help
usage: mastr-export extract-to-duckdb [-h] --export EXPORT [--spec SPEC] --duckdb DUCKDB [--show-per-file-progress]

options:
  -h, --help            show this help message and exit
  --export EXPORT       (input) path to the Marktstammdatenregister export ZIP file
  --spec SPEC           (input) path to the YAML file containing the list of specs
  --duckdb DUCKDB       (output) DuckDB database file path
  --show-per-file-progress
                        show a progress bar for each individual file?
```

... or for SQLite:

```
$ python -m mastr-export extract-to-sqlite --help 
usage: mastr-export extract-to-sqlite [-h] --export EXPORT [--spec SPEC] --sqlite SQLITE [--show-per-file-progress]

options:
  -h, --help            show this help message and exit
  --export EXPORT       (input) path to the Marktstammdatenregister export ZIP file
  --spec SPEC           (input) path to the YAML file containing the list of specs
  --sqlite SQLITE       (output) SQLite database file path
  --show-per-file-progress
                        show a progress bar for each individual file?
```
