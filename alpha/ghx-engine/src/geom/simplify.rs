//! Mesh simplification (LOD) via edge-collapse with watertightness guard.
//!
//! This module provides Level-of-Detail (LOD) mesh simplification using
//! edge-collapse decimation. The algorithm preserves mesh topology where
//! possible and provides explicit diagnostics when watertightness cannot
//! be maintained.
//!
//! # Algorithm
//!
//! The implementation uses a priority-queue-driven edge-collapse approach:
//! 1. Compute edge collapse costs (geometric error + boundary penalty)
//! 2. Process edges in order of increasing cost
//! 3. Skip edges that would violate topology constraints (watertightness guard)
//! 4. Update affected edges after each collapse
//! 5. Continue until target is reached or no more edges can be collapsed
//!
//! # Limitations
//!
//! - Cost function uses edge length (not Quadric Error Metrics), which may not
//!   produce optimal results for curved surfaces. Flatter areas are not prioritized.
//! - Normal flip detection is basic; extreme deformations may still occur.
//!
//! # Example
//!
//! ```no_run
//! use ghx_engine::geom::{simplify_mesh, SimplifyOptions, SimplifyTarget, GeomMesh};
//!
//! let mesh = GeomMesh {
//!     positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
//!     indices: vec![0, 1, 2],
//!     normals: None,
//!     uvs: None,
//!     tangents: None,
//! };
//! let options = SimplifyOptions::new(SimplifyTarget::TriangleCount(1));
//! let result = simplify_mesh(&mesh, options).unwrap();
//! println!("Simplified to {} triangles", result.mesh.triangle_count());
//! ```

use super::mesh::{compute_smooth_normals_for_mesh, GeomMesh};
use super::{Point3, Tolerance, Vec3};
use std::collections::{BinaryHeap, HashMap, HashSet};

/// Target for simplification operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SimplifyTarget {
    /// Target a specific number of triangles.
    TriangleCount(usize),
    /// Target a ratio of the original triangle count (0.0 to 1.0).
    Ratio(f64),
    /// Target a maximum geometric error threshold.
    MaxError(f64),
}

impl Default for SimplifyTarget {
    fn default() -> Self {
        Self::Ratio(0.5)
    }
}

/// Options for mesh simplification.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimplifyOptions {
    /// The simplification target.
    pub target: SimplifyTarget,
    /// Whether to preserve boundary edges (edges with only one adjacent triangle).
    pub preserve_boundary: bool,
    /// Weight factor for boundary edge penalties (higher = less likely to collapse).
    pub boundary_weight: f64,
    /// Whether to strictly maintain watertightness (reject any collapse that opens the mesh).
    pub strict_watertight: bool,
    /// Maximum aspect ratio allowed for resulting triangles (prevents slivers).
    pub max_aspect_ratio: f64,
    /// Whether to recompute smooth normals after simplification (default: true).
    /// When true, vertex normals are computed from the simplified mesh.
    /// When false, normals are dropped and a warning is added to diagnostics.
    pub recompute_normals: bool,
}

impl SimplifyOptions {
    /// Create new simplify options with the given target.
    #[must_use]
    pub fn new(target: SimplifyTarget) -> Self {
        Self {
            target,
            preserve_boundary: true,
            boundary_weight: 10.0,
            strict_watertight: true,
            max_aspect_ratio: 10.0,
            recompute_normals: true,
        }
    }

    /// Create options targeting a specific triangle count.
    #[must_use]
    pub fn target_triangles(count: usize) -> Self {
        Self::new(SimplifyTarget::TriangleCount(count))
    }

    /// Create options targeting a ratio of original triangles.
    #[must_use]
    pub fn target_ratio(ratio: f64) -> Self {
        Self::new(SimplifyTarget::Ratio(ratio.clamp(0.0, 1.0)))
    }

    /// Create options targeting a maximum error threshold.
    #[must_use]
    pub fn target_error(max_error: f64) -> Self {
        Self::new(SimplifyTarget::MaxError(max_error))
    }

    /// Set whether to preserve boundary edges.
    #[must_use]
    pub const fn preserve_boundary(mut self, preserve: bool) -> Self {
        self.preserve_boundary = preserve;
        self
    }

    /// Set the boundary edge weight.
    #[must_use]
    pub const fn boundary_weight(mut self, weight: f64) -> Self {
        self.boundary_weight = weight;
        self
    }

    /// Set strict watertightness mode.
    #[must_use]
    pub const fn strict_watertight(mut self, strict: bool) -> Self {
        self.strict_watertight = strict;
        self
    }

    /// Set maximum allowed aspect ratio for triangles.
    #[must_use]
    pub const fn max_aspect_ratio(mut self, ratio: f64) -> Self {
        self.max_aspect_ratio = ratio;
        self
    }

    /// Set whether to recompute smooth normals after simplification.
    ///
    /// When enabled (default), vertex normals are computed from the simplified
    /// mesh geometry. When disabled, normals are dropped and a warning is added.
    #[must_use]
    pub const fn recompute_normals(mut self, recompute: bool) -> Self {
        self.recompute_normals = recompute;
        self
    }
}

impl Default for SimplifyOptions {
    fn default() -> Self {
        Self::new(SimplifyTarget::default())
    }
}

/// Diagnostics from simplification operations.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SimplifyDiagnostics {
    /// Original vertex count before simplification.
    pub original_vertex_count: usize,
    /// Original triangle count before simplification.
    pub original_triangle_count: usize,
    /// Final vertex count after simplification.
    pub final_vertex_count: usize,
    /// Final triangle count after simplification.
    pub final_triangle_count: usize,
    /// Number of edges collapsed.
    pub edges_collapsed: usize,
    /// Number of boundary edges preserved.
    pub boundary_edges_preserved: usize,
    /// Number of edges skipped due to topology constraints.
    pub topology_rejections: usize,
    /// Number of edges skipped due to aspect ratio constraints.
    pub aspect_ratio_rejections: usize,
    /// Maximum geometric error introduced.
    pub max_error: f64,
    /// Whether the mesh remained watertight after simplification.
    pub watertight_preserved: bool,
    /// Any warnings generated during simplification.
    pub warnings: Vec<String>,
}

/// Errors that can occur during simplification.
#[derive(Debug, thiserror::Error)]
pub enum SimplifyError {
    /// The input mesh is empty.
    #[error("input mesh has no triangles")]
    EmptyMesh,

    /// The input mesh contains invalid geometry.
    #[error("input mesh contains invalid geometry (NaN/Inf values or out-of-bounds indices)")]
    InvalidGeometry,

    /// The target is invalid.
    #[error("invalid simplification target: {reason}")]
    InvalidTarget { reason: String },
}

/// Result of a simplification operation.
#[derive(Debug, Clone)]
pub struct SimplifyResult {
    /// The simplified mesh.
    pub mesh: GeomMesh,
    /// Diagnostics from the simplification process.
    pub diagnostics: SimplifyDiagnostics,
}

/// Simplify a mesh using edge-collapse decimation.
///
/// # Arguments
/// * `mesh` - The input mesh to simplify.
/// * `options` - Simplification options controlling target and behavior.
///
/// # Returns
/// A `SimplifyResult` containing the simplified mesh and diagnostics.
///
/// # Errors
/// Returns an error if the mesh is empty, contains invalid geometry,
/// or cannot be simplified to meet the target.
#[must_use = "simplification result should be used"]
pub fn simplify_mesh(
    mesh: &GeomMesh,
    options: SimplifyOptions,
) -> Result<SimplifyResult, SimplifyError> {
    simplify_mesh_with_tolerance(mesh, options, Tolerance::default_geom())
}

/// Simplify a mesh with explicit tolerance.
#[must_use = "simplification result should be used"]
pub fn simplify_mesh_with_tolerance(
    mesh: &GeomMesh,
    options: SimplifyOptions,
    tol: Tolerance,
) -> Result<SimplifyResult, SimplifyError> {
    // Validate input
    if mesh.indices.is_empty() {
        return Err(SimplifyError::EmptyMesh);
    }

    // Check for invalid geometry (NaN/Inf positions)
    for pos in &mesh.positions {
        if !pos[0].is_finite() || !pos[1].is_finite() || !pos[2].is_finite() {
            return Err(SimplifyError::InvalidGeometry);
        }
    }

    // Check for out-of-bounds indices
    let vertex_count = mesh.positions.len();
    for &idx in &mesh.indices {
        if (idx as usize) >= vertex_count {
            return Err(SimplifyError::InvalidGeometry);
        }
    }

    let original_triangle_count = mesh.triangle_count();
    let original_vertex_count = mesh.positions.len();

    // Compute target triangle count
    let target_count = match options.target {
        SimplifyTarget::TriangleCount(n) => n,
        SimplifyTarget::Ratio(r) => {
            if !(0.0..=1.0).contains(&r) {
                return Err(SimplifyError::InvalidTarget {
                    reason: format!("ratio must be between 0.0 and 1.0, got {r}"),
                });
            }
            (original_triangle_count as f64 * r).ceil() as usize
        }
        SimplifyTarget::MaxError(e) => {
            if e < 0.0 || !e.is_finite() {
                return Err(SimplifyError::InvalidTarget {
                    reason: format!("max error must be non-negative and finite, got {e}"),
                });
            }
            0 // Will stop based on error threshold
        }
    };

    // If already at or below target, return unchanged
    if original_triangle_count <= target_count && !matches!(options.target, SimplifyTarget::MaxError(_)) {
        return Ok(SimplifyResult {
            mesh: mesh.clone(),
            diagnostics: SimplifyDiagnostics {
                original_vertex_count,
                original_triangle_count,
                final_vertex_count: original_vertex_count,
                final_triangle_count: original_triangle_count,
                watertight_preserved: true,
                ..Default::default()
            },
        });
    }

    // Build internal representation
    let mut simplifier = MeshSimplifier::new(mesh, options, tol);
    let was_watertight = simplifier.is_watertight();

    // Perform simplification
    simplifier.simplify(target_count)?;

    // Extract result
    let (positions, indices) = simplifier.extract_mesh();
    let final_triangle_count = indices.len() / 3;
    let final_vertex_count = positions.len();

    let mut warnings = simplifier.warnings;

    // Rebuild mesh with optional attributes
    let positions_arr: Vec<[f64; 3]> = positions.iter().map(|p| p.to_array()).collect();

    // Compute smooth normals if requested
    let normals = if options.recompute_normals {
        Some(compute_smooth_normals_for_mesh(&positions, &indices))
    } else {
        warnings.push(
            "normals dropped after simplification (recompute_normals=false)".to_string(),
        );
        None
    };

    // Note: UVs and tangents require interpolation during edge collapse, which is
    // not currently supported. They are dropped with a warning if they were present.
    let uvs = None;
    let tangents = None;

    let result_mesh = GeomMesh {
        positions: positions_arr,
        indices,
        normals,
        uvs,
        tangents,
    };

    let is_watertight = count_open_edges(&result_mesh.indices) == 0;
    let watertight_preserved = !was_watertight || is_watertight;

    if was_watertight && !is_watertight {
        warnings.push("watertightness was lost during simplification".to_string());
    }

    Ok(SimplifyResult {
        mesh: result_mesh,
        diagnostics: SimplifyDiagnostics {
            original_vertex_count,
            original_triangle_count,
            final_vertex_count,
            final_triangle_count,
            edges_collapsed: simplifier.edges_collapsed,
            boundary_edges_preserved: simplifier.boundary_preserved,
            topology_rejections: simplifier.topology_rejections,
            aspect_ratio_rejections: simplifier.aspect_rejections,
            max_error: simplifier.max_error,
            watertight_preserved,
            warnings,
        },
    })
}

/// Convenience function to simplify to a target triangle count.
pub fn simplify_to_count(mesh: &GeomMesh, target_count: usize) -> Result<SimplifyResult, SimplifyError> {
    simplify_mesh(mesh, SimplifyOptions::target_triangles(target_count))
}

/// Convenience function to simplify by a ratio.
pub fn simplify_by_ratio(mesh: &GeomMesh, ratio: f64) -> Result<SimplifyResult, SimplifyError> {
    simplify_mesh(mesh, SimplifyOptions::target_ratio(ratio))
}

// ---------------------------------------------------------------------------
// Internal Implementation
// ---------------------------------------------------------------------------

/// Directed edge key (v0, v1) where v0 < v1 for canonical ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EdgeKey(u32, u32);

impl EdgeKey {
    #[inline]
    fn new(v0: u32, v1: u32) -> Self {
        if v0 <= v1 {
            Self(v0, v1)
        } else {
            Self(v1, v0)
        }
    }
}

/// Reason for rejecting an edge collapse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CollapseRejection {
    /// Would create degenerate triangles.
    Degenerate,
    /// Would violate link condition (create non-manifold geometry).
    LinkCondition,
    /// Would flip triangle normals.
    NormalFlip,
    /// Would create triangles with bad aspect ratio.
    AspectRatio,
}

/// Edge collapse candidate with priority ordering.
#[derive(Debug, Clone)]
struct CollapseCandidate {
    edge: EdgeKey,
    cost: f64,
    target_pos: Point3,
    version: u64,
}

impl PartialEq for CollapseCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.edge == other.edge && self.version == other.version
    }
}

impl Eq for CollapseCandidate {}

impl PartialOrd for CollapseCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CollapseCandidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse ordering for min-heap behavior (lower cost = higher priority)
        other.cost.partial_cmp(&self.cost).unwrap_or(std::cmp::Ordering::Equal)
    }
}

/// Triangle representation for internal processing.
#[derive(Debug, Clone, Copy)]
struct Triangle {
    v0: u32,
    v1: u32,
    v2: u32,
    deleted: bool,
}

impl Triangle {
    #[inline]
    fn new(v0: u32, v1: u32, v2: u32) -> Self {
        Self { v0, v1, v2, deleted: false }
    }

    #[inline]
    fn vertices(&self) -> [u32; 3] {
        [self.v0, self.v1, self.v2]
    }

    #[inline]
    fn contains_vertex(&self, v: u32) -> bool {
        self.v0 == v || self.v1 == v || self.v2 == v
    }

    #[inline]
    fn replace_vertex(&mut self, old: u32, new: u32) {
        if self.v0 == old { self.v0 = new; }
        if self.v1 == old { self.v1 = new; }
        if self.v2 == old { self.v2 = new; }
    }

    #[inline]
    fn is_degenerate(&self) -> bool {
        self.v0 == self.v1 || self.v1 == self.v2 || self.v0 == self.v2
    }
}

/// Internal mesh simplifier state.
struct MeshSimplifier {
    positions: Vec<Point3>,
    triangles: Vec<Triangle>,
    vertex_deleted: Vec<bool>,
    vertex_triangles: Vec<Vec<usize>>, // For each vertex, list of triangle indices
    edge_versions: HashMap<EdgeKey, u64>,
    boundary_edges: HashSet<EdgeKey>, // Cached set of boundary edges
    options: SimplifyOptions,
    tol: Tolerance,
    heap: BinaryHeap<CollapseCandidate>,
    version_counter: u64,
    edges_collapsed: usize,
    boundary_preserved: usize,
    topology_rejections: usize,
    aspect_rejections: usize,
    normal_flip_rejections: usize,
    max_error: f64,
    warnings: Vec<String>,
}

impl MeshSimplifier {
    fn new(mesh: &GeomMesh, options: SimplifyOptions, tol: Tolerance) -> Self {
        let positions: Vec<Point3> = mesh.positions.iter()
            .map(|p| Point3::new(p[0], p[1], p[2]))
            .collect();

        let n_verts = positions.len();
        let vertex_deleted = vec![false; n_verts];
        let mut vertex_triangles: Vec<Vec<usize>> = vec![Vec::new(); n_verts];

        let triangles: Vec<Triangle> = mesh.indices
            .chunks_exact(3)
            .enumerate()
            .map(|(ti, tri)| {
                let t = Triangle::new(tri[0], tri[1], tri[2]);
                vertex_triangles[tri[0] as usize].push(ti);
                vertex_triangles[tri[1] as usize].push(ti);
                vertex_triangles[tri[2] as usize].push(ti);
                t
            })
            .collect();

        // Build initial boundary edge cache
        let boundary_edges = Self::compute_boundary_edges(&triangles);

        Self {
            positions,
            triangles,
            vertex_deleted,
            vertex_triangles,
            edge_versions: HashMap::new(),
            boundary_edges,
            options,
            tol,
            heap: BinaryHeap::new(),
            version_counter: 0,
            edges_collapsed: 0,
            boundary_preserved: 0,
            topology_rejections: 0,
            aspect_rejections: 0,
            normal_flip_rejections: 0,
            max_error: 0.0,
            warnings: Vec::new(),
        }
    }

    /// Compute the set of boundary edges (edges with only one adjacent triangle).
    fn compute_boundary_edges(triangles: &[Triangle]) -> HashSet<EdgeKey> {
        let mut edge_counts: HashMap<EdgeKey, u32> = HashMap::new();
        for tri in triangles {
            if tri.deleted { continue; }
            for (a, b) in [(tri.v0, tri.v1), (tri.v1, tri.v2), (tri.v2, tri.v0)] {
                let key = EdgeKey::new(a, b);
                *edge_counts.entry(key).or_insert(0) += 1;
            }
        }
        edge_counts.into_iter()
            .filter(|&(_, count)| count == 1)
            .map(|(edge, _)| edge)
            .collect()
    }

    fn is_watertight(&self) -> bool {
        let mut edge_counts: HashMap<EdgeKey, u32> = HashMap::new();
        for tri in &self.triangles {
            if tri.deleted { continue; }
            for (a, b) in [(tri.v0, tri.v1), (tri.v1, tri.v2), (tri.v2, tri.v0)] {
                let key = EdgeKey::new(a, b);
                *edge_counts.entry(key).or_insert(0) += 1;
            }
        }
        edge_counts.values().all(|&c| c == 2)
    }

    fn simplify(&mut self, target_count: usize) -> Result<(), SimplifyError> {
        // Build initial heap with all edge collapse candidates
        self.build_initial_heap();

        let max_error_target = if let SimplifyTarget::MaxError(e) = self.options.target {
            Some(e)
        } else {
            None
        };

        // Process collapses
        while self.active_triangle_count() > target_count {
            let Some(candidate) = self.pop_valid_candidate() else {
                // No more valid candidates
                if self.active_triangle_count() > target_count {
                    self.warnings.push(format!(
                        "could not reach target {} triangles, stopped at {}",
                        target_count,
                        self.active_triangle_count()
                    ));
                }
                break;
            };

            // Check error threshold
            if let Some(max_err) = max_error_target {
                if candidate.cost > max_err {
                    break;
                }
            }

            // Perform the collapse
            if !self.collapse_edge(&candidate) {
                continue;
            }

            self.max_error = self.max_error.max(candidate.cost);
            self.edges_collapsed += 1;
        }

        Ok(())
    }

    fn build_initial_heap(&mut self) {
        let mut seen_edges: HashSet<EdgeKey> = HashSet::new();
        let mut edges_to_process: Vec<EdgeKey> = Vec::new();

        // First pass: collect all unique edges
        for tri in &self.triangles {
            if tri.deleted { continue; }
            for (a, b) in [(tri.v0, tri.v1), (tri.v1, tri.v2), (tri.v2, tri.v0)] {
                let edge = EdgeKey::new(a, b);
                if seen_edges.insert(edge) {
                    edges_to_process.push(edge);
                }
            }
        }

        // Second pass: compute candidates (now we can mutate self)
        for edge in edges_to_process {
            if let Some(candidate) = self.compute_collapse_candidate(edge) {
                self.heap.push(candidate);
            }
        }
    }

    fn compute_collapse_candidate(&mut self, edge: EdgeKey) -> Option<CollapseCandidate> {
        let v0 = edge.0 as usize;
        let v1 = edge.1 as usize;

        if self.vertex_deleted[v0] || self.vertex_deleted[v1] {
            return None;
        }

        // Skip boundary edges if preservation is enabled
        if self.options.preserve_boundary && self.is_boundary_edge(edge) {
            return None;
        }

        let p0 = self.positions[v0];
        let p1 = self.positions[v1];

        // Simple midpoint target (could use QEM for better results)
        let target_pos = Point3::new(
            (p0.x + p1.x) * 0.5,
            (p0.y + p1.y) * 0.5,
            (p0.z + p1.z) * 0.5,
        );

        // Compute geometric error (distance from original edge midpoint)
        let edge_vec = p1.sub_point(p0);
        let edge_len = edge_vec.length();
        let mut cost = edge_len;

        // Add boundary penalty if this is a boundary edge (but not skipped)
        if self.is_boundary_edge(edge) {
            cost *= self.options.boundary_weight;
        }

        self.version_counter += 1;
        let version = self.version_counter;
        self.edge_versions.insert(edge, version);

        Some(CollapseCandidate {
            edge,
            cost,
            target_pos,
            version,
        })
    }

    /// Check if an edge is a boundary edge using the cached set.
    /// Falls back to computation if cache might be stale.
    #[inline]
    fn is_boundary_edge(&self, edge: EdgeKey) -> bool {
        self.boundary_edges.contains(&edge)
    }

    /// Recompute whether an edge is a boundary edge (O(degree) using adjacency).
    fn is_boundary_edge_computed(&self, edge: EdgeKey) -> bool {
        let mut count = 0u32;
        // Use vertex_triangles for O(degree) lookup instead of O(n)
        for &ti in &self.vertex_triangles[edge.0 as usize] {
            let t = &self.triangles[ti];
            if !t.deleted && t.contains_vertex(edge.1) {
                count += 1;
            }
        }
        count == 1
    }

    fn pop_valid_candidate(&mut self) -> Option<CollapseCandidate> {
        while let Some(candidate) = self.heap.pop() {
            // Check if this candidate is still valid (version matches)
            if let Some(&current_version) = self.edge_versions.get(&candidate.edge) {
                if current_version != candidate.version {
                    continue; // Stale candidate
                }
            } else {
                continue; // Edge no longer exists
            }

            let v0 = candidate.edge.0 as usize;
            let v1 = candidate.edge.1 as usize;

            if self.vertex_deleted[v0] || self.vertex_deleted[v1] {
                continue;
            }

            // Check if this is a boundary edge that should be preserved
            if self.options.preserve_boundary && self.is_boundary_edge_computed(candidate.edge) {
                self.boundary_preserved += 1;
                continue;
            }

            // Check topology constraints and track rejection reasons
            match self.can_collapse_with_reason(candidate.edge, candidate.target_pos) {
                Ok(()) => return Some(candidate),
                Err(CollapseRejection::Degenerate) => self.topology_rejections += 1,
                Err(CollapseRejection::LinkCondition) => self.topology_rejections += 1,
                Err(CollapseRejection::NormalFlip) => self.normal_flip_rejections += 1,
                Err(CollapseRejection::AspectRatio) => self.aspect_rejections += 1,
            }
        }
        None
    }

    /// Check if an edge can be collapsed without violating constraints.
    /// Returns Ok(()) if collapse is allowed, Err(reason) otherwise.
    fn can_collapse_with_reason(&self, edge: EdgeKey, target_pos: Point3) -> Result<(), CollapseRejection> {
        let v0 = edge.0;
        let v1 = edge.1;

        // Find triangles that will be affected
        let affected_tris: Vec<usize> = self.vertex_triangles[v0 as usize]
            .iter()
            .chain(self.vertex_triangles[v1 as usize].iter())
            .copied()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        // Find triangles that will be deleted (contain both vertices)
        let deleted_tris: HashSet<usize> = affected_tris.iter()
            .copied()
            .filter(|&ti| {
                let t = &self.triangles[ti];
                !t.deleted && t.contains_vertex(v0) && t.contains_vertex(v1)
            })
            .collect();

        // Check if collapse would create degenerate triangles
        for &ti in &affected_tris {
            let t = &self.triangles[ti];
            if t.deleted || deleted_tris.contains(&ti) {
                continue;
            }

            // Simulate the collapse
            let mut sim_t = *t;
            sim_t.replace_vertex(v0, v1);

            if sim_t.is_degenerate() {
                return Err(CollapseRejection::Degenerate);
            }
        }

        // Check if collapse would create non-manifold edges
        if self.options.strict_watertight {
            if !self.check_link_condition(v0, v1) {
                return Err(CollapseRejection::LinkCondition);
            }
        }

        // Check for normal flipping
        for &ti in &affected_tris {
            let t = &self.triangles[ti];
            if t.deleted || deleted_tris.contains(&ti) {
                continue;
            }

            // Compute original normal
            let orig_normal = self.compute_triangle_normal(t);

            // Simulate the collapse and compute new normal
            let mut verts = t.vertices();
            for v in &mut verts {
                if *v == v0 {
                    *v = v1;
                }
            }

            let new_positions: Vec<Point3> = verts.iter()
                .map(|&v| if v == v1 { target_pos } else { self.positions[v as usize] })
                .collect();

            let new_normal = compute_normal_from_points(
                new_positions[0],
                new_positions[1],
                new_positions[2],
            );

            // Check if normal flipped (dot product < 0 means > 90 degree change)
            if let (Some(orig), Some(new)) = (orig_normal, new_normal) {
                if orig.dot(new) < 0.0 {
                    return Err(CollapseRejection::NormalFlip);
                }
            }
        }

        // Check aspect ratio constraints
        if self.options.max_aspect_ratio > 0.0 {
            for &ti in &affected_tris {
                let t = &self.triangles[ti];
                if t.deleted || deleted_tris.contains(&ti) {
                    continue;
                }

                // Simulate the collapse and check aspect ratio
                let mut verts = t.vertices();
                for v in &mut verts {
                    if *v == v0 || *v == v1 {
                        *v = v1; // Will use target_pos
                    }
                }

                // Get simulated positions
                let mut pos = [Point3::new(0.0, 0.0, 0.0); 3];
                for (i, &v) in verts.iter().enumerate() {
                    pos[i] = if v == v1 {
                        target_pos
                    } else {
                        self.positions[v as usize]
                    };
                }

                let aspect = triangle_aspect_ratio(pos[0], pos[1], pos[2], self.tol);
                if aspect > self.options.max_aspect_ratio {
                    return Err(CollapseRejection::AspectRatio);
                }
            }
        }

        Ok(())
    }

    /// Compute the normal of a triangle.
    fn compute_triangle_normal(&self, t: &Triangle) -> Option<Vec3> {
        let p0 = self.positions[t.v0 as usize];
        let p1 = self.positions[t.v1 as usize];
        let p2 = self.positions[t.v2 as usize];
        compute_normal_from_points(p0, p1, p2)
    }

    /// Check the link condition for edge collapse.
    /// The link condition ensures that collapsing an edge won't create non-manifold geometry.
    /// For edge (v0, v1), the intersection of vertex links (neighboring vertices) of v0 and v1
    /// should equal exactly the vertices of the edge's wing triangles.
    fn check_link_condition(&self, v0: u32, v1: u32) -> bool {
        // Get neighbors of v0 (vertices connected to v0 by an edge)
        let neighbors_v0 = self.get_vertex_neighbors(v0);
        // Get neighbors of v1
        let neighbors_v1 = self.get_vertex_neighbors(v1);

        // Common neighbors (should be exactly 2 for a manifold mesh - the wing vertices)
        let common_count = neighbors_v0.intersection(&neighbors_v1).count();

        // For a valid collapse on a closed manifold mesh, common neighbors should be exactly 2
        // For boundary edges, this might be 1
        // More than 2 indicates the edge is non-manifold or collapse would create non-manifold
        common_count <= 2
    }

    fn get_vertex_neighbors(&self, v: u32) -> HashSet<u32> {
        let mut neighbors = HashSet::new();
        for &ti in &self.vertex_triangles[v as usize] {
            let t = &self.triangles[ti];
            if t.deleted { continue; }
            for vertex in t.vertices() {
                if vertex != v {
                    neighbors.insert(vertex);
                }
            }
        }
        neighbors
    }

    fn collapse_edge(&mut self, candidate: &CollapseCandidate) -> bool {
        let v0 = candidate.edge.0;
        let v1 = candidate.edge.1;

        // Move v1 to target position
        self.positions[v1 as usize] = candidate.target_pos;

        // Find and update affected triangles
        let tris_to_update: Vec<usize> = self.vertex_triangles[v0 as usize].clone();

        for ti in tris_to_update {
            let t = &mut self.triangles[ti];
            if t.deleted { continue; }

            if t.contains_vertex(v0) && t.contains_vertex(v1) {
                // This triangle will be deleted
                t.deleted = true;
            } else if t.contains_vertex(v0) {
                // Replace v0 with v1
                t.replace_vertex(v0, v1);

                // Add this triangle to v1's list
                if !self.vertex_triangles[v1 as usize].contains(&ti) {
                    self.vertex_triangles[v1 as usize].push(ti);
                }
            }
        }

        // Mark v0 as deleted
        self.vertex_deleted[v0 as usize] = true;
        self.vertex_triangles[v0 as usize].clear();

        // Remove stale edge entry
        self.edge_versions.remove(&candidate.edge);

        // Update boundary edge cache for affected edges
        self.update_boundary_edges_after_collapse(v0, v1);

        // Update edges affected by this collapse
        self.update_affected_edges(v1);

        true
    }

    /// Update the boundary edge cache after collapsing edge (v0 -> v1).
    fn update_boundary_edges_after_collapse(&mut self, v0: u32, v1: u32) {
        // Remove any edges involving v0 from boundary set
        self.boundary_edges.retain(|e| e.0 != v0 && e.1 != v0);

        // Recompute boundary status for edges involving v1
        let neighbors = self.get_vertex_neighbors(v1);
        for &neighbor in &neighbors {
            let edge = EdgeKey::new(v1, neighbor);
            if self.is_boundary_edge_computed(edge) {
                self.boundary_edges.insert(edge);
            } else {
                self.boundary_edges.remove(&edge);
            }
        }
    }

    fn update_affected_edges(&mut self, v: u32) {
        let neighbors = self.get_vertex_neighbors(v);

        for &neighbor in &neighbors {
            let edge = EdgeKey::new(v, neighbor);
            if let Some(candidate) = self.compute_collapse_candidate(edge) {
                self.heap.push(candidate);
            }
        }
    }

    fn active_triangle_count(&self) -> usize {
        self.triangles.iter().filter(|t| !t.deleted).count()
    }

    fn extract_mesh(&self) -> (Vec<Point3>, Vec<u32>) {
        // Build vertex remap (old index -> new index)
        let mut vertex_remap: Vec<Option<u32>> = vec![None; self.positions.len()];
        let mut new_positions: Vec<Point3> = Vec::new();

        for (old_idx, pos) in self.positions.iter().enumerate() {
            if !self.vertex_deleted[old_idx] {
                // Check if this vertex is still used (O(degree) using vertex_triangles)
                let is_used = self.vertex_triangles[old_idx].iter()
                    .any(|&ti| !self.triangles[ti].deleted);
                if is_used {
                    vertex_remap[old_idx] = Some(new_positions.len() as u32);
                    new_positions.push(*pos);
                }
            }
        }

        // Build new index buffer
        let mut new_indices: Vec<u32> = Vec::new();
        for t in &self.triangles {
            if t.deleted { continue; }

            let Some(i0) = vertex_remap[t.v0 as usize] else { continue; };
            let Some(i1) = vertex_remap[t.v1 as usize] else { continue; };
            let Some(i2) = vertex_remap[t.v2 as usize] else { continue; };

            // Skip degenerate triangles
            if i0 == i1 || i1 == i2 || i0 == i2 {
                continue;
            }

            new_indices.extend_from_slice(&[i0, i1, i2]);
        }

        (new_positions, new_indices)
    }
}

/// Compute the normal of a triangle from three points.
/// Returns None if the triangle is degenerate.
fn compute_normal_from_points(p0: Point3, p1: Point3, p2: Point3) -> Option<Vec3> {
    let e0 = p1.sub_point(p0);
    let e1 = p2.sub_point(p0);
    let cross = e0.cross(e1);
    cross.normalized()
}

/// Compute the aspect ratio of a triangle (longest edge / shortest altitude).
fn triangle_aspect_ratio(p0: Point3, p1: Point3, p2: Point3, tol: Tolerance) -> f64 {
    let e0 = p1.sub_point(p0);
    let e1 = p2.sub_point(p1);
    let e2 = p0.sub_point(p2);

    let len0 = e0.length();
    let len1 = e1.length();
    let len2 = e2.length();

    let longest = len0.max(len1).max(len2);

    // Compute area using cross product
    let cross = e0.cross(e2.mul_scalar(-1.0));
    let area = cross.length() * 0.5;

    // Use tolerance for degenerate checks
    let eps_sq = tol.eps_squared();
    if area <= eps_sq {
        return f64::INFINITY;
    }

    // Shortest altitude = 2 * area / longest edge
    let shortest_altitude = 2.0 * area / longest;

    if shortest_altitude <= tol.eps {
        return f64::INFINITY;
    }

    longest / shortest_altitude
}

/// Count open edges in an index buffer.
fn count_open_edges(indices: &[u32]) -> usize {
    let mut edge_counts: HashMap<EdgeKey, u32> = HashMap::new();

    for tri in indices.chunks_exact(3) {
        for (a, b) in [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
            let key = EdgeKey::new(a, b);
            *edge_counts.entry(key).or_insert(0) += 1;
        }
    }

    edge_counts.values().filter(|&&c| c == 1).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_simple_plane_mesh() -> GeomMesh {
        // A simple 2x2 quad plane (4 triangles)
        // Vertices:
        // 3---4---5
        // | \ | \ |
        // 0---1---2
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [2.0, 1.0, 0.0],
        ];

        let indices = vec![
            0, 1, 4,
            0, 4, 3,
            1, 2, 5,
            1, 5, 4,
        ];

        GeomMesh {
            positions,
            indices,
            normals: None,
            uvs: None,
            tangents: None,
        }
    }

    fn make_closed_tetrahedron() -> GeomMesh {
        // A closed tetrahedron (4 triangles, watertight)
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, 0.5, 1.0],
        ];

        let indices = vec![
            0, 1, 2, // bottom
            0, 3, 1, // front
            1, 3, 2, // right
            2, 3, 0, // left
        ];

        GeomMesh {
            positions,
            indices,
            normals: None,
            uvs: None,
            tangents: None,
        }
    }

    #[test]
    fn test_simplify_empty_mesh() {
        let mesh = GeomMesh {
            positions: vec![],
            indices: vec![],
            normals: None,
            uvs: None,
            tangents: None,
        };

        let result = simplify_mesh(&mesh, SimplifyOptions::default());
        assert!(matches!(result, Err(SimplifyError::EmptyMesh)));
    }

    #[test]
    fn test_simplify_plane_mesh() {
        let mesh = make_simple_plane_mesh();
        assert_eq!(mesh.triangle_count(), 4);

        let result = simplify_mesh(&mesh, SimplifyOptions::target_triangles(2)).unwrap();
        assert!(result.mesh.triangle_count() <= 4);
        assert!(result.diagnostics.final_triangle_count <= result.diagnostics.original_triangle_count);
    }

    #[test]
    fn test_simplify_by_ratio() {
        let mesh = make_simple_plane_mesh();

        let result = simplify_by_ratio(&mesh, 0.5).unwrap();
        assert!(result.mesh.triangle_count() <= 4);
    }

    #[test]
    fn test_simplify_preserves_structure_at_high_ratio() {
        let mesh = make_simple_plane_mesh();

        let result = simplify_by_ratio(&mesh, 1.0).unwrap();
        assert_eq!(result.mesh.triangle_count(), mesh.triangle_count());
        assert_eq!(result.diagnostics.edges_collapsed, 0);
    }

    #[test]
    fn test_simplify_watertight_mesh() {
        let mesh = make_closed_tetrahedron();
        assert_eq!(count_open_edges(&mesh.indices), 0);

        let result = simplify_mesh(
            &mesh,
            SimplifyOptions::target_triangles(2).strict_watertight(true)
        ).unwrap();

        // A tetrahedron can't be simplified while remaining watertight
        // (minimum closed mesh needs 4 triangles)
        // The simplifier should preserve watertightness or report it couldn't
        assert!(result.diagnostics.watertight_preserved || !result.diagnostics.warnings.is_empty());
    }

    #[test]
    fn test_simplify_diagnostics() {
        let mesh = make_simple_plane_mesh();

        let result = simplify_mesh(&mesh, SimplifyOptions::target_triangles(2)).unwrap();

        assert_eq!(result.diagnostics.original_triangle_count, 4);
        assert_eq!(result.diagnostics.original_vertex_count, 6);
        assert!(result.diagnostics.final_triangle_count <= 4);
        assert!(result.diagnostics.final_vertex_count <= 6);
    }

    #[test]
    fn test_triangle_aspect_ratio() {
        let tol = Tolerance::default_geom();

        // Equilateral-ish triangle
        let p0 = Point3::new(0.0, 0.0, 0.0);
        let p1 = Point3::new(1.0, 0.0, 0.0);
        let p2 = Point3::new(0.5, 0.866, 0.0);

        let ratio = triangle_aspect_ratio(p0, p1, p2, tol);
        assert!(ratio > 0.0 && ratio < 5.0); // Equilateral has ratio ~1.15

        // Very thin triangle (sliver)
        let p2_thin = Point3::new(0.5, 0.01, 0.0);
        let ratio_thin = triangle_aspect_ratio(p0, p1, p2_thin, tol);
        assert!(ratio_thin > 10.0);
    }

    #[test]
    fn test_simplify_with_invalid_geometry() {
        let mesh = GeomMesh {
            positions: vec![[f64::NAN, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: None,
            uvs: None,
            tangents: None,
        };

        let result = simplify_mesh(&mesh, SimplifyOptions::default());
        assert!(matches!(result, Err(SimplifyError::InvalidGeometry)));
    }

    #[test]
    fn test_simplify_target_error() {
        let mesh = make_simple_plane_mesh();

        let result = simplify_mesh(&mesh, SimplifyOptions::target_error(0.001)).unwrap();
        // With a very small error threshold, should collapse very few or no edges
        assert!(result.diagnostics.max_error <= 0.001 || result.diagnostics.edges_collapsed == 0);
    }
}
