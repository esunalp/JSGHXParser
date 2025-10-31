# Three.js GHX Parser – Alpha

Dit is het startpunt voor de vereenvoudigde migratie naar een Rust/WASM-gebaseerde Grasshopper parser.

## Structuur

- `ghx-engine/`: Rust crate die naar WebAssembly kan worden gecompileerd.
- `web/`: eenvoudige Three.js front-end (placeholder) die later de WASM-bindings zal gebruiken.
- `tools/ghx-samples/`: collectie GHX-bestanden voor tests en demo's.

## Build instructies

Zodra de implementatie verder gevorderd is kan de WASM build uitgevoerd worden via:

```bash
wasm-pack build --target web --out-dir ../web/pkg
```

Voor een snelle lokale testserver kan het volgende commando worden gebruikt:

```bash
python -m http.server --directory web
```

## Panic hooks

Standaard wordt `console_error_panic_hook` geactiveerd zodat panics in de browserconsole zichtbaar zijn.
Gebruik de feature-vlag `debug_logs` om extra logging te activeren tijdens ontwikkeling:

```bash
cargo build --features debug_logs
```
