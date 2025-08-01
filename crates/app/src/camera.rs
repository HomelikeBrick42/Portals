use crate::{ui_transform, ui_vector3};
use eframe::egui;
use math::{Rotor, Transform, Vector3};
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

#[derive(Debug, Serialize, Deserialize)]
pub struct Camera {
    pub position: Vector3,
    pub rotation: Rotor,
    pub speed: f32,
    pub rotation_speed: f32,
}

impl Camera {
    pub fn transform(&self) -> Transform {
        Transform::translation(self.position).then(Transform::from_rotor(self.rotation))
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Position:");
            changed |= ui_vector3(ui, &mut self.position).changed();
        });
        ui.add_enabled_ui(false, |ui| {
            ui.horizontal(|ui| {
                ui.label("Forward:");
                let mut forward = self.rotation.rotate(Vector3::FORWARD);
                ui_vector3(ui, &mut forward);
            });
            ui.horizontal(|ui| {
                ui.label("Up:");
                let mut up = self.rotation.rotate(Vector3::UP);
                ui_vector3(ui, &mut up);
            });
            ui.horizontal(|ui| {
                ui.label("Right:");
                let mut right = self.rotation.rotate(Vector3::RIGHT);
                ui_vector3(ui, &mut right);
            });
        });
        ui.collapsing("Transform", |ui| {
            ui.add_enabled_ui(false, |ui| {
                ui_transform(ui, &mut self.transform());
            });
        });
        ui.horizontal(|ui| {
            ui.label("Camera Speed:");
            ui.add(egui::DragValue::new(&mut self.speed).speed(0.1));
        });
        ui.horizontal(|ui| {
            ui.label("Camera Rotation Speed:");
            ui.add(egui::DragValue::new(&mut self.rotation_speed).speed(0.1));
        });
        changed
    }

    pub fn update(&mut self, i: &egui::InputState, ts: f32) -> bool {
        let mut changed = false;

        {
            let forward = i.key_down(egui::Key::W) as u8 as f32;
            let backward = i.key_down(egui::Key::S) as u8 as f32;
            let up = i.key_down(egui::Key::E) as u8 as f32;
            let down = i.key_down(egui::Key::Q) as u8 as f32;
            let left = i.key_down(egui::Key::A) as u8 as f32;
            let right = i.key_down(egui::Key::D) as u8 as f32;

            changed |= forward != 0.0
                || backward != 0.0
                || up != 0.0
                || down != 0.0
                || left != 0.0
                || right != 0.0;

            let boost = i.modifiers.shift as u8 as f32 + 1.0;

            let movement = Vector3 {
                x: forward - backward,
                y: up - down,
                z: right - left,
            }
            .normalised();

            self.position += self.rotation.rotate(movement) * self.speed * boost * ts;
        }

        {
            let up = i.key_down(egui::Key::ArrowUp) as u8 as f32;
            let down = i.key_down(egui::Key::ArrowDown) as u8 as f32;
            let left = i.key_down(egui::Key::ArrowLeft) as u8 as f32;
            let right = i.key_down(egui::Key::ArrowRight) as u8 as f32;

            changed |= up != 0.0 || down != 0.0 || left != 0.0 || right != 0.0;

            let vertical = up - down;
            self.rotation = self.rotation.then(Rotor::rotation_xy(
                vertical * self.rotation_speed * TAU * ts,
            ));

            if i.modifiers.shift {
                let roll = right - left;
                self.rotation = self
                    .rotation
                    .then(Rotor::rotation_yz(roll * self.rotation_speed * TAU * ts));
            } else {
                let horizontal = right - left;
                self.rotation = self.rotation.then(Rotor::rotation_xz(
                    horizontal * self.rotation_speed * TAU * ts,
                ));
            }
        }

        if (self.rotation.magnitude() - 1.0).abs() > 0.001 {
            self.rotation = self.rotation.normalised();
            changed |= true;
        }

        changed
    }
}
