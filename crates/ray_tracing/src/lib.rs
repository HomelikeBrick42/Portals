use eframe::wgpu;

pub struct RayTracingRenderer {}

impl RayTracingRenderer {
    pub fn new(_device: &wgpu::Device, _queue: &wgpu::Queue) -> Self {
        Self {}
    }
}

pub struct RayTracingPaintCallback {
    pub width: u32,
    pub height: u32,
}

impl eframe::egui_wgpu::CallbackTrait for RayTracingPaintCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let _renderer: &mut RayTracingRenderer = callback_resources.get_mut().unwrap();

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
