# Grasshopper-componenttemplate voor de GHX-engine

Deze handleiding beschrijft een aanbevolen structuur voor componenten in `alpha/ghx-engine/src/components`.
Het document is bedoeld als leidraad bij het toevoegen van nieuwe componenten in de vereenvoudigde GHX-engine.

## Structuur en commentaar

* Begin elk componentbestand met een `//!`-modulecommentaar dat bondig uitlegt wat de component doet en hoe deze binnen Grasshopper gebruikt wordt.
* Houd de imports consistent: standaard collecties (`BTreeMap`), meta-informatie (`MetaMap`) en waardetypes (`Value`).
* Definieer de uitgangspinnen als `const`-waarden bovenaan het bestand; dat maakt de koppeling met de registry en tests eenvoudig.
* Maak voor elke component een markerstruct (bijvoorbeeld `ComponentImpl`) met `#[derive(Debug, Default, Clone, Copy)]` zodat deze zonder extra state gebruikt kan worden.

## Template

Onderstaande template laat de aanbevolen indeling zien. Vervang de `…`-markers door component-specifieke logica.

```rust
//! Korte modulebeschrijving en Grasshopper-context.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

/// Beschrijf de standaard-uitgangspin (bijv. "R", "P", "L", ...).
const OUTPUT_PIN: &str = "…";

/// Markerstruct voor deze component.
#[derive(Debug, Default, Clone, Copy)]
pub struct ComponentImpl;

impl Component for ComponentImpl {
    fn evaluate(&self, inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        // 1. Controleer het verwachte aantal inputs en eventuele meta-vereisten.
        if inputs.len() < EXPECTED {
            return Err(ComponentError::new("…"));
        }

        // 2. Zet inputs om naar het juiste type (gebruik hulpfuncties).
        let parsed = coerce_something(&inputs[0])?;
        // …

        // 3. Bouw de outputmap en vul de pinwaarden.
        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_PIN.to_owned(), Value::…);

        Ok(outputs)
    }
}

// Eén of meer hulpfuncties om type-coercie of hergebruikte logica te kapselen.
fn coerce_something(value: &Value) -> Result<DesiredType, ComponentError> {
    match value {
        Value::… => Ok(…),
        Value::List(list) if list.len() == 1 => coerce_something(&list[0]),
        other => Err(ComponentError::new(format!(
            "Verwacht …, kreeg {}",
            other.kind()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{Component, ComponentImpl, OUTPUT_PIN, coerce_something};
    use crate::graph::node::MetaMap;
    use crate::graph::value::Value;

    #[test]
    fn doet_wat_het_moet() {
        let component = ComponentImpl;
        let outputs = component
            .evaluate(&[/* inputs */], &MetaMap::new())
            .expect("component slaagt");
        assert!(matches!(
            outputs.get(OUTPUT_PIN),
            Some(Value::…)
        ));
    }

    #[test]
    fn validatie_faalt_bij_foute_input() {
        let component = ComponentImpl;
        let err = component
            .evaluate(&[/* foute input */], &MetaMap::new())
            .unwrap_err();
        assert!(err.message().contains("…"));
    }

    #[test]
    fn hulpfunctie_handelt_lijsten_of_typefouten_af() {
        let err = coerce_something(&Value::…)
            .unwrap_err();
        assert!(err.message().contains("…"));
    }
}
```

## Richtlijnen voor implementatie

1. **Valideer inputs en meta**  
   Controleer in `evaluate` eerst of het aantal binnenkomende waarden voldoet aan de verwachting. Controleer ook relevante meta-informatie (bijvoorbeeld slider-bereiken) voordat je verder rekent.

2. **Gebruik hulpfuncties voor typeconversie**  
   Maak herbruikbare functies voor typecoercie (bijvoorbeeld `coerce_number`, `coerce_point`). Dat houdt de `evaluate`-methode leesbaar en zorgt voor consistente foutmeldingen.

3. **Check de Proof-of-Concept build (/poc-ghx-three/registry-components-XXX.js)
   Het kan zijn dat het te implementeren component al functioneel is in de Javascript versie van de GHX-engine. Dit kan helpen met het sneller bouwen van de componenten.

4. **Opbouw van outputs**  
   Gebruik een `BTreeMap` voor de uitgangen. Vul iedere pinnaam (`String`) met het bijbehorende `Value`-object en retourneer `Ok(outputs)`.

5. **Test dekking**  
   Schrijf unit-tests voor het succespad, typische validatiefouten en hulpfuncties. Zo blijft het componentgedrag stabiel wanneer het parsingsysteem of andere componenten veranderen.

## Hoe de GHX-engine componenten verwerkt

1. **Parser naar Graph**  
   Het GHX-bestand wordt ingelezen en vertaald naar een graph van nodes en wires. Elke node koppelt aan een componentimplementatie via de registry (op GUID, naam of nickname).

2. **Topologische evaluatie**  
   De engine sorteert de graph topologisch en evalueert elke node. De verzamelde `Value`-inputs worden doorgegeven aan `Component::evaluate`, de outputs worden teruggeschreven naar de graph.

3. **Resultaat naar frontend**  
   Geometrische outputs worden gebundeld naar JSON-compatibele structuren en naar de Three.js-laag gestuurd. Daar worden ze omgezet in Three.js objecten voor visualisatie in de browser.

Deze workflow sluit aan op het migratieplan in [`Migratieplan_Threejs_GHX_Parser_vereenvoudigd.md`](./Migratieplan_Threejs_GHX_Parser_vereenvoudigd.md) en zorgt ervoor dat nieuwe componenten naadloos in de vereenvoudigde GHX-engine passen.

## TODO lijst implementaties componenten.
Markeer de taak als deze gedaan is.

[x] 1. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_maths_operators.json staan beschreven.
[x] 2. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_maths_domain.json staan beschreven.
[x] 3. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_maths_polynomials.json staan beschreven.
[x] 4. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_maths_matrix.json staan beschreven.
[x] 5. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_maths_script.json staan beschreven.
[x] 6. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_maths_time.json staan beschreven.
[x] 7. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_maths_trig.json staan beschreven.
[x] 8. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_maths_util.json staan beschreven.
[x] 9. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_vector_vector.json staan beschreven.
[x] 10. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_vector_point.json staan beschreven.
[x] 11. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_vector_plane.json staan beschreven.
[x] 12. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_vector_grid.json staan beschreven.
[x] 13. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_vector_field.json staan beschreven.
[x] 14. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_curve_primitive.json staan beschreven.
[x] 15. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_curve_analysis.json staan beschreven.
[ ] 16. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_curve_division.json staan beschreven.
[ ] 17. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_curve_spline.json staan beschreven.
[ ] 18. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_curve_util.json staan beschreven.
[ ] 19. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_surface_primitive.json staan beschreven.
[ ] 20. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_surface_freeform.json staan beschreven.
[ ] 21. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_surface_analysis.json staan beschreven.
[ ] 22. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_surface_util.json staan beschreven.
[ ] 23. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_surface_subd.json staan beschreven.
[ ] 24. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_transform_affine.json staan beschreven.
[ ] 25. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_transform_euclidean.json staan beschreven.
[ ] 26. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_transform_array.json staan beschreven.
[ ] 27. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_transform_util.json staan beschreven.
[ ] 28. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_complex.json staan beschreven.
[ ] 29. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_scalar.json staan beschreven.
[ ] 30. Implementeer volledig de GHX-engine componenten die in /nodelist/jsghxparser_nodelist_display_preview.json staan beschreven.
