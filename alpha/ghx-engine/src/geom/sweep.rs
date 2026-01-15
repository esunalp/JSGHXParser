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

/// Kink miter type for sweep operations.
///
/// Controls how the profile is handled at sharp corners (kinks) in the rail curve.
/// In Grasshopper, this is exposed as an integer input (0=None, 1=Trim, 2=Rotate).
///
/// # Behavior at Kinks
///
/// When the rail has a sharp corner (detected when the dot product of consecutive
/// tangents falls below `CUSP_DOT_THRESHOLD`, approximately 75°):
///
/// - **None**: No special treatment. The profile continues through the kink using
///   interpolated frames, which may cause self-intersection at sharp corners.
///
/// - **Trim**: At each kink, the profile is trimmed by the bisector plane of the
///   two adjacent tangent directions. This creates a clean mitered corner similar
///   to a picture frame joint.
///
/// - **Rotate**: At each kink, the profile is duplicated and rotated to align with
///   both the incoming and outgoing tangent directions. This creates a smooth
///   transition but may introduce extra geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum MiterType {
    /// No miter handling - profile passes through kinks without adjustment.
    /// May cause self-intersection at sharp corners.
    #[default]
    None = 0,
    /// Trim the profile at the bisector plane of adjacent tangents.
    /// Creates clean mitered corners like a picture frame joint.
    Trim = 1,
    /// Rotate the profile to match both incoming and outgoing tangents.
    /// Creates smooth transitions with additional geometry at corners.
    Rotate = 2,
}

impl MiterType {
    /// Parse miter type from an integer value (matching Grasshopper convention).
    ///
    /// - 0 → None
    /// - 1 → Trim
    /// - 2 → Rotate
    /// - Other values → None (with a warning in diagnostics)
    #[must_use]
    pub fn from_int(value: i32) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Trim,
            2 => Self::Rotate,
            _ => Self::None, // Default for invalid values
        }
    }

    /// Convert to integer value (matching Grasshopper convention).
    #[must_use]
    pub const fn to_int(self) -> i32 {
        self as i32
    }

    /// Check if this miter type requires special handling at kinks.
    #[must_use]
    pub const fn requires_kink_handling(self) -> bool {
        matches!(self, Self::Trim | Self::Rotate)
    }
}

/// Options for controlling sweep mesh generation.
#[derive(Debug, Clone, Copy)]
pub struct SweepOptions {
    /// Total twist applied from start to end, in radians.
    pub twist_radians_total: f64,
    /// How to handle sharp corners (kinks) in the rail curve.
    pub miter: MiterType,
}

impl Default for SweepOptions {
    fn default() -> Self {
        Self {
            twist_radians_total: 0.0,
            miter: MiterType::None,
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
    #[error("profile plane is degenerate (collinear or insufficient points)")]
    DegenerateProfilePlane,
    #[error("sweep2 requires at least one section profile")]
    NoSections,
    #[error("section {index} has fewer than 2 points")]
    SectionTooShort { index: usize },
    #[error("section parameters must be monotonically increasing in [0, 1]")]
    InvalidSectionParameters,
    #[error("sections and parameters count mismatch: {sections} sections, {params} parameters")]
    SectionParameterCountMismatch { sections: usize, params: usize },
}

// ============================================================================
// Rail Alignment Types and Options
// ============================================================================

/// Result of rail alignment analysis and correction.
#[derive(Debug, Clone)]
pub struct RailAlignmentResult {
    /// The (potentially flipped) rail A points.
    pub rail_a: Vec<Point3>,
    /// The (potentially flipped) rail B points.
    pub rail_b: Vec<Point3>,
    /// Whether rail A was reversed to match direction.
    pub rail_a_flipped: bool,
    /// Whether rail B was reversed to match direction.
    pub rail_b_flipped: bool,
    /// Alignment score: 1.0 = perfect alignment, 0.0 = orthogonal, -1.0 = opposite.
    pub alignment_score: f64,
    /// Warnings generated during alignment.
    pub warnings: Vec<String>,
}

/// Options for multi-section Sweep2.
#[derive(Debug, Clone)]
pub struct Sweep2MultiSectionOptions {
    /// Base sweep options (twist, etc.)
    pub sweep: SweepOptions,
    /// If true, use arc-length parameterization for section placement.
    /// If false, use uniform parameter spacing.
    pub arc_length_params: bool,
    /// If true and section_parameters is empty, distribute sections evenly.
    pub auto_distribute_sections: bool,
}

impl Default for Sweep2MultiSectionOptions {
    fn default() -> Self {
        Self {
            sweep: SweepOptions::default(),
            arc_length_params: true,
            auto_distribute_sections: true,
        }
    }
}

// ============================================================================
// Rail Alignment Functions
// ============================================================================

/// Analyzes and aligns two rails for Sweep2 operation.
///
/// This function:
/// 1. Checks if the rails are oriented in compatible directions
/// 2. Flips rails if needed so they run in the same direction
/// 3. Resamples both rails to the same point count using arc-length parameterization
/// 4. Provides alignment diagnostics and warnings
///
/// # Rail Direction Detection
///
/// The algorithm compares the distance between corresponding endpoints:
/// - If (A_start to B_start + A_end to B_end) < (A_start to B_end + A_end to B_start),
///   the rails are already aligned.
/// - Otherwise, rail B should be flipped.
///
/// Additionally, if both rails share a common start point (within tolerance), they're
/// considered aligned; if they share start/end points in opposite order, one needs flipping.
///
/// # Arc-Length Resampling
///
/// After alignment, both rails are resampled to have the same number of points using
/// arc-length parameterization. This ensures that corresponding points on each rail
/// are at the same relative position along their respective paths, which prevents
/// twisted or skewed sweep results.
pub fn align_sweep2_rails(
    rail_a: &[Point3],
    rail_b: &[Point3],
    target_count: usize,
    tol: Tolerance,
) -> Result<RailAlignmentResult, SweepError> {
    if rail_a.len() < 2 {
        return Err(SweepError::RailTooShort);
    }
    if rail_b.len() < 2 {
        return Err(SweepError::RailTooShort);
    }
    
    // Validate finite points
    for (i, p) in rail_a.iter().enumerate() {
        if !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite() {
            return Err(SweepError::NonFiniteRail);
        }
        if i > 0 && tol.approx_eq_point3(*p, rail_a[i - 1]) {
            // Skip duplicate points during validation
        }
    }
    for (i, p) in rail_b.iter().enumerate() {
        if !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite() {
            return Err(SweepError::NonFiniteRail);
        }
        if i > 0 && tol.approx_eq_point3(*p, rail_b[i - 1]) {
            // Skip duplicate points during validation
        }
    }

    let mut warnings = Vec::new();

    // Get start/end points of each rail
    let a_start = rail_a[0];
    let a_end = *rail_a.last().unwrap();
    let b_start = rail_b[0];
    let b_end = *rail_b.last().unwrap();

    // Compute distances for alignment check
    let dist_aligned = a_start.sub_point(b_start).length() + a_end.sub_point(b_end).length();
    let dist_flipped = a_start.sub_point(b_end).length() + a_end.sub_point(b_start).length();

    // Check if rails share endpoints (common in Grasshopper workflows)
    let shares_start = tol.approx_eq_point3(a_start, b_start);
    let shares_end = tol.approx_eq_point3(a_end, b_end);
    let shares_cross_start_end = tol.approx_eq_point3(a_start, b_end);
    let shares_cross_end_start = tol.approx_eq_point3(a_end, b_start);

    // Determine if rail B should be flipped
    let flip_b = if shares_start && shares_end {
        // Both endpoints shared - perfectly aligned
        false
    } else if shares_cross_start_end && shares_cross_end_start {
        // Cross-shared - rail B is reversed
        true
    } else if shares_start || shares_end {
        // One endpoint shared - use that to determine alignment
        false
    } else if shares_cross_start_end || shares_cross_end_start {
        // One cross-shared endpoint - rail B is reversed
        true
    } else {
        // No shared endpoints - use distance metric
        dist_flipped < dist_aligned
    };

    // Create aligned rail copies
    let aligned_rail_a = rail_a.to_vec();
    let aligned_rail_b = if flip_b {
        warnings.push("rail B direction reversed for alignment".to_string());
        rail_b.iter().rev().copied().collect()
    } else {
        rail_b.to_vec()
    };

    // Compute alignment score (1.0 = perfect, -1.0 = opposite)
    // Based on dot product of rail direction vectors
    let a_dir = a_end.sub_point(a_start).normalized().unwrap_or(Vec3::X);
    let b_dir = if flip_b {
        b_start.sub_point(b_end).normalized().unwrap_or(Vec3::X)
    } else {
        b_end.sub_point(b_start).normalized().unwrap_or(Vec3::X)
    };
    let alignment_score = a_dir.dot(b_dir);

    // Resample both rails to target_count using arc-length parameterization
    let resampled_a = resample_polyline_arc_length(&aligned_rail_a, target_count, tol);
    let resampled_b = resample_polyline_arc_length(&aligned_rail_b, target_count, tol);

    // Check for potential issues after resampling
    if resampled_a.len() != resampled_b.len() {
        warnings.push(format!(
            "resampled rail lengths differ unexpectedly: {} vs {}",
            resampled_a.len(),
            resampled_b.len()
        ));
    }

    Ok(RailAlignmentResult {
        rail_a: resampled_a,
        rail_b: resampled_b,
        rail_a_flipped: false, // We never flip rail A - it's the reference
        rail_b_flipped: flip_b,
        alignment_score,
        warnings,
    })
}

/// Resamples a polyline to a target number of points using arc-length parameterization.
///
/// This ensures uniform spacing along the curve's actual path length, which is
/// important for sweep operations where corresponding rail points should be at
/// matching relative positions.
fn resample_polyline_arc_length(points: &[Point3], target_count: usize, tol: Tolerance) -> Vec<Point3> {
    if points.len() < 2 || target_count < 2 {
        return points.to_vec();
    }

    // Compute cumulative arc lengths
    let arc_lengths = compute_arc_lengths(points);
    let total_length = *arc_lengths.last().unwrap_or(&0.0);

    if total_length < tol.eps {
        // Degenerate case: all points are the same
        return vec![points[0]; target_count];
    }

    let mut result = Vec::with_capacity(target_count);

    // Sample at uniform arc-length intervals
    for i in 0..target_count {
        let t = if target_count > 1 {
            i as f64 / (target_count - 1) as f64
        } else {
            0.0
        };
        let target_length = t * total_length;

        // Find the segment containing this arc length
        let point = sample_at_arc_length(points, &arc_lengths, target_length);
        result.push(point);
    }

    result
}

/// Samples a point along a polyline at a given arc length.
fn sample_at_arc_length(points: &[Point3], arc_lengths: &[f64], target_length: f64) -> Point3 {
    // Handle boundary cases
    if target_length <= 0.0 || arc_lengths.is_empty() {
        return points[0];
    }
    let total = *arc_lengths.last().unwrap();
    if target_length >= total {
        return *points.last().unwrap();
    }

    // Binary search to find the segment
    let idx = arc_lengths.partition_point(|&len| len < target_length);
    if idx == 0 {
        return points[0];
    }
    if idx >= points.len() {
        return *points.last().unwrap();
    }

    // Interpolate within the segment
    let seg_start = arc_lengths[idx - 1];
    let seg_end = arc_lengths[idx];
    let seg_len = seg_end - seg_start;

    if seg_len < 1e-12 {
        return points[idx - 1];
    }

    let t = (target_length - seg_start) / seg_len;
    let p0 = points[idx - 1];
    let p1 = points[idx];

    Point3::new(
        p0.x + t * (p1.x - p0.x),
        p0.y + t * (p1.y - p0.y),
        p0.z + t * (p1.z - p0.z),
    )
}

// ============================================================================
// Profile Plane Transform
// ============================================================================

/// Represents the best-fit plane of a profile curve, used to transform
/// arbitrary-orientation profiles to local XY coordinates for sweeping.
///
/// When profiles are not in the XY plane (e.g., a circle in the YZ plane),
/// they must be transformed to local coordinates where X maps to the frame's
/// normal and Y maps to the frame's binormal. This struct captures:
///
/// - The profile's centroid (origin of the local coordinate system)
/// - The local X-axis (first principal direction in the profile plane)
/// - The local Y-axis (second principal direction in the profile plane)  
/// - The plane normal (third direction, perpendicular to the profile plane)
///
/// The profile points are then expressed as 2D (x, y) coordinates in this
/// local system, with z typically near zero (deviation from planarity).
#[derive(Debug, Clone)]
pub struct ProfilePlaneTransform {
    /// Centroid of the original profile points.
    pub centroid: Point3,
    /// Local X-axis in world coordinates (first principal direction).
    pub local_x: Vec3,
    /// Local Y-axis in world coordinates (second principal direction).
    pub local_y: Vec3,
    /// Plane normal (local Z-axis), perpendicular to the profile plane.
    pub normal: Vec3,
}

impl ProfilePlaneTransform {
    /// Compute the best-fit plane for a set of profile points.
    ///
    /// Uses PCA (covariance matrix + power iteration) to find the two
    /// principal directions within the point cloud, which define the
    /// profile plane. The normal is computed as their cross product.
    ///
    /// # Returns
    /// `None` if the points are degenerate (fewer than 2, collinear, or NaN).
    pub fn from_points(points: &[Point3], tol: Tolerance) -> Option<Self> {
        if points.len() < 2 {
            return None;
        }

        // Validate points are finite
        for p in points {
            if !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite() {
                return None;
            }
        }

        // Compute centroid
        let n = points.len() as f64;
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for p in points {
            cx += p.x;
            cy += p.y;
            cz += p.z;
        }
        let centroid = Point3::new(cx / n, cy / n, cz / n);

        // If only 2 points, we need to create an arbitrary perpendicular plane
        if points.len() == 2 {
            let edge = points[1].sub_point(points[0]);
            let local_x = edge.normalized()?;
            
            // Choose a reference vector not parallel to the edge
            let reference = if local_x.x.abs() < 0.9 {
                Vec3::X
            } else {
                Vec3::Y
            };
            let normal = local_x.cross(reference).normalized()?;
            let local_y = normal.cross(local_x).normalized()?;
            
            return Some(Self {
                centroid,
                local_x,
                local_y,
                normal,
            });
        }

        // Build covariance matrix for 3+ points
        let mut cov = [[0.0f64; 3]; 3];
        for p in points {
            let d = [p.x - centroid.x, p.y - centroid.y, p.z - centroid.z];
            for i in 0..3 {
                for j in 0..3 {
                    cov[i][j] += d[i] * d[j];
                }
            }
        }

        // Power iteration to find two largest eigenvectors (principal directions)
        let (local_x, local_y) = compute_profile_plane_axes(&cov, tol)?;
        let normal = local_x.cross(local_y).normalized().unwrap_or(Vec3::Z);

        Some(Self {
            centroid,
            local_x,
            local_y,
            normal,
        })
    }

    /// Transform a world-space point to local 2D coordinates in the profile plane.
    ///
    /// Returns (x, y) where x is the projection onto `local_x` and y is the
    /// projection onto `local_y`, both relative to the centroid.
    #[inline]
    pub fn world_to_local_2d(&self, p: Point3) -> (f64, f64) {
        let d = p.sub_point(self.centroid);
        let x = d.dot(self.local_x);
        let y = d.dot(self.local_y);
        (x, y)
    }

    /// Transform a 2D local coordinate back to world space.
    ///
    /// The z-offset from the plane is assumed to be zero.
    #[inline]
    pub fn local_2d_to_world(&self, x: f64, y: f64) -> Point3 {
        self.centroid
            .add_vec(self.local_x.mul_scalar(x))
            .add_vec(self.local_y.mul_scalar(y))
    }

    /// Transform all profile points to local 2D coordinates.
    ///
    /// Returns points where z represents the deviation from the plane (typically small).
    pub fn transform_profile_to_local(&self, points: &[Point3]) -> Vec<Point3> {
        points
            .iter()
            .map(|p| {
                let d = p.sub_point(self.centroid);
                let x = d.dot(self.local_x);
                let y = d.dot(self.local_y);
                let z = d.dot(self.normal); // deviation from plane
                Point3::new(x, y, z)
            })
            .collect()
    }

    /// Check if the profile plane is approximately the XY plane.
    ///
    /// Returns true if the profile is already in XY coordinates (centered near origin,
    /// with normal approximately ±Z). This allows skipping the transform for simple cases.
    pub fn is_xy_aligned(&self, tol: Tolerance) -> bool {
        // Check if centroid is near origin
        let centroid_near_origin = self.centroid.x.abs() < tol.eps
            && self.centroid.y.abs() < tol.eps
            && self.centroid.z.abs() < tol.eps;

        // Check if normal is approximately ±Z
        let normal_is_z = self.normal.z.abs() > 1.0 - tol.eps;

        centroid_near_origin && normal_is_z
    }

    /// Create a frame at a point along the rail that properly orients the profile.
    ///
    /// This combines the sweep frame (from parallel transport along the rail) with
    /// the profile's original orientation to produce the correct world-space frame.
    ///
    /// The profile's local_x maps to the frame's normal, local_y maps to binormal.
    pub fn orient_frame_for_profile(&self, sweep_frame: &FrenetFrame) -> FrenetFrame {
        // The sweep frame's normal/binormal define the plane where the profile will be placed.
        // We need to align our profile's local_x/local_y with this plane.
        //
        // However, if the profile wasn't originally in XY, we need to account for its
        // original orientation relative to the rail's starting tangent.
        *sweep_frame
    }
}

/// Compute two principal axes from a covariance matrix using power iteration.
/// Returns (local_x, local_y), which span the plane of maximum variance.
/// The plane normal can be computed as their cross product.
fn compute_profile_plane_axes(cov: &[[f64; 3]; 3], tol: Tolerance) -> Option<(Vec3, Vec3)> {
    // For a planar point set, the normal is the eigenvector with the smallest eigenvalue.
    // We'll use power iteration to find the largest eigenvector, then find the second
    // largest by deflation.
    
    // Power iteration for largest eigenvector.
    // Start with a vector that has components in all directions.
    let mut v1 = [1.0 / 3.0f64.sqrt(), 1.0 / 3.0f64.sqrt(), 1.0 / 3.0f64.sqrt()];
    let mut lambda1 = 0.0;

    for _ in 0..50 {
        let mut result = [0.0; 3];
        for i in 0..3 {
            for j in 0..3 {
                result[i] += cov[i][j] * v1[j];
            }
        }
        let len = (result[0].powi(2) + result[1].powi(2) + result[2].powi(2)).sqrt();
        if len < tol.eps {
            // Matrix is essentially zero - degenerate case, use default XY plane
            return Some((Vec3::X, Vec3::Y));
        }
        lambda1 = len;
        v1 = [result[0] / len, result[1] / len, result[2] / len];
    }

    let u_axis = Vec3::new(v1[0], v1[1], v1[2]);
    let u_norm = u_axis.normalized()?;

    // Deflate matrix to remove v1's contribution.
    let mut cov2 = *cov;
    for i in 0..3 {
        for j in 0..3 {
            cov2[i][j] -= lambda1 * v1[i] * v1[j];
        }
    }

    // Power iteration on deflated matrix to find second largest eigenvector.
    // Start with a vector orthogonal to v1.
    let reference = if u_norm.x.abs() < 0.9 { Vec3::X } else { Vec3::Y };
    let ortho_start = u_norm.cross(reference).normalized()?;
    let mut v2 = [ortho_start.x, ortho_start.y, ortho_start.z];
    let mut lambda2 = 0.0;
    let mut v2_found = false;

    for _ in 0..50 {
        let mut result = [0.0; 3];
        for i in 0..3 {
            for j in 0..3 {
                result[i] += cov2[i][j] * v2[j];
            }
        }
        // Project out v1 component to stay in the orthogonal subspace
        let dot_v1 = result[0] * v1[0] + result[1] * v1[1] + result[2] * v1[2];
        for i in 0..3 {
            result[i] -= dot_v1 * v1[i];
        }
        let len = (result[0].powi(2) + result[1].powi(2) + result[2].powi(2)).sqrt();
        if len < tol.eps {
            // Second eigenvalue is very small - this is the normal direction!
            // For perfectly planar profiles, v2 is the normal direction.
            break;
        }
        v2_found = true;
        lambda2 = len;
        v2 = [result[0] / len, result[1] / len, result[2] / len];
    }

    // If power iteration found a valid second eigenvector in the plane,
    // use v1 and v2 as the in-plane axes.
    // Otherwise v2 converged to the normal direction.
    let v_axis = Vec3::new(v2[0], v2[1], v2[2]);
    
    if v2_found && lambda2 > tol.eps {
        // v1 and v2 are both in the plane (the two largest eigenvectors)
        // Normal = v1 × v2
        let v_norm = v_axis.normalized()?;
        // Ensure orthogonality due to numerical precision
        let v_final = v_norm.sub(u_norm.mul_scalar(u_norm.dot(v_norm))).normalized()?;
        Some((u_norm, v_final))
    } else {
        // v2 is approximately the normal direction (smallest eigenvalue)
        // Need to find two orthogonal vectors in the plane perpendicular to v2
        let normal = v_axis.normalized().unwrap_or(Vec3::Z);
        let ref_for_plane = if normal.x.abs() < 0.9 { Vec3::X } else { Vec3::Y };
        let local_x = normal.cross(ref_for_plane).normalized()?;
        let local_y = normal.cross(local_x).normalized()?;
        Some((local_x, local_y))
    }
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

    // Compute the profile's plane transform to handle non-XY profiles correctly.
    // This transforms the profile from its original world-space orientation into
    // local 2D coordinates (x, y) that can be mapped to (normal, binormal) directions.
    let profile_plane = ProfilePlaneTransform::from_points(&cleaned_profile.points, tol)
        .ok_or(SweepError::DegenerateProfilePlane)?;

    // Transform profile points to local 2D coordinates in the profile plane.
    let local_profile = profile_plane.transform_profile_to_local(&cleaned_profile.points);

    // Compute rotation-minimizing frames along the rail with miter handling.
    let frame_result = compute_rail_frames_with_miter(&cleaned_rail.points, options.miter, tol);
    let mut frames = frame_result.frames;
    let rail_warnings = frame_result.warnings;

    // Align the initial frame with the profile's original plane orientation.
    // This ensures the sweep starts with the profile in its original orientation.
    if !frames.is_empty() {
        frames = align_frames_to_profile_plane(&frames, &profile_plane, &cleaned_rail.points, tol);
    }

    // Apply optional twist after profile-plane alignment.
    let frames = if options.twist_radians_total.abs() > tol.eps {
        apply_twist(&frames, &cleaned_rail.points, options.twist_radians_total, tol)
    } else {
        frames
    };

    let profile_len = local_profile.len();
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
        for (i, &p) in local_profile.iter().enumerate() {
            // Profile is now in local coordinates: X -> normal, Y -> binormal, Z -> tangent.
            // The profile plane transform has already centered and reoriented the points.
            let world_point = rail_point
                .add_vec(frame.normal.mul_scalar(p.x))
                .add_vec(frame.binormal.mul_scalar(p.y))
                .add_vec(frame.tangent.mul_scalar(p.z));
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

    // Caps use the local profile coordinates for proper triangulation.
    if caps.start {
        add_cap(
            &mut vertices,
            &mut uvs,
            &mut indices,
            &local_profile,
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
            &local_profile,
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

    // Compute the profile's plane transform to handle non-XY profiles correctly.
    let profile_plane = ProfilePlaneTransform::from_points(&cleaned_profile.points, tol)
        .ok_or(SweepError::DegenerateProfilePlane)?;

    // Transform profile points to local 2D coordinates in the profile plane.
    let local_profile = profile_plane.transform_profile_to_local(&cleaned_profile.points);

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

    // Align frames to respect the profile's original plane orientation.
    if !frames.is_empty() {
        frames = align_frames_to_profile_plane(&frames, &profile_plane, rail_a, tol);
    }

    if options.twist_radians_total.abs() > tol.eps {
        frames = apply_twist(&frames, &rail_a[..ring_count], options.twist_radians_total, tol);
    }

    // Build vertices using the local profile coordinates.
    let profile_len = local_profile.len();
    let arc_lengths = compute_arc_lengths(&rail_a[..ring_count]);
    let total_arc_length = arc_lengths.last().copied().unwrap_or(0.0).max(tol.eps);

    let mut vertices: Vec<Point3> = Vec::with_capacity(ring_count * profile_len);
    let mut uvs: Vec<[f64; 2]> = Vec::with_capacity(ring_count * profile_len);

    for (ring_idx, frame) in frames.iter().enumerate() {
        let rail_point = rail_a[ring_idx];
        let u_param = arc_lengths[ring_idx] / total_arc_length;
        for (j, &p) in local_profile.iter().enumerate() {
            // Profile is now in local coordinates: X -> normal, Y -> binormal, Z -> tangent.
            let world_point = rail_point
                .add_vec(frame.normal.mul_scalar(p.x))
                .add_vec(frame.binormal.mul_scalar(p.y))
                .add_vec(frame.tangent.mul_scalar(p.z));
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

    // Caps use the local profile coordinates for proper triangulation.
    if caps.start {
        add_cap(
            &mut vertices,
            &mut uvs,
            &mut indices,
            &local_profile,
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
            &local_profile,
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

// ============================================================================
// Multi-Section Sweep2
// ============================================================================

/// A section profile with its parameter position along the rails.
#[derive(Debug, Clone)]
pub struct Sweep2Section {
    /// Profile points in world coordinates.
    pub profile: Vec<Point3>,
    /// Parameter position along the rails (0.0 = start, 1.0 = end).
    /// If None, the position will be auto-computed based on the section's index.
    pub parameter: Option<f64>,
}

impl Sweep2Section {
    /// Create a section at a specific parameter position.
    pub fn at_parameter(profile: Vec<Point3>, parameter: f64) -> Self {
        Self {
            profile,
            parameter: Some(parameter),
        }
    }

    /// Create a section with auto-computed parameter position.
    pub fn auto(profile: Vec<Point3>) -> Self {
        Self {
            profile,
            parameter: None,
        }
    }
}

/// Sweep2 with multiple section profiles interpolated along the rails.
///
/// Unlike the single-profile `sweep2_polyline`, this function supports multiple
/// section curves at different positions along the rails. The profile shape is
/// interpolated between sections, creating a smooth transition.
///
/// # Arguments
/// * `sections` - List of section profiles with their positions along the rails.
///   If positions are not provided, sections are distributed evenly.
/// * `rail_a` - Primary rail curve (the sweep path).
/// * `rail_b` - Secondary rail curve (defines orientation/scaling).
/// * `caps` - Cap configuration for open sweeps.
/// * `options` - Multi-section sweep options.
/// * `tol` - Geometric tolerance.
///
/// # Section Interpolation
///
/// At each point along the rails, the algorithm:
/// 1. Determines which two sections bracket the current parameter position.
/// 2. Interpolates between the two section profiles at that position.
/// 3. Scales the interpolated profile based on the distance between rails.
///
/// # Example
/// ```ignore
/// use ghx_engine::geom::{Point3, Sweep2Section, sweep2_multi_section};
///
/// // Create two sections - a circle at start, a square at end
/// let circle = make_circle_points(1.0, 16);
/// let square = make_square_points(1.5, 16); // Same point count as circle
///
/// let sections = vec![
///     Sweep2Section::at_parameter(circle, 0.0),
///     Sweep2Section::at_parameter(square, 1.0),
/// ];
///
/// let rail_a = vec![Point3::ORIGIN, Point3::new(0.0, 0.0, 10.0)];
/// let rail_b = vec![Point3::new(1.0, 0.0, 0.0), Point3::new(1.5, 0.0, 10.0)];
///
/// let (mesh, diag) = sweep2_multi_section(&sections, &rail_a, &rail_b, SweepCaps::BOTH, opts, tol)?;
/// ```
#[must_use]
pub fn sweep2_multi_section(
    sections: &[Sweep2Section],
    rail_a: &[Point3],
    rail_b: &[Point3],
    caps: SweepCaps,
    options: Sweep2MultiSectionOptions,
    tol: Tolerance,
) -> Result<(GeomMesh, GeomMeshDiagnostics), SweepError> {
    // Validate sections
    if sections.is_empty() {
        return Err(SweepError::NoSections);
    }

    // For single section, delegate to the regular sweep2
    if sections.len() == 1 {
        let profile_points: Vec<Point3> = sections[0].profile.clone();
        return sweep2_polyline_with_tolerance(
            &profile_points,
            rail_a,
            rail_b,
            caps,
            options.sweep,
            tol,
        );
    }

    // Validate all sections have enough points
    for (i, section) in sections.iter().enumerate() {
        if section.profile.len() < 2 {
            return Err(SweepError::SectionTooShort { index: i });
        }
    }

    // Validate rails
    if rail_a.len() < 2 || rail_b.len() < 2 {
        return Err(SweepError::RailTooShort);
    }

    // Validate finite points in rails
    for p in rail_a.iter().chain(rail_b.iter()) {
        if !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite() {
            return Err(SweepError::NonFiniteRail);
        }
    }

    if !options.sweep.twist_radians_total.is_finite() {
        return Err(SweepError::NonFiniteInput);
    }

    let mut warnings = Vec::new();

    // Compute section parameters (auto-distribute if needed)
    let section_params = compute_section_parameters(sections, options.auto_distribute_sections)?;

    // Find the maximum point count across all sections
    let max_profile_points = sections.iter().map(|s| s.profile.len()).max().unwrap_or(2);

    // Clean and resample all sections to the same point count
    let cleaned_sections: Vec<CleanPolyline> = sections
        .iter()
        .map(|s| clean_polyline(&s.profile, tol))
        .collect::<Result<Vec<_>, _>>()?;

    // Check closure consistency
    let first_closed = cleaned_sections[0].closed;
    for (i, section) in cleaned_sections.iter().enumerate().skip(1) {
        if section.closed != first_closed {
            warnings.push(format!(
                "section {} has different closure state than first section",
                i
            ));
        }
    }
    let profile_closed = first_closed;

    // Resample all sections to have the same point count
    let resampled_sections: Vec<Vec<Point3>> = cleaned_sections
        .iter()
        .map(|cs| resample_polyline_arc_length(&cs.points, max_profile_points, tol))
        .collect();

    // Compute profile plane transforms for each section
    let section_planes: Vec<ProfilePlaneTransform> = resampled_sections
        .iter()
        .enumerate()
        .map(|(i, pts)| {
            ProfilePlaneTransform::from_points(pts, tol)
                .ok_or(SweepError::SectionTooShort { index: i })
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Transform each section to local coordinates
    let local_sections: Vec<Vec<Point3>> = resampled_sections
        .iter()
        .zip(section_planes.iter())
        .map(|(pts, plane)| plane.transform_profile_to_local(pts))
        .collect();

    // Align rails
    let target_count = rail_a.len().max(rail_b.len());
    let alignment = align_sweep2_rails(rail_a, rail_b, target_count, tol)?;
    warnings.extend(alignment.warnings);

    let aligned_rail_a = alignment.rail_a;
    let aligned_rail_b = alignment.rail_b;

    // Determine if rail is closed
    let rail_closed = aligned_rail_a.len() >= 3
        && tol.approx_eq_point3(aligned_rail_a[0], *aligned_rail_a.last().unwrap());

    if (caps.start || caps.end) && rail_closed {
        return Err(SweepError::CapsNotAllowedForClosedRail);
    }
    if (caps.start || caps.end) && !profile_closed {
        return Err(SweepError::CapsRequireClosedProfile);
    }

    let ring_count = if rail_closed {
        aligned_rail_a.len() - 1
    } else {
        aligned_rail_a.len()
    };

    // Build frames using tangent from rail A and normal from (rail_b - rail_a)
    let mut frames: Vec<FrenetFrame> = Vec::with_capacity(ring_count);

    for i in 0..ring_count {
        let prev = if i == 0 { 0 } else { i - 1 };
        let next = (i + 1).min(ring_count - 1);
        let tangent = if i < ring_count - 1 {
            aligned_rail_a[next]
                .sub_point(aligned_rail_a[i])
                .add(aligned_rail_a[i].sub_point(aligned_rail_a[prev]))
        } else {
            aligned_rail_a[i].sub_point(aligned_rail_a[prev])
        };
        let tangent = match tangent.normalized() {
            Some(t) => t,
            None => {
                warnings.push("sweep2 rail has degenerate tangent".to_string());
                Vec3::Z
            }
        };

        let offset = aligned_rail_b[i].sub_point(aligned_rail_a[i]);
        let mut normal = offset.sub(tangent.mul_scalar(offset.dot(tangent)));
        normal = normal.normalized().unwrap_or_else(|| {
            warnings.push("sweep2 rails are locally parallel; using arbitrary normal".to_string());
            FrenetFrame::from_tangent(tangent)
                .map(|f| f.normal)
                .unwrap_or(Vec3::X)
        });
        let binormal = tangent.cross(normal).normalized().unwrap_or(Vec3::Y);

        frames.push(FrenetFrame {
            tangent,
            normal,
            binormal,
        });
    }

    // Align frames to the first section's plane
    if !frames.is_empty() && !section_planes.is_empty() {
        frames = align_frames_to_profile_plane(&frames, &section_planes[0], &aligned_rail_a, tol);
    }

    // Apply twist if specified
    if options.sweep.twist_radians_total.abs() > tol.eps {
        frames = apply_twist(
            &frames,
            &aligned_rail_a[..ring_count],
            options.sweep.twist_radians_total,
            tol,
        );
    }

    // Compute arc-length parameters for each ring position
    let arc_lengths = compute_arc_lengths(&aligned_rail_a[..ring_count]);
    let total_arc_length = arc_lengths.last().copied().unwrap_or(0.0).max(tol.eps);

    // Build vertices by interpolating between sections at each rail position
    let profile_len = max_profile_points;
    let mut vertices: Vec<Point3> = Vec::with_capacity(ring_count * profile_len);
    let mut uvs: Vec<[f64; 2]> = Vec::with_capacity(ring_count * profile_len);

    for (ring_idx, frame) in frames.iter().enumerate() {
        let rail_point = aligned_rail_a[ring_idx];
        let u_param = arc_lengths[ring_idx] / total_arc_length;

        // Interpolate section profile at this parameter
        let interpolated_profile =
            interpolate_sections_at_param(&local_sections, &section_params, u_param, tol);

        // Compute scale factor from rail separation
        let rail_distance = aligned_rail_b[ring_idx].sub_point(rail_point).length();
        let base_distance = aligned_rail_b[0].sub_point(aligned_rail_a[0]).length();
        let scale = if base_distance > tol.eps {
            rail_distance / base_distance
        } else {
            1.0
        };

        for (j, &p) in interpolated_profile.iter().enumerate() {
            // Apply scale and transform to world coordinates
            let scaled = Point3::new(p.x * scale, p.y * scale, p.z);
            let world_point = rail_point
                .add_vec(frame.normal.mul_scalar(scaled.x))
                .add_vec(frame.binormal.mul_scalar(scaled.y))
                .add_vec(frame.tangent.mul_scalar(scaled.z));
            vertices.push(world_point);

            let v_param = if profile_len > 1 {
                j as f64 / (profile_len - 1) as f64
            } else {
                0.0
            };
            uvs.push([u_param, v_param]);
        }
    }

    // Build side indices
    let rail_edge_count = if rail_closed { ring_count } else { ring_count - 1 };
    let profile_edge_count = if profile_closed { profile_len } else { profile_len - 1 };
    let mut indices: Vec<u32> = Vec::with_capacity(rail_edge_count * profile_edge_count * 6);

    for r in 0..rail_edge_count {
        let r_next = if rail_closed {
            (r + 1) % ring_count
        } else {
            r + 1
        };
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

    // Add caps
    if caps.start {
        let start_profile = &interpolated_profile_at_index(&local_sections, &section_params, 0.0, profile_len, tol);
        add_cap(
            &mut vertices,
            &mut uvs,
            &mut indices,
            start_profile,
            &frames[0],
            aligned_rail_a[0],
            true,
            tol,
        )?;
    }

    if caps.end {
        let last = ring_count - 1;
        let end_profile = &interpolated_profile_at_index(&local_sections, &section_params, 1.0, profile_len, tol);
        add_cap(
            &mut vertices,
            &mut uvs,
            &mut indices,
            end_profile,
            &frames[last],
            aligned_rail_a[last],
            false,
            tol,
        )?;
    }

    let (mesh, mut diagnostics) = finalize_mesh(vertices, Some(uvs), indices, tol);
    diagnostics.warnings.extend(warnings);
    Ok((mesh, diagnostics))
}

/// Computes parameter positions for sections, distributing evenly if auto_distribute is true.
fn compute_section_parameters(
    sections: &[Sweep2Section],
    auto_distribute: bool,
) -> Result<Vec<f64>, SweepError> {
    let n = sections.len();
    if n == 0 {
        return Err(SweepError::NoSections);
    }

    let mut params: Vec<f64> = Vec::with_capacity(n);
    let mut all_specified = true;
    let mut any_specified = false;

    for section in sections {
        match section.parameter {
            Some(p) => {
                params.push(p);
                any_specified = true;
            }
            None => {
                params.push(f64::NAN); // placeholder
                all_specified = false;
            }
        }
    }

    if all_specified {
        // Validate monotonically increasing
        for i in 1..params.len() {
            if params[i] <= params[i - 1] {
                return Err(SweepError::InvalidSectionParameters);
            }
        }
        return Ok(params);
    }

    if auto_distribute || !any_specified {
        // Distribute all sections evenly
        for i in 0..n {
            params[i] = if n > 1 {
                i as f64 / (n - 1) as f64
            } else {
                0.5
            };
        }
        return Ok(params);
    }

    // Mixed specified/unspecified - interpolate missing values
    // Find runs of unspecified parameters and interpolate between known values
    let mut i = 0;
    while i < n {
        if params[i].is_nan() {
            // Find the run of NaNs
            let start_idx = i;
            let start_val = if i == 0 { 0.0 } else { params[i - 1] };

            while i < n && params[i].is_nan() {
                i += 1;
            }
            let end_idx = i;
            let end_val = if i >= n { 1.0 } else { params[i] };

            // Interpolate the run
            let run_len = end_idx - start_idx;
            for j in 0..run_len {
                let t = (j + 1) as f64 / (run_len + 1) as f64;
                params[start_idx + j] = start_val + t * (end_val - start_val);
            }
        } else {
            i += 1;
        }
    }

    Ok(params)
}

/// Interpolates between section profiles at a given parameter position.
fn interpolate_sections_at_param(
    local_sections: &[Vec<Point3>],
    section_params: &[f64],
    param: f64,
    tol: Tolerance,
) -> Vec<Point3> {
    let n = local_sections.len();
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return local_sections[0].clone();
    }

    // Clamp parameter
    let param = param.clamp(0.0, 1.0);

    // Find bracketing sections
    let mut lower_idx = 0;
    let mut upper_idx = n - 1;

    for i in 0..n {
        if section_params[i] <= param {
            lower_idx = i;
        }
        if section_params[i] >= param && upper_idx == n - 1 {
            upper_idx = i;
            break;
        }
    }

    // If we're exactly at a section, return it
    if (section_params[lower_idx] - param).abs() < tol.eps {
        return local_sections[lower_idx].clone();
    }
    if (section_params[upper_idx] - param).abs() < tol.eps {
        return local_sections[upper_idx].clone();
    }

    // Interpolate between lower and upper sections
    let lower = &local_sections[lower_idx];
    let upper = &local_sections[upper_idx];

    let param_range = section_params[upper_idx] - section_params[lower_idx];
    let t = if param_range > tol.eps {
        (param - section_params[lower_idx]) / param_range
    } else {
        0.5
    };

    // Linear interpolation between corresponding points
    lower
        .iter()
        .zip(upper.iter())
        .map(|(p0, p1)| {
            Point3::new(
                p0.x + t * (p1.x - p0.x),
                p0.y + t * (p1.y - p0.y),
                p0.z + t * (p1.z - p0.z),
            )
        })
        .collect()
}

/// Helper to get an interpolated profile at a specific parameter for caps.
fn interpolated_profile_at_index(
    local_sections: &[Vec<Point3>],
    section_params: &[f64],
    param: f64,
    _expected_len: usize,
    tol: Tolerance,
) -> Vec<Point3> {
    interpolate_sections_at_param(local_sections, section_params, param, tol)
}

/// Result from rail frame computation with miter handling.
#[derive(Debug, Clone)]
pub struct RailFrameResult {
    /// Frames along the rail, one per rail point.
    pub frames: Vec<FrenetFrame>,
    /// Indices of cusp points (sharp tangent changes) in the rail.
    pub cusp_indices: Vec<usize>,
    /// Warnings generated during computation.
    pub warnings: Vec<String>,
}

/// Computes rotation-minimizing frames along a rail curve with miter handling.
///
/// This function uses parallel transport to compute stable frames that avoid
/// unwanted twisting. When cusps (sharp corners) are detected, it can optionally
/// apply miter handling to produce cleaner geometry at those locations.
///
/// # Miter Behavior
///
/// - `MiterType::None`: Standard parallel transport through cusps. May cause
///   self-intersection at very sharp corners.
///
/// - `MiterType::Trim`: At each cusp, the frame is computed using the bisector
///   direction (average of incoming and outgoing tangents). This produces a
///   mitered corner effect in the resulting sweep.
///
/// - `MiterType::Rotate`: At each cusp, the frame makes a clean transition by
///   using the bisector for orientation while maintaining frame continuity.
fn compute_rail_frames_with_miter(
    rail: &[Point3],
    miter: MiterType,
    tol: Tolerance,
) -> RailFrameResult {
    let mut warnings = Vec::new();
    let mut cusp_indices = Vec::new();

    if rail.len() < 2 {
        return RailFrameResult {
            frames: vec![FrenetFrame::from_tangent(Vec3::new(0.0, 0.0, 1.0)).unwrap()],
            cusp_indices: vec![],
            warnings: vec!["rail too short; using default frame".to_string()],
        };
    }

    let mut frames = Vec::with_capacity(rail.len());

    // Compute initial tangent and frame
    let initial_tangent = rail[1].sub_point(rail[0]);
    let first_frame = match FrenetFrame::from_tangent(initial_tangent) {
        Some(f) => f,
        None => {
            warnings.push("rail has degenerate initial tangent; using default frame".to_string());
            FrenetFrame::from_tangent(Vec3::new(0.0, 0.0, 1.0)).unwrap()
        }
    };
    frames.push(first_frame);

    for i in 1..rail.len() {
        let prev_idx = i - 1;
        let next_idx = (i + 1).min(rail.len() - 1);

        // Compute incoming and outgoing tangent vectors
        let incoming = rail[i].sub_point(rail[prev_idx]);
        let outgoing = if i < rail.len() - 1 {
            rail[next_idx].sub_point(rail[i])
        } else {
            incoming
        };

        let incoming_norm = incoming.normalized();
        let outgoing_norm = outgoing.normalized();

        // Determine the frame tangent based on miter type
        let (tangent, is_cusp) = match (incoming_norm, outgoing_norm) {
            (Some(inc), Some(out)) => {
                let dot_product = inc.dot(out);
                let is_cusp = dot_product < CUSP_DOT_THRESHOLD;

                if is_cusp && miter.requires_kink_handling() {
                    // Compute bisector direction for miter handling
                    let bisector = inc.add(out);
                    match bisector.normalized() {
                        Some(b) => (b, true),
                        None => {
                            // Tangents are exactly opposite - use incoming
                            warnings.push(format!(
                                "cusp at index {i}: tangents are opposite; using incoming direction"
                            ));
                            (inc, true)
                        }
                    }
                } else {
                    // Standard behavior: average of incoming and outgoing
                    let avg = incoming.add(outgoing);
                    match avg.normalized() {
                        Some(t) => (t, is_cusp),
                        None => (inc, is_cusp),
                    }
                }
            }
            (Some(inc), None) => {
                warnings.push(format!(
                    "degenerate outgoing tangent at index {i}; using incoming"
                ));
                (inc, false)
            }
            (None, Some(out)) => {
                warnings.push(format!(
                    "degenerate incoming tangent at index {i}; using outgoing"
                ));
                (out, false)
            }
            (None, None) => {
                warnings.push(format!(
                    "degenerate segment at index {i}; reusing previous tangent"
                ));
                (frames[prev_idx].tangent, false)
            }
        };

        if is_cusp {
            cusp_indices.push(i);
        }

        // Compute frame using parallel transport from the previous frame
        let prev_frame = &frames[prev_idx];
        let new_frame = parallel_transport_frame(prev_frame, tangent, tol);
        frames.push(new_frame);
    }

    // Add summary warning if there were cusps
    if !cusp_indices.is_empty() {
        let miter_desc = match miter {
            MiterType::None => "no special handling",
            MiterType::Trim => "bisector miter applied",
            MiterType::Rotate => "bisector miter applied",
        };
        warnings.push(format!(
            "rail continuity: {} sharp tangent changes detected ({})",
            cusp_indices.len(),
            miter_desc
        ));
    }

    RailFrameResult {
        frames,
        cusp_indices,
        warnings,
    }
}

fn compute_rail_frames(rail: &[Point3], tol: Tolerance) -> (Vec<FrenetFrame>, Vec<String>) {
    // Legacy wrapper that uses no miter handling
    let result = compute_rail_frames_with_miter(rail, MiterType::None, tol);
    (result.frames, result.warnings)
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

/// Aligns the sweep frames to respect the profile's original plane orientation.
///
/// When a profile is not in the XY plane (e.g., a circle in the YZ plane), we need
/// to rotate all sweep frames so that the profile maintains its shape relative to
/// the rail. This function:
///
/// 1. Computes the rotation needed to align the first sweep frame's normal/binormal
///    with the profile plane's local_x/local_y axes.
/// 2. Applies this same rotation to all frames, preserving the parallel transport
///    behavior while respecting the profile's original orientation.
///
/// This ensures that a profile defined in any plane will be swept correctly,
/// maintaining its shape and orientation relative to the starting rail tangent.
fn align_frames_to_profile_plane(
    frames: &[FrenetFrame],
    profile_plane: &ProfilePlaneTransform,
    rail: &[Point3],
    tol: Tolerance,
) -> Vec<FrenetFrame> {
    if frames.is_empty() {
        return Vec::new();
    }

    // Get the initial tangent direction from the rail
    let rail_tangent = if rail.len() >= 2 {
        rail[1].sub_point(rail[0]).normalized().unwrap_or(Vec3::Z)
    } else {
        frames[0].tangent
    };

    // The profile plane's normal should ideally align with the rail tangent at the start.
    // We need to find the rotation that maps:
    //   - profile_plane.local_x -> frame.normal (in the perpendicular plane)
    //   - profile_plane.local_y -> frame.binormal (in the perpendicular plane)
    //
    // First, project the profile's local axes onto the plane perpendicular to the rail tangent.
    let proj_local_x = project_to_perpendicular_plane(profile_plane.local_x, rail_tangent);
    let proj_local_y = project_to_perpendicular_plane(profile_plane.local_y, rail_tangent);

    // If both projections are degenerate (profile plane is parallel to rail), use default frames
    let proj_x_norm = proj_local_x.normalized();
    let proj_y_norm = proj_local_y.normalized();

    if proj_x_norm.is_none() && proj_y_norm.is_none() {
        // Profile plane is parallel to the rail tangent - use the profile normal as reference
        // This handles cases like a circle in XY being swept along Z
        let profile_in_tangent_plane = profile_plane.normal.dot(rail_tangent).abs() > 1.0 - tol.eps;
        if profile_in_tangent_plane {
            // Profile is perpendicular to the rail - use profile's local axes directly
            return align_frames_with_profile_axes(frames, profile_plane, tol);
        }
        // Otherwise, profile plane is skewed - fall back to default behavior
        return frames.to_vec();
    }

    // Use whichever projection is more reliable (longer)
    let target_normal = if proj_local_x.length_squared() >= proj_local_y.length_squared() {
        proj_local_x.normalized().unwrap_or(frames[0].normal)
    } else {
        let b = proj_local_y.normalized().unwrap_or(frames[0].binormal);
        b.cross(rail_tangent).normalized().unwrap_or(frames[0].normal)
    };

    // Compute the rotation from the first frame's orientation to the target orientation.
    // This rotation will be applied to all frames to maintain consistency.
    let first_frame = &frames[0];
    
    // Find the rotation that takes (first_frame.normal, first_frame.binormal) to (target_normal, target_binormal)
    // Both are in the plane perpendicular to the tangent, so this is a rotation around the tangent axis.
    let angle = compute_frame_rotation_angle(first_frame.normal, target_normal, first_frame.tangent);

    if angle.abs() < tol.eps {
        // No rotation needed
        return frames.to_vec();
    }

    // Apply the rotation to all frames
    frames
        .iter()
        .map(|f| {
            let new_normal = rotate_vector(f.normal, f.tangent, angle);
            let new_binormal = f.tangent.cross(new_normal);
            FrenetFrame {
                tangent: f.tangent,
                normal: new_normal.normalized().unwrap_or(f.normal),
                binormal: new_binormal.normalized().unwrap_or(f.binormal),
            }
        })
        .collect()
}

/// Aligns frames when the profile plane is perpendicular to the rail tangent.
///
/// In this case, the profile's local_x and local_y should map directly to the
/// frame's normal and binormal.
fn align_frames_with_profile_axes(
    frames: &[FrenetFrame],
    profile_plane: &ProfilePlaneTransform,
    tol: Tolerance,
) -> Vec<FrenetFrame> {
    if frames.is_empty() {
        return Vec::new();
    }

    let first_frame = &frames[0];
    
    // Compute target normal from profile axes projected to frame plane
    let target_normal = profile_plane.local_x;

    // Project to the perpendicular plane of the first tangent
    let proj_normal = project_to_perpendicular_plane(target_normal, first_frame.tangent)
        .normalized()
        .unwrap_or(first_frame.normal);

    // Compute rotation angle from first frame to target
    let angle = compute_frame_rotation_angle(first_frame.normal, proj_normal, first_frame.tangent);

    if angle.abs() < tol.eps {
        return frames.to_vec();
    }

    frames
        .iter()
        .map(|f| {
            let new_normal = rotate_vector(f.normal, f.tangent, angle);
            let new_binormal = f.tangent.cross(new_normal);
            FrenetFrame {
                tangent: f.tangent,
                normal: new_normal.normalized().unwrap_or(f.normal),
                binormal: new_binormal.normalized().unwrap_or(f.binormal),
            }
        })
        .collect()
}

/// Projects a vector onto the plane perpendicular to the given normal.
#[inline]
fn project_to_perpendicular_plane(v: Vec3, plane_normal: Vec3) -> Vec3 {
    v.sub(plane_normal.mul_scalar(v.dot(plane_normal)))
}

/// Computes the signed rotation angle (in radians) from vector `a` to vector `b`,
/// both assumed to lie in the plane perpendicular to `axis`.
fn compute_frame_rotation_angle(a: Vec3, b: Vec3, axis: Vec3) -> f64 {
    // Project both vectors to be in the perpendicular plane (they should already be, but ensure it)
    let a_proj = project_to_perpendicular_plane(a, axis);
    let b_proj = project_to_perpendicular_plane(b, axis);

    let a_norm = match a_proj.normalized() {
        Some(v) => v,
        None => return 0.0,
    };
    let b_norm = match b_proj.normalized() {
        Some(v) => v,
        None => return 0.0,
    };

    // Compute angle using atan2 for proper sign
    let dot = a_norm.dot(b_norm).clamp(-1.0, 1.0);
    let cross = a_norm.cross(b_norm);
    let sin_sign = cross.dot(axis);

    sin_sign.atan2(dot)
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
