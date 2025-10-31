//! Implementatie van de Grasshopper "Number Slider" component.

use std::collections::BTreeMap;

use crate::graph::node::{MetaMap, MetaValue};
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

/// Grasshopper sliders hebben standaard een enkele output-pin met naam "OUT".
const OUTPUT_PIN: &str = "OUT";

/// Markerstruct voor een component.
#[derive(Debug, Default, Clone, Copy)]
pub struct ComponentImpl;

impl Component for ComponentImpl {
    fn evaluate(&self, inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        if !inputs.is_empty() {
            return Err(ComponentError::new(
                "Number Slider verwacht geen inkomende waarden",
            ));
        }

        let mut value = required_meta_number(meta, "value")?;
        if value.is_nan() {
            return Err(ComponentError::new(
                "Number Slider waarde is geen geldig getal",
            ));
        }

        let min = meta_number(meta, "min")?.unwrap_or(f64::NEG_INFINITY);
        if min.is_nan() {
            return Err(ComponentError::new("Number Slider minimum is ongeldig"));
        }

        let max = meta_number(meta, "max")?.unwrap_or(f64::INFINITY);
        if max.is_nan() {
            return Err(ComponentError::new("Number Slider maximum is ongeldig"));
        }

        if min > max {
            return Err(ComponentError::new(
                "Number Slider minimum is groter dan het maximum",
            ));
        }

        value = clamp(value, min, max);

        if let Some(step) = meta_number(meta, "step")? {
            if step.is_nan() {
                return Err(ComponentError::new("Number Slider stapgrootte is ongeldig"));
            }
            if step > 0.0 && min.is_finite() && max.is_finite() {
                let steps = ((value - min) / step).round();
                value = min + steps * step;
                value = clamp(value, min, max);
            }
        }

        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_PIN.to_owned(), Value::Number(value));
        Ok(outputs)
    }
}

fn required_meta_number(meta: &MetaMap, key: &str) -> Result<f64, ComponentError> {
    meta_number(meta, key)?
        .ok_or_else(|| ComponentError::new(format!("Number Slider mist meta sleutel `{key}`")))
}

fn meta_number(meta: &MetaMap, key: &str) -> Result<Option<f64>, ComponentError> {
    match meta.get(key) {
        Some(MetaValue::Number(value)) => Ok(Some(*value)),
        Some(MetaValue::Integer(value)) => Ok(Some(*value as f64)),
        Some(MetaValue::List(list)) if list.len() == 1 => match &list[0] {
            MetaValue::Number(value) => Ok(Some(*value)),
            MetaValue::Integer(value) => Ok(Some(*value as f64)),
            _ => Err(ComponentError::new(format!(
                "meta sleutel `{key}` bevat geen numerieke waarde"
            ))),
        },
        Some(_) => Err(ComponentError::new(format!(
            "meta sleutel `{key}` bevat geen numerieke waarde"
        ))),
        None => Ok(None),
    }
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

#[cfg(test)]
mod tests {
    use super::{Component, ComponentImpl, OUTPUT_PIN};
    use crate::components::ComponentError;
    use crate::graph::node::{MetaMap, MetaValue};
    use crate::graph::value::Value;

    fn meta_with_values(value: f64, min: f64, max: f64, step: f64) -> MetaMap {
        let mut meta = MetaMap::new();
        meta.insert("value".to_string(), MetaValue::Number(value));
        meta.insert("min".to_string(), MetaValue::Number(min));
        meta.insert("max".to_string(), MetaValue::Number(max));
        meta.insert("step".to_string(), MetaValue::Number(step));
        meta
    }

    #[test]
    fn produces_number_output() {
        let component = ComponentImpl;
        let meta = meta_with_values(2.5, 0.0, 10.0, 0.5);
        let outputs = component
            .evaluate(&[], &meta)
            .expect("slider evaluation succeeds");

        let value = outputs.get(OUTPUT_PIN).expect("contains OUT pin");
        assert!(matches!(value, Value::Number(v) if (*v - 2.5).abs() < 1e-9));
    }

    #[test]
    fn clamps_to_bounds() {
        let component = ComponentImpl;
        let meta = meta_with_values(25.0, -5.0, 10.0, 0.5);
        let outputs = component.evaluate(&[], &meta).expect("clamp succeeds");
        assert!(matches!(
            outputs.get(OUTPUT_PIN),
            Some(Value::Number(v)) if (*v - 10.0).abs() < 1e-9
        ));
    }

    #[test]
    fn quantises_to_step_size() {
        let component = ComponentImpl;
        let meta = meta_with_values(3.3, 0.0, 10.0, 0.5);
        let outputs = component
            .evaluate(&[], &meta)
            .expect("quantisation succeeds");
        assert!(matches!(
            outputs.get(OUTPUT_PIN),
            Some(Value::Number(v)) if (*v - 3.5).abs() < 1e-9
        ));
    }

    #[test]
    fn rejects_nan_value() {
        let component = ComponentImpl;
        let meta = meta_with_values(f64::NAN, 0.0, 1.0, 0.1);
        let err = component.evaluate(&[], &meta).unwrap_err();
        assert!(err.message().contains("geen geldig"));
    }

    #[test]
    fn errors_on_unexpected_inputs() {
        let component = ComponentImpl;
        let meta = meta_with_values(1.0, 0.0, 1.0, 0.1);
        let err = component
            .evaluate(&[Value::Number(1.0)], &meta)
            .unwrap_err();
        assert!(matches!(err, ComponentError { .. }));
    }
}
