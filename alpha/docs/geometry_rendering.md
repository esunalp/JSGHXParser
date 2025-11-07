# Architectuur van Geometrie Weergave

Dit document beschrijft de datastroom voor geometrie, van de evaluatie van een component in de Rust `ghx-engine` tot de uiteindelijke weergave in de Three.js frontend.

## Overzicht

Het proces bestaat uit de volgende stappen:

1.  **Component Evaluatie**: Een component in de Rust backend produceert een `Value` object.
2.  **Conversie naar `GeometryItem`**: Dit `Value` object wordt omgezet naar een serialiseerbaar `GeometryItem` object.
3.  **WASM-grens**: De `GeometryItem` wordt over de WebAssembly-grens naar de JavaScript frontend gestuurd.
4.  **Conversie naar `THREE.Object3D`**: De frontend zet de `GeometryItem` om in een Three.js object (zoals een `Mesh` of `Line`).
5.  **Rendering**: Het Three.js object wordt aan de scene toegevoegd en weergegeven.

## Gedetailleerde Stappen

### 1. Component Evaluatie (Rust Backend)

Alles begint in een component in de `ghx-engine` (bijvoorbeeld `curve_primitive.rs`). De `evaluate` functie van een component retourneert een `ComponentResult`, wat bij succes een `BTreeMap<String, Value>` bevat.

-   **Punten**: Worden weergegeven als `Value::Point`.
-   **Lijnen/Curves**: Worden doorgaans weergegeven als een lijst van punten: `Value::List(vec![Value::Point(...)])`.
-   **Oppervlakken/Meshes**: Worden weergegeven als `Value::Surface { vertices: Vec<[f64; 3]>, faces: Vec<Vec<u32>> }`.

### 2. Conversie naar `GeometryItem` (Rust Backend)

Nadat de graph is geëvalueerd, roept de frontend de `engine.get_geometry()` functie aan. Binnen deze functie wordt de `geometry_item_from_value` functie (`alpha/ghx-engine/src/lib.rs`) gebruikt om de `Value` objecten om te zetten in `GeometryItem` enums. Dit is een `enum` die specifiek is ontworpen om over de WASM-grens te worden geserialiseerd.

De `GeometryItem` heeft de volgende varianten:
-   `Point { coordinates: [f64; 3] }`
-   `CurveLine { points: Vec<[f64; 3]> }`
-   `Surface { vertices: Vec<[f64; 3]>, faces: Vec<Vec<u32>> }`

### 3. Frontend Verwerking (JavaScript)

De JavaScript-code in `alpha/web/three_integration.js` ontvangt een `GeometryResponse` object dat een lijst van `GeometryItem`s bevat. De functie `updateGeometry` itereert over deze lijst.

Voor elke `GeometryItem` wordt een overeenkomstige functie aangeroepen om een Three.js object te maken:

-   `Point` → `createPointsObject` → `THREE.InstancedMesh`
-   `CurveLine` → `createSegmentsObject` → `THREE.Line`
-   `Surface` → `createSurfaceMesh` → `THREE.Mesh`

### 4. Rendering en Vereisten

De gemaakte Three.js objecten worden toegevoegd aan een `THREE.Group`, die vervolgens aan de hoofdscene wordt toegevoegd.

Een **belangrijke vereiste** van de rendering pipeline is de post-processing stap (Screen Space Reflections - SSR). Deze stap vereist dat **alle** geometrieën die worden gerenderd, inclusief lijnen en helpers, een `normal` vertex-attribuut hebben. Zonder dit attribuut zal de geometrie niet correct worden weergegeven. De `ensureGeometryHasVertexNormals` helper-functie wordt gebruikt om dit te garanderen.
