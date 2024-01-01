use camera::{Camera, CameraUniform};
use glam::Vec3;
use instance::{Instance, InstanceRaw};
use light::LightUniform;
use model::{DrawLight, DrawModel, Model, Vertex};
use resources::load_model;
use texture::Texture;
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

pub mod camera;
pub mod instance;
pub mod light;
mod model;
mod resources;
mod texture;

/// This holds all the required information for rendering the scene.
pub struct RenderState {
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    models: Vec<Option<Model>>,
    instance_buffers: Vec<Option<wgpu::Buffer>>,
    recreate_instance_buffers: Vec<usize>,
    depth_texture: Texture,
    // these are temporary
    texture_bind_group_layout: wgpu::BindGroupLayout,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_bind_group: wgpu::BindGroup,
    light_bind_group: wgpu::BindGroup,
    //---
    render_pipelines: Vec<wgpu::RenderPipeline>,
    _compute_pipelines: Vec<wgpu::ComputePipeline>,
}

impl RenderState {
    pub async fn new(window: &Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });

        let surface = unsafe { instance.create_surface(window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        let models = vec![];

        let instance_buffers = vec![];
        let recreate_instance_buffers = vec![];

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &surface_config, "depth_texture");

        let camera = Camera {
            // position the camera 1 unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 20.0, 0.01).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: Vec3::Y,
            aspect: surface_config.width as f32 / surface_config.height as f32,
            fovy: 96.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_projection(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let light_uniform = LightUniform {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
        };

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &render_pipeline_layout,
                surface_config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc(), InstanceRaw::desc()],
                shader,
            )
        };

        let light_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Light Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("light.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &layout,
                surface_config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc()],
                shader,
            )
        };

        let render_pipelines = vec![render_pipeline, light_render_pipeline];
        let _compute_pipelines = vec![];

        Self {
            surface,
            surface_config,
            device,
            queue,
            models,
            instance_buffers,
            recreate_instance_buffers,
            depth_texture,
            texture_bind_group_layout,
            camera,
            camera_uniform,
            camera_bind_group,
            light_bind_group,
            render_pipelines,
            _compute_pipelines,
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.surface_config.width = size.width;
            self.surface_config.height = size.height;
            self.surface.configure(&self.device, &self.surface_config);
            self.depth_texture = texture::Texture::create_depth_texture(
                &self.device,
                &self.surface_config,
                "depth_texture",
            );
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        if !self.recreate_instance_buffers.is_empty() {
            for i in &self.recreate_instance_buffers {
                let model = self.models[*i].as_mut().unwrap();

                let instance_data = model
                    .instances
                    .iter()
                    .map(Instance::to_raw)
                    .collect::<Vec<_>>();

                self.instance_buffers[*i] = Some(self.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Instance Buffer"),
                        contents: bytemuck::cast_slice(&instance_data),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    },
                ));
            }
        }

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // TODO: *IMPROVEMENTS MUST BE MADE*
            let mut index = 0_usize;
            self.models.iter().for_each(|model| {
                if model.is_some() {
                    let model = model.as_ref().unwrap();
                    render_pass.set_vertex_buffer(
                        1,
                        self.instance_buffers[index].as_ref().unwrap().slice(..),
                    );
                    render_pass.set_pipeline(&self.render_pipelines[1]);
                    render_pass.draw_light_model(
                        model,
                        &self.camera_bind_group,
                        &self.light_bind_group,
                    );
                    render_pass.set_pipeline(&self.render_pipelines[0]);
                    render_pass.draw_model_instanced(
                        model,
                        0..model.instances.len() as u32,
                        &self.camera_bind_group,
                        &self.light_bind_group,
                    );
                }

                index += 1;
            });
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// This can be used for adding custom shaders using a [`wgpu::RenderPipelineDescriptor`].
    pub fn add_render_pipeline(&mut self, desc: &wgpu::RenderPipelineDescriptor) {
        let render_pipeline = self.device.create_render_pipeline(desc);
        self.render_pipelines.push(render_pipeline);
    }

    /// Adds a [`Model`] and returns it's id
    pub async fn load_model_instanced(
        &mut self,
        model_file: &str,
        instances: Vec<Instance>,
    ) -> usize {
        let model = load_model(
            model_file,
            &self.device,
            &self.queue,
            &self.texture_bind_group_layout,
            instances,
        )
        .await
        .unwrap();

        let instance_data = model
            .instances
            .iter()
            .map(Instance::to_raw)
            .collect::<Vec<_>>();

        let instance_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        if self.models.iter().any(|option| option.is_none()) {
            let index = self
                .models
                .iter()
                .position(|option| option.is_none())
                .unwrap();

            self.models[index] = Some(model);
            self.instance_buffers[index] = Some(instance_buffer);

            index
        } else {
            let index = self.models.len();
            self.models.push(Some(model));

            self.instance_buffers.push(Some(instance_buffer));

            index
        }
    }

    /// Remove a [`Model`] from the [`RenderState`]
    pub fn remove_model(&mut self, model_id: usize) {
        self.models[model_id] = None;
        self.instance_buffers[model_id] = None;
    }

    pub fn push_instance(&mut self, model_id: usize, instance: Instance) -> usize {
        let model = self.models[model_id].as_mut().unwrap();
        let index = model.instances.len();
        model.instances.push(instance);

        if !self
            .recreate_instance_buffers
            .iter()
            .any(|id| id == &model_id)
        {
            self.recreate_instance_buffers.push(model_id);
        }

        index
    }

    /// This will remove the requested [`Instance`], however that also means the [`Instance`] [`Vec`] for that [`Model`] will shrink.
    /// That means any indexes your application uses that are larger than the requested index *will have to be subtracted by 1*.
    // TODO: Maybe use Hashmaps in the future
    pub fn remove_instance(&mut self, model_id: usize, instance_id: usize) {
        self.models[model_id]
            .as_mut()
            .unwrap()
            .instances
            .remove(instance_id);

        if !self
            .recreate_instance_buffers
            .iter()
            .any(|id| id == &model_id)
        {
            self.recreate_instance_buffers.push(model_id);
        }
    }

    /// Override the specified instance.
    pub fn override_instance(
        &mut self,
        model_id: usize,
        instance_id: usize,
        instance_override: Instance,
    ) {
        self.queue.write_buffer(
            self.instance_buffers[model_id].as_ref().unwrap(),
            instance_id as u64 * 100,
            bytemuck::cast_slice(&[instance_override.to_raw()]),
        );
        self.models[model_id].as_mut().unwrap().instances[instance_id] = instance_override;
    }

    /// Returns a reference to the requested [`Instance`]. To modify an [`Instance`] use `override_instance()`.
    pub fn get_instance(&self, model_id: usize, instance_id: usize) -> &Instance {
        &self.models[model_id].as_ref().unwrap().instances[instance_id]
    }
}

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: vertex_layouts,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState {
                    alpha: wgpu::BlendComponent::REPLACE,
                    color: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}
