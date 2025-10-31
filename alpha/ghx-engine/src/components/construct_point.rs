//! Construct Point component produceert een 3D-punt uit drie scalars.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

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

        let x = coerce_number(&inputs[0])?;
        let y = coerce_number(&inputs[1])?;
        let z = coerce_number(&inputs[2])?;

        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_PIN.to_owned(), Value::Point([x, y, z]));
        Ok(outputs)
    }
}

fn coerce_number(value: &Value) -> Result<f64, ComponentError> {
    match value {
        Value::Number(number) => Ok(*number),
        Value::List(values) if values.len() == 1 => coerce_number(&values[0]),
        other => Err(ComponentError::new(format!(
            "Construct Point verwacht een numerieke waarde, kreeg {}",
            other.kind()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{Component, ComponentImpl, OUTPUT_PIN, coerce_number};
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
        assert!(err.message().contains("numerieke"));
    }

    #[test]
    fn coerce_rejects_multi_values() {
        let err =
            coerce_number(&Value::List(vec![Value::Number(1.0), Value::Number(2.0)])).unwrap_err();
        assert!(err.message().contains("numerieke"));
    }
}
