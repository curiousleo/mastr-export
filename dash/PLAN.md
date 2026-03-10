# MaStR Dashboard Content Plan

## Context

The Marktstammdatenregister (MaStR) data has been ingested into ClickHouse. The goal is to build a public-facing website with dashboards that are useful on their own and demonstrate the dataset's possibilities (API/DB access will be offered alongside).

Key audiences:

- Researchers with a focus on the energy transition and the energy grid
- Municipal energy managers who want to compare their municipality to peers
- Interested citizens and stakeholders

Technology choices are deferred — this plan focuses on **what to show**.

---

## Navigation Structure

```
/                        Germany Overview (landing page)
/solar                   Solar Deep Dive
/wind                    Wind Deep Dive
/biomasse                Biomass Deep Dive
/wasser                  Hydro Deep Dive
/konventionell           Conventional & Nuclear
/speicher                Storage Deep Dive
/bundesland/:id          State Profile
/gemeinde/:ags           Municipality Profile
/vergleich               Municipality Comparison Tool
```

---

## Page 1: Germany Overview (Landing Page)

**Purpose**: Show the scale and trajectory of Germany's Energiewende at a glance.

### Hero KPIs (5 cards)
- Total installed renewable capacity (GW)
- Total solar capacity (GW)
- Total wind capacity (GW)
- Total battery storage capacity (GW)
- Net new capacity added in last 12 months (GW)

### Charts
1. **Cumulative Installed Capacity Over Time** (stacked area) — The Energiewende chart. X=year (~1990–present), Y=cumulative GW. Stacks: Solar, Wind Onshore, Wind Offshore, Biomass, Hydro, Storage. Must account for decommissioning (only count units where `Inbetriebnahmedatum <= year_end AND (DatumEndgueltigeStilllegung IS NULL OR > year_end)`).

2. **Annual Net Additions by Source** (grouped bar) — X=year, Y=MW added net of decommissioning, grouped by source. Shows momentum/stalls (e.g. wind plateau 2019–2020).

3. **National Map** (clustered point map) — All units with lat/lon, color-coded by source (solar=yellow, wind=blue, biomass=green, hydro=cyan, conventional=gray, nuclear=red, storage=purple). Clusters at low zoom, individual points at high zoom. Toggle sources, filter by status/date. Immediately shows solar-south/wind-north pattern.

4. **Capacity by Bundesland** (choropleth + horizontal stacked bar) — Choropleth map of renewable capacity per state. Horizontal bars below it stacked by source. Click to drill into state profile.

5. **Current Energy Mix** (donut) — Share of each source in total operating capacity across all Einheiten tables.

6. **Recent Activity** — Stats for last 30 days: units commissioned, MW added, by source.

---

## Page 2: Solar Deep Dive

### Hero KPIs
Total solar GW | Number of installations | Average system size (kW) | Rooftop vs ground-mounted split | Last 12 months additions

### Charts
1. **Annual Installations** (dual-axis) — Bars: count of installations (stacked by ArtDerSolaranlage rooftop/ground). Line: total MW. Shows average system size rising over time.

2. **System Size Distribution** (histogram) — Buckets: 0–5 kW, 5–10, 10–30, 30–100, 100–750, 750+ kW. Filter by year/state/type.

3. **Solar Map** (heatmap at national zoom, dots at high zoom) — Color intensity by capacity density. Filters: rooftop/ground, date range.

4. **Solar by Bundesland** (stacked bar) — Sorted by total, stacked rooftop/ground.

5. **Panel Orientation** (polar/rose chart) — Distribution by Hauptausrichtung (compass direction). Filter by year to see east-west trend.

6. **Monthly Commissioning Pace** (line, last 3 years) — One line per year for direct comparison. Annotation line for government target if applicable.

7. **Co-located Storage Trend** — Fraction of new solar paired with storage (SpeicherAmGleichenOrt) over time.

---

## Page 3: Wind Deep Dive

### Hero KPIs
Total wind GW (onshore/offshore split) | Turbine count | Average turbine size (MW) | Average hub height (m) | Average rotor diameter (m)

### Charts
1. **Cumulative Wind Capacity** (stacked area, onshore vs offshore)

2. **Turbine Technology Evolution** (scatter) — X=year, Y=capacity per turbine (MW), bubble size=rotor diameter, color=onshore/offshore. Shows dramatic growth from <1 MW to 5–15 MW.

3. **Hub Height & Rotor Diameter Trends** (dual line) — Average per year of commissioning, onshore only.

4. **Manufacturer Market Share** (stacked area + treemap) — By year: stacked area of annual installations by Hersteller. Current fleet: treemap. Key players: Enercon, Vestas, Nordex, Siemens Gamesa.

5. **Wind Map** — Individual turbine dots (sparse enough to show). Color by status, size by capacity. Offshore clusters.

6. **Wind by Bundesland** (bar, split onshore/offshore)

7. **Repowering Analysis** — Decommissioned turbines by year/location, overlaid with new installations in same areas as repowering proxy.

8. **Curtailment Constraints** (bar) — Fraction of turbines with each constraint type (noise, wildlife, shadow, ice). Filter by state/year.

---

## Page 4: Biomass Deep Dive

### Hero KPIs
Total capacity | Count | By Biomasseart | By Technologie

### Charts
1. **Capacity Over Time** (stacked area by Biomasseart) — Shows EEG-driven boom 2000–2014 and plateau.
2. **Fuel Type Breakdown** (treemap) — By Hauptbrennstoff, sized by capacity.
3. **Technology Distribution** (donut) — CHP vs steam turbine vs gas engine etc.
4. **Biomass Map** (dots colored by Biomasseart) — Agricultural region concentration.
5. **Age Profile** (histogram by commissioning year) — Wave pattern from EEG subsidy rounds.

---

## Page 5: Hydro Deep Dive

### Hero KPIs
Total capacity | Count | By ArtDerWasserkraftanlage

### Charts
1. **By Type** (horizontal bar) — Run-of-river, reservoir, etc.
2. **Map** — Individual markers sized by capacity, along river systems.
3. **Age Profile** — Oldest infrastructure; many plants pre-war. Histogram by decade.

---

## Page 6: Conventional & Nuclear

**Purpose**: The "other side" of the Energiewende — what's being phased out.

### Hero KPIs
Total conventional GW | Decommissioned conventional GW | Nuclear: all shut down

### Charts
1. **Fleet by Fuel** (stacked bar) — Energietraeger (coal, gas, oil, waste), stacked by status (operating/reserve/decommissioned).
2. **Nuclear Timeline** (Gantt chart) — All ~17 reactor blocks, horizontal bars from commissioning to decommissioning. Labels: NameKraftwerk + NameKraftwerksblock. Color by technology (PWR/BWR).
3. **Coal Phase-Out Tracker** — Hard coal + lignite: operating vs reserve vs decommissioned over time.
4. **Conventional Plant Map** — Large markers with plant names, color by fuel, size by capacity, status toggle.
5. **Gas Fleet Analysis** — New vs old gas plants, capacity by Technologie (CCGT, OCGT, gas engine).
6. **Reserve Status Table** — Units in Netzreserve or Kapazitaetsreserve, with plant names, capacity, dates.

---

## Page 7: Storage Deep Dive

### Hero KPIs
Total battery storage GW + count | Pumped hydro GW | Home battery % | Solar co-location %

### Charts
1. **Battery Growth** (area) — Exponential curve, split by Einsatzort (residential/commercial/grid-scale) or Batterietechnologie.
2. **Size Distribution** (histogram) — <5 kW home, 5–30 commercial, 30–100, 100–1000, >1000 kW.
3. **Solar Co-location Trend** — Fraction of new storage with GemeinsamRegistrierteSolareinheitMastrNummer over time.
4. **Storage Map** — Home batteries: heatmap. Grid-scale: individual markers.
5. **Technology Mix** (donut) — Batterietechnologie (Li-ion dominant, plus lead-acid, redox flow, etc.).

---

## Page 8: Bundesland Profile (`/bundesland/:id`)

**Purpose**: Everything about one state's energy infrastructure, compared to national average.

### Charts
1. **Energy Mix** (donut, side-by-side with national average)
2. **Capacity Growth** (stacked area, with national growth rate as reference line)
3. **State Map** (all units, zoomed to state, color by source)
4. **Top Municipalities Table** (sortable by total/solar/wind/storage capacity, click to drill into Gemeinde profile)
5. **Landkreis Comparison** (choropleth of districts within the state by renewable capacity)
6. **Source-Specific Stats** — Solar breakdown (rooftop/ground, avg size), Wind (count, avg turbine size, top manufacturers), Biomass (fuel types), Conventional (fuel breakdown)

---

## Page 9: Gemeinde Profile (`/gemeinde/:ags`)

**Purpose**: The page for a municipality energy manager. "Here is everything about your Gemeinde."

### Charts
1. **Local Energy Mix** (donut)
2. **Capacity Timeline** (stacked area over time for this Gemeinde)
3. **Unit Table** (sortable, paginated) — All units: ID, type, capacity, status, commissioning date, operator name (joined from Marktakteure). Searchable and exportable.
4. **Local Map** — All units, zoomed to municipality.
5. **Comparison Strip** (bar) — This Gemeinde vs Landkreis avg vs Bundesland avg vs national avg. Metrics: total capacity, solar, wind, storage. Per-capita if population data available.
6. **Operator Concentration** — Top operators by capacity (Marktakteure join).
7. **Recent & Planned** — Units commissioned last 12 months + units with GeplantesInbetriebnahmedatum in the future.

---

## Page 10: Municipality Comparison Tool (`/vergleich`)

**Purpose**: Key differentiator for the energy manager audience.

### Controls
1. **Select reference municipality** (search by name or Gemeindeschluessel)
2. **Select comparison set**: Same Landkreis | Same Bundesland | Custom selection | "Similar municipalities" (same capacity bracket — or same population bracket if external data added)
3. **Select metric**: Total capacity | Solar capacity | Wind capacity | Storage capacity | Number of installations | Avg solar system size | Growth rate (last 3 years)

### Charts
1. **Ranking Bar Chart** — Horizontal bars per municipality sorted by metric. Reference highlighted. Shows rank: "#X of Y in [comparison set]".
2. **Scatter Plot** — X=one metric, Y=another (e.g. solar vs wind capacity). Reference highlighted. Reveals "lots of solar, no wind" patterns.
3. **Time Series Comparison** (overlaid lines or small multiples) — Annual additions for selected municipalities.
4. **Mix Comparison** (side-by-side stacked bars) — Capacity breakdown by source per municipality.

---

## External Data Recommendation

The MaStR does NOT contain population or land area. **Strongly recommend adding a reference table** keyed on Gemeindeschluessel (8-digit AGS) with:
- Population (Einwohner) — from Destatis, updated annually, freely available
- Area (km2) — from Destatis Gemeindeverzeichnis
- Municipality type (Stadt/Landgemeinde)

This enables the most meaningful comparisons:
- Solar kW per capita
- Wind MW per km2
- "Compare me to other rural municipalities of similar size"

---

## Technical Notes for Query Layer

1. **Katalogwerte resolution**: Most fields (Bundesland, EinheitBetriebsstatus, ArtDerSolaranlage, Energietraeger, Hersteller, etc.) are numeric IDs requiring JOIN to Katalogwerte. Recommend ClickHouse Dictionary or materialized views with resolved labels.

2. **Cross-table union view**: Create `alle_einheiten` VIEW that UNIONs common columns from all Einheiten tables with a `quelle` (source type) discriminator column.

3. **Pre-aggregation**: Materialized views for daily/monthly/yearly capacity totals at national, state, and municipality level for fast dashboard queries.

---

## Implementation Priority

1. Germany Overview — highest impact landing page
2. Solar Deep Dive — largest dataset, most public interest
3. Wind Deep Dive — rich metadata, second largest
4. Municipality Comparison Tool — key differentiator
5. Bundesland Profile — natural drill-down from overview
6. Gemeinde Profile — natural drill-down from state
7. Storage Deep Dive — fastest-growing, high interest
8. Conventional & Nuclear — important Energiewende context
9. Biomass Deep Dive — niche audience
10. Hydro Deep Dive — smallest, mostly historical
