-- Battery storage operating capacity (separate table, not in Einheiten view)
-- JOIN with Katalogwerte instead of dictGet to avoid CREATE TEMPORARY TABLE privilege
SELECT
    round(sum(s.Bruttoleistung) / 1e6, 3) AS Kapazitaet_GW,
    count(*) AS Anzahl
FROM EinheitenStromSpeicher s
INNER JOIN Katalogwerte k ON k.Id = s.EinheitBetriebsstatus
WHERE k.Wert = 'In Betrieb'
