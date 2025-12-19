//! Tests for surface builder types: FourPointSurface, RuledSurface, EdgeSurface, SumSurface, NetworkSurface.

use crate::geom::{
    mesh_edge_surface, mesh_edge_surface_from_edges, mesh_four_point_surface,
    mesh_four_point_surface_from_points, mesh_network_surface, mesh_network_surface_from_grid,
    mesh_ruled_surface, mesh_sum_surface, EdgeSurface, FourPointSurface, NetworkSurface, Point3,
    RuledSurface, SumSurface, Surface, SurfaceBuilderQuality, Tolerance,
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
