-- Current operating capacity by source type (for KPIs + donut chart)
SELECT
    Quelle,
    CASE Quelle
        WHEN 'GeothermieGrubengasDruckentspannung' THEN 'Geothermie u.a.'
        ELSE Quelle
    END AS Quelle_Label,
    round(sum(Bruttoleistung) / 1e6, 3) AS Kapazitaet_GW,
    count(*) AS Anzahl
FROM Einheiten
WHERE EinheitBetriebsstatus = 'In Betrieb'
GROUP BY Quelle
ORDER BY Kapazitaet_GW DESC
