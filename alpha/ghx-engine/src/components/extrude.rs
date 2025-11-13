//! Eenvoudige extrude component: lijn + hoogte → rechthoekig vlak.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

/// Grasshopper schrijft extrude-resultaten doorgaans naar pin "S" (surface).
const OUTPUT_PIN: &str = "S";

/// Markerstruct voor een component.
#[derive(Debug, Default, Clone, Copy)]
pub struct ComponentImpl;

impl Component for ComponentImpl {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new(
                "Extrude component vereist een curve en een hoogte",
            ));
        }

        let base_curves = collect_curves(&inputs[0])?;
        if base_curves.is_empty() {
            return Err(ComponentError::new(
                "Extrude component kon geen curve herkennen",
            ));
        }

        let direction = coerce_direction(&inputs[1])?;
        if is_zero_vector(direction) {
            return Err(ComponentError::new(
                "Extrude component vereist een niet-nul hoogte",
            ));
        }

        let mut surfaces = Vec::with_capacity(base_curves.len());
        for (p1, p2) in base_curves {
            surfaces.push(extrude_curve(p1, p2, direction));
        }

        let output_value = if surfaces.len() == 1 {
            surfaces.into_iter().next().unwrap()
        } else {
            Value::List(surfaces)
        };

        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_PIN.to_owned(), output_value);
        Ok(outputs)
    }
}

fn collect_curves(value: &Value) -> Result<Vec<([f32; 3], [f32; 3])>, ComponentError> {
    super::coerce::coerce_curve_segments(value)
}

fn coerce_direction(value: &Value) -> Result<[f32; 3], ComponentError> {
    match value {
        Value::Vector(vector) => Ok(*vector),
        Value::Number(height) => Ok([0.0, 0.0, *height]),
        Value::List(values) if values.len() == 1 => coerce_direction(&values[0]),
        other => Err(ComponentError::new(format!(
            "Extrude component verwacht een vector of getal, kreeg {}",
            other.kind()
        ))),
    }
}

fn extrude_curve(p1: [f32; 3], p2: [f32; 3], direction: [f32; 3]) -> Value {
    let top1 = add_vector(p1, direction);
    let top2 = add_vector(p2, direction);

    let vertices = vec![p1, p2, top2, top1];
    let faces = vec![vec![0, 1, 2], vec![0, 2, 3]];
    Value::Surface { vertices, faces }
}

fn add_vector(point: [f32; 3], direction: [f32; 3]) -> [f32; 3] {
    [
        point[0] + direction[0],
        point[1] + direction[1],
        point[2] + direction[2],
    ]
}

fn is_zero_vector(vector: [f32; 3]) -> bool {
    vector.iter().all(|component| component.abs() < 1e-9)
}

#[cfg(test)]
mod tests {
    use super::{
        Component, ComponentImpl, OUTPUT_PIN, add_vector, coerce_direction, collect_curves,
        extrude_curve,
    };
    use crate::components::ComponentError;
    use crate::graph::node::MetaMap;
    use crate::graph::value::Value;

    #[test]
    fn extrudes_line_with_numeric_height() {
        let component = ComponentImpl;
        let outputs = component
            .evaluate(
                &[
                    Value::CurveLine {
                        p1: [0.0, 0.0, 0.0],
                        p2: [1.0, 0.0, 0.0],
                    },
                    Value::Number(2.0),
                ],
                &MetaMap::new(),
            )
            .expect("extrude succeeded");

        let surface = outputs.get(OUTPUT_PIN).expect("surface output").clone();
        match surface {
            Value::Surface { vertices, faces } => {
                assert_eq!(vertices.len(), 4);
                assert_eq!(faces.len(), 2);
            }
            other => panic!("unexpected value: {other:?}"),
        }
    }

    #[test]
    fn extrudes_line_with_vector_height() {
        let component = ComponentImpl;
        let outputs = component
            .evaluate(
                &[
                    Value::CurveLine {
                        p1: [0.0, 0.0, 0.0],
                        p2: [0.0, 1.0, 0.0],
                    },
                    Value::Vector([0.0, 0.0, 1.0]),
                ],
                &MetaMap::new(),
            )
            .expect("vector extrude succeeded");

        let Value::Surface { vertices, .. } = outputs.get(OUTPUT_PIN).expect("surface output")
        else {
            panic!("expected surface");
        };
        assert_eq!(vertices[2], [0.0, 1.0, 1.0]);
    }

    #[test]
    fn propagates_multiple_curves() {
        let component = ComponentImpl;
        let inputs = [
            Value::List(vec![
                Value::CurveLine {
                    p1: [0.0, 0.0, 0.0],
                    p2: [1.0, 0.0, 0.0],
                },
                Value::CurveLine {
                    p1: [1.0, 0.0, 0.0],
                    p2: [1.0, 1.0, 0.0],
                },
            ]),
            Value::Number(1.0),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("multi extrude");

        let Value::List(items) = outputs.get(OUTPUT_PIN).expect("list output") else {
            panic!("expected list of surfaces");
        };
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn rejects_zero_height() {
        let component = ComponentImpl;
        let err = component
            .evaluate(
                &[
                    Value::CurveLine {
                        p1: [0.0, 0.0, 0.0],
                        p2: [1.0, 0.0, 0.0],
                    },
                    Value::Number(0.0),
                ],
                &MetaMap::new(),
            )
            .unwrap_err();
        assert!(err.message().contains("niet-nul"));
    }

    #[test]
    fn collect_curves_errors_on_non_curves() {
        let err = collect_curves(&Value::Number(1.0)).unwrap_err();
        assert!(matches!(err, ComponentError::Message(_)));
    }

    #[test]
    fn coerce_direction_accepts_list() {
        let dir = coerce_direction(&Value::List(vec![Value::Number(2.0)])).expect("direction");
        assert_eq!(dir, [0.0, 0.0, 2.0]);
    }

    #[test]
    fn add_vector_translates_point() {
        assert_eq!(
            add_vector([1.0, 2.0, 3.0], [0.5, 0.0, -1.0]),
            [1.5, 2.0, 2.0]
        );
    }

    #[test]
    fn extrude_curve_builds_surface() {
        let value = extrude_curve([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        if let Value::Surface { vertices, faces } = value {
            assert_eq!(vertices.len(), 4);
            assert_eq!(faces.len(), 2);
        } else {
            panic!("expected surface");
        }
    }
}
