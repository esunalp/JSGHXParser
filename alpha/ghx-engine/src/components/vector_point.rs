//! Implementaties van Grasshopper "Vector → Point" componenten.

use std::cmp::Ordering;
use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::{ColorValue, PlaneValue, TextTagValue, Value};

use super::{coerce, Component, ComponentError, ComponentResult};

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
const PIN_OUTPUT_GROUPS: &str = "G";
const PIN_OUTPUT_PHI: &str = "P";
const PIN_OUTPUT_THETA: &str = "T";
const PIN_OUTPUT_RADIUS: &str = "R";
const PIN_OUTPUT_TAGS: &str = "Tag";

const EPSILON: f64 = 1e-9;

/// Beschikbare componenten binnen deze module.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    ConstructPoint,
    NumbersToPoints,
    TextTag3D,
    TextTag,
    PointsToNumbers,
    Distance,
    Deconstruct,
    ClosestPoint,
    ClosestPoints,
    SortPoints,
    CullDuplicates,
    Barycentric,
    ConstructPointOriented,
    PointOriented,
    PointCylindrical,
    PointPolar,
    ToPolar,
    SortAlongCurve,
    PointGroups,
    ProjectPoint,
    PullPoint,
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
        guids: &["{3581f42a-9592-4549-bd6b-1c0fc39d067b}"],
        names: &["Construct Point", "Pt"],
        kind: ComponentKind::ConstructPoint,
    },
    Registration {
        guids: &["{0ae07da9-951b-4b9b-98ca-d312c252374d}"],
        names: &["Numbers to Points", "Num2Pt"],
        kind: ComponentKind::NumbersToPoints,
    },
    Registration {
        guids: &[
            "{18564c36-5652-4c63-bb6f-f0e1273666dd}",
            "{ebf4d987-09b9-4825-a735-cac3d4770c19}",
        ],
        names: &["Text Tag 3D", "Tag 3D", "Text Tag3D"],
        kind: ComponentKind::TextTag3D,
    },
    Registration {
        guids: &["{4b3d38d3-0620-42e5-9ae8-0d4d9ad914cd}"],
        names: &["Text Tag", "Tag"],
        kind: ComponentKind::TextTag,
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
        guids: &["{670fcdba-da07-4eb4-b1c1-bfa0729d767d}"],
        names: &["Deconstruct Point", "DePoint"],
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
    Registration {
        guids: &["{8a5aae11-8775-4ee5-b4fc-db3a1bd89c2f}"],
        names: &["Construct Point Oriented", "Pt Orient"],
        kind: ComponentKind::ConstructPointOriented,
    },
    Registration {
        guids: &["{aa333235-5922-424c-9002-1e0b866a854b}"],
        names: &["Point Oriented", "Point UVW"],
        kind: ComponentKind::PointOriented,
    },
    Registration {
        guids: &["{23603075-be64-4d86-9294-c3c125a12104}"],
        names: &["Point Cylindrical", "Point Cylinder"],
        kind: ComponentKind::PointCylindrical,
    },
    Registration {
        guids: &["{a435f5c8-28a2-43e8-a52a-0b6e73c2e300}"],
        names: &["Point Polar", "Point Spherical"],
        kind: ComponentKind::PointPolar,
    },
    Registration {
        guids: &["{61647ba2-31eb-4921-9632-df81e3286f7d}"],
        names: &["To Polar", "Point To Polar"],
        kind: ComponentKind::ToPolar,
    },
    Registration {
        guids: &["{59aaebf8-6654-46b7-8386-89223c773978}"],
        names: &["Sort Along Curve", "AlongCrv"],
        kind: ComponentKind::SortAlongCurve,
    },
    Registration {
        guids: &["{81f6afc9-22d9-49f0-8579-1fd7e0df6fa6}"],
        names: &["Point Groups", "PGroups"],
        kind: ComponentKind::PointGroups,
    },
    Registration {
        guids: &["{5184b8cb-b71e-4def-a590-cd2c9bc58906}"],
        names: &["Project Point", "Project"],
        kind: ComponentKind::ProjectPoint,
    },
    Registration {
        guids: &[
            "{902289da-28dc-454b-98d4-b8f8aa234516}",
            "{cf3a0865-4882-46bd-91a1-d512acf95be4}",
        ],
        names: &["Pull Point", "Pull"],
        kind: ComponentKind::PullPoint,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::ConstructPoint => evaluate_construct_point(inputs),
            Self::NumbersToPoints => evaluate_numbers_to_points(inputs),
            Self::TextTag3D => evaluate_text_tag_3d(inputs),
            Self::TextTag => evaluate_text_tag(inputs),
            Self::PointsToNumbers => evaluate_points_to_numbers(inputs),
            Self::Distance => evaluate_distance(inputs),
            Self::Deconstruct => evaluate_deconstruct(inputs),
            Self::ClosestPoint => evaluate_closest_point(inputs),
            Self::ClosestPoints => evaluate_closest_points(inputs),
            Self::SortPoints => evaluate_sort_points(inputs),
            Self::CullDuplicates => evaluate_cull_duplicates(inputs),
            Self::Barycentric => evaluate_barycentric(inputs),
            Self::ConstructPointOriented => evaluate_construct_point_oriented(inputs),
            Self::PointOriented => evaluate_point_oriented(inputs),
            Self::PointCylindrical => evaluate_point_cylindrical(inputs),
            Self::PointPolar => evaluate_point_polar(inputs),
            Self::ToPolar => evaluate_to_polar(inputs),
            Self::SortAlongCurve => evaluate_sort_along_curve(inputs),
            Self::PointGroups => evaluate_point_groups(inputs),
            Self::ProjectPoint => evaluate_project_point(inputs),
            Self::PullPoint => evaluate_pull_point(inputs),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::ConstructPoint => "Construct Point",
            Self::NumbersToPoints => "Numbers to Points",
            Self::TextTag3D => "Text Tag 3D",
            Self::TextTag => "Text Tag",
            Self::PointsToNumbers => "Points to Numbers",
            Self::Distance => "Point Distance",
            Self::Deconstruct => "Deconstruct Point",
            Self::ClosestPoint => "Closest Point",
            Self::ClosestPoints => "Closest Points",
            Self::SortPoints => "Sort Points",
            Self::CullDuplicates => "Cull Duplicates",
            Self::Barycentric => "Barycentric Point",
            Self::ConstructPointOriented => "Construct Point Oriented",
            Self::PointOriented => "Point Oriented",
            Self::PointCylindrical => "Point Cylindrical",
            Self::PointPolar => "Point Polar",
            Self::ToPolar => "To Polar",
            Self::SortAlongCurve => "Sort Along Curve",
            Self::PointGroups => "Point Groups",
            Self::ProjectPoint => "Project Point",
            Self::PullPoint => "Pull Point",
        }
    }
}

fn evaluate_text_tag_3d(inputs: &[Value]) -> ComponentResult {
    let context = "Text Tag 3D";
    let planes = collect_tag_planes(inputs.get(0), context)?;
    let texts = collect_texts(inputs.get(1), context)?;
    let mut sizes = collect_numbers(inputs.get(2), context)?;
    if sizes.is_empty() {
        sizes.push(1.0);
    }
    let colors = collect_colors(inputs.get(3));

    let tags = build_tag_values(&planes, &texts, &sizes, &colors);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_TAGS.to_owned(), Value::List(tags));
    Ok(outputs)
}

fn evaluate_text_tag(inputs: &[Value]) -> ComponentResult {
    let context = "Text Tag";
    let planes = collect_tag_planes(inputs.get(0), context)?;
    let texts = collect_texts(inputs.get(1), context)?;
    let sizes = vec![1.0];
    let colors = vec![None];

    let tags = build_tag_values(&planes, &texts, &sizes, &colors);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_TAGS.to_owned(), Value::List(tags));
    Ok(outputs)
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
    let a = coerce::coerce_point_with_default(inputs.get(0));
    let b = coerce::coerce_point_with_default(inputs.get(1));
    let distance = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_DISTANCE.to_owned(), Value::Number(distance));
    Ok(outputs)
}

fn evaluate_deconstruct(inputs: &[Value]) -> ComponentResult {
    let point = coerce::coerce_point_with_default(inputs.get(0));

    let coords = if let Some(system) = inputs.get(1) {
        if let Value::Null = system {
            point
        } else {
            let plane = coerce_plane(system, "Deconstruct Point")?;
            plane_coordinates(point, &plane)
        }
    } else {
        point
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_X.to_owned(), Value::Number(coords[0]));
    outputs.insert(PIN_OUTPUT_Y.to_owned(), Value::Number(coords[1]));
    outputs.insert(PIN_OUTPUT_Z.to_owned(), Value::Number(coords[2]));
    Ok(outputs)
}

fn evaluate_closest_point(inputs: &[Value]) -> ComponentResult {
    let context = "Closest Point";
    let target = coerce::coerce_point_with_default(inputs.get(0));
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
    let target = coerce::coerce_point_with_default(inputs.get(0));
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
    let a = coerce::coerce_point_with_default(inputs.get(0));
    let b = coerce::coerce_point_with_default(inputs.get(1));
    let c = coerce::coerce_point_with_default(inputs.get(2));
    let u = coerce::coerce_number_with_default(inputs.get(3));
    let v = coerce::coerce_number_with_default(inputs.get(4));
    let w = match inputs.get(5) {
        Some(&Value::Null) | None => 1.0 - u - v,
        Some(value) => coerce::coerce_number(value).unwrap_or(1.0 - u - v),
    };

    let point = [
        a[0] * u + b[0] * v + c[0] * w,
        a[1] * u + b[1] * v + c[1] * w,
        a[2] * u + b[2] * v + c[2] * w,
    ];

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINT.to_owned(), Value::Point(point));
    Ok(outputs)
}

fn evaluate_construct_point_oriented(inputs: &[Value]) -> ComponentResult {
    let context = "Construct Point Oriented";
    if inputs.len() < 4 {
        return Err(ComponentError::new(format!(
            "{} vereist drie coördinaten en een referentievlak",
            context
        )));
    }

    let x = coerce_number(Some(&inputs[0]), context)?;
    let y = coerce_number(Some(&inputs[1]), context)?;
    let z = coerce_number(Some(&inputs[2]), context)?;
    let plane = coerce_plane(&inputs[3], context)?;
    let point = apply_plane(&plane, x, y, z);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINT.to_owned(), Value::Point(point));
    Ok(outputs)
}

fn evaluate_point_oriented(inputs: &[Value]) -> ComponentResult {
    let context = "Point Oriented";
    if inputs.len() < 3 {
        return Err(ComponentError::new(format!(
            "{} vereist een vlak en minimaal twee coördinaten",
            context
        )));
    }

    let plane = coerce_plane(&inputs[0], context)?;
    let u = coerce_number(inputs.get(1), context)?;
    let v = coerce_number(inputs.get(2), context)?;
    let w = coerce_number(inputs.get(3), context).unwrap_or(0.0);
    let point = apply_plane(&plane, u, v, w);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINT.to_owned(), Value::Point(point));
    Ok(outputs)
}

fn evaluate_point_cylindrical(inputs: &[Value]) -> ComponentResult {
    let context = "Point Cylindrical";
    if inputs.len() < 4 {
        return Err(ComponentError::new(format!(
            "{} vereist een vlak, hoek, straal en elevatie",
            context
        )));
    }

    let plane = coerce_plane(&inputs[0], context)?;
    let angle = coerce_number(Some(&inputs[1]), context)?;
    let radius = coerce_number(Some(&inputs[2]), context)?;
    let elevation = coerce_number(Some(&inputs[3]), context)?;
    let x = angle.cos() * radius;
    let y = angle.sin() * radius;
    let point = apply_plane(&plane, x, y, elevation);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINT.to_owned(), Value::Point(point));
    Ok(outputs)
}

fn evaluate_point_polar(inputs: &[Value]) -> ComponentResult {
    let context = "Point Polar";
    if inputs.len() < 4 {
        return Err(ComponentError::new(format!(
            "{} vereist een vlak en drie hoek/distantiewaarden",
            context
        )));
    }

    let plane = coerce_plane(&inputs[0], context)?;
    let phi = coerce_number(Some(&inputs[1]), context)?;
    let theta = coerce_number(Some(&inputs[2]), context)?;
    let distance = coerce_number(Some(&inputs[3]), context)?;
    let horizontal = distance * theta.cos();
    let x = phi.cos() * horizontal;
    let y = phi.sin() * horizontal;
    let z = theta.sin() * distance;
    let point = apply_plane(&plane, x, y, z);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINT.to_owned(), Value::Point(point));
    Ok(outputs)
}

fn evaluate_to_polar(inputs: &[Value]) -> ComponentResult {
    let context = "Point To Polar";
    if inputs.is_empty() {
        return Err(ComponentError::new(format!(
            "{} vereist minimaal een punt",
            context
        )));
    }

    let point = coerce_point(&inputs[0], context)?;
    let plane = if let Some(system) = inputs.get(1) {
        coerce_plane(system, context)?
    } else {
        Plane::default()
    };
    let coords = plane_coordinates(point, &plane);
    let horizontal = (coords[0] * coords[0] + coords[1] * coords[1]).sqrt();
    let radius = (coords[0] * coords[0] + coords[1] * coords[1] + coords[2] * coords[2]).sqrt();
    let phi = coords[1].atan2(coords[0]);
    let theta = coords[2].atan2(horizontal);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_PHI.to_owned(), Value::Number(phi));
    outputs.insert(PIN_OUTPUT_THETA.to_owned(), Value::Number(theta));
    outputs.insert(PIN_OUTPUT_RADIUS.to_owned(), Value::Number(radius));
    Ok(outputs)
}

fn evaluate_sort_along_curve(inputs: &[Value]) -> ComponentResult {
    let context = "Sort Along Curve";
    if inputs.len() < 2 {
        return Err(ComponentError::new(format!(
            "{} vereist een puntenlijst en een curve",
            context
        )));
    }

    let points = collect_points(inputs.get(0), context)?;
    if points.is_empty() {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::List(Vec::new()));
        outputs.insert(PIN_OUTPUT_INDICES.to_owned(), Value::List(Vec::new()));
        return Ok(outputs);
    }

    let curve = coerce_line(
        inputs.get(1).ok_or_else(|| {
            ComponentError::new(format!("{} vereist een curve als tweede invoer", context))
        })?,
        context,
    )?;
    let direction = curve.direction();
    let length_sq = vector_length_squared(direction);

    let mut entries: Vec<(f64, usize, [f64; 3])> = points
        .into_iter()
        .enumerate()
        .map(|(index, point)| {
            let relative = subtract(point, curve.start);
            let parameter = if length_sq < EPSILON {
                vector_length(relative)
            } else {
                dot(relative, direction) / length_sq
            };
            (parameter, index, point)
        })
        .collect();
    entries.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));

    let mut sorted_points = Vec::with_capacity(entries.len());
    let mut indices = Vec::with_capacity(entries.len());
    for (_, original_index, point) in entries {
        sorted_points.push(Value::Point(point));
        indices.push(Value::Number(original_index as f64));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::List(sorted_points));
    outputs.insert(PIN_OUTPUT_INDICES.to_owned(), Value::List(indices));
    Ok(outputs)
}

fn evaluate_point_groups(inputs: &[Value]) -> ComponentResult {
    let context = "Point Groups";
    if inputs.is_empty() {
        return Err(ComponentError::new(format!(
            "{} vereist minimaal een puntenlijst",
            context
        )));
    }

    let points = collect_points(inputs.get(0), context)?;
    let distance = coerce_number(inputs.get(1), context)
        .unwrap_or(0.1)
        .max(0.0);
    let mut outputs = BTreeMap::new();

    if points.is_empty() {
        outputs.insert(PIN_OUTPUT_GROUPS.to_owned(), Value::List(Vec::new()));
        outputs.insert(PIN_OUTPUT_INDICES.to_owned(), Value::List(Vec::new()));
        return Ok(outputs);
    }

    let threshold_sq = distance * distance;
    let mut parents: Vec<usize> = (0..points.len()).collect();
    let mut ranks = vec![0u8; points.len()];

    for i in 0..points.len() {
        for j in (i + 1)..points.len() {
            if distance_squared(points[i], points[j]) <= threshold_sq {
                union_sets(&mut parents, &mut ranks, i, j);
            }
        }
    }

    let mut groups: BTreeMap<usize, (Vec<Value>, Vec<Value>)> = BTreeMap::new();
    for (index, point) in points.iter().enumerate() {
        let root = find_parent(&mut parents, index);
        let entry = groups
            .entry(root)
            .or_insert_with(|| (Vec::new(), Vec::new()));
        entry.0.push(Value::Point(*point));
        entry.1.push(Value::Number(index as f64));
    }

    let group_values: Vec<Value> = groups
        .values()
        .map(|(pts, _)| Value::List(pts.clone()))
        .collect();
    let index_values: Vec<Value> = groups
        .values()
        .map(|(_, idx)| Value::List(idx.clone()))
        .collect();

    outputs.insert(PIN_OUTPUT_GROUPS.to_owned(), Value::List(group_values));
    outputs.insert(PIN_OUTPUT_INDICES.to_owned(), Value::List(index_values));
    Ok(outputs)
}

fn evaluate_project_point(inputs: &[Value]) -> ComponentResult {
    let context = "Project Point";
    if inputs.len() < 3 {
        return Err(ComponentError::new(format!(
            "{} vereist een punt, richting en geometrie",
            context
        )));
    }

    let origin = coerce_point(&inputs[0], context)?;
    let mut direction = coerce_vector(&inputs[1], context)?;
    let length_sq = vector_length_squared(direction);
    if length_sq < EPSILON {
        return Err(ComponentError::new(
            "Project Point vereist een geldige richtingsvector",
        ));
    }
    direction = scale(direction, 1.0 / length_sq.sqrt());

    let planes = collect_planes(inputs.get(2), context)?;
    if planes.is_empty() {
        return Err(ComponentError::new(
            "Project Point ondersteunt momenteel alleen vlak-geometry",
        ));
    }

    let mut best: Option<(f64, usize, [f64; 3])> = None;
    for (index, plane) in planes.iter().enumerate() {
        if let Some((intersection, distance)) = intersect_ray_plane(origin, direction, plane) {
            if distance >= 0.0
                && best
                    .as_ref()
                    .map(|(best_distance, _, _)| distance < *best_distance)
                    .unwrap_or(true)
            {
                best = Some((distance, index, intersection));
            }
        }
    }

    let (_, index, intersection) = best.ok_or_else(|| {
        ComponentError::new("Project Point vond geen snijpunt met de opgegeven geometrie")
    })?;

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINT.to_owned(), Value::Point(intersection));
    outputs.insert(PIN_OUTPUT_INDEX.to_owned(), Value::Number(index as f64));
    Ok(outputs)
}

fn evaluate_construct_point(inputs: &[Value]) -> ComponentResult {
    let x = coerce::coerce_number_with_default(inputs.get(0));
    let y = coerce::coerce_number_with_default(inputs.get(1));
    let z = coerce::coerce_number_with_default(inputs.get(2));

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINT.to_owned(), Value::Point([x, y, z]));
    Ok(outputs)
}

fn evaluate_pull_point(inputs: &[Value]) -> ComponentResult {
    let context = "Pull Point";
    let point = coerce::coerce_point_with_default(inputs.get(0));
    let prefer_closest = coerce::coerce_boolean_with_default(inputs.get(2));

    let planes = collect_planes(inputs.get(1), context)?;
    let point_candidates = collect_points(inputs.get(1), context)?;

    let mut candidates: Vec<([f64; 3], f64)> = Vec::new();
    for plane in planes {
        let coords = plane_coordinates(point, &plane);
        let projection = apply_plane(&plane, coords[0], coords[1], 0.0);
        candidates.push((projection, coords[2].abs()));
    }
    for candidate in point_candidates {
        let distance = distance_squared(candidate, point).sqrt();
        candidates.push((candidate, distance));
    }

    if candidates.is_empty() {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_POINT.to_owned(), Value::Point(point));
        outputs.insert(PIN_OUTPUT_DISTANCE.to_owned(), Value::Number(0.0));
        return Ok(outputs);
    }

    candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
    let chosen = if prefer_closest {
        candidates.first().unwrap()
    } else {
        candidates.last().unwrap()
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINT.to_owned(), Value::Point(chosen.0));
    outputs.insert(PIN_OUTPUT_DISTANCE.to_owned(), Value::Number(chosen.1));
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

fn collect_tag_planes(value: Option<&Value>, context: &str) -> Result<Vec<Plane>, ComponentError> {
    let mut planes = collect_planes(value, context)?;
    if planes.is_empty() {
        if let Some(value) = value {
            planes.push(coerce_plane(value, context)?);
        } else {
            planes.push(Plane::default());
        }
    }
    Ok(planes)
}

fn collect_texts(value: Option<&Value>, context: &str) -> Result<Vec<String>, ComponentError> {
    let mut texts = Vec::new();
    if let Some(value) = value {
        collect_texts_into(value, context, &mut texts)?;
    }
    if texts.is_empty() {
        texts.push(String::new());
    }
    Ok(texts)
}

fn collect_texts_into(
    value: &Value,
    context: &str,
    output: &mut Vec<String>,
) -> Result<(), ComponentError> {
    match value {
        Value::Text(text) => {
            output.push(text.clone());
            Ok(())
        }
        Value::Number(number) => {
            output.push(number.to_string());
            Ok(())
        }
        Value::Boolean(boolean) => {
            output.push(boolean.to_string());
            Ok(())
        }
        Value::List(values) => {
            if values.is_empty() {
                return Ok(());
            }
            if values.len() == 1 {
                collect_texts_into(&values[0], context, output)
            } else {
                for entry in values {
                    collect_texts_into(entry, context, output)?;
                }
                Ok(())
            }
        }
        other => Err(ComponentError::new(format!(
            "{} verwacht tekstuele waarden, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn collect_colors(value: Option<&Value>) -> Vec<Option<ColorValue>> {
    let mut colors = Vec::new();
    if let Some(value) = value {
        collect_colors_into(value, &mut colors);
    }
    if colors.is_empty() {
        colors.push(None);
    }
    colors
}

fn collect_colors_into(value: &Value, output: &mut Vec<Option<ColorValue>>) {
    match value {
        Value::List(values) => {
            if let Some(color) = parse_color_value(value) {
                output.push(Some(color));
            } else {
                for entry in values {
                    collect_colors_into(entry, output);
                }
            }
        }
        other => output.push(parse_color_value(other)),
    }
}

fn build_tag_values(
    planes: &[Plane],
    texts: &[String],
    sizes: &[f64],
    colors: &[Option<ColorValue>],
) -> Vec<Value> {
    let count = planes
        .len()
        .max(texts.len())
        .max(sizes.len())
        .max(colors.len())
        .max(1);
    let plane_fallback = planes.get(0).copied().unwrap_or_else(Plane::default);
    let text_fallback = texts.get(0).cloned().unwrap_or_default();
    let size_fallback = sizes.get(0).copied().unwrap_or(1.0);
    let color_fallback = colors.get(0).cloned().unwrap_or(None);

    let mut tags = Vec::with_capacity(count);
    for index in 0..count {
        let plane = planes.get(index).copied().unwrap_or(plane_fallback);
        let text = texts
            .get(index)
            .cloned()
            .unwrap_or_else(|| text_fallback.clone());
        let size = sizes.get(index).copied().unwrap_or(size_fallback).max(0.0);
        let color = colors.get(index).cloned().unwrap_or(color_fallback);
        let tag = TextTagValue::new(plane.to_value(), text, size, color);
        tags.push(Value::Tag(tag));
    }
    tags
}

pub(crate) fn parse_color_value(value: &Value) -> Option<ColorValue> {
    match value {
        Value::Number(number) => parse_color_from_number(*number),
        Value::Boolean(boolean) => Some(if *boolean {
            ColorValue::new(1.0, 1.0, 1.0)
        } else {
            ColorValue::new(0.0, 0.0, 0.0)
        }),
        Value::Point(point) | Value::Vector(point) => {
            Some(ColorValue::new(point[0], point[1], point[2]))
        }
        Value::List(values) => {
            if values.is_empty() {
                return None;
            }
            if values.len() >= 3 {
                let mut components = Vec::new();
                for entry in values.iter().take(3) {
                    if let Some(number) = parse_color_number(entry) {
                        components.push(number);
                    } else {
                        return None;
                    }
                }
                if components.iter().any(|value| value.abs() > 1.0) {
                    Some(ColorValue::from_rgb255(
                        components[0],
                        components[1],
                        components[2],
                    ))
                } else {
                    Some(ColorValue::new(components[0], components[1], components[2]))
                }
            } else if values.len() == 1 {
                parse_color_value(&values[0])
            } else {
                None
            }
        }
        Value::Text(text) => parse_color_text(text),
        Value::Null
        | Value::CurveLine { .. }
        | Value::Surface { .. }
        | Value::Domain(_)
        | Value::Matrix(_)
        | Value::DateTime(_)
        | Value::Complex(_)
        | Value::Color(_)
        | Value::Material(_)
        | Value::Symbol(_)
        | Value::Tag(_) => None,
    }
}

fn parse_color_number(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => Some(*number),
        Value::Boolean(boolean) => Some(if *boolean { 1.0 } else { 0.0 }),
        Value::Text(text) => text.trim().parse::<f64>().ok(),
        Value::List(values) if values.len() == 1 => parse_color_number(&values[0]),
        _ => None,
    }
}

fn parse_color_from_number(number: f64) -> Option<ColorValue> {
    if !number.is_finite() {
        return None;
    }
    if number.abs() <= 1.0 {
        return Some(ColorValue::new(number, number, number));
    }
    if (0.0..=255.0).contains(&number) {
        return Some(ColorValue::from_rgb255(number, number, number));
    }

    let mut encoded = number.round() as i64;
    encoded &= 0x00FF_FFFF;
    let r = ((encoded >> 16) & 0xFF) as f64;
    let g = ((encoded >> 8) & 0xFF) as f64;
    let b = (encoded & 0xFF) as f64;
    Some(ColorValue::from_rgb255(r, g, b))
}

fn parse_color_text(text: &str) -> Option<ColorValue> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(color) = parse_hex_color(trimmed) {
        return Some(color);
    }
    if let Some(color) = parse_delimited_color(trimmed) {
        return Some(color);
    }
    named_color(trimmed)
}

fn parse_hex_color(text: &str) -> Option<ColorValue> {
    let digits = if let Some(stripped) = text.strip_prefix('#') {
        stripped
    } else if let Some(stripped) = text.strip_prefix("0x") {
        stripped
    } else {
        return None;
    };

    let expanded = match digits.len() {
        3 => {
            let mut result = String::with_capacity(6);
            for ch in digits.chars() {
                result.push(ch);
                result.push(ch);
            }
            result
        }
        6 => digits.to_owned(),
        _ => return None,
    };

    u32::from_str_radix(&expanded, 16).ok().map(|value| {
        let r = ((value >> 16) & 0xFF) as f64;
        let g = ((value >> 8) & 0xFF) as f64;
        let b = (value & 0xFF) as f64;
        ColorValue::from_rgb255(r, g, b)
    })
}

fn parse_delimited_color(text: &str) -> Option<ColorValue> {
    let cleaned = text
        .replace(['(', ')'], " ")
        .replace(|c: char| c == ';', " ");
    let tokens: Vec<&str> = cleaned
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|token| !token.is_empty())
        .collect();
    if tokens.len() < 3 {
        return None;
    }
    let mut values = Vec::new();
    for token in tokens.iter().take(3) {
        if let Ok(number) = token.parse::<f64>() {
            values.push(number);
        } else {
            return None;
        }
    }
    if values.iter().any(|component| component.abs() > 1.0) {
        Some(ColorValue::from_rgb255(values[0], values[1], values[2]))
    } else {
        Some(ColorValue::new(values[0], values[1], values[2]))
    }
}

fn named_color(text: &str) -> Option<ColorValue> {
    match text.to_ascii_lowercase().as_str() {
        "white" => Some(ColorValue::new(1.0, 1.0, 1.0)),
        "black" => Some(ColorValue::new(0.0, 0.0, 0.0)),
        "red" => Some(ColorValue::new(1.0, 0.0, 0.0)),
        "green" => Some(ColorValue::new(0.0, 1.0, 0.0)),
        "blue" => Some(ColorValue::new(0.0, 0.0, 1.0)),
        "yellow" => Some(ColorValue::new(1.0, 1.0, 0.0)),
        "magenta" | "fuchsia" => Some(ColorValue::new(1.0, 0.0, 1.0)),
        "cyan" | "aqua" => Some(ColorValue::new(0.0, 1.0, 1.0)),
        "orange" => Some(ColorValue::from_rgb255(255.0, 165.0, 0.0)),
        "purple" => Some(ColorValue::from_rgb255(128.0, 0.0, 128.0)),
        "gray" | "grey" => Some(ColorValue::new(0.5, 0.5, 0.5)),
        _ => None,
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
        Value::Null
        | Value::Number(_)
        | Value::Boolean(_)
        | Value::Point(_)
        | Value::Vector(_)
        | Value::CurveLine { .. }
        | Value::Surface { .. }
        | Value::Domain(_)
        | Value::Matrix(_)
        | Value::DateTime(_)
        | Value::Complex(_)
        | Value::Color(_)
        | Value::Material(_)
        | Value::Symbol(_)
        | Value::Tag(_) => {
            // Geen maskinformatie aanwezig.
        }
    }
}

fn coerce_vector(value: &Value, context: &str) -> Result<[f64; 3], ComponentError> {
    match value {
        Value::Vector(vector) | Value::Point(vector) => Ok(*vector),
        Value::List(values) if values.len() == 1 => coerce_vector(&values[0], context),
        Value::List(values) if values.len() >= 3 => {
            let x = coerce_number(Some(&values[0]), context)?;
            let y = coerce_number(Some(&values[1]), context)?;
            let z = coerce_number(Some(&values[2]), context)?;
            Ok([x, y, z])
        }
        Value::List(values) if values.len() == 2 => {
            let x = coerce_number(Some(&values[0]), context)?;
            let y = coerce_number(Some(&values[1]), context)?;
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

fn coerce_boolean(value: Option<&Value>, context: &str) -> Result<bool, ComponentError> {
    match value {
        None => Err(ComponentError::new(format!(
            "{} vereist een booleaanse waarde",
            context
        ))),
        Some(Value::Boolean(boolean)) => Ok(*boolean),
        Some(Value::Number(number)) => Ok(*number != 0.0),
        Some(Value::List(values)) if values.len() == 1 => coerce_boolean(values.get(0), context),
        Some(other) => Err(ComponentError::new(format!(
            "{} verwacht een booleaanse waarde, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn collect_planes(value: Option<&Value>, context: &str) -> Result<Vec<Plane>, ComponentError> {
    let mut planes = Vec::new();
    if let Some(value) = value {
        collect_planes_into(value, context, &mut planes)?;
    }
    Ok(planes)
}

fn collect_planes_into(
    value: &Value,
    context: &str,
    output: &mut Vec<Plane>,
) -> Result<(), ComponentError> {
    match value {
        Value::List(values) => {
            if let Ok(plane) = coerce_plane(value, context) {
                output.push(plane);
                Ok(())
            } else {
                for entry in values {
                    collect_planes_into(entry, context, output)?;
                }
                Ok(())
            }
        }
        Value::Point(_) | Value::Vector(_) => Ok(()),
        _ => {
            output.push(coerce_plane(value, context)?);
            Ok(())
        }
    }
}

fn coerce_plane(value: &Value, context: &str) -> Result<Plane, ComponentError> {
    match value {
        Value::List(values) if values.len() >= 3 => {
            let a = coerce_point(&values[0], context)?;
            let b = coerce_point(&values[1], context)?;
            let c = coerce_point(&values[2], context)?;
            Ok(Plane::from_points(a, b, c))
        }
        Value::List(values) if values.len() == 2 => {
            let origin = coerce_point(&values[0], context)?;
            let direction = coerce_vector(&values[1], context)?;
            if vector_length_squared(direction) < EPSILON {
                Ok(Plane::default())
            } else {
                let x_axis = normalize(direction);
                let z_axis = orthogonal_vector(x_axis);
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

fn coerce_line(value: &Value, context: &str) -> Result<Line, ComponentError> {
    match value {
        Value::CurveLine { p1, p2 } => Ok(Line {
            start: *p1,
            end: *p2,
        }),
        Value::List(values) if values.len() >= 2 => {
            let start = coerce_point(&values[0], context)?;
            let mut end = coerce_point(&values[1], context)?;
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

fn plane_coordinates(point: [f64; 3], plane: &Plane) -> [f64; 3] {
    let relative = subtract(point, plane.origin);
    [
        dot(relative, plane.x_axis),
        dot(relative, plane.y_axis),
        dot(relative, plane.z_axis),
    ]
}

fn apply_plane(plane: &Plane, u: f64, v: f64, w: f64) -> [f64; 3] {
    add(
        add(
            add(plane.origin, scale(plane.x_axis, u)),
            scale(plane.y_axis, v),
        ),
        scale(plane.z_axis, w),
    )
}

fn intersect_ray_plane(
    origin: [f64; 3],
    direction: [f64; 3],
    plane: &Plane,
) -> Option<([f64; 3], f64)> {
    let normal = plane.z_axis;
    let denom = dot(direction, normal);
    if denom.abs() < EPSILON {
        return None;
    }
    let relative = subtract(plane.origin, origin);
    let distance = dot(relative, normal) / denom;
    if !distance.is_finite() {
        return None;
    }
    let point = add(origin, scale(direction, distance));
    Some((point, distance))
}

fn vector_length_squared(vector: [f64; 3]) -> f64 {
    dot(vector, vector)
}

fn vector_length(vector: [f64; 3]) -> f64 {
    vector_length_squared(vector).sqrt()
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

fn safe_normalized(vector: [f64; 3]) -> Option<([f64; 3], f64)> {
    let length = vector_length(vector);
    if length < EPSILON {
        None
    } else {
        Some((scale(vector, 1.0 / length), length))
    }
}

fn normalize(vector: [f64; 3]) -> [f64; 3] {
    safe_normalized(vector)
        .map(|(unit, _)| unit)
        .unwrap_or([0.0, 0.0, 0.0])
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

fn find_parent(parents: &mut [usize], index: usize) -> usize {
    if parents[index] != index {
        let parent = parents[index];
        let root = find_parent(parents, parent);
        parents[index] = root;
    }
    parents[index]
}

fn union_sets(parents: &mut [usize], ranks: &mut [u8], a: usize, b: usize) {
    let mut root_a = find_parent(parents, a);
    let mut root_b = find_parent(parents, b);
    if root_a == root_b {
        return;
    }
    if ranks[root_a] < ranks[root_b] {
        let temp = root_a;
        root_a = root_b;
        root_b = temp;
    }
    parents[root_b] = root_a;
    if ranks[root_a] == ranks[root_b] {
        ranks[root_a] = ranks[root_a].saturating_add(1);
    }
}

#[derive(Debug, Clone, Copy)]
struct Plane {
    origin: [f64; 3],
    x_axis: [f64; 3],
    y_axis: [f64; 3],
    z_axis: [f64; 3],
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

    fn to_value(self) -> PlaneValue {
        PlaneValue::new(self.origin, self.x_axis, self.y_axis, self.z_axis)
    }
}

#[derive(Debug, Clone, Copy)]
struct Line {
    start: [f64; 3],
    end: [f64; 3],
}

impl Line {
    fn direction(self) -> [f64; 3] {
        subtract(self.end, self.start)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Component, ComponentKind, PIN_OUTPUT_DISTANCE, PIN_OUTPUT_GROUPS, PIN_OUTPUT_INDEX,
        PIN_OUTPUT_INDICES, PIN_OUTPUT_NUMBERS, PIN_OUTPUT_PHI, PIN_OUTPUT_POINT,
        PIN_OUTPUT_POINTS, PIN_OUTPUT_RADIUS, PIN_OUTPUT_TAGS, PIN_OUTPUT_THETA,
        PIN_OUTPUT_VALENCE, collect_mask, collect_numbers, collect_points, compare_points,
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
    fn text_tag_3d_combines_inputs() {
        let component = ComponentKind::TextTag3D;
        let outputs = component
            .evaluate(
                &[
                    Value::List(vec![
                        Value::Point([0.0, 0.0, 0.0]),
                        Value::Point([1.0, 0.0, 0.0]),
                    ]),
                    Value::List(vec![
                        Value::Text("First".into()),
                        Value::Text("Second".into()),
                    ]),
                    Value::List(vec![Value::Number(2.0), Value::Number(3.0)]),
                    Value::List(vec![Value::Text("#ff0000".into()), Value::Number(0.0)]),
                ],
                &MetaMap::new(),
            )
            .expect("text tag 3d succeeds");

        let tags = outputs
            .get(PIN_OUTPUT_TAGS)
            .and_then(|value| value.expect_list().ok())
            .expect("tags output present");
        assert_eq!(tags.len(), 2);

        let first = tags[0].expect_tag().expect("first tag");
        assert_eq!(first.text, "First");
        assert!((first.size - 2.0).abs() < 1e-9);
        assert!(
            matches!(first.color, Some(color) if (color.r - 1.0).abs() < 1e-9 && color.g < 1e-9)
        );

        let second = tags[1].expect_tag().expect("second tag");
        assert_eq!(second.text, "Second");
        assert!((second.size - 3.0).abs() < 1e-9);
        assert!(
            matches!(second.color, Some(color) if color.r < 1e-9 && color.g < 1e-9 && color.b < 1e-9)
        );
    }

    #[test]
    fn text_tag_defaults_when_inputs_missing() {
        let component = ComponentKind::TextTag;
        let outputs = component
            .evaluate(&[Value::Point([2.0, 3.0, 4.0])], &MetaMap::new())
            .expect("text tag defaults");

        let tags = outputs
            .get(PIN_OUTPUT_TAGS)
            .and_then(|value| value.expect_list().ok())
            .expect("tags output present");
        assert_eq!(tags.len(), 1);

        let tag = tags[0].expect_tag().expect("single tag");
        assert_eq!(tag.text, "");
        assert!((tag.size - 1.0).abs() < 1e-9);
        assert!(tag.color.is_none());
        assert_eq!(tag.plane.origin, [2.0, 3.0, 4.0]);
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

    #[test]
    fn construct_point_oriented_respects_plane() {
        let component = ComponentKind::ConstructPointOriented;
        let plane = Value::List(vec![
            Value::Point([10.0, 0.0, 0.0]),
            Value::Point([10.0, 1.0, 0.0]),
            Value::Point([10.0, 0.0, 1.0]),
        ]);
        let outputs = component
            .evaluate(
                &[
                    Value::Number(2.0),
                    Value::Number(3.0),
                    Value::Number(4.0),
                    plane,
                ],
                &MetaMap::new(),
            )
            .expect("construct oriented succeeds");
        let point = outputs
            .get(PIN_OUTPUT_POINT)
            .and_then(|value| value.expect_point().ok())
            .expect("point output present");
        assert_eq!(point, [14.0, 2.0, 3.0]);
    }

    #[test]
    fn point_oriented_uses_plane_coordinates() {
        let component = ComponentKind::PointOriented;
        let plane = Value::List(vec![
            Value::Point([5.0, 5.0, 0.0]),
            Value::Point([5.0, 6.0, 0.0]),
            Value::Point([5.0, 5.0, 1.0]),
        ]);
        let outputs = component
            .evaluate(
                &[plane, Value::Number(1.0), Value::Number(2.0)],
                &MetaMap::new(),
            )
            .expect("point oriented succeeds");
        let point = outputs
            .get(PIN_OUTPUT_POINT)
            .and_then(|value| value.expect_point().ok())
            .unwrap();
        assert_eq!(point, [5.0, 6.0, 2.0]);
    }

    #[test]
    fn point_cylindrical_converts_coordinates() {
        let component = ComponentKind::PointCylindrical;
        let plane = Value::List(vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Point([0.0, 1.0, 0.0]),
        ]);
        let outputs = component
            .evaluate(
                &[
                    plane,
                    Value::Number(std::f64::consts::FRAC_PI_2),
                    Value::Number(2.0),
                    Value::Number(5.0),
                ],
                &MetaMap::new(),
            )
            .expect("point cylindrical succeeds");
        let point = outputs
            .get(PIN_OUTPUT_POINT)
            .and_then(|value| value.expect_point().ok())
            .unwrap();
        assert!((point[0]).abs() < 1e-9);
        assert!((point[1] - 2.0).abs() < 1e-9);
        assert!((point[2] - 5.0).abs() < 1e-9);
    }

    #[test]
    fn point_polar_converts_coordinates() {
        let component = ComponentKind::PointPolar;
        let plane = Value::List(vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Point([0.0, 1.0, 0.0]),
        ]);
        let outputs = component
            .evaluate(
                &[
                    plane,
                    Value::Number(0.0),
                    Value::Number(0.0),
                    Value::Number(5.0),
                ],
                &MetaMap::new(),
            )
            .expect("point polar succeeds");
        let point = outputs
            .get(PIN_OUTPUT_POINT)
            .and_then(|value| value.expect_point().ok())
            .unwrap();
        assert!((point[0] - 5.0).abs() < 1e-9);
        assert!(point[1].abs() < 1e-9);
        assert!(point[2].abs() < 1e-9);
    }

    #[test]
    fn to_polar_returns_angles() {
        let component = ComponentKind::ToPolar;
        let outputs = component
            .evaluate(&[Value::Point([0.0, 1.0, 1.0])], &MetaMap::new())
            .expect("to polar succeeds");
        let phi = outputs
            .get(PIN_OUTPUT_PHI)
            .and_then(|value| value.expect_number().ok())
            .unwrap();
        let theta = outputs
            .get(PIN_OUTPUT_THETA)
            .and_then(|value| value.expect_number().ok())
            .unwrap();
        let radius = outputs
            .get(PIN_OUTPUT_RADIUS)
            .and_then(|value| value.expect_number().ok())
            .unwrap();
        assert!((phi - std::f64::consts::FRAC_PI_2).abs() < 1e-9);
        assert!((theta - std::f64::consts::FRAC_PI_4).abs() < 1e-9);
        assert!((radius - (2.0f64).sqrt()).abs() < 1e-9);
    }

    #[test]
    fn sort_along_curve_orders_points() {
        let component = ComponentKind::SortAlongCurve;
        let curve = Value::CurveLine {
            p1: [0.0, 0.0, 0.0],
            p2: [3.0, 0.0, 0.0],
        };
        let outputs = component
            .evaluate(
                &[
                    Value::List(vec![
                        Value::Point([2.0, 0.0, 0.0]),
                        Value::Point([1.0, 0.0, 0.0]),
                        Value::Point([3.0, 0.0, 0.0]),
                    ]),
                    curve,
                ],
                &MetaMap::new(),
            )
            .expect("sort along curve succeeds");
        let points = outputs
            .get(PIN_OUTPUT_POINTS)
            .and_then(|value| value.expect_list().ok())
            .unwrap();
        let ordered: Vec<[f64; 3]> = points
            .iter()
            .map(|value| value.expect_point().unwrap())
            .collect();
        assert_eq!(
            ordered,
            vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [3.0, 0.0, 0.0]]
        );
    }

    #[test]
    fn point_groups_clusters_by_distance() {
        let component = ComponentKind::PointGroups;
        let outputs = component
            .evaluate(
                &[
                    Value::List(vec![
                        Value::Point([0.0, 0.0, 0.0]),
                        Value::Point([0.05, 0.0, 0.0]),
                        Value::Point([5.0, 0.0, 0.0]),
                    ]),
                    Value::Number(0.1),
                ],
                &MetaMap::new(),
            )
            .expect("point groups succeeds");
        let groups = outputs
            .get(PIN_OUTPUT_GROUPS)
            .and_then(|value| value.expect_list().ok())
            .unwrap();
        assert_eq!(groups.len(), 2);
        let indices = outputs
            .get(PIN_OUTPUT_INDICES)
            .and_then(|value| value.expect_list().ok())
            .unwrap();
        assert_eq!(indices.len(), 2);
    }

    #[test]
    fn project_point_intersects_plane() {
        let component = ComponentKind::ProjectPoint;
        let plane = Value::List(vec![
            Value::Point([0.0, 0.0, 5.0]),
            Value::Point([1.0, 0.0, 5.0]),
            Value::Point([0.0, 1.0, 5.0]),
        ]);
        let outputs = component
            .evaluate(
                &[
                    Value::Point([0.0, 0.0, 0.0]),
                    Value::Vector([0.0, 0.0, 1.0]),
                    plane,
                ],
                &MetaMap::new(),
            )
            .expect("project point succeeds");
        let intersection = outputs
            .get(PIN_OUTPUT_POINT)
            .and_then(|value| value.expect_point().ok())
            .unwrap();
        assert_eq!(intersection, [0.0, 0.0, 5.0]);
        let index = outputs
            .get(PIN_OUTPUT_INDEX)
            .and_then(|value| value.expect_number().ok())
            .unwrap();
        assert_eq!(index, 0.0);
    }

    #[test]
    fn pull_point_prefers_closest_projection() {
        let component = ComponentKind::PullPoint;
        let plane = Value::List(vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Point([0.0, 1.0, 0.0]),
        ]);
        let outputs = component
            .evaluate(&[Value::Point([0.0, 0.0, 5.0]), plane], &MetaMap::new())
            .expect("pull point succeeds");
        let pulled = outputs
            .get(PIN_OUTPUT_POINT)
            .and_then(|value| value.expect_point().ok())
            .unwrap();
        assert_eq!(pulled, [0.0, 0.0, 0.0]);
        let distance = outputs
            .get(PIN_OUTPUT_DISTANCE)
            .and_then(|value| value.expect_number().ok())
            .unwrap();
        assert!((distance - 5.0).abs() < 1e-9);
    }

    #[test]
    fn construct_point_builds_point_from_numbers() {
        let component = ComponentKind::ConstructPoint;
        let outputs = component
            .evaluate(
                &[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)],
                &MetaMap::new(),
            )
            .expect("construct succeeded");
        assert!(matches!(
            outputs.get(PIN_OUTPUT_POINT),
            Some(Value::Point(coords)) if *coords == [1.0, 2.0, 3.0]
        ));
    }

    #[test]
    fn construct_point_collapses_single_item_lists() {
        let component = ComponentKind::ConstructPoint;
        let inputs = [
            Value::List(vec![Value::Number(0.5)]),
            Value::List(vec![Value::Number(1.5)]),
            Value::List(vec![Value::Number(2.5)]),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("list inputs collapse");
        assert!(matches!(
            outputs.get(PIN_OUTPUT_POINT),
            Some(Value::Point(coords)) if *coords == [0.5, 1.5, 2.5]
        ));
    }

    #[test]
    fn construct_point_builds_point_from_text() {
        let component = ComponentKind::ConstructPoint;
        let outputs = component
            .evaluate(
                &[
                    Value::Text("1.0".to_string()),
                    Value::Text("2.0".to_string()),
                    Value::Text("3.0".to_string()),
                ],
                &MetaMap::new(),
            )
            .expect("construct from text succeeded");
        assert!(matches!(
            outputs.get(PIN_OUTPUT_POINT),
            Some(Value::Point(coords)) if *coords == [1.0, 2.0, 3.0]
        ));
    }

    #[test]
    fn construct_point_handles_non_numeric_inputs() {
        let component = ComponentKind::ConstructPoint;
        let outputs = component
            .evaluate(
                &[
                    Value::Number(1.0),
                    Value::Point([0.0, 0.0, 0.0]),
                    Value::Number(2.0),
                ],
                &MetaMap::new(),
            )
            .expect("should handle non-numeric input by coercing");

        assert!(matches!(
            outputs.get(PIN_OUTPUT_POINT),
            Some(Value::Point(coords)) if *coords == [1.0, 0.0, 2.0]
        ));
    }
}

    #[test]
    fn construct_point_defaults_to_origin() {
        let component = ComponentKind::ConstructPoint;
        let outputs = component
            .evaluate(&[], &MetaMap::new())
            .expect("construct with no inputs succeeds");
        assert!(matches!(
            outputs.get(PIN_OUTPUT_POINT),
            Some(Value::Point(coords)) if *coords == [0.0, 0.0, 0.0]
        ));
    }

    #[test]
    fn distance_defaults_to_zero() {
        let component = ComponentKind::Distance;
        let outputs = component
            .evaluate(&[], &MetaMap::new())
            .expect("distance with no inputs succeeds");
        let distance = outputs
            .get(PIN_OUTPUT_DISTANCE)
            .and_then(|value| value.expect_number().ok())
            .expect("distance output present");
        assert!(distance.abs() < 1e-9);
    }

    #[test]
    fn deconstruct_defaults_to_origin() {
        let component = ComponentKind::Deconstruct;
        let outputs = component
            .evaluate(&[], &MetaMap::new())
            .expect("deconstruct with no inputs succeeds");
        let x = outputs.get(PIN_OUTPUT_X).and_then(|v| v.expect_number().ok()).unwrap();
        let y = outputs.get(PIN_OUTPUT_Y).and_then(|v| v.expect_number().ok()).unwrap();
        let z = outputs.get(PIN_OUTPUT_Z).and_then(|v| v.expect_number().ok()).unwrap();
        assert!(x.abs() < 1e-9);
        assert!(y.abs() < 1e-9);
        assert!(z.abs() < 1e-9);
    }

    #[test]
    fn closest_point_defaults_to_origin_target() {
        let component = ComponentKind::ClosestPoint;
        let outputs = component
            .evaluate(
                &[
                    Value::Null, // Missing target point
                    Value::List(vec![
                        Value::Point([2.0, 0.0, 0.0]),
                        Value::Point([1.0, 0.0, 0.0]),
                    ]),
                ],
                &MetaMap::new(),
            )
            .expect("closest point with default target succeeds");
        let point = outputs
            .get(PIN_OUTPUT_POINT)
            .and_then(|value| value.expect_point().ok())
            .expect("point output present");
        assert_eq!(point, [1.0, 0.0, 0.0]); // Closest to origin
        let index = outputs
            .get(PIN_OUTPUT_INDEX)
            .and_then(|value| value.expect_number().ok())
            .unwrap();
        assert_eq!(index, 1.0);
    }

    #[test]
    fn barycentric_defaults_to_origin() {
        let component = ComponentKind::Barycentric;
        let outputs = component
            .evaluate(&[], &MetaMap::new())
            .expect("barycentric with no inputs succeeds");
        let point = outputs
            .get(PIN_OUTPUT_POINT)
            .and_then(|value| value.expect_point().ok())
            .unwrap();
        assert_eq!(point, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn pull_point_defaults_to_origin() {
        let component = ComponentKind::PullPoint;
        let outputs = component
            .evaluate(&[], &MetaMap::new())
            .expect("pull point with no inputs succeeds");
        let pulled = outputs
            .get(PIN_OUTPUT_POINT)
            .and_then(|value| value.expect_point().ok())
            .unwrap();
        assert_eq!(pulled, [0.0, 0.0, 0.0]);
        let distance = outputs
            .get(PIN_OUTPUT_DISTANCE)
            .and_then(|value| value.expect_number().ok())
            .unwrap();
        assert!((distance).abs() < 1e-9);
    }
