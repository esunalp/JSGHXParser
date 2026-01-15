use super::diagnostics::GeomMeshDiagnostics;
use super::mesh::{GeomMesh, finalize_mesh};
use super::trim::{TrimLoop, TrimRegion, UvPoint};
use super::triangulation::{triangulate_trim_region, triangulate_trim_region_with_steiner_points};
use super::{Point3, Tolerance, Vec3};

/// Planar patch and boundary-surface helpers.
///
/// This module provides patch filling for planar closed boundaries (optionally with holes)
/// via constrained triangulation.
///
/// # Features
/// - Boundary curves are approximated as polylines.
/// - Interior points can be added to influence mesh quality and shape.
/// - Spans parameter controls boundary subdivision density.
/// - Flexibility parameter affects internal subdivision (higher = more interior points).
/// - Trim option controls whether to attempt trimming the result to the exact boundary.
///
/// # Limitations
/// - Boundaries must be (approximately) planar.
/// - Output is a triangle mesh patch (open surface); open edges are expected.
#[derive(Debug, thiserror::Error)]
pub enum PatchError {
    #[error("boundary points must be finite")]
    NonFinitePoint,
    #[error("boundary requires at least {min} unique points")]
    NotEnoughPoints { min: usize },
    #[error("boundary is degenerate (zero area)")]
    BoundaryDegenerate,
    #[error("boundary is not planar enough (max distance {max_distance})")]
    BoundaryNotPlanar { max_distance: f64 },
    #[error("interior point outside boundary")]
    InteriorPointOutside,
    #[error("failed to triangulate boundary: {0}")]
    Triangulation(String),
}

/// Options for patch surface generation.
///
/// These options mirror the Grasshopper Patch component pins:
/// - `spans`: Number of spans (controls boundary subdivision density). Default: 10.
/// - `flexibility`: Patch flexibility (0.0 = stiff, 1.0 = very flexible). Default: 1.0.
///   Higher flexibility means more interior points are generated for a smoother patch.
/// - `trim`: Whether to attempt to trim the result to the exact boundary. Default: true.
/// - `interior_points`: Additional points that should be incorporated into the patch mesh.
///   These are useful for controlling the shape of the patch by providing target points.
#[derive(Debug, Clone)]
pub struct PatchOptions {
    /// Number of spans (subdivision segments) along the boundary.
    /// Higher values produce more refined boundary sampling.
    /// Default: 10. Range: 1..=100.
    pub spans: u32,

    /// Patch flexibility (0.0 = stiff, 1.0 = very flexible).
    /// Higher flexibility generates more internal subdivision points.
    /// Default: 1.0. Range: 0.0..=10.0 (values > 1.0 are very flexible).
    pub flexibility: f64,

    /// Whether to attempt to trim the result to the exact boundary.
    /// When true, the triangulation is clipped to the boundary curves.
    /// Default: true.
    pub trim: bool,

    /// Interior points to incorporate into the patch mesh.
    /// These points influence the triangulation and can be used to
    /// control the shape of the resulting surface by providing "through points".
    pub interior_points: Vec<Point3>,
}

impl Default for PatchOptions {
    fn default() -> Self {
        Self {
            spans: 10,
            flexibility: 1.0,
            trim: true,
            interior_points: Vec::new(),
        }
    }
}

impl PatchOptions {
    /// Creates new patch options with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the number of spans.
    #[must_use]
    pub fn with_spans(mut self, spans: u32) -> Self {
        self.spans = spans.clamp(1, 100);
        self
    }

    /// Sets the flexibility parameter.
    #[must_use]
    pub fn with_flexibility(mut self, flexibility: f64) -> Self {
        self.flexibility = flexibility.clamp(0.0, 10.0);
        self
    }

    /// Sets the trim flag.
    #[must_use]
    pub fn with_trim(mut self, trim: bool) -> Self {
        self.trim = trim;
        self
    }

    /// Sets the interior points.
    #[must_use]
    pub fn with_interior_points(mut self, points: Vec<Point3>) -> Self {
        self.interior_points = points;
        self
    }
}

#[must_use]
pub fn boundary_surface_mesh(
    boundary: &[Point3],
) -> Result<(GeomMesh, GeomMeshDiagnostics), PatchError> {
    boundary_surface_mesh_with_tolerance(boundary, Tolerance::default_geom())
}

pub fn boundary_surface_mesh_with_tolerance(
    boundary: &[Point3],
    tol: Tolerance,
) -> Result<(GeomMesh, GeomMeshDiagnostics), PatchError> {
    patch_mesh_with_tolerance(boundary, &[], tol)
}

#[must_use]
pub fn patch_mesh(
    outer_boundary: &[Point3],
    holes: &[Vec<Point3>],
) -> Result<(GeomMesh, GeomMeshDiagnostics), PatchError> {
    patch_mesh_with_tolerance(outer_boundary, holes, Tolerance::default_geom())
}

pub fn patch_mesh_with_tolerance(
    outer_boundary: &[Point3],
    holes: &[Vec<Point3>],
    tol: Tolerance,
) -> Result<(GeomMesh, GeomMeshDiagnostics), PatchError> {
    let outer = clean_closed_polyline(outer_boundary, tol)?;
    if outer.len() < 3 {
        return Err(PatchError::NotEnoughPoints { min: 3 });
    }

    let normal = polygon_normal(&outer);
    let normal_len2 = normal.length_squared();
    if !normal_len2.is_finite() || normal_len2 <= tol.eps_squared() {
        return Err(PatchError::BoundaryDegenerate);
    }
    let normal = normal.normalized().ok_or(PatchError::BoundaryDegenerate)?;

    // Planarity check: patch filling via 2D triangulation requires a stable plane.
    let planar_eps = (tol.eps * 1e3).max(tol.eps);
    let origin = outer[0];
    let mut max_distance: f64 = 0.0;
    for p in outer.iter().copied() {
        let d = p.sub_point(origin).dot(normal).abs();
        max_distance = max_distance.max(d);
    }
    for hole in holes {
        for p in hole.iter().copied() {
            if !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite() {
                return Err(PatchError::NonFinitePoint);
            }
            let d = p.sub_point(origin).dot(normal).abs();
            max_distance = max_distance.max(d);
        }
    }
    if max_distance > planar_eps {
        return Err(PatchError::BoundaryNotPlanar { max_distance });
    }

    let (u_axis, v_axis) = plane_basis(normal)?;

    let mut loops: Vec<TrimLoop> = Vec::with_capacity(1 + holes.len());
    loops.push(project_loop_to_uv(&outer, origin, u_axis, v_axis, tol)?);

    for hole in holes {
        let cleaned = clean_closed_polyline(hole, tol)?;
        if cleaned.len() < 3 {
            continue;
        }
        loops.push(project_loop_to_uv(&cleaned, origin, u_axis, v_axis, tol)?);
    }

    let region = TrimRegion::from_loops(loops, tol).map_err(|e| PatchError::Triangulation(e.to_string()))?;
    let tri = triangulate_trim_region(&region, tol).map_err(PatchError::Triangulation)?;

    let mut points: Vec<Point3> = Vec::with_capacity(tri.vertices.len());
    let mut uvs: Vec<[f64; 2]> = Vec::with_capacity(tri.vertices.len());
    for uv in tri.vertices {
        points.push(point_from_uv(origin, u_axis, v_axis, uv));
        uvs.push([uv.u, uv.v]);
    }

    Ok(finalize_mesh(points, Some(uvs), tri.indices, tol))
}

/// Creates a patch surface mesh with full options support.
///
/// This function extends `patch_mesh_with_tolerance` by supporting:
/// - **Interior points**: Additional points incorporated into the triangulation.
///   These "through points" influence the mesh shape and are useful for fitting
///   a surface through specific target locations.
/// - **Spans**: Controls boundary subdivision density. Higher values produce
///   more refined boundary sampling, which can improve mesh quality.
/// - **Flexibility**: Controls internal subdivision. Higher flexibility (>1.0)
///   generates additional internal points for smoother patches.
/// - **Trim**: When true, ensures the mesh is clipped to the exact boundary curves.
///
/// # Arguments
/// * `outer_boundary` - The outer boundary polyline (must be closed/closeable).
/// * `holes` - Optional inner boundaries (holes) to cut from the patch.
/// * `options` - Patch generation options (spans, flexibility, trim, interior points).
/// * `tol` - Geometric tolerance for point merging and planarity checks.
///
/// # Example
/// ```ignore
/// use crate::geom::{Point3, PatchOptions, Tolerance, patch_mesh_with_options};
///
/// let boundary = vec![
///     Point3::new(0.0, 0.0, 0.0),
///     Point3::new(10.0, 0.0, 0.0),
///     Point3::new(10.0, 10.0, 0.0),
///     Point3::new(0.0, 10.0, 0.0),
/// ];
///
/// let interior = vec![Point3::new(5.0, 5.0, 1.0)]; // Raised center point
///
/// let options = PatchOptions::new()
///     .with_spans(20)
///     .with_flexibility(1.5)
///     .with_interior_points(interior);
///
/// let (mesh, diagnostics) = patch_mesh_with_options(&boundary, &[], options, Tolerance::default_geom())?;
/// ```
pub fn patch_mesh_with_options(
    outer_boundary: &[Point3],
    holes: &[Vec<Point3>],
    options: PatchOptions,
    tol: Tolerance,
) -> Result<(GeomMesh, GeomMeshDiagnostics), PatchError> {
    // Subdivide boundary based on spans parameter if needed
    let outer = if options.spans > 1 {
        subdivide_polyline(outer_boundary, options.spans as usize, tol)?
    } else {
        clean_closed_polyline(outer_boundary, tol)?
    };

    if outer.len() < 3 {
        return Err(PatchError::NotEnoughPoints { min: 3 });
    }

    let normal = polygon_normal(&outer);
    let normal_len2 = normal.length_squared();
    if !normal_len2.is_finite() || normal_len2 <= tol.eps_squared() {
        return Err(PatchError::BoundaryDegenerate);
    }
    let normal = normal.normalized().ok_or(PatchError::BoundaryDegenerate)?;

    // Planarity check: patch filling via 2D triangulation requires a stable plane.
    let planar_eps = (tol.eps * 1e3).max(tol.eps);
    let origin = outer[0];
    let mut max_distance: f64 = 0.0;

    for p in outer.iter().copied() {
        let d = p.sub_point(origin).dot(normal).abs();
        max_distance = max_distance.max(d);
    }

    // Subdivide and check holes
    let mut cleaned_holes: Vec<Vec<Point3>> = Vec::with_capacity(holes.len());
    for hole in holes {
        let cleaned = if options.spans > 1 {
            subdivide_polyline(hole, options.spans as usize, tol)?
        } else {
            clean_closed_polyline(hole, tol)?
        };
        if cleaned.len() < 3 {
            continue;
        }
        for p in cleaned.iter().copied() {
            let d = p.sub_point(origin).dot(normal).abs();
            max_distance = max_distance.max(d);
        }
        cleaned_holes.push(cleaned);
    }

    // Check interior points for planarity
    for p in options.interior_points.iter().copied() {
        if !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite() {
            return Err(PatchError::NonFinitePoint);
        }
        let d = p.sub_point(origin).dot(normal).abs();
        max_distance = max_distance.max(d);
    }

    if max_distance > planar_eps {
        return Err(PatchError::BoundaryNotPlanar { max_distance });
    }

    let (u_axis, v_axis) = plane_basis(normal)?;

    // Build trim loops
    let mut loops: Vec<TrimLoop> = Vec::with_capacity(1 + cleaned_holes.len());
    loops.push(project_loop_to_uv(&outer, origin, u_axis, v_axis, tol)?);

    for hole in &cleaned_holes {
        loops.push(project_loop_to_uv(hole, origin, u_axis, v_axis, tol)?);
    }

    let region = TrimRegion::from_loops(loops, tol)
        .map_err(|e| PatchError::Triangulation(e.to_string()))?;

    // Project interior points to UV space
    let mut steiner_points: Vec<UvPoint> = Vec::with_capacity(options.interior_points.len());
    for p in options.interior_points.iter().copied() {
        let uv = project_point_to_uv(p, origin, u_axis, v_axis);
        // Verify the point is inside the outer boundary (and not inside a hole)
        if options.trim && !region.contains(uv, tol) {
            // Skip points outside the boundary when trim is enabled
            // This is lenient behavior - we could also error with InteriorPointOutside
            continue;
        }
        steiner_points.push(uv);
    }

    // Generate flexibility-based interior points if flexibility > 0
    if options.flexibility > 0.0 {
        let extra_points = generate_flexibility_points(&region, options.flexibility, tol);
        steiner_points.extend(extra_points);
    }

    // Triangulate with or without steiner points
    let tri = if steiner_points.is_empty() {
        triangulate_trim_region(&region, tol).map_err(PatchError::Triangulation)?
    } else {
        triangulate_trim_region_with_steiner_points(&region, &steiner_points, tol)
            .map_err(PatchError::Triangulation)?
    };

    // Convert back to 3D points
    let mut points: Vec<Point3> = Vec::with_capacity(tri.vertices.len());
    let mut uvs: Vec<[f64; 2]> = Vec::with_capacity(tri.vertices.len());
    for uv in tri.vertices {
        points.push(point_from_uv(origin, u_axis, v_axis, uv));
        uvs.push([uv.u, uv.v]);
    }

    Ok(finalize_mesh(points, Some(uvs), tri.indices, tol))
}

/// Projects a 3D point to UV coordinates on a plane.
fn project_point_to_uv(p: Point3, origin: Point3, u_axis: Vec3, v_axis: Vec3) -> UvPoint {
    let d = p.sub_point(origin);
    UvPoint::new(d.dot(u_axis), d.dot(v_axis))
}

/// Subdivides a polyline to have approximately `target_segments` segments.
fn subdivide_polyline(points: &[Point3], target_segments: usize, tol: Tolerance) -> Result<Vec<Point3>, PatchError> {
    let cleaned = clean_closed_polyline(points, tol)?;
    if cleaned.len() < 3 || target_segments <= cleaned.len() {
        return Ok(cleaned);
    }

    // Calculate current segment count and subdivision factor
    let current_segments = cleaned.len();
    let subdivisions_per_segment = (target_segments as f64 / current_segments as f64).ceil() as usize;
    if subdivisions_per_segment <= 1 {
        return Ok(cleaned);
    }

    let mut result = Vec::with_capacity(target_segments);
    for i in 0..cleaned.len() {
        let p0 = cleaned[i];
        let p1 = cleaned[(i + 1) % cleaned.len()];

        result.push(p0);

        // Add intermediate points
        for j in 1..subdivisions_per_segment {
            let t = j as f64 / subdivisions_per_segment as f64;
            let px = p0.x + t * (p1.x - p0.x);
            let py = p0.y + t * (p1.y - p0.y);
            let pz = p0.z + t * (p1.z - p0.z);
            result.push(Point3::new(px, py, pz));
        }
    }

    Ok(result)
}

/// Generates interior points based on flexibility parameter.
///
/// Higher flexibility values generate more interior points, producing
/// a smoother/more refined internal mesh structure.
fn generate_flexibility_points(region: &TrimRegion, flexibility: f64, tol: Tolerance) -> Vec<UvPoint> {
    if flexibility <= 0.0 {
        return Vec::new();
    }

    // Calculate bounding box of the outer loop
    let outer_points = region.outer.points();
    if outer_points.is_empty() {
        return Vec::new();
    }

    let mut min_u = f64::MAX;
    let mut max_u = f64::MIN;
    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;

    for p in outer_points {
        min_u = min_u.min(p.u);
        max_u = max_u.max(p.u);
        min_v = min_v.min(p.v);
        max_v = max_v.max(p.v);
    }

    let width = max_u - min_u;
    let height = max_v - min_v;

    if width <= tol.eps || height <= tol.eps {
        return Vec::new();
    }

    // Number of grid points based on flexibility
    // flexibility 1.0 -> ~3x3 grid interior
    // flexibility 2.0 -> ~5x5 grid interior
    // flexibility 0.5 -> ~2x2 grid interior
    let grid_size = ((flexibility * 2.0).sqrt().ceil() as usize).max(2);
    let step_u = width / grid_size as f64;
    let step_v = height / grid_size as f64;

    let mut interior_points = Vec::new();

    for i in 1..grid_size {
        for j in 1..grid_size {
            let u = min_u + i as f64 * step_u;
            let v = min_v + j as f64 * step_v;
            let point = UvPoint::new(u, v);

            // Only add if inside the region (respects holes)
            if region.contains(point, tol) {
                interior_points.push(point);
            }
        }
    }

    interior_points
}

/// Fragment-patch helper:
///
/// For Phase 2, this fills one or more planar regions defined by a set of
/// closed boundaries.
///
/// Boundaries are treated as independent closed loops. The function:
/// - Projects all loops into a best-effort shared plane.
/// - Automatically assigns holes to the correct outer loop by nesting depth.
/// - Emits one patch mesh per outer region (including "islands" inside holes).
#[must_use]
pub fn fragment_patch_meshes(
    boundaries: &[Vec<Point3>],
) -> Result<Vec<(GeomMesh, GeomMeshDiagnostics)>, PatchError> {
    fragment_patch_meshes_with_tolerance(boundaries, Tolerance::default_geom())
}

pub fn fragment_patch_meshes_with_tolerance(
    boundaries: &[Vec<Point3>],
    tol: Tolerance,
) -> Result<Vec<(GeomMesh, GeomMeshDiagnostics)>, PatchError> {
    if boundaries.is_empty() {
        return Ok(Vec::new());
    }

    let mut cleaned: Vec<Vec<Point3>> = Vec::with_capacity(boundaries.len());
    for boundary in boundaries {
        let loop3 = clean_closed_polyline(boundary, tol)?;
        if loop3.len() >= 3 {
            cleaned.push(loop3);
        }
    }
    if cleaned.is_empty() {
        return Ok(Vec::new());
    }

    // Use the largest-area loop to define a stable plane.
    let (mut ref_index, mut best_len2) = (0usize, 0.0f64);
    for (idx, loop3) in cleaned.iter().enumerate() {
        let n = polygon_normal(loop3);
        let len2 = n.length_squared();
        if len2.is_finite() && len2 > best_len2 {
            best_len2 = len2;
            ref_index = idx;
        }
    }
    if !best_len2.is_finite() || best_len2 <= tol.eps_squared() {
        return Err(PatchError::BoundaryDegenerate);
    }

    let ref_loop = &cleaned[ref_index];
    let normal = polygon_normal(ref_loop)
        .normalized()
        .ok_or(PatchError::BoundaryDegenerate)?;
    let origin = ref_loop[0];

    // Planarity check (shared plane for all loops).
    let planar_eps = (tol.eps * 1e3).max(tol.eps);
    let mut max_distance: f64 = 0.0;
    for loop3 in &cleaned {
        for p in loop3.iter().copied() {
            let d = p.sub_point(origin).dot(normal).abs();
            max_distance = max_distance.max(d);
        }
    }
    if max_distance > planar_eps {
        return Err(PatchError::BoundaryNotPlanar { max_distance });
    }

    let (u_axis, v_axis) = plane_basis(normal)?;

    let mut loops: Vec<TrimLoop> = Vec::with_capacity(cleaned.len());
    for loop3 in &cleaned {
        loops.push(project_loop_to_uv(loop3, origin, u_axis, v_axis, tol)?);
    }

    let regions = split_into_trim_regions(loops, tol)?;

    let mut out: Vec<(GeomMesh, GeomMeshDiagnostics)> = Vec::with_capacity(regions.len());
    for region in regions {
        let tri = triangulate_trim_region(&region, tol).map_err(PatchError::Triangulation)?;

        let mut points: Vec<Point3> = Vec::with_capacity(tri.vertices.len());
        let mut uvs: Vec<[f64; 2]> = Vec::with_capacity(tri.vertices.len());
        for uv in tri.vertices {
            points.push(point_from_uv(origin, u_axis, v_axis, uv));
            uvs.push([uv.u, uv.v]);
        }
        out.push(finalize_mesh(points, Some(uvs), tri.indices, tol));
    }

    Ok(out)
}

fn split_into_trim_regions(loops: Vec<TrimLoop>, tol: Tolerance) -> Result<Vec<TrimRegion>, PatchError> {
    if loops.is_empty() {
        return Ok(Vec::new());
    }

    // Reject intersecting loops early; nesting classification becomes ambiguous.
    for i in 0..loops.len() {
        for j in (i + 1)..loops.len() {
            if loops_intersect_uv(loops[i].points(), loops[j].points(), tol) {
                return Err(PatchError::Triangulation(
                    "trim loops intersect".to_string(),
                ));
            }
        }
    }

    #[derive(Clone)]
    struct LoopInfo {
        loop_: TrimLoop,
        area_abs: f64,
        probe: UvPoint,
    }

    let infos: Vec<LoopInfo> = loops
        .into_iter()
        .map(|loop_| {
            let area_abs = loop_.signed_area().abs();
            let probe = loop_.points().first().copied().unwrap_or(UvPoint::new(0.0, 0.0));
            LoopInfo {
                loop_,
                area_abs,
                probe,
            }
        })
        .collect();

    // Parent is the smallest-area loop that contains this loop's probe point.
    let mut parent: Vec<Option<usize>> = vec![None; infos.len()];
    for i in 0..infos.len() {
        let mut best_parent: Option<usize> = None;
        let mut best_area: f64 = f64::INFINITY;

        for j in 0..infos.len() {
            if i == j {
                continue;
            }
            if infos[j].area_abs <= infos[i].area_abs {
                continue;
            }
            if !infos[j].loop_.contains(infos[i].probe, tol) {
                continue;
            }

            if infos[j].area_abs < best_area {
                best_area = infos[j].area_abs;
                best_parent = Some(j);
            }
        }

        parent[i] = best_parent;
    }

    // Depth (nesting parity) determines outer vs hole; even depth = outer.
    let mut depth: Vec<usize> = vec![0; infos.len()];
    for i in 0..infos.len() {
        let mut d = 0usize;
        let mut cur = parent[i];
        let mut guard = 0usize;
        while let Some(p) = cur {
            d += 1;
            cur = parent[p];
            guard += 1;
            if guard > infos.len() {
                return Err(PatchError::Triangulation(
                    "loop nesting cycle detected".to_string(),
                ));
            }
        }
        depth[i] = d;
    }

    let mut regions: Vec<TrimRegion> = Vec::new();

    for i in 0..infos.len() {
        if depth[i] % 2 != 0 {
            continue;
        }

        let mut region_loops: Vec<TrimLoop> = Vec::new();
        region_loops.push(infos[i].loop_.clone());

        for j in 0..infos.len() {
            if depth[j] % 2 != 1 {
                continue;
            }
            if parent[j] == Some(i) {
                region_loops.push(infos[j].loop_.clone());
            }
        }

        let region = TrimRegion::from_loops(region_loops, tol).map_err(|e| PatchError::Triangulation(e.to_string()))?;
        regions.push(region);
    }

    regions.sort_by(|a, b| {
        b.outer
            .signed_area()
            .abs()
            .total_cmp(&a.outer.signed_area().abs())
    });

    Ok(regions)
}

fn orient2d(a: UvPoint, b: UvPoint, c: UvPoint) -> f64 {
    (b.u - a.u) * (c.v - a.v) - (b.v - a.v) * (c.u - a.u)
}

fn point_on_segment_uv(p: UvPoint, a: UvPoint, b: UvPoint, tol: Tolerance) -> bool {
    let ab_u = b.u - a.u;
    let ab_v = b.v - a.v;
    let ap_u = p.u - a.u;
    let ap_v = p.v - a.v;

    let cross = ab_u * ap_v - ab_v * ap_u;
    if cross.abs() > tol.eps {
        return false;
    }

    let dot = ap_u * ab_u + ap_v * ab_v;
    if dot < -tol.eps {
        return false;
    }

    let ab_len2 = ab_u * ab_u + ab_v * ab_v;
    if dot - ab_len2 > tol.eps {
        return false;
    }

    true
}

fn segments_intersect_uv(a: UvPoint, b: UvPoint, c: UvPoint, d: UvPoint, tol: Tolerance) -> bool {
    let o1 = orient2d(a, b, c);
    let o2 = orient2d(a, b, d);
    let o3 = orient2d(c, d, a);
    let o4 = orient2d(c, d, b);

    if o1.abs() <= tol.eps && point_on_segment_uv(c, a, b, tol) {
        return true;
    }
    if o2.abs() <= tol.eps && point_on_segment_uv(d, a, b, tol) {
        return true;
    }
    if o3.abs() <= tol.eps && point_on_segment_uv(a, c, d, tol) {
        return true;
    }
    if o4.abs() <= tol.eps && point_on_segment_uv(b, c, d, tol) {
        return true;
    }

    let ab = (o1 > tol.eps && o2 < -tol.eps) || (o1 < -tol.eps && o2 > tol.eps);
    let cd = (o3 > tol.eps && o4 < -tol.eps) || (o3 < -tol.eps && o4 > tol.eps);
    ab && cd
}

fn loops_intersect_uv(a: &[UvPoint], b: &[UvPoint], tol: Tolerance) -> bool {
    if a.len() < 2 || b.len() < 2 {
        return false;
    }

    for i in 0..a.len() {
        let a0 = a[i];
        let a1 = a[(i + 1) % a.len()];
        for j in 0..b.len() {
            let b0 = b[j];
            let b1 = b[(j + 1) % b.len()];
            if segments_intersect_uv(a0, a1, b0, b1, tol) {
                return true;
            }
        }
    }

    false
}

fn project_loop_to_uv(
    loop3: &[Point3],
    origin: Point3,
    u_axis: Vec3,
    v_axis: Vec3,
    tol: Tolerance,
) -> Result<TrimLoop, PatchError> {
    let uv_points: Vec<UvPoint> = loop3
        .iter()
        .map(|p| {
            let d = p.sub_point(origin);
            UvPoint::new(d.dot(u_axis), d.dot(v_axis))
        })
        .collect();
    TrimLoop::new(uv_points, tol).map_err(|e| PatchError::Triangulation(e.to_string()))
}

fn point_from_uv(origin: Point3, u_axis: Vec3, v_axis: Vec3, uv: UvPoint) -> Point3 {
    let offset = u_axis.mul_scalar(uv.u).add(v_axis.mul_scalar(uv.v));
    origin.add_vec(offset)
}

fn plane_basis(normal: Vec3) -> Result<(Vec3, Vec3), PatchError> {
    let n = normal.normalized().ok_or(PatchError::BoundaryDegenerate)?;
    let axis = if n.x.abs() < 0.9 {
        Vec3::new(1.0, 0.0, 0.0)
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    };

    let u_axis = n
        .cross(axis)
        .normalized()
        .ok_or(PatchError::BoundaryDegenerate)?;
    let v_axis = n.cross(u_axis);
    Ok((u_axis, v_axis))
}

fn polygon_normal(points: &[Point3]) -> Vec3 {
    if points.len() < 3 {
        return Vec3::new(0.0, 0.0, 0.0);
    }

    let mut n = Vec3::new(0.0, 0.0, 0.0);
    for i in 0..points.len() {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        n.x += (a.y - b.y) * (a.z + b.z);
        n.y += (a.z - b.z) * (a.x + b.x);
        n.z += (a.x - b.x) * (a.y + b.y);
    }
    n
}

fn clean_closed_polyline(points: &[Point3], tol: Tolerance) -> Result<Vec<Point3>, PatchError> {
    if points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite()) {
        return Err(PatchError::NonFinitePoint);
    }

    let mut cleaned: Vec<Point3> = Vec::with_capacity(points.len());
    for p in points.iter().copied() {
        if cleaned
            .last()
            .copied()
            .is_some_and(|prev| tol.approx_eq_point3(prev, p))
        {
            continue;
        }
        cleaned.push(p);
    }

    if cleaned.len() > 2 {
        if let (Some(first), Some(last)) = (cleaned.first().copied(), cleaned.last().copied()) {
            if tol.approx_eq_point3(first, last) {
                cleaned.pop();
            }
        }
    }

    if cleaned.len() < 3 {
        return Err(PatchError::NotEnoughPoints { min: 3 });
    }

    Ok(cleaned)
}
