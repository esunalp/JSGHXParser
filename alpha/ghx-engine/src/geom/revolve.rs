use super::diagnostics::GeomMeshDiagnostics;
use super::mesh::{GeomMesh, finalize_mesh};
use super::triangulation::triangulate_trim_region;
use super::trim::{TrimLoop, TrimRegion, UvPoint};
use super::{Point3, Tolerance, Vec3};
use std::f64::consts::PI;

/// Configuration for cap generation on revolved surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RevolveCaps {
    pub start: bool,
    pub end: bool,
}

impl RevolveCaps {
    pub const NONE: Self = Self { start: false, end: false };
    pub const START: Self = Self { start: true, end: false };
    pub const END: Self = Self { start: false, end: true };
    pub const BOTH: Self = Self { start: true, end: true };
}

/// Options for controlling revolve mesh generation.
#[derive(Debug, Clone, Copy)]
pub struct RevolveOptions {
    /// Minimum number of angular steps (overridden by adaptive subdivision)
    pub min_steps: usize,
    /// Maximum number of angular steps
    pub max_steps: usize,
    /// Whether to weld seam vertices for full 360° revolutions
    pub weld_seam: bool,
}

impl Default for RevolveOptions {
    fn default() -> Self {
        Self {
            min_steps: 8,
            max_steps: 128,
            weld_seam: true,
        }
    }
}

/// Options for rail revolution operations.
/// 
/// This struct provides fine control over how the profile is positioned and oriented
/// along the rail curve. The most important option is `reference_axis`, which defines
/// how the profile's local coordinate system maps to the rail's Frenet frames.
/// 
/// # Profile Coordinate System
/// 
/// The profile is defined in a local coordinate system where:
/// - Profile points with `z` offset map along the rail tangent
/// - Profile points with `x` offset map along the frame normal
/// - Profile points with `y` offset map along the frame binormal
/// 
/// Without a reference axis, the initial frame orientation is arbitrary (but consistent).
/// With a reference axis, the frame's normal is aligned to be as close as possible to
/// the axis direction, giving predictable and controllable orientation.
/// 
/// # Example
/// 
/// ```ignore
/// // Create a rail revolve with the profile oriented so its "up" direction
/// // aligns with the world Z axis
/// let options = RailRevolveOptions {
///     reference_axis: Some(RailRevolveAxis {
///         origin: Point3::new(0.0, 0.0, 0.0),
///         direction: Vec3::new(0.0, 0.0, 1.0),
///     }),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Copy)]
pub struct RailRevolveOptions {
    /// Reference axis for orienting the profile frames along the rail.
    /// 
    /// When provided, this axis controls the initial orientation of the Frenet frames:
    /// - The axis direction is used as a "reference up" vector
    /// - The frame's normal is aligned to lie in the plane containing both the
    ///   rail tangent and the axis direction
    /// - Parallel transport preserves this orientation along the rail
    /// 
    /// When `None`, the initial frame uses an arbitrary but consistent orientation
    /// based on world axes.
    pub reference_axis: Option<RailRevolveAxis>,
    
    /// Cap configuration for closed profiles.
    pub caps: RevolveCaps,
}

impl Default for RailRevolveOptions {
    fn default() -> Self {
        Self {
            reference_axis: None,
            caps: RevolveCaps::NONE,
        }
    }
}

/// Defines a reference axis for rail revolution operations.
/// 
/// The axis specifies how the profile should be oriented relative to the rail.
/// In Grasshopper terms, this corresponds to the "Axis" input of the RailRevolution
/// component.
/// 
/// # Interpretation
/// 
/// The axis defines a line in 3D space. The direction of this line establishes
/// a preferred "up" direction for the profile orientation:
/// 
/// - **origin**: A point on the axis (used for profile positioning if the profile
///   needs to be translated to the rail start)
/// - **direction**: The axis direction vector (used to orient frames so the profile's
///   local X-axis is as close as possible to this direction)
/// 
/// # Behavior
/// 
/// When the reference axis is nearly parallel to the rail tangent at a point,
/// the frame orientation gracefully degrades to the default behavior (using
/// world axes as reference).
#[derive(Debug, Clone, Copy)]
pub struct RailRevolveAxis {
    /// Origin point of the reference axis.
    /// 
    /// This can be used to define where the profile should be positioned
    /// before sweeping along the rail.
    pub origin: Point3,
    
    /// Direction of the reference axis.
    /// 
    /// This vector establishes the preferred orientation for the profile's
    /// local coordinate frame. The frame's normal will be oriented to lie
    /// in the plane containing both the rail tangent and this direction.
    pub direction: Vec3,
}

#[derive(Debug, thiserror::Error)]
pub enum RevolveError {
    #[error("revolve axis must be finite and non-zero")]
    InvalidAxis,
    #[error("profile points must be finite")]
    NonFinitePoint,
    #[error("revolve inputs must be finite")]
    NonFiniteInput,
    #[error("profile requires at least {min} unique points")]
    NotEnoughPoints { min: usize },
    #[error("caps require a closed profile")]
    CapsRequireClosedProfile,
    #[error("profile is degenerate (zero area)")]
    ProfileDegenerate,
    #[error("profile is not planar enough to cap (max distance {max_distance})")]
    ProfileNotPlanar { max_distance: f64 },
    #[error("failed to triangulate cap: {0}")]
    CapTriangulation(String),
    #[error("revolve angle must be between 0 and 2π radians")]
    InvalidAngle,
    #[error("rail curve must be finite and valid")]
    InvalidRailCurve,
    #[error("rail curve requires at least 2 points")]
    RailTooShort,
    #[error("profile intersects axis of revolution")]
    ProfileIntersectsAxis,
}

/// Revolve a polyline profile around an axis to create a surface of revolution.
///
/// # Arguments
/// * `profile` - Points defining the profile curve to revolve
/// * `axis_start` - Start point of the axis of revolution
/// * `axis_end` - End point of the axis of revolution
/// * `angle` - Angle of revolution in radians (0 < angle <= 2π)
/// * `caps` - Cap configuration for closed profiles
///
/// # Returns
/// A tuple of (mesh, diagnostics) on success
#[must_use]
pub fn revolve_polyline(
    profile: &[Point3],
    axis_start: Point3,
    axis_end: Point3,
    angle: f64,
    caps: RevolveCaps,
) -> Result<(GeomMesh, GeomMeshDiagnostics), RevolveError> {
    revolve_polyline_with_tolerance(profile, axis_start, axis_end, angle, caps, Tolerance::default_geom())
}

/// Revolve a polyline profile around an axis with explicit tolerance.
#[must_use]
pub fn revolve_polyline_with_tolerance(
    profile: &[Point3],
    axis_start: Point3,
    axis_end: Point3,
    angle: f64,
    caps: RevolveCaps,
    tol: Tolerance,
) -> Result<(GeomMesh, GeomMeshDiagnostics), RevolveError> {
    revolve_polyline_with_options(profile, axis_start, axis_end, angle, caps, RevolveOptions::default(), tol)
}

/// Revolve a polyline profile around an axis with full options control.
#[must_use]
pub fn revolve_polyline_with_options(
    profile: &[Point3],
    axis_start: Point3,
    axis_end: Point3,
    angle: f64,
    caps: RevolveCaps,
    options: RevolveOptions,
    tol: Tolerance,
) -> Result<(GeomMesh, GeomMeshDiagnostics), RevolveError> {
    // Validate inputs
    if !angle.is_finite() || angle <= 0.0 || angle > 2.0 * PI + tol.eps {
        return Err(RevolveError::InvalidAngle);
    }
    let angle = angle.min(2.0 * PI);

    let (axis_origin, axis_dir) = validate_axis(axis_start, axis_end)?;
    let cleaned = clean_profile(profile, caps.start || caps.end, tol)?;

    // Check if profile intersects axis
    if profile_intersects_axis(&cleaned.points, axis_origin, axis_dir, tol) {
        return Err(RevolveError::ProfileIntersectsAxis);
    }

    // Determine if this is a full revolution (seam case)
    let is_full_revolution = (angle - 2.0 * PI).abs() < tol.eps;
    let weld_seam = options.weld_seam && is_full_revolution;

    // Calculate number of angular steps based on angle, profile radius, and tolerance
    let steps = calculate_revolve_steps(&cleaned.points, axis_origin, axis_dir, angle, &options, tol);

    // Build the revolved mesh
    let (vertices, uvs, indices) = build_revolved_mesh_rodrigues(
        &cleaned.points,
        axis_origin,
        axis_dir,
        angle,
        steps,
        cleaned.closed,
        weld_seam,
        tol,
    )?;

    // Add caps if requested
    let (vertices, uvs, indices) = if caps.start || caps.end {
        if !cleaned.closed {
            return Err(RevolveError::CapsRequireClosedProfile);
        }
        add_revolve_caps(
            vertices,
            uvs,
            indices,
            &cleaned.points,
            axis_origin,
            axis_dir,
            angle,
            caps,
            tol,
        )?
    } else {
        (vertices, uvs, indices)
    };

    Ok(finalize_mesh(vertices, Some(uvs), indices, tol))
}

/// Create a surface by revolving a profile along a rail curve.
/// 
/// The rail curve defines a path of rotation instead of a fixed axis.
/// This creates a more general swept/revolved surface.
///
/// # Arguments
/// * `profile` - Points defining the profile curve. **Important:** The profile is assumed
///   to be defined in a local coordinate system centered at the origin `(0, 0, 0)`. The
///   profile's Z-axis aligns with the rail tangent, X-axis with the rail normal, and
///   Y-axis with the rail binormal. If your profile is positioned elsewhere, subtract
///   its centroid before calling this function.
/// * `rail` - Points defining the rail/axis path curve. The profile will be positioned
///   at each point along the rail using Frenet frames (parallel transport).
/// * `caps` - Cap configuration for closed profiles. Caps are only added if the profile
///   forms a closed loop (first point equals last point within tolerance).
///
/// # Coordinate System
/// The profile is transformed at each rail point using a rotation-minimizing frame:
/// - `local_offset.z` -> tangent direction (along the rail)
/// - `local_offset.x` -> normal direction (perpendicular, in osculating plane)
/// - `local_offset.y` -> binormal direction (perpendicular to both)
///
/// # Example
/// ```ignore
/// // Create a circular profile centered at origin
/// let profile: Vec<Point3> = (0..=16).map(|i| {
///     let t = (i as f64 / 16.0) * 2.0 * PI;
///     Point3::new(t.cos() * 0.5, t.sin() * 0.5, 0.0)
/// }).collect();
/// 
/// // Create a curved rail
/// let rail = vec![
///     Point3::new(0.0, 0.0, 0.0),
///     Point3::new(1.0, 0.0, 1.0),
///     Point3::new(2.0, 0.0, 0.0),
/// ];
/// 
/// let (mesh, diagnostics) = rail_revolve_polyline(&profile, &rail, RevolveCaps::BOTH)?;
/// ```
#[must_use]
pub fn rail_revolve_polyline(
    profile: &[Point3],
    rail: &[Point3],
    caps: RevolveCaps,
) -> Result<(GeomMesh, GeomMeshDiagnostics), RevolveError> {
    rail_revolve_polyline_with_tolerance(profile, rail, caps, Tolerance::default_geom())
}

/// Rail revolution with explicit tolerance.
#[must_use]
pub fn rail_revolve_polyline_with_tolerance(
    profile: &[Point3],
    rail: &[Point3],
    caps: RevolveCaps,
    tol: Tolerance,
) -> Result<(GeomMesh, GeomMeshDiagnostics), RevolveError> {
    // Validate rail curve
    if rail.len() < 2 {
        return Err(RevolveError::RailTooShort);
    }

    if !rail.iter().all(|p| p.x.is_finite() && p.y.is_finite() && p.z.is_finite()) {
        return Err(RevolveError::NonFinitePoint);
    }

    let cleaned_profile = clean_profile(profile, caps.start || caps.end, tol)?;
    
    // Compute Frenet frames along the rail
    let frames = compute_rail_frames(rail, tol)?;
    
    // Build vertices by positioning profile at each rail point using Frenet frames
    let mut vertices = Vec::with_capacity(rail.len() * cleaned_profile.points.len());
    let mut uvs = Vec::with_capacity(rail.len() * cleaned_profile.points.len());
    
    // Calculate total rail arc length for UV parameterization
    let rail_arc_lengths = compute_arc_lengths(rail);
    let total_arc_length = *rail_arc_lengths.last().unwrap_or(&1.0);
    
    for (rail_idx, (rail_point, frame)) in rail.iter().zip(frames.iter()).enumerate() {
        let u_param = if total_arc_length > tol.eps {
            rail_arc_lengths[rail_idx] / total_arc_length
        } else {
            rail_idx as f64 / (rail.len() - 1).max(1) as f64
        };
        
        for (profile_idx, &profile_point) in cleaned_profile.points.iter().enumerate() {
            // Transform profile point using the frame
            // Profile is assumed to be in a local coordinate system
            let local_offset = profile_point.sub_point(Point3::new(0.0, 0.0, 0.0));
            
            let world_point = rail_point
                .add_vec(frame.tangent.mul_scalar(local_offset.z))
                .add_vec(frame.normal.mul_scalar(local_offset.x))
                .add_vec(frame.binormal.mul_scalar(local_offset.y));
            
            vertices.push(world_point);
            
            let v_param = profile_idx as f64 / (cleaned_profile.points.len() - 1).max(1) as f64;
            uvs.push([u_param, v_param]);
        }
    }
    
    // Build indices
    let profile_len = cleaned_profile.points.len();
    let is_closed_profile = cleaned_profile.closed;
    let mut indices = Vec::new();
    
    for rail_segment in 0..rail.len() - 1 {
        let profile_edge_count = if is_closed_profile { profile_len } else { profile_len - 1 };
        
        for i in 0..profile_edge_count {
            let i_next = (i + 1) % profile_len;
            
            let i0 = (rail_segment * profile_len + i) as u32;
            let i1 = (rail_segment * profile_len + i_next) as u32;
            let i2 = ((rail_segment + 1) * profile_len + i_next) as u32;
            let i3 = ((rail_segment + 1) * profile_len + i) as u32;
            
            // Two triangles per quad
            indices.extend_from_slice(&[i0, i1, i2]);
            indices.extend_from_slice(&[i0, i2, i3]);
        }
    }
    
    // Add caps if requested
    let (vertices, uvs, indices) = if (caps.start || caps.end) && is_closed_profile {
        add_rail_revolve_caps(vertices, uvs, indices, &cleaned_profile.points, &frames, rail, caps, tol)?
    } else if caps.start || caps.end {
        return Err(RevolveError::CapsRequireClosedProfile);
    } else {
        (vertices, uvs, indices)
    };
    
    Ok(finalize_mesh(vertices, Some(uvs), indices, tol))
}

/// Rail revolution with full options control including reference axis.
/// 
/// This is the most flexible rail revolution function, allowing control over:
/// - Profile orientation via a reference axis
/// - Cap generation for closed profiles
/// 
/// # Arguments
/// * `profile` - Points defining the profile curve. The profile should be defined
///   in a local coordinate system where the origin is the point that will be placed
///   on the rail. See `RailRevolveOptions` for details on coordinate interpretation.
/// * `rail` - Points defining the rail/axis path curve
/// * `options` - Configuration options including reference axis and caps
/// * `tol` - Tolerance for geometric operations
/// 
/// # Profile Coordinate System
/// 
/// The profile is transformed at each rail point using Frenet frames:
/// - Profile Z-coordinate → along rail tangent
/// - Profile X-coordinate → along frame normal (influenced by reference axis)
/// - Profile Y-coordinate → along frame binormal
/// 
/// If a reference axis is provided in options, the frame's normal is aligned to
/// lie in the plane containing the tangent and the axis direction. This gives
/// predictable, controllable orientation instead of arbitrary frame selection.
/// 
/// # Example
/// ```ignore
/// use geom::{Point3, Vec3, RailRevolveOptions, RailRevolveAxis, RevolveCaps};
/// 
/// // Profile centered at origin, oriented in XY plane
/// let profile = vec![
///     Point3::new(0.5, 0.0, 0.0),
///     Point3::new(0.0, 0.5, 0.0),
///     Point3::new(-0.5, 0.0, 0.0),
/// ];
/// 
/// let rail = vec![
///     Point3::new(0.0, 0.0, 0.0),
///     Point3::new(0.0, 0.0, 1.0),
///     Point3::new(0.0, 0.0, 2.0),
/// ];
/// 
/// // Orient so profile's X aligns with world X direction
/// let options = RailRevolveOptions {
///     reference_axis: Some(RailRevolveAxis {
///         origin: Point3::ORIGIN,
///         direction: Vec3::new(1.0, 0.0, 0.0),
///     }),
///     caps: RevolveCaps::BOTH,
/// };
/// 
/// let (mesh, diagnostics) = rail_revolve_polyline_with_options(&profile, &rail, options, tol)?;
/// ```
#[must_use]
pub fn rail_revolve_polyline_with_options(
    profile: &[Point3],
    rail: &[Point3],
    options: RailRevolveOptions,
    tol: Tolerance,
) -> Result<(GeomMesh, GeomMeshDiagnostics), RevolveError> {
    let caps = options.caps;
    
    // Validate rail curve
    if rail.len() < 2 {
        return Err(RevolveError::RailTooShort);
    }

    if !rail.iter().all(|p| p.x.is_finite() && p.y.is_finite() && p.z.is_finite()) {
        return Err(RevolveError::NonFinitePoint);
    }

    let cleaned_profile = clean_profile(profile, caps.start || caps.end, tol)?;
    
    // Compute Frenet frames along the rail, optionally using reference axis for initial orientation
    let frames = compute_rail_frames_with_reference(rail, options.reference_axis, tol)?;
    
    // Build vertices by positioning profile at each rail point using Frenet frames
    let mut vertices = Vec::with_capacity(rail.len() * cleaned_profile.points.len());
    let mut uvs = Vec::with_capacity(rail.len() * cleaned_profile.points.len());
    
    // Calculate total rail arc length for UV parameterization
    let rail_arc_lengths = compute_arc_lengths(rail);
    let total_arc_length = *rail_arc_lengths.last().unwrap_or(&1.0);
    
    for (rail_idx, (rail_point, frame)) in rail.iter().zip(frames.iter()).enumerate() {
        let u_param = if total_arc_length > tol.eps {
            rail_arc_lengths[rail_idx] / total_arc_length
        } else {
            rail_idx as f64 / (rail.len() - 1).max(1) as f64
        };
        
        for (profile_idx, &profile_point) in cleaned_profile.points.iter().enumerate() {
            // Transform profile point using the frame
            // Profile is assumed to be in a local coordinate system
            let local_offset = profile_point.sub_point(Point3::new(0.0, 0.0, 0.0));
            
            let world_point = rail_point
                .add_vec(frame.tangent.mul_scalar(local_offset.z))
                .add_vec(frame.normal.mul_scalar(local_offset.x))
                .add_vec(frame.binormal.mul_scalar(local_offset.y));
            
            vertices.push(world_point);
            
            let v_param = profile_idx as f64 / (cleaned_profile.points.len() - 1).max(1) as f64;
            uvs.push([u_param, v_param]);
        }
    }
    
    // Build indices
    let profile_len = cleaned_profile.points.len();
    let is_closed_profile = cleaned_profile.closed;
    let mut indices = Vec::new();
    
    for rail_segment in 0..rail.len() - 1 {
        let profile_edge_count = if is_closed_profile { profile_len } else { profile_len - 1 };
        
        for i in 0..profile_edge_count {
            let i_next = (i + 1) % profile_len;
            
            let i0 = (rail_segment * profile_len + i) as u32;
            let i1 = (rail_segment * profile_len + i_next) as u32;
            let i2 = ((rail_segment + 1) * profile_len + i_next) as u32;
            let i3 = ((rail_segment + 1) * profile_len + i) as u32;
            
            // Two triangles per quad
            indices.extend_from_slice(&[i0, i1, i2]);
            indices.extend_from_slice(&[i0, i2, i3]);
        }
    }
    
    // Add caps if requested
    let (vertices, uvs, indices) = if (caps.start || caps.end) && is_closed_profile {
        add_rail_revolve_caps(vertices, uvs, indices, &cleaned_profile.points, &frames, rail, caps, tol)?
    } else if caps.start || caps.end {
        return Err(RevolveError::CapsRequireClosedProfile);
    } else {
        (vertices, uvs, indices)
    };
    
    Ok(finalize_mesh(vertices, Some(uvs), indices, tol))
}

fn validate_axis(axis_start: Point3, axis_end: Point3) -> Result<(Point3, Vec3), RevolveError> {
    if !axis_start.x.is_finite() || !axis_start.y.is_finite() || !axis_start.z.is_finite() {
        return Err(RevolveError::InvalidAxis);
    }
    if !axis_end.x.is_finite() || !axis_end.y.is_finite() || !axis_end.z.is_finite() {
        return Err(RevolveError::InvalidAxis);
    }
    let axis = axis_end.sub_point(axis_start);
    let axis_dir = axis.normalized().ok_or(RevolveError::InvalidAxis)?;
    Ok((axis_start, axis_dir))
}

#[derive(Debug, Clone)]
struct CleanProfile {
    points: Vec<Point3>,
    closed: bool,
}

fn clean_profile(
    points: &[Point3],
    require_closed: bool,
    tol: Tolerance,
) -> Result<CleanProfile, RevolveError> {
    if points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite()) {
        return Err(RevolveError::NonFinitePoint);
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
        return Err(RevolveError::NotEnoughPoints { min: 2 });
    }

    // Check if profile is naturally closed (first == last)
    let mut closed = false;
    if cleaned.len() > 2 {
        if let (Some(first), Some(last)) = (cleaned.first().copied(), cleaned.last().copied()) {
            if tol.approx_eq_point3(first, last) {
                cleaned.pop();
                closed = true;
            }
        }
    }
    
    // For closed profiles, we need at least 3 unique points
    if closed && cleaned.len() < 3 {
        return Err(RevolveError::NotEnoughPoints { min: 3 });
    }
    
    // If caps are required but profile isn't closed, return error
    if require_closed && !closed {
        return Err(RevolveError::CapsRequireClosedProfile);
    }

    Ok(CleanProfile {
        points: cleaned,
        closed,
    })
}

/// Check if any profile point lies on or very close to the axis of revolution.
fn profile_intersects_axis(
    profile: &[Point3],
    axis_origin: Point3,
    axis_dir: Vec3,
    tol: Tolerance,
) -> bool {
    for &point in profile {
        // Vector from axis origin to point
        let ap = point.sub_point(axis_origin);

        // Project point onto axis (infinite line)
        let t = ap.dot(axis_dir);
        let closest_on_axis = axis_origin.add_vec(axis_dir.mul_scalar(t));
        let distance_sq = point.sub_point(closest_on_axis).length_squared();

        if distance_sq <= tol.eps_squared() {
            return true;
        }
    }

    false
}

/// Calculate the distance from a point to an axis (infinite line).
fn distance_from_axis(point: Point3, axis_origin: Point3, axis_dir: Vec3) -> f64 {
    let ap = point.sub_point(axis_origin);
    let t = ap.dot(axis_dir);
    let closest_on_axis = axis_origin.add_vec(axis_dir.mul_scalar(t));
    point.sub_point(closest_on_axis).length()
}

/// Calculate optimal number of angular steps for revolution.
/// 
/// Uses adaptive subdivision based on:
/// - The angle of revolution
/// - The maximum distance of profile points from the axis (affects arc length)
/// - The tolerance (controls chord deviation)
fn calculate_revolve_steps(
    profile: &[Point3],
    axis_origin: Point3,
    axis_dir: Vec3,
    angle: f64,
    options: &RevolveOptions,
    tol: Tolerance,
) -> usize {
    // Find the maximum radius (distance from axis) in the profile
    let max_radius = profile.iter()
        .map(|&p| distance_from_axis(p, axis_origin, axis_dir))
        .fold(0.0_f64, f64::max);
    
    // If profile is very close to axis, use minimum steps
    if max_radius < tol.eps {
        return options.min_steps;
    }
    
    // Calculate the arc length at the maximum radius
    let arc_length = max_radius * angle;
    
    // Target segment length based on tolerance
    // Use sqrt of tolerance for a reasonable balance between accuracy and performance
    let target_segment = tol.eps.sqrt().max(1e-6);
    
    // Steps based on arc length subdivision
    let steps_by_arc = (arc_length / target_segment).ceil() as usize;
    
    // Also consider angle-based subdivision (don't exceed certain degrees per step)
    let angle_fraction = angle / (2.0 * PI);
    let steps_by_angle = (angle_fraction * options.max_steps as f64).ceil() as usize;
    
    // Take the larger of the two estimates, clamped to options range
    steps_by_arc.max(steps_by_angle)
        .clamp(options.min_steps, options.max_steps)
}

/// Rotate a point around an axis using Rodrigues' rotation formula.
/// This is the mathematically correct way to rotate a point around an arbitrary axis.
fn rotate_point_around_axis(
    point: Point3,
    axis_origin: Point3,
    axis_dir: Vec3,
    angle: f64,
) -> Point3 {
    // Vector from axis origin to the point
    let v = point.sub_point(axis_origin);
    
    // Rodrigues' rotation formula:
    // v_rot = v * cos(θ) + (k × v) * sin(θ) + k * (k · v) * (1 - cos(θ))
    // where k is the unit axis vector
    let cos_angle = angle.cos();
    let sin_angle = angle.sin();
    
    let k_cross_v = axis_dir.cross(v);
    let k_dot_v = axis_dir.dot(v);
    
    let v_rot = v.mul_scalar(cos_angle)
        .add(k_cross_v.mul_scalar(sin_angle))
        .add(axis_dir.mul_scalar(k_dot_v * (1.0 - cos_angle)));
    
    axis_origin.add_vec(v_rot)
}

/// Build the revolved mesh using Rodrigues' rotation formula.
/// This properly rotates each profile point around the axis in 3D space.
/// 
/// # Arguments
/// * `profile` - The profile points to revolve
/// * `axis_origin` - Origin point of the rotation axis
/// * `axis_dir` - Unit direction of the rotation axis
/// * `total_angle` - Total angle of revolution in radians
/// * `steps` - Number of angular steps
/// * `is_closed_profile` - Whether the profile forms a closed loop
/// * `weld_seam` - Whether to weld the seam for full revolutions
/// * `tol` - Tolerance for detecting coincident vertices at the seam
fn build_revolved_mesh_rodrigues(
    profile: &[Point3],
    axis_origin: Point3,
    axis_dir: Vec3,
    total_angle: f64,
    steps: usize,
    is_closed_profile: bool,
    weld_seam: bool,
    tol: Tolerance,
) -> Result<(Vec<Point3>, Vec<[f64; 2]>, Vec<u32>), RevolveError> {
    let profile_len = profile.len();
    
    // For a full revolution with seam welding, we use 'steps' rings (not steps+1)
    // because the last ring wraps back to the first
    let ring_count = if weld_seam { steps } else { steps + 1 };
    
    let mut vertices = Vec::with_capacity(ring_count * profile_len);
    let mut uvs = Vec::with_capacity(ring_count * profile_len);
    
    // Generate vertices for each angular step
    for step in 0..ring_count {
        let angle = (step as f64 / steps as f64) * total_angle;
        let u_param = step as f64 / steps as f64;
        
        for (profile_idx, &profile_point) in profile.iter().enumerate() {
            let rotated = rotate_point_around_axis(profile_point, axis_origin, axis_dir, angle);
            vertices.push(rotated);
            
            let v_param = profile_idx as f64 / (profile_len - 1).max(1) as f64;
            uvs.push([u_param, v_param]);
        }
    }
    
    // Build indices for the side faces
    let mut indices = Vec::new();
    let profile_edge_count = if is_closed_profile { profile_len } else { profile_len - 1 };
    
    // For seam welding, we need to handle the wrap-around case
    // When weld_seam is true, we have 'steps' rings and the last connects back to the first
    // When weld_seam is false, we have 'steps + 1' rings and only 'steps' connecting segments
    let angular_steps = steps;
    
    // Debug verification: for full revolutions with seam welding, check that the first ring's
    // vertices would align with where the "last" ring would be (at angle = total_angle).
    // This validates that Rodrigues' rotation correctly produces a closed surface.
    #[cfg(debug_assertions)]
    if weld_seam && ring_count > 1 {
        for (i, &original_point) in profile.iter().enumerate() {
            let first_ring_vertex = vertices[i];
            // Compute where the vertex would be at the full revolution angle
            let hypothetical_last = rotate_point_around_axis(original_point, axis_origin, axis_dir, total_angle);
            let distance_sq = first_ring_vertex.sub_point(hypothetical_last).length_squared();
            debug_assert!(
                distance_sq <= tol.eps_squared() * 100.0, // Allow some numerical slack
                "Seam alignment failed: vertex {} has gap {} (expected < {})",
                i, distance_sq.sqrt(), tol.eps * 10.0
            );
        }
    }
    
    for step in 0..angular_steps {
        let next_step = if weld_seam && step == angular_steps - 1 {
            0 // Wrap back to first ring
        } else {
            step + 1
        };
        
        for i in 0..profile_edge_count {
            let i_next = (i + 1) % profile_len;
            
            let i0 = (step * profile_len + i) as u32;
            let i1 = (step * profile_len + i_next) as u32;
            let i2 = (next_step * profile_len + i_next) as u32;
            let i3 = (next_step * profile_len + i) as u32;
            
            // Two triangles per quad - consistent winding
            indices.extend_from_slice(&[i0, i1, i2]);
            indices.extend_from_slice(&[i0, i2, i3]);
        }
    }
    
    Ok((vertices, uvs, indices))
}

/// Add caps to a revolved mesh for closed profiles.
/// 
/// Cap orientation is determined by the revolution direction:
/// - Start cap: faces opposite to the revolution direction (inward normal)
/// - End cap: faces along the revolution direction (outward normal)
fn add_revolve_caps(
    mut vertices: Vec<Point3>,
    mut uvs: Vec<[f64; 2]>,
    mut indices: Vec<u32>,
    profile: &[Point3],
    axis_origin: Point3,
    axis_dir: Vec3,
    angle: f64,
    caps: RevolveCaps,
    tol: Tolerance,
) -> Result<(Vec<Point3>, Vec<[f64; 2]>, Vec<u32>), RevolveError> {
    // Start cap (at angle 0) - faces opposite to revolution direction
    if caps.start {
        // For start cap, the outward normal should face opposite to axis_dir
        // (i.e., into the "back" of the revolution)
        let start_cap_normal = axis_dir.mul_scalar(-1.0);
        let cap_tri = build_revolve_cap_triangulation_oriented(profile, start_cap_normal, tol)?;
        
        let offset = vertices.len() as u32;
        
        for &uv in &cap_tri.vertices {
            let point = cap_tri.point_at(uv);
            vertices.push(point);
            uvs.push([0.0, 0.5]); // UV at start of revolution
        }
        
        // Winding is determined by the triangulation with correct normal orientation
        for tri in cap_tri.indices.chunks_exact(3) {
            indices.extend_from_slice(&[
                offset + tri[0],
                offset + tri[1],
                offset + tri[2],
            ]);
        }
    }
    
    // End cap (at final angle) - faces along revolution direction
    if caps.end {
        // For end cap, the outward normal should face along axis_dir
        // (i.e., out of the "front" of the revolution)
        let end_cap_normal = axis_dir;
        let cap_tri = build_revolve_cap_triangulation_oriented(profile, end_cap_normal, tol)?;
        
        let offset = vertices.len() as u32;
        
        for &uv in &cap_tri.vertices {
            let point = cap_tri.point_at(uv);
            // Rotate the cap point to the end position
            let rotated = rotate_point_around_axis(point, axis_origin, axis_dir, angle);
            vertices.push(rotated);
            uvs.push([1.0, 0.5]); // UV at end of revolution
        }
        
        // Winding is determined by the triangulation with correct normal orientation
        for tri in cap_tri.indices.chunks_exact(3) {
            indices.extend_from_slice(&[
                offset + tri[0],
                offset + tri[1],
                offset + tri[2],
            ]);
        }
    }
    
    Ok((vertices, uvs, indices))
}

#[derive(Debug, Clone)]
struct CapTriangulation {
    origin: Point3,
    u_axis: Vec3,
    v_axis: Vec3,
    vertices: Vec<UvPoint>,
    indices: Vec<u32>,
}

impl CapTriangulation {
    fn point_at(&self, uv: UvPoint) -> Point3 {
        let offset = self.u_axis.mul_scalar(uv.u).add(self.v_axis.mul_scalar(uv.v));
        self.origin.add_vec(offset)
    }
}

/// Build cap triangulation for a closed profile with explicit outward normal direction.
/// 
/// # Arguments
/// * `profile` - The closed profile points
/// * `desired_outward_normal` - The direction the cap's outward normal should face
/// * `tol` - Tolerance for planarity checks and triangulation
fn build_revolve_cap_triangulation_oriented(
    profile: &[Point3],
    desired_outward_normal: Vec3,
    tol: Tolerance,
) -> Result<CapTriangulation, RevolveError> {
    // Calculate profile normal using Newell's method
    let normal = polygon_normal(profile);
    let mut normal = normal.normalized().ok_or(RevolveError::ProfileDegenerate)?;
    
    // Check planarity - use a looser tolerance than the base geometric tolerance
    // because real-world profiles may have small deviations while still being
    // "planar enough" for cap generation. The 1000x multiplier allows profiles
    // with sub-millimeter deviations when using default tolerance (1e-9 * 1e3 = 1e-6).
    let origin = profile[0];
    let planar_eps = (tol.eps * 1e3).max(tol.eps);
    let mut max_distance: f64 = 0.0;
    for &p in profile {
        let d = p.sub_point(origin).dot(normal).abs();
        max_distance = max_distance.max(d);
    }
    if max_distance > planar_eps {
        return Err(RevolveError::ProfileNotPlanar { max_distance });
    }
    
    // Orient normal to match the desired outward direction
    // This ensures correct winding for the triangulated cap
    let needs_flip = normal.dot(desired_outward_normal) < 0.0;
    if needs_flip {
        normal = normal.mul_scalar(-1.0);
    }
    
    // Create basis vectors for the cap plane
    let (u_axis, v_axis) = plane_basis(normal)?;
    
    // Project profile to 2D
    // If we flipped the normal, we need to ensure consistent winding in 2D
    let uv_points: Vec<UvPoint> = profile
        .iter()
        .map(|p| {
            let d = p.sub_point(origin);
            UvPoint::new(d.dot(u_axis), d.dot(v_axis))
        })
        .collect();
    
    // Triangulate using constrained triangulation
    let loop_ = TrimLoop::new(uv_points, tol).map_err(|e| RevolveError::CapTriangulation(e.to_string()))?;
    let region = TrimRegion::from_loops(vec![loop_], tol).map_err(|e| RevolveError::CapTriangulation(e.to_string()))?;
    let tri = triangulate_trim_region(&region, tol).map_err(RevolveError::CapTriangulation)?;
    
    // If we flipped the normal, we need to flip the triangle winding
    let indices = if needs_flip {
        tri.indices.chunks_exact(3)
            .flat_map(|t| [t[0], t[2], t[1]])
            .collect()
    } else {
        tri.indices
    };
    
    Ok(CapTriangulation {
        origin,
        u_axis,
        v_axis,
        vertices: tri.vertices,
        indices,
    })
}

/// Create orthonormal basis vectors for a plane given its normal.
fn plane_basis(normal: Vec3) -> Result<(Vec3, Vec3), RevolveError> {
    let n = normal.normalized().ok_or(RevolveError::ProfileDegenerate)?;
    
    // Choose a reference vector that's not parallel to the normal
    let reference = if n.x.abs() < 0.9 {
        Vec3::new(1.0, 0.0, 0.0)
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    };
    
    let u_axis = n.cross(reference).normalized().ok_or(RevolveError::ProfileDegenerate)?;
    let v_axis = n.cross(u_axis);
    
    Ok((u_axis, v_axis))
}

/// Calculate polygon normal using Newell's method.
fn polygon_normal(points: &[Point3]) -> Vec3 {
    if points.len() < 3 {
        return Vec3::new(0.0, 0.0, 0.0);
    }
    
    let mut n = Vec3::new(0.0, 0.0, 0.0);
    for i in 0..points.len() {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        n.x += (a.y - b.y) * (a.z + b.z);
        n.y += (a.z - b.z) * (a.x + b.x);
        n.z += (a.x - b.x) * (a.y + b.y);
    }
    n
}
// ============================================================================
// Frenet Frame computation for Rail Revolution
// ============================================================================

/// A Frenet frame representing a local coordinate system along a curve.
/// 
/// The frame consists of three orthonormal vectors:
/// - **tangent**: Unit vector pointing along the curve direction
/// - **normal**: Unit vector perpendicular to the tangent, lying in the osculating plane
/// - **binormal**: Unit vector perpendicular to both tangent and normal (completes the right-handed frame)
/// 
/// These frames are used in rail revolution and sweep operations to position and orient
/// profiles along a path curve while minimizing twist.
/// 
/// # Frame Orientation
/// The frame follows the right-hand rule: `binormal = tangent × normal`
/// 
/// # Stability
/// For straight or nearly-straight sections, the frame uses a stable reference vector
/// to avoid gimbal lock. Frames are computed using parallel transport (rotation-minimizing)
/// to ensure smooth transitions along the curve.
/// 
/// # Example
/// ```ignore
/// let tangent = Vec3::new(0.0, 0.0, 1.0); // Pointing along Z
/// if let Some(frame) = FrenetFrame::from_tangent(tangent) {
///     // frame.normal will be perpendicular to Z
///     // frame.binormal completes the orthonormal basis
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct FrenetFrame {
    /// Unit vector pointing along the curve direction.
    pub tangent: Vec3,
    /// Unit vector perpendicular to the tangent, in the osculating plane.
    pub normal: Vec3,
    /// Unit vector perpendicular to both tangent and normal (tangent × normal).
    pub binormal: Vec3,
}

impl FrenetFrame {
    /// Create a new Frenet frame with given vectors.
    /// 
    /// # Arguments
    /// * `tangent` - The tangent direction (should be normalized)
    /// * `normal` - The normal direction (should be normalized and perpendicular to tangent)
    /// * `binormal` - The binormal direction (should equal tangent × normal)
    /// 
    /// # Note
    /// This constructor assumes the input vectors are already normalized and orthogonal.
    /// No validation is performed for performance reasons.
    pub fn new(tangent: Vec3, normal: Vec3, binormal: Vec3) -> Self {
        Self { tangent, normal, binormal }
    }
    
    /// Create a frame from a tangent vector, choosing arbitrary but consistent
    /// normal and binormal vectors.
    /// 
    /// This is useful when you only know the direction along the curve and need
    /// to establish a stable perpendicular frame for positioning profiles.
    /// 
    /// # Arguments
    /// * `tangent` - The tangent direction (will be normalized)
    /// 
    /// # Returns
    /// `None` if the tangent vector is zero or cannot be normalized.
    /// 
    /// # Stability
    /// Uses a reference vector approach to ensure consistent frame orientation
    /// regardless of the tangent direction. The reference vector is chosen to
    /// avoid being parallel to the tangent.
    pub fn from_tangent(tangent: Vec3) -> Option<Self> {
        let tangent = tangent.normalized()?;
        
        // Choose a reference vector that's not parallel to tangent
        // Use X-axis unless tangent is nearly parallel to it
        let reference = if tangent.x.abs() < 0.9 {
            Vec3::new(1.0, 0.0, 0.0)
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        };
        
        let normal = tangent.cross(reference).normalized()?;
        let binormal = tangent.cross(normal);
        
        Some(Self { tangent, normal, binormal })
    }
    
    /// Create a frame from a tangent vector with a preferred "up" direction.
    /// 
    /// This method creates a frame where the normal is oriented to be as close
    /// as possible to the provided reference direction while remaining perpendicular
    /// to the tangent. This is essential for rail revolution operations where
    /// the user specifies an axis to control profile orientation.
    /// 
    /// # Arguments
    /// * `tangent` - The tangent direction (will be normalized)
    /// * `up` - A preferred direction for the normal. The actual normal will be
    ///   the component of `up` that is perpendicular to the tangent.
    /// 
    /// # Returns
    /// `None` if:
    /// - The tangent vector is zero or cannot be normalized
    /// - The `up` vector is parallel to the tangent (no perpendicular component)
    /// 
    /// # Behavior
    /// The frame is constructed so that:
    /// - `binormal = tangent × up` (normalized, perpendicular to both)
    /// - `normal = binormal × tangent` (completing the orthonormal frame)
    /// 
    /// This ensures the normal lies in the plane defined by `tangent` and `up`,
    /// pointing in the direction of the `up` component perpendicular to `tangent`.
    /// 
    /// # Example
    /// ```ignore
    /// // Create a frame where the normal aligns with world Z as much as possible
    /// let tangent = Vec3::new(1.0, 0.0, 0.0); // Along X
    /// let up = Vec3::new(0.0, 0.0, 1.0);      // World Z
    /// if let Some(frame) = FrenetFrame::from_tangent_with_up(tangent, up) {
    ///     // frame.normal will point along Z
    ///     // frame.binormal will point along -Y (right-hand rule)
    /// }
    /// ```
    pub fn from_tangent_with_up(tangent: Vec3, up: Vec3) -> Option<Self> {
        let tangent = tangent.normalized()?;
        let up_normalized = up.normalized()?;
        
        // First compute binormal: perpendicular to both tangent and up
        let binormal = tangent.cross(up_normalized);
        let binormal_len_sq = binormal.length_squared();
        
        // If tangent and up are parallel, we can't establish a unique frame
        // Fall back to the standard from_tangent method
        if binormal_len_sq < 1e-12 {
            return Self::from_tangent(tangent);
        }
        
        let binormal = binormal.mul_scalar(1.0 / binormal_len_sq.sqrt());
        
        // Normal is perpendicular to both tangent and binormal
        // This ensures normal is in the tangent-up plane, pointing toward up
        let normal = binormal.cross(tangent);
        
        Some(Self { tangent, normal, binormal })
    }
    
    /// Check if this frame is approximately equal to another within tolerance.
    #[allow(dead_code)]
    pub fn approx_eq(&self, other: &FrenetFrame, eps: f64) -> bool {
        let t_diff = self.tangent.sub(other.tangent).length_squared();
        let n_diff = self.normal.sub(other.normal).length_squared();
        let b_diff = self.binormal.sub(other.binormal).length_squared();
        let eps_sq = eps * eps;
        t_diff <= eps_sq && n_diff <= eps_sq && b_diff <= eps_sq
    }
}

/// Compute Frenet frames along a polyline rail curve.
/// Uses parallel transport to minimize frame rotation along the curve.
fn compute_rail_frames(rail: &[Point3], tol: Tolerance) -> Result<Vec<FrenetFrame>, RevolveError> {
    if rail.len() < 2 {
        return Err(RevolveError::RailTooShort);
    }
    
    let mut frames = Vec::with_capacity(rail.len());
    
    // Compute initial tangent
    let initial_tangent = rail[1].sub_point(rail[0]);
    let first_frame = FrenetFrame::from_tangent(initial_tangent)
        .ok_or(RevolveError::InvalidRailCurve)?;
    frames.push(first_frame);
    
    // Use rotation-minimizing frames (parallel transport) for subsequent points
    for i in 1..rail.len() {
        let prev_idx = i - 1;
        let next_idx = (i + 1).min(rail.len() - 1);
        
        // Compute tangent at current point
        let tangent = if i < rail.len() - 1 {
            // Use central difference for interior points
            let forward = rail[next_idx].sub_point(rail[i]);
            let backward = rail[i].sub_point(rail[prev_idx]);
            forward.add(backward)
        } else {
            // Use backward difference for last point
            rail[i].sub_point(rail[prev_idx])
        };
        
        let tangent = tangent.normalized().ok_or(RevolveError::InvalidRailCurve)?;
        
        // Parallel transport: rotate previous frame to align with new tangent
        let prev_frame = &frames[prev_idx];
        let new_frame = parallel_transport_frame(prev_frame, tangent, tol);
        frames.push(new_frame);
    }
    
    Ok(frames)
}

/// Compute Frenet frames along a polyline rail curve with optional reference axis.
/// 
/// When a reference axis is provided, the initial frame is oriented so that its
/// normal lies in the plane containing both the rail tangent and the axis direction.
/// This provides predictable, user-controllable orientation for the profile.
/// 
/// Subsequent frames are computed using parallel transport (rotation-minimizing),
/// preserving the initial orientation as much as possible along the rail.
fn compute_rail_frames_with_reference(
    rail: &[Point3],
    reference: Option<RailRevolveAxis>,
    tol: Tolerance,
) -> Result<Vec<FrenetFrame>, RevolveError> {
    if rail.len() < 2 {
        return Err(RevolveError::RailTooShort);
    }
    
    let mut frames = Vec::with_capacity(rail.len());
    
    // Compute initial tangent
    let initial_tangent = rail[1].sub_point(rail[0]);
    
    // Compute first frame, optionally using reference axis for orientation
    let first_frame = if let Some(ref axis) = reference {
        // Validate reference axis direction
        let axis_dir = axis.direction.normalized()
            .ok_or(RevolveError::InvalidAxis)?;
        
        // Try to create a frame aligned with the reference axis
        // If the axis is parallel to the tangent, fall back to default
        FrenetFrame::from_tangent_with_up(initial_tangent, axis_dir)
            .ok_or(RevolveError::InvalidRailCurve)?
    } else {
        FrenetFrame::from_tangent(initial_tangent)
            .ok_or(RevolveError::InvalidRailCurve)?
    };
    frames.push(first_frame);
    
    // Use rotation-minimizing frames (parallel transport) for subsequent points
    for i in 1..rail.len() {
        let prev_idx = i - 1;
        let next_idx = (i + 1).min(rail.len() - 1);
        
        // Compute tangent at current point
        let tangent = if i < rail.len() - 1 {
            // Use central difference for interior points
            let forward = rail[next_idx].sub_point(rail[i]);
            let backward = rail[i].sub_point(rail[prev_idx]);
            forward.add(backward)
        } else {
            // Use backward difference for last point
            rail[i].sub_point(rail[prev_idx])
        };
        
        let tangent = tangent.normalized().ok_or(RevolveError::InvalidRailCurve)?;
        
        // Parallel transport: rotate previous frame to align with new tangent
        let prev_frame = &frames[prev_idx];
        let new_frame = parallel_transport_frame(prev_frame, tangent, tol);
        frames.push(new_frame);
    }
    
    Ok(frames)
}

/// Parallel transport a frame to a new tangent direction.
/// This minimizes rotation around the tangent axis.
fn parallel_transport_frame(prev_frame: &FrenetFrame, new_tangent: Vec3, tol: Tolerance) -> FrenetFrame {
    let old_tangent = prev_frame.tangent;
    
    // If tangents are nearly parallel, just update the tangent
    let cross = old_tangent.cross(new_tangent);
    let cross_len_sq = cross.length_squared();
    
    if cross_len_sq < tol.eps_squared() {
        // Tangents are parallel (or anti-parallel)
        let dot = old_tangent.dot(new_tangent);
        if dot < 0.0 {
            // Anti-parallel: flip the frame
            FrenetFrame {
                tangent: new_tangent,
                normal: prev_frame.normal.mul_scalar(-1.0),
                binormal: prev_frame.binormal.mul_scalar(-1.0),
            }
        } else {
            // Parallel: keep orientation
            FrenetFrame {
                tangent: new_tangent,
                normal: prev_frame.normal,
                binormal: prev_frame.binormal,
            }
        }
    } else {
        // Rotate the frame using Rodrigues' formula
        let rotation_axis = cross.normalized().unwrap_or(Vec3::new(0.0, 0.0, 1.0));
        let dot = old_tangent.dot(new_tangent).clamp(-1.0, 1.0);
        let angle = dot.acos();
        
        let new_normal = rotate_vector(prev_frame.normal, rotation_axis, angle);
        let new_binormal = new_tangent.cross(new_normal);
        
        FrenetFrame {
            tangent: new_tangent,
            normal: new_normal,
            binormal: new_binormal,
        }
    }
}

/// Rotate a vector around an axis using Rodrigues' formula.
fn rotate_vector(v: Vec3, axis: Vec3, angle: f64) -> Vec3 {
    let cos_angle = angle.cos();
    let sin_angle = angle.sin();
    
    let k_cross_v = axis.cross(v);
    let k_dot_v = axis.dot(v);
    
    v.mul_scalar(cos_angle)
        .add(k_cross_v.mul_scalar(sin_angle))
        .add(axis.mul_scalar(k_dot_v * (1.0 - cos_angle)))
}

/// Compute cumulative arc lengths along a polyline.
fn compute_arc_lengths(points: &[Point3]) -> Vec<f64> {
    let mut arc_lengths = Vec::with_capacity(points.len());
    let mut cumulative = 0.0;
    arc_lengths.push(cumulative);
    
    for i in 1..points.len() {
        let segment_length = points[i].sub_point(points[i - 1]).length();
        cumulative += segment_length;
        arc_lengths.push(cumulative);
    }
    
    arc_lengths
}

/// Add caps to a rail-revolved mesh.
/// 
/// Cap orientation is determined by the frame tangent direction at each end:
/// - Start cap: faces opposite to the initial tangent (into the rail)
/// - End cap: faces along the final tangent (out of the rail)
fn add_rail_revolve_caps(
    mut vertices: Vec<Point3>,
    mut uvs: Vec<[f64; 2]>,
    mut indices: Vec<u32>,
    profile: &[Point3],
    frames: &[FrenetFrame],
    rail: &[Point3],
    caps: RevolveCaps,
    tol: Tolerance,
) -> Result<(Vec<Point3>, Vec<[f64; 2]>, Vec<u32>), RevolveError> {
    // Start cap (at first rail point) - faces opposite to tangent direction
    if caps.start {
        let frame = &frames[0];
        let rail_point = rail[0];
        
        // Transform profile to world coordinates at start
        let cap_profile: Vec<Point3> = profile.iter().map(|p| {
            let local_offset = p.sub_point(Point3::new(0.0, 0.0, 0.0));
            rail_point
                .add_vec(frame.tangent.mul_scalar(local_offset.z))
                .add_vec(frame.normal.mul_scalar(local_offset.x))
                .add_vec(frame.binormal.mul_scalar(local_offset.y))
        }).collect();
        
        // Build cap triangulation with outward normal facing opposite to tangent
        let start_cap_normal = frame.tangent.mul_scalar(-1.0);
        let cap_tri = build_revolve_cap_triangulation_oriented(&cap_profile, start_cap_normal, tol)?;
        
        let offset = vertices.len() as u32;
        for &uv in &cap_tri.vertices {
            vertices.push(cap_tri.point_at(uv));
            uvs.push([0.0, 0.5]);
        }
        
        // Winding is already correct from oriented triangulation
        for tri in cap_tri.indices.chunks_exact(3) {
            indices.extend_from_slice(&[offset + tri[0], offset + tri[1], offset + tri[2]]);
        }
    }
    
    // End cap (at last rail point) - faces along tangent direction
    if caps.end {
        let frame = &frames[frames.len() - 1];
        let rail_point = rail[rail.len() - 1];
        
        // Transform profile to world coordinates at end
        let cap_profile: Vec<Point3> = profile.iter().map(|p| {
            let local_offset = p.sub_point(Point3::new(0.0, 0.0, 0.0));
            rail_point
                .add_vec(frame.tangent.mul_scalar(local_offset.z))
                .add_vec(frame.normal.mul_scalar(local_offset.x))
                .add_vec(frame.binormal.mul_scalar(local_offset.y))
        }).collect();
        
        // Build cap triangulation with outward normal facing along tangent
        let end_cap_normal = frame.tangent;
        let cap_tri = build_revolve_cap_triangulation_oriented(&cap_profile, end_cap_normal, tol)?;
        
        let offset = vertices.len() as u32;
        for &uv in &cap_tri.vertices {
            vertices.push(cap_tri.point_at(uv));
            uvs.push([1.0, 0.5]);
        }
        
        // Winding is already correct from oriented triangulation
        for tri in cap_tri.indices.chunks_exact(3) {
            indices.extend_from_slice(&[offset + tri[0], offset + tri[1], offset + tri[2]]);
        }
    }
    
    Ok((vertices, uvs, indices))
}