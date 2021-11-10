use eframe::egui;

#[derive(Debug)]
pub struct Plannet{
    pub pos: egui::Pos2,
    pub vel: egui::Vec2,
    pub mass: f32
}

impl Plannet{
    pub fn new(pos: egui::Pos2, vel: egui::Vec2, mass: f32) -> Plannet{
        Plannet{
            pos,
            vel,
            mass
        }
    }
}
