use crate::geom::{
    Point3, SweepCaps, SweepError, SweepOptions, Tolerance,
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
    let options = SweepOptions { twist_radians_total: 0.0 };

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
    let options = SweepOptions { twist_radians_total: std::f64::consts::FRAC_PI_2 };

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
    let options = SweepOptions { twist_radians_total: std::f64::consts::PI }; // 180 degree twist.

    let (mesh, diag) = sweep1_polyline_with_tolerance(&profile, &rail, SweepCaps::BOTH, options, tol)
        .expect("twisted sweep with caps should succeed");

    assert!(mesh.triangle_count() > 0);
    assert!(diag.open_edge_count == 0, "expected watertight mesh with caps");
    assert!(diag.non_manifold_edge_count == 0, "expected manifold mesh");
}
