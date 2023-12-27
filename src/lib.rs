use wgpu::{
    BindGroup, Buffer, Device, Instance, Queue, RenderPipeline, Surface, SurfaceConfiguration,
    Texture,
};
use winit::{dpi::PhysicalSize, window::Window};

pub mod camera;

pub struct RenderState<'w> {
    surface: Surface,
    device: Device,
    queue: Queue,
    surface_config: SurfaceConfiguration,
    surface_size: PhysicalSize<u32>,
    render_pipeline: RenderPipeline,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    instances: Vec<Instance>,
    instance_buffer: Buffer,
    depth_texture: Texture,
    models: Vec<Model>,
    lights: Vec<LightUniform>,
    light_buffer: Buffer,
    light_bind_group: BindGroup,
    light_render_pipeline: RenderPipeline,
    window: &'w Window,
}
