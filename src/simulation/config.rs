use crate::vec2::Vec2;

#[derive(Clone, Copy)]
pub struct Config {
    pub n_cells: u32,
    pub cell_radius: f32,
    pub bounds: (Vec2, Vec2),
}

impl Config {
    pub fn default() -> Config {
        Config {
            n_cells: 100_000,
            cell_radius: 2.0,
            bounds: (Vec2::new(0.0, 0.0), Vec2::new(8192.0, 8192.0)),
        }
    }
}
