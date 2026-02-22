# Prestatiebudget-sjabloon

Gebruik dit sjabloon om meetbare en toetsbare prestatiebudgetten vast te leggen voordat je features bouwt.

## Documentmetadata
- Project: `<projectnaam>`
- Versie: `<v0.1>`
- Eigenaar: `<naam/team>`
- Datum: `<YYYY-MM-DD>`
- Status: `Concept | Goedgekeurd | Verouderd`

## Scope
- Binnen scope: `<spelmodi, platformen, scènes>`
- Buiten scope: `<uitgesloten platformen/features>`
- Doel: `<welke spelerervaring dit budget beschermt>`

## Doelhardware per niveau

| Niveau | Doelapparaat | CPU | GPU | RAM | OS |
|---|---|---|---|---|---|
| Minimum | `<apparaat>` | `<cpu>` | `<gpu>` | `<ram>` | `<os>` |
| Aanbevolen | `<apparaat>` | `<cpu>` | `<gpu>` | `<ram>` | `<os>` |
| High-end | `<apparaat>` | `<cpu>` | `<gpu>` | `<ram>` | `<os>` |

## Prestatie-scenario's

| Scenario-ID | Beschrijving | Duur | Verwachte belasting |
|---|---|---|---|
| S1 | `<lege map / menu>` | `<30s>` | `<laag>` |
| S2 | `<typische gameplay>` | `<60s>` | `<gemiddeld>` |
| S3 | `<stresstest>` | `<60s>` | `<hoog>` |

## Frame-time en FPS-budgetten

| Metriek | Minimum | Aanbevolen | High-end | Opmerking |
|---|---|---|---|---|
| Doel-FPS | `<60>` | `<60/120>` | `<120+>` | |
| Gem. frame-time (ms) | `<16.67>` | `<16.67>` | `<8.33>` | |
| P95 frame-time (ms) | `<20>` | `<16>` | `<10>` | |
| P99 frame-time (ms) | `<25>` | `<20>` | `<12>` | |
| Stotterpercentage (% frames > drempel) | `<1%>` | `<0.5%>` | `<0.2%>` | definieer drempel |

## CPU/GPU-budgetten (per frame)

| Metriek | Budget | Opmerking |
|---|---|---|
| Main thread tijd (ms) | `<...>` | |
| Render thread tijd (ms) | `<...>` | |
| GPU frame-time (ms) | `<...>` | |
| ECS update-tijd (ms) | `<...>` | noem zware systemen |
| Physics-tijd (ms) | `<...>` | |
| AI/gameplay logic (ms) | `<...>` | |

## Content- en renderbudgetten

| Metriek | Budget | Opmerking |
|---|---|---|
| Draw calls | `<...>` | per scenario |
| Zichtbare entities | `<...>` | per scenario |
| Gerenderde triangles | `<...>` | |
| Lights (dynamic) | `<...>` | |
| Shadow casters | `<...>` | |
| Particles/effects-aantal | `<...>` | |

## Geheugenbudgetten

| Metriek | Minimum-budget | Opmerking |
|---|---|---|
| Totaal RAM-gebruik | `<...>` | |
| VRAM-gebruik | `<...>` | |
| Piek transient allocatie | `<...>` | |
| Asset cache-grootte | `<...>` | |

## Laad- en streamingbudgetten

| Metriek | Budget | Opmerking |
|---|---|---|
| Koude start tot hoofdmenu | `<...s>` | |
| Save laadtijd | `<...s>` | |
| Level laadtijd | `<...s>` | |
| Maximale streaming-hitch (ms) | `<...>` | |

## Instrumentatie en meetmethode
- Buildtype: `<release/debug>`
- Meettools: `<Bevy diagnostics, Tracy, RenderDoc, etc.>`
- Meetduur: `<bijv. 60 seconden>`
- Aantal runs: `<bijv. 3>`
- Aggregatiemethode: `<gemiddelde + p95 + p99>`
- Meetcommando/script: `<pad of commando>`

## Regressiepoorten

| Poort | Voorwaarde | Actie |
|---|---|---|
| Waarschuwing | `metriek overschrijdt budget met <5%` | onderzoek in huidige PR |
| Fout | `metriek overschrijdt budget met >=10%` | merge blokkeren |
| Uitzonderingsproces | `<goedkeuring eigenaar + follow-up ticket>` | verplicht |

## Rapportagesjabloon (per PR)
- Benchmarkdatum: `<YYYY-MM-DD>`
- Branch/PR: `<id>`
- Getest hardwareniveau: `<niveau>`
- Scenario(s): `<S1, S2, S3>`
- Voor: `<metingen>`
- Na: `<metingen>`
- Delta: `<+/- waarden>`
- Besluit: `Geslaagd | Waarschuwing | Afgekeurd`
- Vervolgacties: `<tickets>`

## Goedkeuring
- Engineering-eigenaar: `<naam>`
- Tech lead: `<naam>`
- Datum goedgekeurd: `<YYYY-MM-DD>`
