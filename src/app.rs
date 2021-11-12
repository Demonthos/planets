use crate::plannet::Plannet;
use eframe::{egui, epi};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct App {
    particles: Vec<Plannet>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    gravity: f32,
    #[cfg_attr(feature = "persistence", serde(skip))]
    size: f32,
    #[cfg_attr(feature = "persistence", serde(skip))]
    mass: f32,
    #[cfg_attr(feature = "persistence", serde(skip))]
    creating: Option<egui::Pos2>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    last_id: i32,
    #[cfg_attr(feature = "persistence", serde(skip))]
    selected: i32,
    #[cfg_attr(feature = "persistence", serde(skip))]
    force_fields: bool,
    #[cfg_attr(feature = "persistence", serde(skip))]
    min_trail_update: f32,
    #[cfg_attr(feature = "persistence", serde(skip))]
    arrow_size: f32,
}

impl Default for App {
    fn default() -> Self {
        Self {
            particles: Vec::new(),
            gravity: 20.0,
            size: 5.0,
            mass: 5.0,
            creating: None,
            last_id: 0,
            selected: -1,
            force_fields: false,
            min_trail_update: 0.1,
            arrow_size: 10.0,
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

        let pointer = &ctx.input().pointer;
        let old_selected = self.selected;
        if let Some(mouse_pos) = pointer.interact_pos() {
            let mut offset_pos = egui::Vec2::ZERO;
            let mut offset_vel = egui::Vec2::ZERO;
            if self.selected >= 0 {
                self.particles.iter().for_each(&mut |p: &Plannet| {
                    if p.id == self.selected {
                        offset_pos =
                        p.pos.to_vec2() - ctx.available_rect().size() / 2.0;
                        offset_vel = p.vel;
                    }
                });
            }
            if pointer.any_released() {
                self.selected = -1;
                self.particles.iter().for_each(&mut |p: &Plannet| {
                    let pos = p.pos-offset_pos;
                    // println!("{:?}", p.pos.distance(i));
                    if pos.distance(mouse_pos) <= p.size {
                        self.selected = p.id;
                    }
                });
            }
            if self.selected < 0 {
                if let Some(pos) = self.creating {
                    if pointer.any_released() {
                        self.particles.push(Plannet::new(
                            pos + offset_pos,
                            ((pos - mouse_pos) / 10.0) + offset_vel,
                            self.mass,
                            self.size,
                            self.last_id,
                        ));
                        self.last_id += 1;
                        self.selected = old_selected;
                    }
                }
            }
        }

        // a = g*m/(d^2)
        let mut old = self.particles.clone();
        self.particles.iter_mut().for_each(|p| {
            if if let Some(l) = p.trail.last() {
                (*l - p.pos).length_sq() > self.min_trail_update.powf(2.0)
            } else {
                true
            } {
                p.trail.push(p.pos);
                if p.trail.len() > 100 {
                    p.trail.remove(0);
                }
            }
        });
        self.particles.iter_mut().for_each(|p| p.pos += p.vel);
        let zoom_dt = ctx.input().scroll_delta.y;
        if zoom_dt != 0.0 {
            self.gravity *= zoom_dt / 50.0;
        }

        let grav = self.gravity;

        self.particles.iter_mut().for_each(|p| {
            p.vel += old
                .iter()
                .filter(|d| d.id != p.id)
                .map(|d| {
                    // (d.pos - p.pos).normalized() * dt * (grav * p.mass * d.mass)
                    (d.pos - p.pos).normalized() * dt * (grav.powf(2.0) * d.mass)
                        / d.pos.distance_sq(p.pos)
                })
                .fold(egui::Vec2::ZERO, |v1, v2| v1 + v2)
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut responces = vec![
                ui.add(egui::Slider::new(&mut self.gravity, 0.0..=500.0).text("gravity")),
                ui.add(egui::Slider::new(&mut self.mass, 1.0..=100.0).text("mass")),
                ui.add(egui::Slider::new(&mut self.size, 1.0..=100.0).text("size")),
                ui.add(
                    egui::Slider::new(&mut self.min_trail_update, 0.1..=2.0).text("trial length"),
                ),
                ui.checkbox(&mut self.force_fields, "force arrows"),
            ];
            if self.force_fields{
                responces.push(ui.add(egui::Slider::new(&mut self.arrow_size, 1.0..=60.0).text("arrow size")))
            }
            if responces.iter().any(|r| r.dragged() || r.hovered()) {
                self.selected = old_selected;
                self.creating = None;
            } else {
                self.creating = pointer.press_origin();
            }
            if ui.button("reset").clicked() {
                self.particles = Vec::new();
            }
            let mut selected_pos = egui::Vec2::ZERO;
            if self.selected >= 0 {
                self.particles.iter_mut().for_each(|p| {
                    if p.id == self.selected {
                        selected_pos = p.pos.to_vec2() - ctx.available_rect().size() / 2.0
                    }
                });
            }
            if self.force_fields {
                let size = ctx.available_rect().size();
                let mut key_points = Vec::new();
                let key_points_dist = 2.0;
                for x in 0..((size.x / self.arrow_size)/key_points_dist).ceil() as usize + 1{
                    for y in 0..((size.y / self.arrow_size)/key_points_dist).ceil() as usize + 1{
                        let pos = (egui::Vec2::new(x as f32, y as f32) * self.arrow_size * 2.0) + selected_pos;
                        let mut min_dist_sq = 10000.0;
                        let vel = old
                            .iter()
                            .map(|d| {
                                // (d.pos - p.pos).normalized() * dt * (grav * p.mass * d.mass)
                                let dist_sq = d.pos.distance_sq(pos.to_pos2());
                                if dist_sq < min_dist_sq{
                                    min_dist_sq = dist_sq
                                }
                                (d.pos - pos).to_vec2().normalized()
                                    * dt
                                    * (grav.powf(2.0) * d.mass)
                                    / dist_sq
                            })
                            .fold(egui::Vec2::ZERO, |v1, v2| v1 + v2);
                        let color = (vel.length() * 10000.0 / (self.gravity.powf(2.0))).min(1.0);
                        if y == 0{
                            key_points.push(Vec::new());
                        }
                        key_points.last_mut().unwrap().push((vel, color, min_dist_sq < 1000.0))
                    }
                }

                for x in 0..(size.x.ceil() / self.arrow_size) as usize {
                    for y in 0..(size.y.ceil() / self.arrow_size) as usize {
                        let pos = (egui::Vec2::new(x as f32, y as f32) * self.arrow_size) + selected_pos;
                        let left = (x as f32/key_points_dist).floor() as usize;
                        let right = (x as f32/key_points_dist).ceil() as usize;
                        let x_frac = (x as f32/key_points_dist).fract().powf(2.0);
                        let top = (y as f32/key_points_dist).floor() as usize;
                        let bottom = (y as f32/key_points_dist).ceil() as usize;
                        let y_frac = (y as f32/key_points_dist).fract().powf(2.0);
                        let any_close = [key_points[left][bottom], key_points[right][bottom], key_points[left][top], key_points[right][top]].iter().map(|e| e.2).any(|e|e);
                        let vel = if any_close{
                            old
                                .iter()
                                .map(|d| {
                                    // (d.pos - p.pos).normalized() * dt * (grav * p.mass * d.mass)
                                    (d.pos - pos).to_vec2().normalized()
                                        * dt
                                        * (grav.powf(2.0) * d.mass)
                                        / d.pos.distance_sq(pos.to_pos2())
                                })
                                .fold(egui::Vec2::ZERO, |v1, v2| v1 + v2)
                        }
                        else{
                            y_frac*(key_points[left][bottom].0*x_frac + key_points[right][bottom].0*(1.0-x_frac)) + (1.0-y_frac)*(key_points[left][top].0*x_frac + key_points[right][top].0*(1.0-x_frac))
                        };
                        let color = y_frac*(key_points[left][bottom].1*x_frac + key_points[right][bottom].1*(1.0-x_frac)) + (1.0-y_frac)*(key_points[left][top].1*x_frac + key_points[right][top].1*(1.0-x_frac));
                        ui.painter().arrow(
                            (pos - selected_pos).to_pos2(),
                            vel.normalized() * self.arrow_size,
                            egui::Stroke::new(1.0, egui::color::Hsva::new(color, 1.0, 1.0, color)),
                        );
                    }
                }
            }
            let painter = ui.painter();
            self.particles.iter_mut().for_each(|p| {
                painter.circle_filled(p.pos - selected_pos, p.size, egui::Color32::BLUE);
                p.trail.windows(2).for_each(|w| {
                    painter.line_segment(
                        [w[0] - selected_pos, w[1] - selected_pos],
                        egui::Stroke::new(2.0, egui::Color32::BLUE),
                    )
                })
            });
            if let Some(pos) = self.creating {
                painter.circle_filled(pos, self.size, egui::Color32::GREEN);
                if let Some(hover) = pointer.interact_pos() {
                    let vel = pos - hover;
                    painter.arrow(pos, vel, egui::Stroke::new(1.0, egui::Color32::GREEN));
                    let mut offset_pos = egui::Vec2::ZERO;
                    let mut offset_vel = egui::Vec2::ZERO;
                    if self.selected >= 0 {
                        old.iter().for_each(&mut |p: &Plannet| {
                            if p.id == self.selected {
                                offset_pos =
                                    p.pos.to_vec2() - ctx.available_rect().size() / 2.0;
                                offset_vel = p.vel;
                            }
                        });
                    }
                    old.push(Plannet::new(
                        pos + offset_pos,
                        ((pos - hover) / 10.0) + offset_vel,
                        self.mass,
                        self.size,
                        self.last_id,
                    ));
                    let mut points = Vec::new();
                    for _ in 0..300 {
                        let temp = old.clone();
                        let mut offset_pos = egui::Vec2::ZERO;
                        if self.selected >= 0 {
                            old.iter().for_each(&mut |p: &Plannet| {
                                if p.id == self.selected {
                                    offset_pos =
                                        p.pos.to_vec2() - ctx.available_rect().size() / 2.0;
                                }
                            });
                        }
                        points.push(
                            old.last().unwrap().pos - offset_pos
                        );
                        old.iter_mut().for_each(|d| d.pos += d.vel);
                        old.iter_mut().for_each(|p| {
                            p.vel += temp
                                .iter()
                                .filter(|d| d.id != p.id)
                                .map(|d| {
                                    // (d.pos - p.pos).normalized() * dt * (grav * p.mass * d.mass)
                                    (d.pos - p.pos).normalized() * dt * (grav.powf(2.0) * d.mass)
                                        / d.pos.distance_sq(p.pos)
                                })
                                .fold(egui::Vec2::ZERO, |v1, v2| v1 + v2)
                        });
                    }
                    points.windows(2).for_each(|w| {
                        painter.line_segment(
                            [w[0], w[1]],
                            egui::Stroke::new(2.0, egui::Color32::GREEN),
                        )
                    })
                }
            }
            // let com = self.particles.center_of_mass();
            // painter.circle_filled(com.1, com.0, egui::Color32::RED);
            egui::warn_if_debug_build(ui);
        });
    }
}
