extern crate sdl2;
use sdl2::{render::{WindowCanvas}, pixels::{Color}};
use std::mem::swap;
use crate::Engine;
use crate::{Vec3, Tri3d};
use sdl2::gfx::primitives::DrawRenderer;
use crate::ColFuncs;

pub trait DrawTri{
    fn draw_triangle(&mut self, p1 : [f32;3], p2 : [f32;3], p3 : [f32;3], c : Color);
    fn fill_triangle(&mut self, p1 : [f32;3], p2 : [f32;3], p3 : [f32;3], c : Color);
    fn textured_triangle(&mut self, p1 : [f32;3], p2 : [f32;3], p3 : [f32;3], t1 : [f32;3], t2 : [f32;3], t3 : [f32;3], c: Color, buffer : &[u8], pitch : usize, width : f32, height : f32, engine : &mut Engine, light_info : [[f32;4];3], light_dir : [f32;4], light_color : Color);
}
impl DrawTri for WindowCanvas{
    #[inline]
    fn draw_triangle(&mut self, p1 : [f32;3], p2 : [f32;3], p3 : [f32;3], c : Color){
        self.polygon(
            &[p1[0] as i16, p2[0] as i16, p3[0] as i16], 
            &[p1[1] as i16, p2[1] as i16, p3[1] as i16], 
            c
        );
    }
    #[inline]
    fn fill_triangle(&mut self, p1 : [f32;3], p2 : [f32;3], p3 : [f32;3], c : Color){
        self.filled_polygon(
            &[p1[0] as i16, p2[0] as i16, p3[0] as i16],
            &[p1[1] as i16, p2[1] as i16, p3[1] as i16],
            c
        );
        
    }
    #[inline]
    fn textured_triangle(&mut self, p1 : [f32;3], p2 : [f32;3], p3 : [f32;3], t1 : [f32;3], t2 : [f32;3], t3 : [f32;3], c: Color, buffer : &[u8], pitch : usize, width : f32, height : f32, engine : &mut Engine, light_info : [[f32;4];3], light_dir : [f32;4], light_color : Color){
        let s = (engine.window_width, engine.window_height);
        let mut c1 = p1;
        let mut c2 = p2;
        let mut c3 = p3;
        let mut i1 = t1; 
        let mut i2 = t2; 
        let mut i3 = t3;
        let mut l1 = light_info[0];
        let mut l2 = light_info[1];
        let mut l3 = light_info[2];
        if c1[1] > c2[1]{
            swap(&mut c1, &mut c2);
            swap(&mut i1, &mut i2);
            swap(&mut l1, &mut l2);
        }
        
        if c1[1] > c3[1]{
            swap(&mut c1, &mut c3);
            swap(&mut i1, &mut i3);
            swap(&mut l1, &mut l3);
        }
        
        if c2[1] > c3[1]{
            swap(&mut c2, &mut c3);
            swap(&mut i2, &mut i3);
            swap(&mut l2, &mut l3);
        }
        

        let mut dax_step = 0.0; let mut dbx_step = 0.0; let mut dcx_step = 0.0;
        let mut du1_step = 0.0; let mut dv1_step = 0.0; let mut dw1_step = 0.0;
        let mut du2_step = 0.0; let mut dv2_step = 0.0; let mut dw2_step = 0.0;
        let mut du3_step = 0.0; let mut dv3_step = 0.0; let mut dw3_step = 0.0;
        
        let mut la_step = [0.0, 0.0, 0.0, 1.0];
        let mut lb_step = [0.0, 0.0, 0.0, 1.0];
        let mut lc_step = [0.0, 0.0, 0.0, 1.0];

        
        
        let dya = (c2[1] - c1[1]).abs() as f32;
        let dyb = (c3[1] - c1[1]).abs() as f32;
        let dyc = (c3[1] - c2[1]).abs() as f32;
        
        if dya != 0.0{ //point a to point b
            let da = 1.0/dya;
            dax_step = (c2[0] - c1[0])*da;
            du1_step = (i2[0] - i1[0])*da;
            dv1_step = (i2[1] - i1[1])*da;
            dw1_step = (i2[2] - i1[2])*da;

            la_step = l2.subtract(l1).scale([da, da, da, 1.0]);
        }
        
        if dyb != 0.0{ //point a to point c
            let db = 1.0/dyb;
            dbx_step = (c3[0] - c1[0])*db;
            du2_step = (i3[0] - i1[0])*db; 
            dv2_step = (i3[1] - i1[1])*db;
            dw2_step = (i3[2] - i1[2])*db;

            lb_step = l3.subtract(l1).scale([db, db, db, 1.0]);

        };
        
        
        if dyc != 0.0{ //point b to point c
            let dc = 1.0/dyc;
            dcx_step = (c3[0] - c2[0])*dc;
            du3_step = (i3[0] - i2[0])*dc;
            dv3_step = (i3[1] - i2[1])*dc;
            dw3_step = (i3[2] - i2[2])*dc;

            lc_step = l3.subtract(l2).scale([dc, dc, dc, 1.0]);

        }


        /*
        Linearly interpolate between vectors??
        Interpolate each value
        Yes ok
        */

        if dya != 0.0 || dyc != 0.0{           
            for y in c1[1] as i32+1..c3[1] as i32+1{
                if y > 0 && y < s.1 as i32{
                    let mut tex_su : f32;
                    let mut tex_sv : f32;
                    let mut tex_sw : f32;
                    
                    let mut tex_eu : f32;
                    let mut tex_ev : f32;
                    let mut tex_ew : f32;
                    
                    let mut ax : i32;
                    let mut bx : i32;

                    let mut ls : [f32;4];
                    let mut le : [f32;4];
                    let ys1 = y as f32-c1[1];
                    let ys2 = y as f32-c2[1];
                    let ysl1 = [ys1, ys1, ys1, 1.0];
                    if y < c2[1] as i32+1 {
                        ax = (c1[0] + (ys1) * dax_step) as i32;
                        bx = (c1[0] + (ys1) * dbx_step) as i32;

                        tex_su = i1[0] + (ys1) * du1_step;
                        tex_sv = i1[1] + (ys1) * dv1_step;
                        tex_sw = i1[2] + (ys1) * dw1_step;
                        
                        tex_eu = i1[0] + (ys1) * du2_step;
                        tex_ev = i1[1] + (ys1) * dv2_step;
                        tex_ew = i1[2] + (ys1) * dw2_step;
                        

                        ls = l1.add(la_step.scale(ysl1));
                        le = l1.add(lb_step.scale(ysl1));


                    } else {
                        ax = (c2[0] + (ys2) * dcx_step) as i32;
                        bx = (c1[0] + (ys1) * dbx_step) as i32;

                        tex_su = i2[0] + (ys2) * du3_step;
                        tex_sv = i2[1] + (ys2) * dv3_step;
                        tex_sw = i2[2] + (ys2) * dw3_step;
                        
                        tex_eu = i1[0] + (ys1) * du2_step;
                        tex_ev = i1[1] + (ys1) * dv2_step;
                        tex_ew = i1[2] + (ys1) * dw2_step;
                        

                        ls = l2.add(lc_step.scale([ys2, ys2, ys2, 1.0]));
                        le = l1.add(lb_step.scale(ysl1));

                    }
                    if ax > bx{
                        swap(&mut ax, &mut bx);
                        swap(&mut tex_su, &mut tex_eu);
                        swap(&mut tex_sv, &mut tex_ev);
                        swap(&mut tex_sw, &mut tex_ew);
                        swap(&mut ls, &mut le);
                    }
                    let tstep = 1.0/(bx - ax) as f32;
                    for x in ax..bx{
                        if x > 0 && x < s.0 as i32{

                            let t = (x-ax) as f32*tstep;
                            let tex_w = (1.0 - t) * tex_sw + t * tex_ew;
                            let dbi = (x+s.0 as i32*y) as usize;

                            if tex_w > engine.depth_buffer[dbi]{
                                engine.depth_buffer[dbi] = tex_w;
                                let t1 = 1.0-t;
                                let dp = ls.scale([t1, t1, t1, 1.0]).add(le.scale([t, t, t, 1.0])).normalize().dot_product(light_dir.normalize());
                                let ind = (
                                    (pitch/width as usize) * ((width-0.1) * ((1.0 - t) * tex_su + t * tex_eu)/tex_w) as usize +
                                    pitch * ((height-0.1) * ((1.0 - t) * tex_sv + t * tex_ev)/tex_w) as usize
                                );
                                
                                
                                
                                let c = (dp*255.0) as u8;
                                self.pixel(
                                    x as i16,
                                    y as i16, 
                                    if ind < buffer.len()-2{
                                        Color::from((buffer[ind], buffer[ind+1], buffer[ind+2])).blend(
                                            Color::from((c, c, c)).blend(light_color)
                                        )
                                    } else {
                                        Color::WHITE
                                    }
                                );
                                
                            }
                        }
                    }
                    
                }
            }
        }
        
    }
    
}
