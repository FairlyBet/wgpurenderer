use bytemuck::{Pod, Zeroable};
use glam;
use glfw::{Action, Key};
use std::borrow::Cow;
use std::num::NonZeroU32;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Uniforms {
    model: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
    light_color: [f32; 3],
    _padding: f32,
}

fn create_cube_mesh() -> (Vec<Vertex>, Vec<u16>) {
    let vertices = vec![
        // Front face (z = 0.5)
        Vertex {
            position: [-0.5, -0.5, 0.5],
            normal: [0.0, 0.0, 1.0],
        },
        Vertex {
            position: [0.5, -0.5, 0.5],
            normal: [0.0, 0.0, 1.0],
        },
        Vertex {
            position: [0.5, 0.5, 0.5],
            normal: [0.0, 0.0, 1.0],
        },
        Vertex {
            position: [-0.5, 0.5, 0.5],
            normal: [0.0, 0.0, 1.0],
        },
        // Back face (z = -0.5)
        Vertex {
            position: [0.5, -0.5, -0.5],
            normal: [0.0, 0.0, -1.0],
        },
        Vertex {
            position: [-0.5, -0.5, -0.5],
            normal: [0.0, 0.0, -1.0],
        },
        Vertex {
            position: [-0.5, 0.5, -0.5],
            normal: [0.0, 0.0, -1.0],
        },
        Vertex {
            position: [0.5, 0.5, -0.5],
            normal: [0.0, 0.0, -1.0],
        },
        // Top face (y = 0.5)
        Vertex {
            position: [-0.5, 0.5, 0.5],
            normal: [0.0, 1.0, 0.0],
        },
        Vertex {
            position: [0.5, 0.5, 0.5],
            normal: [0.0, 1.0, 0.0],
        },
        Vertex {
            position: [0.5, 0.5, -0.5],
            normal: [0.0, 1.0, 0.0],
        },
        Vertex {
            position: [-0.5, 0.5, -0.5],
            normal: [0.0, 1.0, 0.0],
        },
        // Bottom face (y = -0.5)
        Vertex {
            position: [-0.5, -0.5, -0.5],
            normal: [0.0, -1.0, 0.0],
        },
        Vertex {
            position: [0.5, -0.5, -0.5],
            normal: [0.0, -1.0, 0.0],
        },
        Vertex {
            position: [0.5, -0.5, 0.5],
            normal: [0.0, -1.0, 0.0],
        },
        Vertex {
            position: [-0.5, -0.5, 0.5],
            normal: [0.0, -1.0, 0.0],
        },
        // Right face (x = 0.5)
        Vertex {
            position: [0.5, -0.5, 0.5],
            normal: [1.0, 0.0, 0.0],
        },
        Vertex {
            position: [0.5, -0.5, -0.5],
            normal: [1.0, 0.0, 0.0],
        },
        Vertex {
            position: [0.5, 0.5, -0.5],
            normal: [1.0, 0.0, 0.0],
        },
        Vertex {
            position: [0.5, 0.5, 0.5],
            normal: [1.0, 0.0, 0.0],
        },
        // Left face (x = -0.5)
        Vertex {
            position: [-0.5, -0.5, -0.5],
            normal: [-1.0, 0.0, 0.0],
        },
        Vertex {
            position: [-0.5, -0.5, 0.5],
            normal: [-1.0, 0.0, 0.0],
        },
        Vertex {
            position: [-0.5, 0.5, 0.5],
            normal: [-1.0, 0.0, 0.0],
        },
        Vertex {
            position: [-0.5, 0.5, -0.5],
            normal: [-1.0, 0.0, 0.0],
        },
    ];

    let indices = vec![
        0, 1, 2, 0, 2, 3, // Front
        4, 5, 6, 4, 6, 7, // Back
        8, 9, 10, 8, 10, 11, // Top
        12, 13, 14, 12, 14, 15, // Bottom
        16, 17, 18, 16, 18, 19, // Right
        20, 21, 22, 20, 22, 23, // Left
    ];

    (vertices, indices)
}

struct State {
    size: (u32, u32),
    renderer: wgpurenderer::Renderer,
    msaa_texture: wgpu::Texture,
    depth_texture: wgpu::Texture,
    sample_count: u32,
    uniform_buffer: wgpu::Buffer,
    render_pass: wgpurenderer::renderpass::RenderPass,
    rotation: f32,
    start_time: std::time::Instant,
}

impl State {
    async fn new(context: glfw::PRenderContext) -> Self {
        let size = (800, 600);

        let mut renderer = wgpurenderer::Renderer::new();
        let surface = renderer.context().instance().create_surface(context).unwrap();

        renderer.init_surface(surface, size.0, size.1);

        let (vertices, indices) = create_cube_mesh();
        let num_indices = indices.len() as u32;

        let vertex_buffer =
            renderer.context().device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let index_buffer =
            renderer.context().device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            });

        let uniform_buffer = renderer.context().device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // 1. Create material
        let bind_group_layout = vec![wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }];

        let material = wgpurenderer::Material {
            bind_groups: vec![bind_group_layout.clone()],
            vertex: wgpurenderer::Vertex {
                buffers: vec![Vertex::desc()],
            },
            fragment: Some(wgpurenderer::Fragment {
                targets: vec![Some(wgpu::ColorTargetState {
                    format: renderer.surface_format(),
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            source: Cow::Borrowed(include_str!("../shaders/shader.wgsl")),
        };

        let pipeline_handle = renderer.create_material(&material);

        let bindgroup_handle = renderer.create_bindgroup(
            &bind_group_layout,
            &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        );

        let geometry = wgpurenderer::Geometry {
            index_buffer: Some(index_buffer),
            index_format: wgpu::IndexFormat::Uint16,
            buffers: vec![(vertex_buffer, None)],
            count: num_indices,
        };

        let shader_data = wgpurenderer::ShaderData {
            immediates: Vec::new(),
            bind_groups: smallvec::smallvec![bindgroup_handle],
        };

        let draw_call = wgpurenderer::DrawCall {
            geometry,
            shader_data,
            instance_count: NonZeroU32::new(1).unwrap(),
            render_pipeline_handle: pipeline_handle,
        };

        let sample_count = 4;
        let msaa_texture = Self::create_msaa_texture(
            renderer.context().device(),
            size.0,
            size.1,
            renderer.surface_format(),
            sample_count,
        );
        let depth_texture =
            Self::create_depth_texture(renderer.context().device(), size.0, size.1, sample_count);

        let msaa_view = msaa_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let render_pass = wgpurenderer::renderpass::RenderPass {
            render_target: wgpurenderer::RenderTarget {
                color_attachments: smallvec::smallvec![wgpurenderer::ColorAttachment {
                    view: msaa_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                }],
                depth_stencil_attachment: Some(wgpurenderer::DepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
            },
            multiview_mask: None,
            draw_calls: vec![draw_call],
            executor: None,
        };

        Self {
            size,
            renderer,
            msaa_texture,
            depth_texture,
            sample_count,
            uniform_buffer,
            render_pass,
            rotation: 0.0,
            start_time: std::time::Instant::now(),
        }
    }

    fn create_msaa_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        sample_count: u32,
    ) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("MSAA Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
    }

    fn create_depth_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        sample_count: u32,
    ) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
    }

    fn resize(&mut self, new_size: (u32, u32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.size = new_size;
            self.renderer.resize(self.size.0, self.size.1);

            self.msaa_texture = Self::create_msaa_texture(
                self.renderer.context().device(),
                self.size.0,
                self.size.1,
                self.renderer.surface_format(),
                self.sample_count,
            );
            self.depth_texture = Self::create_depth_texture(
                self.renderer.context().device(),
                self.size.0,
                self.size.1,
                self.sample_count,
            );
            self.render_pass.render_target.color_attachments[0].view =
                self.msaa_texture.create_view(&wgpu::TextureViewDescriptor::default());
            self.render_pass.render_target.depth_stencil_attachment.as_mut().unwrap().view =
                self.depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        }
    }

    fn update(&mut self) {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        self.rotation = elapsed;

        let axis = glam::Vec3::new(0.5, 1.0, 0.0).normalize();
        let model = glam::Mat4::from_axis_angle(axis, self.rotation);
        let view = glam::Mat4::look_at_rh(
            glam::Vec3::new(0.0, 0.0, 3.0),
            glam::Vec3::new(0.0, 0.0, 0.0),
            glam::Vec3::new(0.0, 1.0, 0.0),
        );
        let aspect = self.size.0 as f32 / self.size.1 as f32;
        let projection = glam::Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 0.1, 100.0);

        let uniforms = Uniforms {
            model: model.to_cols_array_2d(),
            view: view.to_cols_array_2d(),
            projection: projection.to_cols_array_2d(),
            light_color: [1.0, 1.0, 0.9],
            _padding: 0.0,
        };

        self.renderer.context().queue().write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::bytes_of(&uniforms),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.renderer.acquire()?;

        self.render_pass.render_target.color_attachments[0].resolve_target =
            Some(self.renderer.surface_view.as_ref().unwrap().clone());

        self.renderer.render(&mut [&mut self.render_pass]);
        self.renderer.present();

        Ok(())
    }
}

fn main() {
    env_logger::init();
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

    glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
    glfw.window_hint(glfw::WindowHint::Resizable(true));

    let (mut window, events) = glfw
        .create_window(800, 600, "WGPU Rotating Cube", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    let context = window.render_context();
    let mut state = pollster::block_on(State::new(context));

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                }
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    state.resize((width as u32, height as u32));
                }
                _ => {}
            }
        }

        state.update();

        match state.render() {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
            Err(wgpu::SurfaceError::OutOfMemory) => {
                eprintln!("Out of memory!");
                break;
            }
            Err(e) => eprintln!("{:?}", e),
        }
    }
}
