use eframe::{egui, wgpu};
use egui_file_dialog::FileDialog;
use math::{Rotor, Transform, Vector3};
use ray_tracing::{Color, GpuCamera, RayTracingPaintCallback, RayTracingRenderer};
use serde::{Deserialize, Serialize};
use std::{f32::consts::PI, sync::Arc, time::Instant};

mod camera;
mod plane;
mod ray;

pub use camera::*;
pub use plane::*;
pub use ray::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct State {
    info_window_open: bool,
    camera_window_open: bool,
    camera: Camera,
    up_sky_color: Color,
    down_sky_color: Color,
    sun_size: f32,
    sun_color: Color,
    sun_light_color: Color,
    sun_direction: Vector3,
    ambient_color: Color,
    recursive_portal_count: u32,
    planes_window_open: bool,
    planes: Vec<Plane>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            info_window_open: true,
            camera_window_open: true,
            camera: Camera {
                position: Vector3::UP * 1.1,
                rotation: Rotor::IDENTITY,
                speed: 2.0,
                rotation_speed: 0.25,
            },
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
                r: 0.3,
                g: 0.3,
                b: 0.3,
            },
            recursive_portal_count: 10,
            planes_window_open: true,
            planes: vec![Plane {
                name: "Ground".into(),
                position: Vector3 {
                    x: 0.0,
                    y: 0.0,
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
                checker_count_x: 10,
                checker_count_z: 10,
                checker_darkness: 0.5,
                front_portal: PortalConnection::default(),
                back_portal: PortalConnection::default(),
            }],
        }
    }
}

struct App {
    last_time: Option<Instant>,
    state: State,
    file_dialog: FileDialog,
    file_interaction: FileInteraction,
    accumulated_frames: u32,
}

enum FileInteraction {
    None,
    Save,
    Load,
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
            state: cc
                .storage
                .and_then(|storage| storage.get_string("State"))
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default(),
            file_dialog: FileDialog::new()
                .add_file_filter_extensions("Scene", vec!["scene"])
                .default_file_filter("Scene")
                .add_save_extension("Scene", "scene")
                .default_save_extension("Scene"),
            file_interaction: FileInteraction::None,
            accumulated_frames: 0,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let time = Instant::now();
        let dt = time - self.last_time.unwrap_or(time);
        self.last_time = Some(time);

        let ts = dt.as_secs_f32();

        {
            let mut reset_everything = false;
            egui::TopBottomPanel::top("Windows").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    reset_everything |= ui.button("RESET EVERYTHING").clicked();
                    if ui.button("Load").clicked() {
                        self.file_interaction = FileInteraction::Load;
                        self.file_dialog.pick_file();
                    }
                    if ui.button("Save").clicked() {
                        self.file_interaction = FileInteraction::Save;
                        self.file_dialog.save_file();
                    }
                    self.state.info_window_open |= ui.button("Info").clicked();
                    self.state.camera_window_open |= ui.button("Camera").clicked();
                    self.state.planes_window_open |= ui.button("Planes").clicked();
                });
            });
            if reset_everything {
                self.state = State::default();
                self.accumulated_frames = 0;
            }
        }

        egui::Window::new("Info")
            .resizable(false)
            .open(&mut self.state.info_window_open)
            .show(ctx, |ui| {
                ui.label(format!("FPS: {:.3}", 1.0 / dt.as_secs_f64()));
                ui.label(format!("Frame Time: {:.3}ms", dt.as_secs_f64() * 1000.0));
            });

        egui::Window::new("Camera")
            .open(&mut self.state.camera_window_open)
            .scroll(true)
            .show(ctx, |ui| {
                if self.state.camera.ui(ui) {
                    self.accumulated_frames = 0;
                }
                ui.horizontal(|ui| {
                    ui.label("Accumulated Frames:");
                    ui.add_enabled(false, egui::DragValue::new(&mut self.accumulated_frames));
                    if ui.button("Clear").clicked() {
                        self.accumulated_frames = 0;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Up Sky Color:");
                    ui.color_edit_button_rgb(self.state.up_sky_color.as_mut());
                });
                ui.horizontal(|ui| {
                    ui.label("Down Sky Color:");
                    ui.color_edit_button_rgb(self.state.down_sky_color.as_mut());
                });
                ui.horizontal(|ui| {
                    ui.label("Sun Angular Radius:");
                    ui.drag_angle(&mut self.state.sun_size);
                    self.state.sun_size = self.state.sun_size.clamp(0.0, PI);
                });
                ui.horizontal(|ui| {
                    ui.label("Sun Color:");
                    ui.color_edit_button_rgb(self.state.sun_color.as_mut());
                });
                ui.horizontal(|ui| {
                    ui.label("Sun Light Color:");
                    ui.color_edit_button_rgb(self.state.sun_light_color.as_mut());
                });
                ui.horizontal(|ui| {
                    ui.label("Sun Direction:");
                    ui_vector3(ui, &mut self.state.sun_direction);
                });
                ui.horizontal(|ui| {
                    ui.label("Ambient Color:");
                    ui.color_edit_button_rgb(self.state.ambient_color.as_mut());
                });
                ui.horizontal(|ui| {
                    ui.label("Recursive Portal Count:");
                    ui.add(egui::DragValue::new(&mut self.state.recursive_portal_count));
                });
            });

        egui::Window::new("Planes")
            .open(&mut self.state.planes_window_open)
            .scroll(true)
            .show(ctx, |ui| {
                if ui.button("New Plane").clicked() {
                    self.state.planes.push(Plane::default());
                }

                let mut to_delete = vec![];
                for index in 0..self.state.planes.len() {
                    egui::CollapsingHeader::new(&self.state.planes[index].name)
                        .id_salt(index)
                        .show(ui, |ui| {
                            let plane = &mut self.state.planes[index];
                            ui.text_edit_singleline(&mut plane.name);
                            ui.horizontal(|ui| {
                                ui.label("Position:");
                                ui_vector3(ui, &mut plane.position);
                            });
                            ui.horizontal(|ui| {
                                ui.label("XY Rotation:");
                                ui.drag_angle(&mut plane.xy_rotation);
                            });
                            ui.horizontal(|ui| {
                                ui.label("YZ Rotation:");
                                ui.drag_angle(&mut plane.yz_rotation);
                            });
                            ui.horizontal(|ui| {
                                ui.label("XZ Rotation:");
                                ui.drag_angle(&mut plane.xz_rotation);
                            });
                            ui.horizontal(|ui| {
                                ui.label("Color:");
                                ui.color_edit_button_rgb(plane.color.as_mut());
                            });
                            ui.horizontal(|ui| {
                                ui.label("Size:");
                                ui.add(
                                    egui::DragValue::new(&mut plane.width)
                                        .speed(0.1)
                                        .prefix("x:"),
                                );
                                ui.add(
                                    egui::DragValue::new(&mut plane.height)
                                        .speed(0.1)
                                        .prefix("z:"),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label("Checker Count:");
                                ui.add(
                                    egui::DragValue::new(&mut plane.checker_count_x).prefix("x:"),
                                );
                                plane.checker_count_x = plane.checker_count_x.max(1);
                                ui.add(
                                    egui::DragValue::new(&mut plane.checker_count_z).prefix("z:"),
                                );
                                plane.checker_count_z = plane.checker_count_z.max(1);
                            });
                            ui.horizontal(|ui| {
                                ui.label("Checker Darkness:");
                                ui.add(egui::Slider::new(&mut plane.checker_darkness, 0.0..=1.0));
                            });
                            fn ui_portal_connection(
                                ui: &mut egui::Ui,
                                planes: &mut [Plane],
                                index: usize,
                                portal: impl Fn(&mut Plane) -> &mut PortalConnection,
                            ) {
                                ui.horizontal(|ui| {
                                    ui.label("Connected Plane:");
                                    egui::ComboBox::new(("Front Connected Portal", index), "")
                                        .selected_text(
                                            portal(&mut planes[index])
                                                .other_index
                                                .map(|other_index| {
                                                    planes[other_index].name.as_str()
                                                })
                                                .unwrap_or("None"),
                                        )
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(
                                                &mut portal(&mut planes[index]).other_index,
                                                None,
                                                "None",
                                            );
                                            for other_index in 0..planes.len() {
                                                let name = planes[other_index].name.clone();
                                                ui.selectable_value(
                                                    &mut portal(&mut planes[index]).other_index,
                                                    Some(other_index),
                                                    name,
                                                );
                                            }
                                        });
                                });
                                // ui.horizontal(|ui| {
                                //     ui.label("Flip:");
                                //     ui.checkbox(&mut portal(&mut planes[index]).flip, "");
                                // });
                            }
                            ui.collapsing("Front Portal", |ui| {
                                ui_portal_connection(ui, &mut self.state.planes, index, |plane| {
                                    &mut plane.front_portal
                                });
                            });
                            ui.collapsing("Back Portal", |ui| {
                                ui_portal_connection(ui, &mut self.state.planes, index, |plane| {
                                    &mut plane.back_portal
                                });
                            });
                            if ui.button("Delete").clicked() {
                                to_delete.push(index);
                            }
                        });
                }
                for index_to_delete in to_delete.into_iter().rev() {
                    for (index, plane) in self.state.planes.iter_mut().enumerate() {
                        if let Some(front_portal_index) = &mut plane.front_portal.other_index {
                            if *front_portal_index == index_to_delete {
                                plane.front_portal.other_index = None;
                            } else if index > index_to_delete {
                                *front_portal_index -= 1;
                            }
                        }
                        if let Some(back_portal_index) = &mut plane.back_portal.other_index {
                            if *back_portal_index == index_to_delete {
                                plane.front_portal.other_index = None;
                            } else if index > index_to_delete {
                                *back_portal_index -= 1;
                            }
                        }
                    }
                    self.state.planes.remove(index_to_delete);
                }
            });

        self.file_dialog.update(ctx);
        if let Some(mut path) = self.file_dialog.take_picked() {
            match std::mem::replace(&mut self.file_interaction, FileInteraction::None) {
                FileInteraction::None => {}
                FileInteraction::Save => {
                    if path.extension().is_none() {
                        path.set_extension("scene");
                    }
                    let state = serde_json::to_string(&self.state).unwrap();
                    _ = std::fs::write(path, state);
                }
                FileInteraction::Load => {
                    if let Ok(s) = std::fs::read_to_string(path)
                        && let Ok(state) = serde_json::from_str(&s)
                    {
                        self.state = state;
                        self.accumulated_frames = 0;
                    }
                }
            }
        }

        if !ctx.wants_keyboard_input() {
            ctx.input(|i| {
                let old_position = self.state.camera.position;
                if self.state.camera.update(i, ts) {
                    self.accumulated_frames = 0;
                }
                let new_position = self.state.camera.position;

                let ray = Ray {
                    origin: old_position,
                    direction: (new_position - old_position).normalised(),
                };

                let closest_hit = self
                    .state
                    .planes
                    .iter()
                    .enumerate()
                    .map(|(i, plane)| (i, plane.intersect(ray)))
                    .fold(None::<(usize, Hit)>, |closest_hit, (index, hit)| {
                        if let Some((closest_index, closest_hit)) = closest_hit {
                            if let Some(hit) = hit
                                && hit.distance < closest_hit.distance
                            {
                                Some((index, hit))
                            } else {
                                Some((closest_index, closest_hit))
                            }
                        } else {
                            hit.map(|hit| (index, hit))
                        }
                    });

                if let Some((index, hit)) = closest_hit
                    && hit.distance < (new_position - old_position).magnitude()
                {
                    let plane = &self.state.planes[index];
                    if let Some(other_index) = plane.front_portal.other_index
                        && hit.front
                    {
                        let other_plane = &self.state.planes[other_index];
                        let transform = other_plane.transform().then(plane.transform().reverse());
                        self.state.camera.position =
                            transform.transform_point(self.state.camera.position);
                        self.state.camera.rotation =
                            transform.rotor_part().then(self.state.camera.rotation);
                        self.accumulated_frames = 0;
                    } else if let Some(other_index) = plane.back_portal.other_index
                        && !hit.front
                    {
                        let other_plane = &self.state.planes[other_index];
                        let transform = other_plane.transform().then(plane.transform().reverse());
                        self.state.camera.position =
                            transform.transform_point(self.state.camera.position);
                        self.state.camera.rotation =
                            transform.rotor_part().then(self.state.camera.rotation);
                        self.accumulated_frames = 0;
                    }
                }
            });
        }

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
                                transform: self.state.camera.transform(),
                                up_sky_color: self.state.up_sky_color,
                                down_sky_color: self.state.down_sky_color,
                                sun_size: self.state.sun_size,
                                sun_color: self.state.sun_color,
                                sun_light_color: self.state.sun_light_color,
                                sun_direction: self.state.sun_direction,
                                ambient_color: self.state.ambient_color,
                                recursive_portal_count: self.state.recursive_portal_count,
                            },
                            accumulated_frames: self.accumulated_frames,
                            planes: self.state.planes.iter().map(Plane::to_gpu).collect(),
                        },
                    ));
                self.accumulated_frames += 1;
            });

        ctx.request_repaint();
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string("State", serde_json::to_string(&self.state).unwrap());
    }
}

pub fn ui_transform(
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

pub fn ui_vector3(ui: &mut egui::Ui, Vector3 { x, y, z }: &mut Vector3) -> egui::Response {
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
                wgpu_setup: eframe::egui_wgpu::WgpuSetup::CreateNew(
                    eframe::egui_wgpu::WgpuSetupCreateNew {
                        device_descriptor: Arc::new(|adapter| wgpu::DeviceDescriptor {
                            label: Some("egui wgpu device"),
                            required_features: wgpu::Features::default(),
                            required_limits: adapter.limits(),
                            memory_hints: wgpu::MemoryHints::default(),
                            trace: wgpu::Trace::Off,
                        }),
                        ..Default::default()
                    },
                ),
                ..Default::default()
            },
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
