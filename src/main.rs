//when shipping game, make sure you got everything in the folder with the game
extern crate sdl2;
extern crate gl;
use sdl2::pixels;
use sdl2::image;
use image::{LoadSurface};
use pixels::{Color};
use sdl2::render::{Canvas, Texture, TextureAccess, WindowCanvas, TextureCreator};
use sdl2::audio::{AudioCallback, AudioSpecWAV, AudioCVT, AudioSpecDesired};
use sdl2::rect::{Point, Rect};
use sdl2::surface::{Surface};
use std::fs::{File, read_to_string};
use std::io::{Read, BufReader, BufRead};
use sdl2::event::{EventType::Window, Event};
use sdl2::keyboard::{Scancode, Keycode};
use sdl2::mouse::{MouseButton};
use sdl2::EventPump;
use sdl2::sys::gfx::primitives::texturedPolygon;
use std::time::Duration;
use std::any::{Any, TypeId};
use std::path::Path;
use std::borrow::Borrow;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::gfx::framerate::FPSManager;

trait Surf{
    fn color_at(&self, x:f32, y:f32)->Color;
}
impl Surf for Surface<'_>{
    fn color_at(&self, x: f32, y: f32)->Color{
        let buf = self.without_lock().unwrap();
        let u = (x) as usize;
        let v = (y) as usize;
        let ind = 3*u+self.pitch() as usize*v;
        return if ind < buf.len()-2{Color::from((buf[ind], buf[ind+1], buf[ind+2]))} else {Color::BLACK};
    }
}
trait ColFuncs{
    fn blend(&self, c:Self)->Self;
}
impl ColFuncs for Color{
    fn blend(&self, c:Self)->Self{
        return Color::from(((self.r as f32*(c.r as f32/255.0)) as u8, (self.g as f32*(c.g as f32/255.0)) as u8, (self.b as f32*(c.b as f32/255.0)) as u8));
    }

}
trait Vec3{
    fn scale(&self, scalar : [f32;4])->Self;
    fn add(&self, a : [f32;4])->Self;
    fn subtract(&self, a : [f32;4])->Self;
    fn magnitude(&self)->f32;
    fn normalize(&self)->Self;
    fn negative(&self)->Self;
    fn cross_product(&self, c : [f32;4])->Self;
    fn dot_product(&self, d : [f32;4])->f32;
    fn multiply_mat(&self, mat : [[f32;4];4])->Self;
}
impl Vec3 for [f32;4]{
    fn scale(&self, scalar : [f32;4])->[f32;4]{
        return [self[0]*scalar[0], self[1]*scalar[1], self[2]*scalar[2], self[3]];
    }
    fn add(&self, a : [f32;4])->[f32;4]{
        return [self[0]+a[0], self[1]+a[1], self[2]+a[2], self[3]];
    }
    fn subtract(&self, a : [f32;4])->[f32;4]{
        return [self[0]-a[0], self[1]-a[1], self[2]-a[2], self[3]];
    }
    fn magnitude(&self)->f32{
        return (self[0].powi(2) + self[1].powi(2) + self[2].powi(2)).sqrt();
    }
    fn normalize(&self)->[f32;4]{
        let m = self.magnitude();
        return [self[0]/m, self[1]/m, self[2]/m, self[3]];
    }
    fn negative(&self)->[f32;4]{
        return [-self[0], -self[1], -self[2], 1.0];
    }
    fn cross_product(&self, c: [f32;4])->[f32;4]{
        return [-self[1]*c[2] + c[1]*self[2], -self[2]*c[0] + c[2]*self[0], -self[0]*c[1] + c[0]*self[1], 1.0];
    }
    fn dot_product(&self, d: Self)->f32{
        return self[0]*d[0] + self[1]*d[1] + self[2]*d[2];
    }
    fn multiply_mat(&self, m : [[f32;4];4])->[f32;4]{
        return [
            self[0] * m[0][0] + self[1] * m[1][0] + self[2] * m[2][0] + self[3] * m[3][0],
            self[0] * m[0][1] + self[1] * m[1][1] + self[2] * m[2][1] + self[3] * m[3][1],
            self[0] * m[0][2] + self[1] * m[1][2] + self[2] * m[2][2] + self[3] * m[3][2],
            self[0] * m[0][3] + self[1] * m[1][3] + self[2] * m[2][3] + self[3] * m[3][3]
        ];
    }
}

trait Tri3d{
    fn normal(&self)->[f32;4];
    fn translate(&self, t:[f32;4])->Self;
    fn scale(&self, t:[f32;4])->Self;
    fn center(&self)->[f32;4];
    fn multiply_mat(&self, m:[[f32;4];4])->Self;
    fn upd(&self, scalar : [f32;4], trans : [f32;4], rot : [f32;4], rot_point : [f32;4], center : [f32;4])->Self;
}
impl Tri3d for ([[f32;4];3], [[f32;3];3]){
    fn normal(&self)->[f32;4]{
        return self.0[2].subtract(self.0[0]).cross_product(self.0[1].subtract(self.0[0])).normalize();//sheeeesh
    }
    fn translate(&self, t : [f32;4])->Self{
        return ([self.0[0].add(t), self.0[1].add(t), self.0[2].add(t)], self.1);
    }
    fn scale(&self, t : [f32;4])->Self{
        return ([self.0[0].scale(t), self.0[1].scale(t), self.0[2].scale(t)], self.1);
    }
    fn center(&self)->[f32;4]{
        return self.0[0].add(self.0[1]).add(self.0[2]).scale([1.0/3.0, 1.0/3.0, 1.0/3.0, 1.0])
    }
    fn multiply_mat(&self, m:[[f32;4];4])->Self{
        return (
            [
                self.0[0].multiply_mat(m),
                self.0[1].multiply_mat(m),
                self.0[2].multiply_mat(m)
            ],
            self.1
        );
    }
    fn upd(&self, scalar : [f32;4], trans : [f32;4], rot : [f32;4], rot_point : [f32;4], center : [f32;4])->Self{
        let mut t = *self;
        if scalar[0] != 0.0 || scalar[1] != 0.0 || scalar[2] != 0.0{
            t = t.translate(center.negative()).scale(scalar).translate(center);
        }
        if rot[0] != 0.0 || rot[1] != 0.0 || rot[2] != 0.0{
            t = t.translate(rot_point.negative());
            if rot[2] != 0.0{
                t = t.multiply_mat(Engine::z_rot(rot[2]));
            }
            if rot[1] != 0.0{
                t = t.multiply_mat(Engine::y_rot(rot[1]));
            }
            if rot[0] != 0.0{
                t = t.multiply_mat(Engine::x_rot(rot[0]));
            }
            t = t.translate(rot_point)
        }
        t = t.translate(trans);
        return t;
    }
}
fn gen_terrain(start : [f32;4], end : [f32;4], increment : f32, func : &dyn Fn(f32, f32)->f32)->Vec<[f32;4]>{
    let mut r : Vec<[f32;4]> = Vec::new();
    for i in start[0] as i32..end[0] as i32{
        for j in start[2] as i32..end[2] as i32{
            if i%increment as i32 == 0 && j%increment as i32 == 0{
                let x = increment*i as f32;
                let z = increment*j as f32;
                let y = func(x, z) + start[1];
                r.push([x, y, z, 1.0]);
            }

        }
    }
    return r;
}


trait operations4x4{
    fn multiply(self, mat : Self)->Self;
}
impl operations4x4 for [[f32;4];4]{
    fn multiply(self, mat : Self)->Self{
        return [
            [
                self[0][0]*mat[0][0] + self[0][1]*mat[1][0] + self[0][2]*mat[2][0] + self[0][3]*mat[3][0], 
                self[0][0]*mat[0][1] + self[0][1]*mat[1][1] + self[0][2]*mat[2][1] + self[0][3]*mat[3][1], 
                self[0][0]*mat[0][2] + self[0][1]*mat[1][2] + self[0][2]*mat[2][2] + self[0][3]*mat[3][2],
                self[0][0]*mat[0][3] + self[0][1]*mat[1][3] + self[0][2]*mat[2][3] + self[0][3]*mat[3][3],
            ],

            [
                self[1][0]*mat[0][0] + self[1][1]*mat[1][0] + self[1][2]*mat[2][0] + self[1][3]*mat[3][0], 
                self[1][0]*mat[0][1] + self[1][1]*mat[1][1] + self[1][2]*mat[2][1] + self[1][3]*mat[3][1], 
                self[1][0]*mat[0][2] + self[1][1]*mat[1][2] + self[1][2]*mat[2][2] + self[1][3]*mat[3][2],
                self[1][0]*mat[0][3] + self[1][1]*mat[1][3] + self[1][2]*mat[2][3] + self[1][3]*mat[3][3],
            ],

            [
                self[2][0]*mat[0][0] + self[2][1]*mat[1][0] + self[2][2]*mat[2][0] + self[2][3]*mat[3][0], 
                self[2][0]*mat[0][1] + self[2][1]*mat[1][1] + self[2][2]*mat[2][1] + self[2][3]*mat[3][1], 
                self[2][0]*mat[0][2] + self[2][1]*mat[1][2] + self[2][2]*mat[2][2] + self[2][3]*mat[3][2],
                self[2][0]*mat[0][3] + self[2][1]*mat[1][3] + self[2][2]*mat[2][3] + self[2][3]*mat[3][3],
            ],

            [
                self[3][0]*mat[0][0] + self[3][1]*mat[1][0] + self[3][2]*mat[2][0] + self[3][3]*mat[3][0], 
                self[3][0]*mat[0][1] + self[3][1]*mat[1][1] + self[3][2]*mat[2][1] + self[3][3]*mat[3][1], 
                self[3][0]*mat[0][2] + self[3][1]*mat[1][2] + self[3][2]*mat[2][2] + self[3][3]*mat[3][2],
                self[3][0]*mat[0][3] + self[3][1]*mat[1][3] + self[3][2]*mat[2][3] + self[3][3]*mat[3][3],
            ],
        ]
    }
}
//hi
#[derive(Copy, Clone)]
struct Camera{
    fov : f32,
    pos : [f32;4],
    rot : [f32;4],
    vel : [f32;4],
    rot_vel : [f32;4],
    vll : f32
}
pub struct Engine{
    camera : Camera,
    clip_distance : f32,
    render_distance : f32,
    window_height : f32,
    window_width : f32,
    objects : Vec<Object>
}

impl Engine{
    fn matrix3d(&self)->[[f32;4];4]{
        let t = ((self.camera.fov/2.0)*(std::f32::consts::PI/180.0)).tan();
        let zratio = self.render_distance/(self.render_distance-self.clip_distance);
        return [
            [-1.0/(t*self.window_width/self.window_height), 0.0, 0.0, 0.0],
            [0.0, -1.0/t, 0.0, 0.0],
            [0.0, 0.0, zratio, 1.0],
            [0.0, 0.0, -self.clip_distance*zratio, 0.0]
        ];
    }
    fn x_rot(angle : f32)->[[f32;4];4]{
        return [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, angle.cos(), angle.sin(), 0.0],
            [0.0, -angle.sin(), angle.cos(), 0.0],
            [0.0, 0.0, 0.0, 1.0]
        ];
    }
    fn y_rot(angle : f32)->[[f32;4];4]{
        return [
            [angle.cos(), 0.0, -angle.sin(), 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [angle.sin(), 0.0, angle.cos(), 0.0],
            [0.0, 0.0, 0.0, 1.0]
        ];
    }
    fn z_rot(angle : f32)->[[f32;4];4]{
        return [
            [angle.cos(), -angle.sin(), 0.0, 0.0],
            [angle.sin(), angle.cos(), 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0]
        ];
    }
    /*[
        [z.cos()*y.cos(), -z.sin(), z.cos()*-y.sin()],
        [z.sin()*y.cos(), z.cos(), z.sin()*-y.sin()],
        [0.0, 0.0, y.cos()]
    ]*/
    fn rot_zyx(x : f32, y : f32, z : f32)->[[f32;4];4]{
        return Engine::z_rot(z).multiply(Engine::y_rot(y)).multiply(Engine::x_rot(x));
    }

}
fn max(n1:f32, n2:f32)->f32{
    return if n1 > n2{n1} else{n2};
}
fn min(n1:f32, n2:f32)->f32{
    return if n1 < n2{n1} else{n2};
}


trait draw_tri{
    fn draw_triangle(&mut self, p1 : [f32;3], p2 : [f32;3], p3 : [f32;3], c : Color);
    fn fill_triangle(&mut self, p1 : [f32;3], p2 : [f32;3], p3 : [f32;3], c : Color);
    fn textured_triangle(&mut self, p1 : [f32;3], p2 : [f32;3], p3 : [f32;3], t1 : [f32;3], t2 : [f32;3], t3 : [f32;3], c: Color, texture : &Surface);
}
impl draw_tri for WindowCanvas{
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
    fn textured_triangle(&mut self, p1 : [f32;3], p2 : [f32;3], p3 : [f32;3], t1 : [f32;3], t2 : [f32;3], t3 : [f32;3], c: Color, texture : &Surface){
        let s = self.output_size().unwrap();
        let mut c1 = p1;
        let mut c2 = p2;
        let mut c3 = p3;
        let mut i1 = t1;
        let mut i2 = t2;
        let mut i3 = t3;
        let w = texture.width() as u32;
        let h = texture.height() as u32;
        
        if c1[1] > c2[1]{
            std::mem::swap(&mut c1, &mut c2);
            std::mem::swap(&mut i1, &mut i2);
        }
        
        if c1[1] > c3[1]{
            std::mem::swap(&mut c1, &mut c3);
            std::mem::swap(&mut i1, &mut i3);
        }
        
        if c2[1] > c3[1]{
            std::mem::swap(&mut c2, &mut c3);
            std::mem::swap(&mut i2, &mut i3);
        }
        
        i1[0] *= w as f32;
        i2[0] *= w as f32;
        i3[0] *= w as f32;
        
        i1[1] *= h as f32;
        i2[1] *= h as f32;
        i3[1] *= h as f32;
    
        
        let mut dax_step = 0.0; let mut dbx_step = 0.0; let mut dcx_step = 0.0;
        let mut du1_step = 0.0; let mut dv1_step = 0.0; let mut dw1_step = 0.0;
        let mut du2_step = 0.0; let mut dv2_step = 0.0; let mut dw2_step = 0.0;
        let mut du3_step = 0.0; let mut dv3_step = 0.0; let mut dw3_step = 0.0;
        
        
        let dya = (c2[1] - c1[1]).abs() as f32;
        
        if dya != 0.0{
            dax_step = (c2[0] - c1[0])/dya;
            du1_step = (i2[0] - i1[0])/dya;
            dv1_step = (i2[1] - i1[1])/dya;
            dw1_step = (i2[2] - i1[2])/dya;
        }
        
        let dyb = (c3[1] - c1[1]).abs() as f32;
        
        if dyb != 0.0{
            dbx_step = (c3[0] - c1[0])/dyb;
            du2_step = (i3[0] - i1[0])/dyb;
            dv2_step = (i3[1] - i1[1])/dyb;
            dw2_step = (i3[2] - i1[2])/dyb;
        };
        
        let dyc = (c3[1] - c2[1]).abs() as f32;
        
        if dyc != 0.0{
            dcx_step = (c3[0] - c2[0])/dyc;
            du3_step = (i3[0] - i2[0])/dyc;
            dv3_step = (i3[1] - i2[1])/dyc;
            dw3_step = (i3[2] - i2[2])/dyc;
        }

        if dya != 0.0 || dyc != 0.0{            
            for y in c1[1] as i16..c3[1] as i16{
                if y > 0 && y < s.1 as i16{
                    let mut tex_su = i1[0] + (y as f32 - c1[1]) * du1_step;
                    let mut tex_sv = i1[1] + (y as f32 - c1[1]) * dv1_step;
                    let mut tex_sw = i1[2] + (y as f32 - c1[1]) * dw1_step;
    
                    let mut tex_eu = i1[0] + (y as f32 - c1[1]) * du2_step;
                    let mut tex_ev = i1[1] + (y as f32 - c1[1]) * dv2_step;
                    let mut tex_ew = i1[2] + (y as f32 - c1[1]) * dw2_step;

                    let mut ax = c1[0] + (y as f32 - c1[1]) * dax_step;
                    let mut bx = c1[0] + (y as f32 - c1[1]) * dbx_step;
                    
                    if y as f32 > c2[1]{

                        tex_su = i2[0] + (y as f32 - c2[1]) * du3_step;
                        tex_sv = i2[1] + (y as f32 - c2[1]) * dv3_step;
                        tex_sw = i2[2] + (y as f32 - c2[1]) * dw3_step;
        
                        tex_eu = i1[0] + (y as f32 - c1[1]) * du2_step;
                        tex_ev = i1[1] + (y as f32 - c1[1]) * dv2_step;
                        tex_ew = i1[2] + (y as f32 - c1[1]) * dw2_step;
    
                        ax = c2[0] + (y as f32 - c2[1])*dcx_step;
                        bx = c1[0] + (y as f32 - c1[1])*dbx_step;
                    };
                    if ax > bx{
                        std::mem::swap(&mut ax, &mut bx);
                        std::mem::swap(&mut tex_su, &mut tex_eu);
                        std::mem::swap(&mut tex_sv, &mut tex_ev);
                        std::mem::swap(&mut tex_sw, &mut tex_ew);
                    }
                    
                    let tstep = 1.0/(bx - ax);

                    for x in ax as i16..bx as i16{
                        if x > 0 && x < s.0 as i16{
                            let t = (x as f32-ax)*tstep;
                            let tex_w = (1.0 - t) * tex_sw + t * tex_ew;
                            let tex_u = (1.0 - t) * tex_su + t * tex_eu;
                            let tex_v = (1.0 - t) * tex_sv + t * tex_ev;
                            
                            self.pixel(
                                x,
                                y, 
                                texture.color_at(tex_u/tex_w, tex_v/tex_w).blend(c)
                            );
                        }
                        
                    }
                }
            }
        }


        //if dyc != 0.0{
        //    for y in c2[1] as i16..c3[1] as i16{
        //        if y > 0 && y < s.1 as i16{
        //            let mut tex_su = i2[0] + (y as f32 - c2[1]) * du3_step;
        //            let mut tex_sv = i2[1] + (y as f32 - c2[1]) * dv3_step;
        //            let mut tex_sw = i2[2] + (y as f32 - c2[1]) * dw3_step;
    //
        //            let mut tex_eu = i1[0] + (y as f32 - c1[1]) * du2_step;
        //            let mut tex_ev = i1[1] + (y as f32 - c1[1]) * dv2_step;
        //            let mut tex_ew = i1[2] + (y as f32 - c1[1]) * dw2_step;
//
        //            let mut ax = c2[0] + (y as f32 - c2[1])*dcx_step;
        //            let mut bx = c1[0] + (y as f32 - c1[1])*dbx_step;
        //            if ax > bx{
        //                std::mem::swap(&mut ax, &mut bx);
        //                std::mem::swap(&mut tex_su, &mut tex_eu);
        //                std::mem::swap(&mut tex_sv, &mut tex_ev);
        //                std::mem::swap(&mut tex_sw, &mut tex_ew);
        //            }
//
        //            for x in ax as i16..bx as i16{
        //                if x > 0 && x < s.0 as i16{
        //                    let t = (x as f32-ax)/(bx - ax);
        //                    let tex_u = (1.0 - t) * tex_su + t * tex_eu;
        //                    let tex_v = (1.0 - t) * tex_sv + t * tex_ev;
        //                    let tex_w = (1.0 - t) * tex_sw + t * tex_ew;
        //                    self.pixel(
        //                        x,
        //                        y, 
        //                        texture.color_at(tex_u/tex_w, tex_v/tex_w).blend(c)
        //                    );
        //                }
        //                
        //            }
        //        }
        //    }
        //}
    }
}
#[derive(Clone)]
struct Object{
    tris : Vec<([[f32;4];3], [[f32;3];3])>,
    rot : [f32;4],
    vel : [f32;4],
    rot_vel : [f32;4]
}

impl Object{
    fn new(tris:Vec<([[f32;4];3], [[f32;3];3])>, rot:[f32;4], t_coords : Vec<[[f32;3];3]>)->Self{
        return Object{tris, rot, vel : [0.0, 0.0, 0.0, 0.0], rot_vel : [0.0, 0.0, 0.0, 0.0]};
    }
    #[inline]
    fn center(&self)->[f32;4]{
        let mut c = [0.0, 0.0, 0.0, 1.0];
        let n = self.tris.len() as f32;
        for tri in &self.tris{
            c = c.add(tri.center());
        }
        return c.scale([1.0/n, 1.0/n, 1.0/n, 1.0])
    }

    fn load_obj_file(file_path:String, texture:bool)->Self{
        let file = File::open(file_path).unwrap();
        let reader = BufReader::new(file);
        let mut ts : Vec<([[f32;4];3], [[f32;3];3])> = Vec::new();
        let mut points : Vec<[f32;4]> = Vec::new();
        let mut t_c : Vec<[f32;3]> = Vec::new();
        for line in reader.lines() {
            
            let ln = Box::leak(line.unwrap().into_boxed_str());
            let mut vals : Vec<&str> = ln.split_whitespace().collect();
            if vals.len() > 0{
                if vals[0].to_string() == "v".to_string() {
                    points.push(
                        [
                            vals[1].parse::<f32>().unwrap(),
                            vals[2].parse::<f32>().unwrap(),
                            vals[3].parse::<f32>().unwrap(),
                            1.0
                        ]
                    );
                } else if vals[0].to_string() == "f".to_string() {
                    if texture{
                        let p1 : Vec<&str> = vals[1].split("/").collect();
                        let p2 : Vec<&str> = vals[2].split("/").collect();
                        let p3 : Vec<&str> = vals[3].split("/").collect();
                        
                        ts.push(
                            (
                                [
                                    points[p1[0].parse::<usize>().unwrap()-1],
                                    points[p2[0].parse::<usize>().unwrap()-1],
                                    points[p3[0].parse::<usize>().unwrap()-1]
                                ],
                                [
                                    t_c[p1[1].parse::<usize>().unwrap()-1],
                                    t_c[p2[1].parse::<usize>().unwrap()-1],
                                    t_c[p3[1].parse::<usize>().unwrap()-1]
                                ]
                            )
                        );

                    } else {
                        ts.push(
                            (
                                [
                                    points[vals[1].parse::<usize>().unwrap()-1],
                                    points[vals[2].parse::<usize>().unwrap()-1],
                                    points[vals[3].parse::<usize>().unwrap()-1]
                                ],
                                [
                                    [0.0, 0.0, 0.0],
                                    [0.0, 0.0, 0.0],
                                    [0.0, 0.0, 0.0]
                                ]
                            )
                        );
                    }
                } else if vals[0].to_string() == "vt".to_string(){
                    t_c.push(
                        [
                            vals[1].parse::<f32>().unwrap(), 
                            vals[1].parse::<f32>().unwrap(), 
                            1.0
                        ]
                    );
                }
            }
        }
        return Object{tris:ts, rot:[0.0, 0.0, 0.0, 0.0], vel:[0.0, 0.0, 0.0, 0.0], rot_vel:[0.0, 0.0, 0.0, 0.0]};
    }
    fn translate(&self, t : [f32;4])->Self{
        let mut s = Vec::new();
        for i in &self.tris{
            s.push(i.translate(t));
        }
        return Object{tris:s, rot:self.rot, vel:self.vel, rot_vel:self.rot_vel};
    }
    fn scale(&self, t : [f32;4])->Self{
        let mut s = Vec::new();
        for i in &self.tris{
            s.push(i.scale(t));
        }
        return Object{tris:s, rot:self.rot, vel:self.vel, rot_vel:self.rot_vel};
    }
    fn rotate_point(&self, deg : [f32;4], point : [f32;4])->Self{
        let mut ts = Vec::new();
        for i in 0..ts.len(){
            ts.push(self.tris[i].translate(point.negative()));
            if deg[2] != 0.0{
                ts[i] = ts[i].multiply_mat(Engine::z_rot(deg[2]));
            }
            if deg[1] != 0.0{
                ts[i] = ts[i].multiply_mat(Engine::y_rot(deg[1]));
            }
            if deg[0] != 0.0{
                ts[i] = ts[i].multiply_mat(Engine::x_rot(deg[0]));
            }
            ts[i] = ts[i].translate(point);
        }
        return Object{tris:ts, rot:self.rot.add(deg), vel:self.vel, rot_vel:self.rot_vel};
    }
    #[inline]
    fn upd(&self, scalar : [f32;4], trans : [f32;4], rot : [f32;4], rot_point : [f32;4])->Self{
        let center = self.center();
        
        let ts = self.tris.iter().map(|&i|{
            return i.upd(scalar, trans, rot, rot_point, center);
        }).collect::<Vec<([[f32;4];3], [[f32;3];3])>>();
        return Object{tris:ts, rot:self.rot.add(rot), vel:self.vel, rot_vel:self.rot_vel};
    }
}
#[inline]
fn sort_objs(engine : &mut Engine){
    let r = engine.camera.rot.negative();
    let p = engine.camera.pos;
    &engine.objects.sort_by(|a, b| b.upd([1.0, 1.0, 1.0, 1.0], p.negative(), r, p).center().magnitude().partial_cmp(&a.upd([1.0, 1.0, 1.0, 1.0], p.negative(), r, p).center().magnitude()).unwrap());
}
#[inline]
fn sort_tris(mut tris : Vec<([[f32;4];3], [[f32;3];3])>)->Vec<([[f32;4];3], [[f32;3];3])>{
    &tris.sort_by(|a,b| b.center()[2].partial_cmp(&a.center()[2]).unwrap());
    return tris;
}

fn find_sdl_gl_driver() -> Option<u32> {
    for (index, item) in sdl2::render::drivers().enumerate() {
        if item.name == "opengl" {
            return Some(index as u32);
        }
    }
    None
}
fn main() {
    let mut fps_manager = FPSManager::new();

    let list_id = [0.0, 0.0, 0.0, 0.0];
    let list_id_sc = [1.0, 1.0, 1.0, 1.0];

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let sdl_image_context = image::init(image::InitFlag::all());

    let mut window = video_subsystem.window("game", 500, 500)
        .opengl()
        .fullscreen_desktop()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();

    let window_texture : sdl2::surface::Surface = image::LoadSurface::from_file(Path::new("assets/dabebe.png")).unwrap();
    window.set_icon(window_texture);

    let mut canvas : WindowCanvas = window
        .into_canvas()
        .index(find_sdl_gl_driver().unwrap())
        .build()
        .map_err(|e| e.to_string())
        .unwrap();
    
    canvas.set_scale(1.0001, 1.0001);
    canvas.window().gl_set_context_to_current();
    
    let screen_width = canvas.output_size().unwrap().0 as i32;
    let screen_height = canvas.output_size().unwrap().1 as i32;
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    let max_fps = 60_u32;
    let mut player_cam = Camera{
        fov : 90.0,
        pos : [0.0, 0.0, 0.0, 1.0],
        rot : [0.0, 0.0, 0.0, 0.0],
        vel : [0.0, 0.0, 0.0, 0.0],
        rot_vel : [0.0, 0.0, 0.0, 0.0],
        vll: 90_f32.to_radians()
    };
    let mut engine = Engine{
        camera : player_cam,
        clip_distance : 1.0,
        render_distance : 1000.0,
        window_height : screen_height as f32,
        window_width : screen_width as f32,
        objects : Vec::new()
    };
    for i in 0..1{
        engine.objects.push(Object::load_obj_file("assets/textured_cube.obj".to_string(), true).translate([i as f32*5.0, 0.0, 5.0, 0.0]));
        engine.objects[i].rot_vel = [0.0, 0_f32.to_radians(), 0.0, 1.0];
    }
    
    let dababy_texture : Surface = image::LoadSurface::from_file(Path::new("assets/dabebe.png")).unwrap();
    let cspeed = 10.0;
    let rspeed = 60.0_f32.to_radians();
    let mat3d = engine.matrix3d();
    let mut depth_buf : Vec<f32> = Vec::new();
    for i in 0..screen_width*screen_height{
        depth_buf.push(0.0);
    }
    let mut buffer = Vec::new();
    for y in 0..256 {
        for x in 0..256 {
            buffer.push(x as u8);
            buffer.push(y as u8);
            buffer.push(0);
        }
    }
    'running: loop {
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        
        let FPS = fps_manager.get_framerate() as f32;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown {keycode: Some(Keycode::W), .. } => {
                    engine.camera.vel[2] = cspeed;
                }, Event::KeyUp {keycode: Some(Keycode::W), .. } => {
                    engine.camera.vel[2] = 0.0;
                },
                
                Event::KeyDown {keycode: Some(Keycode::A), .. } => {
                    engine.camera.vel[0] = cspeed;
                }, Event::KeyUp {keycode: Some(Keycode::A), .. } => {
                    engine.camera.vel[0] = 0.0;
                },
                
                Event::KeyDown {keycode: Some(Keycode::S), .. } => {
                    engine.camera.vel[2] = -cspeed;
                }, Event::KeyUp {keycode: Some(Keycode::S), .. } => {
                    engine.camera.vel[2] = 0.0;
                },
                
                Event::KeyDown {keycode: Some(Keycode::D), .. } => {
                    engine.camera.vel[0] = -cspeed;
                }, Event::KeyUp {keycode: Some(Keycode::D), .. } => {
                    engine.camera.vel[0] = 0.0;
                },
                
                Event::KeyDown {keycode: Some(Keycode::E), .. } => {
                    engine.camera.vel[1] = cspeed;
                }, Event::KeyUp {keycode: Some(Keycode::E), .. } => {
                    engine.camera.vel[1] = 0.0;
                },
                
                Event::KeyDown {keycode: Some(Keycode::Q), .. } => {
                    engine.camera.vel[1] = -cspeed;
                }, Event::KeyUp {keycode: Some(Keycode::Q), .. } => {
                    engine.camera.vel[1] = 0.0;
                },

                
                
                //--------------ROTATE--------------
                Event::KeyDown {keycode: Some(Keycode::Up), .. } => {
                    if engine.camera.rot[0] > (-engine.camera.vll)%360_f32.to_radians(){
                        engine.camera.rot_vel[0] = -rspeed;
                    } else {
                        engine.camera.rot_vel[0] = 0.0;
                    }
                }, Event::KeyUp {keycode: Some(Keycode::Up), .. } => {
                    engine.camera.rot_vel[0] = 0.0;
                },
                
                Event::KeyDown {keycode: Some(Keycode::Down), .. } => {
                    if engine.camera.rot[0] < (engine.camera.vll)%360_f32.to_radians(){
                        engine.camera.rot_vel[0] = rspeed;
                    } else {
                        engine.camera.rot_vel[0] = 0.0;
                    }
                }, Event::KeyUp {keycode: Some(Keycode::Down), .. } => {
                    engine.camera.rot_vel[0] = 0.0;
                },
                
                Event::KeyDown {keycode: Some(Keycode::Left), .. } => {
                    engine.camera.rot_vel[1] = rspeed;
                }, Event::KeyUp {keycode: Some(Keycode::Left), .. } => {
                    engine.camera.rot_vel[1] = 0.0;
                },
                
                Event::KeyDown {keycode: Some(Keycode::Right), .. } => {
                    engine.camera.rot_vel[1] = -rspeed;
                }, Event::KeyUp {keycode: Some(Keycode::Right), .. } => {
                    engine.camera.rot_vel[1] = 0.0;
                },
                

            


                //Event::MouseButtonDown{mouse_btn : MouseButton::Right, x, y,..} => {
                //    engine.camera.rot_vel = engine.camera.rot_vel.add([(y as f32).asin(), (x as f32).asin(), 0.0, 0.0]);
                //},

                _ => {}
            }
        }
        //The rest of the game loop goes here...
        //ok
        //sort_objs(&mut engine);

        let cam = engine.camera;
        engine.camera.rot = cam.rot.add(cam.rot_vel.scale([1.0/FPS, 1.0/FPS, 1.0/FPS, 1.0]));
        let travis : Surface = image::LoadSurface::from_file(Path::new("assets/dabebe.png")).unwrap();
        engine.camera.pos = cam.pos.add(
            cam.vel.multiply_mat([
                [engine.camera.rot[1].cos(), engine.camera.rot[2].sin(), -engine.camera.rot[1].sin(), 0.0],
                [engine.camera.rot[2].sin(), engine.camera.rot[0].cos(), engine.camera.rot[0].sin(), 0.0],
                [engine.camera.rot[1].sin(), -engine.camera.rot[0].sin(), engine.camera.rot[1].cos(), 0.0],
                [0.0, 0.0, 0.0, 1.0]
            ]).scale([1.0/FPS, 1.0/FPS, 1.0/FPS, 1.0])
        );
        let light = [0.0, 0.0, -1.0, 1.0].normalize();
        let ew = engine.window_width/2.0; let eh = engine.window_height/2.0;
        for i in 0..engine.objects.len(){
            
            let center = engine.objects[i].center();
            
            engine.objects[i] = engine.objects[i].upd(list_id_sc, engine.objects[i].vel.scale([1.0/FPS, 1.0/FPS, 1.0/FPS, 1.0]), engine.objects[i].rot_vel.scale([1.0/FPS, 1.0/FPS, 1.0/FPS, 1.0]), center);
            let obj = engine.objects[i].upd(list_id_sc, cam.pos.negative(), cam.rot.negative(), cam.pos);

            for mut tri in obj.tris{
                //engine.objects[i].tris[n] = engine.objects[i].tris[n].upd(list_id_sc, engine.objects[i].vel.scale([1.0/FPS, 1.0/FPS, 1.0/FPS, 1.0]), engine.objects[i].rot_vel.scale([1.0/FPS, 1.0/FPS, 1.0/FPS, 1.0]), center, center);
                //obj.tris[n].upd(list_id_sc, cam.pos.negative(), cam.rot.negative(), cam.pos, cam.pos);
                let normal = tri.normal();
                let c = tri.center();
                if normal.dot_product(tri.0[0]) < 0.0 && c[2] > engine.clip_distance && c[2] < engine.render_distance{
                    let dp = normal.dot_product(light);
                    if dp > 0.0{
                        let t = tri.scale([engine.window_width/2.0, engine.window_height/2.0, 1.0, 1.0]).multiply_mat(mat3d);
                        
                        let t03 = t.0[0][3]; let t13 = t.0[1][3]; let t23 = t.0[2][3]; let c = (255.0*dp) as u8;
                        let o = [(t.0[0][0]/t03+ew), (t.0[0][1]/t03+eh), t.0[0][2]];    
                        let g = [(t.0[1][0]/t13+ew), (t.0[1][1]/t13+eh), t.0[1][2]];
                        let h = [(t.0[2][0]/t23+ew), (t.0[2][1]/t23+eh), t.0[2][2]];
                        //canvas.fill_triangle(
                        //    o,     
                        //    g,
                        //    h,
                        //    Color::from((c, c, c))
                        //);

                        
                        tri.1[0][1] /= t03;
                        tri.1[1][1] /= t13;
                        tri.1[2][1] /= t23;
                        
                        tri.1[0][0] /= t03;
                        tri.1[1][0] /= t13;
                        tri.1[2][0] /= t23;

                        tri.1[0][2] = 1.0/t03;
                        tri.1[1][2] = 1.0/t13;
                        tri.1[2][2] = 1.0/t23;
                        canvas.textured_triangle(
                            o,
                            g,
                            h,
                            tri.1[0],
                            tri.1[1],
                            tri.1[2],
                            Color::from((c, c, c)),
                            &travis
                        );
                        //canvas.draw_triangle(
                        //    o,     
                        //    g,
                        //    h,
                        //    Color::WHITE
                        //);
                                
                    }
                }
            }
        }


        fps_manager.set_framerate(max_fps);
        let del = fps_manager.delay();
        fps_manager.set_framerate(1000/(del+1));

        canvas.string(
            5,
            5,
            &format!("FPS: {}", fps_manager.get_framerate()).to_string(),
            Color::WHITE
        );
        canvas.present();
        //for i in 0..screen_width*screen_height{
        //    depth_buf[i as usize] = 0.0;
        //}
    }
}
