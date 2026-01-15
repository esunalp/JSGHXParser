//! Basis Value-enum waarin componentwaarden en -resultaten worden
//! opgeslagen.

use core::fmt;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use num_complex::Complex;
use time::PrimitiveDateTime;

use super::node::{MetaLookupExt, MetaMap, MetaValue};

/// Een complex getal, aliased van `num_complex::Complex`.
pub type ComplexValue = Complex<f64>;

// ============================================================================
// MeshQuality - Mesh tessellation quality configuration
// ============================================================================

/// Configuration for mesh tessellation quality.
///
/// Controls the resolution and accuracy of mesh generation from curves/surfaces.
/// Components that generate meshes (loft, sweep, extrude, etc.) accept these
/// parameters either directly or via `MetaMap` configuration.
///
/// # Defaults
///
/// The default configuration provides a balance between quality and performance:
/// - `max_edge_length`: 1.0 (world units)
/// - `max_deviation`: 0.01 (distance from ideal surface)
/// - `angle_threshold_degrees`: 15.0 (max angle between adjacent faces)
/// - `min_subdivisions`: 4 (minimum grid density)
/// - `max_subdivisions`: 256 (prevents runaway refinement)
///
/// # Presets
///
/// Use the preset constructors for common use cases:
/// - [`MeshQuality::low()`]: Fast previews, coarse meshes
/// - [`MeshQuality::medium()`]: Balanced quality (default)
/// - [`MeshQuality::high()`]: Fine detail, slower generation
/// - [`MeshQuality::ultra()`]: Maximum quality for final output
///
/// # Example
///
/// ```ignore
/// let quality = MeshQuality::high()
///     .with_max_edge_length(0.5);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct MeshQuality {
    /// Maximum edge length for tessellation (world units).
    ///
    /// Smaller values produce more triangles and finer detail.
    /// Set to `f64::INFINITY` to disable edge-length subdivision.
    pub max_edge_length: f64,

    /// Maximum deviation from the ideal surface (world units).
    ///
    /// Controls how closely the mesh approximates curved surfaces.
    /// Smaller values produce smoother curves but more triangles.
    pub max_deviation: f64,

    /// Angle threshold for adaptive subdivision (degrees).
    ///
    /// When adjacent face normals differ by more than this angle,
    /// additional subdivision is applied. Range: 1.0 to 90.0.
    pub angle_threshold_degrees: f64,

    /// Minimum number of subdivisions along each axis.
    ///
    /// Ensures a baseline mesh density even for simple surfaces.
    pub min_subdivisions: usize,

    /// Maximum number of subdivisions along each axis.
    ///
    /// Prevents runaway refinement on complex surfaces.
    pub max_subdivisions: usize,
}

impl Default for MeshQuality {
    fn default() -> Self {
        Self::medium()
    }
}

impl MeshQuality {
    /// Creates a new `MeshQuality` with specified parameters.
    #[must_use]
    pub fn new(
        max_edge_length: f64,
        max_deviation: f64,
        angle_threshold_degrees: f64,
        min_subdivisions: usize,
        max_subdivisions: usize,
    ) -> Self {
        Self {
            max_edge_length: max_edge_length.max(0.001),
            max_deviation: max_deviation.max(0.0001),
            angle_threshold_degrees: angle_threshold_degrees.clamp(1.0, 90.0),
            min_subdivisions: min_subdivisions.max(1),
            max_subdivisions: max_subdivisions.max(min_subdivisions).min(4096),
        }
    }

    /// Low quality preset: fast previews, coarse meshes.
    #[must_use]
    pub fn low() -> Self {
        Self {
            max_edge_length: 5.0,
            max_deviation: 0.1,
            angle_threshold_degrees: 30.0,
            min_subdivisions: 2,
            max_subdivisions: 64,
        }
    }

    /// Medium quality preset: balanced quality and performance (default).
    #[must_use]
    pub fn medium() -> Self {
        Self {
            max_edge_length: 1.0,
            max_deviation: 0.01,
            angle_threshold_degrees: 15.0,
            min_subdivisions: 4,
            max_subdivisions: 256,
        }
    }

    /// High quality preset: fine detail, suitable for rendering.
    #[must_use]
    pub fn high() -> Self {
        Self {
            max_edge_length: 0.25,
            max_deviation: 0.001,
            angle_threshold_degrees: 8.0,
            min_subdivisions: 8,
            max_subdivisions: 512,
        }
    }

    /// Ultra quality preset: maximum quality for final output.
    #[must_use]
    pub fn ultra() -> Self {
        Self {
            max_edge_length: 0.1,
            max_deviation: 0.0001,
            angle_threshold_degrees: 4.0,
            min_subdivisions: 16,
            max_subdivisions: 1024,
        }
    }

    /// Returns a modified copy with a new `max_edge_length`.
    #[must_use]
    pub fn with_max_edge_length(mut self, value: f64) -> Self {
        self.max_edge_length = value.max(0.001);
        self
    }

    /// Returns a modified copy with a new `max_deviation`.
    #[must_use]
    pub fn with_max_deviation(mut self, value: f64) -> Self {
        self.max_deviation = value.max(0.0001);
        self
    }

    /// Returns a modified copy with a new `angle_threshold_degrees`.
    #[must_use]
    pub fn with_angle_threshold(mut self, degrees: f64) -> Self {
        self.angle_threshold_degrees = degrees.clamp(1.0, 90.0);
        self
    }

    /// Returns a modified copy with new subdivision limits.
    #[must_use]
    pub fn with_subdivisions(mut self, min: usize, max: usize) -> Self {
        self.min_subdivisions = min.max(1);
        self.max_subdivisions = max.max(self.min_subdivisions).min(4096);
        self
    }

    // ========================================================================
    // MetaMap Parsing
    // ========================================================================

    /// Creates a `MeshQuality` from preset name string.
    ///
    /// Supported preset names (case-insensitive):
    /// - `"low"` / `"preview"` / `"draft"` → [`MeshQuality::low()`]
    /// - `"medium"` / `"default"` / `"normal"` → [`MeshQuality::medium()`]
    /// - `"high"` / `"fine"` / `"quality"` → [`MeshQuality::high()`]
    /// - `"ultra"` / `"max"` / `"maximum"` → [`MeshQuality::ultra()`]
    ///
    /// Returns `None` if the preset name is not recognized.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let quality = MeshQuality::from_preset_name("high").unwrap();
    /// assert_eq!(quality.max_edge_length, 0.25);
    /// ```
    #[must_use]
    pub fn from_preset_name(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "low" | "preview" | "draft" | "coarse" => Some(Self::low()),
            "medium" | "default" | "normal" | "standard" => Some(Self::medium()),
            "high" | "fine" | "quality" | "detailed" => Some(Self::high()),
            "ultra" | "max" | "maximum" | "best" => Some(Self::ultra()),
            _ => None,
        }
    }

    /// Creates a `MeshQuality` by parsing a `MetaMap`.
    ///
    /// This method extracts mesh quality settings from component metadata,
    /// allowing Grasshopper components to configure tessellation quality
    /// via their parameter pins.
    ///
    /// # Supported Keys
    ///
    /// All keys are case-insensitive. If a key is not present, its default
    /// value from [`MeshQuality::medium()`] is used.
    ///
    /// | Key | Type | Description |
    /// |-----|------|-------------|
    /// | `mesh_quality` / `quality` / `preset` | Text | Preset name (see [`from_preset_name`]) |
    /// | `max_edge_length` / `edge_length` | Number | Maximum edge length |
    /// | `max_deviation` / `deviation` / `tolerance` | Number | Maximum surface deviation |
    /// | `angle_threshold` / `angle` | Number | Angle threshold in degrees |
    /// | `min_subdivisions` / `min_subdiv` | Number/Integer | Minimum subdivisions |
    /// | `max_subdivisions` / `max_subdiv` | Number/Integer | Maximum subdivisions |
    ///
    /// # Resolution Order
    ///
    /// 1. If a preset is specified, start with that preset's values
    /// 2. Override with any explicitly specified individual parameters
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut meta = MetaMap::new();
    /// meta.insert("mesh_quality".to_string(), MetaValue::Text("high".to_string()));
    /// meta.insert("max_edge_length".to_string(), MetaValue::Number(0.5));
    ///
    /// let quality = MeshQuality::from_meta(&meta);
    /// // Uses "high" preset but overrides max_edge_length to 0.5
    /// assert_eq!(quality.max_edge_length, 0.5);
    /// assert_eq!(quality.max_deviation, 0.001); // From "high" preset
    /// ```
    #[must_use]
    pub fn from_meta(meta: &MetaMap) -> Self {
        // Start with the default or a specified preset
        let mut result = Self::extract_preset(meta).unwrap_or_default();

        // Override with explicitly specified values
        if let Some(edge_length) = Self::extract_number(meta, &["max_edge_length", "edge_length", "edgelength"]) {
            result.max_edge_length = edge_length.max(0.001);
        }

        if let Some(deviation) = Self::extract_number(meta, &["max_deviation", "deviation", "tolerance"]) {
            result.max_deviation = deviation.max(0.0001);
        }

        if let Some(angle) = Self::extract_number(meta, &["angle_threshold", "angle", "angle_degrees"]) {
            result.angle_threshold_degrees = angle.clamp(1.0, 90.0);
        }

        if let Some(min_subdiv) = Self::extract_usize(meta, &["min_subdivisions", "min_subdiv", "minsubdiv"]) {
            result.min_subdivisions = min_subdiv.max(1);
        }

        if let Some(max_subdiv) = Self::extract_usize(meta, &["max_subdivisions", "max_subdiv", "maxsubdiv"]) {
            result.max_subdivisions = max_subdiv.max(result.min_subdivisions).min(4096);
        }

        result
    }

    /// Creates a `MeshQuality` from a `MetaMap`, falling back to default if empty.
    ///
    /// This is a convenience wrapper around [`from_meta`] that handles the common
    /// pattern of extracting mesh quality from component metadata with sensible
    /// defaults when no quality parameters are specified.
    ///
    /// # Example
    ///
    /// ```ignore
    /// impl Component for LoftComponent {
    ///     fn evaluate(&self, inputs: &[Value], meta: &MetaMap) -> ComponentResult {
    ///         let quality = MeshQuality::from_meta_or_default(meta);
    ///         // ... use quality for tessellation
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn from_meta_or_default(meta: &MetaMap) -> Self {
        Self::from_meta(meta)
    }

    /// Attempts to parse a `MeshQuality` from a `Value`.
    ///
    /// Supported conversions:
    /// - `Value::Text(preset_name)` → [`from_preset_name`]
    /// - `Value::Number(preset_index)` → 0=low, 1=medium, 2=high, 3=ultra
    ///
    /// Returns `None` if the value cannot be converted.
    #[must_use]
    pub fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Text(name) => Self::from_preset_name(name),
            Value::Number(n) => {
                let index = *n as i32;
                match index {
                    0 => Some(Self::low()),
                    1 => Some(Self::medium()),
                    2 => Some(Self::high()),
                    3 => Some(Self::ultra()),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    // ========================================================================
    // Private Helpers for MetaMap Parsing
    // ========================================================================

    /// Extracts a preset from the MetaMap by checking common key names.
    fn extract_preset(meta: &MetaMap) -> Option<Self> {
        const PRESET_KEYS: &[&str] = &["mesh_quality", "quality", "preset", "mesh_preset"];

        for key in PRESET_KEYS {
            if let Some(meta_value) = meta.get_normalized(key) {
                match meta_value {
                    MetaValue::Text(name) => {
                        if let Some(preset) = Self::from_preset_name(name) {
                            return Some(preset);
                        }
                    }
                    MetaValue::Integer(index) => {
                        return Self::from_index(*index as i32);
                    }
                    MetaValue::Number(index) => {
                        return Self::from_index(*index as i32);
                    }
                    MetaValue::List(list) if !list.is_empty() => {
                        // Handle single-element list wrapping
                        if let MetaValue::Text(name) = &list[0] {
                            if let Some(preset) = Self::from_preset_name(name) {
                                return Some(preset);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        None
    }

    /// Converts an index to a preset.
    fn from_index(index: i32) -> Option<Self> {
        match index {
            0 => Some(Self::low()),
            1 => Some(Self::medium()),
            2 => Some(Self::high()),
            3 => Some(Self::ultra()),
            _ => None,
        }
    }

    /// Extracts a numeric value from the MetaMap, checking multiple key aliases.
    fn extract_number(meta: &MetaMap, keys: &[&str]) -> Option<f64> {
        for key in keys {
            if let Some(meta_value) = meta.get_normalized(key) {
                match meta_value {
                    MetaValue::Number(n) => return Some(*n),
                    MetaValue::Integer(i) => return Some(*i as f64),
                    MetaValue::List(list) if !list.is_empty() => {
                        // Handle single-element list wrapping
                        match &list[0] {
                            MetaValue::Number(n) => return Some(*n),
                            MetaValue::Integer(i) => return Some(*i as f64),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
        None
    }

    /// Extracts a usize value from the MetaMap, checking multiple key aliases.
    fn extract_usize(meta: &MetaMap, keys: &[&str]) -> Option<usize> {
        Self::extract_number(meta, keys).map(|n| n.max(0.0) as usize)
    }
}

impl fmt::Display for MeshQuality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MeshQuality(edge:{:.3} dev:{:.4} angle:{:.1}° subdiv:{}-{})",
            self.max_edge_length,
            self.max_deviation,
            self.angle_threshold_degrees,
            self.min_subdivisions,
            self.max_subdivisions
        )
    }
}

// ============================================================================
// MeshDiagnostics - Mesh generation diagnostics for the graph layer
// ============================================================================

/// Diagnostics information for mesh generation and repair operations.
///
/// This struct captures topology metrics, quality issues, and repair statistics
/// from mesh generation. It is stored in `Value::Mesh` and can be inspected
/// to determine mesh quality and identify problems.
///
/// # Topology Metrics
///
/// - `open_edge_count`: Number of edges with only one adjacent triangle (holes)
/// - `non_manifold_edge_count`: Edges with more than two adjacent triangles
///
/// # Quality Metrics
///
/// - `degenerate_triangle_count`: Zero-area triangles removed during generation
/// - `self_intersection_count`: Self-intersecting triangle pairs detected
///
/// # Repair Statistics
///
/// - `welded_vertex_count`: Vertices merged during tolerance-based welding
/// - `flipped_triangle_count`: Triangles with corrected winding order
/// - `boolean_fallback_used`: Whether CSG required fallback strategies
///
/// # Example
///
/// ```ignore
/// if let Value::Mesh { diagnostics, .. } = &mesh_value {
///     if let Some(diag) = diagnostics {
///         if !diag.is_watertight() {
///             eprintln!("Warning: mesh has {} open edges", diag.open_edge_count);
///         }
///     }
/// }
/// ```
#[derive(Debug, Default, Clone, PartialEq)]
pub struct MeshDiagnostics {
    /// Total number of vertices in the final mesh.
    pub vertex_count: usize,

    /// Total number of triangles in the final mesh.
    pub triangle_count: usize,

    /// Number of vertices merged during tolerance-based welding.
    pub welded_vertex_count: usize,

    /// Number of triangles whose winding order was corrected.
    pub flipped_triangle_count: usize,

    /// Number of degenerate (zero-area) triangles removed.
    pub degenerate_triangle_count: usize,

    /// Number of open (boundary) edges in the mesh.
    pub open_edge_count: usize,

    /// Number of non-manifold edges in the mesh.
    pub non_manifold_edge_count: usize,

    /// Number of self-intersecting triangle pairs detected.
    pub self_intersection_count: usize,

    /// Whether a boolean operation required a fallback strategy.
    pub boolean_fallback_used: bool,

    /// Human-readable warnings about mesh issues and repairs performed.
    pub warnings: Vec<String>,
}

impl MeshDiagnostics {
    /// Creates a new empty diagnostics struct with all counts at zero.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if the mesh is watertight (no open edges).
    #[must_use]
    pub fn is_watertight(&self) -> bool {
        self.open_edge_count == 0
    }

    /// Returns `true` if the mesh is manifold (no non-manifold edges).
    #[must_use]
    pub fn is_manifold(&self) -> bool {
        self.non_manifold_edge_count == 0
    }

    /// Returns `true` if the mesh is both watertight and manifold.
    #[must_use]
    pub fn is_valid_solid(&self) -> bool {
        self.is_watertight() && self.is_manifold()
    }

    /// Returns `true` if no issues were detected and no repairs were needed.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.open_edge_count == 0
            && self.non_manifold_edge_count == 0
            && self.degenerate_triangle_count == 0
            && self.flipped_triangle_count == 0
            && self.self_intersection_count == 0
            && !self.boolean_fallback_used
            && self.warnings.is_empty()
    }

    /// Returns `true` if any warnings were recorded.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Adds a warning message to the diagnostics.
    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }

    /// Returns the total number of topology issues (open + non-manifold edges).
    ///
    /// This is useful for quickly checking if the mesh has any topology problems
    /// that would prevent it from being used as a valid solid.
    #[must_use]
    pub fn topology_issue_count(&self) -> usize {
        self.open_edge_count + self.non_manifold_edge_count
    }

    /// Returns the total number of repairs performed during mesh generation.
    ///
    /// Includes:
    /// - Welded vertices (merged duplicates)
    /// - Flipped triangles (corrected winding order)
    /// - Degenerate triangles removed
    #[must_use]
    pub fn repair_count(&self) -> usize {
        self.welded_vertex_count + self.flipped_triangle_count + self.degenerate_triangle_count
    }

    /// Returns a short summary string suitable for logging.
    #[must_use]
    pub fn summary(&self) -> String {
        let mut parts = vec![format!("V:{} T:{}", self.vertex_count, self.triangle_count)];

        if self.welded_vertex_count > 0 {
            parts.push(format!("welded:{}", self.welded_vertex_count));
        }
        if self.flipped_triangle_count > 0 {
            parts.push(format!("flipped:{}", self.flipped_triangle_count));
        }
        if self.degenerate_triangle_count > 0 {
            parts.push(format!("degenerate:{}", self.degenerate_triangle_count));
        }
        if self.open_edge_count > 0 {
            parts.push(format!("open:{}", self.open_edge_count));
        }
        if self.non_manifold_edge_count > 0 {
            parts.push(format!("non-manifold:{}", self.non_manifold_edge_count));
        }
        if self.self_intersection_count > 0 {
            parts.push(format!("self-intersect:{}", self.self_intersection_count));
        }
        if self.boolean_fallback_used {
            parts.push("boolean-fallback".to_string());
        }

        parts.join(" ")
    }

    /// Merges another diagnostics struct into this one.
    pub fn merge(&mut self, other: &MeshDiagnostics) {
        self.vertex_count += other.vertex_count;
        self.triangle_count += other.triangle_count;
        self.welded_vertex_count += other.welded_vertex_count;
        self.flipped_triangle_count += other.flipped_triangle_count;
        self.degenerate_triangle_count += other.degenerate_triangle_count;
        self.open_edge_count += other.open_edge_count;
        self.non_manifold_edge_count += other.non_manifold_edge_count;
        self.self_intersection_count += other.self_intersection_count;
        self.boolean_fallback_used = self.boolean_fallback_used || other.boolean_fallback_used;
        self.warnings.extend(other.warnings.iter().cloned());
    }

    // ========================================================================
    // Value Deserialization Helpers
    // ========================================================================

    /// Attempts to create a `MeshDiagnostics` from a `Value::List`.
    ///
    /// This parses the list of key-value pairs produced by the `From<MeshDiagnostics> for Value`
    /// implementation. Missing keys will use default values.
    ///
    /// # Format
    ///
    /// Expects a `Value::List` where each entry is `[key: Text, value: Number/Boolean]`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let diag_value: Value = original_diag.into();
    /// let parsed = MeshDiagnostics::from_value(&diag_value)?;
    /// ```
    pub fn from_value(value: &Value) -> Result<Self, ValueError> {
        let list = value.expect_list()?;
        let mut result = Self::default();

        for entry in list {
            if let Value::List(pair) = entry {
                if pair.len() >= 2 {
                    if let Value::Text(key) = &pair[0] {
                        match key.as_str() {
                            "vertex_count" => {
                                if let Value::Number(n) = &pair[1] {
                                    result.vertex_count = *n as usize;
                                }
                            }
                            "triangle_count" => {
                                if let Value::Number(n) = &pair[1] {
                                    result.triangle_count = *n as usize;
                                }
                            }
                            "open_edge_count" => {
                                if let Value::Number(n) = &pair[1] {
                                    result.open_edge_count = *n as usize;
                                }
                            }
                            "non_manifold_edge_count" => {
                                if let Value::Number(n) = &pair[1] {
                                    result.non_manifold_edge_count = *n as usize;
                                }
                            }
                            "degenerate_triangle_count" => {
                                if let Value::Number(n) = &pair[1] {
                                    result.degenerate_triangle_count = *n as usize;
                                }
                            }
                            "welded_vertex_count" => {
                                if let Value::Number(n) = &pair[1] {
                                    result.welded_vertex_count = *n as usize;
                                }
                            }
                            "flipped_triangle_count" => {
                                if let Value::Number(n) = &pair[1] {
                                    result.flipped_triangle_count = *n as usize;
                                }
                            }
                            "self_intersection_count" => {
                                if let Value::Number(n) = &pair[1] {
                                    result.self_intersection_count = *n as usize;
                                }
                            }
                            "boolean_fallback_used" => {
                                if let Value::Boolean(b) = &pair[1] {
                                    result.boolean_fallback_used = *b;
                                }
                            }
                            "warnings" => {
                                if let Value::List(warnings) = &pair[1] {
                                    result.warnings = warnings
                                        .iter()
                                        .filter_map(|v| {
                                            if let Value::Text(s) = v {
                                                Some(s.clone())
                                            } else {
                                                None
                                            }
                                        })
                                        .collect();
                                }
                            }
                            // Ignore computed/read-only fields like is_watertight, is_manifold, is_valid_solid
                            _ => {}
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    /// Creates a `MeshDiagnostics` with vertex and triangle counts set.
    ///
    /// This is a convenience constructor for creating diagnostics with
    /// the basic mesh stats already filled in.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let diag = MeshDiagnostics::with_counts(100, 50);
    /// assert_eq!(diag.vertex_count, 100);
    /// assert_eq!(diag.triangle_count, 50);
    /// ```
    #[must_use]
    pub fn with_counts(vertex_count: usize, triangle_count: usize) -> Self {
        Self {
            vertex_count,
            triangle_count,
            ..Default::default()
        }
    }

    /// Returns a copy with updated counts.
    ///
    /// This is useful when the mesh has been modified and you need to
    /// update the diagnostics with new vertex/triangle counts.
    #[must_use]
    pub fn with_updated_counts(mut self, vertex_count: usize, triangle_count: usize) -> Self {
        self.vertex_count = vertex_count;
        self.triangle_count = triangle_count;
        self
    }
}

impl fmt::Display for MeshDiagnostics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MeshDiagnostics[{}]", self.summary())
    }
}

// ============================================================================
// GeomMeshDiagnostics Conversion (mesh_engine_next feature)
// ============================================================================

/// Converts from `geom::GeomMeshDiagnostics` to `MeshDiagnostics`.
///
/// This implementation is only available when the `mesh_engine_next` feature
/// is enabled. It allows seamless integration between the new geometry kernel
/// and the graph layer.
///
/// # Note
///
/// The `timing` field from `GeomMeshDiagnostics` is not preserved in
/// `MeshDiagnostics` as timing information is considered internal to the
/// geometry engine. If timing data is needed, it should be collected
/// separately via `GeomMetrics`.
#[cfg(feature = "mesh_engine_next")]
impl From<crate::geom::GeomMeshDiagnostics> for MeshDiagnostics {
    fn from(geom_diag: crate::geom::GeomMeshDiagnostics) -> Self {
        Self {
            vertex_count: geom_diag.vertex_count,
            triangle_count: geom_diag.triangle_count,
            welded_vertex_count: geom_diag.welded_vertex_count,
            flipped_triangle_count: geom_diag.flipped_triangle_count,
            degenerate_triangle_count: geom_diag.degenerate_triangle_count,
            open_edge_count: geom_diag.open_edge_count,
            non_manifold_edge_count: geom_diag.non_manifold_edge_count,
            self_intersection_count: geom_diag.self_intersection_count,
            boolean_fallback_used: geom_diag.boolean_fallback_used,
            warnings: geom_diag.warnings,
        }
    }
}

/// Converts from `MeshDiagnostics` to `geom::GeomMeshDiagnostics`.
///
/// This implementation is only available when the `mesh_engine_next` feature
/// is enabled. It allows passing graph-layer diagnostics back into geometry
/// operations when needed.
///
/// # Note
///
/// The `timing` field in the resulting `GeomMeshDiagnostics` will be `None`
/// since `MeshDiagnostics` does not track timing information.
#[cfg(feature = "mesh_engine_next")]
impl From<MeshDiagnostics> for crate::geom::GeomMeshDiagnostics {
    fn from(diag: MeshDiagnostics) -> Self {
        Self {
            vertex_count: diag.vertex_count,
            triangle_count: diag.triangle_count,
            welded_vertex_count: diag.welded_vertex_count,
            flipped_triangle_count: diag.flipped_triangle_count,
            degenerate_triangle_count: diag.degenerate_triangle_count,
            open_edge_count: diag.open_edge_count,
            non_manifold_edge_count: diag.non_manifold_edge_count,
            self_intersection_count: diag.self_intersection_count,
            boolean_fallback_used: diag.boolean_fallback_used,
            timing: None, // Timing is not preserved in MeshDiagnostics
            warnings: diag.warnings,
        }
    }
}

/// Converts from a reference to `geom::GeomMeshDiagnostics` to `MeshDiagnostics`.
///
/// This is useful when you need to convert without consuming the original.
#[cfg(feature = "mesh_engine_next")]
impl From<&crate::geom::GeomMeshDiagnostics> for MeshDiagnostics {
    fn from(geom_diag: &crate::geom::GeomMeshDiagnostics) -> Self {
        Self {
            vertex_count: geom_diag.vertex_count,
            triangle_count: geom_diag.triangle_count,
            welded_vertex_count: geom_diag.welded_vertex_count,
            flipped_triangle_count: geom_diag.flipped_triangle_count,
            degenerate_triangle_count: geom_diag.degenerate_triangle_count,
            open_edge_count: geom_diag.open_edge_count,
            non_manifold_edge_count: geom_diag.non_manifold_edge_count,
            self_intersection_count: geom_diag.self_intersection_count,
            boolean_fallback_used: geom_diag.boolean_fallback_used,
            warnings: geom_diag.warnings.clone(),
        }
    }
}

#[cfg(feature = "mesh_engine_next")]
impl MeshDiagnostics {
    /// Creates a `MeshDiagnostics` from a `GeomMeshDiagnostics`.
    ///
    /// This is an explicit conversion method that can be used instead of
    /// the `From` trait when you want the conversion to be more visible
    /// in the code.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let geom_diag = geom::mesh_surface(&surface, 10, 10).1;
    /// let diag = MeshDiagnostics::from_geom(geom_diag);
    /// ```
    #[must_use]
    pub fn from_geom(geom_diag: crate::geom::GeomMeshDiagnostics) -> Self {
        Self::from(geom_diag)
    }

    /// Creates a `MeshDiagnostics` from a reference to `GeomMeshDiagnostics`.
    ///
    /// Use this when you need to keep the original `GeomMeshDiagnostics`.
    #[must_use]
    pub fn from_geom_ref(geom_diag: &crate::geom::GeomMeshDiagnostics) -> Self {
        Self::from(geom_diag)
    }

    /// Converts this `MeshDiagnostics` to a `GeomMeshDiagnostics`.
    ///
    /// # Note
    ///
    /// The `timing` field in the resulting `GeomMeshDiagnostics` will be `None`.
    #[must_use]
    pub fn to_geom(self) -> crate::geom::GeomMeshDiagnostics {
        self.into()
    }
}

// ============================================================================
// MeshRef - Borrowed reference to mesh data (returned by expect_mesh)
// ============================================================================

/// Borrowed reference to mesh data from a `Value::Mesh`.
///
/// This struct holds references to the mesh buffers and is returned by
/// [`Value::expect_mesh()`]. Use this when you need read-only access to
/// mesh data without cloning.
///
/// # Example
///
/// ```ignore
/// let mesh_ref = value.expect_mesh()?;
/// println!("Vertex count: {}", mesh_ref.vertices.len());
/// if let Some(normals) = mesh_ref.normals {
///     println!("Has {} normals", normals.len());
/// }
/// ```
#[derive(Debug, Clone)]
pub struct MeshRef<'a> {
    /// Vertex positions as `[x, y, z]` arrays.
    pub vertices: &'a [[f64; 3]],
    /// Triangle indices (length divisible by 3).
    pub indices: &'a [u32],
    /// Optional per-vertex normals.
    pub normals: Option<&'a [[f64; 3]]>,
    /// Optional per-vertex UV coordinates.
    pub uvs: Option<&'a [[f64; 2]]>,
    /// Optional diagnostics about mesh quality.
    pub diagnostics: Option<&'a MeshDiagnostics>,
}

impl<'a> MeshRef<'a> {
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

    /// Converts to an owned `MeshData` by cloning all buffers.
    #[must_use]
    pub fn to_owned(&self) -> MeshData {
        MeshData {
            vertices: self.vertices.to_vec(),
            indices: self.indices.to_vec(),
            normals: self.normals.map(|n| n.to_vec()),
            uvs: self.uvs.map(|u| u.to_vec()),
            diagnostics: self.diagnostics.cloned(),
        }
    }
}

// ============================================================================
// Polygon Face Triangulation
// ============================================================================

/// Triangulates polygon faces into triangles using fan triangulation.
///
/// This function properly handles quads and n-gons by triangulating from the first
/// vertex as a fan, producing (n-2) triangles per n-gon. For 3-gon (triangle) faces,
/// indices are passed through unchanged.
///
/// # Fan Triangulation
///
/// For a polygon with vertices [v0, v1, v2, v3, ...], fan triangulation produces:
/// - Triangle 1: [v0, v1, v2]
/// - Triangle 2: [v0, v2, v3]
/// - Triangle 3: [v0, v3, v4]
/// - etc.
///
/// This is efficient and works well for convex polygons and most quads. For highly
/// non-convex polygons, more sophisticated ear-clipping may be needed, but fan
/// triangulation provides a reasonable approximation that preserves all geometry.
///
/// # Arguments
///
/// * `faces` - Slice of polygon faces, where each face is a list of vertex indices
///
/// # Returns
///
/// A flat vector of triangle indices.
///
/// # Example
///
/// ```ignore
/// // A quad face with vertices [0, 1, 2, 3] produces:
/// // Triangle 1: [0, 1, 2]
/// // Triangle 2: [0, 2, 3]
/// let faces = vec![vec![0, 1, 2, 3]];
/// let indices = triangulate_polygon_faces(&faces);
/// assert_eq!(indices, vec![0, 1, 2, 0, 2, 3]);
/// ```
#[must_use]
pub fn triangulate_polygon_faces(faces: &[Vec<u32>]) -> Vec<u32> {
    // Pre-calculate capacity: each face contributes (n-2) triangles * 3 indices
    let capacity: usize = faces
        .iter()
        .filter(|f| f.len() >= 3)
        .map(|f| (f.len() - 2) * 3)
        .sum();

    let mut indices = Vec::with_capacity(capacity);

    for face in faces {
        if face.len() < 3 {
            continue;
        }
        // Fan triangulation: all triangles share vertex 0
        // Triangle i uses vertices [0, i+1, i+2]
        let num_triangles = face.len() - 2;
        for i in 0..num_triangles {
            indices.push(face[0]);
            indices.push(face[i + 1]);
            indices.push(face[i + 2]);
        }
    }

    indices
}

/// Consumes polygon faces and triangulates them into triangles using fan triangulation.
///
/// This is the owned/consuming version of [`triangulate_polygon_faces`], suitable for
/// use with `into_iter()` when the face data is being moved.
///
/// # Arguments
///
/// * `faces` - Iterator of polygon faces to consume
///
/// # Returns
///
/// A flat vector of triangle indices.
#[must_use]
pub fn triangulate_polygon_faces_owned<I>(faces: I) -> Vec<u32>
where
    I: IntoIterator<Item = Vec<u32>>,
{
    let faces: Vec<Vec<u32>> = faces.into_iter().collect();
    triangulate_polygon_faces(&faces)
}

// ============================================================================
// MeshData - Owned mesh data (returned by expect_mesh_like and conversions)
// ============================================================================

/// Owned mesh data extracted from a `Value::Mesh` or converted from `Value::Surface`.
///
/// This struct owns its data and can be freely manipulated. Use this when you
/// need to modify mesh data or when the original `Value` may go out of scope.
///
/// # Example
///
/// ```ignore
/// let mesh_data = value.expect_mesh_like()?;
/// for vertex in &mesh_data.vertices {
///     println!("Vertex: {:?}", vertex);
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct MeshData {
    /// Vertex positions as `[x, y, z]` arrays.
    pub vertices: Vec<[f64; 3]>,
    /// Triangle indices (length divisible by 3).
    pub indices: Vec<u32>,
    /// Optional per-vertex normals.
    pub normals: Option<Vec<[f64; 3]>>,
    /// Optional per-vertex UV coordinates.
    pub uvs: Option<Vec<[f64; 2]>>,
    /// Optional diagnostics about mesh quality.
    pub diagnostics: Option<MeshDiagnostics>,
}

impl MeshData {
    /// Creates a new `MeshData` with only positions and indices.
    #[must_use]
    pub fn new(vertices: Vec<[f64; 3]>, indices: Vec<u32>) -> Self {
        Self {
            vertices,
            indices,
            normals: None,
            uvs: None,
            diagnostics: None,
        }
    }

    /// Creates a new `MeshData` with all attributes.
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
            diagnostics: None,
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

    /// Converts to a `Value::Mesh`.
    #[must_use]
    pub fn into_value(self) -> Value {
        Value::Mesh {
            vertices: self.vertices,
            indices: self.indices,
            normals: self.normals,
            uvs: self.uvs,
            diagnostics: self.diagnostics,
        }
    }

    /// Converts to a `Value::Surface` (legacy format).
    ///
    /// **Note**: This is a lossy conversion:
    /// - Normals and UVs are discarded
    /// - Each triangle becomes a separate face
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

    /// Validates the mesh data for consistency.
    ///
    /// Returns `Ok(())` if the mesh is valid, or an error message describing
    /// the first issue found.
    pub fn validate(&self) -> Result<(), String> {
        // Check triangle indices
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

    // ========================================================================
    // Serialization Helpers for WASM/three.js Integration
    // ========================================================================

    /// Returns vertices as a flat `Vec<f32>` for efficient WebGL buffer creation.
    ///
    /// Each vertex `[x, y, z]` is flattened into three consecutive floats.
    /// The resulting vector has length `vertices.len() * 3`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mesh = value.expect_mesh_like()?;
    /// let positions = mesh.to_vertices_f32(); // Vec<f32> for BufferGeometry
    /// ```
    #[must_use]
    pub fn to_vertices_f32(&self) -> Vec<f32> {
        self.vertices
            .iter()
            .flat_map(|v| [v[0] as f32, v[1] as f32, v[2] as f32])
            .collect()
    }

    /// Returns indices as a flat `Vec<u32>` (already in correct format).
    ///
    /// This is a convenience method that clones the indices.
    #[must_use]
    pub fn to_indices_u32(&self) -> Vec<u32> {
        self.indices.clone()
    }

    /// Returns normals as a flat `Vec<f32>` for efficient WebGL buffer creation.
    ///
    /// Each normal `[nx, ny, nz]` is flattened into three consecutive floats.
    /// Returns `None` if normals are not present.
    #[must_use]
    pub fn to_normals_f32(&self) -> Option<Vec<f32>> {
        self.normals.as_ref().map(|normals| {
            normals
                .iter()
                .flat_map(|n| [n[0] as f32, n[1] as f32, n[2] as f32])
                .collect()
        })
    }

    /// Returns UVs as a flat `Vec<f32>` for efficient WebGL buffer creation.
    ///
    /// Each UV `[u, v]` is flattened into two consecutive floats.
    /// Returns `None` if UVs are not present.
    #[must_use]
    pub fn to_uvs_f32(&self) -> Option<Vec<f32>> {
        self.uvs.as_ref().map(|uvs| {
            uvs.iter()
                .flat_map(|uv| [uv[0] as f32, uv[1] as f32])
                .collect()
        })
    }

    /// Returns a summary of the mesh suitable for debugging or logging.
    #[must_use]
    pub fn summary(&self) -> String {
        let mut parts = vec![format!(
            "V:{} T:{}",
            self.vertex_count(),
            self.triangle_count()
        )];

        if self.normals.is_some() {
            parts.push("+normals".to_string());
        }
        if self.uvs.is_some() {
            parts.push("+uvs".to_string());
        }
        if let Some(ref diag) = self.diagnostics {
            if !diag.is_clean() {
                parts.push(format!("diag:[{}]", diag.summary()));
            }
        }

        parts.join(" ")
    }

    // ========================================================================
    // Value Conversion Helpers
    // ========================================================================

    /// Attempts to create a `MeshData` from a `Value`.
    ///
    /// This method accepts either `Value::Mesh` or `Value::Surface` (legacy),
    /// returning owned mesh data in either case.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not a mesh-like type.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let value = Value::Mesh { vertices, indices, normals: None, uvs: None, diagnostics: None };
    /// let mesh_data = MeshData::from_value(&value)?;
    /// ```
    pub fn from_value(value: &Value) -> Result<Self, ValueError> {
        value.expect_mesh_like()
    }

    /// Attempts to create a `MeshData` from a `Value`, consuming it.
    ///
    /// This is more efficient than `from_value` when you own the value,
    /// as it avoids cloning the mesh data.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not a mesh-like type.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let value = Value::Mesh { vertices, indices, normals: None, uvs: None, diagnostics: None };
    /// let mesh_data = MeshData::from_value_owned(value)?;
    /// ```
    pub fn from_value_owned(value: Value) -> Result<Self, ValueError> {
        value.into_mesh_data_like()
    }

    /// Creates a `MeshData` with diagnostics attached.
    ///
    /// This is a convenience method for creating mesh data with diagnostics
    /// in a single call.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mesh = MeshData::with_diagnostics(
    ///     vertices,
    ///     indices,
    ///     Some(normals),
    ///     None, // no UVs
    ///     MeshDiagnostics { vertex_count: 3, triangle_count: 1, ..Default::default() },
    /// );
    /// ```
    #[must_use]
    pub fn with_diagnostics(
        vertices: Vec<[f64; 3]>,
        indices: Vec<u32>,
        normals: Option<Vec<[f64; 3]>>,
        uvs: Option<Vec<[f64; 2]>>,
        diagnostics: MeshDiagnostics,
    ) -> Self {
        Self {
            vertices,
            indices,
            normals,
            uvs,
            diagnostics: Some(diagnostics),
        }
    }

    /// Returns `true` if the mesh is empty (no vertices).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    /// Attaches diagnostics to the mesh data.
    ///
    /// This consumes the current `MeshData` and returns a new one with
    /// the provided diagnostics attached.
    #[must_use]
    pub fn with_diagnostics_attached(mut self, diagnostics: MeshDiagnostics) -> Self {
        self.diagnostics = Some(diagnostics);
        self
    }

    /// Sets the normals on the mesh data.
    ///
    /// This consumes the current `MeshData` and returns a new one with
    /// the provided normals.
    #[must_use]
    pub fn with_normals(mut self, normals: Vec<[f64; 3]>) -> Self {
        self.normals = Some(normals);
        self
    }

    /// Sets the UVs on the mesh data.
    ///
    /// This consumes the current `MeshData` and returns a new one with
    /// the provided UVs.
    #[must_use]
    pub fn with_uvs(mut self, uvs: Vec<[f64; 2]>) -> Self {
        self.uvs = Some(uvs);
        self
    }
}

// ============================================================================
// From Trait Implementations for MeshData and Value
// ============================================================================

impl From<MeshData> for Value {
    /// Converts owned `MeshData` into a `Value::Mesh`.
    ///
    /// This is the preferred way to create a `Value::Mesh` from mesh data.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let data = MeshData::new(vertices, indices);
    /// let value: Value = data.into();
    /// ```
    fn from(data: MeshData) -> Self {
        data.into_value()
    }
}

impl From<&MeshData> for Value {
    /// Converts a reference to `MeshData` into a `Value::Mesh` by cloning.
    fn from(data: &MeshData) -> Self {
        Value::Mesh {
            vertices: data.vertices.clone(),
            indices: data.indices.clone(),
            normals: data.normals.clone(),
            uvs: data.uvs.clone(),
            diagnostics: data.diagnostics.clone(),
        }
    }
}

impl From<MeshDiagnostics> for Value {
    /// Converts `MeshDiagnostics` into a `Value::List` representation.
    ///
    /// The diagnostics are serialized as a list of key-value pairs suitable
    /// for the diagnostics output pin on mesh-generating components.
    ///
    /// # Format
    ///
    /// Returns a `Value::List` containing:
    /// - `[vertex_count, triangle_count, open_edges, non_manifold_edges, ...]`
    ///
    /// # Example
    ///
    /// ```ignore
    /// let diag = MeshDiagnostics { vertex_count: 100, ... };
    /// let value: Value = diag.into();
    /// ```
    fn from(diag: MeshDiagnostics) -> Self {
        Value::List(vec![
            Value::List(vec![
                Value::Text("vertex_count".to_string()),
                Value::Number(diag.vertex_count as f64),
            ]),
            Value::List(vec![
                Value::Text("triangle_count".to_string()),
                Value::Number(diag.triangle_count as f64),
            ]),
            Value::List(vec![
                Value::Text("open_edge_count".to_string()),
                Value::Number(diag.open_edge_count as f64),
            ]),
            Value::List(vec![
                Value::Text("non_manifold_edge_count".to_string()),
                Value::Number(diag.non_manifold_edge_count as f64),
            ]),
            Value::List(vec![
                Value::Text("degenerate_triangle_count".to_string()),
                Value::Number(diag.degenerate_triangle_count as f64),
            ]),
            Value::List(vec![
                Value::Text("welded_vertex_count".to_string()),
                Value::Number(diag.welded_vertex_count as f64),
            ]),
            Value::List(vec![
                Value::Text("flipped_triangle_count".to_string()),
                Value::Number(diag.flipped_triangle_count as f64),
            ]),
            Value::List(vec![
                Value::Text("self_intersection_count".to_string()),
                Value::Number(diag.self_intersection_count as f64),
            ]),
            Value::List(vec![
                Value::Text("boolean_fallback_used".to_string()),
                Value::Boolean(diag.boolean_fallback_used),
            ]),
            Value::List(vec![
                Value::Text("is_watertight".to_string()),
                Value::Boolean(diag.is_watertight()),
            ]),
            Value::List(vec![
                Value::Text("is_manifold".to_string()),
                Value::Boolean(diag.is_manifold()),
            ]),
            Value::List(vec![
                Value::Text("is_valid_solid".to_string()),
                Value::Boolean(diag.is_valid_solid()),
            ]),
            Value::List(vec![
                Value::Text("warnings".to_string()),
                Value::List(
                    diag.warnings
                        .into_iter()
                        .map(Value::Text)
                        .collect(),
                ),
            ]),
        ])
    }
}

/// Beschikbare waardetypes binnen de evaluator.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Een null-waarde, vergelijkbaar met `null` in andere talen.
    Null,
    /// Een enkele numerieke waarde.
    Number(f64),
    /// Een booleaanse waarde.
    Boolean(bool),
    /// Een complex getal.
    Complex(ComplexValue),
    /// Een 3D-punt.
    Point([f64; 3]),
    /// Een 3D-vector.
    Vector([f64; 3]),
    /// Een lijnsegment, beschreven door twee punten.
    CurveLine { p1: [f64; 3], p2: [f64; 3] },
    /// Een (prismatische) mesh representatie.
    /// 
    /// **Legacy type** - kept for backward compatibility.
    /// New code should prefer `Value::Mesh` which provides additional
    /// attributes (normals, UVs) and diagnostics information.
    Surface {
        vertices: Vec<[f64; 3]>,
        faces: Vec<Vec<u32>>,
    },
    /// A triangle mesh with optional attributes and diagnostics.
    ///
    /// This is the preferred mesh representation for the new geometry engine.
    /// It provides:
    /// - Indexed triangle list (`indices` must have length divisible by 3)
    /// - Optional per-vertex normals for smooth shading
    /// - Optional per-vertex UV coordinates for texturing
    /// - Optional diagnostics about mesh quality and repairs performed
    ///
    /// # three.js Integration
    ///
    /// Convert to `BufferGeometry` using:
    /// - `vertices` → `Float32Array` position attribute
    /// - `indices` → `Uint32Array` index
    /// - `normals` → `Float32Array` normal attribute (or compute from faces)
    /// - `uvs` → `Float32Array` uv attribute
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mesh = Value::Mesh {
    ///     vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
    ///     indices: vec![0, 1, 2],
    ///     normals: Some(vec![[0.0, 0.0, 1.0]; 3]),
    ///     uvs: Some(vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]]),
    ///     diagnostics: None,
    /// };
    /// ```
    Mesh {
        /// Vertex positions as `[x, y, z]` arrays.
        vertices: Vec<[f64; 3]>,
        /// Triangle indices (length must be divisible by 3).
        /// Each consecutive triple `[i, j, k]` defines one triangle.
        indices: Vec<u32>,
        /// Optional per-vertex normals as `[nx, ny, nz]` arrays.
        /// When present, must have the same length as `vertices`.
        normals: Option<Vec<[f64; 3]>>,
        /// Optional per-vertex UV coordinates as `[u, v]` arrays.
        /// When present, must have the same length as `vertices`.
        uvs: Option<Vec<[f64; 2]>>,
        /// Optional diagnostics about mesh quality and generation.
        diagnostics: Option<MeshDiagnostics>,
    },
    /// Een numeriek domein (1D of 2D).
    Domain(Domain),
    /// Een matrix van numerieke waarden.
    Matrix(Matrix),
    /// Een datum-tijdwaarde zonder tijdzone.
    DateTime(DateTimeValue),
    /// Een lijst van waarden.
    List(Vec<Value>),
    /// Een tekstuele waarde.
    Text(String),
    /// Een tekstlabel met oriëntatie en optionele kleur.
    Tag(TextTagValue),
    /// Een RGB-kleurwaarde.
    Color(ColorValue),
    /// Een weergavemateriaal.
    Material(MaterialValue),
    /// Een weergavesymbool.
    Symbol(SymbolValue),
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            Value::Null => {}
            Value::Number(n) => n.to_bits().hash(state),
            Value::Boolean(b) => b.hash(state),
            Value::Complex(c) => {
                c.re.to_bits().hash(state);
                c.im.to_bits().hash(state);
            }
            Value::Point(p) => {
                p.iter().for_each(|x| x.to_bits().hash(state));
            }
            Value::Vector(v) => {
                v.iter().for_each(|x| x.to_bits().hash(state));
            }
            Value::CurveLine { p1, p2 } => {
                p1.iter().for_each(|x| x.to_bits().hash(state));
                p2.iter().for_each(|x| x.to_bits().hash(state));
            }
            Value::List(l) => l.hash(state),
            Value::Text(s) => s.hash(state),
            Value::DateTime(dt) => dt.hash(state),
            Value::Color(c) => {
                c.r.to_bits().hash(state);
                c.g.to_bits().hash(state);
                c.b.to_bits().hash(state);
            }
            // Non-trivial hash impls below.
            // For now, these are not hashed, which is not ideal but avoids complexity.
            Value::Surface { .. } => {}
            Value::Mesh { .. } => {}
            Value::Domain(_) => {}
            Value::Matrix(_) => {}
            Value::Tag(_) => {}
            Value::Material(_) => {}
            Value::Symbol(_) => {}
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => write!(f, "Null"),
            Self::Number(n) => write!(f, "{}", n),
            Self::Boolean(b) => write!(f, "{}", b),
            Self::Complex(c) => write!(f, "{}", c),
            Self::Point(p) => write!(f, "{},{},{}", p[0], p[1], p[2]),
            Self::Vector(v) => write!(f, "{},{},{}", v[0], v[1], v[2]),
            Self::CurveLine { p1, p2 } => write!(
                f,
                "Line [{},{},{}] to [{},{},{}]",
                p1[0], p1[1], p1[2], p2[0], p2[1], p2[2]
            ),
            Self::Surface { vertices, faces } => {
                write!(
                    f,
                    "Surface [{} vertices, {} faces]",
                    vertices.len(),
                    faces.len()
                )
            }
            Self::Mesh { vertices, indices, normals, uvs, .. } => {
                let tri_count = indices.len() / 3;
                let attrs = match (normals.is_some(), uvs.is_some()) {
                    (true, true) => " +normals +uvs",
                    (true, false) => " +normals",
                    (false, true) => " +uvs",
                    (false, false) => "",
                };
                write!(
                    f,
                    "Mesh [{} vertices, {} triangles{}]",
                    vertices.len(),
                    tri_count,
                    attrs
                )
            }
            Self::Domain(d) => match d {
                Domain::One(d1) => write!(f, "Domain {} to {}", d1.start, d1.end),
                Domain::Two(d2) => write!(
                    f,
                    "Domain U({} to {}), V({} to {})",
                    d2.u.start, d2.u.end, d2.v.start, d2.v.end
                ),
            },
            Self::Matrix(m) => write!(f, "Matrix [{}x{}]", m.rows, m.columns),
            Self::DateTime(dt) => write!(f, "{}", dt.primitive()),
            Self::List(l) => write!(f, "List [{} items]", l.len()),
            Self::Text(s) => write!(f, "{}", s),
            Self::Tag(t) => write!(f, "Tag: {}", t.text),
            Self::Color(c) => write!(f, "Color [R={}, G={}, B={}]", c.r, c.g, c.b),
            Self::Material(_) => write!(f, "Material"),
            Self::Symbol(_) => write!(f, "Symbol"),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
            (Value::Boolean(a), Value::Boolean(b)) => a.partial_cmp(b),
            (Value::DateTime(a), Value::DateTime(b)) => a.primitive().partial_cmp(&b.primitive()),
            (Value::Text(a), Value::Text(b)) => a.partial_cmp(b),
            (Value::Null, Value::Null) => Some(Ordering::Equal),
            _ => None,
        }
    }
}

impl Value {
    /// Geeft de variantnaam terug. Wordt gebruikt in foutmeldingen.
    #[must_use]
    pub fn kind(&self) -> ValueKind {
        match self {
            Self::Null => ValueKind::Null,
            Self::Number(_) => ValueKind::Number,
            Self::Boolean(_) => ValueKind::Boolean,
            Self::Complex(_) => ValueKind::Complex,
            Self::Point(_) => ValueKind::Point,
            Self::Vector(_) => ValueKind::Vector,
            Self::CurveLine { .. } => ValueKind::CurveLine,
            Self::Surface { .. } => ValueKind::Surface,
            Self::Mesh { .. } => ValueKind::Mesh,
            Self::Domain(_) => ValueKind::Domain,
            Self::Matrix(_) => ValueKind::Matrix,
            Self::DateTime(_) => ValueKind::DateTime,
            Self::List(_) => ValueKind::List,
            Self::Text(_) => ValueKind::Text,
            Self::Tag(_) => ValueKind::Tag,
            Self::Color(_) => ValueKind::Color,
            Self::Material(_) => ValueKind::Material,
            Self::Symbol(_) => ValueKind::Symbol,
        }
    }

    /// Verwacht een `Number` en retourneert de f64-waarde.
    pub fn expect_number(&self) -> Result<f64, ValueError> {
        match self {
            Self::Number(value) => Ok(*value),
            _ => Err(ValueError::type_mismatch("Number", self.kind())),
        }
    }

    /// Verwacht een `Boolean` en retourneert de waarde.
    pub fn expect_boolean(&self) -> Result<bool, ValueError> {
        match self {
            Self::Boolean(value) => Ok(*value),
            _ => Err(ValueError::type_mismatch("Boolean", self.kind())),
        }
    }

    /// Verwacht een `Complex` en retourneert de waarde.
    pub fn expect_complex(&self) -> Result<ComplexValue, ValueError> {
        match self {
            Self::Complex(value) => Ok(*value),
            _ => Err(ValueError::type_mismatch("Complex", self.kind())),
        }
    }

    /// Verwacht een `Point` en retourneert de coördinaten.
    pub fn expect_point(&self) -> Result<[f64; 3], ValueError> {
        match self {
            Self::Point(point) => Ok(*point),
            _ => Err(ValueError::type_mismatch("Point", self.kind())),
        }
    }

    /// Verwacht een `Vector` en retourneert de componenten.
    pub fn expect_vector(&self) -> Result<[f64; 3], ValueError> {
        match self {
            Self::Vector(vector) => Ok(*vector),
            _ => Err(ValueError::type_mismatch("Vector", self.kind())),
        }
    }

    /// Verwacht een `CurveLine` en retourneert de eindpunten.
    pub fn expect_curve_line(&self) -> Result<([f64; 3], [f64; 3]), ValueError> {
        match self {
            Self::CurveLine { p1, p2 } => Ok((*p1, *p2)),
            _ => Err(ValueError::type_mismatch("CurveLine", self.kind())),
        }
    }

    /// Verwacht een `Surface` en retourneert de mesh-data.
    pub fn expect_surface(&self) -> Result<(&[[f64; 3]], &[Vec<u32>]), ValueError> {
        match self {
            Self::Surface { vertices, faces } => Ok((vertices, faces)),
            _ => Err(ValueError::type_mismatch("Surface", self.kind())),
        }
    }

    /// Mesh data returned by `expect_mesh`.
    ///
    /// Contains references to the mesh buffers and optional diagnostics.
    /// This struct is used to avoid returning a large tuple.
    pub fn expect_mesh(&self) -> Result<MeshRef<'_>, ValueError> {
        match self {
            Self::Mesh {
                vertices,
                indices,
                normals,
                uvs,
                diagnostics,
            } => Ok(MeshRef {
                vertices,
                indices,
                normals: normals.as_deref(),
                uvs: uvs.as_deref(),
                diagnostics: diagnostics.as_ref(),
            }),
            _ => Err(ValueError::type_mismatch("Mesh", self.kind())),
        }
    }

    /// Expects a `Mesh` value and returns owned `MeshData`.
    ///
    /// Unlike `expect_mesh()` which returns references, this method clones
    /// the mesh data into an owned `MeshData` struct. Use this when you need
    /// to modify the mesh or when the original `Value` may go out of scope.
    ///
    /// **Note**: This only accepts `Value::Mesh`, not `Value::Surface`.
    /// For accepting both types, use `expect_mesh_like()`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mesh_data = value.expect_mesh_owned()?;
    /// // mesh_data is owned and can be modified
    /// mesh_data.normals = compute_normals(&mesh_data.vertices, &mesh_data.indices);
    /// ```
    pub fn expect_mesh_owned(&self) -> Result<MeshData, ValueError> {
        match self {
            Self::Mesh {
                vertices,
                indices,
                normals,
                uvs,
                diagnostics,
            } => Ok(MeshData {
                vertices: vertices.clone(),
                indices: indices.clone(),
                normals: normals.clone(),
                uvs: uvs.clone(),
                diagnostics: diagnostics.clone(),
            }),
            _ => Err(ValueError::type_mismatch("Mesh", self.kind())),
        }
    }

    /// Consumes a `Value::Mesh` and returns owned `MeshData` without cloning.
    ///
    /// This is the most efficient way to extract mesh data when you own the Value.
    /// Returns an error if the value is not a `Mesh`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mesh_data = value.into_mesh_data()?;
    /// // value is consumed, mesh_data owns the data
    /// ```
    pub fn into_mesh_data(self) -> Result<MeshData, ValueError> {
        match self {
            Self::Mesh {
                vertices,
                indices,
                normals,
                uvs,
                diagnostics,
            } => Ok(MeshData {
                vertices,
                indices,
                normals,
                uvs,
                diagnostics,
            }),
            _ => Err(ValueError::type_mismatch("Mesh", self.kind())),
        }
    }

    /// Expects a mesh-like value (`Mesh` or `Surface`) and returns mesh data.
    ///
    /// This is a convenience method that accepts both the new `Mesh` type and
    /// the legacy `Surface` type, converting the latter on the fly.
    ///
    /// For `Value::Surface`, faces are converted to triangle indices by taking
    /// the first three vertices of each face. Normals and UVs are not available
    /// for legacy surfaces.
    pub fn expect_mesh_like(&self) -> Result<MeshData, ValueError> {
        match self {
            Self::Mesh {
                vertices,
                indices,
                normals,
                uvs,
                diagnostics,
            } => Ok(MeshData {
                vertices: vertices.clone(),
                indices: indices.clone(),
                normals: normals.clone(),
                uvs: uvs.clone(),
                diagnostics: diagnostics.clone(),
            }),
            Self::Surface { vertices, faces } => {
                // Convert polygon faces to triangles using fan triangulation.
                // This properly handles quads and n-gons by producing (n-2) triangles
                // per n-gon face, preserving all geometry.
                let indices = triangulate_polygon_faces(faces);
                Ok(MeshData {
                    vertices: vertices.clone(),
                    indices,
                    normals: None,
                    uvs: None,
                    diagnostics: None,
                })
            }
            _ => Err(ValueError::type_mismatch("Mesh or Surface", self.kind())),
        }
    }

    /// Consumes a mesh-like value and returns owned `MeshData` without cloning.
    ///
    /// Accepts both `Value::Mesh` and `Value::Surface`, consuming the value.
    /// For `Value::Surface`, faces are converted to triangle indices.
    ///
    /// This is the most efficient way to extract mesh data when you own the Value.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mesh_data = value.into_mesh_data_like()?;
    /// // value is consumed, mesh_data owns the data
    /// ```
    pub fn into_mesh_data_like(self) -> Result<MeshData, ValueError> {
        match self {
            Self::Mesh {
                vertices,
                indices,
                normals,
                uvs,
                diagnostics,
            } => Ok(MeshData {
                vertices,
                indices,
                normals,
                uvs,
                diagnostics,
            }),
            Self::Surface { vertices, faces } => {
                // Convert polygon faces to triangles using fan triangulation.
                // This properly handles quads and n-gons by producing (n-2) triangles
                // per n-gon face, preserving all geometry.
                let indices = triangulate_polygon_faces_owned(faces);
                Ok(MeshData {
                    vertices,
                    indices,
                    normals: None,
                    uvs: None,
                    diagnostics: None,
                })
            }
            _ => Err(ValueError::type_mismatch("Mesh or Surface", self.kind())),
        }
    }

    /// Converts a `Value::Mesh` to a `Value::Surface` for legacy compatibility.
    ///
    /// Returns `None` if this value is not a `Mesh`.
    ///
    /// **Note**: This conversion is lossy:
    /// - Normals and UVs are discarded
    /// - Triangle indices are converted to single-triangle face lists
    /// - Diagnostics are discarded
    #[must_use]
    pub fn mesh_to_surface_legacy(&self) -> Option<Value> {
        match self {
            Self::Mesh { vertices, indices, .. } => {
                // Convert triangle indices to polygon faces
                let faces: Vec<Vec<u32>> = indices
                    .chunks(3)
                    .filter(|chunk| chunk.len() == 3)
                    .map(|chunk| vec![chunk[0], chunk[1], chunk[2]])
                    .collect();
                Some(Value::Surface {
                    vertices: vertices.clone(),
                    faces,
                })
            }
            _ => None,
        }
    }

    /// Converts a `Value::Surface` to a `Value::Mesh`.
    ///
    /// Returns `None` if this value is not a `Surface`.
    ///
    /// **Note**: 
    /// - Polygon faces are triangulated using fan triangulation (preserves all geometry)
    /// - Normals and UVs are not generated (set to `None`)
    /// - Diagnostics are not generated (set to `None`)
    #[must_use]
    pub fn surface_legacy_to_mesh(&self) -> Option<Value> {
        match self {
            Self::Surface { vertices, faces } => {
                // Convert polygon faces to triangles using fan triangulation.
                // This properly handles quads and n-gons.
                let indices = triangulate_polygon_faces(faces);
                Some(Value::Mesh {
                    vertices: vertices.clone(),
                    indices,
                    normals: None,
                    uvs: None,
                    diagnostics: None,
                })
            }
            _ => None,
        }
    }

    /// Returns `true` if this value is a mesh-like geometry (`Mesh` or `Surface`).
    #[must_use]
    pub fn is_mesh_like(&self) -> bool {
        matches!(self, Self::Mesh { .. } | Self::Surface { .. })
    }

    /// Verwacht een lijst en geeft een slice terug.
    pub fn expect_list(&self) -> Result<&[Value], ValueError> {
        match self {
            Self::List(values) => Ok(values),
            _ => Err(ValueError::type_mismatch("List", self.kind())),
        }
    }

    /// Verwacht een `Domain` en retourneert een verwijzing.
    pub fn expect_domain(&self) -> Result<&Domain, ValueError> {
        match self {
            Self::Domain(domain) => Ok(domain),
            _ => Err(ValueError::type_mismatch("Domain", self.kind())),
        }
    }

    /// Verwacht een `Matrix` en retourneert een verwijzing.
    pub fn expect_matrix(&self) -> Result<&Matrix, ValueError> {
        match self {
            Self::Matrix(matrix) => Ok(matrix),
            _ => Err(ValueError::type_mismatch("Matrix", self.kind())),
        }
    }

    /// Verwacht een `DateTime` en retourneert de waarde.
    pub fn expect_date_time(&self) -> Result<PrimitiveDateTime, ValueError> {
        match self {
            Self::DateTime(date_time) => Ok(date_time.primitive()),
            _ => Err(ValueError::type_mismatch("DateTime", self.kind())),
        }
    }

    /// Verwacht een `Tag` en retourneert de taggegevens.
    pub fn expect_tag(&self) -> Result<&TextTagValue, ValueError> {
        match self {
            Self::Tag(tag) => Ok(tag),
            _ => Err(ValueError::type_mismatch("Tag", self.kind())),
        }
    }

    /// Verwacht een `Text` en retourneert de tekstwaarde als referentie.
    pub fn expect_text(&self) -> Result<&str, ValueError> {
        match self {
            Self::Text(text) => Ok(text),
            _ => Err(ValueError::type_mismatch("Text", self.kind())),
        }
    }

    /// Verwacht een `Text` en retourneert een gekloneerde `String`.
    pub fn expect_text_owned(&self) -> Result<String, ValueError> {
        match self {
            Self::Text(text) => Ok(text.clone()),
            _ => Err(ValueError::type_mismatch("Text", self.kind())),
        }
    }

    /// Verwacht een `Color` en retourneert de kleurwaarde.
    pub fn expect_color(&self) -> Result<ColorValue, ValueError> {
        match self {
            Self::Color(color) => Ok(*color),
            _ => Err(ValueError::type_mismatch("Color", self.kind())),
        }
    }

    /// Verwacht een `Material` en retourneert de materiaalwaarde.
    pub fn expect_material(&self) -> Result<MaterialValue, ValueError> {
        match self {
            Self::Material(material) => Ok(*material),
            _ => Err(ValueError::type_mismatch("Material", self.kind())),
        }
    }

    /// Verwacht een `Symbol` en retourneert een referentie naar de symboolgegevens.
    pub fn expect_symbol(&self) -> Result<&SymbolValue, ValueError> {
        match self {
            Self::Symbol(symbol) => Ok(symbol),
            _ => Err(ValueError::type_mismatch("Symbol", self.kind())),
        }
    }
}

/// Typefout voor wanneer een `Value` naar het verkeerde type wordt
/// geconverteerd.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueError {
    expected: &'static str,
    found: ValueKind,
}

impl ValueError {
    #[must_use]
    pub fn type_mismatch(expected: &'static str, found: ValueKind) -> Self {
        Self { expected, found }
    }

    /// Hulptoegang voor tests en foutafhandeling.
    #[must_use]
    pub fn expected(&self) -> &'static str {
        self.expected
    }

    #[must_use]
    pub fn found(&self) -> ValueKind {
        self.found
    }
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "verwachtte type `{}` maar kreeg `{}`",
            self.expected, self.found
        )
    }
}

impl std::error::Error for ValueError {}

/// Beschrijft het soort `Value`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Null,
    Number,
    Boolean,
    Point,
    Vector,
    CurveLine,
    Surface,
    Mesh,
    Domain,
    List,
    Matrix,
    Complex,
    DateTime,
    Text,
    Tag,
    Color,
    Material,
    Symbol,
}

impl fmt::Display for ValueKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Null => "Null",
            Self::Number => "Number",
            Self::Boolean => "Boolean",
            Self::Point => "Point",
            Self::Vector => "Vector",
            Self::CurveLine => "CurveLine",
            Self::Surface => "Surface",
            Self::Mesh => "Mesh",
            Self::Domain => "Domain",
            Self::Matrix => "Matrix",
            Self::Complex => "Complex",
            Self::DateTime => "DateTime",
            Self::List => "List",
            Self::Text => "Text",
            Self::Tag => "Tag",
            Self::Color => "Color",
            Self::Material => "Material",
            Self::Symbol => "Symbol",
        };
        f.write_str(name)
    }
}

/// Beschrijving van een weergavemateriaal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialValue {
    pub diffuse: ColorValue,
    pub specular: ColorValue,
    pub emission: ColorValue,
    pub transparency: f64,
    pub shine: f64,
}

/// Beschrijving van een weergavesymbool.
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolValue {
    pub style: String,
    pub size_primary: f64,
    pub size_secondary: Option<f64>,
    pub rotation: f64,
    pub fill: ColorValue,
    pub edge: Option<ColorValue>,
    pub width: f64,
    pub adjust: bool,
}

/// Beschrijving van een vlak in de ruimte.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaneValue {
    pub origin: [f64; 3],
    pub x_axis: [f64; 3],
    pub y_axis: [f64; 3],
    pub z_axis: [f64; 3],
}

impl PlaneValue {
    /// Maak een nieuw vlak met opgegeven basisvectoren.
    #[must_use]
    pub fn new(origin: [f64; 3], x_axis: [f64; 3], y_axis: [f64; 3], z_axis: [f64; 3]) -> Self {
        Self {
            origin,
            x_axis,
            y_axis,
            z_axis,
        }
    }

    /// Geeft een standaard vlak terug met assen gelijk aan de wereldassen.
    #[must_use]
    pub fn default() -> Self {
        Self {
            origin: [0.0, 0.0, 0.0],
            x_axis: [1.0, 0.0, 0.0],
            y_axis: [0.0, 1.0, 0.0],
            z_axis: [0.0, 0.0, 1.0],
        }
    }
}

/// RGB kleurwaarden genormaliseerd tussen 0 en 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorValue {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl ColorValue {
    /// Maak een nieuwe kleur aan en klem componenten binnen [0, 1].
    #[must_use]
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self {
            r: clamp01(r),
            g: clamp01(g),
            b: clamp01(b),
        }
    }

    /// Maak een kleur uit waarden in het bereik [0, 255].
    #[must_use]
    pub fn from_rgb255(r: f64, g: f64, b: f64) -> Self {
        Self::new(r / 255.0, g / 255.0, b / 255.0)
    }

    /// Maak een grijstint op basis van een scalar.
    #[must_use]
    pub fn grayscale(value: f64) -> Self {
        if value <= 1.0 {
            Self::new(value, value, value)
        } else {
            Self::from_rgb255(value, value, value)
        }
    }
}

fn clamp01(value: f64) -> f64 {
    if !value.is_finite() {
        return 0.0;
    }
    if value <= 0.0 {
        0.0
    } else if value >= 1.0 {
        1.0
    } else {
        value
    }
}

/// Beschrijving van een Grasshopper teksttag.
#[derive(Debug, Clone, PartialEq)]
pub struct TextTagValue {
    pub plane: PlaneValue,
    pub text: String,
    pub size: f64,
    pub color: Option<ColorValue>,
}

impl TextTagValue {
    /// Maak een nieuwe teksttag aan.
    #[must_use]
    pub fn new(
        plane: PlaneValue,
        text: impl Into<String>,
        size: f64,
        color: Option<ColorValue>,
    ) -> Self {
        Self {
            plane,
            text: text.into(),
            size,
            color,
        }
    }
}

/// Een tijdstip bestaande uit een datum en tijd zonder tijdzone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DateTimeValue {
    datetime: PrimitiveDateTime,
}

impl DateTimeValue {
    /// Maakt een nieuwe datum-tijdwaarde aan.
    #[must_use]
    pub fn from_primitive(datetime: PrimitiveDateTime) -> Self {
        Self { datetime }
    }

    /// Geeft de onderliggende `PrimitiveDateTime` terug.
    #[must_use]
    pub fn primitive(&self) -> PrimitiveDateTime {
        self.datetime
    }
}

/// Een eenvoudige matrixstructuur die door componenten kan worden gebruikt.
#[derive(Debug, Clone, PartialEq)]
pub struct Matrix {
    pub rows: usize,
    pub columns: usize,
    pub values: Vec<f64>,
}

impl Matrix {
    /// Maakt een matrix aan wanneer de afmetingen en waarden overeenkomen.
    #[must_use]
    pub fn new(rows: usize, columns: usize, values: Vec<f64>) -> Option<Self> {
        if rows == 0 || columns == 0 || values.len() != rows * columns {
            return None;
        }
        Some(Self {
            rows,
            columns,
            values,
        })
    }
}

/// Een één-dimensionaal numeriek domein.
#[derive(Debug, Clone, PartialEq)]
pub struct Domain1D {
    pub start: f64,
    pub end: f64,
    pub min: f64,
    pub max: f64,
    pub span: f64,
    pub length: f64,
    pub center: f64,
}

/// Een twee-dimensionaal domein opgebouwd uit twee 1D-domeinen.
#[derive(Debug, Clone, PartialEq)]
pub struct Domain2D {
    pub u: Domain1D,
    pub v: Domain1D,
}

/// Beschikbare domeinvarianten die opgeslagen kunnen worden in `Value::Domain`.
#[derive(Debug, Clone, PartialEq)]
pub enum Domain {
    One(Domain1D),
    Two(Domain2D),
}

#[cfg(test)]
mod tests {
    use super::{ComplexValue, DateTimeValue, Value, ValueError, ValueKind};
    use num_complex::Complex;
    use time::macros::datetime;

    #[test]
    fn expect_number_accepts_number() {
        let value = Value::Number(42.0);
        assert_eq!(value.expect_number().unwrap(), 42.0);
    }

    #[test]
    fn expect_number_rejects_wrong_type() {
        let value = Value::Point([0.0, 0.0, 0.0]);
        let err = value.expect_number().unwrap_err();
        assert_eq!(err.expected(), "Number");
        assert_eq!(err.found(), ValueKind::Point);
    }

    #[test]
    fn expect_boolean_accepts_boolean() {
        let value = Value::Boolean(true);
        assert!(value.expect_boolean().unwrap());
    }

    #[test]
    fn expect_boolean_rejects_other_types() {
        let value = Value::Number(0.0);
        let err = value.expect_boolean().unwrap_err();
        assert_eq!(err.expected(), "Boolean");
        assert_eq!(err.found(), ValueKind::Number);
    }

    #[test]
    fn expect_complex_accepts_complex() {
        let value = Value::Complex(ComplexValue::new(2.0, -3.5));
        assert_eq!(value.expect_complex().unwrap(), Complex::new(2.0, -3.5));
    }

    #[test]
    fn complex_helpers_compute_properties() {
        let complex = ComplexValue::new(3.0, 4.0);
        assert_eq!(complex.norm(), 5.0);
        assert_eq!(complex.conj(), Complex::new(3.0, -4.0));
        assert!((complex.arg() - (4.0f64).atan2(3.0)).abs() < 1e-12);
    }

    #[test]
    fn expect_surface_returns_references() {
        let vertices = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let faces = vec![vec![0, 1, 1]];
        let value = Value::Surface {
            vertices: vertices.clone(),
            faces: faces.clone(),
        };

        let (verts, fcs) = value.expect_surface().unwrap();
        assert_eq!(verts, vertices.as_slice());
        assert_eq!(fcs, faces.as_slice());
    }

    #[test]
    fn expect_curve_line_returns_endpoints() {
        let value = Value::CurveLine {
            p1: [0.0, 0.0, 0.0],
            p2: [1.0, 2.0, 3.0],
        };
        let (p1, p2) = value.expect_curve_line().unwrap();
        assert_eq!(p1, [0.0, 0.0, 0.0]);
        assert_eq!(p2, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn list_expectation_requires_list() {
        let value = Value::List(vec![Value::Number(1.0)]);
        assert_eq!(value.expect_list().unwrap().len(), 1);

        let non_list = Value::Number(3.0);
        assert!(matches!(non_list.expect_list(), Err(ValueError { .. })));
    }

    #[test]
    fn expect_date_time_returns_datetime() {
        let datetime = datetime!(2024-06-01 12:30:45);
        let value = Value::DateTime(DateTimeValue::from_primitive(datetime));
        assert_eq!(value.expect_date_time().unwrap(), datetime);
    }

    #[test]
    fn expect_date_time_rejects_other_types() {
        let value = Value::Number(1.0);
        let err = value.expect_date_time().unwrap_err();
        assert_eq!(err.expected(), "DateTime");
        assert_eq!(err.found(), ValueKind::Number);
    }

    // ========================================================================
    // Tests for Value::Mesh and related types
    // ========================================================================

    #[test]
    fn mesh_value_creation_and_display() {
        let value = Value::Mesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: Some(vec![[0.0, 0.0, 1.0]; 3]),
            uvs: Some(vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]]),
            diagnostics: None,
        };

        assert_eq!(value.kind(), ValueKind::Mesh);
        let display = format!("{}", value);
        assert!(display.contains("3 vertices"));
        assert!(display.contains("1 triangles"));
        assert!(display.contains("+normals"));
        assert!(display.contains("+uvs"));
    }

    #[test]
    fn mesh_value_without_attributes() {
        let value = Value::Mesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: None,
            uvs: None,
            diagnostics: None,
        };

        let display = format!("{}", value);
        assert!(!display.contains("+normals"));
        assert!(!display.contains("+uvs"));
    }

    #[test]
    fn expect_mesh_returns_mesh_ref() {
        let value = Value::Mesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: Some(vec![[0.0, 0.0, 1.0]; 3]),
            uvs: None,
            diagnostics: None,
        };

        let mesh_ref = value.expect_mesh().unwrap();
        assert_eq!(mesh_ref.vertex_count(), 3);
        assert_eq!(mesh_ref.triangle_count(), 1);
        assert!(mesh_ref.has_normals());
        assert!(!mesh_ref.has_uvs());
    }

    #[test]
    fn expect_mesh_rejects_non_mesh() {
        let value = Value::Surface {
            vertices: vec![[0.0, 0.0, 0.0]],
            faces: vec![],
        };
        let err = value.expect_mesh().unwrap_err();
        assert_eq!(err.expected(), "Mesh");
        assert_eq!(err.found(), ValueKind::Surface);
    }

    #[test]
    fn expect_mesh_like_accepts_mesh() {
        let value = Value::Mesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: Some(vec![[0.0, 0.0, 1.0]; 3]),
            uvs: None,
            diagnostics: None,
        };

        let mesh_data = value.expect_mesh_like().unwrap();
        assert_eq!(mesh_data.vertex_count(), 3);
        assert_eq!(mesh_data.triangle_count(), 1);
        assert!(mesh_data.has_normals());
    }

    #[test]
    fn expect_mesh_like_converts_surface() {
        let value = Value::Surface {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            faces: vec![vec![0, 1, 2], vec![0, 2, 3]],
        };

        let mesh_data = value.expect_mesh_like().unwrap();
        assert_eq!(mesh_data.vertex_count(), 4);
        assert_eq!(mesh_data.triangle_count(), 2);
        assert_eq!(mesh_data.indices, vec![0, 1, 2, 0, 2, 3]);
        assert!(!mesh_data.has_normals());
        assert!(!mesh_data.has_uvs());
    }

    #[test]
    fn mesh_to_surface_legacy_conversion() {
        let mesh = Value::Mesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: Some(vec![[0.0, 0.0, 1.0]; 3]),
            uvs: None,
            diagnostics: None,
        };

        let surface = mesh.mesh_to_surface_legacy().unwrap();
        assert_eq!(surface.kind(), ValueKind::Surface);

        let (verts, faces) = surface.expect_surface().unwrap();
        assert_eq!(verts.len(), 3);
        assert_eq!(faces.len(), 1);
        assert_eq!(faces[0], vec![0, 1, 2]);
    }

    #[test]
    fn surface_legacy_to_mesh_conversion() {
        let surface = Value::Surface {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            faces: vec![vec![0, 1, 2], vec![0, 2, 3]],
        };

        let mesh = surface.surface_legacy_to_mesh().unwrap();
        assert_eq!(mesh.kind(), ValueKind::Mesh);

        let mesh_ref = mesh.expect_mesh().unwrap();
        assert_eq!(mesh_ref.vertex_count(), 4);
        assert_eq!(mesh_ref.triangle_count(), 2);
        assert!(!mesh_ref.has_normals());
        assert!(!mesh_ref.has_uvs());
    }

    #[test]
    fn triangulate_polygon_faces_triangles_pass_through() {
        // Triangles should pass through unchanged
        let faces = vec![vec![0, 1, 2], vec![3, 4, 5]];
        let indices = super::triangulate_polygon_faces(&faces);
        assert_eq!(indices, vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn triangulate_polygon_faces_quad_produces_two_triangles() {
        // A quad [0, 1, 2, 3] should produce 2 triangles using fan triangulation
        let faces = vec![vec![0, 1, 2, 3]];
        let indices = super::triangulate_polygon_faces(&faces);
        // Fan from vertex 0: [0, 1, 2] and [0, 2, 3]
        assert_eq!(indices, vec![0, 1, 2, 0, 2, 3]);
    }

    #[test]
    fn triangulate_polygon_faces_pentagon_produces_three_triangles() {
        // A pentagon [0, 1, 2, 3, 4] should produce 3 triangles
        let faces = vec![vec![0, 1, 2, 3, 4]];
        let indices = super::triangulate_polygon_faces(&faces);
        // Fan from vertex 0: [0, 1, 2], [0, 2, 3], [0, 3, 4]
        assert_eq!(indices, vec![0, 1, 2, 0, 2, 3, 0, 3, 4]);
    }

    #[test]
    fn triangulate_polygon_faces_hexagon_produces_four_triangles() {
        // A hexagon [0, 1, 2, 3, 4, 5] should produce 4 triangles
        let faces = vec![vec![0, 1, 2, 3, 4, 5]];
        let indices = super::triangulate_polygon_faces(&faces);
        // Fan from vertex 0: [0, 1, 2], [0, 2, 3], [0, 3, 4], [0, 4, 5]
        assert_eq!(indices, vec![0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5]);
        assert_eq!(indices.len() / 3, 4); // 4 triangles
    }

    #[test]
    fn triangulate_polygon_faces_mixed_sizes() {
        // Mix of triangle, quad, and pentagon
        let faces = vec![vec![0, 1, 2], vec![3, 4, 5, 6], vec![7, 8, 9, 10, 11]];
        let indices = super::triangulate_polygon_faces(&faces);
        // Triangle: 1 tri, Quad: 2 tris, Pentagon: 3 tris = 6 triangles
        assert_eq!(indices.len() / 3, 6);
        // Check specific indices
        assert_eq!(&indices[0..3], &[0, 1, 2]); // triangle
        assert_eq!(&indices[3..9], &[3, 4, 5, 3, 5, 6]); // quad
        assert_eq!(&indices[9..18], &[7, 8, 9, 7, 9, 10, 7, 10, 11]); // pentagon
    }

    #[test]
    fn triangulate_polygon_faces_skips_degenerate() {
        // Faces with fewer than 3 vertices should be skipped
        let faces = vec![vec![0, 1], vec![2, 3, 4]];
        let indices = super::triangulate_polygon_faces(&faces);
        assert_eq!(indices, vec![2, 3, 4]); // Only the triangle
    }

    #[test]
    fn expect_mesh_like_quad_surface_produces_correct_triangles() {
        // A surface with a single quad face should produce 2 triangles
        let value = Value::Surface {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            faces: vec![vec![0, 1, 2, 3]], // Single quad face
        };

        let mesh_data = value.expect_mesh_like().unwrap();
        assert_eq!(mesh_data.vertex_count(), 4);
        assert_eq!(mesh_data.triangle_count(), 2); // Quad -> 2 triangles
        assert_eq!(mesh_data.indices, vec![0, 1, 2, 0, 2, 3]);
    }

    #[test]
    fn into_mesh_data_like_quad_surface_produces_correct_triangles() {
        // Test the owned/consuming version
        let value = Value::Surface {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            faces: vec![vec![0, 1, 2, 3]], // Single quad face
        };

        let mesh_data = value.into_mesh_data_like().unwrap();
        assert_eq!(mesh_data.vertex_count(), 4);
        assert_eq!(mesh_data.triangle_count(), 2); // Quad -> 2 triangles
        assert_eq!(mesh_data.indices, vec![0, 1, 2, 0, 2, 3]);
    }

    #[test]
    fn surface_legacy_to_mesh_quad_produces_correct_triangles() {
        let surface = Value::Surface {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            faces: vec![vec![0, 1, 2, 3]], // Single quad face
        };

        let mesh = surface.surface_legacy_to_mesh().unwrap();
        let mesh_ref = mesh.expect_mesh().unwrap();
        assert_eq!(mesh_ref.triangle_count(), 2); // Quad -> 2 triangles
    }

    #[test]
    fn is_mesh_like_identifies_mesh_types() {
        let mesh = Value::Mesh {
            vertices: vec![],
            indices: vec![],
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        let surface = Value::Surface {
            vertices: vec![],
            faces: vec![],
        };
        let point = Value::Point([0.0, 0.0, 0.0]);

        assert!(mesh.is_mesh_like());
        assert!(surface.is_mesh_like());
        assert!(!point.is_mesh_like());
    }

    #[test]
    fn mesh_data_validation() {
        // Valid mesh
        let valid = super::MeshData::new(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![0, 1, 2],
        );
        assert!(valid.validate().is_ok());

        // Invalid: indices not divisible by 3
        let bad_indices = super::MeshData::new(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            vec![0, 1],
        );
        assert!(bad_indices.validate().is_err());

        // Invalid: out-of-bounds index
        let oob_index = super::MeshData::new(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            vec![0, 1, 5],
        );
        assert!(oob_index.validate().is_err());

        // Invalid: normals length mismatch
        let bad_normals = super::MeshData::with_attributes(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![0, 1, 2],
            Some(vec![[0.0, 0.0, 1.0]]), // Only 1 normal for 3 vertices
            None,
        );
        assert!(bad_normals.validate().is_err());

        // Invalid: NaN in vertex
        let nan_vertex = super::MeshData::new(
            vec![[f64::NAN, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![0, 1, 2],
        );
        assert!(nan_vertex.validate().is_err());
    }

    #[test]
    fn mesh_data_to_value_roundtrip() {
        let data = super::MeshData::with_attributes(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![0, 1, 2],
            Some(vec![[0.0, 0.0, 1.0]; 3]),
            Some(vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]]),
        );

        let value = data.clone().into_value();
        let roundtrip = value.expect_mesh_like().unwrap();

        assert_eq!(roundtrip.vertices, data.vertices);
        assert_eq!(roundtrip.indices, data.indices);
        assert_eq!(roundtrip.normals, data.normals);
        assert_eq!(roundtrip.uvs, data.uvs);
    }

    #[test]
    fn mesh_ref_to_owned() {
        let value = Value::Mesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: Some(vec![[0.0, 0.0, 1.0]; 3]),
            uvs: None,
            diagnostics: None,
        };

        let mesh_ref = value.expect_mesh().unwrap();
        let owned = mesh_ref.to_owned();

        assert_eq!(owned.vertices.len(), 3);
        assert_eq!(owned.indices.len(), 3);
        assert!(owned.normals.is_some());
        assert!(owned.uvs.is_none());
    }

    // ========================================================================
    // Tests for MeshQuality
    // ========================================================================

    #[test]
    fn mesh_quality_presets() {
        let low = super::MeshQuality::low();
        let medium = super::MeshQuality::medium();
        let high = super::MeshQuality::high();
        let ultra = super::MeshQuality::ultra();

        // Lower quality = larger edge length
        assert!(low.max_edge_length > medium.max_edge_length);
        assert!(medium.max_edge_length > high.max_edge_length);
        assert!(high.max_edge_length > ultra.max_edge_length);

        // Lower quality = larger deviation tolerance
        assert!(low.max_deviation > medium.max_deviation);
        assert!(medium.max_deviation > high.max_deviation);
        assert!(high.max_deviation > ultra.max_deviation);
    }

    #[test]
    fn mesh_quality_builder() {
        let quality = super::MeshQuality::medium()
            .with_max_edge_length(0.5)
            .with_max_deviation(0.005)
            .with_angle_threshold(10.0)
            .with_subdivisions(8, 512);

        assert!((quality.max_edge_length - 0.5).abs() < 1e-9);
        assert!((quality.max_deviation - 0.005).abs() < 1e-9);
        assert!((quality.angle_threshold_degrees - 10.0).abs() < 1e-9);
        assert_eq!(quality.min_subdivisions, 8);
        assert_eq!(quality.max_subdivisions, 512);
    }

    #[test]
    fn mesh_quality_clamps_invalid_values() {
        // Test that invalid values are clamped to safe ranges
        let quality = super::MeshQuality::new(
            -1.0,   // Should clamp to 0.001
            -1.0,   // Should clamp to 0.0001
            0.5,    // Should clamp to 1.0
            0,      // Should clamp to 1
            10000,  // Should clamp to 4096
        );

        assert!(quality.max_edge_length >= 0.001);
        assert!(quality.max_deviation >= 0.0001);
        assert!(quality.angle_threshold_degrees >= 1.0);
        assert!(quality.min_subdivisions >= 1);
        assert!(quality.max_subdivisions <= 4096);
    }

    // ========================================================================
    // Tests for MeshDiagnostics
    // ========================================================================

    #[test]
    fn mesh_diagnostics_default_is_clean() {
        let diag = super::MeshDiagnostics::default();
        assert!(diag.is_clean());
        assert!(diag.is_watertight());
        assert!(diag.is_manifold());
        assert!(diag.is_valid_solid());
        assert!(!diag.has_warnings());
    }

    #[test]
    fn mesh_diagnostics_open_edges_not_watertight() {
        let diag = super::MeshDiagnostics {
            open_edge_count: 3,
            ..Default::default()
        };
        assert!(!diag.is_watertight());
        assert!(diag.is_manifold());
        assert!(!diag.is_valid_solid());
        assert!(!diag.is_clean());
    }

    #[test]
    fn mesh_diagnostics_summary() {
        let diag = super::MeshDiagnostics {
            vertex_count: 100,
            triangle_count: 50,
            welded_vertex_count: 5,
            open_edge_count: 2,
            ..Default::default()
        };

        let summary = diag.summary();
        assert!(summary.contains("V:100"));
        assert!(summary.contains("T:50"));
        assert!(summary.contains("welded:5"));
        assert!(summary.contains("open:2"));
    }

    #[test]
    fn mesh_diagnostics_merge() {
        let mut diag1 = super::MeshDiagnostics {
            vertex_count: 100,
            triangle_count: 50,
            open_edge_count: 2,
            ..Default::default()
        };

        let diag2 = super::MeshDiagnostics {
            vertex_count: 200,
            triangle_count: 100,
            open_edge_count: 3,
            boolean_fallback_used: true,
            ..Default::default()
        };

        diag1.merge(&diag2);

        assert_eq!(diag1.vertex_count, 300);
        assert_eq!(diag1.triangle_count, 150);
        assert_eq!(diag1.open_edge_count, 5);
        assert!(diag1.boolean_fallback_used);
    }

    #[test]
    fn mesh_diagnostics_topology_issue_count() {
        let diag = super::MeshDiagnostics {
            open_edge_count: 5,
            non_manifold_edge_count: 3,
            ..Default::default()
        };

        assert_eq!(diag.topology_issue_count(), 8);
    }

    #[test]
    fn mesh_diagnostics_repair_count() {
        let diag = super::MeshDiagnostics {
            welded_vertex_count: 10,
            flipped_triangle_count: 5,
            degenerate_triangle_count: 2,
            ..Default::default()
        };

        assert_eq!(diag.repair_count(), 17);
    }

    #[test]
    fn mesh_diagnostics_add_warning() {
        let mut diag = super::MeshDiagnostics::default();
        assert!(!diag.has_warnings());

        diag.add_warning("test warning 1");
        diag.add_warning(String::from("test warning 2"));

        assert!(diag.has_warnings());
        assert_eq!(diag.warnings.len(), 2);
        assert_eq!(diag.warnings[0], "test warning 1");
        assert_eq!(diag.warnings[1], "test warning 2");
    }

    #[test]
    fn mesh_diagnostics_display() {
        let diag = super::MeshDiagnostics {
            vertex_count: 100,
            triangle_count: 50,
            open_edge_count: 2,
            ..Default::default()
        };

        let display = format!("{}", diag);
        assert!(display.contains("MeshDiagnostics"));
        assert!(display.contains("V:100"));
        assert!(display.contains("T:50"));
        assert!(display.contains("open:2"));
    }

    // ========================================================================
    // Tests for MeshQuality::from_meta() and related parsing
    // ========================================================================

    use crate::graph::node::{MetaMap, MetaValue};

    #[test]
    fn mesh_quality_from_preset_name_low() {
        assert!(super::MeshQuality::from_preset_name("low").is_some());
        assert!(super::MeshQuality::from_preset_name("LOW").is_some());
        assert!(super::MeshQuality::from_preset_name("preview").is_some());
        assert!(super::MeshQuality::from_preset_name("draft").is_some());
        assert!(super::MeshQuality::from_preset_name("coarse").is_some());

        let low = super::MeshQuality::from_preset_name("low").unwrap();
        assert_eq!(low.max_edge_length, 5.0);
    }

    #[test]
    fn mesh_quality_from_preset_name_medium() {
        assert!(super::MeshQuality::from_preset_name("medium").is_some());
        assert!(super::MeshQuality::from_preset_name("MEDIUM").is_some());
        assert!(super::MeshQuality::from_preset_name("default").is_some());
        assert!(super::MeshQuality::from_preset_name("normal").is_some());
        assert!(super::MeshQuality::from_preset_name("standard").is_some());

        let medium = super::MeshQuality::from_preset_name("medium").unwrap();
        assert_eq!(medium.max_edge_length, 1.0);
    }

    #[test]
    fn mesh_quality_from_preset_name_high() {
        assert!(super::MeshQuality::from_preset_name("high").is_some());
        assert!(super::MeshQuality::from_preset_name("HIGH").is_some());
        assert!(super::MeshQuality::from_preset_name("fine").is_some());
        assert!(super::MeshQuality::from_preset_name("quality").is_some());
        assert!(super::MeshQuality::from_preset_name("detailed").is_some());

        let high = super::MeshQuality::from_preset_name("high").unwrap();
        assert_eq!(high.max_edge_length, 0.25);
    }

    #[test]
    fn mesh_quality_from_preset_name_ultra() {
        assert!(super::MeshQuality::from_preset_name("ultra").is_some());
        assert!(super::MeshQuality::from_preset_name("ULTRA").is_some());
        assert!(super::MeshQuality::from_preset_name("max").is_some());
        assert!(super::MeshQuality::from_preset_name("maximum").is_some());
        assert!(super::MeshQuality::from_preset_name("best").is_some());

        let ultra = super::MeshQuality::from_preset_name("ultra").unwrap();
        assert_eq!(ultra.max_edge_length, 0.1);
    }

    #[test]
    fn mesh_quality_from_preset_name_invalid() {
        assert!(super::MeshQuality::from_preset_name("invalid").is_none());
        assert!(super::MeshQuality::from_preset_name("").is_none());
        assert!(super::MeshQuality::from_preset_name("foo").is_none());
    }

    #[test]
    fn mesh_quality_from_preset_name_with_whitespace() {
        let result = super::MeshQuality::from_preset_name("  high  ");
        assert!(result.is_some());
        assert_eq!(result.unwrap().max_edge_length, 0.25);
    }

    #[test]
    fn mesh_quality_from_meta_empty() {
        let meta = MetaMap::new();
        let quality = super::MeshQuality::from_meta(&meta);
        // Should return default (medium) when empty
        assert_eq!(quality.max_edge_length, 1.0);
        assert_eq!(quality.max_deviation, 0.01);
    }

    #[test]
    fn mesh_quality_from_meta_preset_only() {
        let mut meta = MetaMap::new();
        meta.insert("mesh_quality".to_string(), MetaValue::Text("high".to_string()));

        let quality = super::MeshQuality::from_meta(&meta);
        assert_eq!(quality.max_edge_length, 0.25);
        assert_eq!(quality.max_deviation, 0.001);
    }

    #[test]
    fn mesh_quality_from_meta_quality_key() {
        let mut meta = MetaMap::new();
        meta.insert("quality".to_string(), MetaValue::Text("low".to_string()));

        let quality = super::MeshQuality::from_meta(&meta);
        assert_eq!(quality.max_edge_length, 5.0);
    }

    #[test]
    fn mesh_quality_from_meta_preset_key() {
        let mut meta = MetaMap::new();
        meta.insert("preset".to_string(), MetaValue::Text("ultra".to_string()));

        let quality = super::MeshQuality::from_meta(&meta);
        assert_eq!(quality.max_edge_length, 0.1);
    }

    #[test]
    fn mesh_quality_from_meta_individual_params() {
        let mut meta = MetaMap::new();
        meta.insert("max_edge_length".to_string(), MetaValue::Number(2.5));
        meta.insert("max_deviation".to_string(), MetaValue::Number(0.05));
        meta.insert("angle_threshold".to_string(), MetaValue::Number(20.0));
        meta.insert("min_subdivisions".to_string(), MetaValue::Integer(6));
        meta.insert("max_subdivisions".to_string(), MetaValue::Integer(300));

        let quality = super::MeshQuality::from_meta(&meta);
        assert!((quality.max_edge_length - 2.5).abs() < 1e-9);
        assert!((quality.max_deviation - 0.05).abs() < 1e-9);
        assert!((quality.angle_threshold_degrees - 20.0).abs() < 1e-9);
        assert_eq!(quality.min_subdivisions, 6);
        assert_eq!(quality.max_subdivisions, 300);
    }

    #[test]
    fn mesh_quality_from_meta_preset_with_overrides() {
        let mut meta = MetaMap::new();
        meta.insert("mesh_quality".to_string(), MetaValue::Text("high".to_string()));
        meta.insert("max_edge_length".to_string(), MetaValue::Number(0.5)); // Override

        let quality = super::MeshQuality::from_meta(&meta);
        // max_edge_length overridden
        assert!((quality.max_edge_length - 0.5).abs() < 1e-9);
        // Other values from "high" preset
        assert!((quality.max_deviation - 0.001).abs() < 1e-9);
    }

    #[test]
    fn mesh_quality_from_meta_alternate_keys() {
        let mut meta = MetaMap::new();
        meta.insert("edge_length".to_string(), MetaValue::Number(1.5));
        meta.insert("deviation".to_string(), MetaValue::Number(0.02));
        meta.insert("angle".to_string(), MetaValue::Number(12.0));
        meta.insert("min_subdiv".to_string(), MetaValue::Number(5.0));
        meta.insert("max_subdiv".to_string(), MetaValue::Number(200.0));

        let quality = super::MeshQuality::from_meta(&meta);
        assert!((quality.max_edge_length - 1.5).abs() < 1e-9);
        assert!((quality.max_deviation - 0.02).abs() < 1e-9);
        assert!((quality.angle_threshold_degrees - 12.0).abs() < 1e-9);
        assert_eq!(quality.min_subdivisions, 5);
        assert_eq!(quality.max_subdivisions, 200);
    }

    #[test]
    fn mesh_quality_from_meta_integer_preset() {
        let mut meta = MetaMap::new();
        meta.insert("mesh_quality".to_string(), MetaValue::Integer(2)); // 2 = high

        let quality = super::MeshQuality::from_meta(&meta);
        assert_eq!(quality.max_edge_length, 0.25);
    }

    #[test]
    fn mesh_quality_from_meta_number_preset() {
        let mut meta = MetaMap::new();
        meta.insert("quality".to_string(), MetaValue::Number(0.0)); // 0 = low

        let quality = super::MeshQuality::from_meta(&meta);
        assert_eq!(quality.max_edge_length, 5.0);
    }

    #[test]
    fn mesh_quality_from_meta_clamps_invalid_values() {
        let mut meta = MetaMap::new();
        meta.insert("max_edge_length".to_string(), MetaValue::Number(-5.0));
        meta.insert("max_deviation".to_string(), MetaValue::Number(-1.0));
        meta.insert("angle_threshold".to_string(), MetaValue::Number(200.0));
        meta.insert("min_subdivisions".to_string(), MetaValue::Integer(-10));
        meta.insert("max_subdivisions".to_string(), MetaValue::Integer(10000));

        let quality = super::MeshQuality::from_meta(&meta);
        assert!(quality.max_edge_length >= 0.001);
        assert!(quality.max_deviation >= 0.0001);
        assert!(quality.angle_threshold_degrees <= 90.0);
        assert!(quality.min_subdivisions >= 1);
        assert!(quality.max_subdivisions <= 4096);
    }

    #[test]
    fn mesh_quality_from_meta_list_wrapping() {
        // Some Grasshopper inputs wrap values in single-element lists
        let mut meta = MetaMap::new();
        meta.insert(
            "mesh_quality".to_string(),
            MetaValue::List(vec![MetaValue::Text("high".to_string())]),
        );
        meta.insert(
            "max_edge_length".to_string(),
            MetaValue::List(vec![MetaValue::Number(0.75)]),
        );

        let quality = super::MeshQuality::from_meta(&meta);
        assert!((quality.max_edge_length - 0.75).abs() < 1e-9);
        assert!((quality.max_deviation - 0.001).abs() < 1e-9); // From "high" preset
    }

    #[test]
    fn mesh_quality_from_meta_case_insensitive_keys() {
        // The get_normalized trait allows searching with UPPERCASE keys
        // to find lowercase-stored values (how GHX parsing typically stores keys)
        let mut meta = MetaMap::new();
        meta.insert("max_edge_length".to_string(), MetaValue::Number(3.0));

        // Search with uppercase key should find lowercase stored value
        let quality = super::MeshQuality::from_meta(&meta);
        assert!((quality.max_edge_length - 3.0).abs() < 1e-9);

        // Also verify preset lookup works case-insensitively
        let mut meta2 = MetaMap::new();
        meta2.insert("mesh_quality".to_string(), MetaValue::Text("HIGH".to_string()));
        let quality2 = super::MeshQuality::from_meta(&meta2);
        assert_eq!(quality2.max_edge_length, 0.25);
    }

    #[test]
    fn mesh_quality_from_meta_or_default() {
        let empty = MetaMap::new();
        let quality = super::MeshQuality::from_meta_or_default(&empty);
        assert_eq!(quality, super::MeshQuality::medium());
    }

    #[test]
    fn mesh_quality_from_value_text() {
        let value = Value::Text("high".to_string());
        let quality = super::MeshQuality::from_value(&value).unwrap();
        assert_eq!(quality.max_edge_length, 0.25);
    }

    #[test]
    fn mesh_quality_from_value_number() {
        let value = Value::Number(1.0);
        let quality = super::MeshQuality::from_value(&value).unwrap();
        assert_eq!(quality, super::MeshQuality::medium()); // 1 = medium
    }

    #[test]
    fn mesh_quality_from_value_invalid() {
        let value = Value::Point([0.0, 0.0, 0.0]);
        assert!(super::MeshQuality::from_value(&value).is_none());

        let value = Value::Number(99.0); // Invalid preset index
        assert!(super::MeshQuality::from_value(&value).is_none());
    }

    #[test]
    fn mesh_quality_display() {
        let quality = super::MeshQuality::medium();
        let display = format!("{}", quality);
        assert!(display.contains("edge:"));
        assert!(display.contains("dev:"));
        assert!(display.contains("angle:"));
        assert!(display.contains("subdiv:"));
    }

    #[test]
    fn mesh_quality_tolerance_key() {
        // "tolerance" should map to max_deviation
        let mut meta = MetaMap::new();
        meta.insert("tolerance".to_string(), MetaValue::Number(0.015));

        let quality = super::MeshQuality::from_meta(&meta);
        assert!((quality.max_deviation - 0.015).abs() < 1e-9);
    }

    // ========================================================================
    // Tests for MeshDiagnostics conversion from GeomMeshDiagnostics
    // (only when mesh_engine_next feature is enabled)
    // ========================================================================

    #[cfg(feature = "mesh_engine_next")]
    mod geom_conversion_tests {
        use super::super::MeshDiagnostics;
        use crate::geom::GeomMeshDiagnostics;

        #[test]
        fn mesh_diagnostics_from_geom_owned() {
            let geom_diag = GeomMeshDiagnostics {
                vertex_count: 100,
                triangle_count: 50,
                welded_vertex_count: 10,
                flipped_triangle_count: 5,
                degenerate_triangle_count: 2,
                open_edge_count: 3,
                non_manifold_edge_count: 1,
                self_intersection_count: 0,
                boolean_fallback_used: true,
                timing: None,
                warnings: vec!["test warning".to_string()],
            };

            let diag: MeshDiagnostics = geom_diag.into();

            assert_eq!(diag.vertex_count, 100);
            assert_eq!(diag.triangle_count, 50);
            assert_eq!(diag.welded_vertex_count, 10);
            assert_eq!(diag.flipped_triangle_count, 5);
            assert_eq!(diag.degenerate_triangle_count, 2);
            assert_eq!(diag.open_edge_count, 3);
            assert_eq!(diag.non_manifold_edge_count, 1);
            assert_eq!(diag.self_intersection_count, 0);
            assert!(diag.boolean_fallback_used);
            assert_eq!(diag.warnings, vec!["test warning".to_string()]);
        }

        #[test]
        fn mesh_diagnostics_from_geom_ref() {
            let geom_diag = GeomMeshDiagnostics {
                vertex_count: 200,
                triangle_count: 100,
                welded_vertex_count: 20,
                flipped_triangle_count: 0,
                degenerate_triangle_count: 3,
                open_edge_count: 0,
                non_manifold_edge_count: 0,
                self_intersection_count: 2,
                boolean_fallback_used: false,
                timing: None,
                warnings: vec!["warning 1".to_string(), "warning 2".to_string()],
            };

            let diag = MeshDiagnostics::from(&geom_diag);

            assert_eq!(diag.vertex_count, 200);
            assert_eq!(diag.triangle_count, 100);
            assert_eq!(diag.welded_vertex_count, 20);
            assert_eq!(diag.self_intersection_count, 2);
            assert!(!diag.boolean_fallback_used);
            assert_eq!(diag.warnings.len(), 2);

            // Original should still be usable
            assert_eq!(geom_diag.vertex_count, 200);
        }

        #[test]
        fn mesh_diagnostics_to_geom() {
            let diag = MeshDiagnostics {
                vertex_count: 150,
                triangle_count: 75,
                welded_vertex_count: 5,
                flipped_triangle_count: 2,
                degenerate_triangle_count: 1,
                open_edge_count: 4,
                non_manifold_edge_count: 2,
                self_intersection_count: 0,
                boolean_fallback_used: true,
                warnings: vec!["test".to_string()],
            };

            let geom_diag: GeomMeshDiagnostics = diag.into();

            assert_eq!(geom_diag.vertex_count, 150);
            assert_eq!(geom_diag.triangle_count, 75);
            assert_eq!(geom_diag.welded_vertex_count, 5);
            assert_eq!(geom_diag.flipped_triangle_count, 2);
            assert_eq!(geom_diag.degenerate_triangle_count, 1);
            assert_eq!(geom_diag.open_edge_count, 4);
            assert_eq!(geom_diag.non_manifold_edge_count, 2);
            assert_eq!(geom_diag.self_intersection_count, 0);
            assert!(geom_diag.boolean_fallback_used);
            assert!(geom_diag.timing.is_none()); // Timing is not preserved
            assert_eq!(geom_diag.warnings, vec!["test".to_string()]);
        }

        #[test]
        fn mesh_diagnostics_from_geom_method() {
            let geom_diag = GeomMeshDiagnostics {
                vertex_count: 50,
                triangle_count: 25,
                ..Default::default()
            };

            let diag = MeshDiagnostics::from_geom(geom_diag);

            assert_eq!(diag.vertex_count, 50);
            assert_eq!(diag.triangle_count, 25);
            assert!(diag.is_clean());
        }

        #[test]
        fn mesh_diagnostics_from_geom_ref_method() {
            let geom_diag = GeomMeshDiagnostics {
                vertex_count: 75,
                triangle_count: 40,
                open_edge_count: 2,
                ..Default::default()
            };

            let diag = MeshDiagnostics::from_geom_ref(&geom_diag);

            assert_eq!(diag.vertex_count, 75);
            assert_eq!(diag.open_edge_count, 2);
            assert!(!diag.is_watertight());

            // Original should still be usable
            assert_eq!(geom_diag.triangle_count, 40);
        }

        #[test]
        fn mesh_diagnostics_to_geom_method() {
            let diag = MeshDiagnostics {
                vertex_count: 100,
                triangle_count: 50,
                non_manifold_edge_count: 1,
                ..Default::default()
            };

            let geom_diag = diag.to_geom();

            assert_eq!(geom_diag.vertex_count, 100);
            assert_eq!(geom_diag.non_manifold_edge_count, 1);
            assert!(!geom_diag.is_manifold());
        }

        #[test]
        fn mesh_diagnostics_roundtrip_conversion() {
            let original = MeshDiagnostics {
                vertex_count: 500,
                triangle_count: 250,
                welded_vertex_count: 25,
                flipped_triangle_count: 10,
                degenerate_triangle_count: 5,
                open_edge_count: 8,
                non_manifold_edge_count: 3,
                self_intersection_count: 1,
                boolean_fallback_used: true,
                warnings: vec!["warning A".to_string(), "warning B".to_string()],
            };

            // Convert to geom and back
            let geom_diag: GeomMeshDiagnostics = original.clone().into();
            let roundtrip: MeshDiagnostics = geom_diag.into();

            assert_eq!(roundtrip, original);
        }

        #[test]
        fn mesh_diagnostics_properties_preserved_through_conversion() {
            // Create a geom diagnostics with specific properties
            let geom_diag = GeomMeshDiagnostics {
                vertex_count: 100,
                triangle_count: 50,
                open_edge_count: 0,
                non_manifold_edge_count: 0,
                degenerate_triangle_count: 0,
                flipped_triangle_count: 0,
                self_intersection_count: 0,
                boolean_fallback_used: false,
                welded_vertex_count: 0,
                timing: None,
                warnings: vec![],
            };

            assert!(geom_diag.is_clean());
            assert!(geom_diag.is_watertight());
            assert!(geom_diag.is_manifold());
            assert!(geom_diag.is_valid_solid());

            // Convert and check that properties are preserved
            let diag = MeshDiagnostics::from_geom(geom_diag);

            assert!(diag.is_clean());
            assert!(diag.is_watertight());
            assert!(diag.is_manifold());
            assert!(diag.is_valid_solid());
        }
    }

    // ========================================================================
    // Tests for new expect_mesh_owned and into_mesh_data helpers
    // ========================================================================

    #[test]
    fn expect_mesh_owned_returns_owned_data() {
        let value = Value::Mesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: Some(vec![[0.0, 0.0, 1.0]; 3]),
            uvs: Some(vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]]),
            diagnostics: None,
        };

        let mesh_data = value.expect_mesh_owned().unwrap();
        assert_eq!(mesh_data.vertex_count(), 3);
        assert_eq!(mesh_data.triangle_count(), 1);
        assert!(mesh_data.has_normals());
        assert!(mesh_data.has_uvs());
    }

    #[test]
    fn expect_mesh_owned_rejects_surface() {
        let value = Value::Surface {
            vertices: vec![[0.0, 0.0, 0.0]],
            faces: vec![],
        };
        let err = value.expect_mesh_owned().unwrap_err();
        assert_eq!(err.expected(), "Mesh");
        assert_eq!(err.found(), ValueKind::Surface);
    }

    #[test]
    fn into_mesh_data_consumes_and_returns_owned() {
        let value = Value::Mesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: Some(vec![[0.0, 0.0, 1.0]; 3]),
            uvs: None,
            diagnostics: None,
        };

        let mesh_data = value.into_mesh_data().unwrap();
        assert_eq!(mesh_data.vertex_count(), 3);
        assert_eq!(mesh_data.triangle_count(), 1);
        assert!(mesh_data.has_normals());
        // value is consumed, cannot use it anymore
    }

    #[test]
    fn into_mesh_data_like_accepts_mesh() {
        let value = Value::Mesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: Some(vec![[0.0, 0.0, 1.0]; 3]),
            uvs: None,
            diagnostics: None,
        };

        let mesh_data = value.into_mesh_data_like().unwrap();
        assert_eq!(mesh_data.vertex_count(), 3);
        assert!(mesh_data.has_normals());
    }

    #[test]
    fn into_mesh_data_like_converts_surface() {
        let value = Value::Surface {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            faces: vec![vec![0, 1, 2], vec![0, 2, 3]],
        };

        let mesh_data = value.into_mesh_data_like().unwrap();
        assert_eq!(mesh_data.vertex_count(), 4);
        assert_eq!(mesh_data.triangle_count(), 2);
        assert!(!mesh_data.has_normals());
    }

    // ========================================================================
    // Tests for MeshData serialization helpers
    // ========================================================================

    #[test]
    fn mesh_data_to_vertices_f32() {
        let data = super::MeshData::new(
            vec![[0.0, 1.0, 2.0], [3.0, 4.0, 5.0]],
            vec![],
        );

        let f32_verts = data.to_vertices_f32();
        assert_eq!(f32_verts.len(), 6);
        assert_eq!(f32_verts[0], 0.0f32);
        assert_eq!(f32_verts[1], 1.0f32);
        assert_eq!(f32_verts[2], 2.0f32);
        assert_eq!(f32_verts[3], 3.0f32);
        assert_eq!(f32_verts[4], 4.0f32);
        assert_eq!(f32_verts[5], 5.0f32);
    }

    #[test]
    fn mesh_data_to_indices_u32() {
        let data = super::MeshData::new(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![0, 1, 2],
        );

        let u32_indices = data.to_indices_u32();
        assert_eq!(u32_indices, vec![0, 1, 2]);
    }

    #[test]
    fn mesh_data_to_normals_f32() {
        let data = super::MeshData::with_attributes(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            vec![],
            Some(vec![[0.0, 0.0, 1.0], [0.0, 1.0, 0.0]]),
            None,
        );

        let f32_normals = data.to_normals_f32().unwrap();
        assert_eq!(f32_normals.len(), 6);
        assert_eq!(f32_normals[0], 0.0f32);
        assert_eq!(f32_normals[1], 0.0f32);
        assert_eq!(f32_normals[2], 1.0f32);
        assert_eq!(f32_normals[3], 0.0f32);
        assert_eq!(f32_normals[4], 1.0f32);
        assert_eq!(f32_normals[5], 0.0f32);
    }

    #[test]
    fn mesh_data_to_normals_f32_none() {
        let data = super::MeshData::new(
            vec![[0.0, 0.0, 0.0]],
            vec![],
        );

        assert!(data.to_normals_f32().is_none());
    }

    #[test]
    fn mesh_data_to_uvs_f32() {
        let data = super::MeshData::with_attributes(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            vec![],
            None,
            Some(vec![[0.0, 0.0], [1.0, 1.0]]),
        );

        let f32_uvs = data.to_uvs_f32().unwrap();
        assert_eq!(f32_uvs.len(), 4);
        assert_eq!(f32_uvs[0], 0.0f32);
        assert_eq!(f32_uvs[1], 0.0f32);
        assert_eq!(f32_uvs[2], 1.0f32);
        assert_eq!(f32_uvs[3], 1.0f32);
    }

    #[test]
    fn mesh_data_summary() {
        let data = super::MeshData::with_attributes(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![0, 1, 2],
            Some(vec![[0.0, 0.0, 1.0]; 3]),
            Some(vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]]),
        );

        let summary = data.summary();
        assert!(summary.contains("V:3"));
        assert!(summary.contains("T:1"));
        assert!(summary.contains("+normals"));
        assert!(summary.contains("+uvs"));
    }

    // ========================================================================
    // Tests for From trait implementations
    // ========================================================================

    #[test]
    fn mesh_data_from_impl() {
        let data = super::MeshData::new(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![0, 1, 2],
        );

        let value: Value = data.into();
        assert_eq!(value.kind(), ValueKind::Mesh);

        let mesh_ref = value.expect_mesh().unwrap();
        assert_eq!(mesh_ref.vertex_count(), 3);
        assert_eq!(mesh_ref.triangle_count(), 1);
    }

    #[test]
    fn mesh_data_ref_from_impl() {
        let data = super::MeshData::new(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![0, 1, 2],
        );

        let value: Value = Value::from(&data);
        assert_eq!(value.kind(), ValueKind::Mesh);

        let mesh_ref = value.expect_mesh().unwrap();
        assert_eq!(mesh_ref.vertex_count(), 3);

        // Original data should still be usable
        assert_eq!(data.vertex_count(), 3);
    }

    #[test]
    fn mesh_diagnostics_to_value_conversion() {
        let diag = super::MeshDiagnostics {
            vertex_count: 100,
            triangle_count: 50,
            welded_vertex_count: 10,
            flipped_triangle_count: 0,
            degenerate_triangle_count: 2,
            open_edge_count: 0,
            non_manifold_edge_count: 0,
            self_intersection_count: 0,
            boolean_fallback_used: false,
            warnings: vec!["test warning".to_string()],
        };

        let value: Value = diag.into();
        assert_eq!(value.kind(), ValueKind::List);

        // Verify it's a list of key-value pairs
        if let Value::List(entries) = value {
            assert!(!entries.is_empty());
            // First entry should be vertex_count
            if let Value::List(ref first_entry) = entries[0] {
                assert_eq!(first_entry.len(), 2);
                if let Value::Text(ref key) = first_entry[0] {
                    assert_eq!(key, "vertex_count");
                }
                if let Value::Number(n) = first_entry[1] {
                    assert_eq!(n, 100.0);
                }
            }
        } else {
            panic!("Expected Value::List");
        }
    }

    // ========================================================================
    // Tests for new expect_* helpers (Text, Color, Material, Symbol)
    // ========================================================================

    #[test]
    fn expect_text_returns_text_ref() {
        let value = Value::Text("hello world".to_string());
        assert_eq!(value.expect_text().unwrap(), "hello world");
    }

    #[test]
    fn expect_text_rejects_non_text() {
        let value = Value::Number(42.0);
        let err = value.expect_text().unwrap_err();
        assert_eq!(err.expected(), "Text");
        assert_eq!(err.found(), ValueKind::Number);
    }

    #[test]
    fn expect_text_owned_returns_owned_string() {
        let value = Value::Text("hello world".to_string());
        let owned = value.expect_text_owned().unwrap();
        assert_eq!(owned, "hello world");
    }

    #[test]
    fn expect_color_returns_color_value() {
        let color = super::ColorValue::new(0.5, 0.25, 0.75);
        let value = Value::Color(color);
        let result = value.expect_color().unwrap();
        assert!((result.r - 0.5).abs() < 1e-9);
        assert!((result.g - 0.25).abs() < 1e-9);
        assert!((result.b - 0.75).abs() < 1e-9);
    }

    #[test]
    fn expect_color_rejects_non_color() {
        let value = Value::Point([0.0, 0.0, 0.0]);
        let err = value.expect_color().unwrap_err();
        assert_eq!(err.expected(), "Color");
        assert_eq!(err.found(), ValueKind::Point);
    }

    #[test]
    fn expect_material_returns_material_value() {
        let material = super::MaterialValue {
            diffuse: super::ColorValue::new(1.0, 0.0, 0.0),
            specular: super::ColorValue::new(1.0, 1.0, 1.0),
            emission: super::ColorValue::new(0.0, 0.0, 0.0),
            transparency: 0.5,
            shine: 30.0,
        };
        let value = Value::Material(material);
        let result = value.expect_material().unwrap();
        assert!((result.transparency - 0.5).abs() < 1e-9);
        assert!((result.shine - 30.0).abs() < 1e-9);
    }

    #[test]
    fn expect_material_rejects_non_material() {
        let value = Value::Boolean(true);
        let err = value.expect_material().unwrap_err();
        assert_eq!(err.expected(), "Material");
        assert_eq!(err.found(), ValueKind::Boolean);
    }

    #[test]
    fn expect_symbol_returns_symbol_ref() {
        let symbol = super::SymbolValue {
            style: "circle".to_string(),
            size_primary: 5.0,
            size_secondary: Some(3.0),
            rotation: 45.0,
            fill: super::ColorValue::new(1.0, 0.0, 0.0),
            edge: None,
            width: 1.0,
            adjust: true,
        };
        let value = Value::Symbol(symbol);
        let result = value.expect_symbol().unwrap();
        assert_eq!(result.style, "circle");
        assert!((result.size_primary - 5.0).abs() < 1e-9);
        assert!(result.adjust);
    }

    #[test]
    fn expect_symbol_rejects_non_symbol() {
        let value = Value::Null;
        let err = value.expect_symbol().unwrap_err();
        assert_eq!(err.expected(), "Symbol");
        assert_eq!(err.found(), ValueKind::Null);
    }

    // ========================================================================
    // Tests for MeshData value conversion helpers
    // ========================================================================

    #[test]
    fn mesh_data_from_value_accepts_mesh() {
        let value = Value::Mesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: Some(vec![[0.0, 0.0, 1.0]; 3]),
            uvs: None,
            diagnostics: None,
        };

        let mesh_data = super::MeshData::from_value(&value).unwrap();
        assert_eq!(mesh_data.vertex_count(), 3);
        assert_eq!(mesh_data.triangle_count(), 1);
        assert!(mesh_data.has_normals());
    }

    #[test]
    fn mesh_data_from_value_accepts_surface() {
        let value = Value::Surface {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            faces: vec![vec![0, 1, 2]],
        };

        let mesh_data = super::MeshData::from_value(&value).unwrap();
        assert_eq!(mesh_data.vertex_count(), 3);
        assert_eq!(mesh_data.triangle_count(), 1);
    }

    #[test]
    fn mesh_data_from_value_rejects_non_mesh() {
        let value = Value::Point([0.0, 0.0, 0.0]);
        assert!(super::MeshData::from_value(&value).is_err());
    }

    #[test]
    fn mesh_data_from_value_owned() {
        let value = Value::Mesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: None,
            uvs: None,
            diagnostics: None,
        };

        let mesh_data = super::MeshData::from_value_owned(value).unwrap();
        assert_eq!(mesh_data.vertex_count(), 3);
        // value is consumed
    }

    #[test]
    fn mesh_data_with_diagnostics() {
        let diag = super::MeshDiagnostics::with_counts(3, 1);
        let mesh = super::MeshData::with_diagnostics(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![0, 1, 2],
            None,
            None,
            diag,
        );

        assert!(mesh.diagnostics.is_some());
        assert_eq!(mesh.diagnostics.as_ref().unwrap().vertex_count, 3);
    }

    #[test]
    fn mesh_data_is_empty() {
        let empty = super::MeshData::new(vec![], vec![]);
        assert!(empty.is_empty());

        let non_empty = super::MeshData::new(
            vec![[0.0, 0.0, 0.0]],
            vec![],
        );
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn mesh_data_builder_methods() {
        let mesh = super::MeshData::new(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![0, 1, 2],
        )
        .with_normals(vec![[0.0, 0.0, 1.0]; 3])
        .with_uvs(vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]])
        .with_diagnostics_attached(super::MeshDiagnostics::with_counts(3, 1));

        assert!(mesh.has_normals());
        assert!(mesh.has_uvs());
        assert!(mesh.diagnostics.is_some());
    }

    // ========================================================================
    // Tests for MeshDiagnostics from_value deserialization
    // ========================================================================

    #[test]
    fn mesh_diagnostics_from_value_roundtrip() {
        let original = super::MeshDiagnostics {
            vertex_count: 100,
            triangle_count: 50,
            welded_vertex_count: 10,
            flipped_triangle_count: 5,
            degenerate_triangle_count: 2,
            open_edge_count: 3,
            non_manifold_edge_count: 1,
            self_intersection_count: 4,
            boolean_fallback_used: true,
            warnings: vec!["warning 1".to_string(), "warning 2".to_string()],
        };

        // Serialize to Value
        let value: Value = original.clone().into();

        // Deserialize back
        let parsed = super::MeshDiagnostics::from_value(&value).unwrap();

        assert_eq!(parsed.vertex_count, original.vertex_count);
        assert_eq!(parsed.triangle_count, original.triangle_count);
        assert_eq!(parsed.welded_vertex_count, original.welded_vertex_count);
        assert_eq!(parsed.flipped_triangle_count, original.flipped_triangle_count);
        assert_eq!(parsed.degenerate_triangle_count, original.degenerate_triangle_count);
        assert_eq!(parsed.open_edge_count, original.open_edge_count);
        assert_eq!(parsed.non_manifold_edge_count, original.non_manifold_edge_count);
        assert_eq!(parsed.self_intersection_count, original.self_intersection_count);
        assert_eq!(parsed.boolean_fallback_used, original.boolean_fallback_used);
        assert_eq!(parsed.warnings, original.warnings);
    }

    #[test]
    fn mesh_diagnostics_from_value_handles_missing_keys() {
        // Create a partial diagnostics value (only some keys)
        let value = Value::List(vec![
            Value::List(vec![
                Value::Text("vertex_count".to_string()),
                Value::Number(50.0),
            ]),
            Value::List(vec![
                Value::Text("triangle_count".to_string()),
                Value::Number(25.0),
            ]),
        ]);

        let parsed = super::MeshDiagnostics::from_value(&value).unwrap();

        assert_eq!(parsed.vertex_count, 50);
        assert_eq!(parsed.triangle_count, 25);
        // Other fields should be default (0 or false)
        assert_eq!(parsed.open_edge_count, 0);
        assert!(!parsed.boolean_fallback_used);
        assert!(parsed.warnings.is_empty());
    }

    #[test]
    fn mesh_diagnostics_from_value_rejects_non_list() {
        let value = Value::Number(42.0);
        assert!(super::MeshDiagnostics::from_value(&value).is_err());
    }

    #[test]
    fn mesh_diagnostics_with_counts_constructor() {
        let diag = super::MeshDiagnostics::with_counts(200, 100);
        assert_eq!(diag.vertex_count, 200);
        assert_eq!(diag.triangle_count, 100);
        assert!(diag.is_clean());
    }

    #[test]
    fn mesh_diagnostics_with_updated_counts() {
        let diag = super::MeshDiagnostics {
            vertex_count: 10,
            triangle_count: 5,
            open_edge_count: 2,
            ..Default::default()
        };

        let updated = diag.with_updated_counts(100, 50);
        assert_eq!(updated.vertex_count, 100);
        assert_eq!(updated.triangle_count, 50);
        // Other fields should be preserved
        assert_eq!(updated.open_edge_count, 2);
    }
}