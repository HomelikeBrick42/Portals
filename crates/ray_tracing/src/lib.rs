use eframe::wgpu;
use encase::{ShaderSize, ShaderType};
use math::Transform;

mod color;

pub use color::*;

#[derive(Debug, Clone, Copy, ShaderType)]
pub struct GpuCamera {
    pub transform: Transform,
    pub up_sky_color: Color,
    pub down_sky_color: Color,
    pub sun_color: Color,
    pub sun_light_color: Color,
}

#[derive(Debug, Clone, Copy, ShaderType)]
pub struct GpuSceneInfo {
    pub camera: GpuCamera,
    pub aspect: f32,
}

pub struct RayTracingRenderer {
    scene_info_buffer: wgpu::Buffer,
}

impl RayTracingRenderer {
    pub fn new(device: &wgpu::Device, _queue: &wgpu::Queue) -> Self {
        let scene_info_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Scene Info"),
            size: GpuSceneInfo::SHADER_SIZE.get(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self { scene_info_buffer }
    }
}

pub struct RayTracingPaintCallback {
    pub width: u32,
    pub height: u32,
    pub camera: GpuCamera,
}

impl eframe::egui_wgpu::CallbackTrait for RayTracingPaintCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let renderer: &mut RayTracingRenderer = callback_resources.get_mut().unwrap();

        {
            let scene_info = GpuSceneInfo {
                camera: self.camera,
                aspect: self.width as f32 / self.height as f32,
            };

            let mut scene_info_buffer = queue
                .write_buffer_with(&renderer.scene_info_buffer, 0, GpuSceneInfo::SHADER_SIZE)
                .unwrap();
            encase::UniformBuffer::new(&mut *scene_info_buffer)
                .write(&scene_info)
                .unwrap();
        }

        vec![]
    }

    fn paint(
        &self,
        _info: eframe::egui::PaintCallbackInfo,
        _render_pass: &mut wgpu::RenderPass<'static>,
        callback_resources: &eframe::egui_wgpu::CallbackResources,
    ) {
        let _renderer: &RayTracingRenderer = callback_resources.get().unwrap();
    }
}
