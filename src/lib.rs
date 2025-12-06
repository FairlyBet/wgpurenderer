pub mod camera;
pub mod transform;

pub use camera::Camera;
pub use transform::Transform;

pub struct Scene {
    nodes: Vec<Node>,
}

pub struct Node {
    transform: Transform,
}

pub struct Mesh {
    device: wgpu::Device,
    vertex_data: (),
    uniform_data: (),
    textures: (),
    output_targets: (),
    pipeline_params: (),
}

impl Mesh {
    pub fn add_vertex_data() {
        
    }
}

pub struct ShaderSource {
    src: Vec<Box<str>>,
}



pub struct Renderer {}

impl Renderer {
    pub fn render(mesh: Mesh) {}
}
