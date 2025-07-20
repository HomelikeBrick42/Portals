use eframe::{egui, wgpu};
use math::Transform;
use ray_tracing::{GpuCamera, RayTracingPaintCallback, RayTracingRenderer};
use std::time::Instant;

struct App {
    last_time: Option<Instant>,
    info_window_open: bool,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let render_state = cc.wgpu_render_state.as_ref().unwrap();
        let ray_tracer = RayTracingRenderer::new(&render_state.device, &render_state.queue);
        render_state
            .renderer
            .write()
            .callback_resources
            .insert(ray_tracer);

        Self {
            last_time: None,
            info_window_open: true,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let time = Instant::now();
        let dt = time - self.last_time.unwrap_or(time);
        self.last_time = Some(time);

        egui::TopBottomPanel::top("Windows").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.info_window_open |= ui.button("Info").clicked();
            });
        });

        egui::Window::new("Info")
            .open(&mut self.info_window_open)
            .show(ctx, |ui| {
                ui.label(format!("FPS: {:.3}", 1.0 / dt.as_secs_f64()));
                ui.label(format!("Frame Time: {:.3}ms", dt.as_secs_f64() * 1000.0));
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(255, 0, 255)))
            .show(ctx, |ui| {
                let (rect, _response) =
                    ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());

                ui.painter()
                    .add(eframe::egui_wgpu::Callback::new_paint_callback(
                        rect,
                        RayTracingPaintCallback {
                            width: rect.width() as u32,
                            height: rect.height() as u32,
                            camera: GpuCamera {
                                transform: Transform::IDENTITY,
                            },
                        },
                    ));
            });

        ctx.request_repaint();
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Portals",
        eframe::NativeOptions {
            vsync: false,
            renderer: eframe::Renderer::Wgpu,
            wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
                present_mode: wgpu::PresentMode::AutoNoVsync,
                ..Default::default()
            },
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
