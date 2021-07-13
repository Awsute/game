extern crate sdl2;
use sdl2::{render::{WindowCanvas}, pixels::{Color}};
use std::mem::swap;
use crate::world::{Engine, look_at, point_at};
use crate::ops::{clamp};
use crate::{Vec3, Tri3d};
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::rect::Point;
use crate::ColFuncs;
use crate::light::Light;

pub trait DrawTri{
    fn draw_triangle(&mut self, p1 : [f32;3], p2 : [f32;3], p3 : [f32;3], c : Color);
    fn fill_triangle(&mut self, p1 : [f32;3], p2 : [f32;3], p3 : [f32;3], c : Color);
    fn textured_triangle(&mut self, p : [[f32;4];3], t : [[f32;3];3], buffer : &[u8], pitch : usize, width : f32, height : f32, engine : &mut Engine, tri_info : Tri3d, light : &mut Light, ambient : Color, draw : bool);
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
    fn textured_triangle(&mut self, p : [[f32;4];3], t : [[f32;3];3], buffer : &[u8], pitch : usize, width : f32, height : f32, engine : &mut Engine, tri_info : Tri3d, light : &mut Light, ambient : Color, draw : bool){
        let s = (engine.camera.window_width, engine.camera.window_height);
        let mut c1 = p[0];
        let mut c2 = p[1];
        let mut c3 = p[2];
        let mut i1 = t[0]; 
        let mut i2 = t[1]; 
        let mut i3 = t[2];
        let mut l1 = tri_info.ns[0];
        let mut l2 = tri_info.ns[1];
        let mut l3 = tri_info.ns[2];

        let mut v1 = tri_info.ps[0];
        let mut v2 = tri_info.ps[1];
        let mut v3 = tri_info.ps[2];


        if c1[1] > c2[1]{
            swap(&mut c1, &mut c2);
            swap(&mut i1, &mut i2);
            swap(&mut l1, &mut l2);
            swap(&mut v1, &mut v2);
        }
        
        if c1[1] > c3[1]{
            swap(&mut c1, &mut c3);
            swap(&mut i1, &mut i3);
            swap(&mut l1, &mut l3);
            swap(&mut v1, &mut v3);
        }
        
        if c2[1] > c3[1]{
            swap(&mut c2, &mut c3);
            swap(&mut i2, &mut i3);
            swap(&mut l2, &mut l3);
            swap(&mut v2, &mut v3);
        }
        

        let mut dax_step = 0.0; let mut dbx_step = 0.0; let mut dcx_step = 0.0;
        let mut du1_step = 0.0; let mut dv1_step = 0.0; let mut dw1_step = 0.0;
        let mut du2_step = 0.0; let mut dv2_step = 0.0; let mut dw2_step = 0.0;
        let mut du3_step = 0.0; let mut dv3_step = 0.0; let mut dw3_step = 0.0;
        
        let mut dav_step = [0.0, 0.0, 0.0, 0.0]; 
        let mut dbv_step = [0.0, 0.0, 0.0, 0.0]; 
        let mut dcv_step = [0.0, 0.0, 0.0, 0.0];
        
        
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
            
            dav_step = v2.subtract(v1).scale_c(da);
            la_step = l2.subtract(l1).scale_c(da);
        }
        
        if dyb != 0.0{ //point a to point c
            let db = 1.0/dyb;
            dbx_step = (c3[0] - c1[0])*db;
            du2_step = (i3[0] - i1[0])*db; 
            dv2_step = (i3[1] - i1[1])*db;
            dw2_step = (i3[2] - i1[2])*db;
            
            dbv_step = v3.subtract(v1).scale_c(db);
            lb_step = l3.subtract(l1).scale_c(db);

        };
        
        
        if dyc != 0.0{ //point b to point c
            let dc = 1.0/dyc;
            dcx_step = (c3[0] - c2[0])*dc;
            du3_step = (i3[0] - i2[0])*dc;
            dv3_step = (i3[1] - i2[1])*dc;
            dw3_step = (i3[2] - i2[2])*dc;
            
            dcv_step = v3.subtract(v2).scale_c(dc);
            lc_step = l3.subtract(l2).scale_c(dc);

        }



        if dya != 0.0 || dyc != 0.0{           
            for y in c1[1] as i32+1..c3[1] as i32+1{
                if y > 0 && y < s.1 as i32{
                    let mut tex_su : f32;
                    let mut tex_sv : f32;
                    let mut tex_sw : f32;
                    
                    let mut tex_eu : f32;
                    let mut tex_ev : f32;
                    let mut tex_ew : f32;
                    
                    let mut point_s : [f32;4];
                    let mut point_e : [f32;4];

                    let mut ax : i32;
                    let mut bx : i32;

                    let mut ls : [f32;4];
                    let mut le : [f32;4];
                    let ys1 = y as f32-c1[1];
                    let ys2 = y as f32-c2[1];
                    if y < c2[1] as i32+1 {
                        ax = (c1[0] + (ys1) * dax_step) as i32;
                        bx = (c1[0] + (ys1) * dbx_step) as i32;

                        tex_su = i1[0] + (ys1) * du1_step;
                        tex_sv = i1[1] + (ys1) * dv1_step;
                        tex_sw = i1[2] + (ys1) * dw1_step;
                        
                        tex_eu = i1[0] + (ys1) * du2_step;
                        tex_ev = i1[1] + (ys1) * dv2_step;
                        tex_ew = i1[2] + (ys1) * dw2_step;

                        ls = l1.add(la_step.scale_c(ys1));
                        le = l1.add(lb_step.scale_c(ys1));

                        point_s = v1.add(dav_step.scale_c(ys1));
                        point_e = v1.add(dbv_step.scale_c(ys1));


                    } else {
                        ax = (c2[0] + (ys2) * dcx_step) as i32;
                        bx = (c1[0] + (ys1) * dbx_step) as i32;

                        tex_su = i2[0] + (ys2) * du3_step;
                        tex_sv = i2[1] + (ys2) * dv3_step;
                        tex_sw = i2[2] + (ys2) * dw3_step;
                        
                        tex_eu = i1[0] + (ys1) * du2_step;
                        tex_ev = i1[1] + (ys1) * dv2_step;
                        tex_ew = i1[2] + (ys1) * dw2_step;
                        

                        ls = l2.add(lc_step.scale_c(ys2));
                        le = l1.add(lb_step.scale_c(ys1));

                        point_s = v2.add(dcv_step.scale_c(ys2));
                        point_e = v1.add(dbv_step.scale_c(ys1));

                    }
                    if ax > bx{
                        swap(&mut ax, &mut bx);
                        swap(&mut tex_su, &mut tex_eu);
                        swap(&mut tex_sv, &mut tex_ev);
                        swap(&mut tex_sw, &mut tex_ew);
                        swap(&mut ls, &mut le);
                        swap(&mut point_s, &mut point_e);
                    }
                    let tstep = 1.0/(bx - ax) as f32;
                    for x in ax..bx{
                        if x > 0 && x < s.0 as i32{
                            
                            let t = (x-ax) as f32*tstep;
                            let tex_w = (1.0 - t) * tex_sw + t * tex_ew;
                            let dbi = (x+s.0 as i32*y) as usize;
                            if tex_w >= engine.depth_buffer[dbi] || engine.transparency_buffer[dbi].0 > 0.0{
                                let tr_buf = engine.transparency_buffer[dbi];

                                let ind = (pitch/width as usize) * ((width-0.1) * ((1.0 - t) * tex_su + t * tex_eu)/tex_w) as usize + pitch * ((height-0.1) * ((1.0 - t) * tex_sv + t * tex_ev)/tex_w) as usize;
                                let norm = ls.scale_c(1.0-t).add(le.scale_c(t));
                                let point = point_s.scale_c(1.0-t).add(point_e.scale_c(t)).scale_c(1.0/tex_w);
                                let dp = norm.dot_product(light.dir.negative().normalize());
                                let cos_theta = clamp(dp, 0.0, 1.0);

                                let c_cos_theta = (cos_theta*255.0) as u8;

                                //let c = (dp.powi(5)*255.0) as u8;
                                let r = norm.scale_c(2.0*(dp)).subtract(light.dir.negative()).normalize().dot_product(engine.camera.dir.negative());
                                let r = clamp(r, 0.0, 1.0)*tri_info.rfl;
                                let g = light.is_lit(point);

                                let shadow_c = (g*255.0) as u8;
                                //note: col = (diff*cos_theta + spec*r^5)*shadow*light_color*light_power + ambient
                                let col = if ind < buffer.len()-2{
                                    let diff = tri_info.col.blend(Color::RGB(c_cos_theta, c_cos_theta, c_cos_theta));
                                    let specr = (r.powi(5)*255.0) as u8;
                                    let spec_r = Color::RGB(specr, specr, specr);
                                    let modif = spec_r.avg(diff);

                                    let shadow = Color::RGB(shadow_c, shadow_c, shadow_c);
                                    let pot_col = Color::RGB(buffer[ind], buffer[ind+1], buffer[ind+2]).blend(
                                        modif.blend(shadow).blend(light.col).avg(ambient)
                                    );
                                    let col = if tr_buf.0 > 0.0{
                                        tr_buf.1.scale(1.0-tr_buf.0).add(pot_col.scale(tr_buf.0))
                                    } else {
                                        pot_col
                                    };
                                    col

                                } else {
                                    Color::BLACK
                                };

                                
                                if draw{
                                    self.set_draw_color(col);
                                    self.draw_point(
                                        Point::new(x, y)
                                    );
                                } else if tex_w >= engine.depth_buffer[dbi]{
                                    engine.depth_buffer[dbi] = tex_w;
                                    engine.transparency_buffer[dbi] = if tr_buf.0 > 0.0{
                                        ((tri_info.trs-tr_buf.0).abs(), col)
                                    } else {
                                        (tri_info.trs, col)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }   
    }  
}
