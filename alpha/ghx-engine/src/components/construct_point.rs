//! Placeholder componentimplementatie.

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

/// Markerstruct voor een component.
#[derive(Debug, Default, Clone, Copy)]
pub struct ComponentImpl;

impl Component for ComponentImpl {
    fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        Err(ComponentError::new(
            "Construct Point evaluatie nog niet geïmplementeerd",
        ))
    }
}
