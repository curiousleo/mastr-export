-- Monthly cumulative capacity by source for sparklines (last 12 months)
-- Uses Einheiten view (generation) grouped by month
SELECT
    Quelle,
    toUnixTimestamp(toStartOfMonth(Inbetriebnahmedatum)) AS Monat_ts,
    round(sum(Bruttoleistung) / 1000, 1) AS Brutto_MW
FROM Einheiten
WHERE Inbetriebnahmedatum IS NOT NULL
  AND Inbetriebnahmedatum >= today() - INTERVAL 24 MONTH
GROUP BY Quelle, Monat_ts
ORDER BY Quelle, Monat_ts
