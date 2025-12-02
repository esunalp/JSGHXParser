//! Implementaties van Grasshopper "Transform → Euclidean" componenten.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult, coerce};

const PIN_OUTPUT_GEOMETRY: &str = "G";
const PIN_OUTPUT_TRANSFORM: &str = "X";

/// Beschikbare componenten binnen Transform → Euclidean.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    Move,
    MoveWithTransform,
    Orient,
    OrientWithTransform,
    Rotate3D,
    Rotate3DWithTransform,
    Rotate,
    RotateWithTransform,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Registraties van alle Transform → Euclidean componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{b40f28a2-ba30-4ac2-afe5-a6ece7f985fc}"],
        names: &["Move"],
        kind: ComponentKind::Move,
    },
    Registration {
        guids: &["{e9eb1dcf-92f6-4d4d-84ae-96222d60f56b}"],
        names: &["Move"],
        kind: ComponentKind::MoveWithTransform,
    },
    Registration {
        guids: &["{a35811bc-1034-4491-acb8-608a8cfa27b1}"],
        names: &["Orient"],
        kind: ComponentKind::Orient,
    },
    Registration {
        guids: &["{378d0690-9da0-4dd1-ab16-1d15246e7c22}"],
        names: &["Orient"],
        kind: ComponentKind::OrientWithTransform,
    },
    Registration {
        guids: &["{955d887b-c83b-4c61-bf35-df5d4c4abd9b}"],
        names: &["Rotate 3D", "Rot3D"],
        kind: ComponentKind::Rotate3D,
    },
    Registration {
        guids: &["{3dfb9a77-6e05-4016-9f20-94f78607d672}"],
        names: &["Rotate 3D", "Rot3D"],
        kind: ComponentKind::Rotate3DWithTransform,
    },
    Registration {
        guids: &["{b661519d-43fd-4e5a-b244-d54d9fae2bde}"],
        names: &["Rotate"],
        kind: ComponentKind::Rotate,
    },
    Registration {
        guids: &["{b7798b74-037e-4f0c-8ac7-dc1043d093e0}"],
        names: &["Rotate"],
        kind: ComponentKind::RotateWithTransform,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::Move => evaluate_move(inputs, false),
            Self::MoveWithTransform => evaluate_move(inputs, true),
            Self::Orient => evaluate_orient(inputs, false),
            Self::OrientWithTransform => evaluate_orient(inputs, true),
            Self::Rotate3D => evaluate_rotate_3d(inputs, false),
            Self::Rotate3DWithTransform => evaluate_rotate_3d(inputs, true),
            Self::Rotate => evaluate_rotate(inputs, false),
            Self::RotateWithTransform => evaluate_rotate(inputs, true),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Move | Self::MoveWithTransform => "Move",
            Self::Orient | Self::OrientWithTransform => "Orient",
            Self::Rotate3D | Self::Rotate3DWithTransform => "Rotate 3D",
            Self::Rotate | Self::RotateWithTransform => "Rotate",
        }
    }
}

fn evaluate_move(inputs: &[Value], include_transform: bool) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Move vereist geometrie en een translatie vector",
        ));
    }

    let geometry = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Move vereist geometrie"))?;
    let translations = coerce_translation_list(inputs.get(1), "Move translatie")?;

    let transformed = if translations.len() == 1 {
        apply_translation(&geometry, translations[0])
    } else if let Value::List(items) = &geometry {
        if items.len() == translations.len() {
            let mapped = items
                .iter()
                .zip(translations.iter())
                .map(|(item, translation)| apply_translation(item, *translation))
                .collect();
            Value::List(mapped)
        } else {
            Value::List(
                translations
                    .iter()
                    .map(|translation| apply_translation(&geometry, *translation))
                    .collect(),
            )
        }
    } else {
        Value::List(
            translations
                .iter()
                .map(|translation| apply_translation(&geometry, *translation))
                .collect(),
        )
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_GEOMETRY.to_owned(), transformed);

    if include_transform {
        outputs.insert(
            PIN_OUTPUT_TRANSFORM.to_owned(),
            if translations.len() == 1 {
                Value::List(vec![
                    Value::Text("Move".into()),
                    Value::Vector(translations[0]),
                ])
            } else {
                Value::List(
                    translations
                        .iter()
                        .map(|translation| {
                            Value::List(vec![
                                Value::Text("Move".into()),
                                Value::Vector(*translation),
                            ])
                        })
                        .collect(),
                )
            },
        );
    }

    Ok(outputs)
}

fn evaluate_orient(inputs: &[Value], include_transform: bool) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Orient vereist geometrie, een bronvlak en een doelvlak",
        ));
    }

    let geometry = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Orient vereist geometrie"))?;
    let source_plane = coerce_plane(inputs.get(1), "Orient bronvlak")?;
    let target_plane = coerce_plane(inputs.get(2), "Orient doelvlak")?;

    let mut point_fn = |point: [f64; 3]| {
        let local = source_plane.to_local(point);
        target_plane.from_local(local)
    };
    let mut vector_fn = |vector: [f64; 3]| {
        let local = source_plane.vector_to_local(vector);
        target_plane.vector_from_local(local)
    };
    let transformed = map_geometry(&geometry, &mut point_fn, &mut vector_fn);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_GEOMETRY.to_owned(), transformed);

    if include_transform {
        outputs.insert(
            PIN_OUTPUT_TRANSFORM.to_owned(),
            Value::List(vec![
                Value::Text("Orient".into()),
                Value::Point(source_plane.origin),
                Value::Point(target_plane.origin),
            ]),
        );
    }

    Ok(outputs)
}

fn evaluate_rotate_3d(inputs: &[Value], include_transform: bool) -> ComponentResult {
    if inputs.len() < 4 {
        return Err(ComponentError::new(
            "Rotate 3D vereist geometrie, een hoek, een centrum en een as",
        ));
    }

    let geometry = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Rotate 3D vereist geometrie"))?;
    let angle = coerce_number(inputs.get(1), "Rotate 3D hoek")?;
    let center = coerce_point(inputs.get(2), "Rotate 3D centrum")?;
    let axis = coerce_vector(inputs.get(3), "Rotate 3D as")?;

    let mut point_fn = |point: [f64; 3]| {
        let translated = subtract(point, center);
        let rotated = rotate_vector(translated, axis, angle);
        add(rotated, center)
    };
    let mut vector_fn = |vector: [f64; 3]| rotate_vector(vector, axis, angle);
    let transformed = map_geometry(&geometry, &mut point_fn, &mut vector_fn);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_GEOMETRY.to_owned(), transformed);

    if include_transform {
        outputs.insert(
            PIN_OUTPUT_TRANSFORM.to_owned(),
            Value::List(vec![
                Value::Text("Rotate 3D".into()),
                Value::Point(center),
                Value::Vector(axis),
                Value::Number(angle),
            ]),
        );
    }

    Ok(outputs)
}

fn evaluate_rotate(inputs: &[Value], include_transform: bool) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Rotate vereist geometrie, een hoek en een vlak",
        ));
    }

    let geometry = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Rotate vereist geometrie"))?;
    let angle = coerce_number(inputs.get(1), "Rotate hoek")?;
    let plane = coerce_plane(inputs.get(2), "Rotate vlak")?;

    let mut point_fn = |point: [f64; 3]| {
        let translated = subtract(point, plane.origin);
        let rotated = rotate_vector(translated, plane.z_axis, angle);
        add(rotated, plane.origin)
    };
    let mut vector_fn = |vector: [f64; 3]| rotate_vector(vector, plane.z_axis, angle);
    let transformed = map_geometry(&geometry, &mut point_fn, &mut vector_fn);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_GEOMETRY.to_owned(), transformed);

    if include_transform {
        outputs.insert(
            PIN_OUTPUT_TRANSFORM.to_owned(),
            Value::List(vec![
                Value::Text("Rotate".into()),
                Value::Point(plane.origin),
                Value::Vector(plane.z_axis),
                Value::Number(angle),
            ]),
        );
    }

    Ok(outputs)
}

// HELPER FUNCTIONS

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

fn apply_translation(value: &Value, translation: [f64; 3]) -> Value {
    let mut point_fn = |point: [f64; 3]| add(point, translation);
    let mut vector_fn = |vector: [f64; 3]| vector;
    map_geometry(value, &mut point_fn, &mut vector_fn)
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

    fn from_coerce_plane(plane: coerce::Plane) -> Self {
        Self::normalize_axes(plane.origin, plane.x_axis, plane.y_axis, plane.z_axis)
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

fn coerce_plane(value: Option<&Value>, context: &str) -> Result<Plane, ComponentError> {
    match value {
        None => Ok(Plane::default()),
        Some(value) => coerce::coerce_plane(value, context).map(Plane::from_coerce_plane),
    }
}

fn coerce_point(value: Option<&Value>, context: &str) -> Result<[f64; 3], ComponentError> {
    match value {
        Some(value) => coerce::coerce_point_with_context(value, context),
        None => Err(ComponentError::new(format!("{} vereist een punt", context))),
    }
}

fn coerce_vector(value: Option<&Value>, context: &str) -> Result<[f64; 3], ComponentError> {
    match value {
        Some(value) => coerce::coerce_vector(value, context),
        None => Err(ComponentError::new(format!(
            "{} vereist een vector",
            context
        ))),
    }
}

fn coerce_translation_list(
    value: Option<&Value>,
    context: &str,
) -> Result<Vec<[f64; 3]>, ComponentError> {
    match value {
        Some(value) => {
            let vectors = coerce::coerce_vector_list(value, context)?;
            if vectors.is_empty() {
                Err(ComponentError::new(format!(
                    "{} vereist een vector",
                    context
                )))
            } else {
                Ok(vectors)
            }
        }
        None => Err(ComponentError::new(format!(
            "{} vereist een vector",
            context
        ))),
    }
}

fn coerce_number(value: Option<&Value>, context: &str) -> Result<f64, ComponentError> {
    match value {
        Some(value) => coerce::coerce_number(value, Some(context)),
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
    fn move_point() {
        let component = ComponentKind::Move;
        let outputs = component
            .evaluate(
                &[
                    Value::Point([1.0, 2.0, 3.0]),
                    Value::Vector([10.0, 20.0, 30.0]),
                ],
                &MetaMap::new(),
            )
            .expect("move");
        let Value::Point(point) = outputs
            .get(super::PIN_OUTPUT_GEOMETRY)
            .cloned()
            .expect("geometry output")
        else {
            panic!("expected point output");
        };
        assert!((point[0] - 11.0).abs() < 1e-6);
        assert!((point[1] - 22.0).abs() < 1e-6);
        assert!((point[2] - 33.0).abs() < 1e-6);
    }

    #[test]
    fn move_line() {
        let component = ComponentKind::Move;
        let outputs = component
            .evaluate(
                &[
                    Value::CurveLine {
                        p1: [0.0, 0.0, 0.0],
                        p2: [1.0, 0.0, 0.0],
                    },
                    Value::Vector([0.0, 10.0, 0.0]),
                ],
                &MetaMap::new(),
            )
            .expect("move");
        let Value::CurveLine { p1, p2 } = outputs
            .get(super::PIN_OUTPUT_GEOMETRY)
            .cloned()
            .expect("geometry output")
        else {
            panic!("expected line output");
        };
        assert!((p1[1] - 10.0).abs() < 1e-6);
        assert!((p2[1] - 10.0).abs() < 1e-6);
    }

    #[test]
    fn move_with_translation_list_single_geometry() {
        let component = ComponentKind::Move;
        let outputs = component
            .evaluate(
                &[
                    Value::Point([1.0, 2.0, 3.0]),
                    Value::List(vec![
                        Value::Vector([0.0, 0.0, 1.0]),
                        Value::Vector([0.0, 0.0, 2.0]),
                    ]),
                ],
                &MetaMap::new(),
            )
            .expect("move");
        let Value::List(result_list) = outputs
            .get(super::PIN_OUTPUT_GEOMETRY)
            .cloned()
            .expect("geometry output")
        else {
            panic!("expected list output");
        };
        assert_eq!(result_list.len(), 2);

        match &result_list[0] {
            Value::Point(point) => assert!((point[2] - 4.0).abs() < 1e-9),
            other => panic!("expected point output, got {}", other.kind()),
        }
        match &result_list[1] {
            Value::Point(point) => assert!((point[2] - 5.0).abs() < 1e-9),
            other => panic!("expected point output, got {}", other.kind()),
        }
    }

    #[test]
    fn move_with_translation_list_zipped_geometry() {
        let component = ComponentKind::Move;
        let outputs = component
            .evaluate(
                &[
                    Value::List(vec![
                        Value::Point([0.0, 0.0, 0.0]),
                        Value::Point([1.0, 0.0, 0.0]),
                    ]),
                    Value::List(vec![
                        Value::Vector([0.0, 1.0, 0.0]),
                        Value::Vector([0.0, 2.0, 0.0]),
                    ]),
                ],
                &MetaMap::new(),
            )
            .expect("move zipped");
        let Value::List(result_list) = outputs
            .get(super::PIN_OUTPUT_GEOMETRY)
            .cloned()
            .expect("geometry output")
        else {
            panic!("expected list output");
        };
        assert_eq!(result_list.len(), 2);

        match &result_list[0] {
            Value::Point(point) => {
                assert!((point[0]).abs() < 1e-9);
                assert!((point[1] - 1.0).abs() < 1e-9);
            }
            other => panic!("expected point output, got {}", other.kind()),
        }

        match &result_list[1] {
            Value::Point(point) => {
                assert!((point[0] - 1.0).abs() < 1e-9);
                assert!((point[1] - 2.0).abs() < 1e-9);
            }
            other => panic!("expected point output, got {}", other.kind()),
        }
    }

    #[test]
    fn orient_point() {
        let component = ComponentKind::Orient;
        let outputs = component
            .evaluate(
                &[
                    Value::Point([1.0, 0.0, 0.0]),
                    Value::List(vec![
                        // XY plane
                        Value::Point([0.0, 0.0, 0.0]),
                        Value::Point([1.0, 0.0, 0.0]),
                        Value::Point([0.0, 1.0, 0.0]),
                    ]),
                    Value::List(vec![
                        // YZ plane
                        Value::Point([0.0, 0.0, 0.0]),
                        Value::Point([0.0, 1.0, 0.0]),
                        Value::Point([0.0, 0.0, 1.0]),
                    ]),
                ],
                &MetaMap::new(),
            )
            .expect("orient");
        let Value::Point(point) = outputs
            .get(super::PIN_OUTPUT_GEOMETRY)
            .cloned()
            .expect("geometry output")
        else {
            panic!("expected point output");
        };
        assert!((point[0]).abs() < 1e-6);
        assert!((point[1] - 1.0).abs() < 1e-6);
        assert!((point[2]).abs() < 1e-6);
    }

    #[test]
    fn rotate_3d_point() {
        let component = ComponentKind::Rotate3D;
        let outputs = component
            .evaluate(
                &[
                    Value::Point([1.0, 0.0, 0.0]),
                    Value::Number(std::f64::consts::PI / 2.0),
                    Value::Point([0.0, 0.0, 0.0]),
                    Value::Vector([0.0, 0.0, 1.0]),
                ],
                &MetaMap::new(),
            )
            .expect("rotate 3d");
        let Value::Point(point) = outputs
            .get(super::PIN_OUTPUT_GEOMETRY)
            .cloned()
            .expect("geometry output")
        else {
            panic!("expected point output");
        };
        assert!((point[0]).abs() < 1e-6);
        assert!((point[1] - 1.0).abs() < 1e-6);
        assert!((point[2]).abs() < 1e-6);
    }

    #[test]
    fn rotate_point_in_plane() {
        let component = ComponentKind::Rotate;
        let outputs = component
            .evaluate(
                &[
                    Value::Point([1.0, 0.0, 0.0]),
                    Value::Number(std::f64::consts::PI / 2.0),
                    Value::List(vec![
                        // XY plane
                        Value::Point([0.0, 0.0, 0.0]),
                        Value::Point([1.0, 0.0, 0.0]),
                        Value::Point([0.0, 1.0, 0.0]),
                    ]),
                ],
                &MetaMap::new(),
            )
            .expect("rotate");
        let Value::Point(point) = outputs
            .get(super::PIN_OUTPUT_GEOMETRY)
            .cloned()
            .expect("geometry output")
        else {
            panic!("expected point output");
        };
        assert!((point[0]).abs() < 1e-6);
        assert!((point[1] - 1.0).abs() < 1e-6);
        assert!((point[2]).abs() < 1e-6);
    }
}
