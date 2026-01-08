//! Grasshopper components for mesh triangulation, delaunay, and voronoi operations.
//!
//! # Mesh Engine Integration (Phase 3)
//!
//! Mesh-producing components in this module now output `Value::Mesh` as the primary type.
//! This provides:
//! - Indexed triangle list with flat indices (divisible by 3)
//! - Optional per-vertex normals for smooth shading
//! - Mesh diagnostics with vertex/triangle counts
//!
//! Components that produce edges, polylines, or other non-mesh outputs are unchanged.
//! The legacy `Value::Surface` output is no longer emitted. Downstream consumers
//! should use `expect_mesh_like()` for backward compatibility with both types.

use std::collections::BTreeMap;

use crate::components::coerce;
use crate::graph::node::MetaMap;
use crate::graph::value::{MeshDiagnostics, Value};

use super::{Component, ComponentError, ComponentResult};

// ============================================================================
// Mesh Triangulation Helpers
// ============================================================================

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

/// Creates a `Value::Mesh` output from vertices and triangle indices.
///
/// This helper converts the raw triangulation output into a proper `Value::Mesh`
/// with smooth normals and diagnostics.
///
/// # Arguments
///
/// * `vertices` - Vertex positions (typically from input points)
/// * `indices` - Triangle indices from Delaunay triangulation (flat list, divisible by 3)
///
/// # Returns
///
/// A `Value::Mesh` with positions, triangle indices, smooth normals,
/// and diagnostics information.
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

#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    DelaunayMesh,
    FacetDome,
    QuadRemesh,
    Substrate,
    Proximity2D,
    VoronoiCell,
    QuadTree,
    TriRemesh,
    ConvexHull,
    VoronoiGroups,
    Voronoi,
    OcTree,
    Voronoi3D,
    MetaBallTCustom,
    MetaBallT,
    DelaunayEdges,
    MetaBall,
    Proximity3D,
}

impl ComponentKind {
    pub fn evaluate(self, inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        match self {
            Self::DelaunayMesh => DelaunayMesh.evaluate(inputs, meta),
            Self::FacetDome => FacetDome.evaluate(inputs, meta),
            Self::QuadRemesh => QuadRemesh.evaluate(inputs, meta),
            Self::Substrate => Substrate.evaluate(inputs, meta),
            Self::Proximity2D => Proximity2D.evaluate(inputs, meta),
            Self::VoronoiCell => VoronoiCell.evaluate(inputs, meta),
            Self::QuadTree => QuadTree.evaluate(inputs, meta),
            Self::TriRemesh => TriRemesh.evaluate(inputs, meta),
            Self::ConvexHull => ConvexHull.evaluate(inputs, meta),
            Self::VoronoiGroups => VoronoiGroups.evaluate(inputs, meta),
            Self::Voronoi => Voronoi.evaluate(inputs, meta),
            Self::OcTree => OcTree.evaluate(inputs, meta),
            Self::Voronoi3D => Voronoi3D.evaluate(inputs, meta),
            Self::MetaBallTCustom => MetaBallTCustom.evaluate(inputs, meta),
            Self::MetaBallT => MetaBallT.evaluate(inputs, meta),
            Self::DelaunayEdges => DelaunayEdges.evaluate(inputs, meta),
            Self::MetaBall => MetaBall.evaluate(inputs, meta),
            Self::Proximity3D => Proximity3D.evaluate(inputs, meta),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::DelaunayMesh => "Delaunay Mesh",
            Self::FacetDome => "Facet Dome",
            Self::QuadRemesh => "Quad Remesh",
            Self::Substrate => "Substrate",
            Self::Proximity2D => "Proximity 2D",
            Self::VoronoiCell => "Voronoi Cell",
            Self::QuadTree => "QuadTree",
            Self::TriRemesh => "TriRemesh",
            Self::ConvexHull => "Convex Hull",
            Self::VoronoiGroups => "Voronoi Groups",
            Self::Voronoi => "Voronoi",
            Self::OcTree => "OcTree",
            Self::Voronoi3D => "Voronoi 3D",
            Self::MetaBallTCustom => "MetaBall(t) Custom",
            Self::MetaBallT => "MetaBall(t)",
            Self::DelaunayEdges => "Delaunay Edges",
            Self::MetaBall => "MetaBall",
            Self::Proximity3D => "Proximity 3D",
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
        guids: &["1eb4f6ff-3547-4184-bead-1b01e7cfd668"],
        names: &["Delaunay Mesh", "Del"],
        kind: ComponentKind::DelaunayMesh,
    },
    Registration {
        guids: &["190c0070-8cbf-4347-94c2-d84bbb488d55"],
        names: &["Facet Dome", "Facet"],
        kind: ComponentKind::FacetDome,
    },
    Registration {
        guids: &["1a17d3f0-c8f8-4ee9-8dab-ea1c29db6a49"],
        names: &["Quad Remesh", "QRMesh"],
        kind: ComponentKind::QuadRemesh,
    },
    Registration {
        guids: &["415750fd-c0ec-4411-84d0-01f28ab23066"],
        names: &["Substrate"],
        kind: ComponentKind::Substrate,
    },
    Registration {
        guids: &["458ed0e0-19a3-419b-8ead-f524925b8a35"],
        names: &["Proximity 2D", "Prox"],
        kind: ComponentKind::Proximity2D,
    },
    Registration {
        guids: &["7b181be1-30e7-4a97-915a-1b461741aef8"],
        names: &["Voronoi Cell", "VCell"],
        kind: ComponentKind::VoronoiCell,
    },
    Registration {
        guids: &["8102032b-9699-4949-ab12-3017a31d1062"],
        names: &["QuadTree", "QT"],
        kind: ComponentKind::QuadTree,
    },
    Registration {
        guids: &["866222ee-6093-4af8-8944-2f9264885385"],
        names: &["TriRemesh"],
        kind: ComponentKind::TriRemesh,
    },
    Registration {
        guids: &["9d0c5284-ea24-4f9f-a183-ef57fc48b5b8"],
        names: &["Convex Hull", "Hull"],
        kind: ComponentKind::ConvexHull,
    },
    Registration {
        guids: &[
            "9d4854fe-70db-4863-967b-4120d0b6d2e4",
            "ab454a50-debf-46d1-9bd1-82648416a802",
        ],
        names: &["Voronoi Groups", "VorGroup"],
        kind: ComponentKind::VoronoiGroups,
    },
    Registration {
        guids: &[
            "a4011be0-1c91-45bd-8280-17dd3a9f46f1",
            "ee9261ab-75a4-478f-b483-a50b755b07fd",
        ],
        names: &["Voronoi"],
        kind: ComponentKind::Voronoi,
    },
    Registration {
        guids: &["a59a68ad-fdd6-41dd-88f0-d7a6fb8d2e16"],
        names: &["OcTree", "OcT"],
        kind: ComponentKind::OcTree,
    },
    Registration {
        guids: &["ba9bb57a-61cf-4207-a1c4-994e371ba4f9"],
        names: &["Voronoi 3D", "Voronoi³"],
        kind: ComponentKind::Voronoi3D,
    },
    Registration {
        guids: &["c4373505-a4cf-4992-8db1-fd6e6bb5850d"],
        names: &["MetaBall(t) Custom"],
        kind: ComponentKind::MetaBallTCustom,
    },
    Registration {
        guids: &["c48cf4d4-432c-41b6-b77a-77650479a31f"],
        names: &["MetaBall(t)"],
        kind: ComponentKind::MetaBallT,
    },
    Registration {
        guids: &["db2a4d25-23fa-4887-8983-ee5293cc82c0"],
        names: &["Delaunay Edges", "Con"],
        kind: ComponentKind::DelaunayEdges,
    },
    Registration {
        guids: &["dc934310-67eb-4d1d-8607-7cc62a501dd9"],
        names: &["MetaBall"],
        kind: ComponentKind::MetaBall,
    },
    Registration {
        guids: &["e504d619-4467-437a-92fa-c6822d16b066"],
        names: &["Proximity 3D", "Prox"],
        kind: ComponentKind::Proximity3D,
    },
];

macro_rules! placeholder_component {
    ($name:ident, $component_name:expr) => {
        #[derive(Debug, Default, Clone, Copy)]
        pub struct $name;

        impl Component for $name {
            fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
                Err(ComponentError::NotYetImplemented(
                    $component_name.to_string(),
                ))
            }
        }
    };
}

const OUTPUT_PATTERN: &str = "P";
const OUTPUT_DOME: &str = "D";

/// Component for creating a facetted dome from a set of 2D points.
///
/// Uses Delaunay triangulation to create a mesh from the input points.
/// The Z-coordinates of the input points are preserved in the output mesh.
///
/// # Inputs
///
/// * `P` (Points): List of points to triangulate
///
/// # Outputs
///
/// * `P` (Pattern): Edges of the triangulation as line curves
/// * `D` (Dome): The triangulated mesh as `Value::Mesh`
///
/// # Mesh Engine Integration
///
/// This component now outputs `Value::Mesh` with:
/// - Indexed triangle list (flat indices)
/// - Smooth per-vertex normals
/// - Mesh diagnostics with vertex/triangle counts
#[derive(Debug, Default, Clone, Copy)]
pub struct FacetDome;

impl Component for FacetDome {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new("Input `P` is missing"));
        }

        let points = match &inputs[0] {
            Value::List(list) => list
                .iter()
                .map(coerce::coerce_point)
                .collect::<Result<Vec<_>, _>>()?,
            _ => return Err(ComponentError::new("Input `P` is not a list")),
        };

        if points.len() < 3 {
            return Err(ComponentError::new("Not enough points for triangulation"));
        }

        // Project points to 2D for Delaunay triangulation
        let delaunator_points: Vec<delaunator::Point> = points
            .iter()
            .map(|p| delaunator::Point { x: p[0], y: p[1] })
            .collect();

        let triangulation = delaunator::triangulate(&delaunator_points);

        // Create the "Dome" mesh using Value::Mesh with proper normals and diagnostics
        let indices: Vec<u32> = triangulation
            .triangles
            .iter()
            .map(|&i| i as u32)
            .collect();

        let dome = create_mesh_from_triangles(points.clone(), indices);

        // Create the "Pattern" (Edges of the triangulation)
        let mut pattern_edges = Vec::new();
        for i in 0..triangulation.triangles.len() {
            let endpoint = triangulation.halfedges[i];
            if i < endpoint {
                let p1_idx = triangulation.triangles[i];
                let p2_idx = triangulation.triangles[delaunator::next_halfedge(i)];
                pattern_edges.push(Value::CurveLine {
                    p1: points[p1_idx],
                    p2: points[p2_idx],
                });
            }
        }

        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_PATTERN.to_owned(), Value::List(pattern_edges));
        outputs.insert(OUTPUT_DOME.to_owned(), dome);

        Ok(outputs)
    }
}

placeholder_component!(QuadRemesh, "Quad Remesh");
placeholder_component!(Substrate, "Substrate");

const OUTPUT_LINKS: &str = "L";
const OUTPUT_TOPOLOGY: &str = "T";

/// Component for finding 2D proximity within a point list.
#[derive(Debug, Default, Clone, Copy)]
pub struct Proximity2D;

impl Component for Proximity2D {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 1 {
            return Err(ComponentError::new("Input `P` is missing"));
        }

        let points = match &inputs[0] {
            Value::List(list) => list
                .iter()
                .map(coerce::coerce_point)
                .collect::<Result<Vec<_>, _>>()?,
            _ => return Err(ComponentError::new("Input `P` is not a list")),
        };
        // inputs[1] is Plane, which is ignored for now.
        let group = if inputs.len() > 2 && !matches!(&inputs[2], Value::Null) {
            coerce::coerce_integer(&inputs[2])? as usize
        } else {
            usize::MAX
        };
        let min_radius_sq = if inputs.len() > 3 && !matches!(&inputs[3], Value::Null) {
            let r = coerce::coerce_number(&inputs[3], None)?;
            r * r
        } else {
            0.0
        };
        let max_radius_sq = if inputs.len() > 4 && !matches!(&inputs[4], Value::Null) {
            let r = coerce::coerce_number(&inputs[4], None)?;
            r * r
        } else {
            f64::INFINITY
        };

        if points.is_empty() {
            return Ok(BTreeMap::new());
        }

        let mut links_tree = Vec::new();
        let mut topology_tree = Vec::new();

        for (i, p1) in points.iter().enumerate() {
            let mut neighbors = Vec::new();
            for (j, p2) in points.iter().enumerate() {
                if i == j {
                    continue;
                }

                let dist_sq = (p1[0] - p2[0]).powi(2) + (p1[1] - p2[1]).powi(2);

                if dist_sq >= min_radius_sq && dist_sq <= max_radius_sq {
                    neighbors.push((dist_sq, j, p2));
                }
            }

            neighbors.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            let mut links_branch = Vec::new();
            let mut topology_branch = Vec::new();

            for (_, j, p2) in neighbors.iter().take(group) {
                links_branch.push(Value::CurveLine { p1: *p1, p2: **p2 });
                topology_branch.push(Value::Text(format!("{{{};{}}}", i, j)));
            }

            links_tree.push(Value::List(links_branch));
            topology_tree.push(Value::List(topology_branch));
        }

        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_LINKS.to_owned(), Value::List(links_tree));
        outputs.insert(OUTPUT_TOPOLOGY.to_owned(), Value::List(topology_tree));

        Ok(outputs)
    }
}

placeholder_component!(VoronoiCell, "Voronoi Cell");
placeholder_component!(QuadTree, "QuadTree");
placeholder_component!(TriRemesh, "TriRemesh");

const OUTPUT_HULL: &str = "H";
const OUTPUT_HULL_Z: &str = "Hz";
const OUTPUT_INDICES: &str = "I";

/// Component for creating a convex hull from a set of points.
#[derive(Debug, Default, Clone, Copy)]
pub struct ConvexHull;

impl Component for ConvexHull {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new("Input `P` is missing"));
        }

        let points_value = &inputs[0];
        let points_list = match points_value {
            Value::List(list) => list,
            _ => return Err(ComponentError::new("Input `P` is not a list")),
        };

        let mut vertices = Vec::with_capacity(points_list.len());
        let mut delaunator_points = Vec::with_capacity(points_list.len());

        for value in points_list {
            let point = coerce::coerce_point(value)?;
            vertices.push(point);
            delaunator_points.push(delaunator::Point {
                x: point[0],
                y: point[1],
            });
        }

        if vertices.len() < 3 {
            return Err(ComponentError::new("Not enough points for triangulation"));
        }

        let triangulation = delaunator::triangulate(&delaunator_points);
        let hull_indices: Vec<Value> = triangulation
            .hull
            .iter()
            .map(|&i| Value::Number(i as f64))
            .collect();

        let mut hull_lines = Vec::new();
        if triangulation.hull.len() > 1 {
            for i in 0..triangulation.hull.len() {
                let p1_idx = triangulation.hull[i];
                let p2_idx = triangulation.hull[(i + 1) % triangulation.hull.len()];
                hull_lines.push(Value::CurveLine {
                    p1: vertices[p1_idx],
                    p2: vertices[p2_idx],
                });
            }
        }

        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_HULL.to_owned(), Value::List(hull_lines.clone()));
        outputs.insert(OUTPUT_HULL_Z.to_owned(), Value::List(hull_lines));
        outputs.insert(OUTPUT_INDICES.to_owned(), Value::List(hull_indices));

        Ok(outputs)
    }
}
placeholder_component!(VoronoiGroups, "Voronoi Groups");

const OUTPUT_CELLS: &str = "C";

/// Component for creating a Voronoi diagram.
#[derive(Debug, Default, Clone, Copy)]
pub struct Voronoi;

impl Component for Voronoi {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new("Input `P` is missing"));
        }

        let sites = match &inputs[0] {
            Value::List(list) => list
                .iter()
                .map(|v| {
                    let p = coerce::coerce_point(v)?;
                    Ok(voronoice::Point { x: p[0], y: p[1] })
                })
                .collect::<Result<Vec<_>, _>>()?,
            _ => return Err(ComponentError::new("Input `P` is not a list")),
        };

        let mut builder = voronoice::VoronoiBuilder::default();
        builder = builder.set_sites(sites);

        if inputs.len() > 2 && !matches!(&inputs[2], Value::Null) {
            if let Value::CurveLine { p1, p2 } = &inputs[2] {
                let width = (p2[0] - p1[0]).abs();
                let height = (p2[1] - p1[1]).abs();
                let center_x = p1[0] + width / 2.0;
                let center_y = p1[1] + height / 2.0;

                let bounding_box = voronoice::BoundingBox::new(
                    voronoice::Point {
                        x: center_x,
                        y: center_y,
                    },
                    width,
                    height,
                );
                builder = builder.set_bounding_box(bounding_box);
            }
        }

        if let Some(voronoi) = builder.build() {
            let cells = voronoi
                .iter_cells()
                .map(|cell| {
                    let mut points = cell
                        .iter_vertices()
                        .map(|p| Value::Point([p.x, p.y, 0.0]))
                        .collect::<Vec<_>>();
                    if !points.is_empty() {
                        points.push(points[0].clone()); // Close the polyline
                    }
                    Value::List(points)
                })
                .collect();

            let mut outputs = BTreeMap::new();
            outputs.insert(OUTPUT_CELLS.to_owned(), Value::List(cells));
            Ok(outputs)
        } else {
            Err(ComponentError::new("Failed to build Voronoi diagram"))
        }
    }
}

placeholder_component!(OcTree, "OcTree");
placeholder_component!(Voronoi3D, "Voronoi 3D");
placeholder_component!(MetaBallTCustom, "MetaBall(t) Custom");
placeholder_component!(MetaBallT, "MetaBall(t)");
const OUTPUT_CONNECTIVITY: &str = "C";
const OUTPUT_EDGES: &str = "E";

/// Component for creating Delaunay edges from a set of points.
#[derive(Debug, Default, Clone, Copy)]
pub struct DelaunayEdges;

impl Component for DelaunayEdges {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new("Input `P` is missing"));
        }

        let points_value = &inputs[0];
        let points_list = match points_value {
            Value::List(list) => list,
            _ => return Err(ComponentError::new("Input `P` is not a list")),
        };

        let mut vertices = Vec::with_capacity(points_list.len());
        let mut delaunator_points = Vec::with_capacity(points_list.len());

        for value in points_list {
            let point = coerce::coerce_point(value)?;
            vertices.push(point);
            delaunator_points.push(delaunator::Point {
                x: point[0],
                y: point[1],
            });
        }

        if vertices.len() < 3 {
            return Err(ComponentError::new("Not enough points for triangulation"));
        }

        let triangulation = delaunator::triangulate(&delaunator_points);
        let mut edges = Vec::new();
        let mut connectivity: BTreeMap<usize, Vec<Value>> = BTreeMap::new();

        for i in 0..triangulation.triangles.len() {
            let endpoint = triangulation.halfedges[i];
            if i < endpoint {
                let p1_idx = triangulation.triangles[i];
                let p2_idx = triangulation.triangles[delaunator::next_halfedge(i)];

                let p1 = vertices[p1_idx];
                let p2 = vertices[p2_idx];
                edges.push(Value::CurveLine { p1, p2 });

                connectivity
                    .entry(p1_idx)
                    .or_default()
                    .push(Value::Number(p2_idx as f64));
                connectivity
                    .entry(p2_idx)
                    .or_default()
                    .push(Value::Number(p1_idx as f64));
            }
        }

        let connectivity_tree = Value::List(
            (0..vertices.len())
                .map(|i| Value::List(connectivity.remove(&i).unwrap_or_default()))
                .collect(),
        );

        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_CONNECTIVITY.to_owned(), connectivity_tree);
        outputs.insert(OUTPUT_EDGES.to_owned(), Value::List(edges));

        Ok(outputs)
    }
}
placeholder_component!(MetaBall, "MetaBall");

/// Component for finding 3D proximity within a point list.
#[derive(Debug, Default, Clone, Copy)]
pub struct Proximity3D;

impl Component for Proximity3D {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 1 {
            return Err(ComponentError::new("Input `P` is missing"));
        }

        let points = match &inputs[0] {
            Value::List(list) => list
                .iter()
                .map(coerce::coerce_point)
                .collect::<Result<Vec<_>, _>>()?,
            _ => return Err(ComponentError::new("Input `P` is not a list")),
        };

        let group = if inputs.len() > 1 && !matches!(&inputs[1], Value::Null) {
            coerce::coerce_integer(&inputs[1])? as usize
        } else {
            usize::MAX
        };
        let min_radius_sq = if inputs.len() > 2 && !matches!(&inputs[2], Value::Null) {
            let r = coerce::coerce_number(&inputs[2], None)?;
            r * r
        } else {
            0.0
        };
        let max_radius_sq = if inputs.len() > 3 && !matches!(&inputs[3], Value::Null) {
            let r = coerce::coerce_number(&inputs[3], None)?;
            r * r
        } else {
            f64::INFINITY
        };

        if points.is_empty() {
            return Ok(BTreeMap::new());
        }

        let mut links_tree = Vec::new();
        let mut topology_tree = Vec::new();

        for (i, p1) in points.iter().enumerate() {
            let mut neighbors = Vec::new();
            for (j, p2) in points.iter().enumerate() {
                if i == j {
                    continue;
                }

                let dist_sq =
                    (p1[0] - p2[0]).powi(2) + (p1[1] - p2[1]).powi(2) + (p1[2] - p2[2]).powi(2);

                if dist_sq >= min_radius_sq && dist_sq <= max_radius_sq {
                    neighbors.push((dist_sq, j, p2));
                }
            }

            neighbors.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            let mut links_branch = Vec::new();
            let mut topology_branch = Vec::new();

            for (_, j, p2) in neighbors.iter().take(group) {
                links_branch.push(Value::CurveLine { p1: *p1, p2: **p2 });
                topology_branch.push(Value::Text(format!("{{{};{}}}", i, j)));
            }

            links_tree.push(Value::List(links_branch));
            topology_tree.push(Value::List(topology_branch));
        }

        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_LINKS.to_owned(), Value::List(links_tree));
        outputs.insert(OUTPUT_TOPOLOGY.to_owned(), Value::List(topology_tree));

        Ok(outputs)
    }
}

/// Output pin for the mesh result.
const OUTPUT_MESH: &str = "M";

/// Component for creating a Delaunay mesh from a set of 2D points.
///
/// Uses Delaunay triangulation to create a mesh from the input points.
/// The Z-coordinates of the input points are preserved in the output mesh.
///
/// # Inputs
///
/// * `P` (Points): List of points to triangulate
///
/// # Outputs
///
/// * `M` (Mesh): The triangulated mesh as `Value::Mesh`
///
/// # Mesh Engine Integration
///
/// This component now outputs `Value::Mesh` with:
/// - Indexed triangle list (flat indices)
/// - Smooth per-vertex normals
/// - Mesh diagnostics with vertex/triangle counts
#[derive(Debug, Default, Clone, Copy)]
pub struct DelaunayMesh;

impl Component for DelaunayMesh {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new("Input `P` is missing"));
        }

        let points_value = &inputs[0];
        let points_list = match points_value {
            Value::List(list) => list,
            _ => return Err(ComponentError::new("Input `P` is not a list")),
        };

        let mut vertices = Vec::with_capacity(points_list.len());
        let mut delaunator_points = Vec::with_capacity(points_list.len());

        for value in points_list {
            let point = coerce::coerce_point(value)?;
            vertices.push(point);
            delaunator_points.push(delaunator::Point {
                x: point[0],
                y: point[1],
            });
        }

        if vertices.len() < 3 {
            return Err(ComponentError::new("Not enough points for triangulation"));
        }

        let triangulation = delaunator::triangulate(&delaunator_points);

        // Convert triangle indices to flat list for Value::Mesh
        let indices: Vec<u32> = triangulation
            .triangles
            .iter()
            .map(|&i| i as u32)
            .collect();

        // Create mesh with smooth normals and diagnostics
        let mesh = create_mesh_from_triangles(vertices, indices);

        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_MESH.to_owned(), mesh);

        Ok(outputs)
    }
}
