//! Implementaties van Grasshopper "Vector → Field" componenten.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::{Matrix, Value};

use super::{Component, ComponentError, ComponentResult};

const EPSILON: f64 = 1e-9;

const PIN_OUTPUT_DISPLAY: &str = "D";
const PIN_OUTPUT_FIELD: &str = "F";
const PIN_OUTPUT_FIELDS: &str = "F";
const PIN_OUTPUT_TENSOR: &str = "T";
const PIN_OUTPUT_STRENGTH: &str = "S";
const PIN_OUTPUT_CURVE: &str = "C";

/// Beschikbare componenten binnen deze module.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    TensorDisplay,
    SpinForce,
    ScalarDisplay,
    DirectionDisplay,
    LineCharge,
    EvaluateField,
    FieldLine,
    BreakField,
    PerpendicularDisplay,
    PointCharge,
    VectorForce,
    MergeFields,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Volledige lijst van componentregistraties voor de vector-field componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{08619b6d-f9c4-4cb2-adcd-90959f08dc0d}"],
        names: &["Tensor Display", "FTensor"],
        kind: ComponentKind::TensorDisplay,
    },
    Registration {
        guids: &["{4b59e893-d4ee-4e31-ae24-a489611d1088}"],
        names: &["Spin Force", "FSpin"],
        kind: ComponentKind::SpinForce,
    },
    Registration {
        guids: &["{55f9ce6a-490c-4f25-a536-a3d47b794752}"],
        names: &["Scalar Display", "FScalar"],
        kind: ComponentKind::ScalarDisplay,
    },
    Registration {
        guids: &["{5ba20fab-6d71-48ea-a98f-cb034db6bbdc}"],
        names: &["Direction Display", "FDir"],
        kind: ComponentKind::DirectionDisplay,
    },
    Registration {
        guids: &["{8cc9eb88-26a7-4baa-a896-13e5fc12416a}"],
        names: &["Line Charge", "LCharge"],
        kind: ComponentKind::LineCharge,
    },
    Registration {
        guids: &["{a7c9f738-f8bd-4f64-8e7f-33341183e493}"],
        names: &["Evaluate Field", "EvF"],
        kind: ComponentKind::EvaluateField,
    },
    Registration {
        guids: &["{add6be3e-c57f-4740-96e4-5680abaa9169}"],
        names: &["Field Line", "FLine"],
        kind: ComponentKind::FieldLine,
    },
    Registration {
        guids: &["{b27d53bc-e713-475d-81fd-71cdd8de2e58}"],
        names: &["Break Field", "BreakF"],
        kind: ComponentKind::BreakField,
    },
    Registration {
        guids: &["{bf106e4c-68f4-476f-b05b-9c15fb50e078}"],
        names: &["Perpendicular Display", "FPerp"],
        kind: ComponentKind::PerpendicularDisplay,
    },
    Registration {
        guids: &["{cffdbaf3-8d33-4b38-9cad-c264af9fc3f4}"],
        names: &["Point Charge", "PCharge"],
        kind: ComponentKind::PointCharge,
    },
    Registration {
        guids: &["{d27cc1ea-9ef7-47bf-8ee2-c6662da0e3d9}"],
        names: &["Vector Force", "FVector"],
        kind: ComponentKind::VectorForce,
    },
    Registration {
        guids: &["{d9a6fbd2-2e9f-472e-8147-33bf0233a115}"],
        names: &["Merge Fields", "MergeF"],
        kind: ComponentKind::MergeFields,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::TensorDisplay => evaluate_tensor_display(inputs),
            Self::SpinForce => evaluate_spin_force(inputs),
            Self::ScalarDisplay => evaluate_scalar_display(inputs),
            Self::DirectionDisplay => evaluate_direction_display(inputs),
            Self::LineCharge => evaluate_line_charge(inputs),
            Self::EvaluateField => evaluate_field_value(inputs),
            Self::FieldLine => evaluate_field_line(inputs),
            Self::BreakField => evaluate_break_field(inputs),
            Self::PerpendicularDisplay => evaluate_perpendicular_display(inputs),
            Self::PointCharge => evaluate_point_charge(inputs),
            Self::VectorForce => evaluate_vector_force(inputs),
            Self::MergeFields => evaluate_merge_fields(inputs),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::TensorDisplay => "Tensor Display",
            Self::SpinForce => "Spin Force",
            Self::ScalarDisplay => "Scalar Display",
            Self::DirectionDisplay => "Direction Display",
            Self::LineCharge => "Line Charge",
            Self::EvaluateField => "Evaluate Field",
            Self::FieldLine => "Field Line",
            Self::BreakField => "Break Field",
            Self::PerpendicularDisplay => "Perpendicular Display",
            Self::PointCharge => "Point Charge",
            Self::VectorForce => "Vector Force",
            Self::MergeFields => "Merge Fields",
        }
    }
}

#[derive(Debug, Clone, Default)]
struct FieldValue {
    sources: Vec<FieldSource>,
    bounds: Option<FieldBounds>,
}

#[derive(Debug, Clone, Copy)]
struct FieldBounds {
    min: [f64; 3],
    max: [f64; 3],
}

#[derive(Debug, Clone, Copy)]
enum FieldSource {
    PointCharge {
        point: [f64; 3],
        charge: f64,
        decay: f64,
    },
    LineCharge {
        start: [f64; 3],
        end: [f64; 3],
        charge: f64,
        decay: f64,
    },
    VectorForce {
        start: [f64; 3],
        end: [f64; 3],
    },
    SpinForce {
        origin: [f64; 3],
        normal: [f64; 3],
        radius: f64,
        strength: f64,
        decay: f64,
    },
}

#[derive(Debug, Clone, Copy)]
struct FieldEvaluation {
    vector: [f64; 3],
    magnitude: f64,
    strength: f64,
    tensor: [[f64; 3]; 3],
}

#[derive(Debug, Clone, Copy)]
struct Plane {
    origin: [f64; 3],
    x_axis: [f64; 3],
    y_axis: [f64; 3],
    normal: [f64; 3],
}

impl Default for Plane {
    fn default() -> Self {
        Self {
            origin: [0.0, 0.0, 0.0],
            x_axis: [1.0, 0.0, 0.0],
            y_axis: [0.0, 1.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Section {
    plane: Plane,
    min_u: f64,
    max_u: f64,
    min_v: f64,
    max_v: f64,
}

impl Default for Section {
    fn default() -> Self {
        Self {
            plane: Plane::default(),
            min_u: -0.5,
            max_u: 0.5,
            min_v: -0.5,
            max_v: 0.5,
        }
    }
}

impl Section {
    fn sample_point(
        &self,
        u_index: usize,
        v_index: usize,
        u_count: usize,
        v_count: usize,
    ) -> [f64; 3] {
        let u_ratio = if u_count <= 1 {
            0.0
        } else {
            u_index as f64 / (u_count as f64 - 1.0)
        };
        let v_ratio = if v_count <= 1 {
            0.0
        } else {
            v_index as f64 / (v_count as f64 - 1.0)
        };
        let u = lerp(self.min_u, self.max_u, u_ratio);
        let v = lerp(self.min_v, self.max_v, v_ratio);
        let mut point = self.plane.origin;
        point = add(point, scale(self.plane.x_axis, u));
        point = add(point, scale(self.plane.y_axis, v));
        point
    }
}

fn evaluate_tensor_display(inputs: &[Value]) -> ComponentResult {
    let field = parse_field(inputs.get(0), "Tensor Display")?;
    let section = parse_section(inputs.get(1))?;
    let (samples_u, samples_v) = parse_samples(inputs.get(2));

    let mut entries = Vec::new();
    for u in 0..samples_u {
        for v in 0..samples_v {
            let point = section.sample_point(u, v, samples_u, samples_v);
            let evaluation = evaluate_field_at_point(&field, point);
            let tensor_matrix = matrix_from_tensor(evaluation.tensor);
            entries.push(Value::List(vec![
                Value::Point(point),
                Value::Vector(evaluation.vector),
                Value::Number(evaluation.magnitude),
                Value::Number(evaluation.strength),
                Value::Matrix(tensor_matrix),
            ]));
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_DISPLAY.to_owned(), Value::List(entries));
    Ok(outputs)
}

fn evaluate_scalar_display(inputs: &[Value]) -> ComponentResult {
    let field = parse_field(inputs.get(0), "Scalar Display")?;
    let section = parse_section(inputs.get(1))?;
    let (samples_u, samples_v) = parse_samples(inputs.get(2));

    let mut entries = Vec::new();
    for u in 0..samples_u {
        for v in 0..samples_v {
            let point = section.sample_point(u, v, samples_u, samples_v);
            let evaluation = evaluate_field_at_point(&field, point);
            entries.push(Value::List(vec![
                Value::Point(point),
                Value::Number(evaluation.magnitude),
            ]));
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_DISPLAY.to_owned(), Value::List(entries));
    Ok(outputs)
}

fn evaluate_direction_display(inputs: &[Value]) -> ComponentResult {
    let field = parse_field(inputs.get(0), "Direction Display")?;
    let section = parse_section(inputs.get(1))?;
    let (samples_u, samples_v) = parse_samples(inputs.get(2));

    let mut entries = Vec::new();
    for u in 0..samples_u {
        for v in 0..samples_v {
            let point = section.sample_point(u, v, samples_u, samples_v);
            let evaluation = evaluate_field_at_point(&field, point);
            let direction = if evaluation.magnitude > EPSILON {
                normalize(evaluation.vector)
            } else {
                [0.0, 0.0, 0.0]
            };
            entries.push(Value::List(vec![
                Value::Point(point),
                Value::Vector(direction),
            ]));
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_DISPLAY.to_owned(), Value::List(entries));
    Ok(outputs)
}

fn evaluate_perpendicular_display(inputs: &[Value]) -> ComponentResult {
    let field = parse_field(inputs.get(0), "Perpendicular Display")?;
    let section = parse_section(inputs.get(1))?;
    let (samples_u, samples_v) = parse_samples(inputs.get(2));
    let positive = parse_colour(inputs.get(3)).unwrap_or([0.95, 0.45, 0.35]);
    let negative = parse_colour(inputs.get(4)).unwrap_or([0.35, 0.55, 0.95]);

    let mut entries = Vec::new();
    for u in 0..samples_u {
        for v in 0..samples_v {
            let point = section.sample_point(u, v, samples_u, samples_v);
            let evaluation = evaluate_field_at_point(&field, point);
            let alignment = if evaluation.magnitude > EPSILON {
                dot(normalize(evaluation.vector), section.plane.normal)
            } else {
                0.0
            };
            let factor = ((alignment + 1.0) / 2.0).clamp(0.0, 1.0);
            let colour = lerp_colour(negative, positive, factor);
            entries.push(Value::List(vec![
                Value::Point(point),
                Value::Vector(colour),
            ]));
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_DISPLAY.to_owned(), Value::List(entries));
    Ok(outputs)
}

fn evaluate_point_charge(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 1 {
        return Err(ComponentError::new(
            "Point Charge vereist minimaal een punt",
        ));
    }
    let point = coerce_point(inputs.get(0), "Point Charge")?;
    let charge = coerce_number(inputs.get(1), 1.0, "Point Charge")?;
    let decay = coerce_number(inputs.get(2), 2.0, "Point Charge")?.max(0.0);
    let bounds = parse_bounds(inputs.get(3))?;

    let mut field = FieldValue::default();
    field.sources.push(FieldSource::PointCharge {
        point,
        charge,
        decay,
    });
    field.bounds = bounds;

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_FIELD.to_owned(), field_to_value(&field));
    Ok(outputs)
}

fn evaluate_line_charge(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Line Charge vereist minimaal een lijn"));
    }
    let line = coerce_line(inputs.get(0), "Line Charge")?;
    let charge = coerce_number(inputs.get(1), 1.0, "Line Charge")?;
    let decay = coerce_number(inputs.get(2), 2.0, "Line Charge")?.max(0.0);
    let bounds = parse_bounds(inputs.get(3))?;

    let mut field = FieldValue::default();
    field.sources.push(FieldSource::LineCharge {
        start: line.0,
        end: line.1,
        charge,
        decay,
    });
    field.bounds = bounds;

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_FIELD.to_owned(), field_to_value(&field));
    Ok(outputs)
}

fn evaluate_vector_force(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Vector Force vereist minimaal een lijn",
        ));
    }
    let line = coerce_line(inputs.get(0), "Vector Force")?;
    let bounds = parse_bounds(inputs.get(1))?;

    let mut field = FieldValue::default();
    field.sources.push(FieldSource::VectorForce {
        start: line.0,
        end: line.1,
    });
    field.bounds = bounds;

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_FIELD.to_owned(), field_to_value(&field));
    Ok(outputs)
}

fn evaluate_spin_force(inputs: &[Value]) -> ComponentResult {
    let plane = parse_plane(inputs.get(0), "Spin Force")?;
    let strength = coerce_number(inputs.get(1), 1.0, "Spin Force")?;
    let radius = coerce_number(inputs.get(2), 1.0, "Spin Force")?.abs();
    let decay = coerce_number(inputs.get(3), 2.0, "Spin Force")?.max(0.0);
    let bounds = parse_bounds(inputs.get(4))?;

    let mut field = FieldValue::default();
    field.sources.push(FieldSource::SpinForce {
        origin: plane.origin,
        normal: plane.normal,
        radius,
        strength,
        decay,
    });
    field.bounds = bounds;

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_FIELD.to_owned(), field_to_value(&field));
    Ok(outputs)
}

fn evaluate_merge_fields(inputs: &[Value]) -> ComponentResult {
    let fields = collect_fields(inputs.get(0))?;
    if fields.is_empty() {
        let mut outputs = BTreeMap::new();
        outputs.insert(
            PIN_OUTPUT_FIELD.to_owned(),
            field_to_value(&FieldValue::default()),
        );
        return Ok(outputs);
    }

    let mut merged = FieldValue::default();
    for field in fields {
        for source in field.sources {
            merged.sources.push(source);
        }
        merged.bounds = merge_bounds(merged.bounds, field.bounds);
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_FIELD.to_owned(), field_to_value(&merged));
    Ok(outputs)
}

fn evaluate_break_field(inputs: &[Value]) -> ComponentResult {
    let field = parse_field(inputs.get(0), "Break Field")?;
    if field.sources.is_empty() {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_FIELDS.to_owned(), Value::List(Vec::new()));
        return Ok(outputs);
    }

    let mut list = Vec::new();
    for source in field.sources {
        let mut entry = FieldValue::default();
        entry.sources.push(source);
        entry.bounds = field.bounds;
        list.push(field_to_value(&entry));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_FIELDS.to_owned(), Value::List(list));
    Ok(outputs)
}

fn evaluate_field_value(inputs: &[Value]) -> ComponentResult {
    let field = parse_field(inputs.get(0), "Evaluate Field")?;
    let point = coerce_point(inputs.get(1), "Evaluate Field")?;

    let evaluation = evaluate_field_at_point(&field, point);
    let tensor_matrix = matrix_from_tensor(evaluation.tensor);

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_TENSOR.to_owned(),
        Value::List(vec![
            Value::Point(point),
            Value::Vector(evaluation.vector),
            Value::Number(evaluation.magnitude),
            Value::Vector(if evaluation.magnitude > EPSILON {
                normalize(evaluation.vector)
            } else {
                [0.0, 0.0, 0.0]
            }),
            Value::Matrix(tensor_matrix),
        ]),
    );
    outputs.insert(
        PIN_OUTPUT_STRENGTH.to_owned(),
        Value::Number(evaluation.magnitude),
    );
    Ok(outputs)
}

fn evaluate_field_line(inputs: &[Value]) -> ComponentResult {
    let field = parse_field(inputs.get(0), "Field Line")?;
    let start = coerce_point(inputs.get(1), "Field Line")?;
    let steps = coerce_usize(inputs.get(2), 25, 1, "Field Line steps")?;
    let step_size = coerce_number(inputs.get(3), 0.5, "Field Line accuracy")?.max(EPSILON);
    let method = coerce_usize(inputs.get(4), 4, 1, "Field Line method")?.min(4);

    let mut points = Vec::new();
    points.push(Value::Point(start));
    let mut current = start;
    for _ in 0..steps {
        let evaluation = evaluate_field_at_point(&field, current);
        if evaluation.magnitude < EPSILON {
            break;
        }
        let direction = normalize(evaluation.vector);
        let displacement = match method {
            1 => scale(direction, step_size),
            2 => scale(direction, step_size * 0.75),
            3 => scale(direction, step_size * 0.5),
            _ => scale(direction, step_size),
        };
        current = add(current, displacement);
        points.push(Value::Point(current));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CURVE.to_owned(), Value::List(points));
    Ok(outputs)
}

fn parse_field(value: Option<&Value>, context: &str) -> Result<FieldValue, ComponentError> {
    let Some(value) = value else {
        return Ok(FieldValue::default());
    };

    match value {
        Value::List(entries) => {
            let mut field = FieldValue::default();
            for entry in entries {
                match entry {
                    Value::List(inner) if is_bounds_entry(inner) => {
                        field.bounds = Some(parse_bounds_entry(inner, context)?);
                    }
                    Value::List(inner) if !inner.is_empty() => {
                        field.sources.push(parse_field_source(inner, context)?);
                    }
                    Value::List(inner) if inner.is_empty() => {}
                    Value::List(inner) if inner.len() == 1 => {
                        let nested = parse_field(Some(&inner[0]), context)?;
                        field.sources.extend(nested.sources);
                        field.bounds = merge_bounds(field.bounds, nested.bounds);
                    }
                    other => {
                        return Err(ComponentError::new(format!(
                            "{} verwacht veldinformatie, kreeg {}",
                            context,
                            other.kind()
                        )));
                    }
                }
            }
            Ok(field)
        }
        Value::Text(text) if text.trim().is_empty() => Ok(FieldValue::default()),
        other => Err(ComponentError::new(format!(
            "{} verwacht veldgegevens, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn collect_fields(value: Option<&Value>) -> Result<Vec<FieldValue>, ComponentError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };

    match value {
        Value::List(entries) => {
            let mut fields = Vec::new();
            for entry in entries {
                let parsed = parse_field(Some(entry), "Merge Fields")?;
                if !parsed.sources.is_empty() || parsed.bounds.is_some() {
                    fields.push(parsed);
                } else {
                    fields.extend(collect_fields(Some(entry))?);
                }
            }
            Ok(fields)
        }
        _ => Ok(vec![parse_field(Some(value), "Merge Fields")?]),
    }
}

fn parse_field_source(values: &[Value], context: &str) -> Result<FieldSource, ComponentError> {
    let tag = match &values[0] {
        Value::Text(text) => text.to_ascii_lowercase(),
        Value::Number(_) | Value::Point(_) | Value::Vector(_) | Value::List(_) => {
            "point_charge".to_owned()
        }
        other => {
            return Err(ComponentError::new(format!(
                "{} kon veldbron niet interpreteren uit {}",
                context,
                other.kind()
            )));
        }
    };

    match tag.as_str() {
        "point_charge" | "pointcharge" | "point" => {
            let point = coerce_point(values.get(1).or_else(|| values.get(0)), context)?;
            let charge = coerce_number(values.get(2), 1.0, context)?;
            let decay = coerce_number(values.get(3), 2.0, context)?.max(0.0);
            Ok(FieldSource::PointCharge {
                point,
                charge,
                decay,
            })
        }
        "line_charge" | "linecharge" | "line" => {
            let line_value = values.get(1).or_else(|| values.get(0));
            let line = coerce_line(line_value, context)?;
            let charge = coerce_number(values.get(2), 1.0, context)?;
            let decay = coerce_number(values.get(3), 2.0, context)?.max(0.0);
            Ok(FieldSource::LineCharge {
                start: line.0,
                end: line.1,
                charge,
                decay,
            })
        }
        "vector_force" | "vectorforce" | "vector" => {
            let line_value = values.get(1).or_else(|| values.get(0));
            let line = coerce_line(line_value, context)?;
            Ok(FieldSource::VectorForce {
                start: line.0,
                end: line.1,
            })
        }
        "spin_force" | "spinforce" | "spin" => {
            let plane = parse_plane(values.get(1), context)?;
            let strength = coerce_number(values.get(2), 1.0, context)?;
            let radius = coerce_number(values.get(3), 1.0, context)?.abs();
            let decay = coerce_number(values.get(4), 2.0, context)?.max(0.0);
            Ok(FieldSource::SpinForce {
                origin: plane.origin,
                normal: plane.normal,
                radius,
                strength,
                decay,
            })
        }
        other => Err(ComponentError::new(format!(
            "{} ondersteunt veldbron '{}' niet",
            context, other
        ))),
    }
}

fn is_bounds_entry(values: &[Value]) -> bool {
    if values.len() != 3 {
        return false;
    }
    matches!(values.first(), Some(Value::Text(label)) if label.eq_ignore_ascii_case("bounds"))
}

fn parse_bounds_entry(values: &[Value], context: &str) -> Result<FieldBounds, ComponentError> {
    if values.len() != 3 {
        return Err(ComponentError::new(format!(
            "{} verwacht bounds met twee punten",
            context
        )));
    }
    let min = coerce_point(Some(&values[1]), context)?;
    let max = coerce_point(Some(&values[2]), context)?;
    Ok(normalize_bounds(FieldBounds { min, max }))
}

fn parse_bounds(value: Option<&Value>) -> Result<Option<FieldBounds>, ComponentError> {
    let Some(value) = value else {
        return Ok(None);
    };

    match value {
        Value::List(values) if values.is_empty() => Ok(None),
        Value::List(values) if is_bounds_entry(values) => {
            parse_bounds_entry(values, "Bounds").map(Some)
        }
        Value::List(values) if values.len() == 1 => parse_bounds(values.get(0)),
        Value::List(values) if values.len() >= 2 => {
            let min = coerce_point(values.get(0), "Bounds")?;
            let max = coerce_point(values.get(1), "Bounds")?;
            Ok(Some(normalize_bounds(FieldBounds { min, max })))
        }
        Value::Point(point) => Ok(Some(FieldBounds {
            min: *point,
            max: *point,
        })),
        Value::Vector(vector) => Ok(Some(FieldBounds {
            min: [0.0, 0.0, 0.0],
            max: *vector,
        })),
        _ => Err(ComponentError::new(
            "Bounds verwacht twee punten of een lijst met punten",
        )),
    }
}

fn field_to_value(field: &FieldValue) -> Value {
    let mut entries = Vec::new();
    for source in &field.sources {
        let entry = match source {
            FieldSource::PointCharge {
                point,
                charge,
                decay,
            } => Value::List(vec![
                Value::Text("point_charge".into()),
                Value::Point(*point),
                Value::Number(*charge),
                Value::Number(*decay),
            ]),
            FieldSource::LineCharge {
                start,
                end,
                charge,
                decay,
            } => Value::List(vec![
                Value::Text("line_charge".into()),
                Value::Point(*start),
                Value::Point(*end),
                Value::Number(*charge),
                Value::Number(*decay),
            ]),
            FieldSource::VectorForce { start, end } => Value::List(vec![
                Value::Text("vector_force".into()),
                Value::Point(*start),
                Value::Point(*end),
            ]),
            FieldSource::SpinForce {
                origin,
                normal,
                radius,
                strength,
                decay,
            } => Value::List(vec![
                Value::Text("spin_force".into()),
                Value::Point(*origin),
                Value::Vector(*normal),
                Value::Number(*radius),
                Value::Number(*strength),
                Value::Number(*decay),
            ]),
        };
        entries.push(entry);
    }
    if let Some(bounds) = field.bounds {
        entries.push(Value::List(vec![
            Value::Text("bounds".into()),
            Value::Point(bounds.min),
            Value::Point(bounds.max),
        ]));
    }
    Value::List(entries)
}

fn evaluate_field_at_point(field: &FieldValue, point: [f64; 3]) -> FieldEvaluation {
    let mut vector = [0.0, 0.0, 0.0];
    let mut strength = 0.0;
    let mut tensor = [[0.0; 3]; 3];

    for source in &field.sources {
        let contribution = match source {
            FieldSource::PointCharge {
                point: source_point,
                charge,
                decay,
            } => evaluate_point_charge_source(*source_point, *charge, *decay, point),
            FieldSource::LineCharge {
                start,
                end,
                charge,
                decay,
            } => evaluate_line_charge_source(*start, *end, *charge, *decay, point),
            FieldSource::VectorForce { start, end } => {
                evaluate_vector_force_source(*start, *end, point)
            }
            FieldSource::SpinForce {
                origin,
                normal,
                radius,
                strength,
                decay,
            } => evaluate_spin_force_source(*origin, *normal, *radius, *strength, *decay, point),
        };
        vector = add(vector, contribution.vector);
        strength += contribution.strength;
        tensor = add_matrix(tensor, contribution.tensor);
    }

    let magnitude = length(vector);

    FieldEvaluation {
        vector,
        magnitude,
        strength,
        tensor,
    }
}

fn evaluate_point_charge_source(
    center: [f64; 3],
    charge: f64,
    decay: f64,
    point: [f64; 3],
) -> FieldEvaluation {
    let direction = subtract(point, center);
    let distance = length(direction).max(EPSILON);
    let intensity = charge / distance.powf(decay + 1.0);
    let unit = normalize(direction);
    let vector = scale(unit, intensity);
    let tensor = scale_matrix(outer_product(unit), intensity.abs());
    FieldEvaluation {
        vector,
        magnitude: length(vector),
        strength: intensity.abs(),
        tensor,
    }
}

fn evaluate_line_charge_source(
    start: [f64; 3],
    end: [f64; 3],
    charge: f64,
    decay: f64,
    point: [f64; 3],
) -> FieldEvaluation {
    let direction = subtract(end, start);
    let length_line = length(direction).max(EPSILON);
    let midpoint = scale(add(start, end), 0.5);
    let offset = subtract(point, midpoint);
    let distance = length(offset).max(EPSILON);
    let intensity = (charge / length_line) / distance.powf(decay.max(0.0) + 1.0);
    let unit = normalize(offset);
    let vector = scale(unit, intensity);
    let tensor = scale_matrix(outer_product(unit), intensity.abs());
    FieldEvaluation {
        vector,
        magnitude: length(vector),
        strength: intensity.abs(),
        tensor,
    }
}

fn evaluate_vector_force_source(
    start: [f64; 3],
    end: [f64; 3],
    point: [f64; 3],
) -> FieldEvaluation {
    let direction = subtract(end, start);
    let length_line = length(direction).max(EPSILON);
    let to_point = subtract(point, start);
    let projection = project(to_point, direction);
    let closest = add(start, projection);
    let offset = subtract(point, closest);
    let distance = length(offset).max(EPSILON);
    let base_strength = length_line / (distance * distance);
    let unit = normalize(offset);
    let vector = scale(unit, base_strength);
    let tensor = scale_matrix(outer_product(normalize(direction)), base_strength);
    FieldEvaluation {
        vector,
        magnitude: length(vector),
        strength: base_strength,
        tensor,
    }
}

fn evaluate_spin_force_source(
    origin: [f64; 3],
    normal: [f64; 3],
    radius: f64,
    strength: f64,
    decay: f64,
    point: [f64; 3],
) -> FieldEvaluation {
    let axis = normalize(normal);
    let to_point = subtract(point, origin);
    let axial = scale(axis, dot(to_point, axis));
    let radial = subtract(to_point, axial);
    let distance = length(radial).max(EPSILON);
    let clamped = distance / radius.max(EPSILON);
    let falloff = 1.0 / (1.0 + clamped.powf(decay.max(0.0)));
    let tangent = normalize(cross(axis, radial));
    let vector = scale(tangent, strength * falloff / distance);
    let tensor = scale_matrix(outer_product(tangent), (strength * falloff).abs());
    FieldEvaluation {
        vector,
        magnitude: length(vector),
        strength: (strength * falloff).abs(),
        tensor,
    }
}

fn matrix_from_tensor(tensor: [[f64; 3]; 3]) -> Matrix {
    Matrix {
        rows: 3,
        columns: 3,
        values: tensor.iter().flatten().copied().collect(),
    }
}

fn parse_samples(value: Option<&Value>) -> (usize, usize) {
    let default = (10, 10);
    let Some(value) = value else {
        return default;
    };
    match value {
        Value::Number(number) => {
            let count = number.round().max(1.0) as usize;
            (count, count)
        }
        Value::List(values) if values.is_empty() => default,
        Value::List(values) if values.len() == 1 => parse_samples(values.get(0)),
        Value::List(values) => {
            let u = values
                .get(0)
                .and_then(|value| coerce_number(Some(value), 10.0, "Samples").ok())
                .unwrap_or(10.0)
                .round()
                .max(1.0) as usize;
            let v = values
                .get(1)
                .and_then(|value| coerce_number(Some(value), 10.0, "Samples").ok())
                .unwrap_or(u as f64)
                .round()
                .max(1.0) as usize;
            (u, v)
        }
        _ => default,
    }
}

fn parse_section(value: Option<&Value>) -> Result<Section, ComponentError> {
    let Some(value) = value else {
        return Ok(Section::default());
    };
    match value {
        Value::List(values) if values.len() >= 3 => {
            let a = coerce_point(values.get(0), "Section")?;
            let b = coerce_point(values.get(1), "Section")?;
            let c = coerce_point(values.get(2), "Section")?;
            let plane = plane_from_points(a, b, c);
            let mut section = Section {
                plane,
                ..Section::default()
            };
            let coords = [
                plane_coordinates(a, plane),
                plane_coordinates(b, plane),
                plane_coordinates(c, plane),
            ];
            section.min_u = coords.iter().map(|c| c[0]).fold(f64::INFINITY, f64::min);
            section.max_u = coords
                .iter()
                .map(|c| c[0])
                .fold(f64::NEG_INFINITY, f64::max);
            section.min_v = coords.iter().map(|c| c[1]).fold(f64::INFINITY, f64::min);
            section.max_v = coords
                .iter()
                .map(|c| c[1])
                .fold(f64::NEG_INFINITY, f64::max);
            Ok(section)
        }
        Value::List(values) if values.len() == 2 => {
            let a = coerce_point(values.get(0), "Section")?;
            let b = coerce_point(values.get(1), "Section")?;
            let plane = plane_from_points(a, b, add(a, [0.0, 0.0, 1.0]));
            let mut section = Section {
                plane,
                ..Section::default()
            };
            section.min_u = 0.0;
            section.min_v = 0.0;
            section.max_u = length(subtract(b, a));
            section.max_v = section.max_u;
            Ok(section)
        }
        Value::List(values) if values.len() == 1 => parse_section(values.get(0)),
        _ => Ok(Section::default()),
    }
}

fn parse_plane(value: Option<&Value>, context: &str) -> Result<Plane, ComponentError> {
    let Some(value) = value else {
        return Ok(Plane::default());
    };
    match value {
        Value::List(values) if values.len() >= 3 => {
            let a = coerce_point(values.get(0), context)?;
            let b = coerce_point(values.get(1), context)?;
            let c = coerce_point(values.get(2), context)?;
            Ok(plane_from_points(a, b, c))
        }
        Value::List(values) if values.len() == 2 => {
            let origin = coerce_point(values.get(0), context)?;
            let normal = coerce_vector(values.get(1), context)?;
            Ok(plane_from_normal(origin, normal))
        }
        Value::List(values) if values.len() == 1 => parse_plane(values.get(0), context),
        Value::Point(point) => Ok(Plane {
            origin: *point,
            ..Plane::default()
        }),
        Value::Vector(vector) => Ok(plane_from_normal([0.0, 0.0, 0.0], *vector)),
        _ => Err(ComponentError::new(format!(
            "{} verwacht vlakgegevens, kreeg {}",
            context,
            value.kind()
        ))),
    }
}

fn plane_from_points(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> Plane {
    let ab = subtract(b, a);
    let ac = subtract(c, a);
    let normal = normalize(cross(ab, ac));
    plane_from_axes(a, ab, normal)
}

fn plane_from_normal(origin: [f64; 3], normal: [f64; 3]) -> Plane {
    plane_from_axes(origin, orthogonal_vector(normal), normal)
}

fn plane_from_axes(origin: [f64; 3], x_axis: [f64; 3], normal: [f64; 3]) -> Plane {
    let z = normalize(normal);
    let mut x = normalize(x_axis);
    if length_squared(x) < EPSILON {
        x = orthogonal_vector(z);
    }
    let y = normalize(cross(z, x));
    Plane {
        origin,
        x_axis: x,
        y_axis: y,
        normal: z,
    }
}

fn plane_coordinates(point: [f64; 3], plane: Plane) -> [f64; 3] {
    let relative = subtract(point, plane.origin);
    [
        dot(relative, plane.x_axis),
        dot(relative, plane.y_axis),
        dot(relative, plane.normal),
    ]
}

fn parse_colour(value: Option<&Value>) -> Option<[f64; 3]> {
    match value? {
        Value::Vector(vector) | Value::Point(vector) => Some(*vector),
        Value::List(values) if values.len() >= 3 => {
            let r = coerce_number(values.get(0), 0.0, "colour").ok()?;
            let g = coerce_number(values.get(1), 0.0, "colour").ok()?;
            let b = coerce_number(values.get(2), 0.0, "colour").ok()?;
            Some([r, g, b])
        }
        _ => None,
    }
}

fn coerce_point(value: Option<&Value>, context: &str) -> Result<[f64; 3], ComponentError> {
    let value =
        value.ok_or_else(|| ComponentError::new(format!("{} verwacht een punt", context)))?;
    match value {
        Value::Point(point) => Ok(*point),
        Value::Vector(vector) => Ok(*vector),
        Value::List(values) if values.len() == 1 => coerce_point(values.get(0), context),
        Value::List(values) if values.len() >= 3 => {
            let x = coerce_number(values.get(0), 0.0, context)?;
            let y = coerce_number(values.get(1), 0.0, context)?;
            let z = coerce_number(values.get(2), 0.0, context)?;
            Ok([x, y, z])
        }
        Value::Number(number) => Ok([*number, 0.0, 0.0]),
        other => Err(ComponentError::new(format!(
            "{} verwacht puntgegevens, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_line(
    value: Option<&Value>,
    context: &str,
) -> Result<([f64; 3], [f64; 3]), ComponentError> {
    let value =
        value.ok_or_else(|| ComponentError::new(format!("{} vereist een lijn", context)))?;
    match value {
        Value::CurveLine { p1, p2 } => Ok((*p1, *p2)),
        Value::List(values) if values.len() >= 2 => {
            let start = coerce_point(values.get(0), context)?;
            let end = coerce_point(values.get(1), context)?;
            Ok((start, end))
        }
        Value::List(values) if values.len() == 1 => coerce_line(values.get(0), context),
        other => Err(ComponentError::new(format!(
            "{} verwacht lijngegevens, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_vector(value: Option<&Value>, context: &str) -> Result<[f64; 3], ComponentError> {
    let value =
        value.ok_or_else(|| ComponentError::new(format!("{} verwacht een vector", context)))?;
    match value {
        Value::Vector(vector) => Ok(*vector),
        Value::Point(point) => Ok(*point),
        Value::List(values) if values.len() == 1 => coerce_vector(values.get(0), context),
        Value::List(values) if values.len() >= 3 => {
            let x = coerce_number(values.get(0), 0.0, context)?;
            let y = coerce_number(values.get(1), 0.0, context)?;
            let z = coerce_number(values.get(2), 0.0, context)?;
            Ok([x, y, z])
        }
        Value::Number(number) => Ok([0.0, 0.0, *number]),
        other => Err(ComponentError::new(format!(
            "{} verwacht vectorgegevens, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_number(
    value: Option<&Value>,
    default: f64,
    context: &str,
) -> Result<f64, ComponentError> {
    let Some(value) = value else {
        return Ok(default);
    };
    match value {
        Value::Number(number) => Ok(*number),
        Value::Boolean(boolean) => Ok(if *boolean { 1.0 } else { 0.0 }),
        Value::List(values) if values.len() == 1 => coerce_number(values.get(0), default, context),
        other => Err(ComponentError::new(format!(
            "{} verwacht numerieke waarde, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_usize(
    value: Option<&Value>,
    default: usize,
    min: usize,
    context: &str,
) -> Result<usize, ComponentError> {
    let Some(value) = value else {
        return Ok(default.max(min));
    };
    let number = coerce_number(Some(value), default as f64, context)?;
    let rounded = number.round().max(min as f64) as usize;
    Ok(rounded)
}

fn normalize_bounds(bounds: FieldBounds) -> FieldBounds {
    FieldBounds {
        min: [
            bounds.min[0].min(bounds.max[0]),
            bounds.min[1].min(bounds.max[1]),
            bounds.min[2].min(bounds.max[2]),
        ],
        max: [
            bounds.min[0].max(bounds.max[0]),
            bounds.min[1].max(bounds.max[1]),
            bounds.min[2].max(bounds.max[2]),
        ],
    }
}

fn merge_bounds(a: Option<FieldBounds>, b: Option<FieldBounds>) -> Option<FieldBounds> {
    match (a, b) {
        (Some(left), Some(right)) => Some(FieldBounds {
            min: [
                left.min[0].min(right.min[0]),
                left.min[1].min(right.min[1]),
                left.min[2].min(right.min[2]),
            ],
            max: [
                left.max[0].max(right.max[0]),
                left.max[1].max(right.max[1]),
                left.max[2].max(right.max[2]),
            ],
        }),
        (Some(bounds), None) | (None, Some(bounds)) => Some(bounds),
        (None, None) => None,
    }
}

fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn subtract(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: [f64; 3], factor: f64) -> [f64; 3] {
    [a[0] * factor, a[1] * factor, a[2] * factor]
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

fn length(vector: [f64; 3]) -> f64 {
    length_squared(vector).sqrt()
}

fn length_squared(vector: [f64; 3]) -> f64 {
    dot(vector, vector)
}

fn normalize(vector: [f64; 3]) -> [f64; 3] {
    let len = length(vector);
    if len < EPSILON {
        return [0.0, 0.0, 0.0];
    }
    scale(vector, 1.0 / len)
}

fn outer_product(vector: [f64; 3]) -> [[f64; 3]; 3] {
    [
        [
            vector[0] * vector[0],
            vector[0] * vector[1],
            vector[0] * vector[2],
        ],
        [
            vector[1] * vector[0],
            vector[1] * vector[1],
            vector[1] * vector[2],
        ],
        [
            vector[2] * vector[0],
            vector[2] * vector[1],
            vector[2] * vector[2],
        ],
    ]
}

fn add_matrix(a: [[f64; 3]; 3], b: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut result = a;
    for i in 0..3 {
        for j in 0..3 {
            result[i][j] += b[i][j];
        }
    }
    result
}

fn scale_matrix(matrix: [[f64; 3]; 3], factor: f64) -> [[f64; 3]; 3] {
    let mut result = matrix;
    for row in &mut result {
        for value in row {
            *value *= factor;
        }
    }
    result
}

fn project(vector: [f64; 3], onto: [f64; 3]) -> [f64; 3] {
    let denom = length_squared(onto);
    if denom < EPSILON {
        return [0.0, 0.0, 0.0];
    }
    let factor = dot(vector, onto) / denom;
    scale(onto, factor)
}

fn orthogonal_vector(vector: [f64; 3]) -> [f64; 3] {
    if vector[0].abs() > vector[1].abs() {
        normalize([vector[2], 0.0, -vector[0]])
    } else {
        normalize([0.0, -vector[2], vector[1]])
    }
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn lerp_colour(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [
        lerp(a[0], b[0], t),
        lerp(a[1], b[1], t),
        lerp(a[2], b[2], t),
    ]
}