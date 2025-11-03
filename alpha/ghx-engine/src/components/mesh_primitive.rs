//! Grasshopper-componenten voor het maken van primitieve mesh-geometrie.
//!
//! Categorie: Mesh > Primitive

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{coerce, Component, ComponentError, ComponentResult};

/// Output pin name for the mesh.
const OUTPUT_M: &str = "M";

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
        // De `Value::Surface` enum heeft geen veld voor kleuren.

        let mut outputs = BTreeMap::new();
        outputs.insert(
            OUTPUT_M.to_owned(),
            Value::Surface { vertices, faces },
        );

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
                    let number = coerce::coerce_number(index_value)?;
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
            return Err(ComponentError::new(
                "Minimaal 3 inputs (A, B, C) vereist.",
            ));
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
    let number = coerce::coerce_number(value)?;
    if number < 0.0 || number.fract() != 0.0 {
        return Err(ComponentError::new(format!(
            "Index moet een niet-negatief geheel getal zijn, kreeg {}",
            number
        )));
    }
    Ok(number as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::node::MetaMap;
    use crate::graph::value::Value;

    #[test]
    fn test_construct_mesh_success() {
        let component = ConstructMeshComponent;
        let vertices = Value::List(vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Point([1.0, 1.0, 0.0]),
            Value::Point([0.0, 1.0, 0.0]),
        ]);
        let faces = Value::List(vec![Value::List(vec![
            Value::Number(0.0),
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ])]);
        let inputs = vec![vertices, faces];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();

        let mesh_output = outputs.get(OUTPUT_M).unwrap();
        if let Value::Surface { vertices, faces } = mesh_output {
            assert_eq!(vertices.len(), 4);
            assert_eq!(faces.len(), 1);
            assert_eq!(faces[0], vec![0, 1, 2, 3]);
        } else {
            panic!("Incorrect output type");
        }
    }

    #[test]
    fn test_construct_mesh_invalid_vertices() {
        let component = ConstructMeshComponent;
        let vertices = Value::Number(1.0); // Not a list
        let faces = Value::List(vec![Value::List(vec![Value::Number(0.0)])]);
        let inputs = vec![vertices, faces];
        let err = component.evaluate(&inputs, &MetaMap::new()).unwrap_err();
        assert!(err.message().contains("Vertices moeten een lijst zijn"));
    }

    #[test]
    fn test_construct_mesh_invalid_faces() {
        let component = ConstructMeshComponent;
        let vertices = Value::List(vec![Value::Point([0.0, 0.0, 0.0])]);
        let faces = Value::Number(1.0); // Not a list
        let inputs = vec![vertices, faces];
        let err = component.evaluate(&inputs, &MetaMap::new()).unwrap_err();
        assert!(err.message().contains("Faces moeten een lijst zijn"));
    }

    #[test]
    fn test_construct_mesh_invalid_face_indices() {
        let component = ConstructMeshComponent;
        let vertices = Value::List(vec![
            Value::Point([0.0, 0.0, 0.0]),
        ]);
        let faces = Value::List(vec![Value::List(vec![
            Value::Number(-1.0), // Invalid index
        ])]);
        let inputs = vec![vertices, faces];
        let err = component.evaluate(&inputs, &MetaMap::new()).unwrap_err();
        assert!(err.message().contains("Face index moet een niet-negatief geheel getal zijn"));
    }

    #[test]
    fn test_mesh_triangle() {
        let component = MeshTriangleComponent;
        let inputs = vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        let face = outputs.get(OUTPUT_F).unwrap();
        assert_eq!(
            face,
            &Value::List(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0)
            ])
        );
    }

    #[test]
    fn test_mesh_quad() {
        let component = MeshQuadComponent;
        let inputs = vec![
            Value::Number(4.0),
            Value::Number(5.0),
            Value::Number(6.0),
            Value::Number(7.0),
        ];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        let face = outputs.get(OUTPUT_F).unwrap();
        assert_eq!(
            face,
            &Value::List(vec![
                Value::Number(4.0),
                Value::Number(5.0),
                Value::Number(6.0),
                Value::Number(7.0)
            ])
        );
    }

    #[test]
    fn test_mesh_plane() {
        let component = MeshPlaneComponent;
        let inputs = vec![
            Value::Null, // Boundary - not used
            Value::Number(2.0),
            Value::Number(3.0),
        ];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        let mesh = outputs.get(OUTPUT_M).unwrap();
        if let Value::Surface { vertices, faces } = mesh {
            assert_eq!(vertices.len(), 12); // (2+1) * (3+1)
            assert_eq!(faces.len(), 6); // 2 * 3
        } else {
            panic!("Incorrect output type");
        }
    }

    #[test]
    fn test_mesh_box() {
        let component = MeshBoxComponent;
        let inputs = vec![
            Value::Null, // Base - not used
            Value::Number(1.0),
            Value::Number(1.0),
            Value::Number(1.0),
        ];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        let mesh = outputs.get(OUTPUT_M).unwrap();
        if let Value::Surface { vertices, faces } = mesh {
            assert_eq!(vertices.len(), 24);
            assert_eq!(faces.len(), 6);
        } else {
            panic!("Incorrect output type");
        }
    }

    #[test]
    fn test_mesh_sphere() {
        let component = MeshSphereComponent;
        let inputs = vec![
            Value::Null, // Base - not used
            Value::Number(10.0),
            Value::Number(8.0), // U
            Value::Number(6.0), // V
        ];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        let mesh = outputs.get(OUTPUT_M).unwrap();
        if let Value::Surface { vertices, faces } = mesh {
            // Vertices = (U * (V-1)) + 2 poles
            assert_eq!(vertices.len(), (8 * (6 - 1)) + 2);
            // Faces = (U * (V-2) quads) + (2 * U triangles)
            assert_eq!(faces.len(), (8 * (6-2)) + (2 * 8));
        } else {
            panic!("Incorrect output type");
        }
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
        guids: &["8adbf481-7589-4a40-b490-006531ea001d", "dd8d834f-40f1-4a84-8e4b-9fa8efe7be41"],
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
        let radius = coerce::coerce_number(&inputs[1])?;
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

        let mut outputs = BTreeMap::new();
        outputs.insert(
            OUTPUT_M.to_owned(),
            Value::Surface { vertices, faces },
        );

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
                        (1.0 - u) * (1.0 - v_) * v[0][0] + u * (1.0 - v_) * v[1][0] + u * v_ * v[2][0] + (1.0 - u) * v_ * v[3][0],
                        (1.0 - u) * (1.0 - v_) * v[0][1] + u * (1.0 - v_) * v[1][1] + u * v_ * v[2][1] + (1.0 - u) * v_ * v[3][1],
                        (1.0 - u) * (1.0 - v_) * v[0][2] + u * (1.0 - v_) * v[1][2] + u * v_ * v[2][2] + (1.0 - u) * v_ * v[3][2],
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
        add_face([[-0.5, -0.5, -0.5], [0.5, -0.5, -0.5], [0.5, 0.5, -0.5], [-0.5, 0.5, -0.5]], x_count, y_count);
        // Top face (+z)
        add_face([[-0.5, 0.5, 0.5], [0.5, 0.5, 0.5], [0.5, -0.5, 0.5], [-0.5, -0.5, 0.5]], x_count, y_count);
        // Front face (+y)
        add_face([[-0.5, -0.5, -0.5], [0.5, -0.5, -0.5], [0.5, -0.5, 0.5], [-0.5, -0.5, 0.5]], x_count, z_count);
        // Back face (-y)
        add_face([[-0.5, 0.5, -0.5], [0.5, 0.5, -0.5], [0.5, 0.5, 0.5], [-0.5, 0.5, 0.5]], x_count, z_count);
        // Right face (+x)
        add_face([[0.5, -0.5, -0.5], [0.5, 0.5, -0.5], [0.5, 0.5, 0.5], [0.5, -0.5, 0.5]], y_count, z_count);
        // Left face (-x)
        add_face([[-0.5, -0.5, -0.5], [-0.5, 0.5, -0.5], [-0.5, 0.5, 0.5], [-0.5, -0.5, 0.5]], y_count, z_count);

        let mut outputs = BTreeMap::new();
        outputs.insert(
            OUTPUT_M.to_owned(),
            Value::Surface { vertices, faces },
        );

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

        let mut outputs = BTreeMap::new();
        outputs.insert(
            OUTPUT_M.to_owned(),
            Value::Surface { vertices, faces },
        );
        // De 'Area' output wordt voorlopig niet berekend.
        outputs.insert("A".to_owned(), Value::Number(1.0));

        Ok(outputs)
    }
}
