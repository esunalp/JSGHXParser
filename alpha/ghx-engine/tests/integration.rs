use ghx_engine::Engine;
use ghx_engine::components::ComponentRegistry;
use ghx_engine::graph::Graph;
use ghx_engine::graph::evaluator::{self, EvaluationResult};
use ghx_engine::graph::node::{Node, NodeId};
use ghx_engine::graph::value::Value;
use ghx_engine::parse::ghx_xml;

#[test]
fn engine_initializes() {
    let engine = Engine::new();
    assert!(engine.is_initialized());
}

#[test]
fn slider_updates_require_existing_identifier() {
    let xml = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tools/ghx-samples/minimal_line.ghx"
    ));
    let mut engine = Engine::new();
    engine.load_ghx(xml).expect("load ghx");

    engine
        .update_input_value("Length", Value::Number(4.5))
        .expect("valid slider name");
    assert!(
        engine
            .update_input_value("onbekend", Value::Number(1.0))
            .is_err()
    );
}

#[test]
fn parses_brugtest_boolean_toggle() {
    let xml = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/testfiles/brugtest.ghx"));
    let graph = ghx_xml::parse_str(xml).expect("parse brugtest");

    // GUID for Boolean Toggle: 2e78987b-9dfb-42a2-8b76-3923ac8bd91a
    let toggle = graph
        .nodes()
        .iter()
        .find(|n| n.guid.as_deref() == Some("{2e78987b-9dfb-42a2-8b76-3923ac8bd91a}"));
    assert!(toggle.is_some(), "Boolean Toggle node should exist");

    let toggle = toggle.unwrap();
    let val = toggle.meta("Value").expect("Value meta should be present");
    // Check if it's boolean true (as seen in the file)
    match val {
        ghx_engine::graph::node::MetaValue::Boolean(b) => assert!(*b, "Toggle should be true"),
        _ => panic!("Expected Boolean meta value"),
    }

    // Check if "Output" pin exists (added by my fix)
    assert!(
        toggle.outputs.contains_key("Output"),
        "Output pin should exist"
    );
}

#[test]
fn lijntest2_line_outputs_expected_points() {
    let result = evaluate_sample(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../web/testfiles/lijntest2.ghx"
    )));

    let line = result
        .geometry
        .iter()
        .find_map(|entry| match &entry.value {
            Value::CurveLine { p1, p2 } => Some((p1, p2)),
            _ => None,
        })
        .expect("line geometry present");

    assert_point_close(line.0, [0.0, 0.0, 0.0]);
    assert_point_close(line.1, [1000.0, 0.0, 0.0]);
}

#[test]
fn geometry_requires_evaluation_first() {
    let xml = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tools/ghx-samples/minimal_line.ghx"
    ));
    let mut engine = Engine::new();
    engine.load_ghx(xml).expect("load ghx");

    assert!(engine.get_geometry().is_err());
}

#[test]
fn line_sample_produces_curve_line() {
    let result = evaluate_sample(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tools/ghx-samples/minimal_line.ghx"
    )));
    let curve_count = result
        .geometry
        .iter()
        .filter(|entry| matches!(&entry.value, Value::CurveLine { .. }))
        .count();
    assert_eq!(curve_count, 1, "expected exactly one curve line");

    let line = result
        .geometry
        .iter()
        .find_map(|entry| match &entry.value {
            Value::CurveLine { p1, p2 } => Some((p1, p2)),
            _ => None,
        })
        .expect("line geometry present");

    assert_point_close(line.0, [0.0, 0.0, 0.0]);
    assert_point_close(line.1, [3.0, 0.0, 0.0]);
}

#[test]
fn extrude_sample_produces_surface_with_faces() {
    let result = evaluate_sample(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tools/ghx-samples/minimal_extrude.ghx"
    )));
    let surface = result
        .geometry
        .iter()
        .find_map(|entry| match &entry.value {
            Value::Surface { vertices, faces } => Some((vertices, faces)),
            _ => None,
        })
        .expect("surface output present");

    assert_eq!(surface.0.len(), 4);
    assert!(!surface.1.is_empty());
}

#[test]
fn line_sample_matches_expected_snapshot() {
    let result = evaluate_sample(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tools/ghx-samples/minimal_line.ghx"
    )));
    let curve_entry = result
        .geometry
        .iter()
        .find(|entry| matches!(&entry.value, Value::CurveLine { .. }))
        .expect("curve line present");
    let expected = Value::CurveLine {
        p1: [0.0, 0.0, 0.0],
        p2: [3.0, 0.0, 0.0],
    };

    assert_value_close(&curve_entry.value, &expected, 1e-9);
}

#[test]
fn flip_surface_evaluates_without_guide_input() {
    let mut graph = Graph::new();
    let mut node = Node::new(NodeId::new(0));
    node.guid = Some("{c3d1f2b8-8596-4e8d-8861-c28ba8ffb4f4}".to_owned());
    node.nickname = Some("Flip".to_owned());
    node.add_input_pin("S");
    node.add_input_pin("G");
    node.set_input(
        "S",
        Value::Surface {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            faces: vec![vec![0_u32, 1, 2, 3]],
        },
    );

    let node_id = graph.add_node(node).expect("node added");
    let registry = ComponentRegistry::default();
    let result = evaluator::evaluate(&graph, &registry).expect("flip evaluates");

    let outputs = result
        .node_outputs
        .get(&node_id)
        .expect("flip outputs present");
    let flip_result = outputs.get("R").expect("result pin present");
    assert!(matches!(flip_result, Value::Boolean(true)));
}

/// Verifies that the Polyline component preserves input points exactly.
/// This is a regression test for the issue where adaptive tessellation was
/// resampling the polyline output, which removed/shifted original vertices.
#[test]
fn polyline_component_preserves_input_points_exactly() {
    use ghx_engine::components::curve_spline::ComponentKind as CurveSplineKind;
    use ghx_engine::components::Component;

    // Input points: a simple triangle with specific coordinates
    let input_points = vec![
        Value::Point([0.0, 0.0, 0.0]),
        Value::Point([100.0, 0.0, 0.0]),
        Value::Point([50.0, 86.6025403784, 0.0]), // roughly equilateral triangle
    ];

    // Test open polyline
    {
        let inputs = vec![
            Value::List(input_points.clone()),
            Value::Boolean(false), // closed = false
        ];

        let result = CurveSplineKind::Polyline.evaluate(&inputs, &Default::default())
            .expect("polyline should evaluate");
        let curve = result.get("C").expect("curve output");

        match curve {
            Value::List(points) => {
                assert_eq!(points.len(), 3, "open polyline should have exactly 3 points");
                for (idx, (output, input)) in points.iter().zip(input_points.iter()).enumerate() {
                    match (output, input) {
                        (Value::Point(o), Value::Point(i)) => {
                            for (component, (vo, vi)) in o.iter().zip(i.iter()).enumerate() {
                                let diff = (vo - vi).abs();
                                assert!(
                                    diff < 1e-12,
                                    "point {idx} component {component} differs: {vo} vs {vi} (diff={diff})"
                                );
                            }
                        }
                        _ => panic!("expected points, got {:?} and {:?}", output, input),
                    }
                }
            }
            _ => panic!("expected list, got {:?}", curve),
        }
    }

    // Test closed polyline
    {
        let inputs = vec![
            Value::List(input_points.clone()),
            Value::Boolean(true), // closed = true
        ];

        let result = CurveSplineKind::Polyline.evaluate(&inputs, &Default::default())
            .expect("polyline should evaluate");
        let curve = result.get("C").expect("curve output");

        match curve {
            Value::List(points) => {
                // Closed polyline should have 4 points (original 3 + closing point)
                assert_eq!(points.len(), 4, "closed polyline should have 4 points (3 + closing)");

                // First 3 points should match input exactly
                for (idx, (output, input)) in points.iter().take(3).zip(input_points.iter()).enumerate() {
                    match (output, input) {
                        (Value::Point(o), Value::Point(i)) => {
                            for (component, (vo, vi)) in o.iter().zip(i.iter()).enumerate() {
                                let diff = (vo - vi).abs();
                                assert!(
                                    diff < 1e-12,
                                    "point {idx} component {component} differs: {vo} vs {vi} (diff={diff})"
                                );
                            }
                        }
                        _ => panic!("expected points, got {:?} and {:?}", output, input),
                    }
                }

                // Last point should equal first point (closing)
                match (&points[3], &points[0]) {
                    (Value::Point(last), Value::Point(first)) => {
                        for (component, (vl, vf)) in last.iter().zip(first.iter()).enumerate() {
                            let diff = (vl - vf).abs();
                            assert!(
                                diff < 1e-12,
                                "closing point component {component} differs from first: {vl} vs {vf}"
                            );
                        }
                    }
                    _ => panic!("expected points"),
                }
            }
            _ => panic!("expected list, got {:?}", curve),
        }
    }
}

fn evaluate_sample(xml: &str) -> EvaluationResult {
    let graph = ghx_xml::parse_str(xml).expect("parse ghx");
    let registry = ComponentRegistry::default();
    evaluator::evaluate(&graph, &registry).expect("evaluate graph")
}

fn assert_point_close(actual: &[f64; 3], expected: [f64; 3]) {
    for (idx, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
        let diff = (a - e).abs();
        assert!(diff < 1e-9, "coordinate {idx} differs: {a} vs {e}");
    }
}

fn assert_value_close(actual: &Value, expected: &Value, tol: f64) {
    match (actual, expected) {
        (Value::CurveLine { p1: lp1, p2: lp2 }, Value::CurveLine { p1: rp1, p2: rp2 }) => {
            assert_point_close(lp1, *rp1);
            assert_point_close(lp2, *rp2);
        }
        (
            Value::Surface {
                vertices: lv,
                faces: lf,
            },
            Value::Surface {
                vertices: rv,
                faces: rf,
            },
        ) => {
            assert_eq!(lv.len(), rv.len(), "vertex count differs");
            for (idx, (a, e)) in lv.iter().zip(rv.iter()).enumerate() {
                for (component, (va, ve)) in a.iter().zip(e.iter()).enumerate() {
                    let diff = (va - ve).abs();
                    assert!(
                        diff <= tol,
                        "vertex {idx} component {component} differs: {va} vs {ve}"
                    );
                }
            }
            assert_eq!(lf, rf, "face indices differ");
        }
        (Value::Point(a), Value::Point(b)) => assert_point_close(a, *b),
        _ => panic!("mismatched geometry variants: {actual:?} vs {expected:?}"),
    }
}
