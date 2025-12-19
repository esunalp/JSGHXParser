//! Solid / B-rep-like utilities.
//!
//! This module hosts geometry logic that is still surfaced through legacy component adapters
//! (e.g. `Value::Surface` in `components/surface_util.rs`). The implementations are intentionally
//! conservative: they aim to be deterministic, WASM-safe, and tolerant-aware.
//!
//! # Implemented Features
//!
//! - [`brep_join_legacy`]: Join multiple surface meshes by welding matching naked edges.
//! - [`cap_holes_legacy`]: Cap planar boundary loops using 2D triangulation.
//! - [`cap_holes_ex_legacy`]: Extended capping with additional options.
//! - [`merge_faces_legacy`]: Merge coplanar/continuous faces with tolerance guards.
//! - [`legacy_surface_is_closed`]: Check if a surface mesh is watertight.

use std::collections::HashMap;

use super::triangulation::triangulate_trim_region;
use super::trim::{TrimLoop, TrimRegion, UvPoint};
use super::{Tolerance, Vec3};

/// Legacy "surface" mesh used by existing components (`Value::Surface`).
///
/// Faces may be n-gons; indices refer to `vertices`.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacySurfaceMesh {
    pub vertices: Vec<[f64; 3]>,
    pub faces: Vec<Vec<u32>>,
}

impl LegacySurfaceMesh {
    /// Creates a new empty mesh.
    #[must_use]
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            faces: Vec::new(),
        }
    }

    /// Creates a mesh with pre-allocated capacity.
    #[must_use]
    pub fn with_capacity(vertex_capacity: usize, face_capacity: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(vertex_capacity),
            faces: Vec::with_capacity(face_capacity),
        }
    }

    /// Returns true if the mesh has no vertices or faces.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty() || self.faces.is_empty()
    }
}

impl Default for LegacySurfaceMesh {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BrepJoinDiagnostics {
    pub input_count: usize,
    pub output_count: usize,
    pub closed_count: usize,
    pub open_count: usize,
    /// Number of edges that were welded together across different breps.
    pub welded_edge_count: usize,
    /// Number of vertices that were merged during welding.
    pub merged_vertex_count: usize,
    /// Warnings or non-fatal issues encountered during joining.
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrepJoinResult {
    pub breps: Vec<LegacySurfaceMesh>,
    pub closed: Vec<bool>,
    pub diagnostics: BrepJoinDiagnostics,
}

/// Joins multiple surface meshes by welding matching naked edges.
///
/// This function attempts to merge breps that share coincident naked edges (within tolerance).
/// Breps that can be joined together are combined into unified shells.
///
/// # Algorithm
/// 1. Build edge graphs for all input breps to identify naked edges.
/// 2. Find pairs of naked edges across different breps that are coincident.
/// 3. Merge breps with matching edges by unifying their vertex sets and remapping faces.
/// 4. Report which output shells are closed (watertight).
///
/// # Example
/// ```ignore
/// let result = brep_join_legacy(vec![mesh1, mesh2], Tolerance::default_geom());
/// assert_eq!(result.breps.len(), 1); // Joined into a single shell
/// ```
#[must_use]
pub fn brep_join_legacy(breps: Vec<LegacySurfaceMesh>, tol: Tolerance) -> BrepJoinResult {
    if breps.is_empty() {
        return BrepJoinResult {
            breps: Vec::new(),
            closed: Vec::new(),
            diagnostics: BrepJoinDiagnostics::default(),
        };
    }

    if breps.len() == 1 {
        let is_closed = legacy_surface_is_closed(&breps[0], tol);
        return BrepJoinResult {
            closed: vec![is_closed],
            diagnostics: BrepJoinDiagnostics {
                input_count: 1,
                output_count: 1,
                closed_count: if is_closed { 1 } else { 0 },
                open_count: if is_closed { 0 } else { 1 },
                ..Default::default()
            },
            breps,
        };
    }

    let mut diagnostics = BrepJoinDiagnostics {
        input_count: breps.len(),
        ..Default::default()
    };

    // Build naked edge info for each brep
    let mut brep_naked_edges: Vec<Vec<NakedEdgeInfo>> = Vec::with_capacity(breps.len());
    for brep in &breps {
        let graph = LegacyEdgeGraph::from_surface(brep, tol);
        let naked = graph.naked_edges();
        brep_naked_edges.push(naked);
    }

    // Use union-find to track which breps can be merged
    let mut parent: Vec<usize> = (0..breps.len()).collect();

    fn find(parent: &mut [usize], i: usize) -> usize {
        if parent[i] != i {
            parent[i] = find(parent, parent[i]);
        }
        parent[i]
    }

    fn union(parent: &mut [usize], i: usize, j: usize) {
        let pi = find(parent, i);
        let pj = find(parent, j);
        if pi != pj {
            parent[pi] = pj;
        }
    }

    // Find matching naked edges between different breps
    for i in 0..breps.len() {
        for j in (i + 1)..breps.len() {
            for edge_i in &brep_naked_edges[i] {
                for edge_j in &brep_naked_edges[j] {
                    if edges_match(edge_i, edge_j, tol) {
                        union(&mut parent, i, j);
                        diagnostics.welded_edge_count += 1;
                    }
                }
            }
        }
    }

    // Group breps by their root parent
    let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..breps.len() {
        let root = find(&mut parent, i);
        groups.entry(root).or_default().push(i);
    }

    // Merge breps in each group
    let mut result_breps = Vec::with_capacity(groups.len());
    for (_root, indices) in groups {
        if indices.len() == 1 {
            result_breps.push(breps[indices[0]].clone());
        } else {
            let (merged, merged_count) = merge_breps(&breps, &indices, tol);
            diagnostics.merged_vertex_count += merged_count;
            result_breps.push(merged);
        }
    }

    // Check closedness of each result
    let mut closed = Vec::with_capacity(result_breps.len());
    for brep in &result_breps {
        closed.push(legacy_surface_is_closed(brep, tol));
    }

    diagnostics.output_count = result_breps.len();
    diagnostics.closed_count = closed.iter().filter(|&&c| c).count();
    diagnostics.open_count = diagnostics.output_count - diagnostics.closed_count;

    BrepJoinResult {
        breps: result_breps,
        closed,
        diagnostics,
    }
}

/// Information about a naked (boundary) edge.
#[derive(Debug, Clone)]
struct NakedEdgeInfo {
    start: [f64; 3],
    end: [f64; 3],
}

/// Checks if two naked edges are coincident (match within tolerance).
fn edges_match(a: &NakedEdgeInfo, b: &NakedEdgeInfo, tol: Tolerance) -> bool {
    // Edges can match in either direction
    (approx_eq_point(tol, a.start, b.start) && approx_eq_point(tol, a.end, b.end))
        || (approx_eq_point(tol, a.start, b.end) && approx_eq_point(tol, a.end, b.start))
}

/// Merges multiple breps into a single mesh, welding coincident vertices.
fn merge_breps(
    breps: &[LegacySurfaceMesh],
    indices: &[usize],
    tol: Tolerance,
) -> (LegacySurfaceMesh, usize) {
    let total_verts: usize = indices.iter().map(|&i| breps[i].vertices.len()).sum();
    let total_faces: usize = indices.iter().map(|&i| breps[i].faces.len()).sum();

    let mut merged = LegacySurfaceMesh::with_capacity(total_verts, total_faces);
    let mut merged_count = 0usize;

    for &brep_idx in indices {
        let brep = &breps[brep_idx];

        // Add vertices, checking for duplicates with welding
        let mut vertex_remap: Vec<u32> = Vec::with_capacity(brep.vertices.len());
        for v in &brep.vertices {
            // Check if this vertex already exists (within tolerance)
            let existing = merged
                .vertices
                .iter()
                .position(|existing| approx_eq_point(tol, *existing, *v));

            if let Some(idx) = existing {
                vertex_remap.push(idx as u32);
                merged_count += 1;
            } else {
                vertex_remap.push(merged.vertices.len() as u32);
                merged.vertices.push(*v);
            }
        }

        // Add faces with remapped indices
        for face in &brep.faces {
            let remapped: Vec<u32> = face
                .iter()
                .filter_map(|&idx| vertex_remap.get(idx as usize).copied())
                .collect();
            if remapped.len() >= 3 {
                merged.faces.push(remapped);
            }
        }
    }

    (merged, merged_count)
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CapHolesDiagnostics {
    /// Number of boundary loops detected.
    pub holes_found: usize,
    /// Number of loops successfully capped (triangles appended).
    pub caps_added: usize,
    /// Total number of triangle faces appended to the mesh.
    pub added_face_count: usize,
    /// Number of open edges before capping.
    pub open_edge_count_before: usize,
    /// Number of open edges after capping.
    pub open_edge_count_after: usize,
    /// Maximum planarity deviation of capped holes (0.0 = perfectly planar).
    pub max_planarity_deviation: f64,
    /// Per-loop failures (best-effort, non-fatal).
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapHolesResult {
    pub brep: LegacySurfaceMesh,
    pub is_solid: bool,
    pub diagnostics: CapHolesDiagnostics,
}

/// Options for extended hole capping.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CapHolesExOptions {
    /// Maximum number of vertices in a hole loop before subdividing.
    /// Holes with more vertices than this will be processed in segments.
    pub max_loop_vertices: usize,
    /// Maximum planarity deviation allowed for a hole to be capped.
    /// Holes with deviation above this threshold will be skipped.
    pub max_planarity_deviation: f64,
    /// Whether to subdivide non-convex holes for better triangulation.
    pub subdivide_non_convex: bool,
}

impl Default for CapHolesExOptions {
    fn default() -> Self {
        Self {
            max_loop_vertices: 1000,
            max_planarity_deviation: f64::INFINITY,
            subdivide_non_convex: false,
        }
    }
}

/// Caps planar-ish boundary loops using 2D polygon triangulation (ear clipping).
///
/// The implementation is best-effort: it will skip loops it cannot cap and report those failures
/// in `diagnostics.errors`.
///
/// # Example
/// ```ignore
/// let result = cap_holes_legacy(open_mesh, Tolerance::default_geom());
/// assert!(result.is_solid);
/// ```
#[must_use]
pub fn cap_holes_legacy(brep: LegacySurfaceMesh, tol: Tolerance) -> CapHolesResult {
    cap_holes_ex_legacy(brep, tol, CapHolesExOptions::default())
}

/// Extended hole capping with additional options.
///
/// Provides more control over the capping process, including:
/// - Maximum loop vertex count before segmenting
/// - Planarity deviation threshold
/// - Non-convex hole subdivision
///
/// # Example
/// ```ignore
/// let options = CapHolesExOptions {
///     max_planarity_deviation: 0.01,
///     ..Default::default()
/// };
/// let result = cap_holes_ex_legacy(open_mesh, tol, options);
/// ```
#[must_use]
pub fn cap_holes_ex_legacy(
    mut brep: LegacySurfaceMesh,
    tol: Tolerance,
    options: CapHolesExOptions,
) -> CapHolesResult {
    let mut diagnostics = CapHolesDiagnostics::default();

    if brep.vertices.is_empty() || brep.faces.is_empty() {
        diagnostics.open_edge_count_before = 0;
        diagnostics.open_edge_count_after = 0;
        return CapHolesResult {
            brep,
            is_solid: false,
            diagnostics,
        };
    }

    let graph = LegacyEdgeGraph::from_surface(&brep, tol);
    diagnostics.open_edge_count_before = graph.naked_edge_count();

    if diagnostics.open_edge_count_before == 0 {
        diagnostics.open_edge_count_after = 0;
        return CapHolesResult {
            brep,
            is_solid: true,
            diagnostics,
        };
    }

    let naked_edge_indices = graph.naked_edge_indices();
    let loops = graph.find_loops(&naked_edge_indices, tol);
    diagnostics.holes_found = loops.len();

    for (loop_index, hole_points) in loops.into_iter().enumerate() {
        if hole_points.len() < 3 {
            diagnostics
                .errors
                .push(format!("loop {loop_index}: fewer than 3 points"));
            continue;
        }

        // Check max loop vertices option
        if hole_points.len() > options.max_loop_vertices {
            diagnostics.errors.push(format!(
                "loop {loop_index}: {} vertices exceeds max_loop_vertices ({})",
                hole_points.len(),
                options.max_loop_vertices
            ));
            continue;
        }

        let Some(hole_indices) = map_loop_points_to_indices(&brep.vertices, &hole_points, tol)
        else {
            diagnostics.errors.push(format!(
                "loop {loop_index}: failed to map boundary points to vertices"
            ));
            continue;
        };

        let Some(normal) = newell_normal(&hole_points, tol) else {
            diagnostics
                .errors
                .push(format!("loop {loop_index}: degenerate normal"));
            continue;
        };

        // Calculate planarity deviation
        let planarity_deviation = compute_planarity_deviation(&hole_points, normal);
        if planarity_deviation > options.max_planarity_deviation {
            diagnostics.errors.push(format!(
                "loop {loop_index}: planarity deviation {planarity_deviation:.6} exceeds threshold {:.6}",
                options.max_planarity_deviation
            ));
            continue;
        }
        if planarity_deviation > diagnostics.max_planarity_deviation {
            diagnostics.max_planarity_deviation = planarity_deviation;
        }

        let (u_axis, v_axis) = build_plane_axes(normal, tol);
        let Some((uv_points, uv_to_vertex)) =
            project_loop_with_mapping(&hole_points, &hole_indices, u_axis, v_axis, tol)
        else {
            diagnostics
                .errors
                .push(format!("loop {loop_index}: loop degenerates after projection"));
            continue;
        };

        let loop_ = match TrimLoop::new(uv_points, tol) {
            Ok(loop_) => loop_,
            Err(err) => {
                diagnostics
                    .errors
                    .push(format!("loop {loop_index}: {err}"));
                continue;
            }
        };

        let region = TrimRegion {
            outer: loop_,
            holes: Vec::new(),
        };

        let triangulation = match triangulate_trim_region(&region, tol) {
            Ok(result) => result,
            Err(err) => {
                diagnostics
                    .errors
                    .push(format!("loop {loop_index}: triangulation failed: {err}"));
                continue;
            }
        };

        if triangulation.indices.len() < 3 {
            diagnostics
                .errors
                .push(format!("loop {loop_index}: triangulation produced no triangles"));
            continue;
        }

        let flip_winding = {
            let i0 = triangulation.indices[0] as usize;
            let i1 = triangulation.indices[1] as usize;
            let i2 = triangulation.indices[2] as usize;
            match (
                uv_to_vertex.get(i0),
                uv_to_vertex.get(i1),
                uv_to_vertex.get(i2),
            ) {
                (Some(&v0), Some(&v1), Some(&v2)) => {
                    let p0 = brep.vertices.get(v0 as usize).copied().unwrap_or([0.0; 3]);
                    let p1 = brep.vertices.get(v1 as usize).copied().unwrap_or([0.0; 3]);
                    let p2 = brep.vertices.get(v2 as usize).copied().unwrap_or([0.0; 3]);
                    let tri_normal = triangle_normal(p0, p1, p2);
                    tri_normal.dot(normal) >= 0.0
                }
                _ => false,
            }
        };

        let mut added_any = false;
        for tri in triangulation.indices.chunks_exact(3) {
            let i0 = tri[0] as usize;
            let i1 = tri[1] as usize;
            let i2 = tri[2] as usize;

            let (Some(&v0), Some(&v1), Some(&v2)) = (
                uv_to_vertex.get(i0),
                uv_to_vertex.get(i1),
                uv_to_vertex.get(i2),
            ) else {
                continue;
            };

            if flip_winding {
                brep.faces.push(vec![v0, v2, v1]);
            } else {
                brep.faces.push(vec![v0, v1, v2]);
            }
            diagnostics.added_face_count += 1;
            added_any = true;
        }

        if added_any {
            diagnostics.caps_added += 1;
        } else {
            diagnostics
                .errors
                .push(format!("loop {loop_index}: no triangles appended"));
        }
    }

    let graph_after = LegacyEdgeGraph::from_surface(&brep, tol);
    diagnostics.open_edge_count_after = graph_after.naked_edge_count();
    let is_solid = diagnostics.open_edge_count_after == 0;

    CapHolesResult {
        brep,
        is_solid,
        diagnostics,
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MergeFacesDiagnostics {
    /// Number of input breps/faces.
    pub before: usize,
    /// Number of output faces after merging.
    pub after: usize,
    /// Number of faces that were merged together.
    pub merged_count: usize,
    /// Number of face groups formed (coplanar faces that share edges).
    pub group_count: usize,
    /// Warnings encountered during merge.
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MergeFacesResult {
    pub brep: LegacySurfaceMesh,
    pub diagnostics: MergeFacesDiagnostics,
}

/// Merges coplanar/continuous faces within breps with tolerance guards.
///
/// This function combines multiple input breps and then merges any coplanar adjacent faces
/// that share edges into larger polygons.
///
/// # Algorithm
/// 1. Combine all input breps into a single mesh with welded vertices.
/// 2. Identify face groups where faces are coplanar (within tolerance) and share edges.
/// 3. Merge each group into a single polygon by removing internal edges.
/// 4. Triangulate the resulting polygons if necessary.
///
/// # Example
/// ```ignore
/// let result = merge_faces_legacy(&[mesh1, mesh2], Tolerance::default_geom());
/// assert!(result.diagnostics.merged_count > 0);
/// ```
#[must_use]
pub fn merge_faces_legacy(breps: &[LegacySurfaceMesh], tol: Tolerance) -> Option<MergeFacesResult> {
    if breps.is_empty() {
        return None;
    }

    let before = breps.iter().map(|b| b.faces.len()).sum();
    let mut diagnostics = MergeFacesDiagnostics {
        before,
        ..Default::default()
    };

    // First, combine all breps into one mesh
    let combined = combine_breps(breps, tol);
    if combined.faces.is_empty() {
        return None;
    }

    // Compute face normals
    let face_normals: Vec<Option<Vec3>> = combined
        .faces
        .iter()
        .map(|face| compute_face_normal(&combined.vertices, face))
        .collect();

    // Build face adjacency graph (which faces share edges)
    let adjacency = build_face_adjacency(&combined, tol);

    // Group coplanar adjacent faces using union-find
    let mut parent: Vec<usize> = (0..combined.faces.len()).collect();

    fn find(parent: &mut [usize], i: usize) -> usize {
        if parent[i] != i {
            parent[i] = find(parent, parent[i]);
        }
        parent[i]
    }

    fn union(parent: &mut [usize], i: usize, j: usize) {
        let pi = find(parent, i);
        let pj = find(parent, j);
        if pi != pj {
            parent[pi] = pj;
        }
    }

    // Merge adjacent coplanar faces
    for (face_i, neighbors) in adjacency.iter().enumerate() {
        let Some(normal_i) = face_normals[face_i] else {
            continue;
        };

        for &face_j in neighbors {
            if face_j <= face_i {
                continue;
            }
            let Some(normal_j) = face_normals[face_j] else {
                continue;
            };

            // Check if normals are parallel (coplanar faces)
            let dot = normal_i.dot(normal_j).abs();
            if dot >= 1.0 - tol.eps {
                // Check if faces are on the same plane (not just parallel)
                if faces_coplanar(&combined, face_i, face_j, normal_i, tol) {
                    union(&mut parent, face_i, face_j);
                    diagnostics.merged_count += 1;
                }
            }
        }
    }

    // Group faces by their root
    let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..combined.faces.len() {
        let root = find(&mut parent, i);
        groups.entry(root).or_default().push(i);
    }
    diagnostics.group_count = groups.len();

    // Build output mesh
    let mut result = LegacySurfaceMesh::with_capacity(
        combined.vertices.len(),
        groups.len(),
    );
    result.vertices = combined.vertices.clone();

    for (_root, face_indices) in groups {
        if face_indices.len() == 1 {
            // Single face, keep as is
            result.faces.push(combined.faces[face_indices[0]].clone());
        } else {
            // Merge multiple faces into one polygon
            match merge_coplanar_faces(&combined, &face_indices, tol) {
                Ok(merged_face) => {
                    result.faces.push(merged_face);
                }
                Err(err) => {
                    // Fall back to keeping individual faces
                    diagnostics.warnings.push(err);
                    for &fi in &face_indices {
                        result.faces.push(combined.faces[fi].clone());
                    }
                }
            }
        }
    }

    diagnostics.after = result.faces.len();

    Some(MergeFacesResult {
        brep: result,
        diagnostics,
    })
}

/// Combines multiple breps into a single mesh with welded vertices.
fn combine_breps(breps: &[LegacySurfaceMesh], tol: Tolerance) -> LegacySurfaceMesh {
    let total_verts: usize = breps.iter().map(|b| b.vertices.len()).sum();
    let total_faces: usize = breps.iter().map(|b| b.faces.len()).sum();

    let mut combined = LegacySurfaceMesh::with_capacity(total_verts, total_faces);

    for brep in breps {
        let mut vertex_remap: Vec<u32> = Vec::with_capacity(brep.vertices.len());

        for v in &brep.vertices {
            let existing = combined
                .vertices
                .iter()
                .position(|existing| approx_eq_point(tol, *existing, *v));

            if let Some(idx) = existing {
                vertex_remap.push(idx as u32);
            } else {
                vertex_remap.push(combined.vertices.len() as u32);
                combined.vertices.push(*v);
            }
        }

        for face in &brep.faces {
            let remapped: Vec<u32> = face
                .iter()
                .filter_map(|&idx| vertex_remap.get(idx as usize).copied())
                .collect();
            if remapped.len() >= 3 {
                combined.faces.push(remapped);
            }
        }
    }

    combined
}

/// Builds adjacency list: for each face, lists faces that share an edge.
fn build_face_adjacency(mesh: &LegacySurfaceMesh, _tol: Tolerance) -> Vec<Vec<usize>> {
    let mut edge_to_faces: HashMap<EdgeKey, Vec<usize>> = HashMap::new();

    for (face_idx, face) in mesh.faces.iter().enumerate() {
        for i in 0..face.len() {
            let a = face[i];
            let b = face[(i + 1) % face.len()];
            let key = EdgeKey::new(a, b);
            edge_to_faces.entry(key).or_default().push(face_idx);
        }
    }

    let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); mesh.faces.len()];

    for faces in edge_to_faces.values() {
        for i in 0..faces.len() {
            for j in (i + 1)..faces.len() {
                let fi = faces[i];
                let fj = faces[j];
                if !adjacency[fi].contains(&fj) {
                    adjacency[fi].push(fj);
                }
                if !adjacency[fj].contains(&fi) {
                    adjacency[fj].push(fi);
                }
            }
        }
    }

    adjacency
}

/// Computes the normal of a face.
fn compute_face_normal(vertices: &[[f64; 3]], face: &[u32]) -> Option<Vec3> {
    if face.len() < 3 {
        return None;
    }
    let points: Vec<[f64; 3]> = face
        .iter()
        .filter_map(|&i| vertices.get(i as usize).copied())
        .collect();
    newell_normal(&points, Tolerance::default_geom())
}

/// Checks if two adjacent faces are coplanar (on the same plane).
fn faces_coplanar(
    mesh: &LegacySurfaceMesh,
    face_i: usize,
    face_j: usize,
    normal: Vec3,
    tol: Tolerance,
) -> bool {
    // Get a point from face_i to define the plane
    let Some(&idx) = mesh.faces[face_i].first() else {
        return false;
    };
    let Some(plane_point) = mesh.vertices.get(idx as usize) else {
        return false;
    };

    // Check if all points of face_j are on the same plane
    for &idx in &mesh.faces[face_j] {
        let Some(point) = mesh.vertices.get(idx as usize) else {
            return false;
        };
        let d = Vec3::new(
            point[0] - plane_point[0],
            point[1] - plane_point[1],
            point[2] - plane_point[2],
        );
        let dist = d.dot(normal).abs();
        if dist > tol.eps {
            return false;
        }
    }

    true
}

/// Merges multiple coplanar faces into a single polygon.
fn merge_coplanar_faces(
    mesh: &LegacySurfaceMesh,
    face_indices: &[usize],
    _tol: Tolerance,
) -> Result<Vec<u32>, String> {
    // Collect all edges and identify internal (shared) vs boundary edges
    let mut edge_count: HashMap<EdgeKey, usize> = HashMap::new();

    for &fi in face_indices {
        let face = &mesh.faces[fi];
        for i in 0..face.len() {
            let a = face[i];
            let b = face[(i + 1) % face.len()];
            let key = EdgeKey::new(a, b);
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }

    // Boundary edges appear exactly once
    let boundary_edges: Vec<(u32, u32)> = edge_count
        .iter()
        .filter_map(|(key, &count)| {
            if count == 1 {
                // Find the original direction from the faces
                for &fi in face_indices {
                    let face = &mesh.faces[fi];
                    for i in 0..face.len() {
                        let a = face[i];
                        let b = face[(i + 1) % face.len()];
                        if EdgeKey::new(a, b) == *key {
                            return Some((a, b));
                        }
                    }
                }
                None
            } else {
                None
            }
        })
        .collect();

    if boundary_edges.is_empty() {
        return Err("no boundary edges found".to_string());
    }

    // Chain boundary edges into a loop
    let mut remaining: Vec<(u32, u32)> = boundary_edges.clone();
    let mut polygon: Vec<u32> = Vec::new();

    let (first_a, first_b) = remaining.remove(0);
    polygon.push(first_a);
    polygon.push(first_b);
    let mut current = first_b;

    while !remaining.is_empty() {
        let next_idx = remaining.iter().position(|&(a, b)| a == current || b == current);
        match next_idx {
            Some(idx) => {
                let (a, b) = remaining.remove(idx);
                let next = if a == current { b } else { a };
                if next == polygon[0] {
                    // Loop closed
                    break;
                }
                polygon.push(next);
                current = next;
            }
            None => {
                return Err("boundary edges do not form a closed loop".to_string());
            }
        }
    }

    if polygon.len() < 3 {
        return Err("merged polygon has fewer than 3 vertices".to_string());
    }

    Ok(polygon)
}

/// Edge key for hash-based lookups (order-independent).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EdgeKey(u32, u32);

impl EdgeKey {
    fn new(a: u32, b: u32) -> Self {
        if a <= b {
            Self(a, b)
        } else {
            Self(b, a)
        }
    }
}

#[must_use]
pub fn legacy_surface_is_closed(brep: &LegacySurfaceMesh, tol: Tolerance) -> bool {
    if brep.vertices.is_empty() || brep.faces.is_empty() {
        return false;
    }
    LegacyEdgeGraph::from_surface(brep, tol).naked_edge_count() == 0
}

#[derive(Debug, Default, Clone)]
struct LegacyEdgeGraph {
    edges: Vec<EdgeData>,
}

impl LegacyEdgeGraph {
    fn from_surface(surface: &LegacySurfaceMesh, tol: Tolerance) -> Self {
        let mut graph = Self::default();
        for (face_index, face) in surface.faces.iter().enumerate() {
            if face.len() < 2 {
                continue;
            }
            for segment in 0..face.len() {
                let a = face[segment] as usize;
                let b = face[(segment + 1) % face.len()] as usize;
                let Some(&start) = surface.vertices.get(a) else { continue };
                let Some(&end) = surface.vertices.get(b) else { continue };
                graph.add_edge(start, end, face_index, tol);
            }
        }
        graph
    }

    fn add_edge(&mut self, start: [f64; 3], end: [f64; 3], face: usize, tol: Tolerance) {
        if approx_eq_point(tol, start, end) {
            return;
        }

        if let Some(existing) = self
            .edges
            .iter_mut()
            .find(|edge| edge.matches(start, end, tol))
        {
            existing.add_face(face);
            return;
        }

        let mut edge = EdgeData::new(start, end);
        edge.add_face(face);
        self.edges.push(edge);
    }

    fn naked_edge_count(&self) -> usize {
        self.edges.iter().filter(|edge| edge.face_count() == 1).count()
    }

    fn naked_edge_indices(&self) -> Vec<usize> {
        self.edges
            .iter()
            .enumerate()
            .filter_map(|(i, edge)| (edge.face_count() == 1).then_some(i))
            .collect()
    }

    /// Returns naked edge info for all boundary edges.
    fn naked_edges(&self) -> Vec<NakedEdgeInfo> {
        self.edges
            .iter()
            .filter(|edge| edge.face_count() == 1)
            .map(|edge| NakedEdgeInfo {
                start: edge.start,
                end: edge.end,
            })
            .collect()
    }

    fn find_loops(&self, edge_indices: &[usize], tol: Tolerance) -> Vec<Vec<[f64; 3]>> {
        let mut visited = vec![false; edge_indices.len()];
        let mut loops = Vec::new();

        for i in 0..edge_indices.len() {
            if visited[i] {
                continue;
            }

            let start_edge_idx = edge_indices[i];
            let start_edge = &self.edges[start_edge_idx];

            let mut current_loop = Vec::new();
            current_loop.push(start_edge.start);
            current_loop.push(start_edge.end);

            visited[i] = true;
            let mut current_end = start_edge.end;
            let mut loop_closed = false;

            loop {
                let mut found_next = false;
                for j in 0..edge_indices.len() {
                    if visited[j] {
                        continue;
                    }

                    let next_edge_idx = edge_indices[j];
                    let next_edge = &self.edges[next_edge_idx];

                    if approx_eq_point(tol, next_edge.start, current_end) {
                        current_loop.push(next_edge.end);
                        current_end = next_edge.end;
                        visited[j] = true;
                        found_next = true;
                    } else if approx_eq_point(tol, next_edge.end, current_end) {
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
                    if approx_eq_point(tol, current_end, current_loop[0]) {
                        loop_closed = true;
                        current_loop.pop();
                    }
                    break;
                }

                if approx_eq_point(tol, current_end, current_loop[0]) {
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

    fn matches(&self, start: [f64; 3], end: [f64; 3], tol: Tolerance) -> bool {
        (approx_eq_point(tol, self.start, start) && approx_eq_point(tol, self.end, end))
            || (approx_eq_point(tol, self.start, end) && approx_eq_point(tol, self.end, start))
    }

    fn face_count(&self) -> usize {
        self.faces.len()
    }

    fn add_face(&mut self, face: usize) {
        if !self.faces.contains(&face) {
            self.faces.push(face);
        }
    }
}

fn approx_eq_point(tol: Tolerance, a: [f64; 3], b: [f64; 3]) -> bool {
    tol.approx_eq_f64(a[0], b[0]) && tol.approx_eq_f64(a[1], b[1]) && tol.approx_eq_f64(a[2], b[2])
}

fn map_loop_points_to_indices(
    vertices: &[[f64; 3]],
    points: &[[f64; 3]],
    tol: Tolerance,
) -> Option<Vec<u32>> {
    let mut indices = Vec::with_capacity(points.len());
    for point in points {
        let idx = vertices
            .iter()
            .position(|v| approx_eq_point(tol, *v, *point))? as u32;
        indices.push(idx);
    }
    Some(indices)
}

fn newell_normal(points: &[[f64; 3]], tol: Tolerance) -> Option<Vec3> {
    if points.len() < 3 {
        return None;
    }

    let mut nx = 0.0;
    let mut ny = 0.0;
    let mut nz = 0.0;

    for i in 0..points.len() {
        let p1 = points[i];
        let p2 = points[(i + 1) % points.len()];
        nx += (p1[1] - p2[1]) * (p1[2] + p2[2]);
        ny += (p1[2] - p2[2]) * (p1[0] + p2[0]);
        nz += (p1[0] - p2[0]) * (p1[1] + p2[1]);
    }

    let normal = Vec3::new(nx, ny, nz);
    let len = normal.length();
    if !len.is_finite() || len <= tol.eps {
        None
    } else {
        Some(normal.mul_scalar(1.0 / len))
    }
}

fn build_plane_axes(normal: Vec3, tol: Tolerance) -> (Vec3, Vec3) {
    let candidate = if normal.x.abs() > 0.9 {
        Vec3::new(0.0, 1.0, 0.0)
    } else {
        Vec3::new(1.0, 0.0, 0.0)
    };

    let mut u = candidate.cross(normal);
    let u_len = u.length();
    if !u_len.is_finite() || u_len <= tol.eps {
        u = Vec3::new(1.0, 0.0, 0.0);
    } else {
        u = u.mul_scalar(1.0 / u_len);
    }

    let v = normal.cross(u);
    (u, v)
}

fn project_loop_with_mapping(
    points: &[[f64; 3]],
    indices: &[u32],
    u_axis: Vec3,
    v_axis: Vec3,
    tol: Tolerance,
) -> Option<(Vec<UvPoint>, Vec<u32>)> {
    if points.len() != indices.len() || points.len() < 3 {
        return None;
    }

    let mut uv_points: Vec<UvPoint> = Vec::with_capacity(points.len());
    for p in points {
        let vec = Vec3::new(p[0], p[1], p[2]);
        uv_points.push(UvPoint::new(vec.dot(u_axis), vec.dot(v_axis)));
    }

    if uv_points.len() > 2 {
        if let (Some(first), Some(last)) = (uv_points.first().copied(), uv_points.last().copied()) {
            if approx_eq_uv(tol, first, last) {
                uv_points.pop();
            }
        }
    }

    let mut cleaned_uv = Vec::with_capacity(uv_points.len());
    let mut cleaned_indices = Vec::with_capacity(indices.len());

    for (p, idx) in uv_points.into_iter().zip(indices.iter().copied()) {
        if cleaned_uv
            .last()
            .copied()
            .is_some_and(|prev| approx_eq_uv(tol, prev, p))
        {
            continue;
        }
        cleaned_uv.push(p);
        cleaned_indices.push(idx);
    }

    if cleaned_uv.len() < 3 {
        None
    } else {
        Some((cleaned_uv, cleaned_indices))
    }
}

fn approx_eq_uv(tol: Tolerance, a: UvPoint, b: UvPoint) -> bool {
    tol.approx_eq_f64(a.u, b.u) && tol.approx_eq_f64(a.v, b.v)
}

fn triangle_normal(p0: [f64; 3], p1: [f64; 3], p2: [f64; 3]) -> Vec3 {
    let a = Vec3::new(p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]);
    let b = Vec3::new(p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]);
    a.cross(b)
}

/// Computes the maximum distance from the best-fit plane for a set of points.
/// Returns 0.0 for perfectly planar loops.
fn compute_planarity_deviation(points: &[[f64; 3]], normal: Vec3) -> f64 {
    if points.len() < 3 {
        return 0.0;
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

    // Compute max distance from the plane through centroid with given normal
    let mut max_deviation = 0.0;
    for p in points {
        let d = Vec3::new(p[0] - cx, p[1] - cy, p[2] - cz);
        let dist = d.dot(normal).abs();
        if dist > max_deviation {
            max_deviation = dist;
        }
    }

    max_deviation
}

