-- Net new renewable capacity added in last 12 months
SELECT
    Quelle,
    CASE Quelle
        WHEN 'GeothermieGrubengasDruckentspannung' THEN 'Geothermie u.a.'
        ELSE Quelle
    END AS Quelle_Label,
    round(sum(Bruttoleistung) / 1e6, 3) AS Kapazitaet_GW,
    count(*) AS Anzahl
FROM Einheiten
WHERE Inbetriebnahmedatum >= today() - INTERVAL 12 MONTH
  AND EinheitBetriebsstatus = 'In Betrieb'
GROUP BY Quelle
ORDER BY Kapazitaet_GW DESC
