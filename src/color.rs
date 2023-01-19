use sdl2::pixels::Color;
use crate::ops::{clamp};
pub trait ColFuncs {
    fn blend_with_alpha(&self, c: Self) -> Self;
    fn blend(&self, c: Self) -> Self;
    fn avg(&self, c: Self) -> Self;
    fn scale(&self, s: f32) -> Self;
    fn avg_f32(&self, s: f32) -> Self;
    fn add(&self, c: Self) -> Self;
    fn from_f32_greyscale(f: f32) -> Self;
}

impl ColFuncs for Color {
    #[inline]
    fn blend_with_alpha(&self, c: Self) -> Self {
        Color::RGBA(
            (self.r as f32 * (c.r as f32 / 255.0)) as u8,
            (self.g as f32 * (c.g as f32 / 255.0)) as u8,
            (self.b as f32 * (c.b as f32 / 255.0)) as u8,
            (self.a as f32 * (c.a as f32 / 255.0)) as u8,
        )
    }
    #[inline] 
    fn blend(&self, c: Self) -> Self{
        Color::RGB(
            (self.r as f32 * (c.r as f32 / 255.0)) as u8,
            (self.g as f32 * (c.g as f32 / 255.0)) as u8,
            (self.b as f32 * (c.b as f32 / 255.0)) as u8,
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
            clamp(self.r as u16 + c.r as u16, 0, 255) as u8,
            clamp(self.g as u16 + c.g as u16, 0, 255) as u8, 
            clamp(self.b as u16 + c.b as u16, 0, 255) as u8, 
            clamp(self.a as u16 + c.a as u16, 0, 255) as u8
        )
    }

    fn from_f32_greyscale(f: f32) -> Self {
        let f = (f * 255.0) as u8;
        Color::RGBA(f, f, f, f)
    }
}
#[inline]
pub fn avg_cols(cols : &[Color]) -> Color{
    let mut a : usize = 0;
    let mut b : usize = 0;
    let mut g : usize = 0;
    let mut r : usize = 0;
    for c in cols{
        r = r + c.r as usize;
        g = g + c.g as usize;
        b = b + c.b as usize;
        a = a + c.a as usize;
    }
    let ln = cols.len();
    Color::RGBA((r/ln) as u8, (g/ln) as u8, (b/ln) as u8, (a/ln) as u8)
}
