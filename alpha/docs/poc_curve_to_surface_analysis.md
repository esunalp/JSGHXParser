# Analyse van Curve-naar-Ruled-Surface Dataflow in POC

Dit document beschrijft de methode waarmee curve-outputs van Grasshopper-componenten worden geïnterpreteerd en voorbereid als input voor de `Ruled Surface`-component in de JavaScript Proof-of-Concept (`/poc-ghx-three/`).

## Bronnen

De analyse is gebaseerd op de volgende bestanden:
- `/poc-ghx-three/registry-components-curve.js`
- `/poc-ghx-three/registry-components-surface.js`

## 1. Output van Curve-componenten

De meeste curve-genererende componenten (zoals `Line`, `Circle`, `Arc`, etc.) in `registry-components-curve.js` produceren geen simpele lijst van punten. In plaats daarvan genereren ze een **gestandaardiseerd curve-object**.

### Structuur van het Curve-Object

Een typisch curve-object heeft de volgende structuur:

```javascript
{
  type: 'curve' | 'circle' | 'line' | ..., // Type aanduiding
  path: THREE.Path,                       // Een onderliggend Three.js Path of Curve object
  points: [THREE.Vector3, ...],           // Een array van vooraf berekende, gesamplede punten
  length: Number,                         // De totale lengte van de curve
  closed: Boolean,                        // Geeft aan of de curve gesloten is
  domain: { start: 0, end: 1, ... },      // Het parameterdomein van de curve
  // ... plus methoden:
  getPointAt: function(t) { ... },        // Geeft een Vector3 op parameter t (0-1)
  getTangentAt: function(t) { ... }       // Geeft een genormaliseerde Vector3 raaklijn op t (0-1)
}
```

De functie `createCurveFromPath` speelt een centrale rol in het creëren van deze objecten. Het neemt een `THREE.Path` en berekent hieruit een set van "spaced points" om de `points` array te vullen.

## 2. Input en Verwerking door Ruled Surface

De `Ruled Surface`-component (`{6e5de495-ba76-42d0-9985-a5c265e9aeca}`) in `registry-components-surface.js` neemt twee van deze curve-objecten als input (`curveA` en `curveB`).

De kern van de logica zit in de `eval`-functie en volgt deze stappen:

### Stap 1: Sampling (Discretisatie)

De `eval`-functie roept `sampleCurvePoints(curve, segments)` aan voor elke inputcurve.

- **Doel:** Deze functie converteert het rijke curve-object naar een eenvoudige array van `THREE.Vector3` punten.
- **Implementatie:** Het gebruikt de `getPoints()` of `getSpacedPoints()` methode van het onderliggende `THREE.Path` object, of valt terug op de `points` array als die beschikbaar is.
- **Resultaat:** Twee afzonderlijke arrays van `Vector3`-punten, bijvoorbeeld `pointsA` en `pointsB`. Standaard worden 32 segmenten gebruikt, wat resulteert in 33 punten.

### Stap 2: Resampling (Normalisatie)

De twee puntenlijsten worden doorgegeven aan `createLoftSurfaceFromSections`. Binnen deze functie vindt een cruciale normalisatieslag plaats via `resamplePolyline`.

- **Doel:** Zorgen dat beide puntenlijsten exact hetzelfde aantal punten bevatten, wat een vereiste is voor het opbouwen van een regelmatig grid.
- **Implementatie:** `resamplePolyline` berekent de lengte van beide polylijnen. De kortere lijst wordt opnieuw gesampled (via lineaire interpolatie tussen de bestaande punten) om evenveel punten te bevatten als de langere lijst.
- **Resultaat:** Twee puntenlijsten, `resampledPointsA` en `resampledPointsB`, met een identiek aantal punten.

### Stap 3: Grid Creatie & Oppervlak Generatie

De genormaliseerde puntenlijsten worden als "rijen" doorgegeven aan `createGridSurface`.

- **Doel:** Een parametrisch oppervlak creëren op basis van een 2D-grid van punten.
- **Implementatie:** `createGridSurface` retourneert een object met een `evaluate(u, v)`-functie. Deze functie voert **bilineaire interpolatie** uit op de vier dichtstbijzijnde punten in het grid om een punt op het oppervlak te berekenen voor elke `(u, v)`-coördinaat.
- **Resultaat:** Een parametrisch `surface`-object dat de `Ruled Surface` representeert.

## Conclusie voor de Rust Implementatie

De essentie voor de `Ruled Surface` is niet de verwerking van een complexe, abstracte curve-definitie, maar een proces van **discretiseren en normaliseren**:

1.  **Converteer Input:** Zorg ervoor dat elke curve-input (ongeacht de oorspronkelijke vorm) wordt omgezet in een **lijst van punten**.
2.  **Normaliseer Aantal Punten:** Vergelijk de lengtes van de twee puntenlijsten. Gebruik een `resample`-functie om de kortere lijst te interpoleren zodat deze evenveel punten bevat als de langere lijst.
3.  **Bouw Mesh:** Creëer een mesh (oppervlak) door `quads` (of twee driehoeken) te construeren tussen de corresponderende punten van de twee genormaliseerde lijsten. Bijvoorbeeld, verbind `pointsA[i]`, `pointsA[i+1]`, `pointsB[i+1]`, en `pointsB[i]`.
