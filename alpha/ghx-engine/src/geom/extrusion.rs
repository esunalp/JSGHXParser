use super::diagnostics::GeomMeshDiagnostics;
use super::mesh::{GeomMesh, finalize_mesh};
use super::trim::{TrimLoop, TrimRegion, UvPoint};
use super::triangulation::triangulate_trim_region;
use super::{Point3, Tolerance, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtrusionCaps {
    pub start: bool,
    pub end: bool,
}

impl ExtrusionCaps {
    pub const NONE: Self = Self { start: false, end: false };
    pub const START: Self = Self { start: true, end: false };
    pub const END: Self = Self { start: false, end: true };
    pub const BOTH: Self = Self { start: true, end: true };
}

#[derive(Debug, thiserror::Error)]
pub enum ExtrusionError {
    #[error("extrusion direction must be finite and non-zero")]
    InvalidDirection,
    #[error("profile points must be finite")]
    NonFinitePoint,
    #[error("extrusion inputs must be finite")]
    NonFiniteInput,
    #[error("profile requires at least {min} unique points")]
    NotEnoughPoints { min: usize },
    #[error("caps require a closed profile")]
    CapsRequireClosedProfile,
    #[error("profile is degenerate (zero area)")]
    ProfileDegenerate,
    #[error("profile is not planar enough to cap (max distance {max_distance})")]
    ProfileNotPlanar { max_distance: f64 },
    #[error("failed to triangulate cap: {0}")]
    CapTriangulation(String),
    #[error("angled extrusion requires a closed polyline")]
    AngledRequiresClosedPolyline,
    #[error("angled extrusion requires a planar polyline (constant Z)")]
    AngledRequiresPlanarZ,
    #[error("polyline has a degenerate edge")]
    DegenerateEdge,
    #[error("failed to intersect offset edges")]
    OffsetIntersectionFailed,
}

#[must_use]
pub fn extrude_polyline(
    profile: &[Point3],
    direction: Vec3,
    caps: ExtrusionCaps,
) -> Result<(GeomMesh, GeomMeshDiagnostics), ExtrusionError> {
    extrude_polyline_with_tolerance(profile, direction, caps, Tolerance::default_geom())
}

pub fn extrude_polyline_with_tolerance(
    profile: &[Point3],
    direction: Vec3,
    caps: ExtrusionCaps,
    tol: Tolerance,
) -> Result<(GeomMesh, GeomMeshDiagnostics), ExtrusionError> {
    let direction = validate_direction(direction)?;
    let cleaned = clean_polyline(profile, caps.start || caps.end, tol)?;

    let mut profile_points = cleaned.points;
    let cap = if caps.start || caps.end {
        Some(build_cap_triangulation(&mut profile_points, direction, tol)?)
    } else {
        None
    };

    let n = profile_points.len();
    let mut vertices: Vec<Point3> = Vec::with_capacity(n.saturating_mul(2));
    vertices.extend_from_slice(&profile_points);
    vertices.extend(profile_points.iter().map(|p| p.add_vec(direction)));

    let edge_count = if cleaned.closed { n } else { n.saturating_sub(1) };
    let mut indices: Vec<u32> = Vec::with_capacity(edge_count.saturating_mul(6));

    for i in 0..edge_count {
        let i0 = i as u32;
        let i1 = ((i + 1) % n) as u32;
        let j0 = (n + i) as u32;
        let j1 = (n + ((i + 1) % n)) as u32;

        indices.extend_from_slice(&[i0, i1, j1]);
        indices.extend_from_slice(&[i0, j1, j0]);
    }

    if let Some(cap) = cap {
        if caps.start {
            let offset = vertices.len() as u32;
            vertices.extend(cap.vertices.iter().map(|uv| cap.point_at(*uv)));
            for tri in cap.indices.chunks_exact(3) {
                indices.extend_from_slice(&[
                    offset + tri[0],
                    offset + tri[2],
                    offset + tri[1],
                ]);
            }
        }

        if caps.end {
            let offset = vertices.len() as u32;
            vertices.extend(cap.vertices.iter().map(|uv| cap.point_at(*uv).add_vec(direction)));
            for tri in cap.indices.chunks_exact(3) {
                indices.extend_from_slice(&[
                    offset + tri[0],
                    offset + tri[1],
                    offset + tri[2],
                ]);
            }
        }
    }

    Ok(finalize_mesh(vertices, None, indices, tol))
}

#[must_use]
pub fn extrude_to_point(
    profile: &[Point3],
    tip: Point3,
    cap_base: bool,
) -> Result<(GeomMesh, GeomMeshDiagnostics), ExtrusionError> {
    extrude_to_point_with_tolerance(profile, tip, cap_base, Tolerance::default_geom())
}

pub fn extrude_to_point_with_tolerance(
    profile: &[Point3],
    tip: Point3,
    cap_base: bool,
    tol: Tolerance,
) -> Result<(GeomMesh, GeomMeshDiagnostics), ExtrusionError> {
    if !tip.x.is_finite() || !tip.y.is_finite() || !tip.z.is_finite() {
        return Err(ExtrusionError::NonFinitePoint);
    }

    let cleaned = clean_polyline(profile, cap_base, tol)?;

    let mut profile_points = cleaned.points;
    let cap = if cap_base && cleaned.closed {
        let centroid = centroid(&profile_points);
        let to_tip = tip.sub_point(centroid);
        Some(build_cap_triangulation(&mut profile_points, to_tip, tol)?)
    } else {
        None
    };

    let n = profile_points.len();
    let tip_index = n as u32;

    let mut vertices: Vec<Point3> = Vec::with_capacity(n + 1);
    vertices.extend_from_slice(&profile_points);
    vertices.push(tip);

    let edge_count = if cleaned.closed { n } else { n.saturating_sub(1) };
    let mut indices: Vec<u32> = Vec::with_capacity(edge_count.saturating_mul(3));
    for i in 0..edge_count {
        let i0 = i as u32;
        let i1 = ((i + 1) % n) as u32;
        indices.extend_from_slice(&[i0, i1, tip_index]);
    }

    if let Some(cap) = cap {
        let offset = vertices.len() as u32;
        vertices.extend(cap.vertices.iter().map(|uv| cap.point_at(*uv)));
        for tri in cap.indices.chunks_exact(3) {
            indices.extend_from_slice(&[
                offset + tri[0],
                offset + tri[2],
                offset + tri[1],
            ]);
        }
    }

    Ok(finalize_mesh(vertices, None, indices, tol))
}

#[must_use]
pub fn extrude_angled_polyline(
    polyline: &[Point3],
    base_height: f64,
    top_height: f64,
    angles: &[f64],
) -> Result<(GeomMesh, GeomMeshDiagnostics), ExtrusionError> {
    extrude_angled_polyline_with_tolerance(
        polyline,
        base_height,
        top_height,
        angles,
        Tolerance::default_geom(),
    )
}

pub fn extrude_angled_polyline_with_tolerance(
    polyline: &[Point3],
    base_height: f64,
    top_height: f64,
    angles: &[f64],
    tol: Tolerance,
) -> Result<(GeomMesh, GeomMeshDiagnostics), ExtrusionError> {
    if !base_height.is_finite() || !top_height.is_finite() {
        return Err(ExtrusionError::NonFiniteInput);
    }

    let cleaned = clean_polyline(polyline, true, tol)?;

    let base = cleaned.points;
    let z0 = base.first().copied().map(|p| p.z).unwrap_or(0.0);
    let planar_eps = (tol.eps * 1e3).max(tol.eps);
    if base.iter().any(|p| (p.z - z0).abs() > planar_eps) {
        return Err(ExtrusionError::AngledRequiresPlanarZ);
    }

    let n = base.len();
    if n < 3 {
        return Err(ExtrusionError::NotEnoughPoints { min: 3 });
    }

    let signed_area = polygon_area_xy(&base);
    if !signed_area.is_finite() || signed_area.abs() <= tol.eps {
        return Err(ExtrusionError::ProfileDegenerate);
    }
    let orientation = if signed_area >= 0.0 { 1.0 } else { -1.0 };

    let dh = top_height - base_height;
    let mut line_a = Vec::with_capacity(n);
    let mut line_b = Vec::with_capacity(n);
    let mut line_c = Vec::with_capacity(n);

    for i in 0..n {
        let p0 = base[i];
        let p1 = base[(i + 1) % n];
        let dx = p1.x - p0.x;
        let dy = p1.y - p0.y;
        let len = (dx * dx + dy * dy).sqrt();
        if !len.is_finite() || len <= tol.eps {
            return Err(ExtrusionError::DegenerateEdge);
        }

        let nx = orientation * dy / len;
        let ny = orientation * -dx / len;
        let angle = angle_for_edge(angles, i)?;
        let offset = dh * angle.tan();
        if !offset.is_finite() {
            return Err(ExtrusionError::NonFiniteInput);
        }

        line_a.push(nx);
        line_b.push(ny);
        line_c.push(nx * p0.x + ny * p0.y + offset);
    }

    let mut top_ring: Vec<Point3> = Vec::with_capacity(n);
    for i in 0..n {
        let prev = (i + n - 1) % n;
        let a1 = line_a[prev];
        let b1 = line_b[prev];
        let c1 = line_c[prev];
        let a2 = line_a[i];
        let b2 = line_b[i];
        let c2 = line_c[i];

        let det = a1 * b2 - a2 * b1;
        if !det.is_finite() || det.abs() < 1e-12 {
            return Err(ExtrusionError::OffsetIntersectionFailed);
        }

        let x = (c1 * b2 - c2 * b1) / det;
        let y = (a1 * c2 - a2 * c1) / det;
        if !x.is_finite() || !y.is_finite() {
            return Err(ExtrusionError::OffsetIntersectionFailed);
        }

        top_ring.push(Point3::new(x, y, z0 + top_height));
    }

    let ring0_offset = 0usize;
    let ring1_offset = n;
    let ring2_offset = n * 2;

    let mut vertices: Vec<Point3> = Vec::with_capacity(n * 3);
    vertices.extend_from_slice(&base);
    vertices.extend(base.iter().map(|p| p.add_vec(Vec3::new(0.0, 0.0, base_height))));
    vertices.extend_from_slice(&top_ring);

    let mut indices: Vec<u32> = Vec::new();

    if base_height.abs() > tol.eps {
        for i in 0..n {
            let i0 = i;
            let i1 = (i + 1) % n;
            let b0 = (ring0_offset + i0) as u32;
            let b1 = (ring0_offset + i1) as u32;
            let t0 = (ring1_offset + i0) as u32;
            let t1 = (ring1_offset + i1) as u32;
            indices.extend_from_slice(&[b0, b1, t1]);
            indices.extend_from_slice(&[b0, t1, t0]);
        }
    }

    if (top_height - base_height).abs() > tol.eps {
        for i in 0..n {
            let i0 = i;
            let i1 = (i + 1) % n;
            let b0 = (ring1_offset + i0) as u32;
            let b1 = (ring1_offset + i1) as u32;
            let t0 = (ring2_offset + i0) as u32;
            let t1 = (ring2_offset + i1) as u32;
            indices.extend_from_slice(&[b0, b1, t1]);
            indices.extend_from_slice(&[b0, t1, t0]);
        }
    }

    let cap_tri = triangulate_xy_loop(&base, tol)?;
    let cap_tri_top = triangulate_xy_loop(&top_ring, tol)?;
    {
        let offset = vertices.len() as u32;
        vertices.extend(cap_tri.vertices.iter().map(|uv| Point3::new(uv.u, uv.v, z0)));
        for tri in cap_tri.indices.chunks_exact(3) {
            indices.extend_from_slice(&[
                offset + tri[0],
                offset + tri[2],
                offset + tri[1],
            ]);
        }
    }

    {
        let offset = vertices.len() as u32;
        vertices.extend(
            cap_tri_top
                .vertices
                .iter()
                .map(|uv| Point3::new(uv.u, uv.v, z0 + top_height)),
        );
        for tri in cap_tri_top.indices.chunks_exact(3) {
            indices.extend_from_slice(&[
                offset + tri[0],
                offset + tri[1],
                offset + tri[2],
            ]);
        }
    }

    Ok(finalize_mesh(vertices, None, indices, tol))
}

#[derive(Debug, Clone)]
struct CleanPolyline {
    points: Vec<Point3>,
    closed: bool,
}

fn validate_direction(direction: Vec3) -> Result<Vec3, ExtrusionError> {
    if !direction.x.is_finite() || !direction.y.is_finite() || !direction.z.is_finite() {
        return Err(ExtrusionError::InvalidDirection);
    }
    if direction.length_squared() <= 0.0 {
        return Err(ExtrusionError::InvalidDirection);
    }
    Ok(direction)
}

fn clean_polyline(
    points: &[Point3],
    force_closed: bool,
    tol: Tolerance,
) -> Result<CleanPolyline, ExtrusionError> {
    if points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite()) {
        return Err(ExtrusionError::NonFinitePoint);
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

    if cleaned.len() < 2 {
        return Err(ExtrusionError::NotEnoughPoints { min: 2 });
    }

    let mut closed = force_closed;
    if cleaned.len() > 2 {
        if let (Some(first), Some(last)) = (cleaned.first().copied(), cleaned.last().copied()) {
            if tol.approx_eq_point3(first, last) {
                cleaned.pop();
                closed = true;
            }
        }
    }

    let min = if closed { 3 } else { 2 };
    if cleaned.len() < min {
        return Err(ExtrusionError::NotEnoughPoints { min });
    }

    Ok(CleanPolyline {
        points: cleaned,
        closed,
    })
}

#[derive(Debug, Clone)]
struct CapTriangulation {
    origin: Point3,
    u_axis: Vec3,
    v_axis: Vec3,
    vertices: Vec<UvPoint>,
    indices: Vec<u32>,
}

impl CapTriangulation {
    fn point_at(&self, uv: UvPoint) -> Point3 {
        let offset = self
            .u_axis
            .mul_scalar(uv.u)
            .add(self.v_axis.mul_scalar(uv.v));
        self.origin.add_vec(offset)
    }
}

fn build_cap_triangulation(
    profile: &mut Vec<Point3>,
    direction: Vec3,
    tol: Tolerance,
) -> Result<CapTriangulation, ExtrusionError> {
    let normal = polygon_normal(profile);
    let normal_len2 = normal.length_squared();
    if !normal_len2.is_finite() || normal_len2 <= tol.eps_squared() {
        return Err(ExtrusionError::ProfileDegenerate);
    }
    let mut normal = normal
        .normalized()
        .ok_or(ExtrusionError::ProfileDegenerate)?;

    let planar_eps = (tol.eps * 1e3).max(tol.eps);
    let origin = profile.first().copied().unwrap_or(Point3::new(0.0, 0.0, 0.0));
    let mut max_distance: f64 = 0.0;
    for p in profile.iter().copied() {
        let d = p.sub_point(origin).dot(normal).abs();
        max_distance = max_distance.max(d);
    }
    if max_distance > planar_eps {
        return Err(ExtrusionError::ProfileNotPlanar { max_distance });
    }

    if normal.dot(direction) < 0.0 {
        profile.reverse();
        normal = normal.mul_scalar(-1.0);
    }

    let (u_axis, v_axis) = plane_basis(normal)?;
    let origin = profile[0];

    let uv_points: Vec<UvPoint> = profile
        .iter()
        .map(|p| {
            let d = p.sub_point(origin);
            UvPoint::new(d.dot(u_axis), d.dot(v_axis))
        })
        .collect();

    let loop_ = TrimLoop::new(uv_points, tol).map_err(|e| ExtrusionError::CapTriangulation(e.to_string()))?;
    let region = TrimRegion::from_loops(vec![loop_], tol).map_err(|e| ExtrusionError::CapTriangulation(e.to_string()))?;
    let tri = triangulate_trim_region(&region, tol).map_err(ExtrusionError::CapTriangulation)?;

    Ok(CapTriangulation {
        origin,
        u_axis,
        v_axis,
        vertices: tri.vertices,
        indices: tri.indices,
    })
}

fn plane_basis(normal: Vec3) -> Result<(Vec3, Vec3), ExtrusionError> {
    let n = normal.normalized().ok_or(ExtrusionError::ProfileDegenerate)?;
    let axis = if n.x.abs() < 0.9 {
        Vec3::new(1.0, 0.0, 0.0)
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    };

    let u_axis = n
        .cross(axis)
        .normalized()
        .ok_or(ExtrusionError::ProfileDegenerate)?;
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

fn centroid(points: &[Point3]) -> Point3 {
    if points.is_empty() {
        return Point3::new(0.0, 0.0, 0.0);
    }

    let mut sx = 0.0;
    let mut sy = 0.0;
    let mut sz = 0.0;
    for p in points.iter().copied() {
        sx += p.x;
        sy += p.y;
        sz += p.z;
    }
    let inv = 1.0 / points.len() as f64;
    Point3::new(sx * inv, sy * inv, sz * inv)
}

fn angle_for_edge(angles: &[f64], index: usize) -> Result<f64, ExtrusionError> {
    if angles.is_empty() {
        return Ok(0.0);
    }
    let angle = if angles.len() == 1 {
        angles[0]
    } else {
        angles[index % angles.len()]
    };
    if angle.is_finite() { Ok(angle) } else { Err(ExtrusionError::NonFiniteInput) }
}

fn polygon_area_xy(points: &[Point3]) -> f64 {
    if points.len() < 3 {
        return 0.0;
    }

    let mut area = 0.0;
    for i in 0..points.len() {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        area += a.x * b.y - b.x * a.y;
    }
    0.5 * area
}

#[derive(Debug, Clone)]
struct CapTriangulation2D {
    vertices: Vec<UvPoint>,
    indices: Vec<u32>,
}

fn triangulate_xy_loop(points: &[Point3], tol: Tolerance) -> Result<CapTriangulation2D, ExtrusionError> {
    let uv_points: Vec<UvPoint> = points.iter().map(|p| UvPoint::new(p.x, p.y)).collect();
    let loop_ = TrimLoop::new(uv_points, tol).map_err(|e| ExtrusionError::CapTriangulation(e.to_string()))?;
    let region = TrimRegion::from_loops(vec![loop_], tol).map_err(|e| ExtrusionError::CapTriangulation(e.to_string()))?;
    let tri = triangulate_trim_region(&region, tol).map_err(ExtrusionError::CapTriangulation)?;
    Ok(CapTriangulation2D {
        vertices: tri.vertices,
        indices: tri.indices,
    })
}
