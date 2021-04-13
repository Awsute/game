//when shipping game, make sure you got everything in the folder with the game
extern crate sdl2;
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
use std::time::Duration;
use std::any::{Any, TypeId};
use std::path::Path;
use std::borrow::Borrow;
use sdl2::gfx::primitives::DrawRenderer;
mod impl3d;
use impl3d::Vec3;
use impl3d::Tri3d;
trait operations3x3{
    fn multiply(self, mat : Self)->Self;
}
impl operations3x3 for [[f32;3];3]{
    fn multiply(self, mat : [[f32;3];3])->[[f32;3];3]{
        return [
            [self[0][0]*mat[0][0] + self[0][1]*mat[1][0] + self[0][2]*mat[2][0], self[0][0]*mat[0][1] + self[0][1]*mat[1][1] + self[0][2]*mat[2][1], self[0][0]*mat[0][2] + self[0][1]*mat[1][2] + self[0][2]*mat[2][2]],
            [self[1][0]*mat[0][0] + self[1][1]*mat[1][0] + self[1][2]*mat[2][0], self[1][0]*mat[0][1] + self[1][1]*mat[1][1] + self[1][2]*mat[2][1], self[1][0]*mat[0][2] + self[1][1]*mat[1][2] + self[1][2]*mat[2][2]],
            [self[2][0]*mat[0][0] + self[2][1]*mat[1][0] + self[2][2]*mat[2][0], self[2][0]*mat[0][1] + self[2][1]*mat[1][1] + self[2][2]*mat[2][1], self[2][0]*mat[0][2] + self[2][1]*mat[1][2] + self[2][2]*mat[2][2]],
        ]
    }
}
//hi
#[derive(Copy, Clone)]
struct Camera{
    fov : f32,
    pos : [f32;3],
    rot : [f32;3],
    vel : [f32;3],
    rot_vel : [f32;3],
    vll : f32
}
struct Engine{
    camera : Camera,
    clip_distance : f32,
    render_distance : f32,
    window_height : f32,
    window_width : f32,
    objects : Vec<Object>
}

impl Engine{
    fn matrix(&self)->[[f32;3];3]{
        let t = (self.camera.fov*180.0/(2.0*std::f32::consts::PI)).tan();
        let zr = self.render_distance/(self.render_distance-self.clip_distance);
        return [
            [(self.window_height/self.window_width)/t, 0.0, 0.0],
            [0.0, 1.0/t, 0.0],
            [0.0, 0.0, -self.clip_distance*zr]
        ];
    }
    fn x_rot(angle : f32)->[[f32;3];3]{
        return [
            [1.0, 0.0, 0.0],
            [0.0, angle.cos(), angle.sin()],
            [0.0, -angle.sin(), angle.cos()]
        ];
    }
    fn y_rot(angle : f32)->[[f32;3];3]{
        return [
            [angle.cos(), 0.0, -angle.sin()],
            [0.0, 1.0, 0.0],
            [angle.sin(), 0.0, angle.cos()]
        ];
    }
    fn z_rot(angle : f32)->[[f32;3];3]{
        return [
            [angle.cos(), -angle.sin(), 0.0],
            [angle.sin(), angle.cos(), 0.0],
            [0.0, 0.0, 1.0]
        ];
    }
    /*[
        [z.cos()*y.cos(), -z.sin(), z.cos()*-y.sin()],
        [z.sin()*y.cos(), z.cos(), z.sin()*-y.sin()],
        [0.0, 0.0, y.cos()]
    ]*/
    fn rot_zyx(x : f32, y : f32, z : f32)->[[f32;3];3]{
        return Engine::z_rot(z).multiply(Engine::y_rot(y)).multiply(Engine::x_rot(x));
    }

}


trait draw_tri{
    fn draw_triangle(&mut self, p1:[i16;2], p2:[i16;2], p3:[i16;2], c : Color);
}
impl draw_tri for WindowCanvas{
    fn draw_triangle(&mut self, p1 : [i16;2], p2 : [i16;2], p3 : [i16;2], c : Color){
        let result = self.filled_polygon(
            &[p1[0], p2[0], p3[0]], 
            &[p1[1], p2[1], p3[1]], 
            c
        );
    }
}
#[derive(Clone)]
struct Object{
    tris : Vec<[[f32;3];3]>,
    rot : [f32;3],
    vel : [f32;3],
    rot_vel : [f32;3]
}

impl Object{
    fn new(mut tris:Vec<[[f32;3];3]>, rot:[f32;3])->Self{
        return Object{tris, rot, vel : [0.0, 0.0, 0.0], rot_vel : [0.0, 0.0, 0.0]};
    }
    fn center(&self)->[f32;3]{
        let mut c = [0.0, 0.0, 0.0];
        let n = self.tris.len() as f32;
        for tri in &self.tris{
            c = c.add(tri.center());
        }
        return c.scale([1.0/n, 1.0/n, 1.0/n])
    }
    fn sort_tris(&self)->Vec<[[f32;3];3]>{
        let mut ts = self.tris.clone();
        for i in 0..ts.len(){
            
            for o in 0..ts.len(){
                if i != o{
                    let t1 = ts[i];
                    let t2 = ts[o];
                    //sort
                    if t1.center().magnitude() > t2.center().magnitude(){
                        ts[o] = t1;
                        ts[i] = t2;
                    }
                }
            }
        }
        return ts;
    }
    fn load_obj_file(file_path:String)->Self{
        let file = File::open(file_path).unwrap();
        let reader = BufReader::new(file);
        let mut ts : Vec<[[f32;3];3]> = Vec::new();
        let mut points : Vec<[f32;3]> = Vec::new();
        
        for line in reader.lines() {
            let ln = Box::leak(line.unwrap().into_boxed_str());
            let vals : Vec<&str> = ln.split_whitespace().collect();
            if vals.len() > 0{
                if vals[0].to_string() == "v".to_string() {
                    points.push(
                        [
                            vals[1].parse::<f32>().unwrap(),
                            vals[2].parse::<f32>().unwrap(),
                            vals[3].parse::<f32>().unwrap(),
                        ]
                    );
                } else if vals[0].to_string() == "f".to_string() {
                    ts.push(
                        [
                            points[vals[1].parse::<usize>().unwrap()-1],
                            points[vals[2].parse::<usize>().unwrap()-1],
                            points[vals[3].parse::<usize>().unwrap()-1]
                        ]
                    );
                }
            }
        }
        return Object{tris:ts, rot:[0.0, 0.0, 0.0], vel:[0.0, 0.0, 0.0], rot_vel:[0.0, 0.0, 0.0]};
    }
    fn translate(&self, t : [f32;3])->Self{
        let mut s = self.tris.clone();
        for i in 0..s.len(){
            s[i] = s[i].translate(t);
        }
        return Object{tris:s, rot:self.rot, vel:self.vel, rot_vel:self.rot_vel};
    }
    fn translatem(mut self, t : [f32;3])->Self{
        for i in 0..self.tris.len(){
            self.tris[i] = self.tris[i].translate(t);
        }
        return self;
    }
    fn scale(&self, t : [f32;3])->Self{
        let mut s = self.tris.clone();
        for i in 0..s.len(){
            s[i] = s[i].scale(t);
        }
        return Object{tris:s, rot:self.rot, vel:self.vel, rot_vel:self.rot_vel};
    }
    fn scalem(mut self, t : [f32;3])->Self{
        for i in 0..self.tris.len(){
            self.tris[i] = self.tris[i].scale(t);
        }
        return self;
    }
    fn rotate_point(&self, deg : [f32;3], point : [f32;3])->Self{
        let mut ts = self.tris.clone();
        for i in 0..ts.len(){
            ts[i] = ts[i].translate(point.negative());
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
    fn rotate_origin(&self, deg : [f32;3])->Self{
        return self.rotate_point(deg, [0.0, 0.0, 0.0]);
    }
    fn rotate_self(&self, deg : [f32;3])->Self{
        return self.rotate_point(deg, self.center());
    }
}


pub const FPS: f32 = 60.0;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let sdl_image_context = image::init(image::InitFlag::all());

    let mut window = video_subsystem.window("game", 0, 0)
        .fullscreen_desktop()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();
    //let window_texture : sdl2::surface::Surface = image::LoadSurface::from_file(Path::new("travisScot.png")).unwrap();
    //window.set_icon(window_texture);
    
    let mut canvas : WindowCanvas = window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();
    let screen_width = canvas.output_size().unwrap().0 as i32;
    let screen_height = canvas.output_size().unwrap().1 as i32;
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut obj = Object::load_obj_file("assets/character.obj".to_string()).scale([25.0, 25.0, 25.0]).translate([0.0, 0.0, 15.0]);
    obj.tris = obj.sort_tris();
    let mut objs : Vec<Object> = Vec::new();
    objs.push(obj);
    
    let mut player_cam = Camera{
        fov : 75.0,
        pos : [0.0, 0.0, 0.0],
        rot : [0.0, 0.0, 0.0],
        vel : [0.0, 0.0, 0.0],
        rot_vel : [0.0, 0.0, 0.0],
        vll: 90_f32.to_radians()
    };
    let mut engine = Engine{
        camera : player_cam,
        clip_distance : 0.1,
        render_distance : 1000.0,
        window_height : canvas.output_size().unwrap().1 as f32,
        window_width : canvas.output_size().unwrap().0 as f32,
        objects : objs

    };
    let cspeed = 10.0;
    let rspeed = 60.0_f32.to_radians();
    let mat3d = engine.matrix();
    'running: loop {
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        
        
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
                

            


                Event::MouseButtonDown{mouse_btn : MouseButton::Left, ..} => {

                },

                _ => {}
            }
        }
        // The rest of the game loop goes here...
        // ok

        let cam = engine.camera;
        engine.camera.pos = cam.pos.add(cam.vel.scale([1.0/FPS, 1.0/FPS, 1.0/FPS]));
        engine.camera.rot = cam.rot.add(cam.rot_vel.scale([1.0/FPS, 1.0/FPS, 1.0/FPS]));
        
        for i in 0..engine.objects.len(){
            //Move from camera
            if cam.rot_vel[0] != 0.0 || cam.rot_vel[1] != 0.0 || cam.rot_vel[2] != 0.0{
                engine.objects[i] = engine.objects[i].rotate_origin(cam.rot_vel.scale([1.0/FPS, 1.0/FPS, 1.0/FPS]).negative());
            }
            if cam.vel[0] != 0.0 || cam.vel[1] != 0.0 || cam.vel[2] != 0.0{
                engine.objects[i] = engine.objects[i].translate(cam.vel.scale([1.0/FPS, 1.0/FPS, 1.0/FPS]).negative());
            }

            //Now from the object
            if engine.objects[i].rot_vel[0] != 0.0 || engine.objects[i].rot_vel[1] != 0.0 || engine.objects[i].rot_vel[2] != 0.0{
                engine.objects[i] = engine.objects[i].rotate_self(engine.objects[i].rot_vel.scale([1.0/FPS, 1.0/FPS, 1.0/FPS]));
            }
            if engine.objects[i].vel[0] != 0.0 || engine.objects[i].vel[1] != 0.0 || engine.objects[i].vel[2] != 0.0{
                engine.objects[i] = engine.objects[i].translate(engine.objects[i].vel.scale([1.0/FPS, 1.0/FPS, 1.0/FPS]));
            }
            
            
            let mc = |i : f32| -> bool {i > engine.clip_distance || i < engine.render_distance};
            let light = [0.0, 0.0, -1.0].normalize();
            for tri in &engine.objects[i].tris{
                let normal = tri.normal();
                
                if normal.dot_product(tri[0]) < 0.0{
                    let mut j : [[f32;3];3] = [[0.0,0.0,0.0],[0.0,0.0,0.0],[0.0,0.0,0.0]];
                    let tri = tri.scale([engine.window_width/2.0, engine.window_height/2.0, 1.0]);
                    for i in 0..3{
                        let t = tri[i].multiply_mat(mat3d);
                        
                        j[i][0] = t[0];
                        j[i][1] = t[1];
                        j[i][2] = tri[i][2];
                    }
                    if mc(j[0][2])&&mc(j[1][2])&&mc(j[2][2]){
                        let dp = normal.dot_product(light);
                        if dp >= 0.0{
                            
                            canvas.draw_triangle(
                                [(j[0][0]/j[0][2]+engine.window_width/2.0) as i16, (j[0][1]/j[0][2]+engine.window_height/2.0) as i16],     
                                [(j[1][0]/j[1][2]+engine.window_width/2.0) as i16, (j[1][1]/j[1][2]+engine.window_height/2.0) as i16],
                                [(j[2][0]/j[2][2]+engine.window_width/2.0) as i16, (j[2][1]/j[2][2]+engine.window_height/2.0) as i16],
                                Color::from(((255.0*dp) as u8, (255.0*dp) as u8, (255.0*dp) as u8))
                            );
                        }
                    }
                    
                }
            }   
        }
        canvas.present();
        std::thread::sleep(std::time::Duration::from_millis((1000.0/FPS) as u64));
    }
}
