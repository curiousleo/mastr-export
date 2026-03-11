-- Monthly storage additions for sparkline (last 24 months)
SELECT
    toUnixTimestamp(toStartOfMonth(Inbetriebnahmedatum)) AS Monat_ts,
    round(sum(Bruttoleistung) / 1000, 1) AS Brutto_MW
FROM EinheitenStromSpeicher
WHERE Inbetriebnahmedatum IS NOT NULL
  AND Inbetriebnahmedatum >= today() - INTERVAL 24 MONTH
GROUP BY Monat_ts
ORDER BY Monat_ts
