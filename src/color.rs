use sdl2::pixels::Color;
use std::ops::Add;
pub trait ColFuncs{
    fn blend(&self, c:Self)->Self;
    fn avg(&self, c:Self)->Self;
    fn scale(&self, s:f32)->Self;
    fn add(&self, c:Self)->Self;
    fn from_f32_greyscale(f:f32)->Self;
}

impl ColFuncs for Color{
    #[inline]
    fn blend(&self, c:Self)->Self{
        Color::RGB((self.r as f32*(c.r as f32/255.0)) as u8, (self.g as f32*(c.g as f32/255.0)) as u8, (self.b as f32*(c.b as f32/255.0)) as u8)
    }  
    #[inline]
    fn avg(&self, c:Self)->Self{
        Color::RGB(
            ((self.r as u16 + c.r as u16)/2) as u8, 
            ((self.g as u16 + c.g as u16)/2) as u8, 
            ((self.b as u16 + c.b as u16)/2) as u8,
        )
    }
    #[inline]
    fn scale(&self, s:f32)->Self{
        let c = (s*255.0) as u8;
        self.blend(Color::RGB(c, c, c))
    }

    #[inline]
    fn add(&self, c:Self)->Self{
        Color::RGB(self.r+c.r, self.g+c.g, self.b+c.b)
    }

    fn from_f32_greyscale(f: f32)->Self{
        let f = (f*255.0) as u8;
        Color::RGB(f, f, f)
    }

}