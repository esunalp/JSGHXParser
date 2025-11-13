//! Hulpfuncties voor het converteren van `Value`-types.

use crate::graph::value::Value;

use super::ComponentError;

pub struct Surface<'a> {
    pub vertices: &'a Vec<[f64; 3]>,
    pub faces: &'a Vec<Vec<u32>>,
}

pub fn coerce_number(value: &Value) -> Result<f64, ComponentError> {
    match value {
        Value::Number(n) => Ok(*n),
        Value::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),
        Value::Text(s) => s.parse().map_err(|_| {
            ComponentError::new(format!("Kon tekst '{}' niet naar een getal converteren", s))
        }),
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

pub fn coerce_surface<'a>(value: &'a Value) -> Result<Surface<'a>, ComponentError> {
    match value {
        Value::Surface { vertices, faces } => Ok(Surface { vertices, faces }),
        Value::List(l) if l.len() == 1 => coerce_surface(&l[0]),
        other => Err(ComponentError::new(format!(
            "Verwachtte een surface, kreeg {}",
            other.kind()
        ))),
    }
}

pub fn coerce_curve_segments(value: &Value) -> Result<Vec<([f64; 3], [f64; 3])>, ComponentError> {
    match value {
        Value::Null => Ok(Vec::new()),
        Value::CurveLine { p1, p2 } => Ok(vec![(*p1, *p2)]),
        Value::List(values) => {
            let mut segments = Vec::new();
            let mut last_point: Option<[f64; 3]> = None;

            for entry in values {
                if let Value::Point(p) = entry {
                    if let Some(last) = last_point {
                        segments.push((last, *p));
                    }
                    last_point = Some(*p);
                } else {
                    let sub_segments = coerce_curve_segments(entry)?;
                    if !sub_segments.is_empty() {
                        // Als we een polyline aan het bouwen waren, is de keten nu onderbroken.
                        // We voegen de segmenten van de sub-item toe.
                        segments.extend(sub_segments.clone());
                        // Het "laatste punt" is nu het einde van het laatste segment van de sub-item.
                        last_point = sub_segments.last().map(|s| s.1);
                    } else {
                        // De entry leverde geen segmenten op (bijv. Value::Null),
                        // dus de keten wordt onderbroken.
                        last_point = None;
                    }
                }
            }
            Ok(segments)
        }
        Value::Surface { vertices, .. } => {
            if vertices.len() < 2 {
                return Ok(Vec::new());
            }
            let mut segments = Vec::new();
            for pair in vertices.windows(2) {
                segments.push((pair[0], pair[1]));
            }
            Ok(segments)
        }
        _ => Err(ComponentError::new(format!(
            "Verwachtte een curve-achtige invoer, kreeg {}",
            value.kind()
        ))),
    }
}
