//! Implementaties van Grasshopper "Curve → Util" componenten.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

const PIN_OUTPUT_CURVES: &str = "C";
const PIN_OUTPUT_FLAG: &str = "F";
const PIN_OUTPUT_POLYLINE: &str = "P";
const PIN_OUTPUT_SEGMENTS: &str = "S";
const PIN_OUTPUT_REDUCTION: &str = "R";
const PIN_OUTPUT_SIMPLIFIED: &str = "S";
const PIN_OUTPUT_VERTICES: &str = "V";
const PIN_OUTPUT_COUNT: &str = "N";
const PIN_OUTPUT_OFFSET: &str = "O";
const PIN_OUTPUT_VALID: &str = "V";
const PIN_OUTPUT_FRAMES: &str = "F";
const PIN_OUTPUT_PARAMETERS: &str = "t";
const PIN_OUTPUT_POINTS: &str = "P";
const PIN_OUTPUT_TANGENTS: &str = "T";
const PIN_OUTPUT_PARAMETER: &str = "t";

/// Beschikbare componenten binnen deze module.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    OffsetCurve,
    OffsetCurveLoose,
    OffsetLoose3d,
    OffsetPolyline,
    FlipCurve,
    CurveToPolyline,
    SmoothPolyline,
    JoinCurves,
    Reduce,
    SimplifyCurve,
    FitCurve,
    RebuildCurve,
    Explode,
    PolylineCollapse,
    FilletRadius,
    Seam,
    ExtendCurve,
    PullCurve,
    PerpFramesObsolete,
    FilletDistance,
    DivideCurveObsolete,
    OffsetOnSurface,
    FilletParameter,
    ProjectCurve,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Volledige lijst met componentregistraties voor de curve-util componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{1a38d325-98de-455c-93f1-bca431bc1243}"],
        names: &["Offset Curve", "Offset"],
        kind: ComponentKind::OffsetCurve,
    },
    Registration {
        guids: &["{80e55fc2-933b-4bfb-a353-12358786dba8}"],
        names: &["Offset Curve Loose", "Offset Loose"],
        kind: ComponentKind::OffsetCurveLoose,
    },
    Registration {
        guids: &["{c6fe61e7-25e2-4333-9172-f4e2a123fcfe}"],
        names: &["Offset Loose 3D", "Offset 3D"],
        kind: ComponentKind::OffsetLoose3d,
    },
    Registration {
        guids: &["{e2c6cab3-91ea-4c01-900c-646642d3e436}"],
        names: &["Offset Polyline", "PolyOffset"],
        kind: ComponentKind::OffsetPolyline,
    },
    Registration {
        guids: &["{22990b1f-9be6-477c-ad89-f775cd347105}"],
        names: &["Flip Curve", "Flip"],
        kind: ComponentKind::FlipCurve,
    },
    Registration {
        guids: &["{2956d989-3599-476f-bc92-1d847aff98b6}"],
        names: &["Curve To Polyline", "ToPoly"],
        kind: ComponentKind::CurveToPolyline,
    },
    Registration {
        guids: &["{5c5fbc42-3e1d-4081-9cf1-148d0b1d9610}"],
        names: &["Smooth Polyline", "Smooth"],
        kind: ComponentKind::SmoothPolyline,
    },
    Registration {
        guids: &["{8073a420-6bec-49e3-9b18-367f6fd76ac3}"],
        names: &["Join Curves", "Join"],
        kind: ComponentKind::JoinCurves,
    },
    Registration {
        guids: &["{884646c3-0e70-4ad1-90c5-42601ee26450}"],
        names: &["Reduce", "Poly Reduce"],
        kind: ComponentKind::Reduce,
    },
    Registration {
        guids: &["{922dc7e5-0f0e-4c21-ae4b-f6a8654e63f6}"],
        names: &["Simplify Curve", "Simplify"],
        kind: ComponentKind::SimplifyCurve,
    },
    Registration {
        guids: &["{a3f9f19e-3e6c-4ac7-97c3-946de32c3e8e}"],
        names: &["Fit Curve", "Fit"],
        kind: ComponentKind::FitCurve,
    },
    Registration {
        guids: &["{9333c5b3-11f9-423c-bbb5-7e5156430219}"],
        names: &["Rebuild Curve", "Rebuild"],
        kind: ComponentKind::RebuildCurve,
    },
    Registration {
        guids: &["{afb96615-c59a-45c9-9cac-e27acb1c7ca0}"],
        names: &["Explode", "Explode"],
        kind: ComponentKind::Explode,
    },
    Registration {
        guids: &["{be298882-28c9-45b1-980d-7192a531c9a9}"],
        names: &["Polyline Collapse", "Collapse"],
        kind: ComponentKind::PolylineCollapse,
    },
    Registration {
        guids: &["{2f407944-81c3-4062-a485-276454ec4b8c}"],
        names: &["Fillet", "Fillet (Radius)", "Fillet Radius"],
        kind: ComponentKind::FilletRadius,
    },
    Registration {
        guids: &["{42ad8dc1-b0c0-40df-91f5-2c46e589e6c2}"],
        names: &["Seam", "Adjust Seam"],
        kind: ComponentKind::Seam,
    },
    Registration {
        guids: &["{62cc9684-6a39-422e-aefa-ed44643557b9}"],
        names: &["Extend Curve", "Extend"],
        kind: ComponentKind::ExtendCurve,
    },
    Registration {
        guids: &["{6b5812f5-bb36-4d74-97fc-5a1f2f77452d}"],
        names: &["Pull Curve", "Pull"],
        kind: ComponentKind::PullCurve,
    },
    Registration {
        guids: &["{6da4b70c-ce98-4d52-a2bb-2fadccf39da0}"],
        names: &["Perp Frames [OBSOLETE]", "Perp Frames Legacy"],
        kind: ComponentKind::PerpFramesObsolete,
    },
    Registration {
        guids: &["{6fb21315-a032-400e-a80f-248687f5507f}"],
        names: &["Fillet Distance", "Fillet Dist"],
        kind: ComponentKind::FilletDistance,
    },
    Registration {
        guids: &["{93b1066f-060e-440d-a638-aae8cbe7acb7}"],
        names: &["Divide Curve [OBSOLETE]", "Divide Legacy"],
        kind: ComponentKind::DivideCurveObsolete,
    },
    Registration {
        guids: &["{b6f5cb51-f260-4c74-bf73-deb47de1bf91}"],
        names: &["Offset on Surface", "Offset On Srf"],
        kind: ComponentKind::OffsetOnSurface,
    },
    Registration {
        guids: &["{c92cdfc8-3df8-4c4e-abc1-ede092a0aa8a}"],
        names: &["Fillet Parameter", "Fillet (Parameter)"],
        kind: ComponentKind::FilletParameter,
    },
    Registration {
        guids: &["{d7ee52ff-89b8-4d1a-8662-3e0dd391d0af}"],
        names: &["Project Curve", "Project"],
        kind: ComponentKind::ProjectCurve,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::OffsetCurve => evaluate_offset_curve(inputs, true),
            Self::OffsetCurveLoose => evaluate_offset_curve(inputs, false),
            Self::OffsetLoose3d => evaluate_offset_curve(inputs, false),
            Self::OffsetPolyline => evaluate_offset_polyline(inputs),
            Self::FlipCurve => evaluate_flip_curve(inputs),
            Self::CurveToPolyline => evaluate_curve_to_polyline(inputs),
            Self::SmoothPolyline => evaluate_smooth_polyline(inputs),
            Self::JoinCurves => evaluate_join_curves(inputs),
            Self::Reduce => evaluate_reduce(inputs),
            Self::SimplifyCurve => evaluate_simplify_curve(inputs),
            Self::FitCurve => evaluate_fit_curve(inputs),
            Self::RebuildCurve => evaluate_rebuild_curve(inputs),
            Self::Explode => evaluate_explode(inputs),
            Self::PolylineCollapse => evaluate_polyline_collapse(inputs),
            Self::FilletRadius => evaluate_fillet_radius(inputs),
            Self::Seam => evaluate_seam(inputs),
            Self::ExtendCurve => evaluate_extend_curve(inputs),
            Self::PullCurve => evaluate_pull_curve(inputs),
            Self::PerpFramesObsolete => evaluate_perp_frames_obsolete(inputs),
            Self::FilletDistance => evaluate_fillet_distance(inputs),
            Self::DivideCurveObsolete => evaluate_divide_curve_obsolete(inputs),
            Self::OffsetOnSurface => evaluate_offset_on_surface(inputs),
            Self::FilletParameter => evaluate_fillet_parameter(inputs),
            Self::ProjectCurve => evaluate_project_curve(inputs),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::OffsetCurve => "Offset Curve",
            Self::OffsetCurveLoose => "Offset Curve Loose",
            Self::OffsetLoose3d => "Offset Loose 3D",
            Self::OffsetPolyline => "Offset Polyline",
            Self::FlipCurve => "Flip Curve",
            Self::CurveToPolyline => "Curve To Polyline",
            Self::SmoothPolyline => "Smooth Polyline",
            Self::JoinCurves => "Join Curves",
            Self::Reduce => "Reduce",
            Self::SimplifyCurve => "Simplify Curve",
            Self::FitCurve => "Fit Curve",
            Self::RebuildCurve => "Rebuild Curve",
            Self::Explode => "Explode",
            Self::PolylineCollapse => "Polyline Collapse",
            Self::FilletRadius => "Fillet",
            Self::Seam => "Seam",
            Self::ExtendCurve => "Extend Curve",
            Self::PullCurve => "Pull Curve",
            Self::PerpFramesObsolete => "Perp Frames [OBSOLETE]",
            Self::FilletDistance => "Fillet Distance",
            Self::DivideCurveObsolete => "Divide Curve [OBSOLETE]",
            Self::OffsetOnSurface => "Offset on Surface",
            Self::FilletParameter => "Fillet (Parameter)",
            Self::ProjectCurve => "Project Curve",
        }
    }
}

fn evaluate_offset_curve(inputs: &[Value], closed_hint: bool) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Offset Curve vereist minimaal een curve",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Offset Curve")?;
    let distance = coerce_number(inputs.get(1), "Offset Curve").unwrap_or(0.0);
    let plane = inputs
        .get(2)
        .and_then(|value| coerce_plane(Some(value), "Offset Curve").ok())
        .unwrap_or_else(|| plane_from_polyline(&points));
    let closed = if closed_hint {
        is_closed_polyline(&points)
    } else {
        false
    };
    let offset = offset_polyline_points(&points, &plane, distance, closed);

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_CURVES.to_owned(),
        Value::List(vec![polyline_to_value(offset)]),
    );
    Ok(outputs)
}

fn evaluate_offset_polyline(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Offset Polyline vereist minimaal een polyline",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Offset Polyline")?;
    let distance = coerce_number(inputs.get(1), "Offset Polyline").unwrap_or(0.0);
    let plane = plane_from_polyline(&points);
    let closed = is_closed_polyline(&points);
    let offset = offset_polyline_points(&points, &plane, distance, closed);

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_OFFSET.to_owned(),
        Value::List(vec![polyline_to_value(offset)]),
    );
    outputs.insert(
        PIN_OUTPUT_VALID.to_owned(),
        Value::List(vec![Value::Boolean(true)]),
    );
    Ok(outputs)
}

fn evaluate_flip_curve(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Flip Curve vereist minimaal een curve"));
    }

    let points = coerce_polyline(inputs.get(0), "Flip Curve")?;
    let closed = is_closed_polyline(&points);
    let mut should_flip = true;

    if !closed {
        if let Some(guide_value) = inputs.get(1) {
            if !is_effectively_empty_value(guide_value)
                && let Ok(guide_points) = coerce_polyline(Some(guide_value), "Flip Curve Guide")
            {
                if guide_points.len() >= 2 {
                    let start = points.first().copied().unwrap();
                    let guide_start = guide_points.first().copied().unwrap();
                    let guide_end = guide_points.last().copied().unwrap();
                    let start_to_start = distance(start, guide_start);
                    let start_to_end = distance(start, guide_end);
                    should_flip = start_to_end < start_to_start;
                }
            }
        }
    }

    let final_points = if should_flip {
        let mut reversed = points.clone();
        reversed.reverse();
        reversed
    } else {
        points
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_CURVES.to_owned(),
        polyline_to_value(final_points),
    );
    outputs.insert(PIN_OUTPUT_FLAG.to_owned(), Value::Boolean(should_flip));
    Ok(outputs)
}

fn evaluate_curve_to_polyline(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Curve To Polyline vereist minimaal een curve",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Curve To Polyline")?;
    let distance_tolerance = coerce_number(inputs.get(1), "Curve To Polyline Td")
        .unwrap_or(0.01)
        .max(0.0001);
    let angle_tolerance = coerce_number(inputs.get(2), "Curve To Polyline Ta").unwrap_or(0.0);
    let min_edge = coerce_number(inputs.get(3), "Curve To Polyline MinEdge").unwrap_or(0.0);
    let max_edge = coerce_number(inputs.get(4), "Curve To Polyline MaxEdge")
        .unwrap_or(f64::INFINITY)
        .max(min_edge.max(EPSILON));

    let simplified = rdp_simplify(&points, distance_tolerance, angle_tolerance.max(0.0));
    let remeshed = remesh_polyline(&simplified, min_edge, max_edge);
    let segments = if remeshed.len() > 1 {
        remeshed.len() - 1
    } else {
        0
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POLYLINE.to_owned(), polyline_to_value(remeshed));
    outputs.insert(
        PIN_OUTPUT_SEGMENTS.to_owned(),
        Value::Number(segments as f64),
    );
    Ok(outputs)
}

fn evaluate_smooth_polyline(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Smooth Polyline vereist minimaal een polyline",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Smooth Polyline")?;
    let strength = coerce_number(inputs.get(1), "Smooth Polyline Strength")
        .unwrap_or(0.5)
        .clamp(0.0, 1.0);
    let times = coerce_number(inputs.get(2), "Smooth Polyline Times")
        .unwrap_or(1.0)
        .max(0.0)
        .round() as usize;

    let smoothed = smooth_polyline(&points, strength, times);
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POLYLINE.to_owned(), polyline_to_value(smoothed));
    Ok(outputs)
}

fn evaluate_join_curves(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Join Curves vereist minimaal een lijst met curves",
        ));
    }

    let polylines = coerce_polyline_collection(inputs.get(0), "Join Curves")?;
    let preserve_direction = inputs
        .get(1)
        .and_then(|value| coerce_boolean(value, "Join Curves").ok())
        .unwrap_or(false);

    let joined = join_polylines(polylines, preserve_direction);
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CURVES.to_owned(), polylines_to_value(joined));
    Ok(outputs)
}

fn evaluate_reduce(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Reduce vereist minimaal een polyline"));
    }

    let points = coerce_polyline(inputs.get(0), "Reduce")?;
    let tolerance = coerce_number(inputs.get(1), "Reduce Tolerance")
        .unwrap_or(0.01)
        .max(0.0);

    let reduced = rdp_simplify(&points, tolerance, 0.0);
    let removed = points.len().saturating_sub(reduced.len());

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POLYLINE.to_owned(), polyline_to_value(reduced));
    outputs.insert(
        PIN_OUTPUT_REDUCTION.to_owned(),
        Value::Number(removed as f64),
    );
    Ok(outputs)
}

fn evaluate_simplify_curve(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Simplify Curve vereist minimaal een curve",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Simplify Curve")?;
    let tolerance = coerce_number(inputs.get(1), "Simplify Curve Tolerance")
        .unwrap_or(0.01)
        .max(0.0);
    let angle_tolerance = coerce_number(inputs.get(2), "Simplify Curve Angle")
        .unwrap_or(0.0)
        .max(0.0);

    let simplified = rdp_simplify(&points, tolerance, angle_tolerance);
    let changed = simplified.len() != points.len();

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CURVES.to_owned(), polyline_to_value(simplified));
    outputs.insert(PIN_OUTPUT_SIMPLIFIED.to_owned(), Value::Boolean(changed));
    Ok(outputs)
}

fn evaluate_fit_curve(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Fit Curve vereist minimaal een curve"));
    }

    let points = coerce_polyline(inputs.get(0), "Fit Curve")?;
    let degree = coerce_number(inputs.get(1), "Fit Curve Degree")
        .unwrap_or(3.0)
        .max(1.0);
    let tolerance = coerce_number(inputs.get(2), "Fit Curve Tolerance")
        .unwrap_or(0.01)
        .max(0.0);

    let mut simplified = rdp_simplify(&points, tolerance, 0.0);
    let min_required = degree.round() as usize + 1;
    if simplified.len() < min_required && points.len() >= min_required {
        simplified = points.clone();
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CURVES.to_owned(), polyline_to_value(simplified));
    Ok(outputs)
}

fn evaluate_rebuild_curve(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Rebuild Curve vereist minimaal een curve",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Rebuild Curve")?;
    let count = coerce_number(inputs.get(2), "Rebuild Curve Count")
        .unwrap_or(points.len() as f64)
        .max(2.0)
        .round() as usize;

    let rebuilt = resample_polyline(&points, count);
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CURVES.to_owned(), polyline_to_value(rebuilt));
    Ok(outputs)
}

fn evaluate_explode(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Explode vereist minimaal een curve"));
    }

    let points = coerce_polyline(inputs.get(0), "Explode")?;
    let segments = points
        .windows(2)
        .map(|pair| Value::List(vec![Value::Point(pair[0]), Value::Point(pair[1])]))
        .collect::<Vec<_>>();

    let mut vertices = Vec::new();
    for point in points {
        vertices.push(Value::Point(point));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_SEGMENTS.to_owned(), Value::List(segments));
    outputs.insert(PIN_OUTPUT_VERTICES.to_owned(), Value::List(vertices));
    Ok(outputs)
}

fn evaluate_polyline_collapse(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Polyline Collapse vereist minimaal een polyline",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Polyline Collapse")?;
    let tolerance = coerce_number(inputs.get(1), "Polyline Collapse Tolerance")
        .unwrap_or(0.01)
        .max(0.0);

    let (collapsed, removed) = collapse_polyline(&points, tolerance);
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POLYLINE.to_owned(), polyline_to_value(collapsed));
    outputs.insert(PIN_OUTPUT_COUNT.to_owned(), Value::Number(removed as f64));
    Ok(outputs)
}

fn evaluate_fillet_radius(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Fillet vereist minimaal een curve"));
    }

    let points = coerce_polyline(inputs.get(0), "Fillet")?;
    let radius = coerce_number(inputs.get(1), "Fillet Radius")
        .unwrap_or(0.0)
        .max(0.0);
    let filleted = fillet_polyline(&points, radius);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CURVES.to_owned(), polyline_to_value(filleted));
    Ok(outputs)
}

fn evaluate_seam(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Seam vereist minimaal een curve"));
    }

    let points = coerce_polyline(inputs.get(0), "Seam")?;
    if !is_closed_polyline(&points) {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_CURVES.to_owned(), polyline_to_value(points));
        return Ok(outputs);
    }

    let parameter = coerce_number(inputs.get(1), "Seam Parameter").unwrap_or(0.0);
    let adjusted = rotate_polyline_seam(&points, parameter);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CURVES.to_owned(), polyline_to_value(adjusted));
    Ok(outputs)
}

fn evaluate_extend_curve(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Extend Curve vereist minimaal een curve",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Extend Curve")?;
    let start = coerce_number(inputs.get(2), "Extend Curve Start").unwrap_or(0.0);
    let end = coerce_number(inputs.get(3), "Extend Curve End").unwrap_or(0.0);
    let extended = extend_polyline(&points, start, end);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CURVES.to_owned(), polyline_to_value(extended));
    Ok(outputs)
}

fn evaluate_pull_curve(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Pull Curve vereist minimaal een curve"));
    }

    let points = coerce_polyline(inputs.get(0), "Pull Curve")?;

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CURVES.to_owned(), polyline_to_value(points));
    Ok(outputs)
}

fn evaluate_perp_frames_obsolete(inputs: &[Value]) -> ComponentResult {
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

    let (frames, parameters) = compute_perp_frames(&points, segments.max(1), align);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_FRAMES.to_owned(), Value::List(frames));
    outputs.insert(PIN_OUTPUT_PARAMETERS.to_owned(), Value::List(parameters));
    Ok(outputs)
}

fn evaluate_fillet_distance(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Fillet Distance vereist minimaal een curve",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Fillet Distance")?;
    let distance = coerce_number(inputs.get(1), "Fillet Distance Parameter")
        .unwrap_or(0.0)
        .max(0.0);
    let filleted = fillet_polyline(&points, distance);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CURVES.to_owned(), polyline_to_value(filleted));
    Ok(outputs)
}

fn evaluate_divide_curve_obsolete(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Divide Curve vereist een curve en segmentaantal",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Divide Curve")?;
    let segments = coerce_positive_integer(inputs.get(1), "Divide Curve")?;

    let mut point_values = Vec::new();
    let mut tangent_values = Vec::new();
    let mut parameter_values = Vec::new();

    for step in 0..=segments.max(1) {
        let parameter = step as f64 / segments.max(1) as f64;
        let sample = sample_polyline(&points, parameter);
        point_values.push(Value::Point(sample.point));
        tangent_values.push(Value::Vector(sample.tangent));
        parameter_values.push(Value::Number(parameter));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::List(point_values));
    outputs.insert(PIN_OUTPUT_TANGENTS.to_owned(), Value::List(tangent_values));
    outputs.insert(
        PIN_OUTPUT_PARAMETERS.to_owned(),
        Value::List(parameter_values),
    );
    Ok(outputs)
}

fn evaluate_offset_on_surface(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Offset on Surface vereist minimaal een curve",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Offset on Surface")?;
    let distance = coerce_number(inputs.get(1), "Offset on Surface Distance").unwrap_or(0.0);
    let plane = plane_from_polyline(&points);
    let offset = offset_polyline_points(&points, &plane, distance, is_closed_polyline(&points));

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_CURVES.to_owned(),
        Value::List(vec![polyline_to_value(offset)]),
    );
    Ok(outputs)
}

fn evaluate_fillet_parameter(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Fillet vereist minimaal een curve"));
    }

    let points = coerce_polyline(inputs.get(0), "Fillet Parameter")?;
    let parameter = coerce_number(inputs.get(1), "Fillet Parameter t").unwrap_or(0.5);
    let radius = coerce_number(inputs.get(2), "Fillet Parameter Radius")
        .unwrap_or(0.0)
        .max(0.0);
    let (filleted, actual_parameter) = fillet_polyline_at_parameter(&points, parameter, radius);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CURVES.to_owned(), polyline_to_value(filleted));
    outputs.insert(
        PIN_OUTPUT_PARAMETER.to_owned(),
        Value::Number(actual_parameter),
    );
    Ok(outputs)
}

fn evaluate_project_curve(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Project Curve vereist minimaal een curve",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Project Curve")?;
    let direction = inputs
        .get(2)
        .and_then(|value| coerce_vector(Some(value), "Project Direction").ok())
        .unwrap_or([0.0, 0.0, 1.0]);
    let projected = project_polyline(points, direction);

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_CURVES.to_owned(),
        Value::List(vec![polyline_to_value(projected)]),
    );
    Ok(outputs)
}

// --- Hulpfuncties -----------------------------------------------------------

fn coerce_polyline(value: Option<&Value>, context: &str) -> Result<Vec<[f64; 3]>, ComponentError> {
    let Some(value) = value else {
        return Err(ComponentError::new(format!(
            "{} vereist minimaal een curve",
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

fn coerce_polyline_collection(
    value: Option<&Value>,
    context: &str,
) -> Result<Vec<Vec<[f64; 3]>>, ComponentError> {
    let Some(value) = value else {
        return Err(ComponentError::new(format!(
            "{} vereist een lijst met curves",
            context
        )));
    };

    match value {
        Value::List(values) => {
            let mut polylines = Vec::new();
            for entry in values {
                polylines.push(coerce_polyline(Some(entry), context)?);
            }
            Ok(polylines)
        }
        _ => Ok(vec![coerce_polyline(Some(value), context)?]),
    }
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

fn coerce_positive_integer(value: Option<&Value>, context: &str) -> Result<usize, ComponentError> {
    let number = coerce_number(value, context)?;
    if number < 1.0 {
        return Err(ComponentError::new(format!(
            "{} vereist een positief geheel getal",
            context
        )));
    }
    Ok(number.round().max(1.0) as usize)
}

fn coerce_plane(value: Option<&Value>, context: &str) -> Result<Plane, ComponentError> {
    let Some(value) = value else {
        return Err(ComponentError::new(format!("{} vereist een vlak", context)));
    };

    match value {
        Value::List(values) if values.len() >= 3 => {
            let origin = coerce_point(values.get(0), context)?;
            let second = coerce_point(values.get(1), context)?;
            let third = coerce_point(values.get(2), context)?;
            Ok(Plane::from_points(origin, second, third))
        }
        Value::Point(point) => Ok(Plane::from_origin(*point)),
        Value::CurveLine { p1, p2 } => Ok(Plane::from_points(*p1, *p2, add(*p1, [0.0, 0.0, 1.0]))),
        other => Err(ComponentError::new(format!(
            "{} verwacht een vlak, kreeg {}",
            context,
            other.kind()
        ))),
    }
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
        Value::Point(point) => Ok(*point),
        Value::List(values) if values.len() == 1 => coerce_vector(values.get(0), context),
        other => Err(ComponentError::new(format!(
            "{} verwacht een vector, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn polyline_to_value(points: Vec<[f64; 3]>) -> Value {
    Value::List(points.into_iter().map(Value::Point).collect())
}

fn polylines_to_value(polylines: Vec<Vec<[f64; 3]>>) -> Value {
    Value::List(polylines.into_iter().map(polyline_to_value).collect())
}

fn is_effectively_empty_value(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::List(values) => values.is_empty() || values.iter().all(is_effectively_empty_value),
        _ => false,
    }
}

fn is_closed_polyline(points: &[[f64; 3]]) -> bool {
    if points.len() < 3 {
        return false;
    }
    distance(points[0], *points.last().unwrap()) < 1e-6
}

fn offset_polyline_points(
    points: &[[f64; 3]],
    plane: &Plane,
    distance: f64,
    closed: bool,
) -> Vec<[f64; 3]> {
    if distance.abs() < EPSILON {
        return points.to_vec();
    }

    let coords = points
        .iter()
        .map(|point| plane_coordinates(*point, plane))
        .collect::<Vec<_>>();
    let normals = compute_polyline_normals(&coords, closed);
    coords
        .iter()
        .zip(normals.iter())
        .map(|(coord, normal)| {
            apply_plane(
                plane,
                coord[0] + normal[0] * distance,
                coord[1] + normal[1] * distance,
                0.0,
            )
        })
        .collect()
}

fn compute_polyline_normals(coords: &[[f64; 3]], closed: bool) -> Vec<[f64; 3]> {
    let count = coords.len();
    if count == 0 {
        return Vec::new();
    }
    let mut normals = vec![[0.0, 0.0, 0.0]; count];
    let segment_count = if closed {
        count
    } else {
        count.saturating_sub(1)
    };

    for i in 0..segment_count {
        let a = coords[i];
        let b = coords[(i + 1) % count];
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let length = (dx * dx + dy * dy).sqrt();
        if length < EPSILON {
            continue;
        }
        let nx = -dy / length;
        let ny = dx / length;
        normals[i][0] += nx;
        normals[i][1] += ny;
        normals[(i + 1) % count][0] += nx;
        normals[(i + 1) % count][1] += ny;
    }

    for (index, normal) in normals.iter_mut().enumerate() {
        let length = (normal[0] * normal[0] + normal[1] * normal[1]).sqrt();
        if length < EPSILON {
            let prev_index = if index == 0 {
                if closed { count - 1 } else { 0 }
            } else {
                index - 1
            };
            let next_index = if index + 1 >= count {
                if closed { 0 } else { count - 1 }
            } else {
                index + 1
            };
            let prev = coords[prev_index];
            let next = coords[next_index];
            let dx = next[0] - prev[0];
            let dy = next[1] - prev[1];
            let fallback = (dx * dx + dy * dy).sqrt();
            if fallback > EPSILON {
                normal[0] = -dy / fallback;
                normal[1] = dx / fallback;
            }
        } else {
            normal[0] /= length;
            normal[1] /= length;
        }
    }

    normals
}

fn smooth_polyline(points: &[[f64; 3]], strength: f64, times: usize) -> Vec<[f64; 3]> {
    let mut result = points.to_vec();
    for _ in 0..times {
        if result.len() <= 2 {
            break;
        }
        let mut next = Vec::with_capacity(result.len());
        next.push(result[0]);
        for window in result.windows(3) {
            let prev = window[0];
            let current = window[1];
            let next_point = window[2];
            let target = scale(add(prev, next_point), 0.5);
            next.push(lerp(current, target, strength));
        }
        next.push(*result.last().unwrap());
        result = next;
    }
    result
}

fn join_polylines(polylines: Vec<Vec<[f64; 3]>>, preserve_direction: bool) -> Vec<Vec<[f64; 3]>> {
    let mut remaining = polylines;
    let mut result = Vec::new();

    while let Some(mut current) = remaining.pop() {
        let mut changed = true;
        while changed {
            changed = false;
            let mut index = 0;
            while index < remaining.len() {
                let candidate = &remaining[index];
                if try_merge_polylines(&mut current, candidate, preserve_direction) {
                    remaining.remove(index);
                    changed = true;
                } else {
                    index += 1;
                }
            }
        }
        result.push(current);
    }

    result
}

fn try_merge_polylines(
    target: &mut Vec<[f64; 3]>,
    candidate: &[[f64; 3]],
    preserve_direction: bool,
) -> bool {
    if target.is_empty() || candidate.len() < 2 {
        return false;
    }

    let tolerance = 1e-6;
    let start = target.first().copied().unwrap();
    let end = target.last().copied().unwrap();
    let candidate_start = candidate.first().copied().unwrap();
    let candidate_end = candidate.last().copied().unwrap();

    if distance(end, candidate_start) < tolerance {
        target.extend_from_slice(&candidate[1..]);
        return true;
    }
    if !preserve_direction && distance(end, candidate_end) < tolerance {
        let mut reversed = candidate.to_vec();
        reversed.reverse();
        target.extend_from_slice(&reversed[1..]);
        return true;
    }
    if distance(start, candidate_end) < tolerance {
        let mut new_points = candidate.to_vec();
        new_points.pop();
        new_points.extend_from_slice(target);
        *target = new_points;
        return true;
    }
    if !preserve_direction && distance(start, candidate_start) < tolerance {
        let mut reversed = candidate.to_vec();
        reversed.reverse();
        reversed.pop();
        reversed.extend_from_slice(target);
        *target = reversed;
        return true;
    }
    false
}

fn rdp_simplify(points: &[[f64; 3]], tolerance: f64, _angle_tolerance: f64) -> Vec<[f64; 3]> {
    if points.len() <= 2 {
        return points.to_vec();
    }

    let mut mask = vec![false; points.len()];
    mask[0] = true;
    mask[points.len() - 1] = true;

    rdp_recursive(points, tolerance, 0, points.len() - 1, &mut mask);

    let mut simplified = Vec::new();
    for (index, point) in points.iter().enumerate() {
        if mask[index] {
            simplified.push(*point);
        }
    }

    if simplified.len() < 2 {
        return vec![points[0], points[points.len() - 1]];
    }

    simplified
}

fn rdp_recursive(points: &[[f64; 3]], tolerance: f64, start: usize, end: usize, mask: &mut [bool]) {
    if end <= start + 1 {
        return;
    }

    let segment_start = points[start];
    let segment_end = points[end];
    let mut index = 0;
    let mut max_distance = -1.0;

    for i in start + 1..end {
        let distance = point_segment_distance(points[i], segment_start, segment_end);
        if distance > max_distance {
            max_distance = distance;
            index = i;
        }
    }

    if max_distance > tolerance {
        mask[index] = true;
        rdp_recursive(points, tolerance, start, index, mask);
        rdp_recursive(points, tolerance, index, end, mask);
    }
}

fn point_segment_distance(point: [f64; 3], a: [f64; 3], b: [f64; 3]) -> f64 {
    let ab = subtract(b, a);
    let ap = subtract(point, a);
    let ab_length_squared = dot(ab, ab);
    if ab_length_squared <= EPSILON {
        return length(ap);
    }
    let t = (dot(ap, ab) / ab_length_squared).clamp(0.0, 1.0);
    let projection = add(a, scale(ab, t));
    length(subtract(point, projection))
}

fn remesh_polyline(points: &[[f64; 3]], min_edge: f64, max_edge: f64) -> Vec<[f64; 3]> {
    if points.len() <= 2 {
        return points.to_vec();
    }

    let mut result = Vec::new();
    result.push(points[0]);

    for pair in points.windows(2) {
        let start = pair[0];
        let end = pair[1];
        let segment = subtract(end, start);
        let length = length(segment);
        if length < min_edge {
            continue;
        }
        let steps = (length / max_edge).ceil().max(1.0) as usize;
        for step in 1..=steps {
            let t = step as f64 / steps as f64;
            let point = lerp(start, end, t);
            if step == steps {
                result.push(point);
            } else if length / steps as f64 >= min_edge {
                result.push(point);
            }
        }
    }

    if result.len() < 2 {
        return points.to_vec();
    }

    result
}

fn resample_polyline(points: &[[f64; 3]], count: usize) -> Vec<[f64; 3]> {
    if points.len() < 2 || count <= 2 {
        return vec![points[0], points[points.len() - 1]];
    }

    let total_length = polyline_length(points);
    if total_length < EPSILON {
        return vec![points[0]; count];
    }

    let mut samples = Vec::with_capacity(count);
    samples.push(points[0]);
    let segment_lengths = polyline_segments(points);
    let mut accumulated = 0.0;
    let mut segment_index = 0;
    let mut segment_progress = 0.0;

    for step in 1..count - 1 {
        let target_length = (step as f64 / (count as f64 - 1.0)) * total_length;
        while segment_index < segment_lengths.len()
            && accumulated + segment_lengths[segment_index].length < target_length
        {
            accumulated += segment_lengths[segment_index].length;
            segment_index += 1;
            segment_progress = 0.0;
        }

        if segment_index >= segment_lengths.len() {
            samples.push(*points.last().unwrap());
            continue;
        }

        let segment = &segment_lengths[segment_index];
        let remaining = target_length - accumulated;
        let t = if segment.length < EPSILON {
            0.0
        } else {
            (segment_progress + remaining) / segment.length
        };
        let t = t.clamp(0.0, 1.0);
        samples.push(lerp(segment.start, segment.end, t));
        segment_progress += remaining;
    }

    samples.push(*points.last().unwrap());
    samples
}

fn collapse_polyline(points: &[[f64; 3]], tolerance: f64) -> (Vec<[f64; 3]>, usize) {
    if points.len() <= 2 {
        return (points.to_vec(), 0);
    }

    let mut result = Vec::with_capacity(points.len());
    result.push(points[0]);
    let mut removed = 0;

    for pair in points.windows(2) {
        if distance(pair[0], pair[1]) < tolerance {
            removed += 1;
            continue;
        }
        result.push(pair[1]);
    }

    if result.len() < 2 {
        result.push(points[points.len() - 1]);
    }

    (result, removed)
}

fn polyline_segments(points: &[[f64; 3]]) -> Vec<PolylineSegment> {
    points
        .windows(2)
        .map(|pair| PolylineSegment {
            start: pair[0],
            end: pair[1],
            length: distance(pair[0], pair[1]),
        })
        .collect()
}

fn polyline_length(points: &[[f64; 3]]) -> f64 {
    polyline_segments(points)
        .iter()
        .map(|segment| segment.length)
        .sum()
}

fn plane_coordinates(point: [f64; 3], plane: &Plane) -> [f64; 3] {
    let relative = subtract(point, plane.origin);
    [
        dot(relative, plane.x_axis),
        dot(relative, plane.y_axis),
        dot(relative, plane.normal),
    ]
}

fn apply_plane(plane: &Plane, u: f64, v: f64, w: f64) -> [f64; 3] {
    add(
        add(plane.origin, scale(plane.x_axis, u)),
        add(scale(plane.y_axis, v), scale(plane.normal, w)),
    )
}

fn plane_from_polyline(points: &[[f64; 3]]) -> Plane {
    if points.len() < 3 {
        return Plane::from_origin(points.first().copied().unwrap_or([0.0, 0.0, 0.0]));
    }
    Plane::from_points(points[0], points[1], points[2])
}

#[derive(Debug, Clone, Copy)]
struct Plane {
    origin: [f64; 3],
    x_axis: [f64; 3],
    y_axis: [f64; 3],
    normal: [f64; 3],
}

impl Plane {
    fn from_origin(origin: [f64; 3]) -> Self {
        Self {
            origin,
            x_axis: [1.0, 0.0, 0.0],
            y_axis: [0.0, 1.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        }
    }

    fn from_points(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> Self {
        let x_axis = subtract(b, a);
        let mut normal = cross(subtract(b, a), subtract(c, a));
        if length_squared(normal) < EPSILON {
            normal = [0.0, 0.0, 1.0];
        }
        let normal = normalize(normal);
        let x_axis = normalize(x_axis);
        let y_axis = normalize(cross(normal, x_axis));
        Self {
            origin: a,
            x_axis,
            y_axis,
            normal,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PolylineSegment {
    start: [f64; 3],
    end: [f64; 3],
    length: f64,
}

fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn length(vector: [f64; 3]) -> f64 {
    (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt()
}

fn length_squared(vector: [f64; 3]) -> f64 {
    vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]
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

fn lerp(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn deduplicate_polyline(points: Vec<[f64; 3]>) -> Vec<[f64; 3]> {
    let mut result = Vec::new();
    for point in points {
        if result
            .last()
            .map_or(true, |prev| distance(*prev, point) > EPSILON)
        {
            result.push(point);
        }
    }
    result
}

fn fillet_corner(
    prev: [f64; 3],
    current: [f64; 3],
    next: [f64; 3],
    radius: f64,
) -> Option<([f64; 3], [f64; 3], [f64; 3])> {
    if radius <= EPSILON {
        return None;
    }

    let to_prev = subtract(current, prev);
    let to_next = subtract(next, current);
    let len_prev = length(to_prev);
    let len_next = length(to_next);
    if len_prev < EPSILON || len_next < EPSILON {
        return None;
    }

    let trim = radius.min(len_prev / 2.0).min(len_next / 2.0);
    if trim <= EPSILON {
        return None;
    }

    let dir_prev = scale(to_prev, 1.0 / len_prev);
    let dir_next = scale(to_next, 1.0 / len_next);
    let start = subtract(current, scale(dir_prev, trim));
    let end = add(current, scale(dir_next, trim));
    let mid = lerp(start, end, 0.5);
    Some((start, mid, end))
}

fn fillet_polyline(points: &[[f64; 3]], radius: f64) -> Vec<[f64; 3]> {
    if points.len() < 3 || radius <= EPSILON {
        return points.to_vec();
    }

    let mut result = Vec::with_capacity(points.len() * 2);
    result.push(points[0]);
    for index in 1..points.len() - 1 {
        let prev = points[index - 1];
        let current = points[index];
        let next = points[index + 1];
        if let Some((start, mid, end)) = fillet_corner(prev, current, next, radius) {
            if distance(*result.last().unwrap(), start) > EPSILON {
                result.push(start);
            }
            result.push(mid);
            result.push(end);
        } else {
            result.push(current);
        }
    }
    result.push(*points.last().unwrap());
    deduplicate_polyline(result)
}

fn fillet_polyline_at_parameter(
    points: &[[f64; 3]],
    parameter: f64,
    radius: f64,
) -> (Vec<[f64; 3]>, f64) {
    if points.len() < 3 || radius <= EPSILON {
        let clamped = parameter.clamp(0.0, 1.0);
        return (points.to_vec(), clamped);
    }

    let clamped = parameter.clamp(0.0, 1.0);
    let segments = points.len().saturating_sub(1) as f64;
    let mut index = (clamped * segments).round() as isize;
    index = index.clamp(1, points.len() as isize - 2);
    let index = index as usize;
    let actual = index as f64 / segments;

    if let Some((start, mid, end)) =
        fillet_corner(points[index - 1], points[index], points[index + 1], radius)
    {
        let mut result = Vec::with_capacity(points.len() + 2);
        result.extend_from_slice(&points[..index]);
        if distance(*result.last().unwrap(), start) > EPSILON {
            result.push(start);
        }
        result.push(mid);
        result.push(end);
        result.extend_from_slice(&points[index + 1..]);
        (deduplicate_polyline(result), actual)
    } else {
        (points.to_vec(), actual)
    }
}

fn rotate_polyline_seam(points: &[[f64; 3]], parameter: f64) -> Vec<[f64; 3]> {
    if points.len() < 3 {
        return points.to_vec();
    }

    let total_length = polyline_length(points);
    if total_length < EPSILON {
        return points.to_vec();
    }

    let normalized = if parameter.is_finite() {
        parameter.rem_euclid(1.0)
    } else {
        0.0
    };
    let target = normalized * total_length;
    let segments = polyline_segments(points);
    let mut accumulated = 0.0;

    for (index, segment) in segments.iter().enumerate() {
        if accumulated + segment.length >= target || index == segments.len() - 1 {
            let local = if segment.length < EPSILON {
                0.0
            } else {
                ((target - accumulated).max(0.0) / segment.length).clamp(0.0, 1.0)
            };
            let seam_point = lerp(segment.start, segment.end, local);
            let mut result = Vec::with_capacity(points.len() + 1);
            result.push(seam_point);
            result.extend(points.iter().skip(index + 1).copied());
            result.extend(points.iter().take(index + 1).copied());
            result.push(seam_point);
            return deduplicate_polyline(result);
        }
        accumulated += segment.length;
    }

    points.to_vec()
}

fn extend_polyline(points: &[[f64; 3]], start: f64, end: f64) -> Vec<[f64; 3]> {
    if points.len() < 2 {
        return points.to_vec();
    }

    let mut result = points.to_vec();

    if start < -EPSILON {
        result = trim_polyline_start(&result, -start);
    } else {
        let start_length = start.max(0.0);
        if start_length > EPSILON && result.len() >= 2 {
            let direction = normalize(subtract(result[1], result[0]));
            let new_point = subtract(result[0], scale(direction, start_length));
            result.insert(0, new_point);
        }
    }

    if result.len() < 2 {
        return result;
    }

    if end < -EPSILON {
        result = trim_polyline_end(&result, -end);
    } else {
        let end_length = end.max(0.0);
        if end_length > EPSILON && result.len() >= 2 {
            let direction = normalize(subtract(*result.last().unwrap(), result[result.len() - 2]));
            let new_point = add(*result.last().unwrap(), scale(direction, end_length));
            result.push(new_point);
        }
    }

    deduplicate_polyline(result)
}

fn trim_polyline_start(points: &[[f64; 3]], trim_length: f64) -> Vec<[f64; 3]> {
    if points.len() < 2 || trim_length <= EPSILON {
        return points.to_vec();
    }

    let total_length = polyline_length(points);
    if total_length <= EPSILON {
        return points.to_vec();
    }

    let trim = trim_length.min(total_length);
    let segments = polyline_segments(points);
    let mut accumulated = 0.0;

    for (index, segment) in segments.iter().enumerate() {
        if trim - accumulated >= segment.length - EPSILON {
            accumulated += segment.length;
            continue;
        }

        let remaining = (trim - accumulated).max(0.0);
        let t = if segment.length < EPSILON {
            0.0
        } else {
            (remaining / segment.length).clamp(0.0, 1.0)
        };
        let new_start = lerp(segment.start, segment.end, t);
        let mut result = Vec::with_capacity(points.len() - index);
        result.push(new_start);
        result.extend(points.iter().skip(index + 1).copied());
        return deduplicate_polyline(result);
    }

    vec![*points.last().unwrap()]
}

fn trim_polyline_end(points: &[[f64; 3]], trim_length: f64) -> Vec<[f64; 3]> {
    if points.len() < 2 || trim_length <= EPSILON {
        return points.to_vec();
    }

    let total_length = polyline_length(points);
    if total_length <= EPSILON {
        return points.to_vec();
    }

    let trim = trim_length.min(total_length);
    let segments = polyline_segments(points);
    let mut accumulated = 0.0;

    for (index, segment) in segments.iter().enumerate().rev() {
        if trim - accumulated >= segment.length - EPSILON {
            accumulated += segment.length;
            continue;
        }

        let remaining = (trim - accumulated).max(0.0);
        let t = if segment.length < EPSILON {
            1.0
        } else {
            1.0 - (remaining / segment.length).clamp(0.0, 1.0)
        };
        let new_end = lerp(segment.start, segment.end, t);
        let mut result = Vec::with_capacity(index + 2);
        result.extend(points.iter().take(index + 1).copied());
        result.push(new_end);
        return deduplicate_polyline(result);
    }

    vec![points[0]]
}

#[derive(Debug, Clone, Copy)]
struct PolylineSample {
    point: [f64; 3],
    tangent: [f64; 3],
}

fn sample_polyline(points: &[[f64; 3]], parameter: f64) -> PolylineSample {
    if points.len() < 2 {
        return PolylineSample {
            point: points.first().copied().unwrap_or([0.0, 0.0, 0.0]),
            tangent: [1.0, 0.0, 0.0],
        };
    }

    let clamped = parameter.clamp(0.0, 1.0);
    let total_length = polyline_length(points);
    if total_length < EPSILON {
        return PolylineSample {
            point: points[0],
            tangent: subtract(points[1], points[0]),
        };
    }

    let target = clamped * total_length;
    let segments = polyline_segments(points);
    let mut accumulated = 0.0;

    for segment in &segments {
        if accumulated + segment.length >= target {
            let local = if segment.length < EPSILON {
                0.0
            } else {
                ((target - accumulated).max(0.0) / segment.length).clamp(0.0, 1.0)
            };
            let point = lerp(segment.start, segment.end, local);
            let tangent = subtract(segment.end, segment.start);
            return PolylineSample { point, tangent };
        }
        accumulated += segment.length;
    }

    PolylineSample {
        point: *points.last().unwrap(),
        tangent: subtract(*points.last().unwrap(), points[points.len() - 2]),
    }
}

fn compute_perp_frames(
    points: &[[f64; 3]],
    segments: usize,
    align: bool,
) -> (Vec<Value>, Vec<Value>) {
    let mut frames = Vec::new();
    let mut parameters = Vec::new();
    let mut previous_axes: Option<([f64; 3], [f64; 3])> = None;

    for step in 0..=segments {
        let parameter = step as f64 / segments as f64;
        let sample = sample_polyline(points, parameter);
        let tangent = normalize(sample.tangent);
        let mut normal = normalize(cross([0.0, 0.0, 1.0], tangent));
        if length_squared(normal) < EPSILON {
            normal = normalize(cross([1.0, 0.0, 0.0], tangent));
        }
        let mut binormal = normalize(cross(tangent, normal));

        if align {
            if let Some((prev_normal, prev_binormal)) = previous_axes {
                if dot(normal, prev_normal) < 0.0 {
                    normal = scale(normal, -1.0);
                }
                if dot(binormal, prev_binormal) < 0.0 {
                    binormal = scale(binormal, -1.0);
                }
            }
            previous_axes = Some((normal, binormal));
        }

        frames.push(frame_value(sample.point, tangent, normal, binormal));
        parameters.push(Value::Number(parameter));
    }

    (frames, parameters)
}

fn frame_value(origin: [f64; 3], x_axis: [f64; 3], y_axis: [f64; 3], z_axis: [f64; 3]) -> Value {
    Value::List(vec![
        Value::Point(origin),
        Value::Vector(x_axis),
        Value::Vector(y_axis),
        Value::Vector(z_axis),
    ])
}

fn project_polyline(mut points: Vec<[f64; 3]>, direction: [f64; 3]) -> Vec<[f64; 3]> {
    if points.is_empty() {
        return points;
    }

    let axis = normalize(direction);
    let origin = points[0];
    for point in &mut points {
        let relative = subtract(*point, origin);
        let distance = dot(relative, axis);
        *point = subtract(*point, scale(axis, distance));
    }
    points
}

const EPSILON: f64 = 1e-9;

#[cfg(test)]
mod tests {
    use super::{
        Component, ComponentKind, PIN_OUTPUT_CURVES, PIN_OUTPUT_FLAG, PIN_OUTPUT_PARAMETER,
        PIN_OUTPUT_PARAMETERS, PIN_OUTPUT_POINTS, PIN_OUTPUT_POLYLINE, PIN_OUTPUT_SEGMENTS,
        PIN_OUTPUT_SIMPLIFIED, PIN_OUTPUT_TANGENTS, coerce_polyline,
    };
    use crate::graph::node::MetaMap;
    use crate::graph::value::Value;

    #[test]
    fn offset_curve_moves_points() {
        let component = ComponentKind::OffsetCurve;
        let inputs = vec![
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([1.0, 0.0, 0.0]),
                Value::Point([1.0, 1.0, 0.0]),
            ]),
            Value::Number(1.0),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("offset succeed");
        let curves = outputs
            .get(PIN_OUTPUT_CURVES)
            .and_then(|value| value.expect_list().ok())
            .expect("curve list");
        assert_eq!(curves.len(), 1);
    }

    #[test]
    fn flip_curve_allows_missing_guide_input() {
        let component = ComponentKind::FlipCurve;
        let inputs = vec![
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([2.0, 0.0, 0.0]),
            ]),
            Value::Null,
        ];

        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("flip without guide");

        let curve = outputs
            .get(PIN_OUTPUT_CURVES)
            .and_then(|value| value.expect_list().ok())
            .expect("curve output");
        let flipped: Vec<Value> = curve.to_vec();
        assert_eq!(
            flipped,
            vec![Value::Point([2.0, 0.0, 0.0]), Value::Point([0.0, 0.0, 0.0]),]
        );
        assert_eq!(outputs.get(PIN_OUTPUT_FLAG), Some(&Value::Boolean(true)));
    }

    #[test]
    fn flip_curve_treats_empty_guide_as_missing() {
        let component = ComponentKind::FlipCurve;
        let inputs = vec![
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([3.0, 0.0, 0.0]),
            ]),
            Value::List(vec![]),
        ];

        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("flip with empty guide");

        let curve = outputs
            .get(PIN_OUTPUT_CURVES)
            .and_then(|value| value.expect_list().ok())
            .expect("curve output");
        let flipped: Vec<Value> = curve.to_vec();
        assert_eq!(
            flipped,
            vec![Value::Point([3.0, 0.0, 0.0]), Value::Point([0.0, 0.0, 0.0]),]
        );
        assert_eq!(outputs.get(PIN_OUTPUT_FLAG), Some(&Value::Boolean(true)));
    }

    #[test]
    fn flip_curve_reverses_closed_winding() {
        let component = ComponentKind::FlipCurve;
        let inputs = vec![Value::List(vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Point([0.0, 1.0, 0.0]),
            Value::Point([0.0, 0.0, 0.0]),
        ])];

        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("flip closed curve");

        let curve = outputs
            .get(PIN_OUTPUT_CURVES)
            .and_then(|value| value.expect_list().ok())
            .expect("curve output");
        let flipped: Vec<Value> = curve.to_vec();
        assert_eq!(
            flipped,
            vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([0.0, 1.0, 0.0]),
                Value::Point([1.0, 0.0, 0.0]),
                Value::Point([0.0, 0.0, 0.0]),
            ]
        );
        assert_eq!(outputs.get(PIN_OUTPUT_FLAG), Some(&Value::Boolean(true)));
    }

    #[test]
    fn curve_to_polyline_reports_segment_count() {
        let component = ComponentKind::CurveToPolyline;
        let inputs = vec![Value::List(vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([2.0, 0.0, 0.0]),
            Value::Point([2.0, 2.0, 0.0]),
        ])];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("curve to polyline succeed");
        assert!(outputs.contains_key(PIN_OUTPUT_POLYLINE));
        match outputs.get(PIN_OUTPUT_SEGMENTS) {
            Some(Value::Number(count)) => assert!(*count >= 2.0),
            _ => panic!("expected segment count"),
        }
    }

    #[test]
    fn simplify_curve_reports_change() {
        let component = ComponentKind::SimplifyCurve;
        let inputs = vec![
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([0.5, 0.1, 0.0]),
                Value::Point([1.0, 0.0, 0.0]),
            ]),
            Value::Number(0.05),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("simplify succeed");
        match outputs.get(PIN_OUTPUT_SIMPLIFIED) {
            Some(Value::Boolean(changed)) => assert!(!changed),
            _ => panic!("expected simplified flag"),
        }
    }

    #[test]
    fn coerce_polyline_handles_lists() {
        let points = coerce_polyline(
            Some(&Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([1.0, 0.0, 0.0]),
            ])),
            "Test",
        )
        .expect("polyline");
        assert_eq!(points.len(), 2);
    }

    #[test]
    fn fillet_radius_adds_intermediate_points() {
        let component = ComponentKind::FilletRadius;
        let inputs = vec![
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([1.0, 0.0, 0.0]),
                Value::Point([1.0, 1.0, 0.0]),
            ]),
            Value::Number(0.5),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("fillet succeed");
        let curve = outputs
            .get(PIN_OUTPUT_CURVES)
            .and_then(|value| value.expect_list().ok())
            .expect("fillet polyline");
        assert!(curve.len() >= 4);
    }

    #[test]
    fn seam_rotates_closed_curve() {
        let component = ComponentKind::Seam;
        let inputs = vec![
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([1.0, 0.0, 0.0]),
                Value::Point([1.0, 1.0, 0.0]),
                Value::Point([0.0, 1.0, 0.0]),
                Value::Point([0.0, 0.0, 0.0]),
            ]),
            Value::Number(0.25),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("seam succeed");
        let result = outputs
            .get(PIN_OUTPUT_CURVES)
            .and_then(|value| value.expect_list().ok())
            .expect("seam polyline");
        match result.first() {
            Some(Value::Point(point)) => assert_eq!(*point, [1.0, 0.0, 0.0]),
            other => panic!("unexpected first point: {other:?}"),
        }
    }

    #[test]
    fn extend_curve_adds_endpoints() {
        let component = ComponentKind::ExtendCurve;
        let inputs = vec![
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([1.0, 0.0, 0.0]),
            ]),
            Value::Number(0.0),
            Value::Number(0.5),
            Value::Number(0.5),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("extend succeed");
        let result = outputs
            .get(PIN_OUTPUT_CURVES)
            .and_then(|value| value.expect_list().ok())
            .expect("extended polyline");
        assert!(result.len() >= 3);
    }

    #[test]
    fn extend_curve_trims_start_for_negative_values() {
        let component = ComponentKind::ExtendCurve;
        let inputs = vec![
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([1.0, 0.0, 0.0]),
                Value::Point([2.0, 0.0, 0.0]),
            ]),
            Value::Number(0.0),
            Value::Number(-0.5),
            Value::Number(0.0),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("extend succeed");
        let result = outputs
            .get(PIN_OUTPUT_CURVES)
            .and_then(|value| value.expect_list().ok())
            .expect("trimmed polyline");
        match result.first() {
            Some(Value::Point(point)) => assert!((point[0] - 0.5).abs() < 1e-6),
            other => panic!("unexpected first point: {other:?}"),
        }
    }

    #[test]
    fn extend_curve_trims_end_for_negative_values() {
        let component = ComponentKind::ExtendCurve;
        let inputs = vec![
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([1.0, 0.0, 0.0]),
                Value::Point([2.0, 0.0, 0.0]),
            ]),
            Value::Number(0.0),
            Value::Number(0.0),
            Value::Number(-0.75),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("extend succeed");
        let result = outputs
            .get(PIN_OUTPUT_CURVES)
            .and_then(|value| value.expect_list().ok())
            .expect("trimmed polyline");
        match result.last() {
            Some(Value::Point(point)) => assert!((point[0] - 1.25).abs() < 1e-6),
            other => panic!("unexpected last point: {other:?}"),
        }
    }

    #[test]
    fn divide_curve_obsolete_outputs_samples() {
        let component = ComponentKind::DivideCurveObsolete;
        let inputs = vec![
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([1.0, 0.0, 0.0]),
                Value::Point([1.0, 1.0, 0.0]),
            ]),
            Value::Number(2.0),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("divide succeed");
        assert!(outputs.contains_key(PIN_OUTPUT_POINTS));
        assert!(outputs.contains_key(PIN_OUTPUT_TANGENTS));
        assert!(outputs.contains_key(PIN_OUTPUT_PARAMETERS));
    }

    #[test]
    fn fillet_parameter_returns_parameter_output() {
        let component = ComponentKind::FilletParameter;
        let inputs = vec![
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([1.0, 0.0, 0.0]),
                Value::Point([1.0, 1.0, 0.0]),
            ]),
            Value::Number(0.5),
            Value::Number(0.2),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("fillet parameter succeed");
        assert!(outputs.contains_key(PIN_OUTPUT_PARAMETER));
    }
}
