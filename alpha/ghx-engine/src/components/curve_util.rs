//! Implementaties van Grasshopper "Curve → Util" componenten.

use std::collections::BTreeMap;

use crate::geom::{
    self,
    // Offset
    OffsetPolylineOptions,
    // Join
    JoinPolylinesOptions,
    // Flip
    FlipPolylineOptions,
    // Extend
    ExtendPolylineOptions,
    // Smooth
    SmoothPolylineOptions,
    // Simplify
    SimplifyPolylineOptions,
    // Resample
    ResamplePolylineOptions,
    // Remesh
    RemeshPolylineOptions,
    // Collapse
    CollapsePolylineOptions,
    // Rotate seam
    RotateSeamOptions,
    // Project
    ProjectPolylineOptions,
    // Fillet at parameter
    FilletAtParameterOptions,
    // Perp frames
    PerpFramesOptions,
    // Tolerance
    Tolerance,
};
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

    // Build offset options using geom module
    let closed = if closed_hint {
        is_closed_polyline(&points)
    } else {
        false
    };

    let mut options = OffsetPolylineOptions::new(distance).closed(closed);

    // If a plane is provided, extract its properties
    if let Some(plane_value) = inputs.get(2) {
        if let Ok(plane) = coerce_plane(Some(plane_value), "Offset Curve") {
            options = options
                .with_plane_origin(plane.origin)
                .with_plane_normal(plane.normal)
                .with_plane_x_axis(plane.x_axis);
        }
    }

    // Call geom offset function
    let offset = match geom::offset_polyline(&points, options, Tolerance::default_geom()) {
        Ok((result, _diagnostics)) => result,
        Err(e) => {
            return Err(ComponentError::new(format!("Offset Curve failed: {}", e)));
        }
    };

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
    let closed = is_closed_polyline(&points);

    // Use geom offset function
    let options = OffsetPolylineOptions::new(distance).closed(closed);
    let (offset, _diagnostics) = match geom::offset_polyline(&points, options, Tolerance::default_geom()) {
        Ok(result) => result,
        Err(e) => {
            let mut outputs = BTreeMap::new();
            outputs.insert(
                PIN_OUTPUT_OFFSET.to_owned(),
                Value::List(vec![polyline_to_value(points)]),
            );
            outputs.insert(
                PIN_OUTPUT_VALID.to_owned(),
                Value::List(vec![Value::Boolean(false)]),
            );
            // Log the error but don't fail the component
            eprintln!("Offset Polyline warning: {}", e);
            return Ok(outputs);
        }
    };

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

    // Get optional guide polyline
    let guide: Option<Vec<[f64; 3]>> = inputs
        .get(1)
        .filter(|v| !is_effectively_empty_value(v))
        .and_then(|v| coerce_polyline(Some(v), "Flip Curve Guide").ok());

    // Use geom flip function
    let guide_ref = guide.as_deref();
    let (final_points, diagnostics) = geom::flip_polyline(&points, guide_ref, FlipPolylineOptions::new());

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_CURVES.to_owned(),
        polyline_to_value(final_points),
    );
    outputs.insert(PIN_OUTPUT_FLAG.to_owned(), Value::Boolean(diagnostics.was_flipped));
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

    // Use geom simplify function
    let simplify_options = SimplifyPolylineOptions::new(distance_tolerance)
        .with_angle_tolerance(angle_tolerance.max(0.0));
    let (simplified, _) = geom::simplify_polyline(&points, simplify_options);

    // Use geom remesh function
    let remesh_options = RemeshPolylineOptions::new(min_edge, max_edge);
    let (remeshed, _) = geom::remesh_polyline(&simplified, remesh_options);
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

    // Use geom smooth function
    let options = SmoothPolylineOptions::new(strength, times);
    let (smoothed, _diagnostics) = geom::smooth_polyline(&points, options);
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

    // Use geom join function
    let options = JoinPolylinesOptions::new().preserve_direction(preserve_direction);
    let (joined, _diagnostics) = geom::join_polylines(polylines, options);

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

    // Use geom simplify function
    let options = SimplifyPolylineOptions::new(tolerance);
    let (reduced, diagnostics) = geom::simplify_polyline(&points, options);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POLYLINE.to_owned(), polyline_to_value(reduced));
    outputs.insert(
        PIN_OUTPUT_REDUCTION.to_owned(),
        Value::Number(diagnostics.points_removed as f64),
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

    // Use geom simplify function
    let options = SimplifyPolylineOptions::new(tolerance)
        .with_angle_tolerance(angle_tolerance);
    let (simplified, diagnostics) = geom::simplify_polyline(&points, options);
    let changed = diagnostics.points_removed > 0;

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

    // Use geom simplify function
    let options = SimplifyPolylineOptions::new(tolerance);
    let (mut simplified, _) = geom::simplify_polyline(&points, options);
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

    // Use geom resample function
    let options = ResamplePolylineOptions::new(count);
    let (rebuilt, _diagnostics) = geom::resample_polyline(&points, options);
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

    // Use geom collapse function
    let options = CollapsePolylineOptions::new(tolerance);
    let (collapsed, diagnostics) = geom::collapse_polyline(&points, options);
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POLYLINE.to_owned(), polyline_to_value(collapsed));
    outputs.insert(PIN_OUTPUT_COUNT.to_owned(), Value::Number(diagnostics.points_removed as f64));
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
    // Optional: segment count for arc resolution (default 8 for smooth arcs)
    let segments = coerce_number(inputs.get(2), "Fillet Segments")
        .unwrap_or(8.0)
        .max(1.0)
        .round() as usize;
    let closed = is_closed_polyline(&points);

    // Convert to Point3 for geom fillet function
    let point3_vec: Vec<geom::Point3> = points
        .iter()
        .map(|p| geom::Point3::from_array(*p))
        .collect();

    // Use geom fillet function
    let filleted = match geom::fillet_polyline_points(&point3_vec, radius, segments, closed, Tolerance::default_geom()) {
        Ok((result, diagnostics)) => {
            // Log diagnostics for debugging if any corners were skipped
            if diagnostics.skipped_corner_count > 0 {
                eprintln!(
                    "Fillet: {} of {} corners skipped (radius too large or degenerate)",
                    diagnostics.skipped_corner_count,
                    diagnostics.corner_count
                );
            }
            result
                .into_iter()
                .map(|p| p.to_array())
                .collect()
        }
        Err(e) => {
            // Log error for debugging but return original points for graceful degradation
            eprintln!("Fillet warning: {}", e);
            points
        }
    };

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
    
    // Use geom rotate seam function
    let options = RotateSeamOptions::new(parameter);
    let (adjusted, _diagnostics) = geom::rotate_polyline_seam(&points, options, Tolerance::default_geom());

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

    // Use geom extend function
    let options = ExtendPolylineOptions::new(start, end);
    let (extended, _diagnostics) = geom::extend_polyline(&points, options, Tolerance::default_geom());

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

    // Use geom perp frames function
    let options = PerpFramesOptions::new(segments.max(1), align);
    let (frames, _diagnostics) = geom::compute_perp_frames(&points, options);
    
    // Convert frames to Value format
    let frame_values: Vec<Value> = frames.iter().map(|f| {
        Value::List(vec![
            Value::Point(f.origin),
            Value::Vector(f.tangent),
            Value::Vector(f.normal),
            Value::Vector(f.binormal),
        ])
    }).collect();
    
    let parameter_values: Vec<Value> = frames.iter().map(|f| Value::Number(f.parameter)).collect();

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_FRAMES.to_owned(), Value::List(frame_values));
    outputs.insert(PIN_OUTPUT_PARAMETERS.to_owned(), Value::List(parameter_values));
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
    // Optional: segment count for arc resolution (default 8 for smooth arcs)
    let segments = coerce_number(inputs.get(2), "Fillet Distance Segments")
        .unwrap_or(8.0)
        .max(1.0)
        .round() as usize;
    let closed = is_closed_polyline(&points);

    // Convert to Point3 for geom fillet function
    let point3_vec: Vec<geom::Point3> = points
        .iter()
        .map(|p| geom::Point3::from_array(*p))
        .collect();

    // Use geom fillet function
    let filleted = match geom::fillet_polyline_points(&point3_vec, distance, segments, closed, Tolerance::default_geom()) {
        Ok((result, diagnostics)) => {
            // Log diagnostics for debugging if any corners were skipped
            if diagnostics.skipped_corner_count > 0 {
                eprintln!(
                    "Fillet Distance: {} of {} corners skipped (distance too large or degenerate)",
                    diagnostics.skipped_corner_count,
                    diagnostics.corner_count
                );
            }
            result
                .into_iter()
                .map(|p| p.to_array())
                .collect()
        }
        Err(e) => {
            // Log error for debugging but return original points for graceful degradation
            eprintln!("Fillet Distance warning: {}", e);
            points
        }
    };

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
        // Use geom sample function
        let sample = geom::sample_polyline_at(&points, parameter);
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
    let closed = is_closed_polyline(&points);

    // Use geom offset function
    let options = OffsetPolylineOptions::new(distance).closed(closed);
    let offset = match geom::offset_polyline(&points, options, Tolerance::default_geom()) {
        Ok((result, _diagnostics)) => result,
        Err(_) => points,
    };

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
    
    // Use geom fillet at parameter function
    let options = FilletAtParameterOptions::new(parameter, radius);
    let (filleted, diagnostics) = geom::fillet_polyline_at_parameter(&points, options, Tolerance::default_geom());

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CURVES.to_owned(), polyline_to_value(filleted));
    outputs.insert(
        PIN_OUTPUT_PARAMETER.to_owned(),
        Value::Number(diagnostics.actual_parameter),
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
    
    // Use geom project function
    let options = ProjectPolylineOptions::new(direction);
    let (projected, _diagnostics) = geom::project_polyline(&points, options);

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
    let a = points[0];
    let b = *points.last().unwrap();
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt() < 1e-6
}

/// Tolerance constant used for local checks.
const EPSILON: f64 = 1e-9;

/// Simple plane representation for offset operations.
#[derive(Debug, Clone, Copy)]
struct Plane {
    pub origin: [f64; 3],
    pub x_axis: [f64; 3],
    pub y_axis: [f64; 3],
    pub normal: [f64; 3],
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
        let x_vec = sub(b, a);
        let ac = sub(c, a);
        let mut normal = cross(x_vec, ac);
        if length_squared(normal) < EPSILON {
            normal = [0.0, 0.0, 1.0];
        }
        let normal = normalize(normal);
        let x_axis = normalize(x_vec);
        let y_axis = normalize(cross(normal, x_axis));
        Self {
            origin: a,
            x_axis,
            y_axis,
            normal,
        }
    }
}

// Vector math helpers
fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(v: [f64; 3], s: f64) -> [f64; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn length_squared(v: [f64; 3]) -> f64 {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

fn length(v: [f64; 3]) -> f64 {
    length_squared(v).sqrt()
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = length(v);
    if len < EPSILON {
        [1.0, 0.0, 0.0]
    } else {
        scale(v, 1.0 / len)
    }
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
