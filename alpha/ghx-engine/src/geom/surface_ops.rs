//! Higher-level surface operations for component integration.
//!
//! This module provides surface utility functions that operate on legacy mesh
//! representations (vertices/faces from `Value::Surface`) while leveraging the
//! `geom` kernel's surface abstractions where applicable.
//!
//! # Design Philosophy
//!
//! These functions bridge the gap between:
//! - Legacy `Value::Surface { vertices, faces }` representations in components
//! - The type-safe `geom::Surface` trait implementations
//!
//! ## Vertex-Based Operations (Recommended)
//!
//! When vertex data is available, use the `_from_vertices` variants which preserve
//! the actual surface orientation and curvature:
//!
//! - [`divide_surface_from_vertices`]: Samples using bilinear interpolation over
//!   the vertex grid, preserving the surface's true geometry.
//! - [`surface_frames_from_vertices`]: Computes coordinate frames on the actual
//!   surface with correct normals and tangent directions.
//! - [`isotrim_from_vertices`]: Extracts subsurfaces using true vertex positions.
//!
//! ## Bounds-Based Operations (Legacy/Fallback)
//!
//! When only bounding-box information is available, these functions create an
//! axis-aligned `PlaneSurface` approximation. **Note**: This loses orientation
//! for rotated surfaces and curvature for curved surfaces.
//!
//! - [`divide_surface_from_bounds`]: Axis-aligned grid sampling
//! - [`surface_frames_from_bounds`]: Axis-aligned frame computation
//! - [`isotrim_from_bounds`]: Axis-aligned subsurface extraction
//!
//! # Usage
//!
//! Components should:
//! 1. Try to extract vertex grid data from inputs (preserves geometry)
//! 2. Fall back to bounds-based operations only if vertices unavailable
//! 3. Convert the result back to `Value` types for output

use super::analysis::{surface_frames as geom_surface_frames, SurfaceFrame, SurfaceFramesResult};
use super::surface::{
    divide_surface as geom_divide_surface, isotrim_surface as geom_isotrim_surface,
    ClosedSurfaceSampling, DivideSurfaceOptions, DivideSurfaceResult, IsotrimDiagnostics,
    IsotrimSurface, PlaneSurface, Surface, SurfaceCacheKey,
};
use super::{Point3, Tolerance, Vec3};

// ============================================================================
// VertexGridSurface - A surface defined by a grid of vertices
// ============================================================================

/// A surface defined by a grid of vertices with bilinear interpolation.
///
/// This surface type preserves the actual geometry of mesh-based surfaces,
/// including rotated planes and curved surfaces. It samples using bilinear
/// interpolation between the grid vertices.
///
/// # Grid Layout
///
/// Vertices are organized in row-major order:
/// - `u_count` vertices per row (U direction)
/// - `v_count` rows (V direction)
/// - Total vertices = `u_count * v_count`
///
/// The parametric domain is [0, 1] × [0, 1], mapping to the grid corners.
///
/// # Example
///
/// ```ignore
/// // Create a rotated square surface
/// let vertices = vec![
///     Point3::new(0.0, 0.0, 0.0),  // (u=0, v=0)
///     Point3::new(1.0, 1.0, 0.0),  // (u=1, v=0)
///     Point3::new(-1.0, 1.0, 0.0), // (u=0, v=1)
///     Point3::new(0.0, 2.0, 0.0),  // (u=1, v=1)
/// ];
/// let surface = VertexGridSurface::new(vertices, 2, 2);
/// ```
#[derive(Debug, Clone)]
pub struct VertexGridSurface {
    vertices: Vec<Point3>,
    u_count: usize,
    v_count: usize,
    /// Cached normal computed from first cell for efficiency
    cached_normal: Option<Vec3>,
}

impl VertexGridSurface {
    /// Creates a new vertex grid surface.
    ///
    /// # Arguments
    ///
    /// * `vertices` - Grid vertices in row-major order (u varies fastest)
    /// * `u_count` - Number of vertices in U direction (must be ≥ 2)
    /// * `v_count` - Number of vertices in V direction (must be ≥ 2)
    ///
    /// # Returns
    ///
    /// `None` if the grid dimensions don't match the vertex count, or if
    /// dimensions are less than 2.
    #[must_use]
    pub fn new(vertices: Vec<Point3>, u_count: usize, v_count: usize) -> Option<Self> {
        if u_count < 2 || v_count < 2 {
            return None;
        }
        if vertices.len() != u_count * v_count {
            return None;
        }

        let mut surface = Self {
            vertices,
            u_count,
            v_count,
            cached_normal: None,
        };
        surface.cached_normal = surface.compute_average_normal();
        Some(surface)
    }

    /// Creates a vertex grid surface from array coordinates.
    ///
    /// Convenience constructor that converts `[f64; 3]` arrays to `Point3`.
    #[must_use]
    pub fn from_arrays(vertices: &[[f64; 3]], u_count: usize, v_count: usize) -> Option<Self> {
        let points: Vec<Point3> = vertices.iter().map(|v| Point3::new(v[0], v[1], v[2])).collect();
        Self::new(points, u_count, v_count)
    }

    /// Attempts to infer grid dimensions from a vertex list.
    ///
    /// Uses heuristics to detect the grid structure:
    /// 1. If vertex count is a perfect square, assume square grid
    /// 2. Otherwise, try common aspect ratios (2:1, 3:1, etc.)
    /// 3. Falls back to treating as a single row if no pattern found
    ///
    /// Returns `(u_count, v_count)` or `None` if inference fails.
    #[must_use]
    pub fn infer_grid_dimensions(vertex_count: usize) -> Option<(usize, usize)> {
        if vertex_count < 4 {
            return None;
        }

        // Check for perfect square
        let sqrt = (vertex_count as f64).sqrt();
        if sqrt.fract() == 0.0 {
            let n = sqrt as usize;
            return Some((n, n));
        }

        // Find the factor pair closest to square root
        // Start from sqrt and work down to find the largest factor <= sqrt
        let sqrt_floor = (vertex_count as f64).sqrt() as usize;
        let mut best_pair: Option<(usize, usize)> = None;

        for divisor in (2..=sqrt_floor).rev() {
            if vertex_count % divisor == 0 {
                let other = vertex_count / divisor;
                if other >= 2 && divisor >= 2 {
                    // Return with larger dimension as U (more common for surfaces)
                    best_pair = Some((other.max(divisor), other.min(divisor)));
                    break; // First valid factor from sqrt is closest to square
                }
            }
        }

        best_pair
    }

    /// Returns the grid dimensions (u_count, v_count).
    #[must_use]
    pub fn dimensions(&self) -> (usize, usize) {
        (self.u_count, self.v_count)
    }

    /// Gets the vertex at grid position (i, j).
    #[must_use]
    fn vertex_at(&self, i: usize, j: usize) -> Point3 {
        let idx = j * self.u_count + i;
        self.vertices.get(idx).copied().unwrap_or(Point3::ORIGIN)
    }

    /// Computes the average surface normal from all grid cells.
    fn compute_average_normal(&self) -> Option<Vec3> {
        let mut sum = Vec3::new(0.0, 0.0, 0.0);
        let mut count = 0;

        for j in 0..(self.v_count - 1) {
            for i in 0..(self.u_count - 1) {
                let p00 = self.vertex_at(i, j);
                let p10 = self.vertex_at(i + 1, j);
                let p01 = self.vertex_at(i, j + 1);

                let u_edge = p10.sub_point(p00);
                let v_edge = p01.sub_point(p00);

                if let Some(n) = u_edge.cross(v_edge).normalized() {
                    sum = sum.add(n);
                    count += 1;
                }
            }
        }

        if count > 0 {
            sum.mul_scalar(1.0 / count as f64).normalized()
        } else {
            None
        }
    }

    /// Performs bilinear interpolation to get a point at parametric coordinates.
    fn bilinear_interpolate(&self, u: f64, v: f64) -> Point3 {
        // Map [0,1] to grid cell indices
        let u_span = (self.u_count - 1) as f64;
        let v_span = (self.v_count - 1) as f64;

        let u_scaled = u * u_span;
        let v_scaled = v * v_span;

        let i = (u_scaled.floor() as usize).min(self.u_count - 2);
        let j = (v_scaled.floor() as usize).min(self.v_count - 2);

        let u_frac = u_scaled - i as f64;
        let v_frac = v_scaled - j as f64;

        // Get cell corners
        let p00 = self.vertex_at(i, j);
        let p10 = self.vertex_at(i + 1, j);
        let p01 = self.vertex_at(i, j + 1);
        let p11 = self.vertex_at(i + 1, j + 1);

        // Bilinear interpolation
        let p0 = p00.lerp(p10, u_frac);
        let p1 = p01.lerp(p11, u_frac);
        p0.lerp(p1, v_frac)
    }

    /// Computes partial derivatives at a parametric point using finite differences.
    fn compute_derivatives(&self, u: f64, v: f64) -> (Vec3, Vec3) {
        let u_span = (self.u_count - 1) as f64;
        let v_span = (self.v_count - 1) as f64;

        // Use a step size appropriate for the grid resolution
        let h_u = 0.5 / u_span;
        let h_v = 0.5 / v_span;

        let u_lo = (u - h_u).max(0.0);
        let u_hi = (u + h_u).min(1.0);
        let v_lo = (v - h_v).max(0.0);
        let v_hi = (v + h_v).min(1.0);

        let du = if u_hi > u_lo {
            let p_lo = self.bilinear_interpolate(u_lo, v);
            let p_hi = self.bilinear_interpolate(u_hi, v);
            p_hi.sub_point(p_lo).mul_scalar(1.0 / (u_hi - u_lo))
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };

        let dv = if v_hi > v_lo {
            let p_lo = self.bilinear_interpolate(u, v_lo);
            let p_hi = self.bilinear_interpolate(u, v_hi);
            p_hi.sub_point(p_lo).mul_scalar(1.0 / (v_hi - v_lo))
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        };

        (du, dv)
    }
}

impl Surface for VertexGridSurface {
    fn point_at(&self, u: f64, v: f64) -> Point3 {
        let u_clamped = u.clamp(0.0, 1.0);
        let v_clamped = v.clamp(0.0, 1.0);
        self.bilinear_interpolate(u_clamped, v_clamped)
    }

    fn domain_u(&self) -> (f64, f64) {
        (0.0, 1.0)
    }

    fn domain_v(&self) -> (f64, f64) {
        (0.0, 1.0)
    }

    fn normal_at(&self, u: f64, v: f64) -> Option<Vec3> {
        let (du, dv) = self.compute_derivatives(u.clamp(0.0, 1.0), v.clamp(0.0, 1.0));
        du.cross(dv).normalized().or(self.cached_normal)
    }

    fn partial_derivatives_at(&self, u: f64, v: f64) -> (Vec3, Vec3) {
        self.compute_derivatives(u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
    }

    fn cache_key(&self) -> SurfaceCacheKey {
        // Use a hash of the vertex data for caching
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.u_count.hash(&mut hasher);
        self.v_count.hash(&mut hasher);
        for v in &self.vertices {
            v.x.to_bits().hash(&mut hasher);
            v.y.to_bits().hash(&mut hasher);
            v.z.to_bits().hash(&mut hasher);
        }
        SurfaceCacheKey::Nurbs {
            hash: hasher.finish(),
        }
    }
}

// ============================================================================
// Best-Fit Plane Calculation
// ============================================================================

/// Computes a best-fit plane through a set of points using PCA.
///
/// Returns a `PlaneSurface` with origin at the centroid and axes aligned
/// to the principal directions of the point cloud. This preserves the
/// actual orientation of the geometry.
///
/// # Arguments
///
/// * `points` - The points to fit the plane to
///
/// # Returns
///
/// A `PlaneSurface` oriented to match the point distribution, or `None` if
/// the points are degenerate (colinear or fewer than 3 points).
#[must_use]
pub fn best_fit_plane(points: &[Point3]) -> Option<PlaneSurface> {
    if points.len() < 3 {
        return None;
    }

    // Compute centroid
    let mut centroid = Point3::ORIGIN;
    for p in points {
        centroid = centroid.add_vec(Vec3::new(p.x, p.y, p.z));
    }
    let n = points.len() as f64;
    let origin = Point3::new(centroid.x / n, centroid.y / n, centroid.z / n);

    // Build covariance matrix
    let mut cov = [[0.0f64; 3]; 3];
    for p in points {
        let d = [p.x - origin.x, p.y - origin.y, p.z - origin.z];
        for i in 0..3 {
            for j in 0..3 {
                cov[i][j] += d[i] * d[j];
            }
        }
    }

    // Simple power iteration to find principal eigenvectors
    // (sufficient for planes; full SVD would be overkill)
    let (u_axis, v_axis) = compute_plane_axes(&cov)?;

    // Scale axes to span the point cloud
    let mut u_min = f64::INFINITY;
    let mut u_max = f64::NEG_INFINITY;
    let mut v_min = f64::INFINITY;
    let mut v_max = f64::NEG_INFINITY;

    for p in points {
        let d = Vec3::new(p.x - origin.x, p.y - origin.y, p.z - origin.z);
        let u_proj = d.dot(u_axis);
        let v_proj = d.dot(v_axis);
        u_min = u_min.min(u_proj);
        u_max = u_max.max(u_proj);
        v_min = v_min.min(v_proj);
        v_max = v_max.max(v_proj);
    }

    let u_span = (u_max - u_min).max(1e-10);
    let v_span = (v_max - v_min).max(1e-10);

    // Adjust origin to corner and scale axes
    let corner = origin
        .add_vec(u_axis.mul_scalar(u_min))
        .add_vec(v_axis.mul_scalar(v_min));

    Some(PlaneSurface::new(
        corner,
        u_axis.mul_scalar(u_span),
        v_axis.mul_scalar(v_span),
    ))
}

/// Computes the two largest principal axes from a covariance matrix.
fn compute_plane_axes(cov: &[[f64; 3]; 3]) -> Option<(Vec3, Vec3)> {
    // Power iteration to find dominant eigenvector
    let mut v1 = [1.0, 0.0, 0.0];
    for _ in 0..20 {
        let mut result = [0.0; 3];
        for i in 0..3 {
            for j in 0..3 {
                result[i] += cov[i][j] * v1[j];
            }
        }
        let len = (result[0] * result[0] + result[1] * result[1] + result[2] * result[2]).sqrt();
        if len < 1e-12 {
            break;
        }
        v1 = [result[0] / len, result[1] / len, result[2] / len];
    }

    // Deflate matrix and find second eigenvector
    let mut cov2 = *cov;
    let lambda1 = {
        let mut mv = [0.0; 3];
        for i in 0..3 {
            for j in 0..3 {
                mv[i] += cov[i][j] * v1[j];
            }
        }
        mv[0] * v1[0] + mv[1] * v1[1] + mv[2] * v1[2]
    };
    for i in 0..3 {
        for j in 0..3 {
            cov2[i][j] -= lambda1 * v1[i] * v1[j];
        }
    }

    let mut v2 = [0.0, 1.0, 0.0];
    for _ in 0..20 {
        let mut result = [0.0; 3];
        for i in 0..3 {
            for j in 0..3 {
                result[i] += cov2[i][j] * v2[j];
            }
        }
        let len = (result[0] * result[0] + result[1] * result[1] + result[2] * result[2]).sqrt();
        if len < 1e-12 {
            break;
        }
        v2 = [result[0] / len, result[1] / len, result[2] / len];
    }

    let u_axis = Vec3::new(v1[0], v1[1], v1[2]);
    let v_axis = Vec3::new(v2[0], v2[1], v2[2]);

    // Ensure axes are orthogonal
    let v_axis_orth = v_axis.sub(u_axis.mul_scalar(u_axis.dot(v_axis)));

    let u_norm = u_axis.normalized()?;
    let v_norm = v_axis_orth.normalized()?;

    Some((u_norm, v_norm))
}

// ============================================================================
// DivideSurface Operations
// ============================================================================

/// Options for surface division using bounds-based input.
#[derive(Debug, Clone, Copy)]
pub struct DivideSurfaceBoundsOptions {
    /// Strategy for sampling closed surfaces in U direction.
    pub closed_u: ClosedSurfaceSampling,
    /// Strategy for sampling closed surfaces in V direction.
    pub closed_v: ClosedSurfaceSampling,
}

impl Default for DivideSurfaceBoundsOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl DivideSurfaceBoundsOptions {
    /// Creates options with default exclude-seam behavior.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            closed_u: ClosedSurfaceSampling::ExcludeSeam,
            closed_v: ClosedSurfaceSampling::ExcludeSeam,
        }
    }
}

/// Result of dividing a surface using bounds-based input.
#[derive(Debug, Clone)]
pub struct DivideSurfaceBoundsResult {
    /// Sample points on the surface grid.
    pub points: Vec<[f64; 3]>,
    /// Surface normals at each sample point.
    pub normals: Vec<[f64; 3]>,
    /// UV parameters at each sample point (stored as [u, v, 0.0]).
    pub parameters: Vec<[f64; 3]>,
    /// Number of samples in U direction.
    pub u_count: usize,
    /// Number of samples in V direction.
    pub v_count: usize,
}

/// Divides a surface defined by bounding box min/max into a grid of sample points.
///
/// This function creates a `PlaneSurface` from the given bounds and samples it
/// at the specified grid resolution. It's used when only bounding-box information
/// is available from the input geometry.
///
/// # Arguments
///
/// * `min` - Minimum corner of the bounding box [x, y, z]
/// * `max` - Maximum corner of the bounding box [x, y, z]
/// * `u_segments` - Number of segments in U direction (clamped to ≥1)
/// * `v_segments` - Number of segments in V direction (clamped to ≥1)
/// * `options` - Division options for handling closed surfaces
///
/// # Returns
///
/// A result containing the sampled points, normals, and UV parameters.
///
/// # Example
///
/// ```ignore
/// let result = divide_surface_from_bounds(
///     [0.0, 0.0, 0.0],
///     [10.0, 5.0, 0.0],
///     4,
///     2,
///     DivideSurfaceBoundsOptions::default(),
/// );
/// // Result contains 15 points (5 x 3 grid)
/// ```
#[must_use]
pub fn divide_surface_from_bounds(
    min: [f64; 3],
    max: [f64; 3],
    u_segments: usize,
    v_segments: usize,
    options: DivideSurfaceBoundsOptions,
) -> DivideSurfaceBoundsResult {
    let size = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
    let mid_z = (min[2] + max[2]) * 0.5;

    let plane = PlaneSurface::new(
        Point3::new(min[0], min[1], mid_z),
        Vec3::new(size[0], 0.0, 0.0),
        Vec3::new(0.0, size[1], 0.0),
    );

    let geom_options = DivideSurfaceOptions {
        closed_u: options.closed_u,
        closed_v: options.closed_v,
    };

    let result = geom_divide_surface(&plane, u_segments, v_segments, geom_options);

    DivideSurfaceBoundsResult {
        points: result.points.iter().map(|p| p.to_array()).collect(),
        normals: result.normals.iter().map(|n| [n.x, n.y, n.z]).collect(),
        parameters: result
            .parameters
            .iter()
            .map(|(u, v)| [*u, *v, 0.0])
            .collect(),
        u_count: result.u_count,
        v_count: result.v_count,
    }
}

/// Divides a generic surface into a grid of sample points.
///
/// This is a thin wrapper around `geom::surface::divide_surface` that provides
/// a consistent API for component integration.
///
/// # Arguments
///
/// * `surface` - The surface to sample
/// * `u_segments` - Number of segments in U direction
/// * `v_segments` - Number of segments in V direction
/// * `options` - Division options
///
/// # Returns
///
/// The raw `DivideSurfaceResult` from the geom kernel.
#[must_use]
pub fn divide_surface_generic<S: Surface>(
    surface: &S,
    u_segments: usize,
    v_segments: usize,
    options: DivideSurfaceOptions,
) -> DivideSurfaceResult {
    geom_divide_surface(surface, u_segments, v_segments, options)
}

/// Input specification for vertex-based surface operations.
///
/// Provides options for how to interpret vertex data when creating a
/// surface representation.
#[derive(Debug, Clone)]
pub struct VertexSurfaceInput<'a> {
    /// The vertex data as [x, y, z] arrays.
    pub vertices: &'a [[f64; 3]],
    /// Known grid dimensions (u_count, v_count), if available.
    /// If `None`, dimensions will be inferred from vertex count.
    pub grid_dimensions: Option<(usize, usize)>,
}

impl<'a> VertexSurfaceInput<'a> {
    /// Creates a new vertex surface input with automatic dimension inference.
    #[must_use]
    pub fn new(vertices: &'a [[f64; 3]]) -> Self {
        Self {
            vertices,
            grid_dimensions: None,
        }
    }

    /// Creates a new vertex surface input with explicit grid dimensions.
    #[must_use]
    pub fn with_dimensions(vertices: &'a [[f64; 3]], u_count: usize, v_count: usize) -> Self {
        Self {
            vertices,
            grid_dimensions: Some((u_count, v_count)),
        }
    }
}

/// Divides a surface defined by vertex data into a grid of sample points.
///
/// This function preserves the actual surface geometry, including rotation
/// and curvature, by using the vertex positions directly rather than
/// approximating with an axis-aligned bounding box.
///
/// # Method Selection
///
/// 1. **Grid surface**: If vertex count matches a recognizable grid pattern
///    (or explicit dimensions are provided), uses bilinear interpolation
///    over the vertex grid.
///
/// 2. **Best-fit plane**: If grid structure cannot be determined, computes
///    a best-fit plane through the vertices using PCA, preserving the
///    actual surface orientation.
///
/// # Arguments
///
/// * `input` - Vertex data and optional grid dimensions
/// * `u_segments` - Number of segments in U direction
/// * `v_segments` - Number of segments in V direction
/// * `options` - Division options for handling closed surfaces
///
/// # Returns
///
/// A result containing the sampled points, normals, and UV parameters,
/// or `None` if the vertex data is insufficient.
///
/// # Example
///
/// ```ignore
/// // A rotated square surface
/// let vertices = [
///     [0.0, 0.0, 0.0],
///     [1.0, 1.0, 0.0],
///     [-1.0, 1.0, 0.0],
///     [0.0, 2.0, 0.0],
/// ];
/// let input = VertexSurfaceInput::with_dimensions(&vertices, 2, 2);
/// let result = divide_surface_from_vertices(input, 4, 4, DivideSurfaceBoundsOptions::default());
/// // Result preserves the rotated orientation
/// ```
#[must_use]
pub fn divide_surface_from_vertices(
    input: VertexSurfaceInput<'_>,
    u_segments: usize,
    v_segments: usize,
    options: DivideSurfaceBoundsOptions,
) -> Option<DivideSurfaceBoundsResult> {
    if input.vertices.len() < 4 {
        return None;
    }

    let geom_options = DivideSurfaceOptions {
        closed_u: options.closed_u,
        closed_v: options.closed_v,
    };

    // Try to create a vertex grid surface
    let dimensions = input
        .grid_dimensions
        .or_else(|| VertexGridSurface::infer_grid_dimensions(input.vertices.len()));

    if let Some((u_count, v_count)) = dimensions {
        if let Some(surface) = VertexGridSurface::from_arrays(input.vertices, u_count, v_count) {
            let result = geom_divide_surface(&surface, u_segments, v_segments, geom_options);
            return Some(DivideSurfaceBoundsResult {
                points: result.points.iter().map(|p| p.to_array()).collect(),
                normals: result.normals.iter().map(|n| [n.x, n.y, n.z]).collect(),
                parameters: result
                    .parameters
                    .iter()
                    .map(|(u, v)| [*u, *v, 0.0])
                    .collect(),
                u_count: result.u_count,
                v_count: result.v_count,
            });
        }
    }

    // Fall back to best-fit plane
    let points: Vec<Point3> = input
        .vertices
        .iter()
        .map(|v| Point3::new(v[0], v[1], v[2]))
        .collect();

    let plane = best_fit_plane(&points)?;
    let result = geom_divide_surface(&plane, u_segments, v_segments, geom_options);

    Some(DivideSurfaceBoundsResult {
        points: result.points.iter().map(|p| p.to_array()).collect(),
        normals: result.normals.iter().map(|n| [n.x, n.y, n.z]).collect(),
        parameters: result
            .parameters
            .iter()
            .map(|(u, v)| [*u, *v, 0.0])
            .collect(),
        u_count: result.u_count,
        v_count: result.v_count,
    })
}

// ============================================================================
// SurfaceFrames Operations
// ============================================================================

/// Result of computing surface frames using bounds-based input.
#[derive(Debug, Clone)]
pub struct SurfaceFramesBoundsResult {
    /// Frames organized as rows (outer = v, inner = u within each row).
    /// Each frame is [origin, x_axis, y_axis, z_axis] as [f64; 3] arrays.
    pub frames: Vec<Vec<SurfaceFrameArrays>>,
    /// Parameters organized as rows (outer = v, inner = u within each row).
    /// Each parameter is [u, v, 0.0].
    pub parameters: Vec<Vec<[f64; 3]>>,
    /// Number of samples in U direction.
    pub u_count: usize,
    /// Number of samples in V direction.
    pub v_count: usize,
}

/// A surface frame represented as arrays for component output.
#[derive(Debug, Clone, Copy)]
pub struct SurfaceFrameArrays {
    pub origin: [f64; 3],
    pub x_axis: [f64; 3],
    pub y_axis: [f64; 3],
    pub z_axis: [f64; 3],
}

impl From<SurfaceFrame> for SurfaceFrameArrays {
    fn from(frame: SurfaceFrame) -> Self {
        Self {
            origin: frame.origin.to_array(),
            x_axis: [frame.x_axis.x, frame.x_axis.y, frame.x_axis.z],
            y_axis: [frame.y_axis.x, frame.y_axis.y, frame.y_axis.z],
            z_axis: [frame.z_axis.x, frame.z_axis.y, frame.z_axis.z],
        }
    }
}

/// Computes surface frames on a surface defined by bounding box min/max.
///
/// This function creates a `PlaneSurface` from the given bounds and computes
/// coordinate frames at grid positions. The result is organized in rows for
/// easy conversion to Grasshopper data tree structure.
///
/// # Arguments
///
/// * `min` - Minimum corner of the bounding box [x, y, z]
/// * `max` - Maximum corner of the bounding box [x, y, z]
/// * `u_segments` - Number of segments in U direction (clamped to ≥1)
/// * `v_segments` - Number of segments in V direction (clamped to ≥1)
///
/// # Returns
///
/// A result containing frames and parameters organized in row-major order.
///
/// # Example
///
/// ```ignore
/// let result = surface_frames_from_bounds(
///     [0.0, 0.0, 0.0],
///     [10.0, 5.0, 0.0],
///     4,
///     2,
/// );
/// // Result contains 3 rows (v=0,1,2), each with 5 frames (u=0..4)
/// ```
#[must_use]
pub fn surface_frames_from_bounds(
    min: [f64; 3],
    max: [f64; 3],
    u_segments: usize,
    v_segments: usize,
) -> SurfaceFramesBoundsResult {
    let size = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
    let mid_z = (min[2] + max[2]) * 0.5;

    let plane = PlaneSurface::new(
        Point3::new(min[0], min[1], mid_z),
        Vec3::new(size[0], 0.0, 0.0),
        Vec3::new(0.0, size[1], 0.0),
    );

    let tol = Tolerance::default_geom();
    let result = geom_surface_frames(&plane, u_segments, v_segments, tol);

    // Organize into rows for component output
    let mut frames_rows = Vec::with_capacity(result.v_count);
    let mut parameter_rows = Vec::with_capacity(result.v_count);

    for v in 0..result.v_count {
        let row_offset = v * result.u_count;
        let mut frames_row = Vec::with_capacity(result.u_count);
        let mut parameters_row = Vec::with_capacity(result.u_count);

        for idx in row_offset..(row_offset + result.u_count) {
            frames_row.push(SurfaceFrameArrays::from(result.frames[idx]));
            let (u, v) = result.parameters[idx];
            parameters_row.push([u, v, 0.0]);
        }

        frames_rows.push(frames_row);
        parameter_rows.push(parameters_row);
    }

    SurfaceFramesBoundsResult {
        frames: frames_rows,
        parameters: parameter_rows,
        u_count: result.u_count,
        v_count: result.v_count,
    }
}

/// Computes surface frames on a generic surface.
///
/// This is a thin wrapper around `geom::analysis::surface_frames` that provides
/// a consistent API for component integration.
///
/// # Arguments
///
/// * `surface` - The surface to sample
/// * `u_segments` - Number of segments in U direction
/// * `v_segments` - Number of segments in V direction
/// * `tol` - Tolerance for frame axis computation
///
/// # Returns
///
/// The raw `SurfaceFramesResult` from the geom kernel.
#[must_use]
pub fn surface_frames_generic<S: Surface>(
    surface: &S,
    u_segments: usize,
    v_segments: usize,
    tol: Tolerance,
) -> SurfaceFramesResult {
    geom_surface_frames(surface, u_segments, v_segments, tol)
}

/// Computes surface frames from vertex data.
///
/// This function preserves the actual surface geometry by using the vertex
/// positions directly. For rotated surfaces, the frames will have correct
/// orientations matching the actual surface normals and tangent directions.
///
/// # Method Selection
///
/// 1. **Grid surface**: If vertex count matches a recognizable grid pattern
///    (or explicit dimensions are provided), uses bilinear interpolation
///    to sample frames on the actual surface geometry.
///
/// 2. **Best-fit plane**: If grid structure cannot be determined, computes
///    a best-fit plane through the vertices, preserving orientation.
///
/// # Arguments
///
/// * `input` - Vertex data and optional grid dimensions
/// * `u_segments` - Number of segments in U direction
/// * `v_segments` - Number of segments in V direction
///
/// # Returns
///
/// A result containing frames and parameters, or `None` if vertex data
/// is insufficient.
///
/// # Example
///
/// ```ignore
/// // A tilted plane
/// let vertices = [
///     [0.0, 0.0, 0.0], [2.0, 0.0, 1.0],
///     [0.0, 2.0, 0.0], [2.0, 2.0, 1.0],
/// ];
/// let input = VertexSurfaceInput::with_dimensions(&vertices, 2, 2);
/// let result = surface_frames_from_vertices(input, 3, 3);
/// // Frames have normals matching the tilted surface
/// ```
#[must_use]
pub fn surface_frames_from_vertices(
    input: VertexSurfaceInput<'_>,
    u_segments: usize,
    v_segments: usize,
) -> Option<SurfaceFramesBoundsResult> {
    if input.vertices.len() < 4 {
        return None;
    }

    let tol = Tolerance::default_geom();

    // Try to create a vertex grid surface
    let dimensions = input
        .grid_dimensions
        .or_else(|| VertexGridSurface::infer_grid_dimensions(input.vertices.len()));

    let result = if let Some((u_count, v_count)) = dimensions {
        if let Some(surface) = VertexGridSurface::from_arrays(input.vertices, u_count, v_count) {
            geom_surface_frames(&surface, u_segments, v_segments, tol)
        } else {
            // Fall back to best-fit plane
            let points: Vec<Point3> = input
                .vertices
                .iter()
                .map(|v| Point3::new(v[0], v[1], v[2]))
                .collect();
            let plane = best_fit_plane(&points)?;
            geom_surface_frames(&plane, u_segments, v_segments, tol)
        }
    } else {
        // Fall back to best-fit plane
        let points: Vec<Point3> = input
            .vertices
            .iter()
            .map(|v| Point3::new(v[0], v[1], v[2]))
            .collect();
        let plane = best_fit_plane(&points)?;
        geom_surface_frames(&plane, u_segments, v_segments, tol)
    };

    // Organize into rows for component output
    let mut frames_rows = Vec::with_capacity(result.v_count);
    let mut parameter_rows = Vec::with_capacity(result.v_count);

    for v in 0..result.v_count {
        let row_offset = v * result.u_count;
        let mut frames_row = Vec::with_capacity(result.u_count);
        let mut parameters_row = Vec::with_capacity(result.u_count);

        for idx in row_offset..(row_offset + result.u_count) {
            frames_row.push(SurfaceFrameArrays::from(result.frames[idx]));
            let (u, v) = result.parameters[idx];
            parameters_row.push([u, v, 0.0]);
        }

        frames_rows.push(frames_row);
        parameter_rows.push(parameters_row);
    }

    Some(SurfaceFramesBoundsResult {
        frames: frames_rows,
        parameters: parameter_rows,
        u_count: result.u_count,
        v_count: result.v_count,
    })
}

// ============================================================================
// Isotrim Operations
// ============================================================================

/// Diagnostics from an isotrim operation using bounds-based input.
#[derive(Debug, Clone, Copy, Default)]
pub struct IsotrimBoundsDiagnostics {
    /// Whether U range was clamped to domain bounds.
    pub clamped_u: bool,
    /// Whether V range was clamped to domain bounds.
    pub clamped_v: bool,
    /// Whether U direction was reversed (u0 > u1).
    pub reverse_u: bool,
    /// Whether V direction was reversed (v0 > v1).
    pub reverse_v: bool,
}

impl From<IsotrimDiagnostics> for IsotrimBoundsDiagnostics {
    fn from(d: IsotrimDiagnostics) -> Self {
        Self {
            clamped_u: d.clamped_u,
            clamped_v: d.clamped_v,
            reverse_u: d.reverse_u,
            reverse_v: d.reverse_v,
        }
    }
}

/// Result of an isotrim operation using bounds-based input.
#[derive(Debug, Clone)]
pub struct IsotrimBoundsResult {
    /// The corner vertices of the trimmed surface [v0, v1, v2, v3].
    pub vertices: Vec<[f64; 3]>,
    /// Triangle faces for the trimmed surface.
    pub faces: Vec<Vec<u32>>,
    /// Diagnostics from the isotrim operation.
    pub diagnostics: IsotrimBoundsDiagnostics,
}

/// Extracts a subsurface from a surface defined by bounding box min/max.
///
/// This function creates a `PlaneSurface` from the given bounds and extracts
/// a rectangular region based on the UV parameter ranges. The result is
/// returned as vertices and faces suitable for `Value::Surface`.
///
/// # Arguments
///
/// * `min` - Minimum corner of the bounding box [x, y, z]
/// * `max` - Maximum corner of the bounding box [x, y, z]
/// * `u_range` - (u0, u1) parameter range in 0..1
/// * `v_range` - (v0, v1) parameter range in 0..1
///
/// # Returns
///
/// A result containing the trimmed surface vertices, faces, and diagnostics.
///
/// # Example
///
/// ```ignore
/// let result = isotrim_from_bounds(
///     [0.0, 0.0, 0.0],
///     [10.0, 5.0, 0.0],
///     (0.25, 0.75),
///     (0.0, 0.5),
/// );
/// // Result contains a surface from x=2.5..7.5, y=0..2.5
/// ```
#[must_use]
pub fn isotrim_from_bounds(
    min: [f64; 3],
    max: [f64; 3],
    u_range: (f64, f64),
    v_range: (f64, f64),
) -> IsotrimBoundsResult {
    let size = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
    let mid_z = (min[2] + max[2]) * 0.5;

    let plane = PlaneSurface::new(
        Point3::new(min[0], min[1], mid_z),
        Vec3::new(size[0], 0.0, 0.0),
        Vec3::new(0.0, size[1], 0.0),
    );

    let tol = Tolerance::default_geom();
    let (trimmed, diag) = geom_isotrim_surface(&plane, u_range, v_range, tol);

    // Sample corners of the trimmed surface
    let (u0, u1) = trimmed.domain_u();
    let (v0, v1) = trimmed.domain_v();

    let vertices = vec![
        trimmed.point_at(u0, v0).to_array(),
        trimmed.point_at(u1, v0).to_array(),
        trimmed.point_at(u1, v1).to_array(),
        trimmed.point_at(u0, v1).to_array(),
    ];

    let faces = vec![vec![0, 1, 2], vec![0, 2, 3]];

    IsotrimBoundsResult {
        vertices,
        faces,
        diagnostics: IsotrimBoundsDiagnostics::from(diag),
    }
}

/// Extracts a subsurface from a generic surface.
///
/// This is a thin wrapper around `geom::surface::isotrim_surface` that provides
/// a consistent API for component integration.
///
/// # Arguments
///
/// * `surface` - The surface to trim
/// * `u_range` - (u0, u1) parameter range
/// * `v_range` - (v0, v1) parameter range
/// * `tol` - Tolerance for domain handling
///
/// # Returns
///
/// A tuple containing the trimmed surface wrapper and diagnostics.
pub fn isotrim_generic<'a, S: Surface + ?Sized>(
    surface: &'a S,
    u_range: (f64, f64),
    v_range: (f64, f64),
    tol: Tolerance,
) -> (IsotrimSurface<'a, S>, IsotrimDiagnostics) {
    geom_isotrim_surface(surface, u_range, v_range, tol)
}

/// Extracts a subsurface from vertex data.
///
/// This function preserves the actual surface geometry by using the vertex
/// positions directly. For rotated surfaces, the extracted subsurface will
/// have correct corner positions matching the actual geometry rather than
/// axis-aligned approximations.
///
/// # Method Selection
///
/// 1. **Grid surface**: If vertex count matches a recognizable grid pattern
///    (or explicit dimensions are provided), uses bilinear interpolation
///    to extract corners at the exact parametric positions.
///
/// 2. **Best-fit plane**: If grid structure cannot be determined, computes
///    a best-fit plane and extracts the subsurface from that.
///
/// # Arguments
///
/// * `input` - Vertex data and optional grid dimensions
/// * `u_range` - (u0, u1) parameter range in 0..1
/// * `v_range` - (v0, v1) parameter range in 0..1
///
/// # Returns
///
/// A result containing the trimmed surface vertices, faces, and diagnostics,
/// or `None` if vertex data is insufficient.
///
/// # Example
///
/// ```ignore
/// // A rotated surface
/// let vertices = [
///     [0.0, 0.0, 0.0], [2.0, 2.0, 0.0],
///     [0.0, 2.0, 0.0], [2.0, 4.0, 0.0],
/// ];
/// let input = VertexSurfaceInput::with_dimensions(&vertices, 2, 2);
/// let result = isotrim_from_vertices(input, (0.25, 0.75), (0.0, 0.5));
/// // Corners are on the actual rotated surface
/// ```
#[must_use]
pub fn isotrim_from_vertices(
    input: VertexSurfaceInput<'_>,
    u_range: (f64, f64),
    v_range: (f64, f64),
) -> Option<IsotrimBoundsResult> {
    if input.vertices.len() < 4 {
        return None;
    }

    let tol = Tolerance::default_geom();

    // Try to create a vertex grid surface
    let dimensions = input
        .grid_dimensions
        .or_else(|| VertexGridSurface::infer_grid_dimensions(input.vertices.len()));

    if let Some((u_count, v_count)) = dimensions {
        if let Some(surface) = VertexGridSurface::from_arrays(input.vertices, u_count, v_count) {
            let (trimmed, diag) = geom_isotrim_surface(&surface, u_range, v_range, tol);

            // Sample corners of the trimmed surface
            let (u0, u1) = trimmed.domain_u();
            let (v0, v1) = trimmed.domain_v();

            let vertices = vec![
                trimmed.point_at(u0, v0).to_array(),
                trimmed.point_at(u1, v0).to_array(),
                trimmed.point_at(u1, v1).to_array(),
                trimmed.point_at(u0, v1).to_array(),
            ];

            let faces = vec![vec![0, 1, 2], vec![0, 2, 3]];

            return Some(IsotrimBoundsResult {
                vertices,
                faces,
                diagnostics: IsotrimBoundsDiagnostics::from(diag),
            });
        }
    }

    // Fall back to best-fit plane
    let points: Vec<Point3> = input
        .vertices
        .iter()
        .map(|v| Point3::new(v[0], v[1], v[2]))
        .collect();

    let plane = best_fit_plane(&points)?;
    let (trimmed, diag) = geom_isotrim_surface(&plane, u_range, v_range, tol);

    // Sample corners of the trimmed surface
    let (u0, u1) = trimmed.domain_u();
    let (v0, v1) = trimmed.domain_v();

    let vertices = vec![
        trimmed.point_at(u0, v0).to_array(),
        trimmed.point_at(u1, v0).to_array(),
        trimmed.point_at(u1, v1).to_array(),
        trimmed.point_at(u0, v1).to_array(),
    ];

    let faces = vec![vec![0, 1, 2], vec![0, 2, 3]];

    Some(IsotrimBoundsResult {
        vertices,
        faces,
        diagnostics: IsotrimBoundsDiagnostics::from(diag),
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divide_surface_from_bounds_basic() {
        let result = divide_surface_from_bounds(
            [0.0, 0.0, 0.0],
            [10.0, 5.0, 0.0],
            2,
            2,
            DivideSurfaceBoundsOptions::default(),
        );

        // 3x3 grid = 9 points
        assert_eq!(result.points.len(), 9);
        assert_eq!(result.normals.len(), 9);
        assert_eq!(result.parameters.len(), 9);
        assert_eq!(result.u_count, 3);
        assert_eq!(result.v_count, 3);

        // Check corner points
        assert!((result.points[0][0] - 0.0).abs() < 1e-9);
        assert!((result.points[0][1] - 0.0).abs() < 1e-9);

        // Check last point (u=1, v=1)
        let last = result.u_count * result.v_count - 1;
        assert!((result.points[last][0] - 10.0).abs() < 1e-9);
        assert!((result.points[last][1] - 5.0).abs() < 1e-9);

        // Check normals (should be Z-up for XY plane)
        for normal in &result.normals {
            assert!((normal[2] - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn surface_frames_from_bounds_basic() {
        let result = surface_frames_from_bounds([0.0, 0.0, 0.0], [10.0, 5.0, 0.0], 2, 2);

        // 3 rows (v), each with 3 frames (u)
        assert_eq!(result.frames.len(), 3);
        assert_eq!(result.parameters.len(), 3);
        assert_eq!(result.u_count, 3);
        assert_eq!(result.v_count, 3);

        // Each row has u_count frames
        for row in &result.frames {
            assert_eq!(row.len(), 3);
        }

        // Check first frame origin
        let first = &result.frames[0][0];
        assert!((first.origin[0] - 0.0).abs() < 1e-9);
        assert!((first.origin[1] - 0.0).abs() < 1e-9);

        // Check last frame origin
        let last = &result.frames[2][2];
        assert!((last.origin[0] - 10.0).abs() < 1e-9);
        assert!((last.origin[1] - 5.0).abs() < 1e-9);
    }

    #[test]
    fn isotrim_from_bounds_basic() {
        let result = isotrim_from_bounds(
            [0.0, 0.0, 0.0],
            [10.0, 5.0, 0.0],
            (0.25, 0.75),
            (0.0, 0.5),
        );

        // Should have 4 corners
        assert_eq!(result.vertices.len(), 4);
        assert_eq!(result.faces.len(), 2);

        // Check bounds of trimmed surface
        // U: 0.25 * 10 = 2.5 to 0.75 * 10 = 7.5
        // V: 0.0 * 5 = 0.0 to 0.5 * 5 = 2.5
        let min_x = result.vertices.iter().map(|v| v[0]).fold(f64::INFINITY, f64::min);
        let max_x = result.vertices.iter().map(|v| v[0]).fold(f64::NEG_INFINITY, f64::max);
        let min_y = result.vertices.iter().map(|v| v[1]).fold(f64::INFINITY, f64::min);
        let max_y = result.vertices.iter().map(|v| v[1]).fold(f64::NEG_INFINITY, f64::max);

        assert!((min_x - 2.5).abs() < 1e-9);
        assert!((max_x - 7.5).abs() < 1e-9);
        assert!((min_y - 0.0).abs() < 1e-9);
        assert!((max_y - 2.5).abs() < 1e-9);
    }

    #[test]
    fn isotrim_from_bounds_reversed_range() {
        let result = isotrim_from_bounds(
            [0.0, 0.0, 0.0],
            [10.0, 5.0, 0.0],
            (0.75, 0.25), // Reversed U
            (0.0, 1.0),
        );

        // Should still produce valid geometry, but flag reversed
        assert_eq!(result.vertices.len(), 4);
        assert!(result.diagnostics.reverse_u);
        assert!(!result.diagnostics.reverse_v);
    }

    // ========================================================================
    // VertexGridSurface Tests
    // ========================================================================

    #[test]
    fn vertex_grid_surface_basic_construction() {
        // 2x2 grid of vertices
        let vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
        ];

        let surface = VertexGridSurface::new(vertices, 2, 2);
        assert!(surface.is_some());

        let surface = surface.unwrap();
        assert_eq!(surface.dimensions(), (2, 2));

        // Check corner points
        let p00 = surface.point_at(0.0, 0.0);
        assert!((p00.x - 0.0).abs() < 1e-9);
        assert!((p00.y - 0.0).abs() < 1e-9);

        let p11 = surface.point_at(1.0, 1.0);
        assert!((p11.x - 1.0).abs() < 1e-9);
        assert!((p11.y - 1.0).abs() < 1e-9);
    }

    #[test]
    fn vertex_grid_surface_bilinear_interpolation() {
        // 2x2 grid forming a unit square
        let vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
            Point3::new(0.0, 2.0, 0.0),
            Point3::new(2.0, 2.0, 0.0),
        ];

        let surface = VertexGridSurface::new(vertices, 2, 2).unwrap();

        // Center point should be at (1, 1, 0)
        let center = surface.point_at(0.5, 0.5);
        assert!((center.x - 1.0).abs() < 1e-9);
        assert!((center.y - 1.0).abs() < 1e-9);
        assert!((center.z - 0.0).abs() < 1e-9);

        // Quarter point
        let quarter = surface.point_at(0.25, 0.25);
        assert!((quarter.x - 0.5).abs() < 1e-9);
        assert!((quarter.y - 0.5).abs() < 1e-9);
    }

    #[test]
    fn vertex_grid_surface_rotated_plane() {
        // A 45-degree rotated square in XY plane
        // Vertices form a diamond shape
        let vertices = vec![
            Point3::new(1.0, 0.0, 0.0),  // (0, 0) - right
            Point3::new(2.0, 1.0, 0.0),  // (1, 0) - top-right
            Point3::new(0.0, 1.0, 0.0),  // (0, 1) - top-left
            Point3::new(1.0, 2.0, 0.0),  // (1, 1) - top
        ];

        let surface = VertexGridSurface::new(vertices, 2, 2).unwrap();

        // Center should be at centroid
        let center = surface.point_at(0.5, 0.5);
        assert!((center.x - 1.0).abs() < 1e-9);
        assert!((center.y - 1.0).abs() < 1e-9);

        // Check normal is still Z-up (since all Z values are 0)
        let normal = surface.normal_at(0.5, 0.5).unwrap();
        assert!(normal.z.abs() > 0.9); // Z component should dominate
    }

    #[test]
    fn vertex_grid_surface_tilted_plane() {
        // A plane tilted so that Z increases with Y
        let vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 1.0),
            Point3::new(1.0, 1.0, 1.0),
        ];

        let surface = VertexGridSurface::new(vertices, 2, 2).unwrap();

        // Normal should point in negative-Y, positive-Z direction
        let normal = surface.normal_at(0.5, 0.5).unwrap();
        // The plane has normal perpendicular to both (1,0,0) and (0,1,1)
        // which is (0, -1, 1) normalized
        let expected_y = -1.0 / 2.0_f64.sqrt();
        let expected_z = 1.0 / 2.0_f64.sqrt();
        assert!((normal.y - expected_y).abs() < 1e-6);
        assert!((normal.z - expected_z).abs() < 1e-6);
    }

    #[test]
    fn vertex_grid_surface_3x3_grid() {
        // 3x3 grid with slight curvature
        let vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(1.0, 1.0, 0.5), // Center vertex raised
            Point3::new(2.0, 1.0, 0.0),
            Point3::new(0.0, 2.0, 0.0),
            Point3::new(1.0, 2.0, 0.0),
            Point3::new(2.0, 2.0, 0.0),
        ];

        let surface = VertexGridSurface::new(vertices, 3, 3).unwrap();
        assert_eq!(surface.dimensions(), (3, 3));

        // Center should be at (1, 1, 0.5)
        let center = surface.point_at(0.5, 0.5);
        assert!((center.x - 1.0).abs() < 1e-9);
        assert!((center.y - 1.0).abs() < 1e-9);
        assert!((center.z - 0.5).abs() < 1e-9);
    }

    #[test]
    fn infer_grid_dimensions_square() {
        // Perfect square (4, 9, 16, 25, ...)
        assert_eq!(VertexGridSurface::infer_grid_dimensions(4), Some((2, 2)));
        assert_eq!(VertexGridSurface::infer_grid_dimensions(9), Some((3, 3)));
        assert_eq!(VertexGridSurface::infer_grid_dimensions(16), Some((4, 4)));
    }

    #[test]
    fn infer_grid_dimensions_rectangular() {
        // Rectangular grids
        assert_eq!(VertexGridSurface::infer_grid_dimensions(6), Some((3, 2)));
        assert_eq!(VertexGridSurface::infer_grid_dimensions(12), Some((4, 3)));
        assert_eq!(VertexGridSurface::infer_grid_dimensions(15), Some((5, 3)));
    }

    #[test]
    fn infer_grid_dimensions_invalid() {
        // Too few vertices
        assert_eq!(VertexGridSurface::infer_grid_dimensions(3), None);
        // Prime number (can't form grid)
        assert_eq!(VertexGridSurface::infer_grid_dimensions(7), None);
    }

    // ========================================================================
    // divide_surface_from_vertices Tests
    // ========================================================================

    #[test]
    fn divide_surface_from_vertices_basic() {
        // Simple 2x2 grid
        let vertices = [
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.0, 2.0, 0.0],
            [2.0, 2.0, 0.0],
        ];
        let input = VertexSurfaceInput::with_dimensions(&vertices, 2, 2);

        let result = divide_surface_from_vertices(
            input,
            2,
            2,
            DivideSurfaceBoundsOptions::default(),
        );
        assert!(result.is_some());

        let result = result.unwrap();
        assert_eq!(result.u_count, 3);
        assert_eq!(result.v_count, 3);
        assert_eq!(result.points.len(), 9);
    }

    #[test]
    fn divide_surface_from_vertices_rotated_plane() {
        // A 45-degree rotated plane in XY
        // This tests that the sampling follows the actual geometry
        let vertices = [
            [0.0, 0.0, 0.0],   // origin
            [1.0, 1.0, 0.0],   // u=1, v=0
            [-1.0, 1.0, 0.0],  // u=0, v=1
            [0.0, 2.0, 0.0],   // u=1, v=1
        ];
        let input = VertexSurfaceInput::with_dimensions(&vertices, 2, 2);

        let result = divide_surface_from_vertices(
            input,
            2,
            2,
            DivideSurfaceBoundsOptions::default(),
        );
        assert!(result.is_some());

        let result = result.unwrap();

        // With 2 segments, we get a 3x3 grid (9 points)
        assert_eq!(result.u_count, 3);
        assert_eq!(result.v_count, 3);
        assert_eq!(result.points.len(), 9);

        // Check that sampled points are on the rotated plane, not axis-aligned
        // The center point should be at (0, 1, 0)
        let center_idx = 4; // 3x3 grid, center is at index 4
        let center = &result.points[center_idx];
        assert!((center[0] - 0.0).abs() < 1e-9, "Center X should be 0, got {}", center[0]);
        assert!((center[1] - 1.0).abs() < 1e-9, "Center Y should be 1, got {}", center[1]);
    }

    #[test]
    fn divide_surface_from_vertices_preserves_normals() {
        // Tilted plane with Z increasing along Y
        let vertices = [
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.0, 2.0, 2.0],
            [2.0, 2.0, 2.0],
        ];
        let input = VertexSurfaceInput::with_dimensions(&vertices, 2, 2);

        let result = divide_surface_from_vertices(
            input,
            1,
            1,
            DivideSurfaceBoundsOptions::default(),
        );
        assert!(result.is_some());

        let result = result.unwrap();

        // Normals should reflect the tilted plane, not be Z-up
        for normal in &result.normals {
            // The plane normal is perpendicular to (2,0,0) and (0,2,2)
            // Cross product: (0, -4, 4) -> normalized: (0, -1/sqrt(2), 1/sqrt(2))
            let expected_y = -1.0 / 2.0_f64.sqrt();
            let expected_z = 1.0 / 2.0_f64.sqrt();

            assert!(
                (normal[1] - expected_y).abs() < 0.1,
                "Normal Y should be ~{:.3}, got {:.3}",
                expected_y,
                normal[1]
            );
            assert!(
                (normal[2] - expected_z).abs() < 0.1,
                "Normal Z should be ~{:.3}, got {:.3}",
                expected_z,
                normal[2]
            );
        }
    }

    // ========================================================================
    // surface_frames_from_vertices Tests
    // ========================================================================

    #[test]
    fn surface_frames_from_vertices_basic() {
        let vertices = [
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.0, 2.0, 0.0],
            [2.0, 2.0, 0.0],
        ];
        let input = VertexSurfaceInput::with_dimensions(&vertices, 2, 2);

        let result = surface_frames_from_vertices(input, 2, 2);
        assert!(result.is_some());

        let result = result.unwrap();
        assert_eq!(result.u_count, 3);
        assert_eq!(result.v_count, 3);
        assert_eq!(result.frames.len(), 3);
    }

    #[test]
    fn surface_frames_from_vertices_tilted_plane_normals() {
        // Tilted plane
        let vertices = [
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.0, 2.0, 2.0],
            [2.0, 2.0, 2.0],
        ];
        let input = VertexSurfaceInput::with_dimensions(&vertices, 2, 2);

        let result = surface_frames_from_vertices(input, 1, 1);
        assert!(result.is_some());

        let result = result.unwrap();

        // Check that z_axis (normal) is correctly tilted, not just [0,0,1]
        for row in &result.frames {
            for frame in row {
                // Z-axis should not be purely vertical
                assert!(
                    frame.z_axis[2].abs() < 0.9,
                    "Z-axis should be tilted, got {:?}",
                    frame.z_axis
                );
            }
        }
    }

    // ========================================================================
    // isotrim_from_vertices Tests
    // ========================================================================

    #[test]
    fn isotrim_from_vertices_basic() {
        let vertices = [
            [0.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [0.0, 4.0, 0.0],
            [4.0, 4.0, 0.0],
        ];
        let input = VertexSurfaceInput::with_dimensions(&vertices, 2, 2);

        let result = isotrim_from_vertices(input, (0.25, 0.75), (0.25, 0.75));
        assert!(result.is_some());

        let result = result.unwrap();
        assert_eq!(result.vertices.len(), 4);

        // Check the extracted region is centered
        let center_x: f64 = result.vertices.iter().map(|v| v[0]).sum::<f64>() / 4.0;
        let center_y: f64 = result.vertices.iter().map(|v| v[1]).sum::<f64>() / 4.0;

        assert!((center_x - 2.0).abs() < 1e-9);
        assert!((center_y - 2.0).abs() < 1e-9);
    }

    #[test]
    fn isotrim_from_vertices_rotated_surface() {
        // A 45-degree rotated square
        let vertices = [
            [1.0, 0.0, 0.0],   // (0, 0)
            [2.0, 1.0, 0.0],   // (1, 0)
            [0.0, 1.0, 0.0],   // (0, 1)
            [1.0, 2.0, 0.0],   // (1, 1)
        ];
        let input = VertexSurfaceInput::with_dimensions(&vertices, 2, 2);

        let result = isotrim_from_vertices(input, (0.0, 0.5), (0.0, 0.5));
        assert!(result.is_some());

        let result = result.unwrap();

        // The isotrim should extract the bottom-left quadrant
        // Corner at (0,0) should be at [1, 0, 0]
        let has_origin_corner = result.vertices.iter().any(|v| {
            (v[0] - 1.0).abs() < 1e-9 && v[1].abs() < 1e-9
        });
        assert!(
            has_origin_corner,
            "Should have corner at original (0,0) position"
        );
    }

    // ========================================================================
    // best_fit_plane Tests
    // ========================================================================

    #[test]
    fn best_fit_plane_xy_aligned() {
        let points = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
        ];

        let plane = best_fit_plane(&points);
        assert!(plane.is_some());

        let plane = plane.unwrap();
        // Normal should be Z-up
        let normal = plane.u_axis.cross(plane.v_axis);
        assert!(normal.z.abs() > 0.9);
    }

    #[test]
    fn best_fit_plane_tilted() {
        // Points on a plane tilted 45 degrees about X axis
        let points = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 1.0),
            Point3::new(1.0, 1.0, 1.0),
        ];

        let plane = best_fit_plane(&points);
        assert!(plane.is_some());

        let plane = plane.unwrap();
        let normal = plane.u_axis.cross(plane.v_axis);

        // Normal should have significant Y and Z components
        assert!(normal.y.abs() > 0.3 || normal.z.abs() > 0.3);
    }

    #[test]
    fn best_fit_plane_insufficient_points() {
        let points = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
        ];

        let plane = best_fit_plane(&points);
        assert!(plane.is_none());
    }
}
