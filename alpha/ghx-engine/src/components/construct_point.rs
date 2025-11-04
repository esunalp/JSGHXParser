//! Construct Point component produceert een 3D-punt uit drie scalars.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{coerce, Component, ComponentError, ComponentResult};

/// Standaard Grasshopper-uitgang voor punten is "P".
const OUTPUT_PIN: &str = "P";

/// Markerstruct voor een component.
#[derive(Debug, Default, Clone, Copy)]
pub struct ComponentImpl;

impl Component for ComponentImpl {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 3 {
            return Err(ComponentError::new(
                "Construct Point vereist drie invoerwaarden (X, Y, Z)",
            ));
        }

        let x = coerce::coerce_number(&inputs[0])?;
        let y = coerce::coerce_number(&inputs[1])?;
        let z = coerce::coerce_number(&inputs[2])?;

        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_PIN.to_owned(), Value::Point([x, y, z]));
        Ok(outputs)
    }
}

#[cfg(test)]
mod tests {
    use super::{Component, ComponentImpl, OUTPUT_PIN};
    use crate::components::coerce;
    use crate::graph::node::MetaMap;
    use crate::graph::value::Value;

    #[test]
    fn builds_point_from_numbers() {
        let component = ComponentImpl;
        let outputs = component
            .evaluate(
                &[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)],
                &MetaMap::new(),
            )
            .expect("construct succeeded");
        assert!(matches!(
            outputs.get(OUTPUT_PIN),
            Some(Value::Point(coords)) if *coords == [1.0, 2.0, 3.0]
        ));
    }

    #[test]
    fn collapses_single_item_lists() {
        let component = ComponentImpl;
        let inputs = [
            Value::List(vec![Value::Number(0.5)]),
            Value::List(vec![Value::Number(1.5)]),
            Value::List(vec![Value::Number(2.5)]),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("list inputs collapse");
        assert!(matches!(
            outputs.get(OUTPUT_PIN),
            Some(Value::Point(coords)) if *coords == [0.5, 1.5, 2.5]
        ));
    }

    #[test]
    fn rejects_non_numeric_inputs() {
        let component = ComponentImpl;
        let err = component
            .evaluate(
                &[
                    Value::Number(1.0),
                    Value::Point([0.0, 0.0, 0.0]),
                    Value::Number(2.0),
                ],
                &MetaMap::new(),
            )
            .unwrap_err();
        assert!(err.message().contains("Verwachtte"));
    }

    #[test]
    fn coerce_rejects_multi_values() {
        let err =
            coerce::coerce_number(&Value::List(vec![Value::Number(1.0), Value::Number(2.0)])).unwrap_err();
        assert!(err.message().contains("Verwachtte"));
    }

    #[test]
    fn builds_point_from_text() {
        let component = ComponentImpl;
        let outputs = component
            .evaluate(
                &[Value::Text("1.0".to_string()), Value::Text("2.0".to_string()), Value::Text("3.0".to_string())],
                &MetaMap::new(),
            )
            .expect("construct from text succeeded");
        assert!(matches!(
            outputs.get(OUTPUT_PIN),
            Some(Value::Point(coords)) if *coords == [1.0, 2.0, 3.0]
        ));
    }
}
