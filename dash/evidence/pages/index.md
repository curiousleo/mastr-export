---
title: Deutschland Übersicht
---

# Energiewende Dashboard

Daten aus dem Marktstammdatenregister (MaStR) der Bundesnetzagentur.

<!-- ============================================================ -->
<!-- HERO KPIs                                                     -->
<!-- ============================================================ -->

```sql renewable_spark
SELECT
    epoch_ms(CAST(Monat_ts AS BIGINT) * 1000) AS Monat,
    round(sum(Brutto_MW) / 1000, 2) AS GW
FROM mastr.monthly_cumulative
WHERE Quelle IN ('Solar', 'Wind', 'Biomasse', 'Wasser', 'GeothermieGrubengasDruckentspannung')
GROUP BY Monat_ts
ORDER BY Monat_ts
```

```sql solar_spark
SELECT epoch_ms(CAST(Monat_ts AS BIGINT) * 1000) AS Monat, round(Brutto_MW / 1000, 3) AS GW
FROM mastr.monthly_cumulative
WHERE Quelle = 'Solar'
ORDER BY Monat_ts
```

```sql wind_spark
SELECT epoch_ms(CAST(Monat_ts AS BIGINT) * 1000) AS Monat, round(Brutto_MW / 1000, 3) AS GW
FROM mastr.monthly_cumulative
WHERE Quelle = 'Wind'
ORDER BY Monat_ts
```

```sql storage_spark
SELECT epoch_ms(CAST(Monat_ts AS BIGINT) * 1000) AS Monat, round(Brutto_MW / 1000, 3) AS GW
FROM mastr.speicher_monthly
ORDER BY Monat_ts
```

```sql additions_total
SELECT round(sum(Kapazitaet_GW), 1) AS GW
FROM mastr.additions_12m
WHERE Quelle IN ('Solar', 'Wind', 'Biomasse', 'Wasser', 'GeothermieGrubengasDruckentspannung')
```

<BigValue
  data={renewable_spark}
  value=GW
  title="Erneuerbare Zubau/Monat (GW)"
  sparkline=Monat
  sparklineType=area
/>

<BigValue
  data={solar_spark}
  value=GW
  title="Solar Zubau/Monat (GW)"
  sparkline=Monat
  sparklineType=area
/>

<BigValue
  data={wind_spark}
  value=GW
  title="Wind Zubau/Monat (GW)"
  sparkline=Monat
  sparklineType=area
/>

<BigValue
  data={storage_spark}
  value=GW
  title="Speicher Zubau/Monat (GW)"
  sparkline=Monat
  sparklineType=area
/>

<BigValue
  data={additions_total}
  value=GW
  title="Zubau letzte 12 Monate (GW)"
/>

---

<!-- ============================================================ -->
<!-- CHART 1: Cumulative Installed Capacity Over Time              -->
<!-- ============================================================ -->

## Kumulierte installierte Leistung

```sql cumulative_capacity
SELECT
    e.Jahr,
    e.Quelle_Label,
    round(sum(e2.Brutto_MW) / 1000, 2) AS Kumulativ_GW
FROM mastr.einheiten_by_year e
INNER JOIN mastr.einheiten_by_year e2
    ON e2.Quelle_Label = e.Quelle_Label AND e2.Jahr <= e.Jahr
WHERE e.Quelle_Label NOT IN ('Verbrennung', 'Kernkraft')
GROUP BY e.Jahr, e.Quelle_Label
ORDER BY e.Jahr, e.Quelle_Label
```

<AreaChart
    data={cumulative_capacity}
    x=Jahr
    y=Kumulativ_GW
    series=Quelle_Label
    title="Kumulierte erneuerbare Kapazität (GW)"
    xAxisTitle="Jahr"
    yAxisTitle="GW"
/>

_Basierend auf Inbetriebnahmedatum. Stilllegungen sind noch nicht berücksichtigt._

---

<!-- ============================================================ -->
<!-- CHART 2: Annual Net Additions by Source                       -->
<!-- ============================================================ -->

## Jährlicher Zubau nach Quelle

```sql annual_additions
SELECT
    Jahr,
    Quelle_Label,
    round(Brutto_MW / 1000, 2) AS Brutto_GW
FROM mastr.einheiten_by_year
WHERE Quelle_Label NOT IN ('Verbrennung', 'Kernkraft')
  AND Jahr >= 2000
ORDER BY Jahr, Quelle_Label
```

<BarChart
    data={annual_additions}
    x=Jahr
    y=Brutto_GW
    series=Quelle_Label
    title="Jährlicher Zubau erneuerbare Energien (GW)"
    xAxisTitle="Jahr"
    yAxisTitle="GW"
    type=grouped
/>

---

<!-- ============================================================ -->
<!-- CHART 3: Current Energy Mix (Donut)                           -->
<!-- ============================================================ -->

## Aktueller Erzeugungsmix

```sql energy_mix
SELECT
    Quelle_Label,
    Kapazitaet_GW
FROM mastr.einheiten_operating
ORDER BY Kapazitaet_GW DESC
```

<ECharts config={
    {
        tooltip: { trigger: 'item', formatter: '{b}: {c} GW ({d}%)' },
        series: [{
            type: 'pie',
            radius: ['40%', '70%'],
            itemStyle: { borderRadius: 6, borderColor: '#fff', borderWidth: 2 },
            label: { show: true, formatter: '{b}\n{d}%' },
            data: [...energy_mix].map(row => ({ name: row.Quelle_Label, value: row.Kapazitaet_GW }))
        }]
    }
} />

---

<!-- ============================================================ -->
<!-- CHART 4: Capacity by Bundesland                               -->
<!-- ============================================================ -->

## Kapazität nach Bundesland

```sql by_bundesland
SELECT
    Bundesland,
    Quelle_Label,
    Kapazitaet_GW
FROM mastr.einheiten_by_bundesland
WHERE Quelle_Label NOT IN ('Verbrennung', 'Kernkraft')
ORDER BY Bundesland, Quelle_Label
```

<BarChart
    data={by_bundesland}
    x=Bundesland
    y=Kapazitaet_GW
    series=Quelle_Label
    title="Erneuerbare Kapazität nach Bundesland (GW)"
    yAxisTitle="GW"
    type=stacked
    swapXY=true
/>

---

<!-- ============================================================ -->
<!-- CHART 5: Recent Activity                                      -->
<!-- ============================================================ -->

## Zubau letzte 12 Monate

```sql recent
SELECT
    Quelle_Label,
    Kapazitaet_GW,
    Anzahl
FROM mastr.additions_12m
ORDER BY Kapazitaet_GW DESC
```

<DataTable data={recent}>
    <Column id=Quelle_Label title="Quelle" />
    <Column id=Kapazitaet_GW title="Kapazität (GW)" fmt=num3 />
    <Column id=Anzahl title="Anzahl Einheiten" fmt=num0 />
</DataTable>

_Einheiten mit Inbetriebnahmedatum in den letzten 12 Monaten und Status „In Betrieb"._
