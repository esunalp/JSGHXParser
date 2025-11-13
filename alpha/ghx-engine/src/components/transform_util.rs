//! Implementaties van Grasshopper "Transform → Util" componenten.

use std::collections::{BTreeMap, HashSet};

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

const PIN_OUTPUT_GROUP: &str = "G";
const PIN_OUTPUT_TRANSFORM: &str = "T";
const PIN_OUTPUT_GEOMETRY: &str = "G";
const PIN_OUTPUT_OBJECTS: &str = "O";
const PIN_OUTPUT_FRAGMENTS: &str = "F";
const PIN_OUTPUT_COMPOUND: &str = "X";
const PIN_OUTPUT_GROUP_A: &str = "A";
const PIN_OUTPUT_GROUP_B: &str = "B";

/// Beschikbare componenten binnen Transform → Util.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    MergeGroup,
    InverseTransform,
    Transform,
    Group,
    Split,
    Ungroup,
    Compound,
    SplitGroup,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Registraties van alle Transform → Util componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["15204c6d-bba8-403d-9e8f-6660ab8e0df5"],
        names: &["Merge Group", "GMerge"],
        kind: ComponentKind::MergeGroup,
    },
    Registration {
        guids: &["51f61166-7202-45aa-9126-3d83055b269e"],
        names: &["Inverse Transform", "Inverse"],
        kind: ComponentKind::InverseTransform,
    },
    Registration {
        guids: &["610e689b-5adc-47b3-af8f-e3a32b7ea341"],
        names: &["Transform"],
        kind: ComponentKind::Transform,
    },
    Registration {
        guids: &["874eebe7-835b-4f4f-9811-97e031c41597"],
        names: &["Group"],
        kind: ComponentKind::Group,
    },
    Registration {
        guids: &["915f8f93-f5d1-4a7b-aecb-c327bab88ffb"],
        names: &["Split"],
        kind: ComponentKind::Split,
    },
    Registration {
        guids: &["a45f59c8-11c1-4ea7-9e10-847061b80d75"],
        names: &["Ungroup"],
        kind: ComponentKind::Ungroup,
    },
    Registration {
        guids: &["ca80054a-cde0-4f69-a132-10502b24866d"],
        names: &["Compound", "Comp"],
        kind: ComponentKind::Compound,
    },
    Registration {
        guids: &["fd03419e-e1cc-4603-8a57-6dfa56ed5dec"],
        names: &["Split Group", "GSplit"],
        kind: ComponentKind::SplitGroup,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        match self {
            Self::MergeGroup => evaluate_merge_group(inputs, meta),
            Self::InverseTransform => evaluate_inverse_transform(inputs, meta),
            Self::Transform => evaluate_transform(inputs, meta),
            Self::Group => evaluate_group(inputs, meta),
            Self::Split => evaluate_split(inputs, meta),
            Self::Ungroup => evaluate_ungroup(inputs, meta),
            Self::Compound => evaluate_compound(inputs, meta),
            Self::SplitGroup => evaluate_split_group(inputs, meta),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::MergeGroup => "Merge Group",
            Self::InverseTransform => "Inverse Transform",
            Self::Transform => "Transform",
            Self::Group => "Group",
            Self::Split => "Split",
            Self::Ungroup => "Ungroup",
            Self::Compound => "Compound",
            Self::SplitGroup => "Split Group",
        }
    }
}

fn evaluate_merge_group(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    let mut merged_objects = Vec::new();

    if let Some(group_a) = inputs.get(0) {
        match group_a {
            Value::List(list) => merged_objects.extend(list.iter().cloned()),
            other => merged_objects.push(other.clone()),
        }
    }
    if let Some(group_b) = inputs.get(1) {
        match group_b {
            Value::List(list) => merged_objects.extend(list.iter().cloned()),
            other => merged_objects.push(other.clone()),
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_GROUP.to_owned(), Value::List(merged_objects));
    Ok(outputs)
}

fn evaluate_inverse_transform(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() != 1 {
        return Err(ComponentError::new(
            "Inverse Transform component expects a single Transform input.",
        ));
    }
    let transform = &inputs[0];

    let inverted_transform = match transform {
        Value::List(list) => {
            if let Some(Value::Text(transform_type)) = list.get(0) {
                match transform_type.as_str() {
                    "Move" => {
                        if let Some(Value::Vector(v)) = list.get(1) {
                            Value::List(vec![
                                Value::Text("Move".into()),
                                Value::Vector([-v[0], -v[1], -v[2]]),
                            ])
                        } else {
                            return Err(ComponentError::new("Invalid 'Move' transform format."));
                        }
                    }
                    "Rotate" => {
                        if let (Some(Value::Point(p)), Some(Value::Vector(a)), Some(Value::Number(angle))) =
                            (list.get(1), list.get(2), list.get(3))
                        {
                            Value::List(vec![
                                Value::Text("Rotate".into()),
                                Value::Point(*p),
                                Value::Vector(*a),
                                Value::Number(-angle),
                            ])
                        } else {
                            return Err(ComponentError::new(
                                "Invalid 'Rotate' transform format.",
                            ));
                        }
                    }
                    _ => return Err(ComponentError::new("Unsupported transform type for inversion.")),
                }
            } else {
                return Err(ComponentError::new("Invalid transform format."));
            }
        }
        _ => return Err(ComponentError::new("Invalid transform input.")),
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_TRANSFORM.to_owned(), inverted_transform);
    Ok(outputs)
}

fn evaluate_transform(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Transform component requires Geometry and Transform inputs."));
    }
    let geometry = &inputs[0];
    let transform = &inputs[1];

    let transformed_geometry = apply_transform(geometry, transform)?;

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_GEOMETRY.to_owned(), transformed_geometry);
    Ok(outputs)
}


fn evaluate_group(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    let objects_to_group = if let Some(Value::List(list)) = inputs.get(0) {
        list.clone()
    } else {
        inputs.to_vec()
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_GROUP.to_owned(), Value::List(objects_to_group));
    Ok(outputs)
}

fn evaluate_split(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() != 1 {
        return Err(ComponentError::new("Split component expects a single Transform input."));
    }
    let compound_transform = &inputs[0];

    let fragments = match compound_transform {
        Value::List(list) => list.clone(),
        _ => return Err(ComponentError::new("Invalid compound transform input.")),
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_FRAGMENTS.to_owned(), Value::List(fragments));
    Ok(outputs)
}


fn evaluate_ungroup(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() != 1 {
        return Err(ComponentError::new(
            "Ungroup component expects a single Group input.",
        ));
    }
    let group_to_ungroup = &inputs[0];

    let objects = match group_to_ungroup {
        Value::List(list) => list.clone(),
        other => vec![other.clone()],
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_OBJECTS.to_owned(), Value::List(objects));
    Ok(outputs)
}

fn evaluate_compound(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() != 1 {
        return Err(ComponentError::new("Compound component expects a single list of Transforms."));
    }
    let transforms = match &inputs[0] {
        Value::List(list) => list,
        _ => return Err(ComponentError::new("Invalid input for Compound component.")),
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_COMPOUND.to_owned(), Value::List(transforms.clone()));
    Ok(outputs)
}

fn evaluate_split_group(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Split Group requires Group and Indices inputs.",
        ));
    }
    let group = match inputs.get(0) {
        Some(Value::List(list)) => list,
        _ => {
            return Err(ComponentError::new(
                "Split Group input 'G' must be a list.",
            ))
        }
    };
    let indices = match inputs.get(1) {
        Some(Value::List(list)) => list,
        _ => {
            return Err(ComponentError::new(
                "Split Group input 'I' must be a list.",
            ))
        }
    };
    let wrap = match inputs.get(2) {
        Some(value) => coerce_number(Some(value), "Wrap")? != 0.0,
        None => false,
    };

    let mut split_indices = HashSet::new();
    for index_val in indices {
        let mut index = coerce_number(Some(index_val), "Index")? as isize;
        if wrap {
            if !group.is_empty() {
                index %= group.len() as isize;
                if index < 0 {
                    index += group.len() as isize;
                }
            } else {
                index = 0;
            }
        }
        if index >= 0 && (index as usize) < group.len() {
            split_indices.insert(index as usize);
        }
    }

    let mut group_a = vec![];
    let mut group_b = vec![];

    for (i, item) in group.iter().enumerate() {
        if split_indices.contains(&i) {
            group_a.push(item.clone());
        } else {
            group_b.push(item.clone());
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_GROUP_A.to_owned(), Value::List(group_a));
    outputs.insert(PIN_OUTPUT_GROUP_B.to_owned(), Value::List(group_b));
    Ok(outputs)
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

fn apply_transform(geometry: &Value, transform: &Value) -> Result<Value, ComponentError> {
    match transform {
        Value::List(list) => {
            if let Some(Value::Text(transform_type)) = list.get(0) {
                match transform_type.as_str() {
                    "Move" => {
                        if let Some(Value::Vector(v)) = list.get(1) {
                            Ok(map_geometry(geometry, &mut |p| add(p, *v), &mut |vec| vec))
                        } else {
                            Err(ComponentError::new("Invalid 'Move' transform format."))
                        }
                    }
                    "Rotate" => {
                        if let (Some(Value::Point(p)), Some(Value::Vector(a)), Some(Value::Number(angle))) =
                            (list.get(1), list.get(2), list.get(3))
                        {
                            let mut point_fn = |point: [f64; 3]| {
                                let translated = subtract(point, *p);
                                let rotated = rotate_vector(translated, *a, *angle);
                                add(rotated, *p)
                            };
                             Ok(map_geometry(geometry, &mut point_fn, &mut |vec| rotate_vector(vec, *a, *angle)))
                        } else {
                             Err(ComponentError::new("Invalid 'Rotate' transform format."))
                        }
                    }
                    _ => Err(ComponentError::new("Unsupported transform type.")),
                }
            } else {
                Err(ComponentError::new("Invalid transform format."))
            }
        }
        _ => Err(ComponentError::new("Invalid transform input.")),
    }
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
    let len = length(vector);
    if len.abs() < 1e-9 {
        vector
    } else {
        scale(vector, 1.0 / len)
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
    use super::{Component, ComponentKind, PIN_OUTPUT_GROUP, PIN_OUTPUT_OBJECTS, PIN_OUTPUT_GROUP_A, PIN_OUTPUT_GROUP_B, PIN_OUTPUT_TRANSFORM, PIN_OUTPUT_GEOMETRY, PIN_OUTPUT_FRAGMENTS, PIN_OUTPUT_COMPOUND};
    use crate::graph::node::MetaMap;
    use crate::graph::value::Value;
    use std::f64::consts::PI;

    #[test]
    fn group_numbers() {
        let component = ComponentKind::Group;
        let outputs = component
            .evaluate(
                &[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)],
                &MetaMap::new(),
            )
            .expect("group");

        let Value::List(grouped) = outputs.get(PIN_OUTPUT_GROUP).unwrap() else {
            panic!("expected list");
        };
        assert_eq!(grouped.len(), 3);
        assert!(matches!(grouped[0], Value::Number(1.0)));
    }

    #[test]
    fn ungroup_list() {
        let component = ComponentKind::Ungroup;
        let outputs = component
            .evaluate(
                &[Value::List(vec![
                    Value::Number(1.0),
                    Value::Text("hello".into()),
                ])],
                &MetaMap::new(),
            )
            .expect("ungroup");

        let Value::List(ungrouped) = outputs.get(PIN_OUTPUT_OBJECTS).unwrap() else {
            panic!("expected list");
        };
        assert_eq!(ungrouped.len(), 2);
        assert!(matches!(ungrouped[1], Value::Text(_)));
    }

    #[test]
    fn merge_groups() {
        let component = ComponentKind::MergeGroup;
        let outputs = component
            .evaluate(
                &[
                    Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
                    Value::List(vec![Value::Number(3.0), Value::Number(4.0)]),
                ],
                &MetaMap::new(),
            )
            .expect("merge group");

        let Value::List(merged) = outputs.get(PIN_OUTPUT_GROUP).unwrap() else {
            panic!("expected list");
        };
        assert_eq!(merged.len(), 4);
        assert!(matches!(merged[3], Value::Number(4.0)));
    }

    #[test]
    fn split_group() {
        let component = ComponentKind::SplitGroup;
        let outputs = component
            .evaluate(
                &[
                    Value::List(vec![Value::Number(10.0), Value::Number(20.0), Value::Number(30.0)]),
                    Value::List(vec![Value::Number(0.0), Value::Number(2.0)]),
                    Value::Boolean(false),
                ],
                &MetaMap::new(),
            )
            .expect("split group");

        let Value::List(group_a) = outputs.get(PIN_OUTPUT_GROUP_A).unwrap() else { panic!("expected list A") };
        let Value::List(group_b) = outputs.get(PIN_OUTPUT_GROUP_B).unwrap() else { panic!("expected list B") };

        assert_eq!(group_a.len(), 2);
        assert_eq!(group_b.len(), 1);
        assert!(matches!(group_a[0], Value::Number(10.0)));
        assert!(matches!(group_a[1], Value::Number(30.0)));
        assert!(matches!(group_b[0], Value::Number(20.0)));
    }

    #[test]
    fn inverse_transform_move() {
        let component = ComponentKind::InverseTransform;
        let outputs = component
            .evaluate(
                &[Value::List(vec![
                    Value::Text("Move".into()),
                    Value::Vector([10.0, 20.0, 30.0]),
                ])],
                &MetaMap::new(),
            )
            .expect("inverse transform move");

        let Value::List(inversed) = outputs.get(PIN_OUTPUT_TRANSFORM).unwrap() else {
            panic!("expected list");
        };
        let Value::Vector(v) = inversed[1] else { panic!("expected vector") };
        assert!((v[0] + 10.0).abs() < 1e-6);
    }

    #[test]
    fn inverse_transform_rotate() {
        let component = ComponentKind::InverseTransform;
        let outputs = component.evaluate(&[
            Value::List(vec![
                Value::Text("Rotate".into()),
                Value::Point([0.0, 0.0, 0.0]),
                Value::Vector([0.0, 0.0, 1.0]),
                Value::Number(PI / 2.0),
            ])
        ], &MetaMap::new()).expect("inverse transform rotate");

        let Value::List(inversed) = outputs.get(PIN_OUTPUT_TRANSFORM).unwrap() else {
            panic!("expected list");
        };
        let Value::Number(angle) = inversed[3] else { panic!("expected number") };
        assert!((angle + PI / 2.0).abs() < 1e-6);
        let Value::Text(text) = &inversed[0] else { panic!("expected text") };
        assert_eq!(text, "Rotate");
    }

    #[test]
    fn transform_point_move() {
        let component = ComponentKind::Transform;
        let outputs = component.evaluate(&[
            Value::Point([1.0, 2.0, 3.0]),
            Value::List(vec![
                Value::Text("Move".into()),
                Value::Vector([10.0, 20.0, 30.0])
            ]),
        ], &MetaMap::new()).expect("transform move");

        let Value::Point(p) = outputs.get(PIN_OUTPUT_GEOMETRY).unwrap() else {
            panic!("expected point");
        };
        assert!((p[0] - 11.0).abs() < 1e-6);
        assert!((p[1] - 22.0).abs() < 1e-6);
        assert!((p[2] - 33.0).abs() < 1e-6);
    }

    #[test]
    fn transform_point_rotate() {
        let component = ComponentKind::Transform;
        let outputs = component.evaluate(&[
            Value::Point([10.0, 0.0, 0.0]),
            Value::List(vec![
                Value::Text("Rotate".into()),
                Value::Point([0.0, 0.0, 0.0]),
                Value::Vector([0.0, 0.0, 1.0]),
                Value::Number(PI / 2.0),
            ]),
        ], &MetaMap::new()).expect("transform rotate");

        let Value::Point(p) = outputs.get(PIN_OUTPUT_GEOMETRY).unwrap() else {
            panic!("expected point");
        };
        assert!(p[0].abs() < 1e-6);
        assert!((p[1] - 10.0).abs() < 1e-6);
        assert!(p[2].abs() < 1e-6);
    }

    #[test]
    fn split_compound_transform() {
        let component = ComponentKind::Split;
        let move_transform = Value::List(vec![Value::Text("Move".into()), Value::Vector([1.0, 2.0, 3.0])]);
        let rotate_transform = Value::List(vec![Value::Text("Rotate".into()), Value::Number(90.0)]);
        let compound = Value::List(vec![move_transform.clone(), rotate_transform.clone()]);

        let outputs = component.evaluate(&[compound], &MetaMap::new()).expect("split");

        let Value::List(fragments) = outputs.get(PIN_OUTPUT_FRAGMENTS).unwrap() else {
            panic!("expected list");
        };
        assert_eq!(fragments.len(), 2);
        assert_eq!(fragments[0], move_transform);
        assert_eq!(fragments[1], rotate_transform);
    }

    #[test]
    fn compound_transforms() {
        let component = ComponentKind::Compound;
        let transforms = Value::List(vec![
            Value::List(vec![Value::Text("Move".into()), Value::Vector([1.0, 2.0, 3.0])]),
            Value::List(vec![Value::Text("Rotate".into()), Value::Number(90.0)])
        ]);

        let outputs = component.evaluate(&[transforms.clone()], &MetaMap::new()).expect("compound");

        let compound_transform = outputs.get(PIN_OUTPUT_COMPOUND).unwrap();
        assert_eq!(compound_transform, &transforms);
    }
}
