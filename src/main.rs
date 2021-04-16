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
use sdl2::gfx::framerate::FPSManager;
mod impl3d;
use impl3d::Vec3;
use impl3d::Tri3d;


fn gen_terrain(start : [f32;4], end : [f32;4], increment : f32, func : &dyn Fn(f32, f32)->f32)->Vec<[f32;4]>{
    let mut r : Vec<[f32;4]> = Vec::new();
    for i in start[0] as i32..end[0] as i32{
        for j in start[2] as i32..end[2] as i32{
            if i%increment as i32 == 0 && j%increment as i32 == 0{
                let x = start[0]+i as f32;
                let z = start[2]+j as f32;
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
struct Engine{
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



trait draw_tri{
    fn draw_triangle(&mut self, p1:[i16;2], p2:[i16;2], p3:[i16;2], c : Color);
    fn fill_triangle(&mut self, p1:[i16;2], p2:[i16;2], p3:[i16;2], c : Color);
}
impl draw_tri for WindowCanvas{
    fn draw_triangle(&mut self, p1 : [i16;2], p2 : [i16;2], p3 : [i16;2], c : Color){
        let result = self.polygon(
            &[p1[0], p2[0], p3[0]], 
            &[p1[1], p2[1], p3[1]], 
            c
        );
    }
    fn fill_triangle(&mut self, p1 : [i16;2], p2 : [i16;2], p3 : [i16;2], c : Color){
        let result = self.filled_polygon(
            &[p1[0], p2[0], p3[0]], 
            &[p1[1], p2[1], p3[1]], 
            c
        );
    }
}
#[derive(Clone)]
struct Object{
    tris : Vec<[[f32;4];3]>,
    rot : [f32;4],
    vel : [f32;4],
    rot_vel : [f32;4]
}

impl Object{
    fn new(mut tris:Vec<[[f32;4];3]>, rot:[f32;4])->Self{
        return Object{tris, rot, vel : [0.0, 0.0, 0.0, 0.0], rot_vel : [0.0, 0.0, 0.0, 0.0]};
    }
    fn center(&self)->[f32;4]{
        let mut c = [0.0, 0.0, 0.0, 1.0];
        let n = self.tris.len() as f32;
        for tri in &self.tris{
            c = c.add(tri.center());
        }
        return c.scale([1.0/n, 1.0/n, 1.0/n, 1.0])
    }
    fn sort_tris(&mut self){
        &self.tris.sort_by(|a,b| b.center().magnitude().partial_cmp(&a.center().magnitude()).unwrap());
    }
    fn load_obj_file(file_path:String)->Self{
        let file = File::open(file_path).unwrap();
        let reader = BufReader::new(file);
        let mut ts : Vec<[[f32;4];3]> = Vec::new();
        let mut points : Vec<[f32;4]> = Vec::new();
        
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
                            1.0
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
        for i in 0..self.tris.len(){
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
}

fn sort_objs(engine : &mut Engine){
    let r = engine.camera.rot.negative();
    let p = engine.camera.pos;
    &engine.objects.sort_by(|a, b| b.rotate_point(r, p).translate(p.negative()).center().magnitude().partial_cmp(&a.rotate_point(r, p).translate(p.negative()).center().magnitude()).unwrap());
}


fn main() {
    let mut fps_manager = FPSManager::new();
    fps_manager.set_framerate(60);


    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let sdl_image_context = image::init(image::InitFlag::all());

    let mut window = video_subsystem.window("game", 500, 500)
        .fullscreen_desktop()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();
    let window_texture : sdl2::surface::Surface = image::LoadSurface::from_file(Path::new("assets/dabebe.png")).unwrap();
    window.set_icon(window_texture);
    
    let mut canvas : WindowCanvas = window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();
    let screen_width = canvas.output_size().unwrap().0 as i32;
    let screen_height = canvas.output_size().unwrap().1 as i32;
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut objs : Vec<Object> = Vec::new();
    let max_fps = 60_u32;

    for i in 0..3{
        objs.push(Object::load_obj_file("assets/cube.obj".to_string()).scale([1.0, 1.0, 1.0, 1.0]).translate([i as f32 * 5.0, 0.0, 15.0, 0.0]));
        objs[i].rot_vel = [0.0, 0.0, 90_f32.to_radians(), 0.0];
    }
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
        render_distance : 100.0,
        window_height : canvas.output_size().unwrap().1 as f32,
        window_width : canvas.output_size().unwrap().0 as f32,
        objects : objs

    };
    let cspeed = 10.0;
    let rspeed = 60.0_f32.to_radians();
    let mat3d = engine.matrix3d();
    let mut FPS = max_fps as f32;
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
        engine.camera.rot = cam.rot.add(cam.rot_vel.scale([1.0/FPS, 1.0/FPS, 1.0/FPS, 1.0]));
        sort_objs(&mut engine);
        engine.camera.pos = cam.pos.add(
            cam.vel.multiply_mat([
                [engine.camera.rot[1].cos(), engine.camera.rot[2].sin(), -engine.camera.rot[1].sin(), 0.0],
                [engine.camera.rot[2].sin(), engine.camera.rot[0].cos(), engine.camera.rot[0].sin(), 0.0],
                [engine.camera.rot[1].sin(), -engine.camera.rot[0].sin(), engine.camera.rot[1].cos(), 0.0],
                [0.0, 0.0, 0.0, 1.0]
            ]).scale([1.0/FPS, 1.0/FPS, 1.0/FPS, 1.0])
        );
        let light = [0.0, 0.0, -1.0, 1.0].normalize();
        for i in 0..engine.objects.len(){
            
            let center = engine.objects[i].center();
            if engine.objects[i].vel.magnitude() != 0.0{
                engine.objects[i] = engine.objects[i].translate(engine.objects[i].vel.scale([1.0/FPS, 1.0/FPS, 1.0/FPS, 1.0]));
            }
            if engine.objects[i].rot_vel.magnitude() != 0.0{
                engine.objects[i] = engine.objects[i].rotate_point(engine.objects[i].rot_vel.scale([1.0/FPS, 1.0/FPS, 1.0/FPS, 1.0]), center);
            }
            

            let mut obj = engine.objects[i].rotate_point(cam.rot.negative(), cam.pos).translate(cam.pos.negative());
            obj.sort_tris();
            
            
            for tri in obj.tris{
                let normal = tri.normal();
                let c = tri.center()[2];
                if normal.dot_product(tri[0]) <= 0.0 && c > engine.clip_distance && c < engine.render_distance{
                    let dp = normal.dot_product(light);
                    if dp >= 0.0{
                        let t = tri.scale([engine.window_width/2.0, engine.window_height/2.0, 1.0, 1.0]).multiply_mat(mat3d);
                        let ew = engine.window_width/2.0; let eh = engine.window_height/2.0;
                        let t03 = t[0][3]; let t13 = t[1][3]; let t23 = t[2][3]; let c = (255.0*dp) as u8;
                        canvas.fill_triangle(
                            [(t[0][0]/t03+ew) as i16, (t[0][1]/t03+eh) as i16],     
                            [(t[1][0]/t13+ew) as i16, (t[1][1]/t13+eh) as i16],
                            [(t[2][0]/t23+ew) as i16, (t[2][1]/t23+eh) as i16],
                            Color::from((c, c, c))
                        );
                    }
                }
            }   
        }

        canvas.string(
            5,
            5,
            &format!("FPS: {}", FPS).to_string(),
            Color::WHITE
        );
        let del = fps_manager.delay();
        canvas.present();
        
        if del != 0{
            FPS = (max_fps-100/del) as f32;
        } else {
            FPS = 1.0;
        }
        fps_manager.set_framerate(FPS as u32).unwrap();
        //std::thread::sleep(std::time::Duration::from_millis(1000/max_fps as u64));
        

    }
}
