use ghx_engine::Engine;

#[test]
fn engine_initializes() {
    let engine = Engine::new();
    assert!(engine.is_initialized());
}

#[cfg(target_arch = "wasm32")]
mod wasm_only {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct SliderDto {
        id: String,
        name: String,
        value: f64,
    }

    #[derive(Debug, Deserialize)]
    struct GeometryResponse {
        items: Vec<GeometryItem>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(tag = "type")]
    enum GeometryItem {
        Point {
            coordinates: [f64; 3],
        },
        CurveLine {
            points: Vec<[f64; 3]>,
        },
        Surface {
            vertices: Vec<[f64; 3]>,
            faces: Vec<Vec<u32>>,
        },
    }

    #[test]
    fn sliders_roundtrip_and_geometry_output() {
        let xml = include_str!("../../tools/ghx-samples/minimal_line.ghx");
        let mut engine = Engine::new();
        engine.load_ghx(xml).expect("load ghx");

        let sliders_value = engine.get_sliders().expect("serialize sliders");
        let sliders: Vec<SliderDto> =
            serde_wasm_bindgen::from_value(sliders_value).expect("deserialize sliders");
        assert_eq!(sliders.len(), 1);
        assert_eq!(sliders[0].name, "Slider A");

        engine.evaluate().expect("evaluate graph");
        let geometry_value = engine.get_geometry().expect("geometry available");
        let geometry: GeometryResponse =
            serde_wasm_bindgen::from_value(geometry_value).expect("deserialize geometry");
        assert!(!geometry.items.is_empty());
        assert!(matches!(geometry.items[0], GeometryItem::CurveLine { .. }));
    }

    #[test]
    fn slider_updates_require_existing_identifier() {
        let xml = include_str!("../../tools/ghx-samples/minimal_line.ghx");
        let mut engine = Engine::new();
        engine.load_ghx(xml).expect("load ghx");

        engine
            .set_slider_value("Slider A", 3.5)
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
}
