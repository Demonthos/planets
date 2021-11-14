use rand::prelude::*;
use crate::planet::Planet;
use eframe::{egui, epi};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct App {
    particles: Vec<Planet>,
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
    #[cfg_attr(feature = "persistence", serde(skip))]
    preview_length: i32,
    #[cfg_attr(feature = "persistence", serde(skip))]
    paused: bool,
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
            preview_length: 100,
            paused: false,
        }
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        "Planets!"
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
                self.particles.iter().for_each(&mut |p: &Planet| {
                    if p.id == self.selected {
                        offset_pos =
                        p.pos.to_vec2() - ctx.available_rect().size() / 2.0;
                        offset_vel = p.vel;
                    }
                });
            }
            if pointer.any_released() {
                self.selected = -1;
                self.particles.iter().for_each(&mut |p: &Planet| {
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
                        let vel = if ctx.input().modifiers.shift{
                            if let Some(nearest) = self.particles.iter().max_by(|e1, e2| (e1.mass/e1.pos.distance_sq(mouse_pos)).partial_cmp(&(e2.mass/e2.pos.distance_sq(mouse_pos))).unwrap()){
                                (nearest.pos - pos).normalized().rot90()*pos.distance(mouse_pos)
                            }
                            else{
                                pos - mouse_pos
                            }
                        }
                        else{
                            pos - mouse_pos
                        };
                        self.particles.push(Planet::new(
                            pos + offset_pos,
                            (vel / 10.0) + offset_vel,
                            self.mass,
                            self.size,
                            self.last_id,
                            egui::color::Hsva::new(rand::random::<f32>(), 1.0, rand::random::<f32>(), 1.0)
                        ));
                        self.last_id += 1;
                        self.selected = old_selected;
                    }
                }
            }
        }

        // a = g*m/(d^2)
        let zoom_dt = ctx.input().scroll_delta.y;
        if zoom_dt != 0.0 {
            self.mass += zoom_dt / 20.0;
            self.mass = self.mass.max(1.0);
        }

        if ctx.input().key_pressed(egui::Key::Space){
            self.paused = !self.paused;
        }

        let mut old = self.particles.clone();
        if !self.paused{
            self.particles.iter_mut().for_each(|p| p.update(&old, self.min_trail_update, self.gravity, dt));
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut responces = vec![
                ui.add(egui::Slider::new(&mut self.gravity, 0.0..=100.0).text("gravity")),
                ui.add(egui::Slider::new(&mut self.mass, 1.0..=100.0).text("mass")),
                ui.add(egui::Slider::new(&mut self.size, 1.0..=100.0).text("size")),
                ui.add(
                    egui::Slider::new(&mut self.min_trail_update, 0.1..=2.0).text("trial length"),
                ),
                ui.add(egui::Slider::new(&mut self.preview_length, 100..=2000).text("preview length")),
                ui.checkbox(&mut self.force_fields, "force arrows"),
            ];
            if self.force_fields{
                responces.push(ui.add(egui::Slider::new(&mut self.arrow_size, 3.0..=60.0).text("arrow size")))
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
            let painter = ui.painter();
            if self.force_fields {
                let size = ctx.available_rect().size();
                let mut key_points = Vec::new();
                let key_points_dist = 3.0;
                for x in 0..((size.x / self.arrow_size)/key_points_dist).ceil() as usize + 1{
                    for y in 0..((size.y / self.arrow_size)/key_points_dist).ceil() as usize + 1{
                        let pos = (egui::Vec2::new(x as f32, y as f32) * self.arrow_size * key_points_dist) + selected_pos;
                        let mut min_dist_sq = 10000.0;
                        let vel = old
                            .iter()
                            .map(|d| {
                                // (d.pos - p.pos).normalized() * dt * (self.gravity * p.mass * d.mass)
                                let dist_sq = d.pos.distance_sq(pos.to_pos2());
                                if dist_sq < min_dist_sq{
                                    min_dist_sq = dist_sq
                                }
                                (d.pos - pos).to_vec2().normalized()
                                    * dt
                                    * (self.gravity.powf(2.0) * d.mass)
                                    / dist_sq
                            })
                            .fold(egui::Vec2::ZERO, |v1, v2| v1 + v2);
                        let color = (vel.length() * 10000.0 / (self.gravity.powf(2.0))).min(1.0);
                        if y == 0{
                            key_points.push(Vec::new());
                        }
                        key_points.last_mut().unwrap().push((vel, color, if min_dist_sq < 1000.0 {2} else if color < 0.1 {0} else {1}))
                    }
                }

                let x_size = (size.x.ceil() / self.arrow_size) as usize;
                let y_size = (size.y.ceil() / self.arrow_size) as usize;
                let mut arrows = if self.arrow_size > 5.0{
                    Vec::new()
                }
                else{
                    Vec::with_capacity(x_size*y_size)
                };
                let mut idx = 0;
                for x in 0..x_size {
                    for y in 0..y_size {
                        let mut pos = (egui::Vec2::new(x as f32, y as f32) * self.arrow_size) + selected_pos;
                        let left = (x as f32/key_points_dist).floor() as usize;
                        let right = (x as f32/key_points_dist).ceil() as usize;
                        let x_frac = (x as f32/key_points_dist).fract();
                        let top = (y as f32/key_points_dist).floor() as usize;
                        let bottom = (y as f32/key_points_dist).ceil() as usize;
                        let y_frac = (y as f32/key_points_dist).fract();
                        let highest_rending_level = [key_points[left][bottom], key_points[right][bottom], key_points[left][top], key_points[right][top]].iter().map(|e| e.2).max().unwrap();
                        if highest_rending_level != 1{
                            let vel = if highest_rending_level == 2{
                                old
                                .iter()
                                .map(|d| {
                                    // (d.pos - p.pos).normalized() * dt * (self.gravity * p.mass * d.mass)
                                    (d.pos - pos).to_vec2().normalized()
                                    * dt
                                    * (self.gravity.powf(2.0) * d.mass)
                                    / d.pos.distance_sq(pos.to_pos2())
                                })
                                .fold(egui::Vec2::ZERO, |v1, v2| v1 + v2)
                            }
                            else{
                                y_frac*(key_points[right][bottom].0*x_frac + key_points[left][bottom].0*(1.0-x_frac)) + (1.0-y_frac)*(key_points[right][top].0*x_frac + key_points[left][top].0*(1.0-x_frac))
                            };
                            let color = y_frac*(key_points[right][bottom].1*x_frac + key_points[left][bottom].1*(1.0-x_frac)) + (1.0-y_frac)*(key_points[right][top].1*x_frac + key_points[left][top].1*(1.0-x_frac));
                            pos -= selected_pos;
                            if arrows.is_empty(){
                                painter.arrow(pos.to_pos2(), vel.normalized() * self.arrow_size, egui::Stroke::new(1.0, egui::color::Hsva::new(color, 1.0, 1.0, color)));
                            }
                            else{
                                arrows[idx] = egui::Shape::LineSegment{
                                    points: [pos.to_pos2(), (pos + vel.normalized() * self.arrow_size).to_pos2()],
                                    stroke: egui::Stroke::new(1.0, egui::color::Hsva::new(color, 1.0, 1.0, color)),
                                };
                                idx += 1;
                            }
                        }
                        else{
                            pos -= selected_pos;
                            if arrows.is_empty(){
                                painter.arrow(pos.to_pos2(), key_points[right][bottom].0.normalized() * self.arrow_size, egui::Stroke::new(1.0, egui::color::Hsva::new(key_points[right][bottom].1, 1.0, 1.0, key_points[right][bottom].1)));
                            }
                            else{
                                arrows[idx] = egui::Shape::LineSegment{
                                    points: [pos.to_pos2(), (pos + key_points[right][bottom].0.normalized() * self.arrow_size).to_pos2()],
                                    stroke: egui::Stroke::new(1.0, egui::color::Hsva::new(key_points[right][bottom].1, 1.0, 1.0, key_points[right][bottom].1)),
                                };
                                idx += 1;
                            }
                        }
                    }
                }
                painter.extend(arrows);
            }
            self.particles.iter_mut().for_each(|p| {
                painter.circle_filled(p.pos - selected_pos, p.size, p.color);
                p.trail.windows(2).for_each(|w| {
                    painter.line_segment(
                        [w[0] - selected_pos, w[1] - selected_pos],
                        egui::Stroke::new(2.0, p.color),
                    )
                })
            });
            if let Some(pos) = self.creating {
                painter.circle_filled(pos, self.size, egui::Color32::GREEN);
                if let Some(mouse_pos) = pointer.interact_pos() {
                    let vel = if ctx.input().modifiers.shift{
                        if let Some(nearest) = self.particles.iter().max_by(|e1, e2| (e1.mass/e1.pos.distance_sq(mouse_pos)).partial_cmp(&(e2.mass/e2.pos.distance_sq(mouse_pos))).unwrap()){
                            (nearest.pos - pos).normalized().rot90()*pos.distance(mouse_pos)
                        }
                        else{
                            pos - mouse_pos
                        }
                    }
                    else{
                        pos - mouse_pos
                    };
                    painter.arrow(pos, vel, egui::Stroke::new(1.0, egui::Color32::GREEN));
                    let mut offset_pos = egui::Vec2::ZERO;
                    let mut offset_vel = egui::Vec2::ZERO;
                    if self.selected >= 0 {
                        old.iter().for_each(&mut |p: &Planet| {
                            if p.id == self.selected {
                                offset_pos =
                                    p.pos.to_vec2() - ctx.available_rect().size() / 2.0;
                                offset_vel = p.vel;
                            }
                        });
                    }
                    old.push(Planet::new(
                        pos + offset_pos,
                        (vel / 10.0) + offset_vel,
                        self.mass,
                        self.size,
                        self.last_id,
                        egui::Color32::GREEN
                    ));
                    let mut last_points: Option<Vec<_>> = None;
                    for _ in 0..self.preview_length {
                        let temp = old.clone();
                        let mut offset_pos = egui::Vec2::ZERO;
                        if self.selected >= 0 {
                            old.iter().for_each(&mut |p: &Planet| {
                                if p.id == self.selected {
                                    offset_pos =
                                        p.pos.to_vec2() - ctx.available_rect().size() / 2.0;
                                }
                            });
                        }
                        let new_points = old.iter().map(|e| (e.pos - offset_pos, e.color));
                        if let Some(ops) = last_points{
                            for (i, ps) in ops.iter().zip(new_points.clone()).enumerate(){
                                let (pos, color) = ps.1;
                                let mut color: egui::color::Hsva = color.into();
                                if i != temp.len() - 1{
                                    color.s /= 2.0;
                                }
                                painter.line_segment(
                                    [*ps.0, pos],
                                    egui::Stroke::new(2.0, color),
                                )
                            }
                        }
                        last_points = Some(new_points.map(|e| e.0).collect());
                        old.iter_mut().for_each(|p| p.update(&temp, self.min_trail_update, self.gravity, dt));
                    }
                }
            }
            // let com = self.particles.center_of_mass();
            // painter.circle_filled(com.1, com.0, egui::Color32::RED);
            egui::warn_if_debug_build(ui);
        });
    }
}
