use sdl2::pixels::{Color};
use crate::Engine;
use crate::ops::{Tri3d, Vec3, operations4x4, look_at};
use std::mem::swap;

pub struct Light{
    pub pos : [f32;4],
    pub col : Color,
    pub dir : [f32;4],
    pub proj_mat : [[f32;4];4],
    pub buf : Vec<Vec<f32>>
}
pub const SHADOW_RESOLUTION : (usize, usize) = (1024, 1024);

impl Light{
    pub fn new(pos:[f32;4], col:Color, dir:[f32;4], proj_mat:[[f32;4];4])->Self{
        return Light{pos, col, dir, proj_mat : proj_mat.multiply(look_at(pos, [0.1, 0.1, 5.0, 1.0], [0.0, 1.0, 0.0, 1.0])), buf:Vec::new()};
    }

    pub fn edit_shadow_buffer(&mut self, tri : Tri3d){
        if self.buf.len() < SHADOW_RESOLUTION.0-1{
            for i in self.buf.len()..SHADOW_RESOLUTION.0{
                self.buf.push(Vec::new());
                if self.buf[i].len() < SHADOW_RESOLUTION.1-1{
                    for j in self.buf[i].len()..SHADOW_RESOLUTION.1{
                        self.buf[i].push(0.0)
                    }
                }
            }
        }
        let rw = SHADOW_RESOLUTION.0 as f32/2.0;
        let rh = SHADOW_RESOLUTION.1 as f32/2.0;
                    
        let t = tri.scale([rw, rh, 1.0, 1.0]).multiply_mat(self.proj_mat);
        let t03 = t.ps[0][3]; let t13 = t.ps[1][3]; let t23 = t.ps[2][3];
        let mut c1 = [(t.ps[0][0]/t03+rw), (t.ps[0][1]/t03+rh), 1000.0/t.ps[0][2]];    
        let mut c2 = [(t.ps[1][0]/t13+rw), (t.ps[1][1]/t13+rh), 1000.0/t.ps[1][2]];
        let mut c3 = [(t.ps[2][0]/t23+rw), (t.ps[2][1]/t23+rh), 1000.0/t.ps[2][2]];
        if c1[1] > c2[1]{
            swap(&mut c1, &mut c2);
        }
        
        if c1[1] > c3[1]{
            swap(&mut c1, &mut c3);
        }
        
        if c2[1] > c3[1]{
            swap(&mut c2, &mut c3);
        }
        

        let mut dax_step = 0.0; let mut dbx_step = 0.0; let mut dcx_step = 0.0;
        let mut daz_step = 0.0; let mut dbz_step = 0.0; let mut dcz_step = 0.0;

        
        
        let dya = (c2[1] - c1[1]).abs() as f32;
        let dyb = (c3[1] - c1[1]).abs() as f32;
        let dyc = (c3[1] - c2[1]).abs() as f32;
        
        if dya != 0.0{ //point a to point b
            let da = 1.0/dya;
            dax_step = (c2[0] - c1[0])*da;
            daz_step = (c2[2] - c1[2])*da;

        }
        
        if dyb != 0.0{ //point a to point c
            let db = 1.0/dyb;
            dbx_step = (c3[0] - c1[0])*db;
            dbz_step = (c3[2] - c1[2])*db;

        };
        
        
        if dyc != 0.0{ //point b to point c
            let dc = 1.0/dyc;
            dcx_step = (c3[0] - c2[0])*dc;
            dcz_step = (c3[2] - c2[2])*dc;

        }

        for y in c1[1] as i32+1..c3[1] as i32+1{
            if y > 0 && y < SHADOW_RESOLUTION.1 as i32{
                let mut ax : i32;
                let mut bx : i32;


                let mut az : f32;
                let mut bz : f32;
                let ys1 = y as f32-c1[1];
                let ys2 = y as f32-c2[1];
                if y < c2[1] as i32+1 {
                    ax = (c1[0] + (ys1) * dax_step) as i32;
                    bx = (c1[0] + (ys1) * dbx_step) as i32;

                    az = c1[2] + (ys1) * daz_step;
                    bz = c1[2] + (ys1) * dbz_step;

                } else {
                    ax = (c2[0] + (ys2) * dcx_step) as i32;
                    bx = (c1[0] + (ys1) * dbx_step) as i32;

                    az = c2[2] + (ys2) * dcz_step;
                    bz = c1[2] + (ys1) * dbz_step;


                }
                if ax > bx{
                    swap(&mut ax, &mut bx);
                    swap(&mut az, &mut bz);
                }
                let tstep = 1.0/(bx - ax) as f32;
                for x in ax..bx{
                    if x > 0 && x < SHADOW_RESOLUTION.0 as i32{
                        
                        let t = (x-ax) as f32*tstep;
                        let z = (1.0 - t) * az + t * bz;
                        if z > self.buf[x as usize][y as usize]{
                            self.buf[x as usize][y as usize] = z;
                            println!("{}", z*1000.0);
                        }
                    }
                }
            }
        }
    }
    pub fn is_lit(&mut self, point:[f32;4])->f32{
        let rw = SHADOW_RESOLUTION.0 as f32;
        let rh = SHADOW_RESOLUTION.1 as f32;
        let t = point/*.scale([rw/2.0, rh/2.0, 1.0, 1.0])*/.multiply_mat(self.proj_mat);
        let t03 = t[3];
        let f = [(t[0]/t03), (t[1]/t03), 1000.0/t[2]]; 
        if SHADOW_RESOLUTION.0 > f[0] as usize && SHADOW_RESOLUTION.1 > f[1] as usize {
            if f[2] <= self.buf[f[0] as usize][f[1] as usize]{
                //self.buf[f[0] as usize][f[1] as usize] = f[2];
                return self.buf[f[0] as usize][f[1] as usize];
            }
        }
        return 0.0;
        
    }
}
