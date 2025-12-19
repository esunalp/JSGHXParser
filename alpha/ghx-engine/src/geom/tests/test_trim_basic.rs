use crate::geom::{Tolerance, TrimDiagnostics, TrimError, TrimLoop, TrimRegion, UvDomain, UvPoint, copy_trim_bounds, copy_trim_loops, untrim_bounds, untrim_to_domain};

#[test]
fn trim_region_normalizes_orientation_and_contains_points() {
    let tol = Tolerance::new(1e-9);

    let outer_cw = TrimLoop::new(
        vec![
            UvPoint::new(0.0, 0.0),
            UvPoint::new(0.0, 1.0),
            UvPoint::new(1.0, 1.0),
            UvPoint::new(1.0, 0.0),
        ],
        tol,
    )
    .unwrap();

    let hole_ccw = TrimLoop::new(
        vec![
            UvPoint::new(0.4, 0.4),
            UvPoint::new(0.6, 0.4),
            UvPoint::new(0.6, 0.6),
            UvPoint::new(0.4, 0.6),
        ],
        tol,
    )
    .unwrap();

    let region = TrimRegion::from_loops(vec![outer_cw, hole_ccw], tol).unwrap();
    assert!(region.outer.is_ccw());
    assert_eq!(region.holes.len(), 1);
    assert!(!region.holes[0].is_ccw());

    assert!(region.contains(UvPoint::new(0.2, 0.2), tol));
    assert!(!region.contains(UvPoint::new(0.5, 0.5), tol));
    assert!(!region.contains(UvPoint::new(1.2, 0.2), tol));
}

#[test]
fn trim_loop_unwraps_u_across_periodic_seam() {
    let tol = Tolerance::new(1e-9);

    let loop_ = TrimLoop::new(
        vec![
            UvPoint::new(0.9, 0.0),
            UvPoint::new(0.9, 1.0),
            UvPoint::new(0.1, 1.0),
            UvPoint::new(0.1, 0.0),
        ],
        tol,
    )
    .unwrap();

    let unwrapped = loop_.unwrapped_u_periodic(0.0, 1.0, tol);
    let pts = unwrapped.points();
    assert_eq!(pts.len(), 4);
    for i in 0..pts.len() {
        let a = pts[i].u;
        let b = pts[(i + 1) % pts.len()].u;
        assert!((b - a).abs() <= 0.5 + tol.eps);
    }
}

#[test]
fn trim_loop_rejects_self_intersection() {
    let tol = Tolerance::new(1e-9);
    let loop_ = TrimLoop::new(
        vec![
            UvPoint::new(0.0, 0.0),
            UvPoint::new(1.0, 1.0),
            UvPoint::new(0.0, 1.0),
            UvPoint::new(1.0, 0.0),
        ],
        tol,
    );
    assert!(matches!(loop_, Err(TrimError::SelfIntersection)));
}

#[test]
fn trim_region_rejects_hole_intersecting_outer_loop() {
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
            UvPoint::new(0.5, 0.5),
            UvPoint::new(1.5, 0.5),
            UvPoint::new(1.5, 2.5),
            UvPoint::new(0.5, 2.5),
        ],
        tol,
    )
    .unwrap();

    let err = TrimRegion::from_loops(vec![outer, hole], tol).unwrap_err();
    assert!(matches!(err, TrimError::HoleIntersectsOuter));
}

#[test]
fn trim_region_rejects_nested_holes() {
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

    let hole_outer = TrimLoop::new(
        vec![
            UvPoint::new(1.0, 1.0),
            UvPoint::new(3.0, 1.0),
            UvPoint::new(3.0, 3.0),
            UvPoint::new(1.0, 3.0),
        ],
        tol,
    )
    .unwrap();

    let hole_inner = TrimLoop::new(
        vec![
            UvPoint::new(1.5, 1.5),
            UvPoint::new(2.5, 1.5),
            UvPoint::new(2.5, 2.5),
            UvPoint::new(1.5, 2.5),
        ],
        tol,
    )
    .unwrap();

    let err = TrimRegion::from_loops(vec![outer, hole_outer, hole_inner], tol).unwrap_err();
    assert!(matches!(err, TrimError::NestedHoles));
}

#[test]
fn trim_region_rejects_intersecting_holes() {
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
            UvPoint::new(1.0, 1.0),
            UvPoint::new(2.5, 1.0),
            UvPoint::new(2.5, 2.5),
            UvPoint::new(1.0, 2.5),
        ],
        tol,
    )
    .unwrap();

    let hole_b = TrimLoop::new(
        vec![
            UvPoint::new(2.0, 2.0),
            UvPoint::new(3.0, 2.0),
            UvPoint::new(3.0, 3.0),
            UvPoint::new(2.0, 3.0),
        ],
        tol,
    )
    .unwrap();

    let err = TrimRegion::from_loops(vec![outer, hole_a, hole_b], tol).unwrap_err();
    assert!(matches!(err, TrimError::HolesIntersect));
}

#[test]
fn copy_trim_bounds_returns_intersection_when_overlap_exists() {
    let source_min = [0.0, 0.0, 0.0];
    let source_max = [2.0, 2.0, 2.0];
    let target_min = [1.0, -1.0, 1.0];
    let target_max = [3.0, 1.0, 4.0];

    let (min, max) = copy_trim_bounds(source_min, source_max, target_min, target_max);
    assert_eq!(min, [1.0, 0.0, 1.0]);
    assert_eq!(max, [2.0, 1.0, 2.0]);
}

#[test]
fn copy_trim_bounds_falls_back_to_target_when_no_overlap() {
    let source_min = [0.0, 0.0, 0.0];
    let source_max = [1.0, 1.0, 1.0];
    let target_min = [2.0, 2.0, 2.0];
    let target_max = [3.0, 3.0, 3.0];

    let (min, max) = copy_trim_bounds(source_min, source_max, target_min, target_max);
    assert_eq!(min, target_min);
    assert_eq!(max, target_max);
}

#[test]
fn untrim_bounds_round_trips() {
    let min = [1.0, 2.0, 3.0];
    let max = [4.0, 5.0, 6.0];
    let (out_min, out_max) = untrim_bounds(min, max);
    assert_eq!(out_min, min);
    assert_eq!(out_max, max);
}

// ============================================================================
// New tests for TrimError and TrimDiagnostics
// ============================================================================

#[test]
fn trim_loop_new_with_diagnostics_reports_cleaned_points() {
    let tol = Tolerance::new(1e-9);

    // Points with a duplicate and a closing point
    let points = vec![
        UvPoint::new(0.0, 0.0),
        UvPoint::new(0.5, 0.0),
        UvPoint::new(0.5, 0.0), // duplicate
        UvPoint::new(1.0, 0.0),
        UvPoint::new(1.0, 1.0),
        UvPoint::new(0.0, 1.0),
        UvPoint::new(0.0, 0.0), // closing point
    ];

    let (loop_, diag) = TrimLoop::new_with_diagnostics(points, tol).unwrap();
    assert_eq!(loop_.len(), 5); // 7 - 1 closing - 1 duplicate
    assert_eq!(diag.closing_points_removed, 1);
    assert_eq!(diag.duplicate_points_removed, 1);
    assert!(diag.had_adjustments());
}

#[test]
fn trim_loop_rejects_insufficient_points() {
    let tol = Tolerance::new(1e-9);
    let result = TrimLoop::new(
        vec![UvPoint::new(0.0, 0.0), UvPoint::new(1.0, 0.0)],
        tol,
    );
    assert!(matches!(result, Err(TrimError::InsufficientPoints { count: 2 })));
}

#[test]
fn trim_loop_rejects_non_finite_points() {
    let tol = Tolerance::new(1e-9);
    let result = TrimLoop::new(
        vec![
            UvPoint::new(0.0, 0.0),
            UvPoint::new(f64::NAN, 0.5),
            UvPoint::new(1.0, 1.0),
        ],
        tol,
    );
    assert!(matches!(result, Err(TrimError::NonFinitePoints)));
}

#[test]
fn trim_loop_bounds_are_correct() {
    let tol = Tolerance::new(1e-9);
    let loop_ = TrimLoop::new(
        vec![
            UvPoint::new(0.2, 0.3),
            UvPoint::new(0.8, 0.3),
            UvPoint::new(0.8, 0.9),
            UvPoint::new(0.2, 0.9),
        ],
        tol,
    )
    .unwrap();

    let bounds = loop_.bounds();
    assert!((bounds.u_min - 0.2).abs() < 1e-9);
    assert!((bounds.u_max - 0.8).abs() < 1e-9);
    assert!((bounds.v_min - 0.3).abs() < 1e-9);
    assert!((bounds.v_max - 0.9).abs() < 1e-9);
}

#[test]
fn trim_loop_domain_validation() {
    let tol = Tolerance::new(1e-9);
    let loop_ = TrimLoop::new(
        vec![
            UvPoint::new(0.2, 0.2),
            UvPoint::new(0.8, 0.2),
            UvPoint::new(0.8, 0.8),
            UvPoint::new(0.2, 0.8),
        ],
        tol,
    )
    .unwrap();

    // Should be within unit domain
    let unit = UvDomain::unit();
    assert!(loop_.validate_domain(&unit, tol).is_ok());

    // Should fail for smaller domain
    let small = UvDomain::new(0.3, 0.7, 0.3, 0.7);
    let err = loop_.validate_domain(&small, tol).unwrap_err();
    assert!(matches!(err, TrimError::OutsideDomain { .. }));
}

#[test]
fn trim_loop_map_domain() {
    let tol = Tolerance::new(1e-9);
    let loop_ = TrimLoop::new(
        vec![
            UvPoint::new(0.0, 0.0),
            UvPoint::new(1.0, 0.0),
            UvPoint::new(1.0, 1.0),
            UvPoint::new(0.0, 1.0),
        ],
        tol,
    )
    .unwrap();

    let from = UvDomain::unit();
    let to = UvDomain::new(2.0, 4.0, 3.0, 6.0);
    let mapped = loop_.map_domain(&from, &to);

    let pts = mapped.points();
    assert!((pts[0].u - 2.0).abs() < 1e-9);
    assert!((pts[0].v - 3.0).abs() < 1e-9);
    assert!((pts[1].u - 4.0).abs() < 1e-9);
    assert!((pts[2].v - 6.0).abs() < 1e-9);
}

#[test]
fn trim_region_from_domain() {
    let domain = UvDomain::new(0.5, 1.5, 0.5, 2.5);
    let region = TrimRegion::from_domain(&domain);

    assert!(region.outer.is_ccw());
    assert!(region.holes.is_empty());
    assert_eq!(region.outer.len(), 4);
}

#[test]
fn trim_region_diagnostics_reports_flips() {
    let tol = Tolerance::new(1e-9);

    // CW outer (will be flipped to CCW)
    let outer_cw = TrimLoop::new(
        vec![
            UvPoint::new(0.0, 0.0),
            UvPoint::new(0.0, 1.0),
            UvPoint::new(1.0, 1.0),
            UvPoint::new(1.0, 0.0),
        ],
        tol,
    )
    .unwrap();

    // CCW hole (will be flipped to CW)
    let hole_ccw = TrimLoop::new(
        vec![
            UvPoint::new(0.3, 0.3),
            UvPoint::new(0.7, 0.3),
            UvPoint::new(0.7, 0.7),
            UvPoint::new(0.3, 0.7),
        ],
        tol,
    )
    .unwrap();

    let (region, diag) = TrimRegion::from_loops_with_diagnostics(vec![outer_cw, hole_ccw], tol).unwrap();
    assert!(region.outer.is_ccw());
    assert!(!region.holes[0].is_ccw());
    assert_eq!(diag.orientation_flips, 2); // Both were flipped
    assert_eq!(diag.loop_count, 2);
    assert_eq!(diag.hole_count, 1);
}

#[test]
fn copy_trim_loops_maps_between_domains() {
    let tol = Tolerance::new(1e-9);

    let source_domain = UvDomain::unit();
    let target_domain = UvDomain::new(0.0, 2.0, 0.0, 2.0);

    let loop_ = TrimLoop::new(
        vec![
            UvPoint::new(0.25, 0.25),
            UvPoint::new(0.75, 0.25),
            UvPoint::new(0.75, 0.75),
            UvPoint::new(0.25, 0.75),
        ],
        tol,
    )
    .unwrap();

    let (mapped, diag) = copy_trim_loops(&[loop_], &source_domain, &target_domain, tol).unwrap();

    assert_eq!(mapped.len(), 1);
    assert_eq!(diag.loop_count, 1);

    let pts = mapped[0].points();
    assert!((pts[0].u - 0.5).abs() < 1e-9);
    assert!((pts[0].v - 0.5).abs() < 1e-9);
    assert!((pts[1].u - 1.5).abs() < 1e-9);
}

#[test]
fn untrim_to_domain_returns_full_region() {
    let domain = UvDomain::new(1.0, 3.0, 2.0, 5.0);
    let (region, diag) = untrim_to_domain(&domain);

    assert!(region.holes.is_empty());
    assert_eq!(region.loop_count(), 1);
    assert_eq!(diag.loop_count, 1);

    let bounds = region.bounds();
    assert!((bounds.u_min - 1.0).abs() < 1e-9);
    assert!((bounds.u_max - 3.0).abs() < 1e-9);
    assert!((bounds.v_min - 2.0).abs() < 1e-9);
    assert!((bounds.v_max - 5.0).abs() < 1e-9);
}

#[test]
fn uv_point_distance() {
    let a = UvPoint::new(0.0, 0.0);
    let b = UvPoint::new(3.0, 4.0);
    assert!((a.distance(b) - 5.0).abs() < 1e-9);
}

#[test]
fn uv_domain_contains() {
    let domain = UvDomain::new(0.0, 1.0, 0.0, 1.0);
    let tol = Tolerance::new(1e-9);

    assert!(domain.contains(UvPoint::new(0.5, 0.5), tol));
    assert!(domain.contains(UvPoint::new(0.0, 0.0), tol));
    assert!(domain.contains(UvPoint::new(1.0, 1.0), tol));
    assert!(!domain.contains(UvPoint::new(1.5, 0.5), tol));
    assert!(!domain.contains(UvPoint::new(-0.1, 0.5), tol));
}

#[test]
fn trim_loop_ensure_orientation() {
    let tol = Tolerance::new(1e-9);

    // CCW loop
    let ccw_loop = TrimLoop::new(
        vec![
            UvPoint::new(0.0, 0.0),
            UvPoint::new(1.0, 0.0),
            UvPoint::new(1.0, 1.0),
            UvPoint::new(0.0, 1.0),
        ],
        tol,
    )
    .unwrap();

    assert!(ccw_loop.is_ccw());
    assert!(ccw_loop.ensure_ccw().is_ccw());
    assert!(ccw_loop.ensure_cw().is_cw());

    // CW loop
    let cw_loop = ccw_loop.reversed();
    assert!(cw_loop.is_cw());
    assert!(cw_loop.ensure_cw().is_cw());
    assert!(cw_loop.ensure_ccw().is_ccw());
}

#[test]
fn trim_loop_translated() {
    let tol = Tolerance::new(1e-9);

    let loop_ = TrimLoop::new(
        vec![
            UvPoint::new(0.0, 0.0),
            UvPoint::new(1.0, 0.0),
            UvPoint::new(1.0, 1.0),
            UvPoint::new(0.0, 1.0),
        ],
        tol,
    )
    .unwrap();

    let translated = loop_.translated(2.0, 3.0);
    let pts = translated.points();
    assert!((pts[0].u - 2.0).abs() < 1e-9);
    assert!((pts[0].v - 3.0).abs() < 1e-9);
    assert!((pts[2].u - 3.0).abs() < 1e-9);
    assert!((pts[2].v - 4.0).abs() < 1e-9);
}

#[test]
fn trim_error_display() {
    let err = TrimError::SelfIntersection;
    assert_eq!(err.to_string(), "trim loop self-intersects");

    let err = TrimError::InsufficientPoints { count: 2 };
    assert!(err.to_string().contains("at least 3 points"));
    assert!(err.to_string().contains("2"));
}

#[test]
fn trim_diagnostics_merge() {
    let mut diag1 = TrimDiagnostics::new();
    diag1.loop_count = 2;
    diag1.orientation_flips = 1;

    let mut diag2 = TrimDiagnostics::new();
    diag2.loop_count = 3;
    diag2.orientation_flips = 2;
    diag2.warnings.push("test warning".to_string());

    diag1.merge(&diag2);
    assert_eq!(diag1.loop_count, 5);
    assert_eq!(diag1.orientation_flips, 3);
    assert_eq!(diag1.warnings.len(), 1);
}
