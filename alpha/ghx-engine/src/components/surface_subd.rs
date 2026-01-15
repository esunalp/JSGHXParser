//! Implementaties van Grasshopper "Surface → SubD" componenten.
//!
//! This module provides thin wrappers around the `geom::subdivision` module,
//! keeping all geometry logic in `geom` while handling Value coercion here.

use std::collections::BTreeMap;

use crate::geom::{EdgeTag, SubdMesh, SubdOptions, VertexTag};
use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

const PIN_OUTPUT_SUBD: &str = "S";
const PIN_OUTPUT_PIPE: &str = "P";
const PIN_OUTPUT_FUSE: &str = "F";
const PIN_OUTPUT_LINES: &str = "L";
const PIN_OUTPUT_EDGE_CURVES: &str = "E";
const PIN_OUTPUT_EDGE_TAGS: &str = "T";
const PIN_OUTPUT_EDGE_IDS: &str = "I";
const PIN_OUTPUT_FACE_POINTS: &str = "P";
const PIN_OUTPUT_FACE_COUNTS: &str = "C";
const PIN_OUTPUT_FACE_EDGES: &str = "E";
const PIN_OUTPUT_FACE_VERTICES: &str = "V";
const PIN_OUTPUT_MESH: &str = "M";
const PIN_OUTPUT_POINTS: &str = "P";
const PIN_OUTPUT_IDS: &str = "I";
const PIN_OUTPUT_VERTEX_TAGS: &str = "T";

/// Beschikbare componentvarianten binnen Surface → SubD.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    EdgeTags,
    Box,
    Edges,
    Fuse,
    MultiPipe,
    Faces,
    FromMesh,
    VertexTags,
    MeshFromSubd,
    ControlPolygon,
    VerticesBasic,
    VerticesDetailed,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Registraties van alle Surface → SubD componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{048b219e-284a-49f2-ae40-a60465b08447}"],
        names: &["SubD Edge Tags", "SubDTags"],
        kind: ComponentKind::EdgeTags,
    },
    Registration {
        guids: &["{10487e4e-a405-48b5-b188-5a8a6328418b}"],
        names: &["SubD Box", "SubDBox"],
        kind: ComponentKind::Box,
    },
    Registration {
        guids: &["{2183c4c6-b5b3-45d2-9261-2096c9357f92}"],
        names: &["SubD Edges", "SubDEdges"],
        kind: ComponentKind::Edges,
    },
    Registration {
        guids: &["{264b4aa6-4915-4a67-86a7-22a5c4acf565}"],
        names: &["SubD Fuse", "Fuse"],
        kind: ComponentKind::Fuse,
    },
    Registration {
        guids: &[
            "{4bfe1bf6-fbc9-4ad2-bf28-a7402e1392ee}",
            "{f1b75016-5818-4ece-be56-065253a2357d}",
        ],
        names: &["MultiPipe", "MP"],
        kind: ComponentKind::MultiPipe,
    },
    Registration {
        guids: &["{83c81431-17bc-4bff-bb85-be0a846bd044}"],
        names: &["SubD Faces", "SubDFaces"],
        kind: ComponentKind::Faces,
    },
    Registration {
        guids: &["{855a2c73-31c0-41d2-b061-57d54229d11b}"],
        names: &["SubD from Mesh", "SubDMesh"],
        kind: ComponentKind::FromMesh,
    },
    Registration {
        guids: &["{954a8963-bb2c-4847-9012-69ff34acddd5}"],
        names: &["SubD Vertex Tags", "SubDVTags"],
        kind: ComponentKind::VertexTags,
    },
    Registration {
        guids: &["{c0b3c6e9-d05d-4c51-a0df-1ce2678c7a33}"],
        names: &["Mesh from SubD", "MeshSubD"],
        kind: ComponentKind::MeshFromSubd,
    },
    Registration {
        guids: &["{c1a57c2a-11c5-4f77-851e-0a7dffef848e}"],
        names: &["SubD Control Polygon", "SubDPoly"],
        kind: ComponentKind::ControlPolygon,
    },
    Registration {
        guids: &["{cd9efa8f-0084-4d52-ab13-ad88ff22dc46}"],
        names: &["SubD Vertices", "SubDVerts"],
        kind: ComponentKind::VerticesBasic,
    },
    Registration {
        guids: &["{fc8ad805-2cbf-4447-b41b-50c0be591fcd}"],
        names: &["SubD Vertices", "SubDVerts"],
        kind: ComponentKind::VerticesDetailed,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::EdgeTags => evaluate_edge_tags(inputs),
            Self::Box => evaluate_box(inputs),
            Self::Edges => evaluate_edges(inputs),
            Self::Fuse => evaluate_fuse(inputs),
            Self::MultiPipe => evaluate_multi_pipe(inputs),
            Self::Faces => evaluate_faces(inputs),
            Self::FromMesh => evaluate_from_mesh(inputs),
            Self::VertexTags => evaluate_vertex_tags(inputs),
            Self::MeshFromSubd => evaluate_mesh_from_subd(inputs),
            Self::ControlPolygon => evaluate_control_polygon(inputs),
            Self::VerticesBasic => evaluate_vertices(inputs, false),
            Self::VerticesDetailed => evaluate_vertices(inputs, true),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::EdgeTags => "SubD Edge Tags",
            Self::Box => "SubD Box",
            Self::Edges => "SubD Edges",
            Self::Fuse => "SubD Fuse",
            Self::MultiPipe => "MultiPipe",
            Self::Faces => "SubD Faces",
            Self::FromMesh => "SubD from Mesh",
            Self::VertexTags => "SubD Vertex Tags",
            Self::MeshFromSubd => "Mesh from SubD",
            Self::ControlPolygon => "SubD Control Polygon",
            Self::VerticesBasic | Self::VerticesDetailed => "SubD Vertices",
        }
    }
}

fn evaluate_edge_tags(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "SubD Edge Tags vereist een SubD en een tagwaarde",
        ));
    }
    let mut subd = coerce_subd(inputs.get(0), "SubD Edge Tags")?;
    let tag = EdgeTag::parse(inputs.get(1))
        .ok_or_else(|| ComponentError::new("SubD Edge Tags tag vereist een tagwaarde"))?;
    let ids = collect_indices(inputs.get(2));
    if ids.is_empty() {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_SUBD.to_owned(), subd.to_value());
        return Ok(outputs);
    }
    subd.apply_edge_tag(&ids, tag);
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_SUBD.to_owned(), subd.to_value());
    Ok(outputs)
}

fn evaluate_box(inputs: &[Value]) -> ComponentResult {
    let points = collect_points(inputs.get(0));
    if points.is_empty() {
        return Err(ComponentError::new(
            "SubD Box vereist een box of puntverzameling",
        ));
    }
    let (mut min, mut max) = bounding_box(&points);
    if min == max {
        for value in &mut min {
            *value -= 0.5;
        }
        for value in &mut max {
            *value += 0.5;
        }
    }
    let density = coerce_number(inputs.get(1), "SubD Box dichtheid").unwrap_or(1.0);
    let creases = coerce_boolean(inputs.get(2), false, "SubD Box creases")?;
    let mut subd = SubdMesh::box_from_bounds(min, max);
    if density > 1.0 {
        let steps = density.round().clamp(1.0, 4.0) as usize;
        subd.smooth(steps.saturating_sub(1));
    }
    if creases {
        let ids: Vec<_> = (0..subd.edges.len()).collect();
        subd.apply_edge_tag(&ids, EdgeTag::Crease);
    }
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_SUBD.to_owned(), subd.to_value());
    Ok(outputs)
}

fn evaluate_edges(inputs: &[Value]) -> ComponentResult {
    let subd = coerce_subd(inputs.get(0), "SubD Edges")?;
    let mut lines = Vec::new();
    let mut curves = Vec::new();
    let mut tags = Vec::new();
    let mut ids = Vec::new();
    for edge in &subd.edges {
        if let (Some(start), Some(end)) =
            (subd.vertex(edge.vertices.0), subd.vertex(edge.vertices.1))
        {
            lines.push(Value::CurveLine {
                p1: start.position,
                p2: end.position,
            });
            curves.push(Value::List(vec![
                Value::Point(start.position),
                Value::Point(end.position),
            ]));
            tags.push(Value::Text(edge.tag.as_str().to_owned()));
            ids.push(Value::Number(edge.id as f64));
        }
    }
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_LINES.to_owned(), Value::List(lines));
    outputs.insert(PIN_OUTPUT_EDGE_CURVES.to_owned(), Value::List(curves));
    outputs.insert(PIN_OUTPUT_EDGE_TAGS.to_owned(), Value::List(tags));
    outputs.insert(PIN_OUTPUT_EDGE_IDS.to_owned(), Value::List(ids));
    Ok(outputs)
}

fn evaluate_fuse(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("SubD Fuse vereist minstens één invoer"));
    }
    let subd_a = coerce_subd(inputs.get(0), "SubD Fuse A").ok();
    let subd_b = coerce_subd(inputs.get(1), "SubD Fuse B").ok();
    let option = coerce_number(inputs.get(2), "SubD Fuse optie").unwrap_or(0.0);
    let smoothing = coerce_number(inputs.get(3), "SubD Fuse smoothing").unwrap_or(0.0);
    let mut result = match option.round() as i32 {
        1 => SubdMesh::fuse_intersection(subd_a.clone(), subd_b.clone()),
        2 => subd_a.unwrap_or_else(SubdMesh::empty),
        3 => subd_b.unwrap_or_else(SubdMesh::empty),
        _ => SubdMesh::fuse_union(subd_a.clone(), subd_b.clone()),
    };
    let steps = smoothing.max(0.0).round() as usize;
    if steps > 0 {
        result.smooth(steps);
    }
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_FUSE.to_owned(), result.to_value());
    Ok(outputs)
}

fn evaluate_multi_pipe(inputs: &[Value]) -> ComponentResult {
    let points = collect_points(inputs.get(0));
    if points.is_empty() {
        return Err(ComponentError::new(
            "MultiPipe vereist minstens één curve of punt",
        ));
    }
    let radii = collect_numbers(inputs.get(1));
    let radius = radii
        .into_iter()
        .fold(0.5_f64, |acc, value| acc.max(value.abs()));
    let cap = coerce_boolean(inputs.get(8), false, "MultiPipe caps").unwrap_or(false);
    
    // Use the geom multi_pipe helper
    let subd = SubdMesh::multi_pipe(&points, radius, cap)
        .map_err(|e| ComponentError::new(format!("MultiPipe failed: {e}")))?;
    
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_PIPE.to_owned(), subd.to_value());
    Ok(outputs)
}

fn evaluate_faces(inputs: &[Value]) -> ComponentResult {
    let subd = coerce_subd(inputs.get(0), "SubD Faces")?;
    let mut centres = Vec::new();
    let mut counts = Vec::new();
    let mut edge_ids = Vec::new();
    let mut vertex_ids = Vec::new();
    for face in &subd.faces {
        let centroid = face
            .vertices
            .iter()
            .filter_map(|id| subd.vertex(*id).map(|vertex| vertex.position))
            .fold(([0.0, 0.0, 0.0], 0usize), |(mut acc, count), point| {
                acc[0] += point[0];
                acc[1] += point[1];
                acc[2] += point[2];
                (acc, count + 1)
            });
        let point = if centroid.1 > 0 {
            [
                centroid.0[0] / centroid.1 as f64,
                centroid.0[1] / centroid.1 as f64,
                centroid.0[2] / centroid.1 as f64,
            ]
        } else {
            [0.0, 0.0, 0.0]
        };
        centres.push(Value::Point(point));
        counts.push(Value::Number(face.vertices.len() as f64));
        edge_ids.push(Value::List(
            face.edges
                .iter()
                .map(|id| Value::Number(*id as f64))
                .collect(),
        ));
        vertex_ids.push(Value::List(
            face.vertices
                .iter()
                .map(|id| Value::Number(*id as f64))
                .collect(),
        ));
    }
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_FACE_POINTS.to_owned(), Value::List(centres));
    outputs.insert(PIN_OUTPUT_FACE_COUNTS.to_owned(), Value::List(counts));
    outputs.insert(PIN_OUTPUT_FACE_EDGES.to_owned(), Value::List(edge_ids));
    outputs.insert(PIN_OUTPUT_FACE_VERTICES.to_owned(), Value::List(vertex_ids));
    Ok(outputs)
}

fn evaluate_from_mesh(inputs: &[Value]) -> ComponentResult {
    let mut subd = coerce_mesh_as_subd(inputs.get(0), "SubD from Mesh")?;
    let crease = coerce_boolean(inputs.get(1), false, "SubD from Mesh creases")?;
    let corners = coerce_boolean(inputs.get(2), false, "SubD from Mesh corners")?;
    let interpolate = coerce_boolean(inputs.get(3), false, "SubD from Mesh interpolatie")?;
    if crease {
        subd.crease_boundaries();
    }
    if corners {
        subd.corner_boundary_vertices();
    }
    if interpolate {
        subd.smooth(1);
    }
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_SUBD.to_owned(), subd.to_value());
    Ok(outputs)
}

fn evaluate_vertex_tags(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "SubD Vertex Tags vereist een SubD en een tagwaarde",
        ));
    }
    let mut subd = coerce_subd(inputs.get(0), "SubD Vertex Tags")?;
    let tag = VertexTag::parse(inputs.get(1))
        .ok_or_else(|| ComponentError::new("SubD Vertex Tags tag vereist een tagwaarde"))?;
    let ids = collect_indices(inputs.get(2));
    if ids.is_empty() {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_SUBD.to_owned(), subd.to_value());
        return Ok(outputs);
    }
    subd.apply_vertex_tag(&ids, tag);
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_SUBD.to_owned(), subd.to_value());
    Ok(outputs)
}

fn evaluate_mesh_from_subd(inputs: &[Value]) -> ComponentResult {
    let subd = coerce_subd(inputs.get(0), "Mesh from SubD")?;
    let density = coerce_number(inputs.get(1), "Mesh from SubD dichtheid")?.max(1.0);
    let steps = density.round().clamp(1.0, 5.0) as usize;
    let options = SubdOptions::with_density(steps);
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_MESH.to_owned(), subd.to_mesh_value(options));
    Ok(outputs)
}

fn evaluate_control_polygon(inputs: &[Value]) -> ComponentResult {
    let subd = coerce_subd(inputs.get(0), "SubD Control Polygon")?;
    // density=1 means control mesh (no subdivision)
    let options = SubdOptions::with_density(1);
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_MESH.to_owned(), subd.to_mesh_value(options));
    Ok(outputs)
}

fn evaluate_vertices(inputs: &[Value], include_tags: bool) -> ComponentResult {
    let subd = coerce_subd(inputs.get(0), "SubD Vertices")?;
    let mut points = Vec::new();
    let mut ids = Vec::new();
    let mut tags = Vec::new();
    for vertex in &subd.vertices {
        points.push(Value::Point(vertex.position));
        ids.push(Value::Number(vertex.id as f64));
        if include_tags {
            tags.push(Value::Text(vertex.tag.as_str().to_owned()));
        }
    }
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::List(points));
    outputs.insert(PIN_OUTPUT_IDS.to_owned(), Value::List(ids));
    if include_tags {
        outputs.insert(PIN_OUTPUT_VERTEX_TAGS.to_owned(), Value::List(tags));
    }
    Ok(outputs)
}

// ============================================================================
// Helper functions for coercion
// ============================================================================

/// Coerce a Value to a SubdMesh.
fn coerce_subd(value: Option<&Value>, context: &str) -> Result<SubdMesh, ComponentError> {
    let Some(value) = value else {
        return Err(ComponentError::new(format!("{context} vereist een SubD")));
    };
    if let Some(subd) = SubdMesh::from_value(value) {
        return Ok(subd);
    }
    if let Some(subd) = SubdMesh::from_surface_value(value) {
        return Ok(subd);
    }
    if let Value::List(values) = value {
        for entry in values {
            if let Some(subd) = SubdMesh::from_value(entry) {
                return Ok(subd);
            }
            if let Some(subd) = SubdMesh::from_surface_value(entry) {
                return Ok(subd);
            }
        }
    }
    Err(ComponentError::new(format!(
        "{context} kon SubD niet lezen"
    )))
}

/// Coerce a Value (expected to be a mesh or surface) to a SubdMesh.
fn coerce_mesh_as_subd(value: Option<&Value>, context: &str) -> Result<SubdMesh, ComponentError> {
    let Some(value) = value else {
        return Err(ComponentError::new(format!("{context} vereist een mesh")));
    };
    SubdMesh::from_surface_value(value)
        .ok_or_else(|| ComponentError::new(format!("{context} kon de mesh niet lezen")))
}

/// Collect points from a Value (supports Point, Vector, CurveLine, Surface, Mesh, List).
fn collect_points(value: Option<&Value>) -> Vec<[f64; 3]> {
    match value {
        Some(Value::Point(point)) | Some(Value::Vector(point)) => vec![*point],
        Some(Value::CurveLine { p1, p2 }) => vec![*p1, *p2],
        Some(Value::Surface { vertices, .. }) => vertices.clone(),
        Some(Value::Mesh { vertices, .. }) => vertices.clone(),
        Some(Value::List(values)) => values
            .iter()
            .flat_map(|value| collect_points(Some(value)))
            .collect(),
        _ => Vec::new(),
    }
}

/// Compute bounding box from a slice of points.
fn bounding_box(points: &[[f64; 3]]) -> ([f64; 3], [f64; 3]) {
    if points.is_empty() {
        return ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
    }
    let mut min = [points[0][0], points[0][1], points[0][2]];
    let mut max = min;
    for point in points.iter().skip(1) {
        for axis in 0..3 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }
    (min, max)
}

/// Collect indices from a Value (supports Number, Text, List).
fn collect_indices(value: Option<&Value>) -> Vec<usize> {
    match value {
        Some(Value::Number(number)) if number.is_finite() => {
            if *number < 0.0 {
                Vec::new()
            } else {
                vec![number.round() as usize]
            }
        }
        Some(Value::Text(text)) => text
            .split(|c| c == ',' || c == ';' || c == ' ')
            .filter_map(|part| part.trim().parse::<usize>().ok())
            .collect(),
        Some(Value::List(values)) => values
            .iter()
            .flat_map(|value| collect_indices(Some(value)))
            .collect(),
        _ => Vec::new(),
    }
}

/// Collect numbers from a Value (supports Number, Boolean, List).
fn collect_numbers(value: Option<&Value>) -> Vec<f64> {
    match value {
        Some(Value::Number(number)) if number.is_finite() => vec![*number],
        Some(Value::Boolean(flag)) => vec![if *flag { 1.0 } else { 0.0 }],
        Some(Value::List(values)) => values
            .iter()
            .flat_map(|value| collect_numbers(Some(value)))
            .collect(),
        _ => Vec::new(),
    }
}

/// Coerce a Value to a number.
fn coerce_number(value: Option<&Value>, context: &str) -> Result<f64, ComponentError> {
    match value {
        Some(Value::Number(number)) if number.is_finite() => Ok(*number),
        Some(Value::Boolean(flag)) => Ok(if *flag { 1.0 } else { 0.0 }),
        Some(Value::Text(text)) => text
            .trim()
            .parse::<f64>()
            .map_err(|_| ComponentError::new(format!("{context} verwacht een getal"))),
        Some(Value::List(values)) if !values.is_empty() => coerce_number(values.first(), context),
        None => Ok(0.0),
        _ => Err(ComponentError::new(format!("{context} verwacht een getal"))),
    }
}

/// Coerce a Value to a boolean.
fn coerce_boolean(
    value: Option<&Value>,
    default: bool,
    context: &str,
) -> Result<bool, ComponentError> {
    match value {
        Some(Value::Boolean(flag)) => Ok(*flag),
        Some(Value::Number(number)) => Ok(*number != 0.0),
        Some(Value::Text(text)) => match text.trim().to_lowercase().as_str() {
            "true" | "yes" | "1" => Ok(true),
            "false" | "no" | "0" => Ok(false),
            _ => Err(ComponentError::new(format!(
                "{context} verwacht een boolean",
            ))),
        },
        Some(Value::List(values)) if !values.is_empty() => {
            coerce_boolean(values.first(), default, context)
        }
        None => Ok(default),
        _ => Err(ComponentError::new(format!(
            "{context} verwacht een boolean",
        ))),
    }
}