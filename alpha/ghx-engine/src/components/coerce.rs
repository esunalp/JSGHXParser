//! Hulpfuncties voor het converteren van `Value`-types.

use crate::graph::value::{Domain, Domain1D, Domain2D, PlaneValue, Value};

use super::ComponentError;

pub struct Surface<'a> {
    pub vertices: &'a Vec<[f64; 3]>,
    pub faces: &'a Vec<Vec<u32>>,
}

pub fn create_domain(start: f64, end: f64) -> Option<Domain1D> {
    if !start.is_finite() || !end.is_finite() {
        return None;
    }
    let min = start.min(end);
    let max = start.max(end);
    let span = end - start;
    let length = max - min;
    let center = (start + end) / 2.0;
    Some(Domain1D {
        start,
        end,
        min,
        max,
        span,
        length,
        center,
    })
}

pub fn parse_domain1d(value: &Value) -> Option<Domain1D> {
    match value {
        Value::Domain(Domain::One(domain)) => Some(domain.clone()),
        Value::Domain(Domain::Two(_)) => None,
        Value::Number(number) => create_domain(*number, *number),
        Value::List(values) => {
            if values.len() >= 2 {
                let start = coerce_number(values.get(0)?).ok();
                let end = coerce_number(values.get(1)?).ok();
                match (start, end) {
                    (Some(start), Some(end)) => create_domain(start, end),
                    _ => None,
                }
            } else if values.len() == 1 {
                coerce_domain1d(values.get(0))
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn coerce_domain1d(value: Option<&Value>) -> Option<Domain1D> {
    value.and_then(parse_domain1d)
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

pub fn coerce_number_with_default(value: Option<&Value>) -> f64 {
    match value {
        Some(Value::Null) => 0.0,
        Some(v) => coerce_number(v).unwrap_or(0.0),
        None => 0.0,
    }
}

pub fn coerce_boolean_with_default(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Null) => true,
        Some(v) => coerce_boolean(v).unwrap_or(true),
        None => true,
    }
}

pub fn coerce_point_with_default(value: Option<&Value>) -> [f64; 3] {
    match value {
        Some(Value::Null) => [0.0, 0.0, 0.0],
        Some(v) => coerce_point(v).unwrap_or([0.0, 0.0, 0.0]),
        None => [0.0, 0.0, 0.0],
    }
}

pub fn coerce_vector_with_default(value: Option<&Value>) -> [f64; 3] {
    match value {
        Some(Value::Vector(v)) => *v,
        Some(Value::Point(p)) => *p,
        _ => [0.0, 0.0, 1.0],
    }
}

pub fn coerce_plane_with_default(value: Option<&Value>) -> PlaneValue {
    if let Some(value) = value {
        if let Value::List(l) = value {
            if l.len() >= 3 {
                if let (Ok(p1), Ok(p2), Ok(p3)) = (coerce_point(&l[0]), coerce_point(&l[1]), coerce_point(&l[2])) {
                    let ab = [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]];
                    let ac = [p3[0] - p1[0], p3[1] - p1[1], p3[2] - p1[2]];
                    let z_axis = [
                        ab[1] * ac[2] - ab[2] * ac[1],
                        ab[2] * ac[0] - ab[0] * ac[2],
                        ab[0] * ac[1] - ab[1] * ac[0],
                    ];
                    let x_axis = ab;
                    let y_axis = [
                        z_axis[1] * x_axis[2] - z_axis[2] * x_axis[1],
                        z_axis[2] * x_axis[0] - z_axis[0] * x_axis[2],
                        z_axis[0] * x_axis[1] - z_axis[1] * x_axis[0],
                    ];
                    return PlaneValue::new(p1, x_axis, y_axis, z_axis);
                }
            }
        }
    }
    PlaneValue::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0])
}
