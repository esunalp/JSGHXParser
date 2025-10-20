# POC Buildplan — Three.js GHX Parser (HTML + JavaScript, zonder frameworks)

## Doel
Een minimalistische proof‑of‑concept die een Grasshopper **.ghx** (XML) model kan inlezen, **sliders** automatisch als UI‑sliders toont, een subset van **nodes** evalueert via een lichte dataflow‑engine, en de resulterende **geometrie** rendert met **Three.js** — alles in pure HTML/JS, zonder bundlers of frameworks.

## Scope
- **In**: GHX inlezen via `<input type="file">`, parser voor basis‑nodes (Number Slider, eenvoudige primitives zoals Box), UI‑sliders synchroniseren met engine, renderen in Three.js.
- **Uit**: Volledige GH‑feature parity, performance‑optimalisaties voor grote grafen, alle componenten/plug‑ins.

---

## Architectuur Overzicht
**Modules**
1. **`index.html`** — UI‑shell, bestandsinvoer, opstartlogica.
2. **`three-scene.js`** — Three.js scene/camera/renderer + `updateMesh` helper.
3. **`ghx-loader.js`** — GHX → `{ nodes, wires }` parser (DOMParser). Heuristiek in POC; paden worden na voorbeeldbestanden aangescherpt.
4. **`engine.js`** — Lichte dataflow‑engine: graaf laden, topo‑sort, dirty re‑eval, uitvoerdoorvoer naar renderer.
5. **`registry.js`** — Component registry: `{ guid|name → {inputs, outputs, eval()} }`.
6. **`ui.js`** — Sliders genereren en binden aan `engine.setSliderValue()`.

**Datastroom**
GHX (XML) → `parseGHX()` → `Graph{nodes[], wires[]}` → `engine.loadGraph()` → `engine.evaluate()` → (geometry) → `updateMesh()` → Three.js scene.

---

## Bestandsstructuur (POC)
```
/poc-ghx-three/
  index.html
  three-scene.js
  ghx-loader.js
  engine.js
  registry.js
  ui.js
```

---

## Datamodel
- **Node**: `{ id, name, guid, inputs?:{}, outputs?:{}, meta?:{} }`
- **Wire**: `{ from:{node,pin}, to:{node,pin} }`
- **Graph**: `{ nodes: Node[], wires: Wire[] }`
- **Value**: generiek JS‑type (number, object, arrays) of Three‑object (`THREE.BufferGeometry`).

**Conventies**
- Pins zijn case‑sensitive strings (bv. `W`, `H`, `D`, `value`, `geom`).
- Een node‑`eval()` **mag** extra metadata in `meta` lezen (bv. slider‑range).

---

## Engine (evaluatie)
- **Topologisch sorteren** via Kahn’s algorithm op basis van `wires`.
- **Dirty‑propagatie**: bij sliderupdate markeer node en downstream nodes dirty.
- **Resolve inputs**: per pin naar upstream waardes of fallback naar `node.inputs` defaults.
- **Uitvoer**: eerste gevonden `geom` of `mesh` wordt naar `updateMesh()` gepusht.

**Performantie** (POC‑niveau)
- Small graphs: re‑eval in main thread oké.
- Toekomst: Web Worker voor eval en debouncing van slider events.

---

## Registry (component‑contract)
Een registry‑entry ziet er zo uit:
```js
{
  inputs: { W:1, H:1, D:1 },
  outputs: { geom:null },
  eval: (node, inputs) => ({ geom: new THREE.BoxGeometry(...) })
}
```
**Sleutels**
- Lookup op **GUID** en fallback op **Name**. We vullen jouw officiële GH GUID‑lijst hier in.
- POC bevat `SLIDER` (Number Slider) en `BOX` (BoxPrimitive).

---

## GHX Parser (eerste iteratie)
- **DOMParser** parseert XML string → document.
- Zoek **Object/Chunk**‑entries met `GUID` en `Name` items.
- Sliders detecteren op naam (\"Number Slider\") of bekende GUIDs.
- Default slider‑range: `min=0, max=10, step=0.01, value=1` (wordt vervangen door echte waarden zodra GHX‑voorbeeld binnen is).
- Wires: placeholder (lege set) in POC; echte mapping volgt na analyse van jouw GHX.

**Benodigde voorbeelddata**
- Minimum: GHX met **Number Slider(s) → Box**.
- Optioneel: Circle → Extrude of Loft om curve/mesh‑paden te testen.

---

## UI Sliders
- UI bouwt op basis van `engine.listSliders()`.
- Elke slider krijgt `range` + `number` input, live‑sync.
- `oninput` → `engine.setSliderValue(id, v)` → `engine.evaluate()`.

---

## Three.js Layer
- Scene met `GridHelper`, `AxesHelper`, `DirectionalLight`, `AmbientLight`.
- OrbitControls voor inspectie.
- `updateMesh(api, geomOrMesh)` verwijdert oude mesh en voegt nieuwe toe.

---

## MVP Functionaliteit (Definition of Done)
1. Mock‑graph zichtbaar bij start: **Box** met 3 sliders (W/H/D).
2. GHX upload werkt; parser toont gedetecteerde sliders als UI.
3. Sliderinteractie herberekent en ververst de Box‑geometrie.
4. Registry extensible: toevoegen van node‑implementaties zonder de engine te wijzigen.

---

## Roadmap (stapsgewijs)
**Mijlpaal 1 — POC**
- [ ] Basis skeleton (bestanden en modules).
- [ ] Mock graph (Number Slider → Box) live.
- [ ] GHX inlezen + rudimentaire node‑extractie.

**Mijlpaal 2 — Real GHX wiring**
- [ ] Analyse voorbeeld‑GHX: exacte paden voor nodes/params/wires.
- [ ] Parser upgraden: echte slider‑ranges + bron/target mapping.
- [ ] Registry uitbreiden: Point/Vector/Line/Rectangle/Circle, eenvoudige ops (Add, Multiply), Extrude/Loft (light versie).

**Mijlpaal 3 — UX & Stabiliteit**
- [ ] Error‑badges in UI voor onbekende nodes.
- [ ] Debounce op sliderinput.
- [ ] Persist laatste GHX in `localStorage` (optioneel).

**Mijlpaal 4 — Performantie & Features**
- [ ] Web Worker voor evaluatie.
- [ ] BoundingBox auto‑framing, simple materials, toggles voor helpers.
- [ ] Export knoppen: `GLB` / `OBJ` (optioneel).

---

## Testplan (kort)
- **Unitachtig**: registry‑evals met bekende inputs → verwachte outputs.
- **Integratie**: GHX met 1–3 sliders → Box dimensies wijzigen.
- **Regressie**: onbekende nodes in GHX mogen POC niet laten crashen (worden gelogd, overgeslagen).

---

## Beperkingen & Risico’s
- GHX‑structuur varieert per versie; selectors moeten op echte samples worden afgestemd.
- Grasshopper‑naming vs. GUIDs: altijd GUID‑first matchen om ambiguïteit te voorkomen.
- Evalueren van zware grafen in main thread kan haperen (later: Worker).

---

## Integratie van jouw node‑lijst
- Voeg in `registry.js` entries toe op **GUID** en op **Name** (fallback).
- Leg per node de **input‑/output‑pin‑namen** vast zoals in jouw lijst.
- Waar nodig mappings toevoegen (bv. GH‑pin “R” → interne `Radius`).

---

## Acceptatiecriteria POC
- Binnen één HTML‑pagina alles functioneel.
- Zonder internet (behalve Three.js CDN) te draaien.
- Minstens 1 echte GHX‑case werkt: sliders → geupdate geometrie.

---

## Volgende stappen (actiepunten)
1. **Voorbeeld‑GHX** aanleveren met: Number Slider(s) → Box.
2. **GUID‑lijst** + pin‑namen delen (CSV/JSON is prima).
3. Wij vullen `ghx-loader.js` selectors en `registry.js` implementaties aan.
4. Uitbreiden naar 2–3 aanvullende nodes (Circle, Extrude, Deconstruct Domain) om de keten te bewijzen.

---

## TODO‑lijst (concreet)

### Setup & Skeleton
- [ ] Repo/mapstructuur `/poc-ghx-three/` aanmaken.
- [ ] `index.html` opzetten met file‑input + status + sidebar.
- [ ] `three-scene.js` met scene, camera, OrbitControls en `updateMesh()` schrijven.
- [ ] `engine.js` (topo‑sort, dirty‑propagatie, eval‑loop) implementeren.
- [ ] `registry.js` met `SLIDER` en `BOX` opnemen (GUID + name fallback).
- [ ] `ui.js` voor dynamische slider‑UI bouwen.

### Mock Graph (direct zichtbaar)
- [x] Mock graph met 3 sliders (W/H/D) → Box aan `engine.loadGraph()` toevoegen.
- [x] Live evaluatie en render controleren.

### GHX Parser (eerste iteratie)
- [ ] `ghx-loader.js` basis: DOMParser, nodes detecteren op `GUID`/`Name`.
- [ ] Heuristische slider‑detectie met default min/max/step/value.
- [ ] Logging van onbekende nodes (voor latere mapping).
- [ ] Wires voorlopig overslaan of mocken; engine mag niet crashen.

### Integratie met echte GHX
- [ ] Voorbeeld‑GHX (Number Slider → Box) ontvangen en lokaal testen.
- [ ] Juiste XML‑paden voor nodes/params/wires vastleggen.
- [ ] Slider‑ranges/waarde uit `PersistentData` (of equivalent) uitlezen.
- [ ] Wire‑mapping implementeren (`from {node,pin}` → `to {node,pin}`).
- [ ] Parser‑unitchecks op 2–3 varianten (Rhino 6/7 verschillen indien nodig).

### Registry uitbreiden (minimale keten)
- [ ] `Point` + `Vector` + eenvoudige rekennodes (`Add`, `Multiply`).
- [ ] `Circle (R)` en `Extrude` (lichte variant naar `BufferGeometry`).
- [ ] Pin‑naam mapping vastleggen (GH‑pin → interne key).

### UX & Robuustheid
- [ ] Debounce op sliderinput (b.v. 16–33 ms) om jank te vermijden.
- [ ] Error‑badges/labels voor onbekende of niet‑ondersteunde nodes.
- [ ] Toggle voor helpers (Grid/Axes) en auto‑frame op bounding box.

### Performance (optioneel voor POC)
- [ ] Web Worker prototype voor evaluatie (main thread vrijhouden).
- [ ] Minimal geometry cache per node (invalidatie bij dirty‑pins).

### Export & Opslag (optioneel)
- [ ] `localStorage` voor laatst geopende GHX + sliderwaarden.
- [ ] Export naar GLB/OBJ (Three.js exporter) als proof‑point.

### Acceptatiechecks
- [ ] Start zonder GHX: mock werkt, sliders sturen Box aan.
- [ ] Met GHX: sliders uit GHX verschijnen en sturen geometrie aan.
- [ ] Onbekende nodes breken flow niet; worden gelogd.

## Bijlage A — Mini‑skeleton (samenvatting)
- `index.html`: laadt modules, init scene, mock graph, file‑upload → parse → load/eval → UI build.
- `three-scene.js`: scene setup + `updateMesh`.
- `ghx-loader.js`: DOMParser → nodes/wires (met TODO‑selectors).
- `engine.js`: topo‑sort, dirty‑propagation, eval‑loop, slider‑API.
- `registry.js`: `SLIDER`, `BOX` (GUID‑ en name‑mapping).
- `ui.js`: dynamische slider‑UI gekoppeld aan engine.

**Klaar om te itereren:** zodra jouw GHX + GUID‑lijst binnen is, vullen we parser/registry aan en testen we de volledige keten.

