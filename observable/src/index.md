---
title: Deutschland Übersicht
toc: true
---

# Energiewende Dashboard

Daten aus dem [Marktstammdatenregister (MaStR)](https://www.marktstammdatenregister.de/) der Bundesnetzagentur. Alle Angaben beziehen sich auf die installierte Bruttoleistung (Nennleistung) laut Registermeldung — nicht auf die tatsächliche Einspeisung oder den aktuellen Betriebszustand.

```js
const de = new Intl.NumberFormat("de-DE", { maximumFractionDigits: 0 });
const de1 = new Intl.NumberFormat("de-DE", { maximumFractionDigits: 1 });

const monthly = FileAttachment("data/monthly_cumulative.json").json();
const speicherMonthly = FileAttachment("data/speicher_monthly.json").json();
const additions12m = FileAttachment("data/additions_12m.json").json();
const byYear = FileAttachment("data/einheiten_by_year.json").json();
const operating = FileAttachment("data/einheiten_operating.json").json();
const byBundesland = FileAttachment("data/einheiten_by_bundesland.json").json();
```

```js
const renewableQuellen = ["Solar", "Wind", "Biomasse", "Wasser", "GeothermieGrubengasDruckentspannung"];

const renewableSpark = monthly
  .filter(d => renewableQuellen.includes(d.Quelle))
  .reduce((acc, d) => {
    const existing = acc.find(r => r.Monat === d.Monat);
    if (existing) existing.MW += d.Brutto_MW;
    else acc.push({ Monat: new Date(d.Monat), MW: d.Brutto_MW });
    return acc;
  }, [])
  .sort((a, b) => a.Monat - b.Monat);

const solarSpark = monthly
  .filter(d => d.Quelle === "Solar")
  .map(d => ({ Monat: new Date(d.Monat), MW: d.Brutto_MW }));

const windSpark = monthly
  .filter(d => d.Quelle === "Wind")
  .map(d => ({ Monat: new Date(d.Monat), MW: d.Brutto_MW }));

const storageSpark = speicherMonthly
  .map(d => ({ Monat: new Date(d.Monat), MW: d.Brutto_MW }));

const additionsTotal = additions12m
  .filter(d => renewableQuellen.includes(d.Quelle))
  .reduce((sum, d) => sum + d.Kapazitaet_MW, 0);
```

## Zubau der letzten 24 Monate

<div class="grid grid-cols-5">
  <div class="card">
    <h2>Erneuerbare / Monat</h2>
    <span class="big">${de.format(renewableSpark.at(-1)?.MW ?? 0)} MW</span>
    ${resize((width) => Plot.plot({
      width, height: 40, axis: null, margin: 0, marks: [
        Plot.areaY(renewableSpark, {x: "Monat", y: "MW", fill: "#2563eb", fillOpacity: 0.3, curve: "basis"}),
        Plot.lineY(renewableSpark, {x: "Monat", y: "MW", stroke: "#2563eb", strokeWidth: 1.5, curve: "basis"})
      ]
    }))}
  </div>
  <div class="card">
    <h2>Solar / Monat</h2>
    <span class="big">${de.format(solarSpark.at(-1)?.MW ?? 0)} MW</span>
    ${resize((width) => Plot.plot({
      width, height: 40, axis: null, margin: 0, marks: [
        Plot.areaY(solarSpark, {x: "Monat", y: "MW", fill: "#FFB800", fillOpacity: 0.3, curve: "basis"}),
        Plot.lineY(solarSpark, {x: "Monat", y: "MW", stroke: "#FFB800", strokeWidth: 1.5, curve: "basis"})
      ]
    }))}
  </div>
  <div class="card">
    <h2>Wind / Monat</h2>
    <span class="big">${de.format(windSpark.at(-1)?.MW ?? 0)} MW</span>
    ${resize((width) => Plot.plot({
      width, height: 40, axis: null, margin: 0, marks: [
        Plot.areaY(windSpark, {x: "Monat", y: "MW", fill: "#2563EB", fillOpacity: 0.3, curve: "basis"}),
        Plot.lineY(windSpark, {x: "Monat", y: "MW", stroke: "#2563EB", strokeWidth: 1.5, curve: "basis"})
      ]
    }))}
  </div>
  <div class="card">
    <h2>Speicher / Monat</h2>
    <span class="big">${de.format(storageSpark.at(-1)?.MW ?? 0)} MW</span>
    ${resize((width) => Plot.plot({
      width, height: 40, axis: null, margin: 0, marks: [
        Plot.areaY(storageSpark, {x: "Monat", y: "MW", fill: "#7C3AED", fillOpacity: 0.3, curve: "basis"}),
        Plot.lineY(storageSpark, {x: "Monat", y: "MW", stroke: "#7C3AED", strokeWidth: 1.5, curve: "basis"})
      ]
    }))}
  </div>
  <div class="card">
    <h2>Zubau 12 Monate</h2>
    <span class="big">${de.format(additionsTotal)} MW</span>
  </div>
</div>

_Neu in Betrieb genommene Leistung pro Monat (Bruttoleistung). Zeigt das Tempo des Ausbaus, nicht den Gesamtbestand._

---

## Kumulierte installierte Leistung

```js
const excludeQuellen = ["Verbrennung", "Kernkraft"];

const byYearClean = byYear
  .filter(d => !excludeQuellen.includes(d.Quelle_Label))
  .map(d => ({ ...d, Jahr: +d.Jahr, Brutto_MW: +d.Brutto_MW }));

// compute cumulative MW per source
const cumulativeData = [];
const quellenSet = [...new Set(byYearClean.map(d => d.Quelle_Label))];
for (const quelle of quellenSet) {
  const rows = byYearClean.filter(d => d.Quelle_Label === quelle).sort((a, b) => a.Jahr - b.Jahr);
  let cumSum = 0;
  for (const row of rows) {
    cumSum += row.Brutto_MW;
    cumulativeData.push({ Jahr: row.Jahr, Quelle_Label: quelle, Kumulativ_MW: Math.round(cumSum) });
  }
}
```

```js
Plot.plot({
  width: 928,
  height: 500,
  marginLeft: 60,
  x: { label: "Jahr", tickFormat: "d" },
  y: { label: "MW", tickFormat: (d) => de.format(d) },
  color: { legend: true },
  marks: [
    Plot.areaY(cumulativeData, {
      x: "Jahr", y: "Kumulativ_MW", fill: "Quelle_Label",
      curve: "basis", order: "sum"
    }),
    Plot.ruleY([0])
  ]
})
```

_Summe aller jemals in Betrieb genommenen Anlagen nach Inbetriebnahmedatum. Stilllegungen und Rückbauten sind nicht abgezogen — die tatsächlich aktive Kapazität ist niedriger._

---

## Jährlicher Zubau nach Quelle

```js
const annualData = byYearClean.filter(d => d.Jahr >= 2000);
```

```js
Plot.plot({
  width: 928,
  height: 500,
  marginLeft: 60,
  x: { label: "Jahr", tickFormat: "d" },
  y: { label: "MW", tickFormat: (d) => de.format(d) },
  color: { legend: true },
  marks: [
    Plot.barY(annualData, {
      x: "Jahr", y: "Brutto_MW", fill: "Quelle_Label",
      offset: null
    }),
    Plot.ruleY([0])
  ]
})
```

_Neu installierte Bruttoleistung pro Jahr nach Energieträger. Dies ist der jährliche Zubau, nicht der Gesamtbestand oder die tatsächliche Stromerzeugung._

---

## Installierte Leistung nach Energieträger

```js
const mixData = operating.map(d => ({ name: d.Quelle_Label, value: +d.Kapazitaet_MW }));
```

```js
Plot.plot({
  width: 500,
  height: 500,
  marks: [
    Plot.barX(mixData, {
      x: "value", y: "name", fill: "name",
      sort: { y: "-x" }
    }),
    Plot.text(mixData, {
      x: "value", y: "name",
      text: d => `${de.format(d.value)} MW`,
      dx: 5, textAnchor: "start"
    }),
    Plot.ruleX([0])
  ],
  x: { label: "MW", tickFormat: (d) => de.format(d) },
  y: { label: null },
  color: { legend: false }
})
```

_Anteil der aktuell als „In Betrieb" gemeldeten Bruttoleistung je Energieträger. Die installierte Leistung sagt nichts über die tatsächlich erzeugte Strommenge aus — z.B. liefert Solar nur bei Sonnenschein._

---

## Kapazität nach Bundesland

```js
const bundeslandData = byBundesland
  .filter(d => !excludeQuellen.includes(d.Quelle_Label))
  .map(d => ({ ...d, Kapazitaet_MW: +d.Kapazitaet_MW }));
```

```js
Plot.plot({
  width: 928,
  height: 600,
  marginLeft: 160,
  x: { label: "MW", tickFormat: (d) => de.format(d) },
  y: { label: null },
  color: { legend: true },
  marks: [
    Plot.barX(bundeslandData, Plot.stackX({
      x: "Kapazitaet_MW", y: "Bundesland", fill: "Quelle_Label",
      sort: { y: "-x", reduce: "sum" }
    })),
    Plot.ruleX([0])
  ]
})
```

_Aktuell als „In Betrieb" gemeldete Bruttoleistung je Bundesland und Energieträger._

---

## Zubau letzte 12 Monate

```js
const recentData = additions12m.filter(d => renewableQuellen.includes(d.Quelle));
```

```js
Inputs.table(recentData.map(d => ({
  Quelle: d.Quelle_Label,
  "Kapazität (MW)": de.format(d.Kapazitaet_MW),
  "Anzahl Einheiten": de.format(d.Anzahl)
})))
```

_Anlagen mit Inbetriebnahmedatum in den letzten 12 Monaten und Status „In Betrieb". Bruttoleistung (Nennleistung), nicht tatsächliche Einspeisung._
