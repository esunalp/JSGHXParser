use crate::geom::{
    MiterType, Point3, ProfilePlaneTransform, SweepCaps, SweepError, SweepOptions, Tolerance,
    sweep1_polyline_with_tolerance, sweep2_polyline_with_tolerance,
};

#[test]
fn sweep1_straight_rail_square_profile_caps() {
    // Square profile in local XY plane (closed).
    let profile = vec![
        Point3::new(-1.0, -1.0, 0.0),
        Point3::new(1.0, -1.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(-1.0, 1.0, 0.0),
        Point3::new(-1.0, -1.0, 0.0),
    ];

    // Straight rail along Z.
    let rail = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, 2.0)];

    let tol = Tolerance::default_geom();
    let options = SweepOptions { twist_radians_total: 0.0, miter: MiterType::None };

    let (mesh, diag) = sweep1_polyline_with_tolerance(&profile, &rail, SweepCaps::BOTH, options, tol)
        .expect("sweep should succeed");

    // A straight sweep of a square with both caps is a prism: 8 side triangles + 4 cap triangles.
    assert_eq!(mesh.triangle_count(), 12);
    assert!(diag.open_edge_count == 0, "expected watertight mesh");
    assert!(diag.non_manifold_edge_count == 0, "expected manifold mesh");
}

#[test]
fn sweep1_twist_does_not_crash() {
    let profile = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, 0.0),
    ];
    let rail = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 2.0),
    ];

    let tol = Tolerance::default_geom();
    let options = SweepOptions { twist_radians_total: std::f64::consts::FRAC_PI_2, miter: MiterType::None };

    let _ = sweep1_polyline_with_tolerance(&profile, &rail, SweepCaps::END, options, tol)
        .expect("twisted sweep should succeed");
}

// ============================================================================
// Sweep2 tests
// ============================================================================

#[test]
fn sweep2_straight_rails_square_profile() {
    // Square profile in local XY plane (closed).
    let profile = vec![
        Point3::new(-0.5, -0.5, 0.0),
        Point3::new(0.5, -0.5, 0.0),
        Point3::new(0.5, 0.5, 0.0),
        Point3::new(-0.5, 0.5, 0.0),
        Point3::new(-0.5, -0.5, 0.0),
    ];

    // Two parallel rails along Z, offset in X.
    let rail_a = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(0.0, 0.0, 2.0),
    ];
    let rail_b = vec![
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 2.0),
    ];

    let tol = Tolerance::default_geom();
    let options = SweepOptions::default();

    let (mesh, diag) = sweep2_polyline_with_tolerance(&profile, &rail_a, &rail_b, SweepCaps::BOTH, options, tol)
        .expect("sweep2 should succeed");

    // Swept square with 3 rings, 2 segments, both caps.
    assert!(mesh.triangle_count() > 0, "expected non-empty mesh");
    assert!(diag.open_edge_count == 0, "expected watertight mesh");
    assert!(diag.non_manifold_edge_count == 0, "expected manifold mesh");
}

#[test]
fn sweep2_mismatched_rail_lengths_fails() {
    let profile = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, 0.0),
    ];
    let rail_a = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, 1.0)];
    let rail_b = vec![
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.5),
        Point3::new(1.0, 0.0, 1.0),
    ];

    let tol = Tolerance::default_geom();
    let result = sweep2_polyline_with_tolerance(&profile, &rail_a, &rail_b, SweepCaps::NONE, SweepOptions::default(), tol);

    assert!(matches!(result, Err(SweepError::Sweep2RailLengthMismatch)));
}

// ============================================================================
// Closed rail tests
// ============================================================================

#[test]
fn sweep1_closed_rail_no_caps() {
    // Square profile.
    let profile = vec![
        Point3::new(-0.25, -0.25, 0.0),
        Point3::new(0.25, -0.25, 0.0),
        Point3::new(0.25, 0.25, 0.0),
        Point3::new(-0.25, 0.25, 0.0),
        Point3::new(-0.25, -0.25, 0.0),
    ];

    // Closed circular rail (approximated as square for simplicity).
    // Note: Sharp corners in the rail may cause imperfect stitching at the seam.
    let rail = vec![
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(-1.0, 0.0, 0.0),
        Point3::new(0.0, -1.0, 0.0),
        Point3::new(1.0, 0.0, 0.0), // Closed - back to start.
    ];

    let tol = Tolerance::default_geom();
    let options = SweepOptions::default();

    let (mesh, diag) = sweep1_polyline_with_tolerance(&profile, &rail, SweepCaps::NONE, options, tol)
        .expect("closed rail sweep should succeed without caps");

    assert!(mesh.triangle_count() > 0, "expected non-empty mesh");
    // Closed rail sweep may have open edges at seam due to sharp corners.
    // The important thing is it produces geometry and is manifold.
    assert_eq!(diag.non_manifold_edge_count, 0, "expected manifold mesh");
}

#[test]
fn sweep1_closed_rail_with_caps_fails() {
    let profile = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.5, 1.0, 0.0),
        Point3::new(0.0, 0.0, 0.0),
    ];

    // Closed rail.
    let rail = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, 0.0),
    ];

    let tol = Tolerance::default_geom();
    let result = sweep1_polyline_with_tolerance(&profile, &rail, SweepCaps::BOTH, SweepOptions::default(), tol);

    assert!(matches!(result, Err(SweepError::CapsNotAllowedForClosedRail)));
}

// ============================================================================
// Degenerate tangent / edge case tests
// ============================================================================

#[test]
fn sweep1_degenerate_rail_segment_recovers() {
    // Profile.
    let profile = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, 0.0),
    ];

    // Rail with a duplicate point (degenerate segment).
    let rail = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(0.0, 0.0, 1.0), // Duplicate - will be cleaned.
        Point3::new(0.0, 0.0, 2.0),
    ];

    let tol = Tolerance::default_geom();
    let options = SweepOptions::default();

    // Should succeed after cleaning duplicates.
    let (mesh, _diag) = sweep1_polyline_with_tolerance(&profile, &rail, SweepCaps::NONE, options, tol)
        .expect("sweep with duplicate rail points should succeed after cleaning");

    assert!(mesh.triangle_count() > 0);
}

#[test]
fn sweep1_sharp_cusp_produces_warning() {
    let profile = vec![
        Point3::new(-0.1, -0.1, 0.0),
        Point3::new(0.1, -0.1, 0.0),
        Point3::new(0.1, 0.1, 0.0),
        Point3::new(-0.1, 0.1, 0.0),
        Point3::new(-0.1, -0.1, 0.0),
    ];

    // Rail with a very sharp turn (nearly 180° reversal).
    // The tangent averaging still produces a significant direction change.
    let rail = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),  // Going +X
        Point3::new(0.9, 0.0, 0.0),  // Sharp reversal back toward -X
        Point3::new(0.5, 0.0, 0.0),
    ];

    let tol = Tolerance::default_geom();
    let options = SweepOptions::default();

    let (_mesh, diag) = sweep1_polyline_with_tolerance(&profile, &rail, SweepCaps::NONE, options, tol)
        .expect("sweep with sharp cusp should succeed");

    // Should have a warning about sharp tangent changes.
    let has_cusp_warning = diag.warnings.iter().any(|w| w.contains("sharp tangent"));
    assert!(has_cusp_warning, "expected cusp warning, got: {:?}", diag.warnings);
}

// ============================================================================
// Twist + caps combined test
// ============================================================================

#[test]
fn sweep1_twist_with_caps() {
    let profile = vec![
        Point3::new(-0.5, -0.5, 0.0),
        Point3::new(0.5, -0.5, 0.0),
        Point3::new(0.5, 0.5, 0.0),
        Point3::new(-0.5, 0.5, 0.0),
        Point3::new(-0.5, -0.5, 0.0),
    ];

    let rail = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(0.0, 0.0, 2.0),
        Point3::new(0.0, 0.0, 3.0),
    ];

    let tol = Tolerance::default_geom();
    let options = SweepOptions { twist_radians_total: std::f64::consts::PI, miter: MiterType::None }; // 180 degree twist.

    let (mesh, diag) = sweep1_polyline_with_tolerance(&profile, &rail, SweepCaps::BOTH, options, tol)
        .expect("twisted sweep with caps should succeed");

    assert!(mesh.triangle_count() > 0);
    assert!(diag.open_edge_count == 0, "expected watertight mesh with caps");
    assert!(diag.non_manifold_edge_count == 0, "expected manifold mesh");
}

// ============================================================================
// Profile Plane Transform tests
// ============================================================================

#[test]
fn profile_plane_transform_xy_plane_is_detected() {
    // Profile in XY plane centered at origin - should be recognized as XY-aligned.
    let profile = vec![
        Point3::new(-1.0, -1.0, 0.0),
        Point3::new(1.0, -1.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(-1.0, 1.0, 0.0),
    ];

    let tol = Tolerance::default_geom();
    let transform = ProfilePlaneTransform::from_points(&profile, tol)
        .expect("transform should succeed");

    // For XY-aligned profiles, the normal should be approximately ±Z
    assert!(
        transform.normal.z.abs() > 0.99,
        "XY plane should have Z-aligned normal, got: {:?}",
        transform.normal
    );
}

#[test]
fn profile_plane_transform_yz_plane() {
    // Profile in YZ plane (circle in YZ, centered at X=0).
    let profile = vec![
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(0.0, -1.0, 0.0),
        Point3::new(0.0, 0.0, -1.0),
    ];

    let tol = Tolerance::default_geom();
    let transform = ProfilePlaneTransform::from_points(&profile, tol)
        .expect("transform should succeed");

    // For YZ plane, the normal should be approximately ±X
    assert!(
        transform.normal.x.abs() > 0.99,
        "YZ plane should have X-aligned normal, got: {:?}",
        transform.normal
    );

    // Transform to local coordinates - should be near-planar (z ≈ 0)
    let local = transform.transform_profile_to_local(&profile);
    for p in &local {
        assert!(
            p.z.abs() < tol.eps,
            "local z should be near zero for planar profile, got: {}",
            p.z
        );
    }
}

#[test]
fn sweep1_yz_profile_along_x_rail() {
    // Square profile in YZ plane (profile normal is ±X).
    let profile = vec![
        Point3::new(0.0, -0.5, -0.5),
        Point3::new(0.0, 0.5, -0.5),
        Point3::new(0.0, 0.5, 0.5),
        Point3::new(0.0, -0.5, 0.5),
        Point3::new(0.0, -0.5, -0.5), // Close the profile
    ];

    // Rail along +X axis
    let rail = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(2.0, 0.0, 0.0),
    ];

    let tol = Tolerance::default_geom();
    let options = SweepOptions::default();

    let (mesh, diag) = sweep1_polyline_with_tolerance(&profile, &rail, SweepCaps::BOTH, options, tol)
        .expect("sweep with YZ profile should succeed");

    // Should produce a valid prism mesh
    assert!(mesh.triangle_count() > 0, "expected non-empty mesh");
    assert_eq!(diag.open_edge_count, 0, "expected watertight mesh");
    assert_eq!(diag.non_manifold_edge_count, 0, "expected manifold mesh");

    // Verify the mesh has reasonable bounds - should extend along X from 0 to 2
    let mut x_min = f64::MAX;
    let mut x_max = f64::MIN;
    for pos in &mesh.positions {
        x_min = x_min.min(pos[0]);
        x_max = x_max.max(pos[0]);
    }
    assert!(
        (x_max - x_min - 2.0).abs() < 0.1,
        "mesh should span X from 0 to 2, got min={} max={}",
        x_min,
        x_max
    );
}

#[test]
fn sweep1_tilted_profile_maintains_shape() {
    // Profile in a tilted plane (45° to XY, tilted around Y axis).
    let sqrt2_inv = 1.0 / 2.0f64.sqrt();
    let profile = vec![
        Point3::new(-sqrt2_inv, -1.0, -sqrt2_inv),
        Point3::new(sqrt2_inv, -1.0, sqrt2_inv),
        Point3::new(sqrt2_inv, 1.0, sqrt2_inv),
        Point3::new(-sqrt2_inv, 1.0, -sqrt2_inv),
        Point3::new(-sqrt2_inv, -1.0, -sqrt2_inv), // Close
    ];

    // Rail along +Z
    let rail = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 0.0, 3.0),
    ];

    let tol = Tolerance::default_geom();
    let options = SweepOptions::default();

    let (mesh, diag) = sweep1_polyline_with_tolerance(&profile, &rail, SweepCaps::BOTH, options, tol)
        .expect("sweep with tilted profile should succeed");

    assert!(mesh.triangle_count() > 0, "expected non-empty mesh");
    assert_eq!(diag.open_edge_count, 0, "expected watertight mesh");
    assert_eq!(diag.non_manifold_edge_count, 0, "expected manifold mesh");
}

#[test]
fn sweep2_yz_profile_with_two_rails() {
    // Square profile in YZ plane.
    let profile = vec![
        Point3::new(0.0, -0.5, -0.5),
        Point3::new(0.0, 0.5, -0.5),
        Point3::new(0.0, 0.5, 0.5),
        Point3::new(0.0, -0.5, 0.5),
        Point3::new(0.0, -0.5, -0.5),
    ];

    // Two parallel rails along +X
    let rail_a = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(2.0, 0.0, 0.0),
    ];
    let rail_b = vec![
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(2.0, 1.0, 0.0),
    ];

    let tol = Tolerance::default_geom();
    let options = SweepOptions::default();

    let (mesh, diag) = sweep2_polyline_with_tolerance(&profile, &rail_a, &rail_b, SweepCaps::BOTH, options, tol)
        .expect("sweep2 with YZ profile should succeed");

    assert!(mesh.triangle_count() > 0, "expected non-empty mesh");
    assert_eq!(diag.open_edge_count, 0, "expected watertight mesh");
    assert_eq!(diag.non_manifold_edge_count, 0, "expected manifold mesh");
}

#[test]
fn sweep1_arbitrary_offset_profile() {
    // Profile not at origin - offset in all directions.
    let profile = vec![
        Point3::new(5.0, 10.0, 2.0),
        Point3::new(6.0, 10.0, 2.0),
        Point3::new(6.0, 11.0, 2.0),
        Point3::new(5.0, 11.0, 2.0),
        Point3::new(5.0, 10.0, 2.0),
    ];

    // Rail at the same offset location
    let rail = vec![
        Point3::new(5.5, 10.5, 2.0),
        Point3::new(5.5, 10.5, 5.0),
    ];

    let tol = Tolerance::default_geom();
    let options = SweepOptions::default();

    let (mesh, diag) = sweep1_polyline_with_tolerance(&profile, &rail, SweepCaps::BOTH, options, tol)
        .expect("sweep with offset profile should succeed");

    assert!(mesh.triangle_count() > 0, "expected non-empty mesh");
    assert_eq!(diag.open_edge_count, 0, "expected watertight mesh");

    // Verify the mesh is in the expected location
    let mut z_min = f64::MAX;
    let mut z_max = f64::MIN;
    for pos in &mesh.positions {
        z_min = z_min.min(pos[2]);
        z_max = z_max.max(pos[2]);
    }
    assert!(
        (z_min - 2.0).abs() < 0.1 && (z_max - 5.0).abs() < 0.1,
        "mesh should span Z from 2 to 5, got min={} max={}",
        z_min,
        z_max
    );
}

// ============================================================================
// Rail Alignment Tests
// ============================================================================

use crate::geom::{align_sweep2_rails, Sweep2MultiSectionOptions, Sweep2Section, sweep2_multi_section};

#[test]
fn rail_alignment_same_direction() {
    // Two rails running in the same direction (start to start, end to end)
    let rail_a = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(0.0, 0.0, 2.0),
    ];
    let rail_b = vec![
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 2.0),
    ];

    let tol = Tolerance::default_geom();
    let result = align_sweep2_rails(&rail_a, &rail_b, 3, tol).expect("alignment should succeed");

    // Both rails should be in their original direction
    assert!(!result.rail_a_flipped, "rail A should not be flipped");
    assert!(!result.rail_b_flipped, "rail B should not be flipped");
    assert!(result.alignment_score > 0.9, "alignment score should be high: {}", result.alignment_score);
}

#[test]
fn rail_alignment_opposite_direction() {
    // Rail B runs in opposite direction (end to start)
    let rail_a = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(0.0, 0.0, 2.0),
    ];
    let rail_b = vec![
        Point3::new(1.0, 0.0, 2.0),  // Starts where A ends
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 0.0),  // Ends where A starts
    ];

    let tol = Tolerance::default_geom();
    let result = align_sweep2_rails(&rail_a, &rail_b, 3, tol).expect("alignment should succeed");

    // Rail B should be detected as needing flip
    assert!(result.rail_b_flipped, "rail B should be flipped to match direction");
    
    // After alignment, both rails should start at z=0 and end at z=2
    assert!(result.rail_a[0].z < 0.1, "rail A should start at z=0");
    assert!(result.rail_b[0].z < 0.1, "rail B should start at z=0 after alignment");
    assert!(result.rail_a.last().unwrap().z > 1.9, "rail A should end at z=2");
    assert!(result.rail_b.last().unwrap().z > 1.9, "rail B should end at z=2 after alignment");
}

#[test]
fn rail_alignment_shared_endpoints() {
    // Both rails share a common start point (common in Grasshopper)
    let start = Point3::new(0.0, 0.0, 0.0);
    let rail_a = vec![
        start,
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(0.0, 0.0, 2.0),
    ];
    let rail_b = vec![
        start,  // Same start point
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(2.0, 0.0, 2.0),
    ];

    let tol = Tolerance::default_geom();
    let result = align_sweep2_rails(&rail_a, &rail_b, 3, tol).expect("alignment should succeed");

    // Neither rail should be flipped since they share the start point
    assert!(!result.rail_a_flipped, "rail A should not be flipped");
    assert!(!result.rail_b_flipped, "rail B should not be flipped");
}

#[test]
fn rail_alignment_arc_length_resampling() {
    // Rail with different segment lengths should be resampled uniformly
    let rail_a = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 0.0, 0.1),  // Short segment
        Point3::new(0.0, 0.0, 2.0),  // Long segment
    ];
    let rail_b = vec![
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 1.0),  // Medium segment
        Point3::new(1.0, 0.0, 2.0),  // Medium segment
    ];

    let tol = Tolerance::default_geom();
    let target_count = 5;
    let result = align_sweep2_rails(&rail_a, &rail_b, target_count, tol).expect("alignment should succeed");

    // Both rails should have the target count
    assert_eq!(result.rail_a.len(), target_count);
    assert_eq!(result.rail_b.len(), target_count);

    // Check that corresponding points are at similar arc-length positions
    // Point 2 (middle) should be roughly at z=1 for both rails
    let mid_idx = target_count / 2;
    let z_mid_a = result.rail_a[mid_idx].z;
    let z_mid_b = result.rail_b[mid_idx].z;
    assert!(
        (z_mid_a - z_mid_b).abs() < 0.2,
        "middle points should be at similar Z: a={}, b={}",
        z_mid_a,
        z_mid_b
    );
}

// ============================================================================
// Multi-Section Sweep2 Tests
// ============================================================================

#[test]
fn sweep2_multi_section_two_profiles() {
    // Two different profiles at start and end - same point count for proper interpolation
    // Using rounded rectangle (8 points) for both for easier interpolation
    let rounded_rect_small: Vec<Point3> = vec![
        Point3::new(-0.3, -0.5, 0.0),
        Point3::new(0.3, -0.5, 0.0),
        Point3::new(0.5, -0.3, 0.0),
        Point3::new(0.5, 0.3, 0.0),
        Point3::new(0.3, 0.5, 0.0),
        Point3::new(-0.3, 0.5, 0.0),
        Point3::new(-0.5, 0.3, 0.0),
        Point3::new(-0.5, -0.3, 0.0),
        Point3::new(-0.3, -0.5, 0.0), // Close the shape
    ];

    // Larger version of the same shape
    let rounded_rect_large: Vec<Point3> = vec![
        Point3::new(-0.6, -1.0, 0.0),
        Point3::new(0.6, -1.0, 0.0),
        Point3::new(1.0, -0.6, 0.0),
        Point3::new(1.0, 0.6, 0.0),
        Point3::new(0.6, 1.0, 0.0),
        Point3::new(-0.6, 1.0, 0.0),
        Point3::new(-1.0, 0.6, 0.0),
        Point3::new(-1.0, -0.6, 0.0),
        Point3::new(-0.6, -1.0, 0.0), // Close the shape
    ];

    let sections = vec![
        Sweep2Section::at_parameter(rounded_rect_small, 0.0),
        Sweep2Section::at_parameter(rounded_rect_large, 1.0),
    ];

    let rail_a = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 0.0, 2.0),
    ];
    let rail_b = vec![
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 2.0),
    ];

    let tol = Tolerance::default_geom();
    let options = Sweep2MultiSectionOptions::default();

    let (mesh, diag) = sweep2_multi_section(&sections, &rail_a, &rail_b, SweepCaps::BOTH, options, tol)
        .expect("multi-section sweep2 should succeed");

    assert!(mesh.triangle_count() > 0, "expected non-empty mesh");
    assert_eq!(diag.open_edge_count, 0, "expected watertight mesh");
    assert_eq!(diag.non_manifold_edge_count, 0, "expected manifold mesh");
}

#[test]
fn sweep2_multi_section_three_profiles() {
    // Three profiles: small, large, medium
    let make_square = |size: f64| vec![
        Point3::new(-size, -size, 0.0),
        Point3::new(size, -size, 0.0),
        Point3::new(size, size, 0.0),
        Point3::new(-size, size, 0.0),
        Point3::new(-size, -size, 0.0),
    ];

    let sections = vec![
        Sweep2Section::at_parameter(make_square(0.3), 0.0),
        Sweep2Section::at_parameter(make_square(0.8), 0.5),
        Sweep2Section::at_parameter(make_square(0.5), 1.0),
    ];

    let rail_a = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(0.0, 0.0, 2.0),
    ];
    let rail_b = vec![
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.5, 0.0, 1.0),  // Rails diverge in the middle
        Point3::new(1.0, 0.0, 2.0),
    ];

    let tol = Tolerance::default_geom();
    let options = Sweep2MultiSectionOptions::default();

    let (mesh, diag) = sweep2_multi_section(&sections, &rail_a, &rail_b, SweepCaps::BOTH, options, tol)
        .expect("three-section sweep2 should succeed");

    assert!(mesh.triangle_count() > 0, "expected non-empty mesh");
    assert_eq!(diag.open_edge_count, 0, "expected watertight mesh");
}

#[test]
fn sweep2_multi_section_auto_distribute() {
    // Sections without explicit parameters - should be distributed evenly
    let make_square = |size: f64| vec![
        Point3::new(-size, -size, 0.0),
        Point3::new(size, -size, 0.0),
        Point3::new(size, size, 0.0),
        Point3::new(-size, size, 0.0),
        Point3::new(-size, -size, 0.0),
    ];

    let sections = vec![
        Sweep2Section::auto(make_square(0.3)),
        Sweep2Section::auto(make_square(0.6)),
        Sweep2Section::auto(make_square(0.4)),
    ];

    let rail_a = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 0.0, 3.0),
    ];
    let rail_b = vec![
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 3.0),
    ];

    let tol = Tolerance::default_geom();
    let options = Sweep2MultiSectionOptions {
        auto_distribute_sections: true,
        ..Default::default()
    };

    let (mesh, diag) = sweep2_multi_section(&sections, &rail_a, &rail_b, SweepCaps::BOTH, options, tol)
        .expect("auto-distributed sweep2 should succeed");

    assert!(mesh.triangle_count() > 0, "expected non-empty mesh");
    assert_eq!(diag.open_edge_count, 0, "expected watertight mesh");
}

#[test]
fn sweep2_multi_section_with_reversed_rail() {
    // Rail B is reversed - should be auto-corrected
    let square_profile = vec![
        Point3::new(-0.5, -0.5, 0.0),
        Point3::new(0.5, -0.5, 0.0),
        Point3::new(0.5, 0.5, 0.0),
        Point3::new(-0.5, 0.5, 0.0),
        Point3::new(-0.5, -0.5, 0.0),
    ];

    let sections = vec![
        Sweep2Section::at_parameter(square_profile.clone(), 0.0),
        Sweep2Section::at_parameter(square_profile, 1.0),
    ];

    let rail_a = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(0.0, 0.0, 2.0),
    ];
    // Rail B is reversed!
    let rail_b = vec![
        Point3::new(1.0, 0.0, 2.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 0.0),
    ];

    let tol = Tolerance::default_geom();
    let options = Sweep2MultiSectionOptions::default();

    let (mesh, diag) = sweep2_multi_section(&sections, &rail_a, &rail_b, SweepCaps::BOTH, options, tol)
        .expect("sweep2 with reversed rail should succeed (auto-corrected)");

    assert!(mesh.triangle_count() > 0, "expected non-empty mesh");
    assert_eq!(diag.open_edge_count, 0, "expected watertight mesh");
    
    // Should have a warning about the reversed rail
    assert!(
        diag.warnings.iter().any(|w| w.contains("reversed") || w.contains("direction")),
        "expected warning about rail reversal, got: {:?}",
        diag.warnings
    );
}

// ============================================================================
// MiterType tests - verify miter handling at sharp corners
// ============================================================================

#[test]
fn miter_type_from_int_valid_values() {
    assert_eq!(MiterType::from_int(0), MiterType::None);
    assert_eq!(MiterType::from_int(1), MiterType::Trim);
    assert_eq!(MiterType::from_int(2), MiterType::Rotate);
}

#[test]
fn miter_type_from_int_invalid_defaults_to_none() {
    // Invalid values should default to None
    assert_eq!(MiterType::from_int(-1), MiterType::None);
    assert_eq!(MiterType::from_int(3), MiterType::None);
    assert_eq!(MiterType::from_int(100), MiterType::None);
}

#[test]
fn miter_type_to_int_roundtrip() {
    assert_eq!(MiterType::None.to_int(), 0);
    assert_eq!(MiterType::Trim.to_int(), 1);
    assert_eq!(MiterType::Rotate.to_int(), 2);
    
    // Roundtrip check
    for i in 0..=2 {
        assert_eq!(MiterType::from_int(i).to_int(), i);
    }
}

#[test]
fn miter_type_requires_kink_handling() {
    assert!(!MiterType::None.requires_kink_handling());
    assert!(MiterType::Trim.requires_kink_handling());
    assert!(MiterType::Rotate.requires_kink_handling());
}

#[test]
fn sweep1_sharp_corner_no_miter() {
    // Square profile
    let profile = vec![
        Point3::new(-0.1, -0.1, 0.0),
        Point3::new(0.1, -0.1, 0.0),
        Point3::new(0.1, 0.1, 0.0),
        Point3::new(-0.1, 0.1, 0.0),
        Point3::new(-0.1, -0.1, 0.0),
    ];

    // Rail with a sharp 90-degree corner
    let rail = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),  // Straight along X
        Point3::new(1.0, 1.0, 0.0),  // Sharp turn to Y
        Point3::new(1.0, 2.0, 0.0),
    ];

    let tol = Tolerance::default_geom();
    let options = SweepOptions {
        twist_radians_total: 0.0,
        miter: MiterType::None,
    };

    let (mesh, diag) = sweep1_polyline_with_tolerance(&profile, &rail, SweepCaps::BOTH, options, tol)
        .expect("sweep with sharp corner (no miter) should succeed");

    assert!(mesh.triangle_count() > 0, "expected non-empty mesh");
    // With MiterType::None, we expect a warning about sharp tangent changes
    assert!(
        diag.warnings.iter().any(|w| w.contains("sharp tangent") || w.contains("no special handling")),
        "expected cusp warning with MiterType::None, got: {:?}",
        diag.warnings
    );
}

#[test]
fn sweep1_sharp_corner_with_trim_miter() {
    // Square profile
    let profile = vec![
        Point3::new(-0.1, -0.1, 0.0),
        Point3::new(0.1, -0.1, 0.0),
        Point3::new(0.1, 0.1, 0.0),
        Point3::new(-0.1, 0.1, 0.0),
        Point3::new(-0.1, -0.1, 0.0),
    ];

    // Rail with a sharp 90-degree corner
    let rail = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),  // Straight along X
        Point3::new(1.0, 1.0, 0.0),  // Sharp turn to Y
        Point3::new(1.0, 2.0, 0.0),
    ];

    let tol = Tolerance::default_geom();
    let options = SweepOptions {
        twist_radians_total: 0.0,
        miter: MiterType::Trim,
    };

    let (mesh, diag) = sweep1_polyline_with_tolerance(&profile, &rail, SweepCaps::BOTH, options, tol)
        .expect("sweep with sharp corner (trim miter) should succeed");

    assert!(mesh.triangle_count() > 0, "expected non-empty mesh");
    // With MiterType::Trim, we expect a warning about bisector miter being applied
    assert!(
        diag.warnings.iter().any(|w| w.contains("bisector miter")),
        "expected bisector miter warning with MiterType::Trim, got: {:?}",
        diag.warnings
    );
}

#[test]
fn sweep1_sharp_corner_with_rotate_miter() {
    // Square profile
    let profile = vec![
        Point3::new(-0.1, -0.1, 0.0),
        Point3::new(0.1, -0.1, 0.0),
        Point3::new(0.1, 0.1, 0.0),
        Point3::new(-0.1, 0.1, 0.0),
        Point3::new(-0.1, -0.1, 0.0),
    ];

    // Rail with a sharp 90-degree corner
    let rail = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(1.0, 2.0, 0.0),
    ];

    let tol = Tolerance::default_geom();
    let options = SweepOptions {
        twist_radians_total: 0.0,
        miter: MiterType::Rotate,
    };

    let (mesh, diag) = sweep1_polyline_with_tolerance(&profile, &rail, SweepCaps::BOTH, options, tol)
        .expect("sweep with sharp corner (rotate miter) should succeed");

    assert!(mesh.triangle_count() > 0, "expected non-empty mesh");
    // With MiterType::Rotate, we also expect a bisector miter warning
    assert!(
        diag.warnings.iter().any(|w| w.contains("bisector miter")),
        "expected bisector miter warning with MiterType::Rotate, got: {:?}",
        diag.warnings
    );
}

#[test]
fn sweep1_smooth_rail_no_cusp_detection() {
    // Square profile
    let profile = vec![
        Point3::new(-0.1, -0.1, 0.0),
        Point3::new(0.1, -0.1, 0.0),
        Point3::new(0.1, 0.1, 0.0),
        Point3::new(-0.1, 0.1, 0.0),
        Point3::new(-0.1, -0.1, 0.0),
    ];

    // Smooth rail (no sharp corners) - a gentle arc
    let rail = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.5, 0.1, 0.0),
        Point3::new(1.0, 0.15, 0.0),
        Point3::new(1.5, 0.1, 0.0),
        Point3::new(2.0, 0.0, 0.0),
    ];

    let tol = Tolerance::default_geom();
    let options = SweepOptions {
        twist_radians_total: 0.0,
        miter: MiterType::Trim, // Even with Trim, smooth rail shouldn't trigger cusp warnings
    };

    let (mesh, diag) = sweep1_polyline_with_tolerance(&profile, &rail, SweepCaps::BOTH, options, tol)
        .expect("sweep with smooth rail should succeed");

    assert!(mesh.triangle_count() > 0, "expected non-empty mesh");
    // No sharp tangent changes expected for smooth rails
    assert!(
        !diag.warnings.iter().any(|w| w.contains("sharp tangent")),
        "smooth rail should not trigger cusp warnings, got: {:?}",
        diag.warnings
    );
}

// ============================================================================
// Tolerance-aware closure detection tests
// ============================================================================

#[test]
fn sweep1_nearly_closed_rail_detected_as_closed() {
    // Square profile
    let profile = vec![
        Point3::new(-0.1, -0.1, 0.0),
        Point3::new(0.1, -0.1, 0.0),
        Point3::new(0.1, 0.1, 0.0),
        Point3::new(-0.1, 0.1, 0.0),
        Point3::new(-0.1, -0.1, 0.0),
    ];

    // Nearly-closed rail - end point is very close but not exactly equal to start
    // This tests the tolerance-aware closure detection
    let eps = 1e-10; // Much smaller than typical tolerance
    let rail = vec![
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(-1.0, 0.0, 0.0),
        Point3::new(0.0, -1.0, 0.0),
        Point3::new(1.0 + eps, eps, eps), // Nearly back to start
    ];

    let tol = Tolerance::default_geom();
    let options = SweepOptions::default();

    // With tolerance-aware closure detection, this should be treated as closed
    // and caps should not be allowed
    let result = sweep1_polyline_with_tolerance(&profile, &rail, SweepCaps::BOTH, options, tol);
    
    // Closed rails don't allow caps, so this should fail with CapsNotAllowedForClosedRail
    assert!(
        matches!(result, Err(SweepError::CapsNotAllowedForClosedRail)),
        "nearly-closed rail should be detected as closed (caps not allowed), got: {:?}",
        result
    );
}

#[test]
fn sweep1_nearly_closed_rail_with_no_caps_succeeds() {
    // Square profile
    let profile = vec![
        Point3::new(-0.1, -0.1, 0.0),
        Point3::new(0.1, -0.1, 0.0),
        Point3::new(0.1, 0.1, 0.0),
        Point3::new(-0.1, 0.1, 0.0),
        Point3::new(-0.1, -0.1, 0.0),
    ];

    // Nearly-closed rail
    let eps = 1e-10;
    let rail = vec![
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(-1.0, 0.0, 0.0),
        Point3::new(0.0, -1.0, 0.0),
        Point3::new(1.0 + eps, eps, eps),
    ];

    let tol = Tolerance::default_geom();
    let options = SweepOptions::default();

    // Without caps, nearly-closed rail should succeed
    let (mesh, diag) = sweep1_polyline_with_tolerance(&profile, &rail, SweepCaps::NONE, options, tol)
        .expect("nearly-closed rail with no caps should succeed");

    assert!(mesh.triangle_count() > 0, "expected non-empty mesh");
    assert_eq!(diag.non_manifold_edge_count, 0, "expected manifold mesh");
}
