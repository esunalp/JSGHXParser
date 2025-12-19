use crate::geom::{Point3, Tolerance, Vec3, fragment_patch_meshes_with_tolerance, patch_mesh_with_tolerance};

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
