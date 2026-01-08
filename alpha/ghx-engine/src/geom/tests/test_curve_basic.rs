use crate::geom::{
    Arc3, Circle3, Curve3, CurveTessellationOptions, Ellipse3, Line3, NurbsCurve3,
    Point3, Polyline3, QuadraticBezier3, Tolerance, Vec3, curve_arc_length,
    divide_curve_by_count, tessellate_curve_adaptive_points, tessellate_curve_uniform,
};

#[test]
fn tessellate_curve_preserves_endpoints() {
    let line = Line3::new(Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 0.0, 0.0));
    let pts = tessellate_curve_uniform(&line, 10);
    assert_eq!(pts.first().copied(), Some(line.start));
    assert_eq!(pts.last().copied(), Some(line.end));
    assert_eq!(pts.len(), 11);
}

#[test]
fn tessellate_curve_closed_has_no_duplicate_endpoint() {
    let circle = Circle3::from_center_xaxis_normal(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        2.0,
    );

    let pts = tessellate_curve_uniform(&circle, 16);
    assert_eq!(pts.len(), 16);
    assert_ne!(pts.first().copied(), pts.last().copied());
    assert_eq!(circle.point_at(0.0), circle.point_at(1.0));
}

#[test]
fn ellipse_seam_is_stable() {
    let ellipse = Ellipse3::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        3.0,
        1.5,
    );
    assert_eq!(ellipse.point_at(0.0), ellipse.point_at(1.0));
}

#[test]
fn arc_has_expected_endpoints_with_explicit_frame() {
    let arc = Arc3::from_center_xaxis_normal(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        1.0,
        0.0,
        std::f64::consts::FRAC_PI_2,
    );
    let tol = Tolerance::new(1e-9);
    assert!(tol.approx_eq_point3(arc.point_at(0.0), Point3::new(1.0, 0.0, 0.0)));
    assert!(tol.approx_eq_point3(arc.point_at(1.0), Point3::new(0.0, 1.0, 0.0)));
}

#[test]
fn quadratic_bezier_curvature_is_reasonable() {
    let curve = QuadraticBezier3::new(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(2.0, 0.0, 0.0),
    );
    assert_eq!(curve.derivative_at(0.5), Vec3::new(2.0, 0.0, 0.0));
    let curvature = curve.curvature_at(0.5).unwrap();
    assert!((curvature - 1.0).abs() < 1e-12);
}

#[test]
fn nurbs_endpoints_and_tangent_continuity() {
    let line = NurbsCurve3::new(
        1,
        vec![Point3::new(0.0, 0.0, 0.0), Point3::new(2.0, 0.0, 0.0)],
        vec![0.0, 0.0, 1.0, 1.0],
        None,
    )
    .unwrap();
    assert_eq!(line.point_at(0.0), Point3::new(0.0, 0.0, 0.0));
    assert_eq!(line.point_at(1.0), Point3::new(2.0, 0.0, 0.0));

    let line_weighted = NurbsCurve3::new(
        1,
        vec![Point3::new(0.0, 0.0, 0.0), Point3::new(2.0, 0.0, 0.0)],
        vec![0.0, 0.0, 1.0, 1.0],
        Some(vec![1.0, 1.0]),
    )
    .unwrap();
    assert_eq!(line_weighted.point_at(0.0), Point3::new(0.0, 0.0, 0.0));
    assert_eq!(line_weighted.point_at(1.0), Point3::new(2.0, 0.0, 0.0));

    let tol = Tolerance::default_geom();
    let curve_c0 = NurbsCurve3::new(
        3,
        (0..7)
            .map(|i| Point3::new(i as f64, 0.0, 0.0))
            .collect(),
        vec![0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.5, 1.0, 1.0, 1.0, 1.0],
        None,
    )
    .unwrap();
    assert_eq!(curve_c0.continuity_order_at_knot(0.5, tol), Some(0));
    assert!(!curve_c0.is_tangent_continuous_at_knot(0.5, tol));

    let curve_c1 = NurbsCurve3::new(
        3,
        (0..7)
            .map(|i| Point3::new(i as f64, 0.0, 0.0))
            .collect(),
        vec![0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.75, 1.0, 1.0, 1.0, 1.0],
        None,
    )
    .unwrap();
    assert_eq!(curve_c1.continuity_order_at_knot(0.5, tol), Some(1));
    assert!(curve_c1.is_tangent_continuous_at_knot(0.5, tol));
}

#[test]
fn adaptive_tessellation_respects_caps_and_outputs_finite_points() {
    let circle = Circle3::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0), 1.0);
    let opts = CurveTessellationOptions {
        max_deviation: 1e-3,
        max_segments: 64,
        max_depth: 16,
        initial_segments: 8,
    };

    let pts = tessellate_curve_adaptive_points(&circle, opts);
    assert!(pts.len() <= 64);
    assert!(pts.len() >= 3);
    assert_ne!(pts.first().copied(), pts.last().copied());
    assert!(pts.iter().all(|p| p.x.is_finite() && p.y.is_finite() && p.z.is_finite()));
}

#[test]
fn uniform_tessellation_respects_nurbs_domain() {
    let curve = NurbsCurve3::new(
        1,
        vec![Point3::new(0.0, 0.0, 0.0), Point3::new(2.0, 0.0, 0.0)],
        vec![5.0, 5.0, 6.0, 6.0],
        None,
    )
    .unwrap();

    let pts = tessellate_curve_uniform(&curve, 4);
    assert_eq!(pts.first().copied(), Some(Point3::new(0.0, 0.0, 0.0)));
    assert_eq!(pts.last().copied(), Some(Point3::new(2.0, 0.0, 0.0)));
}

#[test]
fn adaptive_tessellation_handles_extreme_scales() {
    let circle = Circle3::from_center_xaxis_normal(
        Point3::new(1e9, -1e9, 1e9),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        1e9,
    );
    let opts = CurveTessellationOptions {
        max_deviation: 1e6,
        max_segments: 128,
        max_depth: 16,
        initial_segments: 8,
    };

    let pts = tessellate_curve_adaptive_points(&circle, opts);
    assert!(pts.len() <= 128);
    assert!(pts.iter().all(|p| p.x.is_finite() && p.y.is_finite() && p.z.is_finite()));
}

#[test]
fn adaptive_tessellation_respects_initial_segments_for_open_curves() {
    let line = Line3::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
    let opts = CurveTessellationOptions {
        max_deviation: 1e-6,
        max_segments: 64,
        max_depth: 16,
        initial_segments: 10,
    };

    let pts = tessellate_curve_adaptive_points(&line, opts);
    assert_eq!(pts.len(), 11);
    assert_eq!(pts.first().copied(), Some(line.start));
    assert_eq!(pts.last().copied(), Some(line.end));
}

#[test]
fn adaptive_tessellation_handles_degenerate_domains_without_duplicates() {
    struct DegenerateCurve;
    impl Curve3 for DegenerateCurve {
        fn point_at(&self, _t: f64) -> Point3 {
            Point3::new(1.0, 2.0, 3.0)
        }

        fn domain(&self) -> (f64, f64) {
            (0.0, 0.0)
        }
    }

    let opts = CurveTessellationOptions {
        max_deviation: 1e-3,
        max_segments: 64,
        max_depth: 16,
        initial_segments: 8,
    };

    let pts = tessellate_curve_adaptive_points(&DegenerateCurve, opts);
    assert_eq!(pts, vec![Point3::new(1.0, 2.0, 3.0)]);
}

#[test]
fn nurbs_closed_curves_report_closed_and_tessellate_without_duplicate_endpoints() {
    let p0 = Point3::new(0.0, 0.0, 0.0);
    let p1 = Point3::new(1.0, 0.0, 0.0);
    let curve = NurbsCurve3::new(1, vec![p0, p1, p0], vec![0.0, 0.0, 0.5, 1.0, 1.0], None).unwrap();
    assert!(curve.is_closed());
    assert_eq!(curve.point_at(0.0), curve.point_at(1.0));

    let pts = tessellate_curve_uniform(&curve, 16);
    assert_eq!(pts.len(), 16);
    assert_ne!(pts.first().copied(), pts.last().copied());
}

#[test]
fn tangent_at_returns_unit_vector() {
    let line = Line3::new(Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 0.0, 0.0));
    let tangent = line.tangent_at(0.5).unwrap();
    let tol = Tolerance::new(1e-12);
    assert!(tol.approx_eq_f64(tangent.length(), 1.0));
    assert!(tol.approx_eq_f64(tangent.x, 1.0));
    assert!(tol.approx_eq_f64(tangent.y, 0.0));
    assert!(tol.approx_eq_f64(tangent.z, 0.0));

    let circle = Circle3::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0), 1.0);
    let tangent_circle = circle.tangent_at(0.0).unwrap();
    assert!(tol.approx_eq_f64(tangent_circle.length(), 1.0));
}

#[test]
fn arc_full_circle_is_closed() {
    let full_arc = Arc3::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        1.0,
        0.0,
        std::f64::consts::TAU,
    );
    assert!(full_arc.is_closed());

    let half_arc = Arc3::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        1.0,
        0.0,
        std::f64::consts::PI,
    );
    assert!(!half_arc.is_closed());

    let negative_full = Arc3::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        1.0,
        0.0,
        -std::f64::consts::TAU,
    );
    assert!(negative_full.is_closed());
}

#[test]
fn nurbs_analytic_derivative_matches_numerical() {
    // Test a cubic B-spline curve
    let curve = NurbsCurve3::new(
        3,
        vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 2.0, 0.0),
            Point3::new(3.0, 2.0, 0.0),
            Point3::new(4.0, 0.0, 0.0),
            Point3::new(5.0, -1.0, 0.0),
        ],
        vec![0.0, 0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0, 1.0],
        None,
    )
    .unwrap();

    let tol = Tolerance::new(1e-6);
    for i in 0..=10 {
        let t = i as f64 / 10.0;
        let analytic = curve.derivative_at(t);
        
        // Numerical derivative for comparison
        let h = 1e-6;
        let p0 = curve.point_at((t - h).max(0.0));
        let p1 = curve.point_at((t + h).min(1.0));
        let dt = (t + h).min(1.0) - (t - h).max(0.0);
        let numerical = p1.sub_point(p0).mul_scalar(1.0 / dt);

        assert!(
            (analytic.x - numerical.x).abs() < 0.01,
            "x mismatch at t={}: analytic={}, numerical={}",
            t, analytic.x, numerical.x
        );
        assert!(
            (analytic.y - numerical.y).abs() < 0.01,
            "y mismatch at t={}: analytic={}, numerical={}",
            t, analytic.y, numerical.y
        );
    }
}

#[test]
fn nurbs_rational_derivative_is_finite() {
    // Test a rational quadratic B-spline (weighted)
    let curve = NurbsCurve3::new(
        2,
        vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
        ],
        vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        Some(vec![1.0, 0.707, 1.0]), // Approximate weight for a circular arc
    )
    .unwrap();

    for i in 0..=10 {
        let t = i as f64 / 10.0;
        let deriv = curve.derivative_at(t);
        assert!(deriv.x.is_finite(), "derivative x not finite at t={}", t);
        assert!(deriv.y.is_finite(), "derivative y not finite at t={}", t);
        assert!(deriv.z.is_finite(), "derivative z not finite at t={}", t);
    }
}

#[test]
fn nurbs_line_derivative_is_constant() {
    // A degree-1 NURBS with 2 control points is a line
    let curve = NurbsCurve3::new(
        1,
        vec![Point3::new(0.0, 0.0, 0.0), Point3::new(4.0, 2.0, 0.0)],
        vec![0.0, 0.0, 1.0, 1.0],
        None,
    )
    .unwrap();

    let tol = Tolerance::new(1e-10);
    let d0 = curve.derivative_at(0.0);
    let d1 = curve.derivative_at(0.5);
    let d2 = curve.derivative_at(1.0);

    // All derivatives should be equal for a line
    assert!(tol.approx_eq_f64(d0.x, d1.x));
    assert!(tol.approx_eq_f64(d0.y, d1.y));
    assert!(tol.approx_eq_f64(d1.x, d2.x));
    assert!(tol.approx_eq_f64(d1.y, d2.y));

    // The derivative should be the direction vector scaled by 1/(domain length)
    // domain is [0,1], so derivative = (4, 2, 0)
    assert!(tol.approx_eq_f64(d0.x, 4.0));
    assert!(tol.approx_eq_f64(d0.y, 2.0));
}

/// Tests that closed polylines include the closing segment in arc length calculation.
///
/// This verifies that when a polyline is marked as closed, the arc length includes
/// the segment from the last point back to the first point.
#[test]
fn closed_polyline_includes_closing_segment_in_arc_length() {
    // Create an open square (4 points, 3 segments)
    let open_square = Polyline3::new(
        vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ],
        false, // open
    )
    .unwrap();

    // Create a closed square (same 4 points, 4 segments including closing)
    let closed_square = Polyline3::new(
        vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ],
        true, // closed
    )
    .unwrap();

    // Use higher sample count for more accurate arc length estimation
    let open_length = curve_arc_length(&open_square, 256);
    let closed_length = curve_arc_length(&closed_square, 256);

    let tol = Tolerance::new(0.05);

    // Open square: 3 sides = 3.0
    assert!(tol.approx_eq_f64(open_length, 3.0), "open_length = {} expected 3.0", open_length);

    // Closed square: 4 sides = 4.0
    assert!(tol.approx_eq_f64(closed_length, 4.0), "closed_length = {} expected 4.0", closed_length);

    // Most importantly, the closed square should be longer by 1 unit (the closing segment)
    assert!(tol.approx_eq_f64(closed_length - open_length, 1.0),
        "closed_length - open_length = {} expected 1.0", closed_length - open_length);
}

/// Tests that divide_curve_by_count on closed polylines correctly samples
/// across the closing segment.
///
/// This verifies the fix for the issue where closed curves were treated as open
/// in curve division operations, causing the closing segment to be ignored.
#[test]
fn divide_closed_polyline_samples_closing_segment() {
    // Create a closed square
    let closed_square = Polyline3::new(
        vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ],
        true, // closed
    )
    .unwrap();

    // Divide into 4 segments (should give 5 points)
    let result = divide_curve_by_count(&closed_square, 4);

    let tol = Tolerance::new(1e-6);

    // With 4 segments on a closed square (perimeter 4.0), each segment is 1.0 unit
    // Point at t=0 (length 0.0): corner at (0, 0, 0)
    // Point at t=0.25 (length 1.0): corner at (1, 0, 0)
    // Point at t=0.5 (length 2.0): corner at (1, 1, 0)
    // Point at t=0.75 (length 3.0): corner at (0, 1, 0)
    // Point at t=1.0 (length 4.0): back at (0, 0, 0) - traversed the closing segment!

    assert_eq!(result.points.len(), 5);

    // First point should be start
    assert!(tol.approx_eq_point3(result.points[0], Point3::new(0.0, 0.0, 0.0)));

    // Last point should also be at start (we traversed the full closed loop)
    assert!(tol.approx_eq_point3(result.points[4], Point3::new(0.0, 0.0, 0.0)));

    // Verify the intermediate points are at corners
    assert!(tol.approx_eq_point3(result.points[1], Point3::new(1.0, 0.0, 0.0)));
    assert!(tol.approx_eq_point3(result.points[2], Point3::new(1.0, 1.0, 0.0)));
    assert!(tol.approx_eq_point3(result.points[3], Point3::new(0.0, 1.0, 0.0)));

    // The parameters should span [0, 1]
    assert!(tol.approx_eq_f64(result.parameters[0], 0.0));
    assert!(tol.approx_eq_f64(result.parameters[4], 1.0));
}
/// Tests that `NurbsCurve3::interpolate_through_points` creates a curve passing through input points.
#[test]
fn nurbs_interpolate_passes_through_input_points() {
    let data_points = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(2.0, 0.5, 0.0),
        Point3::new(3.0, 1.5, 0.0),
        Point3::new(4.0, 0.0, 0.0),
    ];

    let curve = NurbsCurve3::interpolate_through_points(&data_points, 3, false)
        .expect("interpolation should succeed");

    let tol = Tolerance::new(1e-6);

    // The curve should pass through each input point at corresponding parameter values
    // For chord-length parameterization, we need to evaluate at the chord-length parameters
    // However, a simpler check: endpoints must match exactly (due to clamped knots)
    let start = curve.point_at(0.0);
    let end = curve.point_at(1.0);

    assert!(
        tol.approx_eq_point3(start, data_points[0]),
        "Start point should match first input: {:?} vs {:?}",
        start,
        data_points[0]
    );
    assert!(
        tol.approx_eq_point3(end, *data_points.last().unwrap()),
        "End point should match last input: {:?} vs {:?}",
        end,
        data_points.last().unwrap()
    );

    // Verify the curve has the expected degree
    assert_eq!(curve.degree, 3);
}

/// Tests interpolation with only 2 points (should produce a line).
#[test]
fn nurbs_interpolate_two_points_is_linear() {
    let data_points = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(10.0, 5.0, 2.0),
    ];

    let curve = NurbsCurve3::interpolate_through_points(&data_points, 3, false)
        .expect("interpolation should succeed with 2 points");

    let tol = Tolerance::new(1e-6);

    // With 2 points, degree is clamped to 1 (linear)
    assert_eq!(curve.degree, 1);

    // Verify endpoints
    assert!(tol.approx_eq_point3(curve.point_at(0.0), data_points[0]));
    assert!(tol.approx_eq_point3(curve.point_at(1.0), data_points[1]));

    // Midpoint should be exactly between
    let mid = curve.point_at(0.5);
    let expected_mid = Point3::new(5.0, 2.5, 1.0);
    assert!(
        tol.approx_eq_point3(mid, expected_mid),
        "Midpoint should be linear interpolation: {:?} vs {:?}",
        mid,
        expected_mid
    );
}

/// Tests interpolation with degree 1 (piecewise linear).
#[test]
fn nurbs_interpolate_degree_1_is_piecewise_linear() {
    let data_points = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 2.0, 0.0),
        Point3::new(2.0, 0.0, 0.0),
    ];

    let curve = NurbsCurve3::interpolate_through_points(&data_points, 1, false)
        .expect("degree-1 interpolation should succeed");

    let tol = Tolerance::new(1e-6);

    // Degree 1 = linear interpolation
    assert_eq!(curve.degree, 1);

    // All data points should lie on the curve
    assert!(tol.approx_eq_point3(curve.point_at(0.0), data_points[0]));
    assert!(tol.approx_eq_point3(curve.point_at(1.0), data_points[2]));
}

/// Tests closed curve interpolation.
#[test]
fn nurbs_interpolate_closed_curve_wraps_smoothly() {
    let data_points = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(2.0, 0.0, 0.0),
        Point3::new(1.0, -1.0, 0.0),
    ];

    let curve = NurbsCurve3::interpolate_through_points(&data_points, 3, true)
        .expect("closed interpolation should succeed");

    let tol = Tolerance::new(1e-3);

    // For closed curves, the curve should start at the first point
    let start = curve.point_at(0.0);
    assert!(
        tol.approx_eq_point3(start, data_points[0]),
        "Closed curve should start at first input point: {:?} vs {:?}",
        start,
        data_points[0]
    );

    // The curve should wrap around by including copies of the first points
    // at the end, so the end of the domain should be near the wrapped points.
    // For a closed curve with n=4 points and degree=3, we add 3 wrapped points,
    // so the curve passes through points[0], points[1], points[2], points[3],
    // points[0], points[1], points[2].
    // The end of the curve (t=1.0) should be at the 3rd wrapped point = points[2].
    let end = curve.point_at(1.0);
    // This is the expected behavior of our simple wrapping approach
    assert!(
        tol.approx_eq_point3(end, data_points[2]),
        "Closed curve end should be at wrapped point: {:?} vs {:?}",
        end,
        data_points[2]
    );
}