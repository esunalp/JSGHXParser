# Migratieplan Three.js GHX Parser (vereenvoudigde versie)

> Gebaseerd op het document “Migratieplan Three.js GHX Parser (vereenvoudigde versie)”.

## Doel en Scope van de Migratie

Dit plan beschrijft de migratie van de bestaande Three.js GHX Parser naar een vereenvoudigde Rust/WASM-implementatie. We richten ons uitsluitend op een minimalistische uitvoering met beperkte scope, zodat we stap voor stap de kernfunctionaliteit naar WebAssembly kunnen overbrengen. Belangrijke uitgangspunten hierbij zijn:

- **Enkel één GHX-bestand per sessie**: we ondersteunen slechts het laden en uitvoeren van één Grasshopper `.ghx`-model tegelijk. Functionaliteit voor meerdere gelijktijdige GHX-bestanden of gedeelde inputparameters tussen modellen valt buiten scope.
- **Single-threaded uitvoering**: de gehele parsing en evaluatie verloopt binnen één thread. We maken geen gebruik van multithreading, `wasm-bindgen-rayon` of Web Workers. Dit vereenvoudigt de implementatie en voorkomt complexiteit van thread-synchronisatie.
- **Behoud van Three.js front-end**: de visuele weergave (rendering) blijft gehandhaafd in de bestaande Three.js code (Vanilla JS). De migratie betreft met name de achterliggende GHX parser en reken-engine.

Met deze beperkingen ligt de focus op het functioneel werkend krijgen van de GHX-parser in Rust/WASM, alvorens latere uitbreidingen (zoals multi-file of multi-threading) te overwegen.

## Architectuur Overzicht

In de nieuwe opzet wordt de Grasshopper-graph geëvalueerd binnen WebAssembly (Rust), terwijl de driehoeksgeometrie in de browser via Three.js wordt gerenderd. De architectuur bestaat uit twee hoofdcomponenten:

1. **Rust/WASM GHX Engine**: een Rust-library (gecompileerd naar WebAssembly) die het GHX-bestand inleest, de componenten en verbindingen (wires) parseert, en vervolgens de graph topologisch evalueert tot geometrische output.
2. **JavaScript Three.js Front-end**: een eenvoudige JavaScript-laag die de gebruikersinteractie afhandelt (bestanden inladen, sliders bedienen) en de van WASM ontvangen geometrie omzet naar Three.js objecten voor visualisatie.

**Datastroom in de applicatie**:

1. **GHX inladen**: de gebruiker selecteert een `.ghx`-bestand via de web-UI (bijvoorbeeld een file input element). De JS-front-end leest dit bestand als tekststring.
2. **Parsing in WASM**: de GHX-string wordt doorgegeven aan de Rust/WASM engine (via een geëxposeerde functie). De engine parseert de XML naar interne datastructuren: een `Graph` bestaande uit `Nodes` (componenten) en `Wires` (verbindingen).
3. **Graph evaluatie**: de Rust-engine voert een topologische sortering uit op de nodes en berekent vervolgens elk node-resultaat in de juiste volgorde. Hierbij worden ook eventuele slider-waardes meegenomen (sliders zijn nodes met een initiële waarde, aanpasbaar via de UI).
4. **Geometrie output**: na evaluatie retourneert de Rust-engine de resulterende geometrieën (bijv. punten, lijnen, meshes) in een gestructureerd, JSON-compatibel formaat.
5. **Three.js rendering**: de JS-front-end ontvangt deze geometrie-data en construeert overeenkomstige Three.js geometrie-objecten (zoals `THREE.BufferGeometry` of `THREE.Mesh`) om in de 3D-scene te plaatsen. De scene wordt geüpdatet om de nieuwe geometrie te tonen.

Deze pipeline zorgt voor een duidelijke scheiding: de zware berekeningen en graph-logica gebeuren in gecompileerde Rust/WASM, terwijl de gebruikersinterface en rendering in JavaScript blijven voor eenvoud en direct gebruik van Three.js.

## Ondersteuning voor één GHX-bestand per sessie

In deze vereenvoudigde migratie staat één GHX-bestand centraal per run van de applicatie. Dit betekent concreet:

- **Geen multi-file merging**: er is geen orkestratie nodig om meerdere grafen samen te voegen. De Rust-engine laadt precies één Grasshopper graph.
- **Geen gedeelde sliders tussen modellen**: sliders behoren exclusief tot het ene geladen model. Geen deduplicatie van slider-invoer over meerdere modellen nodig.
- **Eenvoudiger state management**: de engine kan de graph-status (nodes, wires, sliderwaarden, geometrie) als een enkele toestand beheren. Herladen van een nieuw GHX-bestand overschrijft simpelweg de vorige state.

**Implementatiedetail**: in de Rust-engine houden we bijvoorbeeld een struct-instantie van de huidige graph bij. Denk aan:

```rust
struct Graph {
  nodes: Vec<Node>,
  wires: Vec<Wire>,
}
```

Een nieuw GHX-bestand vervangt de oude graph.

## Single-threaded Rust/WASM uitvoering

We kiezen expliciet voor **geen multithreading** in de eerste migratieversie. Alle parsing en evaluatie gebeurt op de hoofddraad (main thread) binnen de WebAssembly-module.

**Implicaties en voordelen**:

- **Eenvoudige Wasm-configuratie**: doordat we niet threaden, zijn `SharedArrayBuffer` en cross-origin isolation (COOP/COEP) overbodig. Hosting blijft eenvoudig.
- **Geen Rayon of Web Worker setup**: geen threadpool in Wasm; minder build- en runtime-complexiteit.
- **Volledig synchroon model**: aanroepen vanuit JavaScript kunnen synchroon blijven (of pseudo-async via `await` op de wasm-init). Een slider-event kan direct `engine.set_slider_value(x)` en `engine.evaluate()` aanroepen en het resultaat terugkrijgen.
- **Performance-overweging**: acceptabel voor POC-niveau grafen. Bij zeer grote grafen kan de UI kort blokkeren; dat valt buiten huidige scope. Later kunnen Web Workers of threading worden toegevoegd.

## Migratie naar Rust/WASM: Parser en Evaluatie-Engine

Het hart van de migratie is het overhevelen van de GHX parser en de evaluatie-engine naar Rust.

### GHX Parser in Rust

De GHX parser leest een Grasshopper `.ghx`-bestand (XML) en bouwt daaruit de in-memory representatie van de graph (nodes + wires). Aanpak:

- **XML parsing**: gebruik een Rust XML-library zoals `quick-xml` of `roxmltree` om de GHX-XML efficiënt te parsen.
- **Identificatie van componenten (Nodes)**: elke Grasshopper component is opgenomen als XML-structuur met o.a. unieke `GUID`, `Name`, `Nickname` en parameters.
  - Herken componenttypes primair aan hun **GUID**. Houd een registry bij met bekende GUIDs voor de minimale component-set (bv. Number Slider, Add, etc.).
  - Als fallback kan `Name` of `Nickname` gebruikt worden.
- **Uitlezen van parameters**: lees ingebedde waarden (persistent data), zoals bij Number Slider: `Value`, `Min`, `Max`, `Step`.
- **Herkennen van wires (verbindingen)**: wires verbinden outputs van een component met inputs van een andere. Leg referenties vast als `from` (node + output-pin) en `to` (node + input-pin).

**Datamodel (indicatief)**:

```rust
struct Node {
  id: usize,
  guid: String,
  name: String,
  nickname: String,
  inputs: HashMap<String, Value>,  // default input-waarden
  outputs: HashMap<String, Value>, // gevuld na evaluatie
  meta: HashMap<String, String>,   // bv. slider range
  // eval_fn: fn(&[Value]) -> NodeResult // of via enum/trait
}

struct Wire {
  from_node: usize,
  from_pin: String,
  to_node: usize,
  to_pin: String,
}

struct Graph {
  nodes: Vec<Node>,
  wires: Vec<Wire>,
}
```

**Component registry in Rust**:

- Map bekende component GUIDs (of namen) naar implementaties, bv. `HashMap<String, ComponentImplementation>` of een `enum ComponentType` met een `match` in de evaluator.
- Per component-type is er logica om uit inputs de outputs te berekenen (trait `ComponentEval` of `match`).

### Minimale Grasshopper componenten (Vector, Math, Curve, Surface)

Focus op een kleine subset (+ basis Input):

| Component               | Categorie | Ingangen                               | Uitgangen                | Beschrijving                                                                 |
|-------------------------|----------:|----------------------------------------|--------------------------|------------------------------------------------------------------------------|
| Number Slider           |     Input | — (heeft interne getalwaarde)          | Numeric value (float)    | Interactieve numerieke invoer (scalar).                                      |
| Construct Point         |    Vector | X (float), Y (float), Z (float)        | Point (3D coördinaat)    | Maakt een 3D-punt/vector uit losse coördinaten.                              |
| Add (Numeric)           |      Math | A (float), B (float)                   | Result (float)           | Som van twee getalinputs.                                                    |
| Line (tussen 2 punten)  |     Curve | P1 (Point), P2 (Point)                 | Curve (Line)             | Lijnsegment tussen twee 3D-punten.                                           |
| Extrude Surface         |   Surface | Curve + Direction/Height (Vector/Num)  | Surface (Mesh/Geometry)  | Extrudeert een curve tot oppervlak/volumina.                                 |

**Evaluatie-voorbeelden (pseudocode)**:

```rust
fn eval_number_slider(default_val: f64) -> NodeResult {
  NodeResult::Value(Value::Number(default_val))
}

fn eval_add(inputs: &[Value]) -> NodeResult {
  let a = inputs[0].as_number();
  let b = inputs[1].as_number();
  NodeResult::Value(Value::Number(a + b))
}

fn eval_construct_point(inputs: &[Value]) -> NodeResult {
  let x = inputs[0].as_number();
  let y = inputs[1].as_number();
  let z = inputs[2].as_number();
  NodeResult::Value(Value::Point(x, y, z))
}

fn eval_line(inputs: &[Value]) -> NodeResult {
  let p1 = inputs[0].as_point();
  let p2 = inputs[1].as_point();
  NodeResult::Value(Value::CurveLine(LineSegment::new(p1, p2)))
}

fn eval_extrude(inputs: &[Value]) -> NodeResult {
  let base_curve = inputs[0].as_curve();
  let direction = inputs[1].as_vector_or_number();
  let mesh = mesh_extrude(base_curve, direction);
  NodeResult::Value(Value::Surface(mesh))
}
```

Gebruik bij voorkeur eenvoudige eigen implementaties voor basisgeometrie; voor extrusie kan een simpele, harde aanpak volstaan in deze POC.

### Topologische sortering en Graph evaluatie

- **Topologische sortering**: Kahn’s algorithm of DFS om een geldige uitvoervolgorde te bepalen op basis van afhankelijkheden (DAG-veronderstelling).
- **Evaluatie-mechanisme**:
  - Verzamel inputs: via wires (uit outputs van eerder geëvalueerde nodes) of via defaultwaarden.
  - Voer componentlogica uit en sla outputs op in de node.
  - **Resultaat bundelen**: verzamel alle geometrie-outputs (`Surface`, `Curve`, etc.) in een lijst voor de JS-laag. Eventueel een specifieke “eindnode” aanwijzen is mogelijk.

- **Foutafhandeling**: retourneer een foutmelding (`Result`) naar JS bij ontbrekende inputs of type mismatches. In POC-modus is stoppen bij eerste fout acceptabel.

## Interactie met de JavaScript frontend

We ontwerpen een duidelijke interface tussen JS en Rust/WASM voor laden, slider-sync en geometrie-output.

### Laden van de Wasm module

We gebruiken `wasm-pack` om Rust naar WebAssembly te compileren met bijbehorende JS glue-code. Voorbeeld:

```html
<script type="module">
import init, { Engine } from "./pkg/ghx_engine.js";

async function start() {
  await init();
  const engine = new Engine();

  const fileInput = document.getElementById('ghxFile');
  fileInput.onchange = async (e) => {
    const file = e.target.files[0];
    const text = await file.text();
    engine.load_ghx(text);
    engine.evaluate();
    const geomData = engine.get_geometry();
    updateThreeScene(geomData);
  };

  document.getElementById('slider1').oninput = (e) => {
    engine.set_slider_value("SliderName1", parseFloat(e.target.value));
    engine.evaluate();
    const geomData = engine.get_geometry();
    updateThreeScene(geomData);
  };
}
start();
</script>
```

### API ontwerp tussen JS en Rust

- `Engine.load_ghx(xml_string: &str)`: parse en bouwt de interne graph.
- `Engine.get_sliders() -> JsValue`: lijst slider-specificaties ({name, value, min, max, step}).
- `Engine.set_slider_value(name: &str, value: f64)`: update sliderwaarde en markeer dirty state.
- `Engine.evaluate()`: evalueer de graph.
- `Engine.get_geometry() -> JsValue`: retourneer geometrie-resultaten (JSON-achtig object).

**Voorbeeld van een mogelijk geometrie-formaat**:

```json
{
  "meshes": [
    { "type": "Surface", "vertices": [[0,0,0], [0,1,0]], "faces": [[0,1,2,3]] },
    { "type": "CurveLine", "points": [[0,0,0], [1,0,0]] }
  ]
}
```

### Three.js integratie (`updateThreeScene`)

Indicatieve implementatie:

```js
function updateThreeScene(geomData) {
  // Verwijder oude geometrie objecten
  scene.traverse(child => {
    if (child.userData.generatedByGHX) {
      scene.remove(child);
    }
  });

  // Voeg nieuwe geometrie toe
  geomData.meshes.forEach(item => {
    if (item.type === "Surface") {
      const geometry = new THREE.BufferGeometry();
      geometry.setAttribute('position', new THREE.Float32BufferAttribute(item.vertices.flat(), 3));
      geometry.setIndex(item.faces.flat());
      geometry.computeVertexNormals();
      const material = new THREE.MeshNormalMaterial();
      const mesh = new THREE.Mesh(geometry, material);
      mesh.userData.generatedByGHX = true;
      scene.add(mesh);
    } else if (item.type === "CurveLine") {
      const points = item.points.map(p => new THREE.Vector3(...p));
      const geometry = new THREE.BufferGeometry().setFromPoints(points);
      const material = new THREE.LineBasicMaterial({ color: 0x000000 });
      const line = new THREE.Line(geometry, material);
      line.userData.generatedByGHX = true;
      scene.add(line);
    }
    // ... andere types (Point -> THREE.Points, etc.)
  });
}
```

**Data-overdracht**: JSON/string is eenvoudig maar heeft overhead; optimalisaties zoals `Float32Array`/`Uint32Array` zijn later mogelijk.

## Build Toolchain en Deployment

- **`wasm-bindgen`**: annoteer Rust-functies en -structs met `#[wasm_bindgen]` om ze extern beschikbaar te maken.
- **`wasm-pack`**: simplificeert het buildproces. Voor web-target:

```bash
wasm-pack build --target web
```

Output in `pkg/` bevat o.a.: `*_bg.wasm`, `*.js`, en `*.d.ts`.

**Projectstructuur**: Rust-code in `ghx-engine/src/lib.rs`. De `pkg/`-output integreer je in de webapp (module import).

**Integratie in de webapp**: zorg dat `.wasm` met `application/wasm` wordt geserveerd.

**Geen speciale headers nodig**: omdat we niet threaden (geen `SharedArrayBuffer`), zijn COOP/COEP-headers niet vereist.

**Development vs productie**: gebruik `--release` voor geoptimaliseerde builds.

**Debuggen**: overweeg `web_sys::console::log_1()` en `console_error_panic_hook` voor zichtbare panics/logs in de browserconsole.

## Conclusie en Vervolg

Met deze vereenvoudigde aanpak migreren we de Three.js GHX Parser naar Rust/WASM met minimale scope: één GHX-model, single-threaded uitvoering. De kern is een robuuste GHX-parser en correcte dataflow-evaluatie die renderbare geometrie oplevert. Three.js blijft de weergave doen in de front-end, terwijl de rekenlogica in snelle, veilige Rust draait.

**Mogelijke vervolgstappen** (buiten scope van deze basis):
- Ondersteuning voor meerdere GHX-bestanden en combinaties (multi-graph orchestratie).
- Multithreading of Web Workers (inclusief COOP/COEP) voor zwaardere berekeningen.
- Uitbreiding van de componentenbibliotheek.
- Optimalisaties in data-uitwisseling (gedeelde memory, typed arrays).

— Einde —


---

# Alpha 0.1 – Uitgebreide TODO & Acceptatiecriteria

> **Scope Alpha 0.1**: Één GHX-bestand per sessie, single-threaded evaluatie in Rust/WASM, minimale componentenset (Number Slider, Construct Point, Add, Line (2pt), eenvoudige Extrude), JSON-bridge naar Three.js, basis-UI voor laden en sliders.

## 0) Projectstructuur & Basis

- [x] Repo aanmaken met volgende structuur:
  ```text
  /alpha/ghx-engine/                # Rust crate (wasm)
    /src/
      lib.rs
      graph/
        mod.rs
        node.rs
        wire.rs
        value.rs
        topo.rs
      components/
        mod.rs
        number_slider.rs
        construct_point.rs
        add.rs
        line.rs
        extrude.rs
      parse/
        mod.rs
        ghx_xml.rs
    /tests/
      integration.rs
    Cargo.toml
  /alpha/web/                       # Vanilla JS, Three.js front-end
    index.html
    main.js
    three_integration.js
    ui.js
    pkg/                      # wasm-pack output
  /alpha/docs/
    README.md
    CHANGELOG.md
  /alpha/tools/
    ghx-samples/              # testbestanden
      minimal_line.ghx
      minimal_extrude.ghx
  ```
- [x] `wasm-pack` pipeline in README documenteren (build, dev-serve).
- [x] `console_error_panic_hook` en feature-vlag `debug_logs` toevoegen.

## 1) Datamodellen & Component Registry

- [x] `Value`-enum: `Number(f64)`, `Point([f64;3])`, `Vector([f64;3])`, `CurveLine{p1,p2}`, `Surface{vertices,faces}`, `List(Vec<Value>)`.
- [x] `Node`-struct met `id`, `guid`, `name`, `inputs`, `outputs`, `meta`.
- [x] `Wire`-struct (from_node, from_pin, to_node, to_pin).
- [x] `Graph`-container met indexen voor snelle lookup.
- [x] `Component`-trait: `fn eval(&self, inputs: &[Value], meta:&Meta) -> Result<Outputs, Error>`.
- [x] Registry op GUID → `ComponentKind` (enum + `match`).

**Acceptatiecriteria**
- [x] Type-veiligheid: `Value` converteert strikt; duidelijke foutmeldingen bij mismatch.
- [x] Registry lookup per GUID + fallback op Name/Nickname voor testsamples.

## 2) GHX Parser (XML)

- [x] Parser met `quick-xml` in `parse::ghx_xml`.
- [x] Uit `Object`-chunks `GUID`, `Name`, `Nickname`, persistente data (sliders) extraheren.
- [x] Wires: koppel (from node/pin) → (to node/pin).
- [x] Slider-params: `Min`, `Max`, `Value`, `Step` uitlezen en in `meta` plaatsen.

**Acceptatiecriteria**
- [ ] Laden van `minimal_line.ghx` reconstrueert identieke graaf (aantal nodes/wires).
- [ ] Sliders krijgen juiste default/range in `Engine.get_sliders()`.

## 3) Topologische Sortering & Evaluatie

- [x] Kahn’s algorithm implementeren (`topo.rs`) met cycle-detectie.
- [x] Evaluator: iterate in topo-volgorde, verzamel inputs via wires of defaults.
- [x] Output opslaan per node en eindcollectie samenstellen (alle renderbare geometrie).

**Acceptatiecriteria**
- [ ] Deterministische output (zelfde input → identieke output).
- [ ] Cycle detectie geeft nette fout met padhint.

## 4) Minimale Componenten (Alpha)

- [x] **Number Slider**: levert `Value::Number` met `meta{min,max,step}`.
- [x] **Add** (numeric): som van twee `Number` inputs (promotie van int niet nodig).
- [x] **Construct Point**: `(x,y,z) -> Point`.
- [x] **Line (2pt)**: `Point,Point -> CurveLine`.
- [x] **Extrude (eenvoudig)**: `CurveLine + hoogte(Number|Vector)` → grof `Surface` (prismatisch), triangulatie simplistisch.

**Acceptatiecriteria**
- [ ] Unit tests per component met randgevallen (NaN, out-of-range slider clamp).
- [ ] Extrude produceert manifold-achtige mesh (faces != 0, vertexnormals berekenbaar).

## 5) WASM API (Engine)

- [x] `Engine.load_ghx(xml: &str) -> Result<(), JsValue>`
- [x] `Engine.get_sliders() -> JsValue` (array van `{id,name,min,max,step,value}`).
- [x] `Engine.set_slider_value(id_or_name: &str, value: f64) -> Result<(), JsValue>`
- [x] `Engine.evaluate() -> Result<(), JsValue>`
- [x] `Engine.get_geometry() -> JsValue` (JSON-achtig; efficiënte arrays later).

**Acceptatiecriteria**
- [x] Onjuiste slidernaam geeft duidelijke fout.
- [x] `get_geometry()` serialiseert zonder panics en bevat alleen renderbare types.

## 6) Front-end (Vanilla JS + Three.js)

- [ ] Bestandsloader (`<input type="file">`) → `engine.load_ghx(text)`.
- [ ] Dynamische slider-UI genereren uit `get_sliders()`.
- [ ] Event `input` → `set_slider_value()` → `evaluate()` → `updateThreeScene()`.
- [ ] `three_integration.js`: mapping naar `THREE.BufferGeometry` + materiaalkeuzes.
- [ ] Opruimen van oude objecten via `userData.generatedByGHX`.

**Acceptatiecriteria**
- [ ] Interactie is vloeiend met kleine grafen (≤ 200 nodes).
- [ ] Geen memory-leak bij herhaald evalueren (Chrome Performance snapshot OK).

## 7) Testen

- [ ] Unit tests voor parser (nodes, wires, slider-meta).
- [ ] Unit tests voor componenten (correctheid, foutpaden).
- [ ] Integratietest: `minimal_line.ghx` → 1 `CurveLine` in output.
- [ ] Integratietest: `minimal_extrude.ghx` → `Surface` met > 0 faces.
- [ ] Snapshot-test: JSON-geometry vergelijking (met toleranties voor floats).

**Acceptatiecriteria**
- [ ] `cargo test` groen; `wasm-pack test --node` optioneel voor pure logica.

## 8) DX & Build

- [ ] `wasm-pack build --target web` scripts in `/web/package.json` (optioneel).
- [ ] Eenvoudige dev-server (`python -m http.server` of `serve`) documenteren.
- [ ] Sourcemaps aan voor JS; `RUSTFLAGS="-C debuginfo=1"` voor dev.
- [ ] `CHANGELOG.md` starten (Keep a Changelog).

## 9) Documentatie

- [ ] `docs/README.md`: installatie, build, run, architectuurdiagram (ASCII).
- [ ] Componentenlijst met GUIDs en status (✅/🟡/❌).
- [ ] Beperkingen Alpha (single GHX, single-thread).

## 10) Acceptatie Demo (Alpha 0.1)

- [ ] Case 1: Twee sliders → `Construct Point` → `Line` render in Three.js.
- [ ] Case 2: `Line + hoogte` → eenvoudige `Extrude` mesh in Three.js.
- [ ] Live aanpassen sliders update direct de scene zonder errors.

---

# AI System Prompt (Codex) – Three.js GHX Parser (Alpha 0.1)

Gebruik onderstaande prompt als **system** bericht voor de AI die de code schrijft. Het is normatief, precies, en gericht op deze repo.

```
You are Codex acting as a senior Rust + WebAssembly + Three.js engineer.
Your task is to implement the Three.js GHX Parser Alpha 0.1 as specified.

NON-NEGOTIABLE SCOPE (ALPHA 0.1)
- Exactly one GHX model per session is supported.
- Single-threaded execution: do NOT use rayon, web workers, or SharedArrayBuffer.
- Rendering remains in Three.js (Vanilla JS). Rust/WASM handles parsing + evaluation only.
- Minimal components: Number Slider, Add (numeric), Construct Point, Line (2pt), simple Extrude.
- Geometry is returned via JSON-like JS values (typed arrays optimization is future work).

CODE REQUIREMENTS
- Rust crate `ghx-engine` compiled to WASM via wasm-bindgen + wasm-pack (`--target web`).
- Provide `Engine` with methods:
  - `load_ghx(xml: &str)`
  - `get_sliders() -> JsValue`
  - `set_slider_value(id_or_name: &str, value: f64)`
  - `evaluate()`
  - `get_geometry() -> JsValue`
- Use strict types: a `Value` enum; clear error messages on mismatches.
- Implement a lightweight component registry keyed by component GUID, with name/nickname fallback for known samples.
- Implement topological sorting (Kahn’s algorithm) with cycle detection.

PARSING
- Parse GHX XML (roxmltree OR quick-xml). Extract nodes (GUID/Name/Nickname), persistent data (sliders), and wires.
- Preserve slider metadata {min,max,step,value} and expose via `get_sliders()`.
- You can find all export Grasshopper components and its values in the files /poc-ghx-three/component-metadata-<category>.js, where <category> is one of the following category names: complex, curve, display, intersect, math, mesh, sets, surface, transform, vector.

EVALUATION
- Evaluate nodes in topological order and store outputs.
- Components to implement precisely:
  - Number Slider: outputs `Value::Number` clamped to {min,max}.
  - Add: sum of two numbers.
  - Construct Point: (x,y,z) → `Value::Point([f64;3])`.
  - Line (2pt): `Point,Point` → `Value::CurveLine { p1, p2 }`.
  - Extrude (simple): `CurveLine` + height (number or vector) → `Value::Surface { vertices, faces }`.
    * A minimal prism mesh is acceptable; normals computed client-side in Three.js.

FRONT-END CONTRACT
- JS is responsible for: loading GHX text, rendering UI sliders, calling Engine APIs, and converting geometry to Three.js objects.
- Geometry format must be consistent and documented. Example:
  {
    "items": [
      {"type":"CurveLine","points":[[x1,y1,z1],[x2,y2,z2]]},
      {"type":"Surface","vertices":[[...],[...],...],"faces":[[a,b,c], ...]}
    ]
  }

QUALITY BAR
- Include unit tests for parser + components; integration tests for sample GHX files.
- No panics in release builds; return `Result` with descriptive error strings.
- Ensure deterministic output for identical inputs.
- Log with feature-gated debug macros; keep console clean in production.

STYLE
- Idiomatic Rust (clippy clean). Commands and names in lowercase_snake_case.
- Small, focused modules: parse/, graph/, components/.
- Clear separation of parsing, evaluation, and JS interop.

OUT OF SCOPE
- Multi-file graphs, shared sliders, multi-threading, advanced geometry kernels, NURBS, robust meshing, typed-array interop.

DELIVERABLES
- Working WASM build in `/alpha/web/pkg` via `wasm-pack build --target web`.
- `/alpha/web/index.html` + `main.js` + `three_integration.js` demo that showcases Line + Extrude.
- `docs/README.md` with build/run instructions and current limitations.
- When finished with a todo task, mark it as done in this file
```

---

# Milestones & Planning (indicatief)

- **Week 1**: Datamodellen, Registry, Parser skeleton, `Engine` scaffolding.
- **Week 2**: Topo-sort + evaluatie, Number Slider, Add, Construct Point, Line.
- **Week 3**: Extrude (simpel), `get_geometry()` schema, front-end hook-up.
- **Week 4**: Testen, demo hardening, documentatie, Alpha 0.1 review.

---

# Checklist voor Release (Alpha 0.1)

- [ ] `cargo clippy` en `cargo fmt` schoon.
- [ ] `wasm-pack build --target web --release` produceert artefacten.
- [ ] Demo werkt met `minimal_line.ghx` en `minimal_extrude.ghx`.
- [ ] Documentatie up-to-date en beperkingen expliciet vermeld.
- [ ] Tag `v0.1.0-alpha` en changelog entry toegevoegd.
