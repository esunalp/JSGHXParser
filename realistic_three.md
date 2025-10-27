# Lumen‑achtige GI in Three.js — Implementatieplan

## Samenvatting
Volledig 1‑op‑1 Lumen reproduceren in Three.js is onrealistisch (engine‑integraties, editor tooling, deep pipeline), maar een **perceptueel gelijkwaardige** hybride is haalbaar voor web: **DDGI (Dynamic Diffuse Global Illumination) via probe‑volumes** + **screen‑space** aanvullingen (SSGI/SSR) + **gerichte ray/path tracing (WebGPU)** waar nodig, met **spatio‑temporale denoising (SVGF)**. Dit levert indoor en mid‑scale outdoor scènes die overtuigend ogen en realtime in de browser draaien.

---

## Doel & Scope
- **Doel**: Een web‑geschikte GI‑stack met Lumen‑achtige look & feel, geschikt voor Three.js + WebGPU (desktop), met WebGL2 fallback.
- **Scope**: Dynamische diffuse GI, geloofwaardige reflecties, stabiele schaduwen, denoising, debug tooling. Niet in scope: volledige UE‑editor tools, Nanite‑equivalent.

### Succescriteria (MVP)
- 60 FPS @ 1080p op RTX 3060‑klasse met ≥1 dynamische area/spot light.
- Ghosting/boiling beperkt (TAA + SVGF).
- Geen lightmaps; alle indirecte verlichting dynamisch.

---

## Lumen vs. Web‑hybride (parity‑matrix)
| Feature | Lumen (UE5) | Three.js aanpak |
|---|---|---|
| Diffuse GI | Surface cache + radiance fields + temporal reuse | **DDGI probe‑volumes** (SH L2/L3), sparse updates + hysteresis |
| Speculaire reflecties | SSR + HW/SW RT fallback | **SSR** + **RT fallback** (WebGPU BVH) |
| Schaduwen | Virtual Shadow Maps (clipmap) | **CSM** → later **virtualized atlas/clipmaps** |
| Occlusie/indirect shadows | Distance/Card fields | **SSAO/SSGI**, optioneel **voxel/SDF clipmaps** |
| Denoising | Spatio‑temporal (custom) | **SVGF‑achtig** + ATrous |

---

## Architectuur (rendergraph)
1. **G‑buffer**: albedo, normals, roughness/metalness, depth, **motion vectors**.
2. **Direct lighting**: raster + **CSM** (stabiele cascades).
3. **Screen‑space**:
   - **SSGI** (contact GI) en **SSAO**.
   - **SSR** (stochastic tracing + temporal resolve).
4. **Diffuse GI fetch**: trilineaire interpolatie uit **DDGI probe volume** (SH).
5. **Ray/Path tracing (optioneel, WebGPU)**: **ReSTIR‑DI** voor direct licht; **specular RT** fallback waar SSR faalt.
6. **Denoising**: **SVGF** (variance‑guided, reprojection) → **TAA resolve**.
7. **Background compute**: ronde‑robin **probe updates** (budget per frame), BVH onderhoud.

---

## Kernalgoritmes
### DDGI (Dynamic Diffuse GI)
- **Opslag**: Sferische Harmonischen (SH L2) per probe + variance/validity.
- **Update**: per frame beperkt aantal probes (bijv. 256) met 8–32 hemisfeer rays; **hysteresis** 0.90–0.98; **relocation** + **backface‑fixup**.
- **Sampling**: importance richting sterke lichten (MIS optioneel).
- **Fetch**: trilineair/interp. + visibility term uit depth (cone trace of occlusion factor).

### SSR/SSGI
- **SSR**: stochastic raymarch met roughness‑dependent cone; **binary search refine**; **history clamp** en **fallback** naar env/probe.
- **SSGI**: half‑res trace + bilateral upscale; combineer met SSAO voor contact.

### ReSTIR‑DI (optioneel)
- **Reservoir sampling** per pixel of 8×8 tile; **spatial reuse** met buren; **temporal reuse** met history; 1–2 lichtsamples vereist.

### Denoising (SVGF)
- **Reprojection** via motion vectors; **variance estimation**; **ATrous** wavelet passes (2–3) met edge‑awareness (normals/depth).

### Schaduwen
- **CSM** met stabilisatie, PCF/PCSS. Later: **virtualized shadow atlas** met demand‑loaded tiles (clipmaps‑achtig).

---

## Parameters (startwaarden)
- **Probe‑spacing**: 1.0–2.0 m indoor; 3–5 m outdoor.
- **Update‑budget**: 256–1024 probes/frame.
- **Hysteresis**: 0.92 (indoor), 0.96 (outdoor).
- **Rays per probe**: 8 (MVP) → 16–32 (HQ).
- **SSR**: max 64 stappen, roughness cutoff 0.8.
- **SVGF ATrous**: 2–3 passes, phi‑normals ~16–32°, phi‑depth ~1–3% scene depth range.

---

## Implementatie‑roadmap
### Fase 1 — MVP
- WebGPU‑renderer pad in Three.js + WebGL2 fallback.
- G‑buffer + motion vectors + TAA.
- **DDGI volume** (1 volume) met round‑robin updates, SH L2.
- **CSM**; **SSAO/SSGI basic**; **SSR basic**.
- **Eenvoudige bilateral denoiser**.

### Fase 2 — Kwaliteit & RT
- **SVGF** denoiser (variance‑guided).
- **three‑mesh‑bvh** integratie; RT pass (WebGPU) voor **ReSTIR‑DI** + **specular hits**.
- **Probe relocation** + **hysteresis tuning**.

### Fase 3 — Schaal & Robuustheid
- **Meerdere probe‑volumes** (per zone/ruimte); **priority updates** op zichtbare gebieden.
- **Virtualized shadows** (atlas/clipmaps).
- Debug‑views: probe occupancy, SSR hit/miss, variance heatmaps.

### Fase 4 — Optioneel (Advanced)
- Voxel/SDF clipmaps voor occlusie.
- Per‑material specular RT policies; transmissive materials.

---

## Repo‑structuur (suggestie)
```
/engine
  /core            # frame graph, resources, passes
  /gbuffer         # mrt setup, motion vectors
  /lighting        # direct light, CSM
  /gi
    /ddgi          # probes, SH storage, update compute
    /ssgi          # screen-space GI
  /reflections
    /ssr           # screen-space reflections
    /rt            # specular RT fallback
  /denoise         # svgf + atrous
  /bvh             # three-mesh-bvh glue, builders
  /debug           # heatmaps, overlays
/scenes            # testscenes: indoor, courtyard, glossy, glass
/tools             # probe baker (WASM), stats, screenshots
```

---

## Pseudocode (richtinggevend)
```js
function render(dt){
  gbuffer.pass(scene, camera);
  directLighting.pass(gbuffer, lights, csm);
  ssgi.pass(gbuffer, history);
  ssr.pass(gbuffer, history);
  ddgiFetch.pass(gbuffer, ddgi.texture);
  if (webgpu) rt.pass(gbuffer, bvh, lights); // ReSTIR/specular
  svgf.pass(gbuffer, history, motion);
  taa.resolve();
  ddgi.update(scene, lights, bvh, budget=256); // compute
  history.swap();
}
```

---

## Testscènes & KPI’s
- **Indoor corridor** (diffuse, klein emissive).
- **Glossy showroom** (mix rough/glossy, SSR heavy).
- **Courtyard** (zon + skylight, middelgrote schaal).
- **Glass box** (transmissie; check RT fallback).

**Targets**: 60 FPS 1080p (desktop), <8 ms GI budget, ghosting <5% luminantie‑drift op camera‑pan.

---

## Risico’s & mitigaties
- **Mobile perf**: fallback pad (minder probes, half‑res passes, SSR uit).
- **History artefacts**: per‑pixel clamp/variance reset, reactive masks.
- **BVH updates**: beperk dynamiek; herbouw per N frames; instabiele meshes markeren.

---

## Physical Sun & Sky (outdoor)
**Doel**: Fysisch-plausibele hemel + zon voor outdoor scènes, consistent met DDGI/SSR/CSM en met real‑time tijd‑van‑de‑dag.

### Modelkeuze
- **Sky radiance**: **Hosek–Wilkie** analytisch hemelmodel (beter dan Preetham in gouden/blauwe uurtje). 
- **Zon**: directionele lichtbron met **zonne‑schijf** (hoekdiameter ≈ 0,53°) en spectrale kleurtemp via **luchtmassa (air mass)** schatting → RGB.
- **Optional**: *Volumetric/Bruneton* (single scattering) voor extra realisme; niet voor MVP vanwege kosten.

### Invoer & besturing
- **Tijd/plaats**: datum/tijd (UTC), **lat/lon**, hoogte. Zonpositie via **SPA** (Solar Position Algorithm). Fallback: simpele astronomische benadering. 
- **Weer/atmosfeer**: 
  - **Turbidity** (2–10) → nevel/helderheid.
  - **Rayleigh** (0.5–3×), **Mie** (β, anisotropy g 0.6–0.9), **ozone** (0.2–0.5 atm‑cm).
  - **Ground albedo** (0.1–0.4). 
- **Belichting**: automatische **EV100** met camera‑curve of handmatige exposure.

### Uitvoer
- **Procedurale HDR‑hemel** (cubemap of lat‑long) voor IBL/SSR fallback.
- **Directionele zon**: intensiteit in **lux**/**kcd/m²** omgezet naar renderer‑eenheden; kleur uit air‑mass.
- **Schaduw**: CSM met **PCSS**; penumbra breedte ≈ tan(0,265°)·distance.

### Integratie met de GI‑stack
- **DDGI**: 
  - Initieer probes met **sky luminance** + zonlicht; occlusie via scene‑BVH (cone‑trace/visibility term).
  - **Time‑of‑day**: beperk grote sprongen met **hysteresis** en **reactive masks** (versnel updates in veranderende sectoren).
- **SSR**: fallback naar **hemel‑env** bij misses; specular RT kan de zon‑highlight doorduwen op glans.
- **CSM**: stabiliseer cascades; bias/normal‑offset tunen voor scherpe, stabiele zon‑schaduwen.

### Parameters (defaults)
- **Turbidity**: 3.0 (helder), 6.0 (hazer). 
- **Mie β**: 0.005–0.02; **g**: 0.8.
- **Rayleigh**: 1.0; **ozone**: 0.35.
- **Sun intensity**: 100–120 klux bij zenit; clamp dynamisch met exposure.
- **CSM**: 3–4 cascades, λ=0.7, stable fit.

### API‑schets
```ts
interface PhysicalSunSkyOptions {
  lat?: number; lon?: number; elevation?: number; // meters
  datetime?: Date; utcOffset?: number;
  turbidity?: number; rayleigh?: number; mieBeta?: number; mieG?: number; ozone?: number;
  groundAlbedo?: number; exposureMode?: 'auto'|'manual'; ev100?: number;
}
const sunSky = new PhysicalSunSky(opts);
sunSky.update(timeNow);
renderer.environment = sunSky.environmentTexture; // HDRI for IBL/SSR
sunLight.direction = sunSky.sunDirection;
sunLight.intensity = sunSky.sunLuminanceToIntensity();
```

### Prestaties
- **Procedurale sky** render naar **mipmapped** HDR texture (512–1024px) bij tijd‑updates; re‑use mips voor roughness.
- SPA berekening is triviaal (CPU). Atmos‑evaluatie gebeurt 1× per update.

### Testscènes (outdoor)
- **Courtyard** met hoge zon (helder), **sunset drive** (lage zon, hoge Mie), **overcast mock** (hoge turbidity + diffuse zon).

### To‑do (Sun & Sky)
- [ ] Hosek–Wilkie implementeren (shader + CPU precompute voor koefficiënten).
- [ ] SPA zonpositie; integratie lat/lon/time UI.
- [ ] HDR sky bake + mips; koppeling als environment.
- [ ] Zon‑kleur via air‑mass; intensiteitsschaal naar renderer units.
- [ ] CSM afstemming op zon‑schijf (PCSS penumbra).
- [ ] DDGI bootstrap met sky; reactive masks bij grote belichtingsverandering.

## To‑do (eerste sprint)
- [ ] WebGPU buildpad en feature‑gating.
- [ ] G‑buffer met motion vectors.
- [ ] DDGI SH L2 opslag + updater (compute).
- [ ] Basic SSR/SSGI + bilateral denoise.
- [ ] CSM stabilisatie.
- [ ] Debug overlay: probe heatmap, SSR hit/miss.
- [ ] Bench harness: GPU timers, per‑pass ms.

---

## Color Grading & LUTs
**Doel**: Consistente, reproduceerbare look via 1D/3D‑LUTs (filmisch, show‑LUTs, technisch). Ondersteuning voor 
`.cube` (3D/1D), `.3dl`, en **Hald CLUT** (PNG). 

### Plaats in de pipeline
1. **Scene‑referred**: linear HDR → exposure → **tonemap (ACES/filmic)** →
2. **Display‑referred**: **LUT (3D/1D)** → gamma encode (sRGB/Display P3). 
> Meestal werkt een creatieve LUT **na** tonemapping (display‑referred). Technische LUTs (bv. ACEScg→sRGB) kunnen vooraf als colorspace‑conversie.

### Implementatie
- **3D‑LUT sampling**: 3D‑texture (N³, bv. 32/33/64). **Tetrahedral** interpolatie (beter dan trilineair), fallback trilineair. 
- **1D‑LUTs**: aparte curves voor R/G/B vóór/na tonemapping (bijv. shoulder/toe fine‑tuning). 
- **Hald CLUT**: parse Hald PNG → opbouwen als 3D‑texture (n = √levels). 
- **ACES pad (optioneel)**: ACEScg/ACEScc transf., ODT‑achtig tonemap; daarna show‑LUT. 

### API‑schets
```ts
interface LutOptions {
  file: File|ArrayBuffer|Texture;   // .cube, .3dl, Hald PNG
  size?: number;                    // 3D size; auto uit bestand indien mogelijk
  domain?: 'display'|'scene';       // waar toepassen in de chain
  interpolation?: 'tetra'|'tri';
  intensity?: number;               // 0..1 blend
}
const lut = await Lut.fromFile(opts);
colorPipeline.setDisplayLUT(lut);
colorPipeline.intensity = 0.6; // mix met graded look
```

### Integratie met de render‑chain
- **Na TAA resolve** maar vóór UI compositing → stabiele grading zonder ghosts op UI. 
- HDR screenshots: optioneel **pre‑LUT** opslaan (EXR) + **post‑LUT** preview (PNG). 
- **Swap‑safe**: LUTs hot‑reload zonder pipeline rebuild; double‑buffer 3D‑texture.

### Best practices
- Werk in **linear→tonemap→LUT**; vermijd LUTs op lineaire HDR om clipping te voorkomen. 
- Houd **LUT‑size** 33³ voor kwaliteit vs. VRAM; 64³ alleen op desktop. 
- **Clamp & gamut mapping** bij verzadigd materiaal (gamut‑warp i.p.v. hard clip). 

### Performance
- 3D‑LUT lookup = 1 texture fetch (tetra = 4 fetches); <0.2 ms @ 1080p op desktop GPU. 
- Hald decode kost 1× CPU/WASM; runtime gratis (gewoon 3D‑tex).

### To‑do (LUTs)
- [ ] `.cube`/`.3dl` parser implementeren (WASM/TS). 
- [ ] Hald CLUT import → 3D‑texture builder. 
- [ ] Tetrahedral interpolatie shader path + fallback. 
- [ ] Color pipeline switch: scene‑ vs display‑referred toepassing. 
- [ ] UI: LUT browser, intensity slider, ACES toggle.

## Vervolg
Na MVP integreren we SVGF en ReSTIR‑DI, en schalen we naar meerdere probe‑volumes en virtualized shadows. Parallel bouwen we debug‑tools en ‘artist controls’ (probe density, bounce gain, denoiser strength). Dit traject levert een robuuste, onderhoudbare GI‑stack die visueel sterk aan Lumen doet denken, maar geoptimaliseerd is voor het web‑ecosysteem.

