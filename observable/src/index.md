---
title: Deutschland Übersicht
toc: true
---

# Energiewende Dashboard

Daten aus dem [Marktstammdatenregister (MaStR)](https://www.marktstammdatenregister.de/) der Bundesnetzagentur. Alle Angaben beziehen sich auf die installierte Bruttoleistung (Nennleistung) laut Registermeldung — nicht auf die tatsächliche Einspeisung oder den aktuellen Betriebszustand.

```js
const fmtMW = new Intl.NumberFormat("de-DE", { maximumFractionDigits: 0 });
const renewableQuellen = new Set(["Solar", "Wind", "Biomasse", "Wasser", "GeothermieGrubengasDruckentspannung"]);
const excludeQuellen = new Set(["Verbrennung", "Kernkraft"]);

const monthly = FileAttachment("data/monthly_cumulative.json").json();
const speicherMonthly = FileAttachment("data/speicher_monthly.json").json();
const additions12m = FileAttachment("data/additions_12m.json").json();
const byYear = FileAttachment("data/einheiten_by_year.json").json();
const operating = FileAttachment("data/einheiten_operating.json").json();
const byBundesland = FileAttachment("data/einheiten_by_bundesland.json").json();
```

```js
function sparkcard(title, data, color) {
  const latest = data.at(-1)?.MW ?? 0;
  return html`<div class="card">
    <h2>${title}</h2>
    <span class="big">${fmtMW.format(latest)} MW</span>
    ${resize((width) => Plot.plot({
      width, height: 40, axis: null, margin: 0, marks: [
        Plot.areaY(data, {x: "Monat", y: "MW", fill: color, fillOpacity: 0.3, curve: "basis"}),
        Plot.lineY(data, {x: "Monat", y: "MW", stroke: color, strokeWidth: 1.5, curve: "basis"})
      ]
    }))}
  </div>`;
}

function tipTitle(d, valueField) {
  return `${d.Quelle_Label}\n${fmtMW.format(d[valueField])} MW`;
}

function toMonthly(rows) {
  return rows.map(d => ({ Monat: new Date(d.Monat), MW: d.Brutto_MW }));
}

const plotDefaults = { width: 928, height: 500, marginLeft: 60 };
const yearAxis = { label: "Jahr", tickFormat: "d" };
const mwAxis = { label: "MW", tickFormat: (d) => fmtMW.format(d) };
```

```js
const solarSpark = toMonthly(monthly.filter(d => d.Quelle === "Solar"));
const windSpark = toMonthly(monthly.filter(d => d.Quelle === "Wind"));
const storageSpark = toMonthly(speicherMonthly);

const renewableSpark = monthly
  .filter(d => renewableQuellen.has(d.Quelle))
  .reduce((acc, d) => {
    const existing = acc.find(r => r.Monat === d.Monat);
    if (existing) existing.MW += d.Brutto_MW;
    else acc.push({ Monat: d.Monat, MW: d.Brutto_MW });
    return acc;
  }, [])
  .sort((a, b) => a.Monat < b.Monat ? -1 : 1)
  .map(d => ({ Monat: new Date(d.Monat), MW: d.MW }));

const additionsTotal = additions12m
  .filter(d => renewableQuellen.has(d.Quelle))
  .reduce((sum, d) => sum + d.Kapazitaet_MW, 0);

const byYearClean = byYear
  .filter(d => !excludeQuellen.has(d.Quelle_Label))
  .map(d => ({ ...d, Jahr: +d.Jahr, Brutto_MW: +d.Brutto_MW }));

const cumulativeData = [];
for (const quelle of new Set(byYearClean.map(d => d.Quelle_Label))) {
  let cumSum = 0;
  for (const row of byYearClean.filter(d => d.Quelle_Label === quelle).sort((a, b) => a.Jahr - b.Jahr)) {
    cumSum += row.Brutto_MW;
    cumulativeData.push({ Jahr: row.Jahr, Quelle_Label: quelle, Kumulativ_MW: Math.round(cumSum) });
  }
}

const annualData = byYearClean.filter(d => d.Jahr >= 2000);

const mixData = operating.map(d => ({ name: d.Quelle_Label, value: +d.Kapazitaet_MW }));

const bundeslandData = byBundesland
  .filter(d => !excludeQuellen.has(d.Quelle_Label))
  .map(d => ({ ...d, Kapazitaet_MW: +d.Kapazitaet_MW }));

const recentData = additions12m.filter(d => renewableQuellen.has(d.Quelle));
```

## Zubau der letzten 24 Monate

<div class="grid grid-cols-4">
  ${sparkcard("Erneuerbare / Monat", renewableSpark, "#2563eb")}
  ${sparkcard("Solar / Monat", solarSpark, "#FFB800")}
  ${sparkcard("Wind / Monat", windSpark, "#2563EB")}
  ${sparkcard("Speicher / Monat", storageSpark, "#7C3AED")}
  <div class="card">
    <h2>Zubau 12 Monate</h2>
    <span class="big">${fmtMW.format(additionsTotal)} MW</span>
  </div>
</div>

_Neu in Betrieb genommene Leistung pro Monat (Bruttoleistung). Zeigt das Tempo des Ausbaus, nicht den Gesamtbestand._

---

## Kumulierte installierte Leistung

```js
Plot.plot({
  ...plotDefaults,
  x: yearAxis,
  y: mwAxis,
  color: { legend: true },
  marks: [
    Plot.areaY(cumulativeData, {
      x: "Jahr", y: "Kumulativ_MW", fill: "Quelle_Label",
      curve: "basis", order: "sum"
    }),
    Plot.tip(cumulativeData, Plot.pointerX(Plot.stackY({
      x: "Jahr", y: "Kumulativ_MW", fill: "Quelle_Label",
      order: "sum",
      title: d => tipTitle(d, "Kumulativ_MW")
    }))),
    Plot.ruleY([0])
  ]
})
```

_Summe aller jemals in Betrieb genommenen Anlagen nach Inbetriebnahmedatum. Stilllegungen und Rückbauten sind nicht abgezogen — die tatsächlich aktive Kapazität ist niedriger._

---

## Jährlicher Zubau nach Quelle

```js
Plot.plot({
  ...plotDefaults,
  x: yearAxis,
  y: mwAxis,
  color: { legend: true },
  marks: [
    Plot.barY(annualData, {
      x: "Jahr", y: "Brutto_MW", fill: "Quelle_Label",
      offset: null
    }),
    Plot.tip(annualData, Plot.pointerX(Plot.stackY({
      x: "Jahr", y: "Brutto_MW", fill: "Quelle_Label",
      title: d => tipTitle(d, "Brutto_MW")
    }))),
    Plot.ruleY([0])
  ]
})
```

_Neu installierte Bruttoleistung pro Jahr nach Energieträger. Dies ist der jährliche Zubau, nicht der Gesamtbestand oder die tatsächliche Stromerzeugung._

---

## Installierte Leistung nach Energieträger

```js
Plot.plot({
  ...plotDefaults,
  marginLeft: 100,
  marginRight: 80,
  marks: [
    Plot.barX(mixData, {
      x: "value", y: "name", fill: "name",
      sort: { y: "-x" }
    }),
    Plot.tip(mixData, Plot.pointer({
      x: "value", y: "name",
      title: d => `${d.name}\n${fmtMW.format(d.value)} MW`
    })),
    Plot.text(mixData, {
      x: "value", y: "name",
      text: d => `${fmtMW.format(d.value)} MW`,
      dx: 5, textAnchor: "start"
    }),
    Plot.ruleX([0])
  ],
  x: { ...mwAxis },
  y: { label: null },
  color: { legend: false }
})
```

_Anteil der aktuell als „In Betrieb" gemeldeten Bruttoleistung je Energieträger. Die installierte Leistung sagt nichts über die tatsächlich erzeugte Strommenge aus — z.B. liefert Solar nur bei Sonnenschein._

---

## Kapazität nach Bundesland

```js
Plot.plot({
  ...plotDefaults,
  marginLeft: 160,
  x: { ...mwAxis },
  y: { label: null },
  color: { legend: true },
  marks: [
    Plot.barX(bundeslandData, Plot.stackX({
      x: "Kapazitaet_MW", y: "Bundesland", fill: "Quelle_Label",
      sort: { y: "-x", reduce: "sum" },
      order: "-sum"
    })),
    Plot.tip(bundeslandData, Plot.pointer(Plot.stackX({
      x: "Kapazitaet_MW", y: "Bundesland", fill: "Quelle_Label",
      sort: { y: "-x", reduce: "sum" },
      order: "-sum",
      title: d => tipTitle(d, "Kapazitaet_MW")
    }))),
    Plot.ruleX([0])
  ]
})
```

_Aktuell als „In Betrieb" gemeldete Bruttoleistung je Bundesland und Energieträger._

---

## Zubau letzte 12 Monate

```js
Inputs.table(recentData.map(d => ({
  Quelle: d.Quelle_Label,
  "Kapazität (MW)": fmtMW.format(d.Kapazitaet_MW),
  "Anzahl Einheiten": fmtMW.format(d.Anzahl)
})), { select: false })
```

_Anlagen mit Inbetriebnahmedatum in den letzten 12 Monaten und Status „In Betrieb". Bruttoleistung (Nennleistung), nicht tatsächliche Einspeisung._
