//! Kern datastructuren voor het modelleren van Grasshopper grafen.

pub mod node;
pub mod topo;
pub mod value;
pub mod wire;

/// Placeholder graph type. Wordt later uitgebreid met nodes, wires en metadata.
#[derive(Debug, Default, Clone)]
pub struct Graph;
