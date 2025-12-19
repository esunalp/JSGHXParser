//! Mesh diagnostics for the geometry engine.
//!
//! This module provides diagnostic information about mesh quality, topology,
//! and repair operations. Diagnostics are collected during mesh generation
//! and can be used for:
//!
//! - Validating mesh quality (watertight, manifold, no degenerates)
//! - Debugging mesh issues (open edges, self-intersections)
//! - Tracking repair operations (welding, flipping, boolean fallbacks)
//! - Performance profiling (timing buckets)
//!
//! # Example
//!
//! ```ignore
//! use ghx_engine::geom::{mesh_surface, GeomMeshDiagnostics};
//!
//! let (mesh, diagnostics) = mesh_surface(&surface, 10, 10);
//!
//! if diagnostics.is_watertight() {
//!     println!("Mesh is watertight with {} triangles", diagnostics.triangle_count);
//! } else {
//!     println!("Mesh has {} open edges", diagnostics.open_edge_count);
//! }
//!
//! // Check for any issues
//! if !diagnostics.is_clean() {
//!     for warning in &diagnostics.warnings {
//!         eprintln!("Warning: {}", warning);
//!     }
//! }
//! ```

use std::fmt;

/// Comprehensive diagnostics for mesh generation and repair operations.
///
/// This struct collects information about mesh topology, quality metrics,
/// repair operations performed, and timing data. It is returned alongside
/// the mesh from all `geom` meshing functions.
///
/// # Topology Metrics
///
/// - `open_edge_count`: Edges with only one adjacent triangle (holes in mesh)
/// - `non_manifold_edge_count`: Edges with more than two adjacent triangles
///
/// # Quality Metrics
///
/// - `degenerate_triangle_count`: Zero-area or collapsed triangles removed
/// - `self_intersection_count`: Self-intersecting triangles detected (future)
///
/// # Repair Statistics
///
/// - `welded_vertex_count`: Vertices merged during tolerance-based welding
/// - `flipped_triangle_count`: Triangles with corrected winding order
/// - `boolean_fallback_used`: Whether CSG required tolerance relaxation or voxel fallback
///
/// # Performance
///
/// - `timing`: Optional timing breakdown by operation category
#[derive(Debug, Default, Clone, PartialEq)]
pub struct GeomMeshDiagnostics {
    /// Total number of vertices in the final mesh.
    pub vertex_count: usize,

    /// Total number of triangles in the final mesh.
    pub triangle_count: usize,

    /// Number of vertices merged during tolerance-based welding.
    ///
    /// A high value relative to `vertex_count` may indicate overlapping geometry
    /// or a tolerance that is too loose.
    pub welded_vertex_count: usize,

    /// Number of triangles whose winding order was corrected for consistency.
    ///
    /// Non-zero values indicate the input had inconsistent face orientations.
    pub flipped_triangle_count: usize,

    /// Number of degenerate (zero-area) triangles removed.
    ///
    /// These are triangles where all three vertices are collinear or coincident
    /// within tolerance.
    pub degenerate_triangle_count: usize,

    /// Number of open (boundary) edges in the mesh.
    ///
    /// A watertight mesh has zero open edges. Open edges indicate holes or
    /// incomplete geometry.
    pub open_edge_count: usize,

    /// Number of non-manifold edges in the mesh.
    ///
    /// Non-manifold edges have more than two adjacent triangles, which is
    /// geometrically invalid for a proper solid. Zero is expected for clean meshes.
    pub non_manifold_edge_count: usize,

    /// Number of self-intersecting triangle pairs detected.
    ///
    /// Currently reserved for future use; set to 0 by default.
    /// Will be populated by boolean operations and mesh validation passes.
    pub self_intersection_count: usize,

    /// Whether a boolean operation required a fallback strategy.
    ///
    /// This is set to `true` when:
    /// - Tolerance had to be relaxed to complete the operation
    /// - A voxel-based fallback was used instead of exact intersection
    /// - The operation completed but with reduced precision
    ///
    /// When `true`, check `warnings` for details about what fallback was used.
    pub boolean_fallback_used: bool,

    /// Optional timing breakdown by operation category.
    ///
    /// Only populated when the `mesh_engine_metrics` feature is enabled
    /// and the target is not WASM.
    pub timing: Option<super::metrics::GeomTimingReport>,

    /// Human-readable warnings about mesh issues and repairs performed.
    ///
    /// Examples:
    /// - "mesh orientation flipped (outward)"
    /// - "mesh has open edges"
    /// - "mesh has non-manifold edges"
    /// - "boolean used tolerance relaxation fallback"
    pub warnings: Vec<String>,
}

impl GeomMeshDiagnostics {
    /// Creates a new empty diagnostics struct with all counts at zero.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if the mesh is watertight (no open edges).
    ///
    /// A watertight mesh forms a closed volume with no holes or gaps.
    #[must_use]
    pub fn is_watertight(&self) -> bool {
        self.open_edge_count == 0
    }

    /// Returns `true` if the mesh is manifold (no non-manifold edges).
    ///
    /// A manifold mesh has at most two triangles sharing each edge,
    /// which is required for valid solid geometry.
    #[must_use]
    pub fn is_manifold(&self) -> bool {
        self.non_manifold_edge_count == 0
    }

    /// Returns `true` if the mesh is both watertight and manifold.
    ///
    /// This is the minimum requirement for a valid solid mesh.
    #[must_use]
    pub fn is_valid_solid(&self) -> bool {
        self.is_watertight() && self.is_manifold()
    }

    /// Returns `true` if no issues were detected and no repairs were needed.
    ///
    /// A "clean" mesh has:
    /// - No open edges
    /// - No non-manifold edges
    /// - No degenerate triangles removed
    /// - No winding corrections needed
    /// - No boolean fallbacks used
    /// - No warnings
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

    /// Returns the total number of topology issues (open + non-manifold edges).
    #[must_use]
    pub fn topology_issue_count(&self) -> usize {
        self.open_edge_count + self.non_manifold_edge_count
    }

    /// Returns the total number of repairs performed (welded + flipped + degenerates).
    #[must_use]
    pub fn repair_count(&self) -> usize {
        self.welded_vertex_count + self.flipped_triangle_count + self.degenerate_triangle_count
    }

    /// Adds a warning message to the diagnostics.
    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }

    /// Merges another diagnostics struct into this one.
    ///
    /// This is useful when combining meshes or aggregating diagnostics
    /// from multiple operations. Counts are summed, warnings are appended,
    /// and boolean flags are OR'd together.
    ///
    /// Note: `timing` from `other` is ignored; use a parent `GeomMetrics`
    /// to track timing across multiple operations.
    pub fn merge(&mut self, other: &GeomMeshDiagnostics) {
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
        // timing is not merged; use a parent GeomMetrics for multi-op timing
    }

    /// Returns a short summary string suitable for logging.
    ///
    /// Format: `"V:{vertices} T:{triangles} [issues...]"`
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
}

impl fmt::Display for GeomMeshDiagnostics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Mesh Diagnostics:")?;
        writeln!(f, "  Vertices: {}", self.vertex_count)?;
        writeln!(f, "  Triangles: {}", self.triangle_count)?;

        if self.welded_vertex_count > 0 || self.flipped_triangle_count > 0 || self.degenerate_triangle_count > 0 {
            writeln!(f, "  Repairs:")?;
            if self.welded_vertex_count > 0 {
                writeln!(f, "    - Welded vertices: {}", self.welded_vertex_count)?;
            }
            if self.flipped_triangle_count > 0 {
                writeln!(f, "    - Flipped triangles: {}", self.flipped_triangle_count)?;
            }
            if self.degenerate_triangle_count > 0 {
                writeln!(f, "    - Degenerate triangles removed: {}", self.degenerate_triangle_count)?;
            }
        }

        if self.open_edge_count > 0 || self.non_manifold_edge_count > 0 || self.self_intersection_count > 0 {
            writeln!(f, "  Topology issues:")?;
            if self.open_edge_count > 0 {
                writeln!(f, "    - Open edges: {}", self.open_edge_count)?;
            }
            if self.non_manifold_edge_count > 0 {
                writeln!(f, "    - Non-manifold edges: {}", self.non_manifold_edge_count)?;
            }
            if self.self_intersection_count > 0 {
                writeln!(f, "    - Self-intersections: {}", self.self_intersection_count)?;
            }
        }

        if self.boolean_fallback_used {
            writeln!(f, "  Boolean: fallback strategy used")?;
        }

        if !self.warnings.is_empty() {
            writeln!(f, "  Warnings:")?;
            for warning in &self.warnings {
                writeln!(f, "    - {}", warning)?;
            }
        }

        if let Some(ref timing) = self.timing {
            writeln!(f, "  Timing: {} ms total", timing.total_ms())?;
        }

        let status = if self.is_clean() {
            "CLEAN"
        } else if self.is_valid_solid() {
            "VALID (with repairs)"
        } else {
            "ISSUES DETECTED"
        };
        writeln!(f, "  Status: {}", status)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_clean() {
        let diag = GeomMeshDiagnostics::default();
        assert!(diag.is_clean());
        assert!(diag.is_watertight());
        assert!(diag.is_manifold());
        assert!(diag.is_valid_solid());
        assert!(!diag.has_warnings());
    }

    #[test]
    fn test_open_edges_not_watertight() {
        let diag = GeomMeshDiagnostics {
            open_edge_count: 3,
            ..Default::default()
        };
        assert!(!diag.is_watertight());
        assert!(diag.is_manifold());
        assert!(!diag.is_valid_solid());
        assert!(!diag.is_clean());
    }

    #[test]
    fn test_non_manifold_not_manifold() {
        let diag = GeomMeshDiagnostics {
            non_manifold_edge_count: 2,
            ..Default::default()
        };
        assert!(diag.is_watertight());
        assert!(!diag.is_manifold());
        assert!(!diag.is_valid_solid());
    }

    #[test]
    fn test_merge() {
        let mut diag1 = GeomMeshDiagnostics {
            vertex_count: 100,
            triangle_count: 50,
            open_edge_count: 2,
            warnings: vec!["first warning".to_string()],
            ..Default::default()
        };

        let diag2 = GeomMeshDiagnostics {
            vertex_count: 200,
            triangle_count: 100,
            open_edge_count: 3,
            boolean_fallback_used: true,
            warnings: vec!["second warning".to_string()],
            ..Default::default()
        };

        diag1.merge(&diag2);

        assert_eq!(diag1.vertex_count, 300);
        assert_eq!(diag1.triangle_count, 150);
        assert_eq!(diag1.open_edge_count, 5);
        assert!(diag1.boolean_fallback_used);
        assert_eq!(diag1.warnings.len(), 2);
    }

    #[test]
    fn test_summary() {
        let diag = GeomMeshDiagnostics {
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
    fn test_display() {
        let diag = GeomMeshDiagnostics {
            vertex_count: 100,
            triangle_count: 50,
            open_edge_count: 2,
            warnings: vec!["test warning".to_string()],
            ..Default::default()
        };

        let output = format!("{}", diag);
        assert!(output.contains("Vertices: 100"));
        assert!(output.contains("Triangles: 50"));
        assert!(output.contains("Open edges: 2"));
        assert!(output.contains("test warning"));
        assert!(output.contains("ISSUES DETECTED"));
    }

    #[test]
    fn test_topology_and_repair_counts() {
        let diag = GeomMeshDiagnostics {
            welded_vertex_count: 10,
            flipped_triangle_count: 5,
            degenerate_triangle_count: 3,
            open_edge_count: 2,
            non_manifold_edge_count: 1,
            ..Default::default()
        };

        assert_eq!(diag.topology_issue_count(), 3);
        assert_eq!(diag.repair_count(), 18);
    }

    #[test]
    fn test_add_warning() {
        let mut diag = GeomMeshDiagnostics::default();
        assert!(!diag.has_warnings());

        diag.add_warning("test warning");
        assert!(diag.has_warnings());
        assert_eq!(diag.warnings.len(), 1);
        assert_eq!(diag.warnings[0], "test warning");
    }
}
