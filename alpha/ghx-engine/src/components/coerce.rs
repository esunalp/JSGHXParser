//! Hulpfuncties voor het converteren van `Value`-types.
//!
//! This module provides coercion utilities for converting between `Value` types
//! and more specific Rust types used by component implementations.
//!
//! # Mesh and Surface Interoperability
//!
//! The module supports both the legacy `Value::Surface` and the new `Value::Mesh` types.
//! Components should prefer the `*_like` functions (`coerce_mesh_like`, `coerce_surface_like`)
//! to accept both types transparently:
//!
//! ```ignore
//! // Preferred: accepts both Mesh and Surface
//! let mesh = coerce_mesh_like(&inputs[0])?;
//!
//! // Legacy: only accepts Surface (use for backward compatibility)
//! let surface = coerce_surface(&inputs[0])?;
//! ```

use super::ComponentError;
use crate::graph::value::{Domain, Domain1D, MeshData, MeshDiagnostics, PlaneValue, Value};
use time::{Date, Month, PrimitiveDateTime, Time};

// ============================================================================
// Surface types - for backward compatibility with Value::Surface
// ============================================================================

/// Borrowed reference to legacy surface data.
///
/// This struct provides a borrowed view over `Value::Surface` data.
/// For accepting both `Value::Mesh` and `Value::Surface`, use [`SurfaceOwned`]
/// via the [`coerce_surface_like`] function.
pub struct Surface<'a> {
    pub vertices: &'a Vec<[f64; 3]>,
    pub faces: &'a Vec<Vec<u32>>,
}

/// Owned surface data, compatible with both `Value::Mesh` and `Value::Surface`.
///
/// This struct provides a unified owned representation for surface-like values.
/// When constructed from a `Value::Mesh`, the triangle indices are converted
/// to polygon face lists.
///
/// # Example
///
/// ```ignore
/// let surface = coerce_surface_like(&inputs[0])?;
/// for face in &surface.faces {
///     // Each face is a list of vertex indices
///     println!("Face with {} vertices", face.len());
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SurfaceOwned {
    /// Vertex positions as `[x, y, z]` arrays.
    pub vertices: Vec<[f64; 3]>,
    /// Polygon faces as lists of vertex indices.
    pub faces: Vec<Vec<u32>>,
}

impl SurfaceOwned {
    /// Creates a new `SurfaceOwned` with the given vertices and faces.
    #[must_use]
    pub fn new(vertices: Vec<[f64; 3]>, faces: Vec<Vec<u32>>) -> Self {
        Self { vertices, faces }
    }

    /// Returns the number of vertices.
    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the number of faces.
    #[must_use]
    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    /// Converts to a `Value::Surface`.
    #[must_use]
    pub fn into_value(self) -> Value {
        Value::Surface {
            vertices: self.vertices,
            faces: self.faces,
        }
    }

    /// Creates a borrowed `Surface` reference from this owned data.
    #[must_use]
    pub fn as_surface(&self) -> Surface<'_> {
        Surface {
            vertices: &self.vertices,
            faces: &self.faces,
        }
    }
}

// ============================================================================
// Polygon Triangulation - re-export from graph::value
// ============================================================================

// Use the centralized triangulation function from graph::value
use crate::graph::value::triangulate_polygon_faces;

// ============================================================================
// Mesh types - the preferred representation for mesh-like values
// ============================================================================

/// Owned mesh data, compatible with both `Value::Mesh` and `Value::Surface`.
///
/// This struct provides a unified view over mesh-like values. When constructed from
/// a `Value::Surface`, the polygon faces are converted to triangle indices.
///
/// This is the preferred type for components working with mesh data, as it
/// accepts both the new `Value::Mesh` type and the legacy `Value::Surface` type.
///
/// # Example
///
/// ```ignore
/// let mesh = coerce_mesh_like(&inputs[0])?;
/// for i in (0..mesh.indices.len()).step_by(3) {
///     let v0 = mesh.vertices[mesh.indices[i] as usize];
///     let v1 = mesh.vertices[mesh.indices[i + 1] as usize];
///     let v2 = mesh.vertices[mesh.indices[i + 2] as usize];
///     // Process triangle...
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Mesh {
    /// Vertex positions as `[x, y, z]` arrays.
    pub vertices: Vec<[f64; 3]>,
    /// Triangle indices (length divisible by 3).
    pub indices: Vec<u32>,
    /// Optional per-vertex normals.
    pub normals: Option<Vec<[f64; 3]>>,
    /// Optional per-vertex UV coordinates.
    pub uvs: Option<Vec<[f64; 2]>>,
}

impl Mesh {
    /// Creates a new mesh with positions and indices only.
    #[must_use]
    pub fn new(vertices: Vec<[f64; 3]>, indices: Vec<u32>) -> Self {
        Self {
            vertices,
            indices,
            normals: None,
            uvs: None,
        }
    }

    /// Creates a new mesh with all attributes.
    #[must_use]
    pub fn with_attributes(
        vertices: Vec<[f64; 3]>,
        indices: Vec<u32>,
        normals: Option<Vec<[f64; 3]>>,
        uvs: Option<Vec<[f64; 2]>>,
    ) -> Self {
        Self {
            vertices,
            indices,
            normals,
            uvs,
        }
    }

    /// Returns the number of vertices in the mesh.
    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the number of triangles in the mesh.
    #[must_use]
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Returns `true` if the mesh has per-vertex normals.
    #[must_use]
    pub fn has_normals(&self) -> bool {
        self.normals.is_some()
    }

    /// Returns `true` if the mesh has UV coordinates.
    #[must_use]
    pub fn has_uvs(&self) -> bool {
        self.uvs.is_some()
    }

    /// Returns `true` if the mesh is empty (no vertices or triangles).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty() || self.indices.is_empty()
    }

    /// Validates the mesh for consistency.
    ///
    /// Returns `Ok(())` if valid, or an error message describing the issue.
    pub fn validate(&self) -> Result<(), String> {
        // Check triangle indices length
        if self.indices.len() % 3 != 0 {
            return Err(format!(
                "indices length {} is not divisible by 3",
                self.indices.len()
            ));
        }

        // Check index bounds
        let n = self.vertices.len() as u32;
        for (i, &idx) in self.indices.iter().enumerate() {
            if idx >= n {
                return Err(format!(
                    "index {} at position {} is out of bounds (vertex count: {})",
                    idx, i, n
                ));
            }
        }

        // Check normals length if present
        if let Some(ref normals) = self.normals {
            if normals.len() != self.vertices.len() {
                return Err(format!(
                    "normals length {} does not match vertices length {}",
                    normals.len(),
                    self.vertices.len()
                ));
            }
        }

        // Check UVs length if present
        if let Some(ref uvs) = self.uvs {
            if uvs.len() != self.vertices.len() {
                return Err(format!(
                    "uvs length {} does not match vertices length {}",
                    uvs.len(),
                    self.vertices.len()
                ));
            }
        }

        // Check for NaN/Inf in vertices
        for (i, v) in self.vertices.iter().enumerate() {
            if !v[0].is_finite() || !v[1].is_finite() || !v[2].is_finite() {
                return Err(format!("vertex {} contains NaN or Inf: {:?}", i, v));
            }
        }

        Ok(())
    }

    /// Converts to `MeshData` for use with graph layer.
    #[must_use]
    pub fn into_mesh_data(self) -> MeshData {
        MeshData {
            vertices: self.vertices,
            indices: self.indices,
            normals: self.normals,
            uvs: self.uvs,
            diagnostics: None,
        }
    }

    /// Converts to `MeshData` with diagnostics.
    #[must_use]
    pub fn into_mesh_data_with_diagnostics(self, diagnostics: MeshDiagnostics) -> MeshData {
        MeshData {
            vertices: self.vertices,
            indices: self.indices,
            normals: self.normals,
            uvs: self.uvs,
            diagnostics: Some(diagnostics),
        }
    }

    /// Converts to a `Value::Mesh`.
    #[must_use]
    pub fn into_value(self) -> Value {
        Value::Mesh {
            vertices: self.vertices,
            indices: self.indices,
            normals: self.normals,
            uvs: self.uvs,
            diagnostics: None,
        }
    }

    /// Converts to a `Value::Mesh` with diagnostics.
    #[must_use]
    pub fn into_value_with_diagnostics(self, diagnostics: MeshDiagnostics) -> Value {
        Value::Mesh {
            vertices: self.vertices,
            indices: self.indices,
            normals: self.normals,
            uvs: self.uvs,
            diagnostics: Some(diagnostics),
        }
    }

    /// Converts to a legacy `Value::Surface`.
    ///
    /// **Note**: This is a lossy conversion - normals and UVs are discarded.
    #[must_use]
    pub fn into_surface_legacy(self) -> Value {
        let faces: Vec<Vec<u32>> = self
            .indices
            .chunks(3)
            .filter(|chunk| chunk.len() == 3)
            .map(|chunk| vec![chunk[0], chunk[1], chunk[2]])
            .collect();
        Value::Surface {
            vertices: self.vertices,
            faces,
        }
    }

    /// Converts to a `SurfaceOwned`.
    ///
    /// **Note**: Normals and UVs are discarded in this conversion.
    #[must_use]
    pub fn into_surface_owned(self) -> SurfaceOwned {
        let faces: Vec<Vec<u32>> = self
            .indices
            .chunks(3)
            .filter(|chunk| chunk.len() == 3)
            .map(|chunk| vec![chunk[0], chunk[1], chunk[2]])
            .collect();
        SurfaceOwned {
            vertices: self.vertices,
            faces,
        }
    }

    /// Creates a mesh from `MeshData`.
    #[must_use]
    pub fn from_mesh_data(data: MeshData) -> Self {
        Self {
            vertices: data.vertices,
            indices: data.indices,
            normals: data.normals,
            uvs: data.uvs,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub start: [f64; 3],
    pub end: [f64; 3],
}

impl Line {
    #[must_use]
    pub fn direction(self) -> [f64; 3] {
        subtract(self.end, self.start)
    }
}

/// Een genormaliseerd vlak dat gebruikt wordt bij sommige componenten.
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub origin: [f64; 3],
    pub x_axis: [f64; 3],
    pub y_axis: [f64; 3],
    pub z_axis: [f64; 3],
}

impl Default for Plane {
    fn default() -> Self {
        Self {
            origin: [0.0, 0.0, 0.0],
            x_axis: [1.0, 0.0, 0.0],
            y_axis: [0.0, 1.0, 0.0],
            z_axis: [0.0, 0.0, 1.0],
        }
    }
}

pub fn create_domain(start: f64, end: f64) -> Option<Domain1D> {
    if !start.is_finite() || !end.is_finite() {
        return None;
    }
    let min = start.min(end);
    let max = start.max(end);
    let span = end - start;
    let length = max - min;
    let center = (start + end) / 2.0;
    Some(Domain1D {
        start,
        end,
        min,
        max,
        span,
        length,
        center,
    })
}

pub fn parse_domain1d(value: &Value) -> Option<Domain1D> {
    match value {
        Value::Domain(Domain::One(domain)) => Some(domain.clone()),
        Value::Domain(Domain::Two(_)) => None,
        Value::Number(number) => create_domain(*number, *number),
        Value::List(values) => {
            if values.len() >= 2 {
                let start = coerce_number(values.get(0)?, None).ok();
                let end = coerce_number(values.get(1)?, None).ok();
                match (start, end) {
                    (Some(start), Some(end)) => create_domain(start, end),
                    _ => None,
                }
            } else if values.len() == 1 {
                coerce_domain1d(values.get(0))
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn coerce_domain1d(value: Option<&Value>) -> Option<Domain1D> {
    value.and_then(parse_domain1d)
}

pub fn coerce_number(value: &Value, context: Option<&str>) -> Result<f64, ComponentError> {
    match value {
        Value::Number(number) => {
            if let Some(ctx) = context {
                if !number.is_finite() {
                    return Err(ComponentError::new(format!(
                        "{} verwacht een eindig getal",
                        ctx
                    )));
                }
            }
            Ok(*number)
        }
        Value::Boolean(boolean) => Ok(if *boolean { 1.0 } else { 0.0 }),
        Value::Text(text) => match parse_boolean_text(text.as_str()) {
            Some(boolean) => Ok(if boolean { 1.0 } else { 0.0 }),
            None => {
                if let Some(ctx) = context {
                    Err(ComponentError::new(format!(
                        "{} verwacht een numerieke waarde, kreeg tekst '{}'",
                        ctx, text
                    )))
                } else {
                    text.parse().map_err(|_| {
                        ComponentError::new(format!(
                            "Kon tekst '{}' niet naar een getal converteren",
                            text
                        ))
                    })
                }
            }
        },
        Value::List(l) if l.len() == 1 => coerce_number(&l[0], context),
        // Value::Null indicates an unconnected input; for required inputs this is an error.
        // For optional inputs, callers should use coerce_optional_number_with_default instead.
        Value::Null => {
            if let Some(ctx) = context {
                Err(ComponentError::new(format!(
                    "{} is niet aangesloten (ontbrekende waarde)",
                    ctx
                )))
            } else {
                Err(ComponentError::new(
                    "Input is niet aangesloten (ontbrekende waarde)".to_string()
                ))
            }
        }
        other => {
            if let Some(ctx) = context {
                Err(ComponentError::new(format!(
                    "{} verwacht een numerieke waarde, kreeg {}",
                    ctx,
                    other.kind()
                )))
            } else {
                Err(ComponentError::new(format!(
                    "Verwachtte een getal, kreeg {}",
                    other.kind()
                )))
            }
        }
    }
}

pub fn coerce_count(
    value: Option<&Value>,
    fallback: usize,
    context: &str,
) -> Result<usize, ComponentError> {
    match value {
        None => Ok(fallback),
        Some(entry) => {
            let number = coerce_number(entry, Some(context))?;
            if !number.is_finite() {
                return Ok(fallback);
            }
            let floored = number.floor();
            if floored < 1.0 {
                Ok(1)
            } else {
                Ok(floored as usize)
            }
        }
    }
}

pub fn coerce_boolean_with_context(value: &Value, context: &str) -> Result<bool, ComponentError> {
    match value {
        Value::Boolean(value) => Ok(*value),
        Value::Number(number) => {
            if number.is_nan() {
                Err(ComponentError::new(format!(
                    "{} verwacht een booleaanse waarde, kreeg NaN",
                    context
                )))
            } else {
                Ok(*number != 0.0)
            }
        }
        Value::Text(text) => parse_boolean_text(text).ok_or_else(|| {
            ComponentError::new(format!(
                "{} verwacht een booleaanse waarde, kreeg tekst '{}'",
                context, text
            ))
        }),
        Value::List(values) if values.len() == 1 => {
            coerce_boolean_with_context(&values[0], context)
        }
        // Value::Null indicates an unconnected input; for required inputs this is an error.
        // For optional inputs, callers should use coerce_boolean_with_default instead.
        Value::Null => Err(ComponentError::new(format!(
            "{} is niet aangesloten (ontbrekende waarde)",
            context
        ))),
        other => Err(ComponentError::new(format!(
            "{} verwacht een booleaanse waarde, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

pub fn coerce_optional_number(
    value: Option<&Value>,
    context: &str,
) -> Result<Option<f64>, ComponentError> {
    match value {
        Some(Value::Null) => Ok(None),
        Some(value) => coerce_number(value, Some(context)).map(Some),
        None => Ok(None),
    }
}

/// Coerces an optional value to a number, returning a default when the value is
/// `None` or `Value::Null` (unconnected pin). This is the preferred function for
/// optional numeric inputs with sensible defaults.
///
/// # Arguments
/// * `value` - The optional value to coerce
/// * `default` - The default value to use when unset
/// * `context` - Context string for error messages
///
/// # Returns
/// * `Ok(number)` - The coerced number or default
/// * `Err(ComponentError)` - If the value is present but cannot be coerced to a number
pub fn coerce_optional_number_with_default(
    value: Option<&Value>,
    default: f64,
    context: &str,
) -> Result<f64, ComponentError> {
    match value {
        Some(Value::Null) | None => Ok(default),
        Some(value) => coerce_number(value, Some(context)),
    }
}

/// Coerces an optional value to a boolean, returning a default when the value is
/// `None` or `Value::Null` (unconnected pin). This is the preferred function for
/// optional boolean inputs with sensible defaults.
///
/// # Arguments
/// * `value` - The optional value to coerce
/// * `default` - The default value to use when unset
/// * `context` - Context string for error messages
///
/// # Returns
/// * `Ok(boolean)` - The coerced boolean or default
/// * `Err(ComponentError)` - If the value is present but cannot be coerced to a boolean
pub fn coerce_optional_boolean_with_default(
    value: Option<&Value>,
    default: bool,
    context: &str,
) -> Result<bool, ComponentError> {
    match value {
        Some(Value::Null) | None => Ok(default),
        Some(value) => coerce_boolean_with_context(value, context),
    }
}

pub fn to_optional_number(value: Option<&Value>) -> Result<Option<f64>, ComponentError> {
    match value {
        Some(Value::Null) | None => Ok(None),
        Some(Value::Number(number)) if number.is_finite() => Ok(Some(*number)),
        Some(Value::Boolean(boolean)) => Ok(Some(if *boolean { 1.0 } else { 0.0 })),
        Some(Value::List(values)) if values.len() == 1 => to_optional_number(values.first()),
        Some(_) => Ok(None),
    }
}

pub fn coerce_text(value: &Value) -> Result<String, ComponentError> {
    match value {
        Value::Text(s) => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Boolean(b) => Ok(b.to_string()),
        Value::List(l) if l.len() == 1 => coerce_text(&l[0]),
        other => Err(ComponentError::new(format!(
            "Verwachtte een tekst, kreeg {}",
            other.kind()
        ))),
    }
}

pub fn coerce_integer(value: &Value) -> Result<i64, ComponentError> {
    match value {
        Value::Number(n) => Ok(n.round() as i64),
        Value::Boolean(b) => Ok(if *b { 1 } else { 0 }),
        Value::Text(s) => match parse_boolean_text(s.as_str()) {
            Some(boolean) => Ok(if boolean { 1 } else { 0 }),
            None => s.parse::<f64>().map(|n| n.round() as i64).map_err(|_| {
                ComponentError::new(format!(
                    "Kon tekst '{}' niet naar een geheel getal converteren",
                    s
                ))
            }),
        },
        Value::List(l) if l.len() == 1 => coerce_integer(&l[0]),
        other => Err(ComponentError::new(format!(
            "Verwachtte een geheel getal, kreeg {}",
            other.kind()
        ))),
    }
}

pub fn coerce_boolean(value: &Value) -> Result<bool, ComponentError> {
    match value {
        Value::Boolean(b) => Ok(*b),
        Value::Number(n) => Ok(n.abs() > 1e-9),
        Value::Text(s) => parse_boolean_text(s.as_str()).ok_or_else(|| {
            ComponentError::new(format!(
                "Kon tekst '{}' niet naar een booleaanse waarde converteren",
                s
            ))
        }),
        Value::List(l) if l.len() == 1 => coerce_boolean(&l[0]),
        // Value::Null indicates an unconnected input; for required inputs this is an error.
        // For optional inputs, callers should use coerce_boolean_with_default instead.
        Value::Null => Err(ComponentError::new(
            "Input is niet aangesloten (ontbrekende booleaanse waarde)".to_string()
        )),
        other => Err(ComponentError::new(format!(
            "Verwachtte een booleaanse waarde, kreeg {}",
            other.kind()
        ))),
    }
}

pub fn coerce_point(value: &Value) -> Result<[f64; 3], ComponentError> {
    match value {
        Value::Point(p) => Ok(*p),
        Value::List(l) if l.len() == 1 => coerce_point(&l[0]),
        other => Err(ComponentError::new(format!(
            "Verwachtte een punt, kreeg {}",
            other.kind()
        ))),
    }
}

// ============================================================================
// Surface coercion functions
// ============================================================================

/// Coerces a `Value::Surface` to a borrowed `Surface` struct.
///
/// This function only accepts `Value::Surface` values (the legacy mesh format).
/// For accepting both `Value::Mesh` and `Value::Surface`, use [`coerce_surface_like`].
///
/// # Example
///
/// ```ignore
/// let surface = coerce_surface(&inputs[0])?;
/// for face in surface.faces {
///     // Each face is a polygon as a list of vertex indices
/// }
/// ```
///
/// # Errors
///
/// Returns an error if the value is not a `Value::Surface`.
pub fn coerce_surface<'a>(value: &'a Value) -> Result<Surface<'a>, ComponentError> {
    match value {
        Value::Surface { vertices, faces } => Ok(Surface { vertices, faces }),
        Value::List(l) if l.len() == 1 => coerce_surface(&l[0]),
        other => Err(ComponentError::new(format!(
            "Verwachtte een surface, kreeg {}",
            other.kind()
        ))),
    }
}

/// Coerces a surface-like value (`Value::Surface` or `Value::Mesh`) to a `SurfaceOwned` struct.
///
/// This is the preferred function for components that work with surface/polygon data
/// and should accept both the legacy `Value::Surface` and the new `Value::Mesh` type.
///
/// For `Value::Mesh`, triangle indices are converted to polygon face lists
/// (each triangle becomes a 3-element face list).
///
/// # Example
///
/// ```ignore
/// let surface = coerce_surface_like(&inputs[0])?;
/// for face in &surface.faces {
///     // Each face is a polygon as a list of vertex indices
///     println!("Face with {} vertices", face.len());
/// }
/// ```
///
/// # Errors
///
/// Returns an error if the value is not a `Value::Surface` or `Value::Mesh`.
pub fn coerce_surface_like(value: &Value) -> Result<SurfaceOwned, ComponentError> {
    match value {
        Value::Surface { vertices, faces } => Ok(SurfaceOwned {
            vertices: vertices.clone(),
            faces: faces.clone(),
        }),
        Value::Mesh {
            vertices, indices, ..
        } => {
            // Convert triangle indices to polygon faces
            let faces: Vec<Vec<u32>> = indices
                .chunks(3)
                .filter(|chunk| chunk.len() == 3)
                .map(|chunk| vec![chunk[0], chunk[1], chunk[2]])
                .collect();
            Ok(SurfaceOwned {
                vertices: vertices.clone(),
                faces,
            })
        }
        Value::List(l) if l.len() == 1 => coerce_surface_like(&l[0]),
        other => Err(ComponentError::new(format!(
            "Verwachtte een surface of mesh, kreeg {}",
            other.kind()
        ))),
    }
}

/// Coerces a surface-like value with a context message for error reporting.
///
/// Similar to [`coerce_surface_like`] but includes a context string in error messages.
///
/// # Example
///
/// ```ignore
/// let surface = coerce_surface_like_with_context(&inputs[0], "Extrude base")?;
/// ```
pub fn coerce_surface_like_with_context(
    value: &Value,
    context: &str,
) -> Result<SurfaceOwned, ComponentError> {
    match value {
        Value::Surface { vertices, faces } => Ok(SurfaceOwned {
            vertices: vertices.clone(),
            faces: faces.clone(),
        }),
        Value::Mesh {
            vertices, indices, ..
        } => {
            let faces: Vec<Vec<u32>> = indices
                .chunks(3)
                .filter(|chunk| chunk.len() == 3)
                .map(|chunk| vec![chunk[0], chunk[1], chunk[2]])
                .collect();
            Ok(SurfaceOwned {
                vertices: vertices.clone(),
                faces,
            })
        }
        Value::List(l) if l.len() == 1 => coerce_surface_like_with_context(&l[0], context),
        other => Err(ComponentError::new(format!(
            "{} verwacht een surface of mesh, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

/// Coerces a list of surface-like values.
///
/// Accepts a `Value::List` containing surface-like values and returns a vector
/// of `SurfaceOwned`. Also accepts a single surface-like value and wraps it.
///
/// # Errors
///
/// Returns an error if any element in the list is not a surface-like value.
pub fn coerce_surface_list(value: &Value, context: &str) -> Result<Vec<SurfaceOwned>, ComponentError> {
    match value {
        Value::List(values) => {
            let mut result = Vec::with_capacity(values.len());
            for (i, entry) in values.iter().enumerate() {
                let surface = coerce_surface_like_with_context(
                    entry,
                    &format!("{} item {}", context, i),
                )?;
                result.push(surface);
            }
            Ok(result)
        }
        other => Ok(vec![coerce_surface_like_with_context(other, context)?]),
    }
}

// ============================================================================
// Mesh coercion functions
// ============================================================================

/// Coerces a `Value::Mesh` to a `Mesh` struct.
///
/// This function only accepts `Value::Mesh` values. For accepting both
/// `Value::Mesh` and `Value::Surface`, use `coerce_mesh_like`.
///
/// # Errors
///
/// Returns an error if the value is not a `Value::Mesh`.
pub fn coerce_mesh(value: &Value) -> Result<Mesh, ComponentError> {
    match value {
        Value::Mesh {
            vertices,
            indices,
            normals,
            uvs,
            ..
        } => Ok(Mesh {
            vertices: vertices.clone(),
            indices: indices.clone(),
            normals: normals.clone(),
            uvs: uvs.clone(),
        }),
        Value::List(l) if l.len() == 1 => coerce_mesh(&l[0]),
        other => Err(ComponentError::new(format!(
            "Verwachtte een mesh, kreeg {}",
            other.kind()
        ))),
    }
}

/// Coerces a mesh-like value (`Value::Mesh` or `Value::Surface`) to a `Mesh` struct.
///
/// This is the preferred function for components that need to work with mesh data
/// and should accept both the new `Value::Mesh` type and the legacy `Value::Surface` type.
///
/// For `Value::Surface`, faces are converted to triangle indices by taking
/// the first three vertices of each face.
///
/// # Example
///
/// ```ignore
/// let mesh = coerce_mesh_like(&inputs[0])?;
/// for i in (0..mesh.indices.len()).step_by(3) {
///     let v0 = mesh.vertices[mesh.indices[i] as usize];
///     // ... process triangle
/// }
/// ```
///
/// # Errors
///
/// Returns an error if the value is not a `Value::Mesh` or `Value::Surface`.
pub fn coerce_mesh_like(value: &Value) -> Result<Mesh, ComponentError> {
    match value {
        Value::Mesh {
            vertices,
            indices,
            normals,
            uvs,
            ..
        } => Ok(Mesh {
            vertices: vertices.clone(),
            indices: indices.clone(),
            normals: normals.clone(),
            uvs: uvs.clone(),
        }),
        Value::Surface { vertices, faces } => {
            // Convert polygon faces to triangles using fan triangulation.
            // This properly handles quads and n-gons by producing (n-2) triangles
            // per n-gon face, preserving all geometry.
            let indices = triangulate_polygon_faces(faces);
            Ok(Mesh {
                vertices: vertices.clone(),
                indices,
                normals: None,
                uvs: None,
            })
        }
        Value::List(l) if l.len() == 1 => coerce_mesh_like(&l[0]),
        other => Err(ComponentError::new(format!(
            "Verwachtte een mesh of surface, kreeg {}",
            other.kind()
        ))),
    }
}

/// Coerces a mesh-like value with a context message for error reporting.
///
/// Similar to `coerce_mesh_like` but includes a context string in error messages.
pub fn coerce_mesh_like_with_context(value: &Value, context: &str) -> Result<Mesh, ComponentError> {
    match value {
        Value::Mesh {
            vertices,
            indices,
            normals,
            uvs,
            ..
        } => Ok(Mesh {
            vertices: vertices.clone(),
            indices: indices.clone(),
            normals: normals.clone(),
            uvs: uvs.clone(),
        }),
        Value::Surface { vertices, faces } => {
            // Convert polygon faces to triangles using fan triangulation.
            // This properly handles quads and n-gons by producing (n-2) triangles
            // per n-gon face, preserving all geometry.
            let indices = triangulate_polygon_faces(faces);
            Ok(Mesh {
                vertices: vertices.clone(),
                indices,
                normals: None,
                uvs: None,
            })
        }
        Value::List(l) if l.len() == 1 => coerce_mesh_like_with_context(&l[0], context),
        other => Err(ComponentError::new(format!(
            "{} verwacht een mesh of surface, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

/// Coerces a list of mesh-like values.
///
/// Accepts a `Value::List` containing mesh-like values and returns a vector of `Mesh`.
/// Also accepts a single mesh-like value and wraps it in a vector.
///
/// # Errors
///
/// Returns an error if any element in the list is not a mesh-like value.
pub fn coerce_mesh_list(value: &Value, context: &str) -> Result<Vec<Mesh>, ComponentError> {
    match value {
        Value::List(values) => {
            let mut result = Vec::with_capacity(values.len());
            for (i, entry) in values.iter().enumerate() {
                let mesh = coerce_mesh_like_with_context(
                    entry,
                    &format!("{} item {}", context, i),
                )?;
                result.push(mesh);
            }
            Ok(result)
        }
        other => Ok(vec![coerce_mesh_like_with_context(other, context)?]),
    }
}

pub fn coerce_curve_segments(value: &Value) -> Result<Vec<([f64; 3], [f64; 3])>, ComponentError> {
    match value {
        Value::Null => Ok(Vec::new()),
        Value::CurveLine { p1, p2 } => Ok(vec![(*p1, *p2)]),
        Value::List(values) => {
            let mut segments = Vec::new();
            let mut last_point: Option<[f64; 3]> = None;

            for entry in values {
                if let Value::Point(p) = entry {
                    if let Some(last) = last_point {
                        segments.push((last, *p));
                    }
                    last_point = Some(*p);
                } else {
                    let sub_segments = coerce_curve_segments(entry)?;
                    if !sub_segments.is_empty() {
                        // Als we een polyline aan het bouwen waren, is de keten nu onderbroken.
                        // We voegen de segmenten van de sub-item toe.
                        segments.extend(sub_segments.clone());
                        // Het "laatste punt" is nu het einde van het laatste segment van de sub-item.
                        last_point = sub_segments.last().map(|s| s.1);
                    } else {
                        // De entry leverde geen segmenten op (bijv. Value::Null),
                        // dus de keten wordt onderbroken.
                        last_point = None;
                    }
                }
            }
            Ok(segments)
        }
        Value::Surface { vertices, .. } => {
            if vertices.len() < 2 {
                return Ok(Vec::new());
            }
            let mut segments = Vec::new();
            for pair in vertices.windows(2) {
                segments.push((pair[0], pair[1]));
            }
            Ok(segments)
        }
        _ => Err(ComponentError::new(format!(
            "Verwachtte een curve-achtige invoer, kreeg {}",
            value.kind()
        ))),
    }
}

pub fn coerce_vector(value: &Value, context: &str) -> Result<[f64; 3], ComponentError> {
    match value {
        Value::Vector(vector) => Ok(*vector),
        Value::Point(point) => Ok(*point),
        Value::List(values) if values.len() == 1 => coerce_vector(&values[0], context),
        Value::List(values) if values.len() >= 3 => {
            let x = coerce_number(values.get(0).unwrap(), Some(context))?;
            let y = coerce_number(values.get(1).unwrap(), Some(context))?;
            let z = coerce_number(values.get(2).unwrap(), Some(context))?;
            Ok([x, y, z])
        }
        Value::List(values) if values.len() == 2 => {
            let x = coerce_number(values.get(0).unwrap(), Some(context))?;
            let y = coerce_number(values.get(1).unwrap(), Some(context))?;
            Ok([x, y, 0.0])
        }
        Value::Number(number) => Ok([0.0, 0.0, *number]),
        other => Err(ComponentError::new(format!(
            "{} verwacht een vector, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

pub fn coerce_point_with_context(value: &Value, context: &str) -> Result<[f64; 3], ComponentError> {
    match value {
        Value::Point(point) => Ok(*point),
        Value::Vector(vector) => Ok(*vector),
        Value::List(values) if values.len() == 1 => coerce_point_with_context(&values[0], context),
        Value::List(values) if values.len() >= 3 => {
            let x = coerce_number(values.get(0).unwrap(), Some(context))?;
            let y = coerce_number(values.get(1).unwrap(), Some(context))?;
            let z = coerce_number(values.get(2).unwrap(), Some(context))?;
            Ok([x, y, z])
        }
        other => Err(ComponentError::new(format!(
            "{} verwacht een punt, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

pub fn coerce_vector_list(value: &Value, context: &str) -> Result<Vec<[f64; 3]>, ComponentError> {
    match value {
        Value::List(values) => {
            let mut result = Vec::new();
            for entry in values {
                match coerce_vector(entry, context) {
                    Ok(vector) => result.push(vector),
                    Err(_) => {
                        if let Value::List(nested) = entry {
                            if let Ok(vector) = coerce_vector(&Value::List(nested.clone()), context)
                            {
                                result.push(vector);
                                continue;
                            }
                        }
                        return Err(ComponentError::new(format!(
                            "{} verwacht een lijst van vectoren",
                            context
                        )));
                    }
                }
            }
            Ok(result)
        }
        other => Ok(vec![coerce_vector(other, context)?]),
    }
}

impl Plane {
    fn normalize_axes(
        origin: [f64; 3],
        x_axis: [f64; 3],
        y_axis: [f64; 3],
        z_axis: [f64; 3],
    ) -> Self {
        let z = safe_normalized(z_axis)
            .map(|(vector, _)| vector)
            .unwrap_or([0.0, 0.0, 1.0]);

        let mut x = safe_normalized(x_axis)
            .map(|(vector, _)| vector)
            .unwrap_or_else(|| orthogonal_vector(z));

        let mut y = safe_normalized(y_axis)
            .map(|(vector, _)| vector)
            .unwrap_or_else(|| normalize(cross(z, x)));

        x = normalize(cross(y, z));
        y = normalize(cross(z, x));

        Self {
            origin,
            x_axis: x,
            y_axis: y,
            z_axis: z,
        }
    }

    fn from_points(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> Self {
        let ab = subtract(b, a);
        let ac = subtract(c, a);
        let normal = cross(ab, ac);
        if vector_length_squared(normal) < EPSILON {
            return Self::default();
        }
        let x_axis = if vector_length_squared(ab) < EPSILON {
            orthogonal_vector(normal)
        } else {
            normalize(ab)
        };
        let z_axis = normalize(normal);
        let y_axis = normalize(cross(z_axis, x_axis));
        Self::normalize_axes(a, x_axis, y_axis, z_axis)
    }

    #[must_use]
    pub fn to_value(self) -> PlaneValue {
        PlaneValue::new(self.origin, self.x_axis, self.y_axis, self.z_axis)
    }
}

pub fn coerce_plane(value: &Value, context: &str) -> Result<Plane, ComponentError> {
    match value {
        Value::List(values) if values.len() >= 3 => {
            let a = coerce_point_with_context(&values[0], context)?;
            let b = coerce_point_with_context(&values[1], context)?;
            let c = coerce_point_with_context(&values[2], context)?;
            Ok(Plane::from_points(a, b, c))
        }
        Value::List(values) if values.len() == 2 => {
            let origin = coerce_point_with_context(&values[0], context)?;
            let direction = coerce_vector(&values[1], context)?;
            if vector_length_squared(direction) < EPSILON {
                Ok(Plane::default())
            } else {
                let x_axis = normalize(direction);
                let z_axis = orthogonal_vector(direction);
                let y_axis = normalize(cross(z_axis, x_axis));
                Ok(Plane::normalize_axes(origin, x_axis, y_axis, z_axis))
            }
        }
        Value::List(values) if values.len() == 1 => coerce_plane(&values[0], context),
        Value::Point(point) => {
            let mut plane = Plane::default();
            plane.origin = *point;
            Ok(plane)
        }
        Value::Vector(vector) => {
            let normal = if vector_length_squared(*vector) < EPSILON {
                [0.0, 0.0, 1.0]
            } else {
                normalize(*vector)
            };
            let x_axis = orthogonal_vector(normal);
            let y_axis = normalize(cross(normal, x_axis));
            Ok(Plane::normalize_axes(
                [0.0, 0.0, 0.0],
                x_axis,
                y_axis,
                normal,
            ))
        }
        other => Err(ComponentError::new(format!(
            "{} verwacht een vlak, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

pub fn coerce_geo_location(value: &Value, context: &str) -> Result<(f64, f64), ComponentError> {
    match value {
        Value::Vector(vector) | Value::Point(vector) => Ok((vector[0], vector[1])),
        Value::List(values) if !values.is_empty() => {
            let longitude = coerce_number(values.get(0).unwrap(), Some(context))?;
            let latitude = if values.len() > 1 {
                coerce_number(values.get(1).unwrap(), Some(context))?
            } else {
                0.0
            };
            Ok((longitude, latitude))
        }
        Value::List(values) if values.len() == 1 => coerce_geo_location(&values[0], context),
        Value::Number(number) => Ok((0.0, *number)),
        other => Err(ComponentError::new(format!(
            "{} verwacht een locatie, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

pub fn coerce_line(value: &Value, context: &str) -> Result<Line, ComponentError> {
    match value {
        Value::CurveLine { p1, p2 } => Ok(Line {
            start: *p1,
            end: *p2,
        }),
        Value::List(values) if values.len() >= 2 => {
            let start = coerce_point_with_context(&values[0], context)?;
            let mut end = coerce_point_with_context(&values[1], context)?;
            if vector_length_squared(subtract(end, start)) < EPSILON && values.len() > 2 {
                end = add(start, coerce_vector(&values[2], context)?);
            }
            Ok(Line { start, end })
        }
        Value::List(values) if values.len() == 1 => coerce_line(&values[0], context),
        other => Err(ComponentError::new(format!(
            "{} verwacht een curve, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

pub fn coerce_date_time(value: &Value) -> PrimitiveDateTime {
    if let Value::DateTime(date_time) = value {
        return date_time.primitive();
    }
    default_datetime()
}

pub fn default_datetime() -> PrimitiveDateTime {
    let date = Date::from_calendar_date(2020, Month::January, 1).unwrap();
    let time = Time::from_hms(12, 0, 0).unwrap();
    PrimitiveDateTime::new(date, time)
}

const EPSILON: f64 = 1e-9;

fn clamp_to_unit(value: f64) -> f64 {
    value.max(-1.0).min(1.0)
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

fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn subtract(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(vector: [f64; 3], factor: f64) -> [f64; 3] {
    [vector[0] * factor, vector[1] * factor, vector[2] * factor]
}

fn vector_length(vector: [f64; 3]) -> f64 {
    vector_length_squared(vector).sqrt()
}

fn vector_length_squared(vector: [f64; 3]) -> f64 {
    dot(vector, vector)
}

fn normalize(vector: [f64; 3]) -> [f64; 3] {
    if let Some((normalized, _)) = safe_normalized(vector) {
        normalized
    } else {
        [0.0, 0.0, 0.0]
    }
}

fn orthogonal_vector(vector: [f64; 3]) -> [f64; 3] {
    let abs_x = vector[0].abs();
    let abs_y = vector[1].abs();
    let abs_z = vector[2].abs();
    if abs_x <= abs_y && abs_x <= abs_z {
        normalize([0.0, -vector[2], vector[1]])
    } else if abs_y <= abs_x && abs_y <= abs_z {
        normalize([-vector[2], 0.0, vector[0]])
    } else {
        normalize([-vector[1], vector[0], 0.0])
    }
}

fn safe_normalized(vector: [f64; 3]) -> Option<([f64; 3], f64)> {
    let length = vector_length(vector);
    if length < EPSILON {
        None
    } else {
        Some((scale(vector, 1.0 / length), length))
    }
}

pub fn coerce_number_with_default(value: Option<&Value>) -> f64 {
    match value {
        Some(Value::Null) => 0.0,
        Some(v) => coerce_number(v, None).unwrap_or(0.0),
        None => 0.0,
    }
}

pub fn coerce_boolean_with_default(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Null) => true,
        Some(v) => coerce_boolean(v).unwrap_or(true),
        None => true,
    }
}

pub fn parse_boolean_text(input: &str) -> Option<bool> {
    match input.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "y" | "on" => Some(true),
        "false" | "0" | "no" | "n" | "off" => Some(false),
        _ => None,
    }
}

pub fn coerce_point_with_default(value: Option<&Value>) -> [f64; 3] {
    match value {
        Some(Value::Null) => [0.0, 0.0, 0.0],
        Some(Value::List(values)) => {
            for entry in values {
                if let Ok(point) = coerce_point_with_context(entry, "point") {
                    return point;
                }
            }
            [0.0, 0.0, 0.0]
        }
        Some(v) => coerce_point_with_context(v, "point").unwrap_or([0.0, 0.0, 0.0]),
        None => [0.0, 0.0, 0.0],
    }
}

// ============================================================================
// Bridge functions for geom::GeomMesh interoperability
// ============================================================================
// These functions are only available when the `mesh_engine_next` feature is enabled.

#[cfg(feature = "mesh_engine_next")]
pub mod geom_bridge {
    //! Bridge functions for converting between `geom::GeomMesh` and component types.
    //!
    //! This module provides interoperability between the new geometry kernel (`geom`)
    //! and the component layer types (`Mesh`, `MeshData`, `Value`).
    //!
    //! # Example
    //!
    //! ```ignore
    //! use ghx_engine::components::coerce::geom_bridge;
    //!
    //! // Convert GeomMesh to Value::Mesh
    //! let (geom_mesh, diag) = geom::mesh_surface(&surface, 10, 10);
    //! let value = geom_bridge::geom_mesh_to_value(geom_mesh, Some(diag));
    //!
    //! // Convert Value::Mesh to GeomMesh
    //! let geom_mesh = geom_bridge::value_to_geom_mesh(&value)?;
    //! ```

    use super::{ComponentError, Mesh, MeshData, SurfaceOwned};
    use crate::geom::{GeomMesh, GeomMeshDiagnostics, SurfaceBuilderQuality};
    use crate::graph::value::{MeshDiagnostics, Value};

    /// Converts a `geom::GeomMesh` to a `Value::Mesh`.
    ///
    /// Optionally includes diagnostics from the geometry kernel.
    #[must_use]
    pub fn geom_mesh_to_value(mesh: GeomMesh, diagnostics: Option<GeomMeshDiagnostics>) -> Value {
        Value::Mesh {
            vertices: mesh.positions,
            indices: mesh.indices,
            normals: mesh.normals,
            uvs: mesh.uvs,
            diagnostics: diagnostics.map(geom_diagnostics_to_value_diagnostics),
        }
    }

    /// Converts a `geom::GeomMesh` to a `MeshData`.
    #[must_use]
    pub fn geom_mesh_to_mesh_data(mesh: GeomMesh, diagnostics: Option<GeomMeshDiagnostics>) -> MeshData {
        MeshData {
            vertices: mesh.positions,
            indices: mesh.indices,
            normals: mesh.normals,
            uvs: mesh.uvs,
            diagnostics: diagnostics.map(geom_diagnostics_to_value_diagnostics),
        }
    }

    /// Converts a `geom::GeomMesh` to a coerce `Mesh`.
    #[must_use]
    pub fn geom_mesh_to_mesh(mesh: GeomMesh) -> Mesh {
        Mesh {
            vertices: mesh.positions,
            indices: mesh.indices,
            normals: mesh.normals,
            uvs: mesh.uvs,
        }
    }

    /// Converts a `geom::GeomMesh` to a legacy `Value::Surface`.
    ///
    /// **Note**: This is a lossy conversion - normals and UVs are discarded.
    #[must_use]
    pub fn geom_mesh_to_surface_legacy(mesh: GeomMesh) -> Value {
        let faces: Vec<Vec<u32>> = mesh
            .indices
            .chunks(3)
            .filter(|chunk| chunk.len() == 3)
            .map(|chunk| vec![chunk[0], chunk[1], chunk[2]])
            .collect();
        Value::Surface {
            vertices: mesh.positions,
            faces,
        }
    }

    /// Converts a `geom::GeomMesh` to a `SurfaceOwned`.
    ///
    /// **Note**: Normals and UVs are discarded.
    #[must_use]
    pub fn geom_mesh_to_surface_owned(mesh: GeomMesh) -> SurfaceOwned {
        let faces: Vec<Vec<u32>> = mesh
            .indices
            .chunks(3)
            .filter(|chunk| chunk.len() == 3)
            .map(|chunk| vec![chunk[0], chunk[1], chunk[2]])
            .collect();
        SurfaceOwned {
            vertices: mesh.positions,
            faces,
        }
    }

    /// Converts a `Value::Mesh` to a `geom::GeomMesh`.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not a `Value::Mesh`.
    pub fn value_to_geom_mesh(value: &Value) -> Result<GeomMesh, ComponentError> {
        match value {
            Value::Mesh {
                vertices,
                indices,
                normals,
                uvs,
                ..
            } => Ok(GeomMesh {
                positions: vertices.clone(),
                indices: indices.clone(),
                normals: normals.clone(),
                uvs: uvs.clone(),
                tangents: None,
            }),
            other => Err(ComponentError::new(format!(
                "Expected Mesh, got {}",
                other.kind()
            ))),
        }
    }

    /// Converts a mesh-like value (`Value::Mesh` or `Value::Surface`) to a `geom::GeomMesh`.
    ///
    /// For `Value::Surface`, faces are converted to triangle indices.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not a mesh-like type.
    pub fn value_to_geom_mesh_like(value: &Value) -> Result<GeomMesh, ComponentError> {
        match value {
            Value::Mesh {
                vertices,
                indices,
                normals,
                uvs,
                ..
            } => Ok(GeomMesh {
                positions: vertices.clone(),
                indices: indices.clone(),
                normals: normals.clone(),
                uvs: uvs.clone(),
                tangents: None,
            }),
            Value::Surface { vertices, faces } => {
                // Use proper fan triangulation to preserve all geometry in quads/n-gons
                let indices = super::triangulate_polygon_faces(faces);
                Ok(GeomMesh {
                    positions: vertices.clone(),
                    indices,
                    normals: None,
                    uvs: None,
                    tangents: None,
                })
            }
            other => Err(ComponentError::new(format!(
                "Expected Mesh or Surface, got {}",
                other.kind()
            ))),
        }
    }

    /// Converts a coerce `Mesh` to a `geom::GeomMesh`.
    #[must_use]
    pub fn mesh_to_geom_mesh(mesh: Mesh) -> GeomMesh {
        GeomMesh {
            positions: mesh.vertices,
            indices: mesh.indices,
            normals: mesh.normals,
            uvs: mesh.uvs,
            tangents: None,
        }
    }

    /// Converts a `MeshData` to a `geom::GeomMesh`.
    #[must_use]
    pub fn mesh_data_to_geom_mesh(data: MeshData) -> GeomMesh {
        GeomMesh {
            positions: data.vertices,
            indices: data.indices,
            normals: data.normals,
            uvs: data.uvs,
            tangents: None,
        }
    }

    /// Converts `geom::GeomMeshDiagnostics` to `MeshDiagnostics`.
    #[must_use]
    pub fn geom_diagnostics_to_value_diagnostics(diag: GeomMeshDiagnostics) -> MeshDiagnostics {
        MeshDiagnostics {
            vertex_count: diag.vertex_count,
            triangle_count: diag.triangle_count,
            welded_vertex_count: diag.welded_vertex_count,
            flipped_triangle_count: diag.flipped_triangle_count,
            degenerate_triangle_count: diag.degenerate_triangle_count,
            open_edge_count: diag.open_edge_count,
            non_manifold_edge_count: diag.non_manifold_edge_count,
            self_intersection_count: diag.self_intersection_count,
            boolean_fallback_used: diag.boolean_fallback_used,
            warnings: diag.warnings,
        }
    }

    /// Converts `MeshDiagnostics` to `geom::GeomMeshDiagnostics`.
    #[must_use]
    pub fn value_diagnostics_to_geom_diagnostics(diag: MeshDiagnostics) -> GeomMeshDiagnostics {
        GeomMeshDiagnostics {
            vertex_count: diag.vertex_count,
            triangle_count: diag.triangle_count,
            welded_vertex_count: diag.welded_vertex_count,
            flipped_triangle_count: diag.flipped_triangle_count,
            degenerate_triangle_count: diag.degenerate_triangle_count,
            open_edge_count: diag.open_edge_count,
            non_manifold_edge_count: diag.non_manifold_edge_count,
            self_intersection_count: diag.self_intersection_count,
            boolean_fallback_used: diag.boolean_fallback_used,
            timing: None,
            warnings: diag.warnings,
        }
    }

    // ========================================================================
    // Tolerance Conversions
    // ========================================================================

    use crate::geom::Tolerance;
    use crate::graph::node::{MetaLookupExt, MetaMap, MetaValue};
    use crate::graph::value::MeshQuality as GraphMeshQuality;

    /// Default tolerance value for geometry operations (matches `Tolerance::DEFAULT`).
    pub const DEFAULT_TOLERANCE: f64 = 1e-9;

    /// Loose tolerance for less precise operations.
    pub const LOOSE_TOLERANCE: f64 = 1e-6;

    /// Tight tolerance for high-precision operations.
    pub const TIGHT_TOLERANCE: f64 = 1e-12;

    /// Creates a `geom::Tolerance` from a numeric value.
    ///
    /// Returns the default tolerance if the value is not a valid positive number.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let tol = tolerance_from_number(1e-6);
    /// assert_eq!(tol.eps, 1e-6);
    /// ```
    #[must_use]
    pub fn tolerance_from_number(eps: f64) -> Tolerance {
        if eps.is_finite() && eps > 0.0 {
            Tolerance::new(eps)
        } else {
            Tolerance::default()
        }
    }

    /// Creates a `geom::Tolerance` from an optional `Value`.
    ///
    /// Accepts:
    /// - `Value::Number(eps)` - Uses the number as the tolerance epsilon
    /// - `Value::Text("default"|"loose"|"tight")` - Uses predefined tolerance presets
    /// - `None` or invalid - Returns `Tolerance::default()`
    ///
    /// # Example
    ///
    /// ```ignore
    /// let tol = tolerance_from_value(Some(&Value::Number(1e-6)));
    /// assert_eq!(tol.eps, 1e-6);
    ///
    /// let tol = tolerance_from_value(Some(&Value::Text("loose".to_string())));
    /// assert_eq!(tol.eps, 1e-6);
    /// ```
    #[must_use]
    pub fn tolerance_from_value(value: Option<&Value>) -> Tolerance {
        match value {
            Some(Value::Number(eps)) => tolerance_from_number(*eps),
            Some(Value::Text(preset)) => tolerance_from_preset(preset),
            Some(Value::List(list)) if list.len() == 1 => tolerance_from_value(list.first()),
            _ => Tolerance::default(),
        }
    }

    /// Creates a `geom::Tolerance` from a preset name.
    ///
    /// Supported presets (case-insensitive):
    /// - `"default"` / `"normal"` / `"standard"` → 1e-9
    /// - `"loose"` / `"coarse"` / `"low"` → 1e-6
    /// - `"tight"` / `"precise"` / `"high"` → 1e-12
    /// - `"weld"` → 1e-9 (same as default, for welding operations)
    /// - `"angle"` → 1e-9 (for angular comparisons)
    /// - `"zero"` / `"degenerate"` → 1e-12 (for zero-length checks)
    ///
    /// Returns `Tolerance::default()` if the preset is not recognized.
    #[must_use]
    pub fn tolerance_from_preset(preset: &str) -> Tolerance {
        match preset.trim().to_ascii_lowercase().as_str() {
            "default" | "normal" | "standard" => Tolerance::DEFAULT,
            "loose" | "coarse" | "low" => Tolerance::LOOSE,
            "tight" | "precise" | "high" => Tolerance::TIGHT,
            "weld" | "welding" => Tolerance::WELD,
            "angle" | "angular" => Tolerance::ANGLE,
            "zero" | "degenerate" | "zerolength" => Tolerance::ZERO_LENGTH,
            "derivative" | "diff" => Tolerance::DERIVATIVE,
            _ => Tolerance::default(),
        }
    }

    /// Extracts a `geom::Tolerance` from a `MetaMap`.
    ///
    /// Checks for the following keys (case-insensitive):
    /// - `tolerance` / `tol` / `eps` / `epsilon` → numeric tolerance value
    /// - `tolerance_preset` / `tol_preset` → preset name string
    ///
    /// If both are present, the numeric value takes precedence.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut meta = MetaMap::new();
    /// meta.insert("tolerance".to_string(), MetaValue::Number(1e-6));
    /// let tol = tolerance_from_meta(&meta);
    /// assert_eq!(tol.eps, 1e-6);
    /// ```
    #[must_use]
    pub fn tolerance_from_meta(meta: &MetaMap) -> Tolerance {
        // Try to get numeric tolerance first
        const NUMERIC_KEYS: &[&str] = &["tolerance", "tol", "eps", "epsilon", "geom_tolerance"];
        for key in NUMERIC_KEYS {
            if let Some(meta_value) = meta.get_normalized(key) {
                match meta_value {
                    MetaValue::Number(eps) => {
                        if *eps > 0.0 && eps.is_finite() {
                            return Tolerance::new(*eps);
                        }
                    }
                    MetaValue::Integer(eps) => {
                        let eps_f = *eps as f64;
                        if eps_f > 0.0 && eps_f.is_finite() {
                            return Tolerance::new(eps_f);
                        }
                    }
                    MetaValue::List(list) if !list.is_empty() => {
                        if let MetaValue::Number(eps) = &list[0] {
                            if *eps > 0.0 && eps.is_finite() {
                                return Tolerance::new(*eps);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Try to get preset tolerance
        const PRESET_KEYS: &[&str] = &["tolerance_preset", "tol_preset", "tolerance_level"];
        for key in PRESET_KEYS {
            if let Some(meta_value) = meta.get_normalized(key) {
                if let MetaValue::Text(preset) = meta_value {
                    return tolerance_from_preset(preset);
                }
            }
        }

        Tolerance::default()
    }

    /// Extracts a `geom::Tolerance` from a `MetaMap`, falling back to a provided default.
    ///
    /// This is useful when components have a specific default tolerance that
    /// differs from the global `Tolerance::DEFAULT`.
    #[must_use]
    pub fn tolerance_from_meta_or(meta: &MetaMap, default: Tolerance) -> Tolerance {
        const NUMERIC_KEYS: &[&str] = &["tolerance", "tol", "eps", "epsilon", "geom_tolerance"];
        for key in NUMERIC_KEYS {
            if let Some(meta_value) = meta.get_normalized(key) {
                match meta_value {
                    MetaValue::Number(eps) if *eps > 0.0 && eps.is_finite() => {
                        return Tolerance::new(*eps);
                    }
                    MetaValue::Integer(eps) => {
                        let eps_f = *eps as f64;
                        if eps_f > 0.0 && eps_f.is_finite() {
                            return Tolerance::new(eps_f);
                        }
                    }
                    _ => {}
                }
            }
        }

        const PRESET_KEYS: &[&str] = &["tolerance_preset", "tol_preset", "tolerance_level"];
        for key in PRESET_KEYS {
            if let Some(MetaValue::Text(preset)) = meta.get_normalized(key) {
                return tolerance_from_preset(preset);
            }
        }

        default
    }

    // ========================================================================
    // MeshQuality Conversions (graph::MeshQuality <-> geom::loft::MeshQuality)
    // ========================================================================

    use crate::geom::MeshQuality as GeomMeshQuality;

    /// Converts `graph::value::MeshQuality` to `geom::loft::MeshQuality`.
    ///
    /// Maps the graph-layer quality parameters to the geom-layer parameters:
    /// - `max_edge_length` → `target_edge_length`
    /// - `max_deviation` → `max_deviation`
    /// - `angle_threshold_degrees` → `max_angle` (converted to radians)
    /// - `min_subdivisions` → `min_points_per_profile`
    /// - `max_subdivisions` → `max_points_per_profile`
    ///
    /// # Example
    ///
    /// ```ignore
    /// let graph_quality = GraphMeshQuality::high();
    /// let geom_quality = graph_quality_to_geom(graph_quality);
    /// ```
    #[must_use]
    pub fn graph_quality_to_geom(quality: GraphMeshQuality) -> GeomMeshQuality {
        GeomMeshQuality {
            target_edge_length: quality.max_edge_length,
            max_deviation: quality.max_deviation,
            max_angle: quality.angle_threshold_degrees.to_radians(),
            min_points_per_profile: quality.min_subdivisions,
            max_points_per_profile: quality.max_subdivisions,
        }
    }

    /// Converts `geom::loft::MeshQuality` to `graph::value::MeshQuality`.
    ///
    /// Maps the geom-layer quality parameters back to the graph-layer:
    /// - `target_edge_length` → `max_edge_length`
    /// - `max_deviation` → `max_deviation`
    /// - `max_angle` → `angle_threshold_degrees` (converted to degrees)
    /// - `min_points_per_profile` → `min_subdivisions`
    /// - `max_points_per_profile` → `max_subdivisions`
    #[must_use]
    pub fn geom_quality_to_graph(quality: GeomMeshQuality) -> GraphMeshQuality {
        GraphMeshQuality::new(
            quality.target_edge_length,
            quality.max_deviation,
            quality.max_angle.to_degrees(),
            quality.min_points_per_profile,
            quality.max_points_per_profile,
        )
    }

    /// Extracts `geom::loft::MeshQuality` from a `MetaMap`.
    ///
    /// First extracts `graph::value::MeshQuality` using its `from_meta` method,
    /// then converts to `geom::loft::MeshQuality`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut meta = MetaMap::new();
    /// meta.insert("mesh_quality".to_string(), MetaValue::Text("high".to_string()));
    /// let quality = geom_quality_from_meta(&meta);
    /// ```
    #[must_use]
    pub fn geom_quality_from_meta(meta: &MetaMap) -> GeomMeshQuality {
        let graph_quality = GraphMeshQuality::from_meta(meta);
        graph_quality_to_geom(graph_quality)
    }

    /// Extracts an optional `geom::loft::MeshQuality` from a `MetaMap`.
    ///
    /// Returns `Some(quality)` only if quality settings are explicitly specified
    /// in the `MetaMap`. This allows the geom layer to use its own defaults
    /// when no quality is specified.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let meta = MetaMap::new(); // Empty
    /// let quality = geom_quality_from_meta_optional(&meta);
    /// assert!(quality.is_none()); // No quality specified
    ///
    /// let mut meta = MetaMap::new();
    /// meta.insert("mesh_quality".to_string(), MetaValue::Text("high".to_string()));
    /// let quality = geom_quality_from_meta_optional(&meta);
    /// assert!(quality.is_some());
    /// ```
    #[must_use]
    pub fn geom_quality_from_meta_optional(meta: &MetaMap) -> Option<GeomMeshQuality> {
        // Check if any quality-related keys are present
        const QUALITY_KEYS: &[&str] = &[
            "mesh_quality",
            "quality",
            "preset",
            "max_edge_length",
            "edge_length",
            "max_deviation",
            "deviation",
            "tolerance",
            "angle_threshold",
            "angle",
            "min_subdivisions",
            "max_subdivisions",
        ];

        let has_quality_settings = QUALITY_KEYS.iter().any(|key| meta.get_normalized(key).is_some());

        if has_quality_settings {
            Some(geom_quality_from_meta(meta))
        } else {
            None
        }
    }

    /// Extracts `geom::loft::MeshQuality` from a `Value`.
    ///
    /// Accepts:
    /// - `Value::Text(preset)` - Quality preset name
    /// - `Value::Number(index)` - Preset index (0=low, 1=medium, 2=high, 3=ultra)
    /// - `Value::List([...])` - First element interpreted as above
    ///
    /// Returns the default quality if the value cannot be parsed.
    #[must_use]
    pub fn geom_quality_from_value(value: Option<&Value>) -> GeomMeshQuality {
        match value {
            Some(v) => {
                if let Some(graph_quality) = GraphMeshQuality::from_value(v) {
                    graph_quality_to_geom(graph_quality)
                } else {
                    GeomMeshQuality::default()
                }
            }
            None => GeomMeshQuality::default(),
        }
    }

    // ========================================================================
    // SurfaceBuilderQuality Conversions
    // ========================================================================

    /// Extracts `SurfaceBuilderQuality` from a `MetaMap`.
    ///
    /// This function parses mesh quality settings from component metadata and
    /// converts them to `SurfaceBuilderQuality` for surface builder operations
    /// (e.g., FourPointSurface, RuledSurface, EdgeSurface, NetworkSurface, SumSurface).
    ///
    /// The conversion uses `min_subdivisions` and `max_subdivisions` from the
    /// `MeshQuality` settings to compute appropriate surface subdivisions using
    /// the geometric mean for a balanced result.
    ///
    /// # Supported Keys
    ///
    /// Inherits all keys from `MeshQuality::from_meta`:
    /// - `mesh_quality` / `quality` / `preset` - Preset name ("low", "medium", "high", "ultra")
    /// - `min_subdivisions` / `min_subdiv` - Minimum subdivisions
    /// - `max_subdivisions` / `max_subdiv` - Maximum subdivisions
    ///
    /// Additionally supports direct surface builder keys:
    /// - `u_subdivisions` / `u_subdiv` - Explicit U subdivisions (overrides preset)
    /// - `v_subdivisions` / `v_subdiv` - Explicit V subdivisions (overrides preset)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut meta = MetaMap::new();
    /// meta.insert("mesh_quality".to_string(), MetaValue::Text("high".to_string()));
    /// let quality = surface_builder_quality_from_meta(&meta);
    /// assert_eq!(quality.u_subdivisions, 20); // High preset
    /// ```
    #[must_use]
    pub fn surface_builder_quality_from_meta(meta: &MetaMap) -> SurfaceBuilderQuality {
        // Check for explicit u/v subdivision overrides first
        let u_subdiv = extract_usize_from_meta(meta, &["u_subdivisions", "u_subdiv", "usubdiv"]);
        let v_subdiv = extract_usize_from_meta(meta, &["v_subdivisions", "v_subdiv", "vsubdiv"]);

        // If both explicit values are provided, use them directly
        if let (Some(u), Some(v)) = (u_subdiv, v_subdiv) {
            return SurfaceBuilderQuality::new(u.max(2), v.max(2));
        }

        // Check for preset name directly
        for key in &["mesh_quality", "quality", "preset", "surface_quality"] {
            if let Some(MetaValue::Text(preset)) = meta.get_normalized(key) {
                if let Some(quality) = SurfaceBuilderQuality::from_preset_name(preset) {
                    // Apply any explicit overrides
                    let u = u_subdiv.unwrap_or(quality.u_subdivisions);
                    let v = v_subdiv.unwrap_or(quality.v_subdivisions);
                    return SurfaceBuilderQuality::new(u.max(2), v.max(2));
                }
            }
        }

        // Fall back to deriving from MeshQuality settings
        let graph_quality = GraphMeshQuality::from_meta(meta);
        let base = SurfaceBuilderQuality::from_subdivision_range(
            graph_quality.min_subdivisions,
            graph_quality.max_subdivisions,
        );

        // Apply any explicit overrides
        SurfaceBuilderQuality::new(
            u_subdiv.unwrap_or(base.u_subdivisions).max(2),
            v_subdiv.unwrap_or(base.v_subdivisions).max(2),
        )
    }

    /// Extracts an optional `SurfaceBuilderQuality` from a `MetaMap`.
    ///
    /// Returns `Some(quality)` only if quality settings are explicitly specified
    /// in the `MetaMap`. This allows the caller to use `SurfaceBuilderQuality::default()`
    /// when no quality is specified.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let quality = surface_builder_quality_from_meta_optional(&meta)
    ///     .unwrap_or_else(SurfaceBuilderQuality::default);
    /// ```
    #[must_use]
    pub fn surface_builder_quality_from_meta_optional(meta: &MetaMap) -> Option<SurfaceBuilderQuality> {
        // Check if any quality-related keys are present
        const QUALITY_KEYS: &[&str] = &[
            "mesh_quality",
            "quality",
            "preset",
            "surface_quality",
            "min_subdivisions",
            "max_subdivisions",
            "u_subdivisions",
            "v_subdivisions",
            "u_subdiv",
            "v_subdiv",
        ];

        let has_quality_settings = QUALITY_KEYS.iter().any(|key| meta.get_normalized(key).is_some());

        if has_quality_settings {
            Some(surface_builder_quality_from_meta(meta))
        } else {
            None
        }
    }

    /// Helper to extract a usize value from meta by trying multiple key names.
    fn extract_usize_from_meta(meta: &MetaMap, keys: &[&str]) -> Option<usize> {
        for key in keys {
            if let Some(meta_value) = meta.get_normalized(key) {
                match meta_value {
                    MetaValue::Number(n) if *n >= 0.0 => return Some(*n as usize),
                    MetaValue::Integer(n) if *n >= 0 => return Some(*n as usize),
                    _ => {}
                }
            }
        }
        None
    }

    // ========================================================================
    // Combined Tolerance + Quality Extraction (Component Helpers)
    // ========================================================================

    /// Context for geometry operations extracted from `MetaMap`.
    ///
    /// This struct bundles tolerance and quality settings that are commonly
    /// needed by geometry-generating components (loft, sweep, extrude, etc.).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let ctx = GeomOperationContext::from_meta(&meta);
    /// let (mesh, diag) = geom::loft::loft_mesh_with_tolerance(profiles, options, ctx.tolerance);
    /// ```
    #[derive(Debug, Clone)]
    pub struct GeomOperationContext {
        /// Tolerance for geometric comparisons and welding.
        pub tolerance: Tolerance,
        /// Optional mesh quality settings (None = use geom defaults).
        pub quality: Option<GeomMeshQuality>,
    }

    impl Default for GeomOperationContext {
        fn default() -> Self {
            Self {
                tolerance: Tolerance::default(),
                quality: None,
            }
        }
    }

    impl GeomOperationContext {
        /// Creates a new context with default tolerance and no explicit quality.
        #[must_use]
        pub fn new() -> Self {
            Self::default()
        }

        /// Creates a context from a `MetaMap`.
        ///
        /// Extracts both tolerance and mesh quality settings from the metadata.
        #[must_use]
        pub fn from_meta(meta: &MetaMap) -> Self {
            Self {
                tolerance: tolerance_from_meta(meta),
                quality: geom_quality_from_meta_optional(meta),
            }
        }

        /// Creates a context with a specific tolerance.
        #[must_use]
        pub fn with_tolerance(tolerance: Tolerance) -> Self {
            Self {
                tolerance,
                quality: None,
            }
        }

        /// Creates a context with a specific quality preset.
        #[must_use]
        pub fn with_quality(quality: GeomMeshQuality) -> Self {
            Self {
                tolerance: Tolerance::default(),
                quality: Some(quality),
            }
        }

        /// Creates a context with both tolerance and quality.
        #[must_use]
        pub fn with_tolerance_and_quality(tolerance: Tolerance, quality: GeomMeshQuality) -> Self {
            Self {
                tolerance,
                quality: Some(quality),
            }
        }

        /// Returns the quality, falling back to the default if not set.
        #[must_use]
        pub fn quality_or_default(&self) -> GeomMeshQuality {
            self.quality.unwrap_or_default()
        }
    }

    // ========================================================================
    // Enhanced Mesh/Surface Conversion with Tolerance
    // ========================================================================

    /// Converts a `Value::Surface` or `Value::Mesh` to a `GeomMesh` with welding.
    ///
    /// For `Value::Surface`, converts polygon faces to triangles and optionally
    /// welds coincident vertices within the specified tolerance.
    ///
    /// # Arguments
    ///
    /// * `value` - The mesh-like value to convert
    /// * `weld` - Whether to weld coincident vertices
    /// * `tolerance` - Tolerance for vertex welding
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not a mesh-like type.
    pub fn value_to_geom_mesh_welded(
        value: &Value,
        weld: bool,
        tolerance: Tolerance,
    ) -> Result<(GeomMesh, GeomMeshDiagnostics), ComponentError> {
        let base_mesh = value_to_geom_mesh_like(value)?;

        if !weld {
            let diag = GeomMeshDiagnostics {
                vertex_count: base_mesh.vertex_count(),
                triangle_count: base_mesh.triangle_count(),
                ..Default::default()
            };
            return Ok((base_mesh, diag));
        }

        // Use the geom mesh repair functions for welding
        use crate::geom::weld_mesh_vertices;

        // Convert to points for welding
        let points: Vec<crate::geom::Point3> = base_mesh
            .positions
            .iter()
            .map(|p| crate::geom::Point3::from_array(*p))
            .collect();

        let uvs_slice = base_mesh.uvs.as_deref();

        let (welded_points, welded_uvs, welded_indices, welded_count) =
            weld_mesh_vertices(points, uvs_slice, base_mesh.indices, tolerance);

        let welded_mesh = GeomMesh {
            positions: welded_points.iter().map(|p| p.to_array()).collect(),
            indices: welded_indices,
            uvs: welded_uvs,
            normals: None, // Normals need recompute after welding
            tangents: None,
        };

        let diag = GeomMeshDiagnostics {
            vertex_count: welded_mesh.vertex_count(),
            triangle_count: welded_mesh.triangle_count(),
            welded_vertex_count: welded_count,
            ..Default::default()
        };

        Ok((welded_mesh, diag))
    }

    /// Coerces a `Value` to `GeomMesh` with context-based tolerance and welding.
    ///
    /// This is the preferred entry point for components that need to convert
    /// input mesh-like values to `GeomMesh` for further processing.
    ///
    /// # Arguments
    ///
    /// * `value` - The mesh-like value to convert
    /// * `ctx` - Operation context containing tolerance settings
    /// * `weld` - Whether to weld coincident vertices
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not a mesh-like type.
    pub fn coerce_value_to_geom_mesh(
        value: &Value,
        ctx: &GeomOperationContext,
        weld: bool,
    ) -> Result<(GeomMesh, GeomMeshDiagnostics), ComponentError> {
        value_to_geom_mesh_welded(value, weld, ctx.tolerance)
    }

    /// Coerces a list of `Value` items to `GeomMesh` instances.
    ///
    /// # Arguments
    ///
    /// * `value` - A `Value::List` of mesh-like values, or a single mesh-like value
    /// * `ctx` - Operation context containing tolerance settings
    /// * `weld` - Whether to weld coincident vertices in each mesh
    /// * `context` - Error context string for error messages
    ///
    /// # Errors
    ///
    /// Returns an error if any value in the list is not a mesh-like type.
    pub fn coerce_value_list_to_geom_meshes(
        value: &Value,
        ctx: &GeomOperationContext,
        weld: bool,
        context: &str,
    ) -> Result<Vec<(GeomMesh, GeomMeshDiagnostics)>, ComponentError> {
        match value {
            Value::List(values) => {
                let mut result = Vec::with_capacity(values.len());
                for (i, entry) in values.iter().enumerate() {
                    let (mesh, diag) = value_to_geom_mesh_welded(entry, weld, ctx.tolerance)
                        .map_err(|e| {
                            ComponentError::new(format!("{} item {}: {}", context, i, e))
                        })?;
                    result.push((mesh, diag));
                }
                Ok(result)
            }
            other => {
                let (mesh, diag) = value_to_geom_mesh_welded(other, weld, ctx.tolerance)?;
                Ok(vec![(mesh, diag)])
            }
        }
    }

    // ========================================================================
    // Reference Conversion Helpers
    // ========================================================================

    /// Converts `geom::GeomMeshDiagnostics` reference to `MeshDiagnostics`.
    #[must_use]
    pub fn geom_diagnostics_to_value_diagnostics_ref(diag: &GeomMeshDiagnostics) -> MeshDiagnostics {
        MeshDiagnostics {
            vertex_count: diag.vertex_count,
            triangle_count: diag.triangle_count,
            welded_vertex_count: diag.welded_vertex_count,
            flipped_triangle_count: diag.flipped_triangle_count,
            degenerate_triangle_count: diag.degenerate_triangle_count,
            open_edge_count: diag.open_edge_count,
            non_manifold_edge_count: diag.non_manifold_edge_count,
            self_intersection_count: diag.self_intersection_count,
            boolean_fallback_used: diag.boolean_fallback_used,
            warnings: diag.warnings.clone(),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn geom_mesh_roundtrip_to_value() {
            let geom_mesh = GeomMesh::new(
                vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
                vec![0, 1, 2],
            );

            let value = geom_mesh_to_value(geom_mesh, None);

            if let Value::Mesh { vertices, indices, .. } = &value {
                assert_eq!(vertices.len(), 3);
                assert_eq!(indices.len(), 3);
            } else {
                panic!("Expected Value::Mesh");
            }

            let back = value_to_geom_mesh(&value).unwrap();
            assert_eq!(back.vertex_count(), 3);
            assert_eq!(back.triangle_count(), 1);
        }

        #[test]
        fn geom_mesh_to_mesh_preserves_attributes() {
            let geom_mesh = GeomMesh::with_attributes(
                vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
                vec![0, 1, 2],
                Some(vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]]),
                Some(vec![[0.0, 0.0, 1.0]; 3]),
            );

            let mesh = geom_mesh_to_mesh(geom_mesh);
            assert_eq!(mesh.vertex_count(), 3);
            assert!(mesh.has_normals());
            assert!(mesh.has_uvs());
        }

        #[test]
        fn value_to_geom_mesh_like_accepts_surface() {
            let value = Value::Surface {
                vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
                faces: vec![vec![0, 1, 2]],
            };

            let geom_mesh = value_to_geom_mesh_like(&value).unwrap();
            assert_eq!(geom_mesh.vertex_count(), 3);
            assert_eq!(geom_mesh.triangle_count(), 1);
        }

        #[test]
        fn diagnostics_roundtrip() {
            let geom_diag = GeomMeshDiagnostics {
                vertex_count: 100,
                triangle_count: 50,
                welded_vertex_count: 5,
                flipped_triangle_count: 2,
                degenerate_triangle_count: 1,
                open_edge_count: 3,
                non_manifold_edge_count: 0,
                self_intersection_count: 0,
                boolean_fallback_used: false,
                timing: None,
                warnings: vec!["test warning".to_string()],
            };

            let value_diag = geom_diagnostics_to_value_diagnostics(geom_diag);
            assert_eq!(value_diag.vertex_count, 100);
            assert_eq!(value_diag.triangle_count, 50);
            assert_eq!(value_diag.welded_vertex_count, 5);
            assert!(!value_diag.warnings.is_empty());

            let back = value_diagnostics_to_geom_diagnostics(value_diag);
            assert_eq!(back.vertex_count, 100);
            assert_eq!(back.warnings.len(), 1);
        }

        // ====================================================================
        // Tolerance Conversion Tests
        // ====================================================================

        #[test]
        fn tolerance_from_number_positive() {
            let tol = tolerance_from_number(1e-6);
            assert_eq!(tol.eps, 1e-6);
        }

        #[test]
        fn tolerance_from_number_invalid_returns_default() {
            let tol = tolerance_from_number(-1.0);
            assert_eq!(tol.eps, Tolerance::DEFAULT.eps);

            let tol = tolerance_from_number(f64::NAN);
            assert_eq!(tol.eps, Tolerance::DEFAULT.eps);

            let tol = tolerance_from_number(f64::INFINITY);
            assert_eq!(tol.eps, Tolerance::DEFAULT.eps);
        }

        #[test]
        fn tolerance_from_preset_recognizes_common_names() {
            assert_eq!(tolerance_from_preset("default").eps, Tolerance::DEFAULT.eps);
            assert_eq!(tolerance_from_preset("LOOSE").eps, Tolerance::LOOSE.eps);
            assert_eq!(tolerance_from_preset("Tight").eps, Tolerance::TIGHT.eps);
            assert_eq!(tolerance_from_preset("weld").eps, Tolerance::WELD.eps);
        }

        #[test]
        fn tolerance_from_preset_unknown_returns_default() {
            let tol = tolerance_from_preset("unknown_preset");
            assert_eq!(tol.eps, Tolerance::DEFAULT.eps);
        }

        #[test]
        fn tolerance_from_value_number() {
            let value = Value::Number(1e-8);
            let tol = tolerance_from_value(Some(&value));
            assert_eq!(tol.eps, 1e-8);
        }

        #[test]
        fn tolerance_from_value_text_preset() {
            let value = Value::Text("loose".to_string());
            let tol = tolerance_from_value(Some(&value));
            assert_eq!(tol.eps, Tolerance::LOOSE.eps);
        }

        #[test]
        fn tolerance_from_value_none_returns_default() {
            let tol = tolerance_from_value(None);
            assert_eq!(tol.eps, Tolerance::DEFAULT.eps);
        }

        #[test]
        fn tolerance_from_meta_extracts_numeric() {
            use crate::graph::node::MetaMap;
            let mut meta = MetaMap::new();
            meta.insert("tolerance".to_string(), MetaValue::Number(1e-7));
            let tol = tolerance_from_meta(&meta);
            assert_eq!(tol.eps, 1e-7);
        }

        #[test]
        fn tolerance_from_meta_extracts_preset() {
            use crate::graph::node::MetaMap;
            let mut meta = MetaMap::new();
            meta.insert("tolerance_preset".to_string(), MetaValue::Text("tight".to_string()));
            let tol = tolerance_from_meta(&meta);
            assert_eq!(tol.eps, Tolerance::TIGHT.eps);
        }

        #[test]
        fn tolerance_from_meta_empty_returns_default() {
            use crate::graph::node::MetaMap;
            let meta = MetaMap::new();
            let tol = tolerance_from_meta(&meta);
            assert_eq!(tol.eps, Tolerance::DEFAULT.eps);
        }

        // ====================================================================
        // MeshQuality Conversion Tests
        // ====================================================================

        #[test]
        fn graph_quality_to_geom_converts_correctly() {
            let graph_q = GraphMeshQuality::high();
            let geom_q = graph_quality_to_geom(graph_q);
            
            assert_eq!(geom_q.target_edge_length, 0.25);
            assert_eq!(geom_q.max_deviation, 0.001);
            // Angle should be converted to radians
            assert!((geom_q.max_angle - 8.0_f64.to_radians()).abs() < 1e-10);
            assert_eq!(geom_q.min_points_per_profile, 8);
            assert_eq!(geom_q.max_points_per_profile, 512);
        }

        #[test]
        fn geom_quality_to_graph_converts_correctly() {
            let geom_q = GeomMeshQuality {
                target_edge_length: 0.5,
                max_deviation: 0.01,
                max_angle: 15.0_f64.to_radians(),
                min_points_per_profile: 4,
                max_points_per_profile: 256,
            };
            let graph_q = geom_quality_to_graph(geom_q);

            assert_eq!(graph_q.max_edge_length, 0.5);
            assert_eq!(graph_q.max_deviation, 0.01);
            assert!((graph_q.angle_threshold_degrees - 15.0).abs() < 1e-6);
        }

        #[test]
        fn geom_quality_from_meta_parses_preset() {
            use crate::graph::node::MetaMap;
            let mut meta = MetaMap::new();
            meta.insert("mesh_quality".to_string(), MetaValue::Text("high".to_string()));
            let quality = geom_quality_from_meta(&meta);
            
            // Should match high preset values
            assert_eq!(quality.target_edge_length, 0.25);
        }

        #[test]
        fn geom_quality_from_meta_optional_returns_none_for_empty() {
            use crate::graph::node::MetaMap;
            let meta = MetaMap::new();
            let quality = geom_quality_from_meta_optional(&meta);
            assert!(quality.is_none());
        }

        #[test]
        fn geom_quality_from_meta_optional_returns_some_when_specified() {
            use crate::graph::node::MetaMap;
            let mut meta = MetaMap::new();
            meta.insert("max_edge_length".to_string(), MetaValue::Number(0.5));
            let quality = geom_quality_from_meta_optional(&meta);
            assert!(quality.is_some());
        }

        // ====================================================================
        // SurfaceBuilderQuality Tests
        // ====================================================================

        #[test]
        fn surface_builder_quality_from_meta_low_preset() {
            use crate::graph::node::MetaMap;
            let mut meta = MetaMap::new();
            meta.insert("mesh_quality".to_string(), MetaValue::Text("low".to_string()));
            let quality = surface_builder_quality_from_meta(&meta);
            assert_eq!(quality.u_subdivisions, 4);
            assert_eq!(quality.v_subdivisions, 4);
        }

        #[test]
        fn surface_builder_quality_from_meta_high_preset() {
            use crate::graph::node::MetaMap;
            let mut meta = MetaMap::new();
            meta.insert("mesh_quality".to_string(), MetaValue::Text("high".to_string()));
            let quality = surface_builder_quality_from_meta(&meta);
            assert_eq!(quality.u_subdivisions, 20);
            assert_eq!(quality.v_subdivisions, 20);
        }

        #[test]
        fn surface_builder_quality_from_meta_explicit_subdivisions() {
            use crate::graph::node::MetaMap;
            let mut meta = MetaMap::new();
            meta.insert("u_subdivisions".to_string(), MetaValue::Integer(8));
            meta.insert("v_subdivisions".to_string(), MetaValue::Integer(12));
            let quality = surface_builder_quality_from_meta(&meta);
            assert_eq!(quality.u_subdivisions, 8);
            assert_eq!(quality.v_subdivisions, 12);
        }

        #[test]
        fn surface_builder_quality_from_meta_preset_with_u_override() {
            use crate::graph::node::MetaMap;
            let mut meta = MetaMap::new();
            meta.insert("mesh_quality".to_string(), MetaValue::Text("high".to_string()));
            meta.insert("u_subdivisions".to_string(), MetaValue::Integer(5));
            let quality = surface_builder_quality_from_meta(&meta);
            // U should be overridden, V should come from preset
            assert_eq!(quality.u_subdivisions, 5);
            assert_eq!(quality.v_subdivisions, 20);
        }

        #[test]
        fn surface_builder_quality_from_meta_optional_returns_none_for_empty() {
            use crate::graph::node::MetaMap;
            let meta = MetaMap::new();
            let quality = surface_builder_quality_from_meta_optional(&meta);
            assert!(quality.is_none());
        }

        #[test]
        fn surface_builder_quality_from_meta_optional_returns_some_for_preset() {
            use crate::graph::node::MetaMap;
            let mut meta = MetaMap::new();
            meta.insert("mesh_quality".to_string(), MetaValue::Text("low".to_string()));
            let quality = surface_builder_quality_from_meta_optional(&meta);
            assert!(quality.is_some());
            let q = quality.unwrap();
            assert_eq!(q.u_subdivisions, 4);
        }

        #[test]
        fn surface_builder_quality_from_meta_optional_returns_some_for_subdiv() {
            use crate::graph::node::MetaMap;
            let mut meta = MetaMap::new();
            meta.insert("u_subdivisions".to_string(), MetaValue::Number(15.0));
            let quality = surface_builder_quality_from_meta_optional(&meta);
            assert!(quality.is_some());
        }

        // ====================================================================
        // GeomOperationContext Tests
        // ====================================================================

        #[test]
        fn geom_operation_context_default() {
            let ctx = GeomOperationContext::new();
            assert_eq!(ctx.tolerance.eps, Tolerance::DEFAULT.eps);
            assert!(ctx.quality.is_none());
        }

        #[test]
        fn geom_operation_context_from_meta() {
            use crate::graph::node::MetaMap;
            let mut meta = MetaMap::new();
            meta.insert("tolerance".to_string(), MetaValue::Number(1e-5));
            meta.insert("mesh_quality".to_string(), MetaValue::Text("high".to_string()));

            let ctx = GeomOperationContext::from_meta(&meta);
            assert_eq!(ctx.tolerance.eps, 1e-5);
            assert!(ctx.quality.is_some());
        }

        #[test]
        fn geom_operation_context_quality_or_default() {
            let ctx = GeomOperationContext::new();
            let quality = ctx.quality_or_default();
            // Should return default quality
            assert_eq!(quality.min_points_per_profile, GeomMeshQuality::default().min_points_per_profile);
        }

        // ====================================================================
        // Enhanced Mesh Conversion Tests
        // ====================================================================

        #[test]
        fn value_to_geom_mesh_welded_no_weld() {
            let value = Value::Mesh {
                vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
                indices: vec![0, 1, 2],
                normals: None,
                uvs: None,
                diagnostics: None,
            };

            let (mesh, diag) = value_to_geom_mesh_welded(&value, false, Tolerance::default()).unwrap();
            assert_eq!(mesh.vertex_count(), 3);
            assert_eq!(diag.welded_vertex_count, 0);
        }

        #[test]
        fn value_to_geom_mesh_welded_with_weld() {
            // Create a mesh with duplicate vertices that should be welded
            let value = Value::Mesh {
                vertices: vec![
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [0.5, 1.0, 0.0],
                    [0.0, 0.0, 0.0], // Duplicate of vertex 0
                    [1.0, 0.0, 0.0], // Duplicate of vertex 1
                    [0.5, 0.0, 0.5],
                ],
                indices: vec![0, 1, 2, 3, 4, 5],
                normals: None,
                uvs: None,
                diagnostics: None,
            };

            let (mesh, diag) = value_to_geom_mesh_welded(&value, true, Tolerance::new(1e-6)).unwrap();
            // Should have welded 2 duplicate vertices
            assert!(mesh.vertex_count() <= 4);
            assert!(diag.welded_vertex_count >= 2);
        }

        #[test]
        fn coerce_value_list_to_geom_meshes_processes_list() {
            let value = Value::List(vec![
                Value::Mesh {
                    vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
                    indices: vec![0, 1, 2],
                    normals: None,
                    uvs: None,
                    diagnostics: None,
                },
                Value::Surface {
                    vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
                    faces: vec![vec![0, 1, 2]],
                },
            ]);

            let ctx = GeomOperationContext::new();
            let meshes = coerce_value_list_to_geom_meshes(&value, &ctx, false, "test").unwrap();
            assert_eq!(meshes.len(), 2);
            assert_eq!(meshes[0].0.vertex_count(), 3);
            assert_eq!(meshes[1].0.vertex_count(), 3);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_boolean_text_accepts_numeric_forms() {
        assert_eq!(parse_boolean_text("1"), Some(true));
        assert_eq!(parse_boolean_text("0"), Some(false));
        assert_eq!(parse_boolean_text(" true "), Some(true));
    }

    #[test]
    fn coerce_boolean_accepts_numeric_strings() {
        assert_eq!(coerce_boolean(&Value::Text("1".to_owned())).unwrap(), true);
        assert_eq!(coerce_boolean(&Value::Text("0".to_owned())).unwrap(), false);
    }

    #[test]
    fn coerce_integer_handles_text_booleans_and_numbers() {
        assert_eq!(coerce_integer(&Value::Text("True".to_owned())).unwrap(), 1);
        assert_eq!(coerce_integer(&Value::Text("0".to_owned())).unwrap(), 0);
        assert_eq!(coerce_integer(&Value::Text("2.4".to_owned())).unwrap(), 2);
    }

    // ========================================================================
    // Tests for Mesh coercion functions
    // ========================================================================

    #[test]
    fn coerce_mesh_accepts_mesh_value() {
        let value = Value::Mesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: Some(vec![[0.0, 0.0, 1.0]; 3]),
            uvs: None,
            diagnostics: None,
        };

        let mesh = coerce_mesh(&value).unwrap();
        assert_eq!(mesh.vertex_count(), 3);
        assert_eq!(mesh.triangle_count(), 1);
        assert!(mesh.has_normals());
        assert!(!mesh.has_uvs());
    }

    #[test]
    fn coerce_mesh_rejects_surface() {
        let value = Value::Surface {
            vertices: vec![[0.0, 0.0, 0.0]],
            faces: vec![],
        };

        let result = coerce_mesh(&value);
        assert!(result.is_err());
    }

    #[test]
    fn coerce_mesh_like_accepts_mesh() {
        let value = Value::Mesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: Some(vec![[0.0, 0.0, 1.0]; 3]),
            uvs: None,
            diagnostics: None,
        };

        let mesh = coerce_mesh_like(&value).unwrap();
        assert_eq!(mesh.vertex_count(), 3);
        assert_eq!(mesh.triangle_count(), 1);
        assert!(mesh.has_normals());
    }

    #[test]
    fn coerce_mesh_like_accepts_surface() {
        let value = Value::Surface {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            faces: vec![vec![0, 1, 2], vec![0, 2, 3]],
        };

        let mesh = coerce_mesh_like(&value).unwrap();
        assert_eq!(mesh.vertex_count(), 4);
        assert_eq!(mesh.triangle_count(), 2);
        assert_eq!(mesh.indices, vec![0, 1, 2, 0, 2, 3]);
        assert!(!mesh.has_normals());
        assert!(!mesh.has_uvs());
    }

    #[test]
    fn coerce_mesh_like_unwraps_single_element_list() {
        let value = Value::List(vec![Value::Mesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: None,
            uvs: None,
            diagnostics: None,
        }]);

        let mesh = coerce_mesh_like(&value).unwrap();
        assert_eq!(mesh.vertex_count(), 3);
        assert_eq!(mesh.triangle_count(), 1);
    }

    #[test]
    fn coerce_mesh_list_accepts_list_of_meshes() {
        let value = Value::List(vec![
            Value::Mesh {
                vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
                indices: vec![0, 1, 2],
                normals: None,
                uvs: None,
                diagnostics: None,
            },
            Value::Surface {
                vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
                faces: vec![vec![0, 1, 2]],
            },
        ]);

        let meshes = coerce_mesh_list(&value, "test").unwrap();
        assert_eq!(meshes.len(), 2);
        assert_eq!(meshes[0].vertex_count(), 3);
        assert_eq!(meshes[1].vertex_count(), 3);
    }

    #[test]
    fn coerce_mesh_list_wraps_single_mesh() {
        let value = Value::Mesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: None,
            uvs: None,
            diagnostics: None,
        };

        let meshes = coerce_mesh_list(&value, "test").unwrap();
        assert_eq!(meshes.len(), 1);
        assert_eq!(meshes[0].vertex_count(), 3);
    }

    #[test]
    fn mesh_into_mesh_data() {
        let value = Value::Mesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: Some(vec![[0.0, 0.0, 1.0]; 3]),
            uvs: None,
            diagnostics: None,
        };

        let mesh = coerce_mesh(&value).unwrap();
        let mesh_data = mesh.into_mesh_data();

        assert_eq!(mesh_data.vertex_count(), 3);
        assert_eq!(mesh_data.triangle_count(), 1);
        assert!(mesh_data.has_normals());
        assert!(mesh_data.diagnostics.is_none());
    }

    // ========================================================================
    // Tests for Surface coercion functions
    // ========================================================================

    #[test]
    fn coerce_surface_accepts_surface_value() {
        let value = Value::Surface {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            faces: vec![vec![0, 1, 2], vec![0, 2, 3]],
        };

        let surface = coerce_surface(&value).unwrap();
        assert_eq!(surface.vertices.len(), 4);
        assert_eq!(surface.faces.len(), 2);
    }

    #[test]
    fn coerce_surface_rejects_mesh() {
        let value = Value::Mesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: None,
            uvs: None,
            diagnostics: None,
        };

        let result = coerce_surface(&value);
        assert!(result.is_err());
    }

    #[test]
    fn coerce_surface_like_accepts_surface() {
        let value = Value::Surface {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            faces: vec![vec![0, 1, 2], vec![0, 2, 3]],
        };

        let surface = coerce_surface_like(&value).unwrap();
        assert_eq!(surface.vertex_count(), 4);
        assert_eq!(surface.face_count(), 2);
    }

    #[test]
    fn coerce_surface_like_accepts_mesh() {
        let value = Value::Mesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: Some(vec![[0.0, 0.0, 1.0]; 3]),
            uvs: None,
            diagnostics: None,
        };

        let surface = coerce_surface_like(&value).unwrap();
        assert_eq!(surface.vertex_count(), 3);
        assert_eq!(surface.face_count(), 1);
        assert_eq!(surface.faces[0], vec![0, 1, 2]);
    }

    #[test]
    fn coerce_surface_like_converts_mesh_triangles_to_faces() {
        let value = Value::Mesh {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            indices: vec![0, 1, 2, 0, 2, 3], // Two triangles
            normals: None,
            uvs: None,
            diagnostics: None,
        };

        let surface = coerce_surface_like(&value).unwrap();
        assert_eq!(surface.vertex_count(), 4);
        assert_eq!(surface.face_count(), 2);
        assert_eq!(surface.faces[0], vec![0, 1, 2]);
        assert_eq!(surface.faces[1], vec![0, 2, 3]);
    }

    #[test]
    fn coerce_surface_like_unwraps_single_element_list() {
        let value = Value::List(vec![Value::Surface {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            faces: vec![vec![0, 1, 2]],
        }]);

        let surface = coerce_surface_like(&value).unwrap();
        assert_eq!(surface.vertex_count(), 3);
        assert_eq!(surface.face_count(), 1);
    }

    #[test]
    fn coerce_surface_list_accepts_list_of_surfaces() {
        let value = Value::List(vec![
            Value::Surface {
                vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
                faces: vec![vec![0, 1, 2]],
            },
            Value::Mesh {
                vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
                indices: vec![0, 1, 2],
                normals: None,
                uvs: None,
                diagnostics: None,
            },
        ]);

        let surfaces = coerce_surface_list(&value, "test").unwrap();
        assert_eq!(surfaces.len(), 2);
        assert_eq!(surfaces[0].vertex_count(), 3);
        assert_eq!(surfaces[1].vertex_count(), 3);
    }

    #[test]
    fn coerce_surface_list_wraps_single_surface() {
        let value = Value::Surface {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            faces: vec![vec![0, 1, 2]],
        };

        let surfaces = coerce_surface_list(&value, "test").unwrap();
        assert_eq!(surfaces.len(), 1);
        assert_eq!(surfaces[0].vertex_count(), 3);
    }

    #[test]
    fn surface_owned_into_value() {
        let surface = SurfaceOwned::new(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![vec![0, 1, 2]],
        );

        let value = surface.into_value();
        if let Value::Surface { vertices, faces } = value {
            assert_eq!(vertices.len(), 3);
            assert_eq!(faces.len(), 1);
        } else {
            panic!("Expected Value::Surface");
        }
    }

    #[test]
    fn mesh_into_surface_owned() {
        let mesh = Mesh::new(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![0, 1, 2],
        );

        let surface = mesh.into_surface_owned();
        assert_eq!(surface.vertex_count(), 3);
        assert_eq!(surface.face_count(), 1);
        assert_eq!(surface.faces[0], vec![0, 1, 2]);
    }

    #[test]
    fn mesh_into_surface_legacy() {
        let mesh = Mesh::new(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![0, 1, 2],
        );

        let value = mesh.into_surface_legacy();
        if let Value::Surface { vertices, faces } = value {
            assert_eq!(vertices.len(), 3);
            assert_eq!(faces.len(), 1);
            assert_eq!(faces[0], vec![0, 1, 2]);
        } else {
            panic!("Expected Value::Surface");
        }
    }

    #[test]
    fn mesh_into_value() {
        let mesh = Mesh::with_attributes(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![0, 1, 2],
            Some(vec![[0.0, 0.0, 1.0]; 3]),
            Some(vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]]),
        );

        let value = mesh.into_value();
        if let Value::Mesh { vertices, indices, normals, uvs, .. } = value {
            assert_eq!(vertices.len(), 3);
            assert_eq!(indices, vec![0, 1, 2]);
            assert!(normals.is_some());
            assert!(uvs.is_some());
        } else {
            panic!("Expected Value::Mesh");
        }
    }

    #[test]
    fn mesh_validation_catches_bad_indices() {
        let mesh = Mesh::new(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![0, 1, 10], // Index 10 is out of bounds
        );

        let result = mesh.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("out of bounds"));
    }

    #[test]
    fn mesh_validation_catches_non_triangle_indices() {
        let mesh = Mesh::new(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![0, 1], // Not divisible by 3
        );

        let result = mesh.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not divisible by 3"));
    }

    #[test]
    fn mesh_validation_catches_nan_vertices() {
        let mesh = Mesh::new(
            vec![[0.0, f64::NAN, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![0, 1, 2],
        );

        let result = mesh.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("NaN or Inf"));
    }

    #[test]
    fn mesh_validation_catches_mismatched_normals() {
        let mesh = Mesh::with_attributes(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![0, 1, 2],
            Some(vec![[0.0, 0.0, 1.0]; 2]), // Only 2 normals for 3 vertices
            None,
        );

        let result = mesh.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("normals length"));
    }

    #[test]
    fn mesh_is_empty_checks_vertices_and_indices() {
        let empty_vertices = Mesh::new(vec![], vec![0, 1, 2]);
        assert!(empty_vertices.is_empty());

        let empty_indices = Mesh::new(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![],
        );
        assert!(empty_indices.is_empty());

        let valid_mesh = Mesh::new(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![0, 1, 2],
        );
        assert!(!valid_mesh.is_empty());
    }

    #[test]
    fn mesh_from_mesh_data() {
        let data = MeshData::new(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![0, 1, 2],
        );

        let mesh = Mesh::from_mesh_data(data);
        assert_eq!(mesh.vertex_count(), 3);
        assert_eq!(mesh.triangle_count(), 1);
    }
}

pub fn coerce_vector_with_default(value: Option<&Value>) -> [f64; 3] {
    match value {
        Some(Value::Vector(v)) => *v,
        Some(Value::Point(p)) => *p,
        _ => [0.0, 0.0, 1.0],
    }
}

pub fn coerce_plane_with_default(value: Option<&Value>) -> PlaneValue {
    if let Some(value) = value {
        if let Value::List(l) = value {
            if l.len() >= 3 {
                if let (Ok(p1), Ok(p2), Ok(p3)) = (
                    coerce_point(&l[0]),
                    coerce_point(&l[1]),
                    coerce_point(&l[2]),
                ) {
                    let ab = [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]];
                    let ac = [p3[0] - p1[0], p3[1] - p1[1], p3[2] - p1[2]];
                    let z_axis = [
                        ab[1] * ac[2] - ab[2] * ac[1],
                        ab[2] * ac[0] - ab[0] * ac[2],
                        ab[0] * ac[1] - ab[1] * ac[0],
                    ];
                    let x_axis = ab;
                    let y_axis = [
                        z_axis[1] * x_axis[2] - z_axis[2] * x_axis[1],
                        z_axis[2] * x_axis[0] - z_axis[0] * x_axis[2],
                        z_axis[0] * x_axis[1] - z_axis[1] * x_axis[0],
                    ];
                    return PlaneValue::new(p1, x_axis, y_axis, z_axis);
                }
            }
        }
    }
    PlaneValue::new(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    )
}
