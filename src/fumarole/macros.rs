

#[macro_export]
macro_rules! single_pass_pipeline {
    ($fumarole:expr, 
     $tt:tt: $arg:expr, 
     $verts:expr) => {
        use geyser::fumarole::Vertex2;
        use std::sync::Arc;

        let render_pass = Arc::new(vulkano::single_pass_renderpass!($fumarole.device(),
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


        let vert_buffer = $fumarole.buffer_from_data($verts);

        mod fs {
            vulkano_shaders::shader!{
                ty: "fragment",
                $tt: $arg, 
            }
        }

        let vs = $crate::fumarole::default_vertex_shader::Shader::load($fumarole.device()).expect("failed to create shader module");
        let fs = fs::Shader::load($fumarole.device()).expect("failed to create shader module");

        let pipeline = Arc::new(vulkano::pipeline::GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex2>()
            .vertex_shader(vs.main_entry_point(), ())
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            .render_pass(vulkano::framebuffer::Subpass::from(render_pass.clone(), 0).unwrap())
            .build($fumarole.device()).unwrap()
        )
    };
}
