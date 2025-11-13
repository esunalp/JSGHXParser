//! Implementaties van Grasshopper "Curve → Division" componenten.

use std::collections::BTreeMap;

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

fn evaluate_curve_frames(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Curve Frames vereist een curve en segmentaantal",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Curve Frames")?;
    let segments = coerce_positive_integer(inputs.get(1), "Curve Frames")?;

    let mut frames = Vec::new();
    let mut parameters = Vec::new();

    let steps = segments.max(1);
    for i in 0..=steps {
        let parameter = i as f32 / steps as f32;
        let sample = sample_curve(&points, parameter);
        let derivative = approximate_derivative(&points, parameter, 1);
        let tangent = safe_normalized(derivative)
            .map(|(v, _)| v)
            .unwrap_or_else(|| sample.tangent.unwrap_or([1.0, 0.0, 0.0]));
        let frame = compute_frenet_frame(&points, parameter, sample.point, tangent);
        frames.push(frame_value(
            frame.origin,
            frame.x_axis,
            frame.y_axis,
            frame.z_axis,
        ));
        parameters.push(Value::Number(parameter));
    }

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

    divide_curve_by_length(&points, distance, "Divide Distance")
}

fn evaluate_divide_length(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Divide Length vereist een curve en lengte",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Divide Length")?;
    let distance = coerce_positive_number(inputs.get(1), "Divide Length")?;

    divide_curve_by_length(&points, distance, "Divide Length")
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

    let mut parameters: Vec<f32> = (0..=segments.max(1))
        .map(|index| index as f32 / segments.max(1) as f32)
        .collect();

    if include_kinks {
        let total_length = polyline_length(&points);
        if total_length > EPSILON {
            let mut length = 0.0;
            for segment in polyline_segments(&points) {
                parameters.push(length / total_length);
                length += segment.length;
            }
            parameters.push(1.0);
        }
    }

    parameters.sort_by(|a, b| a.partial_cmp(b).unwrap());
    parameters.dedup_by(|a, b| (*a - *b).abs() < 1e-9);

    let mut points_out = Vec::new();
    let mut tangents = Vec::new();
    let mut params_out = Vec::new();

    for parameter in parameters {
        let sample = sample_curve(&points, parameter);
        let derivative = approximate_derivative(&points, parameter, 1);
        let tangent = safe_normalized(derivative)
            .map(|(v, _)| v)
            .or(sample.tangent)
            .unwrap_or([1.0, 0.0, 0.0]);
        points_out.push(Value::Point(sample.point));
        tangents.push(Value::Vector(tangent));
        params_out.push(Value::Number(parameter));
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
    let steps = segments.max(1);

    let mut points_out = Vec::new();
    let mut tangents = Vec::new();
    let mut params_out = Vec::new();
    let mut deviations = Vec::new();

    for i in 0..=steps {
        let parameter = i as f32 / steps as f32;
        let sample = sample_curve(&points, parameter);
        let derivative = approximate_derivative(&points, parameter, 1);
        let tangent = safe_normalized(derivative)
            .map(|(v, _)| v)
            .or(sample.tangent)
            .unwrap_or([1.0, 0.0, 0.0]);
        points_out.push(Value::Point(sample.point));
        tangents.push(Value::Vector(tangent));
        params_out.push(Value::Number(parameter));
        deviations.push(Value::Number(0.0));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::List(points_out));
    outputs.insert(PIN_OUTPUT_TANGENTS.to_owned(), Value::List(tangents));
    outputs.insert(PIN_OUTPUT_PARAMETERS.to_owned(), Value::List(params_out));
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

    let steps = segments.max(1);
    let mut frames = Vec::new();
    let mut parameters = Vec::new();
    let mut previous_axes: Option<([f32; 3], [f32; 3])> = None;

    for i in 0..=steps {
        let parameter = i as f32 / steps as f32;
        let sample = sample_curve(&points, parameter);
        let derivative = approximate_derivative(&points, parameter, 1);
        let tangent = safe_normalized(derivative)
            .map(|(v, _)| v)
            .unwrap_or_else(|| sample.tangent.unwrap_or([1.0, 0.0, 0.0]));
        let mut frame = compute_parallel_frame(&points, sample.point, tangent);

        if align {
            if let Some((prev_y, prev_z)) = previous_axes {
                if dot(frame.y_axis, prev_y) < 0.0 {
                    frame.y_axis = scale(frame.y_axis, -1.0);
                }
                if dot(frame.z_axis, prev_z) < 0.0 {
                    frame.z_axis = scale(frame.z_axis, -1.0);
                }
            }
            previous_axes = Some((frame.y_axis, frame.z_axis));
        }

        frames.push(frame_value(
            frame.origin,
            frame.x_axis,
            frame.y_axis,
            frame.z_axis,
        ));
        parameters.push(Value::Number(parameter));
    }

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

    let steps = segments.max(1);
    let mut frames = Vec::new();
    let mut parameters = Vec::new();

    for i in 0..=steps {
        let parameter = i as f32 / steps as f32;
        let sample = sample_curve(&points, parameter);
        let derivative = approximate_derivative(&points, parameter, 1);
        let tangent = safe_normalized(derivative)
            .map(|(v, _)| v)
            .unwrap_or_else(|| sample.tangent.unwrap_or([1.0, 0.0, 0.0]));
        let frame = compute_horizontal_frame(sample.point, tangent);
        frames.push(frame_value(
            frame.origin,
            frame.x_axis,
            frame.y_axis,
            frame.z_axis,
        ));
        parameters.push(Value::Number(parameter));
    }

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

    let mut params = parameters
        .into_iter()
        .map(|value| clamp(value, 0.0, 1.0))
        .collect::<Vec<_>>();
    params.push(0.0);
    params.push(1.0);
    params.sort_by(|a, b| a.partial_cmp(b).unwrap());
    params.dedup_by(|a, b| (*a - *b).abs() < 1e-9);

    let mut segments_out = Vec::new();
    for pair in params.windows(2) {
        let start = pair[0];
        let end = pair[1];
        if (end - start).abs() < 1e-9 {
            continue;
        }
        let subcurve = sample_subcurve(&points, start, end);
        segments_out.push(Value::List(
            subcurve.into_iter().map(Value::Point).collect(),
        ));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_SEGMENTS.to_owned(), Value::List(segments_out));
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
    contour_with_offsets(&points, &plane, &offsets, "Contour (ex)")
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
    contour_with_offsets(&points, &plane, &offsets, "Contour")
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

    let total_length = polyline_length(&points);
    if total_length < EPSILON {
        return Err(ComponentError::new("Dash Pattern curve heeft geen lengte"));
    }

    let mut dashes = Vec::new();
    let mut gaps = Vec::new();

    let mut pattern_index = 0;
    let mut toggle_dash = true;
    let mut remaining = pattern[pattern_index].abs();
    if remaining < EPSILON {
        return Err(ComponentError::new(
            "Dash Pattern elementen moeten groter dan nul zijn",
        ));
    }

    let mut cursor = 0.0;
    let segments = polyline_segments(&points);
    let mut segment_index = 0;
    let mut segment_pos = 0.0;

    while cursor < total_length - 1e-9 {
        if remaining < EPSILON {
            pattern_index = (pattern_index + 1) % pattern.len();
            remaining = pattern[pattern_index].abs();
            if remaining < EPSILON {
                return Err(ComponentError::new(
                    "Dash Pattern elementen moeten groter dan nul zijn",
                ));
            }
            toggle_dash = !toggle_dash;
            continue;
        }

        if segment_index >= segments.len() {
            break;
        }
        let segment = &segments[segment_index];
        let segment_remaining = segment.length - segment_pos;
        let step = remaining.min(segment_remaining);

        let start_param = cursor / total_length;
        cursor += step;
        segment_pos += step;
        let end_param = cursor / total_length;

        let subcurve = sample_subcurve(&points, start_param, end_param);
        if toggle_dash {
            dashes.push(Value::List(
                subcurve.into_iter().map(Value::Point).collect(),
            ));
        } else {
            gaps.push(Value::List(
                subcurve.into_iter().map(Value::Point).collect(),
            ));
        }

        remaining -= step;
        if segment_pos >= segment.length - 1e-9 {
            segment_index += 1;
            segment_pos = 0.0;
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_DASHES.to_owned(), Value::List(dashes));
    outputs.insert(PIN_OUTPUT_GAPS.to_owned(), Value::List(gaps));
    Ok(outputs)
}

fn contour_with_offsets(
    points: &[[f32; 3]],
    plane: &Plane,
    offsets: &[f32],
    context: &str,
) -> ComponentResult {
    if offsets.is_empty() {
        return Err(ComponentError::new(format!(
            "{} vereist minstens één offset",
            context
        )));
    }

    let segments = polyline_segments(points);
    if segments.is_empty() {
        return Err(ComponentError::new(format!(
            "{} curve heeft te weinig punten",
            context
        )));
    }

    let total_length = segments.iter().map(|segment| segment.length).sum::<f32>();
    if total_length < EPSILON {
        return Err(ComponentError::new(format!(
            "{} curve heeft geen lengte",
            context
        )));
    }

    let mut contour_branches = Vec::new();
    let mut parameter_branches = Vec::new();

    for &offset in offsets {
        let mut branch_points = Vec::new();
        let mut branch_parameters = Vec::new();
        let mut accumulated = 0.0;

        for segment in &segments {
            let d1 = dot(subtract(segment.start, plane.origin), plane.normal) - offset;
            let d2 = dot(subtract(segment.end, plane.origin), plane.normal) - offset;

            if d1.abs() < EPSILON {
                branch_points.push(segment.start);
                branch_parameters.push(Value::Number(accumulated / total_length));
            }

            if d1.signum() == d2.signum() {
                accumulated += segment.length;
                continue;
            }

            let denom = d1 - d2;
            if denom.abs() < EPSILON {
                accumulated += segment.length;
                continue;
            }

            let factor = d1 / denom;
            let factor = factor.clamp(0.0, 1.0);
            let point = lerp(segment.start, segment.end, factor);
            let parameter = (accumulated + segment.length * factor) / total_length;
            branch_points.push(point);
            branch_parameters.push(Value::Number(parameter));
            accumulated += segment.length;
        }

        if !branch_points.is_empty() {
            contour_branches.push(Value::List(
                branch_points.into_iter().map(Value::Point).collect(),
            ));
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
    origin: [f32; 3],
    normal: [f32; 3],
    points: &[[f32; 3]],
    offsets: Option<Vec<f32>>,
    distances: Option<Vec<f32>>,
) -> Result<Vec<f32>, ComponentError> {
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

fn determine_offsets_from_distance(plane: &Plane, points: &[[f32; 3]], distance: f32) -> Vec<f32> {
    let projections: Vec<f32> = points
        .iter()
        .map(|point| dot(subtract(*point, plane.origin), plane.normal))
        .collect();

    if projections.is_empty() {
        return Vec::new();
    }

    let min_proj = projections.iter().copied().fold(f32::INFINITY, f32::min);
    let max_proj = projections
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);

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

fn divide_curve_by_length(points: &[[f32; 3]], distance: f32, context: &str) -> ComponentResult {
    let segments = polyline_segments(points);
    if segments.is_empty() {
        return Err(ComponentError::new(format!(
            "{} curve heeft te weinig punten",
            context
        )));
    }

    let total_length = segments.iter().map(|segment| segment.length).sum::<f32>();
    if total_length < EPSILON {
        return Err(ComponentError::new(format!(
            "{} curve heeft geen lengte",
            context
        )));
    }

    let mut points_out = Vec::new();
    let mut tangents = Vec::new();
    let mut parameters = Vec::new();

    let mut accumulated = 0.0;
    let mut next_target = 0.0;
    let mut segment_iter = segments.iter();
    let mut current_segment = segment_iter.next();
    let mut segment_progress = 0.0;

    while let Some(segment) = current_segment {
        if next_target > total_length + 1e-9 {
            break;
        }

        let remaining_in_segment = segment.length - segment_progress;
        if remaining_in_segment < EPSILON {
            accumulated += segment.length - segment_progress;
            current_segment = segment_iter.next();
            segment_progress = 0.0;
            continue;
        }

        if next_target <= accumulated + remaining_in_segment + 1e-9 {
            let distance_into_segment = next_target - accumulated;
            let factor = if segment.length < EPSILON {
                0.0
            } else {
                (segment_progress + distance_into_segment) / segment.length
            };
            let point = lerp(segment.start, segment.end, factor);
            let parameter = next_target / total_length;
            let derivative = approximate_derivative(points, parameter, 1);
            let tangent = safe_normalized(derivative)
                .map(|(v, _)| v)
                .unwrap_or_else(|| {
                    safe_normalized(subtract(segment.end, segment.start))
                        .map(|(v, _)| v)
                        .unwrap_or([1.0, 0.0, 0.0])
                });

            points_out.push(Value::Point(point));
            tangents.push(Value::Vector(tangent));
            parameters.push(Value::Number(parameter));

            if next_target >= total_length - distance * 0.25 {
                break;
            }

            next_target += distance;
        } else {
            accumulated += remaining_in_segment;
            current_segment = segment_iter.next();
            segment_progress = 0.0;
        }
    }

    if points_out.is_empty()
        || parameters.last().map(|value| match value {
            Value::Number(number) => *number,
            _ => 0.0,
        }) != Some(1.0)
    {
        let last_point = *points.last().unwrap();
        points_out.push(Value::Point(last_point));
        tangents.push(Value::Vector(
            safe_normalized(subtract(
                *points.last().unwrap(),
                *points
                    .get(points.len().saturating_sub(2))
                    .unwrap_or(&last_point),
            ))
            .map(|(v, _)| v)
            .unwrap_or([1.0, 0.0, 0.0]),
        ));
        parameters.push(Value::Number(1.0));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::List(points_out));
    outputs.insert(PIN_OUTPUT_TANGENTS.to_owned(), Value::List(tangents));
    outputs.insert(PIN_OUTPUT_PARAMETERS.to_owned(), Value::List(parameters));
    Ok(outputs)
}

fn sample_subcurve(points: &[[f32; 3]], start: f32, end: f32) -> Vec<[f32; 3]> {
    if end <= start {
        let point = sample_curve(points, start).point;
        return vec![point];
    }

    let start_sample = sample_curve(points, start).point;
    let end_sample = sample_curve(points, end).point;
    vec![start_sample, end_sample]
}

// --- Parsers en hulpfuncties ------------------------------------------------

fn coerce_polyline(value: Option<&Value>, context: &str) -> Result<Vec<[f32; 3]>, ComponentError> {
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
    output: &mut Vec<[f32; 3]>,
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

fn coerce_number(value: Option<&Value>, context: &str) -> Result<f32, ComponentError> {
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

fn coerce_positive_number(value: Option<&Value>, context: &str) -> Result<f32, ComponentError> {
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

fn coerce_point(value: Option<&Value>, context: &str) -> Result<[f32; 3], ComponentError> {
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

fn coerce_vector(value: Option<&Value>, context: &str) -> Result<[f32; 3], ComponentError> {
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

fn coerce_number_list(value: &Value, context: &str) -> Result<Vec<f32>, ComponentError> {
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
    origin: [f32; 3],
    normal: [f32; 3],
}

impl Plane {
    fn origin_projection(&self) -> f32 {
        dot(self.origin, self.normal)
    }
}

#[derive(Debug, Clone, Copy)]
struct CurveSample {
    point: [f32; 3],
    tangent: Option<[f32; 3]>,
}

fn sample_curve(points: &[[f32; 3]], parameter: f32) -> CurveSample {
    let (point, tangent, _) = sample_curve_basic(points, clamp(parameter, 0.0, 1.0));
    CurveSample { point, tangent }
}

fn sample_curve_basic(points: &[[f32; 3]], parameter: f32) -> ([f32; 3], Option<[f32; 3]>, f32) {
    let segments = polyline_segments(points);
    if segments.is_empty() {
        return (points.get(0).copied().unwrap_or([0.0, 0.0, 0.0]), None, 0.0);
    }

    let total_length: f32 = segments.iter().map(|segment| segment.length).sum();
    if total_length < EPSILON {
        let tangent = safe_normalized(subtract(segments[0].end, segments[0].start)).map(|(v, _)| v);
        return (segments[0].start, tangent, 0.0);
    }

    let target = parameter * total_length;
    let mut accumulated = 0.0;
    for segment in &segments {
        if accumulated + segment.length >= target {
            let remaining = target - accumulated;
            let factor = if segment.length < EPSILON {
                0.0
            } else {
                remaining / segment.length
            };
            let point = lerp(segment.start, segment.end, factor);
            let tangent = safe_normalized(subtract(segment.end, segment.start)).map(|(v, _)| v);
            return (point, tangent, target);
        }
        accumulated += segment.length;
    }

    let last = segments.last().unwrap();
    let tangent = safe_normalized(subtract(last.end, last.start)).map(|(v, _)| v);
    (last.end, tangent, total_length)
}

fn approximate_derivative(points: &[[f32; 3]], parameter: f32, order: usize) -> [f32; 3] {
    let h = 1.0 / (points.len().max(8) as f32 * 4.0);
    match order {
        1 => {
            let forward = sample_curve(points, clamp(parameter + h, 0.0, 1.0)).point;
            let backward = sample_curve(points, clamp(parameter - h, 0.0, 1.0)).point;
            scale(subtract(forward, backward), 0.5 / h)
        }
        2 => {
            let forward = sample_curve(points, clamp(parameter + h, 0.0, 1.0)).point;
            let backward = sample_curve(points, clamp(parameter - h, 0.0, 1.0)).point;
            let center = sample_curve(points, parameter).point;
            add(
                scale(add(forward, backward), 1.0 / (h * h)),
                scale(center, -2.0 / (h * h)),
            )
        }
        _ => {
            let forward = sample_curve(points, clamp(parameter + 2.0 * h, 0.0, 1.0)).point;
            let forward_mid = sample_curve(points, clamp(parameter + h, 0.0, 1.0)).point;
            let backward_mid = sample_curve(points, clamp(parameter - h, 0.0, 1.0)).point;
            let backward = sample_curve(points, clamp(parameter - 2.0 * h, 0.0, 1.0)).point;
            add(
                scale(
                    add(forward, scale(forward_mid, -3.0)),
                    1.0 / (2.0 * h * h * h),
                ),
                scale(
                    add(scale(backward_mid, 3.0), scale(backward, -1.0)),
                    1.0 / (2.0 * h * h * h),
                ),
            )
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PolylineSegment {
    start: [f32; 3],
    end: [f32; 3],
    length: f32,
}

fn polyline_segments(points: &[[f32; 3]]) -> Vec<PolylineSegment> {
    points
        .windows(2)
        .map(|pair| PolylineSegment {
            start: pair[0],
            end: pair[1],
            length: distance(pair[0], pair[1]),
        })
        .collect()
}

fn polyline_length(points: &[[f32; 3]]) -> f32 {
    polyline_segments(points)
        .iter()
        .map(|segment| segment.length)
        .sum()
}

#[derive(Debug, Clone, Copy)]
struct FrameData {
    origin: [f32; 3],
    x_axis: [f32; 3],
    y_axis: [f32; 3],
    z_axis: [f32; 3],
}

fn compute_frenet_frame(
    points: &[[f32; 3]],
    parameter: f32,
    origin: [f32; 3],
    tangent: [f32; 3],
) -> FrameData {
    let second = approximate_derivative(points, parameter, 2);
    let mut normal = subtract(second, scale(tangent, dot(second, tangent)));
    if length_squared(normal) < EPSILON {
        normal = orthogonal_vector(tangent);
    }
    let normal = normalize(normal);
    let binormal = normalize(cross(tangent, normal));
    let normal = normalize(cross(binormal, tangent));
    FrameData {
        origin,
        x_axis: tangent,
        y_axis: normal,
        z_axis: binormal,
    }
}

fn compute_parallel_frame(points: &[[f32; 3]], origin: [f32; 3], tangent: [f32; 3]) -> FrameData {
    let plane = plane_from_polyline(points);
    let mut binormal = plane.normal;
    if length_squared(binormal) < EPSILON || length_squared(cross(binormal, tangent)) < EPSILON {
        binormal = [0.0, 0.0, 1.0];
    }
    if length_squared(cross(binormal, tangent)) < EPSILON {
        binormal = [1.0, 0.0, 0.0];
    }
    let normal = normalize(cross(binormal, tangent));
    let binormal = normalize(cross(tangent, normal));
    FrameData {
        origin,
        x_axis: tangent,
        y_axis: normal,
        z_axis: binormal,
    }
}

fn compute_horizontal_frame(origin: [f32; 3], tangent: [f32; 3]) -> FrameData {
    let mut binormal = [0.0, 0.0, 1.0];
    if length_squared(cross(binormal, tangent)) < EPSILON {
        binormal = [1.0, 0.0, 0.0];
    }
    let normal = normalize(cross(binormal, tangent));
    let binormal = normalize(cross(tangent, normal));
    FrameData {
        origin,
        x_axis: tangent,
        y_axis: normal,
        z_axis: binormal,
    }
}

fn plane_from_polyline(points: &[[f32; 3]]) -> Plane {
    if points.len() < 3 {
        return Plane {
            origin: points.first().copied().unwrap_or([0.0, 0.0, 0.0]),
            normal: [0.0, 0.0, 1.0],
        };
    }

    let a = points[0];
    let b = points[1];
    let c = points[2];
    let normal = normalize(cross(subtract(b, a), subtract(c, a)));
    Plane { origin: a, normal }
}

fn frame_value(origin: [f32; 3], x_axis: [f32; 3], y_axis: [f32; 3], z_axis: [f32; 3]) -> Value {
    Value::List(vec![
        Value::Point(origin),
        Value::Vector(x_axis),
        Value::Vector(y_axis),
        Value::Vector(z_axis),
    ])
}

fn safe_normalized(vector: [f32; 3]) -> Option<([f32; 3], f32)> {
    let length = length(vector);
    if length < EPSILON {
        None
    } else {
        Some((scale(vector, 1.0 / length), length))
    }
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn length(vector: [f32; 3]) -> f32 {
    (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt()
}

fn length_squared(vector: [f32; 3]) -> f32 {
    vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn subtract(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: [f32; 3], factor: f32) -> [f32; 3] {
    [a[0] * factor, a[1] * factor, a[2] * factor]
}

fn normalize(vector: [f32; 3]) -> [f32; 3] {
    safe_normalized(vector)
        .map(|(v, _)| v)
        .unwrap_or([1.0, 0.0, 0.0])
}

fn orthogonal_vector(vector: [f32; 3]) -> [f32; 3] {
    if vector[0].abs() < vector[1].abs() && vector[0].abs() < vector[2].abs() {
        normalize(cross(vector, [1.0, 0.0, 0.0]))
    } else if vector[1].abs() < vector[2].abs() {
        normalize(cross(vector, [0.0, 1.0, 0.0]))
    } else {
        normalize(cross(vector, [0.0, 0.0, 1.0]))
    }
}

fn lerp(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

const EPSILON: f32 = 1e-9;
