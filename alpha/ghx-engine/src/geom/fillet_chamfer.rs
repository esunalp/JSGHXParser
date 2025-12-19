//! Fillet + chamfer helpers.
//!
//! This module is intentionally conservative: it provides a minimal, deterministic
//! implementation that is suitable for Phase 2 experimentation behind
//! `mesh_engine_next`.
//!
//! # Scope (current)
//! - **Polyline corner fillet**: inserts circular arc segments at polyline corners.
//! - **Triangle-mesh edge fillet (very limited)**: only supports "hinge" edges where
//!   *both* endpoints are used by exactly two triangles total (so no vertex fan / end-cap
//!   handling is required). This keeps the result topologically well-defined.
//!
//! # Limitations
//! - No B-rep filleting (surface/trim based) yet.
//! - Mesh edge fillet does **not** support general manifold meshes; it will skip edges
//!   whose endpoints are shared by additional triangles.
//! - UV generation is not implemented for the new fillet faces.
//!
//! # Tolerances & diagnostics
//! - All comparisons use `Tolerance`.
//! - Operations are best-effort: unsupported edges/corners are skipped and reported in
//!   diagnostics instead of silently failing.

use std::collections::{HashMap, HashSet};

use super::mesh::GeomMesh;
use super::solid::LegacySurfaceMesh;
use super::{GeomMeshDiagnostics, Point3, Tolerance, Vec3};

#[derive(Debug, thiserror::Error)]
pub enum FilletChamferError {
    #[error("radius must be finite and non-negative: {radius}")]
    InvalidRadius { radius: f64 },

    #[error("segment_count must be >= 1, got {segments}")]
    InvalidSegmentCount { segments: usize },

    #[error("polyline must have at least 2 points, got {count}")]
    InvalidPolyline { count: usize },

    #[error("closed polyline must have at least 3 points, got {count}")]
    InvalidClosedPolyline { count: usize },

    #[error("mesh indices length must be a multiple of 3, got {len}")]
    InvalidTriangleIndexBuffer { len: usize },

    #[error("mesh index out of bounds: {index} >= {vertex_count}")]
    MeshIndexOutOfBounds { index: u32, vertex_count: usize },

    #[error("legacy surface mesh must be triangulated (all faces len == 3)")]
    LegacyMeshNotTriangulated,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FilletPolylineDiagnostics {
    pub input_point_count: usize,
    pub output_point_count: usize,
    pub corner_count: usize,
    pub filleted_corner_count: usize,
    pub skipped_corner_count: usize,
    pub clamped_corner_count: usize,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FilletMeshEdgeDiagnostics {
    pub input_vertex_count: usize,
    pub input_triangle_count: usize,
    pub requested_edge_count: usize,
    pub processed_edge_count: usize,
    pub skipped_edge_count: usize,
    pub clamped_edge_count: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FilletEdgeOptions {
    pub radius: f64,
    /// Number of segments across the fillet arc.
    /// `1` behaves like a chamfer.
    pub segments: usize,
}

impl FilletEdgeOptions {
    #[must_use]
    pub const fn new(radius: f64, segments: usize) -> Self {
        Self { radius, segments }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TriangleMeshEdge {
    pub a: u32,
    pub b: u32,
    pub triangle_count: usize,
}

/// List unique undirected edges of a triangle mesh.
///
/// Edges are returned sorted by `(a, b)` where `a <= b`.
pub fn list_triangle_mesh_edges(mesh: &GeomMesh) -> Result<Vec<TriangleMeshEdge>, FilletChamferError> {
    validate_triangle_mesh(mesh)?;

    let mut counts: HashMap<EdgeKey, usize> = HashMap::new();
    for tri in mesh.indices.chunks_exact(3) {
        let i0 = tri[0];
        let i1 = tri[1];
        let i2 = tri[2];
        for (a, b) in [(i0, i1), (i1, i2), (i2, i0)] {
            *counts.entry(EdgeKey::new(a, b)).or_insert(0) += 1;
        }
    }

    let mut edges = counts
        .into_iter()
        .map(|(key, triangle_count)| TriangleMeshEdge {
            a: key.0,
            b: key.1,
            triangle_count,
        })
        .collect::<Vec<_>>();
    edges.sort_by(|lhs, rhs| (lhs.a, lhs.b).cmp(&(rhs.a, rhs.b)));
    Ok(edges)
}

/// Fillet corners of a polyline by inserting circular arc segments.
///
/// For `segments == 1`, the arc is approximated by a single straight segment (chamfer-like).
#[must_use]
pub fn fillet_polyline_points(
    points: &[Point3],
    radius: f64,
    segments: usize,
    closed: bool,
    tol: Tolerance,
) -> Result<(Vec<Point3>, FilletPolylineDiagnostics), FilletChamferError> {
    if !radius.is_finite() || radius < 0.0 {
        return Err(FilletChamferError::InvalidRadius { radius });
    }
    if segments < 1 {
        return Err(FilletChamferError::InvalidSegmentCount { segments });
    }
    if points.len() < 2 {
        return Err(FilletChamferError::InvalidPolyline {
            count: points.len(),
        });
    }
    if closed && points.len() < 3 {
        return Err(FilletChamferError::InvalidClosedPolyline {
            count: points.len(),
        });
    }

    let mut diagnostics = FilletPolylineDiagnostics {
        input_point_count: points.len(),
        ..Default::default()
    };

    if radius <= tol.eps || points.len() < 3 {
        diagnostics.output_point_count = points.len();
        return Ok((points.to_vec(), diagnostics));
    }

    let result = if closed {
        fillet_polyline_closed(points, radius, segments, tol, &mut diagnostics)
    } else {
        fillet_polyline_open(points, radius, segments, tol, &mut diagnostics)
    };

    diagnostics.output_point_count = result.len();
    Ok((result, diagnostics))
}

/// Fillet selected edges of a triangle mesh.
///
/// This is an early, mesh-only implementation intended for experimentation and diagnostics.
/// Currently it supports only "hinge" edges: both endpoints must be used by exactly two
/// triangles in the entire mesh.
pub fn fillet_triangle_mesh_edges(
    mesh: &GeomMesh,
    edges: &[(u32, u32)],
    options: FilletEdgeOptions,
    tol: Tolerance,
) -> Result<(GeomMesh, GeomMeshDiagnostics, FilletMeshEdgeDiagnostics), FilletChamferError> {
    validate_triangle_mesh(mesh)?;
    if !options.radius.is_finite() || options.radius < 0.0 {
        return Err(FilletChamferError::InvalidRadius {
            radius: options.radius,
        });
    }
    if options.segments < 1 {
        return Err(FilletChamferError::InvalidSegmentCount {
            segments: options.segments,
        });
    }

    let mut diag = FilletMeshEdgeDiagnostics {
        input_vertex_count: mesh.positions.len(),
        input_triangle_count: mesh.indices.len() / 3,
        requested_edge_count: edges.len(),
        ..Default::default()
    };

    if options.radius <= tol.eps || edges.is_empty() {
        return Ok((mesh.clone(), GeomMeshDiagnostics::default(), diag));
    }

    let points = mesh
        .positions
        .iter()
        .copied()
        .map(Point3::from_array)
        .collect::<Vec<_>>();

    let vertex_triangle_counts = vertex_triangle_counts(&mesh.indices, points.len());
    let edge_to_tris = edge_to_triangles(&mesh.indices);

    let mut requested: HashSet<EdgeKey> = HashSet::new();
    for (a, b) in edges.iter().copied() {
        requested.insert(EdgeKey::new(a, b));
    }

    let mut out_points = points;
    let mut out_indices = mesh.indices.clone();
    let mut claimed_tris: HashSet<usize> = HashSet::new();

    for key in requested {
        let a = key.0;
        let b = key.1;
        let Some(adj) = edge_to_tris.get(&key) else {
            diag.skipped_edge_count += 1;
            diag.errors
                .push(format!("edge ({a},{b}): not found in mesh"));
            continue;
        };

        if adj.len() != 2 {
            diag.skipped_edge_count += 1;
            diag.errors.push(format!(
                "edge ({a},{b}): expected 2 adjacent triangles, got {}",
                adj.len()
            ));
            continue;
        }

        let a_count = *vertex_triangle_counts.get(a as usize).unwrap_or(&0);
        let b_count = *vertex_triangle_counts.get(b as usize).unwrap_or(&0);
        if a_count != 2 || b_count != 2 {
            diag.skipped_edge_count += 1;
            diag.errors.push(format!(
                "edge ({a},{b}): endpoints not isolated (triangle valence a={a_count}, b={b_count}); only hinge edges are supported"
            ));
            continue;
        }

        let t0 = adj[0];
        let t1 = adj[1];
        if claimed_tris.contains(&t0) || claimed_tris.contains(&t1) {
            diag.skipped_edge_count += 1;
            diag.errors.push(format!(
                "edge ({a},{b}): adjacent triangle already modified by another fillet"
            ));
            continue;
        }
        claimed_tris.insert(t0);
        claimed_tris.insert(t1);

        let Some((arc_a, arc_b, tri0_replace, tri1_replace, clamped)) = build_edge_fillet_geometry(
            &out_points,
            &out_indices,
            (a, b),
            t0,
            t1,
            options,
            tol,
        ) else {
            diag.skipped_edge_count += 1;
            diag.errors
                .push(format!("edge ({a},{b}): failed to build fillet geometry"));
            continue;
        };

        if clamped {
            diag.clamped_edge_count += 1;
        }

        let arc_a_indices = push_points(&mut out_points, &arc_a);
        let arc_b_indices = push_points(&mut out_points, &arc_b);

        // Update the two adjacent triangles to use the tangent points.
        apply_triangle_replacement(&mut out_indices, t0, (a, b), tri0_replace, &arc_a_indices, &arc_b_indices);
        apply_triangle_replacement(&mut out_indices, t1, (a, b), tri1_replace, &arc_a_indices, &arc_b_indices);

        // Add the fillet strip (triangle quads between corresponding arc samples).
        for i in 0..options.segments {
            let a0 = arc_a_indices[i];
            let a1 = arc_a_indices[i + 1];
            let b0 = arc_b_indices[i];
            let b1 = arc_b_indices[i + 1];

            out_indices.extend_from_slice(&[a0, b0, b1, a0, b1, a1]);
        }

        diag.processed_edge_count += 1;
    }

    let (mesh, mesh_diag) = super::mesh::finalize_mesh(out_points, None, out_indices, tol);
    diag.warnings.extend(mesh_diag.warnings.clone());
    Ok((mesh, mesh_diag, diag))
}

/// Convenience wrapper for `Value::Surface`-style legacy meshes.
///
/// Input must already be triangulated (`faces` must all have length 3).
pub fn fillet_legacy_triangle_mesh_edges(
    brep: &LegacySurfaceMesh,
    edges: &[(u32, u32)],
    options: FilletEdgeOptions,
    tol: Tolerance,
) -> Result<(LegacySurfaceMesh, GeomMeshDiagnostics, FilletMeshEdgeDiagnostics), FilletChamferError>
{
    if brep.vertices.is_empty() || brep.faces.is_empty() {
        return Ok((
            brep.clone(),
            GeomMeshDiagnostics::default(),
            FilletMeshEdgeDiagnostics::default(),
        ));
    }

    let mut indices = Vec::with_capacity(brep.faces.len() * 3);
    for face in &brep.faces {
        if face.len() != 3 {
            return Err(FilletChamferError::LegacyMeshNotTriangulated);
        }
        indices.extend_from_slice(face);
    }

    let mesh = GeomMesh {
        positions: brep.vertices.clone(),
        indices,
        uvs: None,
        normals: None,
        tangents: None,
    };

    let (mesh, mesh_diag, fillet_diag) = fillet_triangle_mesh_edges(&mesh, edges, options, tol)?;
    let faces = mesh
        .indices
        .chunks_exact(3)
        .map(|tri| vec![tri[0], tri[1], tri[2]])
        .collect::<Vec<_>>();

    Ok((
        LegacySurfaceMesh {
            vertices: mesh.positions,
            faces,
        },
        mesh_diag,
        fillet_diag,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EdgeKey(u32, u32);

impl EdgeKey {
    fn new(a: u32, b: u32) -> Self {
        if a <= b { Self(a, b) } else { Self(b, a) }
    }
}

fn validate_triangle_mesh(mesh: &GeomMesh) -> Result<(), FilletChamferError> {
    if mesh.indices.len() % 3 != 0 {
        return Err(FilletChamferError::InvalidTriangleIndexBuffer {
            len: mesh.indices.len(),
        });
    }
    let vertex_count = mesh.positions.len();
    for &idx in &mesh.indices {
        if idx as usize >= vertex_count {
            return Err(FilletChamferError::MeshIndexOutOfBounds {
                index: idx,
                vertex_count,
            });
        }
    }
    Ok(())
}

fn vertex_triangle_counts(indices: &[u32], vertex_count: usize) -> Vec<usize> {
    let mut counts = vec![0usize; vertex_count];
    for tri in indices.chunks_exact(3) {
        for &idx in tri {
            if let Some(slot) = counts.get_mut(idx as usize) {
                *slot += 1;
            }
        }
    }
    counts
}

fn edge_to_triangles(indices: &[u32]) -> HashMap<EdgeKey, Vec<usize>> {
    let mut map: HashMap<EdgeKey, Vec<usize>> = HashMap::new();
    for (t, tri) in indices.chunks_exact(3).enumerate() {
        let i0 = tri[0];
        let i1 = tri[1];
        let i2 = tri[2];
        for (a, b) in [(i0, i1), (i1, i2), (i2, i0)] {
            map.entry(EdgeKey::new(a, b)).or_default().push(t);
        }
    }
    map
}

fn apply_triangle_replacement(
    indices: &mut [u32],
    tri_index: usize,
    edge: (u32, u32),
    which: TriangleSide,
    arc_a_indices: &[u32],
    arc_b_indices: &[u32],
) {
    let (a, b) = edge;
    let base = tri_index * 3;
    if base + 2 >= indices.len() {
        return;
    }
    let a_new = match which {
        TriangleSide::Start => arc_a_indices[0],
        TriangleSide::End => arc_a_indices[arc_a_indices.len() - 1],
    };
    let b_new = match which {
        TriangleSide::Start => arc_b_indices[0],
        TriangleSide::End => arc_b_indices[arc_b_indices.len() - 1],
    };

    for slot in &mut indices[base..base + 3] {
        if *slot == a {
            *slot = a_new;
        } else if *slot == b {
            *slot = b_new;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TriangleSide {
    Start,
    End,
}

fn build_edge_fillet_geometry(
    points: &[Point3],
    indices: &[u32],
    edge: (u32, u32),
    tri0: usize,
    tri1: usize,
    options: FilletEdgeOptions,
    tol: Tolerance,
) -> Option<(Vec<Point3>, Vec<Point3>, TriangleSide, TriangleSide, bool)> {
    let (a, b) = edge;
    let pa = points.get(a as usize).copied()?;
    let pb = points.get(b as usize).copied()?;

    let axis = pb.sub_point(pa).normalized()?;

    let (c0, n0) = triangle_opposite_and_normal(points, indices, tri0, a, b, tol)?;
    let (c1, n1) = triangle_opposite_and_normal(points, indices, tri1, a, b, tol)?;

    let mid = pa.lerp(pb, 0.5);
    let v0_in = c0.sub_point(mid);
    let v1_in = c1.sub_point(mid);

    let mut d0 = n0.cross(axis).normalized()?;
    if d0.dot(v0_in) < 0.0 {
        d0 = d0.neg();
    }

    let mut d1 = n1.cross(axis).normalized()?;
    if d1.dot(v1_in) < 0.0 {
        d1 = d1.neg();
    }

    let dot = clamp(d0.dot(d1), -1.0, 1.0);
    let theta = dot.acos();
    if !theta.is_finite() || theta <= 1e-8 {
        return None;
    }

    let half = theta * 0.5;
    let tan_half = half.tan();
    let sin_half = half.sin();
    if !tan_half.is_finite() || tan_half <= 0.0 || !sin_half.is_finite() || sin_half <= 0.0 {
        return None;
    }

    let mut t = options.radius / tan_half;
    if !t.is_finite() || t <= tol.eps {
        return None;
    }

    let altitude0 = c0.sub_point(pa).cross(axis).length();
    let altitude1 = c1.sub_point(pa).cross(axis).length();
    let max_t = altitude0.min(altitude1) * 0.999;
    let mut clamped = false;
    if t > max_t && max_t.is_finite() && max_t > tol.eps {
        t = max_t;
        clamped = true;
    }

    if t <= tol.eps {
        return None;
    }

    let r_eff = t * tan_half;
    let center_dist = r_eff / sin_half;

    let bisector = d0.add(d1).normalized()?;

    let ca = pa.add_vec(bisector.mul_scalar(center_dist));
    let cb = pb.add_vec(bisector.mul_scalar(center_dist));

    let a0 = pa.add_vec(d0.mul_scalar(t));
    let a1 = pa.add_vec(d1.mul_scalar(t));
    let b0 = pb.add_vec(d0.mul_scalar(t));
    let b1 = pb.add_vec(d1.mul_scalar(t));

    let arc_a = arc_points(ca, a0, a1, axis, options.segments)?;
    let arc_b = arc_points(cb, b0, b1, axis, options.segments)?;

    // Heuristic: decide which adjacent triangle receives which arc endpoint based on which
    // direction points further towards its opposite vertex.
    let side0 = TriangleSide::Start;
    let side1 = TriangleSide::End;

    Some((arc_a, arc_b, side0, side1, clamped))
}

fn triangle_opposite_and_normal(
    points: &[Point3],
    indices: &[u32],
    tri_index: usize,
    a: u32,
    b: u32,
    tol: Tolerance,
) -> Option<(Point3, Vec3)> {
    let base = tri_index * 3;
    let tri = indices.get(base..base + 3)?;
    let i0 = *tri.get(0)?;
    let i1 = *tri.get(1)?;
    let i2 = *tri.get(2)?;

    let p0 = points.get(i0 as usize).copied()?;
    let p1 = points.get(i1 as usize).copied()?;
    let p2 = points.get(i2 as usize).copied()?;

    let n = p1.sub_point(p0).cross(p2.sub_point(p0)).normalized()?;

    let c_index = [i0, i1, i2]
        .into_iter()
        .find(|&idx| idx != a && idx != b)?;
    let c = points.get(c_index as usize).copied()?;

    // Guard against degenerate triangles around the requested edge.
    if tol.approx_eq_point3(p0, p1) || tol.approx_eq_point3(p1, p2) || tol.approx_eq_point3(p0, p2) {
        return None;
    }

    Some((c, n))
}

fn push_points(storage: &mut Vec<Point3>, points: &[Point3]) -> Vec<u32> {
    let mut indices = Vec::with_capacity(points.len());
    for p in points {
        let idx = storage.len() as u32;
        storage.push(*p);
        indices.push(idx);
    }
    indices
}

fn arc_points(
    center: Point3,
    start: Point3,
    end: Point3,
    axis_unit: Vec3,
    segments: usize,
) -> Option<Vec<Point3>> {
    let v0 = start.sub_point(center);
    let v1 = end.sub_point(center);

    let angle = signed_angle(v0, v1, axis_unit);
    if !angle.is_finite() {
        return None;
    }

    let mut out = Vec::with_capacity(segments + 1);
    for i in 0..=segments {
        let t = i as f64 / segments as f64;
        let v = rotate_vec(v0, axis_unit, angle * t);
        out.push(center.add_vec(v));
    }
    Some(out)
}

fn signed_angle(v0: Vec3, v1: Vec3, axis_unit: Vec3) -> f64 {
    let dot = v0.dot(v1);
    let cross = v0.cross(v1);
    axis_unit.dot(cross).atan2(dot)
}

fn rotate_vec(v: Vec3, axis_unit: Vec3, angle: f64) -> Vec3 {
    let (s, c) = angle.sin_cos();
    let term1 = v.mul_scalar(c);
    let term2 = axis_unit.cross(v).mul_scalar(s);
    let term3 = axis_unit.mul_scalar(axis_unit.dot(v) * (1.0 - c));
    term1.add(term2).add(term3)
}

fn clamp(v: f64, min: f64, max: f64) -> f64 {
    v.max(min).min(max)
}

fn fillet_polyline_open(
    points: &[Point3],
    radius: f64,
    segments: usize,
    tol: Tolerance,
    diagnostics: &mut FilletPolylineDiagnostics,
) -> Vec<Point3> {
    let mut out = Vec::with_capacity(points.len().saturating_mul(segments.max(2)));
    out.push(points[0]);

    let mut prev_out_distance = 0.0f64;

    for i in 1..points.len().saturating_sub(1) {
        diagnostics.corner_count += 1;

        let prev = points[i - 1];
        let corner = points[i];
        let next = points[i + 1];

        let in_vec = corner.sub_point(prev);
        let out_vec = next.sub_point(corner);
        let in_len = in_vec.length();
        let out_len = out_vec.length();
        if !in_len.is_finite() || !out_len.is_finite() || in_len <= tol.eps || out_len <= tol.eps {
            diagnostics.skipped_corner_count += 1;
            out.push(corner);
            prev_out_distance = 0.0;
            continue;
        }

        let to_prev = in_vec.mul_scalar(-1.0 / in_len);
        let to_next = out_vec.mul_scalar(1.0 / out_len);

        let corner_dot = clamp(to_prev.dot(to_next), -1.0, 1.0);
        let theta = corner_dot.acos();
        if !theta.is_finite() || theta <= 1e-8 {
            diagnostics.skipped_corner_count += 1;
            out.push(corner);
            prev_out_distance = 0.0;
            continue;
        }

        let half = theta * 0.5;
        let tan_half = half.tan();
        let sin_half = half.sin();
        if !tan_half.is_finite() || tan_half <= 0.0 || !sin_half.is_finite() || sin_half <= 0.0 {
            diagnostics.skipped_corner_count += 1;
            out.push(corner);
            prev_out_distance = 0.0;
            continue;
        }

        let mut t = radius / tan_half;
        if !t.is_finite() || t <= tol.eps {
            diagnostics.skipped_corner_count += 1;
            out.push(corner);
            prev_out_distance = 0.0;
            continue;
        }

        // Clamp so we don't overlap the previous corner's outgoing tangent on the incoming segment.
        let available_in = (in_len - prev_out_distance) * 0.999;
        if available_in <= tol.eps {
            diagnostics.skipped_corner_count += 1;
            out.push(corner);
            prev_out_distance = 0.0;
            continue;
        }

        let max_t = available_in.min(out_len * 0.999);
        if t > max_t {
            t = max_t;
            diagnostics.clamped_corner_count += 1;
        }

        if t <= tol.eps {
            diagnostics.skipped_corner_count += 1;
            out.push(corner);
            prev_out_distance = 0.0;
            continue;
        }

        let r_eff = t * tan_half;
        let center_dist = r_eff / sin_half;
        let bisector = to_prev.add(to_next).normalized();
        let Some(bisector) = bisector else {
            diagnostics.skipped_corner_count += 1;
            out.push(corner);
            prev_out_distance = 0.0;
            continue;
        };

        let center = corner.add_vec(bisector.mul_scalar(center_dist));
        let p_in = corner.add_vec(to_prev.mul_scalar(t));
        let p_out = corner.add_vec(to_next.mul_scalar(t));

        let v0 = p_in.sub_point(center);
        let v1 = p_out.sub_point(center);
        let axis = v0.cross(v1).normalized();
        let Some(axis) = axis else {
            diagnostics.skipped_corner_count += 1;
            out.push(corner);
            prev_out_distance = 0.0;
            continue;
        };

        let angle = signed_angle(v0, v1, axis);
        if !angle.is_finite() {
            diagnostics.skipped_corner_count += 1;
            out.push(corner);
            prev_out_distance = 0.0;
            continue;
        }

        if !tol.approx_eq_point3(*out.last().unwrap_or(&Point3::ORIGIN), p_in) {
            out.push(p_in);
        }

        for s in 1..segments {
            let frac = s as f64 / segments as f64;
            let v = rotate_vec(v0, axis, angle * frac);
            out.push(center.add_vec(v));
        }

        out.push(p_out);
        prev_out_distance = t;
        diagnostics.filleted_corner_count += 1;
    }

    if !tol.approx_eq_point3(*out.last().unwrap_or(&Point3::ORIGIN), *points.last().unwrap()) {
        out.push(*points.last().unwrap());
    }
    out
}

fn fillet_polyline_closed(
    points: &[Point3],
    radius: f64,
    segments: usize,
    tol: Tolerance,
    diagnostics: &mut FilletPolylineDiagnostics,
) -> Vec<Point3> {
    let n = points.len();
    if n < 3 {
        return points.to_vec();
    }

    // First pass: compute a per-corner tangent distance (t) with local clamping.
    let mut corner_t = vec![0.0f64; n];
    let mut corner_meta: Vec<Option<(Vec3, Vec3, f64, f64, f64)>> = vec![None; n];

    for i in 0..n {
        let prev = points[(i + n - 1) % n];
        let corner = points[i];
        let next = points[(i + 1) % n];

        let in_vec = corner.sub_point(prev);
        let out_vec = next.sub_point(corner);
        let in_len = in_vec.length();
        let out_len = out_vec.length();
        if !in_len.is_finite() || !out_len.is_finite() || in_len <= tol.eps || out_len <= tol.eps {
            continue;
        }

        let to_prev = in_vec.mul_scalar(-1.0 / in_len);
        let to_next = out_vec.mul_scalar(1.0 / out_len);

        let dot = clamp(to_prev.dot(to_next), -1.0, 1.0);
        let theta = dot.acos();
        if !theta.is_finite() || theta <= 1e-8 {
            continue;
        }

        let half = theta * 0.5;
        let tan_half = half.tan();
        let sin_half = half.sin();
        if !tan_half.is_finite() || tan_half <= 0.0 || !sin_half.is_finite() || sin_half <= 0.0 {
            continue;
        }

        let t = (radius / tan_half).min(in_len * 0.999).min(out_len * 0.999);
        if !t.is_finite() || t <= tol.eps {
            continue;
        }

        corner_t[i] = t;
        corner_meta[i] = Some((to_prev, to_next, theta, tan_half, sin_half));
    }

    // Second pass: resolve overlaps on each segment (i -> i+1).
    let max_iter = (n * 4).max(8);
    for _ in 0..max_iter {
        let mut changed = false;

        for i in 0..n {
            let j = (i + 1) % n;
            let seg_len = points[j].sub_point(points[i]).length();
            if !seg_len.is_finite() || seg_len <= tol.eps {
                continue;
            }

            let sum = corner_t[i] + corner_t[j];
            if sum <= seg_len * 0.999 {
                continue;
            }

            // Scale both tangents down to fit on this segment.
            if sum > 0.0 {
                let scale = (seg_len * 0.999) / sum;
                let new_i = corner_t[i] * scale;
                let new_j = corner_t[j] * scale;
                if (new_i - corner_t[i]).abs() > 1e-12 || (new_j - corner_t[j]).abs() > 1e-12 {
                    corner_t[i] = new_i;
                    corner_t[j] = new_j;
                    changed = true;
                    diagnostics.clamped_corner_count += 1;
                }
            } else {
                corner_t[i] = 0.0;
                corner_t[j] = 0.0;
            }
        }

        if !changed {
            break;
        }
    }

    let mut out = Vec::with_capacity(points.len().saturating_mul(segments.max(2)));

    for i in 0..n {
        diagnostics.corner_count += 1;

        let _prev = points[(i + n - 1) % n];
        let corner = points[i];
        let _next = points[(i + 1) % n];

        let Some((to_prev, to_next, _theta, tan_half, sin_half)) = corner_meta[i] else {
            diagnostics.skipped_corner_count += 1;
            if out.is_empty() || !tol.approx_eq_point3(*out.last().unwrap(), corner) {
                out.push(corner);
            }
            continue;
        };

        let t = corner_t[i];
        if !t.is_finite() || t <= tol.eps {
            diagnostics.skipped_corner_count += 1;
            if out.is_empty() || !tol.approx_eq_point3(*out.last().unwrap(), corner) {
                out.push(corner);
            }
            continue;
        }

        let r_eff = t * tan_half;
        let center_dist = r_eff / sin_half;
        let Some(bisector) = to_prev.add(to_next).normalized() else {
            diagnostics.skipped_corner_count += 1;
            if out.is_empty() || !tol.approx_eq_point3(*out.last().unwrap(), corner) {
                out.push(corner);
            }
            continue;
        };

        let center = corner.add_vec(bisector.mul_scalar(center_dist));
        let p_in = corner.add_vec(to_prev.mul_scalar(t));
        let p_out = corner.add_vec(to_next.mul_scalar(t));

        let v0 = p_in.sub_point(center);
        let v1 = p_out.sub_point(center);
        let Some(axis) = v0.cross(v1).normalized() else {
            diagnostics.skipped_corner_count += 1;
            if out.is_empty() || !tol.approx_eq_point3(*out.last().unwrap(), corner) {
                out.push(corner);
            }
            continue;
        };

        let angle = signed_angle(v0, v1, axis);
        if !angle.is_finite() {
            diagnostics.skipped_corner_count += 1;
            if out.is_empty() || !tol.approx_eq_point3(*out.last().unwrap(), corner) {
                out.push(corner);
            }
            continue;
        }

        if out.is_empty() {
            out.push(p_in);
        } else if !tol.approx_eq_point3(*out.last().unwrap(), p_in) {
            out.push(p_in);
        }

        for s in 1..segments {
            let frac = s as f64 / segments as f64;
            let v = rotate_vec(v0, axis, angle * frac);
            out.push(center.add_vec(v));
        }

        out.push(p_out);
        diagnostics.filleted_corner_count += 1;
    }

    out
}
