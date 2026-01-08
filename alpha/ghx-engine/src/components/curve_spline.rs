//! Implementaties van Grasshopper "Curve → Spline" componenten.
//!
//! This module uses the `geom::curve` primitives for spline curve construction and
//! `geom::tessellation` for adaptive tessellation. Components remain thin
//! wrappers that coerce inputs, build geom curves, and return tessellated output.
//!
//! # Curve Types
//!
//! - **NURBS Curve**: Uses `geom::NurbsCurve3` for B-spline/NURBS representation
//! - **Bezier Span**: Uses `geom::CubicBezier3` for cubic Bezier curves
//! - **Polyline**: Uses `geom::Polyline3` for polyline representation
//! - **Interpolate**: Uses `geom::NurbsCurve3::interpolate_through_points()` for interpolating curves

use std::collections::BTreeMap;

use crate::geom::{
    CubicBezier3, CurveTessellationOptions, Curve3, NurbsCurve3, Point3 as GeomPoint3,
    Polyline3, Vec3 as GeomVec3, tessellate_curve_adaptive_points,
};
use crate::graph::node::MetaMap;
use crate::graph::value::{Domain, Domain1D, Value};

use super::{Component, ComponentError, ComponentResult};

// ============================================================================
// Constants for tessellation
// ============================================================================

/// Default maximum deviation for adaptive curve tessellation.
const DEFAULT_MAX_DEVIATION: f64 = 0.01;

/// Default maximum number of segments for adaptive curve tessellation.
const DEFAULT_MAX_SEGMENTS: usize = 64;

// ============================================================================
// Helper functions for conversion between [f64; 3] and geom types
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
#[allow(dead_code)]
fn to_geom_vec(v: [f64; 3]) -> GeomVec3 {
    GeomVec3::new(v[0], v[1], v[2])
}

/// Tessellates a curve using the geom adaptive tessellator and returns points as arrays.
fn tessellate_curve_to_points<C: Curve3>(
    curve: &C,
    max_deviation: f64,
    max_segments: usize,
) -> Vec<[f64; 3]> {
    let options = CurveTessellationOptions::new(max_deviation, max_segments);
    let geom_points = tessellate_curve_adaptive_points(curve, options);
    geom_points.into_iter().map(from_geom_point).collect()
}

/// Creates default tessellation options for curve primitives.
#[inline]
fn default_curve_tessellation_options() -> (f64, usize) {
    (DEFAULT_MAX_DEVIATION, DEFAULT_MAX_SEGMENTS)
}

/// Generates a uniform knot vector for a B-spline.
///
/// For a B-spline of degree `degree` with `n` control points,
/// the knot vector has `n + degree + 1` elements.
/// This generates a clamped (open) uniform knot vector.
fn generate_uniform_knots(num_points: usize, degree: usize) -> Vec<f64> {
    let n = num_points;
    let k = n + degree + 1;
    let mut knots = Vec::with_capacity(k);

    // Clamped knot vector: repeated values at start and end
    for _i in 0..=degree {
        knots.push(0.0);
    }

    // Interior uniform knots
    let interior_knots = n - degree;
    if interior_knots > 1 {
        for i in 1..interior_knots {
            knots.push(i as f64 / interior_knots as f64);
        }
    }

    // Clamped end
    for _i in 0..=degree {
        knots.push(1.0);
    }

    knots
}

const PIN_OUTPUT_CURVE: &str = "C";
const PIN_OUTPUT_CURVE_A: &str = "A";
const PIN_OUTPUT_CURVE_B: &str = "B";
const PIN_OUTPUT_CIRCLES: &str = "C";
const PIN_OUTPUT_LENGTH: &str = "L";
const PIN_OUTPUT_DOMAIN: &str = "D";
const PIN_OUTPUT_TWEEN: &str = "T";
const PIN_OUTPUT_MATCH: &str = "M";
const PIN_OUTPUT_GEOS: &str = "G";
const PIN_OUTPUT_U: &str = "U";
const PIN_OUTPUT_V: &str = "V";
const PIN_OUTPUT_SUBCURVES: &str = "S";
const PIN_OUTPUT_KNOTS: &str = "K";

/// Beschikbare componentvarianten binnen de curve-spline module.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    CircleFitObsolete,
    TweenCurve,
    BlendCurvePt,
    NurbsCurvePwk,
    IsoCurve,
    Catenary,
    MatchCurve,
    Interpolate,
    BezierSpan,
    SwingArc,
    SubCurve,
    InterpolateT,
    BlendCurve,
    KinkyCurve,
    Polyarc,
    Polyline,
    PolyArc,
    CatenaryEx,
    KnotVector,
    Geodesic,
    ConnectCurves,
    NurbsCurve,
    TangentCurve,
    CurveOnSurface,
}

/// Registratiegegevens voor de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Volledige lijst met componentregistraties.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{0a80e903-e15b-4992-9675-19b2c488e853}"],
        names: &["Circle Fit [OBSOLETE]", "CFit"],
        kind: ComponentKind::CircleFitObsolete,
    },
    Registration {
        guids: &["{139619d2-8b18-47b6-b3b9-bf4fec0d6eb1}"],
        names: &["Tween Curve", "TweenCrv"],
        kind: ComponentKind::TweenCurve,
    },
    Registration {
        guids: &["{14cf43b6-5eb9-460f-899c-bdece732213a}"],
        names: &["Blend Curve Pt", "BlendCPt"],
        kind: ComponentKind::BlendCurvePt,
    },
    Registration {
        guids: &["{1f8e1ff7-8278-4421-b39d-350e71d85d37}"],
        names: &["Nurbs Curve PWK", "NurbCrv"],
        kind: ComponentKind::NurbsCurvePwk,
    },
    Registration {
        guids: &["{21ca41ee-bc18-4ac8-ba20-713e7edf541e}"],
        names: &["Iso Curve", "Iso"],
        kind: ComponentKind::IsoCurve,
    },
    Registration {
        guids: &["{275671d4-3e87-40bd-8aff-8e6a5fdbb892}"],
        names: &["Catenary", "Cat"],
        kind: ComponentKind::Catenary,
    },
    Registration {
        guids: &["{282bf4eb-668a-4a2c-81af-2432ac863ddd}"],
        names: &["Match Curve", "MatchCrv"],
        kind: ComponentKind::MatchCurve,
    },
    Registration {
        guids: &["{2b2a4145-3dff-41d4-a8de-1ea9d29eef33}"],
        names: &["Interpolate", "IntCrv"],
        kind: ComponentKind::Interpolate,
    },
    Registration {
        guids: &["{30ce59ce-22a1-49ee-9e21-e6d16b3684a8}"],
        names: &["Bezier Span", "BzSpan"],
        kind: ComponentKind::BezierSpan,
    },
    Registration {
        guids: &["{3edc4fbd-24c6-43de-aaa8-5bdf0704373d}"],
        names: &["Swing Arc", "Swing"],
        kind: ComponentKind::SwingArc,
    },
    Registration {
        guids: &["{429cbba9-55ee-4e84-98ea-876c44db879a}"],
        names: &["Sub Curve", "SubCrv"],
        kind: ComponentKind::SubCurve,
    },
    Registration {
        guids: &[
            "{50870118-be51-4872-ab3c-410d79f2356e}",
            "{75eb156d-d023-42f9-a85e-2f2456b8bcce}",
            "{e8e00fbb-9710-4cfa-a60f-2aae50b79d06}",
        ],
        names: &["Interpolate (t)", "IntCrv(t)"],
        kind: ComponentKind::InterpolateT,
    },
    Registration {
        guids: &["{5909dbcb-4950-4ce4-9433-7cf9e62ee011}"],
        names: &["Blend Curve", "BlendC"],
        kind: ComponentKind::BlendCurve,
    },
    Registration {
        guids: &["{6f0993e8-5f2f-4fc0-bd73-b84bc240e78e}"],
        names: &["Kinky Curve", "KinkCrv"],
        kind: ComponentKind::KinkyCurve,
    },
    Registration {
        guids: &["{7159ef59-e4ef-44b8-8cb2-91231e278292}"],
        names: &["PolyArc", "PArc"],
        kind: ComponentKind::Polyarc,
    },
    Registration {
        guids: &["{71b5b089-500a-4ea6-81c5-2f960441a0e8}"],
        names: &["PolyLine", "PLine"],
        kind: ComponentKind::Polyline,
    },
    Registration {
        guids: &["{769f9064-17f5-4c4a-921f-c3a0ee05ba3a}"],
        names: &["Catenary Ex", "CatEx"],
        kind: ComponentKind::CatenaryEx,
    },
    Registration {
        guids: &["{846470bd-4918-4d00-9388-7e022b2cba73}"],
        names: &["Knot Vector", "Knots"],
        kind: ComponentKind::KnotVector,
    },
    Registration {
        guids: &["{a5e4f966-417e-465d-afa9-f6607afea056}"],
        names: &["Poly Arc", "PArc"],
        kind: ComponentKind::PolyArc,
    },
    Registration {
        guids: &["{ce5963b4-1cea-4f71-acd2-a3c28ab85662}"],
        names: &["Geodesic"],
        kind: ComponentKind::Geodesic,
    },
    Registration {
        guids: &["{d0a1b843-873d-4d1d-965c-b5423b35f327}"],
        names: &["Connect Curves", "Connect"],
        kind: ComponentKind::ConnectCurves,
    },
    Registration {
        guids: &["{d1d57181-d594-41e8-8efb-041e29f8a5ca}"],
        names: &["Iso Curve", "Iso"],
        kind: ComponentKind::IsoCurve,
    },
    Registration {
        guids: &["{dde71aef-d6ed-40a6-af98-6b0673983c82}"],
        names: &["Nurbs Curve", "Nurbs"],
        kind: ComponentKind::NurbsCurve,
    },
    Registration {
        guids: &["{f5ea9d41-f062-487e-8dbf-7666ca53fbcd}"],
        names: &["Interpolate", "IntCrv"],
        kind: ComponentKind::Interpolate,
    },
    Registration {
        guids: &["{f73498c5-178b-4e09-ad61-73d172fa6e56}"],
        names: &["Tangent Curve", "TanCurve"],
        kind: ComponentKind::TangentCurve,
    },
    Registration {
        guids: &["{ffe2dbed-9b5d-4f91-8fe3-10c8961ac2f8}"],
        names: &["Curve On Surface", "CrvSrf"],
        kind: ComponentKind::CurveOnSurface,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::CircleFitObsolete => evaluate_circle_fit(inputs),
            Self::TweenCurve => evaluate_tween_curve(inputs),
            Self::BlendCurvePt => evaluate_blend_curve_point(inputs),
            Self::NurbsCurvePwk => evaluate_nurbs(inputs, "Nurbs Curve PWK"),
            Self::IsoCurve => evaluate_iso_curve(inputs),
            Self::Catenary => evaluate_catenary(inputs, false),
            Self::MatchCurve => evaluate_match_curve(inputs),
            Self::Interpolate => evaluate_interpolate(inputs, None),
            Self::BezierSpan => evaluate_bezier_span(inputs),
            Self::SwingArc => evaluate_swing_arc(inputs),
            Self::SubCurve => evaluate_sub_curve(inputs),
            Self::InterpolateT => evaluate_interpolate_t(inputs),
            Self::BlendCurve => evaluate_blend_curve(inputs),
            Self::KinkyCurve => evaluate_kinky_curve(inputs),
            Self::Polyarc => evaluate_poly_arc(inputs, "PolyArc"),
            Self::Polyline => evaluate_polyline(inputs),
            Self::PolyArc => evaluate_poly_arc(inputs, "Poly Arc"),
            Self::CatenaryEx => evaluate_catenary(inputs, true),
            Self::KnotVector => evaluate_knot_vector(inputs),
            Self::Geodesic => evaluate_geodesic(inputs),
            Self::ConnectCurves => evaluate_connect_curves(inputs),
            Self::NurbsCurve => evaluate_nurbs(inputs, "Nurbs Curve"),
            Self::TangentCurve => evaluate_tangent_curve(inputs),
            Self::CurveOnSurface => evaluate_curve_on_surface(inputs),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::CircleFitObsolete => "Circle Fit [OBSOLETE]",
            Self::TweenCurve => "Tween Curve",
            Self::BlendCurvePt => "Blend Curve Pt",
            Self::NurbsCurvePwk => "Nurbs Curve PWK",
            Self::IsoCurve => "Iso Curve",
            Self::Catenary => "Catenary",
            Self::MatchCurve => "Match Curve",
            Self::Interpolate => "Interpolate",
            Self::BezierSpan => "Bezier Span",
            Self::SwingArc => "Swing Arc",
            Self::SubCurve => "Sub Curve",
            Self::InterpolateT => "Interpolate (t)",
            Self::BlendCurve => "Blend Curve",
            Self::KinkyCurve => "Kinky Curve",
            Self::Polyarc => "PolyArc",
            Self::Polyline => "PolyLine",
            Self::PolyArc => "Poly Arc",
            Self::CatenaryEx => "Catenary Ex",
            Self::KnotVector => "Knot Vector",
            Self::Geodesic => "Geodesic",
            Self::ConnectCurves => "Connect Curves",
            Self::NurbsCurve => "Nurbs Curve",
            Self::TangentCurve => "Tangent Curve",
            Self::CurveOnSurface => "Curve On Surface",
        }
    }
}

fn evaluate_circle_fit(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Circle Fit vereist twee basis-cirkels en een straal",
        ));
    }

    let center_a = coerce_point(inputs.get(0), "Circle Fit")?;
    let center_b = coerce_point(inputs.get(1), "Circle Fit")?;
    let radius = coerce_positive_number(inputs.get(2), "Circle Fit radius")?;

    let circle_a = approximate_circle(center_a, radius, 32);
    let circle_b = approximate_circle(center_b, radius, 32);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CURVE_A.to_owned(), Value::List(circle_a));
    outputs.insert(PIN_OUTPUT_CURVE_B.to_owned(), Value::List(circle_b));
    Ok(outputs)
}

fn evaluate_tween_curve(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Tween Curve vereist twee curves en een factor",
        ));
    }

    let curve_a = coerce_polyline(inputs.get(0), "Tween Curve")?;
    let curve_b = coerce_polyline(inputs.get(1), "Tween Curve")?;
    let factor = coerce_number(inputs.get(2), "Tween Curve factor")?.clamp(0.0, 1.0);

    let sample_count = curve_a.len().max(curve_b.len()).max(2);
    let samples_a = resample_polyline(&curve_a, sample_count);
    let samples_b = resample_polyline(&curve_b, sample_count);

    let tween_points: Vec<Value> = samples_a
        .iter()
        .zip(samples_b.iter())
        .map(|(a, b)| {
            let point = lerp_point(*a, *b, factor);
            Value::Point(point)
        })
        .collect();

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_TWEEN.to_owned(), Value::List(tween_points));
    Ok(outputs)
}

fn evaluate_blend_curve_point(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Blend Curve Pt vereist twee curves en een punt",
        ));
    }

    let curve_a = coerce_polyline(inputs.get(0), "Blend Curve Pt")?;
    let curve_b = coerce_polyline(inputs.get(1), "Blend Curve Pt")?;
    let point = coerce_point(inputs.get(2), "Blend Curve Pt")?;

    let mut points = Vec::new();
    points.extend(curve_a.first().copied());
    points.push(point);
    points.extend(curve_b.last().copied());

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_CURVE_B.to_owned(),
        Value::List(points.into_iter().map(Value::Point).collect()),
    );
    Ok(outputs)
}

/// Evaluates the NURBS Curve / Nurbs Curve PWK component.
///
/// Creates a B-spline curve through control points using `geom::NurbsCurve3`.
/// The curve is tessellated adaptively based on curvature.
///
/// # Inputs
/// - `inputs[0]`: Control points list
/// - `inputs[1]`: (Optional) Degree (defaults to 3 for cubic, clamped to valid range)
/// - `inputs[2]`: (Optional) Closed flag (defaults to false)
/// - `inputs[3]`: (Optional) Weights list (for rational NURBS)
///
/// # Outputs
/// - `C`: Curve as a list of points (tessellated polyline)
/// - `L`: Length of the curve
/// - `D`: Domain of the curve
fn evaluate_nurbs(inputs: &[Value], context: &str) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(format!(
            "{} vereist ten minste een puntenlijst",
            context
        )));
    }

    let points = coerce_point_list(inputs.get(0), context)?;
    if points.len() < 2 {
        return Err(ComponentError::new(format!(
            "{} vereist minstens twee punten",
            context
        )));
    }

    // Parse optional degree (default 3 = cubic, must be >= 1 and < num_points)
    let requested_degree = inputs
        .get(1)
        .and_then(|v| v.expect_number().ok())
        .map(|d| d as usize)
        .unwrap_or(3);

    // Clamp degree to valid range
    let max_degree = points.len() - 1;
    let degree = requested_degree.clamp(1, max_degree);

    // Parse optional closed flag
    let closed = inputs
        .get(2)
        .and_then(|v| v.expect_boolean().ok())
        .unwrap_or(false);

    // Parse optional weights
    let weights: Option<Vec<f64>> = inputs.get(3).and_then(|v| {
        if let Value::List(list) = v {
            let w: Result<Vec<f64>, _> = list
                .iter()
                .map(|val| val.expect_number())
                .collect();
            w.ok().filter(|ws| ws.len() == points.len())
        } else if let Ok(single) = v.expect_number() {
            // Single weight broadcasts to all points
            Some(vec![single; points.len()])
        } else {
            None
        }
    });

    // Convert points to geom types
    let geom_points: Vec<GeomPoint3> = points.iter().map(|p| to_geom_point(*p)).collect();

    // Handle closed curves by wrapping control points
    let (final_points, final_weights) = if closed {
        // For a closed B-spline, we wrap the first `degree` control points
        let mut wrapped = geom_points.clone();
        for i in 0..degree {
            wrapped.push(geom_points[i]);
        }
        let wrapped_weights = weights.map(|w| {
            let mut ww = w.clone();
            for i in 0..degree {
                ww.push(w[i]);
            }
            ww
        });
        (wrapped, wrapped_weights)
    } else {
        (geom_points, weights)
    };

    // Generate clamped uniform knot vector
    let knots = generate_uniform_knots(final_points.len(), degree);

    // Build the NURBS curve
    let nurbs_result = NurbsCurve3::new(
        degree,
        final_points,
        knots,
        final_weights,
    );

    let tessellated_points = match nurbs_result {
        Ok(nurbs) => {
            let (max_deviation, max_segments) = default_curve_tessellation_options();
            tessellate_curve_to_points(&nurbs, max_deviation, max_segments)
        }
        Err(_) => {
            // Fallback: return the original control points as a simple polyline
            points.clone()
        }
    };

    // For closed curves, ensure the output closes properly
    let final_output = if closed && !tessellated_points.is_empty() {
        let mut result = tessellated_points;
        if let Some(first) = result.first().copied() {
            if result.last() != Some(&first) {
                result.push(first);
            }
        }
        result
    } else {
        tessellated_points
    };

    build_curve_outputs(final_output, 0.0, 1.0)
}

fn evaluate_iso_curve(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Iso Curve vereist een oppervlak"));
    }

    let (vertices, _) = match inputs.get(0) {
        Some(Value::Surface { vertices, faces: _ }) => (vertices.clone(), ()),
        Some(Value::List(values)) if !values.is_empty() => {
            let mut points = Vec::new();
            for value in values {
                points.push(coerce_point(Some(value), "Iso Curve")?);
            }
            (points, ())
        }
        Some(other) => {
            return Err(ComponentError::new(format!(
                "Iso Curve verwacht een oppervlak, kreeg {}",
                other.kind()
            )));
        }
        None => unreachable!(),
    };

    let uv = inputs
        .get(1)
        .map(|value| coerce_uv(value, "Iso Curve"))
        .transpose()?;

    let bbox = bounding_box(&vertices);
    let (u, v) = uv.unwrap_or((0.5, 0.5));

    let u_curve = vec![
        Value::Point([
            lerp(bbox.min[0], bbox.max[0], u),
            bbox.min[1],
            lerp(bbox.min[2], bbox.max[2], v),
        ]),
        Value::Point([
            lerp(bbox.min[0], bbox.max[0], u),
            bbox.max[1],
            lerp(bbox.min[2], bbox.max[2], v),
        ]),
    ];
    let v_curve = vec![
        Value::Point([
            bbox.min[0],
            lerp(bbox.min[1], bbox.max[1], v),
            lerp(bbox.min[2], bbox.max[2], u),
        ]),
        Value::Point([
            bbox.max[0],
            lerp(bbox.min[1], bbox.max[1], v),
            lerp(bbox.min[2], bbox.max[2], u),
        ]),
    ];

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_U.to_owned(), Value::List(u_curve));
    outputs.insert(PIN_OUTPUT_V.to_owned(), Value::List(v_curve));
    Ok(outputs)
}

fn evaluate_catenary(inputs: &[Value], extended: bool) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Catenary vereist begin- en eindpunten"));
    }

    let start = coerce_point(inputs.get(0), "Catenary")?;
    let end = coerce_point(inputs.get(1), "Catenary")?;
    let length = inputs
        .get(2)
        .and_then(|value| value.expect_number().ok())
        .unwrap_or_else(|| distance(start, end));
    let gravity = inputs
        .get(3)
        .and_then(|value| value.expect_number().ok())
        .unwrap_or(9.81);

    let sag = ((length - distance(start, end)).max(0.0)) * 0.25;
    let mid = midpoint(start, end);
    let sag_point = [mid[0], mid[1], mid[2] - sag * (gravity / 9.81)];

    let curve = vec![
        Value::Point(start),
        Value::Point(sag_point),
        Value::Point(end),
    ];

    let mut outputs = BTreeMap::new();
    if extended {
        outputs.insert(PIN_OUTPUT_CURVE.to_owned(), Value::List(curve.clone()));
        outputs.insert(
            PIN_OUTPUT_SUBCURVES.to_owned(),
            Value::List(vec![Value::List(curve)]),
        );
    } else {
        outputs.insert(PIN_OUTPUT_CURVE.to_owned(), Value::List(curve));
    }
    Ok(outputs)
}

fn evaluate_match_curve(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Match Curve vereist twee curves"));
    }

    let a = coerce_polyline(inputs.get(0), "Match Curve")?;
    let b = coerce_polyline(inputs.get(1), "Match Curve")?;

    let mut result = Vec::new();
    result.extend(a.iter().copied());
    result.extend(b.iter().copied());

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_MATCH.to_owned(),
        Value::List(result.into_iter().map(Value::Point).collect()),
    );
    Ok(outputs)
}

/// Evaluates the Interpolate component.
///
/// Creates a smooth curve that interpolates exactly through the given points using
/// `geom::NurbsCurve3::interpolate_through_points()`. The curve is tessellated
/// adaptively based on curvature.
///
/// # Inputs
/// - `inputs[0]`: Control points list (the curve passes through these)
/// - `inputs[1]`: (Optional) Degree (1=linear, 2=quadratic, 3=cubic; defaults to 3)
/// - `inputs[2]`: (Optional) Closed flag (defaults to false)
///
/// # Outputs
/// - `C`: Curve as a list of points (tessellated polyline)
/// - `L`: Length of the curve
/// - `D`: Domain of the curve
fn evaluate_interpolate(inputs: &[Value], domain_override: Option<(f64, f64)>) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Interpolate vereist een puntenlijst"));
    }

    let points = coerce_point_list(inputs.get(0), "Interpolate")?;
    if points.len() < 2 {
        return Err(ComponentError::new(
            "Interpolate vereist minstens twee punten",
        ));
    }

    // Parse optional degree (default 3 = cubic, must be >= 1)
    let requested_degree = inputs
        .get(1)
        .and_then(|v| v.expect_number().ok())
        .map(|d| (d as usize).max(1))
        .unwrap_or(3);

    let closed = inputs
        .get(2)
        .and_then(|v| v.expect_boolean().ok())
        .unwrap_or(false);

    // Convert points to geom types
    let geom_points: Vec<GeomPoint3> = points.iter().map(|p| to_geom_point(*p)).collect();

    // Build the interpolating NURBS curve using geom
    let nurbs_result = NurbsCurve3::interpolate_through_points(&geom_points, requested_degree, closed);

    let tessellated_points = match nurbs_result {
        Ok(nurbs) => {
            // Use adaptive tessellation for smooth output
            let (max_deviation, max_segments) = default_curve_tessellation_options();
            tessellate_curve_to_points(&nurbs, max_deviation, max_segments)
        }
        Err(_) => {
            // Fallback: return the original points as a simple polyline
            points.clone()
        }
    };

    // For closed curves, ensure the output closes properly
    let final_result = if closed && !tessellated_points.is_empty() {
        let mut result = tessellated_points;
        if let Some(first) = result.first().copied() {
            if result.last() != Some(&first) {
                result.push(first);
            }
        }
        result
    } else {
        tessellated_points
    };

    let (start, end) = domain_override.unwrap_or((0.0, 1.0));
    build_curve_outputs(final_result, start, end)
}

/// Evaluates the Bezier Span component.
///
/// Creates a cubic Bezier curve from start and end points with tangent vectors
/// using `geom::CubicBezier3`. The curve is tessellated adaptively.
///
/// # Inputs
/// - `inputs[0]`: Start point
/// - `inputs[1]`: Start tangent vector
/// - `inputs[2]`: End point
/// - `inputs[3]`: End tangent vector
///
/// # Outputs
/// - `C`: Curve as a list of points (tessellated polyline)
/// - `L`: Length of the curve
/// - `D`: Domain of the curve
fn evaluate_bezier_span(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 4 {
        return Err(ComponentError::new(
            "Bezier Span vereist begin- en eindpunten plus raakvectoren",
        ));
    }

    let start = coerce_point(inputs.get(0), "Bezier Span")?;
    let tangent_start = coerce_vector(inputs.get(1), "Bezier Span At")?;
    let end = coerce_point(inputs.get(2), "Bezier Span")?;
    let tangent_end = coerce_vector(inputs.get(3), "Bezier Span Bt")?;

    // Convert tangent vectors to control points.
    // For a cubic Bezier curve with endpoint tangents:
    // P1 = P0 + (tangent_start / 3)
    // P2 = P3 - (tangent_end / 3)
    let control1 = add_vector(start, scale_vector(tangent_start, 1.0 / 3.0));
    let control2 = subtract(end, scale_vector(tangent_end, 1.0 / 3.0));

    // Build geom::CubicBezier3
    let bezier = CubicBezier3::new(
        to_geom_point(start),
        to_geom_point(control1),
        to_geom_point(control2),
        to_geom_point(end),
    );

    // Tessellate adaptively
    let (max_deviation, max_segments) = default_curve_tessellation_options();
    let samples = tessellate_curve_to_points(&bezier, max_deviation, max_segments);

    build_curve_outputs(samples, 0.0, 1.0)
}

fn evaluate_swing_arc(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Swing Arc vereist minimaal één centrum",
        ));
    }

    let centers = coerce_point_list(inputs.get(0), "Swing Arc")?;
    if centers.len() < 2 {
        return Err(ComponentError::new(
            "Swing Arc vereist minstens twee centra",
        ));
    }

    let radius = inputs
        .get(2)
        .and_then(|value| value.expect_number().ok())
        .unwrap_or(1.0)
        .abs();

    let curve_a: Vec<Value> = centers.iter().copied().map(Value::Point).collect();
    let curve_b: Vec<Value> = centers.iter().rev().copied().map(Value::Point).collect();

    let circles: Vec<Value> = centers
        .iter()
        .map(|center| Value::List(approximate_circle(*center, radius, 16)))
        .collect();

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CURVE_A.to_owned(), Value::List(curve_a));
    outputs.insert(PIN_OUTPUT_CURVE_B.to_owned(), Value::List(curve_b));
    outputs.insert(PIN_OUTPUT_CIRCLES.to_owned(), Value::List(circles));
    Ok(outputs)
}

fn evaluate_sub_curve(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Sub Curve vereist een curve en een domein",
        ));
    }

    let curve = coerce_polyline(inputs.get(0), "Sub Curve")?;
    let domain = coerce_domain(inputs.get(1), "Sub Curve")?;

    let samples = sample_polyline_domain(&curve, domain.start, domain.end, 16);
    build_curve_outputs(samples, domain.start, domain.end)
}

fn evaluate_interpolate_t(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Interpolate (t) vereist punten"));
    }

    let start = inputs
        .get(1)
        .and_then(|value| value.expect_number().ok())
        .unwrap_or(0.0);
    let end = inputs
        .get(2)
        .and_then(|value| value.expect_number().ok())
        .unwrap_or(1.0);

    evaluate_interpolate(inputs, Some((start, end)))
}

fn evaluate_blend_curve(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Blend Curve vereist twee curves"));
    }

    let a = coerce_polyline(inputs.get(0), "Blend Curve")?;
    let b = coerce_polyline(inputs.get(1), "Blend Curve")?;

    let factor_a = inputs
        .get(2)
        .and_then(|value| value.expect_number().ok())
        .unwrap_or(0.5)
        .clamp(0.0, 1.0);
    let factor_b = inputs
        .get(3)
        .and_then(|value| value.expect_number().ok())
        .unwrap_or(0.5)
        .clamp(0.0, 1.0);

    let count = a.len().max(b.len()).max(2);
    let samples_a = resample_polyline(&a, count);
    let samples_b = resample_polyline(&b, count);

    let blended: Vec<Value> = samples_a
        .iter()
        .zip(samples_b.iter())
        .map(|(pa, pb)| {
            let blended = lerp_point(*pa, *pb, 0.5 * (factor_a + factor_b));
            Value::Point(blended)
        })
        .collect();

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CURVE_B.to_owned(), Value::List(blended));
    Ok(outputs)
}

/// Evaluates the Kinky Curve component.
///
/// Creates a smoothed curve through control points using a single Chaikin refinement
/// iteration. The result is a "kinky" curve that loosely follows the control polygon.
///
/// # Inputs
/// - `inputs[0]`: Control points list
///
/// # Outputs
/// - `C`: Curve as a list of points (refined polyline)
/// - `L`: Length of the curve
/// - `D`: Domain of the curve
fn evaluate_kinky_curve(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Kinky Curve vereist punten"));
    }

    let points = coerce_point_list(inputs.get(0), "Kinky Curve")?;
    if points.len() < 2 {
        return Err(ComponentError::new(
            "Kinky Curve vereist minstens twee punten",
        ));
    }

    // Use a single Chaikin refinement for the "kinky" smoothing effect.
    // This is intentionally different from full B-spline subdivision.
    let refined = chaikin_refine(&points, 1, false);
    build_curve_outputs(refined, 0.0, 1.0)
}

fn evaluate_poly_arc(inputs: &[Value], context: &str) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(format!(
            "{} vereist een puntenlijst",
            context
        )));
    }

    let points = coerce_point_list(inputs.get(0), context)?;
    if points.len() < 2 {
        return Err(ComponentError::new(format!(
            "{} vereist minstens twee punten",
            context
        )));
    }

    build_curve_outputs(points, 0.0, 1.0)
}

/// Evaluates the Polyline component.
///
/// Creates a polyline through the given points using `geom::Polyline3`.
/// Optionally closes the polyline by connecting the last point to the first.
///
/// # Inputs
/// - `inputs[0]`: Points list
/// - `inputs[1]`: (Optional) Closed flag (defaults to false)
///
/// # Outputs
/// - `C`: Curve as a list of points
/// - `L`: Length of the curve
/// - `D`: Domain of the curve
fn evaluate_polyline(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("PolyLine vereist een puntenlijst"));
    }

    let points = coerce_point_list(inputs.get(0), "PolyLine")?;
    if points.len() < 2 {
        return Err(ComponentError::new("PolyLine vereist minstens twee punten"));
    }

    let closed = inputs
        .get(1)
        .and_then(|value| value.expect_boolean().ok())
        .unwrap_or(false);

    // Build geom::Polyline3 for validation and proper arc-length parameterization.
    // IMPORTANT: We preserve the original input points exactly - polylines should NOT
    // be resampled via adaptive tessellation, as that would remove/shift vertices and
    // change the curve shape (behavioral regression).
    let geom_points: Vec<GeomPoint3> = points.iter().map(|p| to_geom_point(*p)).collect();
    match Polyline3::new(geom_points, closed) {
        Ok(polyline) => {
            // Use the original points directly from the polyline (preserves input vertices exactly).
            // For closed polylines, Polyline3::new removes any duplicate closing point,
            // so we need to add it back for the output representation.
            let mut output_points: Vec<[f64; 3]> =
                polyline.points().iter().map(|p| from_geom_point(*p)).collect();

            // For closed polylines, add the closing point (first point repeated at end)
            if closed && !output_points.is_empty() {
                if let Some(first) = output_points.first().copied() {
                    // Only add if not already closed (should always be true after Polyline3::new)
                    if output_points.last() != Some(&first) {
                        output_points.push(first);
                    }
                }
            }

            build_curve_outputs(output_points, 0.0, 1.0)
        }
        Err(_) => {
            // Fallback: return raw points if geom construction fails
            let mut output_points = points;
            if closed {
                if let Some(first) = output_points.first().copied() {
                    if output_points.last() != Some(&first) {
                        output_points.push(first);
                    }
                }
            }
            build_curve_outputs(output_points, 0.0, 1.0)
        }
    }
}

fn evaluate_knot_vector(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Knot Vector vereist graad, punten en periodiciteit",
        ));
    }

    let count = coerce_number(inputs.get(0), "Knot Vector count")? as usize;
    let degree = coerce_number(inputs.get(1), "Knot Vector degree")? as usize;
    let periodic = inputs
        .get(2)
        .and_then(|value| value.expect_boolean().ok())
        .unwrap_or(false);

    if degree == 0 || count == 0 {
        return Err(ComponentError::new(
            "Knot Vector vereist positieve aantallen",
        ));
    }

    let knot_count = count + degree + if periodic { 0 } else { 1 };
    let mut knots = Vec::with_capacity(knot_count);
    for idx in 0..knot_count {
        knots.push(Value::Number(idx as f64 / (knot_count.max(1) as f64)));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_KNOTS.to_owned(), Value::List(knots));
    Ok(outputs)
}

fn evaluate_geodesic(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Geodesic vereist een oppervlak en twee punten",
        ));
    }

    let start = coerce_point(inputs.get(1), "Geodesic start")?;
    let end = coerce_point(inputs.get(2), "Geodesic end")?;

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_GEOS.to_owned(),
        Value::List(vec![Value::Point(start), Value::Point(end)]),
    );
    Ok(outputs)
}

fn evaluate_connect_curves(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Connect Curves vereist curves"));
    }

    let curves = coerce_curve_collection(inputs.get(0), "Connect Curves")?;
    let closed = inputs
        .get(2)
        .and_then(|value| value.expect_boolean().ok())
        .unwrap_or(false);

    let mut points = Vec::new();
    for curve in curves {
        points.extend(curve);
    }
    if closed {
        if let Some(first) = points.first().copied() {
            points.push(first);
        }
    }

    build_curve_outputs(points, 0.0, 1.0)
}

fn evaluate_tangent_curve(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Tangent Curve vereist punten en raakvectoren",
        ));
    }

    let points = coerce_point_list(inputs.get(0), "Tangent Curve")?;
    let tangents = coerce_vector_list(inputs.get(1), "Tangent Curve")?;
    let blend = inputs
        .get(2)
        .and_then(|value| value.expect_number().ok())
        .unwrap_or(0.5);

    let adjusted: Vec<[f64; 3]> = points
        .iter()
        .zip(tangents.iter().cycle())
        .map(|(point, tangent)| add_vector(*point, scale_vector(*tangent, blend * 0.25)))
        .collect();

    build_curve_outputs(adjusted, 0.0, 1.0)
}

fn evaluate_curve_on_surface(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Curve On Surface vereist oppervlak en uv-coördinaten",
        ));
    }

    let surface_vertices = match inputs.get(0) {
        Some(Value::Surface { vertices, faces: _ }) => vertices.clone(),
        Some(Value::List(values)) => {
            let mut verts = Vec::new();
            for value in values {
                verts.push(coerce_point(Some(value), "Curve On Surface")?);
            }
            verts
        }
        Some(other) => {
            return Err(ComponentError::new(format!(
                "Curve On Surface verwacht een oppervlak, kreeg {}",
                other.kind()
            )));
        }
        None => unreachable!(),
    };

    let uv_list = coerce_uv_list(inputs.get(1), "Curve On Surface")?;
    if uv_list.is_empty() {
        return Err(ComponentError::new(
            "Curve On Surface vereist minstens één uv-paar",
        ));
    }

    let bbox = bounding_box(&surface_vertices);
    let mut points = Vec::new();
    for (u, v) in uv_list {
        let point = [
            lerp(bbox.min[0], bbox.max[0], u),
            lerp(bbox.min[1], bbox.max[1], v),
            lerp(bbox.min[2], bbox.max[2], (u + v) * 0.5),
        ];
        points.push(point);
    }

    let closed = inputs
        .get(2)
        .and_then(|value| value.expect_boolean().ok())
        .unwrap_or(false);
    if closed && points.first() != points.last() {
        if let Some(first) = points.first().copied() {
            points.push(first);
        }
    }

    build_curve_outputs(points, 0.0, 1.0)
}

fn build_curve_outputs(points: Vec<[f64; 3]>, start: f64, end: f64) -> ComponentResult {
    if points.len() < 2 {
        return Err(ComponentError::new(
            "Curve evaluatie vereist minstens twee punten",
        ));
    }

    let length = polyline_length(&points);
    let domain = Value::Domain(Domain::One(create_domain1d(start, end)));
    let curve = Value::List(points.into_iter().map(Value::Point).collect());

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CURVE.to_owned(), curve);
    outputs.insert(PIN_OUTPUT_LENGTH.to_owned(), Value::Number(length));
    outputs.insert(PIN_OUTPUT_DOMAIN.to_owned(), domain);
    Ok(outputs)
}

fn coerce_point(value: Option<&Value>, context: &str) -> Result<[f64; 3], ComponentError> {
    let value =
        value.ok_or_else(|| ComponentError::new(format!("{} vereist een punt", context)))?;
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
    let value =
        value.ok_or_else(|| ComponentError::new(format!("{} vereist een vector", context)))?;
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

fn coerce_point_list(
    value: Option<&Value>,
    context: &str,
) -> Result<Vec<[f64; 3]>, ComponentError> {
    let value =
        value.ok_or_else(|| ComponentError::new(format!("{} vereist een puntenlijst", context)))?;
    match value {
        Value::List(values) => {
            let mut points = Vec::new();
            for entry in values {
                points.push(coerce_point(Some(entry), context)?);
            }
            Ok(points)
        }
        Value::Point(point) => Ok(vec![*point]),
        other => Err(ComponentError::new(format!(
            "{} verwacht een lijst met punten, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_vector_list(
    value: Option<&Value>,
    context: &str,
) -> Result<Vec<[f64; 3]>, ComponentError> {
    let value = value.ok_or_else(|| ComponentError::new(format!("{} vereist vectors", context)))?;
    match value {
        Value::List(values) => {
            let mut vectors = Vec::new();
            for entry in values {
                vectors.push(coerce_vector(Some(entry), context)?);
            }
            Ok(vectors)
        }
        Value::Vector(vector) => Ok(vec![*vector]),
        other => Err(ComponentError::new(format!(
            "{} verwacht een lijst met vectoren, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_curve_collection(
    value: Option<&Value>,
    context: &str,
) -> Result<Vec<Vec<[f64; 3]>>, ComponentError> {
    let value = value.ok_or_else(|| ComponentError::new(format!("{} vereist curves", context)))?;
    match value {
        Value::List(values) => {
            let mut curves = Vec::new();
            for entry in values {
                curves.push(coerce_polyline(Some(entry), context)?);
            }
            Ok(curves)
        }
        _ => Ok(vec![coerce_polyline(Some(value), context)?]),
    }
}

fn coerce_polyline(value: Option<&Value>, context: &str) -> Result<Vec<[f64; 3]>, ComponentError> {
    let value =
        value.ok_or_else(|| ComponentError::new(format!("{} vereist een curve", context)))?;
    match value {
        Value::List(values) => {
            let mut points = Vec::new();
            for entry in values {
                points.push(coerce_point(Some(entry), context)?);
            }
            if points.len() < 2 {
                return Err(ComponentError::new(format!(
                    "{} vereist minstens twee punten",
                    context
                )));
            }
            Ok(points)
        }
        Value::CurveLine { p1, p2 } => Ok(vec![*p1, *p2]),
        other => Err(ComponentError::new(format!(
            "{} verwacht een curve, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_domain(value: Option<&Value>, context: &str) -> Result<Domain1D, ComponentError> {
    let value =
        value.ok_or_else(|| ComponentError::new(format!("{} vereist een domein", context)))?;
    match value {
        Value::Domain(Domain::One(domain)) => Ok(domain.clone()),
        Value::List(values) if values.len() >= 2 => {
            let start = coerce_number(values.get(0), context)?;
            let end = coerce_number(values.get(1), context)?;
            Ok(create_domain1d(start, end))
        }
        other => Err(ComponentError::new(format!(
            "{} verwacht een domein, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_uv(value: &Value, context: &str) -> Result<(f64, f64), ComponentError> {
    match value {
        Value::Vector(vector) => Ok((vector[0], vector[1])),
        Value::Point(point) => Ok((point[0], point[1])),
        Value::List(values) if values.len() >= 2 => {
            let u = coerce_number(values.get(0), context)?;
            let v = coerce_number(values.get(1), context)?;
            Ok((u, v))
        }
        other => Err(ComponentError::new(format!(
            "{} verwacht uv-coördinaten, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_uv_list(value: Option<&Value>, context: &str) -> Result<Vec<(f64, f64)>, ComponentError> {
    let value =
        value.ok_or_else(|| ComponentError::new(format!("{} vereist uv-coördinaten", context)))?;
    match value {
        Value::List(values) => {
            let mut coords = Vec::new();
            for entry in values {
                coords.push(coerce_uv(entry, context)?);
            }
            Ok(coords)
        }
        _ => Ok(vec![coerce_uv(value, context)?]),
    }
}

fn coerce_number(value: Option<&Value>, context: &str) -> Result<f64, ComponentError> {
    let value = value
        .ok_or_else(|| ComponentError::new(format!("{} vereist een numerieke waarde", context)))?;
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

fn chaikin_refine(points: &[[f64; 3]], iterations: usize, closed: bool) -> Vec<[f64; 3]> {
    let mut result = points.to_vec();
    for _ in 0..iterations {
        if result.len() < 2 {
            break;
        }
        let mut refined = Vec::with_capacity(result.len() * 2);
        for window in result.windows(2) {
            let a = window[0];
            let b = window[1];
            refined.push(lerp_point(a, b, 0.25));
            refined.push(lerp_point(a, b, 0.75));
        }
        if !closed {
            if let Some(first) = result.first().copied() {
                refined.insert(0, first);
            }
            if let Some(last) = result.last().copied() {
                refined.push(last);
            }
        }
        result = refined;
    }
    result
}

fn resample_polyline(points: &[[f64; 3]], count: usize) -> Vec<[f64; 3]> {
    if count <= 2 || points.len() <= 2 {
        return points.to_vec();
    }
    let total_length = polyline_length(points);
    if total_length == 0.0 {
        return vec![points[0]; count];
    }

    let mut result = Vec::with_capacity(count);
    for i in 0..count {
        let t = i as f64 / (count - 1) as f64;
        result.push(sample_polyline(points, t));
    }
    result
}

fn sample_polyline(points: &[[f64; 3]], t: f64) -> [f64; 3] {
    if points.len() < 2 {
        return points.first().copied().unwrap_or([0.0, 0.0, 0.0]);
    }

    let total = polyline_length(points);
    if total == 0.0 {
        return points[0];
    }
    let mut target = t.clamp(0.0, 1.0) * total;
    let mut previous = points[0];
    for current in &points[1..] {
        let segment = distance(previous, *current);
        if target <= segment {
            let ratio = if segment == 0.0 {
                0.0
            } else {
                target / segment
            };
            return lerp_point(previous, *current, ratio);
        }
        target -= segment;
        previous = *current;
    }
    *points.last().unwrap()
}

fn sample_polyline_domain(
    points: &[[f64; 3]],
    start: f64,
    end: f64,
    segments: usize,
) -> Vec<[f64; 3]> {
    let mut samples = Vec::new();
    for i in 0..=segments {
        let t = if segments == 0 {
            0.0
        } else {
            start + (end - start) * (i as f64 / segments as f64)
        };
        samples.push(sample_polyline(
            points,
            ((t - start) / (end - start)).clamp(0.0, 1.0),
        ));
    }
    samples
}

fn polyline_length(points: &[[f64; 3]]) -> f64 {
    points
        .windows(2)
        .map(|segment| distance(segment[0], segment[1]))
        .sum()
}

fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn midpoint(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (a[2] + b[2]) * 0.5,
    ]
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn lerp_point(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [
        lerp(a[0], b[0], t),
        lerp(a[1], b[1], t),
        lerp(a[2], b[2], t),
    ]
}

fn add_vector(point: [f64; 3], vector: [f64; 3]) -> [f64; 3] {
    [
        point[0] + vector[0],
        point[1] + vector[1],
        point[2] + vector[2],
    ]
}

fn subtract(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale_vector(vector: [f64; 3], scale: f64) -> [f64; 3] {
    [vector[0] * scale, vector[1] * scale, vector[2] * scale]
}

#[allow(dead_code)]
fn cubic_bezier(p0: [f64; 3], p1: [f64; 3], p2: [f64; 3], p3: [f64; 3], t: f64) -> [f64; 3] {
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;
    let uuu = uu * u;
    let ttt = tt * t;

    let mut point = scale_vector(p0, uuu);
    point = add_vector(point, scale_vector(p1, 3.0 * uu * t));
    point = add_vector(point, scale_vector(p2, 3.0 * u * tt));
    add_vector(point, scale_vector(p3, ttt))
}

fn approximate_circle(center: [f64; 3], radius: f64, segments: usize) -> Vec<Value> {
    let mut values = Vec::with_capacity(segments + 1);
    for i in 0..=segments {
        let angle = (i as f64 / segments as f64) * std::f64::consts::TAU;
        values.push(Value::Point([
            center[0] + radius * angle.cos(),
            center[1] + radius * angle.sin(),
            center[2],
        ]));
    }
    values
}

fn create_domain1d(start: f64, end: f64) -> Domain1D {
    let min = start.min(end);
    let max = start.max(end);
    Domain1D {
        start,
        end,
        min,
        max,
        span: max - min,
        length: (end - start).abs(),
        center: (start + end) * 0.5,
    }
}

#[derive(Clone, Debug)]
struct BoundingBox {
    min: [f64; 3],
    max: [f64; 3],
}

fn bounding_box(points: &[[f64; 3]]) -> BoundingBox {
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for point in points {
        for i in 0..3 {
            min[i] = min[i].min(point[i]);
            max[i] = max[i].max(point[i]);
        }
    }
    if !min.iter().all(|value| value.is_finite()) {
        min = [0.0, 0.0, 0.0];
    }
    if !max.iter().all(|value| value.is_finite()) {
        max = [1.0, 1.0, 1.0];
    }
    BoundingBox { min, max }
}