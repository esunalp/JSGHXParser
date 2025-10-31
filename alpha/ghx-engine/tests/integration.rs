use ghx_engine::components::ComponentRegistry;
use ghx_engine::graph::evaluator::{self, EvaluationResult};
use ghx_engine::graph::value::Value;
use ghx_engine::parse::ghx_xml;
use ghx_engine::Engine;

#[test]
fn engine_initializes() {
    let engine = Engine::new();
    assert!(engine.is_initialized());
}

#[test]
fn slider_updates_require_existing_identifier() {
    let xml = include_str!("../../tools/ghx-samples/minimal_line.ghx");
    let mut engine = Engine::new();
    engine.load_ghx(xml).expect("load ghx");

    engine
        .set_slider_value("Length", 4.5)
        .expect("valid slider name");
    assert!(engine.set_slider_value("onbekend", 1.0).is_err());
}

#[test]
fn geometry_requires_evaluation_first() {
    let xml = include_str!("../../tools/ghx-samples/minimal_line.ghx");
    let mut engine = Engine::new();
    engine.load_ghx(xml).expect("load ghx");

    assert!(engine.get_geometry().is_err());
}

#[test]
fn line_sample_produces_curve_line() {
    let result = evaluate_sample(include_str!("../../tools/ghx-samples/minimal_line.ghx"));
    let curve_count = result
        .geometry
        .iter()
        .filter(|value| matches!(value, Value::CurveLine { .. }))
        .count();
    assert_eq!(curve_count, 1, "expected exactly one curve line");

    let line = result
        .geometry
        .iter()
        .find_map(|value| match value {
            Value::CurveLine { p1, p2 } => Some((p1, p2)),
            _ => None,
        })
        .expect("line geometry present");

    assert_point_close(line.0, [0.0, 0.0, 0.0]);
    assert_point_close(line.1, [3.0, 0.0, 0.0]);
}

#[test]
fn extrude_sample_produces_surface_with_faces() {
    let result = evaluate_sample(include_str!("../../tools/ghx-samples/minimal_extrude.ghx"));
    let surface = result
        .geometry
        .iter()
        .find_map(|value| match value {
            Value::Surface { vertices, faces } => Some((vertices, faces)),
            _ => None,
        })
        .expect("surface output present");

    assert_eq!(surface.0.len(), 4);
    assert!(!surface.1.is_empty());
}

#[test]
fn line_sample_matches_expected_snapshot() {
    let result = evaluate_sample(include_str!("../../tools/ghx-samples/minimal_line.ghx"));
    let curve = result
        .geometry
        .iter()
        .find(|value| matches!(value, Value::CurveLine { .. }))
        .expect("curve line present");
    let expected = Value::CurveLine {
        p1: [0.0, 0.0, 0.0],
        p2: [3.0, 0.0, 0.0],
    };

    assert_value_close(curve, &expected, 1e-9);
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
        (
            Value::CurveLine { p1: lp1, p2: lp2 },
            Value::CurveLine { p1: rp1, p2: rp2 },
        ) => {
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
