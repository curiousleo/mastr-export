# Census data

The most recent national census data is from 2022 as far as I can tell.

It's coded by [Amtlicher Regionalschlüssel (ARS)](https://de.wikipedia.org/wiki/Amtlicher_Gemeindeschl%C3%BCssel#Regionalschl%C3%BCssel) whereas the `Gemeindeschluessel` field in Marktstammdatenregister uses Amtlicher Gemeindeschlüssel (AGS).

The conversion from the 12-letter ARS to the 8-letter AGS is to take only the first 5 + the last 3 letters.

To regenerate `zensus2022.parquet`, run:

1. Download "CSV (Flat)" from [ergebnisse.zensus2022.de](https://ergebnisse.zensus2022.de/datenbank/online/statistic/1000A/table/1000A-0000)
2. Unzip `1000A-0000_de_flat.zip`
2. In DuckDB, run:
   ```
   copy (
     select left("1_variable_attribute_code", 5) || right("1_variable_attribute_code", 3) as AGS,
            "1_variable_attribute_label" as Gemeinde,
            "value" as AnzahlPersonen
     from read_csv('~/Downloads/1000A-0000_de_flat/1000A-0000_de_flat.csv')
   ) to 'zensus2022.parquet' (format parquet, compression zstd);
   ```

[View `zensus2022.parquet` in the DuckDB Shell.](https://shell.duckdb.org/#queries=v0,CREATE-TABLE-Zensus-AS-(SELECT-*-FROM-'https%3A%2F%2Fraw.githubusercontent.com%2Fcuriousleo%2Fmastr%20export%2Fmain%2Fmastr_export%2Fstatic_data%2Fzensus2022.parquet')~,SELECT-*-FROM-Zensus-LIMIT-20~)
