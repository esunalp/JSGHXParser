//! Grasshopper-componenten voor het maken van primitieve mesh-geometrie.
//!
//! Categorie: Mesh > Primitive
//!
//! # Mesh Engine Integration (Phase 3)
//!
//! All mesh primitive components now output `Value::Mesh` as the primary type.
//! This provides:
//! - Indexed triangle list with flat indices (divisible by 3)
//! - Optional per-vertex normals for smooth shading
//! - Optional per-vertex UVs (for textured primitives)
//! - Mesh diagnostics with vertex/triangle counts
//!
//! The legacy `Value::Surface` output is no longer emitted. Downstream consumers
//! should use `expect_mesh_like()` for backward compatibility with both types.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::{MeshDiagnostics, Value};

use super::{Component, ComponentError, ComponentResult, coerce};

/// Output pin name for the mesh.
const OUTPUT_M: &str = "M";

// ============================================================================
// Mesh Primitive Helpers
// ============================================================================

/// Triangulates polygon faces into a flat triangle index list.
///
/// This function converts mixed polygon faces (triangles, quads, n-gons) into
/// a flat list of triangle indices suitable for `Value::Mesh`.
///
/// # Triangulation Strategy
///
/// - **Triangles** (3 vertices): passed through unchanged
/// - **Quads** (4 vertices): split into 2 triangles (0,1,2) and (0,2,3)
/// - **N-gons** (5+ vertices): fan triangulation from first vertex
/// - **Degenerate** (<3 vertices): skipped with warning in diagnostics
///
/// # Arguments
///
/// * `faces` - Polygon faces as lists of vertex indices
/// * `vertex_count` - Total number of vertices (for bounds checking)
///
/// # Returns
///
/// A tuple of (triangle_indices, degenerate_count) where degenerate_count
/// is the number of faces that were skipped due to insufficient vertices.
fn triangulate_faces(faces: &[Vec<u32>], vertex_count: usize) -> (Vec<u32>, usize) {
    let mut indices = Vec::with_capacity(faces.len() * 3);
    let mut degenerate_count = 0;
    let max_idx = vertex_count as u32;

    for face in faces {
        let n = face.len();
        
        // Skip faces with fewer than 3 vertices
        if n < 3 {
            degenerate_count += 1;
            continue;
        }

        // Validate all indices are in bounds
        if face.iter().any(|&idx| idx >= max_idx) {
            degenerate_count += 1;
            continue;
        }

        // Triangulate using fan method from first vertex
        // For triangle: just add the 3 indices
        // For quad: (0,1,2), (0,2,3)
        // For n-gon: (0,1,2), (0,2,3), (0,3,4), ...
        for i in 1..(n - 1) {
            indices.push(face[0]);
            indices.push(face[i] as u32);
            indices.push(face[i + 1] as u32);
        }
    }

    (indices, degenerate_count)
}

/// Computes smooth vertex normals by averaging adjacent face normals.
///
/// Each vertex normal is the normalized average of the normals of all
/// triangles that share that vertex. This produces smooth shading.
///
/// # Arguments
///
/// * `vertices` - Vertex positions
/// * `indices` - Triangle indices (length must be divisible by 3)
///
/// # Returns
///
/// Per-vertex normals matching the vertex array length. Vertices not
/// referenced by any triangle get a default normal of (0, 0, 1).
fn compute_smooth_normals(vertices: &[[f64; 3]], indices: &[u32]) -> Vec<[f64; 3]> {
    let mut normals = vec![[0.0_f64; 3]; vertices.len()];

    // Accumulate face normals at each vertex
    for tri in indices.chunks_exact(3) {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;

        if i0 >= vertices.len() || i1 >= vertices.len() || i2 >= vertices.len() {
            continue;
        }

        let v0 = vertices[i0];
        let v1 = vertices[i1];
        let v2 = vertices[i2];

        // Edge vectors
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

        // Cross product = face normal (not normalized, so area-weighted)
        let n = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];

        // Accumulate at each vertex
        for &idx in &[i0, i1, i2] {
            normals[idx][0] += n[0];
            normals[idx][1] += n[1];
            normals[idx][2] += n[2];
        }
    }

    // Normalize all accumulated normals
    for normal in &mut normals {
        let len_sq = normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2];
        if len_sq > 1e-12 {
            let len = len_sq.sqrt();
            normal[0] /= len;
            normal[1] /= len;
            normal[2] /= len;
        } else {
            // Default to +Z for degenerate normals
            *normal = [0.0, 0.0, 1.0];
        }
    }

    normals
}

/// Creates a `Value::Mesh` output from vertices and polygon faces.
///
/// This is the standard helper for mesh primitive components. It:
/// 1. Triangulates polygon faces to a flat index list
/// 2. Computes smooth vertex normals
/// 3. Builds mesh diagnostics
///
/// # Arguments
///
/// * `vertices` - Vertex positions
/// * `faces` - Polygon faces (triangles, quads, or n-gons)
///
/// # Returns
///
/// A `Value::Mesh` with positions, triangle indices, smooth normals,
/// and diagnostics information.
fn create_mesh_from_faces(vertices: Vec<[f64; 3]>, faces: Vec<Vec<u32>>) -> Value {
    let (indices, degenerate_count) = triangulate_faces(&faces, vertices.len());
    let normals = compute_smooth_normals(&vertices, &indices);

    let mut diagnostics = MeshDiagnostics::with_counts(vertices.len(), indices.len() / 3);
    diagnostics.degenerate_triangle_count = degenerate_count;

    Value::Mesh {
        vertices,
        indices,
        normals: Some(normals),
        uvs: None,
        diagnostics: Some(diagnostics),
    }
}

/// Creates a `Value::Mesh` output directly from vertices and triangle indices.
///
/// Use this when you already have a triangulated mesh (indices are already
/// a flat list of triangle vertex indices).
///
/// # Arguments
///
/// * `vertices` - Vertex positions
/// * `indices` - Triangle indices (length must be divisible by 3)
///
/// # Returns
///
/// A `Value::Mesh` with positions, indices, smooth normals, and diagnostics.
#[allow(dead_code)] // Useful utility for future components
fn create_mesh_from_triangles(vertices: Vec<[f64; 3]>, indices: Vec<u32>) -> Value {
    let normals = compute_smooth_normals(&vertices, &indices);

    let diagnostics = MeshDiagnostics::with_counts(vertices.len(), indices.len() / 3);

    Value::Mesh {
        vertices,
        indices,
        normals: Some(normals),
        uvs: None,
        diagnostics: Some(diagnostics),
    }
}

// ============================================================================
// ConstructMeshComponent
// ============================================================================

/// Component to construct a mesh from vertices and faces.
#[derive(Debug, Default, Clone, Copy)]
pub struct ConstructMeshComponent;

impl Component for ConstructMeshComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 2 {
            return Err(ComponentError::new(
                "Minimaal 2 inputs (Vertices, Faces) vereist.",
            ));
        }

        let vertices = coerce_vertices(&inputs[0])?;
        let faces = coerce_faces(&inputs[1])?;

        // TODO: Kleuren-input (inputs[2]) wordt nog niet ondersteund.
        // Value::Mesh does not have a colors field; this would require
        // a separate vertex colors attribute in the future.

        // Create mesh with triangulated faces and smooth normals
        let mesh = create_mesh_from_faces(vertices, faces);

        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_M.to_owned(), mesh);

        Ok(outputs)
    }
}

/// Converteert een `Value` naar een lijst van vertices (`Vec<[f64; 3]>`).
/// Verwacht een `Value::List` van `Value::Point`.
fn coerce_vertices(value: &Value) -> Result<Vec<[f64; 3]>, ComponentError> {
    let list = value
        .expect_list()
        .map_err(|e| ComponentError::new(format!("Vertices moeten een lijst zijn: {}", e)))?;

    list.iter()
        .map(coerce::coerce_point)
        .collect::<Result<Vec<_>, _>>()
}

/// Converteert een `Value` naar een lijst van faces (`Vec<Vec<u32]>`).
/// Verwacht een `Value::List` van `Value::List` van `Value::Number`.
fn coerce_faces(value: &Value) -> Result<Vec<Vec<u32>>, ComponentError> {
    let list_of_faces = value
        .expect_list()
        .map_err(|e| ComponentError::new(format!("Faces moeten een lijst zijn: {}", e)))?;

    list_of_faces
        .iter()
        .map(|face_value| {
            let face_list = face_value.expect_list().map_err(|e| {
                ComponentError::new(format!("Elke face moet een lijst van indices zijn: {}", e))
            })?;
            face_list
                .iter()
                .map(|index_value| {
                    let number = coerce::coerce_number(index_value, None)?;
                    if number < 0.0 || number.fract() != 0.0 {
                        return Err(ComponentError::new(format!(
                            "Face index moet een niet-negatief geheel getal zijn, kreeg {}",
                            number
                        )));
                    }
                    Ok(number as u32)
                })
                .collect::<Result<Vec<u32>, _>>()
        })
        .collect::<Result<Vec<Vec<u32>>, _>>()
}

// Output pin name for faces.
const OUTPUT_F: &str = "F";

/// Component to create a triangular mesh face.
#[derive(Debug, Default, Clone, Copy)]
pub struct MeshTriangleComponent;

impl Component for MeshTriangleComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 3 {
            return Err(ComponentError::new("Minimaal 3 inputs (A, B, C) vereist."));
        }
        let a = coerce_index(&inputs[0])?;
        let b = coerce_index(&inputs[1])?;
        let c = coerce_index(&inputs[2])?;

        let face = Value::List(vec![
            Value::Number(a as f64),
            Value::Number(b as f64),
            Value::Number(c as f64),
        ]);

        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_F.to_owned(), face);
        Ok(outputs)
    }
}

/// Component to create a quadrangular mesh face.
#[derive(Debug, Default, Clone, Copy)]
pub struct MeshQuadComponent;

impl Component for MeshQuadComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 4 {
            return Err(ComponentError::new(
                "Minimaal 4 inputs (A, B, C, D) vereist.",
            ));
        }
        let a = coerce_index(&inputs[0])?;
        let b = coerce_index(&inputs[1])?;
        let c = coerce_index(&inputs[2])?;
        let d = coerce_index(&inputs[3])?;

        let face = Value::List(vec![
            Value::Number(a as f64),
            Value::Number(b as f64),
            Value::Number(c as f64),
            Value::Number(d as f64),
        ]);

        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_F.to_owned(), face);
        Ok(outputs)
    }
}

/// Coerces a `Value` into a `u32` for use as a face index.
fn coerce_index(value: &Value) -> Result<u32, ComponentError> {
    let number = coerce::coerce_number(value, None)?;
    if number < 0.0 || number.fract() != 0.0 {
        return Err(ComponentError::new(format!(
            "Index moet een niet-negatief geheel getal zijn, kreeg {}",
            number
        )));
    }
    Ok(number as u32)
}

/// Component to create a mesh sphere from square patches.
#[derive(Debug, Default, Clone, Copy)]
pub struct MeshSphereExComponent;

impl Component for MeshSphereExComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 3 {
            return Err(ComponentError::new(
                "Minimaal 3 inputs (Base, Radius, Count) vereist.",
            ));
        }

        // Base plane (inputs[0]) is ignored for now.
        let radius = coerce::coerce_number(&inputs[1], None)?;
        let count = coerce_index(&inputs[2])? as usize;

        if radius <= 0.0 {
            return Err(ComponentError::new("Radius moet groter zijn dan 0."));
        }
        if count == 0 {
            return Err(ComponentError::new("Count moet groter zijn dan 0."));
        }

        let mut vertices = Vec::new();
        let mut faces = Vec::new();

        let directions = [
            [0.0, 0.0, 1.0],  // Top
            [0.0, 0.0, -1.0], // Bottom
            [1.0, 0.0, 0.0],  // Right
            [-1.0, 0.0, 0.0], // Left
            [0.0, 1.0, 0.0],  // Front
            [0.0, -1.0, 0.0], // Back
        ];

        let mut vertex_map = BTreeMap::new();

        for dir in &directions {
            let axis_a = [dir[1], dir[2], dir[0]];
            let axis_b = [
                dir[1] * 0.0 - dir[2] * 1.0,
                dir[2] * 0.0 - dir[0] * 0.0,
                dir[0] * 1.0 - dir[1] * 0.0,
            ];

            for j in 0..=count {
                for i in 0..=count {
                    let u = (i as f64 / count as f64 - 0.5) * 2.0;
                    let v = (j as f64 / count as f64 - 0.5) * 2.0;

                    let px = dir[0] + axis_a[0] * u + axis_b[0] * v;
                    let py = dir[1] + axis_a[1] * u + axis_b[1] * v;
                    let pz = dir[2] + axis_a[2] * u + axis_b[2] * v;

                    let length = (px * px + py * py + pz * pz).sqrt();
                    let normalized = [px / length, py / length, pz / length];

                    let key = (
                        (normalized[0] * 1e6) as i64,
                        (normalized[1] * 1e6) as i64,
                        (normalized[2] * 1e6) as i64,
                    );

                    if !vertex_map.contains_key(&key) {
                        let vertex = [
                            normalized[0] * radius,
                            normalized[1] * radius,
                            normalized[2] * radius,
                        ];
                        vertex_map.insert(key, vertices.len() as u32);
                        vertices.push(vertex);
                    }
                }
            }
        }

        for dir in &directions {
            let axis_a = [dir[1], dir[2], dir[0]];
            let axis_b = [
                dir[1] * 0.0 - dir[2] * 1.0,
                dir[2] * 0.0 - dir[0] * 0.0,
                dir[0] * 1.0 - dir[1] * 0.0,
            ];

            for j in 0..count {
                for i in 0..count {
                    let mut face_indices = Vec::with_capacity(4);
                    for (u_offset, v_offset) in [(0, 0), (1, 0), (1, 1), (0, 1)] {
                        let u = ((i + u_offset) as f64 / count as f64 - 0.5) * 2.0;
                        let v = ((j + v_offset) as f64 / count as f64 - 0.5) * 2.0;

                        let px = dir[0] + axis_a[0] * u + axis_b[0] * v;
                        let py = dir[1] + axis_a[1] * u + axis_b[1] * v;
                        let pz = dir[2] + axis_a[2] * u + axis_b[2] * v;

                        let length = (px * px + py * py + pz * pz).sqrt();
                        let normalized = [px / length, py / length, pz / length];

                        let key = (
                            (normalized[0] * 1e6) as i64,
                            (normalized[1] * 1e6) as i64,
                            (normalized[2] * 1e6) as i64,
                        );
                        face_indices.push(*vertex_map.get(&key).unwrap());
                    }
                    faces.push(face_indices);
                }
            }
        }

        // Convert quad faces to triangulated Value::Mesh
        let mesh = create_mesh_from_faces(vertices, faces);

        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_M.to_owned(), mesh);

        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MeshColoursComponent;

impl Component for MeshColoursComponent {
    fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        Err(ComponentError::new(
            "Component Mesh Colours is not yet implemented.",
        ))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MeshSprayComponent;

impl Component for MeshSprayComponent {
    fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        Err(ComponentError::new(
            "Component Mesh Spray is not yet implemented.",
        ))
    }
}

use std::f64::consts::PI;

/// Enum to differentiate between the mesh primitive components.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    ConstructMesh(ConstructMeshComponent),
    MeshTriangle(MeshTriangleComponent),
    MeshQuad(MeshQuadComponent),
    MeshPlane(MeshPlaneComponent),
    MeshBox(MeshBoxComponent),
    MeshSphere(MeshSphereComponent),
    MeshSphereEx(MeshSphereExComponent),
    MeshColours(MeshColoursComponent),
    MeshSpray(MeshSprayComponent),
}

impl ComponentKind {
    pub fn evaluate(&self, inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        match self {
            Self::ConstructMesh(c) => c.evaluate(inputs, meta),
            Self::MeshTriangle(c) => c.evaluate(inputs, meta),
            Self::MeshQuad(c) => c.evaluate(inputs, meta),
            Self::MeshPlane(c) => c.evaluate(inputs, meta),
            Self::MeshBox(c) => c.evaluate(inputs, meta),
            Self::MeshSphere(c) => c.evaluate(inputs, meta),
            Self::MeshSphereEx(c) => c.evaluate(inputs, meta),
            Self::MeshColours(c) => c.evaluate(inputs, meta),
            Self::MeshSpray(c) => c.evaluate(inputs, meta),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::ConstructMesh(_) => "Construct Mesh",
            Self::MeshTriangle(_) => "Mesh Triangle",
            Self::MeshQuad(_) => "Mesh Quad",
            Self::MeshPlane(_) => "Mesh Plane",
            Self::MeshBox(_) => "Mesh Box",
            Self::MeshSphere(_) => "Mesh Sphere",
            Self::MeshSphereEx(_) => "Mesh Sphere Ex",
            Self::MeshColours(_) => "Mesh Colours",
            Self::MeshSpray(_) => "Mesh Spray",
        }
    }
}

/// Registration info for a component.
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["e2c0f9db-a862-4bd9-810c-ef2610e7a56f"],
        names: &["Construct Mesh", "ConMesh"],
        kind: ComponentKind::ConstructMesh(ConstructMeshComponent),
    },
    Registration {
        guids: &["5a4ddedd-5af9-49e5-bace-12910a8b9366"],
        names: &["Mesh Triangle", "Triangle"],
        kind: ComponentKind::MeshTriangle(MeshTriangleComponent),
    },
    Registration {
        guids: &["1cb59c86-7f6b-4e52-9a0c-6441850e9520"],
        names: &["Mesh Quad", "Quad"],
        kind: ComponentKind::MeshQuad(MeshQuadComponent),
    },
    Registration {
        guids: &[
            "8adbf481-7589-4a40-b490-006531ea001d",
            "dd8d834f-40f1-4a84-8e4b-9fa8efe7be41",
        ],
        names: &["Mesh Plane", "MPlane"],
        kind: ComponentKind::MeshPlane(MeshPlaneComponent),
    },
    Registration {
        guids: &["2696bd14-3fb5-4750-827f-86df6c31d664"],
        names: &["Mesh Box", "MBox"],
        kind: ComponentKind::MeshBox(MeshBoxComponent),
    },
    Registration {
        guids: &["0a391eac-5048-443c-9c1b-f592299b6dd6"],
        names: &["Mesh Sphere", "MSphere"],
        kind: ComponentKind::MeshSphere(MeshSphereComponent),
    },
    Registration {
        guids: &["76f85ee4-5a88-4511-8ba7-30df07e50533"],
        names: &["Mesh Sphere Ex", "MSphereEx"],
        kind: ComponentKind::MeshSphereEx(MeshSphereExComponent),
    },
    Registration {
        guids: &["d2cedf38-1149-4adc-8dbf-b06571cb5106"],
        names: &["Mesh Colours", "MCol"],
        kind: ComponentKind::MeshColours(MeshColoursComponent),
    },
    Registration {
        guids: &["edcf10e1-02a0-48a4-ae2d-70c50d903dc8"],
        names: &["Mesh Spray", "MSpray"],
        kind: ComponentKind::MeshSpray(MeshSprayComponent),
    },
];

/// Component to create a mesh sphere.
#[derive(Debug, Default, Clone, Copy)]
pub struct MeshSphereComponent;

impl Component for MeshSphereComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 4 {
            return Err(ComponentError::new(
                "Minimaal 4 inputs (Base, Radius, U Count, V Count) vereist.",
            ));
        }

        // Base plane (inputs[0]) is ignored for now.
        let radius = coerce::coerce_number(&inputs[1], None)?;
        let u_count = coerce_index(&inputs[2])? as usize;
        let v_count = coerce_index(&inputs[3])? as usize;

        if radius <= 0.0 {
            return Err(ComponentError::new("Radius moet groter zijn dan 0."));
        }
        if u_count < 3 || v_count < 2 {
            return Err(ComponentError::new(
                "U count moet >= 3 en V count moet >= 2 zijn.",
            ));
        }

        let mut vertices = Vec::new();
        let mut faces = Vec::new();

        // Add top pole vertex
        vertices.push([0.0, 0.0, radius]);

        // Generate vertices for the rings
        for i in 1..v_count {
            let phi = PI * i as f64 / v_count as f64;
            let z = radius * phi.cos();
            let ring_radius = radius * phi.sin();

            for j in 0..u_count {
                let theta = 2.0 * PI * j as f64 / u_count as f64;
                let x = ring_radius * theta.cos();
                let y = ring_radius * theta.sin();
                vertices.push([x, y, z]);
            }
        }

        // Add bottom pole vertex
        vertices.push([0.0, 0.0, -radius]);

        // Generate faces
        // Top cap (triangles)
        for i in 0..u_count {
            let i0 = 0;
            let i1 = i + 1;
            let i2 = (i + 1) % u_count + 1;
            faces.push(vec![i0 as u32, i2 as u32, i1 as u32]);
        }

        // Middle rings (quads)
        for i in 0..(v_count - 2) {
            for j in 0..u_count {
                let current_ring_start = i * u_count + 1;
                let next_ring_start = (i + 1) * u_count + 1;

                let i0 = current_ring_start + j;
                let i1 = current_ring_start + (j + 1) % u_count;
                let i2 = next_ring_start + (j + 1) % u_count;
                let i3 = next_ring_start + j;
                faces.push(vec![i0 as u32, i1 as u32, i2 as u32, i3 as u32]);
            }
        }

        // Bottom cap (triangles)
        let bottom_pole_index = vertices.len() as u32 - 1;
        let last_ring_start = (v_count - 2) * u_count + 1;
        for i in 0..u_count {
            let i0 = bottom_pole_index;
            let i1 = last_ring_start as u32 + i as u32;
            let i2 = last_ring_start as u32 + ((i + 1) % u_count) as u32;
            faces.push(vec![i0, i1, i2]);
        }

        // Convert mixed triangle/quad faces to triangulated Value::Mesh
        let mesh = create_mesh_from_faces(vertices, faces);

        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_M.to_owned(), mesh);

        Ok(outputs)
    }
}

/// Component to create a mesh box.
#[derive(Debug, Default, Clone, Copy)]
pub struct MeshBoxComponent;

impl Component for MeshBoxComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 4 {
            return Err(ComponentError::new(
                "Minimaal 4 inputs (Base, X Count, Y Count, Z Count) vereist.",
            ));
        }

        // De 'Base' input wordt momenteel niet ondersteund, omdat er geen
        // `Value::Box`-type bestaat. We gaan uit van eenheidskubus.
        let x_count = coerce_index(&inputs[1])? as usize;
        let y_count = coerce_index(&inputs[2])? as usize;
        let z_count = coerce_index(&inputs[3])? as usize;

        if x_count == 0 || y_count == 0 || z_count == 0 {
            return Err(ComponentError::new(
                "X, Y en Z counts moeten groter zijn dan 0.",
            ));
        }

        let mut vertices = Vec::new();
        let mut faces = Vec::new();
        let mut add_face = |v: [[f64; 3]; 4], nx: usize, ny: usize| {
            let base_index = vertices.len() as u32;
            for j in 0..=ny {
                for i in 0..=nx {
                    let u = i as f64 / nx as f64;
                    let v_ = j as f64 / ny as f64;
                    let p = [
                        (1.0 - u) * (1.0 - v_) * v[0][0]
                            + u * (1.0 - v_) * v[1][0]
                            + u * v_ * v[2][0]
                            + (1.0 - u) * v_ * v[3][0],
                        (1.0 - u) * (1.0 - v_) * v[0][1]
                            + u * (1.0 - v_) * v[1][1]
                            + u * v_ * v[2][1]
                            + (1.0 - u) * v_ * v[3][1],
                        (1.0 - u) * (1.0 - v_) * v[0][2]
                            + u * (1.0 - v_) * v[1][2]
                            + u * v_ * v[2][2]
                            + (1.0 - u) * v_ * v[3][2],
                    ];
                    vertices.push(p);
                }
            }
            for j in 0..ny {
                for i in 0..nx {
                    let i0 = base_index + (j * (nx + 1) + i) as u32;
                    let i1 = i0 + 1;
                    let i2 = i0 + (nx + 1) as u32;
                    let i3 = i2 + 1;
                    faces.push(vec![i0, i1, i3, i2]);
                }
            }
        };

        // Bottom face (-z)
        add_face(
            [
                [-0.5, -0.5, -0.5],
                [0.5, -0.5, -0.5],
                [0.5, 0.5, -0.5],
                [-0.5, 0.5, -0.5],
            ],
            x_count,
            y_count,
        );
        // Top face (+z)
        add_face(
            [
                [-0.5, 0.5, 0.5],
                [0.5, 0.5, 0.5],
                [0.5, -0.5, 0.5],
                [-0.5, -0.5, 0.5],
            ],
            x_count,
            y_count,
        );
        // Front face (+y)
        add_face(
            [
                [-0.5, -0.5, -0.5],
                [0.5, -0.5, -0.5],
                [0.5, -0.5, 0.5],
                [-0.5, -0.5, 0.5],
            ],
            x_count,
            z_count,
        );
        // Back face (-y)
        add_face(
            [
                [-0.5, 0.5, -0.5],
                [0.5, 0.5, -0.5],
                [0.5, 0.5, 0.5],
                [-0.5, 0.5, 0.5],
            ],
            x_count,
            z_count,
        );
        // Right face (+x)
        add_face(
            [
                [0.5, -0.5, -0.5],
                [0.5, 0.5, -0.5],
                [0.5, 0.5, 0.5],
                [0.5, -0.5, 0.5],
            ],
            y_count,
            z_count,
        );
        // Left face (-x)
        add_face(
            [
                [-0.5, -0.5, -0.5],
                [-0.5, 0.5, -0.5],
                [-0.5, 0.5, 0.5],
                [-0.5, -0.5, 0.5],
            ],
            y_count,
            z_count,
        );

        // Convert quad faces to triangulated Value::Mesh
        let mesh = create_mesh_from_faces(vertices, faces);

        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_M.to_owned(), mesh);

        Ok(outputs)
    }
}

/// Component to create a mesh plane.
#[derive(Debug, Default, Clone, Copy)]
pub struct MeshPlaneComponent;

impl Component for MeshPlaneComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 3 {
            return Err(ComponentError::new(
                "Minimaal 3 inputs (Boundary, Width count, Height count) vereist.",
            ));
        }

        // De 'Boundary' input wordt momenteel niet ondersteund, omdat er geen
        // `Value::Rectangle`-type bestaat. We gaan uit van eenheidsvierkant op het XY-vlak.
        let w_count = coerce_index(&inputs[1])? as usize;
        let h_count = coerce_index(&inputs[2])? as usize;

        if w_count == 0 || h_count == 0 {
            return Err(ComponentError::new(
                "Width en Height counts moeten groter zijn dan 0.",
            ));
        }

        let mut vertices = Vec::with_capacity((w_count + 1) * (h_count + 1));
        for j in 0..=h_count {
            for i in 0..=w_count {
                let x = i as f64 / w_count as f64;
                let y = j as f64 / h_count as f64;
                vertices.push([x, y, 0.0]);
            }
        }

        let mut faces = Vec::with_capacity(w_count * h_count);
        for j in 0..h_count {
            for i in 0..w_count {
                let i0 = (j * (w_count + 1) + i) as u32;
                let i1 = i0 + 1;
                let i2 = i0 + (w_count + 1) as u32;
                let i3 = i2 + 1;
                faces.push(vec![i0, i1, i3, i2]);
            }
        }

        // Convert quad faces to triangulated Value::Mesh
        let mesh = create_mesh_from_faces(vertices, faces);

        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_M.to_owned(), mesh);
        // De 'Area' output wordt voorlopig niet berekend.
        outputs.insert("A".to_owned(), Value::Number(1.0));

        Ok(outputs)
    }
}
