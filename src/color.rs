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
            ((self.r as u16 + c.r as u16) / 2) as u8,
            ((self.g as u16 + c.g as u16) / 2) as u8,
            ((self.b as u16 + c.b as u16) / 2) as u8,            
            ((self.a as u16 + c.a as u16) / 2) as u8,
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
        let c = (s * 255.0) as u16;
        Color::RGBA(
            ((self.r as u16 + c) / 2) as u8,
            ((self.g as u16 + c) / 2) as u8,
            ((self.b as u16 + c) / 2) as u8,            
            ((self.a as u16 + c) / 2) as u8,
        )
    }

    #[inline]
    fn add(&self, c: Self) -> Self {
        Color::RGBA(
            clamp(self.r as f32 + c.r as f32, 0.0, 255.0) as u8,
            clamp(self.g as f32 + c.g as f32, 0.0, 255.0) as u8, 
            clamp(self.b as f32 + c.b as f32, 0.0, 255.0) as u8, 
            clamp(self.a as f32 + c.a as f32, 0.0, 255.0) as u8
        )
    }

    fn from_f32_greyscale(f: f32) -> Self {
        let f = (f * 255.0) as u8;
        Color::RGBA(f, f, f, f)
    }
}
