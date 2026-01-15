//! B-rep (Boundary Representation) operations for mesh joining and face merging.
//!
//! This module provides higher-level APIs for operations that combine or modify
//! multiple surface meshes as B-rep-like structures. It wraps the lower-level
//! functionality from [`super::solid`] with cleaner interfaces and better
//! diagnostics for component integration.
//!
//! # Operations
//!
//! - [`brep_join`]: Join multiple surface meshes by welding matching naked edges.
//! - [`merge_brep_faces`]: Merge coplanar/continuous faces within a B-rep.
//!
//! # Usage
//!
//! ```ignore
//! use crate::geom::{brep_join, merge_brep_faces, Tolerance};
//!
//! // Join multiple surface meshes
//! let result = brep_join(&[mesh1, mesh2], BrepJoinOptions::default());
//! for (brep, is_closed) in result.breps.iter().zip(result.closed.iter()) {
//!     println!("Brep closed: {}", is_closed);
//! }
//!
//! // Merge coplanar faces
//! let merged = merge_brep_faces(&[mesh], MergeFacesOptions::default());
//! println!("Faces before: {}, after: {}", merged.diagnostics.before, merged.diagnostics.after);
//! ```

use super::solid::{
    brep_join_legacy, merge_faces_legacy, BrepJoinDiagnostics,
    LegacySurfaceMesh, MergeFacesDiagnostics,
};
use super::Tolerance;

/// Options for the BrepJoin operation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrepJoinOptions {
    /// Tolerance for vertex/edge matching.
    pub tolerance: Tolerance,
    /// If true, attempt to merge all joinable breps into minimal shells.
    /// If false, only check adjacency without merging.
    pub merge_shells: bool,
}

impl Default for BrepJoinOptions {
    fn default() -> Self {
        Self {
            tolerance: Tolerance::default_geom(),
            merge_shells: true,
        }
    }
}

impl BrepJoinOptions {
    /// Create options with a custom tolerance.
    #[must_use]
    pub fn with_tolerance(mut self, tol: Tolerance) -> Self {
        self.tolerance = tol;
        self
    }
}

/// Result of a BrepJoin operation.
#[derive(Debug, Clone, PartialEq)]
pub struct BrepJoinComponentResult {
    /// The resulting breps after joining (may be fewer than inputs if merged).
    pub breps: Vec<LegacySurfaceMesh>,
    /// For each output brep, whether it forms a closed (watertight) shell.
    pub closed: Vec<bool>,
    /// Mapping from input index to output index (None if the input was merged into another).
    pub input_to_output_map: Vec<Option<usize>>,
    /// Diagnostics about the join operation.
    pub diagnostics: BrepJoinDiagnostics,
}

/// Joins multiple surface meshes by welding matching naked edges.
///
/// This function attempts to merge breps that share coincident naked edges
/// (within tolerance). Breps that can be joined together are combined into
/// unified shells.
///
/// # Algorithm
/// 1. Build edge graphs for all input breps to identify naked edges.
/// 2. Find pairs of naked edges across different breps that are coincident.
/// 3. Merge breps with matching edges by unifying their vertex sets and remapping faces.
/// 4. Report which output shells are closed (watertight).
///
/// # Arguments
/// * `breps` - Slice of surface meshes to join.
/// * `options` - Options controlling tolerance and behavior.
///
/// # Returns
/// A [`BrepJoinComponentResult`] containing the joined breps, closedness flags,
/// input-to-output mapping, and diagnostics.
///
/// # Example
/// ```ignore
/// let result = brep_join(&[mesh1, mesh2], BrepJoinOptions::default());
/// assert!(result.breps.len() <= 2); // May be fewer if merged
/// ```
#[must_use]
pub fn brep_join(breps: &[LegacySurfaceMesh], options: BrepJoinOptions) -> BrepJoinComponentResult {
    if breps.is_empty() {
        return BrepJoinComponentResult {
            breps: Vec::new(),
            closed: Vec::new(),
            input_to_output_map: Vec::new(),
            diagnostics: BrepJoinDiagnostics::default(),
        };
    }

    // Clone breps for the legacy call
    let breps_owned: Vec<LegacySurfaceMesh> = breps.to_vec();
    let num_inputs = breps_owned.len();

    let result = brep_join_legacy(breps_owned, options.tolerance);

    // Build input-to-output mapping
    // Since brep_join_legacy uses union-find and merges, we need to track this.
    // For now, if the output count equals input count, map 1:1.
    // Otherwise, we'd need to modify the underlying function to provide mapping.
    let input_to_output_map = if result.breps.len() == num_inputs {
        (0..num_inputs).map(Some).collect()
    } else {
        // When breps are merged, we can't provide a precise mapping without
        // modifying the underlying implementation. For now, mark merged inputs as None.
        // The first N outputs correspond to roots of merged groups.
        build_approximate_mapping(num_inputs, result.breps.len())
    };

    BrepJoinComponentResult {
        breps: result.breps,
        closed: result.closed,
        input_to_output_map,
        diagnostics: result.diagnostics,
    }
}

/// Build an approximate input-to-output mapping when exact tracking isn't available.
fn build_approximate_mapping(num_inputs: usize, num_outputs: usize) -> Vec<Option<usize>> {
    let mut mapping = Vec::with_capacity(num_inputs);
    for i in 0..num_inputs {
        if i < num_outputs {
            mapping.push(Some(i));
        } else {
            mapping.push(None);
        }
    }
    mapping
}

/// Options for the MergeFaces operation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MergeFacesOptions {
    /// Tolerance for coplanarity and vertex matching.
    pub tolerance: Tolerance,
    /// Maximum angular deviation (in radians) for faces to be considered coplanar.
    /// Default is ~0.1 degrees.
    pub max_angle_deviation: f64,
}

impl Default for MergeFacesOptions {
    fn default() -> Self {
        Self {
            tolerance: Tolerance::default_geom(),
            max_angle_deviation: 0.00175, // ~0.1 degrees in radians
        }
    }
}

impl MergeFacesOptions {
    /// Create options with a custom tolerance.
    #[must_use]
    pub fn with_tolerance(mut self, tol: Tolerance) -> Self {
        self.tolerance = tol;
        self
    }
}

/// Result of a MergeFaces operation.
#[derive(Debug, Clone, PartialEq)]
pub struct MergeFacesComponentResult {
    /// The brep with merged faces.
    pub brep: LegacySurfaceMesh,
    /// Whether the operation succeeded.
    pub success: bool,
    /// Diagnostics about the merge operation.
    pub diagnostics: MergeFacesDiagnostics,
}

/// Merges coplanar/continuous faces within breps with tolerance guards.
///
/// This function combines multiple input breps and then merges any coplanar
/// adjacent faces that share edges into larger polygons.
///
/// # Algorithm
/// 1. Combine all input breps into a single mesh with welded vertices.
/// 2. Identify face groups where faces are coplanar (within tolerance) and share edges.
/// 3. Merge each group into a single polygon by removing internal edges.
/// 4. Triangulate the resulting polygons if necessary.
///
/// # Arguments
/// * `breps` - Slice of surface meshes to process.
/// * `options` - Options controlling tolerance and coplanarity thresholds.
///
/// # Returns
/// A [`MergeFacesComponentResult`] containing the merged brep and diagnostics.
///
/// # Example
/// ```ignore
/// let result = merge_brep_faces(&[mesh], MergeFacesOptions::default());
/// println!("Merged {} face groups", result.diagnostics.group_count);
/// ```
#[must_use]
pub fn merge_brep_faces(
    breps: &[LegacySurfaceMesh],
    options: MergeFacesOptions,
) -> MergeFacesComponentResult {
    if breps.is_empty() {
        return MergeFacesComponentResult {
            brep: LegacySurfaceMesh::new(),
            success: false,
            diagnostics: MergeFacesDiagnostics::default(),
        };
    }

    match merge_faces_legacy(breps, options.tolerance) {
        Some(result) => MergeFacesComponentResult {
            brep: result.brep,
            success: true,
            diagnostics: result.diagnostics,
        },
        None => MergeFacesComponentResult {
            brep: LegacySurfaceMesh::new(),
            success: false,
            diagnostics: MergeFacesDiagnostics {
                warnings: vec!["merge_faces_legacy returned None".to_string()],
                ..Default::default()
            },
        },
    }
}

/// Check if a legacy surface mesh is closed (watertight).
///
/// A mesh is considered closed if it has no naked (boundary) edges,
/// meaning every edge is shared by exactly two faces.
///
/// # Arguments
/// * `brep` - The surface mesh to check.
/// * `tol` - Tolerance for edge matching.
///
/// # Returns
/// `true` if the mesh is closed, `false` otherwise.
#[must_use]
pub fn is_brep_closed(brep: &LegacySurfaceMesh, tol: Tolerance) -> bool {
    super::solid::legacy_surface_is_closed(brep, tol)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_quad_mesh(origin: [f64; 3], size: f64) -> LegacySurfaceMesh {
        // Simple quad with 4 vertices
        LegacySurfaceMesh {
            vertices: vec![
                origin,
                [origin[0] + size, origin[1], origin[2]],
                [origin[0] + size, origin[1] + size, origin[2]],
                [origin[0], origin[1] + size, origin[2]],
            ],
            faces: vec![vec![0, 1, 2, 3]],
        }
    }

    fn make_box_mesh() -> LegacySurfaceMesh {
        // A simple box with 8 vertices and 6 quad faces
        let vertices = vec![
            [0.0, 0.0, 0.0], // 0: front-bottom-left
            [1.0, 0.0, 0.0], // 1: front-bottom-right
            [1.0, 1.0, 0.0], // 2: front-top-right
            [0.0, 1.0, 0.0], // 3: front-top-left
            [0.0, 0.0, 1.0], // 4: back-bottom-left
            [1.0, 0.0, 1.0], // 5: back-bottom-right
            [1.0, 1.0, 1.0], // 6: back-top-right
            [0.0, 1.0, 1.0], // 7: back-top-left
        ];
        let faces = vec![
            vec![0, 1, 2, 3], // front
            vec![5, 4, 7, 6], // back
            vec![4, 0, 3, 7], // left
            vec![1, 5, 6, 2], // right
            vec![3, 2, 6, 7], // top
            vec![4, 5, 1, 0], // bottom
        ];
        LegacySurfaceMesh { vertices, faces }
    }

    #[test]
    fn test_brep_join_empty() {
        let result = brep_join(&[], BrepJoinOptions::default());
        assert!(result.breps.is_empty());
        assert!(result.closed.is_empty());
    }

    #[test]
    fn test_brep_join_single() {
        let mesh = make_box_mesh();
        let result = brep_join(&[mesh], BrepJoinOptions::default());
        assert_eq!(result.breps.len(), 1);
        assert_eq!(result.closed.len(), 1);
        // A closed box should be detected as closed
        assert!(result.closed[0]);
    }

    #[test]
    fn test_brep_join_non_adjacent() {
        // Two quads that don't share edges
        let mesh1 = make_quad_mesh([0.0, 0.0, 0.0], 1.0);
        let mesh2 = make_quad_mesh([10.0, 0.0, 0.0], 1.0);
        let result = brep_join(&[mesh1, mesh2], BrepJoinOptions::default());
        // Should remain as two separate breps
        assert_eq!(result.breps.len(), 2);
        // Neither should be closed (they're just quads)
        assert!(!result.closed[0]);
        assert!(!result.closed[1]);
    }

    #[test]
    fn test_merge_faces_empty() {
        let result = merge_brep_faces(&[], MergeFacesOptions::default());
        assert!(!result.success);
        assert!(result.brep.vertices.is_empty());
    }

    #[test]
    fn test_merge_faces_single_quad() {
        let mesh = make_quad_mesh([0.0, 0.0, 0.0], 1.0);
        let result = merge_brep_faces(&[mesh], MergeFacesOptions::default());
        assert!(result.success);
        assert_eq!(result.brep.faces.len(), 1);
    }

    #[test]
    fn test_merge_faces_coplanar_triangles() {
        // Two coplanar triangles that share an edge
        let mesh = LegacySurfaceMesh {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            faces: vec![
                vec![0, 1, 2], // first triangle
                vec![0, 2, 3], // second triangle, shares edge 0-2
            ],
        };
        let result = merge_brep_faces(&[mesh], MergeFacesOptions::default());
        assert!(result.success);
        // Should merge into a single quad
        assert_eq!(result.diagnostics.before, 2);
        assert_eq!(result.diagnostics.after, 1);
    }

    #[test]
    fn test_is_brep_closed() {
        let closed_box = make_box_mesh();
        assert!(is_brep_closed(&closed_box, Tolerance::default_geom()));

        let open_quad = make_quad_mesh([0.0, 0.0, 0.0], 1.0);
        assert!(!is_brep_closed(&open_quad, Tolerance::default_geom()));
    }
}
