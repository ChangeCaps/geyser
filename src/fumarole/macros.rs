#[macro_export]
macro_rules! entire_window_loop {
    ($instance:expr, $tt:tt: $arg:expr, $loop:block) => {
        use geyser::fumarole::Vertex2;
        use std::sync::Arc;

        let render_pass = Arc::new(vulkano::single_pass_renderpass!($instance.device(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: vulkano::format::Format::R8G8B8A8Unorm,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        ).unwrap());

        let verts = vec![
            Vertex2::new(-1.0, -1.0),
            Vertex2::new(1.0, -1.0),
            Vertex2::new(-1.0, 1.0),

            Vertex2::new(1.0, 1.0),
            Vertex2::new(-1.0, 1.0),
            Vertex2::new(1.0, -1.0),
        ];

        let vert_buffer = $instance.buffer_from_data(verts);

        mod vs {
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

        mod fs {
            vulkano_shaders::shader!{
                ty: "fragment",
                $tt: $arg, 
            }
        }


        let vs = vs::Shader::load($instance.device()).expect("failed to create shader module");
        let fs = fs::Shader::load($instance.device()).expect("failed to create shader module");

        let pipeline = Arc::new(vulkano::pipeline::GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex2>()
            .vertex_shader(vs.main_entry_point(), ())
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            .render_pass(vulkano::framebuffer::Subpass::from(render_pass.clone(), 0).unwrap())
            .build($instance.device()).unwrap()
        );
    };
}