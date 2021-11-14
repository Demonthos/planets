use eframe::egui;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct Planet {
    pub pos: egui::Pos2,
    pub vel: egui::Vec2,
    pub mass: f32,
    pub size: f32,
    pub id: i32,
    pub trail: Vec<egui::Pos2>,
    pub color: egui::Color32
}

impl Planet {
    pub fn new(pos: egui::Pos2, vel: egui::Vec2, mass: f32, size: f32, id: i32, color: impl Into<egui::Color32>) -> Self {
        Self {
            pos,
            vel,
            mass,
            size,
            id,
            trail: Vec::new(),
            color: color.into()
        }
    }

    pub fn update(&mut self, old: &Vec<Self>, min_trail_update: f32, gravity: f32, dt: f32){
        if if let Some(l) = self.trail.last() {
            (*l - self.pos).length_sq() > min_trail_update.powf(2.0)
        } else {
            true
        } {
            self.trail.push(self.pos);
            if self.trail.len() > 100 {
                self.trail.remove(0);
            }
        }

        self.pos += self.vel;

        self.vel += old
            .iter()
            .filter(|d| d.id != self.id)
            .map(|d| {
                // (d.pos - self.pos).normalized() * dt * (self.gravity * self.mass * d.mass)
                (d.pos - self.pos).normalized() * dt * (gravity.powf(2.0) * d.mass)
                / d.pos.distance_sq(self.pos)
            })
            .fold(egui::Vec2::ZERO, |v1, v2| v1 + v2)
    }
}
