//! Implementaties van Grasshopper "Transform → Array" componenten.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

const PIN_OUTPUT_GEOMETRY: &str = "G";
const PIN_OUTPUT_TRANSFORM: &str = "X";

/// Beschikbare componenten binnen Transform → Array.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    BoxArray,
    Kaleidoscope,
    CurveArray,
    RectangularArray,
    LinearArray,
    PolarArray,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Registraties van alle Transform → Array componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{9f6f954c-ba7b-4428-bf1e-1768cdff666c}"],
        names: &["Box Array", "ArrBox"],
        kind: ComponentKind::BoxArray,
    },
    Registration {
        guids: &["{b90eaa92-6e38-4054-a915-afcf486224b3}"],
        names: &["Kaleidoscope", "KScope"],
        kind: ComponentKind::Kaleidoscope,
    },
    Registration {
        guids: &["{c6f23658-617f-4ac8-916d-d0d9e7241b25}"],
        names: &["Curve Array", "ArrCurve"],
        kind: ComponentKind::CurveArray,
    },
    Registration {
        guids: &["{e521f7c8-92f4-481c-888b-eea109e3d6e9}"],
        names: &["Rectangular Array", "ArrRec"],
        kind: ComponentKind::RectangularArray,
    },
    Registration {
        guids: &["{e87db220-a0a0-4d67-a405-f97fd14b2d7a}"],
        names: &["Linear Array", "ArrLinear"],
        kind: ComponentKind::LinearArray,
    },
    Registration {
        guids: &["{fca5ad7e-ecac-401d-a357-edda0a251cbc}"],
        names: &["Polar Array", "ArrPolar"],
        kind: ComponentKind::PolarArray,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        match self {
            Self::BoxArray => evaluate_box_array(inputs, meta),
            Self::Kaleidoscope => evaluate_kaleidoscope(inputs, meta),
            Self::CurveArray => evaluate_curve_array(inputs, meta),
            Self::RectangularArray => evaluate_rectangular_array(inputs, meta),
            Self::LinearArray => evaluate_linear_array(inputs, meta),
            Self::PolarArray => evaluate_polar_array(inputs, meta),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::BoxArray => "Box Array",
            Self::Kaleidoscope => "Kaleidoscope",
            Self::CurveArray => "Curve Array",
            Self::RectangularArray => "Rectangular Array",
            Self::LinearArray => "Linear Array",
            Self::PolarArray => "Polar Array",
        }
    }
}

fn evaluate_box_array(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 5 {
        return Err(ComponentError::new(
            "Box Array vereist geometrie, een cel, en X/Y/Z-aantallen",
        ));
    }

    let geometry = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Box Array vereist geometrie"))?;

    let cell_points = match inputs.get(1) {
        Some(Value::List(pts)) => Ok(pts),
        _ => Err(ComponentError::new(
            "Box Array cel moet een lijst van punten zijn",
        )),
    }?;
    if cell_points.len() < 4 {
        return Err(ComponentError::new(
            "Box Array cel vereist minstens 4 punten",
        ));
    }
    let origin = coerce_point(cell_points.get(0), "Box Array cel oorsprong")?;
    let x_point = coerce_point(cell_points.get(1), "Box Array cel X")?;
    let y_point = coerce_point(cell_points.get(2), "Box Array cel Y")?;
    let z_point = coerce_point(cell_points.get(3), "Box Array cel Z")?;
    let x_axis = subtract(x_point, origin);
    let y_axis = subtract(y_point, origin);
    let z_axis = subtract(z_point, origin);

    let x_count = coerce_number(inputs.get(2), "Box Array X-aantal")? as usize;
    let y_count = coerce_number(inputs.get(3), "Box Array Y-aantal")? as usize;
    let z_count = coerce_number(inputs.get(4), "Box Array Z-aantal")? as usize;

    let mut geometries = Vec::with_capacity(x_count * y_count * z_count);
    let mut transforms = Vec::with_capacity(x_count * y_count * z_count);

    for z in 0..z_count {
        for y in 0..y_count {
            for x in 0..x_count {
                let translation_x = scale(x_axis, x as f64);
                let translation_y = scale(y_axis, y as f64);
                let translation_z = scale(z_axis, z as f64);
                let translation = add(add(translation_x, translation_y), translation_z);

                let mut point_fn = |point: [f64; 3]| add(point, translation);
                let mut vector_fn = |vector: [f64; 3]| vector;
                geometries.push(map_geometry(&geometry, &mut point_fn, &mut vector_fn));
                transforms.push(Value::List(vec![
                    Value::Text("Move".into()),
                    Value::Vector(translation),
                ]));
            }
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_GEOMETRY.to_owned(), Value::List(geometries));
    outputs.insert(PIN_OUTPUT_TRANSFORM.to_owned(), Value::List(transforms));

    Ok(outputs)
}

fn evaluate_kaleidoscope(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Kaleidoscope vereist geometrie, een vlak en een aantal segmenten",
        ));
    }

    let geometry = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Kaleidoscope vereist geometrie"))?;
    let plane = coerce_plane(inputs.get(1), "Kaleidoscope vlak")?;
    let segments = coerce_number(inputs.get(2), "Kaleidoscope segmenten")? as usize;

    let angle_step = 2.0 * std::f64::consts::PI / segments as f64;
    let mut geometries = Vec::with_capacity(segments);
    let mut transforms = Vec::with_capacity(segments);

    for i in 0..segments {
        let current_angle = i as f64 * angle_step;

        let mut final_point_fn = |point: [f64; 3]| {
            let mut transformed_point = point;

            if i % 2 == 1 {
                let local = plane.to_local(transformed_point);
                let mirrored_local = [local[0], -local[1], local[2]];
                transformed_point = plane.from_local(mirrored_local);
            }

            let translated = subtract(transformed_point, plane.origin);
            let rotated = rotate_vector(translated, plane.z_axis, current_angle);
            add(rotated, plane.origin)
        };

        let mut final_vector_fn = |vector: [f64; 3]| {
            let mut transformed_vector = vector;

            if i % 2 == 1 {
                let local = plane.vector_to_local(transformed_vector);
                let mirrored_local = [local[0], -local[1], local[2]];
                transformed_vector = plane.vector_from_local(mirrored_local);
            }

            rotate_vector(transformed_vector, plane.z_axis, current_angle)
        };

        geometries.push(map_geometry(
            &geometry,
            &mut final_point_fn,
            &mut final_vector_fn,
        ));
        transforms.push(Value::List(vec![
            Value::Text("Kaleidoscope".into()),
            Value::Number(i as f64),
        ]));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_GEOMETRY.to_owned(), Value::List(geometries));
    outputs.insert(PIN_OUTPUT_TRANSFORM.to_owned(), Value::List(transforms));

    Ok(outputs)
}

fn evaluate_curve_array(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Curve Array vereist geometrie, een curve en een aantal",
        ));
    }

    let geometry = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Curve Array vereist geometrie"))?;
    let curve = coerce_curve(inputs.get(1), "Curve Array curve")?;
    let count = coerce_number(inputs.get(2), "Curve Array aantal")? as usize;

    let mut geometries = Vec::with_capacity(count);
    let mut transforms = Vec::with_capacity(count);

    for i in 0..count {
        let t = if count > 1 {
            i as f64 / (count - 1) as f64
        } else {
            0.0
        };
        let point = curve.point_at(t);
        let tangent = curve.tangent_at(t);
        let plane = Plane::from_origin_and_normal(point, tangent);

        let mut point_fn = |p: [f64; 3]| {
            let local = Plane::default().to_local(p);
            plane.from_local(local)
        };
        let mut vector_fn = |v: [f64; 3]| {
            let local = Plane::default().vector_to_local(v);
            plane.vector_from_local(local)
        };

        geometries.push(map_geometry(&geometry, &mut point_fn, &mut vector_fn));
        transforms.push(Value::List(vec![
            Value::Text("Orient".into()),
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point(point),
        ]));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_GEOMETRY.to_owned(), Value::List(geometries));
    outputs.insert(PIN_OUTPUT_TRANSFORM.to_owned(), Value::List(transforms));

    Ok(outputs)
}

fn evaluate_rectangular_array(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 4 {
        return Err(ComponentError::new(
            "Rectangular Array vereist geometrie, een cel, en X/Y-aantallen",
        ));
    }

    let geometry = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Rectangular Array vereist geometrie"))?;

    let cell_points = match inputs.get(1) {
        Some(Value::List(pts)) => Ok(pts),
        _ => Err(ComponentError::new(
            "Rectangular Array cel moet een lijst van punten zijn",
        )),
    }?;
    if cell_points.len() < 3 {
        return Err(ComponentError::new(
            "Rectangular Array cel vereist minstens 3 punten",
        ));
    }
    let origin = coerce_point(cell_points.get(0), "Rectangular Array cel oorsprong")?;
    let x_point = coerce_point(cell_points.get(1), "Rectangular Array cel X")?;
    let y_point = coerce_point(cell_points.get(2), "Rectangular Array cel Y")?;
    let x_axis = subtract(x_point, origin);
    let y_axis = subtract(y_point, origin);

    let x_count = coerce_number(inputs.get(2), "Rectangular Array X-aantal")? as usize;
    let y_count = coerce_number(inputs.get(3), "Rectangular Array Y-aantal")? as usize;

    let mut geometries = Vec::with_capacity(x_count * y_count);
    let mut transforms = Vec::with_capacity(x_count * y_count);

    for y in 0..y_count {
        for x in 0..x_count {
            let translation_x = scale(x_axis, x as f64);
            let translation_y = scale(y_axis, y as f64);
            let translation = add(translation_x, translation_y);

            let mut point_fn = |point: [f64; 3]| add(point, translation);
            let mut vector_fn = |vector: [f64; 3]| vector;
            geometries.push(map_geometry(&geometry, &mut point_fn, &mut vector_fn));
            transforms.push(Value::List(vec![
                Value::Text("Move".into()),
                Value::Vector(translation),
            ]));
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_GEOMETRY.to_owned(), Value::List(geometries));
    outputs.insert(PIN_OUTPUT_TRANSFORM.to_owned(), Value::List(transforms));

    Ok(outputs)
}

fn evaluate_linear_array(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Linear Array vereist geometrie, een richting en een aantal",
        ));
    }

    let geometry = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Linear Array vereist geometrie"))?;
    let direction = coerce_vector(inputs.get(1), "Linear Array richting")?;
    let count = coerce_number(inputs.get(2), "Linear Array aantal")? as usize;

    let mut geometries = Vec::with_capacity(count);
    let mut transforms = Vec::with_capacity(count);

    for i in 0..count {
        let translation = scale(direction, i as f64);
        let mut point_fn = |point: [f64; 3]| add(point, translation);
        let mut vector_fn = |vector: [f64; 3]| vector;
        geometries.push(map_geometry(&geometry, &mut point_fn, &mut vector_fn));
        transforms.push(Value::List(vec![
            Value::Text("Move".into()),
            Value::Vector(translation),
        ]));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_GEOMETRY.to_owned(), Value::List(geometries));
    outputs.insert(PIN_OUTPUT_TRANSFORM.to_owned(), Value::List(transforms));

    Ok(outputs)
}

fn evaluate_polar_array(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 4 {
        return Err(ComponentError::new(
            "Polar Array vereist geometrie, een vlak, een aantal en een hoek",
        ));
    }

    let geometry = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Polar Array vereist geometrie"))?;
    let plane = coerce_plane(inputs.get(1), "Polar Array vlak")?;
    let count = coerce_number(inputs.get(2), "Polar Array aantal")? as usize;
    let angle = coerce_number(inputs.get(3), "Polar Array hoek")?;

    let mut geometries = Vec::with_capacity(count);
    let mut transforms = Vec::with_capacity(count);

    for i in 0..count {
        let current_angle = if count > 1 {
            angle * (i as f64 / (count - 1) as f64)
        } else {
            0.0
        };
        let mut point_fn = |point: [f64; 3]| {
            let translated = subtract(point, plane.origin);
            let rotated = rotate_vector(translated, plane.z_axis, current_angle);
            add(rotated, plane.origin)
        };
        let mut vector_fn = |vector: [f64; 3]| rotate_vector(vector, plane.z_axis, current_angle);
        geometries.push(map_geometry(&geometry, &mut point_fn, &mut vector_fn));
        transforms.push(Value::List(vec![
            Value::Text("Rotate".into()),
            Value::Point(plane.origin),
            Value::Vector(plane.z_axis),
            Value::Number(current_angle),
        ]));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_GEOMETRY.to_owned(), Value::List(geometries));
    outputs.insert(PIN_OUTPUT_TRANSFORM.to_owned(), Value::List(transforms));

    Ok(outputs)
}

fn map_geometry<FPoint, FVector>(
    value: &Value,
    point_fn: &mut FPoint,
    vector_fn: &mut FVector,
) -> Value
where
    FPoint: FnMut([f64; 3]) -> [f64; 3],
    FVector: FnMut([f64; 3]) -> [f64; 3],
{
    match value {
        Value::Point(point) => Value::Point(point_fn(*point)),
        Value::Vector(vector) => Value::Vector(vector_fn(*vector)),
        Value::CurveLine { p1, p2 } => Value::CurveLine {
            p1: point_fn(*p1),
            p2: point_fn(*p2),
        },
        Value::Surface { vertices, faces } => Value::Surface {
            vertices: vertices.iter().map(|v| point_fn(*v)).collect(),
            faces: faces.clone(),
        },
        Value::List(values) => {
            let mut mapped = Vec::with_capacity(values.len());
            for value in values {
                mapped.push(map_geometry(value, point_fn, vector_fn));
            }
            Value::List(mapped)
        }
        _ => value.clone(),
    }
}

#[derive(Debug, Clone, Copy)]
struct Plane {
    origin: [f64; 3],
    x_axis: [f64; 3],
    y_axis: [f64; 3],
    z_axis: [f64; 3],
}

impl Plane {
    fn default() -> Self {
        Self {
            origin: [0.0, 0.0, 0.0],
            x_axis: [1.0, 0.0, 0.0],
            y_axis: [0.0, 1.0, 0.0],
            z_axis: [0.0, 0.0, 1.0],
        }
    }

    fn from_points(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> Self {
        let x_axis = safe_normalize(subtract(b, a))
            .map(|(axis, _)| axis)
            .unwrap_or([1.0, 0.0, 0.0]);
        let raw_y = subtract(c, a);
        let y_projection = subtract(raw_y, scale(x_axis, dot(raw_y, x_axis)));
        let y_axis = safe_normalize(y_projection)
            .map(|(axis, _)| axis)
            .unwrap_or([0.0, 1.0, 0.0]);
        let z_axis = normalize(cross(x_axis, y_axis));
        Self::normalize_axes(a, x_axis, y_axis, z_axis)
    }

    fn from_origin(origin: [f64; 3]) -> Self {
        Self {
            origin,
            ..Self::default()
        }
    }

    fn from_origin_and_normal(origin: [f64; 3], normal: [f64; 3]) -> Self {
        let x_axis = orthogonal_vector(normal);
        let y_axis = normalize(cross(normal, x_axis));
        Self::normalize_axes(origin, x_axis, y_axis, normal)
    }

    fn normalize_axes(
        origin: [f64; 3],
        x_axis: [f64; 3],
        y_axis: [f64; 3],
        z_axis: [f64; 3],
    ) -> Self {
        let x_axis = normalize(x_axis);
        let mut y_axis = subtract(y_axis, scale(x_axis, dot(y_axis, x_axis)));
        if length_squared(y_axis) < 1e-9 {
            y_axis = orthogonal_vector(x_axis);
        }
        let y_axis = normalize(y_axis);
        let z_axis = normalize(z_axis);
        Self {
            origin,
            x_axis,
            y_axis,
            z_axis,
        }
    }

    fn to_local(&self, point: [f64; 3]) -> [f64; 3] {
        let delta = subtract(point, self.origin);
        [
            dot(delta, self.x_axis),
            dot(delta, self.y_axis),
            dot(delta, self.z_axis),
        ]
    }

    fn from_local(&self, coords: [f64; 3]) -> [f64; 3] {
        add(
            add(
                add(self.origin, scale(self.x_axis, coords[0])),
                scale(self.y_axis, coords[1]),
            ),
            scale(self.z_axis, coords[2]),
        )
    }

    fn vector_to_local(&self, vector: [f64; 3]) -> [f64; 3] {
        [
            dot(vector, self.x_axis),
            dot(vector, self.y_axis),
            dot(vector, self.z_axis),
        ]
    }

    fn vector_from_local(&self, coords: [f64; 3]) -> [f64; 3] {
        add(
            add(scale(self.x_axis, coords[0]), scale(self.y_axis, coords[1])),
            scale(self.z_axis, coords[2]),
        )
    }
}

trait Curve {
    fn point_at(&self, t: f64) -> [f64; 3];
    fn tangent_at(&self, t: f64) -> [f64; 3];
}

struct LineCurve {
    p1: [f64; 3],
    p2: [f64; 3],
}

impl Curve for LineCurve {
    fn point_at(&self, t: f64) -> [f64; 3] {
        add(scale(self.p1, 1.0 - t), scale(self.p2, t))
    }

    fn tangent_at(&self, _t: f64) -> [f64; 3] {
        normalize(subtract(self.p2, self.p1))
    }
}

fn coerce_curve(value: Option<&Value>, context: &str) -> Result<Box<dyn Curve>, ComponentError> {
    match value {
        Some(Value::CurveLine { p1, p2 }) => Ok(Box::new(LineCurve { p1: *p1, p2: *p2 })),
        Some(other) => Err(ComponentError::new(format!(
            "{} verwacht een curve, kreeg {}",
            context,
            other.kind()
        ))),
        None => Err(ComponentError::new(format!(
            "{} vereist een curve",
            context
        ))),
    }
}

fn coerce_plane(value: Option<&Value>, context: &str) -> Result<Plane, ComponentError> {
    match value {
        None => Ok(Plane::default()),
        Some(Value::List(values)) if values.len() >= 3 => {
            let a = coerce_point(values.get(0), context)?;
            let b = coerce_point(values.get(1), context)?;
            let c = coerce_point(values.get(2), context)?;
            Ok(Plane::from_points(a, b, c))
        }
        Some(Value::List(values)) if values.len() == 2 => {
            let origin = coerce_point(values.get(0), context)?;
            let direction = coerce_vector(values.get(1), context)?;
            if length_squared(direction) < 1e-9 {
                Ok(Plane::from_origin(origin))
            } else {
                let x_axis = normalize(direction);
                let y_axis = orthogonal_vector(x_axis);
                let z_axis = normalize(cross(x_axis, y_axis));
                Ok(Plane::normalize_axes(origin, x_axis, y_axis, z_axis))
            }
        }
        Some(Value::List(values)) if values.len() == 1 => coerce_plane(values.get(0), context),
        Some(Value::Point(point)) => Ok(Plane::from_origin(*point)),
        Some(Value::Vector(vector)) => {
            let normal = if length_squared(*vector) < 1e-9 {
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
        Some(other) => Err(ComponentError::new(format!(
            "{} verwacht een vlak, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_point(value: Option<&Value>, context: &str) -> Result<[f64; 3], ComponentError> {
    match value {
        Some(Value::Point(point)) => Ok(*point),
        Some(Value::Vector(vector)) => Ok(*vector),
        Some(Value::List(values)) if values.len() == 1 => coerce_point(values.get(0), context),
        Some(Value::List(values)) if values.len() >= 3 => {
            let x = coerce_number(values.get(0), context)?;
            let y = coerce_number(values.get(1), context)?;
            let z = coerce_number(values.get(2), context)?;
            Ok([x, y, z])
        }
        Some(other) => Err(ComponentError::new(format!(
            "{} verwacht een punt, kreeg {}",
            context,
            other.kind()
        ))),
        None => Err(ComponentError::new(format!("{} vereist een punt", context))),
    }
}

fn coerce_vector(value: Option<&Value>, context: &str) -> Result<[f64; 3], ComponentError> {
    match value {
        Some(Value::Vector(vector)) => Ok(*vector),
        Some(Value::Point(point)) => Ok(*point),
        Some(Value::List(values)) if values.len() == 1 => coerce_vector(values.get(0), context),
        Some(Value::List(values)) if values.len() >= 3 => {
            let x = coerce_number(values.get(0), context)?;
            let y = coerce_number(values.get(1), context)?;
            let z = coerce_number(values.get(2), context)?;
            Ok([x, y, z])
        }
        Some(other) => Err(ComponentError::new(format!(
            "{} verwacht een vector, kreeg {}",
            context,
            other.kind()
        ))),
        None => Err(ComponentError::new(format!(
            "{} vereist een vector",
            context
        ))),
    }
}

fn coerce_number(value: Option<&Value>, context: &str) -> Result<f64, ComponentError> {
    match value {
        Some(Value::Number(number)) => Ok(*number),
        Some(Value::Boolean(flag)) => Ok(if *flag { 1.0 } else { 0.0 }),
        Some(Value::List(values)) if !values.is_empty() => coerce_number(values.get(0), context),
        Some(other) => Err(ComponentError::new(format!(
            "{} verwacht een numerieke waarde, kreeg {}",
            context,
            other.kind()
        ))),
        None => Err(ComponentError::new(format!(
            "{} vereist een numerieke waarde",
            context
        ))),
    }
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

fn length_squared(vector: [f64; 3]) -> f64 {
    dot(vector, vector)
}

fn length(vector: [f64; 3]) -> f64 {
    length_squared(vector).sqrt()
}

fn normalize(vector: [f64; 3]) -> [f64; 3] {
    safe_normalize(vector)
        .map(|(unit, _)| unit)
        .unwrap_or([1.0, 0.0, 0.0])
}

fn safe_normalize(vector: [f64; 3]) -> Option<([f64; 3], f64)> {
    let length = length(vector);
    if length < 1e-9 {
        None
    } else {
        Some((
            [vector[0] / length, vector[1] / length, vector[2] / length],
            length,
        ))
    }
}

fn orthogonal_vector(vector: [f64; 3]) -> [f64; 3] {
    if vector[0].abs() < vector[1].abs() && vector[0].abs() < vector[2].abs() {
        normalize([0.0, -vector[2], vector[1]])
    } else if vector[1].abs() < vector[2].abs() {
        normalize([-vector[2], 0.0, vector[0]])
    } else {
        normalize([-vector[1], vector[0], 0.0])
    }
}

fn rotate_vector(vector: [f64; 3], axis: [f64; 3], angle: f64) -> [f64; 3] {
    if angle.abs() < 1e-9 {
        return vector;
    }
    let axis = normalize(axis);
    let cos_angle = angle.cos();
    let sin_angle = angle.sin();
    add(
        add(
            scale(vector, cos_angle),
            scale(cross(axis, vector), sin_angle),
        ),
        scale(axis, dot(axis, vector) * (1.0 - cos_angle)),
    )
}

#[cfg(test)]
mod tests {
    use super::{Component, ComponentKind};
    use crate::graph::node::MetaMap;
    use crate::graph::value::Value;

    #[test]
    fn linear_array_point() {
        let component = ComponentKind::LinearArray;
        let outputs = component
            .evaluate(
                &[
                    Value::Point([1.0, 2.0, 3.0]),
                    Value::Vector([10.0, 20.0, 30.0]),
                    Value::Number(3.0),
                ],
                &MetaMap::new(),
            )
            .expect("linear array");

        let Value::List(geometries) = outputs
            .get(super::PIN_OUTPUT_GEOMETRY)
            .cloned()
            .expect("geometry output")
        else {
            panic!("expected list of geometries");
        };

        assert_eq!(geometries.len(), 3);

        let Value::Point(p2) = geometries[2] else {
            panic!("expected point")
        };
        assert!((p2[0] - 21.0).abs() < 1e-6);
    }

    #[test]
    fn rectangular_array_point() {
        let component = ComponentKind::RectangularArray;
        let outputs = component
            .evaluate(
                &[
                    Value::Point([0.0, 0.0, 0.0]),
                    Value::List(vec![
                        Value::Point([0.0, 0.0, 0.0]),
                        Value::Point([10.0, 0.0, 0.0]),
                        Value::Point([0.0, 5.0, 0.0]),
                    ]),
                    Value::Number(2.0),
                    Value::Number(3.0),
                ],
                &MetaMap::new(),
            )
            .expect("rectangular array");

        let Value::List(geometries) = outputs
            .get(super::PIN_OUTPUT_GEOMETRY)
            .cloned()
            .expect("geometry output")
        else {
            panic!("expected list of geometries");
        };

        assert_eq!(geometries.len(), 6);

        let Value::Point(p5) = geometries[5] else {
            panic!("expected point")
        };
        assert!((p5[0] - 10.0).abs() < 1e-6 && (p5[1] - 10.0).abs() < 1e-6);
    }

    #[test]
    fn polar_array_point() {
        let component = ComponentKind::PolarArray;
        let outputs = component
            .evaluate(
                &[
                    Value::Point([10.0, 0.0, 0.0]),
                    Value::List(vec![
                        Value::Point([0.0, 0.0, 0.0]),
                        Value::Point([1.0, 0.0, 0.0]),
                        Value::Point([0.0, 1.0, 0.0]),
                    ]),
                    Value::Number(3.0),
                    Value::Number(std::f64::consts::PI),
                ],
                &MetaMap::new(),
            )
            .expect("polar array");

        let Value::List(geometries) = outputs
            .get(super::PIN_OUTPUT_GEOMETRY)
            .cloned()
            .expect("geometry output")
        else {
            panic!("expected list of geometries");
        };

        assert_eq!(geometries.len(), 3);

        let Value::Point(p2) = geometries[2] else {
            panic!("expected point")
        };
        assert!((p2[0] - -10.0).abs() < 1e-6 && (p2[1] - 0.0).abs() < 1e-7);
    }

    #[test]
    fn box_array_point() {
        let component = ComponentKind::BoxArray;
        let outputs = component
            .evaluate(
                &[
                    Value::Point([0.0, 0.0, 0.0]),
                    Value::List(vec![
                        Value::Point([0.0, 0.0, 0.0]),
                        Value::Point([10.0, 0.0, 0.0]),
                        Value::Point([0.0, 5.0, 0.0]),
                        Value::Point([0.0, 0.0, 2.0]),
                    ]),
                    Value::Number(2.0),
                    Value::Number(2.0),
                    Value::Number(2.0),
                ],
                &MetaMap::new(),
            )
            .expect("box array");

        let Value::List(geometries) = outputs
            .get(super::PIN_OUTPUT_GEOMETRY)
            .cloned()
            .expect("geometry output")
        else {
            panic!("expected list of geometries");
        };

        assert_eq!(geometries.len(), 8);

        let Value::Point(p7) = geometries[7] else {
            panic!("expected point")
        };
        assert!(
            (p7[0] - 10.0).abs() < 1e-6 && (p7[1] - 5.0).abs() < 1e-6 && (p7[2] - 2.0).abs() < 1e-6
        );
    }

    #[test]
    fn curve_array_point() {
        let component = ComponentKind::CurveArray;
        let outputs = component
            .evaluate(
                &[
                    Value::Point([0.0, 0.0, 0.0]),
                    Value::CurveLine {
                        p1: [0.0, 0.0, 0.0],
                        p2: [10.0, 0.0, 0.0],
                    },
                    Value::Number(3.0),
                ],
                &MetaMap::new(),
            )
            .expect("curve array");

        let Value::List(geometries) = outputs
            .get(super::PIN_OUTPUT_GEOMETRY)
            .cloned()
            .expect("geometry output")
        else {
            panic!("expected list of geometries");
        };

        assert_eq!(geometries.len(), 3);

        let Value::Point(p2) = geometries[2] else {
            panic!("expected point")
        };
        assert!((p2[0] - 10.0).abs() < 1e-6 && (p2[1] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn kaleidoscope_point() {
        let component = ComponentKind::Kaleidoscope;
        let outputs = component
            .evaluate(
                &[
                    Value::Point([10.0, 5.0, 0.0]),
                    Value::List(vec![
                        Value::Point([0.0, 0.0, 0.0]),
                        Value::Point([1.0, 0.0, 0.0]),
                        Value::Point([0.0, 1.0, 0.0]),
                    ]),
                    Value::Number(4.0),
                ],
                &MetaMap::new(),
            )
            .expect("kaleidoscope");

        let Value::List(geometries) = outputs
            .get(super::PIN_OUTPUT_GEOMETRY)
            .cloned()
            .expect("geometry output")
        else {
            panic!("expected list of geometries");
        };

        assert_eq!(geometries.len(), 4);

        let Value::Point(p1) = geometries[1] else {
            panic!("expected point")
        };
        assert!((p1[0] - 5.0).abs() < 1e-6 && (p1[1] - 10.0).abs() < 1e-6);
    }
}
