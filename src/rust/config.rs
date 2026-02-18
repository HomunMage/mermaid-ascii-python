/// Configuration for the rendering pipeline.
///
/// Mirrors Python's `config.py` RenderConfig dataclass.

#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Use Unicode box-drawing characters (true) or plain ASCII (false).
    pub unicode: bool,
    /// Padding inside node boxes (in characters).
    pub padding: usize,
    /// Override the diagram direction (e.g. "LR", "TD"). None = use diagram's own direction.
    pub direction_override: Option<String>,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            unicode: true,
            padding: 1,
            direction_override: None,
        }
    }
}

impl RenderConfig {
    pub fn new() -> Self {
        Self::default()
    }
}
