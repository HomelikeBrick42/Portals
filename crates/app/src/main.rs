use eframe::{egui, wgpu};
use egui_file_dialog::FileDialog;
use math::{Rotor, Transform, Vector3};
use ray_tracing::{
    Color, GpuCamera, RENDER_TYPE_LIT, RENDER_TYPE_UNLIT, RayTracingPaintCallback,
    RayTracingRenderer,
};
use serde::{Deserialize, Serialize};
use std::{f32::consts::PI, sync::Arc, time::Instant};

mod camera;
mod plane;
mod ray;

pub use camera::*;
pub use plane::*;
pub use ray::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
enum RenderType {
    Unlit,
    Lit,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct RenderSettings {
    info_window_open: bool,
    camera_window_open: bool,
    render_settings_window_open: bool,
    planes_window_open: bool,
    render_type: RenderType,
    antialiasing: bool,
    recursive_portal_count: u32,
    max_bounces: u32,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            info_window_open: true,
            camera_window_open: true,
            render_settings_window_open: true,
            planes_window_open: true,
            render_type: RenderType::Unlit,
            antialiasing: true,
            recursive_portal_count: 10,
            max_bounces: 3,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct Scene {
    camera: Camera,
    up_sky_color: Color,
    up_sky_intensity: f32,
    down_sky_color: Color,
    down_sky_intensity: f32,
    sun_color: Color,
    sun_intensity: f32,
    sun_direction: Vector3,
    sun_size: f32,
    planes: Vec<Plane>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
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
            up_sky_intensity: 1.0,
            down_sky_color: Color {
                r: 0.4,
                g: 0.4,
                b: 0.4,
            },
            down_sky_intensity: 1.0,
            sun_size: 6.0f32.to_radians(),
            sun_color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
            },
            sun_intensity: 100.0,
            sun_direction: Vector3 {
                x: 0.4,
                y: 1.0,
                z: 0.2,
            },
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
                width: 10.0,
                height: 10.0,
                checker_count_x: 10,
                checker_count_z: 10,
                color: Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                },
                checker_darkness: 0.5,
                emissive_color: Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                },
                emission_intensity: 0.0,
                emissive_checker_darkness: 0.5,
                front_portal: PortalConnection::default(),
                back_portal: PortalConnection::default(),
            }],
        }
    }
}

struct App {
    last_time: Option<Instant>,
    scene: Scene,
    render_settings: RenderSettings,
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
            scene: cc
                .storage
                .and_then(|storage| storage.get_string("Scene"))
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default(),
            render_settings: cc
                .storage
                .and_then(|storage| storage.get_string("RenderSettings"))
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

        let mut rendering_changed = false;

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
                    self.render_settings.info_window_open |= ui.button("Info").clicked();
                    self.render_settings.render_settings_window_open |=
                        ui.button("Render Settings").clicked();
                    self.render_settings.camera_window_open |= ui.button("Camera").clicked();
                    self.render_settings.planes_window_open |= ui.button("Planes").clicked();
                });
            });
            if reset_everything {
                self.scene = Scene::default();
                rendering_changed = true;
            }
        }

        egui::Window::new("Info")
            .resizable(false)
            .open(&mut self.render_settings.info_window_open)
            .show(ctx, |ui| {
                ui.label(format!("FPS: {:.3}", 1.0 / dt.as_secs_f64()));
                ui.label(format!("Frame Time: {:.3}ms", dt.as_secs_f64() * 1000.0));
            });

        egui::Window::new("Render Settings")
            .open(&mut self.render_settings.render_settings_window_open)
            .scroll(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Render Type:");
                    let name = |render_type: &RenderType| match render_type {
                        RenderType::Unlit => "Unlit",
                        RenderType::Lit => "Lit",
                    };
                    egui::ComboBox::new("Render Type", "")
                        .selected_text(name(&self.render_settings.render_type))
                        .show_ui(ui, |ui| {
                            rendering_changed |= ui
                                .selectable_value(
                                    &mut self.render_settings.render_type,
                                    RenderType::Unlit,
                                    name(&RenderType::Unlit),
                                )
                                .changed();
                            rendering_changed |= ui
                                .selectable_value(
                                    &mut self.render_settings.render_type,
                                    RenderType::Lit,
                                    name(&RenderType::Lit),
                                )
                                .changed();
                        });
                });
                ui.horizontal(|ui| {
                    ui.label("Anti-aliasing:");
                    rendering_changed |= ui
                        .checkbox(&mut self.render_settings.antialiasing, "")
                        .changed();
                });
                ui.horizontal(|ui| {
                    ui.label("Max Portal Recursion:");
                    rendering_changed |= ui
                        .add(egui::DragValue::new(
                            &mut self.render_settings.recursive_portal_count,
                        ))
                        .changed();
                });
                ui.horizontal(|ui| {
                    ui.label("Max Light Bounces:");
                    rendering_changed |= ui
                        .add(egui::DragValue::new(&mut self.render_settings.max_bounces))
                        .changed();
                });
                ui.horizontal(|ui| {
                    ui.label("Accumulated Frames:");
                    ui.add_enabled(false, egui::DragValue::new(&mut self.accumulated_frames));
                    if ui.button("Clear").clicked() {
                        self.accumulated_frames = 0;
                    }
                });
            });

        egui::Window::new("Camera")
            .open(&mut self.render_settings.camera_window_open)
            .scroll(true)
            .show(ctx, |ui| {
                rendering_changed |= self.scene.camera.ui(ui);
                ui.horizontal(|ui| {
                    ui.label("Up Sky Color:");
                    rendering_changed |= ui
                        .color_edit_button_rgb(self.scene.up_sky_color.as_mut())
                        .changed();
                });
                ui.horizontal(|ui| {
                    ui.label("Up Sky Intensity:");
                    rendering_changed |= ui
                        .add(egui::DragValue::new(&mut self.scene.up_sky_intensity).speed(0.1))
                        .changed();
                });
                ui.horizontal(|ui| {
                    ui.label("Down Sky Color:");
                    rendering_changed |= ui
                        .color_edit_button_rgb(self.scene.down_sky_color.as_mut())
                        .changed();
                });
                ui.horizontal(|ui| {
                    ui.label("Down Sky Intensity:");
                    rendering_changed |= ui
                        .add(egui::DragValue::new(&mut self.scene.down_sky_intensity).speed(0.1))
                        .changed();
                });
                ui.horizontal(|ui| {
                    ui.label("Sun Color:");
                    rendering_changed |= ui
                        .color_edit_button_rgb(self.scene.sun_color.as_mut())
                        .changed();
                });
                ui.horizontal(|ui| {
                    ui.label("Sun Intensity:");
                    rendering_changed |= ui
                        .add(egui::DragValue::new(&mut self.scene.sun_intensity).speed(0.1))
                        .changed();
                });
                ui.horizontal(|ui| {
                    ui.label("Sun Angular Radius:");
                    rendering_changed |= ui.drag_angle(&mut self.scene.sun_size).changed();
                    self.scene.sun_size = self.scene.sun_size.clamp(0.0, PI);
                });
                ui.horizontal(|ui| {
                    ui.label("Sun Direction:");
                    rendering_changed |= ui_vector3(ui, &mut self.scene.sun_direction).changed();
                });
            });

        egui::Window::new("Planes")
            .open(&mut self.render_settings.planes_window_open)
            .scroll(true)
            .show(ctx, |ui| {
                if ui.button("New Plane").clicked() {
                    self.scene.planes.push(Plane::default());
                    rendering_changed = true;
                }

                let mut to_delete = vec![];
                for index in 0..self.scene.planes.len() {
                    egui::CollapsingHeader::new(&self.scene.planes[index].name)
                        .id_salt(index)
                        .show(ui, |ui| {
                            let plane = &mut self.scene.planes[index];
                            ui.text_edit_singleline(&mut plane.name);
                            ui.horizontal(|ui| {
                                ui.label("Position:");
                                rendering_changed |= ui_vector3(ui, &mut plane.position).changed();
                            });
                            ui.horizontal(|ui| {
                                ui.label("XY Rotation:");
                                rendering_changed |=
                                    ui.drag_angle(&mut plane.xy_rotation).changed();
                            });
                            ui.horizontal(|ui| {
                                ui.label("YZ Rotation:");
                                rendering_changed |=
                                    ui.drag_angle(&mut plane.yz_rotation).changed();
                            });
                            ui.horizontal(|ui| {
                                ui.label("XZ Rotation:");
                                rendering_changed |=
                                    ui.drag_angle(&mut plane.xz_rotation).changed();
                            });
                            ui.horizontal(|ui| {
                                ui.label("Size:");
                                rendering_changed |= ui
                                    .add(
                                        egui::DragValue::new(&mut plane.width)
                                            .speed(0.1)
                                            .prefix("x:"),
                                    )
                                    .changed();
                                rendering_changed |= ui
                                    .add(
                                        egui::DragValue::new(&mut plane.height)
                                            .speed(0.1)
                                            .prefix("z:"),
                                    )
                                    .changed();
                            });
                            ui.horizontal(|ui| {
                                ui.label("Checker Count:");
                                rendering_changed |= ui
                                    .add(
                                        egui::DragValue::new(&mut plane.checker_count_x)
                                            .prefix("x:"),
                                    )
                                    .changed();
                                plane.checker_count_x = plane.checker_count_x.max(1);
                                rendering_changed |= ui
                                    .add(
                                        egui::DragValue::new(&mut plane.checker_count_z)
                                            .prefix("z:"),
                                    )
                                    .changed();
                                plane.checker_count_z = plane.checker_count_z.max(1);
                            });
                            ui.horizontal(|ui| {
                                ui.label("Color:");
                                rendering_changed |=
                                    ui.color_edit_button_rgb(plane.color.as_mut()).changed();
                            });
                            ui.horizontal(|ui| {
                                ui.label("Checker Darkness:");
                                rendering_changed |= ui
                                    .add(egui::Slider::new(&mut plane.checker_darkness, 0.0..=1.0))
                                    .changed();
                            });
                            ui.horizontal(|ui| {
                                ui.label("Emssive Color:");
                                rendering_changed |= ui
                                    .color_edit_button_rgb(plane.emissive_color.as_mut())
                                    .changed();
                            });
                            ui.horizontal(|ui| {
                                ui.label("Emission Intensity:");
                                rendering_changed |= ui
                                    .add(
                                        egui::DragValue::new(&mut plane.emission_intensity)
                                            .speed(0.1),
                                    )
                                    .changed();
                            });
                            ui.horizontal(|ui| {
                                ui.label("Emissive Checker Darkness:");
                                rendering_changed |= ui
                                    .add(egui::Slider::new(
                                        &mut plane.emissive_checker_darkness,
                                        0.0..=1.0,
                                    ))
                                    .changed();
                            });
                            fn ui_portal_connection(
                                ui: &mut egui::Ui,
                                planes: &mut [Plane],
                                index: usize,
                                portal: impl Fn(&mut Plane) -> &mut PortalConnection,
                            ) -> bool {
                                let mut changed = false;
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
                                            changed |= ui
                                                .selectable_value(
                                                    &mut portal(&mut planes[index]).other_index,
                                                    None,
                                                    "None",
                                                )
                                                .changed();
                                            for other_index in 0..planes.len() {
                                                let name = planes[other_index].name.clone();
                                                changed |= ui
                                                    .selectable_value(
                                                        &mut portal(&mut planes[index]).other_index,
                                                        Some(other_index),
                                                        name,
                                                    )
                                                    .changed();
                                            }
                                        });
                                });
                                // ui.horizontal(|ui| {
                                //     ui.label("Flip:");
                                //     ui.checkbox(&mut portal(&mut planes[index]).flip, "");
                                // });
                                changed
                            }
                            ui.collapsing("Front Portal", |ui| {
                                rendering_changed |= ui_portal_connection(
                                    ui,
                                    &mut self.scene.planes,
                                    index,
                                    |plane| &mut plane.front_portal,
                                );
                            });
                            ui.collapsing("Back Portal", |ui| {
                                rendering_changed |= ui_portal_connection(
                                    ui,
                                    &mut self.scene.planes,
                                    index,
                                    |plane| &mut plane.back_portal,
                                );
                            });
                            if ui.button("Delete").clicked() {
                                to_delete.push(index);
                                rendering_changed = true;
                            }
                        });
                }
                for index_to_delete in to_delete.into_iter().rev() {
                    for (index, plane) in self.scene.planes.iter_mut().enumerate() {
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
                    self.scene.planes.remove(index_to_delete);
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
                    let state = serde_json::to_string(&self.scene).unwrap();
                    _ = std::fs::write(path, state);
                }
                FileInteraction::Load => {
                    if let Ok(s) = std::fs::read_to_string(path)
                        && let Ok(state) = serde_json::from_str(&s)
                    {
                        self.scene = state;
                        rendering_changed = true;
                    }
                }
            }
        }

        if !ctx.wants_keyboard_input() {
            ctx.input(|i| {
                let old_position = self.scene.camera.position;
                rendering_changed |= self.scene.camera.update(i, ts);
                let new_position = self.scene.camera.position;

                let ray = Ray {
                    origin: old_position,
                    direction: (new_position - old_position).normalised(),
                };

                let closest_hit = self
                    .scene
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
                    let plane = &self.scene.planes[index];
                    if let Some(other_index) = plane.front_portal.other_index
                        && hit.front
                    {
                        let other_plane = &self.scene.planes[other_index];
                        let transform = other_plane.transform().then(plane.transform().reverse());
                        self.scene.camera.position =
                            transform.transform_point(self.scene.camera.position);
                        self.scene.camera.rotation =
                            transform.rotor_part().then(self.scene.camera.rotation);
                        rendering_changed = true;
                    } else if let Some(other_index) = plane.back_portal.other_index
                        && !hit.front
                    {
                        let other_plane = &self.scene.planes[other_index];
                        let transform = other_plane.transform().then(plane.transform().reverse());
                        self.scene.camera.position =
                            transform.transform_point(self.scene.camera.position);
                        self.scene.camera.rotation =
                            transform.rotor_part().then(self.scene.camera.rotation);
                        rendering_changed = true;
                    }
                }
            });
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(255, 0, 255)))
            .show(ctx, |ui| {
                let (rect, _response) =
                    ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());

                if rendering_changed {
                    self.accumulated_frames = 0;
                }
                ui.painter()
                    .add(eframe::egui_wgpu::Callback::new_paint_callback(
                        rect,
                        RayTracingPaintCallback {
                            width: rect.width() as u32,
                            height: rect.height() as u32,
                            camera: GpuCamera {
                                transform: self.scene.camera.transform(),
                                up_sky_color: self.scene.up_sky_color * self.scene.up_sky_intensity,
                                down_sky_color: self.scene.down_sky_color
                                    * self.scene.down_sky_intensity,
                                sun_color: self.scene.sun_color * self.scene.sun_intensity,
                                sun_direction: self.scene.sun_direction.normalised(),
                                sun_size: self.scene.sun_size,
                                recursive_portal_count: self.render_settings.recursive_portal_count,
                                max_bounces: self.render_settings.max_bounces,
                            },
                            accumulated_frames: self.accumulated_frames,
                            random_seed: rand::random(),
                            render_type: match self.render_settings.render_type {
                                RenderType::Unlit => RENDER_TYPE_UNLIT,
                                RenderType::Lit => RENDER_TYPE_LIT,
                            },
                            antialiasing: self.render_settings.antialiasing,
                            planes: self.scene.planes.iter().map(Plane::to_gpu).collect(),
                        },
                    ));
                self.accumulated_frames += 1;
            });

        ctx.request_repaint();
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string("Scene", serde_json::to_string(&self.scene).unwrap());
        storage.set_string(
            "RenderSettings",
            serde_json::to_string(&self.render_settings).unwrap(),
        );
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
                            label: Some("Device"),
                            required_features:
                                wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
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
