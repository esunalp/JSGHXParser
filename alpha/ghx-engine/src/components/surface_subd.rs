//! Implementaties van Grasshopper "Surface → SubD" componenten.

use std::collections::{BTreeMap, BTreeSet, HashMap};

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
    let tag = EdgeTag::parse(inputs.get(1), "SubD Edge Tags tag")?;
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
    let mut subd = Subd::box_from_bounds(min, max);
    if density > 1.0 {
        let steps = density.round().clamp(1.0, 4.0) as usize;
        subd.smooth(steps.saturating_sub(1));
    }
    if creases {
        let ids: Vec<_> = (0..subd.edges.len()).collect();
        subd.apply_edge_tag(&ids, EdgeTag::new("crease"));
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
                p1: start.point,
                p2: end.point,
            });
            curves.push(Value::List(vec![
                Value::Point(start.point),
                Value::Point(end.point),
            ]));
            tags.push(Value::Text(edge.tag.to_string()));
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
        1 => fuse_intersection(subd_a.clone(), subd_b.clone()),
        2 => subd_a.unwrap_or_else(Subd::empty),
        3 => subd_b.unwrap_or_else(Subd::empty),
        _ => fuse_union(subd_a.clone(), subd_b.clone()),
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
    let (mut min, mut max) = bounding_box(&points);
    let radii = collect_numbers(inputs.get(1));
    let radius = radii
        .into_iter()
        .fold(0.5_f64, |acc, value| acc.max(value.abs()));
    expand_bounds(&mut min, &mut max, radius.max(0.25_f64));
    let mut subd = Subd::box_from_bounds(min, max);
    if coerce_boolean(inputs.get(8), false, "MultiPipe caps").unwrap_or(false) {
        let ids: Vec<_> = (0..subd.edges.len()).collect();
        subd.apply_edge_tag(&ids, EdgeTag::new("crease"));
    }
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
            .filter_map(|id| subd.vertex(*id).map(|vertex| vertex.point))
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
    if crease {
        let ids: Vec<_> = subd
            .edges
            .iter()
            .filter(|edge| edge.faces.len() <= 1)
            .map(|edge| edge.id)
            .collect();
        subd.apply_edge_tag(&ids, EdgeTag::new("crease"));
    }
    if corners {
        let ids: Vec<_> = subd.boundary_vertices();
        subd.apply_vertex_tag(&ids, VertexTag::new("corner"));
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
    let tag = VertexTag::parse(inputs.get(1), "SubD Vertex Tags tag")?;
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
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_MESH.to_owned(), subd.to_surface());
    Ok(outputs)
}

fn evaluate_control_polygon(inputs: &[Value]) -> ComponentResult {
    let subd = coerce_subd(inputs.get(0), "SubD Control Polygon")?;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_MESH.to_owned(), subd.to_surface());
    Ok(outputs)
}

fn evaluate_vertices(inputs: &[Value], include_tags: bool) -> ComponentResult {
    let subd = coerce_subd(inputs.get(0), "SubD Vertices")?;
    let mut points = Vec::new();
    let mut ids = Vec::new();
    let mut tags = Vec::new();
    for vertex in &subd.vertices {
        points.push(Value::Point(vertex.point));
        ids.push(Value::Number(vertex.id as f64));
        if include_tags {
            tags.push(Value::Text(vertex.tag.to_string()));
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
fn fuse_union(a: Option<Subd>, b: Option<Subd>) -> Subd {
    match (a, b) {
        (Some(left), Some(right)) => left.combine(right),
        (Some(left), None) => left,
        (None, Some(right)) => right,
        (None, None) => Subd::empty(),
    }
}

fn fuse_intersection(a: Option<Subd>, b: Option<Subd>) -> Subd {
    let Some(a) = a else {
        return b.unwrap_or_else(Subd::empty);
    };
    let Some(b) = b else {
        return a;
    };
    match (a.bounding_box(), b.bounding_box()) {
        (Some((amin, amax)), Some((bmin, bmax))) => {
            let min = [
                amin[0].max(bmin[0]),
                amin[1].max(bmin[1]),
                amin[2].max(bmin[2]),
            ];
            let max = [
                amax[0].min(bmax[0]),
                amax[1].min(bmax[1]),
                amax[2].min(bmax[2]),
            ];
            if min[0] > max[0] || min[1] > max[1] || min[2] > max[2] {
                Subd::empty()
            } else {
                Subd::box_from_bounds(min, max)
            }
        }
        _ => Subd::empty(),
    }
}

fn coerce_subd(value: Option<&Value>, context: &str) -> Result<Subd, ComponentError> {
    let Some(value) = value else {
        return Err(ComponentError::new(format!("{context} vereist een SubD")));
    };
    if let Some(subd) = Subd::from_value(value) {
        return Ok(subd);
    }
    if let Some(subd) = Subd::from_surface(value) {
        return Ok(subd);
    }
    if let Value::List(values) = value {
        for entry in values {
            if let Some(subd) = Subd::from_value(entry) {
                return Ok(subd);
            }
            if let Some(subd) = Subd::from_surface(entry) {
                return Ok(subd);
            }
        }
    }
    Err(ComponentError::new(format!(
        "{context} kon SubD niet lezen"
    )))
}

fn coerce_mesh_as_subd(value: Option<&Value>, context: &str) -> Result<Subd, ComponentError> {
    let Some(value) = value else {
        return Err(ComponentError::new(format!("{context} vereist een mesh")));
    };
    Subd::from_surface(value)
        .ok_or_else(|| ComponentError::new(format!("{context} kon de mesh niet lezen")))
}

#[derive(Debug, Clone)]
struct Subd {
    vertices: Vec<SubdVertex>,
    edges: Vec<SubdEdge>,
    faces: Vec<SubdFace>,
}

impl Subd {
    fn empty() -> Self {
        Self {
            vertices: Vec::new(),
            edges: Vec::new(),
            faces: Vec::new(),
        }
    }

    fn from_value(value: &Value) -> Option<Self> {
        let Value::List(items) = value else {
            return None;
        };
        if items.len() != 4 {
            return None;
        }
        let Value::Text(label) = &items[0] else {
            return None;
        };
        if !label.eq_ignore_ascii_case("subd") {
            return None;
        }
        let vertices = parse_vertices(&items[1])?;
        let edges = parse_edges(&items[2]).unwrap_or_default();
        let faces = parse_faces(&items[3])?;
        let mut subd = Self {
            vertices,
            edges,
            faces,
        };
        subd.rebuild_topology();
        Some(subd)
    }

    fn from_surface(value: &Value) -> Option<Self> {
        match value {
            Value::Surface { vertices, faces } => {
                let face_indices: Vec<Vec<usize>> = faces
                    .iter()
                    .filter(|face| face.len() >= 3)
                    .map(|face| face.iter().map(|index| *index as usize).collect())
                    .collect();
                Some(Self::from_vertices_faces(vertices.clone(), face_indices))
            }
            Value::List(values) => {
                for entry in values {
                    if let Some(subd) = Self::from_surface(entry) {
                        return Some(subd);
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn from_vertices_faces(points: Vec<[f64; 3]>, faces: Vec<Vec<usize>>) -> Self {
        let vertices = points
            .into_iter()
            .enumerate()
            .map(|(id, point)| SubdVertex {
                id,
                point,
                tag: VertexTag::default(),
            })
            .collect();
        let faces = faces
            .into_iter()
            .enumerate()
            .filter_map(|(id, vertices)| {
                if vertices.len() < 3 {
                    None
                } else {
                    Some(SubdFace {
                        id,
                        vertices,
                        edges: Vec::new(),
                    })
                }
            })
            .collect();
        let mut subd = Self {
            vertices,
            edges: Vec::new(),
            faces,
        };
        subd.rebuild_topology();
        subd
    }

    fn box_from_bounds(min: [f64; 3], max: [f64; 3]) -> Self {
        let vertices = vec![
            [min[0], min[1], min[2]],
            [max[0], min[1], min[2]],
            [max[0], max[1], min[2]],
            [min[0], max[1], min[2]],
            [min[0], min[1], max[2]],
            [max[0], min[1], max[2]],
            [max[0], max[1], max[2]],
            [min[0], max[1], max[2]],
        ];
        let faces = vec![
            vec![0, 1, 2, 3],
            vec![4, 5, 6, 7],
            vec![0, 1, 5, 4],
            vec![1, 2, 6, 5],
            vec![2, 3, 7, 6],
            vec![3, 0, 4, 7],
        ];
        Self::from_vertices_faces(vertices, faces)
    }

    fn to_value(&self) -> Value {
        let mut clone = self.clone();
        clone.rebuild_topology();
        let vertices = clone
            .vertices
            .iter()
            .map(|vertex| {
                Value::List(vec![
                    Value::Number(vertex.id as f64),
                    Value::Point(vertex.point),
                    Value::Text(vertex.tag.to_string()),
                ])
            })
            .collect();
        let edges = clone
            .edges
            .iter()
            .map(|edge| {
                let mut entry = vec![
                    Value::Number(edge.id as f64),
                    Value::List(vec![
                        Value::Number(edge.vertices.0 as f64),
                        Value::Number(edge.vertices.1 as f64),
                    ]),
                    Value::Text(edge.tag.to_string()),
                ];
                if !edge.faces.is_empty() {
                    entry.push(Value::List(
                        edge.faces
                            .iter()
                            .map(|id| Value::Number(*id as f64))
                            .collect(),
                    ));
                }
                Value::List(entry)
            })
            .collect();
        let faces = clone
            .faces
            .iter()
            .map(|face| {
                let mut entry = vec![
                    Value::Number(face.id as f64),
                    Value::List(
                        face.vertices
                            .iter()
                            .map(|id| Value::Number(*id as f64))
                            .collect(),
                    ),
                ];
                if !face.edges.is_empty() {
                    entry.push(Value::List(
                        face.edges
                            .iter()
                            .map(|id| Value::Number(*id as f64))
                            .collect(),
                    ));
                }
                Value::List(entry)
            })
            .collect();
        Value::List(vec![
            Value::Text("subd".to_owned()),
            Value::List(vertices),
            Value::List(edges),
            Value::List(faces),
        ])
    }

    fn to_surface(&self) -> Value {
        let mut clone = self.clone();
        clone.rebuild_topology();
        let vertices = clone.vertices.iter().map(|vertex| vertex.point).collect();
        let faces = clone
            .faces
            .iter()
            .filter(|face| face.vertices.len() >= 3)
            .map(|face| face.vertices.iter().map(|index| *index as u32).collect())
            .collect();
        Value::Surface { vertices, faces }
    }

    fn vertex(&self, id: usize) -> Option<&SubdVertex> {
        self.vertices.iter().find(|vertex| vertex.id == id)
    }

    fn apply_edge_tag(&mut self, ids: &[usize], tag: EdgeTag) {
        if ids.is_empty() {
            return;
        }
        self.rebuild_topology();
        let lookup: BTreeSet<_> = ids.iter().copied().collect();
        for edge in &mut self.edges {
            if lookup.contains(&edge.id) {
                edge.tag = tag.clone();
            }
        }
    }

    fn apply_vertex_tag(&mut self, ids: &[usize], tag: VertexTag) {
        if ids.is_empty() {
            return;
        }
        self.rebuild_topology();
        let lookup: BTreeSet<_> = ids.iter().copied().collect();
        for vertex in &mut self.vertices {
            if lookup.contains(&vertex.id) {
                vertex.tag = tag.clone();
            }
        }
    }

    fn boundary_vertices(&self) -> Vec<usize> {
        let mut lookup = BTreeSet::new();
        for edge in &self.edges {
            if edge.faces.len() <= 1 {
                lookup.insert(edge.vertices.0);
                lookup.insert(edge.vertices.1);
            }
        }
        lookup.into_iter().collect()
    }

    fn smooth(&mut self, steps: usize) {
        if steps == 0 || self.vertices.is_empty() {
            return;
        }
        self.rebuild_topology();
        for _ in 0..steps {
            let mut sums = vec![[0.0, 0.0, 0.0]; self.vertices.len()];
            let mut counts = vec![0usize; self.vertices.len()];
            for edge in &self.edges {
                let (a, b) = edge.vertices;
                let pa = self.vertices[a].point;
                let pb = self.vertices[b].point;
                sums[a][0] += pb[0];
                sums[a][1] += pb[1];
                sums[a][2] += pb[2];
                counts[a] += 1;
                sums[b][0] += pa[0];
                sums[b][1] += pa[1];
                sums[b][2] += pa[2];
                counts[b] += 1;
            }
            for (index, vertex) in self.vertices.iter_mut().enumerate() {
                if counts[index] == 0 {
                    continue;
                }
                let avg = [
                    sums[index][0] / counts[index] as f64,
                    sums[index][1] / counts[index] as f64,
                    sums[index][2] / counts[index] as f64,
                ];
                vertex.point = [
                    (vertex.point[0] + avg[0]) * 0.5,
                    (vertex.point[1] + avg[1]) * 0.5,
                    (vertex.point[2] + avg[2]) * 0.5,
                ];
            }
        }
        self.rebuild_topology();
    }

    fn combine(mut self, other: Subd) -> Subd {
        let vertex_offset = self.vertices.len();
        for (index, mut vertex) in other.vertices.into_iter().enumerate() {
            vertex.id = vertex_offset + index;
            self.vertices.push(vertex);
        }
        let face_offset = self.faces.len();
        for (index, mut face) in other.faces.into_iter().enumerate() {
            face.id = face_offset + index;
            face.vertices = face
                .vertices
                .into_iter()
                .map(|id| id + vertex_offset)
                .collect();
            face.edges.clear();
            self.faces.push(face);
        }
        self.edges.extend(other.edges.into_iter().map(|mut edge| {
            edge.vertices = (
                edge.vertices.0 + vertex_offset,
                edge.vertices.1 + vertex_offset,
            );
            edge.faces = edge.faces.into_iter().map(|id| id + face_offset).collect();
            edge
        }));
        self.rebuild_topology();
        self
    }

    fn bounding_box(&self) -> Option<([f64; 3], [f64; 3])> {
        if self.vertices.is_empty() {
            return None;
        }
        let mut min = self.vertices[0].point;
        let mut max = self.vertices[0].point;
        for vertex in &self.vertices {
            for axis in 0..3 {
                min[axis] = min[axis].min(vertex.point[axis]);
                max[axis] = max[axis].max(vertex.point[axis]);
            }
        }
        Some((min, max))
    }

    fn rebuild_topology(&mut self) {
        for (index, vertex) in self.vertices.iter_mut().enumerate() {
            vertex.id = index;
        }
        for (index, face) in self.faces.iter_mut().enumerate() {
            face.id = index;
        }
        let mut tag_lookup: HashMap<(usize, usize), EdgeTag> = HashMap::new();
        for edge in &self.edges {
            let key = normalized_pair(edge.vertices);
            tag_lookup.entry(key).or_insert_with(|| edge.tag.clone());
        }
        let mut edge_map = BTreeMap::new();
        let mut edges = Vec::new();
        for face in &mut self.faces {
            face.edges.clear();
            if face.vertices.len() < 2 {
                continue;
            }
            for index in 0..face.vertices.len() {
                let a = face.vertices[index];
                let b = face.vertices[(index + 1) % face.vertices.len()];
                let key = normalized_pair((a, b));
                let entry = edge_map.entry(key).or_insert_with(|| {
                    let id = edges.len();
                    let tag = tag_lookup.get(&key).cloned().unwrap_or_default();
                    edges.push(SubdEdge {
                        id,
                        vertices: key,
                        tag,
                        faces: Vec::new(),
                    });
                    id
                });
                let edge = &mut edges[*entry];
                if !edge.faces.contains(&face.id) {
                    edge.faces.push(face.id);
                }
                face.edges.push(*entry);
            }
        }
        self.edges = edges;
    }
}

#[derive(Debug, Clone)]
struct SubdVertex {
    id: usize,
    point: [f64; 3],
    tag: VertexTag,
}

#[derive(Debug, Clone)]
struct SubdEdge {
    id: usize,
    vertices: (usize, usize),
    tag: EdgeTag,
    faces: Vec<usize>,
}

#[derive(Debug, Clone)]
struct SubdFace {
    id: usize,
    vertices: Vec<usize>,
    edges: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EdgeTag(String);

impl EdgeTag {
    fn new(value: impl AsRef<str>) -> Self {
        Self(value.as_ref().trim().to_lowercase())
    }

    fn parse(value: Option<&Value>, context: &str) -> Result<Self, ComponentError> {
        match value {
            Some(Value::Text(text)) => Ok(Self::from_descriptor(text)),
            Some(Value::Number(number)) => {
                if number.round() as i32 == 1 {
                    Ok(Self::new("crease"))
                } else {
                    Ok(Self::default())
                }
            }
            Some(Value::List(values)) if !values.is_empty() => Self::parse(values.get(0), context),
            None => Ok(Self::default()),
            _ => Err(ComponentError::new(format!(
                "{context} verwacht een tekstuele tag",
            ))),
        }
    }

    fn from_descriptor(descriptor: &str) -> Self {
        match descriptor.trim().to_lowercase().as_str() {
            "s" | "smooth" => Self::new("smooth"),
            "c" | "crease" | "sharp" => Self::new("crease"),
            other => Self::new(other),
        }
    }

    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl Default for EdgeTag {
    fn default() -> Self {
        Self::new("smooth")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VertexTag(String);

impl VertexTag {
    fn new(value: impl AsRef<str>) -> Self {
        Self(value.as_ref().trim().to_lowercase())
    }

    fn parse(value: Option<&Value>, context: &str) -> Result<Self, ComponentError> {
        match value {
            Some(Value::Text(text)) => Ok(Self::from_descriptor(text)),
            Some(Value::Number(number)) => match number.round() as i32 {
                1 => Ok(Self::new("crease")),
                2 => Ok(Self::new("corner")),
                3 => Ok(Self::new("dart")),
                _ => Ok(Self::default()),
            },
            Some(Value::List(values)) if !values.is_empty() => Self::parse(values.get(0), context),
            None => Ok(Self::default()),
            _ => Err(ComponentError::new(format!(
                "{context} verwacht een tekstuele tag",
            ))),
        }
    }

    fn from_descriptor(descriptor: &str) -> Self {
        match descriptor.trim().to_lowercase().as_str() {
            "s" | "smooth" => Self::new("smooth"),
            "c" | "crease" => Self::new("crease"),
            "l" | "corner" => Self::new("corner"),
            "d" | "dart" => Self::new("dart"),
            other => Self::new(other),
        }
    }

    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl Default for VertexTag {
    fn default() -> Self {
        Self::new("smooth")
    }
}

fn normalized_pair(vertices: (usize, usize)) -> (usize, usize) {
    if vertices.0 <= vertices.1 {
        vertices
    } else {
        (vertices.1, vertices.0)
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

fn bounding_box(points: &[[f64; 3]]) -> ([f64; 3], [f64; 3]) {
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

fn expand_bounds(min: &mut [f64; 3], max: &mut [f64; 3], radius: f64) {
    let padding = radius.max(0.0);
    for axis in 0..3 {
        min[axis] -= padding;
        max[axis] += padding;
        if (max[axis] - min[axis]).abs() < 1e-6 {
            max[axis] += 0.5;
            min[axis] -= 0.5;
        }
    }
}

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

fn coerce_number(value: Option<&Value>, context: &str) -> Result<f64, ComponentError> {
    match value {
        Some(Value::Number(number)) if number.is_finite() => Ok(*number),
        Some(Value::Boolean(flag)) => Ok(if *flag { 1.0 } else { 0.0 }),
        Some(Value::Text(text)) => text
            .trim()
            .parse::<f64>()
            .map_err(|_| ComponentError::new(format!("{context} verwacht een getal"))),
        Some(Value::List(values)) if !values.is_empty() => coerce_number(values.get(0), context),
        None => Ok(0.0),
        _ => Err(ComponentError::new(format!("{context} verwacht een getal"))),
    }
}

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
            coerce_boolean(values.get(0), default, context)
        }
        None => Ok(default),
        _ => Err(ComponentError::new(format!(
            "{context} verwacht een boolean",
        ))),
    }
}

fn parse_vertices(value: &Value) -> Option<Vec<SubdVertex>> {
    let Value::List(entries) = value else {
        return None;
    };
    let mut vertices = Vec::with_capacity(entries.len());
    for (index, entry) in entries.iter().enumerate() {
        let Value::List(items) = entry else {
            return None;
        };
        let mut id = index;
        let mut point = None;
        let mut tag = VertexTag::default();
        for item in items {
            match item {
                Value::Number(number) => {
                    if number.is_finite() {
                        id = number.round().max(0.0) as usize;
                    }
                }
                Value::Point(value) | Value::Vector(value) => {
                    point = Some(*value);
                }
                Value::List(values) => {
                    if point.is_none() {
                        if let Some(parsed) = list_to_point(values) {
                            point = Some(parsed);
                        }
                    }
                }
                Value::Text(text) => {
                    tag = VertexTag::from_descriptor(text);
                }
                _ => {}
            }
        }
        let point = point?;
        vertices.push(SubdVertex { id, point, tag });
    }
    Some(vertices)
}

fn parse_edges(value: &Value) -> Option<Vec<SubdEdge>> {
    let Value::List(entries) = value else {
        return None;
    };
    let mut edges = Vec::with_capacity(entries.len());
    for (index, entry) in entries.iter().enumerate() {
        let Value::List(items) = entry else {
            return None;
        };
        let mut iter = items.iter();
        let mut id = index;
        if let Some(Value::Number(number)) = iter.next() {
            if number.is_finite() {
                id = number.round().max(0.0) as usize;
            }
        }
        let vertex_indices = collect_indices(iter.next());
        let vertices = if vertex_indices.len() >= 2 {
            (vertex_indices[0], vertex_indices[1])
        } else {
            (0, 0)
        };
        let mut tag = EdgeTag::default();
        let mut faces = Vec::new();
        if let Some(value) = iter.next() {
            if let Value::Text(text) = value {
                tag = EdgeTag::from_descriptor(text);
            } else {
                faces = collect_indices(Some(value));
            }
        }
        if faces.is_empty() {
            faces = collect_indices(iter.next());
        }
        edges.push(SubdEdge {
            id,
            vertices,
            tag,
            faces,
        });
    }
    Some(edges)
}

fn parse_faces(value: &Value) -> Option<Vec<SubdFace>> {
    let Value::List(entries) = value else {
        return None;
    };
    let mut faces = Vec::with_capacity(entries.len());
    for (index, entry) in entries.iter().enumerate() {
        let Value::List(items) = entry else {
            return None;
        };
        let mut iter = items.iter();
        let mut id = index;
        if let Some(Value::Number(number)) = iter.next() {
            if number.is_finite() {
                id = number.round().max(0.0) as usize;
            }
        }
        let vertices = iter
            .next()
            .map(|value| collect_indices(Some(value)))
            .unwrap_or_default();
        let edges = iter
            .next()
            .map(|value| collect_indices(Some(value)))
            .unwrap_or_default();
        faces.push(SubdFace {
            id,
            vertices,
            edges,
        });
    }
    Some(faces)
}

fn list_to_point(values: &[Value]) -> Option<[f64; 3]> {
    if values.len() < 3 {
        return None;
    }
    let x = match &values[0] {
        Value::Number(value) => *value,
        _ => return None,
    };
    let y = match &values[1] {
        Value::Number(value) => *value,
        _ => return None,
    };
    let z = match &values[2] {
        Value::Number(value) => *value,
        _ => return None,
    };
    Some([x, y, z])
}

#[cfg(test)]
mod tests {
    use super::{Component, ComponentKind, PIN_OUTPUT_FACE_COUNTS, PIN_OUTPUT_SUBD};
    use super::{PIN_OUTPUT_VERTEX_TAGS, Subd};
    use crate::graph::node::MetaMap;
    use crate::graph::value::Value;

    #[test]
    fn subd_box_produces_basisvorm() {
        let component = ComponentKind::Box;
        let inputs = vec![Value::List(vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 1.0, 1.0]),
        ])];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("evaluatie slaagt");
        let subd_value = outputs.get(PIN_OUTPUT_SUBD).expect("SubD output");
        let subd = Subd::from_value(subd_value).expect("SubD structuur");
        assert_eq!(subd.vertices.len(), 8);
        assert_eq!(subd.faces.len(), 6);
    }

    #[test]
    fn edge_tags_worden_toegepast() {
        let base = ComponentKind::Box
            .evaluate(
                &[Value::List(vec![
                    Value::Point([0.0, 0.0, 0.0]),
                    Value::Point([1.0, 1.0, 1.0]),
                ])],
                &MetaMap::new(),
            )
            .unwrap();
        let subd_value = base.get(PIN_OUTPUT_SUBD).unwrap().clone();
        let outputs = ComponentKind::EdgeTags
            .evaluate(
                &[
                    subd_value,
                    Value::Text("crease".into()),
                    Value::List(vec![Value::Number(0.0)]),
                ],
                &MetaMap::new(),
            )
            .unwrap();
        let updated = Subd::from_value(outputs.get(PIN_OUTPUT_SUBD).unwrap()).unwrap();
        assert!(
            updated
                .edges
                .iter()
                .any(|edge| edge.id == 0 && edge.tag.to_string() == "crease")
        );
    }

    #[test]
    fn faces_melden_aantal_vertices() {
        let base = ComponentKind::Box
            .evaluate(
                &[Value::List(vec![
                    Value::Point([0.0, 0.0, 0.0]),
                    Value::Point([1.0, 1.0, 1.0]),
                ])],
                &MetaMap::new(),
            )
            .unwrap();
        let outputs = ComponentKind::Faces
            .evaluate(
                &[base.get(PIN_OUTPUT_SUBD).unwrap().clone()],
                &MetaMap::new(),
            )
            .unwrap();
        let counts = outputs.get(PIN_OUTPUT_FACE_COUNTS).unwrap();
        let Value::List(entries) = counts else {
            panic!("verwachte lijst");
        };
        assert!(
            entries.iter().all(
                |value| matches!(value, Value::Number(number) if (*number - 4.0).abs() < 1e-6)
            )
        );
    }

    #[test]
    fn vertices_leveren_tags() {
        let base = ComponentKind::Box
            .evaluate(
                &[Value::List(vec![
                    Value::Point([0.0, 0.0, 0.0]),
                    Value::Point([1.0, 1.0, 1.0]),
                ])],
                &MetaMap::new(),
            )
            .unwrap();
        let outputs = ComponentKind::VerticesDetailed
            .evaluate(
                &[base.get(PIN_OUTPUT_SUBD).unwrap().clone()],
                &MetaMap::new(),
            )
            .unwrap();
        let tags = outputs.get(PIN_OUTPUT_VERTEX_TAGS).unwrap();
        let Value::List(entries) = tags else {
            panic!("verwachte taglijst");
        };
        assert_eq!(entries.len(), 8);
    }
}
