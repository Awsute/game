use crate::ops::{clamp, operations4x4, Tri3d, Vec3};
use crate::world::{quick_inv, point_at, Engine, POISSON_DISK};
use crate::ColFuncs;
use sdl2::pixels::Color;
use sdl2::surface::Surface;
use std::mem::swap;


pub const SHADOW_RESOLUTION: (usize, usize) = (1024, 1024);
pub const SPREAD_VAL: f32 = 1.0;

pub struct Light {
    pub pos: [f32; 4],
    pub col: Color,
    pub dir: [f32; 4],
    pub proj_mat: [[f32; 4]; 4],
    pub look_mat : [[f32; 4]; 4],
    pub buf: Vec<f32>,
}

impl Light {
    pub fn new(pos: [f32; 4], col: Color, dir: [f32; 4], proj_mat: [[f32; 4]; 4])  -> Self {
        Light {
            pos,
            col,
            dir,
            proj_mat,
            look_mat : quick_inv(point_at(pos, pos.add(dir), [0.0, 1.0, 0.0, 1.0])),
            buf: vec![1.0; SHADOW_RESOLUTION.0 * SHADOW_RESOLUTION.1],
        }
    }
    #[inline]
    pub fn edit_shadow_buffer(&mut self, tri: Tri3d) {
        let rw = SHADOW_RESOLUTION.0 as f32 * 0.5;
        let rh = SHADOW_RESOLUTION.1 as f32 * 0.5;
        let t = tri
            .multiply_mat(self.look_mat)
            .multiply_mat(self.proj_mat);

        if tri.normal().dot_product(self.dir) >= 0.0 {
            return;
        }

        let t03 = 1.0 / (t.ps[0][3] + 1.0);
        let t13 = 1.0 / (t.ps[1][3] + 1.0);
        let t23 = 1.0 / (t.ps[2][3] + 1.0);

        let mut c1 = [
            (t.ps[0][0] * t03 + 1.0) * rw,
            (t.ps[0][1] * t03 + 1.0) * rh,
            t.ps[0][2],
            t03,
        ];
        let mut c2 = [
            (t.ps[1][0] * t13 + 1.0) * rw,
            (t.ps[1][1] * t13 + 1.0) * rh,
            t.ps[1][2],
            t13,
        ];
        let mut c3 = [
            (t.ps[2][0] * t23 + 1.0) * rw,
            (t.ps[2][1] * t23 + 1.0) * rh,
            t.ps[2][2],
            t23,
        ];
        if c1[1] > c2[1] {
            swap(&mut c1, &mut c2);
        }

        if c1[1] > c3[1] {
            swap(&mut c1, &mut c3);
        }

        if c2[1] > c3[1] {
            swap(&mut c2, &mut c3);
        }

        let mut dax_step = 0.0;
        let mut dbx_step = 0.0;
        let mut dcx_step = 0.0;
        let mut daz_step = 0.0;
        let mut dbz_step = 0.0;
        let mut dcz_step = 0.0;
        let mut daw_step = 0.0;
        let mut dbw_step = 0.0;
        let mut dcw_step = 0.0;

        let dya = (c2[1] - c1[1]).abs();
        let dyb = (c3[1] - c1[1]).abs();
        let dyc = (c3[1] - c2[1]).abs();

        if dya != 0.0 {
            //point a to point b
            let da = 1.0 / dya;
            dax_step = (c2[0] - c1[0]) * da;
            daz_step = (c2[2] - c1[2]) * da;
            daw_step = (c2[3] - c1[3]) * da;
        }

        if dyb != 0.0 {
            //point a to point c
            let db = 1.0 / dyb;
            dbx_step = (c3[0] - c1[0]) * db;
            dbz_step = (c3[2] - c1[2]) * db;
            dbw_step = (c3[3] - c1[3]) * db;
        };

        if dyc != 0.0 {
            //point b to point c
            let dc = 1.0 / dyc;
            dcx_step = (c3[0] - c2[0]) * dc;
            dcz_step = (c3[2] - c2[2]) * dc;
            dcw_step = (c3[3] - c2[3]) * dc;
        }
        for y in c1[1] as i32 + 1..c3[1] as i32 + 1 {
            if y > 0 && y < SHADOW_RESOLUTION.1 as i32 {
                let mut ax: f32;
                let mut bx: f32;

                let mut az: f32;
                let mut bz: f32;

                let mut aw: f32;
                let mut bw: f32;
                let ys1 = y as f32 - c1[1];
                let ys2 = y as f32 - c2[1];
                if y < c2[1] as i32 + 1 {
                    ax = c1[0] + (ys1) * dax_step;

                    az = c1[2] + (ys1) * daz_step;

                    aw = c1[3] + (ys1) * daw_step;
                } else {
                    ax = c2[0] + (ys2) * dcx_step;
                    
                    az = c2[2] + (ys2) * dcz_step;
                    
                    aw = c2[3] + (ys2) * dcw_step;
                }
                bx = c1[0] + (ys1) * dbx_step;
                bz = c1[2] + (ys1) * dbz_step;
                bw = c1[3] + (ys1) * dbw_step;
                if ax > bx {
                    swap(&mut ax, &mut bx);
                    swap(&mut az, &mut bz);
                    swap(&mut aw, &mut bw);
                }
                let tstep = 1.0 / (bx - ax);
                for x in ax as usize..bx as usize {
                    if x > 0 && x < SHADOW_RESOLUTION.0 {
                        let t = (x as f32 - ax) * tstep;
                        let z = ((1.0 - t) * az + t * bz) * ((1.0 - t) * aw + t * bw);
                        let ind = x as usize + SHADOW_RESOLUTION.0 * y as usize;
                        if z < self.buf[ind] && z > 0.0 {
                            self.buf[ind] = z;
                        }
                    }
                }
            }
        }
    }
    #[inline]
    pub fn is_lit(&self, point: [f32; 4], norm: [f32; 4]) -> f32 {
        let b = clamp(0.005 * (norm.dot_product(self.dir.negative()).acos().tan()), 0.001, 0.1);
        //let b = 0.005;
        let t = point
            .multiply_mat(self.look_mat)
            .multiply_mat(self.proj_mat);

        let t3 = 1.0 / (t[3] + 1.0);
        let f = [(t[0] * t3 + 1.0) * SHADOW_RESOLUTION.0 as f32 * 0.5, (t[1] * t3 + 1.0) * SHADOW_RESOLUTION.1 as f32 * 0.5];
        let d_val = t[2] * t3;
        let mut l = 0.0;
        for item in POISSON_DISK.iter().take(4) { //make the loop customizable (1 to 16 iters)
            let ind = (f[0] + item[0] * SPREAD_VAL) as usize
                + SHADOW_RESOLUTION.0
                    * (f[1] + item[1] * SPREAD_VAL) as usize;
            if ind < SHADOW_RESOLUTION.0 * SHADOW_RESOLUTION.1
                && d_val - b <= self.buf[ind]
                && d_val > 0.0
            {
                l += 1.0 / 4.0;
            }
        }
        l
        
    }
}
