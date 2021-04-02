//when shipping game, make sure you got everything in the folder with the game
extern crate sdl2;
use sdl2::pixels;
use sdl2::image;
use pixels::{Color};
use sdl2::render::{Canvas, Texture, TextureAccess, WindowCanvas, TextureCreator};
use sdl2::audio::{AudioCallback, AudioSpecWAV, AudioCVT, AudioSpecDesired};
use sdl2::rect::{Point, Rect};
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
mod impl3d;
use impl3d::Vec3;
use impl3d::Tri3d;
//hi

struct Engine{
    fov : f32,
    clip_distance : f32,
    render_distance : f32,
    window_height : f32,
    window_width : f32,
    objects : Vec<Object>
}

impl Engine{
    fn matrix(&self)->[[f32;3];3]{
        let t = (self.fov*180.0/(2.0*std::f32::consts::PI)).tan();
        let zr = self.render_distance/(self.render_distance-self.clip_distance);
        return [
            [(self.window_width/self.window_height)/t, 0.0, 0.0],
            [0.0, 1.0/t, 0.0],
            [0.0, 0.0, -self.clip_distance*zr]
        ];
    }
}


trait draw_tri{
    fn draw_triangle(&mut self, p1:Point, p2:Point, p3:Point);
}
impl draw_tri for WindowCanvas{
    fn draw_triangle(&mut self, p1:Point, p2:Point, p3:Point){
        self.draw_line(p1, p2);
        self.draw_line(p2, p3);
        self.draw_line(p3, p1);
    }
}
struct Object{
    tris : Vec<[[f32;3];3]>,
    rot : [f32;3],
    texture : String
}

impl Object{
    fn new(mut tris:Vec<[[f32;3];3]>, rot:[f32;3], texture:String)->Self{
        return Object{tris, rot, texture};
    }
    fn load_obj_file(file_path:String)->Self{
        let file = File::open(file_path).unwrap();
        let reader = BufReader::new(file);
        let mut ts : Vec<[[f32;3];3]> = Vec::new();
        let mut points : Vec<[f32;3]> = Vec::new();
        for line in reader.lines() {
            let ln = Box::leak(line.unwrap().into_boxed_str());
            let vals : Vec<&str> = ln.split_whitespace().collect();
            
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
        return Object{tris:ts, rot:[0.0, 0.0, 0.0], texture:"".to_string()};
    }
    fn draw(&self, matrix : [[f32;3];3])->Vec<[Point;3]>{
        let mut ps : Vec<[Point;3]> = Vec::new();
        for tri in &self.tris{
            let mut j : [[f32;4];3] = [[0.0,0.0,0.0,0.0],[0.0,0.0,0.0,0.0],[0.0,0.0,0.0,0.0]];
            for i in 0..3{
                let tz = tri[0][2];
                let t = tri[0].multiply_mat(matrix);
                j[i][0] = t[0];
                j[i][1] = t[1];
                j[i][2] = t[2];
                j[i][3] = tz;
            }
            
            ps.push(
                [
                    Point::new((tri[0][0]) as i32, (tri[0][1]) as i32), 
                    Point::new((tri[1][0]) as i32, (tri[1][1]) as i32),
                    Point::new((tri[2][0]) as i32, (tri[2][1]) as i32)
                ]
            );
            
        }
        return ps;
    }
    fn translate(&mut self, t : [f32;3])->&Self{
        for i in 0..self.tris.len(){
            self.tris[i] = self.tris[i].translate(t);
        }
        return self;
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
    let mut obj = Object::load_obj_file("assets/cube.obj".to_string());
    obj.translate([0.0, 0.0, 0.0]);
    let mut objs : Vec<Object> = Vec::new();
    let mut engine = Engine{
        fov : 90.0,
        clip_distance : 0.1,
        render_distance : 100.0,
        window_height : canvas.output_size().unwrap().1 as f32,
        window_width : canvas.output_size().unwrap().0 as f32,
        objects : objs

    };
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
                    
                }, Event::KeyUp {keycode: Some(Keycode::W), .. } => {
                    
                },
                
                Event::KeyDown {keycode: Some(Keycode::A), .. } => {

                }, Event::KeyUp {keycode: Some(Keycode::A), .. } => {

                },
                
                Event::KeyDown {keycode: Some(Keycode::S), .. } => {
                    
                }, Event::KeyUp {keycode: Some(Keycode::S), .. } => {
                    
                },
                
                Event::KeyDown {keycode: Some(Keycode::D), .. } => {

                }, Event::KeyUp {keycode: Some(Keycode::D), .. } => {

                },
                
                Event::KeyDown {keycode: Some(Keycode::E), .. } => {

                }, Event::KeyUp {keycode: Some(Keycode::E), .. } => {

                },
                
                Event::KeyDown {keycode: Some(Keycode::Q), .. } => {

                }, Event::KeyUp {keycode: Some(Keycode::Q), .. } => {

                },
                

            


                Event::MouseButtonDown{mouse_btn : MouseButton::Left, ..} => {

                },
                _ => {}
            }
        }
        // The rest of the game loop goes here...
        // ok
        let ts = obj.draw(mat3d);
        canvas.set_draw_color(Color::WHITE);
        for t in ts{
            canvas.draw_triangle(t[0], t[1], t[2]);
        }
        canvas.present();
        std::thread::sleep(std::time::Duration::from_millis((1000.0/FPS) as u64));
    }
}
