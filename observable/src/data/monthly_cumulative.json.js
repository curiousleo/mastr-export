import { query } from "./clickhouse.js";

const rows = await query(`
  SELECT
    Quelle,
    toStartOfMonth(Inbetriebnahmedatum) AS Monat,
    round(sum(Bruttoleistung) / 1000, 1) AS Brutto_MW
  FROM Einheiten
  WHERE Inbetriebnahmedatum IS NOT NULL
    AND Inbetriebnahmedatum >= today() - INTERVAL 12 MONTH
  GROUP BY Quelle, Monat
  ORDER BY Quelle, Monat
`);

process.stdout.write(JSON.stringify(rows));
