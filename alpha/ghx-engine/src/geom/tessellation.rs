//! Adaptive tessellation utilities for curves and surfaces.
//!
//! This module provides functions to tessellate curves and surfaces into discrete
//! point sets suitable for mesh generation. The tessellation is adaptive, meaning
//! it refines based on curvature and deviation thresholds rather than using a fixed
//! number of subdivisions.
//!
//! # Curve Tessellation
//!
//! Use [`tessellate_curve_adaptive_points`] for curvature-aware tessellation:
//!
//! ```ignore
//! use ghx_engine::geom::{Circle3, Point3, Vec3, CurveTessellationOptions, tessellate_curve_adaptive_points};
//!
//! let circle = Circle3::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0), 1.0);
//! let options = CurveTessellationOptions::new(0.01, 64); // 0.01 max deviation, 64 max segments
//! let points = tessellate_curve_adaptive_points(&circle, options);
//! ```
//!
//! # Surface Tessellation
//!
//! Use [`choose_surface_grid_counts`] to determine adaptive grid resolution:
//!
//! ```ignore
//! use ghx_engine::geom::{PlaneSurface, Point3, Vec3, SurfaceTessellationOptions, choose_surface_grid_counts};
//!
//! let plane = PlaneSurface::new(Point3::origin(), Vec3::unit_x(), Vec3::unit_y());
//! let options = SurfaceTessellationOptions::new(0.01, 1.0); // max deviation, max edge length
//! let (u_count, v_count) = choose_surface_grid_counts(&plane, options);
//! ```
//!
//! # Tolerances
//!
//! - `max_deviation`: Maximum allowed distance between the tessellated output and the true curve/surface.
//! - `max_edge_length`: Maximum allowed edge length in world units.
//! - `max_segments`/`max_u_count`/`max_v_count`: Caps to prevent runaway subdivision
//!   (curve `max_segments` is treated as a base cap and may be raised for long curves).

use super::core::Point3;
use super::curve::Curve3;
use super::surface::Surface;

/// Tessellates a curve uniformly into a fixed number of steps.
///
/// This is a simple wrapper around [`super::curve::tessellate_curve_uniform`].
/// For adaptive tessellation based on curvature, use [`tessellate_curve_adaptive_points`] instead.
///
/// # Arguments
/// * `curve` - The curve to tessellate.
/// * `steps` - Number of segments (output will have `steps + 1` points for open curves,
///   or `steps` points for closed curves).
///
/// # Returns
/// A vector of points sampled along the curve.
#[allow(dead_code)]
#[must_use]
pub fn tessellate_curve_points(curve: &impl Curve3, steps: usize) -> Vec<Point3> {
    super::curve::tessellate_curve_uniform(curve, steps)
}

/// Options controlling adaptive curve tessellation.
///
/// These options determine how finely a curve is subdivided based on
/// geometric criteria rather than a fixed segment count.
///
/// # Example
/// ```ignore
/// use ghx_engine::geom::CurveTessellationOptions;
///
/// // Fine tessellation with 0.001 deviation tolerance
/// let fine = CurveTessellationOptions::new(0.001, 256);
///
/// // Coarse tessellation for preview
/// let coarse = CurveTessellationOptions {
///     max_deviation: 0.1,
///     max_segments: 32,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurveTessellationOptions {
    /// Maximum allowed deviation (chord height) from the true curve.
    /// Smaller values produce finer tessellation. If non-finite or <= 0,
    /// uniform tessellation is used instead.
    pub max_deviation: f64,
    /// Base cap on the number of output segments.
    /// The adaptive tessellator may raise this for long, high-curvature curves.
    /// The actual output may have fewer segments if the deviation threshold is met.
    pub max_segments: usize,
    /// Maximum recursion depth for adaptive subdivision.
    /// Higher values allow finer subdivision but increase computation.
    pub max_depth: usize,
    /// Number of initial segments before adaptive refinement.
    /// Using arc-length parameterization, the curve is first divided into
    /// this many segments, then each is adaptively refined.
    pub initial_segments: usize,
}

impl Default for CurveTessellationOptions {
    fn default() -> Self {
        Self {
            max_deviation: 0.01,
            max_segments: 128,
            max_depth: 16,
            initial_segments: 1,
        }
    }
}

impl CurveTessellationOptions {
    /// Creates new curve tessellation options.
    ///
    /// # Arguments
    /// * `max_deviation` - Maximum chord height deviation from the true curve.
    /// * `max_segments` - Base cap on output segment count.
    #[must_use]
    pub const fn new(max_deviation: f64, max_segments: usize) -> Self {
        Self {
            max_deviation,
            max_segments,
            max_depth: 16,
            initial_segments: 1,
        }
    }
}

/// Adaptively tessellates a curve based on curvature and deviation thresholds.
///
/// This function produces a polyline approximation of the curve where:
/// - No point deviates more than `max_deviation` from the true curve
/// - The output respects a segment cap derived from `max_segments` and curve length/curvature
/// - Closed curves return points without a duplicate endpoint
///
/// # Algorithm
/// 1. The curve is initially divided using arc-length parameterization
/// 2. Each segment is adaptively subdivided based on deviation at 25%, 50%, 75% points
/// 3. Subdivision continues until deviation threshold is met or depth/segment limits are reached
///
/// # Arguments
/// * `curve` - The curve to tessellate (must implement [`Curve3`]).
/// * `options` - Tessellation options controlling quality and limits.
///
/// # Returns
/// A vector of points. For open curves, includes both endpoints.
/// For closed curves, the last point is omitted (not a duplicate of the first).
///
/// # Edge Cases
/// - Degenerate curves (zero domain span) return a single point.
/// - Non-finite deviation values fall back to uniform tessellation.
#[must_use]
pub fn tessellate_curve_adaptive_points(
    curve: &impl Curve3,
    options: CurveTessellationOptions,
) -> Vec<Point3> {
    let base_max_segments = options.max_segments.max(1);
    let max_deviation = options.max_deviation;
    if !max_deviation.is_finite() || max_deviation <= 0.0 {
        return super::curve::tessellate_curve_uniform(curve, base_max_segments);
    }

    let (t0, t1) = curve.domain();
    let span = t1 - t0;

    if !span.is_finite() {
        return super::curve::tessellate_curve_uniform(curve, base_max_segments);
    }

    if span == 0.0 {
        return vec![curve.point_at(t0)];
    }

    let closed = curve.is_closed();
    let base_max_segments = if closed {
        base_max_segments.max(3)
    } else {
        base_max_segments
    };
    let max_segments = estimate_curve_segment_budget(curve, max_deviation, base_max_segments);
    let max_points_output = if closed { max_segments } else { max_segments + 1 };
    let max_points_internal = if closed {
        max_points_output + 1
    } else {
        max_points_output
    };

    let max_depth = options.max_depth.max(1);
    let initial_segments = if closed {
        options.initial_segments.max(3).min(max_segments)
    } else {
        options.initial_segments.max(1).min(max_segments)
    };

    let initial_params = initial_curve_parameters_arc_length(curve, t0, t1, initial_segments);

    let mut points = Vec::new();
    points.push(curve.point_at(t0));

    for segment_index in 0..initial_segments {
        let segments_remaining = initial_segments - segment_index;
        let required_points_remaining = segments_remaining;
        let max_points_this_segment =
            max_points_internal.saturating_sub(required_points_remaining.saturating_sub(1));

        let a = initial_params[segment_index];
        let b = initial_params[segment_index + 1];
        let pa = curve.point_at(a);
        let pb = curve.point_at(b);

        tessellate_curve_segment_adaptive(
            curve,
            a,
            b,
            pa,
            pb,
            max_deviation,
            max_depth,
            max_points_this_segment,
            &mut points,
        );
    }

    if closed && points.len() > 1 {
        points.pop();
    }

    points
}

/// Estimates a segment budget based on arc length and curvature.
fn estimate_curve_segment_budget(
    curve: &impl Curve3,
    max_deviation: f64,
    base_max_segments: usize,
) -> usize {
    let length_samples = (base_max_segments.saturating_mul(4)).clamp(32, 1024);
    let arc_length = super::curve::curve_arc_length(curve, length_samples);
    if !arc_length.is_finite() || arc_length <= 0.0 {
        return base_max_segments;
    }

    let curvature_samples = (base_max_segments.saturating_mul(2)).clamp(16, 512);
    let max_curvature = estimate_max_curvature(curve, curvature_samples);
    if !max_curvature.is_finite() || max_curvature <= 0.0 {
        return base_max_segments;
    }

    let max_chord = 2.0 * (2.0 * max_deviation / max_curvature).sqrt();
    if !max_chord.is_finite() || max_chord <= 0.0 {
        return base_max_segments;
    }

    let required_segments = (arc_length / max_chord).ceil() as usize;
    base_max_segments.max(required_segments)
}

/// Estimates the maximum curvature by sampling the curve.
fn estimate_max_curvature(curve: &impl Curve3, samples: usize) -> f64 {
    let samples = samples.max(1);
    let (t0, t1) = curve.domain();
    let span = t1 - t0;
    if !span.is_finite() || span == 0.0 {
        return 0.0;
    }

    let mut max_curvature = 0.0;
    for i in 0..=samples {
        let t = t0 + span * (i as f64 / samples as f64);
        if let Some(curvature) = curve.curvature_at(t) {
            let curvature = curvature.abs();
            if curvature.is_finite() && curvature > max_curvature {
                max_curvature = curvature;
            }
        }
    }

    max_curvature
}

/// Tessellates a surface into a regular grid of points.
///
/// This is a simple wrapper around [`super::surface::tessellate_surface_grid`].
/// For adaptive resolution selection, use [`choose_surface_grid_counts`] first.
///
/// # Arguments
/// * `surface` - The surface to tessellate.
/// * `u_count` - Number of points in the U direction.
/// * `v_count` - Number of points in the V direction.
///
/// # Returns
/// A vector of `u_count * v_count` points in row-major order (U varies fastest).
#[must_use]
pub fn tessellate_surface_grid_points(
    surface: &impl Surface,
    u_count: usize,
    v_count: usize,
) -> Vec<Point3> {
    super::surface::tessellate_surface_grid(surface, u_count, v_count)
}

/// Options controlling adaptive surface tessellation grid resolution.
///
/// These options determine the U and V grid counts for surface tessellation
/// based on geometric criteria (deviation and edge length).
///
/// # Example
/// ```ignore
/// use ghx_engine::geom::SurfaceTessellationOptions;
///
/// // Fine tessellation with small deviation tolerance
/// let fine = SurfaceTessellationOptions::new(0.001, 0.5);
///
/// // Coarse tessellation for preview
/// let coarse = SurfaceTessellationOptions {
///     max_deviation: 0.1,
///     max_edge_length: 5.0,
///     max_u_count: 32,
///     max_v_count: 32,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceTessellationOptions {
    /// Maximum allowed deviation from the true surface.
    /// The grid is refined until bilinear interpolation error is below this threshold.
    /// Use `f64::NAN` or `f64::INFINITY` to disable deviation-based refinement.
    pub max_deviation: f64,
    /// Maximum allowed edge length in world units.
    /// The grid is refined until edges are shorter than this threshold.
    /// Use `f64::NAN` or `f64::INFINITY` to disable edge-length-based refinement.
    pub max_edge_length: f64,
    /// Hard cap on grid resolution in the U direction.
    pub max_u_count: usize,
    /// Hard cap on grid resolution in the V direction.
    pub max_v_count: usize,
    /// Initial grid resolution in U before adaptive refinement.
    pub initial_u_count: usize,
    /// Initial grid resolution in V before adaptive refinement.
    pub initial_v_count: usize,
    /// Maximum number of refinement iterations.
    pub max_iterations: usize,
}

impl Default for SurfaceTessellationOptions {
    fn default() -> Self {
        Self {
            max_deviation: 0.01,
            max_edge_length: 1.0,
            max_u_count: 256,
            max_v_count: 256,
            initial_u_count: 8,
            initial_v_count: 8,
            max_iterations: 16,
        }
    }
}

impl SurfaceTessellationOptions {
    /// Creates new surface tessellation options.
    ///
    /// # Arguments
    /// * `max_deviation` - Maximum surface deviation tolerance.
    /// * `max_edge_length` - Maximum edge length in world units.
    #[must_use]
    pub const fn new(max_deviation: f64, max_edge_length: f64) -> Self {
        Self {
            max_deviation,
            max_edge_length,
            max_u_count: 256,
            max_v_count: 256,
            initial_u_count: 8,
            initial_v_count: 8,
            max_iterations: 16,
        }
    }
}

/// Determines optimal grid resolution for surface tessellation.\n///
/// This function iteratively refines the grid resolution until the estimated
/// tessellation error (deviation and edge length) meets the specified thresholds,
/// or the maximum counts/iterations are reached.
///
/// # Algorithm
/// 1. Starts with initial U/V counts from options
/// 2. Estimates error by sampling grid cells (bilinear deviation + edge lengths)
/// 3. Doubles U and/or V counts if thresholds are exceeded
/// 4. Repeats until thresholds are met or limits are reached
///
/// # Arguments
/// * `surface` - The surface to analyze.
/// * `options` - Tessellation options with quality thresholds and limits.
///
/// # Returns
/// A tuple `(u_count, v_count)` suitable for grid tessellation.
///
/// # Notes
/// - Closed surfaces (cylinders, tori) have minimum counts of 3 to avoid degeneracy.
/// - Surfaces with poles at both V ends have minimum V count of 3.
/// - Refinement is anisotropic: U and V are refined independently based on error direction.
#[must_use]
pub fn choose_surface_grid_counts(
    surface: &impl Surface,
    options: SurfaceTessellationOptions,
) -> (usize, usize) {
    let wrap_u = surface.is_u_closed();
    let wrap_v = surface.is_v_closed();
    let pole_start = surface.pole_v_start();
    let pole_end = surface.pole_v_end();

    let mut u_count = options.initial_u_count.max(1);
    let mut v_count = options.initial_v_count.max(1);

    u_count = if wrap_u { u_count.max(3) } else { u_count.max(2) };
    v_count = if wrap_v { v_count.max(3) } else { v_count.max(2) };
    if pole_start && pole_end {
        v_count = v_count.max(3);
    }

    let mut u_max = options.max_u_count.max(u_count);
    let mut v_max = options.max_v_count.max(v_count);
    if wrap_u {
        u_max = u_max.max(3);
    }
    if wrap_v {
        v_max = v_max.max(3);
    }

    if !options.max_deviation.is_finite() && !options.max_edge_length.is_finite() {
        return (u_count, v_count);
    }

    let max_deviation = options.max_deviation;
    let max_edge_length = options.max_edge_length;

    for _ in 0..options.max_iterations.max(1) {
        let (dev, edge_u, edge_v) = estimate_surface_grid_error(surface, u_count, v_count);

        let dev_ok = !max_deviation.is_finite() || max_deviation <= 0.0 || dev <= max_deviation;
        let edge_u_ok =
            !max_edge_length.is_finite() || max_edge_length <= 0.0 || edge_u <= max_edge_length;
        let edge_v_ok =
            !max_edge_length.is_finite() || max_edge_length <= 0.0 || edge_v <= max_edge_length;

        if dev_ok && edge_u_ok && edge_v_ok {
            break;
        }

        let prev_u = u_count;
        let prev_v = v_count;

        let mut refine_u = !edge_u_ok;
        let mut refine_v = !edge_v_ok;

        if !dev_ok && !refine_u && !refine_v {
            if edge_u >= edge_v {
                refine_u = true;
            } else {
                refine_v = true;
            }
        }

        if refine_u && u_count < u_max {
            u_count = (u_count.saturating_mul(2)).min(u_max);
        }
        if refine_v && v_count < v_max {
            v_count = (v_count.saturating_mul(2)).min(v_max);
        }

        if !dev_ok && u_count == prev_u && v_count == prev_v {
            if u_count < u_max && !refine_u {
                u_count = (u_count.saturating_mul(2)).min(u_max);
            } else if v_count < v_max && !refine_v {
                v_count = (v_count.saturating_mul(2)).min(v_max);
            }
        }

        if u_count == prev_u && v_count == prev_v {
            break;
        }
    }

    (u_count, v_count)
}

/// Estimates the tessellation error for a given grid resolution.
///
/// Returns (max_deviation, max_edge_u, max_edge_v) where:
/// - `max_deviation`: Maximum bilinear interpolation error at cell centers
/// - `max_edge_u`: Maximum edge length in U direction
/// - `max_edge_v`: Maximum edge length in V direction
///
/// Note: Only samples a subset of cells (up to 16×16) for performance.
fn estimate_surface_grid_error(
    surface: &impl Surface,
    u_count: usize,
    v_count: usize,
) -> (f64, f64, f64) {
    let wrap_u = surface.is_u_closed();
    let wrap_v = surface.is_v_closed();

    let u_count = if wrap_u { u_count.max(3) } else { u_count.max(2) };
    let v_count = if wrap_v { v_count.max(3) } else { v_count.max(2) };

    let (u0, u1) = surface.domain_u();
    let (v0, v1) = surface.domain_v();

    let u_span = u1 - u0;
    let v_span = v1 - v0;

    let quad_u = if wrap_u { u_count } else { u_count - 1 };
    let quad_v = if wrap_v { v_count } else { v_count - 1 };

    let sample_u = quad_u.min(16);
    let sample_v = quad_v.min(16);

    let step_u = quad_u / sample_u.max(1);
    let step_v = quad_v / sample_v.max(1);

    let mut max_dev: f64 = 0.0;
    let mut max_edge_u: f64 = 0.0;
    let mut max_edge_v: f64 = 0.0;

    for v in (0..quad_v).step_by(step_v.max(1)) {
        for u in (0..quad_u).step_by(step_u.max(1)) {
            let (ua, ub) = surface_cell_params(u0, u1, u_span, u, u_count, wrap_u);
            let (va, vb) = surface_cell_params(v0, v1, v_span, v, v_count, wrap_v);

            let p00 = surface.point_at(ua, va);
            let p10 = surface.point_at(ub, va);
            let p01 = surface.point_at(ua, vb);
            let p11 = surface.point_at(ub, vb);

            let edge_u0 = p10.sub_point(p00).length();
            let edge_u1 = p11.sub_point(p01).length();
            let edge_v0 = p01.sub_point(p00).length();
            let edge_v1 = p11.sub_point(p10).length();
            max_edge_u = max_edge_u.max(edge_u0).max(edge_u1);
            max_edge_v = max_edge_v.max(edge_v0).max(edge_v1);

            let um = 0.5 * (ua + ub);
            let vm = 0.5 * (va + vb);
            let pm = surface.point_at(um, vm);

            let bilinear = lerp_point(
                lerp_point(p00, p10, 0.5),
                lerp_point(p01, p11, 0.5),
                0.5,
            );

            let dev = pm.sub_point(bilinear).length();
            max_dev = max_dev.max(dev);
        }
    }

    (max_dev, max_edge_u, max_edge_v)
}

/// Computes parameter bounds for a grid cell.
///
/// Handles both wrapped (closed) and non-wrapped (open) parameterizations.
#[inline]
fn surface_cell_params(
    start: f64,
    end: f64,
    span: f64,
    idx: usize,
    count: usize,
    wrap: bool,
) -> (f64, f64) {
    if !span.is_finite() || span == 0.0 {
        return (start, start);
    }

    if wrap {
        let denom = count as f64;
        let a = start + span * (idx as f64 / denom);
        let b = if idx + 1 == count {
            end
        } else {
            start + span * ((idx + 1) as f64 / denom)
        };
        (a, b)
    } else {
        let denom = (count - 1) as f64;
        let a = start + span * (idx as f64 / denom);
        let b = start + span * ((idx + 1) as f64 / denom);
        (a, b)
    }
}

/// Linearly interpolates between two points.
#[inline]
fn lerp_point(a: Point3, b: Point3, t: f64) -> Point3 {
    Point3::new(
        a.x + (b.x - a.x) * t,
        a.y + (b.y - a.y) * t,
        a.z + (b.z - a.z) * t,
    )
}

/// Computes initial parameter values using arc-length approximation.
///
/// Divides the curve into `segments` portions of approximately equal arc length.
/// Uses chord-length approximation with fine sampling for accuracy.
///
/// # Returns
/// A vector of `segments + 1` parameter values, including `t0` and `t1`.
fn initial_curve_parameters_arc_length(
    curve: &impl Curve3,
    t0: f64,
    t1: f64,
    segments: usize,
) -> Vec<f64> {
    let segments = segments.max(1);
    if segments == 1 {
        return vec![t0, t1];
    }

    let span = t1 - t0;
    if !span.is_finite() || span == 0.0 {
        return vec![t0, t1];
    }

    let sample_count = (segments.saturating_mul(16)).clamp(16, 4096);
    let mut params = Vec::with_capacity(sample_count + 1);
    let mut cumulative = Vec::with_capacity(sample_count + 1);

    params.push(t0);
    cumulative.push(0.0);
    let mut total = 0.0;
    let mut prev = curve.point_at(t0);

    for i in 1..=sample_count {
        let u = i as f64 / sample_count as f64;
        let t = t0 + span * u;
        let p = curve.point_at(t);
        let d = p.sub_point(prev).length();
        if d.is_finite() {
            total += d;
        }
        params.push(t);
        cumulative.push(total);
        prev = p;
    }

    if !total.is_finite() || total <= 0.0 {
        return (0..=segments)
            .map(|i| t0 + span * (i as f64 / segments as f64))
            .collect();
    }

    let mut result = Vec::with_capacity(segments + 1);
    result.push(t0);

    for segment_index in 1..segments {
        let target = total * (segment_index as f64 / segments as f64);
        let idx = match cumulative.binary_search_by(|value| value.total_cmp(&target)) {
            Ok(i) => i,
            Err(i) => i,
        };

        let idx = idx.clamp(1, sample_count);
        let c0 = cumulative[idx - 1];
        let c1 = cumulative[idx];
        let t = if c1 > c0 {
            let ratio = ((target - c0) / (c1 - c0)).clamp(0.0, 1.0);
            params[idx - 1] + (params[idx] - params[idx - 1]) * ratio
        } else {
            params[idx]
        };
        result.push(t);
    }

    result.push(t1);
    result
}

/// Finds the parameter value at approximately the arc-length midpoint of a curve segment.
///
/// Uses sampling to approximate the arc-length and binary search interpolation
/// to find the parameter that corresponds to half the arc length.
///
/// # Arguments
/// * `curve` - The curve to sample.
/// * `t0` - Start parameter.
/// * `t1` - End parameter.
/// * `ratio` - Target ratio of arc length (0.0 = start, 1.0 = end, 0.5 = midpoint).
/// * `samples` - Number of samples for arc-length approximation.
///
/// # Returns
/// The parameter value at approximately the target arc-length ratio.
fn parameter_at_arc_length_ratio<C: Curve3>(
    curve: &C,
    t0: f64,
    t1: f64,
    ratio: f64,
    samples: usize,
) -> f64 {
    let samples = samples.max(4);
    let span = t1 - t0;

    if !span.is_finite() || span.abs() < 1e-15 {
        return t0 + span * ratio;
    }

    // Build a small arc-length table for this segment
    let mut params = Vec::with_capacity(samples + 1);
    let mut cumulative = Vec::with_capacity(samples + 1);

    params.push(t0);
    cumulative.push(0.0);
    let mut total = 0.0;
    let mut prev = curve.point_at(t0);

    for i in 1..=samples {
        let u = i as f64 / samples as f64;
        let t = t0 + span * u;
        let p = curve.point_at(t);
        let d = p.sub_point(prev).length();
        if d.is_finite() {
            total += d;
        }
        params.push(t);
        cumulative.push(total);
        prev = p;
    }

    if !total.is_finite() || total <= 0.0 {
        return t0 + span * ratio;
    }

    let target = total * ratio.clamp(0.0, 1.0);

    // Binary search for the target arc length
    let idx = match cumulative.binary_search_by(|value| value.total_cmp(&target)) {
        Ok(i) => i,
        Err(i) => i,
    };

    let idx = idx.clamp(1, samples);
    let c0 = cumulative[idx - 1];
    let c1 = cumulative[idx];

    if c1 > c0 {
        let local_ratio = ((target - c0) / (c1 - c0)).clamp(0.0, 1.0);
        params[idx - 1] + (params[idx] - params[idx - 1]) * local_ratio
    } else {
        params[idx]
    }
}

/// Adaptively tessellates a single curve segment.
///
/// Uses an explicit stack (iterative) to avoid stack overflow on deep recursion.
/// Subdivides based on deviation at 25%, 50%, and 75% points within each segment.
/// Uses arc-length based subdivision for uniform point distribution.
fn tessellate_curve_segment_adaptive(
    curve: &impl Curve3,
    t0: f64,
    t1: f64,
    p0: Point3,
    p1: Point3,
    max_deviation: f64,
    max_depth: usize,
    max_points: usize,
    points: &mut Vec<Point3>,
) {
    #[derive(Debug, Clone, Copy)]
    struct Segment {
        t0: f64,
        t1: f64,
        p0: Point3,
        p1: Point3,
        depth: usize,
    }

    // Number of samples for arc-length approximation per segment.
    // Higher values give more uniform distribution but cost more evaluations.
    // Use fewer samples at deeper recursion levels since segments are shorter.
    const BASE_ARC_LENGTH_SAMPLES: usize = 8;

    let mut stack = Vec::new();
    stack.push(Segment {
        t0,
        t1,
        p0,
        p1,
        depth: 0,
    });

    while let Some(seg) = stack.pop() {
        if points.len() >= max_points {
            break;
        }

        let point_budget_exhausted = points.len() + stack.len() + 1 >= max_points;
        if seg.depth >= max_depth || point_budget_exhausted {
            if points.len() < max_points {
                points.push(seg.p1);
            }
            continue;
        }

        // Use arc-length based subdivision for uniform spacing.
        // Reduce sample count at deeper levels since segments are shorter.
        let samples = BASE_ARC_LENGTH_SAMPLES.saturating_sub(seg.depth).max(4);

        let tm = parameter_at_arc_length_ratio(curve, seg.t0, seg.t1, 0.5, samples);
        let t25 = parameter_at_arc_length_ratio(curve, seg.t0, seg.t1, 0.25, samples);
        let t75 = parameter_at_arc_length_ratio(curve, seg.t0, seg.t1, 0.75, samples);

        let pm = curve.point_at(tm);
        let p25 = curve.point_at(t25);
        let p75 = curve.point_at(t75);
        let deviation = distance_point_to_line(pm, seg.p0, seg.p1)
            .max(distance_point_to_line(p25, seg.p0, seg.p1))
            .max(distance_point_to_line(p75, seg.p0, seg.p1));

        if deviation.is_finite() && deviation > max_deviation {
            let next_depth = seg.depth + 1;
            stack.push(Segment {
                t0: tm,
                t1: seg.t1,
                p0: pm,
                p1: seg.p1,
                depth: next_depth,
            });
            stack.push(Segment {
                t0: seg.t0,
                t1: tm,
                p0: seg.p0,
                p1: pm,
                depth: next_depth,
            });
        } else {
            points.push(seg.p1);
        }
    }
}

/// Computes the perpendicular distance from point `p` to the line defined by `a` and `b`.
///
/// Uses the cross product formula: distance = |AP × AB| / |AB|
#[inline]
fn distance_point_to_line(p: Point3, a: Point3, b: Point3) -> f64 {
    let ab = b.sub_point(a);
    let ap = p.sub_point(a);
    let ab_len = ab.length();
    if ab_len <= 0.0 || !ab_len.is_finite() {
        return ap.length();
    }
    ap.cross(ab).length() / ab_len
}
