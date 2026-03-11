-- Storage capacity by commissioning year (for cumulative chart inclusion)
SELECT
    toYear(Inbetriebnahmedatum) AS Jahr,
    round(sum(Bruttoleistung) / 1000, 1) AS Brutto_MW,
    count(*) AS Anzahl
FROM EinheitenStromSpeicher
WHERE Inbetriebnahmedatum IS NOT NULL
  AND toYear(Inbetriebnahmedatum) >= 1990
  AND toYear(Inbetriebnahmedatum) <= toYear(now())
GROUP BY Jahr
ORDER BY Jahr
