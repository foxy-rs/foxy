pub struct Renderer {}

impl Drop for Renderer {
    fn drop(&mut self) {}
}

impl Renderer {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {})
    }
}
