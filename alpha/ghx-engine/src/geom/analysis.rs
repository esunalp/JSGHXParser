//! Geometry analysis helpers used by component adapters.
//!
//! This module centralizes lightweight analysis utilities (surface frames, legacy
//! B-rep-like edge extraction) so Grasshopper-style components can remain thin.

use super::solid::LegacySurfaceMesh;
use super::surface::Surface;
use super::{Point3, Tolerance, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct SurfaceFrame {
    pub origin: Point3,
    pub x_axis: Vec3,
    pub y_axis: Vec3,
    pub z_axis: Vec3,
}

#[derive(Debug, Clone)]
pub struct SurfaceFramesResult {
    pub frames: Vec<SurfaceFrame>,
    pub parameters: Vec<(f64, f64)>,
    pub u_count: usize,
    pub v_count: usize,
}

#[must_use]
pub fn surface_frames(
    surface: &impl Surface,
    u_segments: usize,
    v_segments: usize,
    tol: Tolerance,
) -> SurfaceFramesResult {
    let u_segments = u_segments.max(1);
    let v_segments = v_segments.max(1);

    let (u0, u1) = surface.domain_u();
    let (v0, v1) = surface.domain_v();
    let u_span = u1 - u0;
    let v_span = v1 - v0;

    let u_count = u_segments + 1;
    let v_count = v_segments + 1;

    let u_denom = u_segments as f64;
    let v_denom = v_segments as f64;

    let mut frames = Vec::with_capacity(u_count * v_count);
    let mut parameters = Vec::with_capacity(u_count * v_count);

    for v in 0..v_count {
        let fv = v as f64 / v_denom;
        let v_param = if v_span.is_finite() && v_span != 0.0 {
            v0 + v_span * fv
        } else {
            v0
        };

        for u in 0..u_count {
            let fu = u as f64 / u_denom;
            let u_param = if u_span.is_finite() && u_span != 0.0 {
                u0 + u_span * fu
            } else {
                u0
            };

            let origin = surface.point_at(u_param, v_param);
            let (du, dv) = surface.partial_derivatives_at(u_param, v_param);

            let (x_axis, y_axis, z_axis) = surface_frame_axes(du, dv, tol);
            frames.push(SurfaceFrame {
                origin,
                x_axis,
                y_axis,
                z_axis,
            });
            parameters.push((u_param, v_param));
        }
    }

    SurfaceFramesResult {
        frames,
        parameters,
        u_count,
        v_count,
    }
}

fn orthogonal_unit_vector(reference: Vec3) -> Vec3 {
    let candidate = if reference.x.abs() < reference.y.abs() {
        Vec3::new(0.0, -reference.z, reference.y)
    } else {
        Vec3::new(-reference.z, 0.0, reference.x)
    };

    candidate
        .normalized()
        .unwrap_or_else(|| Vec3::new(1.0, 0.0, 0.0))
}

fn surface_frame_axes(du: Vec3, dv: Vec3, tol: Tolerance) -> (Vec3, Vec3, Vec3) {
    let normal = du.cross(dv).normalized().unwrap_or_else(|| Vec3::new(0.0, 0.0, 1.0));

    let x_hint = if du.length() > tol.eps { du } else { dv };
    let projected = x_hint.sub(normal.mul_scalar(x_hint.dot(normal)));
    let x_axis = projected
        .normalized()
        .unwrap_or_else(|| orthogonal_unit_vector(normal));

    let y_axis = normal
        .cross(x_axis)
        .normalized()
        .unwrap_or_else(|| Vec3::new(0.0, 1.0, 0.0));

    let z_axis = x_axis
        .cross(y_axis)
        .normalized()
        .unwrap_or(normal);

    (x_axis, y_axis, z_axis)
}

#[derive(Debug, Clone)]
pub struct LegacyBrepFace {
    pub vertices: Vec<Point3>,
}

impl LegacyBrepFace {
    #[must_use]
    pub fn centroid(&self) -> Point3 {
        if self.vertices.is_empty() {
            return Point3::new(0.0, 0.0, 0.0);
        }

        let mut sum = Vec3::new(0.0, 0.0, 0.0);
        for vertex in &self.vertices {
            sum = sum.add(Vec3::new(vertex.x, vertex.y, vertex.z));
        }
        let scale = 1.0 / self.vertices.len() as f64;
        Point3::new(sum.x * scale, sum.y * scale, sum.z * scale)
    }
}

#[derive(Debug, Clone)]
pub struct LegacyBrepEdge {
    pub start: Point3,
    pub end: Point3,
    pub faces: Vec<usize>,
}

impl LegacyBrepEdge {
    #[must_use]
    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    fn add_face(&mut self, face: usize) {
        if !self.faces.contains(&face) {
            self.faces.push(face);
        }
    }

    #[must_use]
    pub fn vector(&self) -> Vec3 {
        self.end.sub_point(self.start)
    }

    #[must_use]
    pub fn length(&self) -> f64 {
        self.vector().length()
    }

    #[must_use]
    pub fn matches(&self, start: Point3, end: Point3, tol: Tolerance) -> bool {
        (tol.approx_eq_point3(self.start, start) && tol.approx_eq_point3(self.end, end))
            || (tol.approx_eq_point3(self.start, end) && tol.approx_eq_point3(self.end, start))
    }

    #[must_use]
    pub fn touches_point(&self, point: Point3, tolerance: f64) -> bool {
        if !tolerance.is_finite() || tolerance < 0.0 {
            return false;
        }

        self.start.sub_point(point).length() <= tolerance
            || self.end.sub_point(point).length() <= tolerance
    }
}

#[derive(Debug, Default, Clone)]
pub struct LegacyBrepData {
    pub faces: Vec<LegacyBrepFace>,
    pub edges: Vec<LegacyBrepEdge>,
}

impl LegacyBrepData {
    pub fn add_edge(&mut self, start: Point3, end: Point3, face: Option<usize>, tol: Tolerance) {
        if tol.approx_eq_point3(start, end) {
            return;
        }

        if let Some(existing) = self
            .edges
            .iter_mut()
            .find(|edge| edge.matches(start, end, tol))
        {
            if let Some(face_index) = face {
                existing.add_face(face_index);
            }
            return;
        }

        let mut edge = LegacyBrepEdge {
            start,
            end,
            faces: Vec::new(),
        };
        if let Some(face_index) = face {
            edge.add_face(face_index);
        }
        self.edges.push(edge);
    }

    pub fn extend_from_surface_buffers(
        &mut self,
        vertices: &[[f64; 3]],
        faces: &[Vec<u32>],
        tol: Tolerance,
    ) {
        for face in faces {
            let mut face_vertices = Vec::new();
            for &index in face {
                let Some(vertex) = vertices.get(index as usize) else { continue };
                face_vertices.push(Point3::new(vertex[0], vertex[1], vertex[2]));
            }
            if face_vertices.len() < 2 {
                continue;
            }

            let face_index = self.faces.len();
            self.faces.push(LegacyBrepFace {
                vertices: face_vertices.clone(),
            });

            for segment in 0..face_vertices.len() {
                let start = face_vertices[segment];
                let end = face_vertices[(segment + 1) % face_vertices.len()];
                self.add_edge(start, end, Some(face_index), tol);
            }
        }
    }

    pub fn extend_from_surface_mesh(&mut self, surface: &LegacySurfaceMesh, tol: Tolerance) {
        self.extend_from_surface_buffers(&surface.vertices, &surface.faces, tol);
    }

    pub fn extend_from_line(&mut self, p1: Point3, p2: Point3, tol: Tolerance) {
        self.add_edge(p1, p2, None, tol);
    }

    #[must_use]
    pub fn from_bounds(min: Point3, max: Point3) -> Self {
        Self {
            faces: Vec::new(),
            edges: box_edges(min, max),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClosedEdgesResult {
    pub closed: Vec<usize>,
    pub open: Vec<usize>,
}

#[must_use]
pub fn closed_edges(data: &LegacyBrepData) -> ClosedEdgesResult {
    let mut closed = Vec::new();
    let mut open = Vec::new();

    for (idx, edge) in data.edges.iter().enumerate() {
        if edge.face_count() >= 2 {
            closed.push(idx);
        } else {
            open.push(idx);
        }
    }

    ClosedEdgesResult { closed, open }
}

#[derive(Debug, Clone)]
pub struct EdgesFromDirectionsResult {
    pub edges: Vec<usize>,
    pub map: Vec<usize>,
}

#[must_use]
pub fn edges_from_directions(
    data: &LegacyBrepData,
    directions: &[Vec3],
    reflex: bool,
    tolerance_radians: f64,
    tol: Tolerance,
) -> EdgesFromDirectionsResult {
    let mut selected = Vec::new();
    let mut mapping = Vec::new();

    let tolerance_radians = tolerance_radians.abs();

    for (index, edge) in data.edges.iter().enumerate() {
        let vec = edge.vector();
        let len = vec.length();
        if !len.is_finite() || len <= tol.eps {
            continue;
        }
        let direction = vec.mul_scalar(1.0 / len);

        let mut matched = None;
        for (dir_index, candidate) in directions.iter().enumerate() {
            let candidate = candidate
                .normalized()
                .unwrap_or_else(|| Vec3::new(1.0, 0.0, 0.0));
            let dot = clamp(direction.dot(candidate), -1.0, 1.0);
            let angle = dot.acos();
            if angle <= tolerance_radians
                || (reflex && (core::f64::consts::PI - angle) <= tolerance_radians)
            {
                matched = Some(dir_index);
                break;
            }
        }

        if let Some(dir_index) = matched {
            selected.push(index);
            mapping.push(dir_index);
        }
    }

    EdgesFromDirectionsResult {
        edges: selected,
        map: mapping,
    }
}

#[must_use]
pub fn edges_from_faces(data: &LegacyBrepData, points: &[Point3], tolerance: f64) -> Vec<usize> {
    if data.faces.is_empty() {
        return data
            .edges
            .iter()
            .enumerate()
            .filter_map(|(idx, edge)| {
                let include = points.is_empty()
                    || points
                        .iter()
                        .any(|point| edge.touches_point(*point, tolerance));
                include.then_some(idx)
            })
            .collect();
    }

    let mut selected_faces = Vec::new();
    if points.is_empty() {
        selected_faces.extend(0..data.faces.len());
    } else {
        for (face_idx, face) in data.faces.iter().enumerate() {
            let centroid = face.centroid();
            if points
                .iter()
                .any(|point| centroid.sub_point(*point).length() <= tolerance)
            {
                selected_faces.push(face_idx);
            }
        }
    }

    data.edges
        .iter()
        .enumerate()
        .filter_map(|(idx, edge)| {
            edge.faces
                .iter()
                .any(|face_idx| selected_faces.contains(face_idx))
                .then_some(idx)
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct EdgesFromPointsResult {
    pub edges: Vec<usize>,
    pub map: Vec<usize>,
}

#[must_use]
pub fn edges_from_points(
    data: &LegacyBrepData,
    points: &[Point3],
    valence: usize,
    tolerance: f64,
) -> EdgesFromPointsResult {
    let valence = valence.max(1);

    let mut selected = Vec::new();
    let mut mapping = vec![0usize; points.len()];

    for (index, edge) in data.edges.iter().enumerate() {
        let mut matched_points = Vec::new();
        for (point_index, point) in points.iter().enumerate() {
            if edge.touches_point(*point, tolerance) {
                matched_points.push(point_index);
            }
        }

        if matched_points.len() >= valence {
            selected.push(index);
            for point_index in matched_points {
                mapping[point_index] += 1;
            }
        }
    }

    EdgesFromPointsResult {
        edges: selected,
        map: mapping,
    }
}

#[must_use]
pub fn edges_by_length(data: &LegacyBrepData, min_length: f64, max_length: f64) -> Vec<usize> {
    let min_length = min_length.abs();
    let max_length = max_length.abs();
    let max_length = max_length.max(min_length);

    data.edges
        .iter()
        .enumerate()
        .filter_map(|(idx, edge)| {
            let length = edge.length();
            (length >= min_length && length <= max_length).then_some(idx)
        })
        .collect()
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

fn box_edges(min: Point3, max: Point3) -> Vec<LegacyBrepEdge> {
    let corners = box_corners(min, max);
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
        .map(|(a, b)| LegacyBrepEdge {
            start: corners[*a],
            end: corners[*b],
            faces: Vec::new(),
        })
        .collect()
}

fn box_corners(min: Point3, max: Point3) -> Vec<Point3> {
    let mut corners = Vec::with_capacity(8);
    for &z in &[min.z, max.z] {
        for &y in &[min.y, max.y] {
            for &x in &[min.x, max.x] {
                corners.push(Point3::new(x, y, z));
            }
        }
    }
    corners
}
