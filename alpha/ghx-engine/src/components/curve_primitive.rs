//! Implementaties van Grasshopper "Curve → Primitive" componenten.

use std::collections::BTreeMap;
use std::f64::consts::TAU;

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

const PIN_OUTPUT_CIRCLE: &str = "C";

/// Beschikbare componenten binnen deze module.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    Circle,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Volledige lijst van componentregistraties voor de curve-primitive componenten.
pub const REGISTRATIONS: &[Registration] = &[Registration {
    guids: &["{807b86e3-be8d-4970-92b5-f8cdcb45b06b}"],
    names: &["Circle", "Cir"],
    kind: ComponentKind::Circle,
}];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::Circle => evaluate_circle(inputs),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Circle => "Circle",
        }
    }
}

fn evaluate_circle(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Circle component vereist een vlak en straal",
        ));
    }

    let plane = parse_plane(inputs.get(0), "Circle")?;
    let radius = coerce_number(inputs.get(1), "Circle")?;

    if radius <= 0.0 {
        return Err(ComponentError::new(
            "Circle component vereist een straal groter dan nul",
        ));
    }

    let points = sample_circle_points(&plane, radius, 64);

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_CIRCLE.to_owned(),
        Value::List(points.into_iter().map(Value::Point).collect()),
    );
    Ok(outputs)
}

fn sample_circle_points(plane: &Plane, radius: f64, segments: usize) -> Vec<[f64; 3]> {
    let mut points = Vec::with_capacity(segments + 1);
    let step = TAU / segments as f64;
    for i in 0..segments {
        let angle = i as f64 * step;
        let point = plane.apply(radius * angle.cos(), radius * angle.sin());
        points.push(point);
    }
    if let Some(first) = points.first().copied() {
        points.push(first);
    }
    points
}

fn coerce_number(value: Option<&Value>, context: &str) -> Result<f64, ComponentError> {
    let value = value.ok_or_else(|| {
        ComponentError::new(format!("{} vereist minimaal één numerieke invoer", context))
    })?;
    match value {
        Value::Number(number) => Ok(*number),
        Value::List(values) if values.len() == 1 => coerce_number(values.get(0), context),
        other => Err(ComponentError::new(format!(
            "{} verwacht een numerieke waarde, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn parse_plane(value: Option<&Value>, context: &str) -> Result<Plane, ComponentError> {
    let Some(value) = value else {
        return Ok(Plane::default());
    };
    match value {
        Value::List(values) if values.len() >= 3 => {
            let origin = coerce_point(&values[0], context)?;
            let point_x = coerce_point(&values[1], context)?;
            let point_y = coerce_point(&values[2], context)?;
            Ok(Plane::from_points(origin, point_x, point_y))
        }
        Value::Point(point) => Ok(Plane::from_origin(*point)),
        Value::List(values) if values.len() == 1 => parse_plane(values.get(0), context),
        other => Err(ComponentError::new(format!(
            "{} verwacht een vlak, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_point(value: &Value, context: &str) -> Result<[f64; 3], ComponentError> {
    match value {
        Value::Point(point) => Ok(*point),
        Value::List(values) if values.len() == 1 => coerce_point(&values[0], context),
        other => Err(ComponentError::new(format!(
            "{} verwacht punten, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

#[derive(Debug, Clone, Copy)]
struct Plane {
    origin: [f64; 3],
    x_axis: [f64; 3],
    y_axis: [f64; 3],
    _z_axis: [f64; 3],
}

impl Default for Plane {
    fn default() -> Self {
        Self {
            origin: [0.0, 0.0, 0.0],
            x_axis: [1.0, 0.0, 0.0],
            y_axis: [0.0, 1.0, 0.0],
            _z_axis: [0.0, 0.0, 1.0],
        }
    }
}

impl Plane {
    fn from_origin(origin: [f64; 3]) -> Self {
        Self {
            origin,
            ..Self::default()
        }
    }

    fn from_points(origin: [f64; 3], point_x: [f64; 3], point_y: [f64; 3]) -> Self {
        let x_axis = subtract(point_x, origin);
        let y_axis = subtract(point_y, origin);
        let z_axis = cross(x_axis, y_axis);
        Self::normalize_axes(origin, x_axis, y_axis, z_axis)
    }

    fn normalize_axes(
        origin: [f64; 3],
        x_axis: [f64; 3],
        y_axis: [f64; 3],
        z_axis: [f64; 3],
    ) -> Self {
        let z_axis = safe_normalized(z_axis)
            .map(|(vector, _)| vector)
            .unwrap_or([0.0, 0.0, 1.0]);

        let mut x_axis = safe_normalized(x_axis)
            .map(|(vector, _)| vector)
            .unwrap_or_else(|| orthogonal_vector(z_axis));

        let mut y_axis = safe_normalized(y_axis)
            .map(|(vector, _)| vector)
            .unwrap_or_else(|| normalize(cross(z_axis, x_axis)));

        let x_cross = cross(y_axis, z_axis);
        if vector_length_squared(x_cross) < EPSILON {
            x_axis = orthogonal_vector(z_axis);
        } else {
            x_axis = normalize(x_cross);
        }

        let y_cross = cross(z_axis, x_axis);
        if vector_length_squared(y_cross) < EPSILON {
            y_axis = orthogonal_vector(x_axis);
        } else {
            y_axis = normalize(y_cross);
        }

        Self {
            origin,
            x_axis,
            y_axis,
            _z_axis: z_axis,
        }
    }

    fn apply(&self, u: f64, v: f64) -> [f64; 3] {
        add(
            self.origin,
            add(scale(self.x_axis, u), scale(self.y_axis, v)),
        )
    }
}

const EPSILON: f64 = 1e-9;

fn subtract(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(v: [f64; 3], factor: f64) -> [f64; 3] {
    [v[0] * factor, v[1] * factor, v[2] * factor]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn vector_length_squared(v: [f64; 3]) -> f64 {
    dot(v, v)
}

fn vector_length(v: [f64; 3]) -> f64 {
    vector_length_squared(v).sqrt()
}

fn safe_normalized(v: [f64; 3]) -> Option<([f64; 3], f64)> {
    let length = vector_length(v);
    if length < EPSILON {
        None
    } else {
        Some((scale(v, 1.0 / length), length))
    }
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    safe_normalized(v)
        .map(|(vector, _)| vector)
        .unwrap_or([0.0, 0.0, 0.0])
}

fn orthogonal_vector(reference: [f64; 3]) -> [f64; 3] {
    let mut candidate = if reference[0].abs() < reference[1].abs() {
        [0.0, -reference[2], reference[1]]
    } else {
        [-reference[2], 0.0, reference[0]]
    };
    if vector_length_squared(candidate) < EPSILON {
        candidate = [reference[1], -reference[0], 0.0];
    }
    let normalized = normalize(candidate);
    if vector_length_squared(normalized) < EPSILON {
        [1.0, 0.0, 0.0]
    } else {
        normalized
    }
}

#[cfg(test)]
mod tests {
    use super::{Component, ComponentKind, PIN_OUTPUT_CIRCLE};
    use crate::graph::node::MetaMap;
    use crate::graph::value::Value;

    #[test]
    fn circle_requires_plane_and_radius() {
        let component = ComponentKind::Circle;
        let err = component.evaluate(&[], &MetaMap::new()).unwrap_err();
        assert!(err.message().contains("vlak"));
    }

    #[test]
    fn circle_rejects_non_positive_radius() {
        let component = ComponentKind::Circle;
        let err = component
            .evaluate(
                &[
                    Value::List(vec![
                        Value::Point([0.0, 0.0, 0.0]),
                        Value::Point([1.0, 0.0, 0.0]),
                        Value::Point([0.0, 1.0, 0.0]),
                    ]),
                    Value::Number(-1.0),
                ],
                &MetaMap::new(),
            )
            .unwrap_err();
        assert!(err.message().contains("straal"));
    }

    #[test]
    fn circle_generates_points_on_plane() {
        let component = ComponentKind::Circle;
        let outputs = component
            .evaluate(
                &[
                    Value::List(vec![
                        Value::Point([1.0, 2.0, 3.0]),
                        Value::Point([2.0, 2.0, 3.0]),
                        Value::Point([1.0, 3.0, 3.0]),
                    ]),
                    Value::Number(2.0),
                ],
                &MetaMap::new(),
            )
            .expect("circle generated");
        let Some(Value::List(points)) = outputs.get(PIN_OUTPUT_CIRCLE) else {
            panic!("expected list of points");
        };
        assert_eq!(points.len(), 65);
        assert!(matches!(points[0], Value::Point(_)));
        assert!(matches!(points.last(), Some(Value::Point(_))));
    }
}
