use crate::geom::{Point3, PatchOptions, Tolerance, Vec3, fragment_patch_meshes_with_tolerance, patch_mesh_with_options, patch_mesh_with_tolerance};

fn mesh_area(mesh: &crate::geom::GeomMesh) -> f64 {
    let mut area = 0.0;
    for tri in mesh.indices.chunks_exact(3) {
        let a = mesh.positions[tri[0] as usize];
        let b = mesh.positions[tri[1] as usize];
        let c = mesh.positions[tri[2] as usize];
        let ab = Vec3::new(b[0] - a[0], b[1] - a[1], b[2] - a[2]);
        let ac = Vec3::new(c[0] - a[0], c[1] - a[1], c[2] - a[2]);
        area += 0.5 * ab.cross(ac).length();
    }
    area
}

#[test]
fn patch_square_produces_triangles() {
    let tol = Tolerance::default_geom();

    let outer = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, 0.0),
    ];

    let (mesh, diag) = patch_mesh_with_tolerance(&outer, &[], tol).expect("patch mesh failed");
    assert!(mesh.indices.len() >= 6);
    assert!(mesh.positions.len() >= 4);
    assert!(diag.triangle_count >= 2);
}

#[test]
fn patch_rejects_non_planar_boundary() {
    let tol = Tolerance::default_geom();

    let outer = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 1e-3),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, 0.0),
    ];

    let err = patch_mesh_with_tolerance(&outer, &[], tol).unwrap_err();
    let message = err.to_string();
    assert!(message.contains("not planar"));
}

#[test]
fn fragment_patch_groups_hole_into_single_region() {
    let tol = Tolerance::default_geom();

    let outer = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(2.0, 0.0, 0.0),
        Point3::new(2.0, 2.0, 0.0),
        Point3::new(0.0, 2.0, 0.0),
        Point3::new(0.0, 0.0, 0.0),
    ];
    let hole = vec![
        Point3::new(0.5, 0.5, 0.0),
        Point3::new(1.5, 0.5, 0.0),
        Point3::new(1.5, 1.5, 0.0),
        Point3::new(0.5, 1.5, 0.0),
        Point3::new(0.5, 0.5, 0.0),
    ];

    let patches = fragment_patch_meshes_with_tolerance(&[outer, hole], tol).expect("fragment patch failed");
    assert_eq!(patches.len(), 1);

    let area = mesh_area(&patches[0].0);
    assert!((area - 3.0).abs() < 1e-6, "area was {area}");
}

#[test]
fn fragment_patch_emits_multiple_disjoint_regions() {
    let tol = Tolerance::default_geom();

    let a = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, 0.0),
    ];
    let b = vec![
        Point3::new(2.0, 0.0, 0.0),
        Point3::new(3.0, 0.0, 0.0),
        Point3::new(3.0, 1.0, 0.0),
        Point3::new(2.0, 1.0, 0.0),
        Point3::new(2.0, 0.0, 0.0),
    ];

    let patches = fragment_patch_meshes_with_tolerance(&[a, b], tol).expect("fragment patch failed");
    assert_eq!(patches.len(), 2);
}

#[test]
fn fragment_patch_treats_island_inside_hole_as_separate_region() {
    let tol = Tolerance::default_geom();

    let outer = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(4.0, 0.0, 0.0),
        Point3::new(4.0, 4.0, 0.0),
        Point3::new(0.0, 4.0, 0.0),
        Point3::new(0.0, 0.0, 0.0),
    ];
    let hole = vec![
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(3.0, 1.0, 0.0),
        Point3::new(3.0, 3.0, 0.0),
        Point3::new(1.0, 3.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
    ];
    let island = vec![
        Point3::new(1.5, 1.5, 0.0),
        Point3::new(2.5, 1.5, 0.0),
        Point3::new(2.5, 2.5, 0.0),
        Point3::new(1.5, 2.5, 0.0),
        Point3::new(1.5, 1.5, 0.0),
    ];

    let patches = fragment_patch_meshes_with_tolerance(&[outer, hole, island], tol).expect("fragment patch failed");
    assert_eq!(patches.len(), 2);
}

// =============================================================================
// Tests for PatchOptions (spans, flexibility, trim, interior_points)
// =============================================================================

#[test]
fn patch_with_options_default_produces_valid_mesh() {
    let tol = Tolerance::default_geom();

    let outer = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, 0.0),
    ];

    let options = PatchOptions::default();
    let (mesh, diag) = patch_mesh_with_options(&outer, &[], options, tol).expect("patch mesh failed");
    
    assert!(mesh.indices.len() >= 3, "mesh should have at least one triangle");
    assert!(mesh.positions.len() >= 3, "mesh should have at least 3 positions");
    assert!(diag.triangle_count >= 1, "diagnostics should report at least one triangle");
}

#[test]
fn patch_with_interior_points_includes_them() {
    let tol = Tolerance::default_geom();

    let outer = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(10.0, 0.0, 0.0),
        Point3::new(10.0, 10.0, 0.0),
        Point3::new(0.0, 10.0, 0.0),
        Point3::new(0.0, 0.0, 0.0),
    ];

    // Interior point at the center of the square
    let interior_points = vec![Point3::new(5.0, 5.0, 0.0)];

    let options = PatchOptions::default()
        .with_interior_points(interior_points);
    
    let (mesh, diag) = patch_mesh_with_options(&outer, &[], options, tol).expect("patch mesh with interior failed");

    // With an interior point, we expect more triangles than the simple case
    // A square without interior points produces 2 triangles (earclip)
    // With one interior point, we expect 4 triangles (Delaunay with center point)
    assert!(diag.triangle_count >= 2, "should have at least 2 triangles, got {}", diag.triangle_count);
    
    // Check that the center point is in the mesh
    let has_center = mesh.positions.iter().any(|p| {
        (p[0] - 5.0).abs() < tol.eps * 100.0 && (p[1] - 5.0).abs() < tol.eps * 100.0
    });
    assert!(has_center, "interior point should be in the mesh positions");
}

#[test]
fn patch_with_high_spans_produces_more_boundary_points() {
    let tol = Tolerance::default_geom();

    let outer = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(10.0, 0.0, 0.0),
        Point3::new(10.0, 10.0, 0.0),
        Point3::new(0.0, 10.0, 0.0),
        Point3::new(0.0, 0.0, 0.0),
    ];

    // Low spans (default is 10)
    let options_low = PatchOptions::default().with_spans(1);
    let (mesh_low, _) = patch_mesh_with_options(&outer, &[], options_low, tol).expect("low spans failed");

    // High spans
    let options_high = PatchOptions::default().with_spans(20);
    let (mesh_high, _) = patch_mesh_with_options(&outer, &[], options_high, tol).expect("high spans failed");

    // Higher spans should produce more vertices (more boundary subdivision)
    assert!(
        mesh_high.positions.len() >= mesh_low.positions.len(),
        "high spans ({}) should produce >= vertices than low spans ({})",
        mesh_high.positions.len(),
        mesh_low.positions.len()
    );
}

#[test]
fn patch_with_high_flexibility_generates_internal_points() {
    let tol = Tolerance::default_geom();

    let outer = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(10.0, 0.0, 0.0),
        Point3::new(10.0, 10.0, 0.0),
        Point3::new(0.0, 10.0, 0.0),
        Point3::new(0.0, 0.0, 0.0),
    ];

    // Flexibility = 0 (no internal points)
    let options_stiff = PatchOptions::default()
        .with_spans(1)
        .with_flexibility(0.0);
    let (mesh_stiff, _) = patch_mesh_with_options(&outer, &[], options_stiff, tol).expect("stiff failed");

    // Flexibility = 2.0 (should generate internal points)
    let options_flex = PatchOptions::default()
        .with_spans(1)
        .with_flexibility(2.0);
    let (mesh_flex, _) = patch_mesh_with_options(&outer, &[], options_flex, tol).expect("flexible failed");

    // Higher flexibility should produce more vertices (internal subdivision)
    assert!(
        mesh_flex.positions.len() >= mesh_stiff.positions.len(),
        "flexible ({}) should produce >= vertices than stiff ({})",
        mesh_flex.positions.len(),
        mesh_stiff.positions.len()
    );
}

#[test]
fn patch_options_builder_pattern_works() {
    let options = PatchOptions::new()
        .with_spans(15)
        .with_flexibility(0.5)
        .with_trim(false)
        .with_interior_points(vec![Point3::new(1.0, 2.0, 3.0)]);

    assert_eq!(options.spans, 15);
    assert!((options.flexibility - 0.5).abs() < 1e-9);
    assert!(!options.trim);
    assert_eq!(options.interior_points.len(), 1);
    assert!((options.interior_points[0].x - 1.0).abs() < 1e-9);
}

#[test]
fn patch_options_clamps_extreme_values() {
    let options = PatchOptions::new()
        .with_spans(1000) // Should be clamped to 100
        .with_flexibility(-5.0); // Should be clamped to 0.0

    assert_eq!(options.spans, 100);
    assert!((options.flexibility - 0.0).abs() < 1e-9);
}
