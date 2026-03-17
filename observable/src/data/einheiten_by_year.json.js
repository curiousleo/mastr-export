import { query } from "./clickhouse.js";

const rows = await query(`
  SELECT
    Quelle,
    CASE Quelle
      WHEN 'GeothermieGrubengasDruckentspannung' THEN 'Geothermie u.a.'
      ELSE Quelle
    END AS Quelle_Label,
    toYear(Inbetriebnahmedatum) AS Jahr,
    round(sum(Bruttoleistung) / 1000, 1) AS Brutto_MW,
    count(*) AS Anzahl
  FROM Einheiten
  WHERE Inbetriebnahmedatum IS NOT NULL
    AND toYear(Inbetriebnahmedatum) >= 1990
    AND toYear(Inbetriebnahmedatum) <= toYear(now())
  GROUP BY Quelle, Jahr
  ORDER BY Quelle, Jahr
`);

process.stdout.write(JSON.stringify(rows));
