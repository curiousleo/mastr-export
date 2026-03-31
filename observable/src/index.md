---
title: Deutschland Übersicht
toc: true
---

# Energiewende Dashboard

Daten aus dem [Marktstammdatenregister (MaStR)](https://www.marktstammdatenregister.de/) der Bundesnetzagentur. Alle Angaben beziehen sich auf die installierte Bruttoleistung (Nennleistung) laut Registermeldung — nicht auf die tatsächliche Einspeisung oder den aktuellen Betriebszustand.

```js
const fmtMW = new Intl.NumberFormat("de-DE", { maximumFractionDigits: 0 });
const renewableQuellen = new Set(["Solar", "Wind", "Biomasse", "Wasser", "GeothermieGrubengasDruckentspannung"]);

const monthly = FileAttachment("data/monthly_cumulative.json").json();
const speicherMonthly = FileAttachment("data/speicher_monthly.json").json();
const additions12m = FileAttachment("data/additions_12m.json").json();
const byYear = FileAttachment("data/einheiten_by_year.json").json();
const operating = FileAttachment("data/einheiten_operating.json").json();
const byBundesland = FileAttachment("data/einheiten_by_bundesland.json").json();
const rolling12m = FileAttachment("data/rolling_12m.json").json();
const stilllegungen = FileAttachment("data/stilllegungen_by_year.json").json();
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

const quelleColor = new Map([
  ["Solar", "#FFB800"],
  ["Wind", "#2563EB"],
  ["Biomasse", "#16a34a"],
  ["Wasser", "#06b6d4"],
  ["Geothermie u.a.", "#8b5cf6"],
  ["Verbrennung", "#ef4444"],
  ["Kernkraft", "#f97316"]
]);
const colorScale = { domain: [...quelleColor.keys()], range: [...quelleColor.values()] };

const plotDefaults = { width: 928, height: 500, marginLeft: 60, color: colorScale };
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

const mixData = operating.map(d => ({ Quelle_Label: d.Quelle_Label, value: +d.Kapazitaet_MW }));

const bundeslandData = byBundesland
  .map(d => ({ ...d, Kapazitaet_MW: +d.Kapazitaet_MW }));

const recentData = additions12m.filter(d => renewableQuellen.has(d.Quelle));

const stilllegungenClean = stilllegungen
  .map(d => ({ ...d, Jahr: +d.Jahr, Brutto_MW: +d.Brutto_MW }));

const cumulativeStilllegungen = [];
for (const quelle of new Set(stilllegungenClean.map(d => d.Quelle_Label))) {
  let cumSum = 0;
  for (const row of stilllegungenClean.filter(d => d.Quelle_Label === quelle).sort((a, b) => a.Jahr - b.Jahr)) {
    cumSum += row.Brutto_MW;
    cumulativeStilllegungen.push({ Jahr: row.Jahr, Quelle_Label: quelle, Kumulativ_MW: Math.round(cumSum) });
  }
}
```

## Zubau letzte 12 Monate

```js
const maxKap = Math.max(...recentData.map(d => d.Kapazitaet_MW));

function miniSparkline(quelleLabel) {
  const data = rolling12m
    .filter(d => d.Quelle_Label === quelleLabel)
    .map(d => ({Monat: new Date(d.Monat), MW: +d.Rolling_MW}));
  if (data.length === 0) return html``;
  const color = quelleColor.get(quelleLabel) ?? "#888";
  return Plot.plot({
    width: 180, height: 28, axis: null, margin: 0,
    marks: [
      Plot.areaY(data, {x: "Monat", y: "MW", fill: color, fillOpacity: 0.3, curve: "basis"}),
      Plot.lineY(data, {x: "Monat", y: "MW", stroke: color, strokeWidth: 1.5, curve: "basis"})
    ]
  });
}
```

```js
{
  const sorted = recentData.sort((a, b) => b.Kapazitaet_MW - a.Kapazitaet_MW);
  const totalMW = sorted.reduce((s, d) => s + d.Kapazitaet_MW, 0);
  const totalAnzahl = sorted.reduce((s, d) => s + +d.Anzahl, 0);

  const rows = sorted.map(d => {
    const pct = (d.Kapazitaet_MW / maxKap) * 100;
    const color = quelleColor.get(d.Quelle_Label) ?? "#888";
    const barBg = color + "33";
    return html`<tr style="border-bottom: 1px solid var(--theme-foreground-faintest, #eee);">
      <td style="padding: 8px; font-weight: 500;">${d.Quelle_Label}</td>
      <td style="padding: 8px;">${miniSparkline(d.Quelle_Label)}</td>
      <td style="padding: 8px; text-align: right; position: relative;">
        <div style=${{position: "absolute", inset: "4px 0", width: `${pct}%`, background: barBg, borderRadius: "3px"}}></div>
        <span style="position: relative;">${fmtMW.format(d.Kapazitaet_MW)}</span>
      </td>
      <td style="padding: 8px; text-align: right;">${fmtMW.format(d.Anzahl)}</td>
    </tr>`;
  });

  display(html`<table style="width: 100%; border-collapse: collapse; font-variant-numeric: tabular-nums;">
    <thead>
      <tr style="border-bottom: 2px solid var(--theme-foreground-faintest, #ddd); text-align: left;">
        <th style="padding: 8px;">Quelle</th>
        <th style="padding: 8px; width: 200px;">Kapazität Zubau/Jahr seit 1990</th>
        <th style="padding: 8px; text-align: right;">Kapazität (MW)</th>
        <th style="padding: 8px; text-align: right;">Anzahl</th>
      </tr>
    </thead>
    <tbody>
      ${rows}
      <tr style="border-top: 2px solid var(--theme-foreground-faintest, #ddd); font-weight: 600;">
        <td style="padding: 8px;">Gesamt</td>
        <td style="padding: 8px;"></td>
        <td style="padding: 8px; text-align: right;">${fmtMW.format(totalMW)}</td>
        <td style="padding: 8px; text-align: right;">${fmtMW.format(totalAnzahl)}</td>
      </tr>
    </tbody>
  </table>`);
}
```

_Anlagen mit Inbetriebnahmedatum in den letzten 12 Monaten und Status „In Betrieb”. Bruttoleistung (Nennleistung), nicht tatsächliche Einspeisung._

---

## Kumulierte installierte Leistung

```js
Plot.plot({
  ...plotDefaults,
  x: yearAxis,
  y: mwAxis,
  color: { ...colorScale, legend: true },
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
    Plot.areaY(cumulativeStilllegungen.map(d => ({...d, Kumulativ_MW: -d.Kumulativ_MW})), {
      x: "Jahr", y: "Kumulativ_MW", fill: "Quelle_Label",
      curve: "basis", order: "sum", fillOpacity: 0.5
    }),
    Plot.tip(cumulativeStilllegungen.map(d => ({...d, Kumulativ_MW: -d.Kumulativ_MW})), Plot.pointerX(Plot.stackY({
      x: "Jahr", y: "Kumulativ_MW", fill: "Quelle_Label",
      order: "sum",
      title: d => `${d.Quelle_Label} (Stilllegung)\n${fmtMW.format(-d.Kumulativ_MW)} MW`
    }))),
    Plot.ruleY([0])
  ]
})
```

_Obere Fläche: Summe aller jemals in Betrieb genommenen Anlagen (Bruttoleistung). Untere Fläche (negativ): kumulierte endgültige Stilllegungen._

---

## Jährlicher Zubau nach Quelle

```js
Plot.plot({
  ...plotDefaults,
  x: yearAxis,
  y: mwAxis,
  color: { ...colorScale, legend: true },
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
      x: "value", y: "Quelle_Label", fill: "Quelle_Label",
      sort: { y: "-x" }
    }),
    Plot.tip(mixData, Plot.pointer({
      x: "value", y: "Quelle_Label",
      title: d => `${d.Quelle_Label}\n${fmtMW.format(d.value)} MW`
    })),
    Plot.text(mixData, {
      x: "value", y: "Quelle_Label",
      text: d => `${fmtMW.format(d.value)} MW`,
      dx: 5, textAnchor: "start"
    }),
    Plot.ruleX([0])
  ],
  x: { ...mwAxis },
  y: { label: null },
  color: { ...colorScale, legend: false }
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
  color: { ...colorScale, legend: true },
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

_Aktuell als „In Betrieb” gemeldete Bruttoleistung je Bundesland und Energieträger._
