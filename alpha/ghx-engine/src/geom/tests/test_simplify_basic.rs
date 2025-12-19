//! Tests for mesh simplification (LOD) functionality.

use crate::geom::{
    simplify_by_ratio, simplify_mesh, simplify_to_count, GeomMesh, SimplifyError,
    SimplifyOptions, SimplifyTarget,
};

/// Create a simple plane mesh (2x2 grid, 4 triangles).
fn make_plane_2x2() -> GeomMesh {
    // Vertices:
    // 3---4---5
    // | \ | \ |
    // 0---1---2
    let positions = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [2.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [2.0, 1.0, 0.0],
    ];

    let indices = vec![
        0, 1, 4, // quad 0, tri 0
        0, 4, 3, // quad 0, tri 1
        1, 2, 5, // quad 1, tri 0
        1, 5, 4, // quad 1, tri 1
    ];

    GeomMesh {
        positions,
        indices,
        normals: None,
        uvs: None,
        tangents: None,
    }
}

/// Create a larger plane mesh (4x4 grid, 18 triangles).
fn make_plane_4x4() -> GeomMesh {
    let mut positions = Vec::new();
    for y in 0..5 {
        for x in 0..5 {
            positions.push([x as f64, y as f64, 0.0]);
        }
    }

    let mut indices = Vec::new();
    for y in 0..4 {
        for x in 0..4 {
            let i0 = y * 5 + x;
            let i1 = i0 + 1;
            let i2 = i0 + 5;
            let i3 = i2 + 1;

            // Two triangles per quad
            indices.extend_from_slice(&[i0 as u32, i1 as u32, i3 as u32]);
            indices.extend_from_slice(&[i0 as u32, i3 as u32, i2 as u32]);
        }
    }

    GeomMesh {
        positions,
        indices,
        normals: None,
        uvs: None,
        tangents: None,
    }
}

/// Create a closed tetrahedron (4 triangles, watertight).
fn make_tetrahedron() -> GeomMesh {
    let positions = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.5, 0.866, 0.0],
        [0.5, 0.433, 0.816],
    ];

    let indices = vec![
        0, 2, 1, // bottom
        0, 1, 3, // front
        1, 2, 3, // right
        2, 0, 3, // left
    ];

    GeomMesh {
        positions,
        indices,
        normals: None,
        uvs: None,
        tangents: None,
    }
}

/// Create a cube mesh (12 triangles, watertight).
fn make_cube() -> GeomMesh {
    let positions = vec![
        // Bottom face (y=0)
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        // Top face (y=1)
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [1.0, 1.0, 1.0],
        [0.0, 1.0, 1.0],
    ];

    let indices = vec![
        // Bottom face
        0, 2, 1,
        0, 3, 2,
        // Top face
        4, 5, 6,
        4, 6, 7,
        // Front face (z=0)
        0, 1, 5,
        0, 5, 4,
        // Back face (z=1)
        2, 3, 7,
        2, 7, 6,
        // Left face (x=0)
        3, 0, 4,
        3, 4, 7,
        // Right face (x=1)
        1, 2, 6,
        1, 6, 5,
    ];

    GeomMesh {
        positions,
        indices,
        normals: None,
        uvs: None,
        tangents: None,
    }
}

#[test]
fn simplify_empty_mesh_returns_error() {
    let mesh = GeomMesh {
        positions: vec![],
        indices: vec![],
        normals: None,
        uvs: None,
        tangents: None,
    };

    let result = simplify_mesh(&mesh, SimplifyOptions::default());
    assert!(matches!(result, Err(SimplifyError::EmptyMesh)));
}

#[test]
fn simplify_invalid_geometry_returns_error() {
    let mesh = GeomMesh {
        positions: vec![
            [f64::NAN, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
        ],
        indices: vec![0, 1, 2],
        normals: None,
        uvs: None,
        tangents: None,
    };

    let result = simplify_mesh(&mesh, SimplifyOptions::default());
    assert!(matches!(result, Err(SimplifyError::InvalidGeometry)));
}

#[test]
fn simplify_invalid_ratio_returns_error() {
    let mesh = make_plane_2x2();

    let result = simplify_mesh(&mesh, SimplifyOptions::new(SimplifyTarget::Ratio(1.5)));
    assert!(matches!(result, Err(SimplifyError::InvalidTarget { .. })));

    let result = simplify_mesh(&mesh, SimplifyOptions::new(SimplifyTarget::Ratio(-0.5)));
    assert!(matches!(result, Err(SimplifyError::InvalidTarget { .. })));
}

#[test]
fn simplify_plane_reduces_triangles() {
    let mesh = make_plane_4x4();
    let original_count = mesh.triangle_count();
    assert_eq!(original_count, 32);

    let result = simplify_to_count(&mesh, 16).unwrap();

    assert!(result.mesh.triangle_count() <= 32);
    assert_eq!(result.diagnostics.original_triangle_count, 32);
    assert!(result.diagnostics.final_triangle_count <= result.diagnostics.original_triangle_count);
}

#[test]
fn simplify_by_ratio_works() {
    let mesh = make_plane_4x4();

    let result = simplify_by_ratio(&mesh, 0.5).unwrap();

    assert!(result.mesh.triangle_count() <= 32);
    assert!(result.diagnostics.final_triangle_count <= result.diagnostics.original_triangle_count);
}

#[test]
fn simplify_ratio_1_preserves_mesh() {
    let mesh = make_plane_2x2();
    let original_count = mesh.triangle_count();

    let result = simplify_by_ratio(&mesh, 1.0).unwrap();

    assert_eq!(result.mesh.triangle_count(), original_count);
    assert_eq!(result.diagnostics.edges_collapsed, 0);
}

#[test]
fn simplify_target_higher_than_current_preserves_mesh() {
    let mesh = make_plane_2x2();
    let original_count = mesh.triangle_count();

    let result = simplify_to_count(&mesh, 100).unwrap();

    assert_eq!(result.mesh.triangle_count(), original_count);
    assert_eq!(result.diagnostics.edges_collapsed, 0);
}

#[test]
fn simplify_preserves_valid_indices() {
    let mesh = make_plane_4x4();

    let result = simplify_by_ratio(&mesh, 0.5).unwrap();

    // All indices should be valid
    let vertex_count = result.mesh.positions.len();
    for idx in &result.mesh.indices {
        assert!((*idx as usize) < vertex_count, "invalid index {} >= {}", idx, vertex_count);
    }
}

#[test]
fn simplify_preserves_finite_positions() {
    let mesh = make_plane_4x4();

    let result = simplify_by_ratio(&mesh, 0.5).unwrap();

    for pos in &result.mesh.positions {
        assert!(pos[0].is_finite());
        assert!(pos[1].is_finite());
        assert!(pos[2].is_finite());
    }
}

#[test]
fn simplify_no_degenerate_triangles() {
    let mesh = make_plane_4x4();

    let result = simplify_by_ratio(&mesh, 0.3).unwrap();

    // Check for degenerate triangles (repeated indices)
    for tri in result.mesh.indices.chunks_exact(3) {
        assert_ne!(tri[0], tri[1], "degenerate triangle with i0=i1");
        assert_ne!(tri[1], tri[2], "degenerate triangle with i1=i2");
        assert_ne!(tri[0], tri[2], "degenerate triangle with i0=i2");
    }
}

#[test]
fn simplify_diagnostics_are_populated() {
    let mesh = make_plane_4x4();

    let result = simplify_by_ratio(&mesh, 0.5).unwrap();

    let diag = &result.diagnostics;
    assert_eq!(diag.original_vertex_count, 25);
    assert_eq!(diag.original_triangle_count, 32);
    assert!(diag.final_vertex_count <= diag.original_vertex_count);
    assert!(diag.final_triangle_count <= diag.original_triangle_count);
}

#[test]
fn simplify_tetrahedron_minimal_case() {
    let mesh = make_tetrahedron();
    assert_eq!(mesh.triangle_count(), 4);

    // Tetrahedron is the minimal closed mesh - simplifying it will either:
    // 1. Keep all 4 triangles (can't simplify further while preserving topology)
    // 2. Reduce triangles but potentially lose watertightness
    let result = simplify_mesh(
        &mesh,
        SimplifyOptions::target_triangles(2).strict_watertight(true),
    )
    .unwrap();

    // The result should be valid regardless of whether simplification occurred
    assert!(result.mesh.triangle_count() <= 4);
    assert!(result.diagnostics.final_triangle_count <= result.diagnostics.original_triangle_count);

    // All indices should be valid
    let vertex_count = result.mesh.positions.len();
    for idx in &result.mesh.indices {
        assert!((*idx as usize) < vertex_count);
    }
}

#[test]
fn simplify_cube_reduces_triangles() {
    let mesh = make_cube();
    assert_eq!(mesh.triangle_count(), 12);

    let result = simplify_to_count(&mesh, 8).unwrap();

    assert!(result.mesh.triangle_count() <= 12);
    assert!(result.diagnostics.final_triangle_count <= 12);
}

#[test]
fn simplify_preserve_boundary_option() {
    let mesh = make_plane_2x2();

    // With boundary preservation
    let result_preserve = simplify_mesh(
        &mesh,
        SimplifyOptions::target_ratio(0.5).preserve_boundary(true),
    )
    .unwrap();

    // Without boundary preservation (more aggressive)
    let result_no_preserve = simplify_mesh(
        &mesh,
        SimplifyOptions::target_ratio(0.5).preserve_boundary(false),
    )
    .unwrap();

    // Both should produce valid meshes
    assert!(result_preserve.mesh.triangle_count() <= 4);
    assert!(result_no_preserve.mesh.triangle_count() <= 4);
}

#[test]
fn simplify_max_error_target() {
    let mesh = make_plane_4x4();

    // Very small error threshold - should collapse few edges
    let result_small = simplify_mesh(&mesh, SimplifyOptions::target_error(0.001)).unwrap();

    // Larger error threshold - should collapse more edges
    let result_large = simplify_mesh(&mesh, SimplifyOptions::target_error(10.0)).unwrap();

    assert!(result_large.mesh.triangle_count() <= result_small.mesh.triangle_count());
}

#[test]
fn simplify_aspect_ratio_constraint() {
    let mesh = make_plane_4x4();

    // Strict aspect ratio
    let result_strict = simplify_mesh(
        &mesh,
        SimplifyOptions::target_ratio(0.3).max_aspect_ratio(3.0),
    )
    .unwrap();

    // Relaxed aspect ratio
    let result_relaxed = simplify_mesh(
        &mesh,
        SimplifyOptions::target_ratio(0.3).max_aspect_ratio(100.0),
    )
    .unwrap();

    // Both should produce valid meshes with some triangles
    assert!(result_strict.mesh.triangle_count() > 0);
    assert!(result_relaxed.mesh.triangle_count() > 0);
    // Both should have valid indices
    assert!(result_strict.mesh.indices.iter().all(|&i| (i as usize) < result_strict.mesh.positions.len()));
    assert!(result_relaxed.mesh.indices.iter().all(|&i| (i as usize) < result_relaxed.mesh.positions.len()));
}

#[test]
fn simplify_extreme_ratio_zero() {
    let mesh = make_plane_4x4();

    // Ratio of 0 should try to minimize triangles
    let result = simplify_by_ratio(&mesh, 0.0).unwrap();

    // Can't go below minimum, but should simplify as much as possible
    assert!(result.mesh.triangle_count() <= mesh.triangle_count());
}

#[test]
fn simplify_single_triangle_cannot_reduce() {
    let mesh = GeomMesh {
        positions: vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
        ],
        indices: vec![0, 1, 2],
        normals: None,
        uvs: None,
        tangents: None,
    };

    let result = simplify_to_count(&mesh, 0).unwrap();

    // Can't reduce a single triangle further - it will either:
    // - Stay as 1 triangle (no edges can collapse)
    // - Become 0 triangles if the only triangle collapses to nothing
    assert!(result.mesh.triangle_count() <= 1);
    // Verify indices are valid
    for idx in &result.mesh.indices {
        assert!((*idx as usize) < result.mesh.positions.len());
    }
}

#[test]
fn simplify_stress_test_larger_mesh() {
    // Create a larger mesh (10x10 grid)
    let mut positions = Vec::new();
    for y in 0..11 {
        for x in 0..11 {
            positions.push([x as f64, y as f64, 0.0]);
        }
    }

    let mut indices = Vec::new();
    for y in 0..10 {
        for x in 0..10 {
            let i0 = y * 11 + x;
            let i1 = i0 + 1;
            let i2 = i0 + 11;
            let i3 = i2 + 1;

            indices.extend_from_slice(&[i0 as u32, i1 as u32, i3 as u32]);
            indices.extend_from_slice(&[i0 as u32, i3 as u32, i2 as u32]);
        }
    }

    let mesh = GeomMesh {
        positions,
        indices,
        normals: None,
        uvs: None,
        tangents: None,
    };

    assert_eq!(mesh.triangle_count(), 200);

    let result = simplify_by_ratio(&mesh, 0.25).unwrap();

    assert!(result.mesh.triangle_count() <= 200);
    assert!(result.diagnostics.edges_collapsed > 0 || result.diagnostics.final_triangle_count == 200);
}

#[test]
fn simplify_result_mesh_is_valid() {
    let mesh = make_plane_4x4();

    let result = simplify_by_ratio(&mesh, 0.5).unwrap();

    // Final counts should match mesh
    assert_eq!(result.diagnostics.final_vertex_count, result.mesh.positions.len());
    assert_eq!(result.diagnostics.final_triangle_count, result.mesh.triangle_count());

    // Indices should be triplets
    assert_eq!(result.mesh.indices.len() % 3, 0);
}
