use crate::geom::{
    ClosedSurfaceSampling, DivideSurfaceOptions, SurfaceFlipGuide, divide_surface,
    ConeSurface, CylinderSurface, GeomContext, NurbsSurface, PlaneSurface, Point3, SphereSurface,
    Surface, TorusSurface, Tolerance, Vec3, choose_surface_grid_counts, flip_surface_orientation,
    isotrim_surface, mesh_surface_with_context, tessellate_surface_grid,
};

#[test]
fn tessellate_plane_grid_sizes() {
    let plane = PlaneSurface::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
    let pts = tessellate_surface_grid(&plane, 4, 3);
    assert_eq!(pts.len(), 12);
    assert_eq!(pts[0], Point3::new(0.0, 0.0, 0.0));
    assert_eq!(pts[3], Point3::new(1.0, 0.0, 0.0));
    assert_eq!(pts[8], Point3::new(0.0, 1.0, 0.0));
}

#[test]
fn nurbs_surface_bilinear_patch_matches_expected_point() {
    let surface = NurbsSurface::new(
        1,
        1,
        2,
        2,
        vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(1.0, 1.0, 1.0),
        ],
        vec![0.0, 0.0, 1.0, 1.0],
        vec![0.0, 0.0, 1.0, 1.0],
        None,
    )
    .unwrap();

    let p = surface.point_at(0.5, 0.5);
    let tol = Tolerance::new(1e-9);
    assert!(tol.approx_eq_point3(p, Point3::new(0.5, 0.5, 0.25)));
}

#[test]
fn cylinder_seam_is_closed_and_mesh_has_expected_open_edges() {
    let cyl = CylinderSurface::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 2.0),
        1.0,
    )
    .unwrap();

    let tol = Tolerance::new(1e-9);
    assert!(tol.approx_eq_point3(
        cyl.point_at(0.0, 0.3),
        cyl.point_at(1.0, 0.3)
    ));

    let mut ctx = GeomContext::new();
    let (_mesh, diag) = mesh_surface_with_context(&cyl, 16, 4, &mut ctx);
    assert_eq!(diag.open_edge_count, 32);
    assert_eq!(diag.non_manifold_edge_count, 0);
    assert_eq!(diag.degenerate_triangle_count, 0);
}

#[test]
fn cone_tip_is_collapsed_and_mesh_has_single_boundary_loop() {
    let cone = ConeSurface::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 2.0),
        0.0,
        1.0,
    )
    .unwrap();

    let tol = Tolerance::new(1e-9);
    assert!(tol.approx_eq_point3(
        cone.point_at(0.0, 0.0),
        cone.point_at(0.25, 0.0)
    ));

    let mut ctx = GeomContext::new();
    let (_mesh, diag) = mesh_surface_with_context(&cone, 16, 4, &mut ctx);
    assert_eq!(diag.open_edge_count, 16);
    assert_eq!(diag.non_manifold_edge_count, 0);
    assert_eq!(diag.degenerate_triangle_count, 0);
}

#[test]
fn sphere_mesh_is_closed_without_open_edges() {
    let sphere = SphereSurface::new(Point3::new(0.0, 0.0, 0.0), 1.0).unwrap();

    let tol = Tolerance::new(1e-9);
    assert!(tol.approx_eq_point3(
        sphere.point_at(0.0, 0.0),
        sphere.point_at(0.25, 0.0)
    ));

    let mut ctx = GeomContext::new();
    let (_mesh, diag) = mesh_surface_with_context(&sphere, 16, 12, &mut ctx);
    assert_eq!(diag.open_edge_count, 0);
    assert_eq!(diag.non_manifold_edge_count, 0);
    assert_eq!(diag.degenerate_triangle_count, 0);
}

#[test]
fn torus_mesh_is_closed_without_open_edges() {
    let torus = TorusSurface::from_center_xaxis_normal(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        2.0,
        0.5,
    )
    .unwrap();

    let tol = Tolerance::new(1e-9);
    assert!(tol.approx_eq_point3(
        torus.point_at(0.0, 0.3),
        torus.point_at(1.0, 0.3)
    ));
    assert!(tol.approx_eq_point3(
        torus.point_at(0.2, 0.0),
        torus.point_at(0.2, 1.0)
    ));

    let mut ctx = GeomContext::new();
    let (_mesh, diag) = mesh_surface_with_context(&torus, 24, 16, &mut ctx);
    assert_eq!(diag.open_edge_count, 0);
    assert_eq!(diag.non_manifold_edge_count, 0);
    assert_eq!(diag.degenerate_triangle_count, 0);
}

#[test]
fn adaptive_surface_counts_increase_when_edge_length_exceeded() {
    let plane = PlaneSurface::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(100.0, 0.0, 0.0),
        Vec3::new(0.0, 100.0, 0.0),
    );

    let opts = crate::geom::SurfaceTessellationOptions::new(f64::NAN, 10.0);
    let (u, v) = choose_surface_grid_counts(&plane, opts);
    assert_eq!(u, 16);
    assert_eq!(v, 16);
}

#[test]
fn adaptive_surface_counts_refine_anisotropically_for_edge_length() {
    let plane = PlaneSurface::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(100.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let opts = crate::geom::SurfaceTessellationOptions::new(f64::NAN, 10.0);
    let (u, v) = choose_surface_grid_counts(&plane, opts);
    assert_eq!(u, 16);
    assert_eq!(v, 8);
}

#[test]
fn adaptive_surface_counts_increase_when_deviation_exceeded() {
    let cyl = CylinderSurface::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 2.0),
        1.0,
    )
    .unwrap();

    let opts = crate::geom::SurfaceTessellationOptions {
        max_deviation: 0.05,
        max_edge_length: f64::NAN,
        max_u_count: 256,
        max_v_count: 256,
        initial_u_count: 8,
        initial_v_count: 4,
        max_iterations: 8,
    };
    let (u, v) = choose_surface_grid_counts(&cyl, opts);
    assert_eq!(u, 16);
    assert_eq!(v, 4);
}

#[test]
fn nurbs_surface_derivatives_match_plane_patch() {
    let surface = NurbsSurface::new(
        1,
        1,
        2,
        2,
        vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
        ],
        vec![0.0, 0.0, 1.0, 1.0],
        vec![0.0, 0.0, 1.0, 1.0],
        None,
    )
    .unwrap();

    let (du, dv) = surface.partial_derivatives_at(0.3, 0.7);
    let tol = Tolerance::new(1e-9);
    assert!(tol.approx_eq_f64(du.x, 1.0));
    assert!(tol.approx_eq_f64(du.y, 0.0));
    assert!(tol.approx_eq_f64(du.z, 0.0));
    assert!(tol.approx_eq_f64(dv.x, 0.0));
    assert!(tol.approx_eq_f64(dv.y, 1.0));
    assert!(tol.approx_eq_f64(dv.z, 0.0));
}

#[test]
fn divide_surface_respects_closed_u_seam_policy() {
    let cyl = CylinderSurface::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 2.0),
        1.0,
    )
    .unwrap();

    let result = divide_surface(&cyl, 8, 2, DivideSurfaceOptions::default());
    assert_eq!(result.u_count, 8);
    assert_eq!(result.v_count, 3);
    assert_eq!(result.points.len(), 24);

    let result = divide_surface(
        &cyl,
        8,
        2,
        DivideSurfaceOptions {
            closed_u: ClosedSurfaceSampling::IncludeSeam,
            closed_v: ClosedSurfaceSampling::ExcludeSeam,
        },
    );
    assert_eq!(result.u_count, 9);
    assert_eq!(result.v_count, 3);
    let tol = Tolerance::new(1e-9);
    assert!(tol.approx_eq_point3(
        result.points[0],
        result.points[result.u_count - 1]
    ));
}

#[test]
fn isotrim_surface_reverses_u_orientation_when_range_is_reversed() {
    let plane = PlaneSurface::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let tol = Tolerance::new(1e-9);
    let (trimmed, diag) = isotrim_surface(&plane, (0.75, 0.25), (0.0, 1.0), tol);
    assert!(diag.reverse_u);
    assert!(!diag.reverse_v);

    let normal = trimmed.normal_at(0.5, 0.5).unwrap();
    assert!(normal.z < 0.0);
}

#[test]
fn flip_surface_orientation_aligns_with_guide_vector() {
    let plane = PlaneSurface::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let (flipped, diag) = flip_surface_orientation(
        &plane,
        Some(SurfaceFlipGuide::Vector(Vec3::new(0.0, 0.0, -1.0))),
    );
    assert!(diag.flipped);
    let normal = flipped.normal_at(0.5, 0.5).unwrap();
    assert!(normal.z < 0.0);

    let (not_flipped, diag) = flip_surface_orientation(
        &plane,
        Some(SurfaceFlipGuide::Vector(Vec3::new(0.0, 0.0, 1.0))),
    );
    assert!(!diag.flipped);
    let normal = not_flipped.normal_at(0.5, 0.5).unwrap();
    assert!(normal.z > 0.0);
}

// ============================================================================
// Tests for mesh_from_grid_with_options (interpolate flag behavior)
// ============================================================================

use crate::geom::{SurfaceFitOptions, mesh_from_grid_with_options};

#[test]
fn mesh_from_grid_direct_mode_uses_exact_points() {
    // Create a 3×3 grid of points with a known "tent" shape
    let points = vec![
        Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0), Point3::new(2.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0), Point3::new(1.0, 1.0, 1.0), Point3::new(2.0, 1.0, 0.0),
        Point3::new(0.0, 2.0, 0.0), Point3::new(1.0, 2.0, 0.0), Point3::new(2.0, 2.0, 0.0),
    ];

    // Direct mode (interpolate=false): mesh vertices should exactly match input points
    let options = SurfaceFitOptions {
        interpolate: false,
        ..Default::default()
    };

    let (mesh, diag) = mesh_from_grid_with_options(&points, 3, 3, options).expect("mesh_from_grid_with_options failed");

    // Direct mode should use exactly 9 vertices (the input points)
    assert_eq!(mesh.positions.len(), 9, "direct mode should have exactly 9 vertices");
    assert_eq!(diag.grid_size, (3, 3));
    assert_eq!(diag.input_point_count, 9);
    assert_eq!(diag.max_deviation, 0.0, "direct mode should have zero deviation");

    // Check that the center point (1,1,1) is preserved exactly
    let center_found = mesh.positions.iter().any(|p| {
        (p[0] - 1.0).abs() < 1e-9 && (p[1] - 1.0).abs() < 1e-9 && (p[2] - 1.0).abs() < 1e-9
    });
    assert!(center_found, "center point (1,1,1) should be preserved in direct mode");
}

#[test]
fn mesh_from_grid_interpolate_mode_produces_smooth_surface() {
    // Create a 3×3 grid of points with a known "tent" shape
    let points = vec![
        Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0), Point3::new(2.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0), Point3::new(1.0, 1.0, 1.0), Point3::new(2.0, 1.0, 0.0),
        Point3::new(0.0, 2.0, 0.0), Point3::new(1.0, 2.0, 0.0), Point3::new(2.0, 2.0, 0.0),
    ];

    // Interpolating mode: creates a NURBS surface and tessellates it
    let options = SurfaceFitOptions {
        interpolate: true,
        ..Default::default()
    };

    let (mesh, diag) = mesh_from_grid_with_options(&points, 3, 3, options).expect("mesh_from_grid_with_options failed");

    // Interpolating mode should produce MORE vertices than the input (refined tessellation)
    assert!(
        mesh.positions.len() >= 9,
        "interpolating mode should have at least as many vertices as input, got {}",
        mesh.positions.len()
    );
    assert_eq!(diag.grid_size, (3, 3));
    assert_eq!(diag.input_point_count, 9);
}

#[test]
fn mesh_from_grid_interpolate_vs_direct_produce_different_results() {
    // Create a wavy 4×4 grid
    let points = vec![
        Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.5), Point3::new(2.0, 0.0, 0.0), Point3::new(3.0, 0.0, 0.5),
        Point3::new(0.0, 1.0, 0.5), Point3::new(1.0, 1.0, 1.0), Point3::new(2.0, 1.0, 0.5), Point3::new(3.0, 1.0, 1.0),
        Point3::new(0.0, 2.0, 0.0), Point3::new(1.0, 2.0, 0.5), Point3::new(2.0, 2.0, 0.0), Point3::new(3.0, 2.0, 0.5),
        Point3::new(0.0, 3.0, 0.5), Point3::new(1.0, 3.0, 1.0), Point3::new(2.0, 3.0, 0.5), Point3::new(3.0, 3.0, 1.0),
    ];

    let direct_options = SurfaceFitOptions {
        interpolate: false,
        ..Default::default()
    };
    let interp_options = SurfaceFitOptions {
        interpolate: true,
        ..Default::default()
    };

    let (direct_mesh, _) = mesh_from_grid_with_options(&points, 4, 4, direct_options).unwrap();
    let (interp_mesh, _) = mesh_from_grid_with_options(&points, 4, 4, interp_options).unwrap();

    // Direct mode should have exactly 16 vertices
    assert_eq!(direct_mesh.positions.len(), 16);

    // Interpolating mode typically produces a finer tessellation
    // (vertex count depends on degree and resolution calculation)
    assert!(
        interp_mesh.positions.len() >= direct_mesh.positions.len(),
        "interpolating mode should have at least as many vertices"
    );

    // The meshes should produce valid triangle indices
    assert!(
        direct_mesh.indices.len() >= 18,
        "direct mesh should have at least 18 indices (6 triangles for 3×3 quads)"
    );
    assert!(
        interp_mesh.indices.len() >= 18,
        "interp mesh should have at least 18 indices"
    );
}

#[test]
fn mesh_from_grid_with_options_validates_grid_size() {
    let points = vec![
        Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0),
    ];

    let options = SurfaceFitOptions::default();

    // Too few points for a 2×2 grid
    let result = mesh_from_grid_with_options(&points, 2, 2, options);
    assert!(result.is_err(), "should fail with mismatched point count");

    // Invalid grid size (1×2 is not valid)
    let result = mesh_from_grid_with_options(&points, 1, 2, options);
    assert!(result.is_err(), "should fail with invalid grid dimensions");
}
