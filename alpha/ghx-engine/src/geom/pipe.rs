//! Pipe operations (tube mesh generation) for mesh_engine_next.
//!
//! This module provides:
//! - `pipe_polyline*`: constant-radius pipe along a rail polyline
//! - `pipe_variable_polyline*`: variable-radius pipe using parameter/radius lists
//!
//! The implementation is intentionally minimal and entirely contained within `geom/`.
//! Components should call into this module later (Phase 3).

use super::diagnostics::GeomMeshDiagnostics;
use super::mesh::{GeomMesh, finalize_mesh};
use super::{FrenetFrame, Point3, Tolerance, Vec3};

/// Threshold for detecting sharp tangent changes (cusps) in the rail curve.
/// A dot product below this value (~75° angle) is considered a potential cusp.
const CUSP_DOT_THRESHOLD: f64 = 0.25;

/// Configuration for cap generation on pipe meshes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipeCaps {
    pub start: bool,
    pub end: bool,
}

impl PipeCaps {
    pub const NONE: Self = Self { start: false, end: false };
    pub const START: Self = Self { start: true, end: false };
    pub const END: Self = Self { start: false, end: true };
    pub const BOTH: Self = Self { start: true, end: true };
}

/// Options for controlling pipe mesh generation.
#[derive(Debug, Clone, Copy)]
pub struct PipeOptions {
    /// Number of segments around the circular cross-section.
    pub radial_segments: usize,
}

impl Default for PipeOptions {
    fn default() -> Self {
        Self { radial_segments: 24 }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PipeError {
    #[error("rail curve requires at least 2 points")]
    RailTooShort,
    #[error("rail curve must have finite points")]
    NonFiniteRail,
    #[error("radius must be finite and > 0")]
    InvalidRadius,
    #[error("parameters and radii lists must have the same length")]
    ParamRadiusLengthMismatch,
    #[error("pipe variable requires at least one radius")]
    EmptyRadii,
    #[error("parameters must be finite")]
    NonFiniteParameters,
    #[error("radii must be finite")]
    NonFiniteRadii,
    #[error("cannot cap a closed rail pipe")]
    CapsNotAllowedForClosedRail,
    #[error("rail curve is degenerate")]
    InvalidRail,
    #[error("pipe inputs must be finite")]
    NonFiniteInput,
    #[error("pipe requires at least 3 radial segments")]
    NotEnoughRadialSegments,
    #[error("rail has a degenerate segment")]
    DegenerateRailSegment,
    #[error("rail has a near-180° cusp; pipe cannot be generated robustly")]
    CuspNotSupported,
}

#[derive(Debug, Clone)]
struct CleanRail {
    points: Vec<Point3>,
    closed: bool,
}

fn clean_rail(points: &[Point3], tol: Tolerance) -> Result<CleanRail, PipeError> {
    if points.len() < 2 {
        return Err(PipeError::RailTooShort);
    }
    if points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite()) {
        return Err(PipeError::NonFiniteRail);
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
        return Err(PipeError::InvalidRail);
    }

    let closed = cleaned
        .first()
        .copied()
        .zip(cleaned.last().copied())
        .is_some_and(|(a, b)| tol.approx_eq_point3(a, b));

    if closed && cleaned.len() > 2 {
        cleaned.pop();
    }

    if cleaned.len() < 2 {
        return Err(PipeError::InvalidRail);
    }

    Ok(CleanRail { points: cleaned, closed })
}

#[must_use]
pub fn pipe_polyline(
    rail: &[Point3],
    radius: f64,
    caps: PipeCaps,
) -> Result<(GeomMesh, GeomMeshDiagnostics), PipeError> {
    pipe_polyline_with_tolerance(rail, radius, caps, PipeOptions::default(), Tolerance::default_geom())
}

pub fn pipe_polyline_with_tolerance(
    rail: &[Point3],
    radius: f64,
    caps: PipeCaps,
    options: PipeOptions,
    tol: Tolerance,
) -> Result<(GeomMesh, GeomMeshDiagnostics), PipeError> {
    if !radius.is_finite() || radius <= tol.eps {
        return Err(PipeError::InvalidRadius);
    }
    pipe_variable_polyline_with_tolerance(
        rail,
        &[],
        &[radius],
        caps,
        options,
        tol,
    )
}

#[must_use]
pub fn pipe_variable_polyline(
    rail: &[Point3],
    parameters: &[f64],
    radii: &[f64],
    caps: PipeCaps,
) -> Result<(GeomMesh, GeomMeshDiagnostics), PipeError> {
    pipe_variable_polyline_with_tolerance(rail, parameters, radii, caps, PipeOptions::default(), Tolerance::default_geom())
}

pub fn pipe_variable_polyline_with_tolerance(
    rail: &[Point3],
    parameters: &[f64],
    radii: &[f64],
    caps: PipeCaps,
    options: PipeOptions,
    tol: Tolerance,
) -> Result<(GeomMesh, GeomMeshDiagnostics), PipeError> {
    if options.radial_segments < 3 {
        return Err(PipeError::NotEnoughRadialSegments);
    }
    if parameters.iter().any(|v| !v.is_finite()) {
        return Err(PipeError::NonFiniteParameters);
    }
    if radii.is_empty() {
        return Err(PipeError::EmptyRadii);
    }
    if radii.iter().any(|v| !v.is_finite()) {
        return Err(PipeError::NonFiniteRadii);
    }
    if !parameters.is_empty() && parameters.len() != radii.len() {
        return Err(PipeError::ParamRadiusLengthMismatch);
    }

    let cleaned = clean_rail(rail, tol)?;
    if (caps.start || caps.end) && cleaned.closed {
        return Err(PipeError::CapsNotAllowedForClosedRail);
    }

    let ring_count = cleaned.points.len();

    // Compute radii per ring.
    let arc_lengths = compute_arc_lengths(&cleaned.points);
    let total_arc_length = arc_lengths.last().copied().unwrap_or(0.0).max(tol.eps);

    let mut radii_by_ring = if parameters.is_empty() {
        if radii.len() == 1 {
            vec![radii[0].abs(); ring_count]
        } else if radii.len() == ring_count {
            radii.iter().map(|r| r.abs()).collect()
        } else {
            // Fallback: use average radius if list doesn't match.
            let avg = radii.iter().map(|r| r.abs()).sum::<f64>() / radii.len() as f64;
            vec![avg; ring_count]
        }
    } else {
        let pairs = normalize_param_radius_pairs(parameters, radii, tol);
        (0..ring_count)
            .map(|i| {
                let t = (arc_lengths[i] / total_arc_length).clamp(0.0, 1.0);
                sample_radius(&pairs, t, tol).abs()
            })
            .collect()
    };

    if radii_by_ring.iter().any(|r| !r.is_finite() || *r <= tol.eps) {
        return Err(PipeError::InvalidRadius);
    }

    let mut warnings = Vec::new();

    // Self-intersection / degeneracy guards.
    warnings.extend(apply_radius_guards(
        &mut radii_by_ring,
        &cleaned.points,
        cleaned.closed,
        tol,
    )?);

    // Compute rotation-minimizing frames along the rail.
    let (frames, frame_warnings) = compute_rail_frames(&cleaned.points, tol);
    warnings.extend(frame_warnings);

    // Build vertices.
    let radial_segments = options.radial_segments;
    let mut vertices: Vec<Point3> = Vec::with_capacity(ring_count * radial_segments);
    let mut uvs: Vec<[f64; 2]> = Vec::with_capacity(ring_count * radial_segments);

    for ring_idx in 0..ring_count {
        let origin = cleaned.points[ring_idx];
        let frame = frames[ring_idx];
        let radius = radii_by_ring[ring_idx];

        let u_param = arc_lengths[ring_idx] / total_arc_length;

        for seg in 0..radial_segments {
            let v_param = seg as f64 / radial_segments as f64;
            let angle = 2.0 * std::f64::consts::PI * v_param;
            let local_x = radius * angle.cos();
            let local_y = radius * angle.sin();

            let world = origin
                .add_vec(frame.normal.mul_scalar(local_x))
                .add_vec(frame.binormal.mul_scalar(local_y));

            vertices.push(world);
            uvs.push([u_param, v_param]);
        }
    }

    // Build side indices.
    let rail_edge_count = if cleaned.closed { ring_count } else { ring_count - 1 };
    let mut indices: Vec<u32> = Vec::with_capacity(rail_edge_count * radial_segments * 6);

    for r in 0..rail_edge_count {
        let r_next = if cleaned.closed { (r + 1) % ring_count } else { r + 1 };
        for seg in 0..radial_segments {
            let seg_next = (seg + 1) % radial_segments;

            let i0 = (r * radial_segments + seg) as u32;
            let i1 = (r * radial_segments + seg_next) as u32;
            let i2 = (r_next * radial_segments + seg_next) as u32;
            let i3 = (r_next * radial_segments + seg) as u32;

            indices.extend_from_slice(&[i0, i1, i2]);
            indices.extend_from_slice(&[i0, i2, i3]);
        }
    }

    // Caps.
    if caps.start {
        add_cap_circle(
            &mut vertices,
            &mut uvs,
            &mut indices,
            cleaned.points[0],
            frames[0],
            radii_by_ring[0],
            true,
            radial_segments,
        );
    }
    if caps.end {
        let last = ring_count - 1;
        add_cap_circle(
            &mut vertices,
            &mut uvs,
            &mut indices,
            cleaned.points[last],
            frames[last],
            radii_by_ring[last],
            false,
            radial_segments,
        );
    }

    let (mesh, mut diagnostics) = finalize_mesh(vertices, Some(uvs), indices, tol);
    diagnostics.warnings.extend(warnings);
    Ok((mesh, diagnostics))
}

fn compute_arc_lengths(points: &[Point3]) -> Vec<f64> {
    let mut arc_lengths = Vec::with_capacity(points.len());
    let mut cumulative = 0.0;
    arc_lengths.push(cumulative);

    for i in 1..points.len() {
        let segment_length = points[i].sub_point(points[i - 1]).length();
        if segment_length.is_finite() {
            cumulative += segment_length;
        }
        arc_lengths.push(cumulative);
    }

    arc_lengths
}

fn compute_rail_frames(rail: &[Point3], tol: Tolerance) -> (Vec<FrenetFrame>, Vec<String>) {
    let mut warnings = Vec::new();

    if rail.len() < 2 {
        return (
            vec![FrenetFrame::from_tangent(Vec3::new(0.0, 0.0, 1.0)).unwrap()],
            vec!["rail too short; using default frame".to_string()],
        );
    }

    let mut frames = Vec::with_capacity(rail.len());

    let initial_tangent = rail[1].sub_point(rail[0]);
    let first_frame = match FrenetFrame::from_tangent(initial_tangent) {
        Some(f) => f,
        None => {
            warnings.push("rail has degenerate initial tangent; using default frame".to_string());
            FrenetFrame::from_tangent(Vec3::new(0.0, 0.0, 1.0)).unwrap()
        }
    };
    frames.push(first_frame);

    let mut cusp_like = 0usize;

    for i in 1..rail.len() {
        let prev_idx = i - 1;
        let next_idx = (i + 1).min(rail.len() - 1);

        let tangent = if i < rail.len() - 1 {
            let forward = rail[next_idx].sub_point(rail[i]);
            let backward = rail[i].sub_point(rail[prev_idx]);
            forward.add(backward)
        } else {
            rail[i].sub_point(rail[prev_idx])
        };

        let tangent = match tangent.normalized() {
            Some(t) => t,
            None => {
                warnings.push("rail has degenerate segment; reusing previous tangent".to_string());
                frames[prev_idx].tangent
            }
        };

        if frames[prev_idx].tangent.dot(tangent) < CUSP_DOT_THRESHOLD {
            cusp_like += 1;
        }

        let prev_frame = &frames[prev_idx];
        let new_frame = parallel_transport_frame(prev_frame, tangent, tol);
        frames.push(new_frame);
    }

    if cusp_like > 0 {
        warnings.push(format!(
            "rail continuity warning: {cusp_like} sharp tangent changes"
        ));
    }

    (frames, warnings)
}

fn parallel_transport_frame(prev_frame: &FrenetFrame, new_tangent: Vec3, tol: Tolerance) -> FrenetFrame {
    let old_tangent = prev_frame.tangent;

    let cross = old_tangent.cross(new_tangent);
    let cross_len_sq = cross.length_squared();

    if cross_len_sq < tol.eps_squared() {
        let dot = old_tangent.dot(new_tangent);
        if dot < 0.0 {
            FrenetFrame {
                tangent: new_tangent,
                normal: prev_frame.normal.mul_scalar(-1.0),
                binormal: prev_frame.binormal.mul_scalar(-1.0),
            }
        } else {
            FrenetFrame {
                tangent: new_tangent,
                normal: prev_frame.normal,
                binormal: prev_frame.binormal,
            }
        }
    } else {
        let rotation_axis = cross.normalized().unwrap_or(Vec3::new(0.0, 0.0, 1.0));
        let dot = old_tangent.dot(new_tangent).clamp(-1.0, 1.0);
        let angle = dot.acos();

        let new_normal = rotate_vector(prev_frame.normal, rotation_axis, angle)
            .normalized()
            .unwrap_or(prev_frame.normal);
        let new_binormal = new_tangent
            .cross(new_normal)
            .normalized()
            .unwrap_or(prev_frame.binormal);

        FrenetFrame {
            tangent: new_tangent,
            normal: new_normal,
            binormal: new_binormal,
        }
    }
}

fn rotate_vector(v: Vec3, axis: Vec3, angle: f64) -> Vec3 {
    let cos_angle = angle.cos();
    let sin_angle = angle.sin();

    let k_cross_v = axis.cross(v);
    let k_dot_v = axis.dot(v);

    v.mul_scalar(cos_angle)
        .add(k_cross_v.mul_scalar(sin_angle))
        .add(axis.mul_scalar(k_dot_v * (1.0 - cos_angle)))
}

fn add_cap_circle(
    vertices: &mut Vec<Point3>,
    uvs: &mut Vec<[f64; 2]>,
    indices: &mut Vec<u32>,
    origin: Point3,
    frame: FrenetFrame,
    radius: f64,
    is_start: bool,
    radial_segments: usize,
) {
    let offset = vertices.len() as u32;

    // Center vertex.
    vertices.push(origin);
    uvs.push([0.5, 0.5]);

    // Circle ring.
    for seg in 0..radial_segments {
        let t = seg as f64 / radial_segments as f64;
        let angle = 2.0 * std::f64::consts::PI * t;
        let local_x = radius * angle.cos();
        let local_y = radius * angle.sin();
        let world = origin
            .add_vec(frame.normal.mul_scalar(local_x))
            .add_vec(frame.binormal.mul_scalar(local_y));
        vertices.push(world);
        uvs.push([0.5 + 0.5 * angle.cos(), 0.5 + 0.5 * angle.sin()]);
    }

    // Triangle fan.
    let center = offset;
    let ring_start = offset + 1;
    for seg in 0..radial_segments {
        let next = (seg + 1) % radial_segments;
        let a = ring_start + seg as u32;
        let b = ring_start + next as u32;

        if is_start {
            // Outward normal at start points along -tangent.
            indices.extend_from_slice(&[center, b, a]);
        } else {
            // Outward normal at end points along +tangent.
            indices.extend_from_slice(&[center, a, b]);
        }
    }
}

fn normalize_param_radius_pairs(params: &[f64], radii: &[f64], tol: Tolerance) -> Vec<(f64, f64)> {
    let mut pairs: Vec<(f64, f64)> = params
        .iter()
        .copied()
        .zip(radii.iter().copied())
        .collect();

    pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    // Merge nearly-equal parameters (keep last radius).
    let mut merged: Vec<(f64, f64)> = Vec::with_capacity(pairs.len());
    for (p, r) in pairs {
        if let Some((last_p, last_r)) = merged.last_mut() {
            if (p - *last_p).abs() <= tol.eps {
                *last_r = r;
                continue;
            }
        }
        merged.push((p, r));
    }

    merged
}

fn sample_radius(pairs: &[(f64, f64)], t: f64, tol: Tolerance) -> f64 {
    if pairs.is_empty() {
        return 0.0;
    }
    if pairs.len() == 1 {
        return pairs[0].1;
    }

    // Clamp to ends.
    if t <= pairs[0].0 + tol.eps {
        return pairs[0].1;
    }
    if t >= pairs[pairs.len() - 1].0 - tol.eps {
        return pairs[pairs.len() - 1].1;
    }

    // Find segment.
    let mut lo = 0usize;
    let mut hi = pairs.len() - 1;
    while lo + 1 < hi {
        let mid = (lo + hi) / 2;
        if t < pairs[mid].0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }

    let (t0, r0) = pairs[lo];
    let (t1, r1) = pairs[hi];
    let denom = (t1 - t0).max(tol.eps);
    let u = ((t - t0) / denom).clamp(0.0, 1.0);
    r0 + (r1 - r0) * u
}

fn apply_radius_guards(
    radii_by_ring: &mut [f64],
    rail: &[Point3],
    closed: bool,
    tol: Tolerance,
) -> Result<Vec<String>, PipeError> {
    let mut warnings = Vec::new();
    let ring_count = rail.len();

    // Guard based on segment lengths (very short segments with large radii lead to degeneracy).
    let edge_count = if closed { ring_count } else { ring_count - 1 };
    for i in 0..edge_count {
        let next = if closed { (i + 1) % ring_count } else { i + 1 };
        let seg_len = rail[next].sub_point(rail[i]).length();
        if !seg_len.is_finite() || seg_len <= tol.eps {
            return Err(PipeError::DegenerateRailSegment);
        }

        let cap = 0.49 * seg_len;
        let ri = radii_by_ring[i];
        let rj = radii_by_ring[next];
        if ri > cap || rj > cap {
            radii_by_ring[i] = ri.min(cap);
            radii_by_ring[next] = rj.min(cap);
            warnings.push(format!(
                "radius clamped near short segment (len={seg_len:.6})"
            ));
        }
    }

    // Guard against near-180° cusps (tube cannot be robustly defined).
    let start = if closed { 0 } else { 1 };
    let end = if closed { ring_count } else { ring_count - 1 };

    for i in start..end {
        let prev = if i == 0 { ring_count - 1 } else { i - 1 };
        let next = if i + 1 == ring_count { 0 } else { i + 1 };

        let a = rail[i].sub_point(rail[prev]);
        let b = rail[next].sub_point(rail[i]);
        let (Some(da), Some(db)) = (a.normalized(), b.normalized()) else {
            continue;
        };

        let dot = da.dot(db).clamp(-1.0, 1.0);
        if dot < -0.999 {
            return Err(PipeError::CuspNotSupported);
        }

        // For sharp turns, warn if radius is comparable to segment lengths.
        let angle = dot.acos();
        if angle > 1.2 {
            let len_prev = a.length();
            let len_next = b.length();
            let max_allowed = 0.49 * len_prev.min(len_next);
            if radii_by_ring[i] > max_allowed {
                radii_by_ring[i] = radii_by_ring[i].min(max_allowed);
                warnings.push("radius clamped near sharp junction".to_string());
            }
        }
    }

    Ok(warnings)
}
