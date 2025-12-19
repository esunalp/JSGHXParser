use crate::geom::{
    PipeCaps, PipeError, PipeOptions, Point3, Tolerance,
    pipe_polyline_with_tolerance, pipe_variable_polyline_with_tolerance,
};

#[test]
fn pipe_straight_caps_is_watertight() {
    let rail = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, 2.0)];

    let tol = Tolerance::default_geom();
    let options = PipeOptions { radial_segments: 16 };

    let (mesh, diag) = pipe_polyline_with_tolerance(&rail, 0.5, PipeCaps::BOTH, options, tol)
        .expect("pipe should succeed");

    assert!(mesh.triangle_count() > 0);
    assert_eq!(diag.open_edge_count, 0, "expected watertight mesh");
    assert_eq!(diag.non_manifold_edge_count, 0, "expected manifold mesh");
}

#[test]
fn pipe_closed_rail_no_caps() {
    // Closed square loop in XY.
    let rail = vec![
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(-1.0, 0.0, 0.0),
        Point3::new(0.0, -1.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
    ];

    let tol = Tolerance::default_geom();
    let options = PipeOptions { radial_segments: 12 };

    let (mesh, diag) = pipe_polyline_with_tolerance(&rail, 0.2, PipeCaps::NONE, options, tol)
        .expect("closed rail pipe should succeed");

    assert!(mesh.triangle_count() > 0);
    // Closed rail should generally be watertight.
    assert_eq!(diag.non_manifold_edge_count, 0);
}

#[test]
fn pipe_closed_rail_with_caps_fails() {
    let rail = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, 0.0),
    ];

    let tol = Tolerance::default_geom();
    let result = pipe_polyline_with_tolerance(&rail, 0.2, PipeCaps::BOTH, PipeOptions::default(), tol);

    assert!(matches!(result, Err(PipeError::CapsNotAllowedForClosedRail)));
}

#[test]
fn pipe_variable_interpolates() {
    let rail = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(0.0, 0.0, 2.0),
    ];

    // Radius grows from 0.2 to 0.6 along the rail.
    let params = vec![0.0, 1.0];
    let radii = vec![0.2, 0.6];

    let tol = Tolerance::default_geom();
    let options = PipeOptions { radial_segments: 10 };

    let (mesh, diag) = pipe_variable_polyline_with_tolerance(&rail, &params, &radii, PipeCaps::BOTH, options, tol)
        .expect("pipe variable should succeed");

    assert!(mesh.triangle_count() > 0);
    assert_eq!(diag.open_edge_count, 0);
}
