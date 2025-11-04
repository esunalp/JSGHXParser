//! Creëert een lijnsegment tussen twee punten.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentResult};

/// Grasshopper levert lijnen doorgaans op pin "L".
const OUTPUT_PIN: &str = "L";

/// Markerstruct voor een component.
#[derive(Debug, Default, Clone, Copy)]
pub struct ComponentImpl;

impl Component for ComponentImpl {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        let start = inputs.get(0).and_then(coerce_point);
        let end = inputs.get(1).and_then(coerce_point);

        let output = match (start, end) {
            (Some(start), Some(end)) if start != end => Value::CurveLine { p1: start, p2: end },
            _ => Value::Null,
        };

        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_PIN.to_owned(), output);
        Ok(outputs)
    }
}

fn coerce_point(value: &Value) -> Option<[f64; 3]> {
    match value {
        Value::Point(point) => Some(*point),
        Value::List(values) if values.len() == 1 => coerce_point(&values[0]),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{coerce_point, Component, ComponentImpl, OUTPUT_PIN};
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
    fn returns_null_for_identical_points() {
        let component = ComponentImpl;
        let outputs = component
            .evaluate(
                &[Value::Point([0.0, 0.0, 0.0]), Value::Point([0.0, 0.0, 0.0])],
                &MetaMap::new(),
            )
            .unwrap();
        assert!(matches!(outputs.get(OUTPUT_PIN), Some(Value::Null)));
    }

    #[test]
    fn returns_null_for_null_input() {
        let component = ComponentImpl;
        let outputs = component
            .evaluate(
                &[Value::Null, Value::Point([0.0, 0.0, 0.0])],
                &MetaMap::new(),
            )
            .unwrap();
        assert!(matches!(outputs.get(OUTPUT_PIN), Some(Value::Null)));
    }

    #[test]
    fn coerce_returns_none_for_non_points() {
        assert!(coerce_point(&Value::Number(1.0)).is_none());
    }

    #[test]
    fn coerce_returns_none_for_null() {
        assert!(coerce_point(&Value::Null).is_none());
    }
}
