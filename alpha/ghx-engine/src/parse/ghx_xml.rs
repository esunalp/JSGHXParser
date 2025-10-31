//! Parser voor GHX XML-bestanden (placeholder).

use crate::graph::Graph;

/// Leest een GHX-document en geeft een lege graph terug.
#[allow(clippy::unnecessary_wraps)]
pub fn parse_str(_input: &str) -> Result<Graph, &'static str> {
    Ok(Graph::default())
}
