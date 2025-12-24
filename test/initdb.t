Set up the testing environment.
  $ . "$TESTDIR"/setup.sh

Show the help message.
  $ mastr-export initdb --help | sed -E 's/ +$//'
  Usage: initdb [options]...
  Options:
        --db-name DB_NAME       (input) Database name
        --parquet-dir PARQUET_DIR
                                (input) Directory containing Parquet files
        --parquet-dir-override PARQUET_DIR_OVERRIDE
                                (input) Ducklake data directory override
        --db-dir DB_DIR         (output) DuckDB directory (output is written to $db_dir/catalog.ducklake)
        --help

Create a database.
  $ mkdir -p ./work/data
  $ cp "$TESTDIR/EinheitenKernkraft.parquet" ./work/data
  $ mastr-export initdb --db-name mastr \
  >   --parquet-dir ./data \
  >   --parquet-dir-override https://s3.mydata.com/mastr-bucket \
  >   --db-dir .
  .mode list
  LOAD ducklake;
  ATTACH 'ducklake:./catalog.ducklake' AS mastr (DATA_PATH './tmp_always_empty');
  CREATE TABLE mastr.EinheitenKernkraft AS SELECT * FROM read_parquet('./data/EinheitenKernkraft.parquet') WITH NO DATA;
  CALL ducklake_add_data_files('mastr', 'EinheitenKernkraft', './data/EinheitenKernkraft.parquet');
  DETACH mastr;
  ATTACH './catalog.ducklake' AS catalog;
  UPDATE catalog.ducklake_data_file
     SET path = replace(path, './data', 'https://s3.mydata.com/mastr-bucket')
   WHERE path LIKE './data/%';
  DETACH catalog;
