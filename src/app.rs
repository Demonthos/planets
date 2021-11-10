use eframe::{egui, epi};
use crate::kd::Kd;
use crate::plannet::Plannet;

const GRAVITY: f32 = 1.0;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    #[cfg_attr(feature = "persistence", serde(skip))]
    kd: Kd
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            kd: {
                let num = 20;
                let plannets: Vec<_> = (0..num).map(|i| std::f32::consts::TAU*(i as f32)/(num as f32)).map(|i| Plannet::new(egui::Pos2::new(300.0, 300.0) + egui::Pos2::new(i.cos(), i.sin()).to_vec2()*100.0, -egui::Vec2::angled(i), 10.0)).collect();
                // println!("{:#?}", plannets);
                let kd = Kd::new(plannets);
                // println!("{:#?}", kd);
                kd
            }
        }
    }
}

impl epi::App for TemplateApp {
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
        let dt = ctx.input().unstable_dt.min(1.0 / 30.0);

        // f = g*m1*m2/(d^2)
        let mut old = Vec::new();
        self.kd.for_each(&mut |p| old.push((p.pos, p.mass)));

        self.kd.for_each(&mut |p| p.vel += old.iter().filter(|d| d.0 != p.pos).map(|d| dt*(d.0-p.pos).normalized()*(GRAVITY*p.mass*d.1)/d.0.distance_sq(p.pos)).fold(egui::Vec2::ZERO, |v1, v2| v1 + v2));
        self.kd.for_each(&mut |p| p.pos += p.vel);
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();
            self.kd.for_each(&mut |p| painter.circle_filled(p.pos, p.mass, egui::Color32::BLUE));
            egui::warn_if_debug_build(ui);
        });
    }
}
