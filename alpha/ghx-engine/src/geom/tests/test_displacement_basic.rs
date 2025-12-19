//! Tests for displacement operations.

use crate::geom::{
    DisplacementError, DisplacementOptions, DisplacementSource,
    GeomMesh, Tolerance, Vec3,
    displace_mesh, displace_mesh_heightfield, displace_mesh_noise,
    displace_mesh_per_vertex, displace_mesh_uniform,
};

/// Create a simple planar quad mesh for testing.
fn make_quad_mesh() -> GeomMesh {
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

/// Create a slightly larger mesh for more robust testing.
fn make_grid_mesh(n: usize) -> GeomMesh {
    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut normals = Vec::new();
    
    for j in 0..n {
        for i in 0..n {
            let x = i as f64 / (n - 1) as f64;
            let y = j as f64 / (n - 1) as f64;
            positions.push([x, y, 0.0]);
            uvs.push([x, y]);
            normals.push([0.0, 0.0, 1.0]);
        }
    }

    let mut indices = Vec::new();
    for j in 0..(n - 1) {
        for i in 0..(n - 1) {
            let idx = (j * n + i) as u32;
            // First triangle
            indices.push(idx);
            indices.push(idx + 1);
            indices.push(idx + n as u32);
            // Second triangle
            indices.push(idx + 1);
            indices.push(idx + n as u32 + 1);
            indices.push(idx + n as u32);
        }
    }

    GeomMesh {
        positions,
        indices,
        uvs: Some(uvs),
        normals: Some(normals),
        tangents: None,
    }
}

#[test]
fn uniform_displacement_moves_all_vertices_equally() {
    let mesh = make_quad_mesh();
    let tol = Tolerance::default_geom();

    let (displaced, diag) = displace_mesh_uniform(&mesh, 0.5, tol).unwrap();

    // All vertices should move 0.5 in Z
    for pos in &displaced.positions {
        assert!(
            (pos[2] - 0.5).abs() < 1e-6,
            "Expected Z=0.5, got Z={}",
            pos[2]
        );
    }

    assert_eq!(diag.original_vertex_count, 4);
    assert!((diag.avg_displacement_applied - 0.5).abs() < 1e-6);
    assert!((diag.min_displacement_applied - 0.5).abs() < 1e-6);
    assert!((diag.max_displacement_applied - 0.5).abs() < 1e-6);
}

#[test]
fn per_vertex_displacement_applies_individual_values() {
    let mesh = make_quad_mesh();
    let tol = Tolerance::default_geom();

    let values = vec![0.1, 0.2, 0.3, 0.4];
    let (displaced, diag) = displace_mesh_per_vertex(&mesh, values.clone(), tol).unwrap();

    // Each vertex should have its own displacement
    for (i, pos) in displaced.positions.iter().enumerate() {
        assert!(
            (pos[2] - values[i]).abs() < 1e-6,
            "Vertex {}: expected Z={}, got Z={}",
            i,
            values[i],
            pos[2]
        );
    }

    assert!((diag.min_displacement_applied - 0.1).abs() < 1e-6);
    assert!((diag.max_displacement_applied - 0.4).abs() < 1e-6);
}

#[test]
fn per_vertex_count_mismatch_returns_error() {
    let mesh = make_quad_mesh();
    let tol = Tolerance::default_geom();

    let values = vec![0.1, 0.2]; // Wrong count
    let result = displace_mesh_per_vertex(&mesh, values, tol);

    assert!(matches!(
        result,
        Err(DisplacementError::VertexCountMismatch { expected: 4, got: 2 })
    ));
}

#[test]
fn heightfield_displacement_samples_grid() {
    let mesh = make_quad_mesh();
    let tol = Tolerance::default_geom();

    // 2x2 heightfield: corners are 0, 0.5, 0.5, 1.0
    let heightfield = vec![0.0, 0.5, 0.5, 1.0];
    let (displaced, _) = displace_mesh_heightfield(&mesh, heightfield, 2, 2, 1.0, tol).unwrap();

    // Corner at UV (0,0) should get value 0
    assert!((displaced.positions[0][2] - 0.0).abs() < 1e-6);
    // Corner at UV (1,1) should get value 1.0
    assert!((displaced.positions[2][2] - 1.0).abs() < 1e-6);
}

#[test]
fn heightfield_without_uvs_returns_error() {
    let mut mesh = make_quad_mesh();
    mesh.uvs = None;
    let tol = Tolerance::default_geom();

    let heightfield = vec![0.0, 0.5, 0.5, 1.0];
    let result = displace_mesh_heightfield(&mesh, heightfield, 2, 2, 1.0, tol);

    assert!(matches!(result, Err(DisplacementError::MissingUvs)));
}

#[test]
fn heightfield_invalid_dimensions_returns_error() {
    let mesh = make_quad_mesh();
    let tol = Tolerance::default_geom();

    // 3 values but claiming 2x2 grid (needs 4)
    let heightfield = vec![0.0, 0.5, 1.0];
    let result = displace_mesh_heightfield(&mesh, heightfield, 2, 2, 1.0, tol);

    assert!(matches!(
        result,
        Err(DisplacementError::InvalidHeightfieldDimensions { width: 2, height: 2, value_count: 3 })
    ));
}

#[test]
fn gradient_displacement_varies_across_mesh() {
    let mesh = make_grid_mesh(5);
    let tol = Tolerance::default_geom();

    let options = DisplacementOptions::new(DisplacementSource::gradient(
        Vec3::new(1.0, 0.0, 0.0), // X-direction
        0.0,
        1.0,
    ));

    let (displaced, diag) = displace_mesh(&mesh, options, tol).unwrap();

    // Vertices at x=0 should be near 0, vertices at x=1 should be near 1
    assert!(diag.min_displacement_applied < 0.1);
    assert!(diag.max_displacement_applied > 0.9);

    // Check all positions are finite
    for pos in &displaced.positions {
        assert!(pos[0].is_finite());
        assert!(pos[1].is_finite());
        assert!(pos[2].is_finite());
    }
}

#[test]
fn noise_displacement_is_deterministic() {
    let mesh = make_quad_mesh();
    let tol = Tolerance::default_geom();

    let (result1, _) = displace_mesh_noise(&mesh, 1.0, 0.5, 42, tol).unwrap();
    let (result2, _) = displace_mesh_noise(&mesh, 1.0, 0.5, 42, tol).unwrap();

    // Same seed should produce identical results
    for (p1, p2) in result1.positions.iter().zip(result2.positions.iter()) {
        assert!((p1[0] - p2[0]).abs() < 1e-12);
        assert!((p1[1] - p2[1]).abs() < 1e-12);
        assert!((p1[2] - p2[2]).abs() < 1e-12);
    }
}

#[test]
fn noise_displacement_different_seeds_produce_different_results() {
    let mesh = make_quad_mesh();
    let tol = Tolerance::default_geom();

    let (result1, _) = displace_mesh_noise(&mesh, 1.0, 0.5, 42, tol).unwrap();
    let (result2, _) = displace_mesh_noise(&mesh, 1.0, 0.5, 999, tol).unwrap();

    // Different seeds should produce at least one different Z value
    let any_different = result1
        .positions
        .iter()
        .zip(result2.positions.iter())
        .any(|(p1, p2)| (p1[2] - p2[2]).abs() > 1e-9);

    assert!(any_different, "Different seeds should produce different noise");
}

#[test]
fn displacement_clamping_applies_correctly() {
    let mesh = make_quad_mesh();
    let tol = Tolerance::default_geom();

    let options = DisplacementOptions::new(DisplacementSource::uniform(10.0))
        .max_displacement(1.0)
        .min_displacement(-1.0);

    let (displaced, diag) = displace_mesh(&mesh, options, tol).unwrap();

    // All values should be clamped to 1.0
    for pos in &displaced.positions {
        assert!(
            (pos[2] - 1.0).abs() < 1e-6,
            "Expected clamped Z=1.0, got Z={}",
            pos[2]
        );
    }

    assert_eq!(diag.clamped_vertex_count, 4);
    assert!((diag.max_displacement_applied - 1.0).abs() < 1e-6);
}

#[test]
fn empty_mesh_returns_error() {
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
fn mesh_with_nan_returns_error() {
    let mesh = GeomMesh {
        positions: vec![
            [f64::NAN, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
        ],
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
fn mesh_with_inf_returns_error() {
    let mesh = GeomMesh {
        positions: vec![
            [f64::INFINITY, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
        ],
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
fn uniform_nan_displacement_returns_error() {
    let mesh = make_quad_mesh();
    let tol = Tolerance::default_geom();

    let options = DisplacementOptions::new(DisplacementSource::uniform(f64::NAN));
    let result = displace_mesh(&mesh, options, tol);

    assert!(matches!(
        result,
        Err(DisplacementError::InvalidDisplacementValues)
    ));
}

#[test]
fn per_vertex_nan_displacement_returns_error() {
    let mesh = make_quad_mesh();
    let tol = Tolerance::default_geom();

    let values = vec![0.0, f64::NAN, 0.5, 0.5];
    let result = displace_mesh_per_vertex(&mesh, values, tol);

    assert!(matches!(
        result,
        Err(DisplacementError::InvalidDisplacementValues)
    ));
}

#[test]
fn displacement_without_normals_uses_fallback() {
    let mesh = GeomMesh {
        positions: vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
        ],
        indices: vec![0, 1, 2],
        uvs: None,
        normals: None, // No normals provided
        tangents: None,
    };
    let tol = Tolerance::default_geom();

    // Should still work - normals computed from face
    let result = displace_mesh_uniform(&mesh, 0.5, tol);
    assert!(result.is_ok());

    let (displaced, _) = result.unwrap();
    // All vertices should be displaced (in some direction)
    for pos in &displaced.positions {
        assert!(pos[2].is_finite());
    }
}

#[test]
fn use_normals_false_displaces_along_z() {
    let mesh = make_quad_mesh();
    let tol = Tolerance::default_geom();

    let options = DisplacementOptions::new(DisplacementSource::uniform(0.5))
        .use_normals(false); // Displace along Z axis

    let (displaced, _) = displace_mesh(&mesh, options, tol).unwrap();

    // All Z values should increase by 0.5
    for (i, pos) in displaced.positions.iter().enumerate() {
        let expected_z = mesh.positions[i][2] + 0.5;
        assert!(
            (pos[2] - expected_z).abs() < 1e-6,
            "Vertex {}: expected Z={}, got Z={}",
            i,
            expected_z,
            pos[2]
        );
    }
}

#[test]
fn diagnostics_reports_correct_statistics() {
    let mesh = make_quad_mesh();
    let tol = Tolerance::default_geom();

    let values = vec![0.1, 0.2, 0.3, 0.4];
    let (_, diag) = displace_mesh_per_vertex(&mesh, values, tol).unwrap();

    assert_eq!(diag.original_vertex_count, 4);
    assert_eq!(diag.original_triangle_count, 2);
    assert!(diag.result_vertex_count > 0);
    assert!(diag.result_triangle_count > 0);

    // Check stats
    assert!((diag.min_displacement_applied - 0.1).abs() < 1e-6);
    assert!((diag.max_displacement_applied - 0.4).abs() < 1e-6);
    let expected_avg = (0.1 + 0.2 + 0.3 + 0.4) / 4.0;
    assert!((diag.avg_displacement_applied - expected_avg).abs() < 1e-6);
}

#[test]
fn negative_displacement_moves_vertices_inward() {
    let mesh = make_quad_mesh();
    let tol = Tolerance::default_geom();

    let (displaced, _) = displace_mesh_uniform(&mesh, -0.5, tol).unwrap();

    // All Z values should decrease by 0.5
    for pos in &displaced.positions {
        assert!(
            (pos[2] - (-0.5)).abs() < 1e-6,
            "Expected Z=-0.5, got Z={}",
            pos[2]
        );
    }
}
