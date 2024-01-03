mod model;
mod resources;
mod texture;
mod camera;

use model::Vertex;
use model::DrawModel;

use sdl2::{
    event::{
        Event,
        WindowEvent
    },
    keyboard::Keycode
};
use wgpu::util::DeviceExt;
use cgmath::{
    Rotation3,
    Zero,
    InnerSpace
};
use cgmath::{
    Matrix4,
    Matrix3,
    Vector3,
    Point3,
    Quaternion,
    Deg
};
use std::{
    time::{
        Duration,
        Instant
    },
    thread
};

struct Instance {
    position: Vector3<f32>,
    rotation: Quaternion<f32>
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            transform_matrix: (Matrix4::from_translation(self.position) * Matrix4::from(self.rotation)).into(),
            normal: Matrix3::from(self.rotation).into()
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    transform_matrix: [[f32; 4]; 4],
    normal: [[f32; 3]; 3]
}

impl InstanceRaw {
    const BUFFER_LAYOUT_ATTRIBS: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Float32x4, 7 => Float32x3, 8 => Float32x3, 9 => Float32x3];
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::BUFFER_LAYOUT_ATTRIBS
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct LightRaw {
    position: [f32; 3],
    _padding0: u32,
    color: [f32; 3],
    _padding1: u32
}

struct State {
    // event_pump: sdl2::EventPump,
    sdl_context: sdl2::Sdl,
    _window: sdl2::video::Window,
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    instance_buffer: wgpu::Buffer,
    instances: Vec<Instance>,
    obj_model: model::Model,
    // texture_bind_group: wgpu::BindGroup,
    depth_texture: texture::Texture,
    camera_controller: camera::CameraController,
    camera: camera::Camera,
    camera_proj: camera::CameraProjection,
    camera_proj_raw: camera::CameraProjectionRaw,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    light_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    last_instant: Instant,
    deltatime: Duration,
    running: bool
}

impl State {
    async fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        // let event_pump = sdl_context.event_pump();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("rust-sdl2 demo", 1000, 800)
            .position_centered()
            .resizable()
            .vulkan()
            .build()
            .unwrap();

        let (window_width, window_height) = window.size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface)
            })
            .await
            .expect("No adapter found");
        
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("device"),
                features: wgpu::Features::POLYGON_MODE_LINE,
                limits: wgpu::Limits::default()
            }, None)
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let texture_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: texture_format,
            width: window_width,
            height: window_height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![]
        };
        surface.configure(&device, &surface_config);

        // -----------------------------

        let instances = vec![
            Instance {
                position: Vector3::new(0.0, 0.0, 0.0),
                rotation: Quaternion::from_angle_y(Deg(0.0))
            }
        ];
        // let instances: Vec<Instance> = (0..10).flat_map(|z| (0..10).map(move |x| {
        //     let x = 4.0 * (x as f32 - 5.0);
        //     let z = 4.0 * (z as f32 - 5.0);
        //     let position = Vector3 { x, y: 0.0, z };
        //     let rotation = if position.is_zero() {
        //         Quaternion::from_axis_angle(
        //             Vector3::unit_z(),
        //             Deg(0.0)
        //         )
        //     } else {
        //         Quaternion::from_axis_angle(position.normalize(), Deg(45.0))
        //     };

        //     Instance { position, rotation }
        // })).collect();
        let raw_instances: Vec<InstanceRaw> = instances.iter().map(Instance::to_raw).collect();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instance_buffer"),
            contents: bytemuck::cast_slice(&raw_instances),
            usage: wgpu::BufferUsages::VERTEX
        });

        // let texture = texture::Texture::from_image_bytes(include_bytes!("dirt.jpg"), "dirt.jpg", &device, &queue);

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None
                }
            ]
        });

        let obj_model = resources::load_model("teapot.obj", &texture_bind_group_layout, &device, &queue).unwrap();

        // let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     label: Some("texture_bind_group"),
        //     layout: &texture_bind_group_layout,
        //     entries: &[
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: wgpu::BindingResource::TextureView(&texture.view)
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 1,
        //             resource: wgpu::BindingResource::Sampler(&texture.sampler)
        //         }
        //     ]
        // });
        
        let depth_texture = texture::Texture::new_depth_texture(window_width, window_height, &device);

        let camera_controller = camera::CameraController::new(2.0, 2.0);
        let camera = camera::Camera::new(
            Point3::new(0.0, 0.0, -5.0),
            Deg(90.0),
            Deg(0.0)
        );
        let camera_proj = camera::CameraProjection::new(
            Deg(45.0),
            window_width as f32,
            window_height as f32,
            0.1,
            100.0
        );
        let mut camera_proj_raw = camera::CameraProjectionRaw::new();
        camera_proj_raw.update_proj_matrix(&camera_proj, &camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera_buffer"),
            contents: bytemuck::cast_slice(&[camera_proj_raw]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        });

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None
                },
                count: None
            }]
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding()
            }]
        });

        let light_raw = LightRaw {
            position: [2.0, 2.0, 2.0],
            _padding0: 0,
            color: [1.0, 1.0, 1.0],
            _padding1: 0
        };
        
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light_buffer"),
            contents: bytemuck::cast_slice(&[light_raw]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        });

        let light_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("light_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None
                },
                count: None
            }]
        });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("light_bind_group"),
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding()
            }]
        });

        // -----------------------------------------

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &camera_bind_group_layout,
                &light_bind_group_layout
            ],
            push_constant_ranges: &[]
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[model::ModelVertex::desc(), InstanceRaw::desc()]
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_TEXTURE_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default()
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::all()
                })]
            }),
            multiview: None
        });

        Self {
            // event_pump,
            sdl_context,
            _window: window,
            surface,
            surface_config,
            device,
            queue,
            // vertex_buffer,
            // index_buffer,
            instance_buffer,
            instances,
            obj_model,
            // texture_bind_group,
            depth_texture,
            camera_controller,
            camera,
            camera_proj,
            camera_proj_raw,
            camera_buffer,
            camera_bind_group,
            light_bind_group,
            render_pipeline,
            // num_indices,
            last_instant: Instant::now(),
            deltatime: Duration::ZERO,
            running: false
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;

        let output = frame.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("surface_texture_view"),
            ..Default::default()
        });

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("command_encoder") });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {r: 0.5, g: 0.5, b: 0.5, a: 1.0}),
                        store: wgpu::StoreOp::Store
                    }
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store
                    }),
                    stencil_ops: None
                }),
                timestamp_writes: None,
                occlusion_query_set: None
            });
            render_pass.set_pipeline(&self.render_pipeline);
            // render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(2, &self.light_bind_group, &[]);
            // render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            // render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            // render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len().try_into().unwrap());
            render_pass.draw_model(&self.obj_model, 0..self.instances.len().try_into().unwrap());
        }
        
        self.queue.submit([encoder.finish()]);
        frame.present();

        Ok(())
    }

    fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera, &self.deltatime);
        self.camera_proj_raw.update_proj_matrix(&self.camera_proj, &self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_proj_raw]));
    }

    fn input(&mut self) {
        let mut event_pump = self.sdl_context.event_pump().unwrap();

        for event in event_pump.poll_iter() {
            match event {
                Event::Window { win_event: WindowEvent::Resized(width, height), .. } => {
                    self.resize(width.try_into().unwrap(), height.try_into().unwrap());
                    // (self.surface_config.width, self.surface_config.height) = (width.try_into().unwrap(), height.try_into().unwrap());
                    // self.surface.configure(&self.device, &self.surface_config);
                },

                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    self.running = false;
                    break;
                },

                _ => { self.camera_controller.process_event(event); }
            }
        }
    }
    
    fn resize(&mut self, width: u32, height: u32) {
        (self.surface_config.width, self.surface_config.height) = (width, height);
        self.surface.configure(&self.device, &self.surface_config);

        self.camera_proj.resize(width as f32, height as f32);
        self.depth_texture = texture::Texture::new_depth_texture(width, height, &self.device)
    }

    fn run(&mut self) {
        self.running = true;
        while self.running {
            self.input();
            self.update();
            self.render().unwrap();

            self.deltatime = self.last_instant.elapsed();
            self.last_instant = Instant::now();
            thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }
}

pub async fn run() {
    let mut state = State::new().await;

    state.run();
}
