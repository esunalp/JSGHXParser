use crate::geom::{Point3, Tolerance, TrimLoop, TrimRegion, UvPoint};
use crate::geom::mesh::{fix_triangle_winding_consistency, weld_mesh_vertices};
use crate::geom::triangulation::{triangulate_grid, triangulate_trim_region};

#[test]
fn triangulate_grid_counts_and_bounds() {
    let u = 5;
    let v = 4;
    let indices = triangulate_grid(u, v);

    let expected_triangles = (u - 1) * (v - 1) * 2;
    assert_eq!(indices.len(), expected_triangles * 3);

    let vertex_count = u * v;
    assert!(indices.iter().all(|i| (*i as usize) < vertex_count));
}

#[test]
fn triangulate_trim_region_with_hole_stays_inside_region() {
    let tol = Tolerance::new(1e-9);
    let outer = TrimLoop::new(
        vec![
            UvPoint::new(0.0, 0.0),
            UvPoint::new(2.0, 0.0),
            UvPoint::new(2.0, 2.0),
            UvPoint::new(0.0, 2.0),
        ],
        tol,
    )
    .unwrap();

    let hole = TrimLoop::new(
        vec![
            UvPoint::new(0.75, 0.75),
            UvPoint::new(0.75, 1.25),
            UvPoint::new(1.25, 1.25),
            UvPoint::new(1.25, 0.75),
        ],
        tol,
    )
    .unwrap();

    let region = TrimRegion::from_loops(vec![outer, hole], tol).unwrap();
    let result = triangulate_trim_region(&region, tol).unwrap();

    assert!(result.indices.len() >= 6);
    assert!(result
        .indices
        .iter()
        .all(|i| (*i as usize) < result.vertices.len()));

    let mut total_area = 0.0;
    for tri in result.indices.chunks_exact(3) {
        let a = result.vertices[tri[0] as usize];
        let b = result.vertices[tri[1] as usize];
        let c = result.vertices[tri[2] as usize];

        let centroid = UvPoint::new((a.u + b.u + c.u) / 3.0, (a.v + b.v + c.v) / 3.0);
        assert!(region.contains(centroid, tol));

        let area2 = (b.u - a.u) * (c.v - a.v) - (b.v - a.v) * (c.u - a.u);
        total_area += 0.5 * area2.abs();
    }

    let expected = region.outer.signed_area().abs()
        - region.holes.iter().map(|h| h.signed_area().abs()).sum::<f64>();
    assert!((total_area - expected).abs() <= 1e-9);
}

#[test]
fn triangulate_trim_region_with_multiple_holes_matches_expected_area() {
    let tol = Tolerance::new(1e-9);
    let outer = TrimLoop::new(
        vec![
            UvPoint::new(0.0, 0.0),
            UvPoint::new(4.0, 0.0),
            UvPoint::new(4.0, 4.0),
            UvPoint::new(0.0, 4.0),
        ],
        tol,
    )
    .unwrap();

    let hole_a = TrimLoop::new(
        vec![
            UvPoint::new(0.5, 0.5),
            UvPoint::new(1.5, 0.5),
            UvPoint::new(1.5, 1.5),
            UvPoint::new(0.5, 1.5),
        ],
        tol,
    )
    .unwrap();

    let hole_b = TrimLoop::new(
        vec![
            UvPoint::new(2.5, 2.5),
            UvPoint::new(3.5, 2.5),
            UvPoint::new(3.5, 3.5),
            UvPoint::new(2.5, 3.5),
        ],
        tol,
    )
    .unwrap();

    let region = TrimRegion::from_loops(vec![outer, hole_a, hole_b], tol).unwrap();
    let result = triangulate_trim_region(&region, tol).unwrap();

    let mut total_area = 0.0;
    for tri in result.indices.chunks_exact(3) {
        let a = result.vertices[tri[0] as usize];
        let b = result.vertices[tri[1] as usize];
        let c = result.vertices[tri[2] as usize];

        let centroid = UvPoint::new((a.u + b.u + c.u) / 3.0, (a.v + b.v + c.v) / 3.0);
        assert!(region.contains(centroid, tol));

        let area2 = (b.u - a.u) * (c.v - a.v) - (b.v - a.v) * (c.u - a.u);
        total_area += 0.5 * area2.abs();
    }

    let expected = region.outer.signed_area().abs()
        - region.holes.iter().map(|h| h.signed_area().abs()).sum::<f64>();
    assert!((total_area - expected).abs() <= 1e-9);
}

#[test]
fn triangulate_concave_outer_with_hole_matches_expected_area() {
    let tol = Tolerance::new(1e-9);
    let outer = TrimLoop::new(
        vec![
            UvPoint::new(0.0, 0.0),
            UvPoint::new(4.0, 0.0),
            UvPoint::new(4.0, 4.0),
            UvPoint::new(0.0, 4.0),
            UvPoint::new(2.0, 2.0),
        ],
        tol,
    )
    .unwrap();

    let hole = TrimLoop::new(
        vec![
            UvPoint::new(2.7, 0.7),
            UvPoint::new(3.3, 0.7),
            UvPoint::new(3.3, 1.3),
            UvPoint::new(2.7, 1.3),
        ],
        tol,
    )
    .unwrap();

    let region = TrimRegion::from_loops(vec![outer, hole], tol).unwrap();
    let result = triangulate_trim_region(&region, tol).unwrap();

    let mut total_area = 0.0;
    for tri in result.indices.chunks_exact(3) {
        let a = result.vertices[tri[0] as usize];
        let b = result.vertices[tri[1] as usize];
        let c = result.vertices[tri[2] as usize];

        let centroid = UvPoint::new((a.u + b.u + c.u) / 3.0, (a.v + b.v + c.v) / 3.0);
        assert!(region.contains(centroid, tol));

        let area2 = (b.u - a.u) * (c.v - a.v) - (b.v - a.v) * (c.u - a.u);
        total_area += 0.5 * area2.abs();
    }

    let expected = region.outer.signed_area().abs()
        - region.holes.iter().map(|h| h.signed_area().abs()).sum::<f64>();
    assert!((total_area - expected).abs() <= 1e-9);
}

#[test]
fn weld_mesh_vertices_merges_duplicates_stably() {
    let points = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
    ];
    let uvs = vec![[0.0, 0.0], [0.1, 0.1], [1.0, 0.0]];
    let indices = vec![0u32, 2, 1];

    let tol = Tolerance::new(1e-6);
    let (out_points, out_uvs, out_indices, welded) =
        weld_mesh_vertices(points, Some(&uvs), indices, tol);

    assert_eq!(welded, 1);
    assert_eq!(out_points.len(), 2);
    assert_eq!(out_indices, vec![0u32, 1, 0]);

    let out_uvs = out_uvs.unwrap();
    assert_eq!(out_uvs.len(), 2);
    assert_eq!(out_uvs[0], [0.0, 0.0]);
}

#[test]
fn fix_triangle_winding_consistency_flips_neighbor_triangle() {
    let mut indices = vec![0u32, 1, 2, 0, 3, 2];
    let flipped = fix_triangle_winding_consistency(&mut indices);
    assert_eq!(flipped, 1);

    let tri0 = [indices[0], indices[1], indices[2]];
    let tri1 = [indices[3], indices[4], indices[5]];

    fn edge_dir(tri: [u32; 3], a: u32, b: u32) -> Option<(u32, u32)> {
        let edges = [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])];
        edges
            .into_iter()
            .find(|(u, v)| (*u == a && *v == b) || (*u == b && *v == a))
            .map(|(u, v)| (u, v))
    }

    let e0 = edge_dir(tri0, 0, 2).unwrap();
    let e1 = edge_dir(tri1, 0, 2).unwrap();
    assert_eq!(e0.0, e1.1);
    assert_eq!(e0.1, e1.0);
}
