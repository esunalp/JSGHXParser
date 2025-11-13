//! Implementatie van de numerieke optelcomponent.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

/// Grasshopper geeft de opteloutput doorgaans uit op pin "R" (result).
const OUTPUT_PIN: &str = "R";

/// Markerstruct voor een component.
#[derive(Debug, Default, Clone, Copy)]
pub struct ComponentImpl;

impl Component for ComponentImpl {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new(
                "Add component vereist twee invoerwaarden",
            ));
        }

        let a = coerce_number(&inputs[0])?;
        let b = coerce_number(&inputs[1])?;
        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_PIN.to_owned(), Value::Number(a + b));
        Ok(outputs)
    }
}

fn coerce_number(value: &Value) -> Result<f64, ComponentError> {
    match value {
        Value::Number(number) => {
            if number.is_nan() {
                Err(ComponentError::new("Add component ontving NaN"))
            } else {
                Ok(*number)
            }
        }
        Value::List(values) => {
            if values.len() == 1 {
                coerce_number(&values[0])
            } else {
                Err(ComponentError::new(
                    "Add component verwacht een enkel getal per pin",
                ))
            }
        }
        other => Err(ComponentError::new(format!(
            "Add component verwacht nummers, kreeg {}",
            other.kind()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{Component, ComponentImpl, OUTPUT_PIN, coerce_number};
    use crate::components::ComponentError;
    use crate::graph::node::MetaMap;
    use crate::graph::value::Value;

    #[test]
    fn sums_two_numbers() {
        let component = ComponentImpl;
        let outputs = component
            .evaluate(&[Value::Number(2.0), Value::Number(3.5)], &MetaMap::new())
            .expect("add succeeded");
        assert!(matches!(
            outputs.get(OUTPUT_PIN),
            Some(Value::Number(result)) if (*result - 5.5).abs() < 1e-9
        ));
    }

    #[test]
    fn flattens_single_item_lists() {
        let component = ComponentImpl;
        let inputs = [
            Value::List(vec![Value::Number(1.0)]),
            Value::List(vec![Value::Number(2.0)]),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("lists collapse");
        assert!(matches!(
            outputs.get(OUTPUT_PIN),
            Some(Value::Number(result)) if (*result - 3.0).abs() < 1e-9
        ));
    }

    #[test]
    fn rejects_nan_input() {
        let component = ComponentImpl;
        let err = component
            .evaluate(
                &[Value::Number(f64::NAN), Value::Number(1.0)],
                &MetaMap::new(),
            )
            .unwrap_err();
        assert!(err.message().contains("NaN"));
    }

    #[test]
    fn rejects_multiple_values_in_list() {
        let err =
            coerce_number(&Value::List(vec![Value::Number(1.0), Value::Number(2.0)])).unwrap_err();
        assert!(matches!(err, ComponentError::Message(_)));
    }
}
