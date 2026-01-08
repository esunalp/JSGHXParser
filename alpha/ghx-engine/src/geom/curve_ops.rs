//! Curve utility operations: offset, join, flip, extend.
//!
//! This module provides polyline/curve manipulation operations used by the
//! Grasshopper "Curve → Util" components. All geometry logic lives here;
//! components only coerce inputs and call these functions.
//!
//! # Operations
//! - **Offset**: Offsets a polyline by a distance in a plane.
//! - **Join**: Joins multiple polylines into connected chains.
//! - **Flip**: Reverses the direction of a polyline.
//! - **Extend**: Extends or trims a polyline from start/end.
//!
//! # Example
//!
//! ```ignore
//! use ghx_engine::geom::{offset_polyline, OffsetPolylineOptions, Tolerance};
//!
//! let points = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]];
//! let options = OffsetPolylineOptions::new(0.1);
//! let (result, diag) = offset_polyline(&points, options, Tolerance::default_geom())?;
//! ```

use super::core::Tolerance;

// ============================================================================
// Offset Polyline
// ============================================================================

/// Options for polyline offset operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OffsetPolylineOptions {
    /// The offset distance. Positive = left (CCW), negative = right (CW).
    pub distance: f64,
    /// Whether the polyline is closed.
    pub closed: bool,
    /// Custom plane origin (if None, derived from first 3 points).
    pub plane_origin: Option<[f64; 3]>,
    /// Custom plane normal (if None, derived from first 3 points).
    pub plane_normal: Option<[f64; 3]>,
    /// Custom plane X axis (if None, derived from first 2 points).
    pub plane_x_axis: Option<[f64; 3]>,
}

impl OffsetPolylineOptions {
    /// Create new offset options with the given distance.
    #[must_use]
    pub const fn new(distance: f64) -> Self {
        Self {
            distance,
            closed: false,
            plane_origin: None,
            plane_normal: None,
            plane_x_axis: None,
        }
    }

    /// Set whether the polyline is closed.
    #[must_use]
    pub const fn closed(mut self, closed: bool) -> Self {
        self.closed = closed;
        self
    }

    /// Set a custom plane origin.
    #[must_use]
    pub const fn with_plane_origin(mut self, origin: [f64; 3]) -> Self {
        self.plane_origin = Some(origin);
        self
    }

    /// Set a custom plane normal.
    #[must_use]
    pub const fn with_plane_normal(mut self, normal: [f64; 3]) -> Self {
        self.plane_normal = Some(normal);
        self
    }

    /// Set a custom plane X axis.
    #[must_use]
    pub const fn with_plane_x_axis(mut self, x_axis: [f64; 3]) -> Self {
        self.plane_x_axis = Some(x_axis);
        self
    }
}

impl Default for OffsetPolylineOptions {
    fn default() -> Self {
        Self::new(0.0)
    }
}

/// Errors that can occur during offset operations.
#[derive(Debug, thiserror::Error)]
pub enum OffsetPolylineError {
    /// The input polyline has fewer than 2 points.
    #[error("polyline must have at least 2 points, got {count}")]
    InsufficientPoints { count: usize },

    /// The offset distance is not finite.
    #[error("offset distance must be finite: {distance}")]
    InvalidDistance { distance: f64 },

    /// Could not determine a valid offset plane.
    #[error("could not determine offset plane from degenerate polyline")]
    DegeneratePlane,
}

/// Diagnostics for polyline offset operations.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct OffsetPolylineDiagnostics {
    /// Number of input points.
    pub input_point_count: usize,
    /// Number of output points.
    pub output_point_count: usize,
    /// Whether the polyline was treated as closed.
    pub closed: bool,
    /// Whether the plane was auto-detected.
    pub plane_auto_detected: bool,
    /// Warnings generated during the operation.
    pub warnings: Vec<String>,
}

/// Offsets a polyline by a distance in a plane.
///
/// The offset direction follows the right-hand rule: positive distance
/// offsets to the left (CCW) of the polyline direction, negative to the right.
///
/// # Arguments
/// * `points` - The input polyline points.
/// * `options` - Offset options including distance, plane, and closed flag.
/// * `tol` - Tolerance for geometry operations.
///
/// # Returns
/// A tuple of the offset points and diagnostics.
///
/// # Errors
/// Returns an error if the polyline is too short or degenerate.
pub fn offset_polyline(
    points: &[[f64; 3]],
    options: OffsetPolylineOptions,
    tol: Tolerance,
) -> Result<(Vec<[f64; 3]>, OffsetPolylineDiagnostics), OffsetPolylineError> {
    if points.len() < 2 {
        return Err(OffsetPolylineError::InsufficientPoints {
            count: points.len(),
        });
    }

    if !options.distance.is_finite() {
        return Err(OffsetPolylineError::InvalidDistance {
            distance: options.distance,
        });
    }

    let mut diagnostics = OffsetPolylineDiagnostics {
        input_point_count: points.len(),
        closed: options.closed,
        ..Default::default()
    };

    // If distance is essentially zero, return the input unchanged
    if options.distance.abs() < tol.eps {
        diagnostics.output_point_count = points.len();
        return Ok((points.to_vec(), diagnostics));
    }

    // Determine the plane
    let plane = determine_offset_plane(points, &options, tol, &mut diagnostics)?;

    // Project points to 2D plane coordinates
    let coords_2d: Vec<[f64; 2]> = points
        .iter()
        .map(|p| plane.project_to_2d(*p))
        .collect();

    // Compute offset normals in 2D
    let normals_2d = compute_offset_normals_2d(&coords_2d, options.closed);

    // Apply offset and project back to 3D
    let offset_points: Vec<[f64; 3]> = coords_2d
        .iter()
        .zip(normals_2d.iter())
        .map(|(coord, normal)| {
            let offset_2d = [
                coord[0] + normal[0] * options.distance,
                coord[1] + normal[1] * options.distance,
            ];
            plane.project_to_3d(offset_2d)
        })
        .collect();

    diagnostics.output_point_count = offset_points.len();
    Ok((offset_points, diagnostics))
}

/// Internal plane representation for offset operations.
#[derive(Debug, Clone, Copy)]
struct OffsetPlane {
    origin: [f64; 3],
    x_axis: [f64; 3],
    y_axis: [f64; 3],
    normal: [f64; 3],
}

impl OffsetPlane {
    fn project_to_2d(&self, point: [f64; 3]) -> [f64; 2] {
        let relative = sub(point, self.origin);
        [dot(relative, self.x_axis), dot(relative, self.y_axis)]
    }

    fn project_to_3d(&self, coord: [f64; 2]) -> [f64; 3] {
        add(
            add(self.origin, scale(self.x_axis, coord[0])),
            scale(self.y_axis, coord[1]),
        )
    }
}

fn determine_offset_plane(
    points: &[[f64; 3]],
    options: &OffsetPolylineOptions,
    tol: Tolerance,
    diagnostics: &mut OffsetPolylineDiagnostics,
) -> Result<OffsetPlane, OffsetPolylineError> {
    let origin = options.plane_origin.unwrap_or(points[0]);

    // Try to get normal and x_axis from options or derive them
    let (x_axis, normal) = match (options.plane_x_axis, options.plane_normal) {
        (Some(x), Some(n)) => (normalize(x), normalize(n)),
        (Some(x), None) => {
            // Derive normal from points
            let x_axis = normalize(x);
            let normal = derive_plane_normal(points, tol).unwrap_or([0.0, 0.0, 1.0]);
            (x_axis, normal)
        }
        (None, Some(n)) => {
            // Derive x_axis from first segment
            let normal = normalize(n);
            let x_axis = derive_x_axis(points, normal, tol).unwrap_or([1.0, 0.0, 0.0]);
            (x_axis, normal)
        }
        (None, None) => {
            // Auto-detect both
            diagnostics.plane_auto_detected = true;
            let normal = derive_plane_normal(points, tol).unwrap_or([0.0, 0.0, 1.0]);
            let x_axis = derive_x_axis(points, normal, tol).unwrap_or([1.0, 0.0, 0.0]);
            (x_axis, normal)
        }
    };

    // Compute y_axis as normal × x_axis (right-handed system)
    let y_axis = normalize(cross(normal, x_axis));

    // Verify we have valid axes
    if length(x_axis) < tol.eps || length(y_axis) < tol.eps {
        return Err(OffsetPolylineError::DegeneratePlane);
    }

    Ok(OffsetPlane {
        origin,
        x_axis,
        y_axis,
        normal,
    })
}

fn derive_plane_normal(points: &[[f64; 3]], tol: Tolerance) -> Option<[f64; 3]> {
    if points.len() < 3 {
        // For 2-point polylines, use world Z
        return Some([0.0, 0.0, 1.0]);
    }

    // Try first 3 non-collinear points
    let a = points[0];
    for i in 1..points.len() {
        let b = points[i];
        let ab = sub(b, a);
        if length(ab) < tol.eps {
            continue;
        }

        for j in (i + 1)..points.len() {
            let c = points[j];
            let ac = sub(c, a);
            let normal = cross(ab, ac);
            if length(normal) > tol.eps {
                return Some(normalize(normal));
            }
        }
    }

    // Fallback to world Z for collinear points
    Some([0.0, 0.0, 1.0])
}

fn derive_x_axis(points: &[[f64; 3]], normal: [f64; 3], tol: Tolerance) -> Option<[f64; 3]> {
    if points.len() < 2 {
        return None;
    }

    // Use first non-zero segment direction, projected onto the plane
    for window in points.windows(2) {
        let dir = sub(window[1], window[0]);
        if length(dir) < tol.eps {
            continue;
        }
        // Project direction onto plane (remove component along normal)
        let projected = sub(dir, scale(normal, dot(dir, normal)));
        if length(projected) > tol.eps {
            return Some(normalize(projected));
        }
    }

    // Fallback: create orthogonal vector to normal
    let candidate = if normal[0].abs() < normal[1].abs() {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let x = cross(candidate, normal);
    if length(x) > tol.eps {
        Some(normalize(x))
    } else {
        Some([1.0, 0.0, 0.0])
    }
}

fn compute_offset_normals_2d(coords: &[[f64; 2]], closed: bool) -> Vec<[f64; 2]> {
    let count = coords.len();
    if count == 0 {
        return Vec::new();
    }
    if count == 1 {
        return vec![[0.0, 0.0]];
    }

    let mut normals = vec![[0.0, 0.0]; count];
    let segment_count = if closed { count } else { count - 1 };

    // Accumulate normals from each segment
    for i in 0..segment_count {
        let a = coords[i];
        let b = coords[(i + 1) % count];
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let len = (dx * dx + dy * dy).sqrt();
        if len < 1e-12 {
            continue;
        }
        // Left normal in 2D: rotate direction 90° CCW → (-dy, dx)
        let nx = -dy / len;
        let ny = dx / len;

        normals[i][0] += nx;
        normals[i][1] += ny;
        normals[(i + 1) % count][0] += nx;
        normals[(i + 1) % count][1] += ny;
    }

    // Normalize the averaged normals
    for (index, normal) in normals.iter_mut().enumerate() {
        let len = (normal[0] * normal[0] + normal[1] * normal[1]).sqrt();
        if len < 1e-12 {
            // Fallback: use direction from neighbors
            let prev_idx = if index == 0 {
                if closed { count - 1 } else { 0 }
            } else {
                index - 1
            };
            let next_idx = if index + 1 >= count {
                if closed { 0 } else { count - 1 }
            } else {
                index + 1
            };
            let prev = coords[prev_idx];
            let next = coords[next_idx];
            let dx = next[0] - prev[0];
            let dy = next[1] - prev[1];
            let fallback_len = (dx * dx + dy * dy).sqrt();
            if fallback_len > 1e-12 {
                normal[0] = -dy / fallback_len;
                normal[1] = dx / fallback_len;
            }
        } else {
            normal[0] /= len;
            normal[1] /= len;
        }
    }

    normals
}

// ============================================================================
// Join Polylines
// ============================================================================

/// Options for joining polylines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JoinPolylinesOptions {
    /// Whether to preserve the direction of input polylines (no reversal).
    pub preserve_direction: bool,
    /// Tolerance for endpoint matching.
    pub tolerance: f64,
}

impl JoinPolylinesOptions {
    /// Create new join options.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            preserve_direction: false,
            tolerance: 1e-6,
        }
    }

    /// Set whether to preserve polyline directions.
    #[must_use]
    pub const fn preserve_direction(mut self, preserve: bool) -> Self {
        self.preserve_direction = preserve;
        self
    }

    /// Set the endpoint matching tolerance.
    #[must_use]
    pub const fn with_tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = tolerance;
        self
    }
}

impl Default for JoinPolylinesOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Diagnostics for join operations.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct JoinPolylinesDiagnostics {
    /// Number of input polylines.
    pub input_polyline_count: usize,
    /// Number of output polylines after joining.
    pub output_polyline_count: usize,
    /// Number of joins performed.
    pub join_count: usize,
    /// Warnings generated during the operation.
    pub warnings: Vec<String>,
}

/// Joins multiple polylines into connected chains where endpoints match.
///
/// # Arguments
/// * `polylines` - The input polylines to join.
/// * `options` - Join options including tolerance and direction preservation.
///
/// # Returns
/// A tuple of the joined polylines and diagnostics.
pub fn join_polylines(
    polylines: Vec<Vec<[f64; 3]>>,
    options: JoinPolylinesOptions,
) -> (Vec<Vec<[f64; 3]>>, JoinPolylinesDiagnostics) {
    let mut diagnostics = JoinPolylinesDiagnostics {
        input_polyline_count: polylines.len(),
        ..Default::default()
    };

    if polylines.is_empty() {
        return (Vec::new(), diagnostics);
    }

    let tolerance = options.tolerance.max(1e-12);
    let mut remaining = polylines;
    let mut result = Vec::new();

    while let Some(mut current) = remaining.pop() {
        if current.len() < 2 {
            result.push(current);
            continue;
        }

        let mut changed = true;
        while changed {
            changed = false;
            let mut index = 0;
            while index < remaining.len() {
                let candidate = &remaining[index];
                if candidate.len() < 2 {
                    remaining.remove(index);
                    continue;
                }

                if let Some(merged) =
                    try_merge_polylines(&current, candidate, options.preserve_direction, tolerance)
                {
                    current = merged;
                    remaining.remove(index);
                    diagnostics.join_count += 1;
                    changed = true;
                } else {
                    index += 1;
                }
            }
        }
        result.push(current);
    }

    diagnostics.output_polyline_count = result.len();
    (result, diagnostics)
}

fn try_merge_polylines(
    target: &[[f64; 3]],
    candidate: &[[f64; 3]],
    preserve_direction: bool,
    tolerance: f64,
) -> Option<Vec<[f64; 3]>> {
    if target.is_empty() || candidate.len() < 2 {
        return None;
    }

    let start = target[0];
    let end = *target.last()?;
    let candidate_start = candidate[0];
    let candidate_end = *candidate.last()?;

    // Case 1: end of target meets start of candidate
    if distance(end, candidate_start) < tolerance {
        let mut merged = target.to_vec();
        merged.extend_from_slice(&candidate[1..]);
        return Some(merged);
    }

    // Case 2: end of target meets end of candidate (reverse candidate)
    if !preserve_direction && distance(end, candidate_end) < tolerance {
        let mut merged = target.to_vec();
        let mut reversed: Vec<_> = candidate.iter().copied().collect();
        reversed.reverse();
        merged.extend_from_slice(&reversed[1..]);
        return Some(merged);
    }

    // Case 3: start of target meets end of candidate
    if distance(start, candidate_end) < tolerance {
        let mut merged = candidate.to_vec();
        merged.pop(); // Remove duplicate point
        merged.extend_from_slice(target);
        return Some(merged);
    }

    // Case 4: start of target meets start of candidate (reverse candidate)
    if !preserve_direction && distance(start, candidate_start) < tolerance {
        let mut reversed: Vec<_> = candidate.iter().copied().collect();
        reversed.reverse();
        reversed.pop(); // Remove duplicate point
        reversed.extend_from_slice(target);
        return Some(reversed);
    }

    None
}

// ============================================================================
// Flip Polyline
// ============================================================================

/// Options for flipping polylines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlipPolylineOptions {
    /// Whether to force flipping regardless of guide.
    pub force: bool,
}

impl FlipPolylineOptions {
    /// Create new flip options.
    #[must_use]
    pub const fn new() -> Self {
        Self { force: false }
    }

    /// Set whether to force the flip.
    #[must_use]
    pub const fn force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }
}

impl Default for FlipPolylineOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Diagnostics for flip operations.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FlipPolylineDiagnostics {
    /// Whether the polyline was flipped.
    pub was_flipped: bool,
    /// Reason for the flip decision.
    pub reason: String,
}

/// Flips (reverses) a polyline, optionally based on a guide curve.
///
/// When a guide is provided, the polyline is flipped if its start is closer
/// to the end of the guide than to the start of the guide.
///
/// # Arguments
/// * `points` - The input polyline points.
/// * `guide` - Optional guide polyline to determine flip direction.
/// * `options` - Flip options.
///
/// # Returns
/// A tuple of the (possibly flipped) points, whether it was flipped, and diagnostics.
pub fn flip_polyline(
    points: &[[f64; 3]],
    guide: Option<&[[f64; 3]]>,
    options: FlipPolylineOptions,
) -> (Vec<[f64; 3]>, FlipPolylineDiagnostics) {
    let mut diagnostics = FlipPolylineDiagnostics::default();

    if points.len() < 2 {
        diagnostics.reason = "polyline too short to flip".to_string();
        return (points.to_vec(), diagnostics);
    }

    // Check if closed (closed polylines always flip by default in GH behavior)
    let is_closed = is_polyline_closed(points, 1e-6);

    let should_flip = if options.force {
        diagnostics.reason = "forced flip".to_string();
        true
    } else if is_closed {
        diagnostics.reason = "closed polyline, default flip".to_string();
        true
    } else if let Some(guide_pts) = guide {
        if guide_pts.len() >= 2 {
            let start = points[0];
            let guide_start = guide_pts[0];
            let guide_end = *guide_pts.last().unwrap();
            let dist_to_start = distance(start, guide_start);
            let dist_to_end = distance(start, guide_end);
            let flip = dist_to_end < dist_to_start;
            diagnostics.reason = format!(
                "guide comparison: dist_to_start={:.6}, dist_to_end={:.6}",
                dist_to_start, dist_to_end
            );
            flip
        } else {
            diagnostics.reason = "guide too short, default flip".to_string();
            true
        }
    } else {
        diagnostics.reason = "no guide provided, default flip".to_string();
        true
    };

    diagnostics.was_flipped = should_flip;

    let result = if should_flip {
        let mut reversed = points.to_vec();
        reversed.reverse();
        reversed
    } else {
        points.to_vec()
    };

    (result, diagnostics)
}

/// Simple flip without guide (always reverses).
pub fn flip_polyline_simple(points: &[[f64; 3]]) -> Vec<[f64; 3]> {
    let mut result = points.to_vec();
    result.reverse();
    result
}

// ============================================================================
// Extend Polyline
// ============================================================================

/// Options for extending polylines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExtendPolylineOptions {
    /// Extension length at the start (positive = extend, negative = trim).
    pub start_length: f64,
    /// Extension length at the end (positive = extend, negative = trim).
    pub end_length: f64,
}

impl ExtendPolylineOptions {
    /// Create new extend options.
    #[must_use]
    pub const fn new(start_length: f64, end_length: f64) -> Self {
        Self {
            start_length,
            end_length,
        }
    }

    /// Create options to extend/trim only the start.
    #[must_use]
    pub const fn start(length: f64) -> Self {
        Self {
            start_length: length,
            end_length: 0.0,
        }
    }

    /// Create options to extend/trim only the end.
    #[must_use]
    pub const fn end(length: f64) -> Self {
        Self {
            start_length: 0.0,
            end_length: length,
        }
    }
}

impl Default for ExtendPolylineOptions {
    fn default() -> Self {
        Self::new(0.0, 0.0)
    }
}

/// Diagnostics for extend operations.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExtendPolylineDiagnostics {
    /// Number of input points.
    pub input_point_count: usize,
    /// Number of output points.
    pub output_point_count: usize,
    /// Whether the start was modified.
    pub start_modified: bool,
    /// Whether the end was modified.
    pub end_modified: bool,
    /// Warnings generated during the operation.
    pub warnings: Vec<String>,
}

/// Extends or trims a polyline at its start and/or end.
///
/// Positive lengths extend the polyline along its tangent direction.
/// Negative lengths trim the polyline.
///
/// # Arguments
/// * `points` - The input polyline points.
/// * `options` - Extend options with start/end lengths.
/// * `tol` - Tolerance for geometry operations.
///
/// # Returns
/// A tuple of the extended/trimmed points and diagnostics.
pub fn extend_polyline(
    points: &[[f64; 3]],
    options: ExtendPolylineOptions,
    tol: Tolerance,
) -> (Vec<[f64; 3]>, ExtendPolylineDiagnostics) {
    let mut diagnostics = ExtendPolylineDiagnostics {
        input_point_count: points.len(),
        ..Default::default()
    };

    if points.len() < 2 {
        diagnostics.output_point_count = points.len();
        return (points.to_vec(), diagnostics);
    }

    let mut result = points.to_vec();

    // Handle start extension/trimming
    if options.start_length.abs() > tol.eps {
        diagnostics.start_modified = true;
        result = if options.start_length < 0.0 {
            // Trim from start
            trim_polyline_start(&result, -options.start_length, tol)
        } else {
            // Extend from start
            extend_polyline_start(&result, options.start_length, tol)
        };
    }

    // Handle end extension/trimming
    if result.len() >= 2 && options.end_length.abs() > tol.eps {
        diagnostics.end_modified = true;
        result = if options.end_length < 0.0 {
            // Trim from end
            trim_polyline_end(&result, -options.end_length, tol)
        } else {
            // Extend from end
            extend_polyline_end(&result, options.end_length, tol)
        };
    }

    // Remove duplicate consecutive points
    result = deduplicate_consecutive(result, tol.eps);

    diagnostics.output_point_count = result.len();
    (result, diagnostics)
}

fn extend_polyline_start(points: &[[f64; 3]], length: f64, tol: Tolerance) -> Vec<[f64; 3]> {
    if points.len() < 2 || length <= tol.eps {
        return points.to_vec();
    }

    let direction = sub(points[0], points[1]);
    let dir_len = vec_length(direction);
    if dir_len < tol.eps {
        return points.to_vec();
    }

    let dir_unit = scale(direction, 1.0 / dir_len);
    let new_start = add(points[0], scale(dir_unit, length));

    let mut result = Vec::with_capacity(points.len() + 1);
    result.push(new_start);
    result.extend_from_slice(points);
    result
}

fn extend_polyline_end(points: &[[f64; 3]], length: f64, tol: Tolerance) -> Vec<[f64; 3]> {
    if points.len() < 2 || length <= tol.eps {
        return points.to_vec();
    }

    let n = points.len();
    let direction = sub(points[n - 1], points[n - 2]);
    let dir_len = vec_length(direction);
    if dir_len < tol.eps {
        return points.to_vec();
    }

    let dir_unit = scale(direction, 1.0 / dir_len);
    let new_end = add(points[n - 1], scale(dir_unit, length));

    let mut result = points.to_vec();
    result.push(new_end);
    result
}

fn trim_polyline_start(points: &[[f64; 3]], trim_length: f64, tol: Tolerance) -> Vec<[f64; 3]> {
    if points.len() < 2 || trim_length <= tol.eps {
        return points.to_vec();
    }

    let total_length = polyline_length(points);
    if total_length <= tol.eps {
        return points.to_vec();
    }

    let trim = trim_length.min(total_length - tol.eps);
    if trim <= tol.eps {
        return points.to_vec();
    }

    // Walk along segments from start until we've trimmed enough
    let mut accumulated = 0.0;
    for i in 0..points.len() - 1 {
        let seg_len = distance(points[i], points[i + 1]);
        if accumulated + seg_len >= trim - tol.eps {
            // This segment contains the trim point
            let remaining = trim - accumulated;
            let t = if seg_len < tol.eps {
                0.0
            } else {
                (remaining / seg_len).clamp(0.0, 1.0)
            };
            let new_start = lerp(points[i], points[i + 1], t);
            let mut result = Vec::with_capacity(points.len() - i);
            result.push(new_start);
            result.extend_from_slice(&points[i + 1..]);
            return deduplicate_consecutive(result, tol.eps);
        }
        accumulated += seg_len;
    }

    // If we get here, trim the entire polyline (shouldn't happen with our guards)
    vec![*points.last().unwrap()]
}

fn trim_polyline_end(points: &[[f64; 3]], trim_length: f64, tol: Tolerance) -> Vec<[f64; 3]> {
    if points.len() < 2 || trim_length <= tol.eps {
        return points.to_vec();
    }

    let total_length = polyline_length(points);
    if total_length <= tol.eps {
        return points.to_vec();
    }

    let trim = trim_length.min(total_length - tol.eps);
    if trim <= tol.eps {
        return points.to_vec();
    }

    // Walk along segments from end until we've trimmed enough
    let mut accumulated = 0.0;
    for i in (1..points.len()).rev() {
        let seg_len = distance(points[i - 1], points[i]);
        if accumulated + seg_len >= trim - tol.eps {
            // This segment contains the trim point
            let remaining = trim - accumulated;
            let t = if seg_len < tol.eps {
                1.0
            } else {
                1.0 - (remaining / seg_len).clamp(0.0, 1.0)
            };
            let new_end = lerp(points[i - 1], points[i], t);
            let mut result = Vec::with_capacity(i + 1);
            result.extend_from_slice(&points[..i]);
            result.push(new_end);
            return deduplicate_consecutive(result, tol.eps);
        }
        accumulated += seg_len;
    }

    // If we get here, trim the entire polyline
    vec![points[0]]
}

// ============================================================================
// Smooth Polyline
// ============================================================================

/// Options for polyline smoothing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SmoothPolylineOptions {
    /// Smoothing strength (0.0 = no smoothing, 1.0 = full smoothing towards neighbors).
    pub strength: f64,
    /// Number of smoothing iterations.
    pub iterations: usize,
}

impl SmoothPolylineOptions {
    /// Create new smoothing options.
    #[must_use]
    pub const fn new(strength: f64, iterations: usize) -> Self {
        Self { strength, iterations }
    }
}

impl Default for SmoothPolylineOptions {
    fn default() -> Self {
        Self::new(0.5, 1)
    }
}

/// Diagnostics for smoothing operations.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SmoothPolylineDiagnostics {
    /// Number of input points.
    pub input_point_count: usize,
    /// Number of output points.
    pub output_point_count: usize,
    /// Number of smoothing iterations applied.
    pub iterations_applied: usize,
}

/// Smooths a polyline by moving interior points towards the midpoint of neighbors.
///
/// Endpoint vertices remain fixed.
///
/// # Arguments
/// * `points` - The input polyline points.
/// * `options` - Smoothing options.
///
/// # Returns
/// A tuple of the smoothed points and diagnostics.
pub fn smooth_polyline(
    points: &[[f64; 3]],
    options: SmoothPolylineOptions,
) -> (Vec<[f64; 3]>, SmoothPolylineDiagnostics) {
    let mut diagnostics = SmoothPolylineDiagnostics {
        input_point_count: points.len(),
        ..Default::default()
    };

    let strength = options.strength.clamp(0.0, 1.0);
    if points.len() <= 2 || options.iterations == 0 || strength < 1e-12 {
        diagnostics.output_point_count = points.len();
        return (points.to_vec(), diagnostics);
    }

    let mut result = points.to_vec();
    for _ in 0..options.iterations {
        let mut next = Vec::with_capacity(result.len());
        next.push(result[0]);
        for window in result.windows(3) {
            let prev = window[0];
            let current = window[1];
            let next_pt = window[2];
            let target = scale(add(prev, next_pt), 0.5);
            next.push(lerp(current, target, strength));
        }
        next.push(*result.last().unwrap());
        result = next;
        diagnostics.iterations_applied += 1;
    }

    diagnostics.output_point_count = result.len();
    (result, diagnostics)
}

// ============================================================================
// Simplify Polyline (RDP Algorithm)
// ============================================================================

/// Options for polyline simplification.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimplifyPolylineOptions {
    /// Distance tolerance for point removal.
    pub tolerance: f64,
    /// Angle tolerance in radians (additional filtering).
    pub angle_tolerance: f64,
}

impl SimplifyPolylineOptions {
    /// Create new simplify options.
    #[must_use]
    pub const fn new(tolerance: f64) -> Self {
        Self {
            tolerance,
            angle_tolerance: 0.0,
        }
    }

    /// Set angle tolerance.
    #[must_use]
    pub const fn with_angle_tolerance(mut self, angle: f64) -> Self {
        self.angle_tolerance = angle;
        self
    }
}

impl Default for SimplifyPolylineOptions {
    fn default() -> Self {
        Self::new(0.01)
    }
}

/// Diagnostics for simplification operations.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SimplifyPolylineDiagnostics {
    /// Number of input points.
    pub input_point_count: usize,
    /// Number of output points.
    pub output_point_count: usize,
    /// Number of points removed.
    pub points_removed: usize,
}

/// Simplifies a polyline using the Ramer-Douglas-Peucker algorithm.
///
/// # Arguments
/// * `points` - The input polyline points.
/// * `options` - Simplification options.
///
/// # Returns
/// A tuple of the simplified points and diagnostics.
pub fn simplify_polyline(
    points: &[[f64; 3]],
    options: SimplifyPolylineOptions,
) -> (Vec<[f64; 3]>, SimplifyPolylineDiagnostics) {
    let mut diagnostics = SimplifyPolylineDiagnostics {
        input_point_count: points.len(),
        ..Default::default()
    };

    if points.len() <= 2 {
        diagnostics.output_point_count = points.len();
        return (points.to_vec(), diagnostics);
    }

    let tolerance = options.tolerance.max(0.0);
    let mut mask = vec![false; points.len()];
    mask[0] = true;
    mask[points.len() - 1] = true;

    rdp_recursive(points, tolerance, 0, points.len() - 1, &mut mask);

    let mut simplified = Vec::new();
    for (index, point) in points.iter().enumerate() {
        if mask[index] {
            simplified.push(*point);
        }
    }

    if simplified.len() < 2 {
        simplified = vec![points[0], points[points.len() - 1]];
    }

    diagnostics.output_point_count = simplified.len();
    diagnostics.points_removed = diagnostics.input_point_count.saturating_sub(diagnostics.output_point_count);
    (simplified, diagnostics)
}

fn rdp_recursive(points: &[[f64; 3]], tolerance: f64, start: usize, end: usize, mask: &mut [bool]) {
    if end <= start + 1 {
        return;
    }

    let segment_start = points[start];
    let segment_end = points[end];
    let mut max_idx = 0;
    let mut max_distance = -1.0;

    for i in start + 1..end {
        let d = point_segment_distance(points[i], segment_start, segment_end);
        if d > max_distance {
            max_distance = d;
            max_idx = i;
        }
    }

    if max_distance > tolerance {
        mask[max_idx] = true;
        rdp_recursive(points, tolerance, start, max_idx, mask);
        rdp_recursive(points, tolerance, max_idx, end, mask);
    }
}

fn point_segment_distance(point: [f64; 3], a: [f64; 3], b: [f64; 3]) -> f64 {
    let ab = sub(b, a);
    let ap = sub(point, a);
    let ab_len_sq = dot(ab, ab);
    if ab_len_sq <= 1e-12 {
        return length(ap);
    }
    let t = (dot(ap, ab) / ab_len_sq).clamp(0.0, 1.0);
    let projection = add(a, scale(ab, t));
    length(sub(point, projection))
}

// ============================================================================
// Resample Polyline
// ============================================================================

/// Options for polyline resampling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResamplePolylineOptions {
    /// Target number of output points.
    pub count: usize,
}

impl ResamplePolylineOptions {
    /// Create new resample options.
    #[must_use]
    pub const fn new(count: usize) -> Self {
        Self { count }
    }
}

impl Default for ResamplePolylineOptions {
    fn default() -> Self {
        Self::new(10)
    }
}

/// Diagnostics for resampling operations.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResamplePolylineDiagnostics {
    /// Number of input points.
    pub input_point_count: usize,
    /// Number of output points.
    pub output_point_count: usize,
    /// Total arc length of the polyline.
    pub total_length: f64,
}

/// Resamples a polyline to a target number of uniformly distributed points.
///
/// # Arguments
/// * `points` - The input polyline points.
/// * `options` - Resampling options.
///
/// # Returns
/// A tuple of the resampled points and diagnostics.
pub fn resample_polyline(
    points: &[[f64; 3]],
    options: ResamplePolylineOptions,
) -> (Vec<[f64; 3]>, ResamplePolylineDiagnostics) {
    let mut diagnostics = ResamplePolylineDiagnostics {
        input_point_count: points.len(),
        ..Default::default()
    };

    let count = options.count.max(2);
    if points.len() < 2 || count <= 2 {
        let result = if points.len() >= 2 {
            vec![points[0], *points.last().unwrap()]
        } else if !points.is_empty() {
            vec![points[0], points[0]]
        } else {
            Vec::new()
        };
        diagnostics.output_point_count = result.len();
        return (result, diagnostics);
    }

    let total_length = polyline_length(points);
    diagnostics.total_length = total_length;

    if total_length < 1e-12 {
        let result = vec![points[0]; count];
        diagnostics.output_point_count = count;
        return (result, diagnostics);
    }

    let mut samples = Vec::with_capacity(count);
    samples.push(points[0]);

    // Build segment data
    let mut segments = Vec::with_capacity(points.len() - 1);
    for pair in points.windows(2) {
        let len = distance(pair[0], pair[1]);
        segments.push((pair[0], pair[1], len));
    }

    let mut accumulated = 0.0;
    let mut seg_idx = 0;
    let mut seg_progress = 0.0;

    for step in 1..count - 1 {
        let target_length = (step as f64 / (count as f64 - 1.0)) * total_length;
        
        // Advance to the correct segment
        while seg_idx < segments.len() && accumulated + segments[seg_idx].2 < target_length {
            accumulated += segments[seg_idx].2;
            seg_idx += 1;
            seg_progress = 0.0;
        }

        if seg_idx >= segments.len() {
            samples.push(*points.last().unwrap());
            continue;
        }

        let (seg_start, seg_end, seg_len) = segments[seg_idx];
        let remaining = target_length - accumulated;
        let t = if seg_len < 1e-12 {
            0.0
        } else {
            (seg_progress + remaining) / seg_len
        };
        let t = t.clamp(0.0, 1.0);
        samples.push(lerp(seg_start, seg_end, t));
        seg_progress += remaining;
    }

    samples.push(*points.last().unwrap());
    diagnostics.output_point_count = samples.len();
    (samples, diagnostics)
}

// ============================================================================
// Remesh Polyline
// ============================================================================

/// Options for polyline remeshing (segment length control).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RemeshPolylineOptions {
    /// Minimum allowed segment length; shorter segments are skipped.
    pub min_edge: f64,
    /// Maximum allowed segment length; longer segments are subdivided.
    pub max_edge: f64,
}

impl RemeshPolylineOptions {
    /// Create new remesh options with specified edge length bounds.
    #[must_use]
    pub const fn new(min_edge: f64, max_edge: f64) -> Self {
        Self { min_edge, max_edge }
    }

    /// Create remesh options with only min edge constraint.
    #[must_use]
    pub const fn with_min_edge(min_edge: f64) -> Self {
        Self {
            min_edge,
            max_edge: f64::INFINITY,
        }
    }

    /// Create remesh options with only max edge constraint.
    #[must_use]
    pub const fn with_max_edge(max_edge: f64) -> Self {
        Self {
            min_edge: 0.0,
            max_edge,
        }
    }
}

impl Default for RemeshPolylineOptions {
    fn default() -> Self {
        Self::new(0.0, f64::INFINITY)
    }
}

/// Diagnostics for remesh operations.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RemeshPolylineDiagnostics {
    /// Number of input points.
    pub input_point_count: usize,
    /// Number of output points.
    pub output_point_count: usize,
    /// Number of segments removed (too short).
    pub segments_removed: usize,
    /// Number of segments subdivided (too long).
    pub segments_subdivided: usize,
}

/// Remesh a polyline to ensure segment lengths are within bounds.
///
/// This operation:
/// - Skips segments shorter than `min_edge`
/// - Subdivides segments longer than `max_edge`
///
/// # Arguments
/// * `points` - The input polyline points.
/// * `options` - Remesh options specifying min/max edge lengths.
///
/// # Returns
/// A tuple of the remeshed points and diagnostics.
pub fn remesh_polyline(
    points: &[[f64; 3]],
    options: RemeshPolylineOptions,
) -> (Vec<[f64; 3]>, RemeshPolylineDiagnostics) {
    let mut diagnostics = RemeshPolylineDiagnostics {
        input_point_count: points.len(),
        ..Default::default()
    };

    if points.len() <= 2 {
        diagnostics.output_point_count = points.len();
        return (points.to_vec(), diagnostics);
    }

    let min_edge = options.min_edge.max(0.0);
    let max_edge = if options.max_edge.is_finite() && options.max_edge > min_edge {
        options.max_edge
    } else {
        f64::INFINITY
    };

    let mut result = Vec::new();
    result.push(points[0]);

    for pair in points.windows(2) {
        let start = pair[0];
        let end = pair[1];
        let segment = sub(end, start);
        let seg_len = length(segment);

        // Skip very short segments
        if seg_len < min_edge {
            diagnostics.segments_removed += 1;
            continue;
        }

        // Subdivide long segments
        if seg_len > max_edge && max_edge > 0.0 {
            diagnostics.segments_subdivided += 1;
            let steps = (seg_len / max_edge).ceil().max(1.0) as usize;
            for step in 1..=steps {
                let t = step as f64 / steps as f64;
                let point = lerp(start, end, t);
                // Only add if subdivided segment is long enough
                if step == steps {
                    result.push(point);
                } else if seg_len / steps as f64 >= min_edge {
                    result.push(point);
                }
            }
        } else {
            result.push(end);
        }
    }

    // Ensure we have at least 2 points
    if result.len() < 2 {
        diagnostics.output_point_count = points.len();
        return (points.to_vec(), diagnostics);
    }

    diagnostics.output_point_count = result.len();
    (result, diagnostics)
}

// ============================================================================
// Collapse Polyline
// ============================================================================

/// Options for polyline collapse (short segment removal).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollapsePolylineOptions {
    /// Minimum segment length; shorter segments are collapsed.
    pub tolerance: f64,
}

impl CollapsePolylineOptions {
    /// Create new collapse options.
    #[must_use]
    pub const fn new(tolerance: f64) -> Self {
        Self { tolerance }
    }
}

impl Default for CollapsePolylineOptions {
    fn default() -> Self {
        Self::new(0.01)
    }
}

/// Diagnostics for collapse operations.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CollapsePolylineDiagnostics {
    /// Number of input points.
    pub input_point_count: usize,
    /// Number of output points.
    pub output_point_count: usize,
    /// Number of points removed.
    pub points_removed: usize,
}

/// Collapses short segments in a polyline by removing consecutive points
/// that are closer than the tolerance.
///
/// # Arguments
/// * `points` - The input polyline points.
/// * `options` - Collapse options.
///
/// # Returns
/// A tuple of the collapsed points and diagnostics.
pub fn collapse_polyline(
    points: &[[f64; 3]],
    options: CollapsePolylineOptions,
) -> (Vec<[f64; 3]>, CollapsePolylineDiagnostics) {
    let mut diagnostics = CollapsePolylineDiagnostics {
        input_point_count: points.len(),
        ..Default::default()
    };

    if points.len() <= 2 {
        diagnostics.output_point_count = points.len();
        return (points.to_vec(), diagnostics);
    }

    let tolerance = options.tolerance.max(0.0);
    let mut result = Vec::with_capacity(points.len());
    result.push(points[0]);

    for pair in points.windows(2) {
        if distance(pair[0], pair[1]) < tolerance {
            diagnostics.points_removed += 1;
            continue;
        }
        result.push(pair[1]);
    }

    if result.len() < 2 {
        result.push(*points.last().unwrap());
    }

    diagnostics.output_point_count = result.len();
    (result, diagnostics)
}

// ============================================================================
// Rotate Polyline Seam
// ============================================================================

/// Options for rotating a closed polyline's seam point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RotateSeamOptions {
    /// Parameter (0.0-1.0) indicating where to place the new seam.
    pub parameter: f64,
}

impl RotateSeamOptions {
    /// Create new rotate seam options.
    #[must_use]
    pub const fn new(parameter: f64) -> Self {
        Self { parameter }
    }
}

impl Default for RotateSeamOptions {
    fn default() -> Self {
        Self::new(0.0)
    }
}

/// Diagnostics for seam rotation operations.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RotateSeamDiagnostics {
    /// Number of input points.
    pub input_point_count: usize,
    /// Number of output points.
    pub output_point_count: usize,
    /// Whether the input was treated as closed.
    pub was_closed: bool,
    /// The normalized parameter used.
    pub effective_parameter: f64,
}

/// Rotates the seam (start/end point) of a closed polyline.
///
/// For open polylines, returns the input unchanged.
///
/// # Arguments
/// * `points` - The input polyline points.
/// * `options` - Seam rotation options.
/// * `tol` - Tolerance for closedness check.
///
/// # Returns
/// A tuple of the rotated points and diagnostics.
pub fn rotate_polyline_seam(
    points: &[[f64; 3]],
    options: RotateSeamOptions,
    tol: Tolerance,
) -> (Vec<[f64; 3]>, RotateSeamDiagnostics) {
    let mut diagnostics = RotateSeamDiagnostics {
        input_point_count: points.len(),
        ..Default::default()
    };

    if points.len() < 3 {
        diagnostics.output_point_count = points.len();
        return (points.to_vec(), diagnostics);
    }

    let is_closed = is_polyline_closed(points, tol.eps);
    diagnostics.was_closed = is_closed;

    if !is_closed {
        diagnostics.output_point_count = points.len();
        return (points.to_vec(), diagnostics);
    }

    let total_length = polyline_length(points);
    if total_length < tol.eps {
        diagnostics.output_point_count = points.len();
        return (points.to_vec(), diagnostics);
    }

    let normalized = if options.parameter.is_finite() {
        options.parameter.rem_euclid(1.0)
    } else {
        0.0
    };
    diagnostics.effective_parameter = normalized;

    let target = normalized * total_length;
    let mut accumulated = 0.0;

    for i in 0..points.len() - 1 {
        let seg_len = distance(points[i], points[i + 1]);
        if accumulated + seg_len >= target || i == points.len() - 2 {
            let local = if seg_len < tol.eps {
                0.0
            } else {
                ((target - accumulated).max(0.0) / seg_len).clamp(0.0, 1.0)
            };
            let seam_point = lerp(points[i], points[i + 1], local);
            
            let mut result = Vec::with_capacity(points.len() + 1);
            result.push(seam_point);
            result.extend(points.iter().skip(i + 1).copied());
            result.extend(points.iter().take(i + 1).copied());
            result.push(seam_point);
            
            let deduped = deduplicate_consecutive(result, tol.eps);
            diagnostics.output_point_count = deduped.len();
            return (deduped, diagnostics);
        }
        accumulated += seg_len;
    }

    diagnostics.output_point_count = points.len();
    (points.to_vec(), diagnostics)
}

// ============================================================================
// Project Polyline
// ============================================================================

/// Options for projecting a polyline onto a plane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProjectPolylineOptions {
    /// Projection direction (normalized internally).
    pub direction: [f64; 3],
}

impl ProjectPolylineOptions {
    /// Create new projection options.
    #[must_use]
    pub const fn new(direction: [f64; 3]) -> Self {
        Self { direction }
    }

    /// Project along Z axis.
    #[must_use]
    pub const fn z() -> Self {
        Self::new([0.0, 0.0, 1.0])
    }
}

impl Default for ProjectPolylineOptions {
    fn default() -> Self {
        Self::z()
    }
}

/// Diagnostics for projection operations.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProjectPolylineDiagnostics {
    /// Number of input points.
    pub input_point_count: usize,
    /// Number of output points.
    pub output_point_count: usize,
    /// The projection direction used (normalized).
    pub direction: [f64; 3],
}

/// Projects a polyline onto a plane by removing the component along the given direction.
///
/// The plane passes through the first point of the polyline.
///
/// # Arguments
/// * `points` - The input polyline points.
/// * `options` - Projection options.
///
/// # Returns
/// A tuple of the projected points and diagnostics.
pub fn project_polyline(
    points: &[[f64; 3]],
    options: ProjectPolylineOptions,
) -> (Vec<[f64; 3]>, ProjectPolylineDiagnostics) {
    let mut diagnostics = ProjectPolylineDiagnostics {
        input_point_count: points.len(),
        ..Default::default()
    };

    if points.is_empty() {
        return (Vec::new(), diagnostics);
    }

    let axis = normalize(options.direction);
    diagnostics.direction = axis;

    if length(axis) < 1e-12 {
        // Invalid direction, return unchanged
        diagnostics.output_point_count = points.len();
        return (points.to_vec(), diagnostics);
    }

    let origin = points[0];
    let result: Vec<[f64; 3]> = points
        .iter()
        .map(|point| {
            let relative = sub(*point, origin);
            let dist = dot(relative, axis);
            sub(*point, scale(axis, dist))
        })
        .collect();

    diagnostics.output_point_count = result.len();
    (result, diagnostics)
}

// ============================================================================
// Sample Polyline
// ============================================================================

/// A sample point on a polyline with position and tangent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolylineSample {
    /// The sampled position.
    pub point: [f64; 3],
    /// The tangent direction at the sample point.
    pub tangent: [f64; 3],
    /// The normalized parameter (0.0-1.0).
    pub parameter: f64,
}

/// Samples a polyline at a normalized parameter (0.0 to 1.0).
///
/// # Arguments
/// * `points` - The polyline points.
/// * `parameter` - Normalized parameter (0.0 = start, 1.0 = end).
///
/// # Returns
/// A `PolylineSample` with the position and tangent at the given parameter.
pub fn sample_polyline_at(points: &[[f64; 3]], parameter: f64) -> PolylineSample {
    if points.len() < 2 {
        return PolylineSample {
            point: points.first().copied().unwrap_or([0.0, 0.0, 0.0]),
            tangent: [1.0, 0.0, 0.0],
            parameter: parameter.clamp(0.0, 1.0),
        };
    }

    let clamped = parameter.clamp(0.0, 1.0);
    let total_length = polyline_length(points);
    
    if total_length < 1e-12 {
        return PolylineSample {
            point: points[0],
            tangent: sub(points[1], points[0]),
            parameter: clamped,
        };
    }

    let target = clamped * total_length;
    let mut accumulated = 0.0;

    for i in 0..points.len() - 1 {
        let seg_len = distance(points[i], points[i + 1]);
        if accumulated + seg_len >= target {
            let local = if seg_len < 1e-12 {
                0.0
            } else {
                ((target - accumulated).max(0.0) / seg_len).clamp(0.0, 1.0)
            };
            let point = lerp(points[i], points[i + 1], local);
            let tangent = sub(points[i + 1], points[i]);
            return PolylineSample { point, tangent, parameter: clamped };
        }
        accumulated += seg_len;
    }

    PolylineSample {
        point: *points.last().unwrap(),
        tangent: sub(points[points.len() - 1], points[points.len() - 2]),
        parameter: clamped,
    }
}

// ============================================================================
// Fillet Polyline at Parameter
// ============================================================================

/// Options for filleting a polyline at a specific parameter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FilletAtParameterOptions {
    /// The parameter (0.0-1.0) indicating which corner to fillet.
    pub parameter: f64,
    /// The fillet radius.
    pub radius: f64,
}

impl FilletAtParameterOptions {
    /// Create new fillet-at-parameter options.
    #[must_use]
    pub const fn new(parameter: f64, radius: f64) -> Self {
        Self { parameter, radius }
    }
}

/// Diagnostics for fillet-at-parameter operations.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FilletAtParameterDiagnostics {
    /// Number of input points.
    pub input_point_count: usize,
    /// Number of output points.
    pub output_point_count: usize,
    /// The corner index that was filleted.
    pub corner_index: usize,
    /// The actual parameter used.
    pub actual_parameter: f64,
    /// Whether the fillet was successfully applied.
    pub fillet_applied: bool,
}

/// Fillets a single corner of a polyline at a specific parameter.
///
/// # Arguments
/// * `points` - The input polyline points.
/// * `options` - Fillet options.
/// * `tol` - Tolerance for geometry operations.
///
/// # Returns
/// A tuple of the filleted points and diagnostics.
pub fn fillet_polyline_at_parameter(
    points: &[[f64; 3]],
    options: FilletAtParameterOptions,
    tol: Tolerance,
) -> (Vec<[f64; 3]>, FilletAtParameterDiagnostics) {
    let mut diagnostics = FilletAtParameterDiagnostics {
        input_point_count: points.len(),
        ..Default::default()
    };

    if points.len() < 3 || options.radius <= tol.eps {
        diagnostics.output_point_count = points.len();
        diagnostics.actual_parameter = options.parameter.clamp(0.0, 1.0);
        return (points.to_vec(), diagnostics);
    }

    let clamped = options.parameter.clamp(0.0, 1.0);
    let segments = points.len().saturating_sub(1) as f64;
    let mut index = (clamped * segments).round() as isize;
    index = index.clamp(1, points.len() as isize - 2);
    let index = index as usize;
    
    diagnostics.corner_index = index;
    diagnostics.actual_parameter = index as f64 / segments;

    let prev = points[index - 1];
    let current = points[index];
    let next = points[index + 1];

    if let Some((start, mid, end)) = fillet_corner(prev, current, next, options.radius, tol) {
        let mut result = Vec::with_capacity(points.len() + 2);
        result.extend_from_slice(&points[..index]);
        
        if distance(*result.last().unwrap_or(&[0.0, 0.0, 0.0]), start) > tol.eps {
            result.push(start);
        }
        result.push(mid);
        result.push(end);
        result.extend_from_slice(&points[index + 1..]);
        
        let deduped = deduplicate_consecutive(result, tol.eps);
        diagnostics.output_point_count = deduped.len();
        diagnostics.fillet_applied = true;
        (deduped, diagnostics)
    } else {
        diagnostics.output_point_count = points.len();
        (points.to_vec(), diagnostics)
    }
}

fn fillet_corner(
    prev: [f64; 3],
    current: [f64; 3],
    next: [f64; 3],
    radius: f64,
    tol: Tolerance,
) -> Option<([f64; 3], [f64; 3], [f64; 3])> {
    if radius <= tol.eps {
        return None;
    }

    let to_prev = sub(current, prev);
    let to_next = sub(next, current);
    let len_prev = length(to_prev);
    let len_next = length(to_next);
    
    if len_prev < tol.eps || len_next < tol.eps {
        return None;
    }

    let trim = radius.min(len_prev / 2.0).min(len_next / 2.0);
    if trim <= tol.eps {
        return None;
    }

    let dir_prev = scale(to_prev, 1.0 / len_prev);
    let dir_next = scale(to_next, 1.0 / len_next);
    let start = sub(current, scale(dir_prev, trim));
    let end = add(current, scale(dir_next, trim));
    let mid = lerp(start, end, 0.5);
    
    Some((start, mid, end))
}

// ============================================================================
// Compute Perpendicular Frames
// ============================================================================

/// A frame (position + axes) on a polyline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolylineFrame {
    /// Origin point of the frame.
    pub origin: [f64; 3],
    /// Tangent (X axis) direction.
    pub tangent: [f64; 3],
    /// Normal (Y axis) direction.
    pub normal: [f64; 3],
    /// Binormal (Z axis) direction.
    pub binormal: [f64; 3],
    /// Parameter along the polyline.
    pub parameter: f64,
}

/// Options for computing perpendicular frames.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerpFramesOptions {
    /// Number of frame segments.
    pub segments: usize,
    /// Whether to align frames to reduce twist.
    pub align: bool,
}

impl PerpFramesOptions {
    /// Create new perp frames options.
    #[must_use]
    pub const fn new(segments: usize, align: bool) -> Self {
        Self { segments, align }
    }
}

impl Default for PerpFramesOptions {
    fn default() -> Self {
        Self::new(10, false)
    }
}

/// Diagnostics for perpendicular frame computation.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PerpFramesDiagnostics {
    /// Number of frames computed.
    pub frame_count: usize,
}

/// Computes perpendicular frames along a polyline.
///
/// # Arguments
/// * `points` - The polyline points.
/// * `options` - Frame computation options.
///
/// # Returns
/// A tuple of the frames and diagnostics.
pub fn compute_perp_frames(
    points: &[[f64; 3]],
    options: PerpFramesOptions,
) -> (Vec<PolylineFrame>, PerpFramesDiagnostics) {
    let mut diagnostics = PerpFramesDiagnostics::default();
    let segments = options.segments.max(1);

    let mut frames = Vec::with_capacity(segments + 1);
    let mut previous_axes: Option<([f64; 3], [f64; 3])> = None;

    for step in 0..=segments {
        let parameter = step as f64 / segments as f64;
        let sample = sample_polyline_at(points, parameter);
        let tangent = normalize(sample.tangent);
        
        // Compute initial normal (perpendicular to tangent)
        let mut normal = normalize(cross([0.0, 0.0, 1.0], tangent));
        if length(normal) < 1e-12 {
            normal = normalize(cross([1.0, 0.0, 0.0], tangent));
        }
        if length(normal) < 1e-12 {
            normal = [0.0, 1.0, 0.0];
        }
        
        let mut binormal = normalize(cross(tangent, normal));

        // Align frames if requested
        if options.align {
            if let Some((prev_normal, prev_binormal)) = previous_axes {
                if dot(normal, prev_normal) < 0.0 {
                    normal = scale(normal, -1.0);
                }
                if dot(binormal, prev_binormal) < 0.0 {
                    binormal = scale(binormal, -1.0);
                }
            }
            previous_axes = Some((normal, binormal));
        }

        frames.push(PolylineFrame {
            origin: sample.point,
            tangent,
            normal,
            binormal,
            parameter,
        });
    }

    diagnostics.frame_count = frames.len();
    (frames, diagnostics)
}

// ============================================================================
// Utility Functions
// ============================================================================

fn is_polyline_closed(points: &[[f64; 3]], tolerance: f64) -> bool {
    if points.len() < 3 {
        return false;
    }
    distance(points[0], *points.last().unwrap()) < tolerance
}

fn polyline_length(points: &[[f64; 3]]) -> f64 {
    points
        .windows(2)
        .map(|w| distance(w[0], w[1]))
        .sum()
}

fn deduplicate_consecutive(points: Vec<[f64; 3]>, tolerance: f64) -> Vec<[f64; 3]> {
    let mut result = Vec::with_capacity(points.len());
    for point in points {
        if result.last().map_or(true, |prev| distance(*prev, point) > tolerance) {
            result.push(point);
        }
    }
    result
}

// Vector math helpers (using [f64; 3] for consistency with component layer)
fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(v: [f64; 3], s: f64) -> [f64; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn vec_length(v: [f64; 3]) -> f64 {
    length(v)
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = length(v);
    if len < 1e-12 {
        [0.0, 0.0, 0.0]
    } else {
        scale(v, 1.0 / len)
    }
}

fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    length(sub(a, b))
}

fn lerp(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    fn approx_eq_point(a: [f64; 3], b: [f64; 3]) -> bool {
        approx_eq(a[0], b[0]) && approx_eq(a[1], b[1]) && approx_eq(a[2], b[2])
    }

    #[test]
    fn test_offset_polyline_simple() {
        let points = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let options = OffsetPolylineOptions::new(0.1);
        let (result, diag) = offset_polyline(&points, options, Tolerance::default_geom()).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(diag.input_point_count, 3);
        assert_eq!(diag.output_point_count, 3);
        // First point should be offset in Y direction (left of first segment)
        assert!(result[0][1] > 0.0);
    }

    #[test]
    fn test_offset_polyline_zero_distance() {
        let points = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let options = OffsetPolylineOptions::new(0.0);
        let (result, _) = offset_polyline(&points, options, Tolerance::default_geom()).unwrap();
        assert_eq!(result, points);
    }

    #[test]
    fn test_join_polylines_simple() {
        let p1 = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let p2 = vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let (result, diag) = join_polylines(vec![p1, p2], JoinPolylinesOptions::new());
        assert_eq!(result.len(), 1);
        assert_eq!(diag.join_count, 1);
        assert_eq!(result[0].len(), 3);
    }

    #[test]
    fn test_join_polylines_reverse() {
        let p1 = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let p2 = vec![[2.0, 0.0, 0.0], [1.0, 0.0, 0.0]]; // Reversed
        let (result, diag) = join_polylines(vec![p1, p2], JoinPolylinesOptions::new());
        assert_eq!(result.len(), 1);
        assert_eq!(diag.join_count, 1);
    }

    #[test]
    fn test_join_polylines_preserve_direction() {
        let p1 = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let p2 = vec![[2.0, 0.0, 0.0], [1.0, 0.0, 0.0]]; // Reversed
        let options = JoinPolylinesOptions::new().preserve_direction(true);
        let (result, _) = join_polylines(vec![p1, p2], options);
        // Should not join since directions don't match
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_flip_polyline_simple() {
        let points = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let (result, diag) = flip_polyline(&points, None, FlipPolylineOptions::new());
        assert!(diag.was_flipped);
        assert_eq!(result[0], [2.0, 0.0, 0.0]);
        assert_eq!(result[2], [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_flip_polyline_with_guide() {
        let points = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let guide = vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0]];
        // Start (0,0,0) is closer to guide start (2,0,0) than guide end (3,0,0)
        let (result, diag) = flip_polyline(&points, Some(&guide), FlipPolylineOptions::new());
        assert!(!diag.was_flipped);
        assert_eq!(result, points);
    }

    #[test]
    fn test_extend_polyline_end() {
        let points = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let options = ExtendPolylineOptions::end(0.5);
        let (result, diag) = extend_polyline(&points, options, Tolerance::default_geom());
        assert!(diag.end_modified);
        assert!(!diag.start_modified);
        assert_eq!(result.len(), 3);
        assert!(approx_eq_point(result[2], [1.5, 0.0, 0.0]));
    }

    #[test]
    fn test_extend_polyline_start() {
        let points = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let options = ExtendPolylineOptions::start(0.5);
        let (result, diag) = extend_polyline(&points, options, Tolerance::default_geom());
        assert!(diag.start_modified);
        assert!(!diag.end_modified);
        assert_eq!(result.len(), 3);
        assert!(approx_eq_point(result[0], [-0.5, 0.0, 0.0]));
    }

    #[test]
    fn test_trim_polyline_end() {
        let points = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let options = ExtendPolylineOptions::end(-0.5);
        let (result, diag) = extend_polyline(&points, options, Tolerance::default_geom());
        assert!(diag.end_modified);
        // Should trim 0.5 from end
        let last = *result.last().unwrap();
        assert!(approx_eq_point(last, [1.5, 0.0, 0.0]));
    }

    #[test]
    fn test_trim_polyline_start() {
        let points = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let options = ExtendPolylineOptions::start(-0.5);
        let (result, diag) = extend_polyline(&points, options, Tolerance::default_geom());
        assert!(diag.start_modified);
        // Should trim 0.5 from start
        assert!(approx_eq_point(result[0], [0.5, 0.0, 0.0]));
    }
}
