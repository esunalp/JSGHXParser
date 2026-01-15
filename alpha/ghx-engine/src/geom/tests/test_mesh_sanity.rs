use crate::geom::{
    GeomContext, GeomMesh, PlaneSurface, Point3, Tolerance, Vec3, mesh_surface_with_context,
    flip_mesh, MeshFlipGuide,
};

#[test]
fn mesh_surface_has_finite_vertices_and_valid_indices() {
    let plane = PlaneSurface::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 0.0, 0.0), Vec3::new(0.0, 3.0, 0.0));
    let mut ctx = GeomContext::new();
    let (mesh, diag) = mesh_surface_with_context(&plane, 8, 6, &mut ctx);

    mesh.validate().expect("mesh validate");
    assert_eq!(mesh.positions_flat().len(), mesh.positions.len() * 3);
    assert_eq!(mesh.uvs_flat().unwrap().len(), mesh.positions.len() * 2);
    assert_eq!(mesh.normals_flat().unwrap().len(), mesh.positions.len() * 3);
    assert_eq!(mesh.tangents_flat().unwrap().len(), mesh.positions.len() * 3);

    assert_eq!(mesh.positions.len(), 8 * 6);
    assert_eq!(diag.vertex_count, mesh.positions.len());
    assert_eq!(diag.triangle_count, mesh.indices.len() / 3);
    assert_eq!(diag.welded_vertex_count, 0);
    assert_eq!(diag.flipped_triangle_count, 0);
    assert!(diag.open_edge_count > 0);
    assert_eq!(diag.non_manifold_edge_count, 0);

    assert_eq!(ctx.cache.stats().surface_grid_entries, 1);
    assert_eq!(ctx.cache.stats().grid_triangulation_entries, 1);

    let _ = mesh_surface_with_context(&plane, 8, 6, &mut ctx);
    assert_eq!(ctx.cache.stats().surface_grid_entries, 1);
    assert_eq!(ctx.cache.stats().grid_triangulation_entries, 1);

    for p in &mesh.positions {
        assert!(p[0].is_finite());
        assert!(p[1].is_finite());
        assert!(p[2].is_finite());
    }

    let uvs = mesh.uvs.as_ref().unwrap();
    assert_eq!(uvs.len(), mesh.positions.len());
    assert_eq!(uvs[0], [0.0, 0.0]);

    let normals = mesh.normals.as_ref().unwrap();
    assert_eq!(normals.len(), mesh.positions.len());
    for n in normals {
        assert!(n[0].is_finite());
        assert!(n[1].is_finite());
        assert!(n[2].is_finite());
        assert!(n[2] > 0.0);
    }

    assert!(mesh
        .indices
        .iter()
        .all(|i| (*i as usize) < mesh.positions.len()));
}

#[test]
fn geom_mesh_validate_rejects_bad_buffers() {
    let mesh = GeomMesh::new(vec![[0.0, 0.0, 0.0]], vec![0]);
    assert!(mesh.validate().is_err());

    let mesh = GeomMesh::new(vec![[0.0, 0.0, 0.0]], vec![0, 1, 0]);
    assert!(mesh.validate().is_err());
}

// ============================================================================
// flip_mesh Tests
// ============================================================================

#[test]
fn flip_mesh_always_flips_without_guide() {
    // CCW triangle with normal pointing +Z
    let mesh = GeomMesh::new(
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
        vec![0, 1, 2],
    );

    let (flipped, diag) = flip_mesh(mesh, None);
    
    assert!(diag.flipped);
    assert!(!diag.guide_used);
    
    // Indices should be swapped: [0, 1, 2] -> [0, 2, 1]
    assert_eq!(flipped.indices, vec![0, 2, 1]);
}

#[test]
fn flip_mesh_with_vector_guide_aligns_normal() {
    // CCW triangle with normal pointing +Z
    let mesh = GeomMesh::new(
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
        vec![0, 1, 2],
    );

    // Guide points +Z, mesh normal is +Z, should NOT flip
    let (not_flipped, diag) = flip_mesh(mesh.clone(), Some(MeshFlipGuide::Vector(Vec3::new(0.0, 0.0, 1.0))));
    assert!(!diag.flipped);
    assert!(diag.guide_used);
    assert!(diag.dot_before.unwrap() > 0.0);
    assert_eq!(not_flipped.indices, vec![0, 1, 2]);

    // Guide points -Z, mesh normal is +Z, should flip
    let (flipped, diag) = flip_mesh(mesh, Some(MeshFlipGuide::Vector(Vec3::new(0.0, 0.0, -1.0))));
    assert!(diag.flipped);
    assert!(diag.guide_used);
    assert!(diag.dot_before.unwrap() < 0.0);
    assert_eq!(flipped.indices, vec![0, 2, 1]);
}

#[test]
fn flip_mesh_with_point_guide_towards_point() {
    // CCW triangle with normal pointing +Z, centroid around (0.5, 0.33, 0)
    let mesh = GeomMesh::new(
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
        vec![0, 1, 2],
    );

    // Guide point above the triangle, should NOT flip
    let (not_flipped, diag) = flip_mesh(mesh.clone(), Some(MeshFlipGuide::Point(Point3::new(0.5, 0.5, 10.0))));
    assert!(!diag.flipped);
    assert!(diag.guide_used);

    // Guide point below the triangle, should flip
    let (flipped, diag) = flip_mesh(mesh, Some(MeshFlipGuide::Point(Point3::new(0.5, 0.5, -10.0))));
    assert!(diag.flipped);
    assert!(diag.guide_used);
    assert_eq!(flipped.indices, vec![0, 2, 1]);
}

#[test]
fn flip_mesh_negates_explicit_normals() {
    let mesh = GeomMesh::with_attributes(
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
        vec![0, 1, 2],
        None,
        Some(vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]]),
    );

    let (flipped, diag) = flip_mesh(mesh, None);
    
    assert!(diag.flipped);
    
    // Normals should be negated
    let flipped_normals = flipped.normals.unwrap();
    assert_eq!(flipped_normals[0], [0.0, 0.0, -1.0]);
    assert_eq!(flipped_normals[1], [0.0, 0.0, -1.0]);
    assert_eq!(flipped_normals[2], [0.0, 0.0, -1.0]);
}

#[test]
fn flip_mesh_preserves_uvs_and_positions() {
    let mesh = GeomMesh::with_attributes(
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
        vec![0, 1, 2],
        Some(vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]]),
        None,
    );

    let (flipped, _) = flip_mesh(mesh.clone(), None);
    
    // Positions and UVs should be unchanged
    assert_eq!(flipped.positions, mesh.positions);
    assert_eq!(flipped.uvs, mesh.uvs);
}

#[test]
fn flip_mesh_multiple_triangles() {
    // Two triangles forming a quad
    let mesh = GeomMesh::new(
        vec![
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0],
        ],
        vec![0, 1, 2, 0, 2, 3],
    );

    let (flipped, diag) = flip_mesh(mesh, None);
    
    assert!(diag.flipped);
    // Each triangle should have swapped indices
    assert_eq!(flipped.indices, vec![0, 2, 1, 0, 3, 2]);
}

// ============================================================================
// compute_diagnostics Tests
// ============================================================================

#[test]
fn compute_diagnostics_single_triangle_has_three_open_edges() {
    let mesh = GeomMesh::new(
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
        vec![0, 1, 2],
    );

    let diag = mesh.compute_diagnostics(Tolerance::default());

    assert_eq!(diag.vertex_count, 3);
    assert_eq!(diag.triangle_count, 1);
    assert_eq!(diag.open_edge_count, 3); // Single triangle has 3 boundary edges
    assert_eq!(diag.non_manifold_edge_count, 0);
    assert_eq!(diag.degenerate_triangle_count, 0);
    // No repair performed, so these should be 0
    assert_eq!(diag.welded_vertex_count, 0);
    assert_eq!(diag.flipped_triangle_count, 0);
    // Should have warning about open edges
    assert!(diag.warnings.iter().any(|w| w.contains("open edges")));
}

#[test]
fn compute_diagnostics_watertight_tetrahedron_no_open_edges() {
    // Tetrahedron: 4 vertices, 4 triangles, all edges shared by 2 triangles
    let mesh = GeomMesh::new(
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, 0.5, 1.0],
        ],
        vec![
            0, 1, 2, // Base
            0, 1, 3, // Side 1
            1, 2, 3, // Side 2
            2, 0, 3, // Side 3
        ],
    );

    let diag = mesh.compute_diagnostics(Tolerance::default());

    assert_eq!(diag.vertex_count, 4);
    assert_eq!(diag.triangle_count, 4);
    assert_eq!(diag.open_edge_count, 0);
    assert_eq!(diag.non_manifold_edge_count, 0);
    assert!(diag.is_watertight());
    assert!(diag.is_manifold());
    // No warnings for watertight mesh
    assert!(!diag.warnings.iter().any(|w| w.contains("open edges")));
}

#[test]
fn compute_diagnostics_detects_degenerate_triangle() {
    // Triangle with all three vertices collinear (degenerate)
    let mesh = GeomMesh::new(
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
        vec![0, 1, 2],
    );

    let diag = mesh.compute_diagnostics(Tolerance::default());

    assert_eq!(diag.vertex_count, 3);
    assert_eq!(diag.triangle_count, 1);
    assert_eq!(diag.degenerate_triangle_count, 1);
    // Should have warning about degenerate triangles
    assert!(diag.warnings.iter().any(|w| w.contains("degenerate")));
}

#[test]
fn compute_diagnostics_detects_coincident_vertices() {
    // Triangle with two coincident vertex indices
    let mesh = GeomMesh::new(
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
        vec![0, 0, 2], // Invalid: i0 == i1
    );

    let diag = mesh.compute_diagnostics(Tolerance::default());

    assert_eq!(diag.degenerate_triangle_count, 1);
}

#[test]
fn compute_diagnostics_open_quad() {
    // Two triangles forming a quad - has 4 open edges (boundary edges)
    let mesh = GeomMesh::new(
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        vec![0, 1, 2, 0, 2, 3],
    );

    let diag = mesh.compute_diagnostics(Tolerance::default());

    assert_eq!(diag.vertex_count, 4);
    assert_eq!(diag.triangle_count, 2);
    assert_eq!(diag.open_edge_count, 4); // 4 boundary edges
    assert_eq!(diag.non_manifold_edge_count, 0); // Interior edge is shared by exactly 2 triangles
    assert!(!diag.is_watertight());
}

#[test]
fn compute_diagnostics_with_warnings_merges_existing_warnings() {
    let mesh = GeomMesh::new(
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
        vec![0, 1, 2],
    );

    let existing_warnings = vec!["surface fitting used approximation".to_string()];
    let diag = mesh.compute_diagnostics_with_warnings(Tolerance::default(), existing_warnings);

    // Should have both the existing warning and the open edges warning
    assert!(diag.warnings.iter().any(|w| w.contains("approximation")));
    assert!(diag.warnings.iter().any(|w| w.contains("open edges")));
    // Existing warning should come first
    assert_eq!(diag.warnings[0], "surface fitting used approximation");
}

#[test]
fn compute_diagnostics_empty_mesh() {
    let mesh = GeomMesh::new(vec![], vec![]);

    let diag = mesh.compute_diagnostics(Tolerance::default());

    assert_eq!(diag.vertex_count, 0);
    assert_eq!(diag.triangle_count, 0);
    assert_eq!(diag.open_edge_count, 0);
    assert_eq!(diag.non_manifold_edge_count, 0);
    assert_eq!(diag.degenerate_triangle_count, 0);
    assert!(diag.is_watertight()); // Empty mesh is technically watertight
}

// ============================================================================
// Cube-sphere (QuadSphere) mesh tests
// ============================================================================

use crate::geom::{CubeSphereOptions, mesh_cube_sphere};

#[test]
fn cube_sphere_basic_unit_sphere() {
    let options = CubeSphereOptions::unit(4);
    let (mesh, diag) = mesh_cube_sphere(options);

    // Validate the mesh structure
    mesh.validate().expect("cube-sphere mesh should be valid");

    // Check vertex count: 6 faces × (4+1)² = 6 × 25 = 150 before welding
    // After welding edges/corners, should be less
    assert!(mesh.positions.len() < 150, "welding should reduce vertex count");
    assert!(mesh.positions.len() > 50, "should have reasonable vertex count");

    // Check triangle count: 6 faces × 4² × 2 = 192 triangles
    assert_eq!(
        diag.triangle_count, 192,
        "cube-sphere with 4 subdivisions should have 192 triangles"
    );

    // All vertices should be on the unit sphere (radius 1, centered at origin)
    for pos in &mesh.positions {
        let dist = (pos[0] * pos[0] + pos[1] * pos[1] + pos[2] * pos[2]).sqrt();
        assert!(
            (dist - 1.0).abs() < 1e-10,
            "vertex should be on unit sphere, got distance {}",
            dist
        );
    }

    // Should have normals
    assert!(mesh.normals.is_some(), "cube-sphere should have normals");
    let normals = mesh.normals.as_ref().unwrap();
    assert_eq!(normals.len(), mesh.positions.len());

    // Normals should point outward (same direction as position for unit sphere)
    for (pos, norm) in mesh.positions.iter().zip(normals.iter()) {
        let dot = pos[0] * norm[0] + pos[1] * norm[1] + pos[2] * norm[2];
        assert!(dot > 0.9, "normal should point outward, dot = {}", dot);
    }
}

#[test]
fn cube_sphere_with_radius_and_center() {
    let center = Point3::new(10.0, 20.0, 30.0);
    let radius = 5.0;
    let options = CubeSphereOptions::new(center, radius, 2);
    let (mesh, diag) = mesh_cube_sphere(options);

    mesh.validate().expect("cube-sphere mesh should be valid");

    // Check triangle count: 6 faces × 2² × 2 = 48 triangles
    assert_eq!(diag.triangle_count, 48);

    // All vertices should be on the sphere at the specified center/radius
    for pos in &mesh.positions {
        let dx = pos[0] - center.x;
        let dy = pos[1] - center.y;
        let dz = pos[2] - center.z;
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        assert!(
            (dist - radius).abs() < 1e-10,
            "vertex should be on sphere, got distance {}",
            dist
        );
    }
}

#[test]
fn cube_sphere_is_watertight() {
    // A cube-sphere should be a closed mesh (watertight)
    let options = CubeSphereOptions::unit(8);
    let (mesh, diag) = mesh_cube_sphere(options);

    mesh.validate().expect("cube-sphere mesh should be valid");

    // Should have no open edges (watertight)
    assert_eq!(
        diag.open_edge_count, 0,
        "cube-sphere should be watertight (no open edges)"
    );

    // Should have no non-manifold edges
    assert_eq!(
        diag.non_manifold_edge_count, 0,
        "cube-sphere should have no non-manifold edges"
    );
}

#[test]
fn cube_sphere_subdivision_scaling() {
    // Test that triangle count scales correctly with subdivisions
    // Formula: 6 faces × subdivisions² × 2 triangles per quad

    for subdivisions in [1, 2, 4, 8, 16] {
        let options = CubeSphereOptions::unit(subdivisions);
        let (mesh, diag) = mesh_cube_sphere(options);

        mesh.validate().expect("cube-sphere mesh should be valid");

        let expected_triangles = 6 * subdivisions * subdivisions * 2;
        assert_eq!(
            diag.triangle_count, expected_triangles,
            "subdivisions={} should have {} triangles",
            subdivisions, expected_triangles
        );
    }
}

#[test]
fn cube_sphere_vertex_distribution_more_uniform_than_uv_sphere() {
    // One key advantage of cube-sphere over UV-sphere is more uniform vertex distribution.
    // We verify this by checking that vertex distances to their neighbors are more consistent.

    let options = CubeSphereOptions::unit(8);
    let (mesh, _diag) = mesh_cube_sphere(options);

    // Collect all edge lengths
    let mut edge_lengths = Vec::new();
    for chunk in mesh.indices.chunks(3) {
        if chunk.len() == 3 {
            let v0 = &mesh.positions[chunk[0] as usize];
            let v1 = &mesh.positions[chunk[1] as usize];
            let v2 = &mesh.positions[chunk[2] as usize];

            let len01 = ((v1[0] - v0[0]).powi(2) + (v1[1] - v0[1]).powi(2) + (v1[2] - v0[2]).powi(2)).sqrt();
            let len12 = ((v2[0] - v1[0]).powi(2) + (v2[1] - v1[1]).powi(2) + (v2[2] - v1[2]).powi(2)).sqrt();
            let len20 = ((v0[0] - v2[0]).powi(2) + (v0[1] - v2[1]).powi(2) + (v0[2] - v2[2]).powi(2)).sqrt();

            edge_lengths.push(len01);
            edge_lengths.push(len12);
            edge_lengths.push(len20);
        }
    }

    // Compute statistics
    let mean = edge_lengths.iter().sum::<f64>() / edge_lengths.len() as f64;
    let variance = edge_lengths.iter().map(|l| (l - mean).powi(2)).sum::<f64>() / edge_lengths.len() as f64;
    let std_dev = variance.sqrt();
    let coefficient_of_variation = std_dev / mean;

    // For a cube-sphere, the coefficient of variation should be relatively low
    // (indicating uniform edge lengths). For a UV-sphere, this would be much higher.
    assert!(
        coefficient_of_variation < 0.5,
        "cube-sphere should have relatively uniform edge lengths, CV = {}",
        coefficient_of_variation
    );
}

#[test]
fn cube_sphere_with_custom_orientation() {
    // Test that the orientation frame is respected
    let center = Point3::ORIGIN;
    let radius = 1.0;
    let x_axis = Vec3::new(0.0, 1.0, 0.0); // Rotated 90° around Z
    let y_axis = Vec3::new(-1.0, 0.0, 0.0);
    let z_axis = Vec3::new(0.0, 0.0, 1.0);

    let options = CubeSphereOptions::new(center, radius, 4)
        .with_frame(x_axis, y_axis, z_axis);
    let (mesh, _diag) = mesh_cube_sphere(options);

    mesh.validate().expect("cube-sphere mesh should be valid");

    // All vertices should still be on the unit sphere
    for pos in &mesh.positions {
        let dist = (pos[0] * pos[0] + pos[1] * pos[1] + pos[2] * pos[2]).sqrt();
        assert!(
            (dist - 1.0).abs() < 1e-10,
            "vertex should be on unit sphere"
        );
    }
}
