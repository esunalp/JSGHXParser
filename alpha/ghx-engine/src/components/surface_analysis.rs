//! Implementaties van Grasshopper "Surface → Analysis" componenten.
//!
//! This module provides surface analysis components that work with both the new
//! `Value::Mesh` type and the legacy `Value::Surface` type for backward compatibility.
//!
//! # Mesh Support
//!
//! Components that only need a triangulated representation (e.g., for computing
//! normals, areas, bounds, or closest points) accept both `Value::Mesh` and
//! `Value::Surface` inputs transparently.
//!
//! Components that require true surface parameterization (e.g., curvature evaluation,
//! osculating circles) will produce clear error messages when given a `Value::Mesh`
//! input, explaining that the operation requires a parametric surface.

use std::collections::{BTreeMap, HashMap};

use crate::geom::{
    classify_point_in_mesh, closest_point_on_mesh, GeomMesh, Point3 as GeomPoint3,
    PointContainment, Tolerance as GeomTolerance,
    // Surface curvature analysis
    analyze_surface_curvature, VertexGridSurface,
};
use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

const PIN_OUTPUT_CURVES: &str = "C";
const PIN_OUTPUT_BOX_PLANE: &str = "Pl";
const PIN_OUTPUT_BOX_POINT: &str = "Pt";
const PIN_OUTPUT_BOX_INCLUDE: &str = "I";
const PIN_OUTPUT_NAKED_EDGES: &str = "En";
const PIN_OUTPUT_INTERIOR_EDGES: &str = "Ei";
const PIN_OUTPUT_NON_MANIFOLD_EDGES: &str = "Em";
const PIN_OUTPUT_POINTS: &str = "P";
const PIN_OUTPUT_WEIGHTS: &str = "W";
const PIN_OUTPUT_GREVILLE: &str = "G";
const PIN_OUTPUT_U_COUNT: &str = "U";
const PIN_OUTPUT_V_COUNT: &str = "V";
const PIN_OUTPUT_AREA: &str = "A";
const PIN_OUTPUT_VOLUME: &str = "V";
const PIN_OUTPUT_CENTROID: &str = "C";
const PIN_OUTPUT_INERTIA: &str = "I";
const PIN_OUTPUT_INERTIA_ERROR: &str = "I±";
const PIN_OUTPUT_SECONDARY: &str = "S";
const PIN_OUTPUT_SECONDARY_ERROR: &str = "S±";
const PIN_OUTPUT_GYRATION: &str = "G";
const PIN_OUTPUT_RELATION: &str = "R";
const PIN_OUTPUT_FRAME: &str = "F";
const PIN_OUTPUT_NORMAL: &str = "N";
const PIN_OUTPUT_DISTANCE: &str = "D";
const PIN_OUTPUT_U_DIRECTION: &str = "U";
const PIN_OUTPUT_V_DIRECTION: &str = "V";
const PIN_OUTPUT_UV_POINT: &str = "uvP";
const PIN_OUTPUT_CIRCLE_ONE: &str = "C1";
const PIN_OUTPUT_CIRCLE_TWO: &str = "C2";
const PIN_OUTPUT_WIREFRAME: &str = "W";
const PIN_OUTPUT_PLANAR: &str = "F";
const PIN_OUTPUT_PLANE: &str = "P";
const PIN_OUTPUT_X_SIZE: &str = "X";
const PIN_OUTPUT_Y_SIZE: &str = "Y";
const PIN_OUTPUT_Z_SIZE: &str = "Z";
const PIN_OUTPUT_INSIDE: &str = "I";
const PIN_OUTPUT_INDEX: &str = "i";
const PIN_OUTPUT_FACES: &str = "F";
const PIN_OUTPUT_EDGES: &str = "E";
const PIN_OUTPUT_VERTICES: &str = "V";
const PIN_OUTPUT_FACE_FACE: &str = "FF";
const PIN_OUTPUT_FACE_EDGE: &str = "FE";
const PIN_OUTPUT_EDGE_FACE: &str = "EF";

const EPSILON: f64 = 1e-9;

// ============================================================================
// Mesh/Surface Detection Helpers
// ============================================================================

/// Checks if a value is a mesh (not a parametric surface).
///
/// Used to provide clear error messages for operations that require
/// surface parameterization and cannot work with triangulated meshes.
fn is_mesh_value(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Mesh { .. }) => true,
        Some(Value::List(items)) if !items.is_empty() => is_mesh_value(items.first()),
        _ => false,
    }
}

/// Returns an error for operations that require surface parameterization.
///
/// Use this in components that need (u,v) parameter evaluation, curvature
/// computation, or other operations that only make sense on parametric surfaces.
fn require_parametric_surface(
    value: Option<&Value>,
    component_name: &str,
    operation: &str,
) -> Result<(), ComponentError> {
    if is_mesh_value(value) {
        Err(ComponentError::new(format!(
            "{} requires a parametric surface for {} evaluation. \
            The input is a triangulated mesh which does not have (u,v) parameterization. \
            Consider using a NURBS or other parametric surface, or use mesh-based \
            analysis components instead.",
            component_name, operation
        )))
    } else {
        Ok(())
    }
}

/// Checks if a value could be a surface (for error messages).
/// Currently unused but retained for potential input validation diagnostics.
#[allow(dead_code)]
fn is_surface_or_mesh(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Surface { .. }) | Some(Value::Mesh { .. }) => true,
        Some(Value::List(items)) if !items.is_empty() => is_surface_or_mesh(items.first()),
        _ => false,
    }
}

// ============================================================================
// VertexGridSurface Extraction Helper
// ============================================================================

/// Attempts to extract a `VertexGridSurface` from a Value for proper surface analysis.
///
/// This enables accurate surface evaluation (normals, curvature, partial derivatives)
/// for surfaces stored as vertex grids (e.g., from loft, sweep, or explicit grid inputs).
///
/// Returns `None` if:
/// - The input is not a surface or mesh
/// - The vertex count is too small (< 4 vertices)
/// - Grid dimensions cannot be inferred
fn try_extract_vertex_grid_surface(value: Option<&Value>) -> Option<VertexGridSurface> {
    let (vertices, faces) = match value {
        Some(Value::Surface { vertices, faces }) if vertices.len() >= 4 => {
            (vertices.clone(), Some(faces.clone()))
        }
        Some(Value::Mesh { vertices, indices, .. }) if vertices.len() >= 4 => {
            // Convert flat indices to face list for grid inference
            let faces: Vec<Vec<u32>> = indices
                .chunks(3)
                .map(|chunk| chunk.to_vec())
                .collect();
            (vertices.clone(), Some(faces))
        }
        Some(Value::List(items)) if !items.is_empty() => {
            // Try first item in list
            return try_extract_vertex_grid_surface(items.first());
        }
        _ => return None,
    };

    // Try to infer grid dimensions from faces
    let grid_dims = infer_grid_from_faces_and_vertices(vertices.len(), faces.as_deref());

    // Convert vertices to Point3
    let points: Vec<GeomPoint3> = vertices
        .iter()
        .map(|v| GeomPoint3::new(v[0], v[1], v[2]))
        .collect();

    if let Some((u_count, v_count)) = grid_dims {
        VertexGridSurface::new(points, u_count, v_count)
    } else {
        None
    }
}

/// Infers grid dimensions from face structure and vertex count.
///
/// For quad grids triangulated as pairs, face_count = 2 * (u-1) * (v-1),
/// so we can solve for grid dimensions that match the vertex count.
fn infer_grid_from_faces_and_vertices(
    vertex_count: usize,
    faces: Option<&[Vec<u32>]>,
) -> Option<(usize, usize)> {
    if vertex_count < 4 {
        return None;
    }

    // Try to infer from face structure first
    if let Some(faces) = faces {
        if !faces.is_empty() {
            let face_count = faces.len();
            // For quad grids triangulated as pairs: face_count = 2 * (u-1) * (v-1)
            if face_count % 2 == 0 {
                let cell_count = face_count / 2;
                
                // Find factor pair closest to square
                let sqrt = (cell_count as f64).sqrt() as usize;
                for u_cells in (1..=sqrt).rev() {
                    if cell_count % u_cells == 0 {
                        let v_cells = cell_count / u_cells;
                        let expected_vertices = (u_cells + 1) * (v_cells + 1);
                        if expected_vertices == vertex_count {
                            return Some((u_cells + 1, v_cells + 1));
                        }
                    }
                }
                // Try the other direction
                for v_cells in (1..=sqrt).rev() {
                    if cell_count % v_cells == 0 {
                        let u_cells = cell_count / v_cells;
                        let expected_vertices = (u_cells + 1) * (v_cells + 1);
                        if expected_vertices == vertex_count {
                            return Some((u_cells + 1, v_cells + 1));
                        }
                    }
                }
            }
        }
    }

    // Fall back to vertex count inference
    VertexGridSurface::infer_grid_dimensions(vertex_count)
}

// ============================================================================
// Component Definitions
// ============================================================================

/// Beschikbare componentvarianten binnen Surface → Analysis.
#[derive(Debug, Default, Clone, Copy)]
pub enum ComponentKind {
    #[default]
    SurfaceInflection,
    EvaluateBox,
    BrepEdges,
    SurfacePoints,
    AreaMoments,
    Volume,
    ShapeInBrep,
    Area,
    VolumeMoments,
    EvaluateSurface,
    PrincipalCurvature,
    SurfaceCurvature,
    SurfaceClosestPoint,
    BrepClosestPointWithNormal,
    BrepClosestPoint,
    BrepAreaMoments,
    PointInBreps,
    BrepTopology,
    DeconstructBrep,
    BoxCorners,
    BrepArea,
    BrepWireframe,
    BoxProperties,
    OsculatingCircles,
    BrepVolume,
    IsPlanar,
    DeconstructBox,
    PointInBrep,
    Dimensions,
    PointInTrim,
    BrepVolumeMoments,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Volledige lijst van componentregistraties voor Surface → Analysis.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{0148a65d-6f42-414a-9db7-9a9b2eb78437}"],
        names: &["Brep Edges", "Edges"],
        kind: ComponentKind::BrepEdges,
    },
    Registration {
        guids: &["{0efd7f0c-f63d-446d-970e-9fb0e636ea41}"],
        names: &["Surface Inflection", "SInf"],
        kind: ComponentKind::SurfaceInflection,
    },
    Registration {
        guids: &["{13b40e9c-3aed-4669-b2e8-60bd02091421}"],
        names: &["Evaluate Box", "Box"],
        kind: ComponentKind::EvaluateBox,
    },
    Registration {
        guids: &["{15128198-399d-4d6c-9586-1f65db3ce7bf}"],
        names: &["Surface Points", "SrfPt"],
        kind: ComponentKind::SurfacePoints,
    },
    Registration {
        guids: &[
            "{1eb7b856-ec7d-40b6-a76c-f216a11df37c}",
            "{c98c1666-5f29-4bb8-aafd-bb5a708e8a95}",
        ],
        names: &["Area Moments", "AMoments"],
        kind: ComponentKind::AreaMoments,
    },
    Registration {
        guids: &[
            "{224f7648-5956-4b26-80d9-8d771f3dfd5d}",
            "{7c0523e8-79c9-45a2-8777-cf0d46bc5432}",
        ],
        names: &["Volume"],
        kind: ComponentKind::Volume,
    },
    Registration {
        guids: &["{2ba64356-be21-4c12-bbd4-ced54f04c8ef}"],
        names: &["Shape In Brep", "ShapeIn"],
        kind: ComponentKind::ShapeInBrep,
    },
    Registration {
        guids: &[
            "{2e205f24-9279-47b2-b414-d06dcd0b21a7}",
            "{86b28a7e-94d9-4791-8306-e13e10d5f8d5}",
            "{ab766b01-a3f5-4257-831a-fc84d7b288b4}",
        ],
        names: &["Area"],
        kind: ComponentKind::Area,
    },
    Registration {
        guids: &[
            "{2e685fd9-7b8f-461b-b330-44857b099937}",
            "{4b5f79e1-c2b3-4b9c-b97d-470145a3ca74}",
            "{ffdfcfc5-3933-4c38-b680-8bb530e243ff}",
        ],
        names: &["Volume Moments", "VMoments"],
        kind: ComponentKind::VolumeMoments,
    },
    Registration {
        guids: &[
            "{353b206e-bde5-4f02-a913-b3b8a977d4b9}",
            "{aa1dc107-70de-473e-9636-836030160fc3}",
        ],
        names: &["Evaluate Surface", "EvalSrf"],
        kind: ComponentKind::EvaluateSurface,
    },
    Registration {
        guids: &["{404f75ac-5594-4c48-ad8a-7d0f472bbf8a}"],
        names: &["Principal Curvature", "Curvature"],
        kind: ComponentKind::PrincipalCurvature,
    },
    Registration {
        guids: &["{4139f3a3-cf93-4fc0-b5e0-18a3acd0b003}"],
        names: &["Surface Curvature", "Curvature"],
        kind: ComponentKind::SurfaceCurvature,
    },
    Registration {
        guids: &["{4a9e9a8e-0943-4438-b360-129c30f2bb0f}"],
        names: &["Surface Closest Point", "Srf CP"],
        kind: ComponentKind::SurfaceClosestPoint,
    },
    Registration {
        guids: &["{4beead95-8aa2-4613-8bb9-24758a0f5c4c}"],
        names: &["Brep Closest Point", "Brep CP"],
        kind: ComponentKind::BrepClosestPointWithNormal,
    },
    Registration {
        guids: &["{5d2fb801-2905-4a55-9d48-bbb22c73ad13}"],
        names: &["Brep Area Moments", "AMoments"],
        kind: ComponentKind::BrepAreaMoments,
    },
    Registration {
        guids: &["{859daa86-3ab7-49cb-9eda-f2811c984070}"],
        names: &["Point In Breps", "BrepsInc"],
        kind: ComponentKind::PointInBreps,
    },
    Registration {
        guids: &["{866ee39d-9ebf-4e1d-b209-324c56825605}"],
        names: &["Brep Topology", "Topology"],
        kind: ComponentKind::BrepTopology,
    },
    Registration {
        guids: &["{8d372bdc-9800-45e9-8a26-6e33c5253e21}"],
        names: &["Deconstruct Brep", "DeBrep"],
        kind: ComponentKind::DeconstructBrep,
    },
    Registration {
        guids: &["{a10e8cdf-7c7a-4aac-aa70-ddb7010ab231}"],
        names: &["Box Corners"],
        kind: ComponentKind::BoxCorners,
    },
    Registration {
        guids: &["{ac750e41-2450-4f98-9658-98fef97b01b2}"],
        names: &["Brep Wireframe", "Wires"],
        kind: ComponentKind::BrepWireframe,
    },
    Registration {
        guids: &["{af9cdb9d-9617-4827-bb3c-9efd88c76a70}"],
        names: &["Box Properties", "BoxProp"],
        kind: ComponentKind::BoxProperties,
    },
    Registration {
        guids: &["{b799b7c0-76df-4bdb-b3cc-401b1d021aa5}"],
        names: &["Osculating Circles", "Osc"],
        kind: ComponentKind::OsculatingCircles,
    },
    Registration {
        guids: &["{c72d0184-bb99-4af4-a629-4662e1c3d428}"],
        names: &["Brep Volume", "Volume"],
        kind: ComponentKind::BrepVolume,
    },
    Registration {
        guids: &["{cdd5d441-3bad-4f19-a370-6cf180b6f0fa}"],
        names: &["Brep Closest Point", "Brep CP"],
        kind: ComponentKind::BrepClosestPoint,
    },
    Registration {
        guids: &["{d4bc9653-c770-4bee-a31d-d120cbb75b39}"],
        names: &["Is Planar", "Planar"],
        kind: ComponentKind::IsPlanar,
    },
    Registration {
        guids: &["{db7d83b1-2898-4ef9-9be5-4e94b4e2048d}"],
        names: &["Deconstruct Box", "DeBox"],
        kind: ComponentKind::DeconstructBox,
    },
    Registration {
        guids: &["{e03561f8-0e66-41d3-afde-62049f152443}"],
        names: &["Point In Brep", "BrepInc"],
        kind: ComponentKind::PointInBrep,
    },
    Registration {
        guids: &["{f241e42e-8983-4ed3-b869-621c07630b00}"],
        names: &["Dimensions", "Dim"],
        kind: ComponentKind::Dimensions,
    },
    Registration {
        guids: &["{f881810b-96de-4668-a95a-f9a6d683e65c}"],
        names: &["Point In Trim", "TrimInc"],
        kind: ComponentKind::PointInTrim,
    },
    Registration {
        guids: &["{ffdfcfc5-3933-4c38-b680-8bb530e243ff}"],
        names: &["Brep Volume Moments", "VMoments"],
        kind: ComponentKind::BrepVolumeMoments,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::SurfaceInflection => evaluate_surface_inflection(),
            Self::EvaluateBox => evaluate_box(inputs),
            Self::BrepEdges => evaluate_brep_edges(inputs),
            Self::SurfacePoints => evaluate_surface_points(inputs),
            Self::AreaMoments => evaluate_area_moments(inputs, "Area Moments"),
            Self::Volume => evaluate_volume(inputs, "Volume"),
            Self::ShapeInBrep => evaluate_shape_in_brep(inputs),
            Self::Area => evaluate_area(inputs, "Area"),
            Self::VolumeMoments => evaluate_volume_moments(inputs, "Volume Moments"),
            Self::EvaluateSurface => evaluate_surface_sample_component(inputs),
            Self::PrincipalCurvature => evaluate_principal_curvature(inputs),
            Self::SurfaceCurvature => evaluate_surface_curvature(inputs),
            Self::SurfaceClosestPoint => evaluate_surface_closest_point(inputs),
            Self::BrepClosestPointWithNormal => evaluate_brep_closest_point(inputs, true),
            Self::BrepClosestPoint => evaluate_brep_closest_point(inputs, false),
            Self::BrepAreaMoments => evaluate_area_moments(inputs, "Brep Area Moments"),
            Self::PointInBreps => evaluate_point_in_breps(inputs),
            Self::BrepTopology => evaluate_brep_topology(inputs),
            Self::DeconstructBrep => evaluate_deconstruct_brep(inputs),
            Self::BoxCorners => evaluate_box_corners(inputs),
            Self::BrepArea => evaluate_area(inputs, "Brep Area"),
            Self::BrepWireframe => evaluate_brep_wireframe(inputs),
            Self::BoxProperties => evaluate_box_properties(inputs),
            Self::OsculatingCircles => evaluate_osculating_circles(inputs),
            Self::BrepVolume => evaluate_volume(inputs, "Brep Volume"),
            Self::IsPlanar => evaluate_is_planar(inputs),
            Self::DeconstructBox => evaluate_deconstruct_box(inputs),
            Self::PointInBrep => evaluate_point_in_brep(inputs),
            Self::Dimensions => evaluate_dimensions(inputs),
            Self::PointInTrim => evaluate_point_in_trim(inputs),
            Self::BrepVolumeMoments => evaluate_volume_moments(inputs, "Brep Volume Moments"),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::SurfaceInflection => "Surface Inflection",
            Self::EvaluateBox => "Evaluate Box",
            Self::BrepEdges => "Brep Edges",
            Self::SurfacePoints => "Surface Points",
            Self::AreaMoments => "Area Moments",
            Self::Volume => "Volume",
            Self::ShapeInBrep => "Shape In Brep",
            Self::Area => "Area",
            Self::VolumeMoments => "Volume Moments",
            Self::EvaluateSurface => "Evaluate Surface",
            Self::PrincipalCurvature => "Principal Curvature",
            Self::SurfaceCurvature => "Surface Curvature",
            Self::SurfaceClosestPoint => "Surface Closest Point",
            Self::BrepClosestPointWithNormal | Self::BrepClosestPoint => "Brep Closest Point",
            Self::BrepAreaMoments => "Brep Area Moments",
            Self::PointInBreps => "Point In Breps",
            Self::BrepTopology => "Brep Topology",
            Self::DeconstructBrep => "Deconstruct Brep",
            Self::BoxCorners => "Box Corners",
            Self::BrepArea => "Brep Area",
            Self::BrepWireframe => "Brep Wireframe",
            Self::BoxProperties => "Box Properties",
            Self::OsculatingCircles => "Osculating Circles",
            Self::BrepVolume => "Brep Volume",
            Self::IsPlanar => "Is Planar",
            Self::DeconstructBox => "Deconstruct Box",
            Self::PointInBrep => "Point In Brep",
            Self::Dimensions => "Dimensions",
            Self::PointInTrim => "Point In Trim",
            Self::BrepVolumeMoments => "Brep Volume Moments",
        }
    }
}

fn evaluate_surface_inflection() -> ComponentResult {
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CURVES.to_owned(), Value::List(Vec::new()));
    Ok(outputs)
}

fn evaluate_box(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 4 {
        return Err(ComponentError::new(
            "Evaluate Box verwacht een box en drie parameters",
        ));
    }

    let points = match inputs[0] {
        Value::List(ref values) if values.len() >= 8 => values
            .iter()
            .filter_map(|value| match value {
                Value::Point(point) => Some(*point),
                _ => None,
            })
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    };

    if points.len() < 8 {
        return Err(ComponentError::new("Evaluate Box vereist acht hoekpunten"));
    }

    let u = coerce_number(inputs.get(1), "Evaluate Box U")?.clamp(0.0, 1.0);
    let v = coerce_number(inputs.get(2), "Evaluate Box V")?.clamp(0.0, 1.0);
    let w = coerce_number(inputs.get(3), "Evaluate Box W")?.clamp(0.0, 1.0);

    let (min, max) = bounding_box(&points);
    let location = [
        min[0] + (max[0] - min[0]) * u,
        min[1] + (max[1] - min[1]) * v,
        min[2] + (max[2] - min[2]) * w,
    ];

    let plane = Value::List(vec![
        Value::Point(location),
        Value::Point([location[0] + 1.0, location[1], location[2]]),
        Value::Point([location[0], location[1] + 1.0, location[2]]),
    ]);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BOX_PLANE.to_owned(), plane);
    outputs.insert(PIN_OUTPUT_BOX_POINT.to_owned(), Value::Point(location));
    outputs.insert(PIN_OUTPUT_BOX_INCLUDE.to_owned(), Value::Boolean(true));
    Ok(outputs)
}

fn evaluate_brep_edges(inputs: &[Value]) -> ComponentResult {
    // Try to use proper mesh topology extraction first
    if let Some(topology) = try_mesh_topology(inputs.get(0)) {
        let naked_edges = topology
            .naked_edges()
            .into_iter()
            .map(|(p1, p2)| Value::CurveLine { p1, p2 })
            .collect();
        let interior_edges = topology
            .interior_edges()
            .into_iter()
            .map(|(p1, p2)| Value::CurveLine { p1, p2 })
            .collect();
        let non_manifold_edges = topology
            .non_manifold_edges()
            .into_iter()
            .map(|(p1, p2)| Value::CurveLine { p1, p2 })
            .collect();

        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_NAKED_EDGES.to_owned(), Value::List(naked_edges));
        outputs.insert(PIN_OUTPUT_INTERIOR_EDGES.to_owned(), Value::List(interior_edges));
        outputs.insert(PIN_OUTPUT_NON_MANIFOLD_EDGES.to_owned(), Value::List(non_manifold_edges));
        return Ok(outputs);
    }

    // Fallback to AABB-based wireframe for non-mesh inputs (legacy support)
    let metrics = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new("Brep Edges vereist een brep"))?;
    let wireframe = create_wireframe(&metrics)
        .into_iter()
        .map(|(p1, p2)| Value::CurveLine { p1, p2 })
        .collect();
    let mut outputs = BTreeMap::new();
    // For AABB fallback, all edges are treated as "naked" (boundary edges)
    outputs.insert(PIN_OUTPUT_NAKED_EDGES.to_owned(), Value::List(wireframe));
    outputs.insert(
        PIN_OUTPUT_INTERIOR_EDGES.to_owned(),
        Value::List(Vec::new()),
    );
    outputs.insert(
        PIN_OUTPUT_NON_MANIFOLD_EDGES.to_owned(),
        Value::List(Vec::new()),
    );
    Ok(outputs)
}

fn evaluate_surface_points(inputs: &[Value]) -> ComponentResult {
    let surface = inputs
        .get(0)
        .ok_or_else(|| ComponentError::new("Surface Points vereist een surface invoer"))?;

    // Reject mesh inputs: Greville points and U/V counts require parametric surface data.
    // Meshes are triangulated geometry without (u,v) parameterization, control point weights,
    // or knot vectors that define Greville abscissae.
    require_parametric_surface(
        Some(surface),
        "Surface Points",
        "control point extraction (Greville points, weights, U/V counts)",
    )?;

    if let Some(grid) = collect_point_grid(Some(surface)) {
        let v_count = grid.len();
        let u_count = grid.iter().map(|row| row.len()).max().unwrap_or(0);
        let mut point_values = Vec::new();
        let mut weights = Vec::new();
        let mut greville = Vec::new();
        for (v_index, row) in grid.iter().enumerate() {
            for (u_index, point) in row.iter().enumerate() {
                point_values.push(Value::Point(*point));
                weights.push(Value::Number(1.0));
                let u = if u_count > 1 {
                    u_index as f64 / (u_count - 1) as f64
                } else {
                    0.0
                };
                let v = if v_count > 1 {
                    v_index as f64 / (v_count - 1) as f64
                } else {
                    0.0
                };
                greville.push(Value::Point([u, v, 0.0]));
            }
        }
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::List(point_values));
        outputs.insert(PIN_OUTPUT_WEIGHTS.to_owned(), Value::List(weights));
        outputs.insert(PIN_OUTPUT_GREVILLE.to_owned(), Value::List(greville));
        outputs.insert(PIN_OUTPUT_U_COUNT.to_owned(), Value::Number(u_count as f64));
        outputs.insert(PIN_OUTPUT_V_COUNT.to_owned(), Value::Number(v_count as f64));
        Ok(outputs)
    } else {
        let metrics = ShapeMetrics::from_inputs(Some(surface))
            .ok_or_else(|| ComponentError::new("Surface Points kon geen punten vinden"))?;
        let points = metrics
            .points
            .iter()
            .map(|point| Value::Point(*point))
            .collect();
        let weight_list = (0..metrics.points.len())
            .map(|_| Value::Number(1.0))
            .collect();
        let greville = (0..metrics.points.len())
            .map(|index| {
                let u = if metrics.points.len() > 1 {
                    index as f64 / (metrics.points.len() - 1) as f64
                } else {
                    0.0
                };
                Value::Point([u, 0.0, 0.0])
            })
            .collect();
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::List(points));
        outputs.insert(PIN_OUTPUT_WEIGHTS.to_owned(), Value::List(weight_list));
        outputs.insert(PIN_OUTPUT_GREVILLE.to_owned(), Value::List(greville));
        outputs.insert(
            PIN_OUTPUT_U_COUNT.to_owned(),
            Value::Number(metrics.points.len() as f64),
        );
        outputs.insert(PIN_OUTPUT_V_COUNT.to_owned(), Value::Number(1.0));
        Ok(outputs)
    }
}

fn evaluate_area_moments(inputs: &[Value], context: &str) -> ComponentResult {
    // First, try to use proper mesh analysis for Value::Mesh or Value::Surface
    if let Some(mut mesh_metrics) = MeshMetrics::from_value(inputs.get(0)) {
        let area = mesh_metrics.area();
        let centroid = Value::Point(mesh_metrics.centroid());
        let inertia = mesh_metrics.inertia();
        let secondary = mesh_metrics.secondary_moments();
        let gyration = mesh_metrics.gyration();

        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_AREA.to_owned(), Value::Number(area));
        outputs.insert(PIN_OUTPUT_CENTROID.to_owned(), centroid);
        outputs.insert(PIN_OUTPUT_INERTIA.to_owned(), to_number_list(&inertia));
        outputs.insert(
            PIN_OUTPUT_INERTIA_ERROR.to_owned(),
            to_number_list(&[0.0; 3]),
        );
        outputs.insert(PIN_OUTPUT_SECONDARY.to_owned(), to_number_list(&secondary));
        outputs.insert(
            PIN_OUTPUT_SECONDARY_ERROR.to_owned(),
            to_number_list(&[0.0; 3]),
        );
        outputs.insert(PIN_OUTPUT_GYRATION.to_owned(), to_number_list(&gyration));
        return Ok(outputs);
    }

    // Fallback to AABB-based ShapeMetrics for point clouds and other geometry
    let metrics = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new(format!("{} vereist geometrische invoer", context)))?;
    let area = metrics.area();
    let centroid = Value::Point(metrics.center());
    let inertia = simple_inertia(metrics.size(), area);
    let secondary = simple_secondary(metrics.size(), area);
    let gyration = simple_gyration(inertia, area);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_AREA.to_owned(), Value::Number(area));
    outputs.insert(PIN_OUTPUT_CENTROID.to_owned(), centroid);
    outputs.insert(PIN_OUTPUT_INERTIA.to_owned(), to_number_list(&inertia));
    outputs.insert(
        PIN_OUTPUT_INERTIA_ERROR.to_owned(),
        to_number_list(&[0.0; 3]),
    );
    outputs.insert(PIN_OUTPUT_SECONDARY.to_owned(), to_number_list(&secondary));
    outputs.insert(
        PIN_OUTPUT_SECONDARY_ERROR.to_owned(),
        to_number_list(&[0.0; 3]),
    );
    outputs.insert(PIN_OUTPUT_GYRATION.to_owned(), to_number_list(&gyration));
    Ok(outputs)
}

fn evaluate_volume(inputs: &[Value], context: &str) -> ComponentResult {
    // First, try to use proper mesh analysis for Value::Mesh or Value::Surface
    if let Some(mut mesh_metrics) = MeshMetrics::from_value(inputs.get(0)) {
        let volume = mesh_metrics.volume();
        let centroid = mesh_metrics.centroid();

        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_VOLUME.to_owned(), Value::Number(volume));
        outputs.insert(PIN_OUTPUT_CENTROID.to_owned(), Value::Point(centroid));
        return Ok(outputs);
    }

    // Fallback to AABB-based ShapeMetrics for point clouds and other geometry
    let metrics = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new(format!("{} vereist geometrische invoer", context)))?;
    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_VOLUME.to_owned(),
        Value::Number(metrics.volume()),
    );
    outputs.insert(
        PIN_OUTPUT_CENTROID.to_owned(),
        Value::Point(metrics.center()),
    );
    Ok(outputs)
}

fn evaluate_shape_in_brep(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Shape In Brep vereist een brep en een vorm",
        ));
    }
    let brep = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new("Shape In Brep vereist een brep"))?;
    let shape = ShapeMetrics::from_inputs(inputs.get(1))
        .ok_or_else(|| ComponentError::new("Shape In Brep vereist een vorm"))?;

    let shape_corners = create_box_corners_points(&shape);
    let inside = shape_corners
        .iter()
        .all(|point| point_in_metrics(&brep, *point, false));
    let relation = if inside {
        0
    } else if boxes_overlap(&brep, &shape) {
        1
    } else {
        2
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_RELATION.to_owned(),
        Value::Number(relation as f64),
    );
    Ok(outputs)
}

fn evaluate_area(inputs: &[Value], context: &str) -> ComponentResult {
    // First, try to use proper mesh analysis for Value::Mesh or Value::Surface
    if let Some(mut mesh_metrics) = MeshMetrics::from_value(inputs.get(0)) {
        let area = mesh_metrics.area();
        let centroid = mesh_metrics.centroid();

        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_AREA.to_owned(), Value::Number(area));
        outputs.insert(PIN_OUTPUT_CENTROID.to_owned(), Value::Point(centroid));
        return Ok(outputs);
    }

    // Fallback to AABB-based ShapeMetrics for point clouds and other geometry
    let metrics = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new(format!("{} vereist geometrische invoer", context)))?;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_AREA.to_owned(), Value::Number(metrics.area()));
    outputs.insert(
        PIN_OUTPUT_CENTROID.to_owned(),
        Value::Point(metrics.center()),
    );
    Ok(outputs)
}

fn evaluate_volume_moments(inputs: &[Value], context: &str) -> ComponentResult {
    // First, try to use proper mesh analysis for Value::Mesh or Value::Surface
    if let Some(mut mesh_metrics) = MeshMetrics::from_value(inputs.get(0)) {
        let volume = mesh_metrics.volume();
        let centroid = mesh_metrics.centroid();
        let inertia = mesh_metrics.inertia();
        let secondary = mesh_metrics.secondary_moments();
        let gyration = mesh_metrics.gyration();

        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_VOLUME.to_owned(), Value::Number(volume));
        outputs.insert(PIN_OUTPUT_CENTROID.to_owned(), Value::Point(centroid));
        outputs.insert(PIN_OUTPUT_INERTIA.to_owned(), to_number_list(&inertia));
        outputs.insert(
            PIN_OUTPUT_INERTIA_ERROR.to_owned(),
            to_number_list(&[0.0; 3]),
        );
        outputs.insert(PIN_OUTPUT_SECONDARY.to_owned(), to_number_list(&secondary));
        outputs.insert(
            PIN_OUTPUT_SECONDARY_ERROR.to_owned(),
            to_number_list(&[0.0; 3]),
        );
        outputs.insert(PIN_OUTPUT_GYRATION.to_owned(), to_number_list(&gyration));
        return Ok(outputs);
    }

    // Fallback to AABB-based ShapeMetrics for point clouds and other geometry
    let metrics = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new(format!("{} vereist geometrische invoer", context)))?;
    let volume = metrics.volume();
    let inertia = simple_inertia(metrics.size(), volume);
    let secondary = simple_secondary(metrics.size(), volume);
    let gyration = simple_gyration(inertia, volume);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_VOLUME.to_owned(), Value::Number(volume));
    outputs.insert(
        PIN_OUTPUT_CENTROID.to_owned(),
        Value::Point(metrics.center()),
    );
    outputs.insert(PIN_OUTPUT_INERTIA.to_owned(), to_number_list(&inertia));
    outputs.insert(
        PIN_OUTPUT_INERTIA_ERROR.to_owned(),
        to_number_list(&[0.0; 3]),
    );
    outputs.insert(PIN_OUTPUT_SECONDARY.to_owned(), to_number_list(&secondary));
    outputs.insert(
        PIN_OUTPUT_SECONDARY_ERROR.to_owned(),
        to_number_list(&[0.0; 3]),
    );
    outputs.insert(PIN_OUTPUT_GYRATION.to_owned(), to_number_list(&gyration));
    Ok(outputs)
}

fn evaluate_surface_sample_component(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Evaluate Surface requires a surface input",
        ));
    }
    
    let uv = coerce_uv(inputs.get(1)).unwrap_or((0.5, 0.5));
    
    // Try to extract a vertex grid surface for proper evaluation
    if let Some(grid_surface) = try_extract_vertex_grid_surface(inputs.get(0)) {
        // Use proper surface analysis
        let analysis = analyze_surface_curvature(&grid_surface, uv.0, uv.1);
        
        let point = [analysis.point.x, analysis.point.y, analysis.point.z];
        let normal = [analysis.normal.x, analysis.normal.y, analysis.normal.z];
        let u_dir = [analysis.du.x, analysis.du.y, analysis.du.z];
        let v_dir = [analysis.dv.x, analysis.dv.y, analysis.dv.z];
        
        // Normalize the directions for output
        let u_dir_normalized = normalize_vec3(u_dir);
        let v_dir_normalized = normalize_vec3(v_dir);
        
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::Point(point));
        outputs.insert(PIN_OUTPUT_NORMAL.to_owned(), Value::Vector(normal));
        outputs.insert(PIN_OUTPUT_U_DIRECTION.to_owned(), Value::Vector(u_dir_normalized));
        outputs.insert(PIN_OUTPUT_V_DIRECTION.to_owned(), Value::Vector(v_dir_normalized));
        outputs.insert(PIN_OUTPUT_FRAME.to_owned(), frame_from_axes(point, u_dir_normalized, v_dir_normalized, normal));
        return Ok(outputs);
    }
    
    // Check if input is a mesh without grid structure (cannot evaluate properly)
    require_parametric_surface(
        inputs.get(0),
        "Evaluate Surface",
        "(u,v) point",
    )?;
    
    // Fallback to AABB-based sampling (with computed normal from bounds)
    let surface = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new("Evaluate Surface could not read the surface"))?;
    let point = surface.sample_point(uv);
    
    // Try to compute a reasonable normal from the point cloud
    let (normal, u_dir, v_dir) = estimate_surface_frame_from_metrics(&surface, uv);
    
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::Point(point));
    outputs.insert(PIN_OUTPUT_NORMAL.to_owned(), Value::Vector(normal));
    outputs.insert(PIN_OUTPUT_U_DIRECTION.to_owned(), Value::Vector(u_dir));
    outputs.insert(PIN_OUTPUT_V_DIRECTION.to_owned(), Value::Vector(v_dir));
    outputs.insert(PIN_OUTPUT_FRAME.to_owned(), frame_from_axes(point, u_dir, v_dir, normal));
    Ok(outputs)
}

fn evaluate_principal_curvature(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Principal Curvature requires a surface input",
        ));
    }
    
    let uv = coerce_uv(inputs.get(1)).unwrap_or((0.5, 0.5));
    
    // Try to extract a vertex grid surface for proper curvature analysis
    if let Some(grid_surface) = try_extract_vertex_grid_surface(inputs.get(0)) {
        let analysis = analyze_surface_curvature(&grid_surface, uv.0, uv.1);
        
        if !analysis.valid {
            // Degenerate point - return zeros with warning
            let point = [analysis.point.x, analysis.point.y, analysis.point.z];
            let mut outputs = BTreeMap::new();
            outputs.insert(PIN_OUTPUT_FRAME.to_owned(), plane_from_point(point));
            outputs.insert("Maximum".to_owned(), Value::Number(0.0));
            outputs.insert("Minimum".to_owned(), Value::Number(0.0));
            outputs.insert("K¹".to_owned(), Value::Vector([1.0, 0.0, 0.0]));
            outputs.insert("K²".to_owned(), Value::Vector([0.0, 1.0, 0.0]));
            return Ok(outputs);
        }
        
        let point = [analysis.point.x, analysis.point.y, analysis.point.z];
        let normal = [analysis.normal.x, analysis.normal.y, analysis.normal.z];
        let u_dir = normalize_vec3([analysis.du.x, analysis.du.y, analysis.du.z]);
        let v_dir = normalize_vec3([analysis.dv.x, analysis.dv.y, analysis.dv.z]);
        
        let k1_dir = [analysis.k1_direction.x, analysis.k1_direction.y, analysis.k1_direction.z];
        let k2_dir = [analysis.k2_direction.x, analysis.k2_direction.y, analysis.k2_direction.z];
        
        let mut outputs = BTreeMap::new();
        outputs.insert(
            PIN_OUTPUT_FRAME.to_owned(), 
            frame_from_axes(point, u_dir, v_dir, normal)
        );
        outputs.insert(
            "Maximum".to_owned(),
            Value::Number(analysis.k1.clamp(-1e6, 1e6)),
        );
        outputs.insert(
            "Minimum".to_owned(),
            Value::Number(analysis.k2.clamp(-1e6, 1e6)),
        );
        outputs.insert("K¹".to_owned(), Value::Vector(k1_dir));
        outputs.insert("K²".to_owned(), Value::Vector(k2_dir));
        return Ok(outputs);
    }
    
    // Check if input is a mesh without grid structure
    require_parametric_surface(
        inputs.get(0),
        "Principal Curvature",
        "curvature",
    )?;
    
    // Fallback to AABB-based approximation with improved estimation
    let surface = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new("Principal Curvature could not read the surface"))?;
    let point = surface.sample_point(uv);
    
    // Estimate curvature from local geometry if we have enough points
    let (k1, k2, k1_dir, k2_dir) = estimate_curvature_from_metrics(&surface, uv);
    let (normal, u_dir, v_dir) = estimate_surface_frame_from_metrics(&surface, uv);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_FRAME.to_owned(), frame_from_axes(point, u_dir, v_dir, normal));
    outputs.insert(
        "Maximum".to_owned(),
        Value::Number(k1.clamp(-1e6, 1e6)),
    );
    outputs.insert(
        "Minimum".to_owned(),
        Value::Number(k2.clamp(-1e6, 1e6)),
    );
    outputs.insert("K¹".to_owned(), Value::Vector(k1_dir));
    outputs.insert("K²".to_owned(), Value::Vector(k2_dir));
    Ok(outputs)
}

fn evaluate_surface_curvature(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Surface Curvature requires a surface input",
        ));
    }
    
    let uv = coerce_uv(inputs.get(1)).unwrap_or((0.5, 0.5));
    
    // Try to extract a vertex grid surface for proper curvature analysis
    if let Some(grid_surface) = try_extract_vertex_grid_surface(inputs.get(0)) {
        let analysis = analyze_surface_curvature(&grid_surface, uv.0, uv.1);
        
        let point = [analysis.point.x, analysis.point.y, analysis.point.z];
        let normal = [analysis.normal.x, analysis.normal.y, analysis.normal.z];
        let u_dir = normalize_vec3([analysis.du.x, analysis.du.y, analysis.du.z]);
        let v_dir = normalize_vec3([analysis.dv.x, analysis.dv.y, analysis.dv.z]);
        
        let mut outputs = BTreeMap::new();
        outputs.insert(
            PIN_OUTPUT_FRAME.to_owned(), 
            frame_from_axes(point, u_dir, v_dir, normal)
        );
        outputs.insert("Gaussian".to_owned(), Value::Number(analysis.gaussian.clamp(-1e6, 1e6)));
        outputs.insert("Mean".to_owned(), Value::Number(analysis.mean.clamp(-1e6, 1e6)));
        return Ok(outputs);
    }
    
    // Check if input is a mesh without grid structure
    require_parametric_surface(
        inputs.get(0),
        "Surface Curvature",
        "curvature",
    )?;
    
    // Fallback to estimated curvature from point geometry
    let surface = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new("Surface Curvature could not read the surface"))?;
    let point = surface.sample_point(uv);
    
    // Estimate curvature from local geometry
    let (k1, k2, _, _) = estimate_curvature_from_metrics(&surface, uv);
    let gaussian = k1 * k2;
    let mean = (k1 + k2) * 0.5;
    let (normal, u_dir, v_dir) = estimate_surface_frame_from_metrics(&surface, uv);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_FRAME.to_owned(), frame_from_axes(point, u_dir, v_dir, normal));
    outputs.insert("Gaussian".to_owned(), Value::Number(gaussian.clamp(-1e6, 1e6)));
    outputs.insert("Mean".to_owned(), Value::Number(mean.clamp(-1e6, 1e6)));
    Ok(outputs)
}

fn evaluate_surface_closest_point(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Surface Closest Point vereist een surface en een uv-waarde",
        ));
    }
    let target = coerce_point(inputs.get(0), "Surface Closest Point punt")?;
    
    // Try to use proper mesh-based closest point computation first
    if let Some(mesh_result) = try_mesh_closest_point(inputs.get(1), target) {
        // For meshes, we approximate UV from barycentric coordinates
        // This is not a true parametric UV, but gives a reasonable approximation
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::Point(mesh_result.point));
        outputs.insert(
            PIN_OUTPUT_UV_POINT.to_owned(),
            Value::Point([mesh_result.uv.0, mesh_result.uv.1, 0.0]),
        );
        outputs.insert(PIN_OUTPUT_DISTANCE.to_owned(), Value::Number(mesh_result.distance));
        return Ok(outputs);
    }
    
    // Fallback to AABB-based approach for non-mesh inputs (legacy support)
    let surface = ShapeMetrics::from_inputs(inputs.get(1))
        .ok_or_else(|| ComponentError::new("Surface Closest Point vereist een surface"))?;
    let closest = clamp_to_metrics(&surface, target);
    let uv = uv_from_point(&surface, closest);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::Point(closest));
    outputs.insert(
        PIN_OUTPUT_UV_POINT.to_owned(),
        Value::Point([uv.0, uv.1, 0.0]),
    );
    outputs.insert(
        PIN_OUTPUT_DISTANCE.to_owned(),
        Value::Number(distance(&target, &closest)),
    );
    Ok(outputs)
}

fn evaluate_brep_closest_point(inputs: &[Value], include_normal: bool) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Brep Closest Point vereist een punt en een brep",
        ));
    }
    let target = coerce_point(inputs.get(0), "Brep Closest Point punt")?;
    
    // Try to use proper mesh-based closest point computation first
    if let Some(mesh_result) = try_mesh_closest_point(inputs.get(1), target) {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::Point(mesh_result.point));
        if include_normal {
            outputs.insert(PIN_OUTPUT_NORMAL.to_owned(), Value::Vector(mesh_result.normal));
        }
        outputs.insert(PIN_OUTPUT_DISTANCE.to_owned(), Value::Number(mesh_result.distance));
        return Ok(outputs);
    }
    
    // Fallback to AABB-based approach for non-mesh inputs (legacy support)
    let brep = ShapeMetrics::from_inputs(inputs.get(1))
        .ok_or_else(|| ComponentError::new("Brep Closest Point vereist een brep"))?;
    let closest = clamp_to_metrics(&brep, target);
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::Point(closest));
    if include_normal {
        outputs.insert(PIN_OUTPUT_NORMAL.to_owned(), Value::Vector([0.0, 0.0, 1.0]));
    }
    outputs.insert(
        PIN_OUTPUT_DISTANCE.to_owned(),
        Value::Number(distance(&target, &closest)),
    );
    Ok(outputs)
}

fn evaluate_point_in_breps(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Point In Breps vereist een lijst met breps en een punt",
        ));
    }
    let list = match &inputs[0] {
        Value::List(values) => values,
        other => {
            return Err(ComponentError::new(format!(
                "Point In Breps verwacht een lijst, kreeg {}",
                other.kind()
            )));
        }
    };
    let target = coerce_point(inputs.get(1), "Point In Breps punt")?;
    let strict = coerce_boolean(inputs.get(2), false)?;

    let mut inside_index = -1;
    for (index, entry) in list.iter().enumerate() {
        // Try proper mesh-based containment test first
        if let Some(is_inside) = try_mesh_point_containment(Some(entry), target, strict) {
            if is_inside {
                inside_index = index as i32;
                break;
            }
        } else if let Some(metrics) = ShapeMetrics::from_inputs(Some(entry)) {
            // Fallback to AABB-based containment for non-mesh inputs
            if point_in_metrics(&metrics, target, strict) {
                inside_index = index as i32;
                break;
            }
        }
    }
    let inside = inside_index >= 0;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_INSIDE.to_owned(), Value::Boolean(inside));
    outputs.insert(
        PIN_OUTPUT_INDEX.to_owned(),
        Value::Number(inside_index as f64),
    );
    Ok(outputs)
}

fn evaluate_brep_topology(inputs: &[Value]) -> ComponentResult {
    // Try to use proper mesh topology extraction first
    if let Some(topology) = try_mesh_topology(inputs.get(0)) {
        // Face-to-face adjacency: for each face, list of adjacent face indices
        let face_face = topology
            .get_face_face_adjacency()
            .into_iter()
            .map(|adjacent| {
                Value::List(adjacent.into_iter().map(|idx| Value::Number(idx as f64)).collect())
            })
            .collect();

        // Face-to-edge mapping: for each face, list of edge indices
        let face_edge = topology
            .get_face_edge_mapping()
            .into_iter()
            .map(|edges| {
                Value::List(edges.into_iter().map(|idx| Value::Number(idx as f64)).collect())
            })
            .collect();

        // Edge-to-face mapping: for each edge, list of face indices
        let edge_face = topology
            .get_edge_face_mapping()
            .into_iter()
            .map(|faces| {
                Value::List(faces.into_iter().map(|idx| Value::Number(idx as f64)).collect())
            })
            .collect();

        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_FACE_FACE.to_owned(), Value::List(face_face));
        outputs.insert(PIN_OUTPUT_FACE_EDGE.to_owned(), Value::List(face_edge));
        outputs.insert(PIN_OUTPUT_EDGE_FACE.to_owned(), Value::List(edge_face));
        return Ok(outputs);
    }

    // Fallback to static box topology for non-mesh inputs (legacy support)
    // This assumes a 6-faced box with 12 edges
    let face_face = (0..6)
        .map(|index| Value::List(vec![Value::Number(((index + 1) % 6) as f64)]))
        .collect();
    let face_edge = (0..6)
        .map(|index| Value::List(vec![Value::Number(index as f64)]))
        .collect();
    let edge_face = (0..12)
        .map(|index| Value::List(vec![Value::Number((index % 6) as f64)]))
        .collect();

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_FACE_FACE.to_owned(), Value::List(face_face));
    outputs.insert(PIN_OUTPUT_FACE_EDGE.to_owned(), Value::List(face_edge));
    outputs.insert(PIN_OUTPUT_EDGE_FACE.to_owned(), Value::List(edge_face));
    Ok(outputs)
}

fn evaluate_deconstruct_brep(inputs: &[Value]) -> ComponentResult {
    // Try to use proper mesh topology extraction first
    if let Some(topology) = try_mesh_topology(inputs.get(0)) {
        // Get actual mesh faces as point lists
        let faces = topology
            .face_points()
            .into_iter()
            .map(|face| Value::List(face.into_iter().map(Value::Point).collect()))
            .collect();

        // Get actual unique mesh edges
        let edges = topology
            .all_edges()
            .into_iter()
            .map(|(p1, p2)| Value::CurveLine { p1, p2 })
            .collect();

        // Get all vertices actually used by the mesh
        let vertices = topology
            .used_vertices()
            .into_iter()
            .map(Value::Point)
            .collect();

        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_FACES.to_owned(), Value::List(faces));
        outputs.insert(PIN_OUTPUT_EDGES.to_owned(), Value::List(edges));
        outputs.insert(PIN_OUTPUT_VERTICES.to_owned(), Value::List(vertices));
        return Ok(outputs);
    }

    // Fallback to AABB-based deconstruction for non-mesh inputs (legacy support)
    let metrics = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new("Deconstruct Brep vereist een brep"))?;
    let corners = create_box_corners_points(&metrics);
    let faces = create_box_faces(&corners)
        .into_iter()
        .map(|face| Value::List(face.into_iter().map(Value::Point).collect()))
        .collect();
    let edges = create_wireframe(&metrics)
        .into_iter()
        .map(|(p1, p2)| Value::CurveLine { p1, p2 })
        .collect();
    let vertices = corners.into_iter().map(Value::Point).collect();

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_FACES.to_owned(), Value::List(faces));
    outputs.insert(PIN_OUTPUT_EDGES.to_owned(), Value::List(edges));
    outputs.insert(PIN_OUTPUT_VERTICES.to_owned(), Value::List(vertices));
    Ok(outputs)
}

fn evaluate_box_corners(inputs: &[Value]) -> ComponentResult {
    let metrics = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new("Box Corners vereist een box"))?;
    let corners = create_box_corners_points(&metrics)
        .into_iter()
        .map(Value::Point)
        .collect();
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::List(corners));
    Ok(outputs)
}

fn evaluate_brep_wireframe(inputs: &[Value]) -> ComponentResult {
    // Try to use proper mesh topology extraction first
    if let Some(topology) = try_mesh_topology(inputs.get(0)) {
        let wireframe = topology
            .all_edges()
            .into_iter()
            .map(|(p1, p2)| Value::CurveLine { p1, p2 })
            .collect();
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_WIREFRAME.to_owned(), Value::List(wireframe));
        return Ok(outputs);
    }

    // Fallback to AABB-based wireframe for non-mesh inputs (legacy support)
    let metrics = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new("Brep Wireframe vereist een brep"))?;
    let wireframe = create_wireframe(&metrics)
        .into_iter()
        .map(|(p1, p2)| Value::CurveLine { p1, p2 })
        .collect();
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_WIREFRAME.to_owned(), Value::List(wireframe));
    Ok(outputs)
}

fn evaluate_box_properties(inputs: &[Value]) -> ComponentResult {
    let metrics = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new("Box Properties vereist een box"))?;
    let size = metrics.size();
    let diagonal = [size[0], size[1], size[2]];
    let degeneracy = diagonal
        .iter()
        .filter(|value| value.abs() <= EPSILON)
        .count();

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_CENTROID.to_owned(),
        Value::Point(metrics.center()),
    );
    outputs.insert(PIN_OUTPUT_DISTANCE.to_owned(), Value::Vector(diagonal));
    outputs.insert(PIN_OUTPUT_AREA.to_owned(), Value::Number(metrics.area()));
    outputs.insert(
        PIN_OUTPUT_VOLUME.to_owned(),
        Value::Number(metrics.volume()),
    );
    outputs.insert("d".to_owned(), Value::Number(degeneracy as f64));
    Ok(outputs)
}

fn evaluate_osculating_circles(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Osculating Circles requires a surface input",
        ));
    }
    
    // Check if input is a mesh and provide clear error
    require_parametric_surface(
        inputs.get(0),
        "Osculating Circles",
        "curvature circle",
    )?;
    
    let metrics = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new("Osculating Circles could not read the surface"))?;
    let uv = coerce_uv(inputs.get(1)).unwrap_or((0.5, 0.5));
    let point = metrics.sample_point(uv);
    let size = metrics.size();
    let radius_u = if size[0].abs() <= EPSILON {
        0.0
    } else {
        size[0].abs() * 0.5
    };
    let radius_v = if size[1].abs() <= EPSILON {
        0.0
    } else {
        size[1].abs() * 0.5
    };
    let circle_u = Value::List(vec![
        Value::Point(point),
        Value::Point([point[0] + radius_u, point[1], point[2]]),
    ]);
    let circle_v = Value::List(vec![
        Value::Point(point),
        Value::Point([point[0], point[1] + radius_v, point[2]]),
    ]);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::Point(point));
    outputs.insert(PIN_OUTPUT_CIRCLE_ONE.to_owned(), circle_u);
    outputs.insert(PIN_OUTPUT_CIRCLE_TWO.to_owned(), circle_v);
    Ok(outputs)
}

fn evaluate_is_planar(inputs: &[Value]) -> ComponentResult {
    let metrics = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new("Is Planar vereist een surface"))?;
    let size = metrics.size();
    let planar = size[2].abs() <= EPSILON;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_PLANAR.to_owned(), Value::Boolean(planar));
    outputs.insert(
        PIN_OUTPUT_PLANE.to_owned(),
        plane_from_point(metrics.center()),
    );
    Ok(outputs)
}

fn evaluate_deconstruct_box(inputs: &[Value]) -> ComponentResult {
    let metrics = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new("Deconstruct Box vereist een box"))?;
    let size = metrics.size();
    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_PLANE.to_owned(),
        plane_from_point(metrics.center()),
    );
    outputs.insert(PIN_OUTPUT_X_SIZE.to_owned(), Value::Number(size[0].abs()));
    outputs.insert(PIN_OUTPUT_Y_SIZE.to_owned(), Value::Number(size[1].abs()));
    outputs.insert(PIN_OUTPUT_Z_SIZE.to_owned(), Value::Number(size[2].abs()));
    Ok(outputs)
}

fn evaluate_point_in_brep(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Point In Brep vereist een brep en een punt",
        ));
    }
    let point = coerce_point(inputs.get(1), "Point In Brep punt")?;
    let strict = coerce_boolean(inputs.get(2), false)?;
    
    // Try proper mesh-based containment test first
    if let Some(is_inside) = try_mesh_point_containment(inputs.get(0), point, strict) {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_INSIDE.to_owned(), Value::Boolean(is_inside));
        return Ok(outputs);
    }
    
    // Fallback to AABB-based containment for non-mesh inputs
    let brep = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new("Point In Brep vereist een brep"))?;
    let inside = point_in_metrics(&brep, point, strict);
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_INSIDE.to_owned(), Value::Boolean(inside));
    Ok(outputs)
}

fn evaluate_dimensions(inputs: &[Value]) -> ComponentResult {
    // Reject mesh inputs: U/V dimensions refer to the extent of the surface parameter domain,
    // not the bounding box size. Meshes lack (u,v) parameterization entirely. Using AABB
    // extents would fabricate semantically incorrect values.
    require_parametric_surface(
        inputs.get(0),
        "Dimensions",
        "parameter domain size (U/V extents)",
    )?;

    let metrics = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new("Dimensions vereist een surface"))?;
    let size = metrics.size();
    let mut outputs = BTreeMap::new();
    outputs.insert("U".to_owned(), Value::Number(size[0].abs()));
    outputs.insert("V".to_owned(), Value::Number(size[1].abs()));
    Ok(outputs)
}

fn evaluate_point_in_trim(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Point In Trim requires a surface and a UV point",
        ));
    }
    
    // Check if input is a mesh and provide clear error
    require_parametric_surface(
        inputs.get(0),
        "Point In Trim",
        "trim region",
    )?;
    
    let uv = coerce_uv(inputs.get(1)).unwrap_or((0.5, 0.5));
    let inside = (0.0..=1.0).contains(&uv.0) && (0.0..=1.0).contains(&uv.1);
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_INSIDE.to_owned(), Value::Boolean(inside));
    Ok(outputs)
}

fn coerce_number(value: Option<&Value>, context: &str) -> Result<f64, ComponentError> {
    match value {
        Some(Value::Number(number)) => Ok(*number),
        Some(Value::Boolean(flag)) => Ok(if *flag { 1.0 } else { 0.0 }),
        Some(Value::List(values)) if values.len() == 1 => coerce_number(values.get(0), context),
        Some(other) => Err(ComponentError::new(format!(
            "{} verwacht een getal, kreeg {}",
            context,
            other.kind()
        ))),
        None => Err(ComponentError::new(format!(
            "{} vereist een numerieke invoer",
            context
        ))),
    }
}

fn coerce_boolean(value: Option<&Value>, default: bool) -> Result<bool, ComponentError> {
    match value {
        None => Ok(default),
        Some(Value::Boolean(flag)) => Ok(*flag),
        Some(Value::Number(number)) => Ok(*number != 0.0),
        Some(Value::List(values)) if values.len() == 1 => coerce_boolean(values.get(0), default),
        Some(Value::Text(text)) => {
            let normalized = text.trim().to_ascii_lowercase();
            if ["true", "yes", "1", "on"].contains(&normalized.as_str()) {
                Ok(true)
            } else if ["false", "no", "0", "off"].contains(&normalized.as_str()) {
                Ok(false)
            } else {
                Err(ComponentError::new(format!(
                    "Kon boolean niet afleiden uit '{}'",
                    text
                )))
            }
        }
        Some(other) => Err(ComponentError::new(format!(
            "Kon boolean niet afleiden uit {}",
            other.kind()
        ))),
    }
}

fn coerce_point(value: Option<&Value>, context: &str) -> Result<[f64; 3], ComponentError> {
    match value {
        Some(Value::Point(point)) | Some(Value::Vector(point)) => Ok(*point),
        Some(Value::List(values)) if values.len() >= 3 => {
            let x = coerce_number(values.get(0), context)?;
            let y = coerce_number(values.get(1), context)?;
            let z = coerce_number(values.get(2), context)?;
            Ok([x, y, z])
        }
        Some(Value::List(values)) if !values.is_empty() => coerce_point(values.get(0), context),
        Some(other) => Err(ComponentError::new(format!(
            "{} verwacht een punt, kreeg {}",
            context,
            other.kind()
        ))),
        None => Err(ComponentError::new(format!("{} vereist een punt", context))),
    }
}

fn coerce_uv(value: Option<&Value>) -> Option<(f64, f64)> {
    match value {
        Some(Value::Point([u, v, _])) => Some((*u, *v)),
        Some(Value::Vector([u, v, _])) => Some((*u, *v)),
        Some(Value::Number(number)) => Some((*number, *number)),
        Some(Value::List(values)) if values.len() >= 2 => {
            let u = coerce_number(values.get(0), "uv").ok()?;
            let v = coerce_number(values.get(1), "uv").ok()?;
            Some((u, v))
        }
        Some(Value::List(values)) if !values.is_empty() => coerce_uv(values.get(0)),
        _ => None,
    }
}

fn to_number_list(values: &[f64; 3]) -> Value {
    Value::List(values.iter().copied().map(Value::Number).collect())
}

fn distance(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

// ============================================================================
// Proper Mesh-Based Geometry Operations
// ============================================================================
// These functions use the geom module's proper algorithms for mesh queries,
// providing correct results for concave and rotated geometry.

/// Result of a mesh closest point query.
struct MeshClosestPointResult {
    /// The closest point on the mesh surface.
    point: [f64; 3],
    /// The distance from the query point to the closest point.
    distance: f64,
    /// The surface normal at the closest point.
    normal: [f64; 3],
    /// Approximate UV coordinates (derived from barycentric coordinates and triangle index).
    uv: (f64, f64),
}

/// Attempts to compute the closest point on a mesh using proper triangle-based
/// distance computation instead of AABB clamping.
///
/// Returns `Some(MeshClosestPointResult)` if the value is a valid mesh (Value::Mesh
/// or Value::Surface with triangle indices), `None` otherwise.
fn try_mesh_closest_point(value: Option<&Value>, target: [f64; 3]) -> Option<MeshClosestPointResult> {
    let geom_mesh = value_to_geom_mesh(value)?;
    let query = GeomPoint3::new(target[0], target[1], target[2]);
    
    let result = closest_point_on_mesh(&geom_mesh, query)?;
    
    // Approximate UV from triangle index and barycentric coordinates
    // For a proper UV, we'd need the original surface parameterization
    let tri_count = geom_mesh.indices.len() / 3;
    let u_approx = if tri_count > 0 {
        (result.triangle_index as f64) / (tri_count as f64)
    } else {
        0.5
    };
    // Use barycentric v coordinate as approximate V
    let v_approx = result.barycentric.1.clamp(0.0, 1.0);
    
    Some(MeshClosestPointResult {
        point: [result.point.x, result.point.y, result.point.z],
        distance: result.distance_squared.sqrt(),
        normal: [result.normal.x, result.normal.y, result.normal.z],
        uv: (u_approx, v_approx),
    })
}

/// Attempts to test if a point is inside a mesh using proper ray-casting
/// containment testing instead of AABB containment.
///
/// Returns `Some(bool)` if the value is a valid mesh (Value::Mesh or Value::Surface
/// with triangle indices), `None` otherwise.
///
/// # Arguments
/// * `value` - The mesh value to test against.
/// * `point` - The point to test.
/// * `strict` - If true, points on the surface are considered outside.
fn try_mesh_point_containment(value: Option<&Value>, point: [f64; 3], strict: bool) -> Option<bool> {
    let geom_mesh = value_to_geom_mesh(value)?;
    let query = GeomPoint3::new(point[0], point[1], point[2]);
    
    // Use a reasonable tolerance for containment testing
    let tol = GeomTolerance::new(EPSILON);
    
    let containment = classify_point_in_mesh(query, &geom_mesh, tol);
    
    let is_inside = match containment {
        PointContainment::Inside => true,
        PointContainment::OnSurface => !strict,
        PointContainment::Outside => false,
        PointContainment::Indeterminate => {
            // Fall back to false for indeterminate cases
            // The caller may want to try AABB as a fallback
            return None;
        }
    };
    
    Some(is_inside)
}

/// Converts a Value to a GeomMesh if possible.
///
/// Handles both Value::Mesh and Value::Surface (legacy format).
fn value_to_geom_mesh(value: Option<&Value>) -> Option<GeomMesh> {
    match value {
        Some(Value::Mesh { vertices, indices, normals, uvs, .. }) => {
            if vertices.is_empty() || indices.len() < 3 {
                return None;
            }
            Some(GeomMesh {
                positions: vertices.clone(),
                indices: indices.clone(),
                normals: normals.clone(),
                uvs: uvs.clone(),
                tangents: None,
            })
        }
        Some(Value::Surface { vertices, faces }) => {
            if vertices.is_empty() || faces.is_empty() {
                return None;
            }
            // Convert face list to triangle indices
            let indices = surface_faces_to_triangle_indices(faces);
            if indices.is_empty() {
                return None;
            }
            Some(GeomMesh::new(vertices.clone(), indices))
        }
        _ => None,
    }
}

// ============================================================================
// Triangle Mesh Geometry Helpers
// ============================================================================

/// Computes the cross product of two 3D vectors.
#[inline]
fn cross_product(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Computes the dot product of two 3D vectors.
#[inline]
fn dot_product(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Computes the vector from point `a` to point `b`.
#[inline]
fn vec_sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Computes the length of a 3D vector.
#[inline]
fn vec_length(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Computes the area of a triangle given three vertices using the cross product method.
/// Area = 0.5 * |AB × AC|
fn triangle_area(v0: [f64; 3], v1: [f64; 3], v2: [f64; 3]) -> f64 {
    let ab = vec_sub(v1, v0);
    let ac = vec_sub(v2, v0);
    let cross = cross_product(ab, ac);
    0.5 * vec_length(cross)
}

/// Computes the signed volume of a tetrahedron formed by a triangle and the origin.
/// This is used for computing the volume of a closed mesh.
/// Volume = (1/6) * (v0 · (v1 × v2))
fn signed_tetrahedron_volume(v0: [f64; 3], v1: [f64; 3], v2: [f64; 3]) -> f64 {
    let cross = cross_product(v1, v2);
    dot_product(v0, cross) / 6.0
}

/// Computes the centroid of a triangle.
fn triangle_centroid(v0: [f64; 3], v1: [f64; 3], v2: [f64; 3]) -> [f64; 3] {
    [
        (v0[0] + v1[0] + v2[0]) / 3.0,
        (v0[1] + v1[1] + v2[1]) / 3.0,
        (v0[2] + v1[2] + v2[2]) / 3.0,
    ]
}

// ============================================================================
// MeshMetrics: Proper triangle mesh analysis
// ============================================================================

/// Metrics for a triangle mesh that properly computes area, volume, and
/// moments of inertia from the actual triangle faces rather than bounding box.
#[derive(Debug, Clone)]
struct MeshMetrics {
    vertices: Vec<[f64; 3]>,
    /// Triangle indices (every 3 consecutive indices form one triangle)
    indices: Vec<u32>,
    min: [f64; 3],
    max: [f64; 3],
    /// Cached surface area (sum of all triangle areas)
    cached_area: Option<f64>,
    /// Cached signed volume (sum of signed tetrahedra volumes)
    cached_volume: Option<f64>,
    /// Cached centroid (area-weighted average of triangle centroids)
    cached_centroid: Option<[f64; 3]>,
}

impl MeshMetrics {
    /// Attempts to create MeshMetrics from a Value::Mesh or Value::Surface.
    /// Returns None if the input is not a mesh-like value.
    fn from_value(value: Option<&Value>) -> Option<Self> {
        match value {
            Some(Value::Mesh { vertices, indices, .. }) => {
                if vertices.is_empty() || indices.len() < 3 {
                    return None;
                }
                let (min, max) = bounding_box(vertices);
                Some(Self {
                    vertices: vertices.clone(),
                    indices: indices.clone(),
                    min,
                    max,
                    cached_area: None,
                    cached_volume: None,
                    cached_centroid: None,
                })
            }
            Some(Value::Surface { vertices, faces }) => {
                if vertices.is_empty() || faces.is_empty() {
                    return None;
                }
                // Convert face list to triangle indices
                let indices = surface_faces_to_triangle_indices(faces);
                if indices.is_empty() {
                    return None;
                }
                let (min, max) = bounding_box(vertices);
                Some(Self {
                    vertices: vertices.clone(),
                    indices,
                    min,
                    max,
                    cached_area: None,
                    cached_volume: None,
                    cached_centroid: None,
                })
            }
            Some(Value::List(items)) if !items.is_empty() => {
                // Try to extract from first mesh in list
                Self::from_value(items.first())
            }
            _ => None,
        }
    }

    /// Returns the number of triangles in this mesh.
    fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Computes the total surface area by summing all triangle areas.
    fn area(&mut self) -> f64 {
        if let Some(area) = self.cached_area {
            return area;
        }

        let mut total_area = 0.0;
        let tri_count = self.triangle_count();

        for i in 0..tri_count {
            let idx0 = self.indices[i * 3] as usize;
            let idx1 = self.indices[i * 3 + 1] as usize;
            let idx2 = self.indices[i * 3 + 2] as usize;

            if idx0 < self.vertices.len()
                && idx1 < self.vertices.len()
                && idx2 < self.vertices.len()
            {
                let v0 = self.vertices[idx0];
                let v1 = self.vertices[idx1];
                let v2 = self.vertices[idx2];
                total_area += triangle_area(v0, v1, v2);
            }
        }

        self.cached_area = Some(total_area);
        total_area
    }

    /// Computes the signed volume of a closed mesh using the divergence theorem.
    /// For open meshes, this returns an approximate volume (may be inaccurate).
    fn volume(&mut self) -> f64 {
        if let Some(volume) = self.cached_volume {
            return volume;
        }

        let mut total_volume = 0.0;
        let tri_count = self.triangle_count();

        for i in 0..tri_count {
            let idx0 = self.indices[i * 3] as usize;
            let idx1 = self.indices[i * 3 + 1] as usize;
            let idx2 = self.indices[i * 3 + 2] as usize;

            if idx0 < self.vertices.len()
                && idx1 < self.vertices.len()
                && idx2 < self.vertices.len()
            {
                let v0 = self.vertices[idx0];
                let v1 = self.vertices[idx1];
                let v2 = self.vertices[idx2];
                total_volume += signed_tetrahedron_volume(v0, v1, v2);
            }
        }

        // Take absolute value since winding order may vary
        let volume = total_volume.abs();
        self.cached_volume = Some(volume);
        volume
    }

    /// Computes the centroid (center of mass) as the area-weighted average
    /// of triangle centroids.
    fn centroid(&mut self) -> [f64; 3] {
        if let Some(centroid) = self.cached_centroid {
            return centroid;
        }

        let mut weighted_sum = [0.0, 0.0, 0.0];
        let mut total_area = 0.0;
        let tri_count = self.triangle_count();

        for i in 0..tri_count {
            let idx0 = self.indices[i * 3] as usize;
            let idx1 = self.indices[i * 3 + 1] as usize;
            let idx2 = self.indices[i * 3 + 2] as usize;

            if idx0 < self.vertices.len()
                && idx1 < self.vertices.len()
                && idx2 < self.vertices.len()
            {
                let v0 = self.vertices[idx0];
                let v1 = self.vertices[idx1];
                let v2 = self.vertices[idx2];

                let area = triangle_area(v0, v1, v2);
                if area > EPSILON {
                    let center = triangle_centroid(v0, v1, v2);
                    weighted_sum[0] += center[0] * area;
                    weighted_sum[1] += center[1] * area;
                    weighted_sum[2] += center[2] * area;
                    total_area += area;
                }
            }
        }

        let centroid = if total_area > EPSILON {
            [
                weighted_sum[0] / total_area,
                weighted_sum[1] / total_area,
                weighted_sum[2] / total_area,
            ]
        } else {
            // Fallback to bounding box center
            [
                (self.min[0] + self.max[0]) * 0.5,
                (self.min[1] + self.max[1]) * 0.5,
                (self.min[2] + self.max[2]) * 0.5,
            ]
        };

        self.cached_centroid = Some(centroid);
        centroid
    }

    /// Computes approximate moments of inertia for the mesh.
    /// This uses the volume-weighted approach for solid meshes.
    fn inertia(&mut self) -> [f64; 3] {
        let volume = self.volume();
        if volume <= EPSILON {
            return [0.0; 3];
        }

        let centroid = self.centroid();
        let mut ixx = 0.0;
        let mut iyy = 0.0;
        let mut izz = 0.0;

        let tri_count = self.triangle_count();

        for i in 0..tri_count {
            let idx0 = self.indices[i * 3] as usize;
            let idx1 = self.indices[i * 3 + 1] as usize;
            let idx2 = self.indices[i * 3 + 2] as usize;

            if idx0 < self.vertices.len()
                && idx1 < self.vertices.len()
                && idx2 < self.vertices.len()
            {
                let v0 = self.vertices[idx0];
                let v1 = self.vertices[idx1];
                let v2 = self.vertices[idx2];

                // Translate to centroid-relative coordinates
                let p0 = vec_sub(v0, centroid);
                let p1 = vec_sub(v1, centroid);
                let p2 = vec_sub(v2, centroid);

                // Compute contribution using the tetrahedron method
                let tet_vol = signed_tetrahedron_volume(p0, p1, p2).abs();
                if tet_vol > EPSILON {
                    // Use the average squared distance from centroid for each axis
                    let avg_y2_z2 = (p0[1].powi(2) + p0[2].powi(2)
                        + p1[1].powi(2) + p1[2].powi(2)
                        + p2[1].powi(2) + p2[2].powi(2)) / 3.0;
                    let avg_x2_z2 = (p0[0].powi(2) + p0[2].powi(2)
                        + p1[0].powi(2) + p1[2].powi(2)
                        + p2[0].powi(2) + p2[2].powi(2)) / 3.0;
                    let avg_x2_y2 = (p0[0].powi(2) + p0[1].powi(2)
                        + p1[0].powi(2) + p1[1].powi(2)
                        + p2[0].powi(2) + p2[1].powi(2)) / 3.0;

                    ixx += tet_vol * avg_y2_z2;
                    iyy += tet_vol * avg_x2_z2;
                    izz += tet_vol * avg_x2_y2;
                }
            }
        }

        [ixx, iyy, izz]
    }

    /// Computes products of inertia (Ixy, Iyz, Izx).
    fn secondary_moments(&mut self) -> [f64; 3] {
        let volume = self.volume();
        if volume <= EPSILON {
            return [0.0; 3];
        }

        let centroid = self.centroid();
        let mut ixy = 0.0;
        let mut iyz = 0.0;
        let mut izx = 0.0;

        let tri_count = self.triangle_count();

        for i in 0..tri_count {
            let idx0 = self.indices[i * 3] as usize;
            let idx1 = self.indices[i * 3 + 1] as usize;
            let idx2 = self.indices[i * 3 + 2] as usize;

            if idx0 < self.vertices.len()
                && idx1 < self.vertices.len()
                && idx2 < self.vertices.len()
            {
                let v0 = self.vertices[idx0];
                let v1 = self.vertices[idx1];
                let v2 = self.vertices[idx2];

                let p0 = vec_sub(v0, centroid);
                let p1 = vec_sub(v1, centroid);
                let p2 = vec_sub(v2, centroid);

                let tet_vol = signed_tetrahedron_volume(p0, p1, p2).abs();
                if tet_vol > EPSILON {
                    let avg_xy = (p0[0] * p0[1] + p1[0] * p1[1] + p2[0] * p2[1]) / 3.0;
                    let avg_yz = (p0[1] * p0[2] + p1[1] * p1[2] + p2[1] * p2[2]) / 3.0;
                    let avg_zx = (p0[2] * p0[0] + p1[2] * p1[0] + p2[2] * p2[0]) / 3.0;

                    ixy += tet_vol * avg_xy;
                    iyz += tet_vol * avg_yz;
                    izx += tet_vol * avg_zx;
                }
            }
        }

        [ixy.abs(), iyz.abs(), izx.abs()]
    }

    /// Computes radii of gyration from inertia and volume.
    fn gyration(&mut self) -> [f64; 3] {
        let volume = self.volume();
        if volume <= EPSILON {
            return [0.0; 3];
        }

        let inertia = self.inertia();
        [
            (inertia[0] / volume).abs().sqrt(),
            (inertia[1] / volume).abs().sqrt(),
            (inertia[2] / volume).abs().sqrt(),
        ]
    }

    /// Returns bounding box center (for compatibility with ShapeMetrics).
    /// Currently unused but retained for potential future use.
    #[allow(dead_code)]
    fn bbox_center(&self) -> [f64; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }
}

/// Converts legacy Surface face lists (which may be quads/n-gons) to triangle indices.
fn surface_faces_to_triangle_indices(faces: &[Vec<u32>]) -> Vec<u32> {
    let mut indices = Vec::new();
    for face in faces {
        if face.len() < 3 {
            continue;
        }
        // Fan triangulation for polygons
        for i in 1..(face.len() - 1) {
            indices.push(face[0]);
            indices.push(face[i] as u32);
            indices.push(face[i + 1] as u32);
        }
    }
    indices
}

// ============================================================================
// ShapeMetrics: AABB-based metrics (used when no mesh indices available)
// ============================================================================

#[derive(Debug, Clone)]
struct ShapeMetrics {
    points: Vec<[f64; 3]>,
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
        Some(Self { points, min, max })
    }

    fn center(&self) -> [f64; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }

    fn size(&self) -> [f64; 3] {
        [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ]
    }

    fn volume(&self) -> f64 {
        let size = self.size();
        size[0].abs() * size[1].abs() * size[2].abs()
    }

    fn area(&self) -> f64 {
        let size = self.size();
        let xy = size[0].abs() * size[1].abs();
        let yz = size[1].abs() * size[2].abs();
        let zx = size[0].abs() * size[2].abs();
        if yz <= EPSILON && zx <= EPSILON {
            xy
        } else {
            2.0 * (xy + yz + zx)
        }
    }

    fn sample_point(&self, uv: (f64, f64)) -> [f64; 3] {
        [
            self.min[0] + self.size()[0] * uv.0,
            self.min[1] + self.size()[1] * uv.1,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }
}

fn simple_inertia(size: [f64; 3], mass: f64) -> [f64; 3] {
    if mass.abs() <= EPSILON {
        return [0.0; 3];
    }
    [
        mass * (size[1].powi(2) + size[2].powi(2)) / 12.0,
        mass * (size[0].powi(2) + size[2].powi(2)) / 12.0,
        mass * (size[0].powi(2) + size[1].powi(2)) / 12.0,
    ]
}

fn simple_secondary(size: [f64; 3], mass: f64) -> [f64; 3] {
    if mass.abs() <= EPSILON {
        return [0.0; 3];
    }
    [
        mass * size[0].abs() * size[1].abs() / 12.0,
        mass * size[1].abs() * size[2].abs() / 12.0,
        mass * size[0].abs() * size[2].abs() / 12.0,
    ]
}

fn simple_gyration(inertia: [f64; 3], mass: f64) -> [f64; 3] {
    if mass.abs() <= EPSILON {
        return [0.0; 3];
    }
    [
        (inertia[0] / mass).abs().sqrt(),
        (inertia[1] / mass).abs().sqrt(),
        (inertia[2] / mass).abs().sqrt(),
    ]
}

fn plane_from_point(origin: [f64; 3]) -> Value {
    Value::List(vec![
        Value::Point(origin),
        Value::Point([origin[0] + 1.0, origin[1], origin[2]]),
        Value::Point([origin[0], origin[1] + 1.0, origin[2]]),
    ])
}

/// Creates a frame value from origin and axis vectors.
fn frame_from_axes(
    origin: [f64; 3],
    x_axis: [f64; 3],
    y_axis: [f64; 3],
    z_axis: [f64; 3],
) -> Value {
    Value::List(vec![
        Value::Point(origin),
        Value::Vector(x_axis),
        Value::Vector(y_axis),
        Value::Vector(z_axis),
    ])
}

/// Normalizes a 3D vector, returning unit X on failure.
fn normalize_vec3(v: [f64; 3]) -> [f64; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < EPSILON {
        [1.0, 0.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

/// Cross product of two 3D vectors.
fn cross_vec3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Estimates a surface frame (normal, u_dir, v_dir) from point cloud metrics.
///
/// Uses PCA-like analysis when possible, or falls back to axis-aligned frame.
fn estimate_surface_frame_from_metrics(
    metrics: &ShapeMetrics,
    _uv: (f64, f64),
) -> ([f64; 3], [f64; 3], [f64; 3]) {
    let points = &metrics.points;
    
    if points.len() < 3 {
        // Too few points - return axis-aligned frame
        return ([0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    }
    
    // Compute centroid
    let n = points.len() as f64;
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for p in points {
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    cx /= n;
    cy /= n;
    cz /= n;
    
    // Build covariance matrix
    let mut cov = [[0.0f64; 3]; 3];
    for p in points {
        let d = [p[0] - cx, p[1] - cy, p[2] - cz];
        for i in 0..3 {
            for j in 0..3 {
                cov[i][j] += d[i] * d[j];
            }
        }
    }
    
    // Simple power iteration for principal directions
    // Find the two largest eigenvectors (in-plane) and the smallest (normal)
    let (u_dir, v_dir, normal) = compute_pca_frame(&cov);
    
    (normal, u_dir, v_dir)
}

/// Computes a frame from covariance matrix using power iteration.
fn compute_pca_frame(cov: &[[f64; 3]; 3]) -> ([f64; 3], [f64; 3], [f64; 3]) {
    // Power iteration for first eigenvector (largest eigenvalue)
    let mut v1 = [1.0, 0.0, 0.0];
    for _ in 0..20 {
        let mut result = [0.0; 3];
        for i in 0..3 {
            for j in 0..3 {
                result[i] += cov[i][j] * v1[j];
            }
        }
        v1 = normalize_vec3(result);
    }
    
    // Deflate covariance for second eigenvector
    let mut cov2 = *cov;
    let e1 = eigenvalue_for_vector(cov, v1);
    for i in 0..3 {
        for j in 0..3 {
            cov2[i][j] -= e1 * v1[i] * v1[j];
        }
    }
    
    // Power iteration for second eigenvector
    let mut v2 = if v1[0].abs() < 0.9 { [1.0, 0.0, 0.0] } else { [0.0, 1.0, 0.0] };
    for _ in 0..20 {
        let mut result = [0.0; 3];
        for i in 0..3 {
            for j in 0..3 {
                result[i] += cov2[i][j] * v2[j];
            }
        }
        v2 = normalize_vec3(result);
    }
    
    // Normal is cross product of the two largest directions
    let normal = normalize_vec3(cross_vec3(v1, v2));
    
    // Ensure orthogonality
    let v2_ortho = normalize_vec3(cross_vec3(normal, v1));
    
    (v1, v2_ortho, normal)
}

/// Computes eigenvalue for a given eigenvector using Rayleigh quotient.
fn eigenvalue_for_vector(cov: &[[f64; 3]; 3], v: [f64; 3]) -> f64 {
    let mut result = [0.0; 3];
    for i in 0..3 {
        for j in 0..3 {
            result[i] += cov[i][j] * v[j];
        }
    }
    // Return v · (cov · v)
    v[0] * result[0] + v[1] * result[1] + v[2] * result[2]
}

/// Estimates principal curvatures from point cloud metrics.
///
/// This is an approximation based on local geometry analysis.
/// For accurate curvature, use a proper parametric surface.
fn estimate_curvature_from_metrics(
    metrics: &ShapeMetrics,
    uv: (f64, f64),
) -> (f64, f64, [f64; 3], [f64; 3]) {
    let points = &metrics.points;
    
    if points.len() < 9 {
        // Too few points for curvature estimation - return zero curvature
        return (0.0, 0.0, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    }
    
    // Get the center point and frame
    let center = metrics.sample_point(uv);
    let (normal, u_dir, v_dir) = estimate_surface_frame_from_metrics(metrics, uv);
    
    // Sample nearby points to estimate local curvature
    // Use deviation from tangent plane as a proxy for curvature
    let mut u_curvature_sum = 0.0;
    let mut v_curvature_sum = 0.0;
    let mut u_count = 0;
    let mut v_count = 0;
    
    for p in points {
        // Distance from center
        let dx = p[0] - center[0];
        let dy = p[1] - center[1];
        let dz = p[2] - center[2];
        
        // Project onto local frame
        let u_proj = dx * u_dir[0] + dy * u_dir[1] + dz * u_dir[2];
        let v_proj = dx * v_dir[0] + dy * v_dir[1] + dz * v_dir[2];
        let n_proj = dx * normal[0] + dy * normal[1] + dz * normal[2];
        
        // Estimate curvature as second derivative approximation
        // κ ≈ 2h / d² where h is height and d is in-plane distance
        let u_dist = u_proj.abs();
        let v_dist = v_proj.abs();
        
        if u_dist > EPSILON && u_dist < metrics.size()[0] * 0.5 {
            // Curvature estimate in u direction
            let kappa = 2.0 * n_proj / (u_dist * u_dist);
            if kappa.is_finite() {
                u_curvature_sum += kappa;
                u_count += 1;
            }
        }
        
        if v_dist > EPSILON && v_dist < metrics.size()[1] * 0.5 {
            // Curvature estimate in v direction
            let kappa = 2.0 * n_proj / (v_dist * v_dist);
            if kappa.is_finite() {
                v_curvature_sum += kappa;
                v_count += 1;
            }
        }
    }
    
    let k_u = if u_count > 0 {
        u_curvature_sum / u_count as f64
    } else {
        0.0
    };
    
    let k_v = if v_count > 0 {
        v_curvature_sum / v_count as f64
    } else {
        0.0
    };
    
    // Principal curvatures are the max/min
    let (k1, k2) = if k_u.abs() >= k_v.abs() {
        (k_u, k_v)
    } else {
        (k_v, k_u)
    };
    
    // Principal directions (approximate - aligned with local frame)
    let (k1_dir, k2_dir) = if k_u.abs() >= k_v.abs() {
        (u_dir, v_dir)
    } else {
        (v_dir, u_dir)
    };
    
    (k1, k2, k1_dir, k2_dir)
}

fn clamp_to_metrics(metrics: &ShapeMetrics, target: [f64; 3]) -> [f64; 3] {
    [
        target[0].clamp(metrics.min[0], metrics.max[0]),
        target[1].clamp(metrics.min[1], metrics.max[1]),
        target[2].clamp(metrics.min[2], metrics.max[2]),
    ]
}

fn uv_from_point(metrics: &ShapeMetrics, point: [f64; 3]) -> (f64, f64) {
    let size = metrics.size();
    let u = if size[0].abs() <= EPSILON {
        0.0
    } else {
        (point[0] - metrics.min[0]) / size[0]
    };
    let v = if size[1].abs() <= EPSILON {
        0.0
    } else {
        (point[1] - metrics.min[1]) / size[1]
    };
    (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
}

fn point_in_metrics(metrics: &ShapeMetrics, point: [f64; 3], strict: bool) -> bool {
    let tolerance = if strict { EPSILON } else { -EPSILON };
    point[0] >= metrics.min[0] - tolerance
        && point[0] <= metrics.max[0] + tolerance
        && point[1] >= metrics.min[1] - tolerance
        && point[1] <= metrics.max[1] + tolerance
        && point[2] >= metrics.min[2] - tolerance
        && point[2] <= metrics.max[2] + tolerance
}

fn boxes_overlap(a: &ShapeMetrics, b: &ShapeMetrics) -> bool {
    !(a.max[0] < b.min[0]
        || a.min[0] > b.max[0]
        || a.max[1] < b.min[1]
        || a.min[1] > b.max[1]
        || a.max[2] < b.min[2]
        || a.min[2] > b.max[2])
}

fn collect_point_grid(value: Option<&Value>) -> Option<Vec<Vec<[f64; 3]>>> {
    match value {
        Some(Value::List(rows)) if rows.iter().all(|row| matches!(row, Value::List(_))) => {
            let mut result = Vec::new();
            for row in rows {
                if let Value::List(entries) = row {
                    let mut parsed_row = Vec::new();
                    for entry in entries {
                        if let Some(point) = try_point(entry) {
                            parsed_row.push(point);
                        }
                    }
                    if !parsed_row.is_empty() {
                        result.push(parsed_row);
                    }
                }
            }
            if result.is_empty() {
                None
            } else {
                Some(result)
            }
        }
        _ => None,
    }
}

fn collect_points(value: Option<&Value>) -> Vec<[f64; 3]> {
    match value {
        Some(Value::Point(point)) | Some(Value::Vector(point)) => vec![*point],
        Some(Value::CurveLine { p1, p2 }) => vec![*p1, *p2],
        Some(Value::Surface { vertices, .. }) => vertices.clone(),
        // Support for Value::Mesh - extract vertices from the mesh
        Some(Value::Mesh { vertices, .. }) => vertices.clone(),
        Some(Value::List(values)) => values
            .iter()
            .flat_map(|value| collect_points(Some(value)))
            .collect(),
        _ => Vec::new(),
    }
}

fn try_point(value: &Value) -> Option<[f64; 3]> {
    match value {
        Value::Point(point) | Value::Vector(point) => Some(*point),
        Value::List(values) if values.len() >= 3 => {
            let x = coerce_number(Some(&values[0]), "punt").ok()?;
            let y = coerce_number(Some(&values[1]), "punt").ok()?;
            let z = coerce_number(Some(&values[2]), "punt").ok()?;
            Some([x, y, z])
        }
        Value::List(values) if !values.is_empty() => try_point(&values[0]),
        _ => None,
    }
}

fn create_wireframe(metrics: &ShapeMetrics) -> Vec<([f64; 3], [f64; 3])> {
    let corners = create_box_corners_points(metrics);
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
        .map(|(a, b)| (corners[*a], corners[*b]))
        .collect()
}

fn create_box_corners_points(metrics: &ShapeMetrics) -> Vec<[f64; 3]> {
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

fn create_box_faces(corners: &[[f64; 3]]) -> Vec<Vec<[f64; 3]>> {
    vec![
        vec![corners[0], corners[1], corners[2], corners[3]],
        vec![corners[4], corners[5], corners[6], corners[7]],
        vec![corners[0], corners[1], corners[5], corners[4]],
        vec![corners[2], corners[3], corners[7], corners[6]],
        vec![corners[1], corners[2], corners[6], corners[5]],
        vec![corners[0], corners[3], corners[7], corners[4]],
    ]
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

// ============================================================================
// Mesh Topology Extraction
// ============================================================================
// These functions extract actual edges and adjacency relationships from mesh
// geometry instead of using bounding-box approximations.

/// A canonical edge key using ordered vertex indices to ensure consistent hashing.
/// Edges are stored as (min_index, max_index) to ensure the same edge is always
/// identified regardless of which direction it's traversed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EdgeKey(u32, u32);

impl EdgeKey {
    /// Creates a canonical edge key from two vertex indices.
    fn new(v1: u32, v2: u32) -> Self {
        if v1 <= v2 {
            EdgeKey(v1, v2)
        } else {
            EdgeKey(v2, v1)
        }
    }
}

/// Classification of mesh edges based on how many faces share them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EdgeClassification {
    /// Naked/boundary edge: shared by exactly 1 face (mesh boundary).
    Naked,
    /// Interior edge: shared by exactly 2 faces (proper manifold edge).
    Interior,
    /// Non-manifold edge: shared by 3 or more faces (indicates mesh issues).
    NonManifold,
}

/// Information about a mesh edge, including its classification and adjacent faces.
#[derive(Debug, Clone)]
struct EdgeInfo {
    /// The canonical key for this edge.
    key: EdgeKey,
    /// Indices of faces that share this edge.
    adjacent_faces: Vec<usize>,
    /// Classification based on face count.
    classification: EdgeClassification,
}

/// Complete topology information for a mesh.
///
/// This struct extracts and stores edge information, face adjacency, and
/// vertex connectivity from a triangle mesh.
#[derive(Debug, Clone)]
struct MeshTopology {
    /// All unique edges in the mesh.
    edges: Vec<EdgeInfo>,
    /// Map from edge key to edge index in the `edges` vector.
    /// Currently unused but retained for potential O(1) edge lookups in future operations.
    #[allow(dead_code)]
    edge_lookup: HashMap<EdgeKey, usize>,
    /// For each face, the indices of edges that form its boundary.
    face_edges: Vec<Vec<usize>>,
    /// For each face, the indices of adjacent faces (sharing an edge).
    face_adjacency: Vec<Vec<usize>>,
    /// The original vertices from the mesh.
    vertices: Vec<[f64; 3]>,
    /// The original face definitions (polygons as vertex index lists).
    faces: Vec<Vec<u32>>,
}

impl MeshTopology {
    /// Extracts topology from a Value::Mesh or Value::Surface.
    ///
    /// Returns `None` if the input is not a valid mesh-like value.
    fn from_value(value: Option<&Value>) -> Option<Self> {
        let (vertices, faces) = match value {
            Some(Value::Mesh { vertices, indices, .. }) => {
                if vertices.is_empty() || indices.len() < 3 {
                    return None;
                }
                // Convert triangle indices to polygon faces
                let faces: Vec<Vec<u32>> = indices
                    .chunks(3)
                    .filter(|chunk| chunk.len() == 3)
                    .map(|chunk| vec![chunk[0], chunk[1], chunk[2]])
                    .collect();
                (vertices.clone(), faces)
            }
            Some(Value::Surface { vertices, faces }) => {
                if vertices.is_empty() || faces.is_empty() {
                    return None;
                }
                (vertices.clone(), faces.clone())
            }
            Some(Value::List(items)) if !items.is_empty() => {
                return Self::from_value(items.first());
            }
            _ => return None,
        };

        Some(Self::build(vertices, faces))
    }

    /// Builds topology from vertices and face definitions.
    fn build(vertices: Vec<[f64; 3]>, faces: Vec<Vec<u32>>) -> Self {
        let mut edge_counts: HashMap<EdgeKey, Vec<usize>> = HashMap::new();

        // First pass: collect all edges and their adjacent faces
        for (face_idx, face) in faces.iter().enumerate() {
            if face.len() < 3 {
                continue;
            }
            for i in 0..face.len() {
                let v1 = face[i];
                let v2 = face[(i + 1) % face.len()];
                let key = EdgeKey::new(v1, v2);
                edge_counts.entry(key).or_default().push(face_idx);
            }
        }

        // Build edge info list
        let mut edges = Vec::with_capacity(edge_counts.len());
        let mut edge_lookup = HashMap::with_capacity(edge_counts.len());

        for (key, adjacent_faces) in edge_counts {
            let classification = match adjacent_faces.len() {
                1 => EdgeClassification::Naked,
                2 => EdgeClassification::Interior,
                _ => EdgeClassification::NonManifold,
            };
            let edge_idx = edges.len();
            edge_lookup.insert(key, edge_idx);
            edges.push(EdgeInfo {
                key,
                adjacent_faces,
                classification,
            });
        }

        // Build face_edges mapping
        let mut face_edges = vec![Vec::new(); faces.len()];
        for (face_idx, face) in faces.iter().enumerate() {
            if face.len() < 3 {
                continue;
            }
            for i in 0..face.len() {
                let v1 = face[i];
                let v2 = face[(i + 1) % face.len()];
                let key = EdgeKey::new(v1, v2);
                if let Some(&edge_idx) = edge_lookup.get(&key) {
                    face_edges[face_idx].push(edge_idx);
                }
            }
        }

        // Build face adjacency: faces sharing an edge are adjacent
        let mut face_adjacency = vec![Vec::new(); faces.len()];
        for edge in &edges {
            for &face_a in &edge.adjacent_faces {
                for &face_b in &edge.adjacent_faces {
                    if face_a != face_b && !face_adjacency[face_a].contains(&face_b) {
                        face_adjacency[face_a].push(face_b);
                    }
                }
            }
        }

        Self {
            edges,
            edge_lookup,
            face_edges,
            face_adjacency,
            vertices,
            faces,
        }
    }

    /// Returns all naked (boundary) edges as line segments.
    fn naked_edges(&self) -> Vec<([f64; 3], [f64; 3])> {
        self.edges
            .iter()
            .filter(|e| e.classification == EdgeClassification::Naked)
            .map(|e| self.edge_to_line(e.key))
            .collect()
    }

    /// Returns all interior edges as line segments.
    fn interior_edges(&self) -> Vec<([f64; 3], [f64; 3])> {
        self.edges
            .iter()
            .filter(|e| e.classification == EdgeClassification::Interior)
            .map(|e| self.edge_to_line(e.key))
            .collect()
    }

    /// Returns all non-manifold edges as line segments.
    fn non_manifold_edges(&self) -> Vec<([f64; 3], [f64; 3])> {
        self.edges
            .iter()
            .filter(|e| e.classification == EdgeClassification::NonManifold)
            .map(|e| self.edge_to_line(e.key))
            .collect()
    }

    /// Returns all unique edges as line segments.
    fn all_edges(&self) -> Vec<([f64; 3], [f64; 3])> {
        self.edges
            .iter()
            .map(|e| self.edge_to_line(e.key))
            .collect()
    }

    /// Converts an edge key to a line segment using vertex coordinates.
    fn edge_to_line(&self, key: EdgeKey) -> ([f64; 3], [f64; 3]) {
        let v1 = self.vertices.get(key.0 as usize).copied().unwrap_or([0.0; 3]);
        let v2 = self.vertices.get(key.1 as usize).copied().unwrap_or([0.0; 3]);
        (v1, v2)
    }

    /// Returns faces as lists of points.
    fn face_points(&self) -> Vec<Vec<[f64; 3]>> {
        self.faces
            .iter()
            .map(|face| {
                face.iter()
                    .filter_map(|&idx| self.vertices.get(idx as usize).copied())
                    .collect()
            })
            .collect()
    }

    /// Returns face-to-face adjacency as a list of lists.
    /// For each face, returns the indices of faces that share an edge with it.
    fn get_face_face_adjacency(&self) -> Vec<Vec<usize>> {
        self.face_adjacency.clone()
    }

    /// Returns face-to-edge mapping as a list of lists.
    /// For each face, returns the indices of edges that form its boundary.
    fn get_face_edge_mapping(&self) -> Vec<Vec<usize>> {
        self.face_edges.clone()
    }

    /// Returns edge-to-face mapping as a list of lists.
    /// For each edge, returns the indices of faces that share it.
    fn get_edge_face_mapping(&self) -> Vec<Vec<usize>> {
        self.edges
            .iter()
            .map(|e| e.adjacent_faces.clone())
            .collect()
    }

    /// Returns all unique vertices that are actually used by the mesh.
    fn used_vertices(&self) -> Vec<[f64; 3]> {
        let mut used_indices: Vec<u32> = self
            .faces
            .iter()
            .flatten()
            .copied()
            .collect();
        used_indices.sort_unstable();
        used_indices.dedup();
        
        used_indices
            .iter()
            .filter_map(|&idx| self.vertices.get(idx as usize).copied())
            .collect()
    }
}

/// Attempts to extract mesh topology from a value.
/// Falls back to None if the value is not a mesh-like value.
fn try_mesh_topology(value: Option<&Value>) -> Option<MeshTopology> {
    MeshTopology::from_value(value)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates a simple mesh Value for testing.
    fn make_test_mesh() -> Value {
        Value::Mesh {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            indices: vec![0, 1, 2, 0, 1, 3, 0, 2, 3, 1, 2, 3],
            normals: None,
            uvs: None,
            diagnostics: None,
        }
    }

    /// Creates a simple surface Value for testing.
    fn make_test_surface() -> Value {
        Value::Surface {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
            ],
            faces: vec![vec![0, 1, 2]],
        }
    }

    // ========================================================================
    // Test: is_mesh_value helper
    // ========================================================================

    #[test]
    fn is_mesh_value_detects_mesh() {
        assert!(is_mesh_value(Some(&make_test_mesh())));
    }

    #[test]
    fn is_mesh_value_returns_false_for_surface() {
        assert!(!is_mesh_value(Some(&make_test_surface())));
    }

    #[test]
    fn is_mesh_value_returns_false_for_none() {
        assert!(!is_mesh_value(None));
    }

    #[test]
    fn is_mesh_value_unwraps_single_element_list() {
        let wrapped = Value::List(vec![make_test_mesh()]);
        assert!(is_mesh_value(Some(&wrapped)));
    }

    // ========================================================================
    // Test: require_parametric_surface helper
    // ========================================================================

    #[test]
    fn require_parametric_surface_rejects_mesh() {
        let result = require_parametric_surface(
            Some(&make_test_mesh()),
            "TestComponent",
            "curvature",
        );
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("TestComponent"));
        assert!(err_msg.contains("parametric surface"));
        assert!(err_msg.contains("triangulated mesh"));
    }

    #[test]
    fn require_parametric_surface_accepts_surface() {
        let result = require_parametric_surface(
            Some(&make_test_surface()),
            "TestComponent",
            "curvature",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn require_parametric_surface_accepts_none() {
        let result = require_parametric_surface(
            None,
            "TestComponent",
            "curvature",
        );
        assert!(result.is_ok());
    }

    // ========================================================================
    // Test: collect_points handles Mesh
    // ========================================================================

    #[test]
    fn collect_points_extracts_from_mesh() {
        let mesh = make_test_mesh();
        let points = collect_points(Some(&mesh));
        assert_eq!(points.len(), 4);
        assert_eq!(points[0], [0.0, 0.0, 0.0]);
        assert_eq!(points[1], [1.0, 0.0, 0.0]);
    }

    #[test]
    fn collect_points_extracts_from_surface() {
        let surface = make_test_surface();
        let points = collect_points(Some(&surface));
        assert_eq!(points.len(), 3);
    }

    // ========================================================================
    // Test: ShapeMetrics accepts Mesh
    // ========================================================================

    #[test]
    fn shape_metrics_from_mesh() {
        let mesh = make_test_mesh();
        let metrics = ShapeMetrics::from_inputs(Some(&mesh));
        assert!(metrics.is_some());
        let m = metrics.unwrap();
        assert_eq!(m.points.len(), 4);
        // Check bounding box
        assert_eq!(m.min[0], 0.0);
        assert_eq!(m.max[0], 1.0);
    }

    // ========================================================================
    // Test: Area component accepts Mesh
    // ========================================================================

    #[test]
    fn area_component_accepts_mesh() {
        let mesh = make_test_mesh();
        let result = evaluate_area(&[mesh], "Area");
        assert!(result.is_ok());
        let outputs = result.unwrap();
        assert!(outputs.contains_key(PIN_OUTPUT_AREA));
    }

    // ========================================================================
    // Test: Volume component accepts Mesh
    // ========================================================================

    #[test]
    fn volume_component_accepts_mesh() {
        let mesh = make_test_mesh();
        let result = evaluate_volume(&[mesh], "Volume");
        assert!(result.is_ok());
        let outputs = result.unwrap();
        assert!(outputs.contains_key(PIN_OUTPUT_VOLUME));
    }

    // ========================================================================
    // Test: BrepClosestPoint accepts Mesh
    // ========================================================================

    #[test]
    fn brep_closest_point_accepts_mesh() {
        let point = Value::Point([0.5, 0.5, 0.5]);
        let mesh = make_test_mesh();
        let result = evaluate_brep_closest_point(&[point, mesh], false);
        assert!(result.is_ok());
    }

    // ========================================================================
    // Test: IsPlanar accepts Mesh
    // ========================================================================

    #[test]
    fn is_planar_accepts_mesh() {
        let mesh = make_test_mesh();
        let result = evaluate_is_planar(&[mesh]);
        assert!(result.is_ok());
    }

    // ========================================================================
    // Test: Surface-parameterization-only components reject Mesh
    // ========================================================================
    // Test: Surface-parameterization components now accept grid-structured meshes
    // These tests verify the improved behavior that uses proper surface analysis
    // when a vertex grid can be inferred from the mesh structure.
    // ========================================================================

    /// Creates a mesh with irregular topology that can't form a valid grid.
    fn make_irregular_mesh() -> Value {
        // 5 vertices - prime number, can't form a valid grid
        Value::Mesh {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
                [0.25, 0.75, 0.5],
            ],
            indices: vec![0, 1, 2, 0, 1, 3, 0, 2, 3, 1, 2, 3, 0, 4, 1],
            normals: None,
            uvs: None,
            diagnostics: None,
        }
    }

    #[test]
    fn evaluate_surface_accepts_grid_mesh() {
        // The test mesh has 4 vertices which can form a 2x2 grid
        let mesh = make_test_mesh();
        let result = evaluate_surface_sample_component(&[mesh]);
        // Should succeed - the mesh can be treated as a grid surface
        assert!(result.is_ok());
        let outputs = result.unwrap();
        assert!(outputs.contains_key(PIN_OUTPUT_POINTS));
        assert!(outputs.contains_key(PIN_OUTPUT_NORMAL));
        assert!(outputs.contains_key(PIN_OUTPUT_U_DIRECTION));
        assert!(outputs.contains_key(PIN_OUTPUT_V_DIRECTION));
    }

    #[test]
    fn evaluate_surface_rejects_irregular_mesh() {
        let mesh = make_irregular_mesh();
        let result = evaluate_surface_sample_component(&[mesh]);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("parametric surface"));
    }

    #[test]
    fn principal_curvature_accepts_grid_mesh() {
        let mesh = make_test_mesh();
        let result = evaluate_principal_curvature(&[mesh]);
        // Should succeed - the mesh can be treated as a grid surface
        assert!(result.is_ok());
        let outputs = result.unwrap();
        assert!(outputs.contains_key("Maximum"));
        assert!(outputs.contains_key("Minimum"));
        assert!(outputs.contains_key("K¹"));
        assert!(outputs.contains_key("K²"));
    }

    #[test]
    fn principal_curvature_rejects_irregular_mesh() {
        let mesh = make_irregular_mesh();
        let result = evaluate_principal_curvature(&[mesh]);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("parametric surface"));
    }

    #[test]
    fn surface_curvature_accepts_grid_mesh() {
        let mesh = make_test_mesh();
        let result = evaluate_surface_curvature(&[mesh]);
        // Should succeed - the mesh can be treated as a grid surface
        assert!(result.is_ok());
        let outputs = result.unwrap();
        assert!(outputs.contains_key("Gaussian"));
        assert!(outputs.contains_key("Mean"));
    }

    #[test]
    fn surface_curvature_rejects_irregular_mesh() {
        let mesh = make_irregular_mesh();
        let result = evaluate_surface_curvature(&[mesh]);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("parametric surface"));
    }

    #[test]
    fn osculating_circles_rejects_mesh() {
        let mesh = make_test_mesh();
        let result = evaluate_osculating_circles(&[mesh]);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("parametric surface"));
    }

    #[test]
    fn point_in_trim_rejects_mesh() {
        let mesh = make_test_mesh();
        let uv_point = Value::Point([0.5, 0.5, 0.0]);
        let result = evaluate_point_in_trim(&[mesh, uv_point]);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("parametric surface"));
    }

    // ========================================================================
    // Test: Surface-parameterization components still accept Surface
    // ========================================================================

    #[test]
    fn evaluate_surface_accepts_surface() {
        let surface = make_test_surface();
        let result = evaluate_surface_sample_component(&[surface]);
        assert!(result.is_ok());
    }

    #[test]
    fn surface_curvature_accepts_surface() {
        let surface = make_test_surface();
        let result = evaluate_surface_curvature(&[surface]);
        assert!(result.is_ok());
    }

    #[test]
    fn osculating_circles_accepts_surface() {
        let surface = make_test_surface();
        let result = evaluate_osculating_circles(&[surface]);
        assert!(result.is_ok());
    }

    // ========================================================================
    // Test: MeshMetrics creation and basic properties
    // ========================================================================

    #[test]
    fn mesh_metrics_from_mesh_value() {
        let mesh = make_test_mesh();
        let metrics = MeshMetrics::from_value(Some(&mesh));
        assert!(metrics.is_some());
        let m = metrics.unwrap();
        assert_eq!(m.vertices.len(), 4);
        assert_eq!(m.indices.len(), 12); // 4 triangles × 3 indices
        assert_eq!(m.triangle_count(), 4);
    }

    #[test]
    fn mesh_metrics_from_surface_value() {
        let surface = make_test_surface();
        let metrics = MeshMetrics::from_value(Some(&surface));
        assert!(metrics.is_some());
        let m = metrics.unwrap();
        assert_eq!(m.vertices.len(), 3);
        assert_eq!(m.triangle_count(), 1);
    }

    #[test]
    fn mesh_metrics_from_none_returns_none() {
        assert!(MeshMetrics::from_value(None).is_none());
    }

    #[test]
    fn mesh_metrics_from_point_returns_none() {
        let point = Value::Point([1.0, 2.0, 3.0]);
        assert!(MeshMetrics::from_value(Some(&point)).is_none());
    }

    // ========================================================================
    // Test: Triangle helper functions
    // ========================================================================

    #[test]
    fn test_triangle_area_unit_right_triangle() {
        // Right triangle with legs of length 1 (area = 0.5)
        let v0 = [0.0, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [0.0, 1.0, 0.0];
        let area = triangle_area(v0, v1, v2);
        assert!((area - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_triangle_area_equilateral() {
        // Equilateral triangle with side length 2 (area = √3)
        let v0 = [0.0, 0.0, 0.0];
        let v1 = [2.0, 0.0, 0.0];
        let v2 = [1.0, 3.0_f64.sqrt(), 0.0];
        let area = triangle_area(v0, v1, v2);
        let expected = 3.0_f64.sqrt();
        assert!((area - expected).abs() < 1e-10);
    }

    #[test]
    fn test_signed_tetrahedron_volume() {
        // Unit tetrahedron with vertices at origin and unit axes
        // Volume = 1/6
        let v0 = [1.0, 0.0, 0.0];
        let v1 = [0.0, 1.0, 0.0];
        let v2 = [0.0, 0.0, 1.0];
        let vol = signed_tetrahedron_volume(v0, v1, v2);
        assert!((vol - 1.0 / 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_triangle_centroid() {
        let v0 = [0.0, 0.0, 0.0];
        let v1 = [3.0, 0.0, 0.0];
        let v2 = [0.0, 3.0, 0.0];
        let centroid = triangle_centroid(v0, v1, v2);
        assert!((centroid[0] - 1.0).abs() < 1e-10);
        assert!((centroid[1] - 1.0).abs() < 1e-10);
        assert!((centroid[2] - 0.0).abs() < 1e-10);
    }

    // ========================================================================
    // Test: MeshMetrics area computation (not AABB-based)
    // ========================================================================

    #[test]
    fn mesh_metrics_area_single_triangle() {
        // Single right triangle with legs 1,1 (area = 0.5)
        let mesh = Value::Mesh {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            indices: vec![0, 1, 2],
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        let mut metrics = MeshMetrics::from_value(Some(&mesh)).unwrap();
        let area = metrics.area();
        assert!((area - 0.5).abs() < 1e-10, "Expected area 0.5, got {}", area);
    }

    #[test]
    fn mesh_metrics_area_unit_cube() {
        // Unit cube: 6 faces × 2 triangles each = 12 triangles
        // Total surface area = 6.0
        let mesh = make_unit_cube_mesh();
        let mut metrics = MeshMetrics::from_value(Some(&mesh)).unwrap();
        let area = metrics.area();
        assert!(
            (area - 6.0).abs() < 1e-10,
            "Expected unit cube surface area 6.0, got {}",
            area
        );
    }

    #[test]
    fn mesh_metrics_area_differs_from_aabb() {
        // Create a tetrahedron - its surface area differs from AABB surface area
        let mesh = make_test_mesh();
        let mut mesh_metrics = MeshMetrics::from_value(Some(&mesh)).unwrap();
        let shape_metrics = ShapeMetrics::from_inputs(Some(&mesh)).unwrap();

        let mesh_area = mesh_metrics.area();
        let aabb_area = shape_metrics.area();

        // They should NOT be equal for a tetrahedron
        assert!(
            (mesh_area - aabb_area).abs() > 0.01,
            "Mesh area ({}) should differ from AABB area ({}) for tetrahedron",
            mesh_area,
            aabb_area
        );
    }

    // ========================================================================
    // Test: MeshMetrics volume computation (not AABB-based)
    // ========================================================================

    #[test]
    fn mesh_metrics_volume_unit_cube() {
        // Unit cube: volume = 1.0
        let mesh = make_unit_cube_mesh();
        let mut metrics = MeshMetrics::from_value(Some(&mesh)).unwrap();
        let volume = metrics.volume();
        assert!(
            (volume - 1.0).abs() < 1e-10,
            "Expected unit cube volume 1.0, got {}",
            volume
        );
    }

    #[test]
    fn mesh_metrics_volume_tetrahedron() {
        // Regular tetrahedron at origin with vertices at:
        // (0,0,0), (1,0,0), (0,1,0), (0,0,1)
        // Volume = 1/6
        let mesh = Value::Mesh {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            indices: vec![
                0, 2, 1, // bottom face (CCW from above)
                0, 1, 3, // front face
                0, 3, 2, // left face
                1, 2, 3, // back face
            ],
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        let mut metrics = MeshMetrics::from_value(Some(&mesh)).unwrap();
        let volume = metrics.volume();
        assert!(
            (volume - 1.0 / 6.0).abs() < 1e-10,
            "Expected tetrahedron volume 1/6, got {}",
            volume
        );
    }

    #[test]
    fn mesh_metrics_volume_differs_from_aabb() {
        // Create a tetrahedron - its volume differs from AABB volume
        let mesh = make_test_mesh();
        let mut mesh_metrics = MeshMetrics::from_value(Some(&mesh)).unwrap();
        let shape_metrics = ShapeMetrics::from_inputs(Some(&mesh)).unwrap();

        let mesh_volume = mesh_metrics.volume();
        let aabb_volume = shape_metrics.volume();

        // They should NOT be equal for a tetrahedron
        // AABB volume = 1.0 × 1.0 × 1.0 = 1.0
        // Tetrahedron volume is much smaller
        assert!(
            mesh_volume < aabb_volume * 0.5,
            "Mesh volume ({}) should be much less than AABB volume ({}) for tetrahedron",
            mesh_volume,
            aabb_volume
        );
    }

    // ========================================================================
    // Test: MeshMetrics centroid computation
    // ========================================================================

    #[test]
    fn mesh_metrics_centroid_unit_cube() {
        let mesh = make_unit_cube_mesh();
        let mut metrics = MeshMetrics::from_value(Some(&mesh)).unwrap();
        let centroid = metrics.centroid();
        // Center of unit cube from (0,0,0) to (1,1,1) is (0.5, 0.5, 0.5)
        assert!(
            (centroid[0] - 0.5).abs() < 1e-10,
            "Expected centroid x=0.5, got {}",
            centroid[0]
        );
        assert!(
            (centroid[1] - 0.5).abs() < 1e-10,
            "Expected centroid y=0.5, got {}",
            centroid[1]
        );
        assert!(
            (centroid[2] - 0.5).abs() < 1e-10,
            "Expected centroid z=0.5, got {}",
            centroid[2]
        );
    }

    // ========================================================================
    // Test: Evaluation functions use MeshMetrics for meshes
    // ========================================================================

    #[test]
    fn evaluate_area_uses_mesh_metrics() {
        let mesh = make_unit_cube_mesh();
        let result = evaluate_area(&[mesh], "Area").unwrap();
        let area = match result.get(PIN_OUTPUT_AREA) {
            Some(Value::Number(n)) => *n,
            _ => panic!("Expected area output"),
        };
        // Unit cube surface area is 6.0
        assert!(
            (area - 6.0).abs() < 1e-10,
            "evaluate_area should return 6.0 for unit cube, got {}",
            area
        );
    }

    #[test]
    fn evaluate_volume_uses_mesh_metrics() {
        let mesh = make_unit_cube_mesh();
        let result = evaluate_volume(&[mesh], "Volume").unwrap();
        let volume = match result.get(PIN_OUTPUT_VOLUME) {
            Some(Value::Number(n)) => *n,
            _ => panic!("Expected volume output"),
        };
        // Unit cube volume is 1.0
        assert!(
            (volume - 1.0).abs() < 1e-10,
            "evaluate_volume should return 1.0 for unit cube, got {}",
            volume
        );
    }

    #[test]
    fn evaluate_area_moments_uses_mesh_metrics() {
        let mesh = make_unit_cube_mesh();
        let result = evaluate_area_moments(&[mesh], "Area Moments").unwrap();
        let area = match result.get(PIN_OUTPUT_AREA) {
            Some(Value::Number(n)) => *n,
            _ => panic!("Expected area output"),
        };
        assert!(
            (area - 6.0).abs() < 1e-10,
            "evaluate_area_moments should return area 6.0 for unit cube, got {}",
            area
        );
    }

    #[test]
    fn evaluate_volume_moments_uses_mesh_metrics() {
        let mesh = make_unit_cube_mesh();
        let result = evaluate_volume_moments(&[mesh], "Volume Moments").unwrap();
        let volume = match result.get(PIN_OUTPUT_VOLUME) {
            Some(Value::Number(n)) => *n,
            _ => panic!("Expected volume output"),
        };
        assert!(
            (volume - 1.0).abs() < 1e-10,
            "evaluate_volume_moments should return volume 1.0 for unit cube, got {}",
            volume
        );
    }

    // ========================================================================
    // Test: Value::Surface also uses MeshMetrics
    // ========================================================================

    #[test]
    fn evaluate_area_uses_mesh_metrics_for_surface() {
        // Single triangle surface with area 0.5
        let surface = Value::Surface {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            faces: vec![vec![0, 1, 2]],
        };
        let result = evaluate_area(&[surface], "Area").unwrap();
        let area = match result.get(PIN_OUTPUT_AREA) {
            Some(Value::Number(n)) => *n,
            _ => panic!("Expected area output"),
        };
        assert!(
            (area - 0.5).abs() < 1e-10,
            "Surface area should be 0.5, got {}",
            area
        );
    }

    // ========================================================================
    // Helper: Create a unit cube mesh
    // ========================================================================

    /// Creates a unit cube mesh from (0,0,0) to (1,1,1).
    /// 8 vertices, 12 triangles (2 per face), 36 indices.
    fn make_unit_cube_mesh() -> Value {
        let vertices = vec![
            // Front face (z=0)
            [0.0, 0.0, 0.0], // 0: bottom-left
            [1.0, 0.0, 0.0], // 1: bottom-right
            [1.0, 1.0, 0.0], // 2: top-right
            [0.0, 1.0, 0.0], // 3: top-left
            // Back face (z=1)
            [0.0, 0.0, 1.0], // 4: bottom-left
            [1.0, 0.0, 1.0], // 5: bottom-right
            [1.0, 1.0, 1.0], // 6: top-right
            [0.0, 1.0, 1.0], // 7: top-left
        ];

        // 12 triangles (2 per face, CCW winding when viewed from outside)
        let indices = vec![
            // Front face (z=0), viewed from negative z
            0, 2, 1, 0, 3, 2,
            // Back face (z=1), viewed from positive z
            4, 5, 6, 4, 6, 7,
            // Left face (x=0), viewed from negative x
            0, 4, 7, 0, 7, 3,
            // Right face (x=1), viewed from positive x
            1, 2, 6, 1, 6, 5,
            // Bottom face (y=0), viewed from negative y
            0, 1, 5, 0, 5, 4,
            // Top face (y=1), viewed from positive y
            3, 7, 6, 3, 6, 2,
        ];

        Value::Mesh {
            vertices,
            indices,
            normals: None,
            uvs: None,
            diagnostics: None,
        }
    }

    // ========================================================================
    // Test: Proper closest point on mesh (not AABB clamping)
    // ========================================================================

    #[test]
    fn closest_point_on_triangle_mesh_basic() {
        // A simple triangle in the XY plane: (0,0,0), (1,0,0), (0.5,1,0)
        let mesh = Value::Mesh {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
            ],
            indices: vec![0, 1, 2],
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        
        // Query point above the triangle centroid
        let target = [0.5, 0.33, 1.0];
        let result = try_mesh_closest_point(Some(&mesh), target);
        assert!(result.is_some(), "Should find closest point on mesh");
        
        let r = result.unwrap();
        // The closest point should be on the triangle (z ≈ 0)
        assert!((r.point[2]).abs() < 1e-10, "Closest point should be on z=0 plane");
        // Distance should be approximately 1.0 (height above triangle)
        assert!((r.distance - 1.0).abs() < 0.1, "Distance should be approximately 1.0");
    }

    #[test]
    fn closest_point_on_rotated_mesh() {
        // A triangle rotated 45 degrees in the XZ plane
        // The triangle plane passes through origin with normal pointing in -Y+Z direction
        let s = std::f64::consts::FRAC_1_SQRT_2;
        let mesh = Value::Mesh {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, -s, s], // Rotated (0.5, 0, 1) around X axis
            ],
            indices: vec![0, 1, 2],
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        
        // Query point at origin - should be on the triangle (vertex 0)
        let target = [0.0, 0.0, 0.0];
        let result = try_mesh_closest_point(Some(&mesh), target);
        assert!(result.is_some());
        
        let r = result.unwrap();
        // Should be at vertex 0
        assert!((r.point[0]).abs() < 1e-10);
        assert!((r.point[1]).abs() < 1e-10);
        assert!((r.point[2]).abs() < 1e-10);
        assert!(r.distance < 1e-10, "Distance should be zero");
    }

    #[test]
    fn closest_point_on_concave_mesh() {
        // L-shaped mesh (concave when viewed from above)
        // This tests that we don't just use AABB clamping
        let mesh = Value::Mesh {
            vertices: vec![
                [0.0, 0.0, 0.0], // 0
                [1.0, 0.0, 0.0], // 1
                [1.0, 0.5, 0.0], // 2
                [0.5, 0.5, 0.0], // 3
                [0.5, 1.0, 0.0], // 4
                [0.0, 1.0, 0.0], // 5
            ],
            indices: vec![
                0, 1, 2,
                0, 2, 3,
                0, 3, 5,
                3, 4, 5,
            ],
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        
        // Query point in the "missing" corner of the L-shape
        // AABB would clamp to (0.75, 0.75, 0.0), but actual closest point is different
        let target = [0.75, 0.75, 0.0];
        let result = try_mesh_closest_point(Some(&mesh), target);
        assert!(result.is_some());
        
        let r = result.unwrap();
        // The closest point should NOT be at (0.75, 0.75, 0.0) - that's in the empty corner
        // It should be on one of the L-shape edges
        let dist_from_query = ((r.point[0] - 0.75).powi(2) + 
                              (r.point[1] - 0.75).powi(2) + 
                              (r.point[2]).powi(2)).sqrt();
        // Should be > 0 because (0.75, 0.75) is not on the L-shape
        assert!(dist_from_query > 0.1, "Closest point should not be at the query point");
    }

    // ========================================================================
    // Test: Proper point-in-brep (ray casting, not AABB containment)
    // ========================================================================

    #[test]
    fn point_in_mesh_cube_inside() {
        let cube = make_unit_cube_mesh();
        let inside_point = [0.5, 0.5, 0.5];
        
        let result = try_mesh_point_containment(Some(&cube), inside_point, false);
        assert!(result.is_some(), "Should be able to test containment for cube mesh");
        assert!(result.unwrap(), "Point at cube center should be inside");
    }

    #[test]
    fn point_in_mesh_cube_outside() {
        let cube = make_unit_cube_mesh();
        let outside_point = [2.0, 0.5, 0.5];
        
        let result = try_mesh_point_containment(Some(&cube), outside_point, false);
        assert!(result.is_some(), "Should be able to test containment for cube mesh");
        assert!(!result.unwrap(), "Point outside cube should be outside");
    }

    #[test]
    fn point_in_concave_mesh_in_cavity() {
        // L-shaped mesh (like a concave 2D shape extruded)
        // Points in the cavity should be outside
        
        // For simplicity, we test with a simple triangle that forms a "bowl"
        // The point "above" the center but outside the triangle should be outside
        let mesh = Value::Mesh {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
            ],
            indices: vec![0, 1, 2],
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        
        // This is an open surface (single triangle), so containment is indeterminate
        // The fallback should return None for open surfaces
        let result = try_mesh_point_containment(Some(&mesh), [0.5, 0.5, 1.0], false);
        // For open surfaces, the result may be None (indeterminate) or Outside
        if let Some(is_inside) = result {
            assert!(!is_inside, "Point above open surface should not be inside");
        }
    }

    #[test]
    fn brep_closest_point_uses_mesh_algorithm() {
        // Test that brep closest point uses the proper mesh algorithm
        let mesh = make_unit_cube_mesh();
        let point = Value::Point([0.5, 0.5, 2.0]); // Above the cube
        
        let result = evaluate_brep_closest_point(&[point, mesh], false);
        assert!(result.is_ok());
        
        let outputs = result.unwrap();
        let closest = match outputs.get(PIN_OUTPUT_POINTS) {
            Some(Value::Point(p)) => *p,
            _ => panic!("Expected point output"),
        };
        let distance = match outputs.get(PIN_OUTPUT_DISTANCE) {
            Some(Value::Number(d)) => *d,
            _ => panic!("Expected distance output"),
        };
        
        // Closest point should be on top face of cube (z ≈ 1.0)
        assert!((closest[2] - 1.0).abs() < 0.1, "Closest point should be on top face, got z={}", closest[2]);
        // Distance should be approximately 1.0
        assert!((distance - 1.0).abs() < 0.1, "Distance should be approximately 1.0, got {}", distance);
    }

    #[test]
    fn point_in_brep_uses_mesh_algorithm() {
        // Test that point in brep uses proper mesh algorithm
        let cube = make_unit_cube_mesh();
        let inside_point = Value::Point([0.5, 0.5, 0.5]);
        let outside_point = Value::Point([2.0, 0.5, 0.5]);
        
        // Test inside point
        let result_inside = evaluate_point_in_brep(&[cube.clone(), inside_point, Value::Boolean(false)]);
        assert!(result_inside.is_ok());
        let outputs_inside = result_inside.unwrap();
        let is_inside = match outputs_inside.get(PIN_OUTPUT_INSIDE) {
            Some(Value::Boolean(b)) => *b,
            _ => panic!("Expected boolean output"),
        };
        assert!(is_inside, "Point at cube center should be inside");
        
        // Test outside point
        let result_outside = evaluate_point_in_brep(&[cube, outside_point, Value::Boolean(false)]);
        assert!(result_outside.is_ok());
        let outputs_outside = result_outside.unwrap();
        let is_outside = match outputs_outside.get(PIN_OUTPUT_INSIDE) {
            Some(Value::Boolean(b)) => *b,
            _ => panic!("Expected boolean output"),
        };
        assert!(!is_outside, "Point outside cube should be outside");
    }

    // ========================================================================
    // Test: MeshTopology extraction
    // ========================================================================

    #[test]
    fn mesh_topology_from_mesh_value() {
        let mesh = make_test_mesh();
        let topology = MeshTopology::from_value(Some(&mesh));
        assert!(topology.is_some(), "Should create topology from mesh");
        let t = topology.unwrap();
        // Tetrahedron has 4 faces (triangles) and 6 unique edges
        assert_eq!(t.faces.len(), 4, "Tetrahedron should have 4 faces");
        assert_eq!(t.edges.len(), 6, "Tetrahedron should have 6 unique edges");
    }

    #[test]
    fn mesh_topology_from_surface_value() {
        let surface = make_test_surface();
        let topology = MeshTopology::from_value(Some(&surface));
        assert!(topology.is_some(), "Should create topology from surface");
        let t = topology.unwrap();
        // Single triangle has 1 face and 3 edges
        assert_eq!(t.faces.len(), 1, "Single triangle should have 1 face");
        assert_eq!(t.edges.len(), 3, "Single triangle should have 3 edges");
    }

    #[test]
    fn mesh_topology_unit_cube() {
        let cube = make_unit_cube_mesh();
        let topology = MeshTopology::from_value(Some(&cube));
        assert!(topology.is_some(), "Should create topology from cube");
        let t = topology.unwrap();
        // Cube has 12 triangular faces and 18 unique edges (6 faces × 2 tris × 3 edges / 2 shared)
        assert_eq!(t.faces.len(), 12, "Cube mesh should have 12 triangular faces");
        // A properly triangulated cube should have all interior edges
    }

    #[test]
    fn mesh_topology_from_none_returns_none() {
        assert!(MeshTopology::from_value(None).is_none());
    }

    #[test]
    fn mesh_topology_from_point_returns_none() {
        let point = Value::Point([1.0, 2.0, 3.0]);
        assert!(MeshTopology::from_value(Some(&point)).is_none());
    }

    // ========================================================================
    // Test: Edge classification
    // ========================================================================

    #[test]
    fn edge_classification_open_triangle() {
        // Single open triangle: all 3 edges should be naked (boundary)
        let mesh = Value::Mesh {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
            ],
            indices: vec![0, 1, 2],
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        let topology = MeshTopology::from_value(Some(&mesh)).unwrap();
        let naked = topology.naked_edges();
        let interior = topology.interior_edges();
        let non_manifold = topology.non_manifold_edges();
        
        assert_eq!(naked.len(), 3, "Open triangle should have 3 naked edges");
        assert_eq!(interior.len(), 0, "Open triangle should have no interior edges");
        assert_eq!(non_manifold.len(), 0, "Open triangle should have no non-manifold edges");
    }

    #[test]
    fn edge_classification_closed_tetrahedron() {
        // Closed tetrahedron: all edges should be interior (shared by 2 faces)
        let mesh = Value::Mesh {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            indices: vec![
                0, 1, 2, // bottom
                0, 1, 3, // front
                1, 2, 3, // right
                0, 2, 3, // left
            ],
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        let topology = MeshTopology::from_value(Some(&mesh)).unwrap();
        let naked = topology.naked_edges();
        let interior = topology.interior_edges();
        let non_manifold = topology.non_manifold_edges();
        
        assert_eq!(naked.len(), 0, "Closed tetrahedron should have no naked edges");
        assert_eq!(interior.len(), 6, "Closed tetrahedron should have 6 interior edges");
        assert_eq!(non_manifold.len(), 0, "Closed tetrahedron should have no non-manifold edges");
    }

    #[test]
    fn edge_classification_non_manifold_bowtie() {
        // Bowtie shape: two triangles sharing a single vertex creates a non-manifold case
        // Actually, for non-manifold edges, we need 3+ faces sharing the same edge
        let mesh = Value::Mesh {
            vertices: vec![
                [0.0, 0.0, 0.0], // 0: shared vertex
                [1.0, 0.0, 0.0], // 1
                [0.5, 0.0, 1.0], // 2
                [0.5, 1.0, 0.5], // 3
            ],
            indices: vec![
                0, 1, 2, // face 1
                0, 1, 3, // face 2 - shares edge 0-1 with face 1
                0, 2, 3, // face 3 - shares edges 0-2 and 2-3
            ],
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        let topology = MeshTopology::from_value(Some(&mesh)).unwrap();
        // This is actually a valid closed surface (tetrahedron missing one face)
        // All edges are shared by at most 2 faces
        let non_manifold = topology.non_manifold_edges();
        assert!(non_manifold.is_empty() || non_manifold.len() <= 1, 
                "Should have few or no non-manifold edges");
    }

    // ========================================================================
    // Test: Face adjacency
    // ========================================================================

    #[test]
    fn face_adjacency_two_triangles() {
        // Two adjacent triangles sharing one edge
        let mesh = Value::Mesh {
            vertices: vec![
                [0.0, 0.0, 0.0], // 0
                [1.0, 0.0, 0.0], // 1
                [0.5, 1.0, 0.0], // 2
                [0.5, -1.0, 0.0], // 3
            ],
            indices: vec![
                0, 1, 2, // face 0
                0, 1, 3, // face 1 - shares edge 0-1 with face 0
            ],
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        let topology = MeshTopology::from_value(Some(&mesh)).unwrap();
        let adjacency = topology.get_face_face_adjacency();
        
        assert_eq!(adjacency.len(), 2, "Should have 2 faces");
        assert!(adjacency[0].contains(&1), "Face 0 should be adjacent to face 1");
        assert!(adjacency[1].contains(&0), "Face 1 should be adjacent to face 0");
    }

    #[test]
    fn face_edge_mapping_triangle() {
        let mesh = Value::Mesh {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
            ],
            indices: vec![0, 1, 2],
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        let topology = MeshTopology::from_value(Some(&mesh)).unwrap();
        let face_edges = topology.get_face_edge_mapping();
        
        assert_eq!(face_edges.len(), 1, "Should have 1 face");
        assert_eq!(face_edges[0].len(), 3, "Triangle face should reference 3 edges");
    }

    #[test]
    fn edge_face_mapping_shared_edge() {
        // Two triangles sharing one edge
        let mesh = Value::Mesh {
            vertices: vec![
                [0.0, 0.0, 0.0], // 0
                [1.0, 0.0, 0.0], // 1
                [0.5, 1.0, 0.0], // 2
                [0.5, -1.0, 0.0], // 3
            ],
            indices: vec![
                0, 1, 2, // face 0
                0, 1, 3, // face 1
            ],
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        let topology = MeshTopology::from_value(Some(&mesh)).unwrap();
        let edge_faces = topology.get_edge_face_mapping();
        
        // Find the edge shared by both faces (edge 0-1)
        let shared_edge_count = edge_faces.iter()
            .filter(|faces| faces.len() == 2 && faces.contains(&0) && faces.contains(&1))
            .count();
        assert_eq!(shared_edge_count, 1, "Should have exactly one shared edge between the two faces");
    }

    // ========================================================================
    // Test: Brep Edges component uses mesh topology
    // ========================================================================

    #[test]
    fn brep_edges_returns_actual_mesh_edges() {
        // Open triangle: all edges should be naked
        let mesh = Value::Mesh {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
            ],
            indices: vec![0, 1, 2],
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        
        let result = evaluate_brep_edges(&[mesh]).unwrap();
        
        let naked = match result.get(PIN_OUTPUT_NAKED_EDGES) {
            Some(Value::List(l)) => l,
            _ => panic!("Expected naked edges list"),
        };
        let interior = match result.get(PIN_OUTPUT_INTERIOR_EDGES) {
            Some(Value::List(l)) => l,
            _ => panic!("Expected interior edges list"),
        };
        let non_manifold = match result.get(PIN_OUTPUT_NON_MANIFOLD_EDGES) {
            Some(Value::List(l)) => l,
            _ => panic!("Expected non-manifold edges list"),
        };
        
        assert_eq!(naked.len(), 3, "Open triangle should have 3 naked edges");
        assert_eq!(interior.len(), 0, "Open triangle should have no interior edges");
        assert_eq!(non_manifold.len(), 0, "Open triangle should have no non-manifold edges");
    }

    #[test]
    fn brep_edges_closed_mesh_has_interior_edges() {
        let cube = make_unit_cube_mesh();
        let result = evaluate_brep_edges(&[cube]).unwrap();
        
        let interior = match result.get(PIN_OUTPUT_INTERIOR_EDGES) {
            Some(Value::List(l)) => l,
            _ => panic!("Expected interior edges list"),
        };
        
        // A properly triangulated cube should have interior edges (shared between triangles)
        assert!(!interior.is_empty(), "Closed cube mesh should have interior edges");
    }

    // ========================================================================
    // Test: Brep Wireframe component uses mesh topology
    // ========================================================================

    #[test]
    fn brep_wireframe_returns_actual_mesh_edges() {
        let mesh = Value::Mesh {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
            ],
            indices: vec![0, 1, 2],
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        
        let result = evaluate_brep_wireframe(&[mesh]).unwrap();
        
        let wireframe = match result.get(PIN_OUTPUT_WIREFRAME) {
            Some(Value::List(l)) => l,
            _ => panic!("Expected wireframe list"),
        };
        
        assert_eq!(wireframe.len(), 3, "Triangle should have 3 edges in wireframe");
    }

    // ========================================================================
    // Test: Deconstruct Brep component uses mesh topology
    // ========================================================================

    #[test]
    fn deconstruct_brep_returns_actual_mesh_data() {
        let mesh = Value::Mesh {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
            ],
            indices: vec![0, 1, 2],
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        
        let result = evaluate_deconstruct_brep(&[mesh]).unwrap();
        
        let faces = match result.get(PIN_OUTPUT_FACES) {
            Some(Value::List(l)) => l,
            _ => panic!("Expected faces list"),
        };
        let edges = match result.get(PIN_OUTPUT_EDGES) {
            Some(Value::List(l)) => l,
            _ => panic!("Expected edges list"),
        };
        let vertices = match result.get(PIN_OUTPUT_VERTICES) {
            Some(Value::List(l)) => l,
            _ => panic!("Expected vertices list"),
        };
        
        assert_eq!(faces.len(), 1, "Single triangle mesh should have 1 face");
        assert_eq!(edges.len(), 3, "Single triangle mesh should have 3 edges");
        assert_eq!(vertices.len(), 3, "Single triangle mesh should have 3 vertices");
    }

    #[test]
    fn deconstruct_brep_cube_has_correct_counts() {
        let cube = make_unit_cube_mesh();
        let result = evaluate_deconstruct_brep(&[cube]).unwrap();
        
        let faces = match result.get(PIN_OUTPUT_FACES) {
            Some(Value::List(l)) => l,
            _ => panic!("Expected faces list"),
        };
        let edges = match result.get(PIN_OUTPUT_EDGES) {
            Some(Value::List(l)) => l,
            _ => panic!("Expected edges list"),
        };
        let vertices = match result.get(PIN_OUTPUT_VERTICES) {
            Some(Value::List(l)) => l,
            _ => panic!("Expected vertices list"),
        };
        
        assert_eq!(faces.len(), 12, "Cube mesh should have 12 triangular faces");
        assert_eq!(vertices.len(), 8, "Cube mesh should have 8 vertices");
        // Cube has 18 unique edges (12 on outer edges + 6 diagonal edges from triangulation)
        assert!(edges.len() >= 12, "Cube mesh should have at least 12 edges");
    }

    // ========================================================================
    // Test: Brep Topology component uses mesh topology
    // ========================================================================

    #[test]
    fn brep_topology_returns_actual_adjacency() {
        // Two adjacent triangles
        let mesh = Value::Mesh {
            vertices: vec![
                [0.0, 0.0, 0.0], // 0
                [1.0, 0.0, 0.0], // 1
                [0.5, 1.0, 0.0], // 2
                [0.5, -1.0, 0.0], // 3
            ],
            indices: vec![
                0, 1, 2, // face 0
                0, 1, 3, // face 1
            ],
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        
        let result = evaluate_brep_topology(&[mesh]).unwrap();
        
        let face_face = match result.get(PIN_OUTPUT_FACE_FACE) {
            Some(Value::List(l)) => l,
            _ => panic!("Expected face-face adjacency list"),
        };
        let face_edge = match result.get(PIN_OUTPUT_FACE_EDGE) {
            Some(Value::List(l)) => l,
            _ => panic!("Expected face-edge mapping list"),
        };
        let edge_face = match result.get(PIN_OUTPUT_EDGE_FACE) {
            Some(Value::List(l)) => l,
            _ => panic!("Expected edge-face mapping list"),
        };
        
        assert_eq!(face_face.len(), 2, "Should have face-face adjacency for 2 faces");
        assert_eq!(face_edge.len(), 2, "Should have face-edge mapping for 2 faces");
        assert!(!edge_face.is_empty(), "Should have edge-face mapping");
        
        // Check that face 0 is adjacent to face 1
        if let Value::List(adj) = &face_face[0] {
            let contains_1 = adj.iter().any(|v| matches!(v, Value::Number(n) if (*n - 1.0).abs() < EPSILON));
            assert!(contains_1, "Face 0 should be adjacent to face 1");
        }
    }

    #[test]
    fn brep_topology_single_triangle_no_adjacency() {
        // Single triangle: no face-to-face adjacency
        let mesh = Value::Mesh {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
            ],
            indices: vec![0, 1, 2],
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        
        let result = evaluate_brep_topology(&[mesh]).unwrap();
        
        let face_face = match result.get(PIN_OUTPUT_FACE_FACE) {
            Some(Value::List(l)) => l,
            _ => panic!("Expected face-face adjacency list"),
        };
        
        assert_eq!(face_face.len(), 1, "Should have 1 face");
        if let Value::List(adj) = &face_face[0] {
            assert!(adj.is_empty(), "Single triangle should have no adjacent faces");
        }
    }

    // ========================================================================
    // Test: Surface Points rejects Mesh inputs
    // ========================================================================

    #[test]
    fn surface_points_rejects_mesh_input() {
        // Surface Points requires parametric surface data for Greville points,
        // control point weights, and U/V counts. Meshes lack (u,v) parameterization.
        let mesh = make_test_mesh();
        let result = evaluate_surface_points(&[mesh]);
        assert!(result.is_err(), "Surface Points should reject mesh input");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("parametric surface"),
            "Error should mention parametric surface requirement: {}",
            err_msg
        );
        assert!(
            err_msg.contains("Surface Points"),
            "Error should mention component name: {}",
            err_msg
        );
    }

    #[test]
    fn surface_points_accepts_surface_input() {
        // Surface inputs should work (falling back to legacy AABB-based extraction)
        let surface = make_test_surface();
        let result = evaluate_surface_points(&[surface]);
        assert!(result.is_ok(), "Surface Points should accept surface input");
        let outputs = result.unwrap();
        assert!(outputs.contains_key(PIN_OUTPUT_POINTS));
        assert!(outputs.contains_key(PIN_OUTPUT_WEIGHTS));
        assert!(outputs.contains_key(PIN_OUTPUT_GREVILLE));
        assert!(outputs.contains_key(PIN_OUTPUT_U_COUNT));
        assert!(outputs.contains_key(PIN_OUTPUT_V_COUNT));
    }

    #[test]
    fn surface_points_rejects_mesh_in_list() {
        // A mesh wrapped in a single-element list should also be rejected
        let mesh_list = Value::List(vec![make_test_mesh()]);
        let result = evaluate_surface_points(&[mesh_list]);
        assert!(result.is_err(), "Surface Points should reject mesh wrapped in list");
    }

    // ========================================================================
    // Test: Dimensions rejects Mesh inputs
    // ========================================================================

    #[test]
    fn dimensions_rejects_mesh_input() {
        // Dimensions outputs U/V domain extents which require parametric surface data.
        // Meshes lack (u,v) parameterization; using AABB size is semantically wrong.
        let mesh = make_test_mesh();
        let result = evaluate_dimensions(&[mesh]);
        assert!(result.is_err(), "Dimensions should reject mesh input");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("parametric surface"),
            "Error should mention parametric surface requirement: {}",
            err_msg
        );
        assert!(
            err_msg.contains("Dimensions"),
            "Error should mention component name: {}",
            err_msg
        );
    }

    #[test]
    fn dimensions_accepts_surface_input() {
        // Surface inputs should work (legacy AABB-based size extraction)
        let surface = make_test_surface();
        let result = evaluate_dimensions(&[surface]);
        assert!(result.is_ok(), "Dimensions should accept surface input");
        let outputs = result.unwrap();
        assert!(outputs.contains_key("U"));
        assert!(outputs.contains_key("V"));
    }

    #[test]
    fn dimensions_rejects_mesh_in_list() {
        // A mesh wrapped in a single-element list should also be rejected
        let mesh_list = Value::List(vec![make_test_mesh()]);
        let result = evaluate_dimensions(&[mesh_list]);
        assert!(result.is_err(), "Dimensions should reject mesh wrapped in list");
    }
}
