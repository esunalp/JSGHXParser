//! Implementaties van Grasshopper "Surface → Util" componenten.

use std::collections::{BTreeMap, BTreeSet};

use delaunator;

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

    #[must_use]
    pub fn optional_input_pins(&self) -> &'static [&'static str] {
        match self {
            Self::Flip => &["G", "Guide"],
            _ => &[],
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

    let mut brep = collect_brep_data(inputs.get(0));
    if brep.edges.is_empty() {
        brep = BrepData::from_metrics(&metrics);
    }

    let mut selected = Vec::new();
    let mut indices = Vec::new();
    let mut mapping = Vec::new();

    for (index, edge) in brep.edges.iter().enumerate() {
        if let Some((direction, _)) = normalize(edge.vector()) {
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
                selected.push(edge.to_value());
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
    let mut brep = collect_brep_data(inputs.get(0));
    if brep.edges.is_empty() {
        brep = BrepData::from_metrics(&metrics);
    }

    let mut closed_edges = Vec::new();
    let mut closed_indices = Vec::new();
    let mut open_edges = Vec::new();
    let mut open_indices = Vec::new();

    for (index, edge) in brep.edges.iter().enumerate() {
        if edge.face_count() >= 2 {
            closed_edges.push(edge.to_value());
            closed_indices.push(Value::Number(index as f64));
        } else {
            open_edges.push(edge.to_value());
            open_indices.push(Value::Number(index as f64));
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CLOSED.to_owned(), Value::List(closed_edges));
    outputs.insert(
        PIN_OUTPUT_CLOSED_INDICES.to_owned(),
        Value::List(closed_indices),
    );
    outputs.insert(PIN_OUTPUT_OPEN.to_owned(), Value::List(open_edges));
    outputs.insert(
        PIN_OUTPUT_OPEN_INDICES.to_owned(),
        Value::List(open_indices),
    );
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

    let mut brep = collect_brep_data(inputs.get(0));
    if brep.edges.is_empty() {
        brep = BrepData::from_metrics(&metrics);
    }

    let mut selected = Vec::new();
    let mut indices = Vec::new();

    if !brep.faces.is_empty() {
        let mut selected_faces = Vec::new();
        if points.is_empty() {
            selected_faces.extend(0..brep.faces.len());
        } else {
            for (face_index, face) in brep.faces.iter().enumerate() {
                let centroid = face.centroid();
                if points
                    .iter()
                    .any(|point| distance(&centroid, point) <= tolerance)
                {
                    selected_faces.push(face_index);
                }
            }
        }

        for (index, edge) in brep.edges.iter().enumerate() {
            if edge
                .faces
                .iter()
                .any(|face_index| selected_faces.contains(face_index))
            {
                selected.push(edge.to_value());
                indices.push(Value::Number(index as f64));
            }
        }
    } else {
        for (index, edge) in brep.edges.iter().enumerate() {
            let include = points.is_empty()
                || points
                    .iter()
                    .any(|point| edge.touches_point(point, tolerance));
            if include {
                selected.push(edge.to_value());
                indices.push(Value::Number(index as f64));
            }
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

    let mut brep = collect_brep_data(inputs.get(0));
    if brep.edges.is_empty() {
        brep = BrepData::from_metrics(&metrics);
    }

    let mut selected = Vec::new();
    let mut indices = Vec::new();
    let mut mapping = vec![0usize; points.len()];

    for (index, edge) in brep.edges.iter().enumerate() {
        let mut matched_points = Vec::new();
        for (point_index, point) in points.iter().enumerate() {
            if edge.touches_point(point, tolerance) {
                matched_points.push(point_index);
            }
        }
        if matched_points.len() >= valence {
            selected.push(edge.to_value());
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
    let mut brep = collect_brep_data(inputs.get(0));
    if brep.edges.is_empty() {
        brep = BrepData::from_metrics(&metrics);
    }

    let mut convex = Vec::new();
    let mut concave = Vec::new();
    let mut mixed = Vec::new();

    for edge in brep.edges {
        let value = edge.to_value();
        match edge.face_count() {
            0 => mixed.push(value),
            1 => concave.push(value),
            _ => convex.push(value),
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
    let mut brep_input = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Cap Holes vereist een brep"))?;

    // Handle lists by taking the first valid surface.
    let mut brep_value_storage;
    let mut brep_value = if let Value::List(values) = &brep_input {
        brep_value_storage = values
            .iter()
            .find(|v| matches!(v, Value::Surface { .. }))
            .cloned()
            .unwrap_or(brep_input.clone());
        &mut brep_value_storage
    } else {
        &mut brep_input
    };

    let (vertices, faces) = match brep_value {
        Value::Surface { vertices, faces } => (vertices, faces),
        _ => {
            // Not a surface, just return it as is.
            let mut outputs = BTreeMap::new();
            outputs.insert(PIN_OUTPUT_BREPS.to_owned(), brep_input);
            if extended {
                outputs.insert(PIN_OUTPUT_CAPS.to_owned(), Value::Number(0.0));
                outputs.insert(PIN_OUTPUT_SOLID.to_owned(), Value::Boolean(false));
            }
            return Ok(outputs);
        }
    };

    if vertices.is_empty() || faces.is_empty() {
        // Empty mesh, nothing to cap.
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_BREPS.to_owned(), brep_value.clone());
        if extended {
            outputs.insert(PIN_OUTPUT_CAPS.to_owned(), Value::Number(0.0));
            outputs.insert(PIN_OUTPUT_SOLID.to_owned(), Value::Boolean(false));
        }
        return Ok(outputs);
    }

    // 1. Find naked edges
    let mut edge_counts: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for face in faces.iter() {
        for i in 0..face.len() {
            let v1_idx = face[i];
            let v2_idx = face[(i + 1) % face.len()];
            // Ensure consistent edge representation (smaller index first)
            let edge = if v1_idx < v2_idx {
                (v1_idx, v2_idx)
            } else {
                (v2_idx, v1_idx)
            };
            *edge_counts.entry(edge).or_insert(0) += 1;
        }
    }

    let naked_edges: Vec<(u32, u32)> = edge_counts
        .into_iter()
        .filter(|(_, count)| *count == 1)
        .map(|(edge, _)| edge)
        .collect();

    if naked_edges.is_empty() {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_BREPS.to_owned(), brep_value.clone());
        if extended {
            outputs.insert(PIN_OUTPUT_CAPS.to_owned(), Value::Number(0.0));
            outputs.insert(PIN_OUTPUT_SOLID.to_owned(), Value::Boolean(true)); // Already solid
        }
        return Ok(outputs);
    }

    // 2. Group naked edges into contiguous loops
    let mut loops = Vec::new();
    let mut remaining_edges_mut = naked_edges;
    while let Some(first_edge) = remaining_edges_mut.pop() {
        let mut current_loop_indices = vec![first_edge.0, first_edge.1];
        let start_node = first_edge.0;
        let mut current_end_node = first_edge.1;

        'loop_building: loop {
            // Find the next connecting edge
            if let Some(pos) = remaining_edges_mut
                .iter()
                .position(|(u, v)| *u == current_end_node || *v == current_end_node)
            {
                let (u, v) = remaining_edges_mut.remove(pos);
                if u == current_end_node {
                    current_loop_indices.push(v);
                    current_end_node = v;
                } else {
                    current_loop_indices.push(u);
                    current_end_node = u;
                }
            } else {
                break 'loop_building; // Path ended
            }

            if current_end_node == start_node {
                break 'loop_building; // Loop is closed
            }
        }

        // A valid hole is a closed loop.
        if *current_loop_indices.first().unwrap() == *current_loop_indices.last().unwrap() {
            current_loop_indices.pop(); // Remove duplicate end node
            loops.push(current_loop_indices);
        }
    }

    let caps_created = loops.len();
    let mut new_faces_count = 0;

    // 3. Triangulate each loop to create new faces
    for hole_indices in loops {
        if hole_indices.len() < 3 {
            continue; // Not a valid polygon to triangulate
        }

        let hole_vertices: Vec<[f64; 3]> = hole_indices
            .iter()
            .map(|&i| vertices[i as usize])
            .collect();

        // Project 3D points to a 2D plane for triangulation.
        // a. Calculate the average normal of the polygon using Newell's method.
        let mut normal = [0.0, 0.0, 0.0];
        for i in 0..hole_vertices.len() {
            let p1 = hole_vertices[i];
            let p2 = hole_vertices[(i + 1) % hole_vertices.len()];
            normal[0] += (p1[1] - p2[1]) * (p1[2] + p2[2]);
            normal[1] += (p1[2] - p2[2]) * (p1[0] + p2[0]);
            normal[2] += (p1[0] - p2[0]) * (p1[1] + p2[1]);
        }

        let norm_mag = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
        if norm_mag < EPSILON {
            continue; // Degenerate polygon
        }
        let normal = [
            normal[0] / norm_mag,
            normal[1] / norm_mag,
            normal[2] / norm_mag,
        ];

        // b. Create an orthonormal basis (u_axis, v_axis) for the 2D plane.
        let u_axis_candidate = if normal[0].abs() > 0.9 {
            [0.0, 1.0, 0.0]
        } else {
            [1.0, 0.0, 0.0]
        };
        let mut u_axis = cross(u_axis_candidate, normal);
        u_axis = normalize(u_axis).map_or([1.0, 0.0, 0.0], |(v, _)| v);
        let v_axis = cross(normal, u_axis);

        // c. Project the 3D vertices onto the 2D plane.
        let points_2d: Vec<delaunator::Point> = hole_vertices
            .iter()
            .map(|p| {
                let p_vec = [p[0], p[1], p[2]];
                delaunator::Point {
                    x: dot(p_vec, u_axis),
                    y: dot(p_vec, v_axis),
                }
            })
            .collect();

        // d. Triangulate the 2D points.
        let triangulation = delaunator::triangulate(&points_2d);
        if triangulation.triangles.is_empty() {
            continue;
        }

        // e. Convert triangulation results back into new faces using original vertex indices.
        for i in (0..triangulation.triangles.len()).step_by(3) {
            let i1 = hole_indices[triangulation.triangles[i]];
            let i2 = hole_indices[triangulation.triangles[i + 1]];
            let i3 = hole_indices[triangulation.triangles[i + 2]];
            faces.push(vec![i1, i2, i3]);
            new_faces_count += 1;
        }
    }

    // 4. Check if the resulting mesh is solid (has no naked edges).
    let is_solid = if new_faces_count > 0 {
        let mut final_edge_counts: BTreeMap<(u32, u32), usize> = BTreeMap::new();
        for face in faces.iter() {
            for i in 0..face.len() {
                let v1_idx = face[i];
                let v2_idx = face[(i + 1) % face.len()];
                let edge = if v1_idx < v2_idx {
                    (v1_idx, v2_idx)
                } else {
                    (v2_idx, v1_idx)
                };
                *final_edge_counts.entry(edge).or_insert(0) += 1;
            }
        }
        final_edge_counts.values().all(|&count| count > 1)
    } else {
        caps_created == 0 // if we didn't add faces, it's solid only if there were no holes to begin with.
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), brep_value.clone());
    if extended {
        outputs.insert(
            PIN_OUTPUT_CAPS.to_owned(),
            Value::Number(caps_created as f64),
        );
        outputs.insert(PIN_OUTPUT_SOLID.to_owned(), Value::Boolean(is_solid));
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

    let mut brep = collect_brep_data(inputs.get(0));
    if brep.edges.is_empty() {
        brep = BrepData::from_metrics(&metrics);
    }

    let mut selected = Vec::new();
    let mut indices = Vec::new();

    for (index, edge) in brep.edges.iter().enumerate() {
        let length = edge.length();
        if length >= min_length && length <= max_length.max(min_length) {
            selected.push(edge.to_value());
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

#[derive(Debug, Clone)]
struct Face {
    vertices: Vec<[f64; 3]>,
}

impl Face {
    fn centroid(&self) -> [f64; 3] {
        if self.vertices.is_empty() {
            return [0.0, 0.0, 0.0];
        }
        let mut sum = [0.0, 0.0, 0.0];
        for vertex in &self.vertices {
            sum[0] += vertex[0];
            sum[1] += vertex[1];
            sum[2] += vertex[2];
        }
        let scale = 1.0 / self.vertices.len() as f64;
        [sum[0] * scale, sum[1] * scale, sum[2] * scale]
    }
}

#[derive(Debug, Clone)]
struct EdgeData {
    start: [f64; 3],
    end: [f64; 3],
    faces: Vec<usize>,
}

impl EdgeData {
    fn new(start: [f64; 3], end: [f64; 3]) -> Self {
        Self {
            start,
            end,
            faces: Vec::new(),
        }
    }

    fn to_value(&self) -> Value {
        Value::CurveLine {
            p1: self.start,
            p2: self.end,
        }
    }

    fn face_count(&self) -> usize {
        self.faces.len()
    }

    fn add_face(&mut self, face: usize) {
        if !self.faces.contains(&face) {
            self.faces.push(face);
        }
    }

    fn vector(&self) -> [f64; 3] {
        [
            self.end[0] - self.start[0],
            self.end[1] - self.start[1],
            self.end[2] - self.start[2],
        ]
    }

    fn length(&self) -> f64 {
        distance(&self.start, &self.end)
    }

    fn matches(&self, start: [f64; 3], end: [f64; 3]) -> bool {
        (nearly_equal_points(&self.start, &start) && nearly_equal_points(&self.end, &end))
            || (nearly_equal_points(&self.start, &end) && nearly_equal_points(&self.end, &start))
    }

    fn touches_point(&self, point: &[f64; 3], tolerance: f64) -> bool {
        distance(&self.start, point) <= tolerance || distance(&self.end, point) <= tolerance
    }
}

#[derive(Debug, Default, Clone)]
struct BrepData {
    faces: Vec<Face>,
    edges: Vec<EdgeData>,
}

impl BrepData {
    fn add_edge(&mut self, start: [f64; 3], end: [f64; 3], face: Option<usize>) {
        if nearly_equal_points(&start, &end) {
            return;
        }
        if let Some(existing) = self.edges.iter_mut().find(|edge| edge.matches(start, end)) {
            if let Some(face_index) = face {
                existing.add_face(face_index);
            }
            return;
        }

        let mut edge = EdgeData::new(start, end);
        if let Some(face_index) = face {
            edge.add_face(face_index);
        }
        self.edges.push(edge);
    }

    fn from_metrics(metrics: &ShapeMetrics) -> Self {
        Self {
            faces: Vec::new(),
            edges: create_box_edges(metrics),
        }
    }

    fn get_naked_edges(&self) -> Vec<usize> {
        self.edges
            .iter()
            .enumerate()
            .filter_map(|(i, edge)| {
                if edge.face_count() == 1 {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }

    fn find_loops(&self, edge_indices: &[usize]) -> Vec<Vec<[f64; 3]>> {
        let mut visited = vec![false; edge_indices.len()];
        let mut loops = Vec::new();

        // Bouw een map van start/end punten naar edge indices voor snelle lookup
        // We gebruiken een eenvoudige aanpak met lineaire zoektochten voor nu,
        // optimalisatie kan later indien nodig.
        
        for i in 0..edge_indices.len() {
            if visited[i] {
                continue;
            }

            let start_edge_idx = edge_indices[i];
            let start_edge = &self.edges[start_edge_idx];
            
            // Begin een nieuwe loop
            let mut current_loop = Vec::new();
            current_loop.push(start_edge.start);
            current_loop.push(start_edge.end);
            
            visited[i] = true;
            let mut current_end = start_edge.end;
            let mut loop_closed = false;

            // Probeer de loop te sluiten
            loop {
                let mut found_next = false;
                for j in 0..edge_indices.len() {
                    if visited[j] {
                        continue;
                    }
                    
                    let next_edge_idx = edge_indices[j];
                    let next_edge = &self.edges[next_edge_idx];

                    if nearly_equal_points(&next_edge.start, &current_end) {
                        current_loop.push(next_edge.end);
                        current_end = next_edge.end;
                        visited[j] = true;
                        found_next = true;
                    } else if nearly_equal_points(&next_edge.end, &current_end) {
                        current_loop.push(next_edge.start);
                        current_end = next_edge.start;
                        visited[j] = true;
                        found_next = true;
                    }

                    if found_next {
                        break;
                    }
                }

                if !found_next {
                    // Check of we terug zijn bij het begin
                    if nearly_equal_points(&current_end, &current_loop[0]) {
                        loop_closed = true;
                        // Het laatste punt is gelijk aan het eerste, verwijder het dubbele punt
                        current_loop.pop();
                    }
                    break;
                }
                
                if nearly_equal_points(&current_end, &current_loop[0]) {
                    loop_closed = true;
                    current_loop.pop();
                    break;
                }
            }

            if loop_closed && current_loop.len() >= 3 {
                loops.push(current_loop);
            }
        }

        loops
    }

    fn to_value(&self) -> Value {
        // Verzamel alle unieke vertices
        let mut vertices = Vec::new();
        let mut faces_indices = Vec::new();

        for face in &self.faces {
            let mut face_indices = Vec::new();
            for vertex in &face.vertices {
                let index = if let Some(pos) = vertices
                    .iter()
                    .position(|v| nearly_equal_points(v, vertex))
                {
                    pos
                } else {
                    vertices.push(*vertex);
                    vertices.len() - 1
                };
                face_indices.push(index as u32);
            }
            faces_indices.push(face_indices);
        }

        Value::Surface {
            vertices,
            faces: faces_indices,
        }
    }
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

fn create_box_edges(metrics: &ShapeMetrics) -> Vec<EdgeData> {
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
        .map(|(a, b)| EdgeData::new(corners[*a], corners[*b]))
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

fn collect_brep_data(value: Option<&Value>) -> BrepData {
    let mut data = BrepData::default();
    collect_brep_data_recursive(value, &mut data);
    data
}

fn collect_brep_data_recursive(value: Option<&Value>, data: &mut BrepData) {
    match value {
        Some(Value::Surface { vertices, faces }) => {
            for face_indices in faces {
                let mut face_vertices = Vec::new();
                for &index in face_indices {
                    if let Some(vertex) = vertices.get(index as usize) {
                        face_vertices.push(*vertex);
                    }
                }
                if face_vertices.len() < 2 {
                    continue;
                }
                let face_index = data.faces.len();
                data.faces.push(Face {
                    vertices: face_vertices.clone(),
                });
                for segment in 0..face_vertices.len() {
                    let start = face_vertices[segment];
                    let end = face_vertices[(segment + 1) % face_vertices.len()];
                    data.add_edge(start, end, Some(face_index));
                }
            }
        }
        Some(Value::CurveLine { p1, p2 }) => {
            data.add_edge(*p1, *p2, None);
        }
        Some(Value::List(values)) => {
            for value in values {
                collect_brep_data_recursive(Some(value), data);
            }
        }
        _ => {}
    }
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

fn nearly_equal_points(a: &[f64; 3], b: &[f64; 3]) -> bool {
    (a[0] - b[0]).abs() <= EPSILON
        && (a[1] - b[1]).abs() <= EPSILON
        && (a[2] - b[2]).abs() <= EPSILON
}

fn distance(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
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
    fn closed_edges_classifies_naked_edges() {
        let component = ComponentKind::ClosedEdges;
        let outputs = component
            .evaluate(&[cube_brep(), Value::Boolean(true)], &MetaMap::new())
            .expect("closed edges");
        let closed = outputs
            .get(super::PIN_OUTPUT_CLOSED)
            .and_then(|value| value.expect_list().ok())
            .unwrap();
        let open = outputs
            .get(super::PIN_OUTPUT_OPEN)
            .and_then(|value| value.expect_list().ok())
            .unwrap();
        assert!(!open.is_empty());
        assert!(closed.len() < open.len());
        let closed_indices = outputs
            .get(super::PIN_OUTPUT_CLOSED_INDICES)
            .and_then(|value| value.expect_list().ok())
            .unwrap();
        assert_eq!(closed.len(), closed_indices.len());
    }

    #[test]
    fn edges_from_faces_uses_face_centroids() {
        let component = ComponentKind::EdgesFromFaces;
        let face_point = Value::Point([2.0 / 3.0, 1.0 / 3.0, 0.0]);
        let outputs = component
            .evaluate(
                &[cube_brep(), Value::List(vec![face_point])],
                &MetaMap::new(),
            )
            .expect("edges from faces");
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
        let indices = outputs
            .get(super::PIN_OUTPUT_INDICES)
            .and_then(|value| value.expect_list().ok())
            .unwrap();
        assert_eq!(edges.len(), indices.len());
    }

    #[test]
    fn edges_from_points_counts_matches() {
        let component = ComponentKind::EdgesFromPoints;
        let point = Value::Point([0.0, 0.0, 0.0]);
        let outputs = component
            .evaluate(
                &[cube_brep(), Value::List(vec![point]), Value::Number(1.0)],
                &MetaMap::new(),
            )
            .expect("edges from points");
        let mapping = outputs
            .get(super::PIN_OUTPUT_MAP)
            .and_then(|value| value.expect_list().ok())
            .unwrap();
        assert_eq!(mapping.len(), 1);
        let count = mapping[0]
            .expect_number()
            .expect("mapping count is numeric");
        assert!(count >= 1.0);
    }

    #[test]
    fn edges_from_length_filters_by_range() {
        let component = ComponentKind::EdgesFromLength;
        let outputs = component
            .evaluate(
                &[cube_brep(), Value::Number(1.0), Value::Number(1.1)],
                &MetaMap::new(),
            )
            .expect("edges from length");
        let edges = outputs
            .get(super::PIN_OUTPUT_BREPS)
            .and_then(|value| value.expect_list().ok())
            .unwrap();
        assert!(!edges.is_empty());
        for edge in edges {
            let Value::CurveLine { p1, p2 } = edge else {
                panic!("expected curve line");
            };
            let length =
                ((p1[0] - p2[0]).powi(2) + (p1[1] - p2[1]).powi(2) + (p1[2] - p2[2]).powi(2))
                    .sqrt();
            assert!(length >= 1.0 - 1e-9);
            assert!(length <= 1.1 + 1e-9);
        }
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

    #[test]
    fn cap_holes_fills_missing_face() {
        let vertices = vec![
            [0.0, 0.0, 0.0], // 0
            [1.0, 0.0, 0.0], // 1
            [1.0, 1.0, 0.0], // 2
            [0.0, 1.0, 0.0], // 3
            [0.0, 0.0, 1.0], // 4
            [1.0, 0.0, 1.0], // 5
            [1.0, 1.0, 1.0], // 6
            [0.0, 1.0, 1.0], // 7
        ];
        // Cube with the top face missing (indices 4, 5, 6, 7)
        let faces = vec![
            // Bottom
            vec![0, 1, 2],
            vec![0, 2, 3],
            // Front
            vec![0, 4, 7],
            vec![0, 7, 3],
            // Back
            vec![1, 5, 6],
            vec![1, 6, 2],
            // Left
            vec![0, 1, 5],
            vec![0, 5, 4],
            // Right
            vec![3, 2, 6],
            vec![3, 6, 7],
        ];
        let initial_face_count = faces.len();
        let surface_with_hole = Value::Surface { vertices, faces };

        let component = ComponentKind::CapHolesEx;
        let outputs = component
            .evaluate(&[surface_with_hole], &MetaMap::new())
            .expect("cap holes ex failed");

        let capped_surface = outputs
            .get(super::PIN_OUTPUT_BREPS)
            .expect("missing brep output");

        let final_face_count = if let Value::Surface { faces, .. } = capped_surface {
            faces.len()
        } else {
            0
        };

        let solid = outputs
            .get(super::PIN_OUTPUT_SOLID)
            .and_then(|v| v.expect_boolean().ok())
            .expect("missing solid output");

        assert!(
            final_face_count > initial_face_count,
            "Face count should increase after capping"
        );
        assert_eq!(
            final_face_count,
            initial_face_count + 2,
            "Expected two new faces for the capped quad"
        );
        assert!(solid, "The capped mesh should be solid");
    }
}
