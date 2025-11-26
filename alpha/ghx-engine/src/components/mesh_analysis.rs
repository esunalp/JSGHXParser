//! Grasshopper Mesh Analysis componenten.

use super::{Component, ComponentError, ComponentResult};
use crate::components::coerce::{coerce_surface, coerce_text};
use crate::graph::node::MetaMap;
use crate::graph::value::Value;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentKind {
    MeshInclusion,
    MeshDepth,
    FaceBoundaries,
    MeshEdges,
    MeshClosestPoint,
    DeconstructFace,
    MeshEval,
    DeconstructMesh,
    FaceNormals,
    FaceCircles,
}

impl ComponentKind {
    pub fn evaluate(self, inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        match self {
            Self::MeshInclusion => MeshInclusion.evaluate(inputs, meta),
            Self::MeshDepth => MeshDepth.evaluate(inputs, meta),
            Self::FaceBoundaries => FaceBoundaries.evaluate(inputs, meta),
            Self::MeshEdges => MeshEdges.evaluate(inputs, meta),
            Self::MeshClosestPoint => MeshClosestPoint.evaluate(inputs, meta),
            Self::DeconstructFace => DeconstructFace.evaluate(inputs, meta),
            Self::MeshEval => MeshEval.evaluate(inputs, meta),
            Self::DeconstructMesh => DeconstructMesh.evaluate(inputs, meta),
            Self::FaceNormals => FaceNormals.evaluate(inputs, meta),
            Self::FaceCircles => FaceCircles.evaluate(inputs, meta),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::MeshInclusion => "Mesh Inclusion",
            Self::MeshDepth => "Mesh Depth",
            Self::FaceBoundaries => "Face Boundaries",
            Self::MeshEdges => "Mesh Edges",
            Self::MeshClosestPoint => "Mesh Closest Point",
            Self::DeconstructFace => "Deconstruct Face",
            Self::MeshEval => "Mesh Eval",
            Self::DeconstructMesh => "Deconstruct Mesh",
            Self::FaceNormals => "Face Normals",
            Self::FaceCircles => "Face Circles",
        }
    }
}

pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["01e3991d-18bd-474f-9fbd-076a8700159f"],
        names: &["Mesh Inclusion", "MInc"],
        kind: ComponentKind::MeshInclusion,
    },
    Registration {
        guids: &["07a3b2a0-c4d0-4638-9044-39ac4681e782"],
        names: &["Mesh Depth", "MDepth"],
        kind: ComponentKind::MeshDepth,
    },
    Registration {
        guids: &[
            "08d45f16-c708-4ede-8fd3-b70a0a7abd8f",
            "0b4ac802-fc4a-4201-9c66-0078b837c1eb",
        ],
        names: &["Face Boundaries", "FaceB"],
        kind: ComponentKind::FaceBoundaries,
    },
    Registration {
        guids: &["2b9bf01d-5fe5-464c-b0b3-b469eb5f2efb"],
        names: &["Mesh Edges", "MEdges"],
        kind: ComponentKind::MeshEdges,
    },
    Registration {
        guids: &["a559fee2-4b76-4370-8042-c7440cd75049"],
        names: &["Mesh Closest Point", "MeshCP"],
        kind: ComponentKind::MeshClosestPoint,
    },
    Registration {
        guids: &["aab142b1-b870-46de-8e86-654c9a554d90"],
        names: &["Deconstruct Face", "DeFace"],
        kind: ComponentKind::DeconstructFace,
    },
    Registration {
        guids: &["b2dc090f-b022-4264-8889-87e22979336e"],
        names: &["Mesh Eval", "MEval"],
        kind: ComponentKind::MeshEval,
    },
    Registration {
        guids: &["ba2d8f57-0738-42b4-b5a5-fe4d853517eb"],
        names: &["Deconstruct Mesh", "DeMesh"],
        kind: ComponentKind::DeconstructMesh,
    },
    Registration {
        guids: &["cb4ca22c-3419-4962-a078-ad4ff7f1f929"],
        names: &["Face Normals", "FaceN"],
        kind: ComponentKind::FaceNormals,
    },
    Registration {
        guids: &["d8cf1555-a0d5-43cb-8a10-46f8c014db3a"],
        names: &["Face Circles", "FaceC"],
        kind: ComponentKind::FaceCircles,
    },
];

#[derive(Debug, Default, Clone, Copy)]
pub struct MeshInclusion;
#[derive(Debug, Default, Clone, Copy)]
pub struct MeshDepth;
#[derive(Debug, Default, Clone, Copy)]
pub struct FaceBoundaries;
#[derive(Debug, Default, Clone, Copy)]
pub struct MeshEdges;
#[derive(Debug, Default, Clone, Copy)]
pub struct MeshClosestPoint;
#[derive(Debug, Default, Clone, Copy)]
pub struct DeconstructFace;
#[derive(Debug, Default, Clone, Copy)]
pub struct MeshEval;
#[derive(Debug, Default, Clone, Copy)]
pub struct DeconstructMesh;
#[derive(Debug, Default, Clone, Copy)]
pub struct FaceNormals;
#[derive(Debug, Default, Clone, Copy)]
pub struct FaceCircles;

impl Component for MeshInclusion {
    fn evaluate(&self, _: &[Value], _: &MetaMap) -> ComponentResult {
        Err(ComponentError::new(
            "Component 'Mesh Inclusion' is not yet implemented.",
        ))
    }
}
impl Component for MeshDepth {
    fn evaluate(&self, _: &[Value], _: &MetaMap) -> ComponentResult {
        Err(ComponentError::new(
            "Component 'Mesh Depth' is not yet implemented.",
        ))
    }
}

impl Component for FaceBoundaries {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new("Input 'Mesh' is missing."));
        }
        let surface = coerce_surface(&inputs[0])?;

        let polylines: Vec<Value> = surface
            .faces
            .iter()
            .map(|face| {
                let polyline_vertices: Vec<Value> = face
                    .iter()
                    .map(|&vertex_index| Value::Point(surface.vertices[vertex_index as usize]))
                    .collect();
                Value::List(polyline_vertices)
            })
            .collect();

        let mut outputs = BTreeMap::new();
        outputs.insert("B".to_owned(), Value::List(polylines));
        Ok(outputs)
    }
}

impl Component for MeshEdges {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new("Input 'Mesh' is missing."));
        }
        let surface = coerce_surface(&inputs[0])?;

        let mut edge_counts: HashMap<(u32, u32), u32> = HashMap::new();

        for face in surface.faces {
            for i in 0..face.len() {
                let v1 = face[i];
                let v2 = face[(i + 1) % face.len()];
                let edge = if v1 < v2 { (v1, v2) } else { (v2, v1) };
                *edge_counts.entry(edge).or_insert(0) += 1;
            }
        }

        let mut naked_edges = Vec::new();
        let mut interior_edges = Vec::new();
        let mut non_manifold_edges = Vec::new();

        for (edge, count) in edge_counts {
            let v1 = surface.vertices[edge.0 as usize];
            let v2 = surface.vertices[edge.1 as usize];
            let line = Value::List(vec![Value::Point(v1), Value::Point(v2)]);

            match count {
                1 => naked_edges.push(line),
                2 => interior_edges.push(line),
                _ => non_manifold_edges.push(line),
            }
        }

        let mut outputs = BTreeMap::new();
        outputs.insert("E1".to_owned(), Value::List(naked_edges));
        outputs.insert("E2".to_owned(), Value::List(interior_edges));
        outputs.insert("E3".to_owned(), Value::List(non_manifold_edges));

        Ok(outputs)
    }
}

impl Component for MeshClosestPoint {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new(
                "Inputs 'Point' and 'Mesh' are required.",
            ));
        }
        let point = match inputs[0] {
            Value::Point(p) => p,
            _ => return Err(ComponentError::new("Input 'Point' must be a single Point.")),
        };
        let surface = coerce_surface(&inputs[1])?;

        let mut min_dist_sq = f64::INFINITY;
        let mut closest_point = [0.0, 0.0, 0.0];
        let mut closest_face_index = 0;
        let mut closest_params = [0.0, 0.0, 0.0];

        for (i, face) in surface.faces.iter().enumerate() {
            if face.len() < 3 {
                continue;
            }

            // Triangulate polygon faces for simplicity
            for j in 1..face.len() - 1 {
                let v0 = surface.vertices[face[0] as usize];
                let v1 = surface.vertices[face[j] as usize];
                let v2 = surface.vertices[face[j + 1] as usize];

                let (dist_sq, p, params) = closest_point_on_triangle(point, v0, v1, v2);

                if dist_sq < min_dist_sq {
                    min_dist_sq = dist_sq;
                    closest_point = p;
                    closest_face_index = i;
                    closest_params = params;
                }
            }
        }

        let mut outputs = BTreeMap::new();
        outputs.insert("P".to_owned(), Value::Point(closest_point));
        outputs.insert("I".to_owned(), Value::Number(closest_face_index as f64));
        outputs.insert(
            "Parameter".to_owned(),
            Value::List(vec![
                Value::Number(closest_params[0]),
                Value::Number(closest_params[1]),
                Value::Number(closest_params[2]),
            ]),
        );

        Ok(outputs)
    }
}

fn closest_point_on_triangle(
    p: [f64; 3],
    a: [f64; 3],
    b: [f64; 3],
    c: [f64; 3],
) -> (f64, [f64; 3], [f64; 3]) {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let ap = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];

    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);

    // Barycentric coordinates (u, v, w)
    let d3 = dot(ab, ab);
    let d4 = dot(ac, ac);
    let d5 = dot(ab, ac);
    let denom = d3 * d4 - d5 * d5;

    let mut v = (d4 * d1 - d5 * d2) / denom;
    let mut w = (d3 * d2 - d5 * d1) / denom;
    let mut u = 1.0 - v - w;

    if u < 0.0 {
        u = 0.0;
    }
    if v < 0.0 {
        v = 0.0;
    }
    if w < 0.0 {
        w = 0.0;
    }

    let sum = u + v + w;
    if sum > 1.0 {
        u /= sum;
        v /= sum;
        w /= sum;
    }

    let closest_point = [
        a[0] + v * ab[0] + w * ac[0],
        a[1] + v * ab[1] + w * ac[1],
        a[2] + v * ab[2] + w * ac[2],
    ];

    let dist_sq = dist_sq(p, closest_point);
    (dist_sq, closest_point, [u, v, w])
}

fn dot(u: [f64; 3], v: [f64; 3]) -> f64 {
    u[0] * v[0] + u[1] * v[1] + u[2] * v[2]
}

fn dist_sq(p1: [f64; 3], p2: [f64; 3]) -> f64 {
    (p1[0] - p2[0]).powi(2) + (p1[1] - p2[1]).powi(2) + (p1[2] - p2[2]).powi(2)
}

impl Component for MeshEval {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new(
                "Inputs 'Mesh' and 'Parameter' are required.",
            ));
        }
        let surface = coerce_surface(&inputs[0])?;
        let (face_index_f64, u, v) = match &inputs[1] {
            Value::List(list) if list.len() == 3 => {
                let face_idx = match list[0] {
                    Value::Number(n) => n,
                    _ => return Err(ComponentError::new("Invalid parameter format.")),
                };
                let u_coord = match list[1] {
                    Value::Number(n) => n,
                    _ => return Err(ComponentError::new("Invalid parameter format.")),
                };
                let v_coord = match list[2] {
                    Value::Number(n) => n,
                    _ => return Err(ComponentError::new("Invalid parameter format.")),
                };
                (face_idx, u_coord, v_coord)
            }
            _ => {
                return Err(ComponentError::new(
                    "Input 'Parameter' must be a list of three numbers [face_idx, u, v].",
                ));
            }
        };

        let face_index = face_index_f64.round() as usize;
        if face_index >= surface.faces.len() {
            return Err(ComponentError::new(format!(
                "Face index {} is out of bounds.",
                face_index
            )));
        }

        let face = &surface.faces[face_index];
        if face.len() < 3 {
            return Err(ComponentError::new(format!(
                "Face {} is not a valid triangle.",
                face_index
            )));
        }

        // We assume the barycentric coordinates are for the first triangle of the face.
        let v0 = surface.vertices[face[0] as usize];
        let v1 = surface.vertices[face[1] as usize];
        let v2 = surface.vertices[face[2] as usize];

        let w = 1.0 - u - v;

        let point = [
            w * v0[0] + u * v1[0] + v * v2[0],
            w * v0[1] + u * v1[1] + v * v2[1],
            w * v0[2] + u * v1[2] + v * v2[2],
        ];

        let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let mut normal = [
            edge1[1] * edge2[2] - edge1[2] * edge2[1],
            edge1[2] * edge2[0] - edge1[0] * edge2[2],
            edge1[0] * edge2[1] - edge1[1] * edge2[0],
        ];
        let mag = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
        if mag > 1e-12 {
            normal[0] /= mag;
            normal[1] /= mag;
            normal[2] /= mag;
        }

        let mut outputs = BTreeMap::new();
        outputs.insert("P".to_owned(), Value::Point(point));
        outputs.insert("N".to_owned(), Value::Vector(normal));
        outputs.insert("C".to_owned(), Value::Null); // Color not supported

        Ok(outputs)
    }
}

impl Component for FaceCircles {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new("Input 'Mesh' is missing."));
        }
        let surface = coerce_surface(&inputs[0])?;

        let mut centers = Vec::new();
        let mut ratios = Vec::new();

        for face in surface.faces {
            if face.len() < 3 {
                continue;
            }

            // Triangulate polygon faces
            for i in 1..face.len() - 1 {
                let v0 = surface.vertices[face[0] as usize];
                let v1 = surface.vertices[face[i] as usize];
                let v2 = surface.vertices[face[i + 1] as usize];

                let a_sq =
                    (v2[0] - v1[0]).powi(2) + (v2[1] - v1[1]).powi(2) + (v2[2] - v1[2]).powi(2);
                let b_sq =
                    (v2[0] - v0[0]).powi(2) + (v2[1] - v0[1]).powi(2) + (v2[2] - v0[2]).powi(2);
                let c_sq =
                    (v1[0] - v0[0]).powi(2) + (v1[1] - v0[1]).powi(2) + (v1[2] - v0[2]).powi(2);

                let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
                let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
                let cross = [
                    edge1[1] * edge2[2] - edge1[2] * edge2[1],
                    edge1[2] * edge2[0] - edge1[0] * edge2[2],
                    edge1[0] * edge2[1] - edge1[1] * edge2[0],
                ];
                let area_doubled_sq = cross[0].powi(2) + cross[1].powi(2) + cross[2].powi(2);

                if area_doubled_sq < 1e-12 {
                    continue; // Collinear vertices
                }

                let w0_num = a_sq * (-a_sq + b_sq + c_sq);
                let w1_num = b_sq * (a_sq - b_sq + c_sq);
                let w2_num = c_sq * (a_sq + b_sq - c_sq);
                let denom = w0_num + w1_num + w2_num;

                if denom.abs() < 1e-12 {
                    continue; // Should be covered by area check, but as a safeguard
                }

                let center = [
                    (w0_num * v0[0] + w1_num * v1[0] + w2_num * v2[0]) / denom,
                    (w0_num * v0[1] + w1_num * v1[1] + w2_num * v2[1]) / denom,
                    (w0_num * v0[2] + w1_num * v1[2] + w2_num * v2[2]) / denom,
                ];
                centers.push(Value::Point(center));

                let area = area_doubled_sq.sqrt() * 0.5;
                let longest_edge_sq = a_sq.max(b_sq).max(c_sq);
                let ratio = 2.0 * area / longest_edge_sq;
                ratios.push(Value::Number(ratio));
            }
        }

        let mut outputs = BTreeMap::new();
        outputs.insert("C".to_owned(), Value::List(centers));
        outputs.insert("R".to_owned(), Value::List(ratios));
        Ok(outputs)
    }
}

impl Component for DeconstructMesh {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new("Input 'Mesh' is missing."));
        }

        let mesh = &inputs[0];

        let (vertices, faces) = match mesh {
            Value::Surface { vertices, faces } => (vertices, faces),
            _ => {
                return Err(ComponentError::new(format!(
                    "Expected a Surface, got {}",
                    mesh.kind()
                )));
            }
        };

        let vertices_list: Vec<Value> = vertices.iter().map(|&v| Value::Point(v)).collect();

        let faces_list: Vec<Value> = faces
            .iter()
            .map(|face| {
                let mut face_str = face.len().to_string();
                for index in face {
                    face_str.push(';');
                    face_str.push_str(&index.to_string());
                }
                Value::Text(face_str)
            })
            .collect();

        let face_normals: Vec<[f64; 3]> = faces
            .iter()
            .map(|face| {
                if face.len() < 3 {
                    return [0.0, 0.0, 0.0];
                }
                let v0 = vertices[face[0] as usize];
                let v1 = vertices[face[1] as usize];
                let v2 = vertices[face[2] as usize];
                let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
                let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
                let mut normal = [
                    edge1[1] * edge2[2] - edge1[2] * edge2[1],
                    edge1[2] * edge2[0] - edge1[0] * edge2[2],
                    edge1[0] * edge2[1] - edge1[1] * edge2[0],
                ];
                let mag = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
                if mag > 1e-12 {
                    normal[0] /= mag;
                    normal[1] /= mag;
                    normal[2] /= mag;
                }
                normal
            })
            .collect();

        let mut vertex_normals = vec![[0.0, 0.0, 0.0]; vertices.len()];
        for (face_idx, face) in faces.iter().enumerate() {
            for &vertex_idx in face {
                let normal = face_normals[face_idx];
                vertex_normals[vertex_idx as usize][0] += normal[0];
                vertex_normals[vertex_idx as usize][1] += normal[1];
                vertex_normals[vertex_idx as usize][2] += normal[2];
            }
        }

        let normals_list: Vec<Value> = vertex_normals
            .iter_mut()
            .map(|normal| {
                let mag = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
                if mag > 1e-12 {
                    normal[0] /= mag;
                    normal[1] /= mag;
                    normal[2] /= mag;
                }
                Value::Vector(*normal)
            })
            .collect();

        let mut outputs = BTreeMap::new();
        outputs.insert("V".to_owned(), Value::List(vertices_list));
        outputs.insert("F".to_owned(), Value::List(faces_list));
        outputs.insert("C".to_owned(), Value::Null);
        outputs.insert("N".to_owned(), Value::List(normals_list));

        Ok(outputs)
    }
}

impl Component for FaceNormals {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new("Input 'Mesh' is missing."));
        }
        let surface = coerce_surface(&inputs[0])?;

        let mut centers = Vec::new();
        let mut normals = Vec::new();

        for face in surface.faces {
            if face.len() < 3 {
                continue;
            }

            let mut center = [0.0, 0.0, 0.0];
            for &vertex_index in face {
                let v = surface.vertices[vertex_index as usize];
                center[0] += v[0];
                center[1] += v[1];
                center[2] += v[2];
            }
            let len = face.len() as f64;
            center[0] /= len;
            center[1] /= len;
            center[2] /= len;
            centers.push(Value::Point(center));

            let v0 = surface.vertices[face[0] as usize];
            let v1 = surface.vertices[face[1] as usize];
            let v2 = surface.vertices[face[2] as usize];

            let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

            let mut normal = [
                edge1[1] * edge2[2] - edge1[2] * edge2[1],
                edge1[2] * edge2[0] - edge1[0] * edge2[2],
                edge1[0] * edge2[1] - edge1[1] * edge2[0],
            ];
            let mag = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
            if mag > 1e-12 {
                normal[0] /= mag;
                normal[1] /= mag;
                normal[2] /= mag;
            }
            normals.push(Value::Vector(normal));
        }

        let mut outputs = BTreeMap::new();
        outputs.insert("C".to_owned(), Value::List(centers));
        outputs.insert("N".to_owned(), Value::List(normals));

        Ok(outputs)
    }
}

impl Component for DeconstructFace {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new("Input 'Face' is missing."));
        }
        let face_str = coerce_text(&inputs[0])?;
        let parts: Vec<Result<f64, _>> = face_str.split(';').map(|s| s.parse()).collect();
        if parts.iter().any(|p| p.is_err()) {
            return Err(ComponentError::new("Invalid face format."));
        }
        let indices: Vec<f64> = parts.into_iter().map(|p| p.unwrap()).collect();

        let a = indices
            .get(1)
            .map(|&v| Value::Number(v))
            .unwrap_or(Value::Null);
        let b = indices
            .get(2)
            .map(|&v| Value::Number(v))
            .unwrap_or(Value::Null);
        let c = indices
            .get(3)
            .map(|&v| Value::Number(v))
            .unwrap_or(Value::Null);
        let d = if indices.len() > 4 {
            indices
                .get(4)
                .map(|&v| Value::Number(v))
                .unwrap_or(Value::Null)
        } else {
            c.clone()
        };

        let mut outputs = BTreeMap::new();
        outputs.insert("A".to_owned(), a);
        outputs.insert("B".to_owned(), b);
        outputs.insert("C".to_owned(), c);
        outputs.insert("D".to_owned(), d);

        Ok(outputs)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Component, DeconstructFace, DeconstructMesh, FaceBoundaries, FaceCircles, FaceNormals,
        MeshClosestPoint, MeshEdges, MeshEval,
    };
    use crate::graph::{node::MetaMap, value::Value};

    #[test]
    fn test_mesh_eval() {
        let component = MeshEval;
        let vertices = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]];
        let faces = vec![vec![0, 1, 2]];
        let mesh = Value::Surface {
            vertices: vertices.clone(),
            faces,
        };
        let params = Value::List(vec![
            Value::Number(0.0),
            Value::Number(0.5),
            Value::Number(0.5),
        ]);

        let outputs = component
            .evaluate(&[mesh, params], &MetaMap::new())
            .expect("Component should succeed");

        let point = outputs.get("P").unwrap();
        if let Value::Point(p) = point {
            assert!((p[0] - 1.5).abs() < 1e-6);
            assert!((p[1] - 1.0).abs() < 1e-6);
            assert!((p[2] - 0.0).abs() < 1e-6);
        } else {
            panic!("Expected a Point");
        }

        let normal = outputs.get("N").unwrap();
        if let Value::Vector(n) = normal {
            assert!((n[0] - 0.0).abs() < 1e-6);
            assert!((n[1] - 0.0).abs() < 1e-6);
            assert!((n[2] - 1.0).abs() < 1e-6);
        } else {
            panic!("Expected a Vector");
        }
    }

    #[test]
    fn test_mesh_closest_point() {
        let component = MeshClosestPoint;
        let vertices = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]];
        let faces = vec![vec![0, 1, 2]];
        let mesh = Value::Surface {
            vertices: vertices.clone(),
            faces,
        };
        let point = Value::Point([1.0, 1.0, 1.0]);

        let outputs = component
            .evaluate(&[point, mesh], &MetaMap::new())
            .expect("Component should succeed");

        let closest_point = outputs.get("P").unwrap();
        if let Value::Point(p) = closest_point {
            assert!((p[0] - 1.0).abs() < 1e-6);
            assert!((p[1] - 1.0).abs() < 1e-6);
            assert!((p[2] - 0.0).abs() < 1e-6);
        } else {
            panic!("Expected a Point");
        }

        let face_index = outputs.get("I").unwrap();
        if let Value::Number(i) = face_index {
            assert_eq!(*i, 0.0);
        } else {
            panic!("Expected a Number for face index");
        }
    }

    #[test]
    fn test_face_circles() {
        let component = FaceCircles;
        let vertices = vec![
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let faces = vec![vec![0, 1, 2], vec![0, 2, 3]]; // One triangle, one quad
        let mesh = Value::Surface {
            vertices: vertices.clone(),
            faces,
        };

        let outputs = component
            .evaluate(&[mesh], &MetaMap::new())
            .expect("Component should succeed");

        let centers = outputs.get("C").unwrap();
        if let Value::List(points) = centers {
            assert_eq!(points.len(), 2);
        } else {
            panic!("Expected a list of points for centers");
        }

        let ratios = outputs.get("R").unwrap();
        if let Value::List(numbers) = ratios {
            assert_eq!(numbers.len(), 2);
        } else {
            panic!("Expected a list of numbers for ratios");
        }
    }

    #[test]
    fn test_deconstruct_mesh() {
        let component = DeconstructMesh;
        let vertices = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let faces = vec![vec![0, 1, 2], vec![0, 2, 3]];
        let mesh = Value::Surface {
            vertices: vertices.clone(),
            faces,
        };

        let outputs = component
            .evaluate(&[mesh], &MetaMap::new())
            .expect("Component should succeed");

        let output_vertices = outputs.get("V").unwrap();
        if let Value::List(points) = output_vertices {
            assert_eq!(points.len(), 4);
            for (i, point) in points.iter().enumerate() {
                if let Value::Point(p) = point {
                    assert_eq!(*p, vertices[i]);
                } else {
                    panic!("Expected a Point");
                }
            }
        } else {
            panic!("Expected a List of Points");
        }

        let output_faces = outputs.get("F").unwrap();
        if let Value::List(faces) = output_faces {
            assert_eq!(faces.len(), 2);
            assert_eq!(faces[0], Value::Text("3;0;1;2".to_string()));
            assert_eq!(faces[1], Value::Text("3;0;2;3".to_string()));
        } else {
            panic!("Expected a List of Text for faces");
        }

        assert!(matches!(outputs.get("C"), Some(Value::Null)));

        let normals = outputs.get("N").unwrap();
        if let Value::List(vectors) = normals {
            assert_eq!(vectors.len(), 4);
            // All faces have the same normal [0, 0, 1] in this simple case.
            for vector in vectors {
                if let Value::Vector(v) = vector {
                    assert!((v[0] - 0.0).abs() < 1e-6);
                    assert!((v[1] - 0.0).abs() < 1e-6);
                    assert!((v[2] - 1.0).abs() < 1e-6);
                } else {
                    panic!("Expected a Vector");
                }
            }
        } else {
            panic!("Expected a List of Vectors for normals");
        }
    }

    #[test]
    fn test_face_boundaries() {
        let component = FaceBoundaries;
        let vertices = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let faces = vec![vec![0, 1, 2], vec![0, 2, 3]];
        let mesh = Value::Surface {
            vertices: vertices.clone(),
            faces,
        };

        let outputs = component
            .evaluate(&[mesh], &MetaMap::new())
            .expect("Component should succeed");

        let boundaries = outputs.get("B").unwrap();
        if let Value::List(polylines) = boundaries {
            assert_eq!(polylines.len(), 2);

            if let Value::List(points) = &polylines[0] {
                assert_eq!(points.len(), 3);
                assert_eq!(points[0], Value::Point(vertices[0]));
                assert_eq!(points[1], Value::Point(vertices[1]));
                assert_eq!(points[2], Value::Point(vertices[2]));
            } else {
                panic!("Expected a list of points for the first polyline");
            }

            if let Value::List(points) = &polylines[1] {
                assert_eq!(points.len(), 3);
                assert_eq!(points[0], Value::Point(vertices[0]));
                assert_eq!(points[1], Value::Point(vertices[2]));
                assert_eq!(points[2], Value::Point(vertices[3]));
            } else {
                panic!("Expected a list of points for the second polyline");
            }
        } else {
            panic!("Expected a list of polylines");
        }
    }

    #[test]
    fn test_face_normals() {
        let component = FaceNormals;
        let vertices = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]];
        let faces = vec![vec![0, 1, 2]];
        let mesh = Value::Surface {
            vertices: vertices.clone(),
            faces,
        };

        let outputs = component
            .evaluate(&[mesh], &MetaMap::new())
            .expect("Component should succeed");

        let centers = outputs.get("C").unwrap();
        if let Value::List(points) = centers {
            assert_eq!(points.len(), 1);
            if let Value::Point(p) = points[0] {
                assert!((p[0] - 2.0 / 3.0).abs() < 1e-6);
                assert!((p[1] - 1.0 / 3.0).abs() < 1e-6);
                assert!((p[2] - 0.0).abs() < 1e-6);
            } else {
                panic!("Expected a Point");
            }
        } else {
            panic!("Expected a list of points for centers");
        }

        let normals = outputs.get("N").unwrap();
        if let Value::List(vectors) = normals {
            assert_eq!(vectors.len(), 1);
            if let Value::Vector(v) = vectors[0] {
                assert!((v[0] - 0.0).abs() < 1e-6);
                assert!((v[1] - 0.0).abs() < 1e-6);
                assert!((v[2] - 1.0).abs() < 1e-6);
            } else {
                panic!("Expected a Vector");
            }
        } else {
            panic!("Expected a list of vectors for normals");
        }
    }

    #[test]
    fn test_deconstruct_face() {
        let component = DeconstructFace;

        let quad_face = Value::Text("4;10;20;30;40".to_string());
        let outputs = component.evaluate(&[quad_face], &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("A").unwrap(), &Value::Number(10.0));
        assert_eq!(outputs.get("B").unwrap(), &Value::Number(20.0));
        assert_eq!(outputs.get("C").unwrap(), &Value::Number(30.0));
        assert_eq!(outputs.get("D").unwrap(), &Value::Number(40.0));

        let tri_face = Value::Text("3;5;6;7".to_string());
        let outputs = component.evaluate(&[tri_face], &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("A").unwrap(), &Value::Number(5.0));
        assert_eq!(outputs.get("B").unwrap(), &Value::Number(6.0));
        assert_eq!(outputs.get("C").unwrap(), &Value::Number(7.0));
        assert_eq!(outputs.get("D").unwrap(), &Value::Number(7.0));
    }

    #[test]
    fn test_mesh_edges() {
        let component = MeshEdges;
        let vertices = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let faces = vec![vec![0, 1, 2], vec![0, 2, 3]];
        let mesh = Value::Surface { vertices, faces };

        let outputs = component.evaluate(&[mesh], &MetaMap::new()).unwrap();

        let naked = outputs.get("E1").unwrap();
        let interior = outputs.get("E2").unwrap();
        let non_manifold = outputs.get("E3").unwrap();

        if let (Value::List(naked), Value::List(interior), Value::List(non_manifold)) =
            (naked, interior, non_manifold)
        {
            assert_eq!(naked.len(), 4);
            assert_eq!(interior.len(), 1);
            assert_eq!(non_manifold.len(), 0);
        } else {
            panic!("Expected lists for edge outputs");
        }
    }
}
