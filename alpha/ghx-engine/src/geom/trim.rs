//! Trimming loop and region structures for surface parameter-space operations.
//!
//! This module provides types and functions for working with trim curves in UV parameter space.
//! Trim loops define boundaries (outer loops) and holes (inner loops) on surfaces.
//!
//! # Main Types
//! - [`TrimLoop`]: A closed loop of UV points representing a trim boundary
//! - [`TrimRegion`]: An outer loop with optional holes
//! - [`TrimError`]: Typed errors for trim operations
//! - [`TrimDiagnostics`]: Statistics and warnings from trim operations
//!
//! # Example
//! ```ignore
//! use crate::geom::{TrimLoop, TrimRegion, UvPoint, Tolerance};
//!
//! let tol = Tolerance::new(1e-9);
//! let outer = TrimLoop::new(vec![
//!     UvPoint::new(0.0, 0.0),
//!     UvPoint::new(1.0, 0.0),
//!     UvPoint::new(1.0, 1.0),
//!     UvPoint::new(0.0, 1.0),
//! ], tol)?;
//!
//! let region = TrimRegion::from_loops(vec![outer], tol)?;
//! assert!(region.contains(UvPoint::new(0.5, 0.5), tol));
//! ```

use super::core::Tolerance;
use super::curve::{Curve3, tessellate_curve_uniform};
use std::fmt;

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur during trim operations.
#[derive(Debug, Clone, PartialEq)]
pub enum TrimError {
    /// Points contain NaN or infinite values.
    NonFinitePoints,
    /// Loop has fewer than 3 distinct points after cleaning.
    InsufficientPoints { count: usize },
    /// Loop edges cross each other.
    SelfIntersection,
    /// Trim curve is not closed.
    CurveNotClosed,
    /// No loops provided when at least one is required.
    EmptyLoopSet,
    /// A hole loop intersects the outer boundary.
    HoleIntersectsOuter,
    /// A hole is not fully contained within the outer loop.
    HoleOutsideBoundary,
    /// Two hole loops intersect each other.
    HolesIntersect,
    /// Nested holes are not supported (hole inside another hole).
    NestedHoles,
    /// Loop is outside the valid surface domain.
    OutsideDomain {
        loop_min_u: f64,
        loop_max_u: f64,
        loop_min_v: f64,
        loop_max_v: f64,
        domain_min_u: f64,
        domain_max_u: f64,
        domain_min_v: f64,
        domain_max_v: f64,
    },
    /// Generic error with a message (for backwards compatibility).
    Other(String),
}

impl fmt::Display for TrimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinitePoints => write!(f, "trim loop points must be finite"),
            Self::InsufficientPoints { count } => {
                write!(f, "trim loop requires at least 3 points, got {}", count)
            }
            Self::SelfIntersection => write!(f, "trim loop self-intersects"),
            Self::CurveNotClosed => write!(f, "trim curve must be closed"),
            Self::EmptyLoopSet => write!(f, "trim region requires at least one loop"),
            Self::HoleIntersectsOuter => write!(f, "trim hole intersects outer loop"),
            Self::HoleOutsideBoundary => write!(f, "trim hole is not inside outer loop"),
            Self::HolesIntersect => write!(f, "trim holes intersect"),
            Self::NestedHoles => write!(f, "nested trim holes are not supported"),
            Self::OutsideDomain {
                loop_min_u,
                loop_max_u,
                loop_min_v,
                loop_max_v,
                domain_min_u,
                domain_max_u,
                domain_min_v,
                domain_max_v,
            } => {
                write!(
                    f,
                    "trim loop bounds [{:.4}, {:.4}] x [{:.4}, {:.4}] outside domain [{:.4}, {:.4}] x [{:.4}, {:.4}]",
                    loop_min_u, loop_max_u, loop_min_v, loop_max_v,
                    domain_min_u, domain_max_u, domain_min_v, domain_max_v
                )
            }
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for TrimError {}

impl From<String> for TrimError {
    fn from(s: String) -> Self {
        Self::Other(s)
    }
}

impl From<TrimError> for String {
    fn from(e: TrimError) -> Self {
        e.to_string()
    }
}

// ============================================================================
// Diagnostics
// ============================================================================

/// Diagnostics collected during trim operations.
///
/// Use this to understand what adjustments were made during loop construction
/// or region assembly.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TrimDiagnostics {
    /// Number of loops processed.
    pub loop_count: usize,
    /// Number of holes identified.
    pub hole_count: usize,
    /// Number of loops that had their orientation flipped.
    pub orientation_flips: usize,
    /// Number of duplicate points removed during cleaning.
    pub duplicate_points_removed: usize,
    /// Number of points snapped due to tolerance.
    pub snapped_points: usize,
    /// Number of closing points removed (first == last).
    pub closing_points_removed: usize,
    /// Warnings generated during processing.
    pub warnings: Vec<String>,
}

impl TrimDiagnostics {
    /// Create empty diagnostics.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Merge another diagnostics into this one.
    pub fn merge(&mut self, other: &Self) {
        self.loop_count += other.loop_count;
        self.hole_count += other.hole_count;
        self.orientation_flips += other.orientation_flips;
        self.duplicate_points_removed += other.duplicate_points_removed;
        self.snapped_points += other.snapped_points;
        self.closing_points_removed += other.closing_points_removed;
        self.warnings.extend(other.warnings.iter().cloned());
    }

    /// Check if any adjustments were made.
    #[must_use]
    pub fn had_adjustments(&self) -> bool {
        self.orientation_flips > 0
            || self.duplicate_points_removed > 0
            || self.snapped_points > 0
            || self.closing_points_removed > 0
    }

    /// Check if there are any warnings.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

// ============================================================================
// UV Domain
// ============================================================================

/// A rectangular domain in UV parameter space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UvDomain {
    /// Minimum U value.
    pub u_min: f64,
    /// Maximum U value.
    pub u_max: f64,
    /// Minimum V value.
    pub v_min: f64,
    /// Maximum V value.
    pub v_max: f64,
}

impl UvDomain {
    /// Create a new UV domain.
    #[must_use]
    pub fn new(u_min: f64, u_max: f64, v_min: f64, v_max: f64) -> Self {
        Self {
            u_min,
            u_max,
            v_min,
            v_max,
        }
    }

    /// Create a unit domain [0, 1] x [0, 1].
    #[must_use]
    pub fn unit() -> Self {
        Self::new(0.0, 1.0, 0.0, 1.0)
    }

    /// Check if a point is inside this domain (with tolerance).
    #[must_use]
    pub fn contains(&self, point: UvPoint, tol: Tolerance) -> bool {
        point.u >= self.u_min - tol.eps
            && point.u <= self.u_max + tol.eps
            && point.v >= self.v_min - tol.eps
            && point.v <= self.v_max + tol.eps
    }

    /// Check if this domain is valid (min <= max for both axes).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.u_min.is_finite()
            && self.u_max.is_finite()
            && self.v_min.is_finite()
            && self.v_max.is_finite()
            && self.u_min <= self.u_max
            && self.v_min <= self.v_max
    }

    /// Get the U span.
    #[must_use]
    pub fn u_span(&self) -> f64 {
        self.u_max - self.u_min
    }

    /// Get the V span.
    #[must_use]
    pub fn v_span(&self) -> f64 {
        self.v_max - self.v_min
    }
}

impl Default for UvDomain {
    fn default() -> Self {
        Self::unit()
    }
}

// ============================================================================
// UvPoint
// ============================================================================

/// A point in UV parameter space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UvPoint {
    /// U parameter value.
    pub u: f64,
    /// V parameter value.
    pub v: f64,
}

impl UvPoint {
    /// Create a new UV point.
    #[must_use]
    pub const fn new(u: f64, v: f64) -> Self {
        Self { u, v }
    }

    /// Check if both coordinates are finite.
    #[must_use]
    pub fn is_finite(&self) -> bool {
        self.u.is_finite() && self.v.is_finite()
    }

    /// Compute squared distance to another point.
    #[must_use]
    pub fn distance_squared(&self, other: UvPoint) -> f64 {
        let du = self.u - other.u;
        let dv = self.v - other.v;
        du * du + dv * dv
    }

    /// Compute distance to another point.
    #[must_use]
    pub fn distance(&self, other: UvPoint) -> f64 {
        self.distance_squared(other).sqrt()
    }

    /// Map this point from one domain to another.
    #[must_use]
    pub fn map_domain(&self, from: &UvDomain, to: &UvDomain) -> Self {
        let u_normalized = if from.u_span() > 0.0 {
            (self.u - from.u_min) / from.u_span()
        } else {
            0.5
        };
        let v_normalized = if from.v_span() > 0.0 {
            (self.v - from.v_min) / from.v_span()
        } else {
            0.5
        };

        Self {
            u: to.u_min + u_normalized * to.u_span(),
            v: to.v_min + v_normalized * to.v_span(),
        }
    }
}

fn approx_eq_uv(tol: Tolerance, a: UvPoint, b: UvPoint) -> bool {
    (a.u - b.u).abs() <= tol.eps && (a.v - b.v).abs() <= tol.eps
}

fn signed_area(points: &[UvPoint]) -> f64 {
    if points.len() < 3 {
        return 0.0;
    }

    let mut area = 0.0;
    for i in 0..points.len() {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        area += a.u * b.v - b.u * a.v;
    }
    0.5 * area
}

fn point_on_segment(p: UvPoint, a: UvPoint, b: UvPoint, tol: Tolerance) -> bool {
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

fn orient2d(a: UvPoint, b: UvPoint, c: UvPoint) -> f64 {
    (b.u - a.u) * (c.v - a.v) - (b.v - a.v) * (c.u - a.u)
}

fn segments_intersect(a: UvPoint, b: UvPoint, c: UvPoint, d: UvPoint, tol: Tolerance) -> bool {
    let o1 = orient2d(a, b, c);
    let o2 = orient2d(a, b, d);
    let o3 = orient2d(c, d, a);
    let o4 = orient2d(c, d, b);

    if o1.abs() <= tol.eps && point_on_segment(c, a, b, tol) {
        return true;
    }
    if o2.abs() <= tol.eps && point_on_segment(d, a, b, tol) {
        return true;
    }
    if o3.abs() <= tol.eps && point_on_segment(a, c, d, tol) {
        return true;
    }
    if o4.abs() <= tol.eps && point_on_segment(b, c, d, tol) {
        return true;
    }

    let ab = (o1 > tol.eps && o2 < -tol.eps) || (o1 < -tol.eps && o2 > tol.eps);
    let cd = (o3 > tol.eps && o4 < -tol.eps) || (o3 < -tol.eps && o4 > tol.eps);
    ab && cd
}

fn loop_self_intersects(points: &[UvPoint], tol: Tolerance) -> bool {
    let n = points.len();
    if n < 4 {
        return false;
    }

    for i in 0..n {
        let a0 = points[i];
        let a1 = points[(i + 1) % n];

        for j in (i + 1)..n {
            let j_next = (j + 1) % n;
            if j == (i + 1) % n || j_next == i {
                continue;
            }

            let b0 = points[j];
            let b1 = points[j_next];
            if segments_intersect(a0, a1, b0, b1, tol) {
                return true;
            }
        }
    }

    false
}

fn loops_intersect(a: &[UvPoint], b: &[UvPoint], tol: Tolerance) -> bool {
    if a.len() < 2 || b.len() < 2 {
        return false;
    }

    for i in 0..a.len() {
        let a0 = a[i];
        let a1 = a[(i + 1) % a.len()];
        for j in 0..b.len() {
            let b0 = b[j];
            let b1 = b[(j + 1) % b.len()];
            if segments_intersect(a0, a1, b0, b1, tol) {
                return true;
            }
        }
    }

    false
}

fn contains_point_polygon(p: UvPoint, points: &[UvPoint], tol: Tolerance) -> bool {
    if points.len() < 3 {
        return false;
    }

    for i in 0..points.len() {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        if point_on_segment(p, a, b, tol) {
            return true;
        }
    }

    let mut inside = false;
    for i in 0..points.len() {
        let a = points[i];
        let b = points[(i + 1) % points.len()];

        let intersects = (a.v > p.v) != (b.v > p.v);
        if !intersects {
            continue;
        }

        let denom = b.v - a.v;
        if denom == 0.0 {
            continue;
        }

        let t = (p.v - a.v) / denom;
        let x = a.u + t * (b.u - a.u);
        if p.u <= x + tol.eps {
            inside = !inside;
        }
    }

    inside
}

// ============================================================================
// TrimLoop
// ============================================================================

/// A closed loop of UV points representing a trim boundary.
///
/// Trim loops are used to define the boundaries of trimmed surfaces.
/// The outer boundary should be counter-clockwise (CCW), while holes
/// should be clockwise (CW).
#[derive(Debug, Clone, PartialEq)]
pub struct TrimLoop {
    points: Vec<UvPoint>,
}

impl TrimLoop {
    /// Create a new trim loop from points.
    ///
    /// The constructor validates the loop:
    /// - All points must be finite
    /// - Duplicate consecutive points are removed
    /// - At least 3 distinct points are required
    /// - The loop must not self-intersect
    ///
    /// # Errors
    /// Returns `TrimError` if validation fails.
    pub fn new(points: Vec<UvPoint>, tol: Tolerance) -> Result<Self, TrimError> {
        let (loop_, _diag) = Self::new_with_diagnostics(points, tol)?;
        Ok(loop_)
    }

    /// Create a new trim loop, also returning diagnostics about adjustments made.
    pub fn new_with_diagnostics(
        mut points: Vec<UvPoint>,
        tol: Tolerance,
    ) -> Result<(Self, TrimDiagnostics), TrimError> {
        let mut diagnostics = TrimDiagnostics::new();
        diagnostics.loop_count = 1;

        // Check for non-finite points
        if points.iter().any(|p| !p.is_finite()) {
            return Err(TrimError::NonFinitePoints);
        }

        // Remove closing point if first == last
        if points.len() > 2 {
            if let (Some(first), Some(last)) = (points.first().copied(), points.last().copied()) {
                if approx_eq_uv(tol, first, last) {
                    points.pop();
                    diagnostics.closing_points_removed += 1;
                }
            }
        }

        // Remove consecutive duplicate points
        let original_count = points.len();
        let mut cleaned = Vec::with_capacity(points.len());
        for p in points {
            if cleaned
                .last()
                .copied()
                .is_some_and(|prev| approx_eq_uv(tol, prev, p))
            {
                continue;
            }
            cleaned.push(p);
        }
        diagnostics.duplicate_points_removed = original_count - cleaned.len();

        // Validate minimum point count
        if cleaned.len() < 3 {
            return Err(TrimError::InsufficientPoints {
                count: cleaned.len(),
            });
        }

        // Check for self-intersection
        if loop_self_intersects(&cleaned, tol) {
            return Err(TrimError::SelfIntersection);
        }

        Ok((Self { points: cleaned }, diagnostics))
    }

    /// Create a trim loop without validation (internal use only).
    ///
    /// # Safety
    /// Caller must ensure points form a valid, non-self-intersecting loop.
    fn new_unchecked(points: Vec<UvPoint>) -> Self {
        Self { points }
    }

    /// Get the points of this loop.
    #[must_use]
    pub fn points(&self) -> &[UvPoint] {
        &self.points
    }

    /// Get the number of points in this loop.
    #[must_use]
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Check if the loop is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Compute the bounding box of this loop.
    #[must_use]
    pub fn bounds(&self) -> UvDomain {
        if self.points.is_empty() {
            return UvDomain::new(0.0, 0.0, 0.0, 0.0);
        }

        let mut u_min = f64::INFINITY;
        let mut u_max = f64::NEG_INFINITY;
        let mut v_min = f64::INFINITY;
        let mut v_max = f64::NEG_INFINITY;

        for p in &self.points {
            u_min = u_min.min(p.u);
            u_max = u_max.max(p.u);
            v_min = v_min.min(p.v);
            v_max = v_max.max(p.v);
        }

        UvDomain::new(u_min, u_max, v_min, v_max)
    }

    /// Check if this loop is fully contained within a domain.
    #[must_use]
    pub fn is_within_domain(&self, domain: &UvDomain, tol: Tolerance) -> bool {
        self.points.iter().all(|p| domain.contains(*p, tol))
    }

    /// Validate that this loop is within a surface domain.
    ///
    /// # Errors
    /// Returns `TrimError::OutsideDomain` if any point is outside the domain.
    pub fn validate_domain(&self, domain: &UvDomain, tol: Tolerance) -> Result<(), TrimError> {
        let bounds = self.bounds();
        if bounds.u_min < domain.u_min - tol.eps
            || bounds.u_max > domain.u_max + tol.eps
            || bounds.v_min < domain.v_min - tol.eps
            || bounds.v_max > domain.v_max + tol.eps
        {
            return Err(TrimError::OutsideDomain {
                loop_min_u: bounds.u_min,
                loop_max_u: bounds.u_max,
                loop_min_v: bounds.v_min,
                loop_max_v: bounds.v_max,
                domain_min_u: domain.u_min,
                domain_max_u: domain.u_max,
                domain_min_v: domain.v_min,
                domain_max_v: domain.v_max,
            });
        }
        Ok(())
    }

    /// Unwrap the loop for U-periodic surfaces.
    ///
    /// Finds the largest jump in U and reorders points to minimize discontinuities.
    #[must_use]
    pub fn unwrapped_u_periodic(&self, start: f64, end: f64, tol: Tolerance) -> Self {
        unwrap_loop_periodic(self, start, end, tol, |p| p.u, |p, value| p.u = value)
    }

    /// Unwrap the loop for V-periodic surfaces.
    ///
    /// Finds the largest jump in V and reorders points to minimize discontinuities.
    #[must_use]
    pub fn unwrapped_v_periodic(&self, start: f64, end: f64, tol: Tolerance) -> Self {
        unwrap_loop_periodic(self, start, end, tol, |p| p.v, |p, value| p.v = value)
    }

    /// Compute the signed area of this loop.
    ///
    /// Positive area = counter-clockwise, negative = clockwise.
    #[must_use]
    pub fn signed_area(&self) -> f64 {
        signed_area(&self.points)
    }

    /// Check if this loop is counter-clockwise.
    #[must_use]
    pub fn is_ccw(&self) -> bool {
        self.signed_area() > 0.0
    }

    /// Check if this loop is clockwise.
    #[must_use]
    pub fn is_cw(&self) -> bool {
        self.signed_area() < 0.0
    }

    /// Return a reversed copy of this loop.
    #[must_use]
    pub fn reversed(&self) -> Self {
        let mut points = self.points.clone();
        points.reverse();
        Self { points }
    }

    /// Ensure the loop is counter-clockwise, reversing if necessary.
    #[must_use]
    pub fn ensure_ccw(&self) -> Self {
        if self.is_ccw() {
            self.clone()
        } else {
            self.reversed()
        }
    }

    /// Ensure the loop is clockwise, reversing if necessary.
    #[must_use]
    pub fn ensure_cw(&self) -> Self {
        if self.is_cw() {
            self.clone()
        } else {
            self.reversed()
        }
    }

    /// Check if a point is inside this loop.
    #[must_use]
    pub fn contains(&self, point: UvPoint, tol: Tolerance) -> bool {
        contains_point_polygon(point, &self.points, tol)
    }

    /// Map this loop from one domain to another.
    ///
    /// Useful for copying trim loops between surfaces with different parameter domains.
    #[must_use]
    pub fn map_domain(&self, from: &UvDomain, to: &UvDomain) -> Self {
        let mapped_points = self
            .points
            .iter()
            .map(|p| p.map_domain(from, to))
            .collect();
        Self::new_unchecked(mapped_points)
    }

    /// Translate this loop by a UV offset.
    #[must_use]
    pub fn translated(&self, du: f64, dv: f64) -> Self {
        let translated_points = self
            .points
            .iter()
            .map(|p| UvPoint::new(p.u + du, p.v + dv))
            .collect();
        Self::new_unchecked(translated_points)
    }

    /// Scale this loop about a center point.
    #[must_use]
    pub fn scaled(&self, center: UvPoint, scale_u: f64, scale_v: f64) -> Self {
        let scaled_points = self
            .points
            .iter()
            .map(|p| {
                UvPoint::new(
                    center.u + (p.u - center.u) * scale_u,
                    center.v + (p.v - center.v) * scale_v,
                )
            })
            .collect();
        Self::new_unchecked(scaled_points)
    }
}

fn unwrap_loop_periodic(
    loop_: &TrimLoop,
    start: f64,
    end: f64,
    tol: Tolerance,
    coord: impl Fn(UvPoint) -> f64,
    set_coord: impl Fn(&mut UvPoint, f64),
) -> TrimLoop {
    let span = end - start;
    if !span.is_finite() || span <= 0.0 || loop_.points.len() < 2 {
        return loop_.clone();
    }

    let half_span = 0.5 * span;
    let threshold = half_span + tol.eps;

    let n = loop_.points.len();
    let mut max_jump = 0.0;
    let mut max_edge = 0usize;

    for i in 0..n {
        let a = coord(loop_.points[i]);
        let b = coord(loop_.points[(i + 1) % n]);
        let jump = (b - a).abs();
        if jump > max_jump {
            max_jump = jump;
            max_edge = i;
        }
    }

    let mut points = Vec::with_capacity(n);
    if max_jump > threshold {
        let start_index = (max_edge + 1) % n;
        points.extend_from_slice(&loop_.points[start_index..]);
        points.extend_from_slice(&loop_.points[..start_index]);
    } else {
        points.extend_from_slice(&loop_.points);
    }

    for i in 1..points.len() {
        let prev = coord(points[i - 1]);
        let mut value = coord(points[i]);
        let delta = value - prev;
        if delta > threshold {
            value -= span;
        } else if delta < -threshold {
            value += span;
        }
        set_coord(&mut points[i], value);
    }

    TrimLoop { points }
}

// ============================================================================
// TrimRegion
// ============================================================================

/// A trim region consisting of an outer boundary and optional holes.
///
/// The outer loop is oriented counter-clockwise (CCW), and all holes
/// are oriented clockwise (CW). This is the standard convention for
/// defining trimmed surface regions.
#[derive(Debug, Clone, PartialEq)]
pub struct TrimRegion {
    /// The outer boundary loop (CCW orientation).
    pub outer: TrimLoop,
    /// Interior hole loops (CW orientation).
    pub holes: Vec<TrimLoop>,
}

impl TrimRegion {
    /// Create a trim region from a list of loops.
    ///
    /// The largest loop by area is used as the outer boundary.
    /// Remaining loops are treated as holes. Orientations are
    /// automatically normalized (outer=CCW, holes=CW).
    ///
    /// # Errors
    /// Returns `TrimError` if:
    /// - No loops provided
    /// - A hole intersects the outer boundary
    /// - A hole is not contained within the outer boundary
    /// - Two holes intersect each other
    /// - Holes are nested
    pub fn from_loops(loops: Vec<TrimLoop>, tol: Tolerance) -> Result<Self, TrimError> {
        let (region, _diag) = Self::from_loops_with_diagnostics(loops, tol)?;
        Ok(region)
    }

    /// Create a trim region from loops, also returning diagnostics.
    pub fn from_loops_with_diagnostics(
        mut loops: Vec<TrimLoop>,
        tol: Tolerance,
    ) -> Result<(Self, TrimDiagnostics), TrimError> {
        let mut diagnostics = TrimDiagnostics::new();

        if loops.is_empty() {
            return Err(TrimError::EmptyLoopSet);
        }

        diagnostics.loop_count = loops.len();

        // Find the largest loop by area (this becomes the outer boundary)
        let (outer_index, _area) = loops
            .iter()
            .enumerate()
            .map(|(idx, loop_)| (idx, loop_.signed_area().abs()))
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap_or((0, 0.0));

        let outer = loops.swap_remove(outer_index);

        // Ensure outer loop is CCW
        let outer = if outer.is_ccw() {
            outer
        } else {
            diagnostics.orientation_flips += 1;
            outer.reversed()
        };

        // Process holes
        let mut holes: Vec<TrimLoop> = Vec::new();
        for loop_ in loops {
            // Ensure hole is CW
            let hole = if loop_.is_ccw() {
                diagnostics.orientation_flips += 1;
                loop_.reversed()
            } else {
                loop_
            };

            let probe = hole
                .points()
                .first()
                .copied()
                .unwrap_or(UvPoint::new(0.0, 0.0));

            // Check hole doesn't intersect outer boundary
            if loops_intersect(outer.points(), hole.points(), tol) {
                return Err(TrimError::HoleIntersectsOuter);
            }

            // Check hole is inside outer boundary
            if !outer.contains(probe, tol) {
                return Err(TrimError::HoleOutsideBoundary);
            }

            // Check hole doesn't intersect or nest with existing holes
            for existing in &holes {
                if loops_intersect(existing.points(), hole.points(), tol) {
                    return Err(TrimError::HolesIntersect);
                }

                let existing_probe = existing
                    .points()
                    .first()
                    .copied()
                    .unwrap_or(UvPoint::new(0.0, 0.0));

                if existing.contains(probe, tol) || hole.contains(existing_probe, tol) {
                    return Err(TrimError::NestedHoles);
                }
            }
            holes.push(hole);
        }

        diagnostics.hole_count = holes.len();

        Ok((Self { outer, holes }, diagnostics))
    }

    /// Create a simple rectangular trim region from a domain.
    #[must_use]
    pub fn from_domain(domain: &UvDomain) -> Self {
        let points = vec![
            UvPoint::new(domain.u_min, domain.v_min),
            UvPoint::new(domain.u_max, domain.v_min),
            UvPoint::new(domain.u_max, domain.v_max),
            UvPoint::new(domain.u_min, domain.v_max),
        ];
        // This is a simple rectangle, no need for validation
        let outer = TrimLoop::new_unchecked(points);
        Self {
            outer,
            holes: Vec::new(),
        }
    }

    /// Get the bounding box of this region (outer loop bounds).
    #[must_use]
    pub fn bounds(&self) -> UvDomain {
        self.outer.bounds()
    }

    /// Check if a point is inside this region (inside outer, outside all holes).
    #[must_use]
    pub fn contains(&self, point: UvPoint, tol: Tolerance) -> bool {
        if !self.outer.contains(point, tol) {
            return false;
        }
        !self.holes.iter().any(|hole| hole.contains(point, tol))
    }

    /// Check if this region is within a surface domain.
    #[must_use]
    pub fn is_within_domain(&self, domain: &UvDomain, tol: Tolerance) -> bool {
        self.outer.is_within_domain(domain, tol)
    }

    /// Validate that this region is within a surface domain.
    pub fn validate_domain(&self, domain: &UvDomain, tol: Tolerance) -> Result<(), TrimError> {
        self.outer.validate_domain(domain, tol)
    }

    /// Map this region from one domain to another.
    #[must_use]
    pub fn map_domain(&self, from: &UvDomain, to: &UvDomain) -> Self {
        Self {
            outer: self.outer.map_domain(from, to),
            holes: self.holes.iter().map(|h| h.map_domain(from, to)).collect(),
        }
    }

    /// Get the total number of loops (outer + holes).
    #[must_use]
    pub fn loop_count(&self) -> usize {
        1 + self.holes.len()
    }

    /// Get the total number of points across all loops.
    #[must_use]
    pub fn total_point_count(&self) -> usize {
        self.outer.len() + self.holes.iter().map(TrimLoop::len).sum::<usize>()
    }
}

// ============================================================================
// Curve-based trim loop creation
// ============================================================================

/// Create a trim loop from a closed 3D curve by projecting to UV space.
///
/// The curve's X and Y coordinates are used as U and V parameters.
/// This is useful for curves that are already in parameter space.
///
/// # Errors
/// Returns `TrimError` if the curve is not closed or loop validation fails.
pub fn trim_loop_from_curve_uv(
    curve: &impl Curve3,
    steps: usize,
    tol: Tolerance,
) -> Result<TrimLoop, TrimError> {
    if !curve.is_closed() {
        return Err(TrimError::CurveNotClosed);
    }

    let steps = steps.max(3);
    let points = tessellate_curve_uniform(curve, steps);
    let uv_points = points
        .into_iter()
        .map(|p| UvPoint::new(p.x, p.y))
        .collect();
    TrimLoop::new(uv_points, tol)
}

// ============================================================================
// Loop-based trim operations
// ============================================================================

/// Copy trim loops from a source surface to a target surface.
///
/// Maps the loops from the source domain to the target domain using
/// normalized parameter mapping.
///
/// # Arguments
/// * `source_loops` - The trim loops to copy
/// * `source_domain` - The parameter domain of the source surface
/// * `target_domain` - The parameter domain of the target surface
/// * `tol` - Tolerance for validation
///
/// # Returns
/// The copied loops mapped to the target domain, plus diagnostics.
///
/// # Errors
/// Returns `TrimError` if any mapped loop falls outside the target domain.
pub fn copy_trim_loops(
    source_loops: &[TrimLoop],
    source_domain: &UvDomain,
    target_domain: &UvDomain,
    tol: Tolerance,
) -> Result<(Vec<TrimLoop>, TrimDiagnostics), TrimError> {
    let mut diagnostics = TrimDiagnostics::new();
    diagnostics.loop_count = source_loops.len();

    let mut result = Vec::with_capacity(source_loops.len());
    for loop_ in source_loops {
        let mapped = loop_.map_domain(source_domain, target_domain);
        mapped.validate_domain(target_domain, tol)?;
        result.push(mapped);
    }

    Ok((result, diagnostics))
}

/// Copy a trim region from a source surface to a target surface.
///
/// # Arguments
/// * `source_region` - The trim region to copy
/// * `source_domain` - The parameter domain of the source surface
/// * `target_domain` - The parameter domain of the target surface
/// * `tol` - Tolerance for validation
///
/// # Returns
/// The copied region mapped to the target domain, plus diagnostics.
pub fn copy_trim_region(
    source_region: &TrimRegion,
    source_domain: &UvDomain,
    target_domain: &UvDomain,
    tol: Tolerance,
) -> Result<(TrimRegion, TrimDiagnostics), TrimError> {
    let mut diagnostics = TrimDiagnostics::new();

    let mapped = source_region.map_domain(source_domain, target_domain);
    mapped.validate_domain(target_domain, tol)?;

    diagnostics.loop_count = mapped.loop_count();
    diagnostics.hole_count = mapped.holes.len();

    Ok((mapped, diagnostics))
}

/// Retrim a surface by applying new trim loops.
///
/// This validates the new loops against the surface domain and
/// ensures they form a valid trim region.
///
/// # Arguments
/// * `loops` - The new trim loops to apply
/// * `surface_domain` - The parameter domain of the surface
/// * `tol` - Tolerance for validation
///
/// # Returns
/// A validated trim region, plus diagnostics.
pub fn retrim_loops(
    loops: Vec<TrimLoop>,
    surface_domain: &UvDomain,
    tol: Tolerance,
) -> Result<(TrimRegion, TrimDiagnostics), TrimError> {
    // First validate all loops are within domain
    for loop_ in &loops {
        loop_.validate_domain(surface_domain, tol)?;
    }

    // Then build the region
    TrimRegion::from_loops_with_diagnostics(loops, tol)
}

/// Remove all trim loops from a surface, returning the full domain.
///
/// This creates a simple rectangular trim region covering the entire
/// surface parameter domain.
///
/// # Arguments
/// * `surface_domain` - The parameter domain of the surface
///
/// # Returns
/// A rectangular trim region covering the full domain, plus diagnostics.
#[must_use]
pub fn untrim_to_domain(surface_domain: &UvDomain) -> (TrimRegion, TrimDiagnostics) {
    let mut diagnostics = TrimDiagnostics::new();
    diagnostics.loop_count = 1;

    let region = TrimRegion::from_domain(surface_domain);
    (region, diagnostics)
}

// ============================================================================
// Bound-based trim operations (legacy compatibility)
// ============================================================================

fn bounds_are_valid(min: [f64; 3], max: [f64; 3]) -> bool {
    for axis in 0..3 {
        if !min[axis].is_finite() || !max[axis].is_finite() || min[axis] > max[axis] {
            return false;
        }
    }
    true
}

/// Copy trim bounds from a source surface to a target surface.
///
/// Computes the intersection of source and target bounds. If no
/// valid intersection exists, returns the target bounds unchanged.
///
/// This is a simplified bound-based operation for backward compatibility.
/// For full trim loop support, use `copy_trim_loops` or `copy_trim_region`.
#[must_use]
pub fn copy_trim_bounds(
    source_min: [f64; 3],
    source_max: [f64; 3],
    target_min: [f64; 3],
    target_max: [f64; 3],
) -> ([f64; 3], [f64; 3]) {
    let min = [
        source_min[0].max(target_min[0]),
        source_min[1].max(target_min[1]),
        source_min[2].max(target_min[2]),
    ];
    let max = [
        source_max[0].min(target_max[0]),
        source_max[1].min(target_max[1]),
        source_max[2].min(target_max[2]),
    ];

    if bounds_are_valid(min, max) {
        (min, max)
    } else {
        (target_min, target_max)
    }
}

/// Retrim bounds by intersecting source with target.
///
/// This is equivalent to `copy_trim_bounds` and is provided for
/// semantic clarity in retrim operations.
#[must_use]
pub fn retrim_bounds(
    source_min: [f64; 3],
    source_max: [f64; 3],
    target_min: [f64; 3],
    target_max: [f64; 3],
) -> ([f64; 3], [f64; 3]) {
    copy_trim_bounds(source_min, source_max, target_min, target_max)
}

/// Remove trim bounds from a surface.
///
/// Returns the full surface bounds unchanged. This is the bound-based
/// equivalent of `untrim_to_domain`.
#[must_use]
pub fn untrim_bounds(min: [f64; 3], max: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    (min, max)
}
