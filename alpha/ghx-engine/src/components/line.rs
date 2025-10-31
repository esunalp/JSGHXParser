//! Creëert een lijnsegment tussen twee punten.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

/// Grasshopper levert lijnen doorgaans op pin "L".
const OUTPUT_PIN: &str = "L";

/// Markerstruct voor een component.
#[derive(Debug, Default, Clone, Copy)]
pub struct ComponentImpl;

impl Component for ComponentImpl {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new("Line component vereist twee punten"));
        }

        let start = coerce_point(&inputs[0])?;
        let end = coerce_point(&inputs[1])?;

        if start == end {
            return Err(ComponentError::new(
                "Line component ontving identieke punten",
            ));
        }

        let mut outputs = BTreeMap::new();
        outputs.insert(
            OUTPUT_PIN.to_owned(),
            Value::CurveLine { p1: start, p2: end },
        );
        Ok(outputs)
    }
}

fn coerce_point(value: &Value) -> Result<[f64; 3], ComponentError> {
    match value {
        Value::Point(point) => Ok(*point),
        Value::List(values) if values.len() == 1 => coerce_point(&values[0]),
        other => Err(ComponentError::new(format!(
            "Line component verwacht punten, kreeg {}",
            other.kind()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{Component, ComponentImpl, OUTPUT_PIN, coerce_point};
    use crate::graph::node::MetaMap;
    use crate::graph::value::Value;

    #[test]
    fn creates_curve_line_from_points() {
        let component = ComponentImpl;
        let outputs = component
            .evaluate(
                &[Value::Point([0.0, 0.0, 0.0]), Value::Point([1.0, 0.0, 0.0])],
                &MetaMap::new(),
            )
            .expect("line created");
        assert!(matches!(
            outputs.get(OUTPUT_PIN),
            Some(Value::CurveLine { p1, p2 }) if *p1 == [0.0, 0.0, 0.0] && *p2 == [1.0, 0.0, 0.0]
        ));
    }

    #[test]
    fn collapses_single_item_lists() {
        let component = ComponentImpl;
        let inputs = [
            Value::List(vec![Value::Point([0.0, 0.0, 0.0])]),
            Value::List(vec![Value::Point([0.0, 1.0, 0.0])]),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("list inputs handled");
        assert!(matches!(
            outputs.get(OUTPUT_PIN),
            Some(Value::CurveLine { p2, .. }) if *p2 == [0.0, 1.0, 0.0]
        ));
    }

    #[test]
    fn rejects_identical_points() {
        let component = ComponentImpl;
        let err = component
            .evaluate(
                &[Value::Point([0.0, 0.0, 0.0]), Value::Point([0.0, 0.0, 0.0])],
                &MetaMap::new(),
            )
            .unwrap_err();
        assert!(err.message().contains("identieke"));
    }

    #[test]
    fn coerce_rejects_non_points() {
        let err = coerce_point(&Value::Number(1.0)).unwrap_err();
        assert!(err.message().contains("punten"));
    }
}
