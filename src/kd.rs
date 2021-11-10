use crate::plannet::Plannet;
use eframe::egui;

#[derive(Debug, Clone)]
pub enum Kd {
    Partition {
        x_split: bool,
        split: f32,
        children: [Box<Kd>; 2],
    },
    Node(Vec<Plannet>),
}

impl Kd {
    pub fn new(mut plannets: Vec<Plannet>) -> Kd {
        let len = plannets.len();
        if len <= 1 {
            Kd::Node(plannets)
        } else {
            let ord = |f1: &f32, f2: &f32| f1.partial_cmp(f2).unwrap();
            let min_x = plannets.iter().map(|p| p.pos.x).min_by(ord).unwrap();
            let min_y = plannets.iter().map(|p| p.pos.y).min_by(ord).unwrap();
            let max_x = plannets.iter().map(|p| p.pos.x).max_by(ord).unwrap();
            let max_y = plannets.iter().map(|p| p.pos.y).max_by(ord).unwrap();
            let mean: egui::Vec2 = plannets
                .iter()
                .map(|p| p.pos.to_vec2())
                .fold(egui::Vec2::ZERO, |p1, p2| p1 + p2)
                / len as f32;
            // println!("{:?}: {:?}", len, mean);
            if (min_x - max_x).abs() >= (min_y - max_y).abs() {
                plannets.sort_by(|p1, p2| ord(&p1.pos.x, &p2.pos.x));
                let split = plannets
                    .iter()
                    .position(|p| p.pos.x > mean.x)
                    .unwrap_or_else(|| {
                        let same: Vec<_> = plannets
                            .iter()
                            .enumerate()
                            .filter(|(_, p)| p.pos.x == mean.x)
                            .collect();
                        same.len() / 2 + 1
                    });
                // println!("{:?}: {:?}", split, mean);
                let other = plannets.split_off(split);
                Kd::Partition {
                    x_split: true,
                    split: mean.x,
                    children: [Box::new(Kd::new(plannets)), Box::new(Kd::new(other))],
                }
            } else {
                plannets.sort_by(|p1, p2| ord(&p1.pos.y, &p2.pos.y));
                let split = plannets
                    .iter()
                    .position(|p| p.pos.y > mean.y)
                    .unwrap_or_else(|| {
                        let same: Vec<_> = plannets
                            .iter()
                            .enumerate()
                            .filter(|(_, p)| p.pos.y == mean.y)
                            .collect();
                        same.len() / 2 + 1
                    });
                // println!("{:?}: {:?}", split, mean);
                let other = plannets.split_off(split);
                Kd::Partition {
                    x_split: false,
                    split: mean.y,
                    children: [Box::new(Kd::new(plannets)), Box::new(Kd::new(other))],
                }
            }
        }
    }

    pub fn for_each(&mut self, f: &mut impl FnMut(&mut Plannet)) {
        match self {
            Kd::Partition {
                x_split: _,
                split: _,
                children,
            } => children.iter_mut().for_each(|c| c.for_each(f)),
            Kd::Node(children) => children.iter_mut().for_each(f),
        }
    }

    pub fn drain(self, f: &mut impl FnMut(Plannet)) {
        match self {
            Kd::Partition {
                x_split: _,
                split: _,
                children,
            } => children.to_vec().into_iter().for_each(|c| c.drain(f)),
            Kd::Node(children) => children.into_iter().for_each(f),
        }
    }
}
