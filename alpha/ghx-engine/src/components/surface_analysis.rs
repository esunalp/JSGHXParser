//! Start van de implementaties voor Grasshopper "Surface → Analysis" componenten.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

const PIN_OUTPUT_CURVES: &str = "C";
const PIN_OUTPUT_BOX_PLANE: &str = "Pl";
const PIN_OUTPUT_BOX_POINT: &str = "Pt";
const PIN_OUTPUT_BOX_INCLUDE: &str = "I";

/// Voorlopige set componentvarianten.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    SurfaceInflection,
    EvaluateBox,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Momenteel beschikbare surface-analysis componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{0efd7f0c-f63d-446d-970e-9fb0e636ea41}"],
        names: &["Surface Inflection", "SInf"],
        kind: ComponentKind::SurfaceInflection,
    },
    Registration {
        guids: &["{13b40e9c-3aed-4669-b2e8-60bd02091421}"],
        names: &["Evaluate Box", "Box"],
        kind: ComponentKind::EvaluateBox,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::SurfaceInflection => evaluate_surface_inflection(),
            Self::EvaluateBox => evaluate_box(inputs),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::SurfaceInflection => "Surface Inflection",
            Self::EvaluateBox => "Evaluate Box",
        }
    }
}

fn evaluate_surface_inflection() -> ComponentResult {
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CURVES.to_owned(), Value::List(Vec::new()));
    Ok(outputs)
}

fn evaluate_box(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 4 {
        return Err(ComponentError::new(
            "Evaluate Box verwacht een box en drie parameters",
        ));
    }

    let point = match inputs[0] {
        Value::List(ref values) if values.len() >= 8 => values
            .iter()
            .filter_map(|value| match value {
                Value::Point(point) => Some(*point),
                _ => None,
            })
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    };

    if point.len() < 8 {
        return Err(ComponentError::new(
            "Evaluate Box vereist acht hoekpunten",
        ));
    }

    let u = extract_number(inputs.get(1), "Evaluate Box U")?.clamp(0.0, 1.0);
    let v = extract_number(inputs.get(2), "Evaluate Box V")?.clamp(0.0, 1.0);
    let w = extract_number(inputs.get(3), "Evaluate Box W")?.clamp(0.0, 1.0);

    let (min, max) = bounding_box(&point);
    let location = [
        min[0] + (max[0] - min[0]) * u,
        min[1] + (max[1] - min[1]) * v,
        min[2] + (max[2] - min[2]) * w,
    ];

    let plane = Value::List(vec![
        Value::Point(location),
        Value::Point([location[0] + 1.0, location[1], location[2]]),
        Value::Point([location[0], location[1] + 1.0, location[2]]),
    ]);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BOX_PLANE.to_owned(), plane);
    outputs.insert(PIN_OUTPUT_BOX_POINT.to_owned(), Value::Point(location));
    outputs.insert(PIN_OUTPUT_BOX_INCLUDE.to_owned(), Value::Boolean(true));
    Ok(outputs)
}

fn bounding_box(points: &[[f64; 3]]) -> ([f64; 3], [f64; 3]) {
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for point in points {
        for axis in 0..3 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }
    (min, max)
}

fn extract_number(value: Option<&Value>, context: &str) -> Result<f64, ComponentError> {
    match value {
        Some(Value::Number(number)) => Ok(*number),
        Some(Value::Boolean(flag)) => Ok(if *flag { 1.0 } else { 0.0 }),
        Some(Value::List(values)) if values.len() == 1 => extract_number(values.get(0), context),
        Some(other) => Err(ComponentError::new(format!(
            "{} verwacht een getal, kreeg {}",
            context,
            other.kind()
        ))),
        None => Err(ComponentError::new(format!(
            "{} vereist een numerieke invoer",
            context
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{evaluate_surface_inflection, Component, ComponentError, ComponentKind};
    use crate::graph::node::MetaMap;
    use crate::graph::value::Value;

    #[test]
    fn surface_inflection_returns_empty_list() {
        let result = evaluate_surface_inflection().expect("valid result");
        let entry = result.get(super::PIN_OUTPUT_CURVES).expect("output present");
        assert!(matches!(entry, Value::List(values) if values.is_empty()));
    }

    #[test]
    fn evaluate_box_requires_box_input() {
        let component = ComponentKind::EvaluateBox;
        let err = component
            .evaluate(&[Value::List(Vec::new())], &MetaMap::new())
            .unwrap_err();
        assert!(matches!(err, ComponentError { .. }));
    }
}
