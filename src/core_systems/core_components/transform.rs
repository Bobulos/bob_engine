use crate::vec::Float2;

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Float2
}

impl Transform {
    pub fn new(position: Float2) -> Self {
        Self {
            position
        }
    }
}