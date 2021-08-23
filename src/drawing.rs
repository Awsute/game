extern crate sdl2;
use sdl2::{render::{WindowCanvas}, pixels::{Color}, surface::{Surface}};
use std::mem::swap;
use crate::world::{Engine};
use crate::{Vec3, Tri3d};
use sdl2::gfx::primitives::DrawRenderer;
use crate::ColFuncs;

pub trait DrawTri{
    fn textured_triangle(&mut self, tri : Tri3d, surf : &Surface, engine : &mut Engine, tri_info : Tri3d);
}
impl DrawTri for WindowCanvas{

    #[inline]
    fn textured_triangle(&mut self, tri : Tri3d, surf : &Surface, engine : &mut Engine, tri_info : Tri3d){
        let s = (engine.camera.window_width as i32, engine.camera.window_height as i32);
        let height = surf.height() as usize;
        let width = surf.width() as usize;
        let pitch = surf.pitch() as usize;
        let buffer = surf.without_lock().unwrap();
        let ps = tri.ps;
        let uvs = tri.uvs;

        let ambient = tri_info.col.blend(engine.ambient);

        let mut c1 = ps[0];
        let mut c2 = ps[1];
        let mut c3 = ps[2];
        let mut i1 = uvs[0]; 
        let mut i2 = uvs[1]; 
        let mut i3 = uvs[2];
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

        
        
        let dya = c2[1] - c1[1];
        let dyb = c3[1] - c1[1];
        let dyc = c3[1] - c2[1];
        
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



        for y in c1[1] as i32+1..c3[1] as i32+1{
            if y > 0 && y < s.1{
                let mut tex_s : [f32;3];
                
                let mut tex_e : [f32;3];
                
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

                    tex_s = [
                        i1[0] + (ys1) * du1_step,
                        i1[1] + (ys1) * dv1_step,
                        i1[2] + (ys1) * dw1_step
                    ];
                    
                    tex_e = [
                        i1[0] + (ys1) * du2_step,
                        i1[1] + (ys1) * dv2_step,
                        i1[2] + (ys1) * dw2_step
                    ];
                    

                    ls = l1.add(la_step.scale_c(ys1));
                    le = l1.add(lb_step.scale_c(ys1));

                    point_s = v1.add(dav_step.scale_c(ys1));
                    point_e = v1.add(dbv_step.scale_c(ys1));


                } else {
                    ax = (c2[0] + (ys2) * dcx_step) as i32;
                    bx = (c1[0] + (ys1) * dbx_step) as i32;

                    tex_s = [
                        i2[0] + (ys2) * du3_step,
                        i2[1] + (ys2) * dv3_step,
                        i2[2] + (ys2) * dw3_step
                    ];
                    
                    tex_e = [
                        i1[0] + (ys1) * du2_step,
                        i1[1] + (ys1) * dv2_step,
                        i1[2] + (ys1) * dw2_step
                    ];
                    

                    ls = l2.add(lc_step.scale_c(ys2));
                    le = l1.add(lb_step.scale_c(ys1));

                    point_s = v2.add(dcv_step.scale_c(ys2));
                    point_e = v1.add(dbv_step.scale_c(ys1));

                }
                if ax > bx{
                    swap(&mut ax, &mut bx);
                    swap(&mut tex_s, &mut tex_e);
                    swap(&mut ls, &mut le);
                    swap(&mut point_s, &mut point_e);
                }
                let tstep = 1.0/(bx - ax) as f32;
                
                for x in ax..bx{
                    if x > 0 && x < s.0{
                        
                        let t = (x-ax) as f32*tstep;
                        let tex_w = (1.0 - t) * tex_s[2] + t * tex_e[2];
                        let dbi = (x+s.0*y) as usize;
                        if tex_w >= engine.depth_buffer[dbi] || engine.transparency_buffer[dbi].0 > 0.0{
                            let tr_buf = engine.transparency_buffer[dbi];
                            let d_buf = engine.depth_buffer[dbi];

                            let ind = (pitch/width) * ((width as f32-0.1) * ((1.0 - t) * tex_s[0] + t * tex_e[0])/tex_w) as usize + pitch * ((height as f32-0.1) * ((1.0 - t) * tex_s[1] + t * tex_e[1])/tex_w) as usize;
                            
                            //note: col = (diff*cos_theta + spec*r^5)*shadow*light_color*light_power + ambient
                            
                            let col = if ind < buffer.len()-2{
                                let norm = ls.scale_c(1.0-t).add(le.scale_c(t)).normalize();
                                let point = point_s.scale_c(1.0-t).add(point_e.scale_c(t)).scale_c(1.0/tex_w);
                                
                                let mut add_col = Color::BLACK;
                                for light in &engine.lights{
                                    let dp = norm.dot_product(light.dir.negative());

                                    let r = norm.scale_c(2.0*dp).add(light.dir).normalize().dot_product(point.negative().add(engine.camera.pos).normalize())*tri_info.rfl;
                                    
                                    let g = light.is_lit(point, norm);
                                    
                                    let diff = tri_info.col.blend(Color::from_f32_greyscale(dp));
                                    let modif = Color::from_f32_greyscale(r.powi(5)).avg(diff);
                                    
                                    let shadow = Color::from_f32_greyscale(g);
                                    add_col = add_col.add(modif.blend(shadow).blend(light.col));
                                }
                                
                                
                                let pot_col = Color::RGB(buffer[ind], buffer[ind+1], buffer[ind+2]).avg(ambient).avg(add_col);
                                
                                if tex_w < d_buf && tr_buf.0 > 0.0{
                                    tr_buf.1.scale(1.0-tr_buf.0).add(pot_col.scale(tr_buf.0))
                                } else if tex_w >= d_buf && tri_info.trs > 0.0{
                                    tr_buf.1.scale(tri_info.trs).add(pot_col.scale(1.0-tri_info.trs))
                                } else {
                                    pot_col
                                }
                            } else {
                                ambient
                            };

                            if tex_w >= d_buf{
                                engine.depth_buffer[dbi] = tex_w;
                                
                                engine.transparency_buffer[dbi].0 = -crate::ops::clamp(1.0-tr_buf.0-tri_info.trs, -1.0, 0.0);
                                engine.transparency_buffer[dbi].1 = col;
                            }
                            self.pixel(
                                x as i16,
                                y as i16,
                                col
                            );
                        }
                    }
                }
            }
        }
          
    }  
}
