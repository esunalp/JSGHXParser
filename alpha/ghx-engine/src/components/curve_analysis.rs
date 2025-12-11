//! Implementaties van Grasshopper "Curve → Analysis" componenten.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::{Domain, Domain1D, Value};

use super::{Component, ComponentError, ComponentResult};

const PIN_OUTPUT_START: &str = "S";
const PIN_OUTPUT_END: &str = "E";
const PIN_OUTPUT_DOMAIN: &str = "D";
const PIN_OUTPUT_POINT: &str = "P";
const PIN_OUTPUT_TANGENT: &str = "T";
const PIN_OUTPUT_ANGLE: &str = "A";
const PIN_OUTPUT_LENGTH: &str = "L";
const PIN_OUTPUT_DISTANCE: &str = "D";
const PIN_OUTPUT_CURVE: &str = "C";
const PIN_OUTPUT_POINTS: &str = "P";
const PIN_OUTPUT_WEIGHTS: &str = "W";
const PIN_OUTPUT_KNOTS: &str = "K";
const PIN_OUTPUT_CLOSED: &str = "C";
const PIN_OUTPUT_PERIODIC: &str = "P";
const PIN_OUTPUT_PLANE: &str = "P";
const PIN_OUTPUT_PLANAR: &str = "p";
const PIN_OUTPUT_DEVIATION: &str = "D";
const PIN_OUTPUT_CENTER_VERTEX: &str = "Cv";
const PIN_OUTPUT_CENTER_EDGE: &str = "Ce";
const PIN_OUTPUT_CENTER_AREA: &str = "Ca";
const PIN_OUTPUT_CENTER_SINGLE: &str = "C";
const PIN_OUTPUT_POINT_A: &str = "A";
const PIN_OUTPUT_POINT_B: &str = "B";
const PIN_OUTPUT_PARAMETER: &str = "t";
const PIN_OUTPUT_LENGTH_MINUS: &str = "L-";
const PIN_OUTPUT_LENGTH_PLUS: &str = "L+";
const PIN_OUTPUT_MIDPOINT: &str = "M";
const PIN_OUTPUT_SHORTEST_LENGTH: &str = "Sl";
const PIN_OUTPUT_SHORTEST_DOMAIN: &str = "Sd";
const PIN_OUTPUT_LONGEST_LENGTH: &str = "Ll";
const PIN_OUTPUT_LONGEST_DOMAIN: &str = "Ld";
const PIN_OUTPUT_RADIUS: &str = "R";
const PIN_OUTPUT_RELATIONSHIP: &str = "R";
const PIN_OUTPUT_INDEX: &str = "I";
const PIN_OUTPUT_POINT_PRIME: &str = "P'";
const PIN_OUTPUT_FRAME: &str = "F";
const PIN_OUTPUT_BASE_PLANE: &str = "B";
const PIN_OUTPUT_CURVATURE_VECTOR: &str = "K";
const PIN_OUTPUT_CURVATURE_CENTER: &str = "C";
const PIN_OUTPUT_FIRST_DERIVATIVE: &str = "1";
const PIN_OUTPUT_DERIVATIVES: &str = "d";
const PIN_OUTPUT_SIDE: &str = "S";
const PIN_OUTPUT_LEFT: &str = "L";
const PIN_OUTPUT_RIGHT: &str = "R";
const PIN_OUTPUT_TORSION: &str = "T";
const PIN_OUTPUT_X_INTERVAL: &str = "X";
const PIN_OUTPUT_Y_INTERVAL: &str = "Y";
const PIN_OUTPUT_HIGHEST: &str = "H";
const PIN_OUTPUT_LOWEST: &str = "L";
const PIN_OUTPUT_MIN_PARAMETER: &str = "tMin";
const PIN_OUTPUT_MIN_DEPTH: &str = "dMin";
const PIN_OUTPUT_MAX_PARAMETER: &str = "tMax";
const PIN_OUTPUT_MAX_DEPTH: &str = "dMax";

/// Beschikbare componenten binnen deze module.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    PointInCurves,
    EndPoints,
    CurveDomain,
    EvaluateCurveBasic,
    EvaluateCurveAngle,
    EvaluateCurveLength,
    LengthDomain,
    DeconstructArc,
    Discontinuity,
    CurveClosestPoint,
    Closed,
    ControlPointsDetailed,
    ControlPointsSimple,
    ControlPolygon,
    Planar,
    PolygonCenterDetailed,
    PolygonCenterEdge,
    PolygonCenterSimple,
    PerpFrame,
    EvaluateLength,
    LengthParameter,
    Length,
    CurveMiddle,
    SegmentLengths,
    CurveProximity,
    CurveFrame,
    CurvatureGraph,
    CurveNearestObject,
    ArcCenter,
    PointInCurve,
    CurveDepth,
    Curvature,
    DerivativesFirst,
    CurveSide,
    HorizontalFrame,
    Containment,
    DerivativesList,
    CurveDomainAdjust,
    Torsion,
    DeconstructRectangle,
    Extremes,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Volledige lijst met componentregistraties voor de curve-analysis componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{0b04e8b9-00d7-47a7-95c3-0d51e654fe88}"],
        names: &["Point in Curves", "InCurves"],
        kind: ComponentKind::PointInCurves,
    },
    Registration {
        guids: &["{11bbd48b-bb0a-4f1b-8167-fa297590390d}"],
        names: &["End Points", "CrvEnd"],
        kind: ComponentKind::EndPoints,
    },
    Registration {
        guids: &["{15ac45a8-b190-420a-bd66-e78ed6bcfaa4}"],
        names: &["Curve Domain", "CrvDom Legacy"],
        kind: ComponentKind::CurveDomain,
    },
    Registration {
        guids: &["{23862862-049a-40be-b558-2418aacbd916}"],
        names: &["Deconstruct Arc", "DeArc"],
        kind: ComponentKind::DeconstructArc,
    },
    Registration {
        guids: &["{269eaa85-9997-4d77-a9ba-4c58cb45c9d3}"],
        names: &["Discontinuity", "CrvDiscontinuity"],
        kind: ComponentKind::Discontinuity,
    },
    Registration {
        guids: &["{164d0429-e5f5-4292-aa80-3f88d43cdac2}"],
        names: &["Evaluate Curve", "Eval Curve"],
        kind: ComponentKind::EvaluateCurveBasic,
    },
    Registration {
        guids: &["{fc6979e4-7e91-4508-8e05-37c680779751}"],
        names: &["Evaluate Curve Angle", "Eval Curve Angle"],
        kind: ComponentKind::EvaluateCurveAngle,
    },
    Registration {
        guids: &["{fdf09135-fae5-4e5f-b427-b1f384ca3009}"],
        names: &["Evaluate Curve Length", "Eval Curve Length"],
        kind: ComponentKind::EvaluateCurveLength,
    },
    Registration {
        guids: &["{188edd02-14a9-4828-a521-34995b0d1e4a}"],
        names: &["Length Domain", "LenDom"],
        kind: ComponentKind::LengthDomain,
    },
    Registration {
        guids: &["{2dc44b22-b1dd-460a-a704-6462d6e91096}"],
        names: &["Curve Closest Point", "CrvCP"],
        kind: ComponentKind::CurveClosestPoint,
    },
    Registration {
        guids: &[
            "{323f3245-af49-4489-8677-7a2c73664077}",
            "{f2030fa9-db3f-437e-9b50-5607db6daf87}",
        ],
        names: &["Curve Closed", "CrvClosed"],
        kind: ComponentKind::Closed,
    },
    Registration {
        guids: &["{424eb433-2b3a-4859-beaf-804d8af0afd7}"],
        names: &["Control Points Detailed", "CrvCPnts"],
        kind: ComponentKind::ControlPointsDetailed,
    },
    Registration {
        guids: &["{d7df7658-e02d-4a48-a345-2195a68db4ef}"],
        names: &["Control Points", "CrvCPnts Simple"],
        kind: ComponentKind::ControlPointsSimple,
    },
    Registration {
        guids: &["{66d2a68e-2f1d-43d2-a53b-c6a4d17e627b}"],
        names: &["Control Polygon", "CrvCPoly"],
        kind: ComponentKind::ControlPolygon,
    },
    Registration {
        guids: &["{5816ec9c-f170-4c59-ac44-364401ff84cd}"],
        names: &["Curve Planar", "CrvPlanar"],
        kind: ComponentKind::Planar,
    },
    Registration {
        guids: &["{59e94548-cefd-4774-b3de-48142fc783fb}"],
        names: &["Polygon Center Detailed", "PolyCenter"],
        kind: ComponentKind::PolygonCenterDetailed,
    },
    Registration {
        guids: &["{87e7f480-14dc-4478-b1e6-2b8b035d9edc}"],
        names: &["Polygon Center Edge", "PolyCenter Edge"],
        kind: ComponentKind::PolygonCenterEdge,
    },
    Registration {
        guids: &["{7bd7b551-ca79-4f01-b95a-7e9ab876f24d}"],
        names: &["Polygon Center", "PolyCenter Simple"],
        kind: ComponentKind::PolygonCenterSimple,
    },
    Registration {
        guids: &["{69f3e5ee-4770-44b3-8851-ae10ae555398}"],
        names: &["Perp Frame", "Perp Frame"],
        kind: ComponentKind::PerpFrame,
    },
    Registration {
        guids: &["{6b021f56-b194-4210-b9a1-6cef3b7d0848}"],
        names: &["Evaluate Length", "Eval Length"],
        kind: ComponentKind::EvaluateLength,
    },
    Registration {
        guids: &["{a1c16251-74f0-400f-9e7c-5e379d739963}"],
        names: &["Length Parameter", "LenParam"],
        kind: ComponentKind::LengthParameter,
    },
    Registration {
        guids: &["{c75b62fa-0a33-4da7-a5bd-03fd0068fd93}"],
        names: &["Curve Length", "CrvLength"],
        kind: ComponentKind::Length,
    },
    Registration {
        guids: &["{ccc7b468-e743-4049-891f-299432545898}"],
        names: &["Curve Middle", "CrvMid"],
        kind: ComponentKind::CurveMiddle,
    },
    Registration {
        guids: &["{f88a6cd9-1035-4361-b896-4f2dfe79272d}"],
        names: &["Segment Lengths", "CrvSegLen"],
        kind: ComponentKind::SegmentLengths,
    },
    Registration {
        guids: &["{6b7ba278-5c9d-42f1-a61d-6209cbd44907}"],
        names: &["Curve Proximity", "CrvProx"],
        kind: ComponentKind::CurveProximity,
    },
    Registration {
        guids: &["{6b2a5853-07aa-4329-ba84-0a5d46b51dbd}"],
        names: &["Curve Frame", "CrvFrame"],
        kind: ComponentKind::CurveFrame,
    },
    Registration {
        guids: &["{7376fe41-74ec-497e-b367-1ffe5072608b}"],
        names: &["Curvature Graph", "CrvGraph"],
        kind: ComponentKind::CurvatureGraph,
    },
    Registration {
        guids: &["{748f214a-bc64-4556-9da5-4fa59a30c5c7}"],
        names: &["Curve Nearest Object", "CrvNearest"],
        kind: ComponentKind::CurveNearestObject,
    },
    Registration {
        guids: &["{afff17ed-5975-460b-9883-525ae0677088}"],
        names: &["Arc Center", "CrvCenter"],
        kind: ComponentKind::ArcCenter,
    },
    Registration {
        guids: &["{a72b0bd3-c7a7-458e-875d-09ae1624638c}"],
        names: &["Point In Curve", "InCurve"],
        kind: ComponentKind::PointInCurve,
    },
    Registration {
        guids: &["{a583f722-240a-4fc9-aa1d-021720a4516a}"],
        names: &["Curve Depth", "CrvDepth"],
        kind: ComponentKind::CurveDepth,
    },
    Registration {
        guids: &["{aaa665bd-fd6e-4ccb-8d2c-c5b33072125d}"],
        names: &["Curvature", "CrvCurvature"],
        kind: ComponentKind::Curvature,
    },
    Registration {
        guids: &["{ab14760f-87a6-462e-b481-4a2c26a9a0d7}"],
        names: &["Derivatives", "CrvDeriv1"],
        kind: ComponentKind::DerivativesFirst,
    },
    Registration {
        guids: &["{bb2e13da-09ca-43fd-bef8-8d71f3653af9}"],
        names: &["Curve Side", "CrvSide"],
        kind: ComponentKind::CurveSide,
    },
    Registration {
        guids: &["{c048ad76-ffcd-43b1-a007-4dd1b2373326}"],
        names: &["Horizontal Frame", "CrvHFrame"],
        kind: ComponentKind::HorizontalFrame,
    },
    Registration {
        guids: &["{c076845a-1a09-4a95-bdcb-cb31c0936c99}"],
        names: &["Containment", "CrvContain"],
        kind: ComponentKind::Containment,
    },
    Registration {
        guids: &["{c2e16ca3-9508-4fa4-aeb3-0b1f0ebb72e3}"],
        names: &["Derivatives", "CrvDerivatives"],
        kind: ComponentKind::DerivativesList,
    },
    Registration {
        guids: &["{ccfd6ba8-ecb1-44df-a47e-08126a653c51}"],
        names: &["Curve Domain", "CrvDomain"],
        kind: ComponentKind::CurveDomainAdjust,
    },
    Registration {
        guids: &["{dbe9fce4-b6b3-465f-9615-34833c4763bd}"],
        names: &["Torsion", "CrvTorsion"],
        kind: ComponentKind::Torsion,
    },
    Registration {
        guids: &["{e5c33a79-53d5-4f2b-9a97-d3d45c780edc}"],
        names: &["Deconstruct Rectangle", "DeRect"],
        kind: ComponentKind::DeconstructRectangle,
    },
    Registration {
        guids: &["{ebd6c758-19ae-4d74-aed7-b8a0392ff743}"],
        names: &["Extremes", "CrvExtremes"],
        kind: ComponentKind::Extremes,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::PointInCurves => evaluate_point_in_curves(inputs),
            Self::EndPoints => evaluate_end_points(inputs),
            Self::CurveDomain => evaluate_curve_domain(inputs),
            Self::EvaluateCurveBasic => evaluate_curve(inputs, EvaluateOutput::PointTangent),
            Self::EvaluateCurveAngle => evaluate_curve(inputs, EvaluateOutput::PointTangentAngle),
            Self::EvaluateCurveLength => evaluate_curve(inputs, EvaluateOutput::PointTangentLength),
            Self::LengthDomain => evaluate_length_domain(inputs),
            Self::DeconstructArc => evaluate_deconstruct_arc(inputs),
            Self::Discontinuity => evaluate_discontinuity(inputs),
            Self::CurveClosestPoint => evaluate_curve_closest_point(inputs),
            Self::Closed => evaluate_closed(inputs),
            Self::ControlPointsDetailed => {
                evaluate_control_points(inputs, ControlPointMode::Detailed)
            }
            Self::ControlPointsSimple => evaluate_control_points(inputs, ControlPointMode::Simple),
            Self::ControlPolygon => evaluate_control_polygon(inputs),
            Self::Planar => evaluate_planar(inputs),
            Self::PolygonCenterDetailed => {
                evaluate_polygon_center(inputs, PolygonCenterMode::VertexEdgeArea)
            }
            Self::PolygonCenterEdge => {
                evaluate_polygon_center(inputs, PolygonCenterMode::VertexEdge)
            }
            Self::PolygonCenterSimple => {
                evaluate_polygon_center(inputs, PolygonCenterMode::VertexOnly)
            }
            Self::PerpFrame => evaluate_perp_frame(inputs),
            Self::EvaluateLength => evaluate_length(inputs),
            Self::LengthParameter => evaluate_length_parameter(inputs),
            Self::Length => evaluate_curve_length(inputs),
            Self::CurveMiddle => evaluate_curve_middle(inputs),
            Self::SegmentLengths => evaluate_segment_lengths(inputs),
            Self::CurveProximity => evaluate_curve_proximity(inputs),
            Self::CurveFrame => evaluate_curve_frame(inputs),
            Self::CurvatureGraph => Ok(BTreeMap::new()),
            Self::CurveNearestObject => evaluate_curve_nearest_object(inputs),
            Self::ArcCenter => evaluate_arc_center(inputs),
            Self::PointInCurve => evaluate_point_in_curve(inputs),
            Self::CurveDepth => evaluate_curve_depth(inputs),
            Self::Curvature => evaluate_curvature(inputs),
            Self::DerivativesFirst => evaluate_derivatives_first(inputs),
            Self::CurveSide => evaluate_curve_side(inputs),
            Self::HorizontalFrame => evaluate_horizontal_frame(inputs),
            Self::Containment => evaluate_containment(inputs),
            Self::DerivativesList => evaluate_derivatives_list(inputs),
            Self::CurveDomainAdjust => evaluate_curve_domain_adjust(inputs),
            Self::Torsion => evaluate_torsion(inputs),
            Self::DeconstructRectangle => evaluate_deconstruct_rectangle(inputs),
            Self::Extremes => evaluate_extremes(inputs),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::PointInCurves => "Point in Curves",
            Self::EndPoints => "End Points",
            Self::CurveDomain => "Curve Domain",
            Self::EvaluateCurveBasic => "Evaluate Curve",
            Self::EvaluateCurveAngle => "Evaluate Curve Angle",
            Self::EvaluateCurveLength => "Evaluate Curve Length",
            Self::LengthDomain => "Length Domain",
            Self::DeconstructArc => "Deconstruct Arc",
            Self::Discontinuity => "Discontinuity",
            Self::CurveClosestPoint => "Curve Closest Point",
            Self::Closed => "Curve Closed",
            Self::ControlPointsDetailed => "Control Points Detailed",
            Self::ControlPointsSimple => "Control Points",
            Self::ControlPolygon => "Control Polygon",
            Self::Planar => "Curve Planar",
            Self::PolygonCenterDetailed => "Polygon Center",
            Self::PolygonCenterEdge => "Polygon Center Edge",
            Self::PolygonCenterSimple => "Polygon Center",
            Self::PerpFrame => "Perp Frame",
            Self::EvaluateLength => "Evaluate Curve Length Factor",
            Self::LengthParameter => "Length Parameter",
            Self::Length => "Curve Length",
            Self::CurveMiddle => "Curve Midpoint",
            Self::SegmentLengths => "Segment Lengths",
            Self::CurveProximity => "Curve Proximity",
            Self::CurveFrame => "Curve Frame",
            Self::CurvatureGraph => "Curvature Graph",
            Self::CurveNearestObject => "Curve Nearest Object",
            Self::ArcCenter => "Curve Center",
            Self::PointInCurve => "Point In Curve",
            Self::CurveDepth => "Curve Depth",
            Self::Curvature => "Curvature",
            Self::DerivativesFirst => "Curve Derivative",
            Self::CurveSide => "Curve Side",
            Self::HorizontalFrame => "Horizontal Frame",
            Self::Containment => "Containment",
            Self::DerivativesList => "Curve Derivatives",
            Self::CurveDomainAdjust => "Curve Domain",
            Self::Torsion => "Torsion",
            Self::DeconstructRectangle => "Deconstruct Rectangle",
            Self::Extremes => "Extremes",
        }
    }
}

fn evaluate_end_points(inputs: &[Value]) -> ComponentResult {
    let points = coerce_polyline(inputs.get(0), "End Points")?;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_START.to_owned(), Value::Point(points[0]));
    outputs.insert(
        PIN_OUTPUT_END.to_owned(),
        Value::Point(*points.last().unwrap()),
    );
    Ok(outputs)
}

fn evaluate_curve_domain(inputs: &[Value]) -> ComponentResult {
    let points = coerce_polyline(inputs.get(0), "Curve Domain")?;
    let length = polyline_length(&points);
    let domain = create_domain1d(0.0, length);
    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_DOMAIN.to_owned(),
        Value::Domain(Domain::One(domain)),
    );
    Ok(outputs)
}

#[derive(Debug, Clone, Copy)]
enum EvaluateOutput {
    PointTangent,
    PointTangentAngle,
    PointTangentLength,
}

fn evaluate_curve(inputs: &[Value], mode: EvaluateOutput) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Evaluate Curve vereist een curve en parameter",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Evaluate Curve")?;
    let parameter = coerce_number(inputs.get(1), "Evaluate Curve")?;
    let evaluation = sample_curve(&points, parameter);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINT.to_owned(), Value::Point(evaluation.point));
    outputs.insert(
        PIN_OUTPUT_TANGENT.to_owned(),
        Value::Vector(evaluation.tangent.unwrap_or([1.0, 0.0, 0.0])),
    );

    match mode {
        EvaluateOutput::PointTangent => {}
        EvaluateOutput::PointTangentAngle => {
            outputs.insert(
                PIN_OUTPUT_ANGLE.to_owned(),
                Value::Number(evaluation.angle.unwrap_or(0.0)),
            );
        }
        EvaluateOutput::PointTangentLength => {
            outputs.insert(
                PIN_OUTPUT_LENGTH.to_owned(),
                Value::Number(evaluation.length_along),
            );
        }
    }

    Ok(outputs)
}

fn evaluate_length_domain(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Length Domain vereist een curve en domein",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Length Domain")?;
    let Some(Value::Domain(Domain::One(domain))) = inputs.get(1) else {
        return Err(ComponentError::new("Length Domain verwacht een 1D domein"));
    };

    let length = polyline_length(&points);
    let start = clamp(domain.start.min(domain.end), 0.0, length);
    let end = clamp(domain.start.max(domain.end), 0.0, length);
    let segment_length = end - start;

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_LENGTH.to_owned(), Value::Number(segment_length));
    Ok(outputs)
}

fn evaluate_deconstruct_arc(inputs: &[Value]) -> ComponentResult {
    let points = coerce_polyline(inputs.get(0), "Deconstruct Arc")?;
    let Some(circle) = fit_circle(&points) else {
        return Err(ComponentError::new(
            "Deconstruct Arc kon geen cirkel fitten uit de invoer",
        ));
    };

    let x_axis = safe_normalized(subtract(points[0], circle.center))
        .map(|(v, _)| v)
        .unwrap_or([1.0, 0.0, 0.0]);
    let mut y_axis = cross(circle.normal, x_axis);
    if length_squared(y_axis) < EPSILON {
        y_axis = cross(x_axis, circle.normal);
    }
    let y_axis = normalize(y_axis);
    let plane_value = Value::List(vec![
        Value::Point(circle.center),
        Value::Point(add(circle.center, x_axis)),
        Value::Point(add(circle.center, y_axis)),
    ]);

    let arc_length = polyline_length(&points);
    let angle = if circle.radius < EPSILON {
        0.0
    } else {
        arc_length / circle.radius
    };
    let angle_domain = create_domain1d(0.0, angle);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BASE_PLANE.to_owned(), plane_value);
    outputs.insert(PIN_OUTPUT_RADIUS.to_owned(), Value::Number(circle.radius));
    outputs.insert(
        PIN_OUTPUT_ANGLE.to_owned(),
        Value::Domain(Domain::One(angle_domain)),
    );
    Ok(outputs)
}

fn evaluate_discontinuity(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Discontinuity vereist minimaal een curve",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Discontinuity")?;
    let level = inputs
        .get(1)
        .map(|value| coerce_number(Some(value), "Discontinuity"))
        .transpose()?
        .unwrap_or(1.0);
    let requested = level.clamp(1.0, 3.0);

    let segments = polyline_segments(&points);
    if segments.len() < 2 {
        return Ok(BTreeMap::new());
    }

    let total_length: f64 = segments.iter().map(|segment| segment.length).sum();
    let mut accumulated = 0.0;
    let mut parameters = Vec::new();
    let mut discontinuity_points = Vec::new();

    for index in 1..segments.len() {
        let previous = &segments[index - 1];
        let next = &segments[index];
        let prev_dir = safe_normalized(subtract(previous.end, previous.start))
            .map(|(v, _)| v)
            .unwrap_or([0.0, 0.0, 0.0]);
        let next_dir = safe_normalized(subtract(next.end, next.start))
            .map(|(v, _)| v)
            .unwrap_or([0.0, 0.0, 0.0]);
        let dot_value = clamp(dot(prev_dir, next_dir), -1.0, 1.0);
        let angle = dot_value.acos();
        let threshold = if requested >= 2.0 { 1e-4 } else { 1e-3 };
        if angle > threshold {
            accumulated += previous.length;
            let parameter = if total_length < EPSILON {
                0.0
            } else {
                accumulated / total_length
            };
            parameters.push(Value::Number(parameter));
            discontinuity_points.push(Value::Point(previous.end));
        } else {
            accumulated += previous.length;
        }
    }

    if is_closed(&points) {
        let first = segments.first().unwrap();
        let last = segments.last().unwrap();
        let first_dir = safe_normalized(subtract(first.end, first.start))
            .map(|(v, _)| v)
            .unwrap_or([0.0, 0.0, 0.0]);
        let last_dir = safe_normalized(subtract(last.end, last.start))
            .map(|(v, _)| v)
            .unwrap_or([0.0, 0.0, 0.0]);
        let dot_value = clamp(dot(last_dir, first_dir), -1.0, 1.0);
        let angle = dot_value.acos();
        let threshold = if requested >= 2.0 { 1e-4 } else { 1e-3 };
        if angle > threshold {
            parameters.push(Value::Number(0.0));
            discontinuity_points.push(Value::Point(first.start));
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_POINTS.to_owned(),
        Value::List(discontinuity_points),
    );
    outputs.insert(PIN_OUTPUT_PARAMETER.to_owned(), Value::List(parameters));
    Ok(outputs)
}

fn evaluate_curve_closest_point(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Curve Closest Point vereist een punt en curve",
        ));
    }

    let point = coerce_point(inputs.get(0), "Curve Closest Point")?;
    let points = coerce_polyline(inputs.get(1), "Curve Closest Point")?;
    let result = closest_point_on_polyline(point, &points);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINT.to_owned(), Value::Point(result.point));
    outputs.insert(
        PIN_OUTPUT_PARAMETER.to_owned(),
        Value::Number(result.parameter),
    );
    outputs.insert(
        PIN_OUTPUT_DISTANCE.to_owned(),
        Value::Number(result.distance),
    );
    Ok(outputs)
}

fn evaluate_perp_frame(inputs: &[Value]) -> ComponentResult {
    evaluate_frame(inputs, "Perp Frame", FrameMode::Parallel)
}

fn evaluate_curve_frame(inputs: &[Value]) -> ComponentResult {
    evaluate_frame(inputs, "Curve Frame", FrameMode::Frenet)
}

fn evaluate_horizontal_frame(inputs: &[Value]) -> ComponentResult {
    evaluate_frame(inputs, "Horizontal Frame", FrameMode::Horizontal)
}

fn evaluate_frame(inputs: &[Value], context: &str, mode: FrameMode) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(format!(
            "{} vereist een curve en parameter",
            context
        )));
    }

    let points = coerce_polyline(inputs.get(0), context)?;
    let parameter = coerce_number(inputs.get(1), context)?;
    let sample = sample_curve(&points, parameter);
    let derivative = approximate_derivative(&points, parameter, 1);
    let tangent = safe_normalized(derivative)
        .map(|(v, _)| v)
        .unwrap_or_else(|| sample.tangent.unwrap_or([1.0, 0.0, 0.0]));

    let frame = match mode {
        FrameMode::Frenet => compute_frenet_frame(&points, parameter, sample.point, tangent),
        FrameMode::Parallel => compute_parallel_frame(&points, sample.point, tangent),
        FrameMode::Horizontal => compute_horizontal_frame(sample.point, tangent),
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_FRAME.to_owned(),
        frame_value(frame.origin, frame.x_axis, frame.y_axis, frame.z_axis),
    );
    Ok(outputs)
}

fn evaluate_point_in_curves(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Point in Curves vereist een punt en een verzameling curves",
        ));
    }

    let point = coerce_point(inputs.get(0), "Point in Curves")?;
    let curves = coerce_polyline_collection(inputs.get(1), "Point in Curves")?;
    if curves.is_empty() {
        return Err(ComponentError::new(
            "Point in Curves vereist minstens één curve",
        ));
    }

    let mut best_relationship = 0;
    let mut best_index: i32 = -1;
    let mut best_projected = point;

    for (index, curve) in curves.iter().enumerate() {
        let classification = classify_point_against_polyline(point, curve);
        if classification.relationship > best_relationship {
            best_relationship = classification.relationship;
            best_index = index as i32;
            best_projected = classification.projected;
            if best_relationship == 2 {
                break;
            }
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_RELATIONSHIP.to_owned(),
        Value::Number(best_relationship as f64),
    );
    outputs.insert(
        PIN_OUTPUT_INDEX.to_owned(),
        Value::Number(if best_relationship == 0 {
            -1.0
        } else {
            best_index as f64
        }),
    );
    outputs.insert(
        PIN_OUTPUT_POINT_PRIME.to_owned(),
        Value::Point(best_projected),
    );
    Ok(outputs)
}

fn evaluate_curve_nearest_object(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Curve Nearest Object vereist een curve en geometrie",
        ));
    }

    let curve = coerce_polyline(inputs.get(0), "Curve Nearest Object")?;
    let geometry_input = inputs
        .get(1)
        .ok_or_else(|| ComponentError::new("Curve Nearest Object vereist geometrie invoer"))?;

    let entries: Vec<&Value> = match geometry_input {
        Value::List(values) => values.iter().collect(),
        other => vec![other],
    };
    if entries.is_empty() {
        return Err(ComponentError::new(
            "Curve Nearest Object vereist minstens één geometrie",
        ));
    }

    let mut best_distance = f64::INFINITY;
    let mut best_point_curve = None;
    let mut best_point_other = None;
    let mut best_index = -1;

    for (index, entry) in entries.iter().enumerate() {
        let points = extract_points(entry);
        if points.is_empty() {
            continue;
        }

        if points.len() == 1 {
            let result = closest_point_on_polyline(points[0], &curve);
            if result.distance < best_distance {
                best_distance = result.distance;
                best_point_curve = Some(result.point);
                best_point_other = Some(points[0]);
                best_index = index as i32;
            }
        } else {
            let other = if points[0] == *points.last().unwrap() {
                points
            } else {
                let mut closed = points.clone();
                closed.push(points[0]);
                closed
            };
            let proximity = closest_points_between_polylines(&curve, &other);
            if proximity.distance < best_distance {
                best_distance = proximity.distance;
                best_point_curve = Some(proximity.point_a);
                best_point_other = Some(proximity.point_b);
                best_index = index as i32;
            }
        }
    }

    if best_index < 0 {
        return Err(ComponentError::new(
            "Curve Nearest Object kon geen dichtstbijzijnde geometrie bepalen",
        ));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_POINT_A.to_owned(),
        Value::Point(best_point_curve.unwrap()),
    );
    outputs.insert(
        PIN_OUTPUT_POINT_B.to_owned(),
        Value::Point(best_point_other.unwrap()),
    );
    outputs.insert(
        PIN_OUTPUT_INDEX.to_owned(),
        Value::Number(best_index as f64),
    );
    Ok(outputs)
}

fn evaluate_curve_depth(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Curve Depth vereist een curve"));
    }

    let points = coerce_polyline(inputs.get(0), "Curve Depth")?;
    let min_limit = inputs
        .get(1)
        .map(|value| coerce_number(Some(value), "Curve Depth"))
        .transpose()?
        .unwrap_or(f64::NEG_INFINITY);
    let max_limit = inputs
        .get(2)
        .map(|value| coerce_number(Some(value), "Curve Depth"))
        .transpose()?
        .unwrap_or(f64::INFINITY);

    let segments = polyline_segments(&points);
    if segments.is_empty() {
        return Err(ComponentError::new(
            "Curve Depth vereist minstens één segment",
        ));
    }

    let total_length: f64 = segments.iter().map(|segment| segment.length).sum();
    let mut accumulated = 0.0;
    let mut best_min: Option<(f64, f64)> = None;
    let mut best_max: Option<(f64, f64)> = None;

    for segment in &segments {
        consider_depth_point(
            segment.start,
            accumulated,
            total_length,
            min_limit,
            max_limit,
            &mut best_min,
            &mut best_max,
        );
        accumulated += segment.length;
        consider_depth_point(
            segment.end,
            accumulated,
            total_length,
            min_limit,
            max_limit,
            &mut best_min,
            &mut best_max,
        );
    }

    if best_min.is_none() && best_max.is_none() {
        return Err(ComponentError::new(
            "Curve Depth vond geen punten binnen de grenzen",
        ));
    }

    let mut outputs = BTreeMap::new();
    if let Some((parameter, depth)) = best_min {
        outputs.insert(
            PIN_OUTPUT_MIN_PARAMETER.to_owned(),
            Value::Number(parameter),
        );
        outputs.insert(PIN_OUTPUT_MIN_DEPTH.to_owned(), Value::Number(depth));
    }
    if let Some((parameter, depth)) = best_max {
        outputs.insert(
            PIN_OUTPUT_MAX_PARAMETER.to_owned(),
            Value::Number(parameter),
        );
        outputs.insert(PIN_OUTPUT_MAX_DEPTH.to_owned(), Value::Number(depth));
    }
    Ok(outputs)
}

fn evaluate_curvature(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Curvature vereist een curve en parameter",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Curvature")?;
    let parameter = coerce_number(inputs.get(1), "Curvature")?;
    let sample = sample_curve(&points, parameter);
    let derivative = approximate_derivative(&points, parameter, 1);
    let second = approximate_derivative(&points, parameter, 2);
    let tangent = safe_normalized(derivative)
        .map(|(v, _)| v)
        .unwrap_or(sample.tangent.unwrap_or([1.0, 0.0, 0.0]));
    let mut normal_component = subtract(second, scale(tangent, dot(second, tangent)));
    if length_squared(normal_component) < EPSILON {
        normal_component = [0.0, 0.0, 0.0];
    }
    let curvature_vector = normal_component;
    let (normalized_normal, magnitude) =
        safe_normalized(curvature_vector).unwrap_or(([0.0, 0.0, 0.0], 0.0));
    let center = if magnitude < EPSILON {
        sample.point
    } else {
        add(sample.point, scale(normalized_normal, 1.0 / magnitude))
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINT.to_owned(), Value::Point(sample.point));
    outputs.insert(
        PIN_OUTPUT_CURVATURE_VECTOR.to_owned(),
        Value::Vector(curvature_vector),
    );
    outputs.insert(PIN_OUTPUT_CURVATURE_CENTER.to_owned(), Value::Point(center));
    Ok(outputs)
}

fn evaluate_derivatives_first(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Derivatives vereist een curve en parameter",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Derivatives")?;
    let parameter = coerce_number(inputs.get(1), "Derivatives")?;
    let sample = sample_curve(&points, parameter);
    let derivative = approximate_derivative(&points, parameter, 1);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINT.to_owned(), Value::Point(sample.point));
    outputs.insert(
        PIN_OUTPUT_FIRST_DERIVATIVE.to_owned(),
        Value::Vector(derivative),
    );
    Ok(outputs)
}

fn evaluate_curve_side(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Curve Side vereist een curve en punt"));
    }

    let curve = coerce_polyline(inputs.get(0), "Curve Side")?;
    let point = coerce_point(inputs.get(1), "Curve Side")?;
    let plane = inputs
        .get(2)
        .map(|value| plane_basis_from_value(value, "Curve Side"))
        .transpose()
        .unwrap_or_else(|_| None)
        .unwrap_or_else(|| plane_from_polyline(&curve));

    let closest = closest_point_on_polyline(point, &curve);
    let tangent = sample_curve(&curve, closest.parameter)
        .tangent
        .unwrap_or([1.0, 0.0, 0.0]);
    let to_point = subtract(point, closest.point);
    let sign = dot(cross(tangent, to_point), plane.normal);
    let side = if sign > 1e-6 {
        1.0
    } else if sign < -1e-6 {
        -1.0
    } else {
        0.0
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_SIDE.to_owned(), Value::Number(side));
    outputs.insert(PIN_OUTPUT_LEFT.to_owned(), Value::Boolean(side > 0.0));
    outputs.insert(PIN_OUTPUT_RIGHT.to_owned(), Value::Boolean(side < 0.0));
    Ok(outputs)
}

fn evaluate_containment(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Containment vereist een punt en curve"));
    }

    let point = coerce_point(inputs.get(0), "Containment")?;
    let curve = coerce_polyline(inputs.get(1), "Containment")?;
    let classification = classify_point_against_polyline(point, &curve);
    let relationship = match classification.relationship {
        0 => 2.0,
        1 => 0.0,
        2 => 1.0,
        _ => 2.0,
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_RELATIONSHIP.to_owned(),
        Value::Number(relationship),
    );
    outputs.insert(
        PIN_OUTPUT_POINT_PRIME.to_owned(),
        Value::Point(classification.projected),
    );
    Ok(outputs)
}

fn evaluate_derivatives_list(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Derivatives vereist een curve en parameter",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Derivatives")?;
    let parameter = coerce_number(inputs.get(1), "Derivatives")?;
    let count = inputs
        .get(2)
        .map(|value| coerce_number(Some(value), "Derivatives"))
        .transpose()?
        .unwrap_or(1.0)
        .round()
        .clamp(1.0, 3.0) as usize;
    let sample = sample_curve(&points, parameter);

    let mut derivatives = Vec::with_capacity(count);
    for order in 1..=count {
        derivatives.push(Value::Vector(approximate_derivative(
            &points, parameter, order,
        )));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINT.to_owned(), Value::Point(sample.point));
    outputs.insert(PIN_OUTPUT_DERIVATIVES.to_owned(), Value::List(derivatives));
    Ok(outputs)
}

fn evaluate_curve_domain_adjust(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Curve Domain vereist een curve"));
    }

    let points = coerce_polyline(inputs.get(0), "Curve Domain")?;
    let total_length = polyline_length(&points);
    let domain_value = inputs.get(1);
    let domain = if let Some(Value::Domain(Domain::One(value))) = domain_value {
        copy_domain1d(value)
    } else {
        create_domain1d(0.0, total_length)
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_CURVE.to_owned(),
        Value::List(points.iter().copied().map(Value::Point).collect()),
    );
    outputs.insert(
        PIN_OUTPUT_DOMAIN.to_owned(),
        Value::Domain(Domain::One(domain)),
    );
    Ok(outputs)
}

fn evaluate_torsion(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Torsion vereist een curve en parameter",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Torsion")?;
    let parameter = coerce_number(inputs.get(1), "Torsion")?;
    let sample = sample_curve(&points, parameter);
    let first = approximate_derivative(&points, parameter, 1);
    let second = approximate_derivative(&points, parameter, 2);
    let third = approximate_derivative(&points, parameter, 3);
    let cross12 = cross(first, second);
    let denominator = length_squared(cross12);
    let torsion = if denominator < EPSILON {
        0.0
    } else {
        dot(cross12, third) / denominator
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINT.to_owned(), Value::Point(sample.point));
    outputs.insert(PIN_OUTPUT_TORSION.to_owned(), Value::Number(torsion));
    Ok(outputs)
}

fn evaluate_deconstruct_rectangle(inputs: &[Value]) -> ComponentResult {
    let points = coerce_polyline(inputs.get(0), "Deconstruct Rectangle")?;
    if points.len() < 4 {
        return Err(ComponentError::new(
            "Deconstruct Rectangle verwacht minstens vier punten",
        ));
    }

    let origin = points[0];
    let mut x_vec = subtract(points[1], origin);
    if length_squared(x_vec) < EPSILON {
        x_vec = [1.0, 0.0, 0.0];
    }
    let mut y_vec = [0.0, 0.0, 0.0];
    for candidate in points.iter().skip(2) {
        let vector = subtract(*candidate, origin);
        if length_squared(cross(x_vec, vector)) > EPSILON {
            y_vec = vector;
            break;
        }
    }
    if length_squared(y_vec) < EPSILON {
        y_vec = [0.0, 1.0, 0.0];
    }

    let x_axis = safe_normalized(x_vec)
        .map(|(v, _)| v)
        .unwrap_or([1.0, 0.0, 0.0]);
    let y_projection = subtract(y_vec, scale(x_axis, dot(y_vec, x_axis)));
    let y_axis = safe_normalized(y_projection)
        .map(|(v, _)| v)
        .unwrap_or([0.0, 1.0, 0.0]);
    let plane_value = Value::List(vec![
        Value::Point(origin),
        Value::Point(add(origin, x_axis)),
        Value::Point(add(origin, y_axis)),
    ]);

    let width = length(x_vec);
    let height = length(subtract(y_vec, scale(x_axis, dot(y_vec, x_axis))));
    let x_interval = create_domain1d(-width * 0.5, width * 0.5);
    let y_interval = create_domain1d(-height * 0.5, height * 0.5);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BASE_PLANE.to_owned(), plane_value);
    outputs.insert(
        PIN_OUTPUT_X_INTERVAL.to_owned(),
        Value::Domain(Domain::One(x_interval)),
    );
    outputs.insert(
        PIN_OUTPUT_Y_INTERVAL.to_owned(),
        Value::Domain(Domain::One(y_interval)),
    );
    Ok(outputs)
}

fn evaluate_extremes(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Extremes vereist een curve"));
    }

    let curve = coerce_polyline(inputs.get(0), "Extremes")?;
    let plane = inputs
        .get(1)
        .map(|value| plane_basis_from_value(value, "Extremes"))
        .transpose()
        .unwrap_or_else(|_| None)
        .unwrap_or_else(|| plane_from_polyline(&curve));

    let mut highest_value = f64::NEG_INFINITY;
    let mut lowest_value = f64::INFINITY;
    let mut highest_points = Vec::new();
    let mut lowest_points = Vec::new();

    for point in &curve {
        let relative = subtract(*point, plane.origin);
        let value = dot(relative, plane.normal);
        if value > highest_value + 1e-6 {
            highest_points.clear();
            highest_value = value;
        }
        if (value - highest_value).abs() <= 1e-6 {
            highest_points.push(Value::Point(*point));
        }

        if value < lowest_value - 1e-6 {
            lowest_points.clear();
            lowest_value = value;
        }
        if (value - lowest_value).abs() <= 1e-6 {
            lowest_points.push(Value::Point(*point));
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_HIGHEST.to_owned(), Value::List(highest_points));
    outputs.insert(PIN_OUTPUT_LOWEST.to_owned(), Value::List(lowest_points));
    Ok(outputs)
}

#[derive(Debug, Clone, Copy)]
struct CircleFit {
    center: [f64; 3],
    radius: f64,
    normal: [f64; 3],
}

fn fit_circle(points: &[[f64; 3]]) -> Option<CircleFit> {
    if points.len() < 3 {
        return None;
    }
    let p0 = points[0];
    let p1 = points[points.len() / 2];
    let p2 = *points.last().unwrap();
    let a = subtract(p1, p0);
    let b = subtract(p2, p0);
    let axb = cross(a, b);
    let denom = 2.0 * dot(axb, axb);
    if denom.abs() < EPSILON {
        return None;
    }
    let a_len2 = dot(a, a);
    let b_len2 = dot(b, b);
    let term1 = cross(axb, a);
    let term2 = cross(b, axb);
    let center_offset = scale(add(scale(term1, b_len2), scale(term2, a_len2)), 1.0 / denom);
    let center = add(p0, center_offset);
    let radius = distance(center, p0);
    let normal = normalize(axb);
    Some(CircleFit {
        center,
        radius,
        normal,
    })
}

#[derive(Debug, Clone, Copy)]
enum FrameMode {
    Frenet,
    Parallel,
    Horizontal,
}

#[derive(Debug, Clone, Copy)]
struct FrameData {
    origin: [f64; 3],
    x_axis: [f64; 3],
    y_axis: [f64; 3],
    z_axis: [f64; 3],
}

fn frame_value(origin: [f64; 3], x_axis: [f64; 3], y_axis: [f64; 3], z_axis: [f64; 3]) -> Value {
    Value::List(vec![
        Value::Point(origin),
        Value::Vector(x_axis),
        Value::Vector(y_axis),
        Value::Vector(z_axis),
    ])
}

fn compute_frenet_frame(
    points: &[[f64; 3]],
    parameter: f64,
    origin: [f64; 3],
    tangent: [f64; 3],
) -> FrameData {
    let second = approximate_derivative(points, parameter, 2);
    let mut normal = subtract(second, scale(tangent, dot(second, tangent)));
    if length_squared(normal) < EPSILON {
        normal = [0.0, 0.0, 0.0];
    }
    let normal = safe_normalized(normal)
        .map(|(v, _)| v)
        .unwrap_or_else(|| orthogonal_vector(tangent));
    let binormal = normalize(cross(tangent, normal));
    let normal = normalize(cross(binormal, tangent));
    FrameData {
        origin,
        x_axis: tangent,
        y_axis: normal,
        z_axis: binormal,
    }
}

fn compute_parallel_frame(points: &[[f64; 3]], origin: [f64; 3], tangent: [f64; 3]) -> FrameData {
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

fn compute_horizontal_frame(origin: [f64; 3], tangent: [f64; 3]) -> FrameData {
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

#[derive(Debug, Clone, Copy)]
struct PlaneBasis {
    origin: [f64; 3],
    normal: [f64; 3],
}

fn plane_basis_from_value(value: &Value, context: &str) -> Result<PlaneBasis, ComponentError> {
    match value {
        Value::List(values) if values.len() >= 3 => {
            let a = coerce_point(values.get(0), context)?;
            let b = coerce_point(values.get(1), context)?;
            let c = coerce_point(values.get(2), context)?;
            Ok(plane_basis_from_points(a, b, c))
        }
        Value::List(values) if values.len() == 2 => {
            let origin = coerce_point(values.get(0), context)?;
            let direction_point = coerce_point(values.get(1), context)
                .or_else(|_| coerce_vector(values.get(1), context))?;
            let mut x_axis = subtract(direction_point, origin);
            if length_squared(x_axis) < EPSILON {
                x_axis = [1.0, 0.0, 0.0];
            }
            let x_axis = normalize(x_axis);
            let normal = orthogonal_vector(x_axis);
            Ok(PlaneBasis { origin, normal })
        }
        Value::Point(point) => Ok(PlaneBasis {
            origin: *point,
            normal: [0.0, 0.0, 1.0],
        }),
        _ => Err(ComponentError::new(format!(
            "{} verwacht een vlak, kreeg {}",
            context,
            value.kind()
        ))),
    }
}

fn plane_basis_from_points(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> PlaneBasis {
    let x_axis = normalize(subtract(b, a));
    let mut temp = subtract(c, a);
    if length_squared(temp) < EPSILON {
        temp = orthogonal_vector(x_axis);
    }
    let y_axis = normalize(subtract(temp, scale(x_axis, dot(temp, x_axis))));
    let normal = normalize(cross(x_axis, y_axis));
    PlaneBasis { origin: a, normal }
}

fn plane_from_polyline(points: &[[f64; 3]]) -> PlaneBasis {
    let analysis = analyse_planarity(points);
    if let Value::List(values) = &analysis.plane_value {
        if values.len() >= 3 {
            if let (Ok(a), Ok(b), Ok(c)) = (
                coerce_point(values.get(0), "Plane"),
                coerce_point(values.get(1), "Plane"),
                coerce_point(values.get(2), "Plane"),
            ) {
                return plane_basis_from_points(a, b, c);
            }
        }
    }
    PlaneBasis {
        origin: points.first().copied().unwrap_or([0.0, 0.0, 0.0]),
        normal: [0.0, 0.0, 1.0],
    }
}

fn consider_depth_point(
    point: [f64; 3],
    length_along: f64,
    total_length: f64,
    min_limit: f64,
    max_limit: f64,
    best_min: &mut Option<(f64, f64)>,
    best_max: &mut Option<(f64, f64)>,
) {
    if point[2] < min_limit - 1e-6 || point[2] > max_limit + 1e-6 {
        return;
    }
    let parameter = if total_length < EPSILON {
        0.0
    } else {
        length_along / total_length
    };
    match best_min {
        None => *best_min = Some((parameter, point[2])),
        Some((_, depth)) if point[2] < *depth => *best_min = Some((parameter, point[2])),
        _ => {}
    }
    match best_max {
        None => *best_max = Some((parameter, point[2])),
        Some((_, depth)) if point[2] > *depth => *best_max = Some((parameter, point[2])),
        _ => {}
    }
}

fn approximate_derivative(points: &[[f64; 3]], parameter: f64, order: usize) -> [f64; 3] {
    let h = 1.0 / (points.len().max(8) as f64 * 4.0);
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

fn copy_domain1d(domain: &Domain1D) -> Domain1D {
    Domain1D {
        start: domain.start,
        end: domain.end,
        min: domain.min,
        max: domain.max,
        span: domain.span,
        length: domain.length,
        center: domain.center,
    }
}

fn extract_points(value: &Value) -> Vec<[f64; 3]> {
    match value {
        Value::Point(point) => vec![*point],
        Value::CurveLine { p1, p2 } => vec![*p1, *p2],
        Value::List(values) => {
            let mut collected = Vec::new();
            for entry in values {
                collected.extend(extract_points(entry));
            }
            collected
        }
        _ => Vec::new(),
    }
}

fn evaluate_closed(inputs: &[Value]) -> ComponentResult {
    let points = coerce_polyline(inputs.get(0), "Curve Closed")?;
    let closed = is_closed(&points);
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CLOSED.to_owned(), Value::Boolean(closed));
    outputs.insert(PIN_OUTPUT_PERIODIC.to_owned(), Value::Boolean(closed));
    Ok(outputs)
}

#[derive(Debug, Clone, Copy)]
enum ControlPointMode {
    Detailed,
    Simple,
}

fn evaluate_control_points(inputs: &[Value], mode: ControlPointMode) -> ComponentResult {
    let points = coerce_polyline(inputs.get(0), "Control Points")?;
    let weight_list: Vec<Value> = points.iter().map(|_| Value::Number(1.0)).collect();
    let knot_list: Vec<Value> = (0..points.len())
        .map(|idx| Value::Number(idx as f64))
        .collect();

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_POINTS.to_owned(),
        Value::List(points.into_iter().map(Value::Point).collect()),
    );

    outputs.insert(PIN_OUTPUT_WEIGHTS.to_owned(), Value::List(weight_list));

    if matches!(mode, ControlPointMode::Detailed) {
        outputs.insert(PIN_OUTPUT_KNOTS.to_owned(), Value::List(knot_list));
    }

    Ok(outputs)
}

fn evaluate_control_polygon(inputs: &[Value]) -> ComponentResult {
    let points = coerce_polyline(inputs.get(0), "Control Polygon")?;
    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_CURVE.to_owned(),
        Value::List(points.iter().copied().map(Value::Point).collect()),
    );
    outputs.insert(
        PIN_OUTPUT_POINTS.to_owned(),
        Value::List(points.into_iter().map(Value::Point).collect()),
    );
    Ok(outputs)
}

fn evaluate_planar(inputs: &[Value]) -> ComponentResult {
    let points = coerce_polyline(inputs.get(0), "Curve Planar")?;
    let analysis = analyse_planarity(&points);
    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_PLANAR.to_owned(),
        Value::Boolean(analysis.planar),
    );
    outputs.insert(
        PIN_OUTPUT_DEVIATION.to_owned(),
        Value::Number(analysis.deviation),
    );
    outputs.insert(PIN_OUTPUT_PLANE.to_owned(), analysis.plane_value);
    Ok(outputs)
}

#[derive(Debug, Clone, Copy)]
enum PolygonCenterMode {
    VertexEdgeArea,
    VertexEdge,
    VertexOnly,
}

fn evaluate_polygon_center(inputs: &[Value], mode: PolygonCenterMode) -> ComponentResult {
    let points = coerce_polyline(inputs.get(0), "Polygon Center")?;
    let vertex = average_points(&points);
    let edge = average_edges(&points);
    let area = polygon_area_centroid(&points).unwrap_or(vertex);

    let mut outputs = BTreeMap::new();
    match mode {
        PolygonCenterMode::VertexEdgeArea => {
            outputs.insert(PIN_OUTPUT_CENTER_VERTEX.to_owned(), Value::Point(vertex));
            outputs.insert(PIN_OUTPUT_CENTER_EDGE.to_owned(), Value::Point(edge));
            outputs.insert(PIN_OUTPUT_CENTER_AREA.to_owned(), Value::Point(area));
        }
        PolygonCenterMode::VertexEdge => {
            outputs.insert(PIN_OUTPUT_CENTER_VERTEX.to_owned(), Value::Point(vertex));
            outputs.insert(PIN_OUTPUT_CENTER_EDGE.to_owned(), Value::Point(edge));
        }
        PolygonCenterMode::VertexOnly => {
            outputs.insert(PIN_OUTPUT_CENTER_SINGLE.to_owned(), Value::Point(vertex));
        }
    }
    Ok(outputs)
}

fn evaluate_length(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Evaluate Length vereist een curve en lengte",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Evaluate Length")?;
    let length_input = coerce_number(inputs.get(1), "Evaluate Length")?;
    let normalized = inputs
        .get(2)
        .map(|value| matches!(value, Value::Boolean(true)))
        .unwrap_or(false);

    let total_length = polyline_length(&points);
    let factor = if normalized || total_length < EPSILON {
        clamp(length_input, 0.0, 1.0)
    } else {
        if total_length < EPSILON {
            0.0
        } else {
            clamp(length_input / total_length, 0.0, 1.0)
        }
    };

    let evaluation = sample_curve(&points, factor);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINT.to_owned(), Value::Point(evaluation.point));
    outputs.insert(
        PIN_OUTPUT_TANGENT.to_owned(),
        Value::Vector(evaluation.tangent.unwrap_or([1.0, 0.0, 0.0])),
    );
    outputs.insert(PIN_OUTPUT_PARAMETER.to_owned(), Value::Number(factor));
    Ok(outputs)
}

fn evaluate_length_parameter(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Length Parameter vereist een curve en parameter",
        ));
    }

    let points = coerce_polyline(inputs.get(0), "Length Parameter")?;
    let parameter = coerce_number(inputs.get(1), "Length Parameter")?;
    let evaluation = sample_curve(&points, parameter);
    let total_length = polyline_length(&points);

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_LENGTH_MINUS.to_owned(),
        Value::Number(evaluation.length_along),
    );
    outputs.insert(
        PIN_OUTPUT_LENGTH_PLUS.to_owned(),
        Value::Number((total_length - evaluation.length_along).max(0.0)),
    );
    Ok(outputs)
}

fn evaluate_curve_length(inputs: &[Value]) -> ComponentResult {
    let points = coerce_polyline(inputs.get(0), "Curve Length")?;
    let length = polyline_length(&points);
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_LENGTH.to_owned(), Value::Number(length));
    Ok(outputs)
}

fn evaluate_curve_middle(inputs: &[Value]) -> ComponentResult {
    let points = coerce_polyline(inputs.get(0), "Curve Middle")?;
    let evaluation = sample_curve(&points, 0.5);
    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_MIDPOINT.to_owned(),
        Value::Point(evaluation.point),
    );
    Ok(outputs)
}

fn evaluate_segment_lengths(inputs: &[Value]) -> ComponentResult {
    let points = coerce_polyline(inputs.get(0), "Segment Lengths")?;
    let segments = polyline_segments(&points);
    if segments.is_empty() {
        return Err(ComponentError::new(
            "Segment Lengths vereist minstens één segment",
        ));
    }

    let mut shortest = None;
    let mut longest = None;
    for (index, segment) in segments.iter().enumerate() {
        let length = segment.length;
        let domain = create_domain1d(
            index as f64 / segments.len() as f64,
            (index + 1) as f64 / segments.len() as f64,
        );
        match shortest {
            None => shortest = Some((length, domain.clone())),
            Some((current, _)) if length < current => shortest = Some((length, domain.clone())),
            _ => {}
        }
        match longest {
            None => longest = Some((length, domain)),
            Some((current, _)) if length > current => longest = Some((length, domain)),
            _ => {}
        }
    }

    let mut outputs = BTreeMap::new();
    if let Some((length, domain)) = shortest {
        outputs.insert(PIN_OUTPUT_SHORTEST_LENGTH.to_owned(), Value::Number(length));
        outputs.insert(
            PIN_OUTPUT_SHORTEST_DOMAIN.to_owned(),
            Value::Domain(Domain::One(domain)),
        );
    }
    if let Some((length, domain)) = longest {
        outputs.insert(PIN_OUTPUT_LONGEST_LENGTH.to_owned(), Value::Number(length));
        outputs.insert(
            PIN_OUTPUT_LONGEST_DOMAIN.to_owned(),
            Value::Domain(Domain::One(domain)),
        );
    }
    Ok(outputs)
}

fn evaluate_curve_proximity(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Curve Proximity vereist twee curves"));
    }

    let curve_a = coerce_polyline(inputs.get(0), "Curve Proximity")?;
    let curve_b = coerce_polyline(inputs.get(1), "Curve Proximity")?;
    let proximity = closest_points_between_polylines(&curve_a, &curve_b);

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_POINT_A.to_owned(),
        Value::Point(proximity.point_a),
    );
    outputs.insert(
        PIN_OUTPUT_POINT_B.to_owned(),
        Value::Point(proximity.point_b),
    );
    outputs.insert(
        PIN_OUTPUT_DISTANCE.to_owned(),
        Value::Number(proximity.distance),
    );
    Ok(outputs)
}

fn evaluate_arc_center(inputs: &[Value]) -> ComponentResult {
    let points = coerce_polyline(inputs.get(0), "Curve Center")?;
    let first = points.first().copied().unwrap_or([0.0, 0.0, 0.0]);
    let last = points.last().copied().unwrap_or(first);
    let center = scale(add(first, last), 0.5);
    let radius = distance(first, last) * 0.5;

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CENTER_SINGLE.to_owned(), Value::Point(center));
    outputs.insert(PIN_OUTPUT_RADIUS.to_owned(), Value::Number(radius));
    Ok(outputs)
}

fn evaluate_point_in_curve(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Point In Curve vereist een punt en curve",
        ));
    }

    let point = coerce_point(inputs.get(0), "Point In Curve")?;
    let curve = coerce_polyline(inputs.get(1), "Point In Curve")?;
    let classification = classify_point_against_polyline(point, &curve);

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_RELATIONSHIP.to_owned(),
        Value::Number(classification.relationship as f64),
    );
    outputs.insert(
        PIN_OUTPUT_POINT_PRIME.to_owned(),
        Value::Point(classification.projected),
    );
    Ok(outputs)
}

// Helper struct voor evaluate_curve
struct CurveSample {
    point: [f64; 3],
    tangent: Option<[f64; 3]>,
    length_along: f64,
    angle: Option<f64>,
}

fn sample_curve(points: &[[f64; 3]], parameter: f64) -> CurveSample {
    let clamped = clamp(parameter, 0.0, 1.0);
    let (point, tangent, length_along) = sample_curve_basic(points, clamped);
    let angle = if points.len() < 3 {
        Some(0.0)
    } else {
        Some(curve_angle(points, clamped))
    };

    CurveSample {
        point,
        tangent,
        length_along,
        angle,
    }
}

fn curve_angle(points: &[[f64; 3]], parameter: f64) -> f64 {
    let dt = 1.0 / (points.len().max(2) as f64 * 8.0);
    let before = clamp(parameter - dt, 0.0, 1.0);
    let after = clamp(parameter + dt, 0.0, 1.0);
    let before_tangent = sample_curve_basic(points, before)
        .1
        .unwrap_or([1.0, 0.0, 0.0]);
    let after_tangent = sample_curve_basic(points, after)
        .1
        .unwrap_or([1.0, 0.0, 0.0]);
    let dot_value = clamp(dot(before_tangent, after_tangent), -1.0, 1.0);
    dot_value.acos()
}

fn sample_curve_basic(points: &[[f64; 3]], parameter: f64) -> ([f64; 3], Option<[f64; 3]>, f64) {
    let segments = polyline_segments(points);
    if segments.is_empty() {
        return (points.get(0).copied().unwrap_or([0.0, 0.0, 0.0]), None, 0.0);
    }

    let total_length: f64 = segments.iter().map(|segment| segment.length).sum();
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

// Structen voor segmentberekeningen
struct PolylineSegment {
    start: [f64; 3],
    end: [f64; 3],
    length: f64,
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

fn is_closed(points: &[[f64; 3]]) -> bool {
    if points.len() < 2 {
        return false;
    }
    distance(points[0], *points.last().unwrap()) < 1e-6
}

struct ClosestPointResult {
    point: [f64; 3],
    parameter: f64,
    distance: f64,
}

fn closest_point_on_polyline(point: [f64; 3], points: &[[f64; 3]]) -> ClosestPointResult {
    let segments = polyline_segments(points);
    if segments.is_empty() {
        return ClosestPointResult {
            point: points.get(0).copied().unwrap_or(point),
            parameter: 0.0,
            distance: 0.0,
        };
    }

    let mut best = ClosestPointResult {
        point: segments[0].start,
        parameter: 0.0,
        distance: f64::INFINITY,
    };
    let mut accumulated = 0.0;
    let total_length: f64 = segments.iter().map(|segment| segment.length).sum();

    for segment in segments {
        let projection = closest_point_on_segment(point, segment.start, segment.end);
        let dist = distance(point, projection.point);
        if dist < best.distance {
            let parameter = if total_length < EPSILON {
                0.0
            } else {
                clamp((accumulated + projection.along) / total_length, 0.0, 1.0)
            };
            best = ClosestPointResult {
                point: projection.point,
                parameter,
                distance: dist,
            };
        }
        accumulated += segment.length;
    }

    best
}

struct SegmentProjection {
    point: [f64; 3],
    along: f64,
}

fn closest_point_on_segment(point: [f64; 3], start: [f64; 3], end: [f64; 3]) -> SegmentProjection {
    let ab = subtract(end, start);
    let ap = subtract(point, start);
    let ab_length_sq = dot(ab, ab);
    if ab_length_sq < EPSILON {
        return SegmentProjection {
            point: start,
            along: 0.0,
        };
    }
    let factor = clamp(dot(ap, ab) / ab_length_sq, 0.0, 1.0);
    SegmentProjection {
        point: add(start, scale(ab, factor)),
        along: distance(start, add(start, scale(ab, factor))),
    }
}

struct ProximityResult {
    point_a: [f64; 3],
    point_b: [f64; 3],
    distance: f64,
}

fn closest_points_between_polylines(a: &[[f64; 3]], b: &[[f64; 3]]) -> ProximityResult {
    let segments_a = polyline_segments(a);
    let segments_b = polyline_segments(b);
    if segments_a.is_empty() || segments_b.is_empty() {
        let point_a = a.get(0).copied().unwrap_or([0.0, 0.0, 0.0]);
        let point_b = b.get(0).copied().unwrap_or([0.0, 0.0, 0.0]);
        return ProximityResult {
            point_a,
            point_b,
            distance: distance(point_a, point_b),
        };
    }
    let mut best = ProximityResult {
        point_a: segments_a
            .first()
            .map(|segment| segment.start)
            .unwrap_or([0.0, 0.0, 0.0]),
        point_b: segments_b
            .first()
            .map(|segment| segment.start)
            .unwrap_or([0.0, 0.0, 0.0]),
        distance: f64::INFINITY,
    };

    for segment_a in &segments_a {
        for segment_b in &segments_b {
            let (pa, pb) = closest_points_on_segments(
                segment_a.start,
                segment_a.end,
                segment_b.start,
                segment_b.end,
            );
            let dist = distance(pa, pb);
            if dist < best.distance {
                best = ProximityResult {
                    point_a: pa,
                    point_b: pb,
                    distance: dist,
                };
            }
        }
    }

    best
}

fn closest_points_on_segments(
    a0: [f64; 3],
    a1: [f64; 3],
    b0: [f64; 3],
    b1: [f64; 3],
) -> ([f64; 3], [f64; 3]) {
    let u = subtract(a1, a0);
    let v = subtract(b1, b0);
    let w0 = subtract(a0, b0);
    let a = dot(u, u);
    let b = dot(u, v);
    let c = dot(v, v);
    let d = dot(u, w0);
    let e = dot(v, w0);
    let denominator = a * c - b * b;

    let (sc, tc) = if denominator.abs() < EPSILON {
        (0.0, clamp(e / c, 0.0, 1.0))
    } else {
        let s = (b * e - c * d) / denominator;
        let t = (a * e - b * d) / denominator;
        (clamp(s, 0.0, 1.0), clamp(t, 0.0, 1.0))
    };

    let point_a = add(a0, scale(u, sc));
    let point_b = add(b0, scale(v, tc));
    (point_a, point_b)
}

fn analyse_planarity(points: &[[f64; 3]]) -> PlanarityAnalysis {
    if points.len() < 3 {
        return PlanarityAnalysis {
            planar: true,
            deviation: 0.0,
            plane_value: Value::List(vec![
                Value::Point(points.get(0).copied().unwrap_or([0.0, 0.0, 0.0])),
                Value::Point([1.0, 0.0, 0.0]),
                Value::Point([0.0, 1.0, 0.0]),
            ]),
        };
    }

    let origin = points[0];
    let mut x_axis = None;
    let mut y_axis = None;

    for point in &points[1..] {
        let vector = subtract(*point, origin);
        if length_squared(vector) > EPSILON {
            if x_axis.is_none() {
                x_axis = Some(vector);
            } else if y_axis.is_none() {
                let candidate = cross(x_axis.unwrap(), vector);
                if length_squared(candidate) > EPSILON {
                    y_axis = Some(vector);
                }
            }
        }
    }

    let x_axis = x_axis.unwrap_or([1.0, 0.0, 0.0]);
    let y_axis = y_axis.unwrap_or([0.0, 1.0, 0.0]);
    let normal = normalize(cross(x_axis, y_axis));

    let mut deviation: f64 = 0.0;
    for point in points {
        let vector = subtract(*point, origin);
        deviation = deviation.max(dot(vector, normal).abs());
    }

    let plane_value = Value::List(vec![
        Value::Point(origin),
        Value::Point(add(origin, normalize(x_axis))),
        Value::Point(add(origin, normalize(y_axis))),
    ]);

    PlanarityAnalysis {
        planar: deviation < 1e-6,
        deviation,
        plane_value,
    }
}

struct PlanarityAnalysis {
    planar: bool,
    deviation: f64,
    plane_value: Value,
}

fn average_points(points: &[[f64; 3]]) -> [f64; 3] {
    if points.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    let mut sum = [0.0, 0.0, 0.0];
    for point in points {
        sum = add(sum, *point);
    }
    scale(sum, 1.0 / points.len() as f64)
}

fn average_edges(points: &[[f64; 3]]) -> [f64; 3] {
    let segments = polyline_segments(points);
    if segments.is_empty() {
        return average_points(points);
    }
    let mut sum = [0.0, 0.0, 0.0];
    for segment in &segments {
        let midpoint = scale(add(segment.start, segment.end), 0.5);
        sum = add(sum, midpoint);
    }
    scale(sum, 1.0 / segments.len() as f64)
}

fn polygon_area_centroid(points: &[[f64; 3]]) -> Option<[f64; 3]> {
    if points.len() < 3 {
        return None;
    }
    let mut area = 0.0;
    let mut centroid = [0.0, 0.0, 0.0];
    for window in points.windows(2) {
        let (a, b) = (window[0], window[1]);
        let cross = a[0] * b[1] - b[0] * a[1];
        area += cross;
        centroid[0] += (a[0] + b[0]) * cross;
        centroid[1] += (a[1] + b[1]) * cross;
        centroid[2] += (a[2] + b[2]) * cross;
    }

    if !is_closed(points) {
        let last = *points.last().unwrap();
        let first = points[0];
        let cross = last[0] * first[1] - first[0] * last[1];
        area += cross;
        centroid[0] += (last[0] + first[0]) * cross;
        centroid[1] += (last[1] + first[1]) * cross;
        centroid[2] += (last[2] + first[2]) * cross;
    }

    if area.abs() < EPSILON {
        return None;
    }

    Some(scale(centroid, 1.0 / (3.0 * area)))
}

fn coerce_polyline(value: Option<&Value>, context: &str) -> Result<Vec<[f64; 3]>, ComponentError> {
    let Some(value) = value else {
        return Err(ComponentError::new(format!(
            "{} vereist minimaal één curve-invoer",
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
            "{} vereist een lijst van curves",
            context
        )));
    };

    match value {
        Value::List(values) => {
            if values.is_empty() {
                return Err(ComponentError::new(format!(
                    "{} vereist minstens één curve",
                    context
                )));
            }

            if values.iter().all(|entry| matches!(entry, Value::List(_))) {
                let mut curves = Vec::with_capacity(values.len());
                for entry in values {
                    curves.push(coerce_polyline(Some(entry), context)?);
                }
                Ok(curves)
            } else {
                Ok(vec![coerce_polyline(Some(value), context)?])
            }
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

struct PointContainment {
    relationship: i32,
    projected: [f64; 3],
}

fn classify_point_against_polyline(point: [f64; 3], polyline: &[[f64; 3]]) -> PointContainment {
    if polyline.len() < 2 {
        return PointContainment {
            relationship: 0,
            projected: point,
        };
    }

    let average_z = polyline.iter().map(|pt| pt[2]).sum::<f64>() / polyline.len() as f64;
    let mut inside = false;
    let mut on_edge = false;
    let mut previous = *polyline.last().unwrap();

    for &current in polyline {
        if point_on_segment_2d(point, previous, current) {
            on_edge = true;
        }

        let yi = previous[1];
        let yj = current[1];
        let intersects = ((yi > point[1]) != (yj > point[1])) && {
            let dy = yj - yi;
            let x_intersection = if dy.abs() < EPSILON {
                current[0]
            } else {
                (current[0] - previous[0]) * (point[1] - yi) / dy + previous[0]
            };
            x_intersection > point[0]
        };

        if intersects {
            inside = !inside;
        }

        previous = current;
    }

    let relationship = if on_edge {
        1
    } else if inside {
        2
    } else {
        0
    };

    PointContainment {
        relationship,
        projected: [point[0], point[1], average_z],
    }
}

fn point_on_segment_2d(point: [f64; 3], start: [f64; 3], end: [f64; 3]) -> bool {
    let seg = [end[0] - start[0], end[1] - start[1]];
    let to_point = [point[0] - start[0], point[1] - start[1]];
    let seg_length_sq = seg[0] * seg[0] + seg[1] * seg[1];
    if seg_length_sq < EPSILON {
        return ((point[0] - start[0]).abs() < EPSILON) && ((point[1] - start[1]).abs() < EPSILON);
    }

    let cross = seg[0] * to_point[1] - seg[1] * to_point[0];
    if cross.abs() > 1e-6 {
        return false;
    }

    let dot = to_point[0] * seg[0] + to_point[1] * seg[1];
    if dot < -1e-6 || dot > seg_length_sq + 1e-6 {
        return false;
    }

    true
}

fn coerce_number(value: Option<&Value>, context: &str) -> Result<f64, ComponentError> {
    let Some(value) = value else {
        return Err(ComponentError::new(format!(
            "{} vereist een numerieke parameter",
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

const EPSILON: f64 = 1e-9;

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
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

fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    length(subtract(a, b))
}

fn length(vector: [f64; 3]) -> f64 {
    dot(vector, vector).sqrt()
}

fn length_squared(vector: [f64; 3]) -> f64 {
    dot(vector, vector)
}

fn normalize(vector: [f64; 3]) -> [f64; 3] {
    safe_normalized(vector)
        .map(|(v, _)| v)
        .unwrap_or([1.0, 0.0, 0.0])
}

fn orthogonal_vector(vector: [f64; 3]) -> [f64; 3] {
    if vector[0].abs() > vector[1].abs() {
        normalize([-vector[2], 0.0, vector[0]])
    } else {
        normalize([0.0, vector[2], -vector[1]])
    }
}

fn safe_normalized(vector: [f64; 3]) -> Option<([f64; 3], f64)> {
    let length = length(vector);
    if length < EPSILON {
        None
    } else {
        Some((scale(vector, 1.0 / length), length))
    }
}

fn lerp(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    add(a, scale(subtract(b, a), t))
}