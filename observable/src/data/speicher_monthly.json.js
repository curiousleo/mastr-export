import { query } from "./clickhouse.js";

const rows = await query(`
  SELECT
    toStartOfMonth(Inbetriebnahmedatum) AS Monat,
    round(sum(Bruttoleistung) / 1000, 1) AS Brutto_MW
  FROM EinheitenStromSpeicher
  WHERE Inbetriebnahmedatum IS NOT NULL
    AND Inbetriebnahmedatum >= today() - INTERVAL 24 MONTH
  GROUP BY Monat
  ORDER BY Monat
`);

process.stdout.write(JSON.stringify(rows));
