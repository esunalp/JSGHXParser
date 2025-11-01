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
    CurveClosestPoint,
    Closed,
    ControlPointsDetailed,
    ControlPointsSimple,
    ControlPolygon,
    Planar,
    PolygonCenterDetailed,
    PolygonCenterEdge,
    PolygonCenterSimple,
    EvaluateLength,
    LengthParameter,
    Length,
    CurveMiddle,
    SegmentLengths,
    CurveProximity,
    ArcCenter,
    PointInCurve,
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
        guids: &["{afff17ed-5975-460b-9883-525ae0677088}"],
        names: &["Arc Center", "CrvCenter"],
        kind: ComponentKind::ArcCenter,
    },
    Registration {
        guids: &["{a72b0bd3-c7a7-458e-875d-09ae1624638c}"],
        names: &["Point In Curve", "InCurve"],
        kind: ComponentKind::PointInCurve,
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
            Self::EvaluateLength => evaluate_length(inputs),
            Self::LengthParameter => evaluate_length_parameter(inputs),
            Self::Length => evaluate_curve_length(inputs),
            Self::CurveMiddle => evaluate_curve_middle(inputs),
            Self::SegmentLengths => evaluate_segment_lengths(inputs),
            Self::CurveProximity => evaluate_curve_proximity(inputs),
            Self::ArcCenter => evaluate_arc_center(inputs),
            Self::PointInCurve => evaluate_point_in_curve(inputs),
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
            Self::CurveClosestPoint => "Curve Closest Point",
            Self::Closed => "Curve Closed",
            Self::ControlPointsDetailed => "Control Points Detailed",
            Self::ControlPointsSimple => "Control Points",
            Self::ControlPolygon => "Control Polygon",
            Self::Planar => "Curve Planar",
            Self::PolygonCenterDetailed => "Polygon Center",
            Self::PolygonCenterEdge => "Polygon Center Edge",
            Self::PolygonCenterSimple => "Polygon Center",
            Self::EvaluateLength => "Evaluate Curve Length Factor",
            Self::LengthParameter => "Length Parameter",
            Self::Length => "Curve Length",
            Self::CurveMiddle => "Curve Midpoint",
            Self::SegmentLengths => "Segment Lengths",
            Self::CurveProximity => "Curve Proximity",
            Self::ArcCenter => "Curve Center",
            Self::PointInCurve => "Point In Curve",
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

fn classify_point_against_polyline(
    point: [f64; 3],
    polyline: &[[f64; 3]],
) -> PointContainment {
    if polyline.len() < 2 {
        return PointContainment {
            relationship: 0,
            projected: point,
        };
    }

    let average_z = polyline
        .iter()
        .map(|pt| pt[2])
        .sum::<f64>()
        / polyline.len() as f64;
    let mut inside = false;
    let mut on_edge = false;
    let mut previous = *polyline.last().unwrap();

    for &current in polyline {
        if point_on_segment_2d(point, previous, current) {
            on_edge = true;
        }

        let yi = previous[1];
        let yj = current[1];
        let intersects = ((yi > point[1]) != (yj > point[1]))
            && {
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
        return ((point[0] - start[0]).abs() < EPSILON)
            && ((point[1] - start[1]).abs() < EPSILON);
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

#[cfg(test)]
mod tests {
    use super::{
        Component, ComponentKind, EPSILON, PIN_OUTPUT_CENTER_AREA, PIN_OUTPUT_CENTER_EDGE,
        PIN_OUTPUT_CENTER_VERTEX, PIN_OUTPUT_DOMAIN, PIN_OUTPUT_END, PIN_OUTPUT_INDEX,
        PIN_OUTPUT_LENGTH, PIN_OUTPUT_PARAMETER, PIN_OUTPUT_POINT, PIN_OUTPUT_POINT_PRIME,
        PIN_OUTPUT_RELATIONSHIP, PIN_OUTPUT_START, PIN_OUTPUT_TANGENT,
    };
    use crate::graph::node::MetaMap;
    use crate::graph::value::{Domain, Value};

    #[test]
    fn end_points_returns_first_and_last() {
        let outputs = ComponentKind::EndPoints
            .evaluate(
                &[Value::List(vec![
                    Value::Point([0.0, 0.0, 0.0]),
                    Value::Point([1.0, 0.0, 0.0]),
                    Value::Point([2.0, 1.0, 0.0]),
                ])],
                &MetaMap::new(),
            )
            .expect("end points");

        assert!(matches!(
            outputs.get(PIN_OUTPUT_START),
            Some(Value::Point(point)) if *point == [0.0, 0.0, 0.0]
        ));
        assert!(matches!(
            outputs.get(PIN_OUTPUT_END),
            Some(Value::Point(point)) if *point == [2.0, 1.0, 0.0]
        ));
    }

    #[test]
    fn curve_domain_reports_length() {
        let outputs = ComponentKind::CurveDomain
            .evaluate(
                &[Value::List(vec![
                    Value::Point([0.0, 0.0, 0.0]),
                    Value::Point([1.0, 0.0, 0.0]),
                ])],
                &MetaMap::new(),
            )
            .expect("domain");

        let Value::Domain(Domain::One(domain)) = outputs.get(PIN_OUTPUT_DOMAIN).unwrap() else {
            panic!("expected 1D domain");
        };
        assert_eq!(domain.start, 0.0);
        assert_eq!(domain.end, 1.0);
    }

    #[test]
    fn evaluate_curve_returns_point_and_tangent() {
        let outputs = ComponentKind::EvaluateCurveBasic
            .evaluate(
                &[
                    Value::List(vec![
                        Value::Point([0.0, 0.0, 0.0]),
                        Value::Point([1.0, 0.0, 0.0]),
                        Value::Point([1.0, 1.0, 0.0]),
                    ]),
                    Value::Number(0.25),
                ],
                &MetaMap::new(),
            )
            .expect("evaluate curve");

        assert!(outputs.contains_key(PIN_OUTPUT_POINT));
        assert!(outputs.contains_key(PIN_OUTPUT_TANGENT));
    }

    #[test]
    fn length_component_sums_segments() {
        let outputs = ComponentKind::Length
            .evaluate(
                &[Value::List(vec![
                    Value::Point([0.0, 0.0, 0.0]),
                    Value::Point([1.0, 0.0, 0.0]),
                    Value::Point([1.0, 1.0, 0.0]),
                ])],
                &MetaMap::new(),
            )
            .expect("length");

        assert!(matches!(
            outputs.get(PIN_OUTPUT_LENGTH),
            Some(Value::Number(length)) if (*length - 2.0).abs() < 1e-9
        ));
    }

    #[test]
    fn polygon_center_reports_averages() {
        let outputs = ComponentKind::PolygonCenterDetailed
            .evaluate(
                &[Value::List(vec![
                    Value::Point([0.0, 0.0, 0.0]),
                    Value::Point([1.0, 0.0, 0.0]),
                    Value::Point([1.0, 1.0, 0.0]),
                    Value::Point([0.0, 1.0, 0.0]),
                    Value::Point([0.0, 0.0, 0.0]),
                ])],
                &MetaMap::new(),
            )
            .expect("polygon center");

        assert!(outputs.contains_key(PIN_OUTPUT_CENTER_VERTEX));
        assert!(outputs.contains_key(PIN_OUTPUT_CENTER_EDGE));
        assert!(outputs.contains_key(PIN_OUTPUT_CENTER_AREA));
    }

    #[test]
    fn evaluate_length_uses_factor() {
        let outputs = ComponentKind::EvaluateLength
            .evaluate(
                &[
                    Value::List(vec![
                        Value::Point([0.0, 0.0, 0.0]),
                        Value::Point([2.0, 0.0, 0.0]),
                    ]),
                    Value::Number(1.0),
                    Value::Boolean(false),
                ],
                &MetaMap::new(),
            )
            .expect("evaluate length");

        assert!(matches!(
            outputs.get(PIN_OUTPUT_PARAMETER),
            Some(Value::Number(param)) if (*param - 0.5).abs() < EPSILON
        ));
    }

    #[test]
    fn point_in_curves_prefers_first_containing_region() {
        let square = Value::List(vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Point([1.0, 1.0, 0.0]),
            Value::Point([0.0, 1.0, 0.0]),
            Value::Point([0.0, 0.0, 0.0]),
        ]);
        let rectangle = Value::List(vec![
            Value::Point([2.0, 0.0, 0.0]),
            Value::Point([3.0, 0.0, 0.0]),
            Value::Point([3.0, 1.0, 0.0]),
            Value::Point([2.0, 1.0, 0.0]),
            Value::Point([2.0, 0.0, 0.0]),
        ]);

        let outputs = ComponentKind::PointInCurves
            .evaluate(
                &[
                    Value::Point([0.25, 0.25, 0.0]),
                    Value::List(vec![square, rectangle]),
                ],
                &MetaMap::new(),
            )
            .expect("point in curves");

        assert!(matches!(
            outputs.get(PIN_OUTPUT_RELATIONSHIP),
            Some(Value::Number(value)) if (*value - 2.0).abs() < EPSILON
        ));
        assert!(matches!(
            outputs.get(PIN_OUTPUT_INDEX),
            Some(Value::Number(value)) if value.abs() < EPSILON
        ));
    }

    #[test]
    fn point_in_curve_reports_outside() {
        let curve = Value::List(vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Point([1.0, 1.0, 0.0]),
            Value::Point([0.0, 1.0, 0.0]),
            Value::Point([0.0, 0.0, 0.0]),
        ]);

        let outputs = ComponentKind::PointInCurve
            .evaluate(
                &[Value::Point([2.0, 2.0, 0.0]), curve],
                &MetaMap::new(),
            )
            .expect("point in curve");

        assert!(matches!(
            outputs.get(PIN_OUTPUT_RELATIONSHIP),
            Some(Value::Number(value)) if value.abs() < EPSILON
        ));
        assert!(matches!(
            outputs.get(PIN_OUTPUT_POINT_PRIME),
            Some(Value::Point(pt)) if (pt[0] - 2.0).abs() < EPSILON && (pt[1] - 2.0).abs() < EPSILON
        ));
    }
}
