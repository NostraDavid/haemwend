# Blender AI Agent Guide

Deze map bevat script-gedreven assetgeneratie voor Blender.
Gebruik deze richtlijnen als stijlcontract voor nieuwe assets (`table.py`, later `tree.py`, `rock.py`, `character.py`, ...).

## Doel

Genereer stylized, game-readable assets (in de geest van klassieke WoW-readability) in plaats van technisch-realistische of orthogonale vormen.

## Stijlregels

1. Vermijd boxy vormen.
2. Leesbaarheid op afstand gaat voor detail.
3. Exaggeratie boven realisme.
4. Lage-frequentie detail boven micro-ruis.
5. Laat textuur vorm ondersteunen (painted light/shadow waar nodig).
6. Gebruik verzadigde, contrastrijke palettes met duidelijke focuspunten.

## Praktische heuristieken voor scripts

1. Vermijd dominante 90 graden silhouetten; voeg tilt, taper, splay of lichte warping toe.
2. Houd proporties duidelijk en overdreven (dikkere primaire vormen, minder kleine losse onderdelen).
3. Beperk kleine details; geef voorkeur aan enkele grote vormaccenten.
4. Valideer output met:
   - geometriechecks (`--validate`)
   - artifact renders uit vaste camerahoeken
   - afstandsleesbaarheid (silhouette/contrast in kleinere preview)

## Workflow

1. Wijzig script in `assets/blender_ai/*.py`.
2. Draai:
   - `just blender-table-validate`
   - `just blender-table-artifacts`
3. Beoordeel render + report in `assets/blender_ai/_artifacts/`.
4. Itereer tot `VALIDATION: PASS` en silhouette/readability visueel klopt.

## Bronnen

- [Massively Overpowered: *Casually Classic: How World of Warcraft made bad graphics look good*](https://massivelyop.com/2021/03/23/casually-classic-how-world-of-warcraft-made-bad-graphics-look-good/)
- [80 Level: *Mastering The Stylized Art of Blizzard*](https://80.lv/articles/matt-mcdaid-mastering-the-stylized-art)
- [WoW Art Style Guide (user-uploaded kopie)](https://www.scribd.com/document/342336308/Concept-Art)
