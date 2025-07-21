use eframe::{egui, wgpu};
use math::{Transform, Vector3};
use ray_tracing::{Color, GpuCamera, RayTracingPaintCallback, RayTracingRenderer};
use std::{f32::consts::PI, time::Instant};

struct App {
    last_time: Option<Instant>,
    info_window_open: bool,
    camera_window_open: bool,
    up_sky_color: Color,
    down_sky_color: Color,
    sun_size: f32,
    sun_color: Color,
    sun_light_color: Color,
    sun_direction: Vector3,
    ambient_color: Color,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let render_state = cc.wgpu_render_state.as_ref().unwrap();
        let ray_tracer = RayTracingRenderer::new(
            &render_state.device,
            &render_state.queue,
            render_state.target_format,
        );
        render_state
            .renderer
            .write()
            .callback_resources
            .insert(ray_tracer);

        Self {
            last_time: None,
            info_window_open: true,
            camera_window_open: true,
            up_sky_color: Color {
                r: 0.4,
                g: 0.5,
                b: 0.8,
            },
            down_sky_color: Color {
                r: 0.4,
                g: 0.4,
                b: 0.4,
            },
            sun_size: 6.0f32.to_radians(),
            sun_color: Color {
                r: 1.0,
                g: 0.8,
                b: 0.1,
            },
            sun_light_color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
            },
            sun_direction: Vector3 {
                x: 0.4,
                y: 1.0,
                z: 0.2,
            },
            ambient_color: Color {
                r: 0.1,
                g: 0.1,
                b: 0.1,
            },
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
                self.camera_window_open |= ui.button("Camera").clicked();
            });
        });

        egui::Window::new("Info")
            .open(&mut self.info_window_open)
            .show(ctx, |ui| {
                ui.label(format!("FPS: {:.3}", 1.0 / dt.as_secs_f64()));
                ui.label(format!("Frame Time: {:.3}ms", dt.as_secs_f64() * 1000.0));
            });

        egui::Window::new("Camera")
            .open(&mut self.camera_window_open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Up Sky Color:");
                    ui.color_edit_button_rgb(self.up_sky_color.as_mut());
                });
                ui.horizontal(|ui| {
                    ui.label("Down Sky Color:");
                    ui.color_edit_button_rgb(self.down_sky_color.as_mut());
                });
                ui.horizontal(|ui| {
                    ui.label("Sun Angular Radius:");
                    ui.drag_angle(&mut self.sun_size);
                    self.sun_size = self.sun_size.clamp(0.0, PI);
                });
                ui.horizontal(|ui| {
                    ui.label("Sun Color:");
                    ui.color_edit_button_rgb(self.sun_color.as_mut());
                });
                ui.horizontal(|ui| {
                    ui.label("Sun Light Color:");
                    ui.color_edit_button_rgb(self.sun_light_color.as_mut());
                });
                ui.horizontal(|ui| {
                    ui.label("Sun Direction:");
                    ui_vector3(ui, &mut self.sun_direction);
                });
                ui.horizontal(|ui| {
                    ui.label("Ambient Color:");
                    ui.color_edit_button_rgb(self.ambient_color.as_mut());
                });
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
                                up_sky_color: self.up_sky_color,
                                down_sky_color: self.down_sky_color,
                                sun_size: self.sun_size,
                                sun_color: self.sun_color,
                                sun_light_color: self.sun_light_color,
                                sun_direction: self.sun_direction,
                                ambient_color: self.ambient_color,
                            },
                        },
                    ));
            });

        ctx.request_repaint();
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}
}

fn ui_vector3(ui: &mut egui::Ui, v: &mut Vector3) -> egui::Response {
    ui.add(egui::DragValue::new(&mut v.x).prefix("x:").speed(0.1))
        | ui.add(egui::DragValue::new(&mut v.y).prefix("y:").speed(0.1))
        | ui.add(egui::DragValue::new(&mut v.z).prefix("z:").speed(0.1))
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
