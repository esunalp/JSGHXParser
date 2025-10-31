#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod components;
pub mod graph;
pub mod parse;

use wasm_bindgen::prelude::*;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "console_error_panic_hook", target_arch = "wasm32"))] {
        #[wasm_bindgen(start)]
        pub fn initialize() {
            console_error_panic_hook::set_once();
        }
    } else {
        #[wasm_bindgen(start)]
        pub fn initialize() {
            // no-op fallback when panic hook is disabled
        }
    }
}

#[macro_export]
macro_rules! debug_log {
    ($($t:tt)*) => {{
        #[cfg(feature = "debug_logs")]
        {
            #[cfg(target_arch = "wasm32")]
            {
                ::web_sys::console::log_1(&::wasm_bindgen::JsValue::from_str(&format!($($t)*)));
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                println!("{}", format!($($t)*));
            }
        }
    }};
}

/// Public entry point for consumers. At dit stadium voorziet de engine enkel in
/// een basisstructuur waarop latere functionaliteit gebouwd wordt.
#[wasm_bindgen]
pub struct Engine {
    /// Placeholder state; zal later de actieve graph en evaluatiecontext bevatten.
    initialized: bool,
}

#[wasm_bindgen]
impl Engine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Engine {
        Engine { initialized: true }
    }

    /// Geeft terug of de engine de minimale initialisatie heeft doorlopen.
    #[wasm_bindgen]
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}
