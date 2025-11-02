//! Hulpfuncties voor het converteren van `Value`-types.

use crate::graph::value::Value;

use super::ComponentError;

pub fn coerce_to_f64(value: &Value) -> Result<f64, ComponentError> {
    match value {
        Value::Number(n) => Ok(*n),
        Value::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),
        Value::List(l) if l.len() == 1 => coerce_to_f64(&l[0]),
        other => Err(ComponentError::new(format!(
            "Verwachtte een getal, kreeg {}",
            other.kind()
        ))),
    }
}

pub fn coerce_to_i64(value: &Value) -> Result<i64, ComponentError> {
    match value {
        Value::Number(n) => Ok(n.round() as i64),
        Value::Boolean(b) => Ok(if *b { 1 } else { 0 }),
        Value::List(l) if l.len() == 1 => coerce_to_i64(&l[0]),
        other => Err(ComponentError::new(format!(
            "Verwachtte een geheel getal, kreeg {}",
            other.kind()
        ))),
    }
}

pub fn coerce_to_boolean(value: &Value) -> Result<bool, ComponentError> {
    match value {
        Value::Boolean(b) => Ok(*b),
        Value::Number(n) => Ok(n.abs() > 1e-9),
        Value::List(l) if l.len() == 1 => coerce_to_boolean(&l[0]),
        other => Err(ComponentError::new(format!(
            "Verwachtte een booleaanse waarde, kreeg {}",
            other.kind()
        ))),
    }
}
