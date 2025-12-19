use super::diagnostics::GeomMeshDiagnostics;
use super::mesh::{GeomMesh, finalize_mesh};
use super::trim::{TrimLoop, TrimRegion, UvPoint};
use super::triangulation::triangulate_trim_region;
use super::{Point3, Tolerance, Vec3};

/// Planar patch and boundary-surface helpers.
///
/// This module is intentionally minimal for Phase 2: it fills *planar* closed
/// boundaries (optionally with holes) via constrained triangulation.
///
/// Limitations (by design for now):
/// - Boundaries must be (approximately) planar.
/// - Only polyline boundaries are supported (no interior constraints).
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
    #[error("failed to triangulate boundary: {0}")]
    Triangulation(String),
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
