use sdl2::pixels::Color;

pub trait ColFuncs{
    fn blend(&self, c:Self)->Self;
    fn avg(&self, c:Self)->Self;
}
impl ColFuncs for Color{
    #[inline]
    fn blend(&self, c:Self)->Self{
        return Color::from(((self.r as f32*(c.r as f32/255.0)) as u8, (self.g as f32*(c.g as f32/255.0)) as u8, (self.b as f32*(c.b as f32/255.0)) as u8));
    }  
    #[inline]
    fn avg(&self, c:Self)->Self{
        return Color::from((
            ((self.r as u16 + c.r as u16)/2) as u8, 
            ((self.g as u16 + c.g as u16)/2) as u8, 
            ((self.b as u16 + c.b as u16)/2) as u8,
        ));
    }

}