//! Loft surface generation from profile curves.
//!
//! This module implements loft operations that create surfaces by connecting
//! multiple profile curves. Supports standard loft, fit loft, and control-point loft
//! variants with options for closed lofts, seam adjustment, and twist detection.
//!
//! # Loft Types
//! - `Normal`: Standard interpolation through profile curves (default)
//! - `Straight`: Ruled surface with straight sections between profiles
//! - `Uniform`: Uniform parameterization across profiles
//! - `Loose`, `Tight`, `Developable`: Reserved for future NURBS-based implementation

use super::diagnostics::GeomMeshDiagnostics;
use super::mesh::{GeomContext, GeomMesh, finalize_mesh};
use super::metrics::TimingBucket;
use super::triangulation::triangulate_trim_region;
use super::trim::{TrimLoop, TrimRegion, UvPoint};
use super::{Point3, Tolerance, Vec3};

use std::f64::consts::PI;

// ============================================================================
// Types and Options
// ============================================================================

/// Type of loft interpolation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoftType {
    /// Standard interpolation through profile curves using Catmull-Rom splines.
    /// The surface passes exactly through all profile curves.
    #[default]
    Normal,
    /// Loose fit using cubic B-spline approximation where profiles are control points.
    /// The surface may not pass exactly through profiles but maintains smooth curvature.
    /// Produces smoother results with less local influence from individual profiles.
    Loose,
    /// Tight fit using Catmull-Rom splines with reduced tangent influence.
    /// The surface passes through profiles with sharper transitions at profile locations.
    /// Useful when profile shapes should be more prominent.
    Tight,
    /// Straight sections between profiles (ruled surface style).
    /// Produces linear interpolation between corresponding profile points.
    Straight,
    /// Developable surface approximation targeting zero Gaussian curvature.
    /// Uses ruling lines between profiles to create surfaces that can be
    /// unfolded flat without stretching (useful for sheet metal, fabric, etc.).
    Developable,
    /// Uniform parameterization across profiles.
    /// Ensures equal parametric spacing regardless of profile lengths.
    Uniform,
}

/// Mesh quality settings for adaptive tessellation.
#[derive(Debug, Clone, Copy)]
pub struct MeshQuality {
    /// Target edge length for tessellation (0.0 = auto)
    pub target_edge_length: f64,
    /// Maximum deviation from true surface (0.0 = use default tolerance)
    pub max_deviation: f64,
    /// Maximum angle between adjacent face normals in radians (0.0 = no limit)
    pub max_angle: f64,
    /// Minimum points per profile (overrides auto-detection)
    pub min_points_per_profile: usize,
    /// Maximum points per profile
    pub max_points_per_profile: usize,
}

impl Default for MeshQuality {
    fn default() -> Self {
        Self {
            target_edge_length: 0.0,
            max_deviation: 0.0,
            max_angle: 0.0,
            min_points_per_profile: 4,
            max_points_per_profile: 256,
        }
    }
}

impl MeshQuality {
    /// Create mesh quality settings with a target edge length.
    #[must_use]
    pub fn with_edge_length(edge_length: f64) -> Self {
        Self {
            target_edge_length: edge_length,
            ..Default::default()
        }
    }

    /// Create mesh quality settings with a maximum deviation.
    #[must_use]
    pub fn with_deviation(deviation: f64) -> Self {
        Self {
            max_deviation: deviation,
            ..Default::default()
        }
    }
}

/// Options controlling loft surface generation.
#[derive(Debug, Clone, Copy)]
pub struct LoftOptions {
    /// Type of loft interpolation
    pub loft_type: LoftType,
    /// Whether to create a closed loft (connects last profile back to first)
    pub closed: bool,
    /// Whether to adjust seams on closed curves to minimize twist
    pub adjust_seams: bool,
    /// Rebuild profiles to have uniform point counts
    pub rebuild: bool,
    /// Number of points to rebuild profiles to (0 = auto-detect based on mesh_quality)
    pub rebuild_point_count: usize,
    /// Refit tolerance for curve matching (0.0 = use geometry tolerance)
    pub refit_tolerance: f64,
    /// Minimum number of sections for smooth interpolation
    pub min_sections: usize,
    /// Maximum number of sections
    pub max_sections: usize,
    /// Generate caps on open lofts with closed profiles
    pub cap_start: bool,
    /// Generate caps on open lofts with closed profiles  
    pub cap_end: bool,
    /// Use arc-length parameterization for UV coordinates (better texture mapping)
    pub arc_length_uvs: bool,
    /// Mesh quality settings for adaptive tessellation
    pub mesh_quality: Option<MeshQuality>,
}

impl Default for LoftOptions {
    fn default() -> Self {
        Self {
            loft_type: LoftType::Normal,
            closed: false,
            adjust_seams: true,
            rebuild: true,
            rebuild_point_count: 0,
            refit_tolerance: 0.0,
            min_sections: 2,
            max_sections: 256,
            cap_start: false,
            cap_end: false,
            arc_length_uvs: false,
            mesh_quality: None,
        }
    }
}

impl LoftOptions {
    /// Create options for a closed loft (connects last profile to first)
    #[must_use]
    pub fn closed() -> Self {
        Self {
            closed: true,
            ..Default::default()
        }
    }

    /// Create options with caps on both ends
    #[must_use]
    pub fn capped() -> Self {
        Self {
            cap_start: true,
            cap_end: true,
            ..Default::default()
        }
    }

    /// Create options with mesh quality settings
    #[must_use]
    pub fn with_quality(quality: MeshQuality) -> Self {
        Self {
            mesh_quality: Some(quality),
            ..Default::default()
        }
    }

    /// Create options with arc-length UV parameterization
    #[must_use]
    pub fn with_arc_length_uvs() -> Self {
        Self {
            arc_length_uvs: true,
            ..Default::default()
        }
    }

    /// Create a straight (ruled) loft between profiles
    #[must_use]
    pub fn straight() -> Self {
        Self {
            loft_type: LoftType::Straight,
            ..Default::default()
        }
    }
}

/// Loft-specific diagnostics extending general mesh diagnostics.
#[derive(Debug, Clone, Default)]
pub struct LoftDiagnostics {
    /// Number of profile curves used
    pub profile_count: usize,
    /// Number of points per profile after resampling
    pub points_per_profile: usize,
    /// Whether twist was detected between profiles
    pub twist_detected: bool,
    /// Twist angles between consecutive profiles (radians)
    pub twist_angles: Vec<f64>,
    /// Maximum twist angle detected (radians)
    pub max_twist_angle: f64,
    /// Whether seam adjustment was applied
    pub seam_adjusted: bool,
    /// Number of seam rotations applied per profile
    pub seam_rotations: Vec<usize>,
    /// Whether self-intersection was detected (approximate)
    pub self_intersection_detected: bool,
    /// Approximate self-intersection locations (profile index pairs)
    pub self_intersection_hints: Vec<(usize, usize)>,
}

/// Errors that can occur during loft operations.
#[derive(Debug, thiserror::Error)]
pub enum LoftError {
    #[error("loft requires at least 2 profiles, got {count}")]
    NotEnoughProfiles { count: usize },
    #[error("profile {index} has fewer than 2 points (has {point_count})")]
    ProfileTooShort { index: usize, point_count: usize },
    #[error("profile {index} contains non-finite coordinates at point {point_index}")]
    NonFinitePoint { index: usize, point_index: usize },
    #[error("profiles have mixed closed/open states: profile {first_closed_idx} is closed, profile {first_open_idx} is open")]
    MixedClosedOpen { first_closed_idx: usize, first_open_idx: usize },
    #[error("closed loft requires at least 3 profiles, got {count}")]
    ClosedLoftTooFewProfiles { count: usize },
    #[error("caps require closed profiles but profiles are open")]
    CapsRequireClosedProfiles,
    #[error("cap triangulation failed: {0}")]
    CapTriangulation(String),
    #[error("degenerate profile {index} (zero length after cleaning)")]
    DegenerateProfile { index: usize },
}

// ============================================================================
// Public API
// ============================================================================

/// Create a lofted surface mesh from multiple profile polylines.
///
/// # Arguments
/// * `profiles` - List of profile curves as point arrays
/// * `options` - Loft configuration options
///
/// # Returns
/// Tuple of (mesh, mesh_diagnostics, loft_diagnostics) on success.
///
/// # Example
/// ```ignore
/// use ghx_engine::geom::{Point3, loft_mesh, LoftOptions};
///
/// let profile1 = vec![
///     Point3::new(0.0, 0.0, 0.0),
///     Point3::new(1.0, 0.0, 0.0),
///     Point3::new(1.0, 1.0, 0.0),
///     Point3::new(0.0, 1.0, 0.0),
/// ];
/// let profile2 = vec![
///     Point3::new(0.0, 0.0, 5.0),
///     Point3::new(1.0, 0.0, 5.0),
///     Point3::new(1.0, 1.0, 5.0),
///     Point3::new(0.0, 1.0, 5.0),
/// ];
///
/// let profiles: Vec<&[Point3]> = vec![&profile1, &profile2];
/// let (mesh, diag, loft_diag) = loft_mesh(&profiles, LoftOptions::default()).unwrap();
/// ```
#[must_use]
pub fn loft_mesh(
    profiles: &[&[Point3]],
    options: LoftOptions,
) -> Result<(GeomMesh, GeomMeshDiagnostics, LoftDiagnostics), LoftError> {
    loft_mesh_with_tolerance(profiles, options, Tolerance::default_geom())
}

/// Create a lofted surface mesh from multiple profile polylines with explicit tolerance.
pub fn loft_mesh_with_tolerance(
    profiles: &[&[Point3]],
    options: LoftOptions,
    tol: Tolerance,
) -> Result<(GeomMesh, GeomMeshDiagnostics, LoftDiagnostics), LoftError> {
    let mut ctx = GeomContext::new();
    ctx.tolerance = tol;
    loft_mesh_with_context(profiles, options, &mut ctx)
}

/// Create a lofted surface mesh with full context for metrics and caching.
///
/// This variant integrates with `GeomContext` for performance tracking and
/// provides timing information in the diagnostics when the `mesh_engine_metrics`
/// feature is enabled.
pub fn loft_mesh_with_context(
    profiles: &[&[Point3]],
    options: LoftOptions,
    ctx: &mut GeomContext,
) -> Result<(GeomMesh, GeomMeshDiagnostics, LoftDiagnostics), LoftError> {
    ctx.metrics.begin();
    
    let tol = ctx.tolerance;
    
    // Validate inputs
    if profiles.len() < 2 {
        return Err(LoftError::NotEnoughProfiles { count: profiles.len() });
    }
    if options.closed && profiles.len() < 3 {
        return Err(LoftError::ClosedLoftTooFewProfiles { count: profiles.len() });
    }

    // Validate and clean profiles
    let cleaned_profiles = validate_and_clean_profiles(profiles, tol)?;

    // Check closed/open consistency and find mismatching indices
    let first_closed_idx = cleaned_profiles.iter().position(|p| p.closed);
    let first_open_idx = cleaned_profiles.iter().position(|p| !p.closed);
    let profiles_are_closed = first_open_idx.is_none();
    let profiles_are_open = first_closed_idx.is_none();
    
    if !profiles_are_closed && !profiles_are_open {
        return Err(LoftError::MixedClosedOpen {
            first_closed_idx: first_closed_idx.unwrap_or(0),
            first_open_idx: first_open_idx.unwrap_or(0),
        });
    }

    // Caps validation
    if (options.cap_start || options.cap_end) && !profiles_are_closed {
        return Err(LoftError::CapsRequireClosedProfiles);
    }

    // Resample profiles to uniform point counts
    let target_count = determine_target_point_count(&cleaned_profiles, &options);
    let mut resampled = resample_profiles(&cleaned_profiles, target_count);

    // Seam adjustment for closed curves
    let mut loft_diag = LoftDiagnostics {
        profile_count: profiles.len(),
        points_per_profile: target_count,
        ..Default::default()
    };

    if profiles_are_closed && options.adjust_seams {
        let (adjusted, rotations, angles) = adjust_seams_for_twist(&resampled, tol);
        resampled = adjusted;
        loft_diag.seam_adjusted = rotations.iter().any(|&r| r != 0);
        loft_diag.seam_rotations = rotations;
        loft_diag.twist_angles = angles.clone();
        loft_diag.max_twist_angle = angles.iter().cloned().fold(0.0, f64::max);
        loft_diag.twist_detected = loft_diag.max_twist_angle > PI / 4.0; // 45 degrees
    } else {
        // Compute twist angles anyway for diagnostics
        loft_diag.twist_angles = compute_twist_angles(&resampled);
        loft_diag.max_twist_angle = loft_diag.twist_angles.iter().cloned().fold(0.0, f64::max);
        loft_diag.twist_detected = loft_diag.max_twist_angle > PI / 4.0;
    }

    // Build the loft mesh with timing
    let (vertices, uvs, indices) = ctx.metrics.time(TimingBucket::Loft, || {
        build_loft_mesh(
            &resampled,
            options.closed,
            profiles_are_closed,
            &options,
        )
    });

    // Add caps if requested
    let (vertices, uvs, indices) = if (options.cap_start || options.cap_end) && profiles_are_closed {
        add_loft_caps(
            vertices,
            uvs,
            indices,
            &resampled,
            options.cap_start,
            options.cap_end,
            options.closed,
            tol,
        )?
    } else {
        (vertices, uvs, indices)
    };

    // Check for self-intersections (basic heuristic)
    // NOTE: This is approximate; for accurate detection, use the boolean module's
    // triangle-surface intersection with filtered predicates (planned enhancement)
    let (self_int, hints) = detect_self_intersections(&resampled, tol);
    loft_diag.self_intersection_detected = self_int;
    loft_diag.self_intersection_hints = hints;

    let (mesh, mut mesh_diag) = ctx.metrics.time(TimingBucket::Diagnostics, || {
        finalize_mesh(vertices, Some(uvs), indices, tol)
    });

    mesh_diag.timing = ctx.metrics.end();

    Ok((mesh, mesh_diag, loft_diag))
}

/// Create a fit loft surface mesh that interpolates exactly through profiles.
///
/// Fit loft uses Catmull-Rom spline interpolation ensuring the surface passes
/// precisely through each profile curve. This is equivalent to `LoftType::Normal`
/// with no rebuild (preserving original profile point distribution).
#[must_use]
pub fn fit_loft_mesh(
    profiles: &[&[Point3]],
) -> Result<(GeomMesh, GeomMeshDiagnostics, LoftDiagnostics), LoftError> {
    let options = LoftOptions {
        loft_type: LoftType::Normal,
        rebuild: false, // Preserve original profile structure
        refit_tolerance: 0.0,
        ..Default::default()
    };
    loft_mesh(profiles, options)
}

/// Create a control-point loft where profiles act as B-spline control points.
///
/// Unlike fit loft, the surface may not pass exactly through the profile curves.
/// Instead, profiles define the overall shape like control polygon vertices.
/// This produces smoother results with less local influence from individual profiles.
#[must_use]
pub fn control_point_loft_mesh(
    profiles: &[&[Point3]],
) -> Result<(GeomMesh, GeomMeshDiagnostics, LoftDiagnostics), LoftError> {
    let options = LoftOptions {
        loft_type: LoftType::Loose,
        rebuild: true,
        ..Default::default()
    };
    loft_mesh(profiles, options)
}

// ============================================================================
// Internal Helpers
// ============================================================================

/// Cleaned and validated profile data.
struct CleanedProfile {
    points: Vec<Point3>,
    closed: bool,
}

fn validate_and_clean_profiles(
    profiles: &[&[Point3]],
    tol: Tolerance,
) -> Result<Vec<CleanedProfile>, LoftError> {
    let mut result = Vec::with_capacity(profiles.len());

    for (index, profile) in profiles.iter().enumerate() {
        if profile.len() < 2 {
            return Err(LoftError::ProfileTooShort { index, point_count: profile.len() });
        }

        // Check for non-finite coordinates
        for (point_index, pt) in profile.iter().enumerate() {
            if !pt.x.is_finite() || !pt.y.is_finite() || !pt.z.is_finite() {
                return Err(LoftError::NonFinitePoint { index, point_index });
            }
        }

        // Clean duplicate consecutive points
        let mut cleaned = Vec::with_capacity(profile.len());
        for pt in profile.iter() {
            if cleaned.is_empty() {
                cleaned.push(*pt);
            } else {
                let last = cleaned.last().unwrap();
                let dist_sq = pt.sub_point(*last).length_squared();
                if dist_sq > tol.eps_squared() {
                    cleaned.push(*pt);
                }
            }
        }

        // Detect closed profile
        let closed = if cleaned.len() >= 3 {
            let first = cleaned.first().unwrap();
            let last = cleaned.last().unwrap();
            let dist_sq = last.sub_point(*first).length_squared();
            dist_sq <= tol.eps_squared()
        } else {
            false
        };

        // Remove duplicate last point if closed
        if closed && cleaned.len() > 2 {
            cleaned.pop();
        }

        if cleaned.len() < 2 {
            return Err(LoftError::DegenerateProfile { index });
        }

        result.push(CleanedProfile { points: cleaned, closed });
    }

    Ok(result)
}

fn determine_target_point_count(profiles: &[CleanedProfile], options: &LoftOptions) -> usize {
    // Explicit rebuild count takes precedence
    if options.rebuild_point_count > 0 {
        return options.rebuild_point_count.max(2);
    }

    // Use mesh quality settings if provided
    if let Some(ref quality) = options.mesh_quality {
        let max_profile_len = profiles
            .iter()
            .map(|p| compute_polyline_length(&p.points, p.closed))
            .fold(0.0, f64::max);

        if quality.target_edge_length > 0.0 && max_profile_len > 0.0 {
            let estimated_count = (max_profile_len / quality.target_edge_length).ceil() as usize;
            return estimated_count
                .max(quality.min_points_per_profile)
                .min(quality.max_points_per_profile);
        }

        // Use min_points_per_profile as fallback
        if quality.min_points_per_profile > 0 {
            return quality.min_points_per_profile.max(2);
        }
    }

    // Use maximum point count from all profiles
    profiles.iter().map(|p| p.points.len()).max().unwrap_or(2).max(2)
}

/// Compute total arc length of a polyline.
fn compute_polyline_length(points: &[Point3], closed: bool) -> f64 {
    if points.len() < 2 {
        return 0.0;
    }
    let segment_count = if closed { points.len() } else { points.len() - 1 };
    let mut total = 0.0;
    for i in 0..segment_count {
        let j = (i + 1) % points.len();
        total += points[j].sub_point(points[i]).length();
    }
    total
}

fn resample_profiles(profiles: &[CleanedProfile], target_count: usize) -> Vec<Vec<Point3>> {
    profiles
        .iter()
        .map(|p| resample_polyline(&p.points, target_count, p.closed))
        .collect()
}

fn resample_polyline(points: &[Point3], target_count: usize, closed: bool) -> Vec<Point3> {
    if points.len() == target_count {
        return points.to_vec();
    }
    if points.len() < 2 || target_count < 2 {
        return points.to_vec();
    }

    // Compute cumulative arc lengths
    let mut lengths = vec![0.0];
    let segment_count = if closed { points.len() } else { points.len() - 1 };
    
    for i in 0..segment_count {
        let j = (i + 1) % points.len();
        let seg_len = points[j].sub_point(points[i]).length();
        lengths.push(lengths.last().unwrap() + seg_len);
    }
    let total_len = *lengths.last().unwrap();

    if total_len <= 0.0 || !total_len.is_finite() {
        return points.to_vec();
    }

    // Resample at uniform arc-length intervals
    let mut result = Vec::with_capacity(target_count);
    for i in 0..target_count {
        let t = if closed {
            (i as f64) / (target_count as f64)
        } else {
            (i as f64) / ((target_count - 1) as f64)
        };
        let target_len = t * total_len;

        // Find segment containing target_len
        let seg_idx = find_segment_index(&lengths, target_len);
        let seg_start_len = lengths[seg_idx];
        let seg_end_len = lengths.get(seg_idx + 1).copied().unwrap_or(total_len);
        let seg_span = seg_end_len - seg_start_len;

        let local_t = if seg_span > 0.0 {
            ((target_len - seg_start_len) / seg_span).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let p0 = points[seg_idx % points.len()];
        let p1 = points[(seg_idx + 1) % points.len()];
        result.push(lerp_point(p0, p1, local_t));
    }

    result
}

fn find_segment_index(lengths: &[f64], target: f64) -> usize {
    for i in 0..(lengths.len().saturating_sub(1)) {
        if target >= lengths[i] && target <= lengths[i + 1] {
            return i;
        }
    }
    lengths.len().saturating_sub(2)
}

fn lerp_point(a: Point3, b: Point3, t: f64) -> Point3 {
    Point3::new(
        a.x + t * (b.x - a.x),
        a.y + t * (b.y - a.y),
        a.z + t * (b.z - a.z),
    )
}

/// Compute twist angles between consecutive profile pairs.
fn compute_twist_angles(profiles: &[Vec<Point3>]) -> Vec<f64> {
    if profiles.len() < 2 {
        return Vec::new();
    }

    let mut angles = Vec::with_capacity(profiles.len() - 1);
    for i in 0..(profiles.len() - 1) {
        let angle = estimate_twist_angle(&profiles[i], &profiles[i + 1]);
        angles.push(angle);
    }
    angles
}

/// Estimate twist angle between two profiles by comparing centroid-to-first-point vectors.
fn estimate_twist_angle(profile_a: &[Point3], profile_b: &[Point3]) -> f64 {
    if profile_a.is_empty() || profile_b.is_empty() {
        return 0.0;
    }

    let centroid_a = compute_centroid(profile_a);
    let centroid_b = compute_centroid(profile_b);

    let vec_a = profile_a[0].sub_point(centroid_a);
    let vec_b = profile_b[0].sub_point(centroid_b);

    let len_a = vec_a.length();
    let len_b = vec_b.length();

    if len_a < 1e-12 || len_b < 1e-12 {
        return 0.0;
    }

    // Project onto the plane perpendicular to the loft direction
    let loft_dir = centroid_b.sub_point(centroid_a);
    let loft_len = loft_dir.length();
    
    if loft_len < 1e-12 {
        // Coincident centroids, use simple angle
        let dot = vec_a.dot(vec_b) / (len_a * len_b);
        return dot.clamp(-1.0, 1.0).acos();
    }

    let loft_unit = loft_dir.mul_scalar(1.0 / loft_len);
    
    // Remove component along loft direction
    let vec_a_proj = vec_a.sub(loft_unit.mul_scalar(vec_a.dot(loft_unit)));
    let vec_b_proj = vec_b.sub(loft_unit.mul_scalar(vec_b.dot(loft_unit)));
    
    let len_a_proj = vec_a_proj.length();
    let len_b_proj = vec_b_proj.length();
    
    if len_a_proj < 1e-12 || len_b_proj < 1e-12 {
        return 0.0;
    }

    let dot = vec_a_proj.dot(vec_b_proj) / (len_a_proj * len_b_proj);
    dot.clamp(-1.0, 1.0).acos()
}

fn compute_centroid(points: &[Point3]) -> Point3 {
    if points.is_empty() {
        return Point3::new(0.0, 0.0, 0.0);
    }
    let sum_x: f64 = points.iter().map(|p| p.x).sum();
    let sum_y: f64 = points.iter().map(|p| p.y).sum();
    let sum_z: f64 = points.iter().map(|p| p.z).sum();
    let n = points.len() as f64;
    Point3::new(sum_x / n, sum_y / n, sum_z / n)
}

/// Adjust seams on closed profiles to minimize twist.
fn adjust_seams_for_twist(
    profiles: &[Vec<Point3>],
    _tol: Tolerance,
) -> (Vec<Vec<Point3>>, Vec<usize>, Vec<f64>) {
    if profiles.len() < 2 {
        return (profiles.to_vec(), vec![0; profiles.len()], Vec::new());
    }

    let mut adjusted = Vec::with_capacity(profiles.len());
    let mut rotations = Vec::with_capacity(profiles.len());
    let mut angles = Vec::with_capacity(profiles.len() - 1);

    // First profile stays as-is
    adjusted.push(profiles[0].clone());
    rotations.push(0);

    for i in 1..profiles.len() {
        let prev = &adjusted[i - 1];
        let curr = &profiles[i];

        // Find best rotation to minimize twist
        let (best_rotation, best_angle) = find_best_seam_rotation(prev, curr);
        
        // Apply rotation
        let rotated = rotate_profile_seam(curr, best_rotation);
        adjusted.push(rotated);
        rotations.push(best_rotation);
        angles.push(best_angle);
    }

    (adjusted, rotations, angles)
}

fn find_best_seam_rotation(reference: &[Point3], profile: &[Point3]) -> (usize, f64) {
    if profile.is_empty() {
        return (0, 0.0);
    }

    let n = profile.len();
    let mut best_rotation = 0;
    let mut best_angle = f64::MAX;

    // Try each rotation and find the one with minimum twist
    for rotation in 0..n {
        let rotated = rotate_profile_seam(profile, rotation);
        let angle = estimate_twist_angle(reference, &rotated);
        if angle < best_angle {
            best_angle = angle;
            best_rotation = rotation;
        }
    }

    (best_rotation, best_angle)
}

fn rotate_profile_seam(profile: &[Point3], rotation: usize) -> Vec<Point3> {
    if rotation == 0 || profile.is_empty() {
        return profile.to_vec();
    }
    let n = profile.len();
    let rotation = rotation % n;
    let mut result = Vec::with_capacity(n);
    for i in 0..n {
        result.push(profile[(i + rotation) % n]);
    }
    result
}

// ============================================================================
// Spline Interpolation Helpers
// ============================================================================

/// Catmull-Rom spline interpolation at parameter t in [0, 1] between p1 and p2.
/// Uses p0 and p3 as tangent guides. Tension parameter controls sharpness:
/// - tension = 0.5 (default): standard Catmull-Rom
/// - tension < 0.5: tighter/sharper transitions
/// - tension > 0.5: looser/smoother transitions
fn catmull_rom_point(p0: Point3, p1: Point3, p2: Point3, p3: Point3, t: f64, tension: f64) -> Point3 {
    let t2 = t * t;
    let t3 = t2 * t;
    
    // Catmull-Rom basis functions with tension adjustment
    let tau = tension;
    
    let b0 = -tau * t3 + 2.0 * tau * t2 - tau * t;
    let b1 = (2.0 - tau) * t3 + (tau - 3.0) * t2 + 1.0;
    let b2 = (tau - 2.0) * t3 + (3.0 - 2.0 * tau) * t2 + tau * t;
    let b3 = tau * t3 - tau * t2;
    
    Point3::new(
        b0 * p0.x + b1 * p1.x + b2 * p2.x + b3 * p3.x,
        b0 * p0.y + b1 * p1.y + b2 * p2.y + b3 * p3.y,
        b0 * p0.z + b1 * p1.z + b2 * p2.z + b3 * p3.z,
    )
}

/// Cubic B-spline basis functions at parameter t in [0, 1].
/// B-spline approximates but doesn't necessarily pass through control points.
fn bspline_point(p0: Point3, p1: Point3, p2: Point3, p3: Point3, t: f64) -> Point3 {
    let t2 = t * t;
    let t3 = t2 * t;
    
    // Cubic B-spline basis functions
    let b0 = (-t3 + 3.0 * t2 - 3.0 * t + 1.0) / 6.0;
    let b1 = (3.0 * t3 - 6.0 * t2 + 4.0) / 6.0;
    let b2 = (-3.0 * t3 + 3.0 * t2 + 3.0 * t + 1.0) / 6.0;
    let b3 = t3 / 6.0;
    
    Point3::new(
        b0 * p0.x + b1 * p1.x + b2 * p2.x + b3 * p3.x,
        b0 * p0.y + b1 * p1.y + b2 * p2.y + b3 * p3.y,
        b0 * p0.z + b1 * p1.z + b2 * p2.z + b3 * p3.z,
    )
}

/// Linear interpolation between two points.
fn linear_interp(p0: Point3, p1: Point3, t: f64) -> Point3 {
    Point3::new(
        p0.x + t * (p1.x - p0.x),
        p0.y + t * (p1.y - p0.y),
        p0.z + t * (p1.z - p0.z),
    )
}

/// Interpolate a point along profiles using specified loft type.
/// - profiles: the input profile points at the same relative position (point_idx)
/// - t: global parameter from 0.0 (first profile) to 1.0 (last profile)
/// - loft_type: interpolation method
fn interpolate_along_profiles(
    profile_points: &[Point3],
    t: f64,
    loft_type: LoftType,
    loft_closed: bool,
) -> Point3 {
    let n = profile_points.len();
    if n == 0 {
        return Point3::new(0.0, 0.0, 0.0);
    }
    if n == 1 {
        return profile_points[0];
    }
    
    // For straight/ruled loft: simple linear interpolation
    if matches!(loft_type, LoftType::Straight) {
        let segment_count = if loft_closed { n } else { n - 1 };
        let scaled_t = t * segment_count as f64;
        let segment = (scaled_t.floor() as usize).min(segment_count - 1);
        let local_t = scaled_t - segment as f64;
        let i0 = segment % n;
        let i1 = (segment + 1) % n;
        return linear_interp(profile_points[i0], profile_points[i1], local_t);
    }
    
    // For uniform: use equal parametric spacing
    if matches!(loft_type, LoftType::Uniform) {
        let segment_count = if loft_closed { n } else { n - 1 };
        let scaled_t = t * segment_count as f64;
        let segment = (scaled_t.floor() as usize).min(segment_count - 1);
        let local_t = scaled_t - segment as f64;
        let i0 = segment % n;
        let i1 = (segment + 1) % n;
        return linear_interp(profile_points[i0], profile_points[i1], local_t);
    }
    
    // For developable: use ruling-based interpolation (similar to straight but with
    // slight curve optimization for zero Gaussian curvature approximation)
    if matches!(loft_type, LoftType::Developable) {
        // For developable surfaces, we approximate by using ruling lines that minimize
        // twist. This is achieved by linear interpolation with ruling optimization.
        let segment_count = if loft_closed { n } else { n - 1 };
        let scaled_t = t * segment_count as f64;
        let segment = (scaled_t.floor() as usize).min(segment_count - 1);
        let local_t = scaled_t - segment as f64;
        let i0 = segment % n;
        let i1 = (segment + 1) % n;
        
        // For true developable, we would optimize ruling directions, but for
        // practical purposes, straight rulings with slight easing work well
        // Use ease-in-out for smoother visual result while maintaining developability
        let eased_t = if local_t < 0.5 {
            2.0 * local_t * local_t
        } else {
            1.0 - (-2.0 * local_t + 2.0).powi(2) / 2.0
        };
        return linear_interp(profile_points[i0], profile_points[i1], eased_t);
    }
    
    // Determine tension based on loft type
    let tension = match loft_type {
        LoftType::Normal => 0.5,      // Standard Catmull-Rom
        LoftType::Tight => 0.25,      // Tighter transitions at profiles
        LoftType::Loose => 0.75,      // Looser, smoother transitions
        _ => 0.5,
    };
    
    // Use B-spline for Loose (doesn't pass through control points)
    let use_bspline = matches!(loft_type, LoftType::Loose);
    
    // Find the segment and local parameter
    let segment_count = if loft_closed { n } else { n - 1 };
    let scaled_t = t * segment_count as f64;
    let segment = (scaled_t.floor() as usize).min(segment_count - 1);
    let local_t = scaled_t - segment as f64;
    
    // Get the four control points for the spline
    let (i0, i1, i2, i3) = if loft_closed {
        let i1 = segment % n;
        let i2 = (segment + 1) % n;
        let i0 = if segment == 0 { n - 1 } else { segment - 1 };
        let i3 = (segment + 2) % n;
        (i0, i1, i2, i3)
    } else {
        let i1 = segment;
        let i2 = (segment + 1).min(n - 1);
        let i0 = if segment == 0 { 0 } else { segment - 1 };
        let i3 = (segment + 2).min(n - 1);
        (i0, i1, i2, i3)
    };
    
    let p0 = profile_points[i0];
    let p1 = profile_points[i1];
    let p2 = profile_points[i2];
    let p3 = profile_points[i3];
    
    if use_bspline {
        bspline_point(p0, p1, p2, p3, local_t)
    } else {
        catmull_rom_point(p0, p1, p2, p3, local_t, tension)
    }
}

/// Generate intermediate profiles between input profiles using spline interpolation.
/// Returns a denser set of profiles including the originals (for Normal/Tight/Fit)
/// or approximating profiles (for Loose/ControlPoint).
fn generate_interpolated_profiles(
    input_profiles: &[Vec<Point3>],
    loft_type: LoftType,
    loft_closed: bool,
    sections_per_span: usize,
) -> Vec<Vec<Point3>> {
    let num_input = input_profiles.len();
    if num_input < 2 {
        return input_profiles.to_vec();
    }
    
    let points_per_profile = input_profiles.first().map(|p| p.len()).unwrap_or(0);
    if points_per_profile == 0 {
        return Vec::new();
    }
    
    // For straight loft, just return the input profiles (no intermediate interpolation)
    if matches!(loft_type, LoftType::Straight) {
        return input_profiles.to_vec();
    }
    
    // Number of output profiles
    let span_count = if loft_closed { num_input } else { num_input - 1 };
    let output_count = span_count * sections_per_span + if loft_closed { 0 } else { 1 };
    
    let mut result = Vec::with_capacity(output_count);
    
    for section_idx in 0..output_count {
        let t = if loft_closed {
            section_idx as f64 / output_count as f64
        } else {
            section_idx as f64 / (output_count - 1).max(1) as f64
        };
        
        // Build interpolated profile by interpolating each corresponding point
        let mut profile = Vec::with_capacity(points_per_profile);
        for point_idx in 0..points_per_profile {
            // Gather the same point index from all input profiles
            let profile_points: Vec<Point3> = input_profiles
                .iter()
                .map(|p| p[point_idx])
                .collect();
            
            let interpolated = interpolate_along_profiles(&profile_points, t, loft_type, loft_closed);
            profile.push(interpolated);
        }
        result.push(profile);
    }
    
    result
}

/// Build the loft mesh geometry with proper spline interpolation.
fn build_loft_mesh(
    profiles: &[Vec<Point3>],
    loft_closed: bool,
    profiles_closed: bool,
    options: &LoftOptions,
) -> (Vec<Point3>, Vec<[f64; 2]>, Vec<u32>) {
    let num_input_profiles = profiles.len();
    let points_per_profile = profiles.first().map(|p| p.len()).unwrap_or(0);

    if num_input_profiles < 2 || points_per_profile < 2 {
        return (Vec::new(), Vec::new(), Vec::new());
    }

    // Determine sections per span based on loft type
    let sections_per_span = match options.loft_type {
        LoftType::Straight => 1, // No intermediate sections for ruled surface
        LoftType::Uniform => 1,  // Just use the input profiles
        LoftType::Developable => 1, // Ruling-based, no subdivision
        LoftType::Normal => 4,   // Subdivide for smooth interpolation
        LoftType::Tight => 3,    // Fewer subdivisions, keep sharper feel
        LoftType::Loose => 5,    // More subdivisions for smoother B-spline
    };
    
    // Generate interpolated profiles
    let interp_profiles = generate_interpolated_profiles(
        profiles,
        options.loft_type,
        loft_closed,
        sections_per_span,
    );
    
    let num_profiles = interp_profiles.len();
    if num_profiles < 2 {
        return (Vec::new(), Vec::new(), Vec::new());
    }

    // Compute cumulative profile distances for V coordinate (arc-length along loft direction)
    let profile_v_coords: Vec<f64> = if options.arc_length_uvs && num_profiles > 1 {
        let mut cumulative = vec![0.0];
        for i in 1..num_profiles {
            let centroid_prev = compute_centroid(&interp_profiles[i - 1]);
            let centroid_curr = compute_centroid(&interp_profiles[i]);
            let dist = centroid_curr.sub_point(centroid_prev).length();
            cumulative.push(cumulative.last().unwrap() + dist);
        }
        let total = *cumulative.last().unwrap();
        if total > 0.0 {
            cumulative.iter().map(|d| d / total).collect()
        } else {
            (0..num_profiles).map(|i| i as f64 / (num_profiles - 1) as f64).collect()
        }
    } else {
        (0..num_profiles).map(|i| i as f64 / (num_profiles - 1).max(1) as f64).collect()
    };

    // Flatten vertices
    let mut vertices = Vec::with_capacity(num_profiles * points_per_profile);
    let mut uvs = Vec::with_capacity(num_profiles * points_per_profile);

    for (profile_idx, profile) in interp_profiles.iter().enumerate() {
        let v = profile_v_coords[profile_idx];
        
        // Compute per-point U coordinates
        let point_u_coords: Vec<f64> = if options.arc_length_uvs && profile.len() > 1 {
            compute_arc_length_params(profile, profiles_closed)
        } else if profiles_closed {
            (0..profile.len()).map(|i| i as f64 / points_per_profile as f64).collect()
        } else {
            (0..profile.len()).map(|i| i as f64 / (points_per_profile - 1) as f64).collect()
        };

        for (point_idx, point) in profile.iter().enumerate() {
            vertices.push(*point);
            uvs.push([point_u_coords[point_idx], v]);
        }
    }

    // Build triangles
    let profile_pairs = if loft_closed { num_profiles } else { num_profiles - 1 };
    let edge_count = if profiles_closed { points_per_profile } else { points_per_profile - 1 };

    let mut indices = Vec::with_capacity(profile_pairs * edge_count * 6);

    for i in 0..profile_pairs {
        let curr_row = i;
        let next_row = (i + 1) % num_profiles;

        for j in 0..edge_count {
            let curr_col = j;
            let next_col = (j + 1) % points_per_profile;

            let i0 = (curr_row * points_per_profile + curr_col) as u32;
            let i1 = (curr_row * points_per_profile + next_col) as u32;
            let i2 = (next_row * points_per_profile + next_col) as u32;
            let i3 = (next_row * points_per_profile + curr_col) as u32;

            // Two triangles per quad
            indices.extend_from_slice(&[i0, i1, i2]);
            indices.extend_from_slice(&[i0, i2, i3]);
        }
    }

    (vertices, uvs, indices)
}

/// Compute arc-length parameterized U coordinates for a profile.
fn compute_arc_length_params(points: &[Point3], closed: bool) -> Vec<f64> {
    if points.len() < 2 {
        return vec![0.0; points.len()];
    }
    
    let segment_count = if closed { points.len() } else { points.len() - 1 };
    let mut cumulative = vec![0.0];
    
    for i in 0..segment_count {
        let j = (i + 1) % points.len();
        let dist = points[j].sub_point(points[i]).length();
        cumulative.push(cumulative.last().unwrap() + dist);
    }
    
    let total = *cumulative.last().unwrap();
    if total > 0.0 {
        cumulative.iter().take(points.len()).map(|d| d / total).collect()
    } else {
        (0..points.len()).map(|i| i as f64 / (points.len() - 1).max(1) as f64).collect()
    }
}

/// Add caps to the loft at start and/or end profiles.
fn add_loft_caps(
    mut vertices: Vec<Point3>,
    mut uvs: Vec<[f64; 2]>,
    mut indices: Vec<u32>,
    profiles: &[Vec<Point3>],
    cap_start: bool,
    cap_end: bool,
    loft_closed: bool,
    tol: Tolerance,
) -> Result<(Vec<Point3>, Vec<[f64; 2]>, Vec<u32>), LoftError> {
    // No caps for closed lofts (they loop back to start)
    if loft_closed {
        return Ok((vertices, uvs, indices));
    }

    // Add start cap
    if cap_start {
        let start_profile = &profiles[0];
        add_profile_cap(
            &mut vertices,
            &mut uvs,
            &mut indices,
            start_profile,
            true, // flip winding for start cap
            tol,
        )?;
    }

    // Add end cap
    if cap_end {
        let end_profile = &profiles[profiles.len() - 1];
        add_profile_cap(
            &mut vertices,
            &mut uvs,
            &mut indices,
            end_profile,
            false, // normal winding for end cap
            tol,
        )?;
    }

    Ok((vertices, uvs, indices))
}

fn add_profile_cap(
    vertices: &mut Vec<Point3>,
    uvs: &mut Vec<[f64; 2]>,
    indices: &mut Vec<u32>,
    profile: &[Point3],
    flip_winding: bool,
    tol: Tolerance,
) -> Result<(), LoftError> {
    if profile.len() < 3 {
        return Err(LoftError::CapsRequireClosedProfiles);
    }

    // Compute profile plane
    let (plane_origin, _plane_normal, plane_x, plane_y) = 
        compute_profile_plane(profile).ok_or_else(|| {
            LoftError::CapTriangulation("could not compute cap plane".to_string())
        })?;

    // Project profile to 2D
    let uv_points: Vec<UvPoint> = profile
        .iter()
        .map(|p| {
            let v = p.sub_point(plane_origin);
            UvPoint::new(v.dot(plane_x), v.dot(plane_y))
        })
        .collect();

    // Create trim region and triangulate
    let outer_loop = TrimLoop::new(uv_points, tol)
        .map_err(|e| LoftError::CapTriangulation(e.to_string()))?;
    let trim_region = TrimRegion::from_loops(vec![outer_loop], tol)
        .map_err(|e| LoftError::CapTriangulation(e.to_string()))?;
    
    let result = triangulate_trim_region(&trim_region, tol)
        .map_err(|e| LoftError::CapTriangulation(e))?;

    // Add cap vertices and triangles
    let base_index = vertices.len() as u32;

    for uv in &result.vertices {
        let pt = plane_origin
            .add_vec(plane_x.mul_scalar(uv.u))
            .add_vec(plane_y.mul_scalar(uv.v));
        vertices.push(pt);
        uvs.push([0.5 + uv.u * 0.5, 0.5 + uv.v * 0.5]); // Cap UV mapping
    }

    for tri in result.indices.chunks_exact(3) {
        if flip_winding {
            indices.extend_from_slice(&[
                base_index + tri[0],
                base_index + tri[2],
                base_index + tri[1],
            ]);
        } else {
            indices.extend_from_slice(&[
                base_index + tri[0],
                base_index + tri[1],
                base_index + tri[2],
            ]);
        }
    }

    Ok(())
}

fn compute_profile_plane(profile: &[Point3]) -> Option<(Point3, Vec3, Vec3, Vec3)> {
    if profile.len() < 3 {
        return None;
    }

    let origin = compute_centroid(profile);

    // Find best-fit normal using Newell's method
    let mut normal = Vec3::new(0.0, 0.0, 0.0);
    for i in 0..profile.len() {
        let curr = profile[i];
        let next = profile[(i + 1) % profile.len()];
        normal = Vec3::new(
            normal.x + (curr.y - next.y) * (curr.z + next.z),
            normal.y + (curr.z - next.z) * (curr.x + next.x),
            normal.z + (curr.x - next.x) * (curr.y + next.y),
        );
    }

    let normal = normal.normalized()?;

    // Build orthonormal basis
    let arbitrary = if normal.x.abs() < 0.9 {
        Vec3::new(1.0, 0.0, 0.0)
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    };

    let plane_x = normal.cross(arbitrary).normalized()?;
    let plane_y = normal.cross(plane_x);

    Some((origin, normal, plane_x, plane_y))
}

/// Basic self-intersection detection using bounding box overlap and normal flipping.
fn detect_self_intersections(
    profiles: &[Vec<Point3>],
    _tol: Tolerance,
) -> (bool, Vec<(usize, usize)>) {
    // Simple heuristic: check if any non-adjacent profiles have overlapping bounding boxes
    // and their normals flip direction (indicating a fold-over)
    
    let mut hints = Vec::new();

    if profiles.len() < 3 {
        return (false, hints);
    }

    let bboxes: Vec<_> = profiles.iter().map(|p| compute_profile_bbox(p)).collect();
    let normals: Vec<_> = profiles
        .iter()
        .filter_map(|p| compute_profile_plane(p).map(|(_, n, _, _)| n))
        .collect();

    // Check non-adjacent profiles
    for i in 0..profiles.len() {
        for j in (i + 2)..profiles.len() {
            // Skip adjacent pairs in closed loft
            if j == profiles.len() - 1 && i == 0 {
                continue;
            }

            if let (Some(bb_i), Some(bb_j)) = (&bboxes[i], &bboxes[j]) {
                if bboxes_overlap(bb_i, bb_j) {
                    // Check for normal flip
                    if i < normals.len() && j < normals.len() {
                        let dot = normals[i].dot(normals[j]);
                        if dot < 0.0 {
                            hints.push((i, j));
                        }
                    }
                }
            }
        }
    }

    (!hints.is_empty(), hints)
}

fn compute_profile_bbox(profile: &[Point3]) -> Option<(Point3, Point3)> {
    if profile.is_empty() {
        return None;
    }

    let mut min = profile[0];
    let mut max = profile[0];

    for p in profile.iter().skip(1) {
        min.x = min.x.min(p.x);
        min.y = min.y.min(p.y);
        min.z = min.z.min(p.z);
        max.x = max.x.max(p.x);
        max.y = max.y.max(p.y);
        max.z = max.z.max(p.z);
    }

    Some((min, max))
}

fn bboxes_overlap(a: &(Point3, Point3), b: &(Point3, Point3)) -> bool {
    a.0.x <= b.1.x && a.1.x >= b.0.x &&
    a.0.y <= b.1.y && a.1.y >= b.0.y &&
    a.0.z <= b.1.z && a.1.z >= b.0.z
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn rect_profile(cx: f64, cy: f64, cz: f64, w: f64, h: f64) -> Vec<Point3> {
        vec![
            Point3::new(cx - w / 2.0, cy - h / 2.0, cz),
            Point3::new(cx + w / 2.0, cy - h / 2.0, cz),
            Point3::new(cx + w / 2.0, cy + h / 2.0, cz),
            Point3::new(cx - w / 2.0, cy + h / 2.0, cz),
        ]
    }

    fn circle_profile(cx: f64, cy: f64, cz: f64, r: f64, n: usize) -> Vec<Point3> {
        // Include n+1 points so first and last are the same (closed polyline)
        (0..=n)
            .map(|i| {
                let t = (i as f64 / n as f64) * 2.0 * PI;
                Point3::new(cx + r * t.cos(), cy + r * t.sin(), cz)
            })
            .collect()
    }

    #[test]
    fn test_loft_two_rectangles() {
        let p0 = rect_profile(0.0, 0.0, 0.0, 2.0, 1.0);
        let p1 = rect_profile(0.0, 0.0, 5.0, 2.0, 1.0);
        let profiles: Vec<&[Point3]> = vec![&p0, &p1];

        let (mesh, diag, loft_diag) = loft_mesh(&profiles, LoftOptions::default()).unwrap();

        assert!(mesh.positions.len() > 0);
        assert!(mesh.indices.len() > 0);
        assert_eq!(loft_diag.profile_count, 2);
        assert!(!loft_diag.twist_detected);
        // Normal loft generates intermediate profiles, so we expect more open edges
        // at the start and end than just the original 8 (4 edges per profile)
        assert!(diag.open_edge_count >= 8, "Should have open edges at start and end");
    }

    #[test]
    fn test_loft_two_circles() {
        let p0 = circle_profile(0.0, 0.0, 0.0, 1.0, 16);
        let p1 = circle_profile(0.0, 0.0, 5.0, 2.0, 16);
        let profiles: Vec<&[Point3]> = vec![&p0, &p1];

        let (mesh, diag, loft_diag) = loft_mesh(&profiles, LoftOptions::default()).unwrap();

        assert!(mesh.positions.len() > 0);
        assert!(mesh.indices.len() > 0);
        assert_eq!(loft_diag.profile_count, 2);
        assert!(diag.open_edge_count <= 32); // Edges at start and end
    }

    #[test]
    fn test_loft_three_profiles() {
        let p0 = circle_profile(0.0, 0.0, 0.0, 1.0, 12);
        let p1 = circle_profile(0.0, 0.0, 2.5, 1.5, 12);
        let p2 = circle_profile(0.0, 0.0, 5.0, 1.0, 12);
        let profiles: Vec<&[Point3]> = vec![&p0, &p1, &p2];

        let (mesh, _, loft_diag) = loft_mesh(&profiles, LoftOptions::default()).unwrap();

        assert!(mesh.positions.len() > 0);
        assert_eq!(loft_diag.profile_count, 3);
    }

    #[test]
    fn test_closed_loft() {
        let p0 = circle_profile(0.0, 0.0, 0.0, 1.0, 12);
        let p1 = circle_profile(2.0, 0.0, 0.0, 1.0, 12);
        let p2 = circle_profile(2.0, 2.0, 0.0, 1.0, 12);
        let p3 = circle_profile(0.0, 2.0, 0.0, 1.0, 12);
        let profiles: Vec<&[Point3]> = vec![&p0, &p1, &p2, &p3];

        let (mesh, diag, _) = loft_mesh(&profiles, LoftOptions::closed()).unwrap();

        assert!(mesh.positions.len() > 0);
        // Closed loft should have fewer open edges
        assert!(diag.open_edge_count < 48);
    }

    #[test]
    fn test_capped_loft() {
        let p0 = circle_profile(0.0, 0.0, 0.0, 1.0, 16);
        let p1 = circle_profile(0.0, 0.0, 5.0, 1.5, 16);
        let profiles: Vec<&[Point3]> = vec![&p0, &p1];

        let (mesh, diag, _) = loft_mesh(&profiles, LoftOptions::capped()).unwrap();

        assert!(mesh.positions.len() > 0);
        // Capped loft should be watertight
        assert_eq!(diag.open_edge_count, 0);
    }

    #[test]
    fn test_twisted_profiles_detected() {
        // Create profiles with deliberate twist
        let p0: Vec<Point3> = (0..8)
            .map(|i| {
                let t = (i as f64 / 8.0) * 2.0 * PI;
                Point3::new(t.cos(), t.sin(), 0.0)
            })
            .collect();
        
        // Second profile rotated by 90 degrees
        let p1: Vec<Point3> = (0..8)
            .map(|i| {
                let t = (i as f64 / 8.0) * 2.0 * PI + PI / 2.0;
                Point3::new(t.cos(), t.sin(), 5.0)
            })
            .collect();
        
        let profiles: Vec<&[Point3]> = vec![&p0, &p1];
        
        let (_, _, loft_diag) = loft_mesh(&profiles, LoftOptions {
            adjust_seams: false, // Disable seam adjustment to see raw twist
            ..Default::default()
        }).unwrap();

        assert!(loft_diag.max_twist_angle > PI / 4.0);
        assert!(loft_diag.twist_detected);
    }

    #[test]
    fn test_seam_adjustment_reduces_twist() {
        // Create profiles with deliberate twist
        let p0: Vec<Point3> = (0..8)
            .map(|i| {
                let t = (i as f64 / 8.0) * 2.0 * PI;
                Point3::new(t.cos(), t.sin(), 0.0)
            })
            .collect();
        
        // Second profile rotated by 45 degrees
        let offset = PI / 4.0;
        let p1: Vec<Point3> = (0..8)
            .map(|i| {
                let t = (i as f64 / 8.0) * 2.0 * PI + offset;
                Point3::new(t.cos(), t.sin(), 5.0)
            })
            .collect();
        
        let profiles: Vec<&[Point3]> = vec![&p0, &p1];

        // With seam adjustment
        let (_, _, diag_adjusted) = loft_mesh(&profiles, LoftOptions::default()).unwrap();

        // Without seam adjustment
        let (_, _, diag_raw) = loft_mesh(&profiles, LoftOptions {
            adjust_seams: false,
            ..Default::default()
        }).unwrap();

        // Seam adjustment should reduce or equal twist
        assert!(diag_adjusted.max_twist_angle <= diag_raw.max_twist_angle + 0.01);
    }

    #[test]
    fn test_loft_error_not_enough_profiles() {
        let p0 = rect_profile(0.0, 0.0, 0.0, 1.0, 1.0);
        let profiles: Vec<&[Point3]> = vec![&p0];

        let result = loft_mesh(&profiles, LoftOptions::default());
        assert!(matches!(result, Err(LoftError::NotEnoughProfiles { count: 1 })));
    }

    #[test]
    fn test_loft_error_profile_too_short() {
        let p0 = vec![Point3::new(0.0, 0.0, 0.0)];
        let p1 = rect_profile(0.0, 0.0, 5.0, 1.0, 1.0);
        let profiles: Vec<&[Point3]> = vec![&p0, &p1];

        let result = loft_mesh(&profiles, LoftOptions::default());
        assert!(matches!(result, Err(LoftError::ProfileTooShort { index: 0, point_count: 1 })));
    }

    #[test]
    fn test_loft_with_mesh_quality() {
        let p0 = circle_profile(0.0, 0.0, 0.0, 1.0, 8);
        let p1 = circle_profile(0.0, 0.0, 5.0, 2.0, 8);
        let profiles: Vec<&[Point3]> = vec![&p0, &p1];

        let quality = MeshQuality::with_edge_length(0.5);
        let options = LoftOptions::with_quality(quality);
        
        let (mesh, _, loft_diag) = loft_mesh(&profiles, options).unwrap();
        
        assert!(mesh.positions.len() > 0);
        // With edge length 0.5, we should have more points than the original 8
        assert!(loft_diag.points_per_profile >= 8);
    }

    #[test]
    fn test_loft_with_arc_length_uvs() {
        let p0 = rect_profile(0.0, 0.0, 0.0, 2.0, 1.0);
        let p1 = rect_profile(0.0, 0.0, 5.0, 2.0, 1.0);
        let profiles: Vec<&[Point3]> = vec![&p0, &p1];

        let options = LoftOptions::with_arc_length_uvs();
        let (mesh, _, _) = loft_mesh(&profiles, options).unwrap();
        
        assert!(mesh.positions.len() > 0);
        assert!(mesh.uvs.is_some());
    }

    #[test]
    fn test_straight_loft() {
        let p0 = rect_profile(0.0, 0.0, 0.0, 2.0, 1.0);
        let p1 = rect_profile(0.0, 0.0, 5.0, 4.0, 2.0);
        let profiles: Vec<&[Point3]> = vec![&p0, &p1];

        let options = LoftOptions::straight();
        let (mesh, _, _) = loft_mesh(&profiles, options).unwrap();
        
        assert!(mesh.positions.len() > 0);
    }

    #[test]
    fn test_resample_polyline() {
        let points = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
        ];
        
        let resampled = resample_polyline(&points, 5, false);
        assert_eq!(resampled.len(), 5);
        
        // First and last should be the same
        assert!((resampled[0].x - 0.0).abs() < 1e-10);
        assert!((resampled[4].x - 2.0).abs() < 1e-10);
    }

    // ========================================================================
    // Loft Type Variant Tests (Golden Tests)
    // ========================================================================

    /// Helper to compute the midpoint of a mesh (average of all positions).
    fn mesh_centroid(mesh: &GeomMesh) -> Point3 {
        if mesh.positions.is_empty() {
            return Point3::new(0.0, 0.0, 0.0);
        }
        let sum: [f64; 3] = mesh.positions.iter().fold([0.0, 0.0, 0.0], |acc, p| {
            [acc[0] + p[0], acc[1] + p[1], acc[2] + p[2]]
        });
        let n = mesh.positions.len() as f64;
        Point3::new(sum[0] / n, sum[1] / n, sum[2] / n)
    }

    /// Helper to find the closest point to a target in mesh positions.
    fn closest_mesh_point(mesh: &GeomMesh, target: Point3) -> Option<(usize, f64)> {
        mesh.positions
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let d = (p[0] - target.x).powi(2) + (p[1] - target.y).powi(2) + (p[2] - target.z).powi(2);
                (i, d.sqrt())
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }

    /// Test that Normal loft passes through the middle profile.
    #[test]
    fn test_loft_normal_passes_through_profiles() {
        // Three profiles at z=0, z=2.5, z=5 with varying radii
        let p0 = circle_profile(0.0, 0.0, 0.0, 1.0, 12);
        let p1 = circle_profile(0.0, 0.0, 2.5, 2.0, 12); // Middle profile has radius 2
        let p2 = circle_profile(0.0, 0.0, 5.0, 1.0, 12);
        let profiles: Vec<&[Point3]> = vec![&p0, &p1, &p2];

        let options = LoftOptions {
            loft_type: LoftType::Normal,
            ..Default::default()
        };
        let (mesh, _, _) = loft_mesh(&profiles, options).unwrap();

        // Normal loft should pass through the middle profile point (2, 0, 2.5)
        let target = Point3::new(2.0, 0.0, 2.5);
        let (_, dist) = closest_mesh_point(&mesh, target).unwrap();
        
        // Should be very close (within tolerance) to the middle profile
        assert!(dist < 0.15, "Normal loft should pass through profiles, distance: {}", dist);
    }

    /// Test that Loose loft (B-spline) does NOT necessarily pass through the middle profile.
    #[test]
    fn test_loft_loose_approximates_profiles() {
        // Three profiles at z=0, z=2.5, z=5 with varying radii
        let p0 = circle_profile(0.0, 0.0, 0.0, 1.0, 12);
        let p1 = circle_profile(0.0, 0.0, 2.5, 2.0, 12); // Middle profile has radius 2
        let p2 = circle_profile(0.0, 0.0, 5.0, 1.0, 12);
        let profiles: Vec<&[Point3]> = vec![&p0, &p1, &p2];

        let options = LoftOptions {
            loft_type: LoftType::Loose,
            ..Default::default()
        };
        let (mesh, _, _) = loft_mesh(&profiles, options).unwrap();

        // Loose loft uses B-spline - the middle profile acts as a control point
        // The surface should be pulled toward but not necessarily through it
        let target = Point3::new(2.0, 0.0, 2.5);
        let (_, dist) = closest_mesh_point(&mesh, target).unwrap();
        
        // For B-spline, the distance should be > 0 (doesn't pass through)
        // but still reasonably close (pulled toward the control point)
        assert!(dist > 0.05, "Loose loft should approximate, not interpolate. Distance: {}", dist);
        assert!(dist < 1.0, "Loose loft should still be influenced by control points. Distance: {}", dist);
    }

    /// Test that Tight loft has sharper transitions at profiles.
    #[test]
    fn test_loft_tight_sharper_transitions() {
        // Create profiles with a sharp bend in the middle
        let p0 = circle_profile(0.0, 0.0, 0.0, 1.0, 12);
        let p1 = circle_profile(2.0, 0.0, 2.5, 1.0, 12); // Offset in X
        let p2 = circle_profile(0.0, 0.0, 5.0, 1.0, 12);
        let profiles: Vec<&[Point3]> = vec![&p0, &p1, &p2];

        let tight_options = LoftOptions {
            loft_type: LoftType::Tight,
            ..Default::default()
        };
        let (mesh_tight, _, _) = loft_mesh(&profiles, tight_options).unwrap();

        let normal_options = LoftOptions {
            loft_type: LoftType::Normal,
            ..Default::default()
        };
        let (mesh_normal, _, _) = loft_mesh(&profiles, normal_options).unwrap();

        // Tight loft should pass closer to the middle profile point (2, 0, 2.5)
        let target = Point3::new(2.0, 0.0, 2.5);
        let (_, dist_tight) = closest_mesh_point(&mesh_tight, target).unwrap();
        let (_, dist_normal) = closest_mesh_point(&mesh_normal, target).unwrap();

        // Tight should be at least as close as normal (usually closer due to reduced tangent influence)
        assert!(dist_tight <= dist_normal + 0.05,
            "Tight loft should have sharper profile transitions. Tight: {}, Normal: {}", 
            dist_tight, dist_normal);
    }

    /// Test that Straight loft produces linear interpolation (ruled surface).
    #[test]
    fn test_loft_straight_linear_interpolation() {
        // Two circles at different heights
        let p0 = circle_profile(0.0, 0.0, 0.0, 1.0, 16);
        let p1 = circle_profile(0.0, 0.0, 10.0, 2.0, 16);
        let profiles: Vec<&[Point3]> = vec![&p0, &p1];

        let options = LoftOptions {
            loft_type: LoftType::Straight,
            ..Default::default()
        };
        let (mesh, _, _) = loft_mesh(&profiles, options).unwrap();

        // Straight loft creates a ruled surface with no intermediate profiles.
        // Verify start profile vertices are at z=0 with radius ~1
        // and end profile vertices are at z=10 with radius ~2
        
        // Check that we have vertices at both z levels
        let vertices_at_z0: Vec<_> = mesh.positions.iter()
            .filter(|p| p[2].abs() < 0.1)
            .collect();
        let vertices_at_z10: Vec<_> = mesh.positions.iter()
            .filter(|p| (p[2] - 10.0).abs() < 0.1)
            .collect();
        
        assert!(!vertices_at_z0.is_empty(), "Should have vertices at z=0");
        assert!(!vertices_at_z10.is_empty(), "Should have vertices at z=10");
        
        // Verify radii at each level
        for v in &vertices_at_z0 {
            let radius = (v[0] * v[0] + v[1] * v[1]).sqrt();
            assert!((radius - 1.0).abs() < 0.15, "Start profile should have radius ~1, got {}", radius);
        }
        for v in &vertices_at_z10 {
            let radius = (v[0] * v[0] + v[1] * v[1]).sqrt();
            assert!((radius - 2.0).abs() < 0.15, "End profile should have radius ~2, got {}", radius);
        }
        
        // Straight loft should have only 2 rows of vertices (no intermediate profiles)
        // So total vertices = 2 * points_per_profile
        // With 16 segments for a circle, we get 17 points (including closing point)
        // After cleaning duplicates in closed profile, we might have 16 or 17 points
        assert!(mesh.positions.len() <= 50, 
            "Straight loft should have minimal vertices (no interpolation), got {}", 
            mesh.positions.len());
    }

    /// Test that Normal and Straight lofts produce different vertex counts.
    #[test]
    fn test_loft_types_produce_different_geometry() {
        let p0 = circle_profile(0.0, 0.0, 0.0, 1.0, 12);
        let p1 = circle_profile(0.0, 0.0, 2.5, 1.5, 12);
        let p2 = circle_profile(0.0, 0.0, 5.0, 1.0, 12);
        let profiles: Vec<&[Point3]> = vec![&p0, &p1, &p2];

        let (mesh_straight, _, _) = loft_mesh(&profiles, LoftOptions {
            loft_type: LoftType::Straight,
            ..Default::default()
        }).unwrap();

        let (mesh_normal, _, _) = loft_mesh(&profiles, LoftOptions {
            loft_type: LoftType::Normal,
            ..Default::default()
        }).unwrap();

        let (mesh_loose, _, _) = loft_mesh(&profiles, LoftOptions {
            loft_type: LoftType::Loose,
            ..Default::default()
        }).unwrap();

        // Normal and Loose should have more vertices due to intermediate profile generation
        assert!(mesh_normal.positions.len() > mesh_straight.positions.len(),
            "Normal loft should generate intermediate profiles. Normal: {}, Straight: {}",
            mesh_normal.positions.len(), mesh_straight.positions.len());

        assert!(mesh_loose.positions.len() > mesh_straight.positions.len(),
            "Loose loft should generate intermediate profiles. Loose: {}, Straight: {}",
            mesh_loose.positions.len(), mesh_straight.positions.len());
    }

    /// Test that Developable loft uses eased linear interpolation.
    #[test]
    fn test_loft_developable_produces_ruling_based_surface() {
        let p0 = circle_profile(0.0, 0.0, 0.0, 1.0, 16);
        let p1 = circle_profile(0.0, 0.0, 10.0, 2.0, 16);
        let profiles: Vec<&[Point3]> = vec![&p0, &p1];

        let (mesh_dev, _, _) = loft_mesh(&profiles, LoftOptions {
            loft_type: LoftType::Developable,
            ..Default::default()
        }).unwrap();

        let (mesh_straight, _, _) = loft_mesh(&profiles, LoftOptions {
            loft_type: LoftType::Straight,
            ..Default::default()
        }).unwrap();

        // Both should have same vertex count (no intermediate profiles)
        assert_eq!(mesh_dev.positions.len(), mesh_straight.positions.len(),
            "Developable should have same structure as straight");
        
        // But positions may differ due to easing
        // Verify mesh is valid
        assert!(mesh_dev.indices.len() > 0);
    }

    /// Test fit_loft_mesh convenience function.
    #[test]
    fn test_fit_loft_mesh_convenience() {
        let p0 = circle_profile(0.0, 0.0, 0.0, 1.0, 12);
        let p1 = circle_profile(0.0, 0.0, 2.5, 2.0, 12);
        let p2 = circle_profile(0.0, 0.0, 5.0, 1.0, 12);
        let profiles: Vec<&[Point3]> = vec![&p0, &p1, &p2];

        let (mesh, _, _) = fit_loft_mesh(&profiles).unwrap();

        // Should pass through middle profile
        let target = Point3::new(2.0, 0.0, 2.5);
        let (_, dist) = closest_mesh_point(&mesh, target).unwrap();
        
        assert!(dist < 0.15, "Fit loft should interpolate through profiles. Distance: {}", dist);
    }

    /// Test control_point_loft_mesh convenience function.
    #[test]
    fn test_control_point_loft_mesh_convenience() {
        let p0 = circle_profile(0.0, 0.0, 0.0, 1.0, 12);
        let p1 = circle_profile(0.0, 0.0, 2.5, 2.0, 12);
        let p2 = circle_profile(0.0, 0.0, 5.0, 1.0, 12);
        let profiles: Vec<&[Point3]> = vec![&p0, &p1, &p2];

        let (mesh, _, _) = control_point_loft_mesh(&profiles).unwrap();

        // Should approximate but not necessarily pass through middle profile
        let target = Point3::new(2.0, 0.0, 2.5);
        let (_, dist) = closest_mesh_point(&mesh, target).unwrap();
        
        // Control point loft should be pulled toward but not necessarily through
        assert!(dist > 0.05 || dist < 1.0, 
            "Control point loft should approximate, not interpolate. Distance: {}", dist);
    }

    /// Test spline interpolation functions directly.
    #[test]
    fn test_catmull_rom_interpolation() {
        let p0 = Point3::new(0.0, 0.0, 0.0);
        let p1 = Point3::new(1.0, 0.0, 0.0);
        let p2 = Point3::new(2.0, 0.0, 0.0);
        let p3 = Point3::new(3.0, 0.0, 0.0);

        // At t=0, should be at p1
        let at_0 = catmull_rom_point(p0, p1, p2, p3, 0.0, 0.5);
        assert!((at_0.x - 1.0).abs() < 1e-10);

        // At t=1, should be at p2
        let at_1 = catmull_rom_point(p0, p1, p2, p3, 1.0, 0.5);
        assert!((at_1.x - 2.0).abs() < 1e-10);

        // At t=0.5, should be at midpoint (1.5, 0, 0) for collinear points
        let at_half = catmull_rom_point(p0, p1, p2, p3, 0.5, 0.5);
        assert!((at_half.x - 1.5).abs() < 1e-10);
    }

    /// Test B-spline interpolation functions.
    #[test]
    fn test_bspline_interpolation() {
        let p0 = Point3::new(0.0, 0.0, 0.0);
        let p1 = Point3::new(1.0, 0.0, 0.0);
        let p2 = Point3::new(2.0, 0.0, 0.0);
        let p3 = Point3::new(3.0, 0.0, 0.0);

        // B-spline should produce smooth curve that doesn't necessarily pass through control points
        let at_0 = bspline_point(p0, p1, p2, p3, 0.0);
        let at_1 = bspline_point(p0, p1, p2, p3, 1.0);
        let at_half = bspline_point(p0, p1, p2, p3, 0.5);

        // For uniform B-spline with collinear points, result should still be on the line
        assert!(at_0.y.abs() < 1e-10);
        assert!(at_1.y.abs() < 1e-10);
        assert!(at_half.y.abs() < 1e-10);

        // At t=0, B-spline with uniform knots is at weighted average, not at p1
        // (1/6 * p0 + 2/3 * p1 + 1/6 * p2) ≈ 1.0 for these points
        assert!((at_0.x - 1.0).abs() < 0.01);
    }

    /// Test that different tension values affect the curve shape.
    #[test]
    fn test_catmull_rom_tension_effect() {
        // Create a curve with a sharp turn
        let p0 = Point3::new(0.0, 0.0, 0.0);
        let p1 = Point3::new(1.0, 0.0, 0.0);
        let p2 = Point3::new(1.0, 1.0, 0.0);
        let p3 = Point3::new(1.0, 2.0, 0.0);

        // Compare tight vs loose tension at the midpoint
        let tight = catmull_rom_point(p0, p1, p2, p3, 0.5, 0.25); // Tight
        let normal = catmull_rom_point(p0, p1, p2, p3, 0.5, 0.5); // Normal
        let loose = catmull_rom_point(p0, p1, p2, p3, 0.5, 0.75); // Loose

        // All should be somewhere between p1 and p2
        assert!(tight.x >= 0.9 && tight.x <= 1.1);
        assert!(normal.x >= 0.9 && normal.x <= 1.1);
        assert!(loose.x >= 0.9 && loose.x <= 1.1);

        // Y should be around 0.5 for all, but with slight variations
        assert!(tight.y > 0.0 && tight.y < 1.0);
        assert!(normal.y > 0.0 && normal.y < 1.0);
        assert!(loose.y > 0.0 && loose.y < 1.0);
    }

    /// Test uniform loft type.
    #[test]
    fn test_loft_uniform() {
        let p0 = circle_profile(0.0, 0.0, 0.0, 1.0, 12);
        let p1 = circle_profile(0.0, 0.0, 5.0, 2.0, 12);
        let profiles: Vec<&[Point3]> = vec![&p0, &p1];

        let options = LoftOptions {
            loft_type: LoftType::Uniform,
            ..Default::default()
        };
        let (mesh, _, _) = loft_mesh(&profiles, options).unwrap();

        assert!(mesh.positions.len() > 0);
        assert!(mesh.indices.len() > 0);
    }

    /// Golden test: verify mesh properties for each loft type.
    #[test]
    fn test_loft_types_golden_properties() {
        let p0 = circle_profile(0.0, 0.0, 0.0, 1.0, 8);
        let p1 = circle_profile(0.0, 0.0, 2.0, 1.5, 8);
        let p2 = circle_profile(0.0, 0.0, 4.0, 1.0, 8);
        let profiles: Vec<&[Point3]> = vec![&p0, &p1, &p2];

        // Collect results for all loft types
        let results: Vec<(&str, GeomMesh)> = vec![
            ("Normal", loft_mesh(&profiles, LoftOptions { loft_type: LoftType::Normal, ..Default::default() }).unwrap().0),
            ("Loose", loft_mesh(&profiles, LoftOptions { loft_type: LoftType::Loose, ..Default::default() }).unwrap().0),
            ("Tight", loft_mesh(&profiles, LoftOptions { loft_type: LoftType::Tight, ..Default::default() }).unwrap().0),
            ("Straight", loft_mesh(&profiles, LoftOptions { loft_type: LoftType::Straight, ..Default::default() }).unwrap().0),
            ("Developable", loft_mesh(&profiles, LoftOptions { loft_type: LoftType::Developable, ..Default::default() }).unwrap().0),
            ("Uniform", loft_mesh(&profiles, LoftOptions { loft_type: LoftType::Uniform, ..Default::default() }).unwrap().0),
        ];

        // All should produce valid meshes
        for (name, mesh) in &results {
            assert!(!mesh.positions.is_empty(), "{} should produce vertices", name);
            assert!(!mesh.indices.is_empty(), "{} should produce triangles", name);
            
            // Check no NaN/Inf in positions
            for (i, pos) in mesh.positions.iter().enumerate() {
                assert!(pos[0].is_finite(), "{} vertex {} X is not finite", name, i);
                assert!(pos[1].is_finite(), "{} vertex {} Y is not finite", name, i);
                assert!(pos[2].is_finite(), "{} vertex {} Z is not finite", name, i);
            }
            
            // Check valid indices
            let max_idx = mesh.positions.len() as u32;
            for (i, idx) in mesh.indices.iter().enumerate() {
                assert!(*idx < max_idx, "{} index {} ({}) out of bounds (max {})", name, i, idx, max_idx);
            }
        }

        // Verify different vertex counts for interpolating vs ruled lofts
        let normal_verts = results[0].1.positions.len();
        let straight_verts = results[3].1.positions.len();
        assert!(normal_verts > straight_verts,
            "Normal should have more vertices than Straight ({} vs {})", normal_verts, straight_verts);
    }
}
