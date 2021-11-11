use eframe::egui;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct Plannet {
    pub pos: egui::Pos2,
    pub vel: egui::Vec2,
    pub mass: f32,
    pub size: f32,
    pub id: i32,
    pub trail: Vec<egui::Pos2>,
}

impl Plannet {
    pub fn new(pos: egui::Pos2, vel: egui::Vec2, mass: f32, size: f32, id: i32) -> Plannet {
        Plannet {
            pos,
            vel,
            mass,
            size,
            id,
            trail: Vec::new(),
        }
    }
}
