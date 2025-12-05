//! Hulpfuncties voor het converteren van `Value`-types.

use super::ComponentError;
use crate::graph::value::{Domain, Domain1D, PlaneValue, Value};
use time::{Date, Month, PrimitiveDateTime, Time};

pub struct Surface<'a> {
    pub vertices: &'a Vec<[f64; 3]>,
    pub faces: &'a Vec<Vec<u32>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub start: [f64; 3],
    pub end: [f64; 3],
}

impl Line {
    #[must_use]
    pub fn direction(self) -> [f64; 3] {
        subtract(self.end, self.start)
    }
}

/// Een genormaliseerd vlak dat gebruikt wordt bij sommige componenten.
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub origin: [f64; 3],
    pub x_axis: [f64; 3],
    pub y_axis: [f64; 3],
    pub z_axis: [f64; 3],
}

impl Default for Plane {
    fn default() -> Self {
        Self {
            origin: [0.0, 0.0, 0.0],
            x_axis: [1.0, 0.0, 0.0],
            y_axis: [0.0, 1.0, 0.0],
            z_axis: [0.0, 0.0, 1.0],
        }
    }
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
                let start = coerce_number(values.get(0)?, None).ok();
                let end = coerce_number(values.get(1)?, None).ok();
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

pub fn coerce_number(value: &Value, context: Option<&str>) -> Result<f64, ComponentError> {
    match value {
        Value::Number(number) => {
            if let Some(ctx) = context {
                if !number.is_finite() {
                    return Err(ComponentError::new(format!(
                        "{} verwacht een eindig getal",
                        ctx
                    )));
                }
            }
            Ok(*number)
        }
        Value::Boolean(boolean) => Ok(if *boolean { 1.0 } else { 0.0 }),
        Value::Text(text) => match parse_boolean_text(text.as_str()) {
            Some(boolean) => Ok(if boolean { 1.0 } else { 0.0 }),
            None => {
                if let Some(ctx) = context {
                    Err(ComponentError::new(format!(
                        "{} verwacht een numerieke waarde, kreeg tekst '{}'",
                        ctx, text
                    )))
                } else {
                    text.parse().map_err(|_| {
                        ComponentError::new(format!(
                            "Kon tekst '{}' niet naar een getal converteren",
                            text
                        ))
                    })
                }
            }
        },
        Value::List(l) if l.len() == 1 => coerce_number(&l[0], context),
        other => {
            if let Some(ctx) = context {
                Err(ComponentError::new(format!(
                    "{} verwacht een numerieke waarde, kreeg {}",
                    ctx,
                    other.kind()
                )))
            } else {
                Err(ComponentError::new(format!(
                    "Verwachtte een getal, kreeg {}",
                    other.kind()
                )))
            }
        }
    }
}

pub fn coerce_count(
    value: Option<&Value>,
    fallback: usize,
    context: &str,
) -> Result<usize, ComponentError> {
    match value {
        None => Ok(fallback),
        Some(entry) => {
            let number = coerce_number(entry, Some(context))?;
            if !number.is_finite() {
                return Ok(fallback);
            }
            let floored = number.floor();
            if floored < 1.0 {
                Ok(1)
            } else {
                Ok(floored as usize)
            }
        }
    }
}

pub fn coerce_boolean_with_context(value: &Value, context: &str) -> Result<bool, ComponentError> {
    match value {
        Value::Boolean(value) => Ok(*value),
        Value::Number(number) => {
            if number.is_nan() {
                Err(ComponentError::new(format!(
                    "{} verwacht een booleaanse waarde, kreeg NaN",
                    context
                )))
            } else {
                Ok(*number != 0.0)
            }
        }
        Value::Text(text) => parse_boolean_text(text).ok_or_else(|| {
            ComponentError::new(format!(
                "{} verwacht een booleaanse waarde, kreeg tekst '{}'",
                context, text
            ))
        }),
        Value::List(values) if values.len() == 1 => {
            coerce_boolean_with_context(&values[0], context)
        }
        other => Err(ComponentError::new(format!(
            "{} verwacht een booleaanse waarde, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

pub fn coerce_optional_number(
    value: Option<&Value>,
    context: &str,
) -> Result<Option<f64>, ComponentError> {
    match value {
        Some(value) => coerce_number(value, Some(context)).map(Some),
        None => Ok(None),
    }
}

pub fn to_optional_number(value: Option<&Value>) -> Result<Option<f64>, ComponentError> {
    match value {
        Some(Value::Number(number)) if number.is_finite() => Ok(Some(*number)),
        Some(Value::Boolean(boolean)) => Ok(Some(if *boolean { 1.0 } else { 0.0 })),
        Some(Value::List(values)) if values.len() == 1 => to_optional_number(values.first()),
        Some(_) => Ok(None),
        None => Ok(None),
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
        Value::Text(s) => match parse_boolean_text(s.as_str()) {
            Some(boolean) => Ok(if boolean { 1 } else { 0 }),
            None => s.parse::<f64>().map(|n| n.round() as i64).map_err(|_| {
                ComponentError::new(format!(
                    "Kon tekst '{}' niet naar een geheel getal converteren",
                    s
                ))
            }),
        },
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
        Value::Text(s) => parse_boolean_text(s.as_str()).ok_or_else(|| {
            ComponentError::new(format!(
                "Kon tekst '{}' niet naar een booleaanse waarde converteren",
                s
            ))
        }),
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

pub fn coerce_vector(value: &Value, context: &str) -> Result<[f64; 3], ComponentError> {
    match value {
        Value::Vector(vector) => Ok(*vector),
        Value::Point(point) => Ok(*point),
        Value::List(values) if values.len() == 1 => coerce_vector(&values[0], context),
        Value::List(values) if values.len() >= 3 => {
            let x = coerce_number(values.get(0).unwrap(), Some(context))?;
            let y = coerce_number(values.get(1).unwrap(), Some(context))?;
            let z = coerce_number(values.get(2).unwrap(), Some(context))?;
            Ok([x, y, z])
        }
        Value::List(values) if values.len() == 2 => {
            let x = coerce_number(values.get(0).unwrap(), Some(context))?;
            let y = coerce_number(values.get(1).unwrap(), Some(context))?;
            Ok([x, y, 0.0])
        }
        Value::Number(number) => Ok([0.0, 0.0, *number]),
        other => Err(ComponentError::new(format!(
            "{} verwacht een vector, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

pub fn coerce_point_with_context(value: &Value, context: &str) -> Result<[f64; 3], ComponentError> {
    match value {
        Value::Point(point) => Ok(*point),
        Value::Vector(vector) => Ok(*vector),
        Value::List(values) if values.len() == 1 => coerce_point_with_context(&values[0], context),
        Value::List(values) if values.len() >= 3 => {
            let x = coerce_number(values.get(0).unwrap(), Some(context))?;
            let y = coerce_number(values.get(1).unwrap(), Some(context))?;
            let z = coerce_number(values.get(2).unwrap(), Some(context))?;
            Ok([x, y, z])
        }
        other => Err(ComponentError::new(format!(
            "{} verwacht een punt, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

pub fn coerce_vector_list(value: &Value, context: &str) -> Result<Vec<[f64; 3]>, ComponentError> {
    match value {
        Value::List(values) => {
            let mut result = Vec::new();
            for entry in values {
                match coerce_vector(entry, context) {
                    Ok(vector) => result.push(vector),
                    Err(_) => {
                        if let Value::List(nested) = entry {
                            if let Ok(vector) = coerce_vector(&Value::List(nested.clone()), context)
                            {
                                result.push(vector);
                                continue;
                            }
                        }
                        return Err(ComponentError::new(format!(
                            "{} verwacht een lijst van vectoren",
                            context
                        )));
                    }
                }
            }
            Ok(result)
        }
        other => Ok(vec![coerce_vector(other, context)?]),
    }
}

impl Plane {
    fn normalize_axes(
        origin: [f64; 3],
        x_axis: [f64; 3],
        y_axis: [f64; 3],
        z_axis: [f64; 3],
    ) -> Self {
        let z = safe_normalized(z_axis)
            .map(|(vector, _)| vector)
            .unwrap_or([0.0, 0.0, 1.0]);

        let mut x = safe_normalized(x_axis)
            .map(|(vector, _)| vector)
            .unwrap_or_else(|| orthogonal_vector(z));

        let mut y = safe_normalized(y_axis)
            .map(|(vector, _)| vector)
            .unwrap_or_else(|| normalize(cross(z, x)));

        x = normalize(cross(y, z));
        y = normalize(cross(z, x));

        Self {
            origin,
            x_axis: x,
            y_axis: y,
            z_axis: z,
        }
    }

    fn from_points(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> Self {
        let ab = subtract(b, a);
        let ac = subtract(c, a);
        let normal = cross(ab, ac);
        if vector_length_squared(normal) < EPSILON {
            return Self::default();
        }
        let x_axis = if vector_length_squared(ab) < EPSILON {
            orthogonal_vector(normal)
        } else {
            normalize(ab)
        };
        let z_axis = normalize(normal);
        let y_axis = normalize(cross(z_axis, x_axis));
        Self::normalize_axes(a, x_axis, y_axis, z_axis)
    }

    #[must_use]
    pub fn to_value(self) -> PlaneValue {
        PlaneValue::new(self.origin, self.x_axis, self.y_axis, self.z_axis)
    }
}

pub fn coerce_plane(value: &Value, context: &str) -> Result<Plane, ComponentError> {
    match value {
        Value::List(values) if values.len() >= 3 => {
            let a = coerce_point_with_context(&values[0], context)?;
            let b = coerce_point_with_context(&values[1], context)?;
            let c = coerce_point_with_context(&values[2], context)?;
            Ok(Plane::from_points(a, b, c))
        }
        Value::List(values) if values.len() == 2 => {
            let origin = coerce_point_with_context(&values[0], context)?;
            let direction = coerce_vector(&values[1], context)?;
            if vector_length_squared(direction) < EPSILON {
                Ok(Plane::default())
            } else {
                let x_axis = normalize(direction);
                let z_axis = orthogonal_vector(direction);
                let y_axis = normalize(cross(z_axis, x_axis));
                Ok(Plane::normalize_axes(origin, x_axis, y_axis, z_axis))
            }
        }
        Value::List(values) if values.len() == 1 => coerce_plane(&values[0], context),
        Value::Point(point) => {
            let mut plane = Plane::default();
            plane.origin = *point;
            Ok(plane)
        }
        Value::Vector(vector) => {
            let normal = if vector_length_squared(*vector) < EPSILON {
                [0.0, 0.0, 1.0]
            } else {
                normalize(*vector)
            };
            let x_axis = orthogonal_vector(normal);
            let y_axis = normalize(cross(normal, x_axis));
            Ok(Plane::normalize_axes(
                [0.0, 0.0, 0.0],
                x_axis,
                y_axis,
                normal,
            ))
        }
        other => Err(ComponentError::new(format!(
            "{} verwacht een vlak, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

pub fn coerce_geo_location(value: &Value, context: &str) -> Result<(f64, f64), ComponentError> {
    match value {
        Value::Vector(vector) | Value::Point(vector) => Ok((vector[0], vector[1])),
        Value::List(values) if !values.is_empty() => {
            let longitude = coerce_number(values.get(0).unwrap(), Some(context))?;
            let latitude = if values.len() > 1 {
                coerce_number(values.get(1).unwrap(), Some(context))?
            } else {
                0.0
            };
            Ok((longitude, latitude))
        }
        Value::List(values) if values.len() == 1 => coerce_geo_location(&values[0], context),
        Value::Number(number) => Ok((0.0, *number)),
        other => Err(ComponentError::new(format!(
            "{} verwacht een locatie, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

pub fn coerce_line(value: &Value, context: &str) -> Result<Line, ComponentError> {
    match value {
        Value::CurveLine { p1, p2 } => Ok(Line {
            start: *p1,
            end: *p2,
        }),
        Value::List(values) if values.len() >= 2 => {
            let start = coerce_point_with_context(&values[0], context)?;
            let mut end = coerce_point_with_context(&values[1], context)?;
            if vector_length_squared(subtract(end, start)) < EPSILON && values.len() > 2 {
                end = add(start, coerce_vector(&values[2], context)?);
            }
            Ok(Line { start, end })
        }
        Value::List(values) if values.len() == 1 => coerce_line(&values[0], context),
        other => Err(ComponentError::new(format!(
            "{} verwacht een curve, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

pub fn coerce_date_time(value: &Value) -> PrimitiveDateTime {
    if let Value::DateTime(date_time) = value {
        return date_time.primitive();
    }
    default_datetime()
}

pub fn default_datetime() -> PrimitiveDateTime {
    let date = Date::from_calendar_date(2020, Month::January, 1).unwrap();
    let time = Time::from_hms(12, 0, 0).unwrap();
    PrimitiveDateTime::new(date, time)
}

const EPSILON: f64 = 1e-9;

fn clamp_to_unit(value: f64) -> f64 {
    value.max(-1.0).min(1.0)
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn subtract(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(vector: [f64; 3], factor: f64) -> [f64; 3] {
    [vector[0] * factor, vector[1] * factor, vector[2] * factor]
}

fn vector_length(vector: [f64; 3]) -> f64 {
    vector_length_squared(vector).sqrt()
}

fn vector_length_squared(vector: [f64; 3]) -> f64 {
    dot(vector, vector)
}

fn normalize(vector: [f64; 3]) -> [f64; 3] {
    if let Some((normalized, _)) = safe_normalized(vector) {
        normalized
    } else {
        [0.0, 0.0, 0.0]
    }
}

fn orthogonal_vector(vector: [f64; 3]) -> [f64; 3] {
    let abs_x = vector[0].abs();
    let abs_y = vector[1].abs();
    let abs_z = vector[2].abs();
    if abs_x <= abs_y && abs_x <= abs_z {
        normalize([0.0, -vector[2], vector[1]])
    } else if abs_y <= abs_x && abs_y <= abs_z {
        normalize([-vector[2], 0.0, vector[0]])
    } else {
        normalize([-vector[1], vector[0], 0.0])
    }
}

fn safe_normalized(vector: [f64; 3]) -> Option<([f64; 3], f64)> {
    let length = vector_length(vector);
    if length < EPSILON {
        None
    } else {
        Some((scale(vector, 1.0 / length), length))
    }
}

pub fn coerce_number_with_default(value: Option<&Value>) -> f64 {
    match value {
        Some(Value::Null) => 0.0,
        Some(v) => coerce_number(v, None).unwrap_or(0.0),
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

pub fn parse_boolean_text(input: &str) -> Option<bool> {
    match input.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "y" | "on" => Some(true),
        "false" | "0" | "no" | "n" | "off" => Some(false),
        _ => None,
    }
}

pub fn coerce_point_with_default(value: Option<&Value>) -> [f64; 3] {
    match value {
        Some(Value::Null) => [0.0, 0.0, 0.0],
        Some(Value::List(values)) => {
            for entry in values {
                if let Ok(point) = coerce_point_with_context(entry, "point") {
                    return point;
                }
            }
            [0.0, 0.0, 0.0]
        }
        Some(v) => coerce_point_with_context(v, "point").unwrap_or([0.0, 0.0, 0.0]),
        None => [0.0, 0.0, 0.0],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_boolean_text_accepts_numeric_forms() {
        assert_eq!(parse_boolean_text("1"), Some(true));
        assert_eq!(parse_boolean_text("0"), Some(false));
        assert_eq!(parse_boolean_text(" true "), Some(true));
    }

    #[test]
    fn coerce_boolean_accepts_numeric_strings() {
        assert_eq!(coerce_boolean(&Value::Text("1".to_owned())).unwrap(), true);
        assert_eq!(coerce_boolean(&Value::Text("0".to_owned())).unwrap(), false);
    }

    #[test]
    fn coerce_integer_handles_text_booleans_and_numbers() {
        assert_eq!(coerce_integer(&Value::Text("True".to_owned())).unwrap(), 1);
        assert_eq!(coerce_integer(&Value::Text("0".to_owned())).unwrap(), 0);
        assert_eq!(coerce_integer(&Value::Text("2.4".to_owned())).unwrap(), 2);
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
                if let (Ok(p1), Ok(p2), Ok(p3)) = (
                    coerce_point(&l[0]),
                    coerce_point(&l[1]),
                    coerce_point(&l[2]),
                ) {
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
    PlaneValue::new(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    )
}
