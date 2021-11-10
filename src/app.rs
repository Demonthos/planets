use crate::kd::Kd;
use crate::plannet::Plannet;
use eframe::{egui, epi};
use std::mem;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct App {
    #[cfg_attr(feature = "persistence", serde(skip))]
    kd: Kd,
    gravity: f32,
    size: f32,
    #[cfg_attr(feature = "persistence", serde(skip))]
    creating: Option<egui::Pos2>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    last_id: i32
}

impl Default for App {
    fn default() -> Self {
        Self {
            kd: Kd::new(vec![]),
            gravity: 1.0,
            size: 5.0,
            creating: None,
            last_id: 0
        }
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        "Plannets!"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
    }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        ctx.request_repaint();

        // let dt = ctx.input().unstable_dt.min(1.0 / 60.0);
        let dt = ctx.input().predicted_dt;

        let old_kd = mem::replace(&mut self.kd, Kd::new(vec![]));
        let mut old = Vec::new();
        old_kd.drain(&mut |p| old.push(p));
        let pointer = &ctx.input().pointer;
        if let Some(pos) = self.creating {
            if pointer.any_released() {
                old.push(Plannet::new(
                    pos,
                    (pos - pointer.hover_pos().unwrap()) / 10.0,
                    self.size,
                    self.last_id
                ));
                self.last_id += 1;
            }
        }

        // f = g*m1*m2/(d^2)
        self.kd = Kd::new(old.clone());
        self.kd.for_each(&mut |p| p.pos += p.vel);

        let grav = self.gravity;

        self.kd.for_each(&mut |p| {
            p.vel += old
                .iter()
                .filter(|d| d.id != p.id)
                .map(|d| {
                    (d.pos - p.pos).normalized() * dt * (grav * p.mass * d.mass)
                        / d.pos.distance_sq(p.pos)
                })
                .fold(egui::Vec2::ZERO, |v1, v2| v1 + v2)
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            let responces = [ui.add(egui::Slider::new(&mut self.gravity, 0.0..=10.0).text("gravity")), ui.add(egui::Slider::new(&mut self.size, 1.0..=100.0).text("mass"))];
            if responces.iter().any(|r| r.hovered()){
                self.creating = None;
            }
            else{
                self.creating = pointer.press_origin();
            }
            if ui.button("reset").clicked() {
                self.kd = Kd::new(vec![]);
            }
            let painter = ui.painter();
            self.kd
                .for_each(&mut |p| painter.circle_filled(p.pos, p.mass, egui::Color32::BLUE));
            if let Some(pos) = self.creating {
                painter.circle_filled(pos, 10.0, egui::Color32::GREEN);
                if let Some(hover) = pointer.hover_pos() {
                    painter.arrow(
                        pos,
                        pos - hover,
                        egui::Stroke::new(1.0, egui::Color32::GREEN),
                    );
                }
            }
            let com = self.kd.center_of_mass();
            painter.circle_filled(com.1, com.0, egui::Color32::RED);
            egui::warn_if_debug_build(ui);
        });
    }
}
