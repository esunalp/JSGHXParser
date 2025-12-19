use super::bvh::Bvh;
use super::mesh::GeomMesh;
use super::diagnostics::GeomMeshDiagnostics;
use super::{BBox, Point3, Tolerance, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOp {
    Union,
    Difference,
    Intersection,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment3 {
    pub a: Point3,
    pub b: Point3,
}

impl Segment3 {
    #[must_use]
    pub const fn new(a: Point3, b: Point3) -> Self {
        Self { a, b }
    }

    #[must_use]
    pub fn direction(self) -> Vec3 {
        self.b.sub_point(self.a)
    }

    #[must_use]
    pub fn length_squared(self) -> f64 {
        self.direction().length_squared()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Triangle3 {
    pub a: Point3,
    pub b: Point3,
    pub c: Point3,
}

impl Triangle3 {
    #[must_use]
    pub const fn new(a: Point3, b: Point3, c: Point3) -> Self {
        Self { a, b, c }
    }

    #[must_use]
    pub fn normal(self) -> Vec3 {
        self.b.sub_point(self.a).cross(self.c.sub_point(self.a))
    }

    #[must_use]
    pub fn bbox(self) -> BBox {
        let mut min = self.a;
        let mut max = self.a;
        for p in [self.b, self.c] {
            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            min.z = min.z.min(p.z);
            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
            max.z = max.z.max(p.z);
        }
        BBox::new(min, max)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TriTriIntersection {
    Point(Point3),
    Segment(Segment3),
    Coplanar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointContainment {
    Inside,
    Outside,
    OnSurface,
    Indeterminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriangleContainment {
    Inside,
    Outside,
    OnSurface,
    Indeterminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriangleSource {
    A,
    B,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaggedTriangle {
    pub indices: [u32; 3],
    pub source: TriangleSource,
    pub containment: TriangleContainment,
    pub on_intersection_band: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BooleanDiagnostics {
    pub op: BooleanOp,
    pub input_a_vertex_count: usize,
    pub input_a_triangle_count: usize,
    pub input_b_vertex_count: usize,
    pub input_b_triangle_count: usize,
    pub intersection_segment_count: usize,
    pub intersection_point_count: usize,
    pub coplanar_pair_count: usize,
    pub split_triangle_count_a: usize,
    pub split_triangle_count_b: usize,
    pub complex_triangle_count_a: usize,
    pub complex_triangle_count_b: usize,
    pub kept_triangle_count_a: usize,
    pub kept_triangle_count_b: usize,
    pub indeterminate_triangle_count: usize,
    pub warnings: Vec<String>,
    pub tolerance_used: f64,
    pub tolerance_relaxed: bool,
    pub voxel_fallback_used: bool,
}

impl Default for BooleanDiagnostics {
    fn default() -> Self {
        Self {
            op: BooleanOp::Union,
            input_a_vertex_count: 0,
            input_a_triangle_count: 0,
            input_b_vertex_count: 0,
            input_b_triangle_count: 0,
            intersection_segment_count: 0,
            intersection_point_count: 0,
            coplanar_pair_count: 0,
            split_triangle_count_a: 0,
            split_triangle_count_b: 0,
            complex_triangle_count_a: 0,
            complex_triangle_count_b: 0,
            kept_triangle_count_a: 0,
            kept_triangle_count_b: 0,
            indeterminate_triangle_count: 0,
            warnings: Vec::new(),
            tolerance_used: 0.0,
            tolerance_relaxed: false,
            voxel_fallback_used: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BooleanResult {
    pub mesh: GeomMesh,
    pub mesh_diagnostics: GeomMeshDiagnostics,
    pub diagnostics: BooleanDiagnostics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlaneSide {
    Positive,
    Negative,
    OnPlane,
}

fn plane_from_triangle(tri: Triangle3, tol: Tolerance) -> Option<(Vec3, f64, f64)> {
    let n = tri.normal();
    let len = n.length();
    if !len.is_finite() || len <= tol.eps {
        return None;
    }
    let d = n.dot(Vec3::from(tri.a));
    Some((n, d, len))
}

fn plane_side(n: Vec3, d: f64, n_len: f64, p: Point3, tol: Tolerance) -> PlaneSide {
    if !n_len.is_finite() || n_len <= tol.eps {
        return PlaneSide::OnPlane;
    }
    let signed = n.dot(Vec3::from(p)) - d;
    if !signed.is_finite() {
        return PlaneSide::OnPlane;
    }
    let bound = tol.eps * n_len;
    if signed > bound {
        PlaneSide::Positive
    } else if signed < -bound {
        PlaneSide::Negative
    } else {
        PlaneSide::OnPlane
    }
}

fn push_unique_point(points: &mut Vec<Point3>, point: Point3, tol: Tolerance) {
    if points.iter().any(|&p| tol.approx_eq_point3(p, point)) {
        return;
    }
    points.push(point);
}

fn triangle_triangle_coplanar(tri_a: Triangle3, tri_b: Triangle3, tol: Tolerance) -> bool {
    let Some((n_a, d_a, n_len)) = plane_from_triangle(tri_a, tol) else {
        return false;
    };
    let Some((n_b, _d_b, _)) = plane_from_triangle(tri_b, tol) else {
        return false;
    };

    // Parallel planes?
    let cross = n_a.cross(n_b);
    let cross_len2 = cross.length_squared();
    let denom = n_len * n_b.length();
    if !cross_len2.is_finite() || !denom.is_finite() || denom <= tol.eps {
        return false;
    }
    // Filtered predicate: if cross is tiny relative to normals, treat as parallel.
    if cross_len2 > (tol.eps * denom).powi(2) {
        return false;
    }

    matches!(plane_side(n_a, d_a, n_len, tri_b.a, tol), PlaneSide::OnPlane)
}

fn dedup_points(points: &mut Vec<Point3>, tol: Tolerance) {
    let mut out = Vec::with_capacity(points.len());
    for p in points.drain(..) {
        push_unique_point(&mut out, p, tol);
    }
    *points = out;
}

fn segment_triangle_intersection(
    segment: Segment3,
    triangle: Triangle3,
    tol: Tolerance,
) -> Option<Point3> {
    // Möller–Trumbore intersection (segment variant).
    let dir = segment.direction();
    let edge1 = triangle.b.sub_point(triangle.a);
    let edge2 = triangle.c.sub_point(triangle.a);
    let h = dir.cross(edge2);
    let det = edge1.dot(h);

    let edge1_len = edge1.length();
    let h_len = h.length();
    let det_eps = tol.eps * edge1_len * h_len;

    if !det.is_finite() || det.abs() <= det_eps {
        return None;
    }

    let inv_det = 1.0 / det;
    let s = segment.a.sub_point(triangle.a);
    let u = inv_det * s.dot(h);
    let u_eps = tol.eps;
    if u < -u_eps || u > 1.0 + u_eps {
        return None;
    }

    let q = s.cross(edge1);
    let v = inv_det * dir.dot(q);
    if v < -u_eps || u + v > 1.0 + u_eps {
        return None;
    }

    let t = inv_det * edge2.dot(q);
    let t_eps = tol.eps;
    if t < -t_eps || t > 1.0 + t_eps {
        return None;
    }

    Some(segment.a + dir * t)
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct RayHit {
    t: f64,
    u: f64,
    v: f64,
}

fn ray_triangle_intersection(origin: Point3, dir: Vec3, tri: Triangle3, tol: Tolerance) -> Option<RayHit> {
    let edge1 = tri.b.sub_point(tri.a);
    let edge2 = tri.c.sub_point(tri.a);
    let h = dir.cross(edge2);
    let det = edge1.dot(h);
    let det_eps = tol.eps * edge1.length() * h.length();
    if !det.is_finite() || det.abs() <= det_eps {
        return None;
    }

    let inv_det = 1.0 / det;
    let s = origin.sub_point(tri.a);
    let u = inv_det * s.dot(h);
    let uv_eps = tol.eps;
    if u < -uv_eps || u > 1.0 + uv_eps {
        return None;
    }

    let q = s.cross(edge1);
    let v = inv_det * dir.dot(q);
    if v < -uv_eps || u + v > 1.0 + uv_eps {
        return None;
    }

    let t = inv_det * edge2.dot(q);
    if !t.is_finite() {
        return None;
    }
    if t < -tol.eps {
        return None;
    }

    Some(RayHit { t, u, v })
}

fn point_on_triangle(point: Point3, tri: Triangle3, tol: Tolerance) -> bool {
    let Some((n, d, n_len)) = plane_from_triangle(tri, tol) else {
        return false;
    };
    let signed = n.dot(Vec3::from(point)) - d;
    if !signed.is_finite() {
        return false;
    }
    if signed.abs() > tol.eps * n_len {
        return false;
    }

    // Barycentric test in-plane.
    let v0 = tri.b.sub_point(tri.a);
    let v1 = tri.c.sub_point(tri.a);
    let v2 = point.sub_point(tri.a);

    let dot00 = v0.dot(v0);
    let dot01 = v0.dot(v1);
    let dot11 = v1.dot(v1);
    let dot02 = v0.dot(v2);
    let dot12 = v1.dot(v2);

    let denom = dot00 * dot11 - dot01 * dot01;
    if !denom.is_finite() || denom.abs() <= tol.eps {
        return false;
    }

    let inv_denom = 1.0 / denom;
    let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
    let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

    let eps = tol.eps;
    u >= -eps && v >= -eps && (u + v) <= 1.0 + eps
}

fn mesh_bbox(mesh: &GeomMesh) -> Option<BBox> {
    let mut iter = mesh.positions.iter().copied();
    let first = iter.next()?;
    let mut min = Point3::from(first);
    let mut max = min;
    for p in iter.map(Point3::from) {
        min.x = min.x.min(p.x);
        min.y = min.y.min(p.y);
        min.z = min.z.min(p.z);
        max.x = max.x.max(p.x);
        max.y = max.y.max(p.y);
        max.z = max.z.max(p.z);
    }
    Some(BBox::new(min, max))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RayVote {
    Certain(PointContainment),
    Ambiguous,
}

fn ray_cast_containment(point: Point3, mesh: &GeomMesh, dir: Vec3, tol: Tolerance) -> RayVote {
    let points: Vec<Point3> = mesh.positions.iter().copied().map(Point3::from).collect();

    let mut intersections = 0usize;
    let mut ambiguous = false;

    for tri in mesh.indices.chunks_exact(3) {
        let Some(a) = points.get(tri[0] as usize).copied() else {
            continue;
        };
        let Some(b) = points.get(tri[1] as usize).copied() else {
            continue;
        };
        let Some(c) = points.get(tri[2] as usize).copied() else {
            continue;
        };
        let tri = Triangle3::new(a, b, c);

        if point_on_triangle(point, tri, tol) {
            return RayVote::Certain(PointContainment::OnSurface);
        }

        let Some(hit) = ray_triangle_intersection(point, dir, tri, tol) else {
            continue;
        };

        if hit.t <= tol.eps {
            continue;
        }

        let edge_eps = tol.eps * 10.0;
        let w = 1.0 - hit.u - hit.v;
        if hit.u.abs() <= edge_eps || hit.v.abs() <= edge_eps || w.abs() <= edge_eps {
            ambiguous = true;
            continue;
        }

        intersections += 1;
    }

    if ambiguous {
        return RayVote::Ambiguous;
    }

    let inside = (intersections % 2) == 1;
    RayVote::Certain(if inside {
        PointContainment::Inside
    } else {
        PointContainment::Outside
    })
}

/// Classify a point against a closed triangle mesh (inside/outside).
///
/// This is used by mesh booleans for classification. When the point lies on the
/// surface (within tolerance), returns [`PointContainment::OnSurface`].
#[must_use]
pub fn classify_point_in_mesh(point: Point3, mesh: &GeomMesh, tol: Tolerance) -> PointContainment {
    let Ok(prepared) = prepare_mesh(mesh, tol) else {
        return classify_point_in_mesh_fallback(point, mesh, tol);
    };

    classify_point_in_prepared_mesh(point, &prepared, tol)
}

fn classify_point_in_mesh_fallback(point: Point3, mesh: &GeomMesh, tol: Tolerance) -> PointContainment {
    let Some(bbox) = mesh_bbox(mesh) else {
        return PointContainment::Indeterminate;
    };
    if !bbox.expand_tolerance(tol).contains_point(point) {
        return PointContainment::Outside;
    }

    let dirs = [
        Vec3::new(1.0, 0.234_567_89, 0.345_678_91),
        Vec3::new(0.345_678_91, 1.0, 0.234_567_89),
        Vec3::new(0.234_567_89, 0.345_678_91, 1.0),
    ];

    let mut vote: Option<PointContainment> = None;
    for dir in dirs {
        let dir = dir.normalized().unwrap_or(Vec3::X);
        match ray_cast_containment(point, mesh, dir, tol) {
            RayVote::Certain(PointContainment::OnSurface) => return PointContainment::OnSurface,
            RayVote::Certain(result) => match vote {
                None => vote = Some(result),
                Some(prev) if prev == result => return result,
                Some(_) => return PointContainment::Indeterminate,
            },
            RayVote::Ambiguous => continue,
        }
    }

    vote.unwrap_or(PointContainment::Indeterminate)
}

/// Classify each triangle (by centroid) of `mesh` against `other`.
#[must_use]
pub fn classify_mesh_triangles(mesh: &GeomMesh, other: &GeomMesh, tol: Tolerance) -> Vec<TriangleContainment> {
    let points: Vec<Point3> = mesh.positions.iter().copied().map(Point3::from).collect();
    let mut out = Vec::with_capacity(mesh.indices.len() / 3);

    let prepared_other = prepare_mesh(other, tol).ok();

    for tri in mesh.indices.chunks_exact(3) {
        let Some(a) = points.get(tri[0] as usize).copied() else {
            out.push(TriangleContainment::Indeterminate);
            continue;
        };
        let Some(b) = points.get(tri[1] as usize).copied() else {
            out.push(TriangleContainment::Indeterminate);
            continue;
        };
        let Some(c) = points.get(tri[2] as usize).copied() else {
            out.push(TriangleContainment::Indeterminate);
            continue;
        };

        let centroid = Point3::new(
            (a.x + b.x + c.x) / 3.0,
            (a.y + b.y + c.y) / 3.0,
            (a.z + b.z + c.z) / 3.0,
        );

        let point_containment = match prepared_other.as_ref() {
            Some(prepared) => classify_point_in_prepared_mesh(centroid, prepared, tol),
            None => classify_point_in_mesh_fallback(centroid, other, tol),
        };
        let containment = match point_containment {
            PointContainment::Inside => TriangleContainment::Inside,
            PointContainment::Outside => TriangleContainment::Outside,
            PointContainment::OnSurface => TriangleContainment::OnSurface,
            PointContainment::Indeterminate => TriangleContainment::Indeterminate,
        };
        out.push(containment);
    }

    out
}

/// Tag triangles with inside/outside classification, optionally marking an "intersection band".
///
/// If `intersection_vertices` is provided, triangles that reference any marked vertex are tagged
/// with `on_intersection_band = true`.
#[must_use]
pub fn tag_mesh_triangles(
    mesh: &GeomMesh,
    other: &GeomMesh,
    source: TriangleSource,
    intersection_vertices: Option<&[bool]>,
    tol: Tolerance,
) -> Vec<TaggedTriangle> {
    let classifications = classify_mesh_triangles(mesh, other, tol);
    let mut out = Vec::with_capacity(classifications.len());

    let vertex_flags = intersection_vertices.filter(|flags| flags.len() == mesh.positions.len());

    for (tri_index, tri) in mesh.indices.chunks_exact(3).enumerate() {
        let indices = [tri[0], tri[1], tri[2]];
        let on_band = vertex_flags.is_some_and(|flags| {
            indices
                .iter()
                .copied()
                .filter_map(|idx| flags.get(idx as usize).copied())
                .any(|flag| flag)
        });

        let containment = classifications
            .get(tri_index)
            .copied()
            .unwrap_or(TriangleContainment::Indeterminate);

        out.push(TaggedTriangle {
            indices,
            source,
            containment,
            on_intersection_band: on_band,
        });
    }

    out
}

/// Triangle/triangle intersection primitive for mesh booleans.
///
/// Returns `None` when triangles do not intersect (within tolerance).
/// Coplanar overlaps are reported as `TriTriIntersection::Coplanar`.
#[must_use]
pub fn triangle_triangle_intersection(
    tri_a: Triangle3,
    tri_b: Triangle3,
    tol: Tolerance,
) -> Option<TriTriIntersection> {
    let bbox_a = tri_a.bbox().expand_tolerance(tol);
    let bbox_b = tri_b.bbox().expand_tolerance(tol);
    if !bbox_a.intersects(bbox_b) {
        return None;
    }

    if triangle_triangle_coplanar(tri_a, tri_b, tol) {
        return Some(TriTriIntersection::Coplanar);
    }

    let mut points = Vec::new();

    for (p0, p1) in [(tri_a.a, tri_a.b), (tri_a.b, tri_a.c), (tri_a.c, tri_a.a)] {
        if let Some(hit) = segment_triangle_intersection(Segment3::new(p0, p1), tri_b, tol) {
            push_unique_point(&mut points, hit, tol);
        }
    }

    for (p0, p1) in [(tri_b.a, tri_b.b), (tri_b.b, tri_b.c), (tri_b.c, tri_b.a)] {
        if let Some(hit) = segment_triangle_intersection(Segment3::new(p0, p1), tri_a, tol) {
            push_unique_point(&mut points, hit, tol);
        }
    }

    match points.len() {
        0 => None,
        1 => Some(TriTriIntersection::Point(points[0])),
        _ => {
            let mut best = (0usize, 1usize, 0.0f64);
            for i in 0..points.len() {
                for j in (i + 1)..points.len() {
                    let d2 = points[j].sub_point(points[i]).length_squared();
                    if d2 > best.2 {
                        best = (i, j, d2);
                    }
                }
            }
            Some(TriTriIntersection::Segment(Segment3::new(
                points[best.0],
                points[best.1],
            )))
        }
    }
}

#[must_use]
pub fn triangle_mesh_intersection_segments(
    triangle: Triangle3,
    mesh: &GeomMesh,
    tol: Tolerance,
) -> Vec<Segment3> {
    let Ok(prepared) = prepare_mesh(mesh, tol) else {
        return Vec::new();
    };

    let query = triangle.bbox().expand_tolerance(tol);
    let mut out = Vec::new();

    prepared.bvh.query_bbox(query, |tri_idx| {
        let tri = prepared.triangles[tri_idx];
        let other = Triangle3::new(
            prepared.points[tri[0] as usize],
            prepared.points[tri[1] as usize],
            prepared.points[tri[2] as usize],
        );
        if let Some(hit) = triangle_triangle_intersection(triangle, other, tol) {
            if let TriTriIntersection::Segment(seg) = hit {
                out.push(seg);
            }
        }
        true
    });

    out
}

#[derive(Debug, Clone)]
struct PreparedMesh {
    points: Vec<Point3>,
    triangles: Vec<[u32; 3]>,
    tri_bboxes: Vec<BBox>,
    bvh: Bvh,
    bbox: BBox,
}

fn prepare_mesh(mesh: &GeomMesh, tol: Tolerance) -> Result<PreparedMesh, BooleanError> {
    if mesh.positions.is_empty() || mesh.indices.is_empty() {
        return Err(BooleanError::EmptyMesh);
    }

    let points: Vec<Point3> = mesh.positions.iter().copied().map(Point3::from).collect();
    for p in &points {
        if !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite() {
            return Err(BooleanError::InvalidGeometry);
        }
    }

    let bbox = BBox::from_points(&points).ok_or(BooleanError::InvalidGeometry)?;

    let mut triangles = Vec::with_capacity(mesh.indices.len() / 3);
    let mut tri_bboxes = Vec::with_capacity(mesh.indices.len() / 3);

    for tri in mesh.indices.chunks_exact(3) {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;
        if i0 >= points.len() || i1 >= points.len() || i2 >= points.len() {
            return Err(BooleanError::InvalidIndices);
        }

        let t = [tri[0], tri[1], tri[2]];
        triangles.push(t);

        let tri_geom = Triangle3::new(points[i0], points[i1], points[i2]);
        tri_bboxes.push(tri_geom.bbox().expand_tolerance(tol));
    }

    let bvh = Bvh::build(&tri_bboxes).ok_or(BooleanError::InvalidGeometry)?;

    Ok(PreparedMesh {
        points,
        triangles,
        tri_bboxes,
        bvh,
        bbox,
    })
}

#[derive(Debug, Clone, Default)]
struct SplitMesh {
    points: Vec<Point3>,
    indices: Vec<u32>,
    intersection_vertices: Vec<bool>,
    split_triangle_count: usize,
    complex_triangle_count: usize,
}

fn point_on_segment(point: Point3, a: Point3, b: Point3, tol: Tolerance) -> Option<f64> {
    let ab = b.sub_point(a);
    let ab_len2 = ab.length_squared();
    if !ab_len2.is_finite() || ab_len2 <= tol.eps_squared() {
        return None;
    }
    let ap = point.sub_point(a);
    let t = ap.dot(ab) / ab_len2;
    let t_eps = tol.eps;
    if t < -t_eps || t > 1.0 + t_eps {
        return None;
    }
    let closest = a + ab * t.clamp(0.0, 1.0);
    let dist2 = point.sub_point(closest).length_squared();
    if !dist2.is_finite() {
        return None;
    }
    if dist2 <= tol.eps_squared() {
        Some(t)
    } else {
        None
    }
}

fn edge_index_for_point(point: Point3, tri: Triangle3, tol: Tolerance) -> Option<u8> {
    let edges = [(tri.a, tri.b), (tri.b, tri.c), (tri.c, tri.a)];
    for (edge_idx, (a, b)) in edges.iter().copied().enumerate() {
        if point_on_segment(point, a, b, tol).is_some() {
            return Some(edge_idx as u8);
        }
    }
    None
}

fn split_triangle_indices(
    tri: [u32; 3],
    edge_a: u8,
    point_a: u32,
    edge_b: u8,
    point_b: u32,
) -> Option<[[u32; 3]; 3]> {
    let v0 = tri[0];
    let v1 = tri[1];
    let v2 = tri[2];

    let (e0, e1) = if edge_a <= edge_b {
        (edge_a, edge_b)
    } else {
        (edge_b, edge_a)
    };

    match (e0, e1) {
        (0, 1) => {
            let p0 = if edge_a == 0 { point_a } else { point_b };
            let p1 = if edge_a == 1 { point_a } else { point_b };
            Some([[p0, v1, p1], [p0, p1, v2], [p0, v2, v0]])
        }
        (1, 2) => {
            let p1 = if edge_a == 1 { point_a } else { point_b };
            let p2 = if edge_a == 2 { point_a } else { point_b };
            Some([[p1, v2, p2], [p1, p2, v0], [p1, v0, v1]])
        }
        (0, 2) => {
            let p0 = if edge_a == 0 { point_a } else { point_b };
            let p2 = if edge_a == 2 { point_a } else { point_b };
            Some([[p2, v0, p0], [p2, p0, v1], [p2, v1, v2]])
        }
        _ => None,
    }
}

fn split_mesh_by_intersections(
    prepared: &PreparedMesh,
    per_triangle_points: &[Vec<Point3>],
    tol: Tolerance,
) -> SplitMesh {
    let mut out = SplitMesh {
        points: prepared.points.clone(),
        indices: Vec::new(),
        intersection_vertices: vec![false; prepared.points.len()],
        split_triangle_count: 0,
        complex_triangle_count: 0,
    };

    for (tri_index, tri) in prepared.triangles.iter().copied().enumerate() {
        let mut points = per_triangle_points
            .get(tri_index)
            .cloned()
            .unwrap_or_default();
        dedup_points(&mut points, tol);

        if points.len() < 2 {
            out.indices.extend_from_slice(&[tri[0], tri[1], tri[2]]);
            continue;
        }

        if points.len() != 2 {
            out.complex_triangle_count += 1;
            out.indices.extend_from_slice(&[tri[0], tri[1], tri[2]]);
            continue;
        }

        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;
        let tri_geom = Triangle3::new(
            prepared.points[i0],
            prepared.points[i1],
            prepared.points[i2],
        );

        let edge_a = edge_index_for_point(points[0], tri_geom, tol);
        let edge_b = edge_index_for_point(points[1], tri_geom, tol);
        let (Some(edge_a), Some(edge_b)) = (edge_a, edge_b) else {
            out.complex_triangle_count += 1;
            out.indices.extend_from_slice(&[tri[0], tri[1], tri[2]]);
            continue;
        };

        if edge_a == edge_b {
            out.complex_triangle_count += 1;
            out.indices.extend_from_slice(&[tri[0], tri[1], tri[2]]);
            continue;
        }

        let point_a_idx = out.points.len() as u32;
        out.points.push(points[0]);
        out.intersection_vertices.push(true);

        let point_b_idx = out.points.len() as u32;
        out.points.push(points[1]);
        out.intersection_vertices.push(true);

        let Some(new_tris) = split_triangle_indices(tri, edge_a, point_a_idx, edge_b, point_b_idx) else {
            out.complex_triangle_count += 1;
            out.indices.extend_from_slice(&[tri[0], tri[1], tri[2]]);
            continue;
        };

        out.split_triangle_count += 1;
        for t in new_tris {
            out.indices.extend_from_slice(&[t[0], t[1], t[2]]);
        }
    }

    out
}

fn boolean_keep_triangle(op: BooleanOp, source: TriangleSource, containment: TriangleContainment) -> bool {
    match op {
        BooleanOp::Union => !matches!(containment, TriangleContainment::Inside),
        BooleanOp::Intersection => matches!(containment, TriangleContainment::Inside | TriangleContainment::OnSurface),
        BooleanOp::Difference => match source {
            TriangleSource::A => !matches!(containment, TriangleContainment::Inside),
            TriangleSource::B => matches!(containment, TriangleContainment::Inside | TriangleContainment::OnSurface),
        },
    }
}

fn ray_cast_containment_prepared(point: Point3, mesh: &PreparedMesh, dir: Vec3, tol: Tolerance) -> RayVote {
    let mut intersections = 0usize;
    let mut ambiguous = false;
    let mut on_surface = false;

    let edge_eps = tol.eps * 10.0;
    let point_box = BBox::new(point, point).expand_by(edge_eps);

    mesh.bvh.query_bbox(point_box, |tri_idx| {
        let tri = mesh.triangles[tri_idx];
        let tri = Triangle3::new(
            mesh.points[tri[0] as usize],
            mesh.points[tri[1] as usize],
            mesh.points[tri[2] as usize],
        );
        if point_on_triangle(point, tri, tol) {
            on_surface = true;
            return false;
        }
        true
    });

    if on_surface {
        return RayVote::Certain(PointContainment::OnSurface);
    }

    mesh.bvh.query_ray(point, dir, 0.0, f64::INFINITY, |tri_idx| {
        let tri = mesh.triangles[tri_idx];
        let tri = Triangle3::new(
            mesh.points[tri[0] as usize],
            mesh.points[tri[1] as usize],
            mesh.points[tri[2] as usize],
        );

        if point_on_triangle(point, tri, tol) {
            on_surface = true;
            return false;
        }

        let Some(hit) = ray_triangle_intersection(point, dir, tri, tol) else {
            return true;
        };

        if hit.t <= tol.eps {
            return true;
        }

        let w = 1.0 - hit.u - hit.v;
        if hit.u.abs() <= edge_eps || hit.v.abs() <= edge_eps || w.abs() <= edge_eps {
            ambiguous = true;
            return false;
        }

        intersections += 1;
        true
    });

    if on_surface {
        return RayVote::Certain(PointContainment::OnSurface);
    }

    if ambiguous {
        return RayVote::Ambiguous;
    }

    let inside = (intersections % 2) == 1;
    RayVote::Certain(if inside {
        PointContainment::Inside
    } else {
        PointContainment::Outside
    })
}

fn classify_point_in_prepared_mesh(point: Point3, mesh: &PreparedMesh, tol: Tolerance) -> PointContainment {
    if !mesh.bbox.expand_tolerance(tol).contains_point(point) {
        return PointContainment::Outside;
    }

    let dirs = [
        Vec3::new(1.0, 0.234_567_89, 0.345_678_91),
        Vec3::new(0.345_678_91, 1.0, 0.234_567_89),
        Vec3::new(0.234_567_89, 0.345_678_91, 1.0),
    ];

    let mut vote: Option<PointContainment> = None;
    for dir in dirs {
        let dir = dir.normalized().unwrap_or(Vec3::X);
        match ray_cast_containment_prepared(point, mesh, dir, tol) {
            RayVote::Certain(PointContainment::OnSurface) => return PointContainment::OnSurface,
            RayVote::Certain(result) => match vote {
                None => vote = Some(result),
                Some(prev) if prev == result => return result,
                Some(_) => return PointContainment::Indeterminate,
            },
            RayVote::Ambiguous => continue,
        }
    }

    vote.unwrap_or(PointContainment::Indeterminate)
}

fn classify_split_triangles(
    points: &[Point3],
    indices: &[u32],
    other: &PreparedMesh,
    tol: Tolerance,
) -> Vec<TriangleContainment> {
    let mut out = Vec::with_capacity(indices.len() / 3);
    for tri in indices.chunks_exact(3) {
        let Some(a) = points.get(tri[0] as usize).copied() else {
            out.push(TriangleContainment::Indeterminate);
            continue;
        };
        let Some(b) = points.get(tri[1] as usize).copied() else {
            out.push(TriangleContainment::Indeterminate);
            continue;
        };
        let Some(c) = points.get(tri[2] as usize).copied() else {
            out.push(TriangleContainment::Indeterminate);
            continue;
        };

        let centroid = Point3::new(
            (a.x + b.x + c.x) / 3.0,
            (a.y + b.y + c.y) / 3.0,
            (a.z + b.z + c.z) / 3.0,
        );

        let containment = match classify_point_in_prepared_mesh(centroid, other, tol) {
            PointContainment::Inside => TriangleContainment::Inside,
            PointContainment::Outside => TriangleContainment::Outside,
            PointContainment::OnSurface => TriangleContainment::OnSurface,
            PointContainment::Indeterminate => TriangleContainment::Indeterminate,
        };
        out.push(containment);
    }
    out
}

fn boolean_meshes_no_fallback(
    mesh_a: &GeomMesh,
    mesh_b: &GeomMesh,
    op: BooleanOp,
    tol: Tolerance,
) -> Result<BooleanResult, BooleanError> {
    let a = prepare_mesh(mesh_a, tol)?;
    let b = prepare_mesh(mesh_b, tol)?;

    let tri_count_a = a.triangles.len();
    let tri_count_b = b.triangles.len();

    let mut per_a: Vec<Vec<Point3>> = vec![Vec::new(); tri_count_a];
    let mut per_b: Vec<Vec<Point3>> = vec![Vec::new(); tri_count_b];

    let mut diagnostics = BooleanDiagnostics {
        op,
        input_a_vertex_count: a.points.len(),
        input_a_triangle_count: tri_count_a,
        input_b_vertex_count: b.points.len(),
        input_b_triangle_count: tri_count_b,
        tolerance_used: tol.eps,
        ..Default::default()
    };

    for (ia, tri_a) in a.triangles.iter().copied().enumerate() {
        let bbox_a = a.tri_bboxes[ia];
        let tri_a_geom = Triangle3::new(
            a.points[tri_a[0] as usize],
            a.points[tri_a[1] as usize],
            a.points[tri_a[2] as usize],
        );

        let per_a_points = &mut per_a[ia];
        b.bvh.query_bbox(bbox_a, |ib| {
            let bbox_b = b.tri_bboxes[ib];
            if !bbox_a.intersects(bbox_b) {
                return true;
            }

            let tri_b = b.triangles[ib];
            let tri_b_geom = Triangle3::new(
                b.points[tri_b[0] as usize],
                b.points[tri_b[1] as usize],
                b.points[tri_b[2] as usize],
            );

            let Some(hit) = triangle_triangle_intersection(tri_a_geom, tri_b_geom, tol) else {
                return true;
            };

            match hit {
                TriTriIntersection::Segment(seg) => {
                    diagnostics.intersection_segment_count += 1;
                    per_a_points.push(seg.a);
                    per_a_points.push(seg.b);
                    per_b[ib].push(seg.a);
                    per_b[ib].push(seg.b);
                }
                TriTriIntersection::Point(p) => {
                    diagnostics.intersection_point_count += 1;
                    per_a_points.push(p);
                    per_b[ib].push(p);
                }
                TriTriIntersection::Coplanar => {
                    diagnostics.coplanar_pair_count += 1;
                }
            }

            true
        });
    }

    for pts in &mut per_a {
        dedup_points(pts, tol);
    }
    for pts in &mut per_b {
        dedup_points(pts, tol);
    }

    let split_a = split_mesh_by_intersections(&a, &per_a, tol);
    let split_b = split_mesh_by_intersections(&b, &per_b, tol);

    diagnostics.split_triangle_count_a = split_a.split_triangle_count;
    diagnostics.split_triangle_count_b = split_b.split_triangle_count;
    diagnostics.complex_triangle_count_a = split_a.complex_triangle_count;
    diagnostics.complex_triangle_count_b = split_b.complex_triangle_count;

    if diagnostics.coplanar_pair_count > 0 {
        diagnostics
            .warnings
            .push("coplanar triangle pairs detected; boolean may be unstable".to_string());
    }
    if diagnostics.complex_triangle_count_a > 0 || diagnostics.complex_triangle_count_b > 0 {
        diagnostics.warnings.push(
            "some triangles had complex intersections; partial cutting was applied".to_string(),
        );
    }

    // Classification and filtering
    let mut out_points = split_a.points.clone();
    let mut out_indices: Vec<u32> = Vec::new();

    let class_a = classify_split_triangles(&split_a.points, &split_a.indices, &b, tol);

    for (ti, tri) in split_a.indices.chunks_exact(3).enumerate() {
        let containment = class_a
            .get(ti)
            .copied()
            .unwrap_or(TriangleContainment::Indeterminate);
        if containment == TriangleContainment::Indeterminate {
            diagnostics.indeterminate_triangle_count += 1;
        }
        if boolean_keep_triangle(op, TriangleSource::A, containment) {
            diagnostics.kept_triangle_count_a += 1;
            out_indices.extend_from_slice(tri);
        }
    }

    let base_b = out_points.len() as u32;
    out_points.extend_from_slice(&split_b.points);

    let class_b = classify_split_triangles(&split_b.points, &split_b.indices, &a, tol);

    for (ti, tri) in split_b.indices.chunks_exact(3).enumerate() {
        let containment = class_b
            .get(ti)
            .copied()
            .unwrap_or(TriangleContainment::Indeterminate);
        if containment == TriangleContainment::Indeterminate {
            diagnostics.indeterminate_triangle_count += 1;
        }
        if !boolean_keep_triangle(op, TriangleSource::B, containment) {
            continue;
        }
        diagnostics.kept_triangle_count_b += 1;
        let mut t = [tri[0] + base_b, tri[1] + base_b, tri[2] + base_b];
        if op == BooleanOp::Difference {
            t.swap(1, 2);
        }
        out_indices.extend_from_slice(&t);
    }

    let (mesh, mut mesh_diag) = super::mesh::finalize_mesh(out_points, None, out_indices, tol);
    if diagnostics.tolerance_relaxed || diagnostics.voxel_fallback_used {
        mesh_diag.boolean_fallback_used = true;
    }
    mesh_diag.warnings.extend(diagnostics.warnings.clone());

    Ok(BooleanResult {
        mesh,
        mesh_diagnostics: mesh_diag,
        diagnostics,
    })
}

fn boolean_quality_ok(result: &BooleanResult) -> bool {
    result.mesh_diagnostics.is_valid_solid()
        && result.diagnostics.indeterminate_triangle_count == 0
        && result.diagnostics.coplanar_pair_count == 0
        && result.diagnostics.complex_triangle_count_a == 0
        && result.diagnostics.complex_triangle_count_b == 0
}

fn boolean_quality_score(result: &BooleanResult) -> i64 {
    let open = result.mesh_diagnostics.open_edge_count as i64;
    let non_manifold = result.mesh_diagnostics.non_manifold_edge_count as i64;
    let indeterminate = result.diagnostics.indeterminate_triangle_count as i64;
    let complex =
        (result.diagnostics.complex_triangle_count_a + result.diagnostics.complex_triangle_count_b)
            as i64;
    open * 10_000 + non_manifold * 1_000_000 + indeterminate * 100 + complex * 1_000
}

fn voxel_boolean_meshes(
    mesh_a: &GeomMesh,
    mesh_b: &GeomMesh,
    op: BooleanOp,
    tol: Tolerance,
    resolution: usize,
) -> Result<BooleanResult, BooleanError> {
    let a = prepare_mesh(mesh_a, tol)?;
    let b = prepare_mesh(mesh_b, tol)?;

    let bbox = a.bbox.union(b.bbox).expand_tolerance(tol);
    let size = bbox.size();
    let max_dim = size.x.max(size.y).max(size.z);
    if !max_dim.is_finite() || max_dim <= tol.eps {
        return Err(BooleanError::InvalidGeometry);
    }

    let resolution = resolution.max(4).min(256);
    let cell = max_dim / resolution as f64;
    if !cell.is_finite() || cell <= tol.eps {
        return Err(BooleanError::InvalidGeometry);
    }

    let nx = ((size.x / cell).ceil() as usize).max(1);
    let ny = ((size.y / cell).ceil() as usize).max(1);
    let nz = ((size.z / cell).ceil() as usize).max(1);

    let total = nx.saturating_mul(ny).saturating_mul(nz);
    if total == 0 || total > 2_000_000 {
        return Err(BooleanError::VoxelGridTooLarge);
    }

    let idx = |x: usize, y: usize, z: usize| -> usize { x + nx * (y + ny * z) };
    let mut inside = vec![false; total];

    for z in 0..nz {
        for y in 0..ny {
            for x in 0..nx {
                let center = Point3::new(
                    bbox.min.x + (x as f64 + 0.5) * cell,
                    bbox.min.y + (y as f64 + 0.5) * cell,
                    bbox.min.z + (z as f64 + 0.5) * cell,
                );

                let in_a = matches!(
                    classify_point_in_prepared_mesh(center, &a, tol),
                    PointContainment::Inside | PointContainment::OnSurface
                );
                let in_b = matches!(
                    classify_point_in_prepared_mesh(center, &b, tol),
                    PointContainment::Inside | PointContainment::OnSurface
                );

                let in_result = match op {
                    BooleanOp::Union => in_a || in_b,
                    BooleanOp::Intersection => in_a && in_b,
                    BooleanOp::Difference => in_a && !in_b,
                };

                inside[idx(x, y, z)] = in_result;
            }
        }
    }

    use std::collections::HashMap;
    let mut vertex_map: HashMap<(i32, i32, i32), u32> = HashMap::new();
    let mut points: Vec<Point3> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let mut get_corner = |cx: i32, cy: i32, cz: i32| -> u32 {
        if let Some(&idx) = vertex_map.get(&(cx, cy, cz)) {
            return idx;
        }
        let p = Point3::new(
            bbox.min.x + cx as f64 * cell,
            bbox.min.y + cy as f64 * cell,
            bbox.min.z + cz as f64 * cell,
        );
        let idx = points.len() as u32;
        points.push(p);
        vertex_map.insert((cx, cy, cz), idx);
        idx
    };

    for z in 0..nz {
        for y in 0..ny {
            for x in 0..nx {
                if !inside[idx(x, y, z)] {
                    continue;
                }

                let x0 = x as i32;
                let y0 = y as i32;
                let z0 = z as i32;
                let x1 = x0 + 1;
                let y1 = y0 + 1;
                let z1 = z0 + 1;

                // -X
                if x == 0 || !inside[idx(x - 1, y, z)] {
                    let v0 = get_corner(x0, y0, z0);
                    let v1 = get_corner(x0, y0, z1);
                    let v2 = get_corner(x0, y1, z1);
                    let v3 = get_corner(x0, y1, z0);
                    indices.extend_from_slice(&[v0, v1, v2, v0, v2, v3]);
                }

                // +X
                if x + 1 >= nx || !inside[idx(x + 1, y, z)] {
                    let v0 = get_corner(x1, y0, z0);
                    let v1 = get_corner(x1, y1, z0);
                    let v2 = get_corner(x1, y1, z1);
                    let v3 = get_corner(x1, y0, z1);
                    indices.extend_from_slice(&[v0, v1, v2, v0, v2, v3]);
                }

                // -Y
                if y == 0 || !inside[idx(x, y - 1, z)] {
                    let v0 = get_corner(x0, y0, z0);
                    let v1 = get_corner(x1, y0, z0);
                    let v2 = get_corner(x1, y0, z1);
                    let v3 = get_corner(x0, y0, z1);
                    indices.extend_from_slice(&[v0, v1, v2, v0, v2, v3]);
                }

                // +Y
                if y + 1 >= ny || !inside[idx(x, y + 1, z)] {
                    let v0 = get_corner(x0, y1, z0);
                    let v1 = get_corner(x0, y1, z1);
                    let v2 = get_corner(x1, y1, z1);
                    let v3 = get_corner(x1, y1, z0);
                    indices.extend_from_slice(&[v0, v1, v2, v0, v2, v3]);
                }

                // -Z
                if z == 0 || !inside[idx(x, y, z - 1)] {
                    let v0 = get_corner(x0, y0, z0);
                    let v1 = get_corner(x0, y1, z0);
                    let v2 = get_corner(x1, y1, z0);
                    let v3 = get_corner(x1, y0, z0);
                    indices.extend_from_slice(&[v0, v1, v2, v0, v2, v3]);
                }

                // +Z
                if z + 1 >= nz || !inside[idx(x, y, z + 1)] {
                    let v0 = get_corner(x0, y0, z1);
                    let v1 = get_corner(x1, y0, z1);
                    let v2 = get_corner(x1, y1, z1);
                    let v3 = get_corner(x0, y1, z1);
                    indices.extend_from_slice(&[v0, v1, v2, v0, v2, v3]);
                }
            }
        }
    }

    let (mesh, mut mesh_diag) = super::mesh::finalize_mesh(points, None, indices, tol);
    mesh_diag.boolean_fallback_used = true;
    mesh_diag.warnings.push("boolean voxel fallback used".to_string());

    let diagnostics = BooleanDiagnostics {
        op,
        input_a_vertex_count: a.points.len(),
        input_a_triangle_count: a.triangles.len(),
        input_b_vertex_count: b.points.len(),
        input_b_triangle_count: b.triangles.len(),
        voxel_fallback_used: true,
        tolerance_used: tol.eps,
        warnings: vec!["boolean voxel fallback used".to_string()],
        ..Default::default()
    };

    Ok(BooleanResult {
        mesh,
        mesh_diagnostics: mesh_diag,
        diagnostics,
    })
}

/// Mesh boolean (geom-only implementation, triangle meshes only).
#[must_use]
pub fn boolean_meshes(
    mesh_a: &GeomMesh,
    mesh_b: &GeomMesh,
    op: BooleanOp,
    tol: Tolerance,
) -> Result<BooleanResult, BooleanError> {
    let mut best: Option<BooleanResult> = None;
    let mut last_err: Option<BooleanError> = None;

    let relax_factors = [1.0, 10.0, 100.0];
    for (attempt, factor) in relax_factors.into_iter().enumerate() {
        let attempt_tol = tol.scaled(factor);
        let mut result = match boolean_meshes_no_fallback(mesh_a, mesh_b, op, attempt_tol) {
            Ok(result) => result,
            Err(err) => {
                last_err = Some(err);
                continue;
            }
        };

        if attempt > 0 {
            result.diagnostics.tolerance_relaxed = true;
            result.diagnostics.tolerance_used = attempt_tol.eps;
            let warning = format!(
                "boolean used tolerance relaxation fallback (eps={:.3e})",
                attempt_tol.eps
            );
            result.diagnostics.warnings.push(warning.clone());
            result.mesh_diagnostics.boolean_fallback_used = true;
            result.mesh_diagnostics.warnings.push(warning);
        }

        if boolean_quality_ok(&result) {
            return Ok(result);
        }

        match best.as_ref() {
            None => best = Some(result),
            Some(prev) => {
                if boolean_quality_score(&result) < boolean_quality_score(prev) {
                    best = Some(result);
                }
            }
        }
    }

    if let Some(ref best) = best {
        if best.mesh_diagnostics.is_valid_solid() {
            return Ok(best.clone());
        }
    }

    if let Ok(voxel) = voxel_boolean_meshes(mesh_a, mesh_b, op, tol, 32) {
        return Ok(voxel);
    }

    if let Some(best) = best {
        return Ok(best);
    }

    Err(last_err.unwrap_or(BooleanError::EmptyMesh))
}

#[derive(Debug, thiserror::Error)]
pub enum BooleanError {
    #[error("mesh is empty")]
    EmptyMesh,
    #[error("mesh contains invalid (non-finite) geometry")]
    InvalidGeometry,
    #[error("mesh contains invalid indices")]
    InvalidIndices,
    #[error("voxel fallback grid too large")]
    VoxelGridTooLarge,
}
