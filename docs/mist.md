# mist

Klassieke (niet-volumetric) game-mist komt neer op: bereken een mist-factor (f) uit afstand en eventueel hoogte, en blend
daarna de scène-kleur naar een mist-kleur (in de object-shader of als post-process op basis van de depth-buffer). De
belangrijkste technieken zijn:

1. Afstands-mist (depth mist) in de shader (per-pixel) of fixed-function/legacy (per-vertex). Je kiest een mode (Linear /
   Exp / Exp2) en een afstandsmaat (view-space (z) of echte range). ([Microsoft Learn][1])
2. Screen-space mist als post-process (depth-buffer → (f) → lerp). Dit is de “moderne klassieke” variant in
   forward/deferred pipelines, maar transparante objecten en particles vereisen vaak aparte handling omdat zij niet
   (goed) in de depth zitten. ([catlikecoding.com][2])
3. Height mist (hoogte-afhankelijkheid bovenop afstand). Dit blijft non-volumetric zolang je alleen ($f(d,y)$) gebruikt
   en geen volumetric lighting/raymarching. ([Epic Games Developers][3])
4. Lokale mist “in klassiek jasje”: mist cards/particles of simpele volumes die je als translucency rendert (geen
   volumetric integratie, wel lokale mistbanken). Dit is vooral een content/artistieke techniek, maar het werkt goed
   naast (1)–(3).

Variabelen (de “knoppen”) die je bijna altijd wilt hebben:

A. Mist-factor functie (vorm van de curve)

* Mode: Linear, Exponential (Exp), Exponential Squared (Exp2). ([Microsoft Learn][1])
* Parameters:

  * Linear: start-distance, end-distance. ([Microsoft Learn][4])
  * Exp/Exp2: density. ([Microsoft Learn][1])
* Afstandsmaat: view-space ($z$) (goedkoop, “vlak”) versus echte euclidische afstand (range-based; “sferischer”, maar
  traditioneel vooral bij vertex mist). ([Microsoft Learn][4])
* Clamp/max distance: waar mist volledig “dicht” mag worden.

B. Kleur en artistieke sturing

* Mist color: constant, of een gradient (near→far), of hemisferisch (kleur richting zon vs tegen-zon), of gekoppeld aan
  sky/horizon. ([Epic Games Developers][3])
* Mist start offset: “clear bubble” rond de camera (handig in third-person).
* Exposure/tonemapping order: mist vóór tonemapping (HDR) versus erna (LDR) verandert de perceptie van dichtheid; dit is
  in de praktijk een belangrijke creatieve knop.

C. Height mist variabelen (als je hoogte meeneemt)

* Base height (referentiehoogte), height falloff (hoe snel density verandert met (y)), en aparte density voor height
  component. ([Epic Games Developers][3])

D. Detail (zonder volumetrics)

* Noise op density (scale, strength, scroll/wind): breekt banding en maakt mist “levend”.
* Layering: ground mist (height) + distance mist (depth) met afzonderlijke curves/kleuren.

Handige standaardformules (klassiek en robuust):

* Linear: ($f=\mathrm{clamp}\big(\frac{end-d}{end-start},0,1\big)$)
* Exp: ($f=\exp(-\text{density}\cdot d)$)
* Exp2: ($f=\exp(-(\text{density}\cdot d)^2)$) en dan: ($\text{color}=\mathrm{lerp}(\text{fogColor}, \text{sceneColor},
  f)$). ([Microsoft Learn][1])

Als je “artist-friendly” parameters wilt: definieer een zichtafstand ($V$) waarbij je nog ($t$) transmittance over wilt
houden (bijv. ($t=0{.}02$)), en zet ($\text{density}=-\ln(t)/V$). Daarmee wordt “visibility distance” je primaire
slider, en density volgt automatisch. ([Microsoft Learn][1])

[1]: https://learn.microsoft.com/en-us/windows/win32/opengl/glfogf?utm_source=chatgpt.com "glFogf function (Gl.h) -
    Win32 apps"
[2]: https://catlikecoding.com/unity/tutorials/rendering/part-14/?utm_source=chatgpt.com "Rendering 14, Fog, a Unity
    Tutorial"
[3]:
    https://dev.epicgames.com/documentation/en-us/unreal-engine/exponential-height-fog-in-unreal-engine?utm_source=chatgpt.com
    "Exponential Height Fog in Unreal Engine"
[4]: https://learn.microsoft.com/en-us/windows/win32/direct3d9/fog-formulas?utm_source=chatgpt.com "Fog Formulas
    (Direct3D 9) - Win32 apps"

---

Voor een “werkende” implementatie wel, maar voor een correcte en robuuste implementatie (die niet stuk gaat bij andere
camera-instellingen, reversed-Z, HDR/tonemapping, transparantie, enzovoort) mis je nog een paar expliciete
specificaties.

Wat je al hebt is genoeg om een LLM een basismist te laten bouwen:

* Fog-curve (linear/exp/exp2) + parameters.
* Blend: `lerp(fogColor, sceneColor, f)`.

Wat je nog expliciet moet meegeven om “correct” te krijgen in een moderne pipeline (post-process op depth):

1. Welke afstand (d) exact is

   * Gebruik je view-space (z) (lineaire diepte langs kijkrichting) of echte range (|p_{view}|)?
   * Voor klassieke mist is view-space (z) meestal prima en stabiel.

2. Hoe je depth lineariseert / view-space reconstrueert Je moet vastleggen welke variant de implementatie gebruikt,
   anders gaat het vaak mis:

   * Optie A (aanrader): reconstrueer view-space positie uit depth met `invProj` of `invViewProj`, en neem daarna (z) of
     range.
   * Optie B: gebruik een “linear depth” formule op basis van near/far en projectiematrix-conventie. Daarbij moet je
     expliciet noemen:
   * Depth range (0..1 of -1..1).
   * Reversed-Z aan/uit.
   * Oneindige far-plane mogelijk ja/nee.

3. In welke kleurruimte je blendt

   * Blend in lineaire ruimte (HDR/linear), niet in gamma/sRGB.
   * En: staat de mist vóór of ná tonemapping? (In HDR vóór tonemapping voelt meestal consistenter.)

4. Wat je doet met sky/background

   * Skybox vaak geen depth (of depth=1). Wil je dat sky wél mistt (meestal nee), of juist “achter” mist valt (meestal al
     impliciet door mist→sky-kleur)?
   * Een simpele regel: als depth == far (of 1.0), behandel als “geen geometry” en laat ongewijzigd, of mist naar
     sky-kleur; kies één.

5. Transparante objecten/particles Een pure post-process mist mist vaak correcte mist op transparanten, omdat zij niet in
   de depth zitten zoals opaques. Je moet kiezen:

   * Of transparanten in hun eigen shader misten (met dezelfde mist-functie), óf
   * Een aparte depth-prepass/soft particles aanpak, óf
   * Accepteren dat transparanten minder correct misten.

6. Banding en stabiliteit Bij zachte gradients zie je snel banding.

   * Dither (kleine noise) op de mist-factor of output is een simpele klassieke fix.
   * Als je TAA hebt: zorg dat dithering niet “zwemt” (bijv. screen-space blue-noise + frame index).

   Als je dit als “implementatiespecificatie” aan een LLM geeft, is het doorgaans genoeg voor een goede implementatie:

   * Pipeline: full-screen post-process met inputs `sceneColor` (HDR), `depthTexture`, camera matrices (`proj`,
     `invProj` of `invViewProj`), plus flags `reversedZ`, `depthRange`.
   * Definieer ($d$): view-space ($z$) (positief naar voren).
   * Mist modes: linear (start/end) en exp/exp2 (density of visibility-distance).
   * Blend in linear HDR vóór tonemapping.
   * Sky handling: depth==far → geen mist of mist naar sky (kies).
   * Transparanten: per-material mist in forward pass (zelfde functie).
   * Optional: height mist: reconstrueer world position (met `invViewProj`) en gebruik `worldY` in ($f(d,y)$).
