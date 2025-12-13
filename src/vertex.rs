/// Vertex structure with position, screen position, and normal
#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub screen_position: [f32; 2],
    pub normal: [f32; 3],
}
