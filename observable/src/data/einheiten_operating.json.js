import { query } from "./clickhouse.js";

const rows = await query(`
  SELECT
    Quelle,
    CASE Quelle
      WHEN 'GeothermieGrubengasDruckentspannung' THEN 'Geothermie u.a.'
      ELSE Quelle
    END AS Quelle_Label,
    round(sum(Bruttoleistung) / 1000, 0) AS Kapazitaet_MW,
    count(*) AS Anzahl
  FROM Einheiten
  WHERE EinheitBetriebsstatus = 'In Betrieb'
  GROUP BY Quelle
  ORDER BY Kapazitaet_MW DESC
`);

process.stdout.write(JSON.stringify(rows));
