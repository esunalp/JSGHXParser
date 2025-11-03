//! Hulpfuncties voor het converteren van `Value`-types.

use crate::graph::value::Value;

use super::ComponentError;

pub fn coerce_number(value: &Value) -> Result<f64, ComponentError> {
    match value {
        Value::Number(n) => Ok(*n),
        Value::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),
        Value::List(l) if l.len() == 1 => coerce_number(&l[0]),
        other => Err(ComponentError::new(format!(
            "Verwachtte een getal, kreeg {}",
            other.kind()
        ))),
    }
}

pub fn coerce_text(value: &Value) -> Result<String, ComponentError> {
    match value {
        Value::Text(s) => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Boolean(b) => Ok(b.to_string()),
        Value::List(l) if l.len() == 1 => coerce_text(&l[0]),
        other => Err(ComponentError::new(format!(
            "Verwachtte een tekst, kreeg {}",
            other.kind()
        ))),
    }
}

pub fn coerce_integer(value: &Value) -> Result<i64, ComponentError> {
    match value {
        Value::Number(n) => Ok(n.round() as i64),
        Value::Boolean(b) => Ok(if *b { 1 } else { 0 }),
        Value::List(l) if l.len() == 1 => coerce_integer(&l[0]),
        other => Err(ComponentError::new(format!(
            "Verwachtte een geheel getal, kreeg {}",
            other.kind()
        ))),
    }
}

pub fn coerce_boolean(value: &Value) -> Result<bool, ComponentError> {
    match value {
        Value::Boolean(b) => Ok(*b),
        Value::Number(n) => Ok(n.abs() > 1e-9),
        Value::List(l) if l.len() == 1 => coerce_boolean(&l[0]),
        other => Err(ComponentError::new(format!(
            "Verwachtte een booleaanse waarde, kreeg {}",
            other.kind()
        ))),
    }
}

pub fn coerce_point(value: &Value) -> Result<[f64; 3], ComponentError> {
    match value {
        Value::Point(p) => Ok(*p),
        Value::List(l) if l.len() == 1 => coerce_point(&l[0]),
        other => Err(ComponentError::new(format!(
            "Verwachtte een punt, kreeg {}",
            other.kind()
        ))),
    }
}
