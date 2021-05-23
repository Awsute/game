use sdl2::pixels::{Color};
use crate::ColFuncs;
use crate::world::{Engine, look_at, point_at, POISSON_DISK};
use crate::ops::{Tri3d, Vec3, operations4x4};
use std::mem::swap;

pub struct Light{
    pub pos : [f32;4],
    pub col : Color,
    pub dir : [f32;4],
    pub proj_mat : [[f32;4];4],
    pub buf : Vec<f32>,
}
pub const SHADOW_RESOLUTION : (usize, usize) = (1024, 1024);

impl Light{
    pub fn new(pos:[f32;4], col:Color, dir:[f32;4], proj_mat:[[f32;4];4])->Self{
        return Light{pos, col, dir, proj_mat : proj_mat, buf:vec![1.0; SHADOW_RESOLUTION.0*SHADOW_RESOLUTION.1]};
    }
    #[inline]
    pub fn edit_shadow_buffer(&mut self, tri : Tri3d){
        let rw = SHADOW_RESOLUTION.0 as f32/2.0;
        let rh = SHADOW_RESOLUTION.1 as f32/2.0;

        let tr = tri.multiply_mat(look_at(self.pos, self.pos.add(self.dir), [0.0, 1.0, 0.0, 1.0]));
        if tr.normal().dot_product(tr.ps[0]) >= 0.0{
            return;
        }
        let t = tr.multiply_mat(self.proj_mat).scale([rw, rh, 1.0, 1.0]);
        let t03 = t.ps[0][3]; let t13 = t.ps[1][3]; let t23 = t.ps[2][3];
        let mut c1 = [(t.ps[0][0]/t03+rw), (t.ps[0][1]/t03+rh), -t.ps[0][2]];    
        let mut c2 = [(t.ps[1][0]/t13+rw), (t.ps[1][1]/t13+rh), -t.ps[1][2]];
        let mut c3 = [(t.ps[2][0]/t23+rw), (t.ps[2][1]/t23+rh), -t.ps[2][2]];
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
                        let ind = x as usize + SHADOW_RESOLUTION.0 * y as usize;
                        if z < self.buf[ind] && z > 0.0 && z < 1.0{
                            self.buf[ind] = z;
                        }
                    }
                }
            }
        }
    }
    #[inline]
    pub fn is_lit(&mut self, point:[f32;4])->f32{
        let rw = SHADOW_RESOLUTION.0 as f32/2.0;
        let rh = SHADOW_RESOLUTION.1 as f32/2.0;
        let t = point.multiply_mat(look_at(self.pos, self.pos.add(self.dir), [0.0, 1.0, 0.0, 1.0])).multiply_mat(self.proj_mat);
        let f = [(t[0]*rw/t[3]+rw) as usize, (t[1]*rh/t[3]+rh) as usize]; 
        let d_val = -t[2];
        let ind = f[0] + SHADOW_RESOLUTION.0 * f[1];
        if ind < SHADOW_RESOLUTION.1*SHADOW_RESOLUTION.1 {
            if d_val-0.005 <= self.buf[ind] && d_val > 0.0 && d_val < 1.0{
                return 1.0
            }
        }
        return 0.0;
        
    }
}
