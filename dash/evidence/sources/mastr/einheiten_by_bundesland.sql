-- Operating capacity by Bundesland and source (for state bar chart)
SELECT
    Bundesland,
    Quelle,
    CASE Quelle
        WHEN 'GeothermieGrubengasDruckentspannung' THEN 'Geothermie u.a.'
        ELSE Quelle
    END AS Quelle_Label,
    round(sum(Bruttoleistung) / 1e6, 3) AS Kapazitaet_GW,
    count(*) AS Anzahl
FROM Einheiten
WHERE EinheitBetriebsstatus = 'In Betrieb'
  AND Bundesland IS NOT NULL
GROUP BY Bundesland, Quelle
ORDER BY Bundesland, Kapazitaet_GW DESC
