use ghx_engine::Engine;

#[test]
fn engine_initializes() {
    let engine = Engine::new();
    assert!(engine.is_initialized());
}
