# Plan voor multi-GHX ondersteuning met gedeelde sliders en Web Workers

## Doel en uitgangspunten
- Ondersteuning voor het gelijktijdig inladen van meerdere GHX-bestanden binnen de bestaande POC (`poc-ghx-three/`). In de eerste iteratie gaan we uit van een vaste set bestanden die door het platform zelf wordt aangeboden: `wireframe.ghx`, `omgeving_<variant>.ghx`, `brugconstructie_<variant>.ghx` en `leuningontwerp_<variant>.ghx`.
- `wireframe.ghx` fungeert als primair model; de overige GHX-bestanden volgen keuzes (sliderwaarden) die in het wireframe gemaakt worden.
- Sliders met gelijke `Nickname` worden logisch gekoppeld zodat de UI één slider toont die meerdere modellen tegelijk beïnvloedt.
- Parsing en evaluatie van GHX-bestanden wordt verplaatst naar Web Workers om de UI responsief te houden en threading mogelijk te maken voor toekomstige uitbreidingen; de workers moeten zowel parameterupdates verwerken als geometrie genereren voor de Three.js scene.
- Gebruikers hoeven geen bestanden te uploaden; de UI behoudt één scenario-selector die onderliggend meerdere GHX-bestanden laadt.
- Alle wijzigingen moeten backwards compatible blijven voor het laden van een enkel GHX-bestand en houd rekening met toekomstige distributie als WebAssembly-applicatie, waarbij parsing en geometrie-engine client-side draaien.

## Uitdagingen en ontwerpkeuzes
1. **Multi-file orchestratie** — bepalen hoe meerdere graph-structuren worden beheerd, inclusief initialisatie, lifecycle, en synchronisatie binnen één gedeelde Three.js scene.
2. **Slider-deduplicatie** — definiëren hoe sliders met dezelfde `Nickname` worden samengevoegd, inclusief regels voor conflicten (verschillende ranges, defaultwaarden, stappen).
3. **Worker-architectuur** — bepalen welke logica naar workers gaat (parsing, evaluatie) en hoe data wordt geserialiseerd/deserialiseerd voor communicatie via `postMessage`.
4. **UI-aanpassingen** — uitbreiden van de UI zodat gebruikers meerdere bestanden kunnen kiezen, statusfeedback krijgen, en gekoppelde sliders op een begrijpelijke manier worden getoond.
5. **Regressie en testing** — opzetten van teststrategie om te garanderen dat bestaande functionaliteit blijft werken en dat concurrency-problemen vermeden worden.

### 1. Multi-file orchestratie
De huidige loader en engine gaan uit van één actieve graph. Voor multi-GHX is een registry nodig die alle geladen grafen bijhoudt, inclusief metadata (bestandsnaam, timestamp, gekoppelde sliders, scene-objecten). Alle geometrieën worden samengevoegd in één Three.js scene, gegroepeerd via `THREE.Group` per GHX-bestand zodat materialen en transforms individueel aanpasbaar blijven maar één renderloop volstaat. De `wireframe`-graph bepaalt de leidende sceneconfiguratie (camera, basismaatvoering); secundaire graphs alignen hun output door referenties te ontvangen naar relevante wireframe-nodes of door gedeelde transforms toe te passen.
Elke graph wordt ingekapseld via een Grasshopper Cluster-component die als extern invoerpunt voor de aanvullende GHX-bestanden dient, zodat workers consistent dezelfde componentenbibliotheek kunnen gebruiken.

Herkenning van dit Cluster-component in GHX-bestanden gebeurt aan de hand van de volgende metadata, die zowel de loader als de worker moeten controleren bij het importeren van externe graphs:

- `category`: `Params`
- `subcategory`: `Util`
- `name`: `Cluster`
- `nickname`: `Cluster`
- `guid`: `865c8275-d9db-4b9a-92d4-883ef3b00b4a`
- `description`: `Contains a cluster of Grasshopper components`
- `inputs`: `[]`
- `outputs`: `[]`

:::task-stub{title="Introduceer graph-registry voor meerdere GHX-bestanden"}
1. Voeg een `GraphRegistry` module toe (bijv. `poc-ghx-three/graph-registry.js`) die graph-instanties kan registreren, opzoeken en verwijderen.
2. Pas `ghx-loader.js` en `engine.js` aan zodat ze via deze registry werken in plaats van vanuit een singleton graph.
3. Implementeer lifecycle hooks (`onGraphAdded`, `onGraphRemoved`) voor UI en render-updates.
:::

### 2. Slider-deduplicatie
Sliders met identieke `Nickname` moeten tot één UI-input worden samengebracht. Hiervoor is een mapping nodig van `Nickname` → lijst van slider-nodes in verschillende graphs. Conflicterende ranges worden opgelost met `wireframe.ghx` als autoriteit: de slider-range, -step en default van het wireframe gelden als norm. Afwijkende waarden uit secundaire graphs worden genormaliseerd (herberekening op basis van ratio) of, indien buiten bereik, gemarkeerd zodat het secundaire graph een fallbackwaarde gebruikt. Documenteer beslisregels en bied feedback in de UI als normalisatie of clamping optreedt.

:::task-stub{title="Implementeer gedeelde slider-synchronisatie"}
1. Breid de parser (`ghx-loader.js`) uit zodat slider-metadata een uniek graph-ID meekrijgt.
2. Bouw een `SliderLinker` util die per `Nickname` alle sliders groepeert en een genormaliseerde range/step/value bepaalt met `wireframe` als leidend referentieprofiel.
3. Update `ui.js` zodat één slider-element meerdere graph/param referenties kan aansturen, inclusief terugkoppeling over normalisatie, en wijzigingen naar alle betrokken graphs pusht.
:::

### 3. Worker-architectuur
Parsing en evaluatie zijn CPU-intensief en moeten naar Web Workers. Bepaal de worker-grenzen: minstens één worker voor parsing; mogelijk aparte workers voor evaluatie/updating. Ontwerp het message-protocol (`LOAD_GHX`, `UPDATE_SLIDER`, `EVALUATION_RESULT`) en definieer transferable data (bijv. gebruik `structuredClone`-veilige objecten). Elke worker draait dezelfde set Grasshopper-componenten (componentbibliotheken zijn reeds beschikbaar) en moet zowel parameterwijzigingen evalueren als meshes/curves genereren en terugsturen naar de main thread. Gebruik de bestaande component-registratie (`poc-ghx-three/registry.js`) als single source of truth: laad deze bij worker-initialisatie zodat workers automatisch dezelfde categorieën en componentimplementaties krijgen als de main thread.

:::task-stub{title="Introduceer GHX worker-pool"}
1. Maak een nieuwe worker-script (`poc-ghx-three/workers/ghx-worker.js`) die `parseGHX` importeert, Cluster-invoerpaden kan initialiseren, en messages verwerkt.
2. Laat de worker tijdens bootstrapping `registry.js` inladen en initialiseren zodat alle huidige componentcategorieën en implementaties beschikbaar zijn; valideer dat ontbrekende componenten een duidelijke fout genereren.
3. Voeg een `WorkerManager` toe die workers beheert, messages dispatcht, en responses terugstuurt naar UI/engine.
4. Migreer bestaande `parseGHX` calls en evaluaties naar async worker-requests; zorg voor foutafhandeling en timeouts, en stuur geproduceerde geometrie als seriële buffers (bijv. `Float32Array`) naar de main thread.
:::

### 4. UI-aanpassingen
UI hoeft geen bestandsselectie te tonen (bestanden worden door de configuratie gekozen), maar moet status per model presenteren en gedeelde sliders visueel markeren. Denk aan een overzichtspanel dat aangeeft welke variant van `omgeving`, `brugconstructie` en `leuningontwerp` actief is en of ze succesvol aan het wireframe zijn gekoppeld. Voeg indicatoren toe als slider-ranges geharmoniseerd moesten worden en geef ontwikkelaars tooling (debug panel) om individuele graphs tijdelijk te pauzeren.

:::task-stub{title="Verhoog UI voor multi-bestand beheer"}
1. Pas `index.html` en `ui.js` aan om meerdere vooraf gedefinieerde bestanden te initialiseren op basis van scenario-selectie (geen user uploads) en laadstatus visueel te tonen.
2. Ontwerp een component (lijst of panel) dat alle geladen graphs toont, inclusief laadstatus en fouten.
3. Toon gedeelde sliders met labels of badges die aangeven hoeveel graphs gekoppeld zijn; bied mogelijkheid om individuele graph-koppelingen te pauzeren.
:::

### 5. Regressie en testing
Zorg voor scenario’s waarmee multi-GHX workflows getest kunnen worden: variabele slider ranges, ontbrekende nicknames, grote bestanden. Automatiseer waar mogelijk (unit-tests voor `SliderLinker`, `GraphRegistry`, worker messaging) en voeg handmatige testcases voor UI/Three.js rendering toe.

:::task-stub{title="Testing en kwaliteitswaarborg"}
1. Voeg unit-tests toe (bijv. met Vitest/Jest) voor nieuwe util-modules en worker message handlers.
2. Documenteer handmatige testcases, inclusief performancemeting met meerdere grote GHX-bestanden en validatie van gedeelde slider-normalisatie.
3. Breid bestaande plan-documentatie uit met regressiescenario’s en voeg checklist toe voor release.
:::

## Open vragen / beslispunten
- Hoe gaan we om met slider-conflicten als ranges niet overeenkomen? → `wireframe` bepaalt grenzen; afwijkingen worden genormaliseerd of gemarkeerd.
- Moet elke graph in dezelfde Three.js scene landen of krijgt elk een aparte scene-layer? → Eén scene met per-graph `THREE.Group`-containers.
- Worden evaluaties ook in workers gedaan of blijft alleen parsing daar? → Eerste iteratie verplaatst parsing én evaluatie naar workers; main thread beperkt zich tot orchestratie en rendering.

## Volgende stappen
1. Bevestig dat alle huidige componentcategorieën uit `registry.js` worker-side beschikbaar komen en voeg monitoring toe voor toekomstige componenten.
2. Finaliseer messageprotocol en dataformaten vóór implementatie.
3. Begin met `GraphRegistry` en worker infrastructuur; daarna UI en slider-koppeling iteratief toevoegen.

## Ontwikkel TODO-lijst
- [x] `GraphRegistry` module aanmaken en loader/engine migreren naar multi-graph beheer.
- [x] Slider-normalisatie en `SliderLinker` bouwen, inclusief UI-koppeling voor gedeelde inputs.
- [x] Worker-bootstrap schrijven die `registry.js` initialiseert en GHX parsing/evaluatie verwerkt.
- [ ] Messageprotocol (`LOAD_GHX`, `UPDATE_SLIDER`, `EVALUATION_RESULT`) definiëren en testen.
- [ ] Three.js scene-groepen per GHX-bestand introduceren voor beheer en debug.
- [ ] UI-panel ontwikkelen voor graph-status, scenarioselectie en slider-badges.
- [ ] Unit-tests toevoegen voor registry, sliderlinking en worker-communicatie.
- [ ] Handmatige regressies uitvoeren met samengestelde GHX-sets (wireframe + varianten) en documenteren.
