//! Implementaties van Grasshopper "Surface → Util" componenten.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::{Domain, Domain1D, Domain2D, Value};

use super::{Component, ComponentError, ComponentResult};

const PIN_OUTPUT_POINTS: &str = "P";
const PIN_OUTPUT_NORMALS: &str = "N";
const PIN_OUTPUT_PARAMETERS: &str = "uv";
const PIN_OUTPUT_FRAMES: &str = "F";
const PIN_OUTPUT_BREPS: &str = "B";
const PIN_OUTPUT_CLOSED: &str = "C";
const PIN_OUTPUT_RESULT: &str = "R";
const PIN_OUTPUT_MAP: &str = "M";
const PIN_OUTPUT_CLOSED_INDICES: &str = "Ci";
const PIN_OUTPUT_OPEN: &str = "O";
const PIN_OUTPUT_OPEN_INDICES: &str = "Oi";
const PIN_OUTPUT_INDICES: &str = "I";
const PIN_OUTPUT_CONVEX: &str = "Cv";
const PIN_OUTPUT_CONCAVE: &str = "Cc";
const PIN_OUTPUT_MIXED: &str = "Mx";
const PIN_OUTPUT_BEFORE: &str = "N0";
const PIN_OUTPUT_AFTER: &str = "N1";
const PIN_OUTPUT_CAPS: &str = "C";
const PIN_OUTPUT_SOLID: &str = "S";

const EPSILON: f64 = 1e-9;

/// Beschikbare componenten binnen Surface → Util.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    DivideSurfaceObsolete,
    BrepJoin,
    SurfaceFrames,
    FilletEdge,
    DivideSurface,
    SurfaceFramesObsolete,
    CopyTrim,
    EdgesFromDirections,
    Isotrim,
    ClosedEdges,
    EdgesFromFaces,
    EdgesFromPoints,
    ConvexEdges,
    Retrim,
    OffsetSurface,
    CapHoles,
    Flip,
    MergeFaces,
    EdgesFromLinearity,
    OffsetSurfaceLoose,
    CapHolesEx,
    Untrim,
    EdgesFromLength,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Registraties van alle Surface → Util componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{082976f0-c741-4df8-a1d4-89891bf8619f}"],
        names: &["Divide Surface [OBSOLETE]", "Divide"],
        kind: ComponentKind::DivideSurfaceObsolete,
    },
    Registration {
        guids: &["{1addcc85-b04e-46e6-bd4a-6f6c93bf7efd}"],
        names: &["Brep Join", "Join"],
        kind: ComponentKind::BrepJoin,
    },
    Registration {
        guids: &["{332378f4-acb2-43fe-8593-ed22bfeb2721}"],
        names: &["Surface Frames", "SFrames"],
        kind: ComponentKind::SurfaceFrames,
    },
    Registration {
        guids: &["{4b87eb13-f87c-4ff1-ae0e-6c9f1f2aecbd}"],
        names: &["Fillet Edge", "FilEdge"],
        kind: ComponentKind::FilletEdge,
    },
    Registration {
        guids: &["{5106bafc-d5d4-4983-83e7-7be3ed07f502}"],
        names: &["Divide Surface", "SDivide"],
        kind: ComponentKind::DivideSurface,
    },
    Registration {
        guids: &["{59143f40-32f3-47c1-b9ae-1a09eb9c926b}"],
        names: &["Surface Frames [OBSOLETE]", "Frames"],
        kind: ComponentKind::SurfaceFramesObsolete,
    },
    Registration {
        guids: &["{5d192b90-1ae3-4439-bbde-b05976fc4ac3}"],
        names: &["Copy Trim", "Trim"],
        kind: ComponentKind::CopyTrim,
    },
    Registration {
        guids: &["{64ff9813-8fe8-4708-ac9f-61b825213e83}"],
        names: &["Edges from Directions", "EdgesDir"],
        kind: ComponentKind::EdgesFromDirections,
    },
    Registration {
        guids: &["{6a9ccaab-1b03-484e-bbda-be9c81584a66}"],
        names: &["Isotrim", "SubSrf"],
        kind: ComponentKind::Isotrim,
    },
    Registration {
        guids: &["{70905be1-e22f-4fa8-b9ae-e119d417904f}"],
        names: &["Closed Edges", "EdgesCls"],
        kind: ComponentKind::ClosedEdges,
    },
    Registration {
        guids: &["{71e99dbb-2d79-4f02-a8a6-e87a09d54f47}"],
        names: &["Edges from Faces", "EdgesFaces"],
        kind: ComponentKind::EdgesFromFaces,
    },
    Registration {
        guids: &["{73269f6a-9645-4638-8d5e-88064dd289bd}"],
        names: &["Edges from Points", "EdgesPt"],
        kind: ComponentKind::EdgesFromPoints,
    },
    Registration {
        guids: &["{8248da39-0729-4e04-8395-267b3259bc2f}"],
        names: &["Convex Edges", "EdgesCvx"],
        kind: ComponentKind::ConvexEdges,
    },
    Registration {
        guids: &["{a1da39b7-6387-4522-bf2b-2eaee6b14072}"],
        names: &["Retrim", "Retrim"],
        kind: ComponentKind::Retrim,
    },
    Registration {
        guids: &["{b25c5762-f90e-4839-9fc5-74b74ab42b1e}"],
        names: &["Offset Surface", "Offset"],
        kind: ComponentKind::OffsetSurface,
    },
    Registration {
        guids: &["{b648d933-ddea-4e75-834c-8f6f3793e311}"],
        names: &["Cap Holes", "Cap"],
        kind: ComponentKind::CapHoles,
    },
    Registration {
        guids: &["{c3d1f2b8-8596-4e8d-8861-c28ba8ffb4f4}"],
        names: &["Flip", "Flip"],
        kind: ComponentKind::Flip,
    },
    Registration {
        guids: &["{d6b43673-55dd-4e2f-95c4-6c69a14513a6}"],
        names: &["Merge Faces", "FMerge"],
        kind: ComponentKind::MergeFaces,
    },
    Registration {
        guids: &["{e4ff8101-73c9-4802-8c5d-704d8721b909}"],
        names: &["Edges from Linearity", "EdgesLin"],
        kind: ComponentKind::EdgesFromLinearity,
    },
    Registration {
        guids: &["{e7e43403-f913-4d83-8aff-5b1c7a7f9fbc}"],
        names: &["Offset Surface Loose", "Offset (L)"],
        kind: ComponentKind::OffsetSurfaceLoose,
    },
    Registration {
        guids: &["{f6409a9c-3d2a-4b14-9f2c-e3c3f2cb72f8}"],
        names: &["Cap Holes Ex", "CapEx"],
        kind: ComponentKind::CapHolesEx,
    },
    Registration {
        guids: &["{fa92858a-a180-4545-ad4d-0dc644b3a2a8}"],
        names: &["Untrim", "Untrim"],
        kind: ComponentKind::Untrim,
    },
    Registration {
        guids: &["{ff187e6a-84bc-4bb9-a572-b39006a0576d}"],
        names: &["Edges from Length", "EdgesLen"],
        kind: ComponentKind::EdgesFromLength,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::DivideSurfaceObsolete | Self::DivideSurface => {
                evaluate_divide_surface(inputs, self.name())
            }
            Self::SurfaceFrames | Self::SurfaceFramesObsolete => {
                evaluate_surface_frames(inputs, self.name())
            }
            Self::BrepJoin => evaluate_brep_join(inputs),
            Self::FilletEdge => evaluate_fillet_edge(inputs),
            Self::CopyTrim | Self::Retrim => evaluate_copy_trim(inputs, self.name()),
            Self::EdgesFromDirections => evaluate_edges_from_directions(inputs),
            Self::Isotrim => evaluate_isotrim(inputs),
            Self::ClosedEdges => evaluate_closed_edges(inputs),
            Self::EdgesFromFaces => evaluate_edges_from_faces(inputs),
            Self::EdgesFromPoints => evaluate_edges_from_points(inputs),
            Self::ConvexEdges => evaluate_convex_edges(inputs),
            Self::OffsetSurface | Self::OffsetSurfaceLoose => {
                evaluate_offset_surface(inputs, self.name())
            }
            Self::CapHoles => evaluate_cap_holes(inputs, false),
            Self::CapHolesEx => evaluate_cap_holes(inputs, true),
            Self::Flip => evaluate_flip(inputs),
            Self::MergeFaces => evaluate_merge_faces(inputs),
            Self::EdgesFromLinearity => evaluate_edges_by_length(inputs, "Edges from Linearity"),
            Self::EdgesFromLength => evaluate_edges_by_length(inputs, "Edges from Length"),
            Self::Untrim => evaluate_untrim(inputs),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::DivideSurfaceObsolete => "Divide Surface [OBSOLETE]",
            Self::BrepJoin => "Brep Join",
            Self::SurfaceFrames => "Surface Frames",
            Self::FilletEdge => "Fillet Edge",
            Self::DivideSurface => "Divide Surface",
            Self::SurfaceFramesObsolete => "Surface Frames [OBSOLETE]",
            Self::CopyTrim => "Copy Trim",
            Self::EdgesFromDirections => "Edges from Directions",
            Self::Isotrim => "Isotrim",
            Self::ClosedEdges => "Closed Edges",
            Self::EdgesFromFaces => "Edges from Faces",
            Self::EdgesFromPoints => "Edges from Points",
            Self::ConvexEdges => "Convex Edges",
            Self::Retrim => "Retrim",
            Self::OffsetSurface => "Offset Surface",
            Self::CapHoles => "Cap Holes",
            Self::Flip => "Flip",
            Self::MergeFaces => "Merge Faces",
            Self::EdgesFromLinearity => "Edges from Linearity",
            Self::OffsetSurfaceLoose => "Offset Surface Loose",
            Self::CapHolesEx => "Cap Holes Ex",
            Self::Untrim => "Untrim",
            Self::EdgesFromLength => "Edges from Length",
        }
    }
}

fn evaluate_divide_surface(inputs: &[Value], component: &str) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(format!(
            "{} vereist een surface en segmentaantallen",
            component
        )));
    }

    let metrics = coerce_shape_metrics(inputs.get(0), component)?;
    let u_segments = coerce_positive_integer(inputs.get(1), &(component.to_owned() + " U"))?;
    let v_segments = coerce_positive_integer(inputs.get(2), &(component.to_owned() + " V"))?;

    let mut points = Vec::new();
    let mut normals = Vec::new();
    let mut parameters = Vec::new();

    for v in 0..=v_segments {
        let fv = if v_segments == 0 {
            0.0
        } else {
            v as f64 / v_segments as f64
        };
        for u in 0..=u_segments {
            let fu = if u_segments == 0 {
                0.0
            } else {
                u as f64 / u_segments as f64
            };
            let point = metrics.sample_point((fu, fv));
            points.push(Value::Point(point));
            normals.push(Value::Vector(metrics.normal_hint()));
            parameters.push(Value::Point([fu, fv, 0.0]));
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::List(points));
    outputs.insert(PIN_OUTPUT_NORMALS.to_owned(), Value::List(normals));
    outputs.insert(PIN_OUTPUT_PARAMETERS.to_owned(), Value::List(parameters));
    Ok(outputs)
}

fn evaluate_surface_frames(inputs: &[Value], component: &str) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(format!(
            "{} vereist een surface en segmentaantallen",
            component
        )));
    }

    let metrics = coerce_shape_metrics(inputs.get(0), component)?;
    let u_segments = coerce_positive_integer(inputs.get(1), &(component.to_owned() + " U"))?;
    let v_segments = coerce_positive_integer(inputs.get(2), &(component.to_owned() + " V"))?;

    let mut frames_rows = Vec::new();
    let mut parameter_rows = Vec::new();

    for v in 0..=v_segments {
        let fv = if v_segments == 0 {
            0.0
        } else {
            v as f64 / v_segments as f64
        };
        let mut frames_row = Vec::new();
        let mut parameters_row = Vec::new();
        for u in 0..=u_segments {
            let fu = if u_segments == 0 {
                0.0
            } else {
                u as f64 / u_segments as f64
            };
            let point = metrics.sample_point((fu, fv));
            let tangent_u = metrics.tangent_hint_u();
            let tangent_v = metrics.tangent_hint_v();
            let normal = metrics.normal_hint();
            frames_row.push(frame_value(point, tangent_u, tangent_v, normal));
            parameters_row.push(Value::Point([fu, fv, 0.0]));
        }
        frames_rows.push(Value::List(frames_row));
        parameter_rows.push(Value::List(parameters_row));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_FRAMES.to_owned(), Value::List(frames_rows));
    outputs.insert(
        PIN_OUTPUT_PARAMETERS.to_owned(),
        Value::List(parameter_rows),
    );
    Ok(outputs)
}

fn evaluate_brep_join(inputs: &[Value]) -> ComponentResult {
    let mut breps = Vec::new();
    let mut closed = Vec::new();

    if let Some(Value::List(values)) = inputs.get(0) {
        for value in values {
            breps.push(value.clone());
            let metrics = ShapeMetrics::from_inputs(Some(value));
            let is_closed = metrics
                .as_ref()
                .map(|m| m.volume().abs() > EPSILON)
                .unwrap_or(false);
            closed.push(Value::Boolean(is_closed));
        }
    } else if let Some(value) = inputs.get(0) {
        breps.push(value.clone());
        let metrics = ShapeMetrics::from_inputs(Some(value));
        let is_closed = metrics
            .as_ref()
            .map(|m| m.volume().abs() > EPSILON)
            .unwrap_or(false);
        closed.push(Value::Boolean(is_closed));
    }

    if breps.is_empty() {
        return Err(ComponentError::new("Brep Join vereist een lijst met breps"));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), Value::List(breps));
    outputs.insert(PIN_OUTPUT_CLOSED.to_owned(), Value::List(closed));
    Ok(outputs)
}

fn evaluate_fillet_edge(inputs: &[Value]) -> ComponentResult {
    let shape = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Fillet Edge vereist geometrie"))?;

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), shape);
    Ok(outputs)
}

fn evaluate_copy_trim(inputs: &[Value], component: &str) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(format!(
            "{} vereist zowel een bron- als doelsurface",
            component
        )));
    }

    let source = coerce_shape_metrics(inputs.get(0), component)?;
    let target = coerce_shape_metrics(inputs.get(1), component)?;
    let min = [
        source.min[0].max(target.min[0]),
        source.min[1].max(target.min[1]),
        source.min[2].max(target.min[2]),
    ];
    let max = [
        source.max[0].min(target.max[0]),
        source.max[1].min(target.max[1]),
        source.max[2].min(target.max[2]),
    ];

    let surface = if min[0] <= max[0] && min[1] <= max[1] && min[2] <= max[2] {
        create_surface_from_bounds(min, max)
    } else {
        create_surface_from_bounds(target.min, target.max)
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), surface);
    Ok(outputs)
}

fn evaluate_edges_from_directions(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 4 {
        return Err(ComponentError::new(
            "Edges from Directions vereist een brep, richtingen en toleranties",
        ));
    }

    let metrics = coerce_shape_metrics(inputs.get(0), "Edges from Directions")?;
    let directions = parse_directions(inputs.get(1));
    if directions.is_empty() {
        return Err(ComponentError::new(
            "Edges from Directions vereist minstens één richting",
        ));
    }
    let reflex = coerce_boolean(inputs.get(2), false)?;
    let tolerance = coerce_number(inputs.get(3), "Edges from Directions hoek")?
        .to_radians()
        .abs();

    let edges = create_wireframe(&metrics);
    let mut selected = Vec::new();
    let mut indices = Vec::new();
    let mut mapping = Vec::new();

    for (index, edge) in edges.iter().enumerate() {
        if let Some((direction, _)) = normalize(edge_direction(edge)) {
            let mut matched = None;
            for (dir_index, candidate) in directions.iter().enumerate() {
                let (candidate, _) = normalize(*candidate).unwrap_or(([1.0, 0.0, 0.0], 1.0));
                let dot = clamp(dot(direction, candidate), -1.0, 1.0);
                let angle = dot.acos();
                if angle <= tolerance || (reflex && (std::f64::consts::PI - angle) <= tolerance) {
                    matched = Some(dir_index);
                    break;
                }
            }
            if let Some(dir_index) = matched {
                selected.push(Value::CurveLine {
                    p1: edge.start,
                    p2: edge.end,
                });
                indices.push(Value::Number(index as f64));
                mapping.push(Value::Number(dir_index as f64));
            }
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), Value::List(selected));
    outputs.insert(PIN_OUTPUT_INDICES.to_owned(), Value::List(indices));
    outputs.insert(PIN_OUTPUT_MAP.to_owned(), Value::List(mapping));
    Ok(outputs)
}

fn evaluate_isotrim(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Isotrim vereist een surface en domein"));
    }

    let metrics = coerce_shape_metrics(inputs.get(0), "Isotrim")?;
    let domain_value = inputs
        .get(1)
        .ok_or_else(|| ComponentError::new("Isotrim vereist een domein"))?;
    let (u_range, v_range) = coerce_domain_pair(domain_value, "Isotrim")?;
    let mut min = metrics.min;
    let mut max = metrics.max;
    min[0] = metrics.min[0] + metrics.size()[0] * clamp01(u_range.0);
    max[0] = metrics.min[0] + metrics.size()[0] * clamp01(u_range.1);
    min[1] = metrics.min[1] + metrics.size()[1] * clamp01(v_range.0);
    max[1] = metrics.min[1] + metrics.size()[1] * clamp01(v_range.1);

    let surface = create_surface_from_bounds(min, max);
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), surface);
    Ok(outputs)
}

fn evaluate_closed_edges(inputs: &[Value]) -> ComponentResult {
    let metrics = coerce_shape_metrics(inputs.get(0), "Closed Edges")?;
    let _tangency = coerce_boolean(inputs.get(1), true).unwrap_or(true);
    let edges = create_wireframe(&metrics);

    let closed_edges: Vec<_> = edges
        .iter()
        .enumerate()
        .map(|(index, edge)| {
            (
                Value::CurveLine {
                    p1: edge.start,
                    p2: edge.end,
                },
                Value::Number(index as f64),
            )
        })
        .collect();

    let closed_list = closed_edges.iter().map(|(edge, _)| edge.clone()).collect();
    let closed_indices = closed_edges
        .iter()
        .map(|(_, index)| index.clone())
        .collect();

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CLOSED.to_owned(), Value::List(closed_list));
    outputs.insert(
        PIN_OUTPUT_CLOSED_INDICES.to_owned(),
        Value::List(closed_indices),
    );
    outputs.insert(PIN_OUTPUT_OPEN.to_owned(), Value::List(Vec::new()));
    outputs.insert(PIN_OUTPUT_OPEN_INDICES.to_owned(), Value::List(Vec::new()));
    Ok(outputs)
}

fn evaluate_edges_from_faces(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Edges from Faces vereist een brep en punten",
        ));
    }
    let metrics = coerce_shape_metrics(inputs.get(0), "Edges from Faces")?;
    let points = collect_point_list(inputs.get(1));
    let tolerance = 1e-3;

    let edges = create_wireframe(&metrics);
    let mut selected = Vec::new();
    let mut indices = Vec::new();

    for (index, edge) in edges.iter().enumerate() {
        let include = points.iter().any(|point| {
            distance(&edge.start, point) <= tolerance || distance(&edge.end, point) <= tolerance
        });
        if include || points.is_empty() {
            selected.push(Value::CurveLine {
                p1: edge.start,
                p2: edge.end,
            });
            indices.push(Value::Number(index as f64));
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), Value::List(selected));
    outputs.insert(PIN_OUTPUT_INDICES.to_owned(), Value::List(indices));
    Ok(outputs)
}

fn evaluate_edges_from_points(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Edges from Points vereist een brep en punten",
        ));
    }
    let metrics = coerce_shape_metrics(inputs.get(0), "Edges from Points")?;
    let points = collect_point_list(inputs.get(1));
    if points.is_empty() {
        return Err(ComponentError::new(
            "Edges from Points vereist minstens één punt",
        ));
    }
    let valence = coerce_positive_integer(inputs.get(2), "Edges from Points valentie")?;
    let tolerance = inputs
        .get(3)
        .map(|value| coerce_number(Some(value), "Edges from Points tolerantie"))
        .transpose()?;
    let tolerance = tolerance.unwrap_or(0.25).abs();

    let edges = create_wireframe(&metrics);
    let mut selected = Vec::new();
    let mut indices = Vec::new();
    let mut mapping = vec![0usize; points.len()];

    for (index, edge) in edges.iter().enumerate() {
        let mut matched_points = Vec::new();
        for (point_index, point) in points.iter().enumerate() {
            if distance(&edge.start, point) <= tolerance || distance(&edge.end, point) <= tolerance
            {
                matched_points.push(point_index);
            }
        }
        if matched_points.len() >= valence {
            selected.push(Value::CurveLine {
                p1: edge.start,
                p2: edge.end,
            });
            indices.push(Value::Number(index as f64));
            for point_index in matched_points {
                mapping[point_index] += 1;
            }
        }
    }

    let map_values = mapping
        .into_iter()
        .map(|count| Value::Number(count as f64))
        .collect();

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), Value::List(selected));
    outputs.insert(PIN_OUTPUT_INDICES.to_owned(), Value::List(indices));
    outputs.insert(PIN_OUTPUT_MAP.to_owned(), Value::List(map_values));
    Ok(outputs)
}

fn evaluate_convex_edges(inputs: &[Value]) -> ComponentResult {
    let metrics = coerce_shape_metrics(inputs.get(0), "Convex Edges")?;
    let edges = create_wireframe(&metrics);

    let mut convex = Vec::new();
    let mut concave = Vec::new();
    let mut mixed = Vec::new();

    for edge in edges {
        let direction = edge_direction(&edge);
        let value = Value::CurveLine {
            p1: edge.start,
            p2: edge.end,
        };
        if direction[2] >= 0.0 {
            convex.push(value);
        } else if direction[2] <= 0.0 {
            concave.push(value);
        } else {
            mixed.push(value);
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CONVEX.to_owned(), Value::List(convex));
    outputs.insert(PIN_OUTPUT_CONCAVE.to_owned(), Value::List(concave));
    outputs.insert(PIN_OUTPUT_MIXED.to_owned(), Value::List(mixed));
    Ok(outputs)
}

fn evaluate_offset_surface(inputs: &[Value], component: &str) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(format!(
            "{} vereist een surface en afstand",
            component
        )));
    }
    let metrics = coerce_shape_metrics(inputs.get(0), component)?;
    let distance = coerce_number(inputs.get(1), &(component.to_owned() + " afstand"))?;

    let mut min = metrics.min;
    let mut max = metrics.max;
    min[2] -= distance.abs();
    max[2] += distance.abs();

    let surface = create_surface_from_bounds(min, max);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), surface);
    Ok(outputs)
}

fn evaluate_cap_holes(inputs: &[Value], extended: bool) -> ComponentResult {
    let brep = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Cap Holes vereist een brep"))?;
    let metrics = ShapeMetrics::from_inputs(inputs.get(0));
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), brep);
    if extended {
        let has_geometry = metrics.is_some();
        let caps = if has_geometry {
            Value::Number(1.0)
        } else {
            Value::Number(0.0)
        };
        let solid = Value::Boolean(has_geometry);
        outputs.insert(PIN_OUTPUT_CAPS.to_owned(), caps);
        outputs.insert(PIN_OUTPUT_SOLID.to_owned(), solid);
    }
    Ok(outputs)
}

fn evaluate_flip(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Flip vereist een surface"));
    }
    let mut surface = inputs[0].clone();
    let guide = inputs.get(1);
    let should_flip = guide
        .and_then(|value| match value {
            Value::Vector(vector) => Some(vector[2] < 0.0),
            Value::Point(point) => Some(point[2] < 0.0),
            _ => None,
        })
        .unwrap_or(true);

    if should_flip {
        if let Value::Surface { vertices, faces } = &mut surface {
            vertices.reverse();
            faces.iter_mut().for_each(|face| face.reverse());
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), surface);
    outputs.insert(PIN_OUTPUT_RESULT.to_owned(), Value::Boolean(should_flip));
    Ok(outputs)
}

fn evaluate_merge_faces(inputs: &[Value]) -> ComponentResult {
    let brep = inputs
        .get(0)
        .ok_or_else(|| ComponentError::new("Merge Faces vereist een brep"))?;
    let surfaces = collect_shapes(Some(brep));
    if surfaces.is_empty() {
        return Err(ComponentError::new(
            "Merge Faces kon geen oppervlakken vinden",
        ));
    }
    let before = surfaces.len();
    let merged_metrics = ShapeMetrics::from_inputs(Some(&Value::List(surfaces.clone()))).unwrap();
    let merged_surface = create_surface_from_bounds(merged_metrics.min, merged_metrics.max);

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_BREPS.to_owned(),
        Value::List(vec![merged_surface]),
    );
    outputs.insert(PIN_OUTPUT_BEFORE.to_owned(), Value::Number(before as f64));
    outputs.insert(PIN_OUTPUT_AFTER.to_owned(), Value::Number(1.0));
    Ok(outputs)
}

fn evaluate_edges_by_length(inputs: &[Value], component: &str) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(format!(
            "{} vereist een brep en een minimaal/maximaal criterium",
            component
        )));
    }
    let metrics = coerce_shape_metrics(inputs.get(0), component)?;
    let min_length = coerce_number(inputs.get(1), &(component.to_owned() + " minimum"))?.abs();
    let max_length = coerce_number(inputs.get(2), &(component.to_owned() + " maximum"))?.abs();

    let edges = create_wireframe(&metrics);
    let mut selected = Vec::new();
    let mut indices = Vec::new();

    for (index, edge) in edges.iter().enumerate() {
        let length = edge_length(edge);
        if length >= min_length && length <= max_length.max(min_length) {
            selected.push(Value::CurveLine {
                p1: edge.start,
                p2: edge.end,
            });
            indices.push(Value::Number(index as f64));
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), Value::List(selected));
    outputs.insert(PIN_OUTPUT_INDICES.to_owned(), Value::List(indices));
    Ok(outputs)
}

fn evaluate_untrim(inputs: &[Value]) -> ComponentResult {
    let metrics = coerce_shape_metrics(inputs.get(0), "Untrim")?;
    let surface = create_surface_from_bounds(metrics.min, metrics.max);
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), surface);
    Ok(outputs)
}

#[derive(Debug, Clone, Copy)]
struct Edge {
    start: [f64; 3],
    end: [f64; 3],
}

#[derive(Debug, Clone)]
struct ShapeMetrics {
    min: [f64; 3],
    max: [f64; 3],
}

impl ShapeMetrics {
    fn from_inputs(value: Option<&Value>) -> Option<Self> {
        let points = collect_points(value);
        if points.is_empty() {
            return None;
        }
        let (min, max) = bounding_box(&points);
        Some(Self { min, max })
    }

    fn size(&self) -> [f64; 3] {
        [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ]
    }

    fn sample_point(&self, uv: (f64, f64)) -> [f64; 3] {
        [
            self.min[0] + self.size()[0] * uv.0,
            self.min[1] + self.size()[1] * uv.1,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }

    fn normal_hint(&self) -> [f64; 3] {
        [0.0, 0.0, 1.0]
    }

    fn tangent_hint_u(&self) -> [f64; 3] {
        [self.size()[0].signum().max(EPSILON), 0.0, 0.0]
    }

    fn tangent_hint_v(&self) -> [f64; 3] {
        [0.0, self.size()[1].signum().max(EPSILON), 0.0]
    }

    fn volume(&self) -> f64 {
        let size = self.size();
        size[0].abs() * size[1].abs() * size[2].abs()
    }
}

fn frame_value(origin: [f64; 3], x_axis: [f64; 3], y_axis: [f64; 3], z_axis: [f64; 3]) -> Value {
    Value::List(vec![
        Value::Point(origin),
        Value::Vector(x_axis),
        Value::Vector(y_axis),
        Value::Vector(z_axis),
    ])
}

fn create_surface_from_bounds(min: [f64; 3], max: [f64; 3]) -> Value {
    let vertices = vec![
        [min[0], min[1], min[2]],
        [max[0], min[1], min[2]],
        [max[0], max[1], max[2]],
        [min[0], max[1], max[2]],
    ];
    let faces = vec![vec![0, 1, 2], vec![0, 2, 3]];
    Value::Surface { vertices, faces }
}

fn create_wireframe(metrics: &ShapeMetrics) -> Vec<Edge> {
    let corners = create_box_corners(metrics);
    let pairs = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];
    pairs
        .iter()
        .map(|(a, b)| Edge {
            start: corners[*a],
            end: corners[*b],
        })
        .collect()
}

fn create_box_corners(metrics: &ShapeMetrics) -> Vec<[f64; 3]> {
    let mut corners = Vec::with_capacity(8);
    for &z in &[metrics.min[2], metrics.max[2]] {
        for &y in &[metrics.min[1], metrics.max[1]] {
            for &x in &[metrics.min[0], metrics.max[0]] {
                corners.push([x, y, z]);
            }
        }
    }
    corners
}

fn collect_points(value: Option<&Value>) -> Vec<[f64; 3]> {
    match value {
        Some(Value::Point(point)) | Some(Value::Vector(point)) => vec![*point],
        Some(Value::CurveLine { p1, p2 }) => vec![*p1, *p2],
        Some(Value::Surface { vertices, .. }) => vertices.clone(),
        Some(Value::List(values)) => values
            .iter()
            .flat_map(|value| collect_points(Some(value)))
            .collect(),
        _ => Vec::new(),
    }
}

fn collect_shapes(value: Option<&Value>) -> Vec<Value> {
    match value {
        Some(Value::List(values)) => values.clone(),
        Some(other) => vec![other.clone()],
        None => Vec::new(),
    }
}

fn collect_point_list(value: Option<&Value>) -> Vec<[f64; 3]> {
    match value {
        Some(Value::List(values)) => values
            .iter()
            .filter_map(|value| match value {
                Value::Point(point) | Value::Vector(point) => Some(*point),
                _ => None,
            })
            .collect(),
        Some(Value::Point(point)) | Some(Value::Vector(point)) => vec![*point],
        _ => Vec::new(),
    }
}

fn bounding_box(points: &[[f64; 3]]) -> ([f64; 3], [f64; 3]) {
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for point in points {
        for axis in 0..3 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }
    (min, max)
}

fn coerce_shape_metrics(
    value: Option<&Value>,
    component: &str,
) -> Result<ShapeMetrics, ComponentError> {
    ShapeMetrics::from_inputs(value)
        .ok_or_else(|| ComponentError::new(format!("{} vereist geometrische invoer", component)))
}

fn coerce_number(value: Option<&Value>, context: &str) -> Result<f64, ComponentError> {
    match value {
        Some(Value::Number(number)) => Ok(*number),
        Some(Value::Boolean(flag)) => Ok(if *flag { 1.0 } else { 0.0 }),
        Some(Value::List(list)) if !list.is_empty() => coerce_number(list.first(), context),
        Some(other) => Err(ComponentError::new(format!(
            "{} verwacht een getal, kreeg {}",
            context,
            other.kind()
        ))),
        None => Err(ComponentError::new(format!(
            "{} vereist een numerieke waarde",
            context
        ))),
    }
}

fn coerce_positive_integer(value: Option<&Value>, context: &str) -> Result<usize, ComponentError> {
    let number = coerce_number(value, context)?;
    if !number.is_finite() {
        return Err(ComponentError::new(format!(
            "{} vereist een eindige waarde",
            context
        )));
    }
    let rounded = number.round().abs();
    Ok(rounded.max(1.0) as usize)
}

fn coerce_boolean(value: Option<&Value>, default: bool) -> Result<bool, ComponentError> {
    match value {
        Some(Value::Boolean(flag)) => Ok(*flag),
        Some(Value::Number(number)) => Ok(*number != 0.0),
        Some(Value::List(list)) if !list.is_empty() => coerce_boolean(list.first(), default),
        Some(Value::Text(text)) => {
            let normalized = text.trim().to_ascii_lowercase();
            if normalized.is_empty() {
                Ok(default)
            } else {
                Ok(matches!(
                    normalized.as_str(),
                    "true" | "yes" | "1" | "y" | "on"
                ))
            }
        }
        Some(_) => Ok(default),
        None => Ok(default),
    }
}

fn coerce_domain_pair(
    value: &Value,
    context: &str,
) -> Result<((f64, f64), (f64, f64)), ComponentError> {
    match value {
        Value::Domain(Domain::Two(Domain2D { u, v })) => Ok(((u.min, u.max), (v.min, v.max))),
        Value::Domain(Domain::One(Domain1D { min, max, .. })) => Ok(((*min, *max), (*min, *max))),
        Value::List(values) if values.len() >= 4 => {
            let u0 = coerce_number(values.get(0), context)?;
            let u1 = coerce_number(values.get(1), context)?;
            let v0 = coerce_number(values.get(2), context)?;
            let v1 = coerce_number(values.get(3), context)?;
            Ok(((u0, u1), (v0, v1)))
        }
        Value::List(values) if values.len() >= 2 => {
            let u0 = coerce_number(values.get(0), context)?;
            let u1 = coerce_number(values.get(1), context)?;
            Ok(((u0, u1), (0.0, 1.0)))
        }
        _ => Err(ComponentError::new(format!(
            "{} verwacht een domein",
            context
        ))),
    }
}

fn parse_directions(value: Option<&Value>) -> Vec<[f64; 3]> {
    fn parse(value: &Value) -> Option<[f64; 3]> {
        match value {
            Value::Vector(vector) | Value::Point(vector) => Some(*vector),
            Value::List(values) if values.len() >= 3 => {
                let x = coerce_number(values.get(0), "richting").ok()?;
                let y = coerce_number(values.get(1), "richting").ok()?;
                let z = coerce_number(values.get(2), "richting").ok()?;
                Some([x, y, z])
            }
            _ => None,
        }
    }

    match value {
        Some(Value::List(values)) => values.iter().filter_map(parse).collect(),
        Some(other) => parse(other).into_iter().collect(),
        None => Vec::new(),
    }
}

fn edge_direction(edge: &Edge) -> [f64; 3] {
    [
        edge.end[0] - edge.start[0],
        edge.end[1] - edge.start[1],
        edge.end[2] - edge.start[2],
    ]
}

fn edge_length(edge: &Edge) -> f64 {
    distance(&edge.start, &edge.end)
}

fn normalize(vector: [f64; 3]) -> Option<([f64; 3], f64)> {
    let length = (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt();
    if length < EPSILON {
        None
    } else {
        Some((
            [vector[0] / length, vector[1] / length, vector[2] / length],
            length,
        ))
    }
}

fn distance(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

fn clamp01(value: f64) -> f64 {
    clamp(value, 0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::{Component, ComponentKind};
    use crate::graph::node::MetaMap;
    use crate::graph::value::Value;

    fn unit_surface() -> Value {
        let vertices = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let faces = vec![vec![0, 1, 2], vec![0, 2, 3]];
        Value::Surface { vertices, faces }
    }

    fn cube_brep() -> Value {
        Value::List(vec![unit_surface()])
    }

    #[test]
    fn divide_surface_returns_grid() {
        let component = ComponentKind::DivideSurface;
        let outputs = component
            .evaluate(
                &[unit_surface(), Value::Number(1.0), Value::Number(1.0)],
                &MetaMap::new(),
            )
            .expect("divide surface");
        let points = outputs
            .get(super::PIN_OUTPUT_POINTS)
            .and_then(|value| value.expect_list().ok())
            .unwrap();
        assert_eq!(points.len(), 4);
        let normals = outputs
            .get(super::PIN_OUTPUT_NORMALS)
            .and_then(|value| value.expect_list().ok())
            .unwrap();
        assert_eq!(normals.len(), 4);
    }

    #[test]
    fn surface_frames_returns_rows() {
        let component = ComponentKind::SurfaceFrames;
        let outputs = component
            .evaluate(
                &[unit_surface(), Value::Number(1.0), Value::Number(1.0)],
                &MetaMap::new(),
            )
            .expect("surface frames");
        let frames = outputs
            .get(super::PIN_OUTPUT_FRAMES)
            .and_then(|value| value.expect_list().ok())
            .unwrap();
        assert_eq!(frames.len(), 2);
        assert!(frames.iter().all(|row| matches!(row, Value::List(_))));
    }

    #[test]
    fn edges_from_directions_filters_axes() {
        let component = ComponentKind::EdgesFromDirections;
        let direction = Value::Vector([1.0, 0.0, 0.0]);
        let outputs = component
            .evaluate(
                &[
                    cube_brep(),
                    Value::List(vec![direction.clone()]),
                    Value::Boolean(false),
                    Value::Number(5.0),
                ],
                &MetaMap::new(),
            )
            .expect("edges from directions");
        let edges = outputs
            .get(super::PIN_OUTPUT_BREPS)
            .and_then(|value| value.expect_list().ok())
            .unwrap();
        assert!(!edges.is_empty());
        assert!(
            edges
                .iter()
                .all(|edge| matches!(edge, Value::CurveLine { .. }))
        );
        let mapping = outputs
            .get(super::PIN_OUTPUT_MAP)
            .and_then(|value| value.expect_list().ok())
            .unwrap();
        assert_eq!(mapping.len(), edges.len());
    }

    #[test]
    fn offset_surface_extends_bounds() {
        let component = ComponentKind::OffsetSurface;
        let outputs = component
            .evaluate(&[unit_surface(), Value::Number(2.0)], &MetaMap::new())
            .expect("offset surface");
        let surface = outputs
            .get(super::PIN_OUTPUT_BREPS)
            .cloned()
            .expect("surface output");
        let Value::Surface { vertices, .. } = surface else {
            panic!("expected surface value");
        };
        let min_z = vertices.iter().map(|v| v[2]).fold(f64::INFINITY, f64::min);
        let max_z = vertices
            .iter()
            .map(|v| v[2])
            .fold(f64::NEG_INFINITY, f64::max);
        assert_eq!(min_z, -2.0);
        assert_eq!(max_z, 2.0);
    }

    #[test]
    fn cap_holes_ex_reports_caps() {
        let component = ComponentKind::CapHolesEx;
        let outputs = component
            .evaluate(&[cube_brep()], &MetaMap::new())
            .expect("cap holes ex");
        let caps = outputs
            .get(super::PIN_OUTPUT_CAPS)
            .and_then(|value| value.expect_number().ok())
            .unwrap();
        assert!(caps >= 0.0);
        let solid = outputs
            .get(super::PIN_OUTPUT_SOLID)
            .and_then(|value| value.expect_boolean().ok())
            .unwrap();
        assert!(solid);
    }

    #[test]
    fn merge_faces_reports_counts() {
        let component = ComponentKind::MergeFaces;
        let outputs = component
            .evaluate(&[cube_brep()], &MetaMap::new())
            .expect("merge faces");
        let before = outputs
            .get(super::PIN_OUTPUT_BEFORE)
            .and_then(|value| value.expect_number().ok())
            .unwrap();
        let after = outputs
            .get(super::PIN_OUTPUT_AFTER)
            .and_then(|value| value.expect_number().ok())
            .unwrap();
        assert!(before >= after);
        assert_eq!(after, 1.0);
    }
}
