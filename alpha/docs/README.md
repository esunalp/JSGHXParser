# Three.js GHX Parser – Alpha

Dit is de documentatie voor de vereenvoudigde migratie naar een Rust/WASM-gebaseerde Grasshopper parser. Het project richt zich op één GHX-model per sessie, single-threaded evaluatie en een minimale componentenset die via Three.js wordt gevisualiseerd.

## Structuur

- `ghx-engine/`: Rust crate die naar WebAssembly kan worden gecompileerd.
- `web/`: Vanilla JavaScript + Three.js front-end die de WASM-bindings gebruikt.
- `tools/ghx-samples/`: collectie GHX-bestanden voor tests en demo's.
- `docs/`: aanvullende ontwerp- en migratiedocumentatie.

## Vereisten

| Tool | Versie advies | Opmerking |
| --- | --- | --- |
| [Rust toolchain](https://rustup.rs/) | 1.70+ | Installeer via `rustup`.
| [`wasm-pack`](https://rustwasm.github.io/wasm-pack/) | 0.11+ | Nodig voor het bouwen van de WASM-artefacten.
| [Node.js](https://nodejs.org/) & npm | 18+ | Voor het draaien van de scripts in `alpha/web`.
| Python | 3.x | Voor de eenvoudige ontwikkelserver (`python -m http.server`).

Controleer na installatie eventueel:

```bash
rustc --version
wasm-pack --version
node --version
npm --version
```

## Snelstart

```bash
cd alpha/web
npm install               # geen dependencies, maar initialiseert package-lock
npm run build:wasm        # bouwt ghx-engine naar ./pkg
npm run serve             # start http://localhost:8080 met de demo
```

Open vervolgens `http://localhost:8080` in de browser en laad een GHX-bestand uit `alpha/tools/ghx-samples/` om de render te bekijken. Gebruik `npm run dev` om build en serve in één stap uit te voeren.

## Build & Run details

### Via npm-scripts

- `npm run build:wasm`: draait `wasm-pack build --target web` vanuit `ghx-engine` en plaatst artefacten in `alpha/web/pkg`.
- `npm run serve`: start `python -m http.server --directory . --bind 0.0.0.0 8080` voor snelle iteratie.
- `npm run dev`: combineert beide stappen voor gemak.

### Handmatig bouwen

Gebruik onderstaande commando's wanneer je buiten npm om wilt werken:

```bash
cd alpha/ghx-engine
wasm-pack build --target web --out-dir ../web/pkg

cd ../web
python -m http.server --directory .
```

### Debug build

Tijdens ontwikkeling kan extra debug-informatie worden meegecompileerd via:

```bash
RUSTFLAGS="-C debuginfo=1" npm run build:wasm
```

`console_error_panic_hook` staat standaard aan zodat panics in de browserconsole verschijnen. Activeer extra logging via de feature-vlag `debug_logs`:

```bash
cd alpha/ghx-engine
cargo build --features debug_logs
```

## Architectuuroverzicht

```
┌────────────────┐      GHX XML      ┌────────────────────┐
│ Three.js UI    │ ───────────────▶ │ Rust WASM Engine    │
│ (Vanilla JS)   │                  │ (ghx-engine crate)  │
│  - file input  │ ◀─────────────── │  - Parser & graph   │
│  - sliders     │  Geometry JSON   │  - Component eval   │
└────────────────┘                  └────────────────────┘
        │                                     │
        └──────── updateThreeScene ───────────┘
```

1. De gebruiker laadt een GHX-bestand via de web-UI.
2. De GHX-string gaat naar de WASM-engine voor parsing en evaluatie.
3. De engine retourneert geometrie in JSON-formaat.
4. `three_integration.js` vertaalt het resultaat naar Three.js objecten en werkt de scene bij.

## Componentenstatus (Alpha 0.1)

| Component | GUID | Status | Opmerking |
| --- | --- | --- | --- |
| Number Slider | `{5e0b22ab-f3aa-4cc2-8329-7e548bb9a58b}` | ✅ | Ondersteunt `min`, `max`, `step` en clamping.
| Addition | `{a0d62394-a118-422d-abb3-6af115c75b25}` | ✅ | Som van twee numerieke inputs.
| Construct Point | `{3581f42a-9592-4549-bd6b-1c0fc39d067b}` | ✅ | Maakt `Value::Point` uit X/Y/Z.
| Line (2pt) | `{4c4e56eb-2f04-43f9-95a3-cc46a14f495a}` | ✅ | Produceert `Value::CurveLine`.
| Extrude (simpel) | `{962034e9-cc27-4394-afc4-5c16e3447cf9}` | ✅ | Maakt een prismatische mesh uit een lijn + hoogte.

Nog niet geïmplementeerde Grasshopper componenten vallen buiten scope van deze alpha.

## Beperkingen Alpha

- Slechts één GHX-bestand per sessie; het laden van een nieuw bestand vervangt de bestaande graph.
- Single-threaded evaluatie: geen Web Workers of `wasm-bindgen-rayon`.
- Simpele JSON-bridge tussen Rust en JS; geen typed array optimalisaties.
- Extrude produceert een basale prismatische mesh bedoeld voor demonstraties.
- Alleen de bovengenoemde componenten zijn ondersteund.

## Referentie

- [Migratieplan (vereenvoudigd)](./Migratieplan_Threejs_GHX_Parser_vereenvoudigd.md)
- [Changelog](./CHANGELOG.md)
