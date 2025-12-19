//! Deformation fields for mesh transformations.
//!
//! This module provides deformation operations that modify mesh geometry
//! by applying spatial transformations to vertices:
//!
//! - **Twist**: Rotate vertices around an axis based on their position along it.
//! - **Bend**: Arc the mesh around a curve or axis (circular deformation).
//! - **Taper**: Scale vertices based on their position along an axis.
//! - **Morph**: Blend between source and target vertex positions.
//!
//! All deformations use deterministic frames and include post weld/normal repair
//! to maintain mesh integrity after transformation.
//!
//! # Example
//!
//! ```ignore
//! use ghx_engine::geom::{twist_mesh, TwistOptions, Point3, Vec3};
//!
//! let (mesh, diag) = some_mesh_source();
//! let options = TwistOptions::new(
//!     Point3::new(0.0, 0.0, 0.0), // axis origin
//!     Vec3::new(0.0, 0.0, 1.0),   // axis direction
//!     std::f64::consts::PI,       // twist angle over full extent
//! );
//! let (twisted, twist_diag) = twist_mesh(&mesh, options, Tolerance::default_geom())?;
//! ```

use super::mesh::{compute_smooth_normals_for_mesh, finalize_mesh, GeomMesh};
use super::{Point3, Tolerance, Vec3};
use std::f64::consts::PI;

// ============================================================================
// Error types
// ============================================================================

/// Errors that can occur during deformation operations.
#[derive(Debug)]
pub enum DeformationError {
    /// The input mesh has no triangles.
    EmptyMesh,

    /// The input mesh contains invalid geometry (NaN/Inf values).
    InvalidGeometry,

    /// Deformation axis is invalid (zero length or non-finite).
    InvalidAxis,

    /// Deformation parameters contain NaN or Inf.
    InvalidParameters,

    /// Morph target has different vertex count than source.
    MorphVertexCountMismatch {
        source_count: usize,
        target_count: usize,
    },

    /// Bend angle is out of valid range.
    InvalidBendAngle,

    /// Taper factors are invalid (negative or non-finite).
    InvalidTaperFactor,
}

impl std::fmt::Display for DeformationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyMesh => write!(f, "input mesh has no triangles"),
            Self::InvalidGeometry => {
                write!(f, "input mesh contains invalid geometry (NaN/Inf values)")
            }
            Self::InvalidAxis => write!(f, "deformation axis must be finite and non-zero"),
            Self::InvalidParameters => {
                write!(f, "deformation parameters contain NaN or Inf")
            }
            Self::MorphVertexCountMismatch {
                source_count,
                target_count,
            } => write!(
                f,
                "morph target vertex count ({target_count}) does not match source ({source_count})"
            ),
            Self::InvalidBendAngle => {
                write!(f, "bend angle must be between -2π and 2π radians")
            }
            Self::InvalidTaperFactor => {
                write!(f, "taper factor must be non-negative and finite")
            }
        }
    }
}

impl std::error::Error for DeformationError {}

// ============================================================================
// Diagnostics
// ============================================================================

/// Diagnostics specific to deformation operations.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DeformationDiagnostics {
    /// Number of vertices in the original mesh.
    pub original_vertex_count: usize,
    /// Number of triangles in the original mesh.
    pub original_triangle_count: usize,
    /// Number of vertices in the result.
    pub result_vertex_count: usize,
    /// Number of triangles in the result.
    pub result_triangle_count: usize,
    /// Minimum displacement distance applied.
    pub min_displacement: f64,
    /// Maximum displacement distance applied.
    pub max_displacement: f64,
    /// Average displacement distance applied.
    pub avg_displacement: f64,
    /// Number of vertices welded during post-processing.
    pub welded_vertex_count: usize,
    /// Warnings generated during the operation.
    pub warnings: Vec<String>,
}

// ============================================================================
// Twist Deformation
// ============================================================================

/// Options for twist deformation.
///
/// Twist rotates vertices around an axis based on their position along it.
/// The rotation angle is interpolated linearly from 0 at the start of the
/// configured `extent` to `angle_radians` at the end of the `extent`.
#[derive(Debug, Clone, Copy)]
pub struct TwistOptions {
    /// Origin point of the twist axis.
    pub axis_origin: Point3,
    /// Direction of the twist axis (will be normalized internally).
    pub axis_direction: Vec3,
    /// Total twist angle in radians over the full axis extent.
    pub angle_radians: f64,
    /// Optional extent limits along the axis.
    /// If `None`, twist applies over the full mesh bounding extent.
    pub extent: Option<(f64, f64)>,
    /// Whether to recompute normals after deformation.
    pub recompute_normals: bool,
    /// Whether to weld vertices after deformation.
    pub weld_vertices: bool,
}

impl TwistOptions {
    /// Create new twist options.
    ///
    /// # Arguments
    /// * `axis_origin` - Point on the twist axis
    /// * `axis_direction` - Direction of the twist axis
    /// * `angle_radians` - Total twist angle in radians
    #[must_use]
    pub fn new(axis_origin: Point3, axis_direction: Vec3, angle_radians: f64) -> Self {
        Self {
            axis_origin,
            axis_direction,
            angle_radians,
            extent: None,
            recompute_normals: true,
            weld_vertices: true,
        }
    }

    /// Set the extent limits along the axis.
    #[must_use]
    pub const fn extent(mut self, start: f64, end: f64) -> Self {
        self.extent = Some((start, end));
        self
    }

    /// Set whether to recompute normals after deformation.
    #[must_use]
    pub const fn recompute_normals(mut self, recompute: bool) -> Self {
        self.recompute_normals = recompute;
        self
    }

    /// Set whether to weld vertices after deformation.
    #[must_use]
    pub const fn weld_vertices(mut self, weld: bool) -> Self {
        self.weld_vertices = weld;
        self
    }
}

/// Apply twist deformation to a mesh.
///
/// Vertices are rotated around the axis based on their distance along it.
/// The rotation angle is linearly interpolated from 0 at the start of the
/// extent to `angle_radians` at the end.
///
/// # Arguments
/// * `mesh` - The input mesh to deform.
/// * `options` - Twist deformation options.
/// * `tol` - Tolerance for geometry operations.
///
/// # Returns
/// A tuple of the twisted mesh and diagnostics.
///
/// # Errors
/// Returns an error if the mesh is empty, contains invalid geometry,
/// or if the axis is invalid.
pub fn twist_mesh(
    mesh: &GeomMesh,
    options: TwistOptions,
    tol: Tolerance,
) -> Result<(GeomMesh, DeformationDiagnostics), DeformationError> {
    // Validate inputs
    validate_mesh(mesh)?;
    validate_axis(options.axis_direction)?;

    if !options.angle_radians.is_finite() {
        return Err(DeformationError::InvalidParameters);
    }

    let axis_dir = options
        .axis_direction
        .normalized()
        .ok_or(DeformationError::InvalidAxis)?;

    let original_vertex_count = mesh.positions.len();
    let original_triangle_count = mesh.triangle_count();

    // Calculate extent if not provided
    let (extent_start, extent_end) = match options.extent {
        Some((s, e)) => (s, e),
        None => compute_mesh_extent_along_axis(mesh, options.axis_origin, axis_dir),
    };

    let extent = ExtentMapping::new(extent_start, extent_end, tol)?;
    if extent.length < tol.eps {
        // No extent to twist over - return unchanged mesh
        let diagnostics = DeformationDiagnostics {
            original_vertex_count,
            original_triangle_count,
            result_vertex_count: original_vertex_count,
            result_triangle_count: original_triangle_count,
            min_displacement: 0.0,
            max_displacement: 0.0,
            avg_displacement: 0.0,
            welded_vertex_count: 0,
            warnings: vec!["mesh has zero extent along twist axis".to_string()],
        };
        return Ok((mesh.clone(), diagnostics));
    }

    // Apply twist to each vertex
    let mut displaced_positions = Vec::with_capacity(mesh.positions.len());
    let mut displacements = Vec::with_capacity(mesh.positions.len());

    for pos in &mesh.positions {
        let point = Point3::new(pos[0], pos[1], pos[2]);
        let v = point.sub_point(options.axis_origin);

        // Project onto axis to get position along it
        let t = v.dot(axis_dir);
        // Normalize to [0, 1] within extent (safe for reversed extents)
        let t_normalized = extent.normalize_clamped(t);

        // Calculate rotation angle at this position
        let angle = options.angle_radians * t_normalized;

        // Rotate point around axis using Rodrigues' formula
        let rotated = rotate_point_around_axis(point, options.axis_origin, axis_dir, angle);

        let displacement = point.sub_point(rotated).length();
        displacements.push(displacement);
        displaced_positions.push([rotated.x, rotated.y, rotated.z]);
    }

    // Compute displacement statistics
    let (min_disp, max_disp, avg_disp) = compute_displacement_stats(&displacements);

    // Build result mesh with optional weld/normal repair
    build_deformed_mesh(
        displaced_positions,
        mesh,
        options.recompute_normals,
        options.weld_vertices,
        tol,
        original_vertex_count,
        original_triangle_count,
        min_disp,
        max_disp,
        avg_disp,
    )
}

// ============================================================================
// Bend Deformation
// ============================================================================

/// Options for bend deformation.
///
/// Bend arcs the mesh around an axis, creating a circular deformation.
/// Vertices are moved in a circular path around a bend center, with the
/// amount of rotation proportional to their distance along the bend axis.
#[derive(Debug, Clone, Copy)]
pub struct BendOptions {
    /// Origin point of the bend axis.
    pub axis_origin: Point3,
    /// Direction of the bend axis (the direction along which bend is applied).
    pub axis_direction: Vec3,
    /// Direction perpendicular to the bend axis, defining the bend plane.
    /// If not provided, will be computed from axis direction.
    pub bend_direction: Option<Vec3>,
    /// Total bend angle in radians (-2π to 2π).
    pub angle_radians: f64,
    /// Optional extent limits along the axis.
    pub extent: Option<(f64, f64)>,
    /// Whether to recompute normals after deformation.
    pub recompute_normals: bool,
    /// Whether to weld vertices after deformation.
    pub weld_vertices: bool,
}

impl BendOptions {
    /// Create new bend options.
    ///
    /// # Arguments
    /// * `axis_origin` - Point on the bend axis
    /// * `axis_direction` - Direction of the bend axis
    /// * `angle_radians` - Total bend angle in radians
    #[must_use]
    pub fn new(axis_origin: Point3, axis_direction: Vec3, angle_radians: f64) -> Self {
        Self {
            axis_origin,
            axis_direction,
            bend_direction: None,
            angle_radians,
            extent: None,
            recompute_normals: true,
            weld_vertices: true,
        }
    }

    /// Set the bend direction (perpendicular to axis).
    #[must_use]
    pub const fn bend_direction(mut self, direction: Vec3) -> Self {
        self.bend_direction = Some(direction);
        self
    }

    /// Set the extent limits along the axis.
    #[must_use]
    pub const fn extent(mut self, start: f64, end: f64) -> Self {
        self.extent = Some((start, end));
        self
    }

    /// Set whether to recompute normals after deformation.
    #[must_use]
    pub const fn recompute_normals(mut self, recompute: bool) -> Self {
        self.recompute_normals = recompute;
        self
    }

    /// Set whether to weld vertices after deformation.
    #[must_use]
    pub const fn weld_vertices(mut self, weld: bool) -> Self {
        self.weld_vertices = weld;
        self
    }
}

/// Apply bend deformation to a mesh.
///
/// Vertices are moved in a circular arc around a bend center. The arc angle
/// is proportional to the vertex's position along the bend axis.
///
/// The bend occurs in the plane spanned by `axis_direction` and `bend_direction`.
/// The rotation axis is `axis_direction × bend_direction`.
///
/// # Arguments
/// * `mesh` - The input mesh to deform.
/// * `options` - Bend deformation options.
/// * `tol` - Tolerance for geometry operations.
///
/// # Returns
/// A tuple of the bent mesh and diagnostics.
///
/// # Errors
/// Returns an error if the mesh is empty, contains invalid geometry,
/// or if the parameters are invalid.
pub fn bend_mesh(
    mesh: &GeomMesh,
    options: BendOptions,
    tol: Tolerance,
) -> Result<(GeomMesh, DeformationDiagnostics), DeformationError> {
    // Validate inputs
    validate_mesh(mesh)?;
    validate_axis(options.axis_direction)?;

    if !options.angle_radians.is_finite()
        || options.angle_radians < -2.0 * PI
        || options.angle_radians > 2.0 * PI
    {
        return Err(DeformationError::InvalidBendAngle);
    }

    let axis_dir = options
        .axis_direction
        .normalized()
        .ok_or(DeformationError::InvalidAxis)?;

    let original_vertex_count = mesh.positions.len();
    let original_triangle_count = mesh.triangle_count();

    // Calculate extent if not provided
    let (extent_start, extent_end) = match options.extent {
        Some((s, e)) => (s, e),
        None => compute_mesh_extent_along_axis(mesh, options.axis_origin, axis_dir),
    };

    let extent = ExtentMapping::new(extent_start, extent_end, tol)?;
    if extent.length < tol.eps || options.angle_radians.abs() < tol.eps {
        // No extent or angle to bend - return unchanged mesh
        let diagnostics = DeformationDiagnostics {
            original_vertex_count,
            original_triangle_count,
            result_vertex_count: original_vertex_count,
            result_triangle_count: original_triangle_count,
            min_displacement: 0.0,
            max_displacement: 0.0,
            avg_displacement: 0.0,
            welded_vertex_count: 0,
            warnings: vec!["mesh has zero extent or bend angle".to_string()],
        };
        return Ok((mesh.clone(), diagnostics));
    }

    // Compute bend direction (perpendicular to axis)
    let bend_dir = match options.bend_direction {
        Some(dir) => dir.normalized().ok_or(DeformationError::InvalidAxis)?,
        None => compute_perpendicular_direction(axis_dir),
    };

    // Ensure bend direction is perpendicular to axis
    let bend_dir = make_perpendicular(bend_dir, axis_dir);

    // Rotation axis is perpendicular to the bend plane.
    let rot_axis = axis_dir
        .cross(bend_dir)
        .normalized()
        .ok_or(DeformationError::InvalidAxis)?;

    // Arc length = radius * angle, so radius = extent.length / angle.
    // Keep the sign of the angle so bend direction is preserved.
    let bend_radius = extent.length / options.angle_radians;

    // Anchor bend at the start of the extent.
    let base_origin = options
        .axis_origin
        .add_vec(axis_dir.mul_scalar(extent.start));

    // Bend center is offset from the base axis line by bend_radius.
    let bend_center = base_origin.add_vec(bend_dir.mul_scalar(-bend_radius));

    // Apply bend to each vertex
    let mut displaced_positions = Vec::with_capacity(mesh.positions.len());
    let mut displacements = Vec::with_capacity(mesh.positions.len());

    for pos in &mesh.positions {
        let point = Point3::new(pos[0], pos[1], pos[2]);
        let v = point.sub_point(options.axis_origin);

        // Project onto axis to get position along it
        let t = v.dot(axis_dir);
        // Normalize to [0, 1] within extent (safe for reversed extents)
        let t_normalized = extent.normalize_clamped(t);

        // Calculate bend angle at this position
        let angle = options.angle_radians * t_normalized;

        // Compute the perpendicular component to the axis.
        // This defines the cross-section offsets we preserve during bending.
        let axis_component = axis_dir.mul_scalar(t);
        let perp_component = v.sub(axis_component);

        // Decompose perpendicular offsets into bend direction (radial) and rotation axis (out-of-plane).
        let radial_offset = perp_component.dot(bend_dir);
        let out_of_plane_offset = perp_component.dot(rot_axis);

        // Radius from bend center (signed) at this vertex.
        let radius = bend_radius + radial_offset;
        let radius_vec = bend_dir.mul_scalar(radius);
        let rotated_radius_vec = rotate_vector_around_axis(radius_vec, rot_axis, angle);

        let new_point = bend_center
            .add_vec(rotated_radius_vec)
            .add_vec(rot_axis.mul_scalar(out_of_plane_offset));

        let displacement = point.sub_point(new_point).length();
        displacements.push(displacement);
        displaced_positions.push([new_point.x, new_point.y, new_point.z]);
    }

    // Compute displacement statistics
    let (min_disp, max_disp, avg_disp) = compute_displacement_stats(&displacements);

    // Build result mesh with optional weld/normal repair
    build_deformed_mesh(
        displaced_positions,
        mesh,
        options.recompute_normals,
        options.weld_vertices,
        tol,
        original_vertex_count,
        original_triangle_count,
        min_disp,
        max_disp,
        avg_disp,
    )
}

// ============================================================================
// Taper Deformation
// ============================================================================

/// Options for taper deformation.
///
/// Taper scales vertices based on their position along an axis. The scale
/// factor is interpolated linearly from `start_factor` at the start of the
/// configured `extent` to `end_factor` at the end of the `extent`.
#[derive(Debug, Clone, Copy)]
pub struct TaperOptions {
    /// Origin point of the taper axis.
    pub axis_origin: Point3,
    /// Direction of the taper axis (will be normalized internally).
    pub axis_direction: Vec3,
    /// Scale factor at the start of the extent (typically 1.0).
    pub start_factor: f64,
    /// Scale factor at the end of the extent.
    pub end_factor: f64,
    /// Optional extent limits along the axis.
    pub extent: Option<(f64, f64)>,
    /// Whether to preserve the axial component when applying taper.
    ///
    /// - `true` (default): classic taper, scales only the component perpendicular
    ///   to the axis while keeping the coordinate along the axis unchanged.
    /// - `false`: scales both perpendicular and axial components, acting like a
    ///   position-dependent uniform scale about `axis_origin`.
    pub symmetric: bool,
    /// Whether to recompute normals after deformation.
    pub recompute_normals: bool,
    /// Whether to weld vertices after deformation.
    pub weld_vertices: bool,
}

impl TaperOptions {
    /// Create new taper options.
    ///
    /// # Arguments
    /// * `axis_origin` - Point on the taper axis
    /// * `axis_direction` - Direction of the taper axis
    /// * `start_factor` - Scale factor at the start (typically 1.0)
    /// * `end_factor` - Scale factor at the end
    #[must_use]
    pub fn new(
        axis_origin: Point3,
        axis_direction: Vec3,
        start_factor: f64,
        end_factor: f64,
    ) -> Self {
        Self {
            axis_origin,
            axis_direction,
            start_factor,
            end_factor,
            extent: None,
            symmetric: true,
            recompute_normals: true,
            weld_vertices: true,
        }
    }

    /// Set the extent limits along the axis.
    #[must_use]
    pub const fn extent(mut self, start: f64, end: f64) -> Self {
        self.extent = Some((start, end));
        self
    }

    /// Set whether to apply symmetric taper.
    #[must_use]
    pub const fn symmetric(mut self, symmetric: bool) -> Self {
        self.symmetric = symmetric;
        self
    }

    /// Set whether to recompute normals after deformation.
    #[must_use]
    pub const fn recompute_normals(mut self, recompute: bool) -> Self {
        self.recompute_normals = recompute;
        self
    }

    /// Set whether to weld vertices after deformation.
    #[must_use]
    pub const fn weld_vertices(mut self, weld: bool) -> Self {
        self.weld_vertices = weld;
        self
    }
}

/// Apply taper deformation to a mesh.
///
/// Vertices are scaled perpendicular to the axis based on their position
/// along it. The scale factor is linearly interpolated between start_factor
/// and end_factor.
///
/// # Arguments
/// * `mesh` - The input mesh to deform.
/// * `options` - Taper deformation options.
/// * `tol` - Tolerance for geometry operations.
///
/// # Returns
/// A tuple of the tapered mesh and diagnostics.
///
/// # Errors
/// Returns an error if the mesh is empty, contains invalid geometry,
/// or if the parameters are invalid.
pub fn taper_mesh(
    mesh: &GeomMesh,
    options: TaperOptions,
    tol: Tolerance,
) -> Result<(GeomMesh, DeformationDiagnostics), DeformationError> {
    // Validate inputs
    validate_mesh(mesh)?;
    validate_axis(options.axis_direction)?;

    if !options.start_factor.is_finite()
        || !options.end_factor.is_finite()
        || options.start_factor < 0.0
        || options.end_factor < 0.0
    {
        return Err(DeformationError::InvalidTaperFactor);
    }

    let axis_dir = options
        .axis_direction
        .normalized()
        .ok_or(DeformationError::InvalidAxis)?;

    let original_vertex_count = mesh.positions.len();
    let original_triangle_count = mesh.triangle_count();

    // Calculate extent if not provided
    let (extent_start, extent_end) = match options.extent {
        Some((s, e)) => (s, e),
        None => compute_mesh_extent_along_axis(mesh, options.axis_origin, axis_dir),
    };

    let extent = ExtentMapping::new(extent_start, extent_end, tol)?;
    if extent.length < tol.eps {
        // No extent to taper over - return unchanged mesh
        let diagnostics = DeformationDiagnostics {
            original_vertex_count,
            original_triangle_count,
            result_vertex_count: original_vertex_count,
            result_triangle_count: original_triangle_count,
            min_displacement: 0.0,
            max_displacement: 0.0,
            avg_displacement: 0.0,
            welded_vertex_count: 0,
            warnings: vec!["mesh has zero extent along taper axis".to_string()],
        };
        return Ok((mesh.clone(), diagnostics));
    }

    // Apply taper to each vertex
    let mut displaced_positions = Vec::with_capacity(mesh.positions.len());
    let mut displacements = Vec::with_capacity(mesh.positions.len());

    for pos in &mesh.positions {
        let point = Point3::new(pos[0], pos[1], pos[2]);
        let v = point.sub_point(options.axis_origin);

        // Project onto axis to get position along it
        let t = v.dot(axis_dir);
        // Normalize to [0, 1] within extent (safe for reversed extents)
        let t_normalized = extent.normalize_clamped(t);

        // Calculate scale factor at this position
        let scale =
            options.start_factor + (options.end_factor - options.start_factor) * t_normalized;

        // Compute component perpendicular to axis
        let axis_component = axis_dir.mul_scalar(t);
        let perp_component = v.sub(axis_component);

        // Apply taper scaling.
        let scaled_axis_component = if options.symmetric {
            axis_component
        } else {
            axis_component.mul_scalar(scale)
        };
        let scaled_perp = perp_component.mul_scalar(scale);

        // Reconstruct the position
        let new_point = options
            .axis_origin
            .add_vec(scaled_axis_component.add(scaled_perp));

        let displacement = point.sub_point(new_point).length();
        displacements.push(displacement);
        displaced_positions.push([new_point.x, new_point.y, new_point.z]);
    }

    // Compute displacement statistics
    let (min_disp, max_disp, avg_disp) = compute_displacement_stats(&displacements);

    // Build result mesh with optional weld/normal repair
    build_deformed_mesh(
        displaced_positions,
        mesh,
        options.recompute_normals,
        options.weld_vertices,
        tol,
        original_vertex_count,
        original_triangle_count,
        min_disp,
        max_disp,
        avg_disp,
    )
}

// ============================================================================
// Morph Deformation
// ============================================================================

/// Options for morph deformation.
///
/// Morph blends between source vertex positions and target positions.
/// The blend factor controls interpolation: 0.0 = source, 1.0 = target.
#[derive(Debug, Clone)]
pub struct MorphOptions {
    /// Target vertex positions to morph towards.
    pub target_positions: Vec<[f64; 3]>,
    /// Blend factor from 0.0 (source) to 1.0 (target).
    /// Values outside [0, 1] are allowed for extrapolation.
    pub blend_factor: f64,
    /// Whether to recompute normals after deformation.
    pub recompute_normals: bool,
    /// Whether to weld vertices after deformation.
    pub weld_vertices: bool,
}

impl MorphOptions {
    /// Create new morph options.
    ///
    /// # Arguments
    /// * `target_positions` - Target positions for each vertex
    /// * `blend_factor` - Interpolation factor (0.0 = source, 1.0 = target)
    #[must_use]
    pub fn new(target_positions: Vec<[f64; 3]>, blend_factor: f64) -> Self {
        Self {
            target_positions,
            blend_factor,
            recompute_normals: true,
            weld_vertices: true,
        }
    }

    /// Set whether to recompute normals after deformation.
    #[must_use]
    pub const fn recompute_normals(mut self, recompute: bool) -> Self {
        self.recompute_normals = recompute;
        self
    }

    /// Set whether to weld vertices after deformation.
    #[must_use]
    pub const fn weld_vertices(mut self, weld: bool) -> Self {
        self.weld_vertices = weld;
        self
    }
}

/// Apply morph deformation to a mesh.
///
/// Vertices are linearly interpolated between their source positions
/// and the target positions based on the blend factor.
///
/// # Arguments
/// * `mesh` - The input mesh to deform.
/// * `options` - Morph deformation options.
/// * `tol` - Tolerance for geometry operations.
///
/// # Returns
/// A tuple of the morphed mesh and diagnostics.
///
/// # Errors
/// Returns an error if the mesh is empty, contains invalid geometry,
/// or if target vertex count doesn't match source.
pub fn morph_mesh(
    mesh: &GeomMesh,
    options: MorphOptions,
    tol: Tolerance,
) -> Result<(GeomMesh, DeformationDiagnostics), DeformationError> {
    // Validate inputs
    validate_mesh(mesh)?;

    if mesh.positions.len() != options.target_positions.len() {
        return Err(DeformationError::MorphVertexCountMismatch {
            source_count: mesh.positions.len(),
            target_count: options.target_positions.len(),
        });
    }

    if !options.blend_factor.is_finite() {
        return Err(DeformationError::InvalidParameters);
    }

    // Validate target positions
    for pos in &options.target_positions {
        if !pos[0].is_finite() || !pos[1].is_finite() || !pos[2].is_finite() {
            return Err(DeformationError::InvalidGeometry);
        }
    }

    let original_vertex_count = mesh.positions.len();
    let original_triangle_count = mesh.triangle_count();

    let t = options.blend_factor;
    let one_minus_t = 1.0 - t;

    // Apply morph to each vertex
    let mut displaced_positions = Vec::with_capacity(mesh.positions.len());
    let mut displacements = Vec::with_capacity(mesh.positions.len());

    for (source, target) in mesh.positions.iter().zip(options.target_positions.iter()) {
        let new_pos = [
            source[0] * one_minus_t + target[0] * t,
            source[1] * one_minus_t + target[1] * t,
            source[2] * one_minus_t + target[2] * t,
        ];

        let dx = new_pos[0] - source[0];
        let dy = new_pos[1] - source[1];
        let dz = new_pos[2] - source[2];
        let displacement = (dx * dx + dy * dy + dz * dz).sqrt();

        displacements.push(displacement);
        displaced_positions.push(new_pos);
    }

    // Compute displacement statistics
    let (min_disp, max_disp, avg_disp) = compute_displacement_stats(&displacements);

    // Build result mesh with optional weld/normal repair
    build_deformed_mesh(
        displaced_positions,
        mesh,
        options.recompute_normals,
        options.weld_vertices,
        tol,
        original_vertex_count,
        original_triangle_count,
        min_disp,
        max_disp,
        avg_disp,
    )
}

// ============================================================================
// Convenience functions
// ============================================================================

/// Twist a mesh around the Z-axis with a given angle.
///
/// Convenience function for simple Z-axis twists.
pub fn twist_mesh_z(
    mesh: &GeomMesh,
    angle_radians: f64,
    tol: Tolerance,
) -> Result<(GeomMesh, DeformationDiagnostics), DeformationError> {
    twist_mesh(
        mesh,
        TwistOptions::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            angle_radians,
        ),
        tol,
    )
}

/// Bend a mesh around the Z-axis with a given angle.
///
/// Convenience function for simple Z-axis bends.
pub fn bend_mesh_z(
    mesh: &GeomMesh,
    angle_radians: f64,
    tol: Tolerance,
) -> Result<(GeomMesh, DeformationDiagnostics), DeformationError> {
    bend_mesh(
        mesh,
        BendOptions::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            angle_radians,
        ),
        tol,
    )
}

/// Taper a mesh along the Z-axis.
///
/// Convenience function for simple Z-axis tapers.
pub fn taper_mesh_z(
    mesh: &GeomMesh,
    start_factor: f64,
    end_factor: f64,
    tol: Tolerance,
) -> Result<(GeomMesh, DeformationDiagnostics), DeformationError> {
    taper_mesh(
        mesh,
        TaperOptions::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            start_factor,
            end_factor,
        ),
        tol,
    )
}

// ============================================================================
// Internal helper functions
// ============================================================================

/// Validate that a mesh is non-empty and contains finite geometry.
fn validate_mesh(mesh: &GeomMesh) -> Result<(), DeformationError> {
    if mesh.indices.is_empty() || mesh.positions.is_empty() {
        return Err(DeformationError::EmptyMesh);
    }

    for pos in &mesh.positions {
        if !pos[0].is_finite() || !pos[1].is_finite() || !pos[2].is_finite() {
            return Err(DeformationError::InvalidGeometry);
        }
    }

    Ok(())
}

/// Validate that an axis direction is non-zero and finite.
fn validate_axis(axis: Vec3) -> Result<(), DeformationError> {
    if !axis.x.is_finite() || !axis.y.is_finite() || !axis.z.is_finite() {
        return Err(DeformationError::InvalidAxis);
    }
    if axis.length_squared() < 1e-20 {
        return Err(DeformationError::InvalidAxis);
    }
    Ok(())
}

/// Compute the extent of a mesh along an axis.
fn compute_mesh_extent_along_axis(
    mesh: &GeomMesh,
    origin: Point3,
    axis_dir: Vec3,
) -> (f64, f64) {
    let mut min_t = f64::MAX;
    let mut max_t = f64::MIN;

    for pos in &mesh.positions {
        let point = Point3::new(pos[0], pos[1], pos[2]);
        let v = point.sub_point(origin);
        let t = v.dot(axis_dir);
        min_t = min_t.min(t);
        max_t = max_t.max(t);
    }

    (min_t, max_t)
}

#[derive(Debug, Clone, Copy)]
struct ExtentMapping {
    start: f64,
    end: f64,
    min: f64,
    max: f64,
    length: f64,
}

impl ExtentMapping {
    fn new(start: f64, end: f64, tol: Tolerance) -> Result<Self, DeformationError> {
        if !start.is_finite() || !end.is_finite() {
            return Err(DeformationError::InvalidParameters);
        }

        let (min, max) = if start <= end { (start, end) } else { (end, start) };
        let length = (end - start).abs();
        if !length.is_finite() || length < tol.eps {
            return Ok(Self {
                start,
                end,
                min,
                max,
                length: 0.0,
            });
        }

        Ok(Self {
            start,
            end,
            min,
            max,
            length,
        })
    }

    fn normalize_clamped(self, t: f64) -> f64 {
        if self.length == 0.0 {
            return 0.0;
        }

        let t_clamped = t.clamp(self.min, self.max);

        // Anchor the deformation at `start` and map toward `end`.
        if self.start <= self.end {
            ((t_clamped - self.start) / self.length).clamp(0.0, 1.0)
        } else {
            ((self.start - t_clamped) / self.length).clamp(0.0, 1.0)
        }
    }
}

/// Rotate a point around an axis using Rodrigues' formula.
fn rotate_point_around_axis(point: Point3, axis_origin: Point3, axis_dir: Vec3, angle: f64) -> Point3 {
    let v = point.sub_point(axis_origin);
    let rotated_v = rotate_vector_around_axis(v, axis_dir, angle);
    axis_origin.add_vec(rotated_v)
}

/// Rotate a vector around an axis using Rodrigues' formula.
fn rotate_vector_around_axis(v: Vec3, axis: Vec3, angle: f64) -> Vec3 {
    let cos_a = angle.cos();
    let sin_a = angle.sin();

    // Rodrigues' rotation formula:
    // v_rot = v * cos(a) + (axis × v) * sin(a) + axis * (axis · v) * (1 - cos(a))
    let v_cos = v.mul_scalar(cos_a);
    let cross_sin = axis.cross(v).mul_scalar(sin_a);
    let axis_component = axis.mul_scalar(axis.dot(v) * (1.0 - cos_a));

    v_cos.add(cross_sin).add(axis_component)
}

/// Compute a perpendicular direction to a given axis.
fn compute_perpendicular_direction(axis: Vec3) -> Vec3 {
    // Use a reference vector that's not parallel to the axis
    let reference = if axis.x.abs() < 0.9 {
        Vec3::new(1.0, 0.0, 0.0)
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    };

    axis.cross(reference)
        .normalized()
        .unwrap_or(Vec3::new(1.0, 0.0, 0.0))
}

/// Make a direction perpendicular to an axis.
fn make_perpendicular(dir: Vec3, axis: Vec3) -> Vec3 {
    // Remove the component parallel to axis
    let parallel = axis.mul_scalar(dir.dot(axis));
    let perp = dir.sub(parallel);
    perp.normalized().unwrap_or_else(|| compute_perpendicular_direction(axis))
}

/// Compute min, max, and average displacement.
fn compute_displacement_stats(displacements: &[f64]) -> (f64, f64, f64) {
    if displacements.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let mut min_d = f64::MAX;
    let mut max_d = f64::MIN;
    let mut sum = 0.0;

    for &d in displacements {
        min_d = min_d.min(d);
        max_d = max_d.max(d);
        sum += d;
    }

    let avg = sum / displacements.len() as f64;
    (min_d, max_d, avg)
}

/// Build the final deformed mesh with optional weld/normal repair.
#[allow(clippy::too_many_arguments)]
fn build_deformed_mesh(
    displaced_positions: Vec<[f64; 3]>,
    original_mesh: &GeomMesh,
    recompute_normals: bool,
    weld_vertices: bool,
    tol: Tolerance,
    original_vertex_count: usize,
    original_triangle_count: usize,
    min_disp: f64,
    max_disp: f64,
    avg_disp: f64,
) -> Result<(GeomMesh, DeformationDiagnostics), DeformationError> {
    let indices = original_mesh.indices.clone();
    let uvs = original_mesh.uvs.clone();

    let mut local_warnings: Vec<String> = Vec::new();

    let (result_mesh, mesh_diagnostics) = if weld_vertices {
        // Welding implies an index/vertex remap, so we can't safely preserve input normals.
        // `finalize_mesh` always computes normals; if the caller disables recompute, drop them.
        let points: Vec<Point3> = displaced_positions
            .iter()
            .map(|p| Point3::new(p[0], p[1], p[2]))
            .collect();

        let (mut mesh, diag) = finalize_mesh(points, uvs, indices, tol);
        if !recompute_normals {
            mesh.normals = None;
            local_warnings.push(
                "normals dropped because weld_vertices=true and recompute_normals=false"
                    .to_string(),
            );
        }
        (mesh, diag)
    } else {
        // No welding: preserve indices and vertex ordering.
        let normals = if recompute_normals {
            let points: Vec<Point3> = displaced_positions
                .iter()
                .map(|p| Point3::new(p[0], p[1], p[2]))
                .collect();
            Some(compute_smooth_normals_for_mesh(&points, &indices))
        } else {
            if original_mesh.normals.is_some() {
                local_warnings.push(
                    "normals preserved without recomputation; they may be stale after deformation"
                        .to_string(),
                );
            }
            original_mesh.normals.clone()
        };

        let result = GeomMesh {
            positions: displaced_positions,
            indices,
            uvs,
            normals,
            tangents: None,
        };
        let diag = super::diagnostics::GeomMeshDiagnostics {
            vertex_count: result.positions.len(),
            triangle_count: result.triangle_count(),
            ..Default::default()
        };
        (result, diag)
    };

    let mut warnings = local_warnings;
    warnings.extend(mesh_diagnostics.warnings);

    let diagnostics = DeformationDiagnostics {
        original_vertex_count,
        original_triangle_count,
        result_vertex_count: result_mesh.positions.len(),
        result_triangle_count: result_mesh.triangle_count(),
        min_displacement: min_disp,
        max_displacement: max_disp,
        avg_displacement: avg_disp,
        welded_vertex_count: mesh_diagnostics.welded_vertex_count,
        warnings,
    };

    Ok((result_mesh, diagnostics))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_cube() -> GeomMesh {
        // Simple 2x2x2 cube centered at origin
        let positions = vec![
            [-1.0, -1.0, -1.0],
            [1.0, -1.0, -1.0],
            [1.0, 1.0, -1.0],
            [-1.0, 1.0, -1.0],
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, 1.0, 1.0],
            [-1.0, 1.0, 1.0],
        ];
        let indices = vec![
            // Bottom face
            0, 1, 2, 0, 2, 3, // Top face
            4, 6, 5, 4, 7, 6, // Front face
            0, 5, 1, 0, 4, 5, // Back face
            2, 7, 3, 2, 6, 7, // Left face
            0, 3, 7, 0, 7, 4, // Right face
            1, 6, 2, 1, 5, 6,
        ];
        GeomMesh {
            positions,
            indices,
            uvs: None,
            normals: None,
            tangents: None,
        }
    }

    #[test]
    fn test_twist_zero_angle() {
        let mesh = create_test_cube();
        let options = TwistOptions::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            0.0,
        );
        let result = twist_mesh(&mesh, options, Tolerance::default_geom());
        assert!(result.is_ok());
        let (twisted, diag) = result.unwrap();
        // Zero angle should produce minimal displacement
        assert!(diag.max_displacement < 1e-10);
        assert_eq!(twisted.positions.len(), mesh.positions.len());
    }

    #[test]
    fn test_twist_90_degrees() {
        let mesh = create_test_cube();
        let options = TwistOptions::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            PI / 2.0,
        );
        let result = twist_mesh(&mesh, options, Tolerance::default_geom());
        assert!(result.is_ok());
        let (twisted, diag) = result.unwrap();
        // Some displacement should occur
        assert!(diag.max_displacement > 0.0);
        // Vertex count should be preserved (or increased due to welding)
        assert!(twisted.positions.len() >= 8);
    }

    #[test]
    fn test_taper_uniform() {
        let mesh = create_test_cube();
        let options = TaperOptions::new(
            Point3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 0.0, 1.0),
            1.0,
            1.0,
        );
        let result = taper_mesh(&mesh, options, Tolerance::default_geom());
        assert!(result.is_ok());
        let (_tapered, diag) = result.unwrap();
        // Uniform scale should produce no displacement
        assert!(diag.max_displacement < 1e-10);
    }

    #[test]
    fn test_taper_shrink() {
        let mesh = create_test_cube();
        let options = TaperOptions::new(
            Point3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 0.0, 1.0),
            1.0,
            0.5,
        );
        let result = taper_mesh(&mesh, options, Tolerance::default_geom());
        assert!(result.is_ok());
        let (tapered, diag) = result.unwrap();
        // Some displacement should occur
        assert!(diag.max_displacement > 0.0);
        assert!(!tapered.indices.is_empty());
    }

    #[test]
    fn test_morph_identity() {
        let mesh = create_test_cube();
        let options = MorphOptions::new(mesh.positions.clone(), 0.0);
        let result = morph_mesh(&mesh, options, Tolerance::default_geom());
        assert!(result.is_ok());
        let (_morphed, diag) = result.unwrap();
        // Zero blend should produce no displacement
        assert!(diag.max_displacement < 1e-10);
    }

    #[test]
    fn test_morph_half_blend() {
        let mesh = create_test_cube();
        // Create target positions (scaled version)
        let target: Vec<[f64; 3]> = mesh
            .positions
            .iter()
            .map(|p| [p[0] * 2.0, p[1] * 2.0, p[2] * 2.0])
            .collect();
        let options = MorphOptions::new(target, 0.5);
        let result = morph_mesh(&mesh, options, Tolerance::default_geom());
        assert!(result.is_ok());
        let (morphed, diag) = result.unwrap();
        // Half blend should produce half the displacement
        assert!(diag.max_displacement > 0.0);
        assert!(!morphed.indices.is_empty());
    }

    #[test]
    fn test_bend_zero_angle() {
        let mesh = create_test_cube();
        let options = BendOptions::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            0.0,
        );
        let result = bend_mesh(&mesh, options, Tolerance::default_geom());
        assert!(result.is_ok());
        let (_bent, diag) = result.unwrap();
        // Zero angle returns unchanged mesh with warning
        assert!(!diag.warnings.is_empty());
    }

    #[test]
    fn test_empty_mesh_error() {
        let empty_mesh = GeomMesh {
            positions: vec![],
            indices: vec![],
            uvs: None,
            normals: None,
            tangents: None,
        };
        let result = twist_mesh(
            &empty_mesh,
            TwistOptions::new(
                Point3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
                PI,
            ),
            Tolerance::default_geom(),
        );
        assert!(matches!(result, Err(DeformationError::EmptyMesh)));
    }

    #[test]
    fn test_invalid_axis_error() {
        let mesh = create_test_cube();
        let result = twist_mesh(
            &mesh,
            TwistOptions::new(
                Point3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 0.0), // Zero axis
                PI,
            ),
            Tolerance::default_geom(),
        );
        assert!(matches!(result, Err(DeformationError::InvalidAxis)));
    }

    #[test]
    fn test_morph_vertex_count_mismatch() {
        let mesh = create_test_cube();
        let options = MorphOptions::new(vec![[0.0, 0.0, 0.0]], 0.5); // Wrong count
        let result = morph_mesh(&mesh, options, Tolerance::default_geom());
        assert!(matches!(
            result,
            Err(DeformationError::MorphVertexCountMismatch { .. })
        ));
    }
}
