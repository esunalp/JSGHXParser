//! Sweep operations for mesh generation.
//!
//! This module provides two sweep variants:
//!
//! - **`sweep1_polyline`**: Single-rail sweep. The profile is swept along a single
//!   rail curve using rotation-minimizing frames (parallel transport) to avoid
//!   unwanted twisting. Use this when you have a profile shape and a path to follow.
//!
//! - **`sweep2_polyline`**: Two-rail sweep. The profile is swept along rail A while
//!   using rail B to define the orientation (normal direction). Rails must have the
//!   same number of points and correspond point-to-point. Use this when you need
//!   precise control over how the profile orients along the path.
//!
//! Both functions support optional end caps (for closed profiles on open rails) and
//! a twist parameter to rotate the profile along the sweep path.
//!
//! # Profile Coordinate Conventions
//!
//! Profiles are assumed to be defined in a local coordinate system centered at the
//! origin. The profile's X and Y coordinates map to the frame's normal and binormal
//! directions respectively, while Z maps along the tangent. If your profile is not
//! centered at the origin, consider translating it before sweeping for predictable
//! results.

use super::diagnostics::GeomMeshDiagnostics;
use super::mesh::{GeomMesh, finalize_mesh};
use super::triangulation::triangulate_trim_region;
use super::trim::{TrimLoop, TrimRegion, UvPoint};
use super::{FrenetFrame, Point3, Tolerance, Vec3};

/// Threshold for detecting sharp tangent changes (cusps) in the rail curve.
/// A dot product below this value (~75° angle) is considered a potential cusp.
const CUSP_DOT_THRESHOLD: f64 = 0.25;

/// Configuration for cap generation on swept surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SweepCaps {
    pub start: bool,
    pub end: bool,
}

impl SweepCaps {
    pub const NONE: Self = Self { start: false, end: false };
    pub const START: Self = Self { start: true, end: false };
    pub const END: Self = Self { start: false, end: true };
    pub const BOTH: Self = Self { start: true, end: true };
}

/// Options for controlling sweep mesh generation.
#[derive(Debug, Clone, Copy)]
pub struct SweepOptions {
    /// Total twist applied from start to end, in radians.
    pub twist_radians_total: f64,
}

impl Default for SweepOptions {
    fn default() -> Self {
        Self {
            twist_radians_total: 0.0,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SweepError {
    #[error("rail curve requires at least 2 points")]
    RailTooShort,
    #[error("rail curve must have finite points")]
    NonFiniteRail,
    #[error("profile points must be finite")]
    NonFiniteProfile,
    #[error("profile requires at least {min} unique points")]
    NotEnoughProfilePoints { min: usize },
    #[error("caps require a closed profile")]
    CapsRequireClosedProfile,
    #[error("cannot cap a closed rail sweep")]
    CapsNotAllowedForClosedRail,
    #[error("rail curve is degenerate")]
    InvalidRail,
    #[error("sweep inputs must be finite")]
    NonFiniteInput,
    #[error("sweep2 requires rails of the same length")]
    Sweep2RailLengthMismatch,
    #[error("failed to triangulate cap: {0}")]
    CapTriangulation(String),
}

#[derive(Debug, Clone)]
struct CleanPolyline {
    points: Vec<Point3>,
    closed: bool,
}

fn clean_polyline(points: &[Point3], tol: Tolerance) -> Result<CleanPolyline, SweepError> {
    if points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite()) {
        return Err(SweepError::NonFiniteProfile);
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
        return Err(SweepError::NotEnoughProfilePoints { min: 2 });
    }

    let closed = cleaned
        .first()
        .copied()
        .zip(cleaned.last().copied())
        .is_some_and(|(a, b)| tol.approx_eq_point3(a, b));

    if closed && cleaned.len() > 2 {
        cleaned.pop();
    }

    Ok(CleanPolyline { points: cleaned, closed })
}

fn clean_rail(points: &[Point3], tol: Tolerance) -> Result<CleanPolyline, SweepError> {
    if points.len() < 2 {
        return Err(SweepError::RailTooShort);
    }
    if points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite()) {
        return Err(SweepError::NonFiniteRail);
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
        return Err(SweepError::InvalidRail);
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
        return Err(SweepError::InvalidRail);
    }

    Ok(CleanPolyline { points: cleaned, closed })
}

#[must_use]
pub fn sweep1_polyline(
    profile: &[Point3],
    rail: &[Point3],
    caps: SweepCaps,
) -> Result<(GeomMesh, GeomMeshDiagnostics), SweepError> {
    sweep1_polyline_with_tolerance(profile, rail, caps, SweepOptions::default(), Tolerance::default_geom())
}

pub fn sweep1_polyline_with_tolerance(
    profile: &[Point3],
    rail: &[Point3],
    caps: SweepCaps,
    options: SweepOptions,
    tol: Tolerance,
) -> Result<(GeomMesh, GeomMeshDiagnostics), SweepError> {
    if !options.twist_radians_total.is_finite() {
        return Err(SweepError::NonFiniteInput);
    }

    let cleaned_profile = clean_polyline(profile, tol)?;
    let cleaned_rail = clean_rail(rail, tol)?;

    if (caps.start || caps.end) && cleaned_rail.closed {
        return Err(SweepError::CapsNotAllowedForClosedRail);
    }
    if (caps.start || caps.end) && !cleaned_profile.closed {
        return Err(SweepError::CapsRequireClosedProfile);
    }

    // Compute rotation-minimizing frames along the rail.
    let (frames, rail_warnings) = compute_rail_frames(&cleaned_rail.points, tol);

    // Apply optional twist.
    let frames = if options.twist_radians_total.abs() > tol.eps {
        apply_twist(&frames, &cleaned_rail.points, options.twist_radians_total, tol)
    } else {
        frames
    };

    let profile_len = cleaned_profile.points.len();
    if profile_len < 2 {
        return Err(SweepError::NotEnoughProfilePoints { min: 2 });
    }

    let ring_count = cleaned_rail.points.len();

    let mut vertices: Vec<Point3> = Vec::with_capacity(ring_count * profile_len);
    let mut uvs: Vec<[f64; 2]> = Vec::with_capacity(ring_count * profile_len);

    let arc_lengths = compute_arc_lengths(&cleaned_rail.points);
    let total_arc_length = arc_lengths.last().copied().unwrap_or(0.0).max(tol.eps);

    for (ring_idx, (rail_point, frame)) in cleaned_rail
        .points
        .iter()
        .zip(frames.iter())
        .enumerate()
    {
        let u_param = arc_lengths[ring_idx] / total_arc_length;
        for (i, &p) in cleaned_profile.points.iter().enumerate() {
            // Profile is assumed centered at origin: X -> normal, Y -> binormal, Z -> tangent.
            // See module docs for coordinate conventions.
            let local = p.sub_point(Point3::new(0.0, 0.0, 0.0));
            let world_point = rail_point
                .add_vec(frame.normal.mul_scalar(local.x))
                .add_vec(frame.binormal.mul_scalar(local.y))
                .add_vec(frame.tangent.mul_scalar(local.z));
            vertices.push(world_point);
            let v_param = if profile_len > 1 {
                i as f64 / (profile_len - 1) as f64
            } else {
                0.0
            };
            uvs.push([u_param, v_param]);
        }
    }

    // Build side indices.
    let rail_edge_count = if cleaned_rail.closed { ring_count } else { ring_count - 1 };
    let profile_edge_count = if cleaned_profile.closed { profile_len } else { profile_len - 1 };

    let mut indices: Vec<u32> = Vec::with_capacity(rail_edge_count * profile_edge_count * 6);

    for r in 0..rail_edge_count {
        let r_next = if cleaned_rail.closed { (r + 1) % ring_count } else { r + 1 };

        for i in 0..profile_edge_count {
            let i_next = (i + 1) % profile_len;

            let i0 = (r * profile_len + i) as u32;
            let i1 = (r * profile_len + i_next) as u32;
            let i2 = (r_next * profile_len + i_next) as u32;
            let i3 = (r_next * profile_len + i) as u32;

            indices.extend_from_slice(&[i0, i1, i2]);
            indices.extend_from_slice(&[i0, i2, i3]);
        }
    }

    // Caps (only meaningful for open rails).
    if caps.start {
        add_cap(
            &mut vertices,
            &mut uvs,
            &mut indices,
            &cleaned_profile.points,
            &frames[0],
            cleaned_rail.points[0],
            true,
            tol,
        )?;
    }

    if caps.end {
        let last = ring_count - 1;
        add_cap(
            &mut vertices,
            &mut uvs,
            &mut indices,
            &cleaned_profile.points,
            &frames[last],
            cleaned_rail.points[last],
            false,
            tol,
        )?;
    }

    let (mesh, mut diagnostics) = finalize_mesh(vertices, Some(uvs), indices, tol);
    diagnostics.warnings.extend(rail_warnings);
    Ok((mesh, diagnostics))
}

/// Sweep2: like Sweep1 but uses a second rail to anchor the frame's normal direction.
///
/// This is intentionally minimal: rails must have the same number of points and are assumed
/// to correspond point-to-point.
#[must_use]
pub fn sweep2_polyline(
    profile: &[Point3],
    rail_a: &[Point3],
    rail_b: &[Point3],
    caps: SweepCaps,
) -> Result<(GeomMesh, GeomMeshDiagnostics), SweepError> {
    sweep2_polyline_with_tolerance(
        profile,
        rail_a,
        rail_b,
        caps,
        SweepOptions::default(),
        Tolerance::default_geom(),
    )
}

pub fn sweep2_polyline_with_tolerance(
    profile: &[Point3],
    rail_a: &[Point3],
    rail_b: &[Point3],
    caps: SweepCaps,
    options: SweepOptions,
    tol: Tolerance,
) -> Result<(GeomMesh, GeomMeshDiagnostics), SweepError> {
    if rail_a.len() != rail_b.len() {
        return Err(SweepError::Sweep2RailLengthMismatch);
    }
    if rail_a.len() < 2 {
        return Err(SweepError::RailTooShort);
    }
    if rail_a.iter().any(|p| !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite())
        || rail_b.iter().any(|p| !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite())
    {
        return Err(SweepError::NonFiniteRail);
    }

    if !options.twist_radians_total.is_finite() {
        return Err(SweepError::NonFiniteInput);
    }

    let cleaned_profile = clean_polyline(profile, tol)?;

    // Determine if rail is closed.
    let closed = rail_a
        .first()
        .copied()
        .zip(rail_a.last().copied())
        .is_some_and(|(a, b)| tol.approx_eq_point3(a, b));

    if (caps.start || caps.end) && closed {
        return Err(SweepError::CapsNotAllowedForClosedRail);
    }
    if (caps.start || caps.end) && !cleaned_profile.closed {
        return Err(SweepError::CapsRequireClosedProfile);
    }

    let rail_edge_count = rail_a.len() - 1;
    let ring_count = if closed { rail_a.len() - 1 } else { rail_a.len() };

    // Build frames using tangent from rail A and normal from (rail_b - rail_a).
    let mut frames: Vec<FrenetFrame> = Vec::with_capacity(ring_count);
    let mut warnings = Vec::new();

    for i in 0..ring_count {
        let prev = if i == 0 { 0 } else { i - 1 };
        let next = (i + 1).min(ring_count - 1);
        let tangent = if i < ring_count - 1 {
            rail_a[next].sub_point(rail_a[i]).add(rail_a[i].sub_point(rail_a[prev]))
        } else {
            rail_a[i].sub_point(rail_a[prev])
        };
        let tangent = match tangent.normalized() {
            Some(t) => t,
            None => {
                warnings.push("sweep2 rail has degenerate tangent".to_string());
                Vec3::new(0.0, 0.0, 1.0)
            }
        };

        let offset = rail_b[i].sub_point(rail_a[i]);
        let mut normal = offset.sub(tangent.mul_scalar(offset.dot(tangent)));
        normal = normal.normalized().unwrap_or_else(|| {
            warnings.push("sweep2 rails are locally parallel; using arbitrary normal".to_string());
            FrenetFrame::from_tangent(tangent)
                .map(|f| f.normal)
                .unwrap_or(Vec3::new(1.0, 0.0, 0.0))
        });
        let binormal = tangent.cross(normal).normalized().unwrap_or(Vec3::new(0.0, 1.0, 0.0));

        frames.push(FrenetFrame { tangent, normal, binormal });
    }

    if options.twist_radians_total.abs() > tol.eps {
        frames = apply_twist(&frames, &rail_a[..ring_count], options.twist_radians_total, tol);
    }

    // Build vertices.
    let profile_len = cleaned_profile.points.len();
    let arc_lengths = compute_arc_lengths(&rail_a[..ring_count]);
    let total_arc_length = arc_lengths.last().copied().unwrap_or(0.0).max(tol.eps);

    let mut vertices: Vec<Point3> = Vec::with_capacity(ring_count * profile_len);
    let mut uvs: Vec<[f64; 2]> = Vec::with_capacity(ring_count * profile_len);

    for (ring_idx, frame) in frames.iter().enumerate() {
        let rail_point = rail_a[ring_idx];
        let u_param = arc_lengths[ring_idx] / total_arc_length;
        for (j, &p) in cleaned_profile.points.iter().enumerate() {
            let local = p.sub_point(Point3::new(0.0, 0.0, 0.0));
            let world_point = rail_point
                .add_vec(frame.normal.mul_scalar(local.x))
                .add_vec(frame.binormal.mul_scalar(local.y))
                .add_vec(frame.tangent.mul_scalar(local.z));
            vertices.push(world_point);
            let v_param = if profile_len > 1 {
                j as f64 / (profile_len - 1) as f64
            } else {
                0.0
            };
            uvs.push([u_param, v_param]);
        }
    }

    // Build side indices.
    let profile_edge_count = if cleaned_profile.closed { profile_len } else { profile_len - 1 };
    let mut indices: Vec<u32> = Vec::with_capacity(rail_edge_count * profile_edge_count * 6);

    for r in 0..rail_edge_count {
        let r_next = if closed { (r + 1) % ring_count } else { r + 1 };
        for i in 0..profile_edge_count {
            let i_next = (i + 1) % profile_len;

            let i0 = (r * profile_len + i) as u32;
            let i1 = (r * profile_len + i_next) as u32;
            let i2 = (r_next * profile_len + i_next) as u32;
            let i3 = (r_next * profile_len + i) as u32;

            indices.extend_from_slice(&[i0, i1, i2]);
            indices.extend_from_slice(&[i0, i2, i3]);
        }
    }

    if caps.start {
        add_cap(
            &mut vertices,
            &mut uvs,
            &mut indices,
            &cleaned_profile.points,
            &frames[0],
            rail_a[0],
            true,
            tol,
        )?;
    }
    if caps.end {
        let last = ring_count - 1;
        add_cap(
            &mut vertices,
            &mut uvs,
            &mut indices,
            &cleaned_profile.points,
            &frames[last],
            rail_a[last],
            false,
            tol,
        )?;
    }

    let (mesh, mut diagnostics) = finalize_mesh(vertices, Some(uvs), indices, tol);
    diagnostics.warnings.extend(warnings);
    Ok((mesh, diagnostics))
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

        let new_normal = rotate_vector(prev_frame.normal, rotation_axis, angle).normalized().unwrap_or(prev_frame.normal);
        let new_binormal = new_tangent.cross(new_normal).normalized().unwrap_or(prev_frame.binormal);

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

fn apply_twist(frames: &[FrenetFrame], rail: &[Point3], twist_total: f64, tol: Tolerance) -> Vec<FrenetFrame> {
    let arc_lengths = compute_arc_lengths(rail);
    let total = arc_lengths.last().copied().unwrap_or(0.0);
    let denom = if total > tol.eps { total } else { 1.0 };

    frames
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let t = arc_lengths[i] / denom;
            let angle = twist_total * t;
            let normal = rotate_vector(f.normal, f.tangent, angle);
            let binormal = f.tangent.cross(normal);
            FrenetFrame {
                tangent: f.tangent,
                normal: normal.normalized().unwrap_or(f.normal),
                binormal: binormal.normalized().unwrap_or(f.binormal),
            }
        })
        .collect()
}

fn add_cap(
    vertices: &mut Vec<Point3>,
    uvs: &mut Vec<[f64; 2]>,
    indices: &mut Vec<u32>,
    profile_local: &[Point3],
    frame: &FrenetFrame,
    origin: Point3,
    is_start: bool,
    tol: Tolerance,
) -> Result<(), SweepError> {
    // Build cap loop in UV space using the frame's (normal, binormal) axes.
    // Note: frame.normal × frame.binormal == frame.tangent.
    let mut loop_uv: Vec<UvPoint> = Vec::with_capacity(profile_local.len());
    for &p in profile_local {
        // cap points are at origin + normal*x + binormal*y (z ignored for cap)
        let u = p.x;
        let v = p.y;
        if !u.is_finite() || !v.is_finite() {
            return Err(SweepError::NonFiniteProfile);
        }
        loop_uv.push(UvPoint { u, v });
    }

    // Ensure loop is oriented so that triangles face cap_normal.
    // Our local cap plane normal is +Z. In world, +Z maps to frame.tangent (since normal×binormal == tangent).
    // So start cap needs reversed orientation.
    if is_start {
        loop_uv.reverse();
    }

    let outer = TrimLoop::new(loop_uv, tol).map_err(|e| SweepError::CapTriangulation(e.to_string()))?;
    let region = TrimRegion { outer, holes: Vec::new() };

    let tri = triangulate_trim_region(&region, tol).map_err(SweepError::CapTriangulation)?;

    let offset = vertices.len() as u32;

    // Add vertices by mapping the triangulation UVs into world cap plane.
    // Compute UV bounds for normalization to [0,1] range.
    let (u_min, u_max, v_min, v_max) = tri.vertices.iter().fold(
        (f64::MAX, f64::MIN, f64::MAX, f64::MIN),
        |(u_min, u_max, v_min, v_max), uv| {
            (u_min.min(uv.u), u_max.max(uv.u), v_min.min(uv.v), v_max.max(uv.v))
        },
    );
    let u_range = (u_max - u_min).max(tol.eps);
    let v_range = (v_max - v_min).max(tol.eps);

    for uv in &tri.vertices {
        let world = origin
            .add_vec(frame.normal.mul_scalar(uv.u))
            .add_vec(frame.binormal.mul_scalar(uv.v));
        vertices.push(world);
        // Map triangulation UVs to normalized [0,1] range for proper texturing.
        let cap_u = (uv.u - u_min) / u_range;
        let cap_v = (uv.v - v_min) / v_range;
        uvs.push([cap_u, cap_v]);
    }

    // Triangulation winding follows loop orientation; no extra swapping needed.
    for chunk in tri.indices.chunks_exact(3) {
        let a = offset + chunk[0];
        let b = offset + chunk[1];
        let c = offset + chunk[2];
        indices.extend_from_slice(&[a, b, c]);
    }

    Ok(())
}
