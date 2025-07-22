use eframe::{egui, wgpu};
use math::{Rotor, Transform, Vector3};
use ray_tracing::{Color, GpuCamera, GpuPlane, RayTracingPaintCallback, RayTracingRenderer};
use std::{
    f32::consts::{PI, TAU},
    time::Instant,
};

pub struct Plane {
    pub position: Vector3,
    pub xy_rotation: f32,
    pub yz_rotation: f32,
    pub xz_rotation: f32,
    pub color: Color,
    pub width: f32,
    pub height: f32,
    pub checker_count: u32,
    pub checker_darkness: f32,
}

impl Plane {
    pub fn to_gpu(&self) -> GpuPlane {
        let Self {
            position,
            xy_rotation,
            yz_rotation,
            xz_rotation,
            color,
            width,
            height,
            checker_count,
            checker_darkness,
        } = *self;
        GpuPlane {
            transform: Transform::translation(position).then(Transform::from_rotor(
                Rotor::rotation_xy(xy_rotation)
                    .then(Rotor::rotation_yz(yz_rotation))
                    .then(Rotor::rotation_xz(xz_rotation)),
            )),
            color,
            width,
            height,
            checker_count,
            checker_darkness,
        }
    }
}

struct App {
    last_time: Option<Instant>,
    info_window_open: bool,
    camera_window_open: bool,
    camera_transform: Transform,
    camera_speed: f32,
    camera_rotation_speed: f32,
    up_sky_color: Color,
    down_sky_color: Color,
    sun_size: f32,
    sun_color: Color,
    sun_light_color: Color,
    sun_direction: Vector3,
    ambient_color: Color,
    planes: Vec<Plane>,
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
            camera_transform: Transform::IDENTITY,
            camera_speed: 2.0,
            camera_rotation_speed: 0.25,
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
            planes: vec![Plane {
                position: Vector3 {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                },
                xy_rotation: 0.0,
                yz_rotation: 0.0,
                xz_rotation: 0.0,
                color: Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                },
                width: 10.0,
                height: 10.0,
                checker_count: 10,
                checker_darkness: 0.5,
            }],
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let time = Instant::now();
        let dt = time - self.last_time.unwrap_or(time);
        self.last_time = Some(time);

        let ts = dt.as_secs_f32();

        egui::TopBottomPanel::top("Windows").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.info_window_open |= ui.button("Info").clicked();
                self.camera_window_open |= ui.button("Camera").clicked();
            });
        });

        egui::Window::new("Info")
            .resizable(false)
            .open(&mut self.info_window_open)
            .show(ctx, |ui| {
                ui.label(format!("FPS: {:.3}", 1.0 / dt.as_secs_f64()));
                ui.label(format!("Frame Time: {:.3}ms", dt.as_secs_f64() * 1000.0));
            });

        egui::Window::new("Camera")
            .open(&mut self.camera_window_open)
            .scroll(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Position:");
                    let original = self.camera_transform.transform_point(Vector3::ZERO);
                    let mut position = original;
                    if ui_vector3(ui, &mut position).changed() {
                        self.camera_transform =
                            Transform::translation(position - original).then(self.camera_transform);
                    }
                });
                ui.add_enabled_ui(false, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Forward:");
                        let mut forward =
                            self.camera_transform.rotor_part().rotate(Vector3::FORWARD);
                        ui_vector3(ui, &mut forward);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Up:");
                        let mut up = self.camera_transform.rotor_part().rotate(Vector3::UP);
                        ui_vector3(ui, &mut up);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Right:");
                        let mut right = self.camera_transform.rotor_part().rotate(Vector3::RIGHT);
                        ui_vector3(ui, &mut right);
                    });
                });
                ui.collapsing("Transform", |ui| {
                    ui.add_enabled_ui(false, |ui| {
                        ui_transform(ui, &mut self.camera_transform);
                    });
                });
                ui.horizontal(|ui| {
                    ui.label("Camera Speed:");
                    ui.add(egui::DragValue::new(&mut self.camera_speed).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("Camera Rotation Speed:");
                    ui.add(egui::DragValue::new(&mut self.camera_rotation_speed).speed(0.1));
                });
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

        ctx.input(|i| {
            {
                let forward = i.key_down(egui::Key::W) as u8 as f32;
                let backward = i.key_down(egui::Key::S) as u8 as f32;
                let up = i.key_down(egui::Key::E) as u8 as f32;
                let down = i.key_down(egui::Key::Q) as u8 as f32;
                let left = i.key_down(egui::Key::A) as u8 as f32;
                let right = i.key_down(egui::Key::D) as u8 as f32;

                let boost = i.modifiers.shift as u8 as f32 + 1.0;

                let movement = Vector3 {
                    x: forward - backward,
                    y: up - down,
                    z: right - left,
                }
                .normalised();

                self.camera_transform = self.camera_transform.then(Transform::translation(
                    movement * self.camera_speed * boost * ts,
                ));
            }

            {
                let up = i.key_down(egui::Key::ArrowUp) as u8 as f32;
                let down = i.key_down(egui::Key::ArrowDown) as u8 as f32;
                let left = i.key_down(egui::Key::ArrowLeft) as u8 as f32;
                let right = i.key_down(egui::Key::ArrowRight) as u8 as f32;

                let vertical = up - down;
                self.camera_transform = self.camera_transform.then(Transform::rotation_xy(
                    vertical * self.camera_rotation_speed * TAU * ts,
                ));

                if i.modifiers.shift {
                    let roll = right - left;
                    self.camera_transform = self.camera_transform.then(Transform::rotation_yz(
                        roll * self.camera_rotation_speed * TAU * ts,
                    ));
                } else {
                    let horizontal = right - left;
                    self.camera_transform = self.camera_transform.then(Transform::rotation_xz(
                        horizontal * self.camera_rotation_speed * TAU * ts,
                    ));
                }
            }
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
                                transform: self.camera_transform,
                                up_sky_color: self.up_sky_color,
                                down_sky_color: self.down_sky_color,
                                sun_size: self.sun_size,
                                sun_color: self.sun_color,
                                sun_light_color: self.sun_light_color,
                                sun_direction: self.sun_direction,
                                ambient_color: self.ambient_color,
                            },
                            planes: self.planes.iter().map(Plane::to_gpu).collect(),
                        },
                    ));
            });

        ctx.request_repaint();
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}
}

fn ui_transform(
    ui: &mut egui::Ui,
    Transform {
        s,
        e12,
        e13,
        e23,
        e01,
        e02,
        e03,
        e0123,
    }: &mut Transform,
) -> egui::Response {
    ui.add(egui::DragValue::new(s).prefix("s:").speed(0.1))
        | ui.add(egui::DragValue::new(e12).prefix("e12:").speed(0.1))
        | ui.add(egui::DragValue::new(e13).prefix("e13:").speed(0.1))
        | ui.add(egui::DragValue::new(e23).prefix("e23:").speed(0.1))
        | ui.add(egui::DragValue::new(e01).prefix("e01:").speed(0.1))
        | ui.add(egui::DragValue::new(e02).prefix("e02:").speed(0.1))
        | ui.add(egui::DragValue::new(e03).prefix("e03:").speed(0.1))
        | ui.add(egui::DragValue::new(e0123).prefix("e0123:").speed(0.1))
}

fn ui_vector3(ui: &mut egui::Ui, Vector3 { x, y, z }: &mut Vector3) -> egui::Response {
    ui.add(egui::DragValue::new(x).prefix("x:").speed(0.1))
        | ui.add(egui::DragValue::new(y).prefix("y:").speed(0.1))
        | ui.add(egui::DragValue::new(z).prefix("z:").speed(0.1))
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
