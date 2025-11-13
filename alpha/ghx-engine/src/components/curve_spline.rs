//! Implementaties van Grasshopper "Curve → Spline" componenten.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::{Domain, Domain1D, Value};

use super::{Component, ComponentError, ComponentResult};

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

    let refined = chaikin_refine(&points, 3, false);
    build_curve_outputs(refined, 0.0, 1.0)
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

fn evaluate_interpolate(inputs: &[Value], domain_override: Option<(f32, f32)>) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Interpolate vereist een puntenlijst"));
    }

    let points = coerce_point_list(inputs.get(0), "Interpolate")?;
    if points.len() < 2 {
        return Err(ComponentError::new(
            "Interpolate vereist minstens twee punten",
        ));
    }

    let refined = chaikin_refine(&points, 2, false);
    let (start, end) = domain_override.unwrap_or((0.0, 1.0));
    build_curve_outputs(refined, start, end)
}

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

    let control1 = add_vector(start, scale_vector(tangent_start, 1.0 / 3.0));
    let control2 = subtract(end, scale_vector(tangent_end, 1.0 / 3.0));

    let mut samples = Vec::new();
    for i in 0..=16 {
        let t = i as f32 / 16.0;
        let point = cubic_bezier(start, control1, control2, end, t);
        samples.push(point);
    }

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

fn evaluate_polyline(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("PolyLine vereist een puntenlijst"));
    }

    let mut points = coerce_point_list(inputs.get(0), "PolyLine")?;
    let closed = inputs
        .get(1)
        .and_then(|value| value.expect_boolean().ok())
        .unwrap_or(false);
    if closed && points.first() != points.last() {
        if let Some(first) = points.first().copied() {
            points.push(first);
        }
    }

    build_curve_outputs(points, 0.0, 1.0)
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
        knots.push(Value::Number(idx as f32 / (knot_count.max(1) as f32)));
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

    let adjusted: Vec<[f32; 3]> = points
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

fn build_curve_outputs(points: Vec<[f32; 3]>, start: f32, end: f32) -> ComponentResult {
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

fn coerce_point(value: Option<&Value>, context: &str) -> Result<[f32; 3], ComponentError> {
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

fn coerce_vector(value: Option<&Value>, context: &str) -> Result<[f32; 3], ComponentError> {
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
) -> Result<Vec<[f32; 3]>, ComponentError> {
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
) -> Result<Vec<[f32; 3]>, ComponentError> {
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
) -> Result<Vec<Vec<[f32; 3]>>, ComponentError> {
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

fn coerce_polyline(value: Option<&Value>, context: &str) -> Result<Vec<[f32; 3]>, ComponentError> {
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

fn coerce_uv(value: &Value, context: &str) -> Result<(f32, f32), ComponentError> {
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

fn coerce_uv_list(value: Option<&Value>, context: &str) -> Result<Vec<(f32, f32)>, ComponentError> {
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

fn coerce_number(value: Option<&Value>, context: &str) -> Result<f32, ComponentError> {
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

fn chaikin_refine(points: &[[f32; 3]], iterations: usize, closed: bool) -> Vec<[f32; 3]> {
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

fn resample_polyline(points: &[[f32; 3]], count: usize) -> Vec<[f32; 3]> {
    if count <= 2 || points.len() <= 2 {
        return points.to_vec();
    }
    let total_length = polyline_length(points);
    if total_length == 0.0 {
        return vec![points[0]; count];
    }

    let mut result = Vec::with_capacity(count);
    for i in 0..count {
        let t = i as f32 / (count - 1) as f32;
        result.push(sample_polyline(points, t));
    }
    result
}

fn sample_polyline(points: &[[f32; 3]], t: f32) -> [f32; 3] {
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
    points: &[[f32; 3]],
    start: f32,
    end: f32,
    segments: usize,
) -> Vec<[f32; 3]> {
    let mut samples = Vec::new();
    for i in 0..=segments {
        let t = if segments == 0 {
            0.0
        } else {
            start + (end - start) * (i as f32 / segments as f32)
        };
        samples.push(sample_polyline(
            points,
            ((t - start) / (end - start)).clamp(0.0, 1.0),
        ));
    }
    samples
}

fn polyline_length(points: &[[f32; 3]]) -> f32 {
    points
        .windows(2)
        .map(|segment| distance(segment[0], segment[1]))
        .sum()
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn midpoint(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (a[2] + b[2]) * 0.5,
    ]
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp_point(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        lerp(a[0], b[0], t),
        lerp(a[1], b[1], t),
        lerp(a[2], b[2], t),
    ]
}

fn add_vector(point: [f32; 3], vector: [f32; 3]) -> [f32; 3] {
    [
        point[0] + vector[0],
        point[1] + vector[1],
        point[2] + vector[2],
    ]
}

fn subtract(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale_vector(vector: [f32; 3], scale: f32) -> [f32; 3] {
    [vector[0] * scale, vector[1] * scale, vector[2] * scale]
}

fn cubic_bezier(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3], p3: [f32; 3], t: f32) -> [f32; 3] {
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

fn approximate_circle(center: [f32; 3], radius: f32, segments: usize) -> Vec<Value> {
    let mut values = Vec::with_capacity(segments + 1);
    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        values.push(Value::Point([
            center[0] + radius * angle.cos(),
            center[1] + radius * angle.sin(),
            center[2],
        ]));
    }
    values
}

fn create_domain1d(start: f32, end: f32) -> Domain1D {
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
    min: [f32; 3],
    max: [f32; 3],
}

fn bounding_box(points: &[[f32; 3]]) -> BoundingBox {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
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

#[cfg(test)]
mod tests {
    use super::{
        Component, ComponentKind, PIN_OUTPUT_CURVE, PIN_OUTPUT_DOMAIN, PIN_OUTPUT_KNOTS,
        PIN_OUTPUT_LENGTH,
    };
    use crate::graph::node::MetaMap;
    use crate::graph::value::{Domain, Domain1D, Value};

    #[test]
    fn interpolate_returns_curve_with_domain() {
        let inputs = vec![Value::List(vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Point([1.0, 1.0, 0.0]),
        ])];
        let result = ComponentKind::Interpolate
            .evaluate(&inputs, &MetaMap::new())
            .expect("interpolate succeeds");
        assert!(matches!(result.get(PIN_OUTPUT_CURVE), Some(Value::List(_))));
        match result.get(PIN_OUTPUT_DOMAIN) {
            Some(Value::Domain(Domain::One(Domain1D { start, end, .. }))) => {
                assert!((*start - 0.0).abs() < 1e-9);
                assert!((*end - 1.0).abs() < 1e-9);
            }
            other => panic!("unexpected domain output: {other:?}"),
        }
        if let Some(Value::Number(length)) = result.get(PIN_OUTPUT_LENGTH) {
            assert!(*length > 0.0);
        } else {
            panic!("length output missing");
        }
    }

    #[test]
    fn polyline_closes_when_requested() {
        let inputs = vec![
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([1.0, 0.0, 0.0]),
            ]),
            Value::Boolean(true),
        ];
        let result = ComponentKind::Polyline
            .evaluate(&inputs, &MetaMap::new())
            .expect("polyline succeeds");
        let curve = match result.get(PIN_OUTPUT_CURVE) {
            Some(Value::List(values)) => values,
            other => panic!("unexpected curve output: {other:?}"),
        };
        let first = match curve.first() {
            Some(Value::Point(point)) => *point,
            other => panic!("unexpected first point: {other:?}"),
        };
        let last = match curve.last() {
            Some(Value::Point(point)) => *point,
            other => panic!("unexpected last point: {other:?}"),
        };
        assert!((first[0] - last[0]).abs() < 1e-9);
        assert!((first[1] - last[1]).abs() < 1e-9);
    }

    #[test]
    fn bezier_span_generates_samples() {
        let inputs = vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Vector([1.0, 0.0, 0.0]),
            Value::Point([1.0, 1.0, 0.0]),
            Value::Vector([0.0, 1.0, 0.0]),
        ];
        let result = ComponentKind::BezierSpan
            .evaluate(&inputs, &MetaMap::new())
            .expect("bezier span succeeds");
        let curve = match result.get(PIN_OUTPUT_CURVE) {
            Some(Value::List(values)) => values,
            other => panic!("unexpected curve output: {other:?}"),
        };
        assert!(curve.len() >= 4);
    }

    #[test]
    fn knot_vector_creates_sequence() {
        let inputs = vec![
            Value::Number(3.0),
            Value::Number(2.0),
            Value::Boolean(false),
        ];
        let result = ComponentKind::KnotVector
            .evaluate(&inputs, &MetaMap::new())
            .expect("knot vector succeeds");
        let knots = match result.get(PIN_OUTPUT_KNOTS) {
            Some(Value::List(values)) => values,
            other => panic!("unexpected knot output: {other:?}"),
        };
        assert!(knots.len() > 3);
    }
}
