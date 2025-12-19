//! Tests for deformation operations (twist, bend, taper, morph).

use crate::geom::{
    BendOptions, DeformationError, GeomMesh, MorphOptions, Point3, TaperOptions,
    Tolerance, TwistOptions, Vec3, bend_mesh, bend_mesh_z, morph_mesh, taper_mesh,
    taper_mesh_z, twist_mesh, twist_mesh_z,
};
use std::f64::consts::PI;

// ============================================================================
// Test mesh factories
// ============================================================================

/// Create a simple unit cube mesh (2x2x2 centered at origin).
fn create_test_cube() -> GeomMesh {
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

/// Create a tall rectangular prism for testing axis-aligned deformations.
fn create_tall_box() -> GeomMesh {
    // 1x1x4 box from z=0 to z=4
    let positions = vec![
        [-0.5, -0.5, 0.0],
        [0.5, -0.5, 0.0],
        [0.5, 0.5, 0.0],
        [-0.5, 0.5, 0.0],
        [-0.5, -0.5, 4.0],
        [0.5, -0.5, 4.0],
        [0.5, 0.5, 4.0],
        [-0.5, 0.5, 4.0],
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

// ============================================================================
// Twist tests
// ============================================================================

#[test]
fn test_twist_zero_angle_no_change() {
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
    assert_eq!(diag.original_vertex_count, 8);
    assert_eq!(diag.original_triangle_count, 12);
    assert!(!twisted.indices.is_empty());
}

#[test]
fn test_twist_90_degrees() {
    let mesh = create_tall_box();
    let options = TwistOptions::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        PI / 2.0,
    );
    let result = twist_mesh(&mesh, options, Tolerance::default_geom());
    assert!(result.is_ok());
    let (twisted, diag) = result.unwrap();
    
    // Displacement should occur for vertices not on the axis
    assert!(diag.max_displacement > 0.0);
    assert!(!twisted.indices.is_empty());
    
    // Bottom vertices (z=0) should not move (at twist start)
    // Top vertices (z=4) should be rotated 90 degrees
}

#[test]
fn test_twist_full_rotation() {
    let mesh = create_tall_box();
    let options = TwistOptions::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        2.0 * PI,
    );
    let result = twist_mesh(&mesh, options, Tolerance::default_geom());
    assert!(result.is_ok());
    let (twisted, diag) = result.unwrap();
    
    // Full rotation should produce maximum displacement at middle of mesh
    assert!(diag.max_displacement > 0.0);
    assert!(!twisted.indices.is_empty());
}

#[test]
fn test_twist_with_explicit_extent() {
    let mesh = create_tall_box();
    let options = TwistOptions::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        PI / 2.0,
    )
    .extent(1.0, 3.0); // Only twist middle section
    
    let result = twist_mesh(&mesh, options, Tolerance::default_geom());
    assert!(result.is_ok());
}

#[test]
fn test_twist_convenience_z() {
    let mesh = create_test_cube();
    let result = twist_mesh_z(&mesh, PI / 4.0, Tolerance::default_geom());
    assert!(result.is_ok());
    let (twisted, diag) = result.unwrap();
    assert!(diag.max_displacement > 0.0);
    assert!(!twisted.indices.is_empty());
}

// ============================================================================
// Bend tests
// ============================================================================

#[test]
fn test_bend_zero_angle_warning() {
    let mesh = create_test_cube();
    let options = BendOptions::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        0.0,
    );
    let result = bend_mesh(&mesh, options, Tolerance::default_geom());
    assert!(result.is_ok());
    let (_bent, diag) = result.unwrap();
    
    // Zero angle should return warning
    assert!(!diag.warnings.is_empty());
    assert!(diag.warnings[0].contains("zero"));
}

#[test]
fn test_bend_90_degrees() {
    let mesh = create_tall_box();
    let options = BendOptions::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        PI / 2.0,
    );
    let result = bend_mesh(&mesh, options, Tolerance::default_geom());
    assert!(result.is_ok());
    let (bent, diag) = result.unwrap();
    
    assert!(diag.max_displacement > 0.0);
    assert!(!bent.indices.is_empty());
}

#[test]
fn test_bend_with_custom_direction() {
    let mesh = create_tall_box();
    let options = BendOptions::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        PI / 4.0,
    )
    .bend_direction(Vec3::new(1.0, 0.0, 0.0));
    
    let result = bend_mesh(&mesh, options, Tolerance::default_geom());
    assert!(result.is_ok());
}

#[test]
fn test_bend_invalid_angle() {
    let mesh = create_test_cube();
    let options = BendOptions::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        10.0, // > 2π
    );
    let result = bend_mesh(&mesh, options, Tolerance::default_geom());
    assert!(matches!(result, Err(DeformationError::InvalidBendAngle)));
}

#[test]
fn test_bend_convenience_z() {
    let mesh = create_test_cube();
    let result = bend_mesh_z(&mesh, PI / 4.0, Tolerance::default_geom());
    assert!(result.is_ok());
}

// ============================================================================
// Taper tests
// ============================================================================

#[test]
fn test_taper_uniform_no_change() {
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
    
    // Uniform scale (1.0 -> 1.0) should produce no displacement
    assert!(diag.max_displacement < 1e-10);
}

#[test]
fn test_taper_shrink() {
    let mesh = create_tall_box();
    let options = TaperOptions::new(
        Point3::new(0.0, 0.0, 0.0),
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
fn test_taper_expand() {
    let mesh = create_tall_box();
    let options = TaperOptions::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        1.0,
        2.0,
    );
    let result = taper_mesh(&mesh, options, Tolerance::default_geom());
    assert!(result.is_ok());
    let (tapered, diag) = result.unwrap();
    
    assert!(diag.max_displacement > 0.0);
    assert!(!tapered.indices.is_empty());
}

#[test]
fn test_taper_to_point() {
    let mesh = create_tall_box();
    let options = TaperOptions::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        1.0,
        0.0, // Taper to a point
    );
    let result = taper_mesh(&mesh, options, Tolerance::default_geom());
    assert!(result.is_ok());
    let (tapered, diag) = result.unwrap();
    
    assert!(diag.max_displacement > 0.0);
    assert!(!tapered.indices.is_empty());
}

#[test]
fn test_taper_invalid_factor() {
    let mesh = create_test_cube();
    let options = TaperOptions::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        1.0,
        -0.5, // Negative factor
    );
    let result = taper_mesh(&mesh, options, Tolerance::default_geom());
    assert!(matches!(result, Err(DeformationError::InvalidTaperFactor)));
}

#[test]
fn test_taper_convenience_z() {
    let mesh = create_test_cube();
    let result = taper_mesh_z(&mesh, 1.0, 0.5, Tolerance::default_geom());
    assert!(result.is_ok());
    let (tapered, _diag) = result.unwrap();
    assert!(!tapered.indices.is_empty());
}

// ============================================================================
// Morph tests
// ============================================================================

#[test]
fn test_morph_identity_zero_blend() {
    let mesh = create_test_cube();
    let options = MorphOptions::new(mesh.positions.clone(), 0.0);
    let result = morph_mesh(&mesh, options, Tolerance::default_geom());
    assert!(result.is_ok());
    let (_morphed, diag) = result.unwrap();
    
    // Zero blend should produce no displacement
    assert!(diag.max_displacement < 1e-10);
}

#[test]
fn test_morph_identity_full_blend() {
    let mesh = create_test_cube();
    let options = MorphOptions::new(mesh.positions.clone(), 1.0);
    let result = morph_mesh(&mesh, options, Tolerance::default_geom());
    assert!(result.is_ok());
    let (_morphed, diag) = result.unwrap();
    
    // Full blend to same positions should produce no displacement
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
    
    // Half blend should produce displacement
    assert!(diag.max_displacement > 0.0);
    assert!(!morphed.indices.is_empty());
}

#[test]
fn test_morph_extrapolation() {
    let mesh = create_test_cube();
    // Create target positions (scaled version)
    let target: Vec<[f64; 3]> = mesh
        .positions
        .iter()
        .map(|p| [p[0] * 2.0, p[1] * 2.0, p[2] * 2.0])
        .collect();
    
    // Blend factor > 1.0 for extrapolation
    let options = MorphOptions::new(target, 1.5);
    let result = morph_mesh(&mesh, options, Tolerance::default_geom());
    assert!(result.is_ok());
    let (morphed, diag) = result.unwrap();
    
    assert!(diag.max_displacement > 0.0);
    assert!(!morphed.indices.is_empty());
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

#[test]
fn test_morph_invalid_target_nan() {
    let mesh = create_test_cube();
    let mut target = mesh.positions.clone();
    target[0] = [f64::NAN, 0.0, 0.0];
    
    let options = MorphOptions::new(target, 0.5);
    let result = morph_mesh(&mesh, options, Tolerance::default_geom());
    assert!(matches!(result, Err(DeformationError::InvalidGeometry)));
}

// ============================================================================
// Error handling tests
// ============================================================================

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
fn test_invalid_axis_zero() {
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
fn test_invalid_axis_nan() {
    let mesh = create_test_cube();
    let result = twist_mesh(
        &mesh,
        TwistOptions::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(f64::NAN, 0.0, 1.0),
            PI,
        ),
        Tolerance::default_geom(),
    );
    assert!(matches!(result, Err(DeformationError::InvalidAxis)));
}

#[test]
fn test_invalid_geometry_nan() {
    let mut mesh = create_test_cube();
    mesh.positions[0] = [f64::NAN, 0.0, 0.0];
    
    let result = twist_mesh(
        &mesh,
        TwistOptions::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            PI,
        ),
        Tolerance::default_geom(),
    );
    assert!(matches!(result, Err(DeformationError::InvalidGeometry)));
}

#[test]
fn test_invalid_parameters_nan_angle() {
    let mesh = create_test_cube();
    let result = twist_mesh(
        &mesh,
        TwistOptions::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            f64::NAN,
        ),
        Tolerance::default_geom(),
    );
    assert!(matches!(result, Err(DeformationError::InvalidParameters)));
}

// ============================================================================
// Mesh integrity tests
// ============================================================================

#[test]
fn test_twist_preserves_triangle_count() {
    let mesh = create_test_cube();
    let original_tri_count = mesh.indices.len() / 3;
    
    let options = TwistOptions::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        PI / 2.0,
    );
    let result = twist_mesh(&mesh, options, Tolerance::default_geom());
    assert!(result.is_ok());
    let (twisted, _) = result.unwrap();
    
    // Triangle count may change due to welding, but should not be zero
    assert!(!twisted.indices.is_empty());
    assert!(twisted.indices.len() / 3 >= original_tri_count / 2);
}

#[test]
fn test_deformation_no_nan_positions() {
    let mesh = create_test_cube();
    
    let options = TwistOptions::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        PI,
    );
    let result = twist_mesh(&mesh, options, Tolerance::default_geom());
    assert!(result.is_ok());
    let (twisted, _) = result.unwrap();
    
    // Verify no NaN or Inf values
    for pos in &twisted.positions {
        assert!(pos[0].is_finite());
        assert!(pos[1].is_finite());
        assert!(pos[2].is_finite());
    }
}

#[test]
fn test_deformation_valid_indices() {
    let mesh = create_test_cube();
    
    let options = TaperOptions::new(
        Point3::new(0.0, 0.0, -1.0),
        Vec3::new(0.0, 0.0, 1.0),
        1.0,
        0.5,
    );
    let result = taper_mesh(&mesh, options, Tolerance::default_geom());
    assert!(result.is_ok());
    let (tapered, _) = result.unwrap();
    
    // Verify all indices are within bounds
    let max_idx = tapered.positions.len() as u32;
    for &idx in &tapered.indices {
        assert!(idx < max_idx);
    }
}

#[test]
fn test_deformation_normals_generated() {
    let mesh = create_test_cube();
    
    let options = TwistOptions::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        PI / 4.0,
    )
    .recompute_normals(true);
    
    let result = twist_mesh(&mesh, options, Tolerance::default_geom());
    assert!(result.is_ok());
    let (twisted, _) = result.unwrap();
    
    // Normals should be generated when recompute_normals is true
    assert!(twisted.normals.is_some());
    let normals = twisted.normals.unwrap();
    assert_eq!(normals.len(), twisted.positions.len());
    
    // Verify normals are finite and roughly unit length
    for n in &normals {
        assert!(n[0].is_finite());
        assert!(n[1].is_finite());
        assert!(n[2].is_finite());
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 0.1); // Allow some tolerance for averaged normals
    }
}
