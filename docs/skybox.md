# Skybox

Ja, een “3D skybox” kan een goede fit zijn, maar vooral als je zones redelijk discreet zijn (laadscherm, tunnel,
bergpas) of als je accepteert dat de horizon “omspringt” bij een zonewissel. Voor een WoW-achtige wereld met veel zones
en (semi-)naadloze overgangen is een pure 3D-skybox aanpak vaak niet de eenvoudigste basis; een hybride werkt doorgaans
beter.

Een 3D skybox is sterk wanneer je per zone een eigen skyline/horizonprops wil met echte parallax (bergen, steden,
schepen, enorme bomen) zonder dat je die wereld op echte schaal hoeft te bouwen. Je kunt het extreem goedkoop houden met
low-poly far-meshes, agressieve fog en simpele materialen. Het nadeel is logica-complexiteit: je moet per zone de “far
scene” streamen, positioneren, en consistent houden met zonstand/fog/kleurbalans. Naadloze overgangen zijn lastig:
parallax-werelden die ineens anders worden voelen snel “teleporty”, en als je de far-geometry echt wereldvast maakt, kom
je weer in precisie/clipping/LOD-problemen terecht.

Wat meestal het meest logisch schaalt voor veel zones:

1. Sky als systeem (niet als asset): één dynamische sky (procedural of dome/cubemap) die time-of-day, zon/maan, sterren,
   wolken en atmosferische scattering regelt. Dit blijft wereldwijd consistent.

2. Zone-“look” als parameters: per zone een set parameters (fog color/density/height, horizon tint, cloud coverage,
   ambient/directional intensiteit, LUT/tonemap tweaks). Daarmee krijg je duidelijke zone-sfeer zonder asset-wissel die
   hard knipt.

3. Distant scenery als aparte laag: per zone een far-mesh set (of impostors) die alleen de horizon verkoopt. Die layer
   render je bij voorkeur met:

   * eigen camera/pass (of minstens eigen depth range) om z-fighting en clip issues te vermijden,
   * geen of minimale schaduwen,
   * sterke fog/atmospheric fade zodat de “join” nooit scherp zichtbaar is,
   * zeer agressieve LOD/triangle budgets.

4. Overgangen: blend over afstand/tijd tussen zone-parameter-sets en between far-mesh sets (cross-fade, dither fade, of
   “gate” achter occluders zoals bossen/bergen).

Wanneer een klassieke 3D-skybox (Source-stijl miniatuurwereld met schaalfactor en parallax) wél de beste keuze is: zones
met duidelijke portals, veel “set dressing” buiten de speelruimte (stad-silhouetten, megastructuren), en een art
direction die die truc accepteert. Wanneer ik hem zou vermijden: volledig open, vlak terrein met lange zichtlijnen en
continue traversal zonder natuurlijke occluders; dan krijg je de meeste winst met een wereldwijde sky + far-mesh layer
die wereldvast is (met origin shifting en goede LOD) in plaats van een geschaalde “achterwereld”.

Als je doel “logisch, uitbreidbaar, veel zones” is: bouw eerst het sky-systeem + zone-parameters + far-layer pipeline.
Daarna kun je per zone beslissen of de far-layer een simpele ring van meshes is, of een echte “3D skybox scene” met
eigen set dressing. Dat houdt het ontwerp consistent en voorkomt dat je hele wereld afhankelijk wordt van één truc.
