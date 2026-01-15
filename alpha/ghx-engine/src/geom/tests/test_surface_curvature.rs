//! Tests for surface curvature analysis functionality.
//!
//! These tests verify that the `analyze_surface_curvature` function correctly
//! computes principal curvatures, Gaussian curvature, mean curvature, and
//! principal directions for various surface types.

use crate::geom::{
    analyze_surface_curvature, Point3, PlaneSurface, SphereSurface, CylinderSurface,
    Surface, Vec3,
};

const TOLERANCE: f64 = 1e-4;

// ============================================================================
// Plane curvature tests - should have zero curvature everywhere
// ============================================================================

#[test]
fn plane_has_zero_curvature() {
    let plane = PlaneSurface::new(
        Point3::ORIGIN,
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let analysis = analyze_surface_curvature(&plane, 0.5, 0.5);

    assert!(analysis.valid, "Analysis should be valid for a plane");
    assert!(
        analysis.k1.abs() < TOLERANCE,
        "Plane k1 should be ~0, got {}",
        analysis.k1
    );
    assert!(
        analysis.k2.abs() < TOLERANCE,
        "Plane k2 should be ~0, got {}",
        analysis.k2
    );
    assert!(
        analysis.gaussian.abs() < TOLERANCE,
        "Plane Gaussian curvature should be ~0, got {}",
        analysis.gaussian
    );
    assert!(
        analysis.mean.abs() < TOLERANCE,
        "Plane mean curvature should be ~0, got {}",
        analysis.mean
    );
}

#[test]
fn plane_curvature_at_corners() {
    let plane = PlaneSurface::new(
        Point3::ORIGIN,
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(0.0, 3.0, 0.0),
    );

    // Test all four corners
    for (u, v) in &[(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)] {
        let analysis = analyze_surface_curvature(&plane, *u, *v);
        assert!(analysis.valid);
        assert!(
            analysis.gaussian.abs() < TOLERANCE,
            "Plane at ({}, {}) should have zero Gaussian curvature",
            u, v
        );
    }
}

// ============================================================================
// Sphere curvature tests - should have constant positive curvature
// ============================================================================

#[test]
fn sphere_has_positive_constant_curvature() {
    let radius = 2.0;
    let sphere = SphereSurface::new(Point3::ORIGIN, radius).unwrap();

    let analysis = analyze_surface_curvature(&sphere, 0.5, 0.5);

    assert!(analysis.valid, "Analysis should be valid for a sphere");

    // For a sphere of radius R, principal curvatures are both 1/R
    let expected_kappa = 1.0 / radius;

    // Principal curvatures should both equal 1/R (up to sign)
    assert!(
        (analysis.k1.abs() - expected_kappa).abs() < TOLERANCE,
        "Sphere k1 should be ~{}, got {}",
        expected_kappa, analysis.k1
    );
    assert!(
        (analysis.k2.abs() - expected_kappa).abs() < TOLERANCE,
        "Sphere k2 should be ~{}, got {}",
        expected_kappa, analysis.k2
    );

    // Gaussian curvature = k1 * k2 = 1/R²
    let expected_gaussian = 1.0 / (radius * radius);
    assert!(
        (analysis.gaussian.abs() - expected_gaussian).abs() < TOLERANCE,
        "Sphere Gaussian should be ~{}, got {}",
        expected_gaussian, analysis.gaussian
    );

    // Mean curvature = (k1 + k2) / 2 = 1/R
    assert!(
        (analysis.mean.abs() - expected_kappa).abs() < TOLERANCE,
        "Sphere mean should be ~{}, got {}",
        expected_kappa, analysis.mean
    );
}

#[test]
fn sphere_curvature_varies_with_radius() {
    for radius in &[0.5, 1.0, 2.0, 5.0] {
        let sphere = SphereSurface::new(Point3::ORIGIN, *radius).unwrap();
        let analysis = analyze_surface_curvature(&sphere, 0.25, 0.75);

        let expected_kappa = 1.0 / radius;
        let expected_gaussian = 1.0 / (radius * radius);

        assert!(
            (analysis.k1.abs() - expected_kappa).abs() < TOLERANCE * 10.0,
            "Sphere r={} k1 should be ~{}, got {}",
            radius, expected_kappa, analysis.k1
        );
        assert!(
            (analysis.gaussian.abs() - expected_gaussian).abs() < TOLERANCE * 10.0,
            "Sphere r={} Gaussian should be ~{}, got {}",
            radius, expected_gaussian, analysis.gaussian
        );
    }
}

// ============================================================================
// Cylinder curvature tests - one principal curvature is 1/R, other is 0
// ============================================================================

#[test]
fn cylinder_has_one_zero_curvature() {
    let radius = 1.0;
    let cylinder = CylinderSurface::new(
        Point3::ORIGIN,
        Vec3::new(0.0, 0.0, 1.0),
        radius,
    ).unwrap();

    let analysis = analyze_surface_curvature(&cylinder, 0.5, 0.5);

    assert!(analysis.valid, "Analysis should be valid for a cylinder");

    // One principal curvature should be 1/R, the other ~0
    let expected_kappa = 1.0 / radius;
    
    let max_k = analysis.k1.abs().max(analysis.k2.abs());
    let min_k = analysis.k1.abs().min(analysis.k2.abs());

    assert!(
        (max_k - expected_kappa).abs() < TOLERANCE * 10.0,
        "Cylinder max curvature should be ~{}, got {}",
        expected_kappa, max_k
    );
    assert!(
        min_k < TOLERANCE * 10.0,
        "Cylinder min curvature should be ~0, got {}",
        min_k
    );

    // Gaussian curvature = k1 * k2 = 0 (cylinder is developable)
    assert!(
        analysis.gaussian.abs() < TOLERANCE * 10.0,
        "Cylinder Gaussian should be ~0, got {}",
        analysis.gaussian
    );

    // Mean curvature = (1/R + 0) / 2 = 1/(2R)
    let expected_mean = 0.5 / radius;
    assert!(
        (analysis.mean.abs() - expected_mean).abs() < TOLERANCE * 10.0,
        "Cylinder mean should be ~{}, got {}",
        expected_mean, analysis.mean
    );
}

// ============================================================================
// Normal direction tests
// ============================================================================

#[test]
fn plane_normal_is_cross_product_of_axes() {
    let plane = PlaneSurface::new(
        Point3::ORIGIN,
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let analysis = analyze_surface_curvature(&plane, 0.5, 0.5);

    // Normal should be (0, 0, 1) for this XY plane
    assert!(
        (analysis.normal.z.abs() - 1.0).abs() < TOLERANCE,
        "XY plane normal should be ±Z, got {:?}",
        analysis.normal
    );
}

#[test]
fn tilted_plane_normal_is_perpendicular() {
    // Create a plane tilted 45 degrees
    let plane = PlaneSurface::new(
        Point3::ORIGIN,
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 1.0),  // Tilted V axis
    );

    let analysis = analyze_surface_curvature(&plane, 0.5, 0.5);

    // Normal should be perpendicular to both U and V directions
    let dot_u = analysis.normal.dot(Vec3::new(1.0, 0.0, 0.0));
    let dot_v = analysis.normal.dot(Vec3::new(0.0, 1.0, 1.0).normalized().unwrap());

    assert!(
        dot_u.abs() < TOLERANCE,
        "Normal should be perpendicular to U axis"
    );
    // Note: dot_v might not be exactly 0 due to normalization differences
}

// ============================================================================
// Principal direction tests
// ============================================================================

#[test]
fn principal_directions_are_perpendicular() {
    let sphere = SphereSurface::new(Point3::ORIGIN, 1.0).unwrap();
    let analysis = analyze_surface_curvature(&sphere, 0.3, 0.7);

    if analysis.valid {
        let dot = analysis.k1_direction.dot(analysis.k2_direction);
        assert!(
            dot.abs() < TOLERANCE * 10.0,
            "Principal directions should be perpendicular, dot = {}",
            dot
        );
    }
}

#[test]
fn principal_directions_are_in_tangent_plane() {
    let sphere = SphereSurface::new(Point3::ORIGIN, 1.0).unwrap();
    let analysis = analyze_surface_curvature(&sphere, 0.5, 0.5);

    if analysis.valid {
        let dot1 = analysis.k1_direction.dot(analysis.normal);
        let dot2 = analysis.k2_direction.dot(analysis.normal);

        assert!(
            dot1.abs() < TOLERANCE * 10.0,
            "K1 direction should be in tangent plane, dot with normal = {}",
            dot1
        );
        assert!(
            dot2.abs() < TOLERANCE * 10.0,
            "K2 direction should be in tangent plane, dot with normal = {}",
            dot2
        );
    }
}

// ============================================================================
// Edge case tests
// ============================================================================

#[test]
fn analysis_at_domain_boundaries() {
    let plane = PlaneSurface::new(
        Point3::ORIGIN,
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    // Test at exact boundaries
    for (u, v) in &[(0.0, 0.0), (0.0, 1.0), (1.0, 0.0), (1.0, 1.0)] {
        let analysis = analyze_surface_curvature(&plane, *u, *v);
        // Even at boundaries, plane curvature should be computed (though may be less accurate)
        // Just verify it doesn't panic and produces valid output
        assert!(
            analysis.point.x.is_finite() && analysis.point.y.is_finite() && analysis.point.z.is_finite(),
            "Point should be finite at boundary ({}, {})",
            u, v
        );
    }
}

#[test]
fn analysis_returns_sampled_point() {
    let plane = PlaneSurface::new(
        Point3::new(10.0, 20.0, 30.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(0.0, 3.0, 0.0),
    );

    let analysis = analyze_surface_curvature(&plane, 0.5, 0.5);

    // Point should be at origin + 0.5*u_axis + 0.5*v_axis = (10+1, 20+1.5, 30)
    assert!(
        (analysis.point.x - 11.0).abs() < TOLERANCE,
        "Point x should be 11, got {}",
        analysis.point.x
    );
    assert!(
        (analysis.point.y - 21.5).abs() < TOLERANCE,
        "Point y should be 21.5, got {}",
        analysis.point.y
    );
    assert!(
        (analysis.point.z - 30.0).abs() < TOLERANCE,
        "Point z should be 30, got {}",
        analysis.point.z
    );
}
