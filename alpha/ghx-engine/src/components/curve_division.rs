//! Implementaties van Grasshopper "Curve → Division" componenten.
//!
//! This module uses the `geom::curve` primitives for curve division, sampling,
//! and frame computation. Components remain thin wrappers that coerce inputs,
//! build geom curves, and return the results.

use std::collections::BTreeMap;

use crate::geom::{
    CurveDivisionResult, CurveFrame, Polyline3, SubCurve,
    Point3 as GeomPoint3, Vec3 as GeomVec3,
    curve_arc_length, curve_plane_intersections,
    divide_curve_by_count, divide_curve_by_distance,
    extract_subcurve, frenet_frames, horizontal_frames, perp_frames,
    sample_curve_at, shatter_curve,
};
use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

const PIN_OUTPUT_FRAMES: &str = "F";
const PIN_OUTPUT_PARAMETERS: &str = "t";
const PIN_OUTPUT_POINTS: &str = "P";
const PIN_OUTPUT_TANGENTS: &str = "T";
const PIN_OUTPUT_SEGMENTS: &str = "S";
const PIN_OUTPUT_CONTOURS: &str = "C";
const PIN_OUTPUT_DASHES: &str = "D";
const PIN_OUTPUT_GAPS: &str = "G";
const PIN_OUTPUT_DEVIATION: &str = "d";

/// Beschikbare componenten binnen deze module.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    CurveFrames,
    DivideDistance,
    DivideCurve,
    Shatter,
    ContourExplicit,
    DivideByDeviation,
    Contour,
    HorizontalFrames,
    DashPattern,
    PerpFrames,
    DivideLength,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Volledige lijst met componentregistraties voor de curve-division componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{0e94542a-2e46-4793-9f98-2200b06b28f4}"],
        names: &["Curve Frames", "Frames"],
        kind: ComponentKind::CurveFrames,
    },
    Registration {
        guids: &["{1e531c08-9c80-46d6-8850-1b50d1dae69f}"],
        names: &["Divide Distance", "DivDist"],
        kind: ComponentKind::DivideDistance,
    },
    Registration {
        guids: &["{2162e72e-72fc-4bf8-9459-d4d82fa8aa14}"],
        names: &["Divide Curve", "Divide"],
        kind: ComponentKind::DivideCurve,
    },
    Registration {
        guids: &["{2ad2a4d4-3de1-42f6-a4b8-f71835f35710}"],
        names: &["Shatter"],
        kind: ComponentKind::Shatter,
    },
    Registration {
        guids: &["{3e7e4827-6edd-4e10-93ac-cc234414d2b9}"],
        names: &["Contour (ex)", "Contour Ex"],
        kind: ComponentKind::ContourExplicit,
    },
    Registration {
        guids: &["{6e9c0577-ae4a-4b21-8880-0ec3daf3eb4d}"],
        names: &["Divide By Deviation", "DivideDev"],
        kind: ComponentKind::DivideByDeviation,
    },
    Registration {
        guids: &["{88cff285-7f5e-41b3-96d5-9588ff9a52b1}"],
        names: &["Contour"],
        kind: ComponentKind::Contour,
    },
    Registration {
        guids: &["{8d058945-ce47-4e7c-82af-3269295d7890}"],
        names: &["Horizontal Frames", "HFrames"],
        kind: ComponentKind::HorizontalFrames,
    },
    Registration {
        guids: &["{95866bbe-648e-4e2b-a97c-7d04679e94e0}"],
        names: &["Dash Pattern", "Dash"],
        kind: ComponentKind::DashPattern,
    },
    Registration {
        guids: &["{983c7600-980c-44da-bc53-c804067f667f}"],
        names: &["Perp Frames", "PFrames"],
        kind: ComponentKind::PerpFrames,
    },
    Registration {
        guids: &["{fdc466a9-d3b8-4056-852a-09dba0f74aca}"],
        names: &["Divide Length", "DivLength"],
        kind: ComponentKind::DivideLength,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::CurveFrames => evaluate_curve_frames(inputs),
            Self::DivideDistance => evaluate_divide_distance(inputs),
            Self::DivideCurve => evaluate_divide_curve(inputs),
            Self::Shatter => evaluate_shatter(inputs),
            Self::ContourExplicit => evaluate_contour_explicit(inputs),
            Self::DivideByDeviation => evaluate_divide_by_deviation(inputs),
            Self::Contour => evaluate_contour(inputs),
            Self::HorizontalFrames => evaluate_horizontal_frames(inputs),
            Self::DashPattern => evaluate_dash_pattern(inputs),
            Self::PerpFrames => evaluate_perp_frames(inputs),
            Self::DivideLength => evaluate_divide_length(inputs),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::CurveFrames => "Curve Frames",
            Self::DivideDistance => "Divide Distance",
            Self::DivideCurve => "Divide Curve",
            Self::Shatter => "Shatter",
            Self::ContourExplicit => "Contour (ex)",
            Self::DivideByDeviation => "Divide By Deviation",
            Self::Contour => "Contour",
            Self::HorizontalFrames => "Horizontal Frames",
            Self::DashPattern => "Dash Pattern",
            Self::PerpFrames => "Perp Frames",
            Self::DivideLength => "Divide Length",
        }
    }
}

// ============================================================================
// Geom type conversion helpers
// ============================================================================

/// Converts an array [f64; 3] to a geom Point3.
#[inline]
fn to_geom_point(p: [f64; 3]) -> GeomPoint3 {
    GeomPoint3::new(p[0], p[1], p[2])
}

/// Converts a geom Point3 to an array [f64; 3].
#[inline]
fn from_geom_point(p: GeomPoint3) -> [f64; 3] {
    [p.x, p.y, p.z]
}

/// Converts an array [f64; 3] to a geom Vec3.
#[inline]
fn to_geom_vec(v: [f64; 3]) -> GeomVec3 {
    GeomVec3::new(v[0], v[1], v[2])
}

/// Converts a geom Vec3 to an array [f64; 3].
#[inline]
fn from_geom_vec(v: GeomVec3) -> [f64; 3] {
    [v.x, v.y, v.z]
}

/// Tolerance for detecting if a polyline is closed (first point equals last point).
const CLOSED_POLYLINE_TOLERANCE: f64 = 1e-6;

/// Checks if a polyline is closed by comparing first and last points.
///
/// A polyline is considered closed if the distance between its first and last
/// points is less than [`CLOSED_POLYLINE_TOLERANCE`].
///
/// # Returns
/// `true` if the polyline is closed, `false` otherwise or if fewer than 3 points.
fn is_closed_polyline(points: &[[f64; 3]]) -> bool {
    if points.len() < 3 {
        return false;
    }
    let first = points[0];
    let last = points[points.len() - 1];
    let dx = first[0] - last[0];
    let dy = first[1] - last[1];
    let dz = first[2] - last[2];
    (dx * dx + dy * dy + dz * dz).sqrt() < CLOSED_POLYLINE_TOLERANCE
}

/// Creates a geom Polyline3 from a list of [f64; 3] points.
///
/// Automatically detects if the polyline is closed by comparing the first and
/// last points. If they are within [`CLOSED_POLYLINE_TOLERANCE`], the polyline
/// is created with `closed = true`, which ensures the closing segment between
/// the last and first points is included in all curve operations (division,
/// frames, shatter, contour, etc.).
fn create_polyline(points: &[[f64; 3]]) -> Result<Polyline3, ComponentError> {
    let closed = is_closed_polyline(points);
    let geom_points: Vec<GeomPoint3> = points.iter().map(|p| to_geom_point(*p)).collect();
    Polyline3::new(geom_points, closed)
        .map_err(|e| ComponentError::new(format!("Failed to create polyline: {}", e)))
}

/// Converts a CurveFrame to a Value::List representation.
fn frame_to_value(frame: &CurveFrame) -> Value {
    Value::List(vec![
        Value::Point(from_geom_point(frame.origin)),
        Value::Vector(from_geom_vec(frame.x_axis)),
        Value::Vector(from_geom_vec(frame.y_axis)),
        Value::Vector(from_geom_vec(frame.z_axis)),
    ])
}

/// Converts a CurveDivisionResult to component outputs.
fn division_result_to_outputs(result: CurveDivisionResult) -> BTreeMap<String, Value> {
    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_POINTS.to_owned(),
        Value::List(result.points.into_iter().map(|p| Value::Point(from_geom_point(p))).collect()),
    );
    outputs.insert(
        PIN_OUTPUT_TANGENTS.to_owned(),
        Value::List(result.tangents.into_iter().map(|t| Value::Vector(from_geom_vec(t))).collect()),
    );
    outputs.insert(
        PIN_OUTPUT_PARAMETERS.to_owned(),
        Value::List(result.parameters.into_iter().map(Value::Number).collect()),
    );
    outputs
}

/// Converts subcurves to Value::List output.
fn subcurves_to_value(subcurves: Vec<SubCurve>) -> Value {
    Value::List(
        subcurves
            .into_iter()
            .map(|sc| {
                Value::List(
                    sc.points
                        .into_iter()
                        .map(|p| Value::Point(from_geom_point(p)))
                        .collect(),
                )
            })
            .collect(),
    )
}

/// Default number of samples per segment for subcurve extraction.
const DEFAULT_SAMPLES_PER_SEGMENT: usize = 2;

/// Default number of samples for arc-length estimation.
const DEFAULT_ARC_LENGTH_SAMPLES: usize = 64;

fn evaluate_curve_frames(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Curve Frames vereist een curve en segmentaantal",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Curve Frames")?;
    let segments = coerce_positive_integer(inputs.get(1), "Curve Frames")?;

    // Build geom Polyline3 and use geom::curve frenet_frames
    let polyline = create_polyline(&points)?;
    let (geom_frames, params) = frenet_frames(&polyline, segments);

    let frames: Vec<Value> = geom_frames.iter().map(frame_to_value).collect();
    let parameters: Vec<Value> = params.into_iter().map(Value::Number).collect();

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_FRAMES.to_owned(), Value::List(frames));
    outputs.insert(PIN_OUTPUT_PARAMETERS.to_owned(), Value::List(parameters));
    Ok(outputs)
}

fn evaluate_divide_distance(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Divide Distance vereist een curve en afstand",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Divide Distance")?;
    let distance = coerce_positive_number(inputs.get(1), "Divide Distance")?;

    // Build geom Polyline3 and use geom::curve divide_curve_by_distance
    let polyline = create_polyline(&points)?;
    let result = divide_curve_by_distance(&polyline, distance, DEFAULT_ARC_LENGTH_SAMPLES);
    Ok(division_result_to_outputs(result))
}

fn evaluate_divide_length(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Divide Length vereist een curve en lengte",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Divide Length")?;
    let distance = coerce_positive_number(inputs.get(1), "Divide Length")?;

    // Build geom Polyline3 and use geom::curve divide_curve_by_distance
    let polyline = create_polyline(&points)?;
    let result = divide_curve_by_distance(&polyline, distance, DEFAULT_ARC_LENGTH_SAMPLES);
    Ok(division_result_to_outputs(result))
}

fn evaluate_divide_curve(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Divide Curve vereist een curve en segmentaantal",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Divide Curve")?;
    let segments = coerce_positive_integer(inputs.get(1), "Divide Curve")?;
    let include_kinks = inputs
        .get(2)
        .map(|value| coerce_boolean(value, "Divide Curve"))
        .transpose()?
        .unwrap_or(false);

    // Build geom Polyline3
    let polyline = create_polyline(&points)?;

    // Use geom::curve divide_curve_by_count for basic division
    let result = divide_curve_by_count(&polyline, segments);

    if !include_kinks {
        return Ok(division_result_to_outputs(result));
    }

    // If including kinks, add additional samples at polyline vertices
    let total_length = curve_arc_length(&polyline, DEFAULT_ARC_LENGTH_SAMPLES);
    if total_length < EPSILON {
        return Ok(division_result_to_outputs(result));
    }

    // Collect all parameters including kink positions
    let mut parameters: Vec<f64> = result.parameters.clone();

    // Add parameters at each polyline vertex (kinks)
    let mut accumulated_length = 0.0;
    for window in points.windows(2) {
        let seg_len = distance(window[0], window[1]);
        if accumulated_length > 0.0 {
            parameters.push(accumulated_length / total_length);
        }
        accumulated_length += seg_len;
    }
    parameters.push(1.0);

    // Sort and deduplicate parameters
    parameters.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    parameters.dedup_by(|a, b| (*a - *b).abs() < 1e-9);

    // Sample at all parameters using geom
    let mut points_out = Vec::new();
    let mut tangents = Vec::new();
    let mut params_out = Vec::new();

    for param in parameters {
        let sample = sample_curve_at(&polyline, param);
        points_out.push(Value::Point(from_geom_point(sample.point)));
        tangents.push(Value::Vector(from_geom_vec(sample.tangent)));
        params_out.push(Value::Number(sample.parameter));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::List(points_out));
    outputs.insert(PIN_OUTPUT_TANGENTS.to_owned(), Value::List(tangents));
    outputs.insert(PIN_OUTPUT_PARAMETERS.to_owned(), Value::List(params_out));
    Ok(outputs)
}

fn evaluate_divide_by_deviation(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Divide By Deviation vereist een curve en segmentaantal",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Divide By Deviation")?;
    let segments = coerce_positive_integer(inputs.get(1), "Divide By Deviation")?;

    // Build geom Polyline3 and use geom::curve divide_curve_by_count
    // Note: For polylines, deviation is always 0 since they're linear segments
    let polyline = create_polyline(&points)?;
    let result = divide_curve_by_count(&polyline, segments);

    // Convert to output format with deviation values (0.0 for polylines)
    let count = result.points.len();
    let deviations: Vec<Value> = vec![Value::Number(0.0); count];

    let mut outputs = division_result_to_outputs(result);
    outputs.insert(PIN_OUTPUT_DEVIATION.to_owned(), Value::List(deviations));
    Ok(outputs)
}

fn evaluate_perp_frames(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Perp Frames vereist een curve en segmentaantal",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Perp Frames")?;
    let segments = coerce_positive_integer(inputs.get(1), "Perp Frames")?;
    let align = inputs
        .get(2)
        .map(|value| coerce_boolean(value, "Perp Frames"))
        .transpose()?
        .unwrap_or(false);

    // Build geom Polyline3 and use geom::curve perp_frames
    let polyline = create_polyline(&points)?;
    let (geom_frames, params) = perp_frames(&polyline, segments, align);

    let frames: Vec<Value> = geom_frames.iter().map(frame_to_value).collect();
    let parameters: Vec<Value> = params.into_iter().map(Value::Number).collect();

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_FRAMES.to_owned(), Value::List(frames));
    outputs.insert(PIN_OUTPUT_PARAMETERS.to_owned(), Value::List(parameters));
    Ok(outputs)
}

fn evaluate_horizontal_frames(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Horizontal Frames vereist een curve en segmentaantal",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Horizontal Frames")?;
    let segments = coerce_positive_integer(inputs.get(1), "Horizontal Frames")?;

    // Build geom Polyline3 and use geom::curve horizontal_frames
    let polyline = create_polyline(&points)?;
    let (geom_frames, params) = horizontal_frames(&polyline, segments);

    let frames: Vec<Value> = geom_frames.iter().map(frame_to_value).collect();
    let parameters: Vec<Value> = params.into_iter().map(Value::Number).collect();

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_FRAMES.to_owned(), Value::List(frames));
    outputs.insert(PIN_OUTPUT_PARAMETERS.to_owned(), Value::List(parameters));
    Ok(outputs)
}

fn evaluate_shatter(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Shatter vereist een curve en parameterlijst",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Shatter")?;
    let parameter_value = inputs
        .get(1)
        .ok_or_else(|| ComponentError::new("Shatter vereist minstens één parameter"))?;
    let parameters = coerce_number_list(parameter_value, "Shatter")?;

    if parameters.is_empty() {
        return Err(ComponentError::new(
            "Shatter vereist minstens één parameter",
        ));
    }

    // Build geom Polyline3 and use geom::curve shatter_curve
    let polyline = create_polyline(&points)?;
    let subcurves = shatter_curve(&polyline, &parameters, DEFAULT_SAMPLES_PER_SEGMENT);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_SEGMENTS.to_owned(), subcurves_to_value(subcurves));
    Ok(outputs)
}

fn evaluate_contour_explicit(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 4 {
        return Err(ComponentError::new(
            "Contour (ex) vereist curve, vlak en offset/distance informatie",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Contour (ex)")?;
    let plane = coerce_plane(inputs.get(1), "Contour (ex)")?;
    let offsets = inputs
        .get(2)
        .map(|value| coerce_number_list(value, "Contour (ex)"))
        .transpose()?;
    let distances = inputs
        .get(3)
        .map(|value| coerce_number_list(value, "Contour (ex)"))
        .transpose()?;

    let offsets = determine_offsets(plane.origin, plane.normal, &points, offsets, distances)?;
    contour_with_offsets_geom(&points, &plane, &offsets, "Contour (ex)")
}

fn evaluate_contour(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 4 {
        return Err(ComponentError::new(
            "Contour vereist curve, startpunt, richting en afstand",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Contour")?;
    let start = coerce_point(inputs.get(1), "Contour")?;
    let direction = coerce_vector(inputs.get(2), "Contour")?;
    let distance = coerce_positive_number(inputs.get(3), "Contour")?;

    let normal = safe_normalized(direction)
        .map(|(v, _)| v)
        .ok_or_else(|| ComponentError::new("Contour vereist een niet-nul richting"))?;

    let plane = Plane {
        origin: start,
        normal,
    };
    let offsets = determine_offsets_from_distance(&plane, &points, distance);
    contour_with_offsets_geom(&points, &plane, &offsets, "Contour")
}

fn evaluate_dash_pattern(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Dash Pattern vereist een curve en patroon",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Dash Pattern")?;
    let pattern_value = inputs
        .get(1)
        .ok_or_else(|| ComponentError::new("Dash Pattern vereist een patroonlijst"))?;
    let pattern = coerce_number_list(pattern_value, "Dash Pattern")?;

    if pattern.is_empty() {
        return Err(ComponentError::new(
            "Dash Pattern vereist minstens één patroonwaarde",
        ));
    }

    // Validate all pattern elements
    if pattern.iter().any(|&v| v.abs() < EPSILON) {
        return Err(ComponentError::new(
            "Dash Pattern elementen moeten groter dan nul zijn",
        ));
    }

    // Build geom Polyline3
    let polyline = create_polyline(&points)?;
    let total_length = curve_arc_length(&polyline, DEFAULT_ARC_LENGTH_SAMPLES);
    if total_length < EPSILON {
        return Err(ComponentError::new("Dash Pattern curve heeft geen lengte"));
    }

    let mut dashes = Vec::new();
    let mut gaps = Vec::new();

    let mut pattern_index = 0;
    let mut toggle_dash = true;
    let mut remaining = pattern[pattern_index].abs();

    let mut cursor = 0.0;

    while cursor < total_length - 1e-9 {
        if remaining < EPSILON {
            pattern_index = (pattern_index + 1) % pattern.len();
            remaining = pattern[pattern_index].abs();
            toggle_dash = !toggle_dash;
            continue;
        }

        let step = remaining.min(total_length - cursor);
        let start_param = cursor / total_length;
        cursor += step;
        let end_param = cursor / total_length;

        // Use geom extract_subcurve
        let subcurve = extract_subcurve(&polyline, start_param, end_param, DEFAULT_SAMPLES_PER_SEGMENT);

        let points_value = Value::List(
            subcurve
                .points
                .into_iter()
                .map(|p| Value::Point(from_geom_point(p)))
                .collect(),
        );

        if toggle_dash {
            dashes.push(points_value);
        } else {
            gaps.push(points_value);
        }

        remaining -= step;
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_DASHES.to_owned(), Value::List(dashes));
    outputs.insert(PIN_OUTPUT_GAPS.to_owned(), Value::List(gaps));
    Ok(outputs)
}

/// Contour implementation using geom::curve utilities.
fn contour_with_offsets_geom(
    points: &[[f64; 3]],
    plane: &Plane,
    offsets: &[f64],
    context: &str,
) -> ComponentResult {
    if offsets.is_empty() {
        return Err(ComponentError::new(format!(
            "{} vereist minstens één offset",
            context
        )));
    }

    // Build geom Polyline3
    let polyline = create_polyline(points)?;

    let plane_normal = to_geom_vec(plane.normal);
    let plane_origin = to_geom_point(plane.origin);

    let mut contour_branches = Vec::new();
    let mut parameter_branches = Vec::new();

    for &offset in offsets {
        // Offset the plane origin by the offset distance along the normal
        let offset_origin = plane_origin.add_vec(plane_normal.mul_scalar(offset));

        // Use geom::curve curve_plane_intersections
        let intersections = curve_plane_intersections(
            &polyline,
            offset_origin,
            plane_normal,
            DEFAULT_ARC_LENGTH_SAMPLES,
        );

        if !intersections.is_empty() {
            let branch_points: Vec<Value> = intersections
                .iter()
                .map(|(pt, _)| Value::Point(from_geom_point(*pt)))
                .collect();
            let branch_parameters: Vec<Value> = intersections
                .iter()
                .map(|(_, t)| Value::Number(*t))
                .collect();

            contour_branches.push(Value::List(branch_points));
            parameter_branches.push(Value::List(branch_parameters));
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_CONTOURS.to_owned(),
        Value::List(contour_branches),
    );
    outputs.insert(
        PIN_OUTPUT_PARAMETERS.to_owned(),
        Value::List(parameter_branches),
    );
    Ok(outputs)
}

fn determine_offsets(
    origin: [f64; 3],
    normal: [f64; 3],
    points: &[[f64; 3]],
    offsets: Option<Vec<f64>>,
    distances: Option<Vec<f64>>,
) -> Result<Vec<f64>, ComponentError> {
    if let Some(offsets) = offsets {
        if offsets.is_empty() {
            return Err(ComponentError::new("Contour vereist minstens één offset"));
        }
        return Ok(offsets);
    }

    let Some(distances) = distances else {
        return Err(ComponentError::new("Contour vereist offsets of afstanden"));
    };

    if distances.is_empty() {
        return Err(ComponentError::new("Contour vereist minstens één afstand"));
    }

    let mut offset_values = Vec::new();
    let mut current = 0.0;
    for distance in distances {
        if distance.abs() < EPSILON {
            return Err(ComponentError::new("Afstanden moeten groter dan nul zijn"));
        }
        current += distance;
        offset_values.push(current);
    }

    if offset_values.is_empty() {
        return Err(ComponentError::new("Contour kon geen offsets bepalen"));
    }

    let base_offset = dot(
        subtract(points.first().copied().unwrap_or(origin), origin),
        normal,
    );
    Ok(offset_values
        .into_iter()
        .map(|value| value + base_offset)
        .collect())
}

fn determine_offsets_from_distance(plane: &Plane, points: &[[f64; 3]], distance: f64) -> Vec<f64> {
    let projections: Vec<f64> = points
        .iter()
        .map(|point| dot(subtract(*point, plane.origin), plane.normal))
        .collect();

    if projections.is_empty() {
        return Vec::new();
    }

    let min_proj = projections.iter().copied().fold(f64::INFINITY, f64::min);
    let max_proj = projections
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);

    let mut offsets = Vec::new();
    let mut current = plane.origin_projection();
    while current <= max_proj + EPSILON {
        offsets.push(current);
        current += distance;
    }

    if current - distance > plane.origin_projection() + EPSILON {
        let mut backward = plane.origin_projection() - distance;
        while backward >= min_proj - EPSILON {
            offsets.push(backward);
            backward -= distance;
        }
    }

    offsets.sort_by(|a, b| a.partial_cmp(b).unwrap());
    offsets
}

// --- Parsers en hulpfuncties ------------------------------------------------

fn coerce_polyline(value: Option<&Value>, context: &str) -> Result<Vec<[f64; 3]>, ComponentError> {
    let Some(value) = value else {
        return Err(ComponentError::new(format!(
            "{} vereist minimaal één curve",
            context
        )));
    };

    let mut points = Vec::new();
    collect_points(value, &mut points, context)?;
    if points.len() < 2 {
        return Err(ComponentError::new(format!(
            "{} vereist minstens twee punten",
            context
        )));
    }
    Ok(points)
}

fn collect_points(
    value: &Value,
    output: &mut Vec<[f64; 3]>,
    context: &str,
) -> Result<(), ComponentError> {
    match value {
        Value::Point(point) => {
            output.push(*point);
            Ok(())
        }
        Value::CurveLine { p1, p2 } => {
            output.push(*p1);
            output.push(*p2);
            Ok(())
        }
        Value::List(values) => {
            if values.is_empty() {
                return Err(ComponentError::new(format!(
                    "{} vereist minstens twee punten",
                    context
                )));
            }
            for entry in values {
                collect_points(entry, output, context)?;
            }
            Ok(())
        }
        other => Err(ComponentError::new(format!(
            "{} verwacht punten of lijnen, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_number(value: Option<&Value>, context: &str) -> Result<f64, ComponentError> {
    let Some(value) = value else {
        return Err(ComponentError::new(format!(
            "{} vereist een numerieke waarde",
            context
        )));
    };
    match value {
        Value::Number(number) => Ok(*number),
        Value::List(values) if values.len() == 1 => coerce_number(values.get(0), context),
        other => Err(ComponentError::new(format!(
            "{} verwacht een nummer, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_positive_number(value: Option<&Value>, context: &str) -> Result<f64, ComponentError> {
    let number = coerce_number(value, context)?;
    if number <= 0.0 {
        return Err(ComponentError::new(format!(
            "{} vereist een positief getal",
            context
        )));
    }
    Ok(number)
}

fn coerce_positive_integer(value: Option<&Value>, context: &str) -> Result<usize, ComponentError> {
    let number = coerce_number(value, context)?;
    let rounded = number.round();
    if rounded < 1.0 {
        return Err(ComponentError::new(format!(
            "{} vereist een positief aantal",
            context
        )));
    }
    Ok(rounded as usize)
}

fn coerce_point(value: Option<&Value>, context: &str) -> Result<[f64; 3], ComponentError> {
    let Some(value) = value else {
        return Err(ComponentError::new(format!("{} vereist een punt", context)));
    };
    match value {
        Value::Point(point) => Ok(*point),
        Value::List(values) if values.len() == 1 => coerce_point(values.get(0), context),
        other => Err(ComponentError::new(format!(
            "{} verwacht een punt, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_vector(value: Option<&Value>, context: &str) -> Result<[f64; 3], ComponentError> {
    let Some(value) = value else {
        return Err(ComponentError::new(format!(
            "{} vereist een vector",
            context
        )));
    };
    match value {
        Value::Vector(vector) => Ok(*vector),
        Value::List(values) if values.len() == 1 => coerce_vector(values.get(0), context),
        other => Err(ComponentError::new(format!(
            "{} verwacht een vector, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_boolean(value: &Value, context: &str) -> Result<bool, ComponentError> {
    match value {
        Value::Boolean(boolean) => Ok(*boolean),
        Value::Number(number) => Ok(*number != 0.0),
        Value::List(values) if values.len() == 1 => coerce_boolean(&values[0], context),
        other => Err(ComponentError::new(format!(
            "{} verwacht een boolean, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_number_list(value: &Value, context: &str) -> Result<Vec<f64>, ComponentError> {
    match value {
        Value::Number(number) => Ok(vec![*number]),
        Value::List(values) => {
            let mut numbers = Vec::with_capacity(values.len());
            for entry in values {
                numbers.push(coerce_number(Some(entry), context)?);
            }
            Ok(numbers)
        }
        other => Err(ComponentError::new(format!(
            "{} verwacht een lijst met getallen, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_plane(value: Option<&Value>, context: &str) -> Result<Plane, ComponentError> {
    let Some(value) = value else {
        return Err(ComponentError::new(format!("{} vereist een vlak", context)));
    };
    match value {
        Value::List(values) if values.len() >= 3 => {
            let origin = coerce_point(values.get(0), context)?;
            let x_axis_point = coerce_point(values.get(1), context)?;
            let y_axis_point = coerce_point(values.get(2), context)?;
            let x_axis = subtract(x_axis_point, origin);
            let y_axis = subtract(y_axis_point, origin);
            let normal = normalize(cross(x_axis, y_axis));
            Ok(Plane { origin, normal })
        }
        Value::Point(point) => Ok(Plane {
            origin: *point,
            normal: [0.0, 0.0, 1.0],
        }),
        other => Err(ComponentError::new(format!(
            "{} verwacht een vlak, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

#[derive(Debug, Clone, Copy)]
struct Plane {
    origin: [f64; 3],
    normal: [f64; 3],
}

impl Plane {
    fn origin_projection(&self) -> f64 {
        dot(self.origin, self.normal)
    }
}

// --- Math Helpers -----------------------------------------------------------

fn safe_normalized(vector: [f64; 3]) -> Option<([f64; 3], f64)> {
    let len = length(vector);
    if len < EPSILON {
        None
    } else {
        Some((scale(vector, 1.0 / len), len))
    }
}

fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn length(vector: [f64; 3]) -> f64 {
    (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt()
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

fn subtract(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: [f64; 3], factor: f64) -> [f64; 3] {
    [a[0] * factor, a[1] * factor, a[2] * factor]
}

fn normalize(vector: [f64; 3]) -> [f64; 3] {
    let len = length(vector);
    if len < EPSILON {
        [1.0, 0.0, 0.0]
    } else {
        scale(vector, 1.0 / len)
    }
}

const EPSILON: f64 = 1e-9;

