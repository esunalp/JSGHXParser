# Three.js GHX Parser – Alpha

Dit is het startpunt voor de vereenvoudigde migratie naar een Rust/WASM-gebaseerde Grasshopper parser.

## Structuur

- `ghx-engine/`: Rust crate die naar WebAssembly kan worden gecompileerd.
- `web/`: eenvoudige Three.js front-end (placeholder) die later de WASM-bindings zal gebruiken.
- `tools/ghx-samples/`: collectie GHX-bestanden voor tests en demo's.

## Build instructies

Gebruik de npm-scripts in `alpha/web/package.json` om de standaard workflow uit te voeren:

```bash
cd alpha/web
npm run build:wasm
```

Dit commando draait `wasm-pack build --target web` vanuit de Rust-crate en plaatst de gegenereerde artefacten in `alpha/web/pkg`.

Voor het lokaal testen van de demo staat er ook een eenvoudige server-script klaar:

```bash
npm run serve
```

Deze script start `python -m http.server` op poort 8080 met de webdirectory als root. Wil je beide stappen na elkaar uitvoeren, gebruik dan:

```bash
npm run dev
```

### Handmatig bouwen

Zodra de implementatie verder gevorderd is kan de WASM build ook handmatig worden uitgevoerd via:

```bash
wasm-pack build --target web --out-dir ../web/pkg
```

Voor een snelle lokale testserver zonder npm-scripts kan het volgende commando worden gebruikt:

```bash
python -m http.server --directory web
```

### Debug-informatie inschakelen

Tijdens ontwikkeling kan extra debug-informatie worden meegecompileerd door `RUSTFLAGS` te zetten:

```bash
RUSTFLAGS="-C debuginfo=1" npm run build:wasm
```

De JavaScript-bestanden in `alpha/web` bevatten inline verwijzingen naar sourcemaps (`*.js.map`) zodat debugging in de browser eenvoudiger is.

## Panic hooks

Standaard wordt `console_error_panic_hook` geactiveerd zodat panics in de browserconsole zichtbaar zijn.
Gebruik de feature-vlag `debug_logs` om extra logging te activeren tijdens ontwikkeling:

```bash
cargo build --features debug_logs
```
