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

mod world;
use world::{Engine, Mesh, Camera};

pub trait Vec3{
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

pub trait Tri3d{
    fn normal(&self)->[f32;4];
    fn translate(&self, t:[f32;4])->Self;
    fn scale(&self, t:[f32;4])->Self;
    fn center(&self)->[f32;4];
    fn multiply_mat(&self, m:[[f32;4];4])->Self;
    fn upd(&self, scalar : [f32;4], trans : [f32;4], rot : [f32;4], rot_point : [f32;4], center : [f32;4])->Self;
}
impl Tri3d for ([[f32;4];3], [[f32;3];3], [[f32;4];3]){
    fn normal(&self)->[f32;4]{
        return self.0[2].subtract(self.0[0]).cross_product(self.0[1].subtract(self.0[0])).normalize();//sheeeesh
    }
    fn translate(&self, t : [f32;4])->Self{
        return ([self.0[0].add(t), self.0[1].add(t), self.0[2].add(t)], self.1, self.2);
    }
    fn scale(&self, t : [f32;4])->Self{
        return ([self.0[0].scale(t), self.0[1].scale(t), self.0[2].scale(t)], self.1, self.2);
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
            self.1,
            [
                self.2[0].multiply_mat(m),
                self.2[1].multiply_mat(m),
                self.2[2].multiply_mat(m)
            ],
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

trait Surf{
    fn color_at(&self, x:f32, y:f32)->Color;
    fn apply_fn(&mut self, f:&dyn Fn(u32, u32, u32, u32, u32, Color)->Color);
}
impl Surf for Surface<'_>{
    fn color_at(&self, x: f32, y: f32)->Color{
        let buf = self.without_lock().unwrap();
        let u = (x+0.5) as usize;
        let v = (y+0.5) as usize;
        let ind = 3*u+self.pitch() as usize*v;
        return if ind < buf.len()-2{Color::from((buf[ind], buf[ind+1], buf[ind+2]))} else {Color::BLACK};
    }
    fn apply_fn(&mut self, f:&dyn Fn(u32, u32, u32, u32, u32, Color)->Color){
        let w = self.width();
        let h = self.height();
        let p = self.pitch();
        let colb = self.without_lock_mut().unwrap();
        for x in 0..w{
            for y in 0..h{
                let i = (x*(p/w) + y*p) as usize;
                let color = f(x, y, w, h, p, Color::from((colb[i], colb[i+1], colb[i+2])));
                colb[i] = color.r;
                colb[i+1] = color.g;
                colb[i+2] = color.b;
            }
        }
    }

}
trait ColFuncs{
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

fn gen_terrain(start : [f32;4], end : [f32;4], spacing : [f32;2], func : &dyn Fn(f32, f32)->f32)->Vec<[f32;4]>{
    let mut r : Vec<[f32;4]> = Vec::new();
    for i in start[0] as i32..end[0] as i32{
        for j in start[2] as i32..end[2] as i32{
            if i%spacing[0] as i32 == 0 && j%spacing[1] as i32 == 0{
                let x = i as f32;
                let z = j as f32;
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

fn max(n1:f32, n2:f32)->f32{
    return if n1 > n2{n1} else{n2};
}
fn min(n1:f32, n2:f32)->f32{
    return if n1 < n2{n1} else{n2};
}


trait DrawTri{
    fn draw_triangle(&mut self, p1 : [f32;3], p2 : [f32;3], p3 : [f32;3], c : Color);
    fn fill_triangle(&mut self, p1 : [f32;3], p2 : [f32;3], p3 : [f32;3], c : Color);
    fn textured_triangle(&mut self, p1 : [f32;3], p2 : [f32;3], p3 : [f32;3], t1 : [f32;3], t2 : [f32;3], t3 : [f32;3], c: Color, buffer : &[u8], pitch : usize, width : f32, height : f32, engine : &mut Engine, light_info : [f32;3], light_color : Color);
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
    fn textured_triangle(&mut self, p1 : [f32;3], p2 : [f32;3], p3 : [f32;3], t1 : [f32;3], t2 : [f32;3], t3 : [f32;3], c: Color, buffer : &[u8], pitch : usize, width : f32, height : f32, engine : &mut Engine, light_info : [f32;3], light_color : Color){
        let s = (engine.window_width, engine.window_height);
        let mut c1 = p1; //screen space
        let mut c2 = p2; //screen space
        let mut c3 = p3; //screen space
        let mut i1 = t1; //texel space
        let mut i2 = t2; //texel space
        let mut i3 = t3; //texel space
        let mut l1 = light_info[0];
        let mut l2 = light_info[1];
        let mut l3 = light_info[2];
        if c1[1] > c2[1]{
            std::mem::swap(&mut c1, &mut c2);
            std::mem::swap(&mut i1, &mut i2);
            std::mem::swap(&mut l1, &mut l2);
        }
        
        if c1[1] > c3[1]{
            std::mem::swap(&mut c1, &mut c3);
            std::mem::swap(&mut i1, &mut i3);
            std::mem::swap(&mut l1, &mut l3);
        }
        
        if c2[1] > c3[1]{
            std::mem::swap(&mut c2, &mut c3);
            std::mem::swap(&mut i2, &mut i3);
            std::mem::swap(&mut l2, &mut l3);
        }
        

        let mut dax_step = 0.0; let mut dbx_step = 0.0; let mut dcx_step = 0.0;
        let mut du1_step = 0.0; let mut dv1_step = 0.0; let mut dw1_step = 0.0;
        let mut du2_step = 0.0; let mut dv2_step = 0.0; let mut dw2_step = 0.0;
        let mut du3_step = 0.0; let mut dv3_step = 0.0; let mut dw3_step = 0.0;
        
        let mut la_step = 0.0;
        let mut lb_step = 0.0;
        let mut lc_step = 0.0;
        
        
        
        let dya = (c2[1] - c1[1]).abs() as f32;
        let dyb = (c3[1] - c1[1]).abs() as f32;
        let dyc = (c3[1] - c2[1]).abs() as f32;
        
        if dya != 0.0{ //point a to point b
            dax_step = (c2[0] - c1[0])/dya;
            du1_step = (i2[0] - i1[0])/dya;
            dv1_step = (i2[1] - i1[1])/dya;
            dw1_step = (i2[2] - i1[2])/dya;

            la_step = (l2-l1)/dya;
        }
        
        if dyb != 0.0{ //point a to point c
            dbx_step = (c3[0] - c1[0])/dyb;
            du2_step = (i3[0] - i1[0])/dyb; 
            dv2_step = (i3[1] - i1[1])/dyb;
            dw2_step = (i3[2] - i1[2])/dyb;

            lb_step = (l3-l1)/dyb;

        };
        
        
        if dyc != 0.0{ //point b to point c
            dcx_step = (c3[0] - c2[0])/dyc;
            du3_step = (i3[0] - i2[0])/dyc;
            dv3_step = (i3[1] - i2[1])/dyc;
            dw3_step = (i3[2] - i2[2])/dyc;

            lc_step = (l3-l2)/dyc;

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

                    let mut ls : f32;
                    let mut le : f32;
                    if y < c2[1] as i32+1 {
                        let ys = y as f32-c1[1];
                        tex_su = i1[0] + (ys) * du1_step;
                        tex_sv = i1[1] + (ys) * dv1_step;
                        tex_sw = i1[2] + (ys) * dw1_step;
                        
                        tex_eu = i1[0] + (ys) * du2_step;
                        tex_ev = i1[1] + (ys) * dv2_step;
                        tex_ew = i1[2] + (ys) * dw2_step;
                        
                        ax = (c1[0] + (ys) * dax_step + 0.5) as i32;
                        bx = (c1[0] + (ys) * dbx_step + 0.5) as i32;

                        ls = l1 + (ys)*la_step;
                        le = l1 + (ys)*lb_step;


                    } else {
                        let ys1 = y as f32-c1[1];
                        let ys2 = y as f32-c2[1];
                        tex_su = i2[0] + (ys2) * du3_step;
                        tex_sv = i2[1] + (ys2) * dv3_step;
                        tex_sw = i2[2] + (ys2) * dw3_step;
                        
                        tex_eu = i1[0] + (ys1) * du2_step;
                        tex_ev = i1[1] + (ys1) * dv2_step;
                        tex_ew = i1[2] + (ys1) * dw2_step;
                        
                        ax = (c2[0] + (ys2) * dcx_step + 0.5) as i32;
                        bx = (c1[0] + (ys1) * dbx_step + 0.5) as i32;
                        
                        ls = l2 + (ys2)*lc_step;
                        le = l1 + (ys1)*lb_step;
                    }
                    if ax > bx{
                        std::mem::swap(&mut ax, &mut bx);
                        std::mem::swap(&mut tex_su, &mut tex_eu);
                        std::mem::swap(&mut tex_sv, &mut tex_ev);
                        std::mem::swap(&mut tex_sw, &mut tex_ew);
                        std::mem::swap(&mut ls, &mut le);
                    }
                    let tstep = 1.0/(bx - ax) as f32;
                    for x in ax..bx{
                        if x > 0 && x < s.0 as i32{

                            
                            
                            let t = (x as f32-ax as f32)*tstep;
                            let tex_w = (1.0 - t) * tex_sw + t * tex_ew;
                            let dbi = (x+s.0 as i32*y) as usize;

                            if tex_w > engine.depth_buffer[dbi]{
                                engine.depth_buffer[dbi] = tex_w;
                                
                                let dp = (1.0-t)*ls+t*le;
                                let ind = 
                                    (pitch/width as usize) * ((width-0.1) * ((1.0 - t) * tex_su + t * tex_eu)/tex_w) as usize +
                                    pitch * ((height-0.1) * ((1.0 - t) * tex_sv + t * tex_ev)/tex_w) as usize;
                                let col = if ind < buffer.len()-2{
                                    Color::from((buffer[ind], buffer[ind+1], buffer[ind+2])).blend(
                                        Color::from(((dp*255.0) as u8, (dp*255.0) as u8, (dp*255.0) as u8)).blend(light_color)
                                    )
                                } else {
                                    Color::WHITE
                                };

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
}


#[inline]
fn sort_objs(engine : &mut Engine){
    let r = engine.camera.rot.negative();
    let p = engine.camera.pos;
    &engine.objects.sort_by(|a, b| b.upd([1.0, 1.0, 1.0, 1.0], p.negative(), r, p).center().magnitude().partial_cmp(&a.upd([1.0, 1.0, 1.0, 1.0], p.negative(), r, p).center().magnitude()).unwrap());
}
#[inline]
fn sort_tris(mut tris : Vec<([[f32;4];3], [[f32;3];3], [[f32;4];3])>)->Vec<([[f32;4];3], [[f32;3];3], [[f32;4];3])>{
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
    let _audio_subsystem = sdl_context.audio().unwrap();
    let _sdl_image_context = image::init(image::InitFlag::all());

    let mut window = video_subsystem.window("game", 750, 750)
        .opengl()
        //.fullscreen_desktop()
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
    let _texture_creator = canvas.texture_creator();
    let max_fps = 60_u32;
    let player_cam = Camera{
        fov : 90.0,
        pos : [0.0, 0.0, 0.0, 1.0],
        rot : [0.0, 0.0, 0.0, 0.0],
        vel : [0.0, 0.0, 0.0, 0.0],
        rot_vel : [0.0, 0.0, 0.0, 0.0],
        vll: 90_f32.to_radians()
    };
    let mut texture_draw : Surface = image::LoadSurface::from_file(Path::new("assets/dabebe.png")).unwrap();

    texture_draw.apply_fn(&|x, y, w, h, p, c|->Color{
        return Color::WHITE;
    });


    
    let mut engine = Engine{
        camera : player_cam,
        clip_distance : 1.0,
        render_distance : 1000.0,
        window_height : screen_height as f32,
        window_width : screen_width as f32,
        objects : Vec::new(),
        depth_buffer : Vec::new()
    };
    
    engine.objects.push(Mesh::load_obj_file("assets/normalized_teapot.obj".to_string()).translate([0.0, 0.0, 5.0, 0.0]));    
    //engine.objects[0].rot_vel = [0.0, 90_f32.to_radians(), 0.0, 1.0];
    

    let cspeed = 10.0;
    let rspeed = 60.0_f32.to_radians();
    let mat3d = engine.matrix3d();
    for i in 0..screen_width*screen_height{
        engine.depth_buffer.push(0.0);
    }
    let mut frames = 0_f32;
    let mut seconds_passed = 0_f32;
    'running: loop {
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        frames += 1.0;
        let FPS = fps_manager.get_framerate() as f32;
        seconds_passed += 1.0/FPS;
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
        engine.camera.pos = cam.pos.add(
            cam.vel.multiply_mat([
                [engine.camera.rot[1].cos(), engine.camera.rot[2].sin(), -engine.camera.rot[1].sin(), 0.0],
                [engine.camera.rot[2].sin(), engine.camera.rot[0].cos(), engine.camera.rot[0].sin(), 0.0],
                [engine.camera.rot[1].sin(), -engine.camera.rot[0].sin(), engine.camera.rot[1].cos(), 0.0],
                [0.0, 0.0, 0.0, 1.0]
            ]).scale([1.0/FPS, 1.0/FPS, 1.0/FPS, 1.0])
        );
        let light = [0.0, 0.0, -1.0, 1.0].normalize();
        let light_color = Color::WHITE;
        let ew = engine.window_width/2.0; let eh = engine.window_height/2.0;
        for i in 0..engine.objects.len(){

            let center = engine.objects[i].center();
            
            engine.objects[i] = engine.objects[i].upd(list_id_sc, engine.objects[i].vel.scale([1.0/FPS, 1.0/FPS, 1.0/FPS, 1.0]), engine.objects[i].rot_vel.scale([1.0/FPS, 1.0/FPS, 1.0/FPS, 1.0]), center);
            let obj = engine.objects[i].upd(list_id_sc, cam.pos.negative(), cam.rot.negative(), cam.pos);

            for j  in 0..obj.tris.len(){
                let mut tri = obj.tris[j];
                //engine.objects[i].tris[n] = engine.objects[i].tris[n].upd(list_id_sc, engine.objects[i].vel.scale([1.0/FPS, 1.0/FPS, 1.0/FPS, 1.0]), engine.objects[i].rot_vel.scale([1.0/FPS, 1.0/FPS, 1.0/FPS, 1.0]), center, center);
                //let mut tri = triangle.upd(list_id_sc, cam.pos.negative(), cam.rot.negative(), cam.pos, cam.pos);
                let normal = tri.normal();
                let c = tri.center();
                if normal.dot_product(tri.0[0]) <= 0.0 && c[2] > engine.clip_distance && c[2] < engine.render_distance{
                    let t = tri.scale([engine.window_width/2.0, engine.window_height/2.0, 1.0, 1.0]).multiply_mat(mat3d);
                    let dp  = normal.dot_product(light);
                    let t03 = t.0[0][3]; let t13 = t.0[1][3]; let t23 = t.0[2][3]; let darkness = (255.0*dp) as u8;
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
                    let etri = engine.objects[i].tris[j];
                    canvas.textured_triangle(
                        o,
                        g,
                        h,
                        tri.1[0],
                        tri.1[1],
                        tri.1[2],
                        Color::from((darkness, darkness, darkness)).avg(Color::from((216, 216, 216))),
                        texture_draw.without_lock().unwrap(),
                        texture_draw.pitch() as usize,
                        texture_draw.width() as f32,
                        texture_draw.height() as f32,
                        &mut engine,
                        [etri.2[0].dot_product(light), etri.2[1].dot_product(light), etri.2[2].dot_product(light)],
                        light_color
                    );

                    //canvas.draw_triangle(
                    //    o,     
                    //    g,
                    //    h,
                    //    Color::GREY
                    //);
                            
                }
                
            }
        }


        fps_manager.set_framerate(max_fps);
        let del = fps_manager.delay();
        fps_manager.set_framerate(1000/del);

        canvas.string(
            5,
            5,
            &format!("FPS: {}", fps_manager.get_framerate()).to_string(),
            Color::WHITE
        );
        canvas.present();
        for i in 0..screen_width*screen_height{
            engine.depth_buffer[i as usize] = 0.0;
        }
    }
}
