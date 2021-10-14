use sdl2::pixels::Color;
use crate::ops::{clamp};
pub trait ColFuncs {
    fn blend(&self, c: Self) -> Self;
    fn avg(&self, c: Self) -> Self;
    fn scale(&self, s: f32) -> Self;
    fn avg_f32(&self, s: f32) -> Self;
    fn add(&self, c: Self) -> Self;
    fn from_f32_greyscale(f: f32) -> Self;
}

impl ColFuncs for Color {
    #[inline]
    fn blend(&self, c: Self) -> Self {
        Color::RGBA(
            (self.r as f32 * (c.r as f32 / 255.0)) as u8,
            (self.g as f32 * (c.g as f32 / 255.0)) as u8,
            (self.b as f32 * (c.b as f32 / 255.0)) as u8,
            (self.a as f32 * (c.a as f32 / 255.0)) as u8,
        )
    }
    #[inline]
    fn avg(&self, c: Self) -> Self {
        Color::RGBA(
            self.r/2 + c.r/2,
            self.g/2 + c.g/2,
            self.b/2 + c.b/2,            
            self.a/2 + c.a/2,
        )
    }
    #[inline]
    fn scale(&self, s: f32) -> Self {
        Color::RGBA(
            (self.r as f32 * s) as u8,
            (self.g as f32 * s) as u8,
            (self.b as f32 * s) as u8,
            (self.a as f32 * s) as u8,
        )
    }

    #[inline]
    fn avg_f32(&self, s: f32) -> Self {
        let c = (s * 127.0) as u8;
        Color::RGBA(
            self.r/2 + c,
            self.g/2 + c,
            self.b/2 + c,            
            self.a/2 + c,
        )
    }

    #[inline]
    fn add(&self, c: Self) -> Self {
        Color::RGBA(
            clamp(self.r + c.r, 0, 255),
            clamp(self.g + c.g, 0, 255), 
            clamp(self.b + c.b, 0, 255), 
            clamp(self.a + c.a, 0, 255)
        )
    }

    fn from_f32_greyscale(f: f32) -> Self {
        let f = (f * 255.0) as u8;
        Color::RGBA(f, f, f, f)
    }
}
