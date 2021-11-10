use eframe::egui;

#[derive(Debug, Clone)]
pub struct Plannet {
    pub pos: egui::Pos2,
    pub vel: egui::Vec2,
    pub mass: f32,
    pub size: f32,
    pub id: i32
}

impl Plannet {
    pub fn new(pos: egui::Pos2, vel: egui::Vec2, mass: f32, size: f32, id: i32) -> Plannet {
        Plannet { pos, vel, mass, size, id }
    }
}
