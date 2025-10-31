//! Component registry en evaluatie-logica (stub).

pub mod add;
pub mod construct_point;
pub mod extrude;
pub mod line;
pub mod number_slider;

/// Placeholder trait dat later de component-evaluatie zal definiëren.
pub trait Component {
    /// Voert de component uit. Momenteel nog niet geïmplementeerd.
    fn evaluate(&self) {
        // Implementatie volgt in latere iteratie.
    }
}
