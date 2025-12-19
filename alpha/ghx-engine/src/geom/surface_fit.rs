//! Surface fitting from point data.
//!
//! This module implements surface construction from point clouds and grids.
//! Supports both grid-arranged points (explicit u×v layout) and scattered
//! point clouds (auto-fitted to a best-fit plane or interpolated surface).
//!
//! # Surface Types
//! - **Grid Surface**: Points arranged in a u×v grid; direct surface construction.
//! - **Scattered Points**: Best-fit plane + projection, or bilinear/bicubic interpolation.
//! - **Interpolated Surface**: NURBS surface passing through all grid points.
//!
//! # Diagnostics
//! Fitting operations return diagnostics including:
//! - Max deviation from input points
//! - Fit quality metrics
//! - Warnings for degenerate or ambiguous inputs

use super::core::{Point3, Tolerance, Vec3};
use super::diagnostics::GeomMeshDiagnostics;
use super::mesh::{GeomContext, GeomMesh, finalize_mesh};
use super::metrics::TimingBucket;
use super::surface::{NurbsSurface, PlaneSurface, Surface};
use super::triangulation::delaunay_triangulate;

// ============================================================================
// Error Type
// ============================================================================

/// Errors that can occur during surface fitting.
#[derive(Debug, Clone)]
pub enum SurfaceFitError {
    /// Not enough points provided for surface fitting.
    InsufficientPoints { provided: usize, required: usize },
    /// Points are collinear or coincident; cannot define a surface.
    DegeneratePoints { reason: String },
    /// Grid dimensions do not match the number of provided points.
    GridSizeMismatch { expected: usize, provided: usize },
    /// Invalid grid dimensions (too small).
    InvalidGridSize { u_count: usize, v_count: usize },
    /// Surface construction failed.
    ConstructionFailed { reason: String },
}

impl std::fmt::Display for SurfaceFitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsufficientPoints { provided, required } => {
                write!(f, "insufficient points: {provided} provided, {required} required")
            }
            Self::DegeneratePoints { reason } => {
                write!(f, "degenerate point configuration: {reason}")
            }
            Self::GridSizeMismatch { expected, provided } => {
                write!(f, "grid size mismatch: expected {expected} points, got {provided}")
            }
            Self::InvalidGridSize { u_count, v_count } => {
                write!(f, "invalid grid size: {u_count}×{v_count} (minimum 2×2)")
            }
            Self::ConstructionFailed { reason } => {
                write!(f, "surface construction failed: {reason}")
            }
        }
    }
}

impl std::error::Error for SurfaceFitError {}

// ============================================================================
// Options
// ============================================================================

/// Options for surface fitting from point data.
#[derive(Debug, Clone, Copy)]
pub struct SurfaceFitOptions {
    /// Whether to interpolate through points (true) or approximate (false).
    /// Interpolation creates a surface passing exactly through grid points.
    /// Approximation may smooth out noise in the data.
    pub interpolate: bool,

    /// Degree of the fitted surface in U direction (for NURBS fitting).
    /// Default is 3 (cubic). Must be >= 1 and < u_count.
    pub degree_u: usize,

    /// Degree of the fitted surface in V direction (for NURBS fitting).
    /// Default is 3 (cubic). Must be >= 1 and < v_count.
    pub degree_v: usize,

    /// Close the surface in U direction if endpoints are close.
    pub close_u: bool,

    /// Close the surface in V direction if endpoints are close.
    pub close_v: bool,

    /// Tolerance for point coincidence and closure detection.
    pub tolerance: Tolerance,
}

impl Default for SurfaceFitOptions {
    fn default() -> Self {
        Self {
            interpolate: true,
            degree_u: 3,
            degree_v: 3,
            close_u: false,
            close_v: false,
            tolerance: Tolerance::default_geom(),
        }
    }
}

impl SurfaceFitOptions {
    /// Create options for interpolating surface through points.
    #[must_use]
    pub fn interpolating() -> Self {
        Self {
            interpolate: true,
            ..Default::default()
        }
    }

    /// Create options for approximating surface through points.
    #[must_use]
    pub fn approximating() -> Self {
        Self {
            interpolate: false,
            ..Default::default()
        }
    }

    /// Set the surface degrees.
    #[must_use]
    pub fn with_degrees(mut self, degree_u: usize, degree_v: usize) -> Self {
        self.degree_u = degree_u.max(1);
        self.degree_v = degree_v.max(1);
        self
    }

    /// Set closure options.
    #[must_use]
    pub fn with_closure(mut self, close_u: bool, close_v: bool) -> Self {
        self.close_u = close_u;
        self.close_v = close_v;
        self
    }
}

// ============================================================================
// Diagnostics
// ============================================================================

/// Diagnostics from surface fitting operations.
#[derive(Debug, Clone, Default)]
pub struct SurfaceFitDiagnostics {
    /// Number of input points.
    pub input_point_count: usize,

    /// Grid dimensions used (u × v).
    pub grid_size: (usize, usize),

    /// Maximum deviation from input points (for approximating fits).
    pub max_deviation: f64,

    /// Average deviation from input points.
    pub avg_deviation: f64,

    /// Whether the surface was closed in U.
    pub closed_u: bool,

    /// Whether the surface was closed in V.
    pub closed_v: bool,

    /// Warnings generated during fitting.
    pub warnings: Vec<String>,
}

// ============================================================================
// Core Fitting Functions
// ============================================================================

/// Fit a surface through a grid of points.
///
/// Points must be arranged in row-major order (U varies fastest):
/// `[row0_col0, row0_col1, ..., row0_colN, row1_col0, ...]`
///
/// # Arguments
/// * `points` - Grid points in row-major order
/// * `u_count` - Number of points in U direction (columns)
/// * `v_count` - Number of points in V direction (rows)
/// * `options` - Fitting options
///
/// # Returns
/// A NURBS surface passing through (interpolate=true) or near (interpolate=false)
/// the input points, along with fitting diagnostics.
pub fn surface_from_grid(
    points: &[Point3],
    u_count: usize,
    v_count: usize,
    options: SurfaceFitOptions,
) -> Result<(NurbsSurface, SurfaceFitDiagnostics), SurfaceFitError> {
    // Validate inputs
    if u_count < 2 || v_count < 2 {
        return Err(SurfaceFitError::InvalidGridSize { u_count, v_count });
    }

    let expected_count = u_count * v_count;
    if points.len() != expected_count {
        return Err(SurfaceFitError::GridSizeMismatch {
            expected: expected_count,
            provided: points.len(),
        });
    }

    // Check for NaN/Inf values
    for (i, p) in points.iter().enumerate() {
        if !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite() {
            return Err(SurfaceFitError::DegeneratePoints {
                reason: format!("point {} contains NaN or infinite values", i),
            });
        }
    }

    let mut diagnostics = SurfaceFitDiagnostics {
        input_point_count: points.len(),
        grid_size: (u_count, v_count),
        ..Default::default()
    };

    // Adjust degrees to fit grid size
    let degree_u = options.degree_u.min(u_count - 1).max(1);
    let degree_v = options.degree_v.min(v_count - 1).max(1);

    // Check for closure
    let close_u = options.close_u && check_u_closure(points, u_count, v_count, options.tolerance);
    let close_v = options.close_v && check_v_closure(points, u_count, v_count, options.tolerance);
    diagnostics.closed_u = close_u;
    diagnostics.closed_v = close_v;

    // Build NURBS surface
    // For interpolation through grid points, we use the grid points as control points
    // with uniform knot vectors and degree adjustments.
    let (knots_u, ctrl_u_count) = build_knot_vector(u_count, degree_u, close_u);
    let (knots_v, ctrl_v_count) = build_knot_vector(v_count, degree_v, close_v);

    // For interpolating surface, control points = input points (for degree 1)
    // For higher degrees with interpolation, we solve a system (simplified: use input directly)
    let control_points = if options.interpolate && degree_u == 1 && degree_v == 1 {
        // Bilinear interpolation: control points are the grid points
        points.to_vec()
    } else if options.interpolate {
        // For higher degree interpolation, we use the grid points as control points
        // This gives an approximating surface that is close to interpolating for dense grids.
        // True interpolation would require solving a linear system.
        if degree_u > 1 || degree_v > 1 {
            diagnostics.warnings.push(
                "Higher-degree interpolation uses control points directly; \
                 true interpolation requires linear solve (future enhancement)".to_string()
            );
        }
        points.to_vec()
    } else {
        // Approximating surface: also use grid points as control points
        // True approximation would use least-squares fitting (future enhancement)
        diagnostics.warnings.push(
            "Approximating mode currently uses direct control points; \
             least-squares fitting planned for future".to_string()
        );
        points.to_vec()
    };

    // Create the NURBS surface
    let surface = NurbsSurface::new(
        degree_u,
        degree_v,
        ctrl_u_count,
        ctrl_v_count,
        control_points,
        knots_u,
        knots_v,
        None, // No weights (non-rational)
    ).map_err(|reason| SurfaceFitError::ConstructionFailed { reason })?;

    // Compute deviation diagnostics
    let (max_dev, avg_dev) = compute_deviation(&surface, points, u_count, v_count);
    diagnostics.max_deviation = max_dev;
    diagnostics.avg_deviation = avg_dev;

    Ok((surface, diagnostics))
}

/// Fit a surface through scattered (unordered) points.
///
/// This function automatically determines an appropriate parameterization
/// for the scattered points and creates a surface. For best results, points
/// should roughly lie on or near a surface (not a volume).
///
/// # Arguments
/// * `points` - Scattered 3D points
/// * `options` - Fitting options (u/v counts are auto-detected)
///
/// # Returns
/// A surface approximating the point cloud, along with fitting diagnostics.
pub fn surface_from_scattered_points(
    points: &[Point3],
    options: SurfaceFitOptions,
) -> Result<(Box<dyn Surface>, SurfaceFitDiagnostics), SurfaceFitError> {
    if points.len() < 3 {
        return Err(SurfaceFitError::InsufficientPoints {
            provided: points.len(),
            required: 3,
        });
    }

    // Check for NaN/Inf values
    for (i, p) in points.iter().enumerate() {
        if !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite() {
            return Err(SurfaceFitError::DegeneratePoints {
                reason: format!("point {} contains NaN or infinite values", i),
            });
        }
    }

    let mut diagnostics = SurfaceFitDiagnostics {
        input_point_count: points.len(),
        ..Default::default()
    };

    // Compute best-fit plane
    let (centroid, normal, u_axis, v_axis) = fit_plane(points, options.tolerance)?;

    // For 3-4 points, create a simple planar surface
    if points.len() <= 4 {
        let bbox = compute_uv_bounds(points, centroid, u_axis, v_axis);
        let plane = PlaneSurface::new(
            centroid.add_vec(u_axis.mul_scalar(bbox.0)).add_vec(v_axis.mul_scalar(bbox.2)),
            u_axis.mul_scalar(bbox.1 - bbox.0),
            v_axis.mul_scalar(bbox.3 - bbox.2),
        );
        diagnostics.grid_size = (2, 2);
        diagnostics.warnings.push("Few points: created planar surface".to_string());

        // Compute deviation
        let max_dev = points.iter()
            .map(|p| point_to_plane_distance(*p, centroid, normal))
            .fold(0.0_f64, |acc, d| acc.max(d));
        diagnostics.max_deviation = max_dev;
        diagnostics.avg_deviation = max_dev;

        return Ok((Box::new(plane), diagnostics));
    }

    // For more points, create a grid surface by projecting onto the best-fit plane
    // and triangulating/gridding
    let (grid_points, u_count, v_count) = project_to_grid(
        points,
        centroid,
        u_axis,
        v_axis,
        normal,
        options.tolerance,
    )?;

    diagnostics.grid_size = (u_count, v_count);

    // Now fit a surface through the grid
    let (surface, grid_diag) = surface_from_grid(&grid_points, u_count, v_count, options)?;

    diagnostics.max_deviation = grid_diag.max_deviation;
    diagnostics.avg_deviation = grid_diag.avg_deviation;
    diagnostics.closed_u = grid_diag.closed_u;
    diagnostics.closed_v = grid_diag.closed_v;
    diagnostics.warnings.extend(grid_diag.warnings);

    Ok((Box::new(surface), diagnostics))
}

/// Create a mesh directly from scattered points using best-fit plane triangulation.
///
/// This is a simpler approach that creates a triangulated mesh from the projected
/// point cloud, suitable for cases where a parametric surface is not required.
///
/// # Arguments
/// * `points` - Scattered 3D points
/// * `tolerance` - Tolerance for coincident point detection
///
/// # Returns
/// A triangulated mesh and diagnostics.
pub fn mesh_from_scattered_points(
    points: &[Point3],
    tolerance: Tolerance,
) -> Result<(GeomMesh, SurfaceFitDiagnostics), SurfaceFitError> {
    let mut ctx = GeomContext::new();
    mesh_from_scattered_points_with_context(points, tolerance, &mut ctx)
}

/// Create a mesh from scattered points with a shared context.
pub fn mesh_from_scattered_points_with_context(
    points: &[Point3],
    tolerance: Tolerance,
    ctx: &mut GeomContext,
) -> Result<(GeomMesh, SurfaceFitDiagnostics), SurfaceFitError> {
    if points.len() < 3 {
        return Err(SurfaceFitError::InsufficientPoints {
            provided: points.len(),
            required: 3,
        });
    }

    ctx.metrics.begin();

    let mut diagnostics = SurfaceFitDiagnostics {
        input_point_count: points.len(),
        ..Default::default()
    };

    // Compute best-fit plane
    let (centroid, normal, u_axis, v_axis) = ctx.metrics.time(TimingBucket::SurfaceTessellation, || {
        fit_plane(points, tolerance)
    })?;

    // Project points to UV space
    let uv_points: Vec<(f64, f64, usize)> = points
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let d = p.sub_point(centroid);
            (u_axis.dot(d), v_axis.dot(d), i)
        })
        .collect();

    // Simple fan triangulation from centroid for convex-ish point sets
    // For more complex shapes, use Delaunay triangulation (future enhancement)
    let (mesh, diag) = ctx.metrics.time(TimingBucket::Triangulation, || {
        triangulate_projected_points(&uv_points, points, normal)
    });

    diagnostics.grid_size = (points.len(), 1);
    diagnostics.max_deviation = diag.max_deviation;
    diagnostics.avg_deviation = diag.avg_deviation;
    diagnostics.warnings = diag.warnings;

    Ok((mesh, diagnostics))
}

/// Create a mesh from a grid of points.
///
/// This directly meshes the grid without creating an intermediate surface,
/// which is faster for visualization purposes.
///
/// # Arguments
/// * `points` - Grid points in row-major order
/// * `u_count` - Number of points in U direction
/// * `v_count` - Number of points in V direction
///
/// # Returns
/// A triangulated mesh and diagnostics.
pub fn mesh_from_grid(
    points: &[Point3],
    u_count: usize,
    v_count: usize,
) -> Result<(GeomMesh, GeomMeshDiagnostics), SurfaceFitError> {
    let mut ctx = GeomContext::new();
    mesh_from_grid_with_context(points, u_count, v_count, &mut ctx)
}

/// Create a mesh from a grid of points with a shared context.
pub fn mesh_from_grid_with_context(
    points: &[Point3],
    u_count: usize,
    v_count: usize,
    ctx: &mut GeomContext,
) -> Result<(GeomMesh, GeomMeshDiagnostics), SurfaceFitError> {
    if u_count < 2 || v_count < 2 {
        return Err(SurfaceFitError::InvalidGridSize { u_count, v_count });
    }

    let expected = u_count * v_count;
    if points.len() != expected {
        return Err(SurfaceFitError::GridSizeMismatch {
            expected,
            provided: points.len(),
        });
    }

    ctx.metrics.begin();

    // Build triangle indices (two triangles per quad)
    let mut indices = Vec::with_capacity((u_count - 1) * (v_count - 1) * 6);

    for v in 0..(v_count - 1) {
        for u in 0..(u_count - 1) {
            let i00 = (v * u_count + u) as u32;
            let i10 = (v * u_count + u + 1) as u32;
            let i01 = ((v + 1) * u_count + u) as u32;
            let i11 = ((v + 1) * u_count + u + 1) as u32;

            // First triangle: i00, i10, i11
            indices.push(i00);
            indices.push(i10);
            indices.push(i11);

            // Second triangle: i00, i11, i01
            indices.push(i00);
            indices.push(i11);
            indices.push(i01);
        }
    }

    // Compute UVs
    let uvs: Vec<[f64; 2]> = (0..v_count)
        .flat_map(|v| {
            let vt = v as f64 / (v_count - 1).max(1) as f64;
            (0..u_count).map(move |u| {
                let ut = u as f64 / (u_count - 1).max(1) as f64;
                [ut, vt]
            })
        })
        .collect();

    // Finalize with normals and diagnostics
    let (mesh, diagnostics) = finalize_mesh(
        points.to_vec(),
        Some(uvs),
        indices,
        ctx.tolerance,
    );

    Ok((mesh, diagnostics))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Build a clamped uniform knot vector for NURBS.
fn build_knot_vector(point_count: usize, degree: usize, _closed: bool) -> (Vec<f64>, usize) {
    // For now, ignore closed and create open/clamped knot vector
    // Closed surfaces need periodic knots (future enhancement)

    let n = point_count; // number of control points
    let p = degree;
    let knot_count = n + p + 1;

    let mut knots = Vec::with_capacity(knot_count);

    // Clamped: first (p+1) knots are 0, last (p+1) are 1
    for _ in 0..=p {
        knots.push(0.0);
    }

    // Interior knots
    let interior_count = knot_count - 2 * (p + 1);
    for i in 1..=interior_count {
        knots.push(i as f64 / (interior_count + 1) as f64);
    }

    for _ in 0..=p {
        knots.push(1.0);
    }

    (knots, n)
}

/// Check if U edges are closed (first and last columns coincide).
fn check_u_closure(points: &[Point3], u_count: usize, v_count: usize, tol: Tolerance) -> bool {
    if u_count < 2 {
        return false;
    }
    for v in 0..v_count {
        let first = points[v * u_count];
        let last = points[v * u_count + u_count - 1];
        if !tol.approx_eq_point3(first, last) {
            return false;
        }
    }
    true
}

/// Check if V edges are closed (first and last rows coincide).
fn check_v_closure(points: &[Point3], u_count: usize, v_count: usize, tol: Tolerance) -> bool {
    if v_count < 2 {
        return false;
    }
    for u in 0..u_count {
        let first = points[u];
        let last = points[(v_count - 1) * u_count + u];
        if !tol.approx_eq_point3(first, last) {
            return false;
        }
    }
    true
}

/// Compute deviation between surface and input points.
fn compute_deviation(
    surface: &NurbsSurface,
    points: &[Point3],
    u_count: usize,
    v_count: usize,
) -> (f64, f64) {
    let mut max_dev = 0.0_f64;
    let mut sum_dev = 0.0_f64;

    for (i, point) in points.iter().enumerate() {
        let v_idx = i / u_count;
        let u_idx = i % u_count;

        let u = u_idx as f64 / (u_count - 1).max(1) as f64;
        let v = v_idx as f64 / (v_count - 1).max(1) as f64;

        let surf_pt = surface.point_at(u, v);
        let dev = point.sub_point(surf_pt).length();

        max_dev = max_dev.max(dev);
        sum_dev += dev;
    }

    let avg_dev = if points.is_empty() { 0.0 } else { sum_dev / points.len() as f64 };
    (max_dev, avg_dev)
}

/// Fit a plane to a set of points using PCA.
fn fit_plane(
    points: &[Point3],
    tol: Tolerance,
) -> Result<(Point3, Vec3, Vec3, Vec3), SurfaceFitError> {
    if points.len() < 3 {
        return Err(SurfaceFitError::InsufficientPoints {
            provided: points.len(),
            required: 3,
        });
    }

    // Compute centroid
    let n = points.len() as f64;
    let centroid = {
        let sum = points.iter().fold(Vec3::new(0.0, 0.0, 0.0), |acc, p| {
            Vec3::new(acc.x + p.x, acc.y + p.y, acc.z + p.z)
        });
        Point3::new(sum.x / n, sum.y / n, sum.z / n)
    };

    // Compute covariance matrix
    let mut cov = [[0.0_f64; 3]; 3];
    for p in points {
        let d = p.sub_point(centroid);
        cov[0][0] += d.x * d.x;
        cov[0][1] += d.x * d.y;
        cov[0][2] += d.x * d.z;
        cov[1][1] += d.y * d.y;
        cov[1][2] += d.y * d.z;
        cov[2][2] += d.z * d.z;
    }
    cov[1][0] = cov[0][1];
    cov[2][0] = cov[0][2];
    cov[2][1] = cov[1][2];

    // Find eigenvectors using power iteration (simplified)
    // The normal is the eigenvector with smallest eigenvalue
    let (normal, u_axis, v_axis) = compute_plane_axes(&cov, tol)?;

    Ok((centroid, normal, u_axis, v_axis))
}

/// Compute plane axes from covariance matrix.
fn compute_plane_axes(
    cov: &[[f64; 3]; 3],
    tol: Tolerance,
) -> Result<(Vec3, Vec3, Vec3), SurfaceFitError> {
    // Simple eigenvalue decomposition using power iteration
    // We find the three orthogonal axes

    // Start with initial guesses
    let axes = [
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
    ];

    // Power iteration to find dominant eigenvector
    let iterations = 20;
    let dominant = power_iteration(cov, axes[0], iterations);

    // Find second eigenvector orthogonal to first
    let mut second = if dominant.x.abs() < 0.9 {
        Vec3::new(1.0, 0.0, 0.0)
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    };
    second = second.sub(dominant.mul_scalar(second.dot(dominant)));
    let second = power_iteration(cov, second.normalized().unwrap_or(second), iterations);

    // Third is cross product
    let third = dominant.cross(second);

    // Normalize
    let ax1 = dominant.normalized();
    let ax2 = second.normalized();
    let ax3 = third.normalized();

    match (ax1, ax2, ax3) {
        (Some(u), Some(v), Some(n)) => {
            // Compute eigenvalues (Rayleigh quotient)
            let e1 = rayleigh_quotient(cov, u);
            let e2 = rayleigh_quotient(cov, v);
            let e3 = rayleigh_quotient(cov, n);

            // Normal is the axis with smallest eigenvalue
            let mut sorted = [(e1, u), (e2, v), (e3, n)];
            sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

            let normal = sorted[0].1;
            let u_axis = sorted[2].1;
            let v_axis = normal.cross(u_axis).normalized().unwrap_or(sorted[1].1);

            // Check for degeneracy
            if sorted[0].0 > tol.eps && (sorted[1].0 - sorted[0].0).abs() < tol.eps {
                return Err(SurfaceFitError::DegeneratePoints {
                    reason: "points are collinear or coincident".to_string(),
                });
            }

            Ok((normal, u_axis, v_axis))
        }
        _ => Err(SurfaceFitError::DegeneratePoints {
            reason: "could not compute plane axes".to_string(),
        }),
    }
}

/// Power iteration to find dominant eigenvector.
fn power_iteration(cov: &[[f64; 3]; 3], initial: Vec3, iterations: usize) -> Vec3 {
    let mut v = initial;
    for _ in 0..iterations {
        let new_v = Vec3::new(
            cov[0][0] * v.x + cov[0][1] * v.y + cov[0][2] * v.z,
            cov[1][0] * v.x + cov[1][1] * v.y + cov[1][2] * v.z,
            cov[2][0] * v.x + cov[2][1] * v.y + cov[2][2] * v.z,
        );
        v = new_v.normalized().unwrap_or(new_v);
    }
    v
}

/// Rayleigh quotient for eigenvalue estimation.
fn rayleigh_quotient(cov: &[[f64; 3]; 3], v: Vec3) -> f64 {
    let mv = Vec3::new(
        cov[0][0] * v.x + cov[0][1] * v.y + cov[0][2] * v.z,
        cov[1][0] * v.x + cov[1][1] * v.y + cov[1][2] * v.z,
        cov[2][0] * v.x + cov[2][1] * v.y + cov[2][2] * v.z,
    );
    v.dot(mv) / v.dot(v).max(1e-12)
}

/// Compute UV bounds for points projected onto plane.
fn compute_uv_bounds(
    points: &[Point3],
    centroid: Point3,
    u_axis: Vec3,
    v_axis: Vec3,
) -> (f64, f64, f64, f64) {
    let mut u_min = f64::MAX;
    let mut u_max = f64::MIN;
    let mut v_min = f64::MAX;
    let mut v_max = f64::MIN;

    for p in points {
        let d = p.sub_point(centroid);
        let u = u_axis.dot(d);
        let v = v_axis.dot(d);
        u_min = u_min.min(u);
        u_max = u_max.max(u);
        v_min = v_min.min(v);
        v_max = v_max.max(v);
    }

    (u_min, u_max, v_min, v_max)
}

/// Distance from point to plane.
fn point_to_plane_distance(point: Point3, plane_origin: Point3, plane_normal: Vec3) -> f64 {
    let d = point.sub_point(plane_origin);
    plane_normal.dot(d).abs()
}

/// Project scattered points onto a grid.
fn project_to_grid(
    points: &[Point3],
    centroid: Point3,
    u_axis: Vec3,
    v_axis: Vec3,
    _normal: Vec3,
    tol: Tolerance,
) -> Result<(Vec<Point3>, usize, usize), SurfaceFitError> {
    // Compute UV bounds
    let (u_min, u_max, v_min, v_max) = compute_uv_bounds(points, centroid, u_axis, v_axis);

    let u_span = u_max - u_min;
    let v_span = v_max - v_min;

    if u_span < tol.eps && v_span < tol.eps {
        return Err(SurfaceFitError::DegeneratePoints {
            reason: "all points coincide".to_string(),
        });
    }

    // Determine grid resolution based on point count
    let n = points.len();
    let aspect = if v_span > tol.eps { u_span / v_span } else { 1.0 };
    let total_cells = ((n as f64).sqrt().ceil() as usize).max(2);
    let u_count = (total_cells as f64 * aspect.sqrt()).ceil() as usize;
    let u_count = u_count.clamp(2, n.min(50));
    let v_count = (total_cells as f64 / aspect.sqrt()).ceil() as usize;
    let v_count = v_count.clamp(2, n.min(50));

    // Create grid by averaging nearby points
    let u_step = if u_count > 1 { u_span / (u_count - 1) as f64 } else { u_span };
    let v_step = if v_count > 1 { v_span / (v_count - 1) as f64 } else { v_span };

    let mut grid_points = Vec::with_capacity(u_count * v_count);

    for vi in 0..v_count {
        for ui in 0..u_count {
            let target_u = u_min + ui as f64 * u_step;
            let target_v = v_min + vi as f64 * v_step;

            // Find closest point or interpolate
            let grid_pt = interpolate_grid_point(
                points,
                centroid,
                u_axis,
                v_axis,
                target_u,
                target_v,
                u_step.max(v_step),
            );
            grid_points.push(grid_pt);
        }
    }

    Ok((grid_points, u_count, v_count))
}

/// Interpolate a grid point from scattered points using inverse distance weighting.
fn interpolate_grid_point(
    points: &[Point3],
    centroid: Point3,
    u_axis: Vec3,
    v_axis: Vec3,
    target_u: f64,
    target_v: f64,
    radius: f64,
) -> Point3 {
    let mut weighted_sum = Vec3::new(0.0, 0.0, 0.0);
    let mut weight_total = 0.0;

    for p in points {
        let d = p.sub_point(centroid);
        let pu = u_axis.dot(d);
        let pv = v_axis.dot(d);

        let dist_sq = (pu - target_u).powi(2) + (pv - target_v).powi(2);
        let dist = dist_sq.sqrt();

        if dist < 1e-12 {
            // Exact match
            return *p;
        }

        // Inverse distance weighting with Gaussian falloff
        let weight = (-dist_sq / (radius * radius).max(1e-12)).exp();
        weighted_sum = weighted_sum.add(Vec3::new(p.x * weight, p.y * weight, p.z * weight));
        weight_total += weight;
    }

    if weight_total > 1e-12 {
        Point3::new(
            weighted_sum.x / weight_total,
            weighted_sum.y / weight_total,
            weighted_sum.z / weight_total,
        )
    } else {
        // Fallback to centroid
        centroid
    }
}

/// Triangulate projected points using Delaunay triangulation.
fn triangulate_projected_points(
    uv_points: &[(f64, f64, usize)],
    original_points: &[Point3],
    _normal: Vec3,
) -> (GeomMesh, SurfaceFitDiagnostics) {
    let mut diagnostics = SurfaceFitDiagnostics::default();
    diagnostics.input_point_count = original_points.len();

    if original_points.len() < 3 {
        return (
            GeomMesh {
                positions: original_points.iter().map(|p| p.to_array()).collect(),
                indices: vec![],
                uvs: None,
                normals: None,
                tangents: None,
            },
            diagnostics,
        );
    }

    // Extract UV coordinates for Delaunay triangulation
    let uv_coords: Vec<(f64, f64)> = uv_points.iter().map(|(u, v, _)| (*u, *v)).collect();
    let original_indices: Vec<usize> = uv_points.iter().map(|(_, _, i)| *i).collect();

    // Perform Delaunay triangulation
    match delaunay_triangulate(&uv_coords) {
        Ok(triangles) => {
            // Build positions array preserving original order
            let positions: Vec<[f64; 3]> = original_points.iter().map(|p| p.to_array()).collect();

            // Convert triangle indices to refer to original point indices
            let mut indices = Vec::with_capacity(triangles.len() * 3);
            for tri in &triangles {
                indices.push(original_indices[tri.0] as u32);
                indices.push(original_indices[tri.1] as u32);
                indices.push(original_indices[tri.2] as u32);
            }

            (
                GeomMesh {
                    positions,
                    indices,
                    uvs: None,
                    normals: None,
                    tangents: None,
                },
                diagnostics,
            )
        }
        Err(reason) => {
            // Fallback to fan triangulation if Delaunay fails
            diagnostics.warnings.push(format!("Delaunay failed ({}); using fan triangulation", reason));
            triangulate_fan_fallback(uv_points, original_points, &mut diagnostics)
        }
    }
}

/// Fallback fan triangulation when Delaunay fails.
fn triangulate_fan_fallback(
    uv_points: &[(f64, f64, usize)],
    original_points: &[Point3],
    diagnostics: &mut SurfaceFitDiagnostics,
) -> (GeomMesh, SurfaceFitDiagnostics) {
    // Sort by angle from centroid for fan triangulation
    let centroid_uv = {
        let (sum_u, sum_v): (f64, f64) = uv_points.iter().map(|(u, v, _)| (*u, *v)).fold(
            (0.0, 0.0),
            |(su, sv), (u, v)| (su + u, sv + v),
        );
        let n = uv_points.len() as f64;
        (sum_u / n, sum_v / n)
    };

    let mut sorted: Vec<(f64, usize)> = uv_points
        .iter()
        .map(|(u, v, i)| {
            let angle = (v - centroid_uv.1).atan2(u - centroid_uv.0);
            (angle, *i)
        })
        .collect();
    sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    // Create positions in sorted order
    let positions: Vec<[f64; 3]> = sorted
        .iter()
        .map(|(_, i)| original_points[*i].to_array())
        .collect();

    let n = positions.len();

    // Fan triangulation from first vertex
    let mut indices = Vec::with_capacity((n - 2) * 3);
    for i in 1..(n - 1) {
        indices.push(0);
        indices.push(i as u32);
        indices.push((i + 1) as u32);
    }

    diagnostics.warnings.push("Used fan triangulation fallback".to_string());

    (
        GeomMesh {
            positions,
            indices,
            uvs: None,
            normals: None,
            tangents: None,
        },
        diagnostics.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_from_grid_2x2() {
        let points = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
        ];

        let result = surface_from_grid(&points, 2, 2, SurfaceFitOptions::default());
        assert!(result.is_ok());

        let (surface, diagnostics) = result.unwrap();
        assert_eq!(diagnostics.input_point_count, 4);
        assert_eq!(diagnostics.grid_size, (2, 2));

        // Check corner points
        let tol = Tolerance::default_geom();
        assert!(tol.approx_eq_point3(surface.point_at(0.0, 0.0), Point3::new(0.0, 0.0, 0.0)));
        assert!(tol.approx_eq_point3(surface.point_at(1.0, 0.0), Point3::new(1.0, 0.0, 0.0)));
        assert!(tol.approx_eq_point3(surface.point_at(0.0, 1.0), Point3::new(0.0, 1.0, 0.0)));
        assert!(tol.approx_eq_point3(surface.point_at(1.0, 1.0), Point3::new(1.0, 1.0, 0.0)));
    }

    #[test]
    fn test_surface_from_grid_3x3() {
        let points = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(0.5, 0.0, 0.1),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 0.5, 0.1),
            Point3::new(0.5, 0.5, 0.2),
            Point3::new(1.0, 0.5, 0.1),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(0.5, 1.0, 0.1),
            Point3::new(1.0, 1.0, 0.0),
        ];

        let result = surface_from_grid(&points, 3, 3, SurfaceFitOptions::default());
        assert!(result.is_ok());

        let (_, diagnostics) = result.unwrap();
        assert_eq!(diagnostics.input_point_count, 9);
        assert_eq!(diagnostics.grid_size, (3, 3));
    }

    #[test]
    fn test_surface_from_grid_invalid_size() {
        let points = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
        ];

        let result = surface_from_grid(&points, 1, 2, SurfaceFitOptions::default());
        assert!(matches!(result, Err(SurfaceFitError::InvalidGridSize { .. })));
    }

    #[test]
    fn test_surface_from_grid_size_mismatch() {
        let points = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ];

        let result = surface_from_grid(&points, 2, 2, SurfaceFitOptions::default());
        assert!(matches!(result, Err(SurfaceFitError::GridSizeMismatch { .. })));
    }

    #[test]
    fn test_mesh_from_grid() {
        let points = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(2.0, 1.0, 0.0),
        ];

        let result = mesh_from_grid(&points, 3, 2);
        assert!(result.is_ok());

        let (mesh, diagnostics) = result.unwrap();
        assert_eq!(mesh.positions.len(), 6);
        // 2x1 grid of quads = 2 quads = 4 triangles = 12 indices
        assert_eq!(mesh.indices.len(), 12);
        assert_eq!(diagnostics.vertex_count, 6);
        assert_eq!(diagnostics.triangle_count, 4);
    }

    #[test]
    fn test_fit_plane_simple() {
        let points = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
        ];

        let result = fit_plane(&points, Tolerance::default_geom());
        assert!(result.is_ok());

        let (centroid, normal, _, _) = result.unwrap();

        // Centroid should be at (0.5, 0.5, 0)
        let tol = Tolerance::new(0.01);
        assert!(tol.approx_eq_point3(centroid, Point3::new(0.5, 0.5, 0.0)));

        // Normal should be close to Z axis (either +Z or -Z)
        assert!(normal.z.abs() > 0.99);
    }

    #[test]
    fn test_scattered_points_planar() {
        let points = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ];

        let result = surface_from_scattered_points(&points, SurfaceFitOptions::default());
        assert!(result.is_ok());

        let (_, diagnostics) = result.unwrap();
        assert_eq!(diagnostics.input_point_count, 3);
        // Max deviation should be small for coplanar points
        assert!(diagnostics.max_deviation < 0.1);
    }

    #[test]
    fn test_mesh_from_scattered_points() {
        let points = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.5, 1.0, 0.0),
            Point3::new(0.5, 0.5, 0.1),
        ];

        let result = mesh_from_scattered_points(&points, Tolerance::default_geom());
        assert!(result.is_ok());

        let (mesh, _) = result.unwrap();
        assert!(!mesh.positions.is_empty());
        assert!(!mesh.indices.is_empty());
    }

    #[test]
    fn test_u_closure_detection() {
        // Grid where first and last columns are the same (closed in U)
        let points = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 0.0, 0.0), // Same as first column
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0), // Same as first column
        ];

        let tol = Tolerance::default_geom();
        assert!(check_u_closure(&points, 3, 2, tol));
        assert!(!check_v_closure(&points, 3, 2, tol));
    }

    #[test]
    fn test_v_closure_detection() {
        // Grid where first and last rows are the same (closed in V)
        let points = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 0.0, 0.0), // Same as first row
            Point3::new(1.0, 0.0, 0.0), // Same as first row
        ];

        let tol = Tolerance::default_geom();
        assert!(!check_u_closure(&points, 2, 3, tol));
        assert!(check_v_closure(&points, 2, 3, tol));
    }

    #[test]
    fn test_nan_point_rejection() {
        let points = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(f64::NAN, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
        ];

        let result = surface_from_grid(&points, 2, 2, SurfaceFitOptions::default());
        assert!(matches!(result, Err(SurfaceFitError::DegeneratePoints { .. })));
    }

    #[test]
    fn test_infinite_point_rejection() {
        let points = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, f64::INFINITY, 0.0),
        ];

        let result = surface_from_scattered_points(&points, SurfaceFitOptions::default());
        assert!(matches!(result, Err(SurfaceFitError::DegeneratePoints { .. })));
    }
}
