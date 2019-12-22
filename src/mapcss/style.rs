#[derive(Debug, Clone, Copy)]
pub struct Size(pub f32);

impl Default for Size {
    fn default() -> Self {
        Size(1.0)
    }
}
