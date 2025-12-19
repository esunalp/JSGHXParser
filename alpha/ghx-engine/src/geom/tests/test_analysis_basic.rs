use crate::geom::{
    LegacyBrepData, PlaneSurface, Point3, Tolerance, Vec3, closed_edges, edges_by_length,
    edges_from_directions, edges_from_faces, edges_from_points, surface_frames,
};

fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
    (a - b).abs() <= eps
}

#[test]
fn surface_frames_plane_has_expected_axes() {
    let plane = PlaneSurface::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(10.0, 0.0, 0.0),
        Vec3::new(0.0, 5.0, 0.0),
    );
    let tol = Tolerance::default_geom();
    let result = surface_frames(&plane, 2, 1, tol);

    assert_eq!(result.u_count, 3);
    assert_eq!(result.v_count, 2);
    assert_eq!(result.frames.len(), 6);
    assert_eq!(result.parameters.len(), 6);

    let frame = result.frames[0];
    assert!(approx_eq(frame.origin.x, 0.0, 1e-9));
    assert!(approx_eq(frame.origin.y, 0.0, 1e-9));
    assert!(approx_eq(frame.origin.z, 0.0, 1e-9));

    assert!(approx_eq(frame.x_axis.x, 1.0, 1e-9));
    assert!(approx_eq(frame.x_axis.y, 0.0, 1e-9));
    assert!(approx_eq(frame.x_axis.z, 0.0, 1e-9));

    assert!(approx_eq(frame.y_axis.x, 0.0, 1e-9));
    assert!(approx_eq(frame.y_axis.y, 1.0, 1e-9));
    assert!(approx_eq(frame.y_axis.z, 0.0, 1e-9));

    assert!(approx_eq(frame.z_axis.x, 0.0, 1e-9));
    assert!(approx_eq(frame.z_axis.y, 0.0, 1e-9));
    assert!(approx_eq(frame.z_axis.z, 1.0, 1e-9));
}

#[test]
fn legacy_brep_edge_selection_basic() {
    let vertices = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]];
    let faces = vec![vec![0, 1, 2, 3]];

    let tol = Tolerance::default_geom();
    let mut brep = LegacyBrepData::default();
    brep.extend_from_surface_buffers(&vertices, &faces, tol);

    assert_eq!(brep.faces.len(), 1);
    assert_eq!(brep.edges.len(), 4);

    let closed = closed_edges(&brep);
    assert!(closed.closed.is_empty());
    assert_eq!(closed.open.len(), 4);

    let dir = vec![Vec3::new(1.0, 0.0, 0.0)];
    let by_dir = edges_from_directions(&brep, &dir, false, 0.01, tol);
    assert_eq!(by_dir.edges.len(), 1);
    assert_eq!(by_dir.map, vec![0]);

    let by_dir_reflex = edges_from_directions(&brep, &dir, true, 0.01, tol);
    assert_eq!(by_dir_reflex.edges.len(), 2);
    assert_eq!(by_dir_reflex.map, vec![0, 0]);

    let points = vec![Point3::new(0.0, 0.0, 0.0)];
    let by_points = edges_from_points(&brep, &points, 1, 1e-9);
    assert_eq!(by_points.edges.len(), 2);
    assert_eq!(by_points.map, vec![2]);

    let by_faces = edges_from_faces(&brep, &[], 1e-3);
    assert_eq!(by_faces.len(), 4);

    let by_length = edges_by_length(&brep, 0.9, 1.1);
    assert_eq!(by_length.len(), 4);
}

