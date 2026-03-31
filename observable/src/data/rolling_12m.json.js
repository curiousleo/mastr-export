import { query } from "./clickhouse.js";

const rows = await query(`
  SELECT
    Quelle,
    CASE Quelle
      WHEN 'GeothermieGrubengasDruckentspannung' THEN 'Geothermie u.a.'
      ELSE Quelle
    END AS Quelle_Label,
    Monat,
    round(sum(Brutto_MW) OVER (
      PARTITION BY Quelle
      ORDER BY Monat
      ROWS BETWEEN 11 PRECEDING AND CURRENT ROW
    ), 1) AS Rolling_MW
  FROM (
    SELECT
      Quelle,
      toStartOfMonth(Inbetriebnahmedatum) AS Monat,
      sum(Bruttoleistung) / 1000 AS Brutto_MW
    FROM Einheiten
    WHERE Inbetriebnahmedatum IS NOT NULL
      AND Inbetriebnahmedatum >= '1990-01-01'
    GROUP BY Quelle, Monat
  )
  ORDER BY Quelle, Monat
`);

process.stdout.write(JSON.stringify(rows));
