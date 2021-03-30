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
    let g = [0.0, 0.0, 0.0] as [f32;3];
    let g1 = g.add([0.0, 0.2, 0.55]);
    //let window_texture : sdl2::surface::Surface = image::LoadSurface::from_file(Path::new("travisScot.png")).unwrap();
    //window.set_icon(window_texture);
    
    let mut canvas : Canvas<sdl2::video::Window> = window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    'running: loop {

    


        
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
        canvas.clear();

        canvas.present();
        std::thread::sleep(std::time::Duration::from_millis((1000.0/FPS) as u64));
    }
}
