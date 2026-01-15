//! Tests for surface builder types: FourPointSurface, RuledSurface, EdgeSurface, SumSurface, NetworkSurface.

use crate::geom::{
    mesh_edge_surface, mesh_edge_surface_from_edges, mesh_four_point_surface,
    mesh_four_point_surface_from_points, mesh_network_surface, mesh_network_surface_from_grid,
    mesh_ruled_surface, mesh_sum_surface, EdgeSurface, FourPointSurface, GeomMesh, NetworkSurface,
    Point3, RuledSurface, SumSurface, Surface, SurfaceBuilderQuality, Tolerance,
};

// ---------------------------------------------------------------------------
// FourPointSurface Tests
// ---------------------------------------------------------------------------

#[test]
fn four_point_surface_corners() {
    let p00 = Point3::new(0.0, 0.0, 0.0);
    let p10 = Point3::new(1.0, 0.0, 0.0);
    let p01 = Point3::new(0.0, 1.0, 0.0);
    let p11 = Point3::new(1.0, 1.0, 0.0);

    let surface = FourPointSurface::new(p00, p10, p01, p11);

    let tol = Tolerance::default_geom();

    // Check corners
    assert!(tol.approx_eq_point3(surface.point_at(0.0, 0.0), p00));
    assert!(tol.approx_eq_point3(surface.point_at(1.0, 0.0), p10));
    assert!(tol.approx_eq_point3(surface.point_at(0.0, 1.0), p01));
    assert!(tol.approx_eq_point3(surface.point_at(1.0, 1.0), p11));
}

#[test]
fn four_point_surface_center() {
    let p00 = Point3::new(0.0, 0.0, 0.0);
    let p10 = Point3::new(2.0, 0.0, 0.0);
    let p01 = Point3::new(0.0, 2.0, 0.0);
    let p11 = Point3::new(2.0, 2.0, 0.0);

    let surface = FourPointSurface::new(p00, p10, p01, p11);

    // Center should be the average of all four corners
    let center = surface.point_at(0.5, 0.5);
    let expected = Point3::new(1.0, 1.0, 0.0);

    let tol = Tolerance::default_geom();
    assert!(tol.approx_eq_point3(center, expected));
}

#[test]
fn four_point_surface_from_three_points() {
    let points = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];

    let surface = FourPointSurface::from_points(&points).expect("should create from 3 points");

    // Fourth point should complete the parallelogram
    let p11 = surface.point_at(1.0, 1.0);
    let expected = Point3::new(1.0, 1.0, 0.0);

    let tol = Tolerance::default_geom();
    assert!(tol.approx_eq_point3(p11, expected));
}

#[test]
fn four_point_surface_mesh_valid() {
    let p00 = Point3::new(0.0, 0.0, 0.0);
    let p10 = Point3::new(1.0, 0.0, 0.0);
    let p01 = Point3::new(0.0, 1.0, 0.0);
    let p11 = Point3::new(1.0, 1.0, 0.0);

    let quality = SurfaceBuilderQuality::new(4, 4);
    let (mesh, diagnostics) = mesh_four_point_surface(p00, p10, p01, p11, quality);

    // Should have vertices and triangles
    assert!(!mesh.positions.is_empty(), "mesh should have vertices");
    assert!(!mesh.indices.is_empty(), "mesh should have indices");
    assert!(
        mesh.indices.len() % 3 == 0,
        "indices should be multiple of 3"
    );

    // Check diagnostics
    assert!(diagnostics.vertex_count > 0);
    assert!(diagnostics.triangle_count > 0);

    // No NaN vertices
    for pos in &mesh.positions {
        assert!(pos[0].is_finite(), "vertex x should be finite");
        assert!(pos[1].is_finite(), "vertex y should be finite");
        assert!(pos[2].is_finite(), "vertex z should be finite");
    }
}

// ---------------------------------------------------------------------------
// RuledSurface Tests
// ---------------------------------------------------------------------------

#[test]
fn ruled_surface_corners() {
    let curve_a = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)];
    let curve_b = vec![Point3::new(0.0, 1.0, 0.0), Point3::new(1.0, 1.0, 0.0)];

    let surface = RuledSurface::new(curve_a.clone(), curve_b.clone()).expect("should create");

    let tol = Tolerance::default_geom();

    // Check corners
    assert!(tol.approx_eq_point3(surface.point_at(0.0, 0.0), curve_a[0]));
    assert!(tol.approx_eq_point3(surface.point_at(1.0, 0.0), curve_a[1]));
    assert!(tol.approx_eq_point3(surface.point_at(0.0, 1.0), curve_b[0]));
    assert!(tol.approx_eq_point3(surface.point_at(1.0, 1.0), curve_b[1]));
}

#[test]
fn ruled_surface_midpoint() {
    let curve_a = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(2.0, 0.0, 0.0)];
    let curve_b = vec![Point3::new(0.0, 2.0, 0.0), Point3::new(2.0, 2.0, 0.0)];

    let surface = RuledSurface::new(curve_a, curve_b).expect("should create");

    // Center should be (1, 1, 0)
    let center = surface.point_at(0.5, 0.5);
    let expected = Point3::new(1.0, 1.0, 0.0);

    let tol = Tolerance::default_geom();
    assert!(tol.approx_eq_point3(center, expected));
}

#[test]
fn ruled_surface_mesh_valid() {
    let curve_a = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.5, 0.0, 0.5),
        Point3::new(1.0, 0.0, 0.0),
    ];
    let curve_b = vec![Point3::new(0.0, 1.0, 0.0), Point3::new(1.0, 1.0, 0.0)];

    let quality = SurfaceBuilderQuality::new(5, 3);
    let (mesh, diagnostics) =
        mesh_ruled_surface(&curve_a, &curve_b, quality).expect("should mesh");

    assert!(!mesh.positions.is_empty());
    assert!(!mesh.indices.is_empty());
    assert!(diagnostics.vertex_count > 0);
    assert!(diagnostics.triangle_count > 0);
}

// ---------------------------------------------------------------------------
// EdgeSurface (Coons Patch) Tests
// ---------------------------------------------------------------------------

#[test]
fn edge_surface_corners() {
    // Create a simple quad boundary
    let edge_u0 = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)];
    let edge_u1 = vec![Point3::new(0.0, 1.0, 0.0), Point3::new(1.0, 1.0, 0.0)];
    let edge_v0 = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0)];
    let edge_v1 = vec![Point3::new(1.0, 0.0, 0.0), Point3::new(1.0, 1.0, 0.0)];

    let surface = EdgeSurface::new(edge_u0.clone(), edge_u1.clone(), edge_v0, edge_v1)
        .expect("should create");

    let tol = Tolerance::default_geom();

    // Check corners
    assert!(tol.approx_eq_point3(surface.point_at(0.0, 0.0), edge_u0[0]));
    assert!(tol.approx_eq_point3(surface.point_at(1.0, 0.0), edge_u0[1]));
    assert!(tol.approx_eq_point3(surface.point_at(0.0, 1.0), edge_u1[0]));
    assert!(tol.approx_eq_point3(surface.point_at(1.0, 1.0), edge_u1[1]));
}

#[test]
fn edge_surface_from_two_edges() {
    // Create from two edges (ruled surface behavior)
    let edge1 = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)];
    let edge2 = vec![Point3::new(0.0, 1.0, 1.0), Point3::new(1.0, 1.0, 1.0)];

    let surface = EdgeSurface::from_edges(&[edge1.clone(), edge2.clone()]).expect("should create");

    let tol = Tolerance::default_geom();

    // Check that the surface passes through the edge curves
    assert!(tol.approx_eq_point3(surface.point_at(0.0, 0.0), edge1[0]));
    assert!(tol.approx_eq_point3(surface.point_at(1.0, 0.0), edge1[1]));
}

#[test]
fn edge_surface_mesh_valid() {
    let edge_u0 = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)];
    let edge_u1 = vec![Point3::new(0.0, 1.0, 0.0), Point3::new(1.0, 1.0, 0.0)];
    let edge_v0 = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0)];
    let edge_v1 = vec![Point3::new(1.0, 0.0, 0.0), Point3::new(1.0, 1.0, 0.0)];

    let quality = SurfaceBuilderQuality::new(5, 5);
    let (mesh, diagnostics) =
        mesh_edge_surface(&edge_u0, &edge_u1, &edge_v0, &edge_v1, quality).expect("should mesh");

    assert!(!mesh.positions.is_empty());
    assert!(!mesh.indices.is_empty());
    assert!(diagnostics.vertex_count > 0);
    assert!(diagnostics.triangle_count > 0);
}

// ---------------------------------------------------------------------------
// SumSurface Tests
// ---------------------------------------------------------------------------

#[test]
fn sum_surface_corners() {
    // U-profile along X axis
    let curve_u = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)];
    // V-profile along Y axis
    let curve_v = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0)];

    let surface = SumSurface::new(curve_u.clone(), curve_v.clone()).expect("should create");

    let tol = Tolerance::default_geom();

    // Origin (u=0, v=0): curve_u[0] + curve_v[0] - origin = (0,0,0)
    let p00 = surface.point_at(0.0, 0.0);
    assert!(tol.approx_eq_point3(p00, Point3::new(0.0, 0.0, 0.0)));

    // (u=1, v=0): curve_u[1] + curve_v[0] - origin = (1,0,0)
    let p10 = surface.point_at(1.0, 0.0);
    assert!(tol.approx_eq_point3(p10, Point3::new(1.0, 0.0, 0.0)));

    // (u=0, v=1): curve_u[0] + curve_v[1] - origin = (0,1,0)
    let p01 = surface.point_at(0.0, 1.0);
    assert!(tol.approx_eq_point3(p01, Point3::new(0.0, 1.0, 0.0)));

    // (u=1, v=1): curve_u[1] + curve_v[1] - origin = (1,1,0)
    let p11 = surface.point_at(1.0, 1.0);
    assert!(tol.approx_eq_point3(p11, Point3::new(1.0, 1.0, 0.0)));
}

#[test]
fn sum_surface_translational() {
    // U-profile: a straight line
    let curve_u = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(2.0, 0.0, 0.0)];
    // V-profile: a raised arc-like curve
    let curve_v = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 0.5, 0.5),
        Point3::new(0.0, 1.0, 0.0),
    ];

    let surface = SumSurface::new(curve_u, curve_v).expect("should create");

    // At v=0.5, the V-profile reaches its maximum Z
    let mid = surface.point_at(0.5, 0.5);
    assert!(mid.z > 0.0, "surface should have positive Z at mid-V");
}

#[test]
fn sum_surface_mesh_valid() {
    let curve_u = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)];
    let curve_v = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.5),
        Point3::new(0.0, 2.0, 0.0),
    ];

    let quality = SurfaceBuilderQuality::new(4, 6);
    let (mesh, diagnostics) =
        mesh_sum_surface(&curve_u, &curve_v, quality).expect("should mesh");

    assert!(!mesh.positions.is_empty());
    assert!(!mesh.indices.is_empty());
    assert!(diagnostics.vertex_count > 0);
    assert!(diagnostics.triangle_count > 0);
}

// ---------------------------------------------------------------------------
// NetworkSurface Tests
// ---------------------------------------------------------------------------

#[test]
fn network_surface_from_grid() {
    // Create a simple 3x3 grid
    let grid = vec![
        vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
        ],
        vec![
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(1.0, 1.0, 0.5),
            Point3::new(2.0, 1.0, 0.0),
        ],
        vec![
            Point3::new(0.0, 2.0, 0.0),
            Point3::new(1.0, 2.0, 0.0),
            Point3::new(2.0, 2.0, 0.0),
        ],
    ];

    let surface = NetworkSurface::from_grid(grid).expect("should create");

    let tol = Tolerance::default_geom();

    // Check corners
    assert!(tol.approx_eq_point3(
        surface.point_at(0.0, 0.0),
        Point3::new(0.0, 0.0, 0.0)
    ));
    assert!(tol.approx_eq_point3(
        surface.point_at(1.0, 0.0),
        Point3::new(2.0, 0.0, 0.0)
    ));
    assert!(tol.approx_eq_point3(
        surface.point_at(0.0, 1.0),
        Point3::new(0.0, 2.0, 0.0)
    ));
    assert!(tol.approx_eq_point3(
        surface.point_at(1.0, 1.0),
        Point3::new(2.0, 2.0, 0.0)
    ));
}

#[test]
fn network_surface_from_curves() {
    // U-curves (rows)
    let u_curves = vec![
        vec![Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)],
        vec![Point3::new(0.0, 1.0, 0.5), Point3::new(1.0, 1.0, 0.5)],
    ];

    // V-curves (columns) - used to determine grid density
    let v_curves = vec![
        vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.5)],
        vec![Point3::new(1.0, 0.0, 0.0), Point3::new(1.0, 1.0, 0.5)],
    ];

    let surface = NetworkSurface::new(&u_curves, &v_curves).expect("should create");

    // Surface should be valid
    let center = surface.point_at(0.5, 0.5);
    assert!(center.x.is_finite());
    assert!(center.y.is_finite());
    assert!(center.z.is_finite());
}

/// Regression test: ensure V-curves contribute to the surface shape.
///
/// This test creates a network where U-curves and V-curves have different heights.
/// The V-curves have a "bump" in the middle that should be reflected in the surface.
/// If V-curves were ignored, the surface would only follow U-curve heights.
#[test]
fn network_surface_v_curves_affect_shape() {
    // U-curves: straight lines at z=0 and z=0
    let u_curves = vec![
        vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(0.5, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
        ],
        vec![
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(0.5, 1.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
        ],
    ];

    // V-curves: have a bulge in the middle at z=2.0
    // This is the key difference - the V-curves have height that U-curves don't capture
    let v_curves = vec![
        vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(0.0, 0.5, 2.0), // Middle point raised to z=2.0
            Point3::new(0.0, 1.0, 0.0),
        ],
        vec![
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(1.0, 0.5, 2.0), // Middle point raised to z=2.0
            Point3::new(1.0, 1.0, 0.0),
        ],
    ];

    let surface = NetworkSurface::new(&u_curves, &v_curves).expect("should create");

    // Check corners match (where U and V curves meet)

    // Corner (0, 0): should be at z=0
    let corner_00 = surface.point_at(0.0, 0.0);
    assert!(
        corner_00.z.abs() < 0.01,
        "Corner (0,0) should have z≈0, got z={:.3}",
        corner_00.z
    );

    // Corner (1, 0): should be at z=0
    let corner_10 = surface.point_at(1.0, 0.0);
    assert!(
        corner_10.z.abs() < 0.01,
        "Corner (1,0) should have z≈0, got z={:.3}",
        corner_10.z
    );

    // Corner (0, 1): should be at z=0
    let corner_01 = surface.point_at(0.0, 1.0);
    assert!(
        corner_01.z.abs() < 0.01,
        "Corner (0,1) should have z≈0, got z={:.3}",
        corner_01.z
    );

    // Corner (1, 1): should be at z=0
    let corner_11 = surface.point_at(1.0, 1.0);
    assert!(
        corner_11.z.abs() < 0.01,
        "Corner (1,1) should have z≈0, got z={:.3}",
        corner_11.z
    );

    // Now check the middle of the left edge (u=0, v=0.5)
    // This should be influenced by the V-curve bulge at z=2.0
    // Note: Since we find intersection/closest points between curves,
    // and the V-curve at x=0 has a point at (0, 0.5, 2.0), this should be reflected.
    let left_mid = surface.point_at(0.0, 0.5);
    
    // The original code would sample U-curves at parameter 0.5, giving z=0
    // The fixed code should find the intersection with V-curve, which has z=2.0 at y=0.5
    // Since we're blending closest points, the actual z might be less than 2.0
    // but should definitely be greater than 0
    assert!(
        left_mid.z > 0.5,
        "Left edge midpoint should have z>0.5 due to V-curve bulge, got z={:.3}. \
         This indicates V-curves are being ignored!",
        left_mid.z
    );
}

/// Test that non-intersecting curves still produce reasonable results.
///
/// When curves don't exactly intersect, the implementation should find
/// closest points and blend them.
#[test]
fn network_surface_non_intersecting_curves() {
    // U-curves at z=0
    let u_curves = vec![
        vec![Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)],
        vec![Point3::new(0.0, 2.0, 0.0), Point3::new(1.0, 2.0, 0.0)],
    ];

    // V-curves at z=1 (intentionally at a different height - they don't intersect)
    let v_curves = vec![
        vec![Point3::new(0.0, 0.0, 1.0), Point3::new(0.0, 2.0, 1.0)],
        vec![Point3::new(1.0, 0.0, 1.0), Point3::new(1.0, 2.0, 1.0)],
    ];

    let surface = NetworkSurface::new(&u_curves, &v_curves).expect("should create");

    // The surface should blend both curve families
    // Since U-curves are at z=0 and V-curves at z=1, corners should be at ~z=0.5
    let corner = surface.point_at(0.0, 0.0);
    assert!(
        (corner.z - 0.5).abs() < 0.01,
        "Corner should blend U (z=0) and V (z=1) curves to z≈0.5, got z={:.3}",
        corner.z
    );

    // Check the grid is valid (no NaN or Inf values)
    for u in 0..=10 {
        for v in 0..=10 {
            let p = surface.point_at(u as f64 / 10.0, v as f64 / 10.0);
            assert!(p.x.is_finite(), "Point ({}, {}) has non-finite x", u, v);
            assert!(p.y.is_finite(), "Point ({}, {}) has non-finite y", u, v);
            assert!(p.z.is_finite(), "Point ({}, {}) has non-finite z", u, v);
        }
    }
}

#[test]
fn network_surface_mesh_valid() {
    let u_curves = vec![
        vec![Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)],
        vec![Point3::new(0.0, 1.0, 0.0), Point3::new(1.0, 1.0, 0.0)],
    ];
    let v_curves = vec![
        vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0)],
        vec![Point3::new(1.0, 0.0, 0.0), Point3::new(1.0, 1.0, 0.0)],
    ];

    let quality = SurfaceBuilderQuality::new(4, 4);
    let (mesh, diagnostics) =
        mesh_network_surface(&u_curves, &v_curves, quality).expect("should mesh");

    assert!(!mesh.positions.is_empty());
    assert!(!mesh.indices.is_empty());
    assert!(diagnostics.vertex_count > 0);
    assert!(diagnostics.triangle_count > 0);
}

// ---------------------------------------------------------------------------
// Mesh Sanity Tests (applies to all surface builders)
// ---------------------------------------------------------------------------

#[test]
fn all_surface_builders_produce_valid_indices() {
    let quality = SurfaceBuilderQuality::default();

    // FourPointSurface
    let (mesh, _) = mesh_four_point_surface(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        quality,
    );
    for idx in &mesh.indices {
        assert!(
            (*idx as usize) < mesh.positions.len(),
            "FourPointSurface index out of bounds"
        );
    }

    // RuledSurface
    let (mesh, _) = mesh_ruled_surface(
        &[Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)],
        &[Point3::new(0.0, 1.0, 0.0), Point3::new(1.0, 1.0, 0.0)],
        quality,
    )
    .expect("should mesh");
    for idx in &mesh.indices {
        assert!(
            (*idx as usize) < mesh.positions.len(),
            "RuledSurface index out of bounds"
        );
    }

    // EdgeSurface
    let (mesh, _) = mesh_edge_surface_from_edges(
        &[
            vec![Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)],
            vec![Point3::new(0.0, 1.0, 0.0), Point3::new(1.0, 1.0, 0.0)],
        ],
        quality,
    )
    .expect("should mesh");
    for idx in &mesh.indices {
        assert!(
            (*idx as usize) < mesh.positions.len(),
            "EdgeSurface index out of bounds"
        );
    }

    // SumSurface
    let (mesh, _) = mesh_sum_surface(
        &[Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)],
        &[Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0)],
        quality,
    )
    .expect("should mesh");
    for idx in &mesh.indices {
        assert!(
            (*idx as usize) < mesh.positions.len(),
            "SumSurface index out of bounds"
        );
    }

    // NetworkSurface
    let (mesh, _) = mesh_network_surface_from_grid(
        vec![
            vec![Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)],
            vec![Point3::new(0.0, 1.0, 0.0), Point3::new(1.0, 1.0, 0.0)],
        ],
        quality,
    )
    .expect("should mesh");
    for idx in &mesh.indices {
        assert!(
            (*idx as usize) < mesh.positions.len(),
            "NetworkSurface index out of bounds"
        );
    }
}

// ---------------------------------------------------------------------------
// Error Handling Tests
// ---------------------------------------------------------------------------

#[test]
fn four_point_surface_rejects_too_few_points() {
    let points = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)];
    let result = FourPointSurface::from_points(&points);
    assert!(result.is_err());
}

#[test]
fn ruled_surface_rejects_single_point() {
    let curve_a = vec![Point3::new(0.0, 0.0, 0.0)];
    let curve_b = vec![Point3::new(0.0, 1.0, 0.0), Point3::new(1.0, 1.0, 0.0)];
    let result = RuledSurface::new(curve_a, curve_b);
    assert!(result.is_err());
}

#[test]
fn edge_surface_rejects_single_point_edge() {
    let edge_u0 = vec![Point3::new(0.0, 0.0, 0.0)]; // Only one point
    let edge_u1 = vec![Point3::new(0.0, 1.0, 0.0), Point3::new(1.0, 1.0, 0.0)];
    let edge_v0 = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0)];
    let edge_v1 = vec![Point3::new(1.0, 0.0, 0.0), Point3::new(1.0, 1.0, 0.0)];

    let result = EdgeSurface::new(edge_u0, edge_u1, edge_v0, edge_v1);
    assert!(result.is_err());
}

#[test]
fn sum_surface_rejects_single_point() {
    let curve_u = vec![Point3::new(0.0, 0.0, 0.0)];
    let curve_v = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0)];
    let result = SumSurface::new(curve_u, curve_v);
    assert!(result.is_err());
}

#[test]
fn network_surface_rejects_empty_grid() {
    let grid: Vec<Vec<Point3>> = vec![];
    let result = NetworkSurface::from_grid(grid);
    assert!(result.is_err());
}

#[test]
fn network_surface_rejects_1x1_grid() {
    let grid = vec![vec![Point3::new(0.0, 0.0, 0.0)]];
    let result = NetworkSurface::from_grid(grid);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// EdgeSurface Auto-Orientation Tests
// ---------------------------------------------------------------------------

/// Test that two edges with opposite directions are auto-oriented correctly.
/// This is the core case that was causing twisted surfaces before the fix.
#[test]
fn edge_surface_auto_orients_reversed_two_edges() {
    // Edge 1: left to right (0,0) -> (1,0)
    let edge1 = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)];
    // Edge 2: deliberately reversed - right to left (1,1) -> (0,1)
    // Without auto-orientation, this would create a twisted surface
    let edge2 = vec![Point3::new(1.0, 1.0, 0.0), Point3::new(0.0, 1.0, 0.0)];

    let surface = EdgeSurface::from_edges(&[edge1, edge2]).expect("should create surface");

    let tol = Tolerance::default_geom();

    // After auto-orientation, the surface should NOT be twisted.
    // The edges should flow in compatible directions.
    // Check that corner p00 (u=0, v=0) is at (0,0,0) and p10 (u=1, v=0) is at (1,0,0)
    assert!(tol.approx_eq_point3(surface.point_at(0.0, 0.0), Point3::new(0.0, 0.0, 0.0)));
    assert!(tol.approx_eq_point3(surface.point_at(1.0, 0.0), Point3::new(1.0, 0.0, 0.0)));

    // After auto-orientation, edge2 should have been flipped, so:
    // p01 (u=0, v=1) should be near (0,1,0) and p11 (u=1, v=1) should be near (1,1,0)
    assert!(tol.approx_eq_point3(surface.point_at(0.0, 1.0), Point3::new(0.0, 1.0, 0.0)));
    assert!(tol.approx_eq_point3(surface.point_at(1.0, 1.0), Point3::new(1.0, 1.0, 0.0)));

    // Verify the surface center is reasonable (not at some weird twisted location)
    let center = surface.point_at(0.5, 0.5);
    assert!(tol.approx_eq_point3(center, Point3::new(0.5, 0.5, 0.0)));
}

/// Test that two edges already in the same direction stay unchanged.
#[test]
fn edge_surface_preserves_same_direction_two_edges() {
    // Both edges go left to right
    let edge1 = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)];
    let edge2 = vec![Point3::new(0.0, 1.0, 0.0), Point3::new(1.0, 1.0, 0.0)];

    let surface = EdgeSurface::from_edges(&[edge1.clone(), edge2.clone()]).expect("should create");

    let tol = Tolerance::default_geom();

    // Corners should match the original edge endpoints
    assert!(tol.approx_eq_point3(surface.point_at(0.0, 0.0), edge1[0]));
    assert!(tol.approx_eq_point3(surface.point_at(1.0, 0.0), edge1[1]));
    assert!(tol.approx_eq_point3(surface.point_at(0.0, 1.0), edge2[0]));
    assert!(tol.approx_eq_point3(surface.point_at(1.0, 1.0), edge2[1]));
}

/// Test auto-orientation with four edges forming a proper boundary loop.
#[test]
fn edge_surface_auto_orients_four_edge_loop() {
    // Four edges forming a square, but given in arbitrary directions:
    // edge0: bottom - left to right (correct)
    // edge1: right - top to bottom (reversed!)
    // edge2: top - left to right (should be reversed to go right to left)
    // edge3: left - bottom to top (correct for the loop)

    let edge0 = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)]; // bottom
    let edge1 = vec![Point3::new(1.0, 1.0, 0.0), Point3::new(1.0, 0.0, 0.0)]; // right (reversed)
    let edge2 = vec![Point3::new(0.0, 1.0, 0.0), Point3::new(1.0, 1.0, 0.0)]; // top
    let edge3 = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0)]; // left

    let surface =
        EdgeSurface::from_edges(&[edge0, edge1, edge2, edge3]).expect("should create surface");

    let tol = Tolerance::LOOSE;

    // The surface should have proper corners after auto-orientation
    // Check that the four corners are at the expected positions
    let p00 = surface.point_at(0.0, 0.0);
    let p10 = surface.point_at(1.0, 0.0);
    let p01 = surface.point_at(0.0, 1.0);
    let p11 = surface.point_at(1.0, 1.0);

    // All corners should be on the unit square boundary
    assert!(
        tol.approx_eq_point3(p00, Point3::new(0.0, 0.0, 0.0)),
        "p00 should be at (0,0,0), got {:?}",
        p00
    );
    assert!(
        tol.approx_eq_point3(p10, Point3::new(1.0, 0.0, 0.0)),
        "p10 should be at (1,0,0), got {:?}",
        p10
    );
    assert!(
        tol.approx_eq_point3(p01, Point3::new(0.0, 1.0, 0.0)),
        "p01 should be at (0,1,0), got {:?}",
        p01
    );
    assert!(
        tol.approx_eq_point3(p11, Point3::new(1.0, 1.0, 0.0)),
        "p11 should be at (1,1,0), got {:?}",
        p11
    );
}

/// Test that a heavily scrambled 4-edge input still produces a valid surface.
#[test]
fn edge_surface_handles_scrambled_four_edges() {
    // All four edges of a square, but in completely random order and directions
    let bottom = vec![Point3::new(1.0, 0.0, 0.0), Point3::new(0.0, 0.0, 0.0)]; // reversed
    let right = vec![Point3::new(1.0, 0.0, 0.0), Point3::new(1.0, 1.0, 0.0)]; // correct
    let top = vec![Point3::new(1.0, 1.0, 0.0), Point3::new(0.0, 1.0, 0.0)]; // correct direction in loop
    let left = vec![Point3::new(0.0, 1.0, 0.0), Point3::new(0.0, 0.0, 0.0)]; // correct direction in loop

    // Pass in scrambled order
    let surface = EdgeSurface::from_edges(&[
        left.clone(),
        top.clone(),
        bottom.clone(),
        right.clone(),
    ])
    .expect("should create surface");

    let tol = Tolerance::LOOSE;

    // The surface should still form a valid quad patch
    // Check that the center point is approximately at the center of the square
    let center = surface.point_at(0.5, 0.5);
    assert!(
        (center.x - 0.5).abs() < 0.1 && (center.y - 0.5).abs() < 0.1 && center.z.abs() < 0.1,
        "Center should be near (0.5, 0.5, 0), got {:?}",
        center
    );

    // Verify the mesh is valid
    let quality = SurfaceBuilderQuality::default();
    let (mesh, diag) = mesh_edge_surface_from_edges(&[left, top, bottom, right], quality)
        .expect("should mesh");

    assert!(!mesh.positions.is_empty(), "mesh should have vertices");
    assert!(!mesh.indices.is_empty(), "mesh should have indices");
    assert!(diag.triangle_count > 0, "should have triangles");

    // Check for NaN/Inf in the mesh
    for pos in &mesh.positions {
        assert!(pos[0].is_finite(), "x should be finite");
        assert!(pos[1].is_finite(), "y should be finite");
        assert!(pos[2].is_finite(), "z should be finite");
    }
}

/// Test that the surface mesh from reversed edges produces similar geometry
/// to the mesh from correctly oriented edges.
#[test]
fn edge_surface_reversed_edges_produce_non_twisted_mesh() {
    // Correct orientation
    let edge1_correct = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.5, 0.0, 0.2),
        Point3::new(1.0, 0.0, 0.0),
    ];
    let edge2_correct = vec![
        Point3::new(0.0, 2.0, 0.0),
        Point3::new(0.5, 2.0, 0.2),
        Point3::new(1.0, 2.0, 0.0),
    ];

    // Reversed edge2
    let edge2_reversed: Vec<Point3> = edge2_correct.iter().copied().rev().collect();

    let quality = SurfaceBuilderQuality::new(10, 10);

    // Mesh with correct orientation
    let (mesh_correct, _) = mesh_edge_surface_from_edges(
        &[edge1_correct.clone(), edge2_correct],
        quality,
    )
    .expect("should mesh correct");

    // Mesh with auto-oriented reversed edge
    let (mesh_auto, _) =
        mesh_edge_surface_from_edges(&[edge1_correct, edge2_reversed], quality)
            .expect("should mesh auto-oriented");

    // Both meshes should have similar bounds (not twisted in weird directions)
    fn compute_bounds(mesh: &crate::geom::GeomMesh) -> ([f64; 3], [f64; 3]) {
        let mut min = [f64::MAX; 3];
        let mut max = [f64::MIN; 3];
        for pos in &mesh.positions {
            for i in 0..3 {
                min[i] = min[i].min(pos[i]);
                max[i] = max[i].max(pos[i]);
            }
        }
        (min, max)
    }

    let (min_correct, max_correct) = compute_bounds(&mesh_correct);
    let (min_auto, max_auto) = compute_bounds(&mesh_auto);

    // The bounds should be very similar (within a small tolerance)
    for i in 0..3 {
        assert!(
            (min_correct[i] - min_auto[i]).abs() < 0.01,
            "min bounds differ at axis {}: {:?} vs {:?}",
            i,
            min_correct,
            min_auto
        );
        assert!(
            (max_correct[i] - max_auto[i]).abs() < 0.01,
            "max bounds differ at axis {}: {:?} vs {:?}",
            i,
            max_correct,
            max_auto
        );
    }
}

/// Test edge surface with 3 edges (triangular patch case).
#[test]
fn edge_surface_three_edges_creates_valid_surface() {
    // Three edges forming a triangle-ish boundary
    let edge0 = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)]; // base
    let edge1 = vec![Point3::new(1.0, 0.0, 0.0), Point3::new(0.5, 1.0, 0.0)]; // right side
    let edge2 = vec![Point3::new(0.5, 1.0, 0.0), Point3::new(0.0, 0.0, 0.0)]; // left side

    let _surface = EdgeSurface::from_edges(&[edge0.clone(), edge1.clone(), edge2.clone()])
        .expect("should create surface");

    // Mesh should be valid
    let quality = SurfaceBuilderQuality::default();
    let (mesh, diag) =
        mesh_edge_surface_from_edges(&[edge0, edge1, edge2], quality).expect("should mesh");

    assert!(!mesh.positions.is_empty(), "mesh should have vertices");
    assert!(diag.triangle_count > 0, "should have triangles");

    // No NaN vertices
    for pos in &mesh.positions {
        assert!(pos[0].is_finite() && pos[1].is_finite() && pos[2].is_finite());
    }
}

/// Test that from_edges_with_tolerance allows custom tolerance.
#[test]
fn edge_surface_custom_tolerance() {
    let edge1 = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)];
    let edge2 = vec![Point3::new(1.0, 1.0, 0.0), Point3::new(0.0, 1.0, 0.0)]; // reversed

    // Use a tight tolerance
    let surface = EdgeSurface::from_edges_with_tolerance(&[edge1, edge2], Tolerance::TIGHT)
        .expect("should create with tight tolerance");

    let tol = Tolerance::default_geom();

    // Should still work correctly
    assert!(tol.approx_eq_point3(surface.point_at(0.0, 0.0), Point3::new(0.0, 0.0, 0.0)));
    assert!(tol.approx_eq_point3(surface.point_at(1.0, 1.0), Point3::new(1.0, 1.0, 0.0)));
}
