use std::fmt::Debug;

pub struct RenderData {}

impl Default for RenderData {
    fn default() -> Self {
        Self {}
    }
}

impl Debug for RenderData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "RenderData {{ .. }}")
    }
}
