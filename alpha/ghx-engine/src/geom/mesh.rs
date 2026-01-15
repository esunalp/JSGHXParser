use super::cache::GeomCache;
use super::diagnostics::GeomMeshDiagnostics;
use super::metrics::{GeomMetrics, TimingBucket};
use super::surface::Surface;
use super::tessellation::{SurfaceTessellationOptions, choose_surface_grid_counts, tessellate_surface_grid_points};
use super::triangulation::triangulate_grid_wrapped;
use super::{Point3, Tolerance, Vec3};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct GeomMesh {
    pub positions: Vec<[f64; 3]>,
    pub indices: Vec<u32>,
    pub uvs: Option<Vec<[f64; 2]>>,
    pub normals: Option<Vec<[f64; 3]>>,
    /// Tangent vectors for normal mapping (computed from UV gradients when available).
    /// Each tangent is a unit vector in the direction of increasing U.
    pub tangents: Option<Vec<[f64; 3]>>,
}

impl GeomMesh {
    /// Create a new mesh with positions and indices only.
    /// 
    /// Use this constructor when you don't need UVs, normals, or tangents.
    #[must_use]
    pub fn new(positions: Vec<[f64; 3]>, indices: Vec<u32>) -> Self {
        Self {
            positions,
            indices,
            uvs: None,
            normals: None,
            tangents: None,
        }
    }

    /// Create a new mesh with positions, indices, UVs, and normals.
    /// 
    /// Tangents will be set to None. Use `with_tangents()` or compute them
    /// separately if needed.
    #[must_use]
    pub fn with_attributes(
        positions: Vec<[f64; 3]>,
        indices: Vec<u32>,
        uvs: Option<Vec<[f64; 2]>>,
        normals: Option<Vec<[f64; 3]>>,
    ) -> Self {
        Self {
            positions,
            indices,
            uvs,
            normals,
            tangents: None,
        }
    }

    #[must_use]
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    /// Returns true if any vertex position contains NaN or Inf values.
    #[must_use]
    pub fn has_invalid_vertices(&self) -> bool {
        self.positions.iter().any(|p| {
            !p[0].is_finite() || !p[1].is_finite() || !p[2].is_finite()
        })
    }

    /// Returns true if all vertex indices are within bounds.
    #[must_use]
    pub fn has_valid_indices(&self) -> bool {
        let n = self.positions.len() as u32;
        self.indices.iter().all(|&i| i < n)
    }

    /// Returns true if indices represent a triangle list.
    #[must_use]
    pub fn has_triangle_indices(&self) -> bool {
        self.indices.len() % 3 == 0
    }

    /// Returns true if all optional vertex attribute buffers match `positions.len()`.
    #[must_use]
    pub fn has_valid_attribute_lengths(&self) -> bool {
        let n = self.positions.len();
        self.uvs.as_ref().map_or(true, |uvs| uvs.len() == n)
            && self.normals.as_ref().map_or(true, |normals| normals.len() == n)
            && self.tangents.as_ref().map_or(true, |tangents| tangents.len() == n)
    }

    pub fn validate(&self) -> Result<(), String> {
        if !self.has_triangle_indices() {
            return Err("mesh indices are not a triangle list (len % 3 != 0)".to_string());
        }
        if self.has_invalid_vertices() {
            return Err("mesh has invalid vertex coordinates (NaN/Inf)".to_string());
        }
        if !self.has_valid_indices() {
            return Err("mesh has out-of-bounds vertex indices".to_string());
        }
        if !self.has_valid_attribute_lengths() {
            return Err("mesh attribute buffers do not match vertex count".to_string());
        }
        Ok(())
    }

    /// Returns the position buffer as a flat slice: `[x0, y0, z0, x1, y1, z1, ...]`.
    ///
    /// This is a zero-copy view over `positions`, useful for wasm/JS adapters that
    /// expect packed numeric buffers.
    #[must_use]
    pub fn positions_flat(&self) -> &[f64] {
        flatten_f64_array_slice::<3>(&self.positions)
    }

    /// Returns the UV buffer as a flat slice: `[u0, v0, u1, v1, ...]`.
    ///
    /// This is a zero-copy view over `uvs` when present.
    #[must_use]
    pub fn uvs_flat(&self) -> Option<&[f64]> {
        self.uvs.as_deref().map(flatten_f64_array_slice::<2>)
    }

    /// Returns the normal buffer as a flat slice: `[nx0, ny0, nz0, nx1, ny1, nz1, ...]`.
    ///
    /// This is a zero-copy view over `normals` when present.
    #[must_use]
    pub fn normals_flat(&self) -> Option<&[f64]> {
        self.normals.as_deref().map(flatten_f64_array_slice::<3>)
    }

    /// Returns the tangent buffer as a flat slice: `[tx0, ty0, tz0, tx1, ty1, tz1, ...]`.
    ///
    /// This is a zero-copy view over `tangents` when present.
    #[must_use]
    pub fn tangents_flat(&self) -> Option<&[f64]> {
        self.tangents.as_deref().map(flatten_f64_array_slice::<3>)
    }

    /// Compute mesh diagnostics by analyzing topology without modifying the mesh.
    ///
    /// This method analyzes the mesh to count open edges, non-manifold edges, and
    /// degenerate triangles without performing any repairs. Use this to get accurate
    /// diagnostics for an already-constructed mesh.
    ///
    /// # Arguments
    /// * `tolerance` - Tolerance for detecting degenerate triangles and coincident vertices
    ///
    /// # Returns
    /// A `GeomMeshDiagnostics` struct with accurate counts for:
    /// - `vertex_count` and `triangle_count` (basic stats)
    /// - `open_edge_count` (boundary edges with only one adjacent triangle)
    /// - `non_manifold_edge_count` (edges with more than two adjacent triangles)
    /// - `degenerate_triangle_count` (zero-area or near-zero-area triangles)
    ///
    /// Note: `welded_vertex_count` and `flipped_triangle_count` will be 0 since
    /// this method does not perform repairs. If you need repair diagnostics, use
    /// the mesh construction functions that include repair passes.
    ///
    /// # Example
    /// ```ignore
    /// use ghx_engine::geom::{GeomMesh, Tolerance};
    ///
    /// let mesh = GeomMesh::new(
    ///     vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
    ///     vec![0, 1, 2],
    /// );
    /// let diag = mesh.compute_diagnostics(Tolerance::default());
    /// assert_eq!(diag.vertex_count, 3);
    /// assert_eq!(diag.triangle_count, 1);
    /// assert_eq!(diag.open_edge_count, 3); // Single triangle has 3 boundary edges
    /// ```
    #[must_use]
    pub fn compute_diagnostics(&self, tolerance: Tolerance) -> GeomMeshDiagnostics {
        let vertex_count = self.positions.len();
        let triangle_count = self.indices.len() / 3;

        // Count edge topology (open and non-manifold edges)
        let (open_edge_count, non_manifold_edge_count) = count_edge_topology(&self.indices);

        // Count degenerate triangles without removing them
        let degenerate_triangle_count = count_degenerate_triangles(
            &self.positions,
            &self.indices,
            tolerance,
        );

        let mut warnings = Vec::new();
        if open_edge_count > 0 {
            warnings.push("mesh has open edges".to_string());
        }
        if non_manifold_edge_count > 0 {
            warnings.push("mesh has non-manifold edges".to_string());
        }
        if degenerate_triangle_count > 0 {
            warnings.push(format!(
                "mesh has {} degenerate triangle(s)",
                degenerate_triangle_count
            ));
        }

        GeomMeshDiagnostics {
            vertex_count,
            triangle_count,
            welded_vertex_count: 0,         // Not applicable - no repair performed
            flipped_triangle_count: 0,      // Not applicable - no repair performed
            degenerate_triangle_count,
            open_edge_count,
            non_manifold_edge_count,
            self_intersection_count: 0,     // Reserved for future use
            boolean_fallback_used: false,
            timing: None,
            warnings,
        }
    }

    /// Compute mesh diagnostics and merge with existing warnings.
    ///
    /// This is a convenience method that computes diagnostics and appends any
    /// existing warnings (e.g., from surface fitting or tessellation) to the
    /// result.
    ///
    /// # Arguments
    /// * `tolerance` - Tolerance for detecting degenerate triangles
    /// * `existing_warnings` - Warnings from prior processing steps to include
    ///
    /// # Returns
    /// A `GeomMeshDiagnostics` struct with topology analysis and merged warnings.
    #[must_use]
    pub fn compute_diagnostics_with_warnings(
        &self,
        tolerance: Tolerance,
        existing_warnings: Vec<String>,
    ) -> GeomMeshDiagnostics {
        let mut diag = self.compute_diagnostics(tolerance);
        // Prepend existing warnings so they appear first (chronological order)
        let mesh_warnings = std::mem::take(&mut diag.warnings);
        diag.warnings = existing_warnings;
        diag.warnings.extend(mesh_warnings);
        diag
    }
}

/// Count degenerate triangles without modifying the mesh.
///
/// A triangle is considered degenerate if its area is below the tolerance threshold.
fn count_degenerate_triangles(
    positions: &[[f64; 3]],
    indices: &[u32],
    tolerance: Tolerance,
) -> usize {
    // Use eps^2 as the area threshold (area is in square units)
    let area_threshold = tolerance.eps_squared() * 0.5;
    let mut count = 0;

    for tri in indices.chunks_exact(3) {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;

        // Skip invalid indices
        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            count += 1; // Count as degenerate
            continue;
        }

        // Check for coincident vertex indices
        if i0 == i1 || i1 == i2 || i0 == i2 {
            count += 1;
            continue;
        }

        let p0 = positions[i0];
        let p1 = positions[i1];
        let p2 = positions[i2];

        // Compute edge vectors
        let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];

        // Cross product gives twice the area
        let cross = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];

        let area_sq = cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2];
        let area = (area_sq * 0.25).sqrt(); // Actual area = 0.5 * |cross|

        if area < area_threshold {
            count += 1;
        }
    }

    count
}

fn flatten_f64_array_slice<const N: usize>(data: &[[f64; N]]) -> &[f64] {
    let count = data.len().checked_mul(N).unwrap_or(0);
    let ptr = data.as_ptr().cast::<f64>();
    // SAFETY: `[[f64; N]]` is stored contiguously, and we compute the element count as `len * N`.
    unsafe { std::slice::from_raw_parts(ptr, count) }
}

#[derive(Debug)]
pub struct GeomContext {
    pub tolerance: Tolerance,
    pub cache: GeomCache,
    pub metrics: GeomMetrics,
}

impl GeomContext {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tolerance: Tolerance::default_geom(),
            cache: GeomCache::default(),
            metrics: GeomMetrics::default(),
        }
    }
}

impl Default for GeomContext {
    fn default() -> Self {
        Self::new()
    }
}

#[must_use]
pub fn mesh_surface(
    surface: &impl Surface,
    u_count: usize,
    v_count: usize,
) -> (GeomMesh, GeomMeshDiagnostics) {
    let mut ctx = GeomContext::new();
    mesh_surface_with_context(surface, u_count, v_count, &mut ctx)
}

#[must_use]
pub fn mesh_surface_adaptive(
    surface: &impl Surface,
    options: SurfaceTessellationOptions,
) -> (GeomMesh, GeomMeshDiagnostics) {
    let mut ctx = GeomContext::new();
    mesh_surface_adaptive_with_context(surface, options, &mut ctx)
}

#[must_use]
pub fn mesh_surface_adaptive_with_context(
    surface: &impl Surface,
    options: SurfaceTessellationOptions,
    ctx: &mut GeomContext,
) -> (GeomMesh, GeomMeshDiagnostics) {
    let (u_count, v_count) = choose_surface_grid_counts(surface, options);
    mesh_surface_with_context(surface, u_count, v_count, ctx)
}

#[must_use]
pub fn mesh_surface_with_context(
    surface: &impl Surface,
    u_count: usize,
    v_count: usize,
    ctx: &mut GeomContext,
) -> (GeomMesh, GeomMeshDiagnostics) {
    ctx.metrics.begin();

    let wrap_u = surface.is_u_closed();
    let wrap_v = surface.is_v_closed();
    let pole_start = surface.pole_v_start();
    let pole_end = surface.pole_v_end();

    let u_count_mesh = if wrap_u { u_count.max(3) } else { u_count.max(2) };
    let mut v_count_mesh = if wrap_v { v_count.max(3) } else { v_count.max(2) };
    if pole_start && pole_end {
        v_count_mesh = v_count_mesh.max(3);
    }

    let ((points, indices), uvs) = if pole_start || pole_end {
        (
            ctx.metrics.time(TimingBucket::SurfaceTessellation, || {
                tessellate_surface_with_poles(surface, u_count_mesh, v_count_mesh)
            }),
            Some(generate_uvs_for_pole_surface(
                u_count_mesh,
                v_count_mesh,
                wrap_u,
                pole_start,
                pole_end,
            )),
        )
    } else {
        let points = ctx.metrics.time(TimingBucket::SurfaceTessellation, || {
            ctx.cache.get_or_insert_surface_grid_points(surface, u_count, v_count, || {
                tessellate_surface_grid_points(surface, u_count_mesh, v_count_mesh)
            })
        });

        let indices = ctx.metrics.time(TimingBucket::Triangulation, || {
            ctx.cache.get_or_insert_triangulated_grid(u_count, v_count, wrap_u, wrap_v, || {
                triangulate_grid_wrapped(u_count_mesh, v_count_mesh, wrap_u, wrap_v)
            })
        });

        (
            (points, indices),
            Some(generate_uvs_for_grid_surface(
                u_count_mesh,
                v_count_mesh,
                wrap_u,
                wrap_v,
            )),
        )
    };

    let (mesh, mut diagnostics) = ctx.metrics.time(TimingBucket::Diagnostics, || {
        let (repaired_points, repaired_uvs, repaired_indices, welded_vertex_count) =
            weld_mesh_vertices(points, uvs.as_deref(), indices, ctx.tolerance);

        let (repaired_indices, degenerate_triangle_count) =
            cull_degenerate_triangles(&repaired_points, &repaired_indices, ctx.tolerance);
        let mut repaired_indices = repaired_indices;

        let flipped_triangle_count = fix_triangle_winding_consistency(&mut repaired_indices);
        let (open_edge_count, non_manifold_edge_count) = count_edge_topology(&repaired_indices);

        let mut warnings = Vec::new();
        if open_edge_count == 0 && non_manifold_edge_count == 0 {
            let volume = signed_volume(&repaired_points, &repaired_indices);
            if volume.is_finite() && volume < 0.0 {
                flip_all_triangles(&mut repaired_indices);
                warnings.push("mesh orientation flipped (outward)".to_string());
            }
        }
        if open_edge_count > 0 {
            warnings.push("mesh has open edges".to_string());
        }
        if non_manifold_edge_count > 0 {
            warnings.push("mesh has non-manifold edges".to_string());
        }

        let normals = compute_smooth_normals(&repaired_points, &repaired_indices);
        let tangents = repaired_uvs.as_ref().map(|uvs| {
            compute_tangents(&repaired_points, &repaired_indices, uvs, &normals)
        });

        let mesh = GeomMesh {
            positions: repaired_points.into_iter().map(|p| p.to_array()).collect(),
            indices: repaired_indices,
            uvs: repaired_uvs,
            normals: Some(normals),
            tangents,
        };

        let diagnostics = GeomMeshDiagnostics {
            vertex_count: mesh.positions.len(),
            triangle_count: mesh.triangle_count(),
            welded_vertex_count,
            flipped_triangle_count,
            degenerate_triangle_count,
            open_edge_count,
            non_manifold_edge_count,
            self_intersection_count: 0,
            boolean_fallback_used: false,
            timing: None,
            warnings,
        };

        (mesh, diagnostics)
    });

    diagnostics.timing = ctx.metrics.end();
    (mesh, diagnostics)
}

pub(crate) fn finalize_mesh(
    points: Vec<Point3>,
    uvs: Option<Vec<[f64; 2]>>,
    indices: Vec<u32>,
    tol: Tolerance,
) -> (GeomMesh, GeomMeshDiagnostics) {
    let (repaired_points, repaired_uvs, repaired_indices, welded_vertex_count) =
        weld_mesh_vertices(points, uvs.as_deref(), indices, tol);

    let (repaired_indices, degenerate_triangle_count) =
        cull_degenerate_triangles(&repaired_points, &repaired_indices, tol);
    let mut repaired_indices = repaired_indices;

    let flipped_triangle_count = fix_triangle_winding_consistency(&mut repaired_indices);
    let (open_edge_count, non_manifold_edge_count) = count_edge_topology(&repaired_indices);

    let mut warnings = Vec::new();
    if open_edge_count == 0 && non_manifold_edge_count == 0 {
        let volume = signed_volume(&repaired_points, &repaired_indices);
        if volume.is_finite() && volume < 0.0 {
            flip_all_triangles(&mut repaired_indices);
            warnings.push("mesh orientation flipped (outward)".to_string());
        }
    }
    if open_edge_count > 0 {
        warnings.push("mesh has open edges".to_string());
    }
    if non_manifold_edge_count > 0 {
        warnings.push("mesh has non-manifold edges".to_string());
    }

    let normals = compute_smooth_normals(&repaired_points, &repaired_indices);
    let tangents = repaired_uvs.as_ref().map(|uvs| {
        compute_tangents(&repaired_points, &repaired_indices, uvs, &normals)
    });

    let mesh = GeomMesh {
        positions: repaired_points.into_iter().map(|p| p.to_array()).collect(),
        indices: repaired_indices,
        uvs: repaired_uvs,
        normals: Some(normals),
        tangents,
    };

    let diagnostics = GeomMeshDiagnostics {
        vertex_count: mesh.positions.len(),
        triangle_count: mesh.triangle_count(),
        welded_vertex_count,
        flipped_triangle_count,
        degenerate_triangle_count,
        open_edge_count,
        non_manifold_edge_count,
        self_intersection_count: 0,
        boolean_fallback_used: false,
        timing: None,
        warnings,
    };

    (mesh, diagnostics)
}

pub(crate) fn compute_smooth_normals_for_mesh(points: &[Point3], indices: &[u32]) -> Vec<[f64; 3]> {
    compute_smooth_normals(points, indices)
}

fn tessellate_surface_with_poles(
    surface: &impl Surface,
    u_count: usize,
    v_count: usize,
) -> (Vec<super::Point3>, Vec<u32>) {
    let (u0, u1) = surface.domain_u();
    let (v0, v1) = surface.domain_v();

    let u_span = u1 - u0;
    let v_span = v1 - v0;

    let wrap_u = surface.is_u_closed();
    let pole_start = surface.pole_v_start();
    let pole_end = surface.pole_v_end();

    let u_count = if wrap_u { u_count.max(3) } else { u_count.max(2) };
    let mut v_count = v_count.max(2);
    if pole_start && pole_end {
        v_count = v_count.max(3);
    }

    let u_denom = if wrap_u {
        u_count as f64
    } else {
        (u_count - 1) as f64
    };
    let v_denom = (v_count - 1) as f64;

    let mut u_params = Vec::with_capacity(u_count);
    for u in 0..u_count {
        let u_u = u as f64 / u_denom;
        let u_t = if u_span.is_finite() && u_span != 0.0 {
            u0 + u_span * u_u
        } else {
            u0
        };
        u_params.push(u_t);
    }

    let mut v_params = Vec::with_capacity(v_count);
    for v in 0..v_count {
        let v_u = v as f64 / v_denom;
        let v_t = if v_span.is_finite() && v_span != 0.0 {
            v0 + v_span * v_u
        } else {
            v0
        };
        v_params.push(v_t);
    }

    let mut points: Vec<super::Point3> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let mut ring_rows = Vec::new();

    if pole_start {
        points.push(surface.point_at(u0, v_params[0]));
    }

    let start_row = if pole_start { 1 } else { 0 };
    let end_row = if pole_end { v_count.saturating_sub(1) } else { v_count };

    for (row_index, &v) in v_params.iter().enumerate().take(end_row).skip(start_row) {
        let row_start = points.len() as u32;
        for &u in &u_params {
            points.push(surface.point_at(u, v));
        }
        ring_rows.push((row_index, row_start));
    }

    if pole_end {
        points.push(surface.point_at(u0, v_params[v_count - 1]));
    }

    let quad_u = if wrap_u { u_count } else { u_count - 1 };
    let next_u = |u: usize| if wrap_u { (u + 1) % u_count } else { u + 1 };

    let ring_count = ring_rows.len();
    if ring_count > 0 {
        if pole_start {
            let pole = 0u32;
            let first_ring_start = 1u32;
            for u in 0..quad_u {
                let i0 = first_ring_start + u as u32;
                let i1 = first_ring_start + next_u(u) as u32;
                indices.extend_from_slice(&[pole, i0, i1]);
            }
        }

        for ring in 0..ring_count.saturating_sub(1) {
            let row0 = ring_rows[ring].1;
            let row1 = ring_rows[ring + 1].1;
            for u in 0..quad_u {
                let u0i = u as u32;
                let u1i = next_u(u) as u32;
                let i0 = row0 + u0i;
                let i1 = row0 + u1i;
                let i2 = row1 + u0i;
                let i3 = row1 + u1i;
                indices.extend_from_slice(&[i0, i1, i2]);
                indices.extend_from_slice(&[i2, i1, i3]);
            }
        }

        if pole_end {
            let pole = (points.len() - 1) as u32;
            let last_ring_start = if pole_start {
                1u32 + (ring_count.saturating_sub(1) * u_count) as u32
            } else {
                (ring_count.saturating_sub(1) * u_count) as u32
            };

            for u in 0..quad_u {
                let i0 = last_ring_start + u as u32;
                let i1 = last_ring_start + next_u(u) as u32;
                indices.extend_from_slice(&[i0, pole, i1]);
            }
        }
    }

    (points, indices)
}

fn generate_uvs_for_grid_surface(
    u_count: usize,
    v_count: usize,
    wrap_u: bool,
    wrap_v: bool,
) -> Vec<[f64; 2]> {
    let u_denom = if wrap_u { u_count as f64 } else { (u_count - 1) as f64 };
    let v_denom = if wrap_v { v_count as f64 } else { (v_count - 1) as f64 };

    let mut uvs = Vec::with_capacity(u_count * v_count);
    for v in 0..v_count {
        let vv = v as f64 / v_denom;
        for u in 0..u_count {
            let uu = u as f64 / u_denom;
            uvs.push([uu, vv]);
        }
    }
    uvs
}

fn generate_uvs_for_pole_surface(
    u_count: usize,
    v_count: usize,
    wrap_u: bool,
    pole_start: bool,
    pole_end: bool,
) -> Vec<[f64; 2]> {
    let u_denom = if wrap_u { u_count as f64 } else { (u_count - 1) as f64 };
    let v_denom = (v_count - 1) as f64;

    let mut uvs = Vec::new();

    if pole_start {
        uvs.push([0.0, 0.0]);
    }

    let start_row = if pole_start { 1 } else { 0 };
    let end_row = if pole_end { v_count.saturating_sub(1) } else { v_count };

    for v in start_row..end_row {
        let vv = v as f64 / v_denom;
        for u in 0..u_count {
            let uu = u as f64 / u_denom;
            uvs.push([uu, vv]);
        }
    }

    if pole_end {
        uvs.push([0.0, 1.0]);
    }

    uvs
}

/// Welds duplicate vertices within tolerance, returning remapped mesh data.
///
/// This function merges vertices that are within the specified tolerance
/// of each other, producing a more compact mesh with shared vertices.
///
/// # Arguments
///
/// * `points` - Input vertex positions
/// * `uvs` - Optional UV coordinates (per-vertex)
/// * `indices` - Triangle indices into the points array
/// * `tol` - Tolerance for vertex matching
///
/// # Returns
///
/// A tuple of:
/// * Welded vertex positions
/// * Welded UV coordinates (if input had UVs)
/// * Remapped triangle indices
/// * Count of vertices that were welded (merged)
pub fn weld_mesh_vertices(
    points: Vec<Point3>,
    uvs: Option<&[[f64; 2]]>,
    indices: Vec<u32>,
    tol: Tolerance,
) -> (Vec<Point3>, Option<Vec<[f64; 2]>>, Vec<u32>, usize) {
    if !tol.eps.is_finite() || tol.eps <= 0.0 {
        let uvs = uvs.map(|src| src.to_vec());
        return (points, uvs, indices, 0);
    }

    use std::collections::HashMap;

    let cell = tol.eps;
    let inv = 1.0 / cell;

    /// Quantize a coordinate value to a grid cell index.
    /// Returns None for non-finite values (NaN/Inf) to prevent incorrect welding.
    fn quantize(value: f64, inv: f64) -> Option<i64> {
        if !value.is_finite() {
            return None;
        }
        let q = (value * inv).floor();
        Some(q.clamp(i64::MIN as f64, i64::MAX as f64) as i64)
    }

    /// Check if a point has all finite coordinates.
    fn is_finite_point(p: Point3) -> bool {
        p.x.is_finite() && p.y.is_finite() && p.z.is_finite()
    }

    let mut buckets: HashMap<(i64, i64, i64), Vec<u32>> = HashMap::new();
    let mut remap: Vec<u32> = Vec::with_capacity(points.len());
    let mut out_points: Vec<Point3> = Vec::with_capacity(points.len());
    let mut out_uvs: Option<Vec<[f64; 2]>> = uvs.map(|_| Vec::with_capacity(points.len()));

    for (i, p) in points.iter().copied().enumerate() {
        // Skip welding for points with non-finite coordinates - they get their own vertex
        let key = match (quantize(p.x, inv), quantize(p.y, inv), quantize(p.z, inv)) {
            (Some(kx), Some(ky), Some(kz)) => Some((kx, ky, kz)),
            _ => None,
        };

        let mut found = None;

        // Only attempt to find matching vertices if this point is finite
        if let Some(key) = key {
            if is_finite_point(p) {

                for dx in -1i64..=1 {
                    for dy in -1i64..=1 {
                        for dz in -1i64..=1 {
                            let lookup = (key.0 + dx, key.1 + dy, key.2 + dz);
                            if let Some(candidates) = buckets.get(&lookup) {
                                for &cand in candidates {
                                    if tol.approx_eq_point3(out_points[cand as usize], p) {
                                        found = Some(cand);
                                        break;
                                    }
                                }
                            }
                            if found.is_some() {
                                break;
                            }
                        }
                        if found.is_some() {
                            break;
                        }
                    }
                    if found.is_some() {
                        break;
                    }
                }
            }
        }

        let out_idx = if let Some(existing) = found {
            existing
        } else {
            let new_idx = out_points.len() as u32;
            out_points.push(p);
            if let (Some(src), Some(dst)) = (uvs, out_uvs.as_mut()) {
                let uv = src.get(i).copied().unwrap_or([0.0, 0.0]);
                dst.push(uv);
            }
            // Only add to spatial bucket if we have a valid key (finite point)
            if let Some(key) = key {
                buckets.entry(key).or_default().push(new_idx);
            }
            new_idx
        };

        remap.push(out_idx);
    }

    let mut out_indices = Vec::with_capacity(indices.len());
    for idx in indices {
        let mapped = remap
            .get(idx as usize)
            .copied()
            .unwrap_or(idx);
        out_indices.push(mapped);
    }

    let welded = points.len().saturating_sub(out_points.len());
    (out_points, out_uvs, out_indices, welded)
}

fn cull_degenerate_triangles(
    points: &[Point3],
    indices: &[u32],
    tol: Tolerance,
) -> (Vec<u32>, usize) {
    let mut out = Vec::with_capacity(indices.len());
    let mut removed = 0usize;

    for tri in indices.chunks_exact(3) {
        let i0 = tri[0];
        let i1 = tri[1];
        let i2 = tri[2];

        if i0 == i1 || i1 == i2 || i0 == i2 {
            removed += 1;
            continue;
        }

        let a = points.get(i0 as usize).copied();
        let b = points.get(i1 as usize).copied();
        let c = points.get(i2 as usize).copied();
        let (Some(a), Some(b), Some(c)) = (a, b, c) else {
            removed += 1;
            continue;
        };

        if tol.approx_eq_point3(a, b) || tol.approx_eq_point3(b, c) || tol.approx_eq_point3(a, c) {
            removed += 1;
            continue;
        }

        let ab = b.sub_point(a);
        let ac = c.sub_point(a);
        let area2 = ab.cross(ac).length_squared();
        if !area2.is_finite() || area2 <= tol.eps_squared() * tol.eps_squared() {
            removed += 1;
            continue;
        }

        out.extend_from_slice(&[i0, i1, i2]);
    }

    (out, removed)
}

pub(crate) fn fix_triangle_winding_consistency(indices: &mut [u32]) -> usize {
    use std::collections::HashMap;

    let tri_count = indices.len() / 3;
    if tri_count == 0 {
        return 0;
    }

    let mut edges: HashMap<(u32, u32), Vec<(usize, bool)>> = HashMap::new();
    edges.reserve(tri_count.saturating_mul(3));

    for t in 0..tri_count {
        let i0 = indices[t * 3];
        let i1 = indices[t * 3 + 1];
        let i2 = indices[t * 3 + 2];
        for (a, b) in [(i0, i1), (i1, i2), (i2, i0)] {
            let (lo, hi, dir) = if a <= b { (a, b, true) } else { (b, a, false) };
            edges.entry((lo, hi)).or_default().push((t, dir));
        }
    }

    let mut visited = vec![false; tri_count];
    let mut flipped = vec![false; tri_count];

    for seed in 0..tri_count {
        if visited[seed] {
            continue;
        }
        visited[seed] = true;
        flipped[seed] = false;
        let mut stack = vec![seed];

        while let Some(t) = stack.pop() {
            let i0 = indices[t * 3];
            let i1 = indices[t * 3 + 1];
            let i2 = indices[t * 3 + 2];

            for (a, b) in [(i0, i1), (i1, i2), (i2, i0)] {
                let (lo, hi, dir_t) = if a <= b { (a, b, true) } else { (b, a, false) };
                let Some(adj) = edges.get(&(lo, hi)) else {
                    continue;
                };
                if adj.len() != 2 {
                    continue;
                }

                let (t0, dir0) = adj[0];
                let (t1, dir1) = adj[1];
                let (other, dir_other) = if t0 == t { (t1, dir1) } else if t1 == t { (t0, dir0) } else { continue };

                let desired = flipped[t] ^ dir_t ^ dir_other ^ true;
                if !visited[other] {
                    visited[other] = true;
                    flipped[other] = desired;
                    stack.push(other);
                }
            }
        }
    }

    let mut flipped_count = 0usize;
    for t in 0..tri_count {
        if flipped[t] {
            indices.swap(t * 3 + 1, t * 3 + 2);
            flipped_count += 1;
        }
    }

    flipped_count
}

fn count_edge_topology(indices: &[u32]) -> (usize, usize) {
    use std::collections::HashMap;

    let mut edge_counts: HashMap<(u32, u32), u32> = HashMap::new();

    for tri in indices.chunks_exact(3) {
        let i0 = tri[0];
        let i1 = tri[1];
        let i2 = tri[2];

        if i0 == i1 || i1 == i2 || i0 == i2 {
            continue;
        }

        let edges = [(i0, i1), (i1, i2), (i2, i0)];
        for (ea, eb) in edges {
            let (lo, hi) = if ea <= eb { (ea, eb) } else { (eb, ea) };
            *edge_counts.entry((lo, hi)).or_insert(0) += 1;
        }
    }

    let mut open_edge_count = 0usize;
    let mut non_manifold_edge_count = 0usize;
    for (_edge, count) in edge_counts {
        if count == 1 {
            open_edge_count += 1;
        } else if count > 2 {
            non_manifold_edge_count += 1;
        }
    }

    (open_edge_count, non_manifold_edge_count)
}

fn flip_all_triangles(indices: &mut [u32]) {
    for tri in indices.chunks_exact_mut(3) {
        tri.swap(1, 2);
    }
}

fn signed_volume(points: &[Point3], indices: &[u32]) -> f64 {
    let mut volume = 0.0;
    for tri in indices.chunks_exact(3) {
        let (Some(a), Some(b), Some(c)) = (
            points.get(tri[0] as usize),
            points.get(tri[1] as usize),
            points.get(tri[2] as usize),
        ) else {
            continue;
        };

        let av = Vec3::new(a.x, a.y, a.z);
        let bv = Vec3::new(b.x, b.y, b.z);
        let cv = Vec3::new(c.x, c.y, c.z);
        volume += av.dot(bv.cross(cv));
    }

    volume / 6.0
}

fn compute_smooth_normals(points: &[Point3], indices: &[u32]) -> Vec<[f64; 3]> {
    let mut normals = vec![[0.0, 0.0, 0.0]; points.len()];

    for tri in indices.chunks_exact(3) {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;

        let (Some(a), Some(b), Some(c)) = (points.get(i0), points.get(i1), points.get(i2)) else {
            continue;
        };

        let abx = b.x - a.x;
        let aby = b.y - a.y;
        let abz = b.z - a.z;

        let acx = c.x - a.x;
        let acy = c.y - a.y;
        let acz = c.z - a.z;

        let nx = aby * acz - abz * acy;
        let ny = abz * acx - abx * acz;
        let nz = abx * acy - aby * acx;

        normals[i0][0] += nx;
        normals[i0][1] += ny;
        normals[i0][2] += nz;

        normals[i1][0] += nx;
        normals[i1][1] += ny;
        normals[i1][2] += nz;

        normals[i2][0] += nx;
        normals[i2][1] += ny;
        normals[i2][2] += nz;
    }

    for n in &mut normals {
        let len2 = n[0] * n[0] + n[1] * n[1] + n[2] * n[2];
        let len = len2.sqrt();
        if len.is_finite() && len > 0.0 {
            let inv = 1.0 / len;
            n[0] *= inv;
            n[1] *= inv;
            n[2] *= inv;
        } else {
            *n = [0.0, 0.0, 1.0];
        }
    }

    normals
}

/// Compute tangent vectors from UV gradients using the MikkTSpace-like algorithm.
/// 
/// Each tangent is a unit vector in the direction of increasing U, orthogonalized
/// against the vertex normal. This is useful for normal mapping in rendering.
/// 
/// Returns a fallback tangent (perpendicular to normal) when UV gradients are
/// degenerate or unavailable.
fn compute_tangents(
    points: &[Point3],
    indices: &[u32],
    uvs: &[[f64; 2]],
    normals: &[[f64; 3]],
) -> Vec<[f64; 3]> {
    let mut tangents = vec![[0.0, 0.0, 0.0]; points.len()];

    for tri in indices.chunks_exact(3) {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;

        let (Some(p0), Some(p1), Some(p2)) = (points.get(i0), points.get(i1), points.get(i2)) else {
            continue;
        };
        let (Some(uv0), Some(uv1), Some(uv2)) = (uvs.get(i0), uvs.get(i1), uvs.get(i2)) else {
            continue;
        };

        let edge1x = p1.x - p0.x;
        let edge1y = p1.y - p0.y;
        let edge1z = p1.z - p0.z;

        let edge2x = p2.x - p0.x;
        let edge2y = p2.y - p0.y;
        let edge2z = p2.z - p0.z;

        // UV deltas
        let du1 = uv1[0] - uv0[0];
        let dv1 = uv1[1] - uv0[1];
        let du2 = uv2[0] - uv0[0];
        let dv2 = uv2[1] - uv0[1];

        // Compute determinant for the UV matrix
        let det = du1 * dv2 - du2 * dv1;
        
        // Skip degenerate UV triangles (zero area in UV space)
        if det.abs() < 1e-12 {
            continue;
        }

        let inv_det = 1.0 / det;

        let tx = inv_det * (dv2 * edge1x - dv1 * edge2x);
        let ty = inv_det * (dv2 * edge1y - dv1 * edge2y);
        let tz = inv_det * (dv2 * edge1z - dv1 * edge2z);

        tangents[i0][0] += tx;
        tangents[i0][1] += ty;
        tangents[i0][2] += tz;

        tangents[i1][0] += tx;
        tangents[i1][1] += ty;
        tangents[i1][2] += tz;

        tangents[i2][0] += tx;
        tangents[i2][1] += ty;
        tangents[i2][2] += tz;
    }

    // Normalize and orthogonalize against vertex normals
    for (i, t) in tangents.iter_mut().enumerate() {
        let n = normals.get(i).copied().unwrap_or([0.0, 0.0, 1.0]);
            
        // Gram-Schmidt orthogonalization: T' = T - (N dot T) * N
        let t_dot_n = t[0] * n[0] + t[1] * n[1] + t[2] * n[2];
        t[0] -= n[0] * t_dot_n;
        t[1] -= n[1] * t_dot_n;
        t[2] -= n[2] * t_dot_n;
            
        // Normalize, with fallback to a vector perpendicular to normal
        let len2 = t[0] * t[0] + t[1] * t[1] + t[2] * t[2];
        let len = len2.sqrt();
        if len.is_finite() && len > 0.0 {
            let inv = 1.0 / len;
            t[0] *= inv;
            t[1] *= inv;
            t[2] *= inv;
            continue;
        }

        let arbitrary = if n[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };

        let cx = n[1] * arbitrary[2] - n[2] * arbitrary[1];
        let cy = n[2] * arbitrary[0] - n[0] * arbitrary[2];
        let cz = n[0] * arbitrary[1] - n[1] * arbitrary[0];

        let clen2 = cx * cx + cy * cy + cz * cz;
        let clen = clen2.sqrt();
        if clen.is_finite() && clen > 0.0 {
            let inv = 1.0 / clen;
            *t = [cx * inv, cy * inv, cz * inv];
        } else {
            *t = [1.0, 0.0, 0.0];
        }
            
    }

    tangents
}

// ---------------------------------------------------------------------------
// Surface Builder Meshing Convenience Functions
// ---------------------------------------------------------------------------

use super::surface::{EdgeSurface, FourPointSurface, NetworkSurface, RuledSurface, SumSurface};

/// Mesh quality options for surface builder meshing.
#[derive(Debug, Clone, Copy)]
pub struct SurfaceBuilderQuality {
    /// Number of subdivisions in U direction (minimum 2)
    pub u_subdivisions: usize,
    /// Number of subdivisions in V direction (minimum 2)
    pub v_subdivisions: usize,
}

impl Default for SurfaceBuilderQuality {
    fn default() -> Self {
        Self {
            u_subdivisions: 10,
            v_subdivisions: 10,
        }
    }
}

impl SurfaceBuilderQuality {
    /// Create a quality setting with specified subdivisions.
    #[must_use]
    pub const fn new(u_subdivisions: usize, v_subdivisions: usize) -> Self {
        Self {
            u_subdivisions,
            v_subdivisions,
        }
    }

    /// Low quality (fewer triangles, faster).
    #[must_use]
    pub const fn low() -> Self {
        Self {
            u_subdivisions: 4,
            v_subdivisions: 4,
        }
    }

    /// High quality (more triangles, smoother).
    #[must_use]
    pub const fn high() -> Self {
        Self {
            u_subdivisions: 20,
            v_subdivisions: 20,
        }
    }

    /// Ultra quality (finest detail, slowest).
    #[must_use]
    pub const fn ultra() -> Self {
        Self {
            u_subdivisions: 32,
            v_subdivisions: 32,
        }
    }

    /// Create quality from min/max subdivision range (uses geometric mean).
    ///
    /// This is useful when converting from general mesh quality settings
    /// that specify subdivision bounds rather than exact counts.
    ///
    /// # Arguments
    /// * `min_subdiv` - Minimum subdivisions (clamped to 2)
    /// * `max_subdiv` - Maximum subdivisions (clamped to min_subdiv..4096)
    ///
    /// # Example
    /// ```
    /// use ghx_engine::geom::SurfaceBuilderQuality;
    /// let q = SurfaceBuilderQuality::from_subdivision_range(4, 256);
    /// assert_eq!(q.u_subdivisions, 32); // geometric mean
    /// ```
    #[must_use]
    pub fn from_subdivision_range(min_subdiv: usize, max_subdiv: usize) -> Self {
        let min = min_subdiv.max(2);
        let max = max_subdiv.max(min).min(4096);
        // Use geometric mean for balanced quality
        let subdiv = ((min as f64) * (max as f64)).sqrt().round() as usize;
        let subdiv = subdiv.clamp(min, max);
        Self {
            u_subdivisions: subdiv,
            v_subdivisions: subdiv,
        }
    }

    /// Create quality from a preset name.
    ///
    /// Accepted names (case-insensitive):
    /// - "low" → 4×4
    /// - "medium" → 10×10 (default)
    /// - "high" → 20×20
    /// - "ultra" → 32×32
    ///
    /// Returns `None` for unrecognized names.
    #[must_use]
    pub fn from_preset_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "low" | "fast" | "preview" => Some(Self::low()),
            "medium" | "default" | "normal" => Some(Self::default()),
            "high" | "fine" | "detailed" => Some(Self::high()),
            "ultra" | "max" | "maximum" => Some(Self::ultra()),
            _ => None,
        }
    }
}

/// Mesh a four-point surface (bilinear patch).
///
/// # Arguments
/// * `p00` - Corner at (u=0, v=0)
/// * `p10` - Corner at (u=1, v=0)
/// * `p01` - Corner at (u=0, v=1)
/// * `p11` - Corner at (u=1, v=1)
/// * `quality` - Mesh subdivision quality
///
/// # Example
/// ```
/// use ghx_engine::geom::{Point3, mesh_four_point_surface, SurfaceBuilderQuality};
///
/// let p00 = Point3::new(0.0, 0.0, 0.0);
/// let p10 = Point3::new(1.0, 0.0, 0.0);
/// let p01 = Point3::new(0.0, 1.0, 0.0);
/// let p11 = Point3::new(1.0, 1.0, 0.0);
///
/// let (mesh, diagnostics) = mesh_four_point_surface(p00, p10, p01, p11, SurfaceBuilderQuality::default());
/// ```
#[must_use]
pub fn mesh_four_point_surface(
    p00: Point3,
    p10: Point3,
    p01: Point3,
    p11: Point3,
    quality: SurfaceBuilderQuality,
) -> (GeomMesh, GeomMeshDiagnostics) {
    let surface = FourPointSurface::new(p00, p10, p01, p11);
    mesh_surface(
        &surface,
        quality.u_subdivisions.max(2),
        quality.v_subdivisions.max(2),
    )
}

/// Mesh a four-point surface from an array of points.
///
/// Returns an error if fewer than 3 points are provided.
/// If exactly 3 points are provided, the fourth is computed as a parallelogram completion.
pub fn mesh_four_point_surface_from_points(
    points: &[Point3],
    quality: SurfaceBuilderQuality,
) -> Result<(GeomMesh, GeomMeshDiagnostics), String> {
    let surface = FourPointSurface::from_points(points)?;
    Ok(mesh_surface(
        &surface,
        quality.u_subdivisions.max(2),
        quality.v_subdivisions.max(2),
    ))
}

/// Mesh a ruled surface from two boundary polylines.
///
/// The polylines are resampled to have equal point counts for consistent interpolation.
///
/// # Example
/// ```
/// use ghx_engine::geom::{Point3, mesh_ruled_surface, SurfaceBuilderQuality};
///
/// let curve_a = vec![
///     Point3::new(0.0, 0.0, 0.0),
///     Point3::new(1.0, 0.0, 0.0),
/// ];
/// let curve_b = vec![
///     Point3::new(0.0, 1.0, 1.0),
///     Point3::new(1.0, 1.0, 1.0),
/// ];
///
/// let result = mesh_ruled_surface(&curve_a, &curve_b, SurfaceBuilderQuality::default());
/// ```
pub fn mesh_ruled_surface(
    curve_a: &[Point3],
    curve_b: &[Point3],
    quality: SurfaceBuilderQuality,
) -> Result<(GeomMesh, GeomMeshDiagnostics), String> {
    let surface = RuledSurface::new(curve_a.to_vec(), curve_b.to_vec())?;
    Ok(mesh_surface(
        &surface,
        quality.u_subdivisions.max(2),
        quality.v_subdivisions.max(2),
    ))
}

/// Mesh an edge surface (Coons patch) from boundary curves.
///
/// # Arguments
/// * `edge_u0` - Bottom edge (v=0), points from u=0 to u=1
/// * `edge_u1` - Top edge (v=1), points from u=0 to u=1
/// * `edge_v0` - Left edge (u=0), points from v=0 to v=1
/// * `edge_v1` - Right edge (u=1), points from v=0 to v=1
/// * `quality` - Mesh subdivision quality
pub fn mesh_edge_surface(
    edge_u0: &[Point3],
    edge_u1: &[Point3],
    edge_v0: &[Point3],
    edge_v1: &[Point3],
    quality: SurfaceBuilderQuality,
) -> Result<(GeomMesh, GeomMeshDiagnostics), String> {
    let surface = EdgeSurface::new(
        edge_u0.to_vec(),
        edge_u1.to_vec(),
        edge_v0.to_vec(),
        edge_v1.to_vec(),
    )?;
    Ok(mesh_surface(
        &surface,
        quality.u_subdivisions.max(2),
        quality.v_subdivisions.max(2),
    ))
}

/// Mesh an edge surface from a list of boundary edges (2-4 edges).
///
/// If 2 edges are provided, creates a ruled surface behavior.
/// If 3-4 edges are provided, constructs a Coons patch.
pub fn mesh_edge_surface_from_edges(
    edges: &[Vec<Point3>],
    quality: SurfaceBuilderQuality,
) -> Result<(GeomMesh, GeomMeshDiagnostics), String> {
    let surface = EdgeSurface::from_edges(edges)?;
    Ok(mesh_surface(
        &surface,
        quality.u_subdivisions.max(2),
        quality.v_subdivisions.max(2),
    ))
}

/// Mesh a sum surface (translational surface) from two profile curves.
///
/// The surface is created by translating `curve_u` along `curve_v`.
/// S(u, v) = curve_u(u) + curve_v(v) - origin
///
/// # Example
/// ```
/// use ghx_engine::geom::{Point3, mesh_sum_surface, SurfaceBuilderQuality};
///
/// // U-profile: a line along X
/// let curve_u = vec![
///     Point3::new(0.0, 0.0, 0.0),
///     Point3::new(1.0, 0.0, 0.0),
/// ];
/// // V-profile: a curve along Y-Z
/// let curve_v = vec![
///     Point3::new(0.0, 0.0, 0.0),
///     Point3::new(0.0, 1.0, 0.5),
///     Point3::new(0.0, 2.0, 0.0),
/// ];
///
/// let result = mesh_sum_surface(&curve_u, &curve_v, SurfaceBuilderQuality::default());
/// ```
pub fn mesh_sum_surface(
    curve_u: &[Point3],
    curve_v: &[Point3],
    quality: SurfaceBuilderQuality,
) -> Result<(GeomMesh, GeomMeshDiagnostics), String> {
    let surface = SumSurface::new(curve_u.to_vec(), curve_v.to_vec())?;
    Ok(mesh_surface(
        &surface,
        quality.u_subdivisions.max(2),
        quality.v_subdivisions.max(2),
    ))
}

/// Mesh a network surface from U-curves and V-curves.
///
/// The curves are expected to form a proper network where U-curves
/// and V-curves intersect. This implementation samples the curves
/// uniformly and creates an interpolation grid.
///
/// # Example
/// ```
/// use ghx_engine::geom::{Point3, mesh_network_surface, SurfaceBuilderQuality};
///
/// let u_curves = vec![
///     vec![Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)],
///     vec![Point3::new(0.0, 1.0, 0.5), Point3::new(1.0, 1.0, 0.5)],
/// ];
/// let v_curves = vec![
///     vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.5)],
///     vec![Point3::new(1.0, 0.0, 0.0), Point3::new(1.0, 1.0, 0.5)],
/// ];
///
/// let result = mesh_network_surface(&u_curves, &v_curves, SurfaceBuilderQuality::default());
/// ```
pub fn mesh_network_surface(
    u_curves: &[Vec<Point3>],
    v_curves: &[Vec<Point3>],
    quality: SurfaceBuilderQuality,
) -> Result<(GeomMesh, GeomMeshDiagnostics), String> {
    let surface = NetworkSurface::new(u_curves, v_curves)?;
    Ok(mesh_surface(
        &surface,
        quality.u_subdivisions.max(2),
        quality.v_subdivisions.max(2),
    ))
}

/// Mesh a network surface from a pre-computed grid of points.
pub fn mesh_network_surface_from_grid(
    grid: Vec<Vec<Point3>>,
    quality: SurfaceBuilderQuality,
) -> Result<(GeomMesh, GeomMeshDiagnostics), String> {
    let surface = NetworkSurface::from_grid(grid)?;
    Ok(mesh_surface(
        &surface,
        quality.u_subdivisions.max(2),
        quality.v_subdivisions.max(2),
    ))
}

// ============================================================================
// Mesh Flip Operations
// ============================================================================

/// Guide direction for mesh flip operations.
///
/// Used to determine whether a mesh should be flipped based on its
/// orientation relative to a reference direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MeshFlipGuide {
    /// Flip if mesh normal doesn't align with this vector.
    Vector(Vec3),
    /// Flip if mesh normal doesn't point toward this point.
    Point(Point3),
}

/// Diagnostics from a mesh flip operation.
#[derive(Debug, Clone, Default)]
pub struct FlipMeshDiagnostics {
    /// Whether the mesh was flipped.
    pub flipped: bool,
    /// Whether a guide was used to determine flip direction.
    pub guide_used: bool,
    /// Dot product between mesh normal and guide direction (before flip).
    /// Negative means normals were pointing away from guide.
    pub dot_before: Option<f64>,
    /// The computed average normal before flip (if available).
    pub average_normal: Option<[f64; 3]>,
    /// Any warnings generated during the operation.
    pub warnings: Vec<String>,
}

/// Flips the orientation of a mesh (reverses triangle winding).
///
/// This function reverses the winding order of all triangles in the mesh,
/// effectively flipping the surface normals. If the mesh has explicit normals,
/// they are also negated.
///
/// # Arguments
///
/// * `mesh` - The mesh to flip
/// * `guide` - Optional guide direction. If provided, the mesh is only flipped
///   if its average normal doesn't align with the guide direction.
///   - `MeshFlipGuide::Vector(v)`: Flip if average normal dot v < 0
///   - `MeshFlipGuide::Point(p)`: Flip if average normal points away from p
///   - `None`: Always flip the mesh
///
/// # Returns
///
/// A tuple containing the flipped mesh and diagnostics.
///
/// # Example
///
/// ```
/// use ghx_engine::geom::{GeomMesh, MeshFlipGuide, Vec3, flip_mesh};
///
/// let mesh = GeomMesh::new(
///     vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
///     vec![0, 1, 2],
/// );
///
/// // Always flip
/// let (flipped, diag) = flip_mesh(mesh.clone(), None);
/// assert!(diag.flipped);
///
/// // Flip only if normal doesn't point up
/// let (flipped, diag) = flip_mesh(mesh, Some(MeshFlipGuide::Vector(Vec3::new(0.0, 0.0, 1.0))));
/// // The CCW triangle has normal pointing up, so it won't be flipped
/// assert!(!diag.flipped);
/// ```
#[must_use]
pub fn flip_mesh(mesh: GeomMesh, guide: Option<MeshFlipGuide>) -> (GeomMesh, FlipMeshDiagnostics) {
    let mut diagnostics = FlipMeshDiagnostics::default();

    // Convert positions to Point3 for normal computation
    let points: Vec<Point3> = mesh.positions.iter().map(|p| Point3::new(p[0], p[1], p[2])).collect();

    // Compute average normal from triangles
    let avg_normal = compute_average_normal(&points, &mesh.indices);
    diagnostics.average_normal = avg_normal.map(|n| [n.x, n.y, n.z]);

    // Determine whether to flip based on guide
    let should_flip = match guide {
        None => true,
        Some(guide_dir) => {
            diagnostics.guide_used = true;
            
            let Some(avg_normal) = avg_normal else {
                diagnostics.warnings.push("flip_mesh: could not compute average normal".to_string());
                return (mesh, diagnostics);
            };

            let desired = match guide_dir {
                MeshFlipGuide::Vector(v) => v.normalized(),
                MeshFlipGuide::Point(p) => {
                    // Compute mesh centroid
                    let centroid = compute_mesh_centroid(&points);
                    p.sub_point(centroid).normalized()
                }
            };

            let Some(desired) = desired else {
                diagnostics.warnings.push("flip_mesh: guide direction is zero".to_string());
                return (mesh, diagnostics);
            };

            let dot = avg_normal.dot(desired);
            diagnostics.dot_before = Some(dot);
            dot < 0.0
        }
    };

    diagnostics.flipped = should_flip;

    if !should_flip {
        return (mesh, diagnostics);
    }

    // Flip triangle winding
    let mut flipped_indices = mesh.indices.clone();
    for tri in flipped_indices.chunks_exact_mut(3) {
        tri.swap(1, 2);
    }

    // Negate normals if present
    let flipped_normals = mesh.normals.map(|normals| {
        normals.iter().map(|n| [-n[0], -n[1], -n[2]]).collect()
    });

    (
        GeomMesh {
            positions: mesh.positions,
            indices: flipped_indices,
            uvs: mesh.uvs,
            normals: flipped_normals,
            tangents: mesh.tangents,
        },
        diagnostics,
    )
}

/// Computes the average normal of a mesh from its triangle faces.
fn compute_average_normal(points: &[Point3], indices: &[u32]) -> Option<Vec3> {
    let mut sum = Vec3::new(0.0, 0.0, 0.0);
    let mut count = 0usize;

    for tri in indices.chunks_exact(3) {
        let (Some(a), Some(b), Some(c)) = (
            points.get(tri[0] as usize),
            points.get(tri[1] as usize),
            points.get(tri[2] as usize),
        ) else {
            continue;
        };

        let ab = b.sub_point(*a);
        let ac = c.sub_point(*a);
        let normal = ab.cross(ac);

        // Weight by triangle area (normal length is 2x area)
        sum = sum.add(normal);
        count += 1;
    }

    if count == 0 {
        return None;
    }

    sum.normalized()
}

/// Computes the centroid of a mesh.
fn compute_mesh_centroid(points: &[Point3]) -> Point3 {
    if points.is_empty() {
        return Point3::ORIGIN;
    }

    let mut sum = Vec3::new(0.0, 0.0, 0.0);
    for p in points {
        sum = sum.add(Vec3::new(p.x, p.y, p.z));
    }

    let n = points.len() as f64;
    Point3::new(sum.x / n, sum.y / n, sum.z / n)
}

// ─────────────────────────────────────────────────────────────────────────────
// Closest Point on Mesh
// ─────────────────────────────────────────────────────────────────────────────

/// Result of finding the closest point on a mesh surface.
#[derive(Debug, Clone, Copy)]
pub struct ClosestPointResult {
    /// The closest point on the mesh surface.
    pub point: Point3,
    /// Squared distance from the query point to the closest point.
    pub distance_squared: f64,
    /// The triangle index (0-based, i.e., triangle 0 uses indices 0,1,2).
    pub triangle_index: usize,
    /// Barycentric coordinates (u, v, w) where w = 1 - u - v.
    /// The closest point = A*(1-u-v) + B*u + C*v for triangle vertices A, B, C.
    pub barycentric: (f64, f64, f64),
    /// Normal at the closest point (interpolated from triangle normal).
    pub normal: Vec3,
}

/// Finds the closest point on a triangle mesh to a given query point.
///
/// This function iterates over all triangles in the mesh and computes the closest
/// point on each triangle, returning the globally closest result. For large meshes,
/// consider using a BVH-accelerated version.
///
/// # Arguments
///
/// * `mesh` - The triangle mesh to query.
/// * `query` - The point to find the closest mesh point for.
///
/// # Returns
///
/// `Some(ClosestPointResult)` if the mesh has valid triangles, `None` otherwise.
///
/// # Example
///
/// ```
/// use ghx_engine::geom::{GeomMesh, Point3, closest_point_on_mesh};
///
/// // Simple triangle mesh (single triangle in XY plane)
/// let mesh = GeomMesh::new(
///     vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
///     vec![0, 1, 2],
/// );
///
/// // Query point above the triangle
/// let query = Point3::new(0.5, 0.3, 1.0);
/// let result = closest_point_on_mesh(&mesh, query).unwrap();
///
/// // Closest point should be on the triangle surface (z = 0)
/// assert!((result.point.z).abs() < 1e-10);
/// assert!((result.distance_squared - 1.0).abs() < 1e-10);
/// ```
#[must_use]
pub fn closest_point_on_mesh(mesh: &GeomMesh, query: Point3) -> Option<ClosestPointResult> {
    if mesh.indices.len() < 3 || mesh.positions.is_empty() {
        return None;
    }

    let points: Vec<Point3> = mesh.positions.iter().map(|p| Point3::from(*p)).collect();
    
    let mut best_result: Option<ClosestPointResult> = None;
    let mut best_dist_sq = f64::INFINITY;

    for (tri_idx, tri) in mesh.indices.chunks_exact(3).enumerate() {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;

        if i0 >= points.len() || i1 >= points.len() || i2 >= points.len() {
            continue;
        }

        let a = points[i0];
        let b = points[i1];
        let c = points[i2];

        let (closest, bary) = closest_point_on_triangle(query, a, b, c);
        let dist_sq = query.sub_point(closest).length_squared();

        if dist_sq < best_dist_sq {
            best_dist_sq = dist_sq;

            // Compute triangle normal
            let ab = b.sub_point(a);
            let ac = c.sub_point(a);
            let normal = ab.cross(ac).normalized().unwrap_or(Vec3::Z);

            best_result = Some(ClosestPointResult {
                point: closest,
                distance_squared: dist_sq,
                triangle_index: tri_idx,
                barycentric: bary,
                normal,
            });
        }
    }

    best_result
}

/// Finds the closest point on a triangle mesh to a given query point,
/// returning only the closest point and distance (simplified API).
///
/// # Arguments
///
/// * `mesh` - The triangle mesh to query.
/// * `query` - The point to find the closest mesh point for.
///
/// # Returns
///
/// A tuple of (closest_point, distance) if the mesh has valid triangles, `None` otherwise.
#[must_use]
pub fn closest_point_on_mesh_simple(mesh: &GeomMesh, query: Point3) -> Option<(Point3, f64)> {
    closest_point_on_mesh(mesh, query).map(|r| (r.point, r.distance_squared.sqrt()))
}

/// Computes the closest point on a triangle to a given query point.
///
/// Uses the algorithm from "Real-Time Collision Detection" by Christer Ericson.
/// This correctly handles all cases: query point projects inside the triangle,
/// onto an edge, or onto a vertex.
///
/// # Arguments
///
/// * `p` - The query point.
/// * `a`, `b`, `c` - The triangle vertices (in counter-clockwise order for standard normal).
///
/// # Returns
///
/// A tuple of:
/// * The closest point on the triangle.
/// * Barycentric coordinates (u, v, w) where the closest point = a*w + b*u + c*v
///   and w = 1 - u - v.
fn closest_point_on_triangle(p: Point3, a: Point3, b: Point3, c: Point3) -> (Point3, (f64, f64, f64)) {
    // Check if P is in vertex region outside A
    let ab = b.sub_point(a);
    let ac = c.sub_point(a);
    let ap = p.sub_point(a);
    
    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);
    
    // Closest to vertex A
    if d1 <= 0.0 && d2 <= 0.0 {
        return (a, (0.0, 0.0, 1.0));
    }

    // Check if P is in vertex region outside B
    let bp = p.sub_point(b);
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);
    
    // Closest to vertex B
    if d3 >= 0.0 && d4 <= d3 {
        return (b, (1.0, 0.0, 0.0));
    }

    // Check if P is in edge region of AB
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        let closest = a.add_vec(ab.mul_scalar(v));
        return (closest, (v, 0.0, 1.0 - v));
    }

    // Check if P is in vertex region outside C
    let cp = p.sub_point(c);
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);
    
    // Closest to vertex C
    if d6 >= 0.0 && d5 <= d6 {
        return (c, (0.0, 1.0, 0.0));
    }

    // Check if P is in edge region of AC
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        let closest = a.add_vec(ac.mul_scalar(w));
        return (closest, (0.0, w, 1.0 - w));
    }

    // Check if P is in edge region of BC
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        let bc = c.sub_point(b);
        let closest = b.add_vec(bc.mul_scalar(w));
        return (closest, (1.0 - w, w, 0.0));
    }

    // P is inside the triangle - compute barycentric coordinates
    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    let u = 1.0 - v - w;
    
    // Closest point is the projection onto the triangle plane
    let closest = Point3::new(
        a.x * u + b.x * v + c.x * w,
        a.y * u + b.y * v + c.y * w,
        a.z * u + b.z * v + c.z * w,
    );
    
    (closest, (v, w, u))
}

// ============================================================================
// Cube-sphere (QuadSphere) mesh generation
// ============================================================================

/// Configuration for cube-sphere mesh generation.
#[derive(Debug, Clone, Copy)]
pub struct CubeSphereOptions {
    /// Center point of the sphere.
    pub center: Point3,
    /// Radius of the sphere.
    pub radius: f64,
    /// Number of subdivisions per cube face edge. Higher values produce
    /// smoother spheres with more triangles.
    /// - 1 subdivision = 8 vertices, 12 triangles (cube)
    /// - 4 subdivisions = 98 vertices, 192 triangles
    /// - 8 subdivisions = 386 vertices, 768 triangles
    /// - 16 subdivisions = 1538 vertices, 3072 triangles
    pub subdivisions: usize,
    /// Orientation frame: X-axis direction on the sphere.
    pub x_axis: Vec3,
    /// Orientation frame: Y-axis direction on the sphere.
    pub y_axis: Vec3,
    /// Orientation frame: Z-axis direction on the sphere (normal/up).
    pub z_axis: Vec3,
}

impl Default for CubeSphereOptions {
    fn default() -> Self {
        Self {
            center: Point3::ORIGIN,
            radius: 1.0,
            subdivisions: 8,
            x_axis: Vec3::new(1.0, 0.0, 0.0),
            y_axis: Vec3::new(0.0, 1.0, 0.0),
            z_axis: Vec3::new(0.0, 0.0, 1.0),
        }
    }
}

impl CubeSphereOptions {
    /// Create options for a unit sphere centered at the origin with given subdivisions.
    #[must_use]
    pub fn unit(subdivisions: usize) -> Self {
        Self {
            subdivisions,
            ..Default::default()
        }
    }

    /// Create options with explicit center, radius, and subdivisions.
    #[must_use]
    pub fn new(center: Point3, radius: f64, subdivisions: usize) -> Self {
        Self {
            center,
            radius,
            subdivisions,
            ..Default::default()
        }
    }

    /// Set the orientation frame from explicit axis vectors.
    #[must_use]
    pub fn with_frame(mut self, x_axis: Vec3, y_axis: Vec3, z_axis: Vec3) -> Self {
        self.x_axis = x_axis;
        self.y_axis = y_axis;
        self.z_axis = z_axis;
        self
    }
}

/// Generates a cube-sphere mesh (spherified cube) with uniform vertex distribution.
///
/// A cube-sphere is created by subdividing each face of a cube into a grid of quads,
/// then projecting (normalizing) each vertex onto the sphere. This produces a much
/// more uniform vertex distribution compared to a traditional UV-sphere, which has
/// vertex compression at the poles.
///
/// # Algorithm
///
/// For each of the 6 cube faces:
/// 1. Generate a grid of `(subdivisions+1) × (subdivisions+1)` points on the face
/// 2. Normalize each point (project onto unit sphere)
/// 3. Scale by radius and translate to center
/// 4. Triangulate each quad in the grid (2 triangles per quad)
///
/// # Advantages over UV-sphere
///
/// - More uniform vertex distribution across the entire sphere
/// - No pole pinching/compression artifacts
/// - Better for physics simulations and uniform sampling
/// - More predictable triangle aspect ratios
///
/// # Arguments
///
/// * `options` - Configuration for center, radius, subdivisions, and orientation
///
/// # Returns
///
/// A tuple containing:
/// * `GeomMesh` - The generated mesh with positions, normals, and UVs
/// * `GeomMeshDiagnostics` - Diagnostics about the mesh quality
///
/// # Example
///
/// ```
/// use ghx_engine::geom::{mesh_cube_sphere, CubeSphereOptions, Point3};
///
/// let options = CubeSphereOptions::new(Point3::ORIGIN, 2.0, 8);
/// let (mesh, diag) = mesh_cube_sphere(options);
/// assert!(mesh.vertex_count() > 0);
/// ```
#[must_use]
pub fn mesh_cube_sphere(options: CubeSphereOptions) -> (GeomMesh, GeomMeshDiagnostics) {
    let mut ctx = GeomContext::new();
    mesh_cube_sphere_with_context(options, &mut ctx)
}

/// Generates a cube-sphere mesh with a provided context for caching and metrics.
#[must_use]
pub fn mesh_cube_sphere_with_context(
    options: CubeSphereOptions,
    ctx: &mut GeomContext,
) -> (GeomMesh, GeomMeshDiagnostics) {
    ctx.metrics.begin();

    let subdivisions = options.subdivisions.max(1);
    let n = subdivisions + 1; // vertices per edge

    // Pre-calculate sizes
    // Each face has n*n vertices, but corner and edge vertices are shared between faces.
    // For a cube-sphere, we handle this by creating vertices per-face and welding.
    let vertices_per_face = n * n;
    let total_vertices_before_weld = 6 * vertices_per_face;
    let quads_per_face = subdivisions * subdivisions;
    let triangles_per_face = quads_per_face * 2;
    let total_triangles = 6 * triangles_per_face;

    let mut positions: Vec<[f64; 3]> = Vec::with_capacity(total_vertices_before_weld);
    let mut normals: Vec<[f64; 3]> = Vec::with_capacity(total_vertices_before_weld);
    let mut uvs: Vec<[f64; 2]> = Vec::with_capacity(total_vertices_before_weld);
    let mut indices: Vec<u32> = Vec::with_capacity(total_triangles * 3);

    // The 6 cube face definitions: (tangent, bitangent, face_normal)
    // Each face is a unit square in the plane perpendicular to face_normal.
    // We parameterize s,t in [-1, 1] to cover the face.
    //
    // Faces are ordered: +X, -X, +Y, -Y, +Z, -Z
    let faces: [(Vec3, Vec3, Vec3); 6] = [
        // +X face: tangent=+Z, bitangent=+Y, normal=+X
        (Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 0.0), Vec3::new(1.0, 0.0, 0.0)),
        // -X face: tangent=-Z, bitangent=+Y, normal=-X
        (Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 1.0, 0.0), Vec3::new(-1.0, 0.0, 0.0)),
        // +Y face: tangent=+X, bitangent=-Z, normal=+Y
        (Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 1.0, 0.0)),
        // -Y face: tangent=+X, bitangent=+Z, normal=-Y
        (Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, -1.0, 0.0)),
        // +Z face: tangent=+X, bitangent=+Y, normal=+Z
        (Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 0.0, 1.0)),
        // -Z face: tangent=-X, bitangent=+Y, normal=-Z
        (Vec3::new(-1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 0.0, -1.0)),
    ];

    // Generate vertices for each face
    for (face_idx, (tangent, bitangent, face_normal)) in faces.iter().enumerate() {
        let base_vertex = positions.len() as u32;

        // Generate n x n vertices for this face
        for j in 0..n {
            let t = (j as f64 / subdivisions as f64) * 2.0 - 1.0; // t in [-1, 1]
            for i in 0..n {
                let s = (i as f64 / subdivisions as f64) * 2.0 - 1.0; // s in [-1, 1]

                // Point on unit cube face
                let cube_point = face_normal
                    .add(tangent.mul_scalar(s))
                    .add(bitangent.mul_scalar(t));

                // Normalize to project onto unit sphere
                let sphere_normal = cube_point.normalized().unwrap_or(*face_normal);

                // Transform by orientation frame and scale by radius
                let local_x = sphere_normal.x;
                let local_y = sphere_normal.y;
                let local_z = sphere_normal.z;

                let world_normal = options.x_axis.mul_scalar(local_x)
                    .add(options.y_axis.mul_scalar(local_y))
                    .add(options.z_axis.mul_scalar(local_z));

                let pos = options.center.add_vec(world_normal.mul_scalar(options.radius));

                positions.push(pos.to_array());
                normals.push(world_normal.to_array());

                // UV mapping: each face gets its own UV region
                // We use a cross-layout UV mapping common for cube maps:
                //       [+Y]
                // [-X][+Z][+X][-Z]
                //       [-Y]
                //
                // But for simplicity, we'll map each face to a 1/3 x 1/2 region
                let (u_offset, v_offset) = match face_idx {
                    0 => (2.0 / 3.0, 1.0 / 2.0), // +X
                    1 => (0.0, 1.0 / 2.0),       // -X
                    2 => (1.0 / 3.0, 0.0),       // +Y (top)
                    3 => (1.0 / 3.0, 1.0),       // -Y (bottom, flipped for layout)
                    4 => (1.0 / 3.0, 1.0 / 2.0), // +Z (front)
                    5 => (1.0, 1.0 / 2.0),       // -Z (back, wraps or separate)
                    _ => (0.0, 0.0),
                };

                // Normalize s,t from [-1,1] to [0,1] for UV
                let u_local = (s + 1.0) / 2.0;
                let v_local = (t + 1.0) / 2.0;

                // Scale to face region (1/3 width, 1/2 height per face)
                let u = u_offset + u_local / 3.0;
                let v = v_offset + v_local / 2.0;

                uvs.push([u.clamp(0.0, 1.0), v.clamp(0.0, 1.0)]);
            }
        }

        // Generate triangles for this face (2 per quad)
        for j in 0..subdivisions {
            for i in 0..subdivisions {
                // Quad corners (CCW winding when viewed from outside)
                let v00 = base_vertex + (j * n + i) as u32;
                let v10 = base_vertex + (j * n + i + 1) as u32;
                let v01 = base_vertex + ((j + 1) * n + i) as u32;
                let v11 = base_vertex + ((j + 1) * n + i + 1) as u32;

                // Two triangles per quad (CCW winding)
                // Triangle 1: v00 -> v10 -> v11
                indices.push(v00);
                indices.push(v10);
                indices.push(v11);

                // Triangle 2: v00 -> v11 -> v01
                indices.push(v00);
                indices.push(v11);
                indices.push(v01);
            }
        }
    }

    // Weld duplicate vertices at cube edges/corners
    let points: Vec<Point3> = positions.iter().map(|p| Point3::new(p[0], p[1], p[2])).collect();
    let (welded_points, welded_uvs, welded_indices, welded_count) =
        weld_mesh_vertices(points, Some(&uvs), indices, ctx.tolerance);

    // Rebuild positions and normals from welded points
    // For a sphere, normals are just the normalized direction from center to vertex
    let welded_positions: Vec<[f64; 3]> = welded_points.iter().map(|p| p.to_array()).collect();
    let welded_normals: Vec<[f64; 3]> = welded_points
        .iter()
        .map(|p| {
            let dir = p.sub_point(options.center);
            dir.normalized()
                .unwrap_or(Vec3::new(0.0, 0.0, 1.0))
                .to_array()
        })
        .collect();

    let mesh = GeomMesh::with_attributes(
        welded_positions,
        welded_indices,
        welded_uvs,
        Some(welded_normals),
    );

    let mut diag = mesh.compute_diagnostics(ctx.tolerance);
    diag.welded_vertex_count = welded_count;

    let _ = ctx.metrics.end();

    (mesh, diag)
}
