//! Implementaties van Grasshopper "Vector → Point" componenten.

use std::cmp::Ordering;
use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

const PIN_OUTPUT_POINT: &str = "P";
const PIN_OUTPUT_POINTS: &str = "P";
const PIN_OUTPUT_INDICES: &str = "I";
const PIN_OUTPUT_INDEX: &str = "i";
const PIN_OUTPUT_DISTANCE: &str = "D";
const PIN_OUTPUT_NUMBERS: &str = "N";
const PIN_OUTPUT_X: &str = "X";
const PIN_OUTPUT_Y: &str = "Y";
const PIN_OUTPUT_Z: &str = "Z";
const PIN_OUTPUT_VALENCE: &str = "V";

/// Beschikbare componenten binnen deze module.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    NumbersToPoints,
    PointsToNumbers,
    Distance,
    Deconstruct,
    ClosestPoint,
    ClosestPoints,
    SortPoints,
    CullDuplicates,
    Barycentric,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Volledige lijst van componentregistraties voor de vector-point componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{0ae07da9-951b-4b9b-98ca-d312c252374d}"],
        names: &["Numbers to Points", "Num2Pt"],
        kind: ComponentKind::NumbersToPoints,
    },
    Registration {
        guids: &["{d24169cc-9922-4923-92bc-b9222efc413f}"],
        names: &["Points to Numbers", "Pt2Num"],
        kind: ComponentKind::PointsToNumbers,
    },
    Registration {
        guids: &["{93b8e93d-f932-402c-b435-84be04d87666}"],
        names: &["Distance", "Dist"],
        kind: ComponentKind::Distance,
    },
    Registration {
        guids: &["{9abae6b7-fa1d-448c-9209-4a8155345841}"],
        names: &["Deconstruct", "pDecon"],
        kind: ComponentKind::Deconstruct,
    },
    Registration {
        guids: &["{571ca323-6e55-425a-bf9e-ee103c7ba4b9}"],
        names: &["Closest Point", "CP"],
        kind: ComponentKind::ClosestPoint,
    },
    Registration {
        guids: &["{446014c4-c11c-45a7-8839-c45dc60950d6}"],
        names: &["Closest Points", "CPs"],
        kind: ComponentKind::ClosestPoints,
    },
    Registration {
        guids: &["{4e86ba36-05e2-4cc0-a0f5-3ad57c91f04e}"],
        names: &["Sort Points", "Sort Pt"],
        kind: ComponentKind::SortPoints,
    },
    Registration {
        guids: &["{6eaffbb2-3392-441a-8556-2dc126aa8910}"],
        names: &["Cull Duplicates", "CullPt"],
        kind: ComponentKind::CullDuplicates,
    },
    Registration {
        guids: &["{9adffd61-f5d1-4e9e-9572-e8d9145730dc}"],
        names: &["Barycentric", "BCentric"],
        kind: ComponentKind::Barycentric,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::NumbersToPoints => evaluate_numbers_to_points(inputs),
            Self::PointsToNumbers => evaluate_points_to_numbers(inputs),
            Self::Distance => evaluate_distance(inputs),
            Self::Deconstruct => evaluate_deconstruct(inputs),
            Self::ClosestPoint => evaluate_closest_point(inputs),
            Self::ClosestPoints => evaluate_closest_points(inputs),
            Self::SortPoints => evaluate_sort_points(inputs),
            Self::CullDuplicates => evaluate_cull_duplicates(inputs),
            Self::Barycentric => evaluate_barycentric(inputs),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::NumbersToPoints => "Numbers to Points",
            Self::PointsToNumbers => "Points to Numbers",
            Self::Distance => "Point Distance",
            Self::Deconstruct => "Deconstruct Point",
            Self::ClosestPoint => "Closest Point",
            Self::ClosestPoints => "Closest Points",
            Self::SortPoints => "Sort Points",
            Self::CullDuplicates => "Cull Duplicates",
            Self::Barycentric => "Barycentric Point",
        }
    }
}

fn evaluate_numbers_to_points(inputs: &[Value]) -> ComponentResult {
    let context = "Numbers to Points";
    if inputs.is_empty() {
        return Err(ComponentError::new(format!(
            "{} vereist minimaal één invoer",
            context
        )));
    }

    let numbers = collect_numbers(inputs.get(0), context)?;
    let mask = parse_mask(inputs.get(1));
    if mask.is_empty() {
        return Err(ComponentError::new(
            "Mask voor Numbers to Points resulteerde in geen assen",
        ));
    }

    let chunk = mask.len();
    if chunk == 0 {
        return Err(ComponentError::new(
            "Mask voor Numbers to Points is ongeldig",
        ));
    }

    let mut points = Vec::new();
    for group in numbers.chunks(chunk) {
        if group.len() < chunk {
            break;
        }
        let mut coords = [0.0, 0.0, 0.0];
        for (axis, value) in mask.iter().zip(group.iter()) {
            match axis {
                'x' => coords[0] = *value,
                'y' => coords[1] = *value,
                'z' => coords[2] = *value,
                _ => {}
            }
        }
        points.push(Value::Point(coords));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::List(points));
    Ok(outputs)
}

fn evaluate_points_to_numbers(inputs: &[Value]) -> ComponentResult {
    let context = "Points to Numbers";
    if inputs.is_empty() {
        return Err(ComponentError::new(format!(
            "{} vereist minimaal één invoer",
            context
        )));
    }

    let points = collect_points(inputs.get(0), context)?;
    let mask = parse_mask(inputs.get(1));
    if mask.is_empty() {
        return Err(ComponentError::new(
            "Mask voor Points to Numbers resulteerde in geen assen",
        ));
    }

    let mut numbers = Vec::new();
    for point in points {
        for axis in &mask {
            match axis {
                'x' => numbers.push(Value::Number(point[0])),
                'y' => numbers.push(Value::Number(point[1])),
                'z' => numbers.push(Value::Number(point[2])),
                _ => {}
            }
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_NUMBERS.to_owned(), Value::List(numbers));
    Ok(outputs)
}

fn evaluate_distance(inputs: &[Value]) -> ComponentResult {
    let context = "Point Distance";
    if inputs.len() < 2 {
        return Err(ComponentError::new(format!(
            "{} vereist twee punten",
            context
        )));
    }

    let a = coerce_point(&inputs[0], context)?;
    let b = coerce_point(&inputs[1], context)?;
    let distance = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_DISTANCE.to_owned(), Value::Number(distance));
    Ok(outputs)
}

fn evaluate_deconstruct(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Deconstruct Point vereist een punt als invoer",
        ));
    }

    let point = coerce_point(&inputs[0], "Deconstruct Point")?;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_X.to_owned(), Value::Number(point[0]));
    outputs.insert(PIN_OUTPUT_Y.to_owned(), Value::Number(point[1]));
    outputs.insert(PIN_OUTPUT_Z.to_owned(), Value::Number(point[2]));
    Ok(outputs)
}

fn evaluate_closest_point(inputs: &[Value]) -> ComponentResult {
    let context = "Closest Point";
    if inputs.len() < 2 {
        return Err(ComponentError::new(format!(
            "{} vereist een doelpunt en een puntenwolk",
            context
        )));
    }

    let target = coerce_point(&inputs[0], context)?;
    let candidates = collect_points(inputs.get(1), context)?;
    if candidates.is_empty() {
        return Err(ComponentError::new(
            "Closest Point vereist minimaal één kandidaatpunt",
        ));
    }

    let mut best_index = 0usize;
    let mut best_distance_sq = f64::INFINITY;
    for (index, candidate) in candidates.iter().enumerate() {
        let dx = candidate[0] - target[0];
        let dy = candidate[1] - target[1];
        let dz = candidate[2] - target[2];
        let distance_sq = dx * dx + dy * dy + dz * dz;
        if distance_sq < best_distance_sq {
            best_distance_sq = distance_sq;
            best_index = index;
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_POINT.to_owned(),
        Value::Point(candidates[best_index]),
    );
    outputs.insert(
        PIN_OUTPUT_INDEX.to_owned(),
        Value::Number(best_index as f64),
    );
    outputs.insert(
        PIN_OUTPUT_DISTANCE.to_owned(),
        Value::Number(best_distance_sq.sqrt()),
    );
    Ok(outputs)
}

fn evaluate_closest_points(inputs: &[Value]) -> ComponentResult {
    let context = "Closest Points";
    if inputs.len() < 2 {
        return Err(ComponentError::new(format!(
            "{} vereist een doelpunt en een puntenwolk",
            context
        )));
    }

    let target = coerce_point(&inputs[0], context)?;
    let candidates = collect_points(inputs.get(1), context)?;
    if candidates.is_empty() {
        return Err(ComponentError::new(
            "Closest Points vereist minimaal één kandidaatpunt",
        ));
    }

    let count = coerce_count(inputs.get(2), 1, context)?;

    let mut entries: Vec<(usize, [f64; 3], f64)> = candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| {
            let dx = candidate[0] - target[0];
            let dy = candidate[1] - target[1];
            let dz = candidate[2] - target[2];
            let distance_sq = dx * dx + dy * dy + dz * dz;
            (index, *candidate, distance_sq)
        })
        .collect();
    entries.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(Ordering::Equal));

    let take = count.min(entries.len());
    let mut points = Vec::with_capacity(take);
    let mut indices = Vec::with_capacity(take);
    let mut distances = Vec::with_capacity(take);

    for entry in entries.iter().take(take) {
        points.push(Value::Point(entry.1));
        indices.push(Value::Number(entry.0 as f64));
        distances.push(Value::Number(entry.2.sqrt()));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::List(points));
    outputs.insert(PIN_OUTPUT_INDICES.to_owned(), Value::List(indices));
    outputs.insert(PIN_OUTPUT_DISTANCE.to_owned(), Value::List(distances));
    Ok(outputs)
}

fn evaluate_sort_points(inputs: &[Value]) -> ComponentResult {
    let context = "Sort Points";
    if inputs.is_empty() {
        return Err(ComponentError::new(format!(
            "{} vereist een lijst met punten",
            context
        )));
    }

    let points = collect_points(inputs.get(0), context)?;
    let mut enumerated: Vec<(usize, [f64; 3])> = points.into_iter().enumerate().collect();
    enumerated.sort_by(|a, b| compare_points(a.1, b.1));

    let mut sorted_points = Vec::with_capacity(enumerated.len());
    let mut indices = Vec::with_capacity(enumerated.len());
    for (index, point) in enumerated {
        sorted_points.push(Value::Point(point));
        indices.push(Value::Number(index as f64));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::List(sorted_points));
    outputs.insert(PIN_OUTPUT_INDICES.to_owned(), Value::List(indices));
    Ok(outputs)
}

fn evaluate_cull_duplicates(inputs: &[Value]) -> ComponentResult {
    let context = "Cull Duplicates";
    if inputs.is_empty() {
        return Err(ComponentError::new(format!(
            "{} vereist een lijst met punten",
            context
        )));
    }

    let points = collect_points(inputs.get(0), context)?;
    let tolerance = coerce_number(inputs.get(1), context)
        .unwrap_or(0.001)
        .max(0.0);

    let mut unique = Vec::new();
    let mut indices = Vec::new();
    let mut valence = Vec::new();
    let tolerance_sq = tolerance * tolerance;

    for (input_index, point) in points.iter().enumerate() {
        let mut found = None;
        for (idx, existing) in unique.iter().enumerate() {
            if distance_squared(*existing, *point) <= tolerance_sq {
                found = Some(idx);
                break;
            }
        }

        match found {
            Some(existing_index) => {
                valence[existing_index] += 1.0;
            }
            None => {
                unique.push(*point);
                indices.push(Value::Number(input_index as f64));
                valence.push(1.0);
            }
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_POINTS.to_owned(),
        Value::List(unique.into_iter().map(Value::Point).collect()),
    );
    outputs.insert(PIN_OUTPUT_INDICES.to_owned(), Value::List(indices));
    outputs.insert(
        PIN_OUTPUT_VALENCE.to_owned(),
        Value::List(valence.into_iter().map(Value::Number).collect()),
    );
    Ok(outputs)
}

fn evaluate_barycentric(inputs: &[Value]) -> ComponentResult {
    let context = "Barycentric";
    if inputs.len() < 6 {
        return Err(ComponentError::new(format!(
            "{} vereist drie punten en drie coördinaten",
            context
        )));
    }

    let a = coerce_point(&inputs[0], context)?;
    let b = coerce_point(&inputs[1], context)?;
    let c = coerce_point(&inputs[2], context)?;
    let u = coerce_number(Some(&inputs[3]), context)?;
    let v = coerce_number(Some(&inputs[4]), context)?;
    let mut w = coerce_number(Some(&inputs[5]), context).unwrap_or(f64::NAN);
    if !w.is_finite() {
        w = 1.0 - u - v;
    }

    let point = [
        a[0] * u + b[0] * v + c[0] * w,
        a[1] * u + b[1] * v + c[1] * w,
        a[2] * u + b[2] * v + c[2] * w,
    ];

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINT.to_owned(), Value::Point(point));
    Ok(outputs)
}

fn compare_points(a: [f64; 3], b: [f64; 3]) -> Ordering {
    compare_f64(a[0], b[0])
        .then(compare_f64(a[1], b[1]))
        .then(compare_f64(a[2], b[2]))
}

fn compare_f64(a: f64, b: f64) -> Ordering {
    match a.partial_cmp(&b) {
        Some(ordering) => ordering,
        None => Ordering::Equal,
    }
}

fn distance_squared(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

fn coerce_count(
    value: Option<&Value>,
    fallback: usize,
    context: &str,
) -> Result<usize, ComponentError> {
    match value {
        None => Ok(fallback),
        Some(entry) => {
            let number = coerce_number(Some(entry), context)?;
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

fn coerce_point(value: &Value, context: &str) -> Result<[f64; 3], ComponentError> {
    match value {
        Value::Point(point) | Value::Vector(point) => Ok(*point),
        Value::List(values) if values.len() == 1 => coerce_point(&values[0], context),
        Value::List(values) if values.len() >= 3 => {
            let x = coerce_number(Some(&values[0]), context)?;
            let y = coerce_number(Some(&values[1]), context)?;
            let z = coerce_number(Some(&values[2]), context)?;
            Ok([x, y, z])
        }
        other => Err(ComponentError::new(format!(
            "{} verwacht een punt, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn collect_points(value: Option<&Value>, context: &str) -> Result<Vec<[f64; 3]>, ComponentError> {
    let mut points = Vec::new();
    if let Some(value) = value {
        collect_points_into(value, context, &mut points)?;
    }
    Ok(points)
}

fn collect_points_into(
    value: &Value,
    context: &str,
    output: &mut Vec<[f64; 3]>,
) -> Result<(), ComponentError> {
    match value {
        Value::Point(point) | Value::Vector(point) => {
            output.push(*point);
            Ok(())
        }
        Value::List(values) => {
            if let Ok(point) = coerce_point(value, context) {
                output.push(point);
                return Ok(());
            }
            for entry in values {
                collect_points_into(entry, context, output)?;
            }
            Ok(())
        }
        Value::Number(number) => {
            output.push([*number, 0.0, 0.0]);
            Ok(())
        }
        Value::Boolean(boolean) => {
            output.push([if *boolean { 1.0 } else { 0.0 }, 0.0, 0.0]);
            Ok(())
        }
        Value::Text(text) => {
            if let Ok(parsed) = text.trim().parse::<f64>() {
                output.push([parsed, 0.0, 0.0]);
                Ok(())
            } else {
                Err(ComponentError::new(format!(
                    "{} kon tekst '{}' niet als punt interpreteren",
                    context, text
                )))
            }
        }
        other => Err(ComponentError::new(format!(
            "{} verwacht puntwaarden, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn collect_numbers(value: Option<&Value>, context: &str) -> Result<Vec<f64>, ComponentError> {
    let mut numbers = Vec::new();
    if let Some(value) = value {
        collect_numbers_into(value, context, &mut numbers)?;
    }
    Ok(numbers)
}

fn collect_numbers_into(
    value: &Value,
    context: &str,
    output: &mut Vec<f64>,
) -> Result<(), ComponentError> {
    match value {
        Value::Number(number) => {
            output.push(*number);
            Ok(())
        }
        Value::Boolean(boolean) => {
            output.push(if *boolean { 1.0 } else { 0.0 });
            Ok(())
        }
        Value::Point(point) | Value::Vector(point) => {
            output.extend(point);
            Ok(())
        }
        Value::List(values) => {
            for entry in values {
                collect_numbers_into(entry, context, output)?;
            }
            Ok(())
        }
        Value::Text(text) => {
            if let Ok(parsed) = text.trim().parse::<f64>() {
                output.push(parsed);
                Ok(())
            } else {
                Err(ComponentError::new(format!(
                    "{} kon tekst '{}' niet als getal interpreteren",
                    context, text
                )))
            }
        }
        other => Err(ComponentError::new(format!(
            "{} verwacht numerieke waarden, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_number(value: Option<&Value>, context: &str) -> Result<f64, ComponentError> {
    match value {
        None => Err(ComponentError::new(format!(
            "{} vereist een numerieke waarde",
            context
        ))),
        Some(value) => match value {
            Value::Number(number) => Ok(*number),
            Value::Boolean(boolean) => Ok(if *boolean { 1.0 } else { 0.0 }),
            Value::List(values) if values.len() == 1 => coerce_number(values.get(0), context),
            Value::Text(text) => text.trim().parse::<f64>().map_err(|_| {
                ComponentError::new(format!(
                    "{} kon tekst '{}' niet als getal interpreteren",
                    context, text
                ))
            }),
            other => Err(ComponentError::new(format!(
                "{} verwacht een getal, kreeg {}",
                context,
                other.kind()
            ))),
        },
    }
}

fn parse_mask(value: Option<&Value>) -> Vec<char> {
    let mut axes = Vec::new();
    if let Some(value) = value {
        collect_mask(value, &mut axes);
    }
    if axes.is_empty() {
        axes.extend(['x', 'y', 'z']);
    }
    axes.retain(|axis| matches!(*axis, 'x' | 'y' | 'z'));
    if axes.is_empty() {
        axes.extend(['x', 'y', 'z']);
    }
    axes
}

fn collect_mask(value: &Value, output: &mut Vec<char>) {
    match value {
        Value::List(values) => {
            for entry in values {
                collect_mask(entry, output);
            }
        }
        Value::Text(text) => {
            for ch in text.chars() {
                let lower = ch.to_ascii_lowercase();
                if matches!(lower, 'x' | 'y' | 'z') {
                    output.push(lower);
                }
            }
        }
        Value::Number(_)
        | Value::Boolean(_)
        | Value::Point(_)
        | Value::Vector(_)
        | Value::CurveLine { .. }
        | Value::Surface { .. }
        | Value::Domain(_)
        | Value::Matrix(_)
        | Value::DateTime(_)
        | Value::Complex(_) => {
            // Geen maskinformatie aanwezig.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Component, ComponentKind, PIN_OUTPUT_DISTANCE, PIN_OUTPUT_INDEX, PIN_OUTPUT_INDICES,
        PIN_OUTPUT_NUMBERS, PIN_OUTPUT_POINT, PIN_OUTPUT_POINTS, PIN_OUTPUT_VALENCE, collect_mask,
        collect_numbers, collect_points, compare_points,
    };
    use crate::graph::node::MetaMap;
    use crate::graph::value::Value;
    use std::cmp::Ordering;

    #[test]
    fn numbers_to_points_creates_points() {
        let component = ComponentKind::NumbersToPoints;
        let outputs = component
            .evaluate(
                &[
                    Value::List(vec![
                        Value::Number(0.0),
                        Value::Number(1.0),
                        Value::Number(2.0),
                        Value::Number(3.0),
                        Value::Number(4.0),
                        Value::Number(5.0),
                    ]),
                    Value::Text("xyz".into()),
                ],
                &MetaMap::new(),
            )
            .expect("numbers to points succeeds");
        let points = outputs
            .get(PIN_OUTPUT_POINTS)
            .and_then(|value| value.expect_list().ok())
            .expect("points output present");
        assert_eq!(points.len(), 2);
        assert!(matches!(points[0], Value::Point([0.0, 1.0, 2.0])));
        assert!(matches!(points[1], Value::Point([3.0, 4.0, 5.0])));
    }

    #[test]
    fn points_to_numbers_extracts_coordinates() {
        let component = ComponentKind::PointsToNumbers;
        let outputs = component
            .evaluate(
                &[
                    Value::List(vec![
                        Value::Point([1.0, 2.0, 3.0]),
                        Value::Point([4.0, 5.0, 6.0]),
                    ]),
                    Value::Text("xy".into()),
                ],
                &MetaMap::new(),
            )
            .expect("points to numbers succeeds");
        let numbers = outputs
            .get(PIN_OUTPUT_NUMBERS)
            .and_then(|value| value.expect_list().ok())
            .expect("numbers output present");
        let values: Vec<f64> = numbers
            .iter()
            .map(|value| value.expect_number().unwrap())
            .collect();
        assert_eq!(values, vec![1.0, 2.0, 4.0, 5.0]);
    }

    #[test]
    fn distance_between_points_is_calculated() {
        let component = ComponentKind::Distance;
        let outputs = component
            .evaluate(
                &[Value::Point([0.0, 0.0, 0.0]), Value::Point([3.0, 4.0, 0.0])],
                &MetaMap::new(),
            )
            .expect("distance succeeds");
        let distance = outputs
            .get(PIN_OUTPUT_DISTANCE)
            .and_then(|value| value.expect_number().ok())
            .expect("distance output present");
        assert!((distance - 5.0).abs() < 1e-9);
    }

    #[test]
    fn closest_point_returns_nearest_candidate() {
        let component = ComponentKind::ClosestPoint;
        let outputs = component
            .evaluate(
                &[
                    Value::Point([0.0, 0.0, 0.0]),
                    Value::List(vec![
                        Value::Point([2.0, 0.0, 0.0]),
                        Value::Point([1.0, 0.0, 0.0]),
                    ]),
                ],
                &MetaMap::new(),
            )
            .expect("closest point succeeds");
        let point = outputs
            .get(PIN_OUTPUT_POINT)
            .and_then(|value| value.expect_point().ok())
            .expect("point output present");
        assert_eq!(point, [1.0, 0.0, 0.0]);
        let index = outputs
            .get(PIN_OUTPUT_INDEX)
            .and_then(|value| value.expect_number().ok())
            .unwrap();
        assert_eq!(index, 1.0);
    }

    #[test]
    fn closest_points_respects_requested_count() {
        let component = ComponentKind::ClosestPoints;
        let outputs = component
            .evaluate(
                &[
                    Value::Point([0.0, 0.0, 0.0]),
                    Value::List(vec![
                        Value::Point([5.0, 0.0, 0.0]),
                        Value::Point([1.0, 0.0, 0.0]),
                        Value::Point([2.0, 0.0, 0.0]),
                    ]),
                    Value::Number(2.0),
                ],
                &MetaMap::new(),
            )
            .expect("closest points succeeds");
        let points = outputs
            .get(PIN_OUTPUT_POINTS)
            .and_then(|value| value.expect_list().ok())
            .unwrap();
        assert_eq!(points.len(), 2);
        assert!(matches!(points[0], Value::Point([1.0, 0.0, 0.0])));
        assert!(matches!(points[1], Value::Point([2.0, 0.0, 0.0])));

        let indices = outputs
            .get(PIN_OUTPUT_INDICES)
            .and_then(|value| value.expect_list().ok())
            .unwrap();
        let values: Vec<f64> = indices
            .iter()
            .map(|value| value.expect_number().unwrap())
            .collect();
        assert_eq!(values, vec![1.0, 2.0]);
    }

    #[test]
    fn sort_points_orders_by_coordinates() {
        let component = ComponentKind::SortPoints;
        let outputs = component
            .evaluate(
                &[Value::List(vec![
                    Value::Point([1.0, 2.0, 3.0]),
                    Value::Point([0.0, 2.0, 5.0]),
                    Value::Point([1.0, 1.0, 4.0]),
                ])],
                &MetaMap::new(),
            )
            .expect("sort points succeeds");
        let points = outputs
            .get(PIN_OUTPUT_POINTS)
            .and_then(|value| value.expect_list().ok())
            .unwrap();
        let sorted: Vec<[f64; 3]> = points
            .iter()
            .map(|value| value.expect_point().unwrap())
            .collect();
        assert_eq!(
            sorted,
            vec![[0.0, 2.0, 5.0], [1.0, 1.0, 4.0], [1.0, 2.0, 3.0]]
        );

        let indices = outputs
            .get(PIN_OUTPUT_INDICES)
            .and_then(|value| value.expect_list().ok())
            .unwrap();
        let values: Vec<f64> = indices
            .iter()
            .map(|value| value.expect_number().unwrap())
            .collect();
        assert_eq!(values, vec![1.0, 2.0, 0.0]);
    }

    #[test]
    fn cull_duplicates_removes_close_points() {
        let component = ComponentKind::CullDuplicates;
        let outputs = component
            .evaluate(
                &[
                    Value::List(vec![
                        Value::Point([0.0, 0.0, 0.0]),
                        Value::Point([0.0, 0.0, 0.0001]),
                        Value::Point([1.0, 0.0, 0.0]),
                    ]),
                    Value::Number(0.001),
                ],
                &MetaMap::new(),
            )
            .expect("cull duplicates succeeds");
        let points = outputs
            .get(PIN_OUTPUT_POINTS)
            .and_then(|value| value.expect_list().ok())
            .unwrap();
        assert_eq!(points.len(), 2);
        let valence = outputs
            .get(PIN_OUTPUT_VALENCE)
            .and_then(|value| value.expect_list().ok())
            .unwrap();
        let counts: Vec<f64> = valence
            .iter()
            .map(|value| value.expect_number().unwrap())
            .collect();
        assert_eq!(counts, vec![2.0, 1.0]);
    }

    #[test]
    fn barycentric_combines_anchor_points() {
        let component = ComponentKind::Barycentric;
        let outputs = component
            .evaluate(
                &[
                    Value::Point([0.0, 0.0, 0.0]),
                    Value::Point([1.0, 0.0, 0.0]),
                    Value::Point([0.0, 1.0, 0.0]),
                    Value::Number(0.25),
                    Value::Number(0.25),
                    Value::Number(0.5),
                ],
                &MetaMap::new(),
            )
            .expect("barycentric succeeds");
        let point = outputs
            .get(PIN_OUTPUT_POINT)
            .and_then(|value| value.expect_point().ok())
            .unwrap();
        assert!((point[0] - 0.25).abs() < 1e-9);
        assert!((point[1] - 0.5).abs() < 1e-9);
        assert!(point[2].abs() < 1e-9);
    }

    #[test]
    fn compare_points_orders_correctly() {
        let a = [0.0, 1.0, 2.0];
        let b = [0.0, 2.0, 1.0];
        assert!(matches!(compare_points(a, b), Ordering::Less));
    }

    #[test]
    fn collect_points_parses_nested_lists() {
        let points = collect_points(
            Some(&Value::List(vec![
                Value::List(vec![
                    Value::Number(1.0),
                    Value::Number(2.0),
                    Value::Number(3.0),
                ]),
                Value::Point([4.0, 5.0, 6.0]),
            ])),
            "Collect",
        )
        .expect("collect points succeeds");
        assert_eq!(points.len(), 2);
    }

    #[test]
    fn collect_numbers_gathers_from_points() {
        let numbers = collect_numbers(Some(&Value::Point([1.0, 2.0, 3.0])), "Collect")
            .expect("collect numbers succeeds");
        assert_eq!(numbers, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn collect_mask_defaults_to_xyz() {
        let mut mask = Vec::new();
        collect_mask(&Value::Text("yz".into()), &mut mask);
        assert_eq!(mask, vec!['y', 'z']);
    }
}
