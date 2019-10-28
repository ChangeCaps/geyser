use crate::fumarole::Vertex2;

pub mod default_vertex_shader {
    vulkano_shaders::shader!{
        ty: "vertex",
        src: "
#version 450

layout(location = 0) in vec2 position;


void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}
        "
    }
}

pub fn cover_screen() -> Vec<Vertex2> {
    vec![
        // Top right
        Vertex2::new(1.0, 1.0),
        Vertex2::new(-1.0, 1.0),
        Vertex2::new(1.0, -1.0),

        //Bottom left
        Vertex2::new(-1.0, -1.0),
        Vertex2::new(-1.0, 1.0),
        Vertex2::new(1.0, -1.0),
    ]
}
