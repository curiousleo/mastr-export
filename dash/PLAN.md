# MaStR Dashboard Content Plan

## Context

The Marktstammdatenregister (MaStR) data has been ingested into ClickHouse. The goal is to build a public-facing website with dashboards that are useful on their own and demonstrate the dataset's possibilities (API/DB access will be offered alongside).

Key audiences:

- Researchers with a focus on the energy transition and the energy grid
- Municipal energy managers who want to compare their municipality to peers
- Interested citizens and stakeholders

The dashboards themselves should be entirely in German.

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
- Net new capacity added in last 12 months (GW, all renewables)

Each card shows the headline number with a **sparkline** (small inline area chart) in the background showing the 12-month trend. This sparkline pattern applies to hero KPIs on all pages where it makes sense.

### Charts
1. **Cumulative Installed Capacity Over Time** (stacked area) — The Energiewende chart. X=year (~1990–present), Y=cumulative GW. Stacks: Solar, Wind Onshore, Wind Offshore, Biomass, Hydro, Storage. Must account for decommissioning: units can be "vorübergehend stillgelegt" (temporarily decommissioned) or "endgültig stillgelegt" (permanently decommissioned). Temporarily offline units should not count as active capacity. Reconstructing historical status at each point in time requires careful exploration of which date columns are populated — this needs query-level investigation against the live data.

2. **Annual Net Additions by Source** (grouped bar) — X=year, Y=MW added net of decommissioning, grouped by source. Shows momentum/stalls (e.g. wind plateau 2019–2020).

3. **National Map** (clustered point map) — All units with lat/lon, color-coded by source (solar=yellow, wind=blue, biomass=green, hydro=cyan, conventional=gray, nuclear=red, storage=purple). Clusters at low zoom, individual points at high zoom. Toggle sources, filter by status/date. Immediately shows solar-south/wind-north pattern.

4. **Capacity by Bundesland** (choropleth + horizontal stacked bar) — Choropleth map of renewable capacity per state. Horizontal bars below it stacked by source. Click to drill into state profile.

5. **Current Energy Mix** (donut) — Share of each source in total operating capacity across all Einheiten tables.

6. **Recent Activity** — Units commissioned and MW added by source. The data is updated daily but lags behind reality; the reporting window needs to be calibrated against the live data (e.g. "last full calendar month" or "last 30 days, offset by one week") to avoid showing incomplete periods.

---

## Page 2: Solar Deep Dive

### Hero KPIs
- Total solar capacity (GW)
- Number of installations (count)
- Average system size (kW)
- Rooftop vs ground-mounted split (GW each)
- Last 12 months additions (count + MW, sparkline showing monthly MW)

### Page-wide filter
A single **date range filter** applies to all charts on the page. For time series charts it controls the x-axis range; for snapshot charts (histogram, map, bar) it filters to units commissioned within the range.

### Charts
1. **Annual Installations** (dual-axis) — Bars: count of installations (stacked by ArtDerSolaranlage rooftop/ground). Line: total MW. Shows average system size rising over time.

2. **System Size Distribution** (histogram) — Buckets: 0–5 kW, 5–10, 10–30, 30–100, 100–750, 750+ kW. Shows units commissioned within the page-wide date range.

3. **Solar Map** (heatmap at national zoom, dots at high zoom) — Color intensity by capacity density.

4. **Solar by Bundesland** (stacked bar) — Sorted by total, stacked rooftop/ground.

5. **Panel Orientation** (polar/rose chart) — Distribution by Hauptausrichtung (compass direction). Each year in the selected date range is overlaid as a semi-transparent layer with a sequential color scale, showing how orientation patterns shift over time (e.g. the growth of east-west installations). If the range spans many years, auto-bucketing into multi-year bands could improve legibility.

6. **Monthly Commissioning Pace** (line) — One line per year for direct comparison. Annotation line for government target if applicable.

7. **Co-located Storage Trend** — Fraction of new solar paired with storage (SpeicherAmGleichenOrt) over time.

---

## Page 3: Wind Deep Dive

### Hero KPIs
- Total wind capacity (GW, onshore/offshore split)
- Number of turbines (count)
- Average turbine size (MW)
- Average hub height (m)
- Average rotor diameter (m)

### Page-wide filter
A single **date range filter** applies to all charts on the page. For time series charts (1, 3, 4 stacked area) it controls the x-axis range. For snapshot charts (2, 4 treemap, 5, 6, 8) it filters to units commissioned within the range.

### Charts
1. **Cumulative Wind Capacity** (stacked area, onshore vs offshore) — X-axis range controlled by filter. Stacks show cumulative totals (not just units from the range).

2. **Turbine Technology Evolution** (scatter) — X=year, Y=capacity per turbine (MW), bubble size=rotor diameter, color=onshore/offshore. Only plots turbines commissioned within the date range. Shows dramatic growth from <1 MW to 5–15 MW.

3. **Hub Height & Rotor Diameter Trends** (dual line) — Average per year of commissioning, onshore only. X-axis range controlled by filter.

4. **Manufacturer Market Share** (stacked area + treemap) — Stacked area: x-axis range controlled by filter. Treemap: shows only fleet commissioned within the range ("all time" = full fleet, narrow range = recent orders). Key players: Enercon, Vestas, Nordex, Siemens Gamesa.

5. **Wind Map** — Clustered at low zoom levels, individual turbine dots at high zoom. Color by status, size by capacity. Shows only turbines commissioned within the date range.

6. **Wind by Bundesland** (bar, split onshore/offshore) — Counts only turbines commissioned within the date range.

7. **Repowering Analysis** *(enhancement, tackle later)* — Decommissioned turbines by year/location, overlaid with new installations in same areas as a repowering proxy. Requires a location-matching heuristic (same Gemeinde or coordinate radius + time window) since the data has no explicit "replaces" link.

8. **Curtailment Constraints** (bar) — Fraction of turbines with each constraint type (noise, wildlife, shadow, ice). Filters to turbines commissioned within the date range.

---

## Page 4: Biomass Deep Dive

### Hero KPIs
- Total biomass capacity (GW)
- Number of installations (count)
- Average plant size (kW)
- Last 12 months additions (count + MW)

### Page-wide filter
A single **date range filter** applies to all charts. Time series (1): x-axis range. Snapshot charts (2, 3, 4, 5, 6): filter to units commissioned within the range.

### Charts
1. **Capacity Over Time** (stacked area by Biomasseart) — Shows EEG-driven boom 2000–2014 and plateau.
2. **Fuel Type Breakdown** (treemap) — By Hauptbrennstoff, sized by capacity.
3. **Technology Distribution** (donut) — CHP vs steam turbine vs gas engine etc.
4. **Biomass Map** — Clustered at low zoom, individual dots at high zoom. Color by Biomasseart. Agricultural region concentration visible.
5. **Biomass by Bundesland** (bar) — Sorted by total capacity.
6. **Age Profile** (histogram by commissioning year) — Wave pattern from EEG subsidy rounds.

---

## Page 5: Hydro Deep Dive

### Hero KPIs
- Total hydro capacity (GW)
- Number of plants (count)
- Average plant size (MW)
- Oldest plant (year)

### Page-wide filter
A single **date range filter** applies to all charts for consistency, though most hydro plants are very old and there is little new construction.

### Charts
1. **By Type** (horizontal bar) — ArtDerWasserkraftanlage: run-of-river, reservoir, etc. Sized by capacity.
2. **Hydro Map** — Clustered at low zoom, individual markers at high zoom. Sized by capacity. Concentrated along Rhine, Danube, alpine rivers.
3. **Hydro by Bundesland** (bar) — Sorted by total capacity. Shows Bavaria/Baden-Württemberg dominance.
4. **Age Profile** — Oldest infrastructure in the dataset; many plants pre-war. Histogram by decade.

---

## Page 6: Conventional & Nuclear

**Purpose**: The "other side" of the Energiewende — what's being phased out. No page-wide date filter — this page is about the full history and current status of the conventional fleet.

### Hero KPIs
- Total conventional capacity operating (GW)
- Total conventional capacity decommissioned (GW)
- Total capacity in reserve (GW)
- Nuclear capacity (GW) — all shut down since April 2023

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
- Total battery storage capacity (GW)
- Number of battery installations (count)
- Total pumped hydro capacity (GW)
- Home battery share (% of count)
- Solar co-location share (% of count)

### Page-wide filter
A single **date range filter** applies to all charts. Especially useful here since storage is the fastest-changing sector. Time series (1, 3): x-axis range. Snapshot charts (2, 4, 5, 6): filter to units commissioned within the range.

### Charts
1. **Battery Growth** (area) — Exponential curve, split by Einsatzort (residential/commercial/grid-scale) or Batterietechnologie.
2. **Size Distribution** (histogram) — <5 kW home, 5–30 commercial, 30–100, 100–1000, >1000 kW.
3. **Solar Co-location Trend** — Fraction of new storage with GemeinsamRegistrierteSolareinheitMastrNummer over time.
4. **Storage Map** — Home batteries: heatmap (too dense for individual dots). Grid-scale: individual markers with capacity labels.
5. **Technology Mix** (donut) — Batterietechnologie (Li-ion dominant, plus lead-acid, redox flow, etc.).
6. **Storage by Bundesland** (bar) — Sorted by total capacity.

---

## Page 8: Bundesland Profile (`/bundesland/:id`)

**Purpose**: Everything about one state's energy infrastructure, compared to national average.

### Hero KPIs
- Total installed capacity (GW)
- Renewable share (%)
- Solar capacity (GW)
- Wind capacity (GW)
- Storage capacity (MW)
- Number of units (count)

### Charts
1. **Energy Mix** (donut, side-by-side with national average)
2. **Capacity Growth** (stacked area, with national growth rate as reference line)
3. **State Map** (all units, zoomed to state, clustered at low zoom, color by source)
4. **Top Municipalities Table** (sortable by total/solar/wind/storage capacity, click to drill into Gemeinde profile)
5. **Landkreis Comparison** (choropleth of districts within the state) — by absolute renewable capacity, or by capacity density (MW/km²) if area data from Destatis is available. Toggle between both.
6. **Source-Specific Stats** (compact stat cards) — Solar: capacity, rooftop/ground split, avg system size. Wind: capacity, count, avg turbine size, top manufacturers. Biomass: capacity, fuel types. Conventional: capacity, fuel breakdown.

---

## Page 9: Gemeinde Profile (`/gemeinde/:ags`)

**Purpose**: The page for a municipality energy manager. "Here is everything about your Gemeinde."

### Hero KPIs
- Total installed capacity (MW)
- Solar capacity (MW)
- Wind capacity (MW)
- Storage capacity (kW)
- Number of units (count)

### Charts
1. **Local Energy Mix** (donut)
2. **Capacity Timeline** (stacked area over time for this Gemeinde)
3. **Unit Table** (sortable, paginated) — All units: ID, type, capacity, status, commissioning date, operator name (joined from Marktakteure). Searchable and exportable as CSV.
4. **Local Map** — All units, zoomed to municipality.
5. **Comparison Strip** (bar) — This Gemeinde vs Landkreis avg vs Bundesland avg vs national avg. Metrics: total capacity, solar, wind, storage. Per-capita if population data available. Also show **percentile rank** (e.g. "73rd percentile for solar in your Bundesland").
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
1. **Ranking Bar Chart** — Horizontal bars per municipality sorted by selected metric. Reference highlighted. Shows rank: "#X of Y in [comparison set]".
2. **Scatter Plot** — Each axis independently selectable from the metric list (e.g. solar capacity vs wind capacity, or total capacity vs growth rate). Reference municipality highlighted. Reveals patterns like "lots of solar, no wind".
3. **Time Series Comparison** (overlaid lines or small multiples) — Annual additions for selected municipalities.
4. **Mix Comparison** (side-by-side stacked bars) — Capacity breakdown by source per municipality.

If Destatis population data is available, "similar municipalities" (same population bracket) becomes the most natural default comparison mode.

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

2. **Cross-table union view**: Create `Einheiten` VIEW that UNIONs common columns from all Einheiten tables with a `Quelle` (source type) discriminator column.

3. **Pre-aggregation**: Materialized views for daily/monthly/yearly capacity totals at national, state, and municipality level for fast dashboard queries. May not be necessary if real-time queries are fast enough.

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
