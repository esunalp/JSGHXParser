//! Displacement and heightfield operations for meshes.
//!
//! This module provides functionality to:
//! - Displace mesh vertices along their normals based on scalar values (heightfield)
//! - Apply procedural displacement patterns (noise, gradient, image-based)
//! - Apply post-displacement weld and normal repair
//!
//! # Example
//!
//! ```ignore
//! use ghx_engine::geom::{displace_mesh, DisplacementOptions, DisplacementSource};
//!
//! let (mesh, diag) = some_mesh_source();
//! let options = DisplacementOptions::new(DisplacementSource::uniform(0.1));
//! let (displaced, disp_diag) = displace_mesh(&mesh, options, Tolerance::default_geom())?;
//! ```

use super::mesh::{finalize_mesh, GeomMesh};
use super::{Point3, Tolerance, Vec3};

/// Source of displacement values for each vertex.
#[derive(Debug, Clone)]
pub enum DisplacementSource {
    /// Uniform displacement - all vertices displaced by the same amount.
    Uniform(f64),

    /// Per-vertex displacement values (must match vertex count).
    PerVertex(Vec<f64>),

    /// UV-mapped heightfield grid (bilinear interpolation).
    /// Grid is row-major with dimensions (width, height) and values in range [0, 1].
    Heightfield {
        values: Vec<f64>,
        width: usize,
        height: usize,
        /// Scale applied to normalized heightfield values.
        scale: f64,
        /// Offset added after scaling.
        offset: f64,
    },

    /// Procedural gradient displacement based on vertex position.
    Gradient {
        /// Direction of the gradient (normalized internally).
        direction: Vec3,
        /// Minimum displacement at gradient start.
        min_value: f64,
        /// Maximum displacement at gradient end.
        max_value: f64,
    },

    /// Simple 3D noise-based displacement (deterministic, not using external RNG).
    Noise {
        /// Scale of the noise pattern in world units.
        frequency: f64,
        /// Amplitude of the displacement.
        amplitude: f64,
        /// Seed for deterministic noise.
        seed: u32,
    },
}

impl DisplacementSource {
    /// Create a uniform displacement source.
    #[must_use]
    pub const fn uniform(value: f64) -> Self {
        Self::Uniform(value)
    }

    /// Create a per-vertex displacement source.
    #[must_use]
    pub fn per_vertex(values: Vec<f64>) -> Self {
        Self::PerVertex(values)
    }

    /// Create a heightfield displacement source.
    #[must_use]
    pub fn heightfield(values: Vec<f64>, width: usize, height: usize, scale: f64) -> Self {
        Self::Heightfield {
            values,
            width,
            height,
            scale,
            offset: 0.0,
        }
    }

    /// Create a heightfield with offset.
    #[must_use]
    pub fn heightfield_with_offset(
        values: Vec<f64>,
        width: usize,
        height: usize,
        scale: f64,
        offset: f64,
    ) -> Self {
        Self::Heightfield {
            values,
            width,
            height,
            scale,
            offset,
        }
    }

    /// Create a gradient displacement source.
    #[must_use]
    pub fn gradient(direction: Vec3, min_value: f64, max_value: f64) -> Self {
        Self::Gradient {
            direction,
            min_value,
            max_value,
        }
    }

    /// Create a noise displacement source.
    #[must_use]
    pub const fn noise(frequency: f64, amplitude: f64, seed: u32) -> Self {
        Self::Noise {
            frequency,
            amplitude,
            seed,
        }
    }
}

/// Options for displacement operations.
#[derive(Debug, Clone)]
pub struct DisplacementOptions {
    /// The source of displacement values.
    pub source: DisplacementSource,

    /// Whether to clamp displacement to a maximum value.
    pub max_displacement: Option<f64>,

    /// Whether to clamp displacement to a minimum value.
    pub min_displacement: Option<f64>,

    /// Whether to use vertex normals for displacement direction.
    /// - If `true` (default): vertices are displaced along their computed or provided normals.
    /// - If `false`: vertices are displaced along the positive Z axis `(0, 0, 1)`.
    pub use_normals: bool,

    /// Whether to recompute normals after displacement.
    pub recompute_normals: bool,

    /// Whether to weld vertices after displacement.
    pub weld_vertices: bool,
}

impl DisplacementOptions {
    /// Create new displacement options with the given source.
    #[must_use]
    pub fn new(source: DisplacementSource) -> Self {
        Self {
            source,
            max_displacement: None,
            min_displacement: None,
            use_normals: true,
            recompute_normals: true,
            weld_vertices: true,
        }
    }

    /// Set maximum displacement clamp.
    #[must_use]
    pub const fn max_displacement(mut self, max: f64) -> Self {
        self.max_displacement = Some(max);
        self
    }

    /// Set minimum displacement clamp.
    #[must_use]
    pub const fn min_displacement(mut self, min: f64) -> Self {
        self.min_displacement = Some(min);
        self
    }

    /// Set whether to use vertex normals for displacement direction.
    #[must_use]
    pub const fn use_normals(mut self, use_normals: bool) -> Self {
        self.use_normals = use_normals;
        self
    }

    /// Set whether to recompute normals after displacement.
    #[must_use]
    pub const fn recompute_normals(mut self, recompute: bool) -> Self {
        self.recompute_normals = recompute;
        self
    }

    /// Set whether to weld vertices after displacement.
    #[must_use]
    pub const fn weld_vertices(mut self, weld: bool) -> Self {
        self.weld_vertices = weld;
        self
    }
}

/// Errors that can occur during displacement operations.
#[derive(Debug)]
pub enum DisplacementError {
    /// The input mesh has no triangles.
    EmptyMesh,

    /// The input mesh contains invalid geometry (NaN/Inf values).
    InvalidGeometry,

    /// Per-vertex displacement array size mismatch.
    VertexCountMismatch {
        expected: usize,
        got: usize,
    },

    /// Heightfield dimensions are invalid.
    InvalidHeightfieldDimensions {
        width: usize,
        height: usize,
        value_count: usize,
    },

    /// Mesh has no UVs but heightfield displacement requires them.
    MissingUvs,

    /// Mesh normals array length does not match positions array length.
    NormalsLengthMismatch {
        positions_len: usize,
        normals_len: usize,
    },

    /// Displacement values or parameters contain NaN or Inf.
    InvalidDisplacementValues,
}

impl std::fmt::Display for DisplacementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyMesh => write!(f, "input mesh has no triangles"),
            Self::InvalidGeometry => {
                write!(f, "input mesh contains invalid geometry (NaN/Inf values)")
            }
            Self::VertexCountMismatch { expected, got } => write!(
                f,
                "per-vertex displacement array size mismatch: expected {expected}, got {got}"
            ),
            Self::InvalidHeightfieldDimensions {
                width,
                height,
                value_count,
            } => write!(
                f,
                "heightfield dimensions invalid: {width}x{height} requires {} values, got {value_count}",
                width * height
            ),
            Self::MissingUvs => write!(f, "mesh has no UVs but heightfield displacement requires them"),
            Self::NormalsLengthMismatch {
                positions_len,
                normals_len,
            } => write!(
                f,
                "mesh normals length ({normals_len}) does not match positions length ({positions_len})"
            ),
            Self::InvalidDisplacementValues => {
                write!(f, "displacement values contain NaN or Inf")
            }
        }
    }
}

impl std::error::Error for DisplacementError {}

/// Diagnostics specific to displacement operations.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DisplacementDiagnostics {
    /// Number of vertices in the original mesh.
    pub original_vertex_count: usize,
    /// Number of triangles in the original mesh.
    pub original_triangle_count: usize,
    /// Number of vertices in the result.
    pub result_vertex_count: usize,
    /// Number of triangles in the result.
    pub result_triangle_count: usize,
    /// Minimum displacement value applied.
    pub min_displacement_applied: f64,
    /// Maximum displacement value applied.
    pub max_displacement_applied: f64,
    /// Average displacement value applied.
    pub avg_displacement_applied: f64,
    /// Number of vertices that were clamped.
    pub clamped_vertex_count: usize,
    /// Number of vertices welded during post-processing.
    pub welded_vertex_count: usize,
    /// Warnings generated during the operation.
    pub warnings: Vec<String>,
}

/// Displace a mesh by moving vertices along a direction based on a displacement source.
///
/// # Arguments
/// * `mesh` - The input mesh to displace.
/// * `options` - Options controlling the displacement operation.
/// * `tol` - Tolerance for geometry operations.
///
/// # Returns
/// A tuple of the displaced mesh and diagnostics.
///
/// # Errors
/// Returns an error if the mesh is empty, contains invalid geometry,
/// or if displacement source parameters are invalid.
pub fn displace_mesh(
    mesh: &GeomMesh,
    options: DisplacementOptions,
    tol: Tolerance,
) -> Result<(GeomMesh, DisplacementDiagnostics), DisplacementError> {
    // Validate inputs
    if mesh.indices.is_empty() || mesh.positions.is_empty() {
        return Err(DisplacementError::EmptyMesh);
    }

    // Validate geometry
    for pos in &mesh.positions {
        if !pos[0].is_finite() || !pos[1].is_finite() || !pos[2].is_finite() {
            return Err(DisplacementError::InvalidGeometry);
        }
    }

    let original_vertex_count = mesh.positions.len();
    let original_triangle_count = mesh.triangle_count();

    // Compute or use existing normals
    let normals = if options.use_normals {
        compute_vertex_normals(mesh)?
    } else {
        // Use Z-up direction for all vertices
        vec![Vec3::new(0.0, 0.0, 1.0); mesh.positions.len()]
    };

    // Compute displacement values for each vertex
    let displacement_values =
        compute_displacement_values(mesh, &options.source, original_vertex_count)?;

    // Apply clamping and collect statistics
    let (clamped_values, stats) = apply_clamping_and_stats(&displacement_values, &options);

    // Apply displacement to positions
    let displaced_positions = apply_displacement(&mesh.positions, &normals, &clamped_values);

    // Build the result mesh
    let indices = mesh.indices.clone();
    let uvs = mesh.uvs.clone();

    let (result_mesh, mesh_diagnostics) = if options.weld_vertices || options.recompute_normals {
        // Use finalize_mesh for welding and normal recomputation
        let points: Vec<Point3> = displaced_positions
            .iter()
            .map(|p| Point3::new(p[0], p[1], p[2]))
            .collect();
        finalize_mesh(points, uvs, indices, tol)
    } else {
        // Build mesh directly without welding
        let result = GeomMesh {
            positions: displaced_positions,
            indices,
            uvs,
            normals: mesh.normals.clone(),
            tangents: mesh.tangents.clone(),
        };
        let diag = super::diagnostics::GeomMeshDiagnostics {
            vertex_count: result.positions.len(),
            triangle_count: result.triangle_count(),
            ..Default::default()
        };
        (result, diag)
    };

    let mut warnings = stats.warnings;
    warnings.extend(mesh_diagnostics.warnings);

    let diagnostics = DisplacementDiagnostics {
        original_vertex_count,
        original_triangle_count,
        result_vertex_count: result_mesh.positions.len(),
        result_triangle_count: result_mesh.triangle_count(),
        min_displacement_applied: stats.min_value,
        max_displacement_applied: stats.max_value,
        avg_displacement_applied: stats.avg_value,
        clamped_vertex_count: stats.clamped_count,
        welded_vertex_count: mesh_diagnostics.welded_vertex_count,
        warnings,
    };

    Ok((result_mesh, diagnostics))
}

/// Displace a mesh uniformly by a given distance along vertex normals.
///
/// Convenience function that calls `displace_mesh` with `DisplacementSource::Uniform`.
pub fn displace_mesh_uniform(
    mesh: &GeomMesh,
    distance: f64,
    tol: Tolerance,
) -> Result<(GeomMesh, DisplacementDiagnostics), DisplacementError> {
    displace_mesh(
        mesh,
        DisplacementOptions::new(DisplacementSource::uniform(distance)),
        tol,
    )
}

/// Displace a mesh using per-vertex displacement values.
///
/// Convenience function that calls `displace_mesh` with `DisplacementSource::PerVertex`.
pub fn displace_mesh_per_vertex(
    mesh: &GeomMesh,
    values: Vec<f64>,
    tol: Tolerance,
) -> Result<(GeomMesh, DisplacementDiagnostics), DisplacementError> {
    displace_mesh(
        mesh,
        DisplacementOptions::new(DisplacementSource::per_vertex(values)),
        tol,
    )
}

/// Displace a mesh using a heightfield (requires UVs).
///
/// Convenience function that calls `displace_mesh` with `DisplacementSource::Heightfield`.
pub fn displace_mesh_heightfield(
    mesh: &GeomMesh,
    heightfield: Vec<f64>,
    width: usize,
    height: usize,
    scale: f64,
    tol: Tolerance,
) -> Result<(GeomMesh, DisplacementDiagnostics), DisplacementError> {
    displace_mesh(
        mesh,
        DisplacementOptions::new(DisplacementSource::heightfield(
            heightfield,
            width,
            height,
            scale,
        )),
        tol,
    )
}

/// Displace a mesh using procedural noise.
///
/// Convenience function that calls `displace_mesh` with `DisplacementSource::Noise`.
pub fn displace_mesh_noise(
    mesh: &GeomMesh,
    frequency: f64,
    amplitude: f64,
    seed: u32,
    tol: Tolerance,
) -> Result<(GeomMesh, DisplacementDiagnostics), DisplacementError> {
    displace_mesh(
        mesh,
        DisplacementOptions::new(DisplacementSource::noise(frequency, amplitude, seed)),
        tol,
    )
}

// ============================================================================
// Internal helper functions
// ============================================================================

/// Compute vertex normals for a mesh by averaging face normals.
///
/// If the mesh already has normals with correct length, those are used.
/// Otherwise, normals are computed by averaging adjacent face normals.
fn compute_vertex_normals(mesh: &GeomMesh) -> Result<Vec<Vec3>, DisplacementError> {
    // If the mesh already has normals with correct length, use them
    if let Some(ref normals) = mesh.normals {
        if normals.len() == mesh.positions.len() {
            return Ok(normals
                .iter()
                .map(|n| Vec3::new(n[0], n[1], n[2]))
                .collect());
        }
        // Normals exist but wrong length - return error rather than silently recomputing
        return Err(DisplacementError::NormalsLengthMismatch {
            positions_len: mesh.positions.len(),
            normals_len: normals.len(),
        });
    }

    let n_verts = mesh.positions.len();
    let mut normals = vec![Vec3::new(0.0, 0.0, 0.0); n_verts];

    for tri in mesh.indices.chunks_exact(3) {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;

        let (Some(p0), Some(p1), Some(p2)) = (
            mesh.positions.get(i0),
            mesh.positions.get(i1),
            mesh.positions.get(i2),
        ) else {
            continue;
        };

        let a = Point3::new(p0[0], p0[1], p0[2]);
        let b = Point3::new(p1[0], p1[1], p1[2]);
        let c = Point3::new(p2[0], p2[1], p2[2]);

        let ab = b.sub_point(a);
        let ac = c.sub_point(a);
        let face_normal = ab.cross(ac);

        // Weight by face area (cross product magnitude)
        normals[i0] = normals[i0].add(face_normal);
        normals[i1] = normals[i1].add(face_normal);
        normals[i2] = normals[i2].add(face_normal);
    }

    // Normalize all vertex normals
    for n in &mut normals {
        if let Some(normalized) = n.normalized() {
            *n = normalized;
        } else {
            // Degenerate normal - use a fallback
            *n = Vec3::new(0.0, 0.0, 1.0);
        }
    }

    Ok(normals)
}

/// Compute displacement values for each vertex based on the source.
fn compute_displacement_values(
    mesh: &GeomMesh,
    source: &DisplacementSource,
    vertex_count: usize,
) -> Result<Vec<f64>, DisplacementError> {
    match source {
        DisplacementSource::Uniform(value) => {
            if !value.is_finite() {
                return Err(DisplacementError::InvalidDisplacementValues);
            }
            Ok(vec![*value; vertex_count])
        }

        DisplacementSource::PerVertex(values) => {
            if values.len() != vertex_count {
                return Err(DisplacementError::VertexCountMismatch {
                    expected: vertex_count,
                    got: values.len(),
                });
            }
            for v in values {
                if !v.is_finite() {
                    return Err(DisplacementError::InvalidDisplacementValues);
                }
            }
            Ok(values.clone())
        }

        DisplacementSource::Heightfield {
            values,
            width,
            height,
            scale,
            offset,
        } => {
            if *width == 0 || *height == 0 || values.len() != width * height {
                return Err(DisplacementError::InvalidHeightfieldDimensions {
                    width: *width,
                    height: *height,
                    value_count: values.len(),
                });
            }

            // Validate heightfield values for NaN/Inf
            for v in values {
                if !v.is_finite() {
                    return Err(DisplacementError::InvalidDisplacementValues);
                }
            }

            // Validate scale and offset
            if !scale.is_finite() || !offset.is_finite() {
                return Err(DisplacementError::InvalidDisplacementValues);
            }

            let uvs = mesh.uvs.as_ref().ok_or(DisplacementError::MissingUvs)?;

            if uvs.len() != vertex_count {
                return Err(DisplacementError::MissingUvs);
            }

            let mut result = Vec::with_capacity(vertex_count);
            for uv in uvs {
                let u = uv[0].clamp(0.0, 1.0);
                let v = uv[1].clamp(0.0, 1.0);

                // Bilinear interpolation
                let x = u * (*width - 1) as f64;
                let y = v * (*height - 1) as f64;

                let x0 = x.floor() as usize;
                let y0 = y.floor() as usize;
                let x1 = (x0 + 1).min(*width - 1);
                let y1 = (y0 + 1).min(*height - 1);

                let fx = x - x0 as f64;
                let fy = y - y0 as f64;

                let v00 = values[y0 * width + x0];
                let v10 = values[y0 * width + x1];
                let v01 = values[y1 * width + x0];
                let v11 = values[y1 * width + x1];

                let interpolated = v00 * (1.0 - fx) * (1.0 - fy)
                    + v10 * fx * (1.0 - fy)
                    + v01 * (1.0 - fx) * fy
                    + v11 * fx * fy;

                result.push(interpolated * scale + offset);
            }

            Ok(result)
        }

        DisplacementSource::Gradient {
            direction,
            min_value,
            max_value,
        } => {
            // Validate gradient parameters
            if !min_value.is_finite() || !max_value.is_finite() {
                return Err(DisplacementError::InvalidDisplacementValues);
            }
            if !direction.x.is_finite() || !direction.y.is_finite() || !direction.z.is_finite() {
                return Err(DisplacementError::InvalidDisplacementValues);
            }

            let dir = direction.normalized().unwrap_or(Vec3::new(0.0, 0.0, 1.0));

            // Find min/max projections along the gradient direction
            let mut min_proj = f64::MAX;
            let mut max_proj = f64::MIN;

            for pos in &mesh.positions {
                let p = Vec3::new(pos[0], pos[1], pos[2]);
                let proj = p.dot(dir);
                min_proj = min_proj.min(proj);
                max_proj = max_proj.max(proj);
            }

            let range = max_proj - min_proj;
            if range < f64::EPSILON {
                // All points on same plane perpendicular to gradient
                return Ok(vec![(*min_value + *max_value) * 0.5; vertex_count]);
            }

            let mut result = Vec::with_capacity(vertex_count);
            for pos in &mesh.positions {
                let p = Vec3::new(pos[0], pos[1], pos[2]);
                let proj = p.dot(dir);
                let t = (proj - min_proj) / range;
                let value = min_value + t * (max_value - min_value);
                result.push(value);
            }

            Ok(result)
        }

        DisplacementSource::Noise {
            frequency,
            amplitude,
            seed,
        } => {
            // Validate noise parameters
            if !frequency.is_finite() || !amplitude.is_finite() {
                return Err(DisplacementError::InvalidDisplacementValues);
            }

            let mut result = Vec::with_capacity(vertex_count);
            for pos in &mesh.positions {
                let noise_value = simple_3d_noise(
                    pos[0] * frequency,
                    pos[1] * frequency,
                    pos[2] * frequency,
                    *seed,
                );
                result.push(noise_value * amplitude);
            }
            Ok(result)
        }
    }
}

/// Simple deterministic 3D noise function (value noise with lattice interpolation).
/// Returns a value in the range [-1, 1].
fn simple_3d_noise(x: f64, y: f64, z: f64, seed: u32) -> f64 {
    // Integer lattice coordinates
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let zi = z.floor() as i32;

    // Fractional parts
    let xf = x - x.floor();
    let yf = y - y.floor();
    let zf = z - z.floor();

    // Smoothstep for interpolation
    let u = xf * xf * (3.0 - 2.0 * xf);
    let v = yf * yf * (3.0 - 2.0 * yf);
    let w = zf * zf * (3.0 - 2.0 * zf);

    // Hash function for lattice points - returns value in [-1, 1]
    // Uses large prime multipliers for good spatial distribution
    let hash = |x: i32, y: i32, z: i32| -> f64 {
        // Large primes chosen for good distribution: 374761393, 668265263, 1274126177
        // Use wrapping operations for deterministic overflow behavior
        let mut n = (x.wrapping_mul(374761393_i32))
            .wrapping_add(y.wrapping_mul(668265263_i32))
            .wrapping_add(z.wrapping_mul(1274126177_i32))
            .wrapping_add(seed as i32);
        // Bit mixing for better randomness
        n = n ^ (n >> 13);
        n = n.wrapping_mul(n.wrapping_mul(n.wrapping_mul(60493_i32).wrapping_add(19990303_i32)).wrapping_add(1376312589_i32));
        // Convert to unsigned for proper normalization, then map to [-1, 1]
        let normalized = (n as u32) as f64 / (u32::MAX as f64);
        normalized * 2.0 - 1.0
    };

    // 8 corner values (use wrapping_add to handle large coordinates safely)
    let n000 = hash(xi, yi, zi);
    let n100 = hash(xi.wrapping_add(1), yi, zi);
    let n010 = hash(xi, yi.wrapping_add(1), zi);
    let n110 = hash(xi.wrapping_add(1), yi.wrapping_add(1), zi);
    let n001 = hash(xi, yi, zi.wrapping_add(1));
    let n101 = hash(xi.wrapping_add(1), yi, zi.wrapping_add(1));
    let n011 = hash(xi, yi.wrapping_add(1), zi.wrapping_add(1));
    let n111 = hash(xi.wrapping_add(1), yi.wrapping_add(1), zi.wrapping_add(1));

    // Trilinear interpolation
    let n00 = n000 * (1.0 - u) + n100 * u;
    let n10 = n010 * (1.0 - u) + n110 * u;
    let n01 = n001 * (1.0 - u) + n101 * u;
    let n11 = n011 * (1.0 - u) + n111 * u;

    let n0 = n00 * (1.0 - v) + n10 * v;
    let n1 = n01 * (1.0 - v) + n11 * v;

    n0 * (1.0 - w) + n1 * w
}

/// Statistics from clamping operation.
struct ClampingStats {
    min_value: f64,
    max_value: f64,
    avg_value: f64,
    clamped_count: usize,
    warnings: Vec<String>,
}

/// Apply clamping to displacement values and collect statistics.
fn apply_clamping_and_stats(
    values: &[f64],
    options: &DisplacementOptions,
) -> (Vec<f64>, ClampingStats) {
    let mut result = values.to_vec();
    let mut clamped_count = 0;
    let mut warnings = Vec::new();

    // Apply clamping
    for v in &mut result {
        let original = *v;

        if let Some(min) = options.min_displacement {
            if *v < min {
                *v = min;
            }
        }
        if let Some(max) = options.max_displacement {
            if *v > max {
                *v = max;
            }
        }

        if (*v - original).abs() > f64::EPSILON {
            clamped_count += 1;
        }
    }

    // Collect statistics
    let mut min_value = f64::MAX;
    let mut max_value = f64::MIN;
    let mut sum = 0.0;

    for &v in &result {
        min_value = min_value.min(v);
        max_value = max_value.max(v);
        sum += v;
    }

    let is_empty = result.is_empty();
    let len = result.len();
    
    let avg_value = if is_empty {
        0.0
    } else {
        sum / len as f64
    };

    if clamped_count > 0 {
        warnings.push(format!(
            "{} vertices had displacement values clamped",
            clamped_count
        ));
    }

    let stats = ClampingStats {
        min_value: if is_empty { 0.0 } else { min_value },
        max_value: if is_empty { 0.0 } else { max_value },
        avg_value,
        clamped_count,
        warnings,
    };

    (result, stats)
}

/// Apply displacement to positions along normals.
fn apply_displacement(
    positions: &[[f64; 3]],
    normals: &[Vec3],
    displacement_values: &[f64],
) -> Vec<[f64; 3]> {
    positions
        .iter()
        .zip(normals.iter())
        .zip(displacement_values.iter())
        .map(|((pos, normal), &disp)| {
            [
                pos[0] + normal.x * disp,
                pos[1] + normal.y * disp,
                pos[2] + normal.z * disp,
            ]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_simple_quad_mesh() -> GeomMesh {
        // Simple quad as two triangles
        GeomMesh {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
            uvs: Some(vec![
                [0.0, 0.0],
                [1.0, 0.0],
                [1.0, 1.0],
                [0.0, 1.0],
            ]),
            normals: Some(vec![
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
            ]),
            tangents: None,
        }
    }

    #[test]
    fn test_uniform_displacement() {
        let mesh = make_simple_quad_mesh();
        let tol = Tolerance::default_geom();

        let result = displace_mesh_uniform(&mesh, 0.5, tol);
        assert!(result.is_ok());

        let (displaced, diag) = result.unwrap();

        // All vertices should be displaced by 0.5 in Z direction
        for pos in &displaced.positions {
            assert!((pos[2] - 0.5).abs() < 1e-6);
        }

        assert_eq!(diag.original_vertex_count, 4);
        assert!((diag.avg_displacement_applied - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_per_vertex_displacement() {
        let mesh = make_simple_quad_mesh();
        let tol = Tolerance::default_geom();

        let values = vec![0.0, 0.25, 0.5, 0.75];
        let result = displace_mesh_per_vertex(&mesh, values.clone(), tol);
        assert!(result.is_ok());

        let (displaced, _) = result.unwrap();

        // Check each vertex is displaced by the expected amount
        for (i, pos) in displaced.positions.iter().enumerate() {
            assert!((pos[2] - values[i]).abs() < 1e-6);
        }
    }

    #[test]
    fn test_per_vertex_count_mismatch() {
        let mesh = make_simple_quad_mesh();
        let tol = Tolerance::default_geom();

        // Wrong number of values
        let values = vec![0.0, 0.25, 0.5]; // 3 values for 4 vertices
        let result = displace_mesh_per_vertex(&mesh, values, tol);
        assert!(matches!(
            result,
            Err(DisplacementError::VertexCountMismatch { expected: 4, got: 3 })
        ));
    }

    #[test]
    fn test_heightfield_displacement() {
        let mesh = make_simple_quad_mesh();
        let tol = Tolerance::default_geom();

        // 2x2 heightfield
        let heightfield = vec![0.0, 0.5, 0.5, 1.0];
        let result = displace_mesh_heightfield(&mesh, heightfield, 2, 2, 1.0, tol);
        assert!(result.is_ok());

        let (displaced, diag) = result.unwrap();

        // Corners should match heightfield values (with bilinear interpolation at corners)
        assert!((displaced.positions[0][2] - 0.0).abs() < 1e-6); // UV (0,0) -> heightfield corner
        assert!((displaced.positions[2][2] - 1.0).abs() < 1e-6); // UV (1,1) -> heightfield corner

        assert!(diag.result_vertex_count > 0);
    }

    #[test]
    fn test_heightfield_missing_uvs() {
        let mut mesh = make_simple_quad_mesh();
        mesh.uvs = None;
        let tol = Tolerance::default_geom();

        let heightfield = vec![0.0, 0.5, 0.5, 1.0];
        let result = displace_mesh_heightfield(&mesh, heightfield, 2, 2, 1.0, tol);
        assert!(matches!(result, Err(DisplacementError::MissingUvs)));
    }

    #[test]
    fn test_gradient_displacement() {
        let mesh = make_simple_quad_mesh();
        let tol = Tolerance::default_geom();

        let options = DisplacementOptions::new(DisplacementSource::gradient(
            Vec3::new(1.0, 0.0, 0.0), // X-direction gradient
            0.0,
            1.0,
        ));

        let result = displace_mesh(&mesh, options, tol);
        assert!(result.is_ok());

        let (displaced, _) = result.unwrap();

        // Vertices at x=0 should have minimal displacement, x=1 should have maximal
        // Find vertices at x=0 and x=1
        for pos in &displaced.positions {
            // Z displacement should correlate with original X position
            // (Note: welding may merge vertices, so this is a basic check)
            assert!(pos[2].is_finite());
        }
    }

    #[test]
    fn test_noise_displacement_deterministic() {
        let mesh = make_simple_quad_mesh();
        let tol = Tolerance::default_geom();

        // Same seed should produce same result
        let result1 = displace_mesh_noise(&mesh, 1.0, 0.5, 42, tol).unwrap();
        let result2 = displace_mesh_noise(&mesh, 1.0, 0.5, 42, tol).unwrap();

        for (p1, p2) in result1.0.positions.iter().zip(result2.0.positions.iter()) {
            assert!((p1[0] - p2[0]).abs() < 1e-9);
            assert!((p1[1] - p2[1]).abs() < 1e-9);
            assert!((p1[2] - p2[2]).abs() < 1e-9);
        }
    }

    #[test]
    fn test_noise_displacement_different_seeds() {
        let mesh = make_simple_quad_mesh();
        let tol = Tolerance::default_geom();

        let result1 = displace_mesh_noise(&mesh, 1.0, 0.5, 42, tol).unwrap();
        let result2 = displace_mesh_noise(&mesh, 1.0, 0.5, 43, tol).unwrap();

        // At least some positions should differ
        let mut any_different = false;
        for (p1, p2) in result1.0.positions.iter().zip(result2.0.positions.iter()) {
            if (p1[2] - p2[2]).abs() > 1e-9 {
                any_different = true;
                break;
            }
        }
        assert!(any_different, "Different seeds should produce different results");
    }

    #[test]
    fn test_displacement_clamping() {
        let mesh = make_simple_quad_mesh();
        let tol = Tolerance::default_geom();

        let options = DisplacementOptions::new(DisplacementSource::uniform(2.0))
            .max_displacement(0.5);

        let result = displace_mesh(&mesh, options, tol);
        assert!(result.is_ok());

        let (displaced, diag) = result.unwrap();

        // All displacements should be clamped to 0.5
        for pos in &displaced.positions {
            assert!((pos[2] - 0.5).abs() < 1e-6);
        }

        assert_eq!(diag.clamped_vertex_count, 4);
    }

    #[test]
    fn test_empty_mesh() {
        let mesh = GeomMesh {
            positions: vec![],
            indices: vec![],
            uvs: None,
            normals: None,
            tangents: None,
        };
        let tol = Tolerance::default_geom();

        let result = displace_mesh_uniform(&mesh, 0.5, tol);
        assert!(matches!(result, Err(DisplacementError::EmptyMesh)));
    }

    #[test]
    fn test_invalid_geometry() {
        let mesh = GeomMesh {
            positions: vec![[f64::NAN, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            uvs: None,
            normals: None,
            tangents: None,
        };
        let tol = Tolerance::default_geom();

        let result = displace_mesh_uniform(&mesh, 0.5, tol);
        assert!(matches!(result, Err(DisplacementError::InvalidGeometry)));
    }

    #[test]
    fn test_simple_3d_noise_range() {
        // Noise should return values in [-1, 1]
        for i in 0..100 {
            let x = i as f64 * 0.1;
            let y = i as f64 * 0.13;
            let z = i as f64 * 0.17;
            let n = simple_3d_noise(x, y, z, 42);
            assert!(n >= -1.0 && n <= 1.0, "Noise value out of range: {}", n);
        }
    }

    #[test]
    fn test_normals_length_mismatch() {
        // Mesh with normals array that doesn't match positions array
        let mesh = GeomMesh {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
            ],
            indices: vec![0, 1, 2],
            uvs: None,
            normals: Some(vec![[0.0, 0.0, 1.0]]), // Only 1 normal for 3 positions
            tangents: None,
        };
        let tol = Tolerance::default_geom();

        let result = displace_mesh_uniform(&mesh, 0.5, tol);
        assert!(matches!(
            result,
            Err(DisplacementError::NormalsLengthMismatch {
                positions_len: 3,
                normals_len: 1
            })
        ));
    }

    #[test]
    fn test_heightfield_with_offset() {
        let mesh = make_simple_quad_mesh();
        let tol = Tolerance::default_geom();

        // Use heightfield with offset
        let options = DisplacementOptions::new(DisplacementSource::heightfield_with_offset(
            vec![0.0, 0.0, 0.0, 0.0], // All zeros
            2,
            2,
            1.0,
            0.5, // Offset of 0.5
        ));

        let result = displace_mesh(&mesh, options, tol);
        assert!(result.is_ok());

        let (displaced, _) = result.unwrap();
        // All vertices should be displaced by the offset (0.5)
        for pos in &displaced.positions {
            assert!((pos[2] - 0.5).abs() < 1e-6);
        }
    }

    #[test]
    fn test_gradient_inverted() {
        // Test gradient where min_value > max_value (inverted gradient)
        let mesh = make_simple_quad_mesh();
        let tol = Tolerance::default_geom();

        let options = DisplacementOptions::new(DisplacementSource::gradient(
            Vec3::new(1.0, 0.0, 0.0),
            1.0,  // min is larger
            0.0,  // max is smaller
        ));

        let result = displace_mesh(&mesh, options, tol);
        assert!(result.is_ok());

        let (_, diag) = result.unwrap();
        // Min and max should be correct (possibly swapped due to inversion)
        assert!(diag.min_displacement_applied <= diag.max_displacement_applied);
    }

    #[test]
    fn test_heightfield_nan_values() {
        let mesh = make_simple_quad_mesh();
        let tol = Tolerance::default_geom();

        // Heightfield with NaN value
        let heightfield = vec![0.0, f64::NAN, 0.5, 1.0];
        let result = displace_mesh_heightfield(&mesh, heightfield, 2, 2, 1.0, tol);
        assert!(matches!(
            result,
            Err(DisplacementError::InvalidDisplacementValues)
        ));
    }

    #[test]
    fn test_heightfield_inf_scale() {
        let mesh = make_simple_quad_mesh();
        let tol = Tolerance::default_geom();

        let options = DisplacementOptions::new(DisplacementSource::Heightfield {
            values: vec![0.0, 0.5, 0.5, 1.0],
            width: 2,
            height: 2,
            scale: f64::INFINITY,
            offset: 0.0,
        });

        let result = displace_mesh(&mesh, options, tol);
        assert!(matches!(
            result,
            Err(DisplacementError::InvalidDisplacementValues)
        ));
    }

    #[test]
    fn test_gradient_nan_direction() {
        let mesh = make_simple_quad_mesh();
        let tol = Tolerance::default_geom();

        let options = DisplacementOptions::new(DisplacementSource::gradient(
            Vec3::new(f64::NAN, 0.0, 0.0),
            0.0,
            1.0,
        ));

        let result = displace_mesh(&mesh, options, tol);
        assert!(matches!(
            result,
            Err(DisplacementError::InvalidDisplacementValues)
        ));
    }

    #[test]
    fn test_gradient_inf_values() {
        let mesh = make_simple_quad_mesh();
        let tol = Tolerance::default_geom();

        let options = DisplacementOptions::new(DisplacementSource::gradient(
            Vec3::new(1.0, 0.0, 0.0),
            f64::NEG_INFINITY,
            f64::INFINITY,
        ));

        let result = displace_mesh(&mesh, options, tol);
        assert!(matches!(
            result,
            Err(DisplacementError::InvalidDisplacementValues)
        ));
    }

    #[test]
    fn test_noise_nan_frequency() {
        let mesh = make_simple_quad_mesh();
        let tol = Tolerance::default_geom();

        let result = displace_mesh_noise(&mesh, f64::NAN, 0.5, 42, tol);
        assert!(matches!(
            result,
            Err(DisplacementError::InvalidDisplacementValues)
        ));
    }

    #[test]
    fn test_noise_inf_amplitude() {
        let mesh = make_simple_quad_mesh();
        let tol = Tolerance::default_geom();

        let result = displace_mesh_noise(&mesh, 1.0, f64::INFINITY, 42, tol);
        assert!(matches!(
            result,
            Err(DisplacementError::InvalidDisplacementValues)
        ));
    }

    #[test]
    fn test_noise_large_coordinates() {
        // Test noise with large coordinates (potential float precision issues)
        // Use moderately large values that exercise the noise function without
        // triggering overflow in mesh finalization's spatial hashing
        let mesh = GeomMesh {
            positions: vec![
                [1e6, 1e6, 1e6],
                [1e6 + 1.0, 1e6, 1e6],
                [1e6 + 0.5, 1e6 + 1.0, 1e6],
            ],
            indices: vec![0, 1, 2],
            uvs: None,
            normals: Some(vec![
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
            ]),
            tangents: None,
        };
        let tol = Tolerance::default_geom();

        // Disable welding to avoid mesh finalization overflow with large coords
        let options = DisplacementOptions::new(DisplacementSource::noise(1.0, 0.5, 42))
            .weld_vertices(false)
            .recompute_normals(false);

        let result = displace_mesh(&mesh, options, tol);
        assert!(result.is_ok());

        let (displaced, _) = result.unwrap();
        // All positions should be finite
        for pos in &displaced.positions {
            assert!(pos[0].is_finite());
            assert!(pos[1].is_finite());
            assert!(pos[2].is_finite());
        }
    }

    #[test]
    fn test_noise_very_large_lattice_coordinates() {
        // Directly test the noise function with very large coordinates
        // to ensure wrapping_add prevents overflow
        let large = 1e15_f64;
        let n1 = simple_3d_noise(large, large, large, 42);
        let n2 = simple_3d_noise(-large, -large, -large, 42);
        
        // Both should return valid values in [-1, 1]
        assert!(n1 >= -1.0 && n1 <= 1.0, "Noise value out of range for large coords: {}", n1);
        assert!(n2 >= -1.0 && n2 <= 1.0, "Noise value out of range for negative large coords: {}", n2);
    }
}
