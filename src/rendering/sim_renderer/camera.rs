use glam::{Vec2, vec2};
use crate::rendering::Display;

static W2S_FAC: f32 = 50.0;

pub struct Camera {
    pub window_size: Vec2,
    pub translation: Vec2,
}

#[allow(dead_code)]
impl Camera {
    pub fn new(display: &Display) -> Self {
        let winsize = display.window.inner_size();
        let size = Vec2::new(
         winsize.width as _,
         winsize.height as _,
        );

        Camera {
            window_size: size,
            translation: size / 2.0,
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.window_size = Vec2::new(
         width as _,
         height as _,
        );
    }

    pub fn translate_by(&mut self, delta: Vec2) {
        self.translation += delta * W2S_FAC;
    }
}