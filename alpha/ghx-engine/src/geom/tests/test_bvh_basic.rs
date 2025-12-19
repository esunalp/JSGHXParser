use crate::geom::{BBox, Point3, Vec3};
use crate::geom::bvh::Bvh;

fn bbox_distance_squared_to_point(bbox: BBox, point: Point3) -> f64 {
    let dx = if point.x < bbox.min.x {
        bbox.min.x - point.x
    } else if point.x > bbox.max.x {
        point.x - bbox.max.x
    } else {
        0.0
    };
    let dy = if point.y < bbox.min.y {
        bbox.min.y - point.y
    } else if point.y > bbox.max.y {
        point.y - bbox.max.y
    } else {
        0.0
    };
    let dz = if point.z < bbox.min.z {
        bbox.min.z - point.z
    } else if point.z > bbox.max.z {
        point.z - bbox.max.z
    } else {
        0.0
    };
    dx * dx + dy * dy + dz * dz
}

#[test]
fn bvh_query_bbox_returns_intersecting_primitives() {
    let bboxes = vec![
        BBox::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 1.0),
        ),
        BBox::new(
            Point3::new(2.0, 2.0, 2.0),
            Point3::new(3.0, 3.0, 3.0),
        ),
        BBox::new(
            Point3::new(0.5, 0.5, 0.5),
            Point3::new(2.5, 2.5, 2.5),
        ),
    ];

    let bvh = Bvh::build_with_leaf_size(&bboxes, 1).expect("bvh build");
    let query = BBox::new(
        Point3::new(0.75, 0.75, 0.75),
        Point3::new(0.8, 0.8, 0.8),
    );

    let mut hits = Vec::new();
    bvh.query_bbox(query, |idx| {
        hits.push(idx);
        true
    });
    hits.sort_unstable();
    hits.dedup();

    assert_eq!(hits, vec![0, 2]);
}

#[test]
fn bvh_query_ray_returns_intersecting_primitives() {
    let bboxes = vec![
        BBox::new(
            Point3::new(0.0, -1.0, -1.0),
            Point3::new(1.0, 1.0, 1.0),
        ),
        BBox::new(
            Point3::new(0.0, 2.0, -1.0),
            Point3::new(1.0, 3.0, 1.0),
        ),
        BBox::new(
            Point3::new(5.0, -0.5, -0.5),
            Point3::new(6.0, 0.5, 0.5),
        ),
    ];

    let bvh = Bvh::build_with_leaf_size(&bboxes, 1).expect("bvh build");

    let origin = Point3::new(-10.0, 0.0, 0.0);
    let dir = Vec3::new(1.0, 0.0, 0.0);

    let mut hits = Vec::new();
    bvh.query_ray(origin, dir, 0.0, f64::INFINITY, |idx| {
        hits.push(idx);
        true
    });
    hits.sort_unstable();
    hits.dedup();

    assert_eq!(hits, vec![0, 2]);
}

#[test]
fn bvh_nearest_finds_closest_primitive() {
    let bboxes = vec![
        BBox::new(
            Point3::new(10.0, 0.0, 0.0),
            Point3::new(11.0, 1.0, 1.0),
        ),
        BBox::new(
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(2.0, 1.0, 1.0),
        ),
        BBox::new(
            Point3::new(-5.0, 0.0, 0.0),
            Point3::new(-4.0, 1.0, 1.0),
        ),
    ];

    let bvh = Bvh::build_with_leaf_size(&bboxes, 1).expect("bvh build");
    let point = Point3::new(0.0, 0.5, 0.5);

    let (idx, dist2) = bvh
        .nearest(point, f64::INFINITY, |prim_idx| {
            Some(bbox_distance_squared_to_point(bboxes[prim_idx], point))
        })
        .expect("nearest hit");

    assert_eq!(idx, 1);
    assert!((dist2 - 1.0).abs() < 1e-12);
}

