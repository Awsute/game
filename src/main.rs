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
use sdl2::gfx::framerate::FPSManager;
use sdl2::gfx::primitives::DrawRenderer;

mod world;
use world::{Engine, Mesh, Camera, vec_intersect_plane, clip_tri};
mod ops;
use ops::{Tri3d, Vec3};
mod drawing;
use drawing::DrawTri;
mod color;
use color::ColFuncs;

mod light;
use light::Light;

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
//hi

 
fn find_sdl_gl_driver() -> Option<u32> {
    for (index, item) in sdl2::render::drivers().enumerate() {
        if item.name == "opengl" {
            return Some(index as u32);
        }
    }
    None
}
fn main() {
    let world_up = [0.0, 1.0, 0.0, 1.0];
    let mut fps_manager = FPSManager::new();

    let list_id = [0.0, 0.0, 0.0, 0.0];
    let list_id_sc = [1.0, 1.0, 1.0, 1.0];

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let _audio_subsystem = sdl_context.audio().unwrap();
    let _sdl_image_context = image::init(image::InitFlag::all());

    let mut window = video_subsystem.window("game", 800, 750)
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
    
    canvas.set_scale(0.999, 0.999);
    canvas.window().gl_set_context_to_current();
    
    let screen_width = canvas.output_size().unwrap().0 as i32;
    let screen_height = canvas.output_size().unwrap().1 as i32;
    let mut event_pump = sdl_context.event_pump().unwrap();
    let _texture_creator = canvas.texture_creator();
    let max_fps = 60_u32;
    let player_cam = Camera{
        fov : 90.0,
        pos : [0.0, 0.0, 0.0, 1.0],
        dir : [0.0, 0.0, 1.0, 1.0],
        vel : [0.0, 0.0, 0.0, 0.0],
        rot_vel : [0.0, 0.0, 0.0, 0.0],
        clip_distance : 0.5,
        render_distance : 500.0,
        window_height : screen_height as f32,
        window_width : screen_width as f32,
        
    };




    
    let mut engine = Engine{
        camera : player_cam,
        objects : Vec::new(),
        depth_buffer : vec![0.0; (player_cam.window_height*player_cam.window_width) as usize],
        transparency_buffer : vec![(0.0, Color::WHITE); (player_cam.window_height*player_cam.window_width) as usize]
    };

    //engine.objects.push(Mesh::load_obj_file("assets/normalized_cube.obj".to_string(),"assets/white.png".to_string(), Color::WHITE, 0.0).scale([1.0, 50.0, 50.0, 1.0]).translate([-10.0, 0.0, -5.0, 0.0]));
    //engine.objects.push(Mesh::load_obj_file("assets/normalized_teapot.obj".to_string(),"assets/white.png".to_string(), Color::RGB(250, 250, 10), 1.0).translate([-5.0, 0.0, 0.0, 0.0]));
        
    engine.objects.push(Mesh::load_obj_file("assets/dasauce.obj".to_string(),"assets/white.png".to_string(), Color::RED, 0.9, 0.0).translate([0.0, 0.0, 0.0, 0.0]));
    crate::world::estimate_normals(&mut engine.objects[0]);
    //engine.objects.push(Mesh::load_obj_file("assets/real_sphere.obj".to_string(),"assets/travisScot.png".to_string(), Color::RED, 1.0, 0.0).translate([-5.0, 0.0, 8.0, 0.0]));
    //crate::world::estimate_normals(&mut engine.objects[1]);

    //engine.objects.push(Mesh::load_obj_file("assets/normalized_cube.obj".to_string(),"assets/white.png".to_string(), Color::WHITE, 0.0, 0.0).scale([1.0, 10.0, 10.0,  1.0]).translate([-7.0, 0.0, 5.0, 0.0]));
    //engine.objects[0].rot_vel = [45_f32.to_radians(), 90_f32.to_radians(), 0.0, 1.0];
    
    let mut l_src = Light::new([0.0, 0.0, 5.0, 1.0], Color::WHITE, [0.0, 0.0, -1.0, 1.0].normalize(), world::matrix3d_ortho(10.0, 10.0, 0.0, 50.0));
    let cspeed = 10.0;
    let rspeed = 60.0_f32.to_radians();
    let mat3d = world::matrix3d_perspective(engine.camera.fov, engine.camera.render_distance, engine.camera.clip_distance, engine.camera.window_width, engine.camera.window_height);
    let mut seconds_passed = 0.0;
    
    
    let objs_moved = |objs : &Vec<Mesh>|->bool{
        for j in objs{
            if j.vel[0] != 0.0 || j.vel[1] != 0.0 || j.vel[2] != 0.0 || j.rot_vel[0] != 0.0 || j.rot_vel[1] != 0.0 || j.rot_vel[2] != 0.0{
                return true;
            }
        }
        return false;
    };

    let cam_moved = |camera : &Camera|->bool{
        return camera.vel[0] != 0.0 || camera.vel[1] != 0.0 || camera.vel[2] != 0.0 || camera.rot_vel[0] != 0.0 || camera.rot_vel[1] != 0.0 || camera.rot_vel[2] != 0.0
    };
    
    
    'running: loop {
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        let FPS = fps_manager.get_framerate() as f32;
        seconds_passed += 1.0/FPS;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown {keycode: Some(Keycode::W), .. } => {
                    engine.camera.vel[2] = 1.0;
                }, Event::KeyUp {keycode: Some(Keycode::W), .. } => {
                    engine.camera.vel[2] = 0.0;
                },
                
                Event::KeyDown {keycode: Some(Keycode::A), .. } => {
                    engine.camera.vel[0] = 1.0;
                }, Event::KeyUp {keycode: Some(Keycode::A), .. } => {
                    engine.camera.vel[0] = 0.0;
                },
                
                Event::KeyDown {keycode: Some(Keycode::S), .. } => {
                    engine.camera.vel[2] = -1.0;
                }, Event::KeyUp {keycode: Some(Keycode::S), .. } => {
                    engine.camera.vel[2] = 0.0;
                },
                
                Event::KeyDown {keycode: Some(Keycode::D), .. } => {
                    engine.camera.vel[0] = -1.0;
                }, Event::KeyUp {keycode: Some(Keycode::D), .. } => {
                    engine.camera.vel[0] = 0.0;
                },
                
                Event::KeyDown {keycode: Some(Keycode::E), .. } => {
                    engine.camera.vel[1] = 1.0;
                }, Event::KeyUp {keycode: Some(Keycode::E), .. } => {
                    engine.camera.vel[1] = 0.0;
                },
                
                Event::KeyDown {keycode: Some(Keycode::Q), .. } => {
                    engine.camera.vel[1] = -1.0;
                }, Event::KeyUp {keycode: Some(Keycode::Q), .. } => {
                    engine.camera.vel[1] = 0.0;
                },

                
                
                //--------------ROTATE--------------
                Event::KeyDown {keycode: Some(Keycode::Up), .. } => {
                    engine.camera.rot_vel[0] = -rspeed;
                }, Event::KeyUp {keycode: Some(Keycode::Up), .. } => {
                    engine.camera.rot_vel[0] = 0.0;
                },
                
                Event::KeyDown {keycode: Some(Keycode::Down), .. } => {
                    engine.camera.rot_vel[0] = rspeed;
                }, Event::KeyUp {keycode: Some(Keycode::Down), .. } => {
                    engine.camera.rot_vel[0] = 0.0;
                },
                
                Event::KeyDown {keycode: Some(Keycode::Left), .. } => {
                    engine.camera.rot_vel[1] = -rspeed;
                }, Event::KeyUp {keycode: Some(Keycode::Left), .. } => {
                    engine.camera.rot_vel[1] = 0.0;
                },
                
                Event::KeyDown {keycode: Some(Keycode::Right), .. } => {
                    engine.camera.rot_vel[1] = rspeed;
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
        
        
        //update camera
        let rvel = engine.camera.rot_vel.scale_c(1.0/FPS);
        engine.camera.dir = engine.camera.dir.multiply_mat(Engine::y_rot(rvel[1]));
        
        let cam_fwd = engine.camera.dir.normalize();
        let cam_up = world_up.subtract(engine.camera.dir.scale_c(world_up.dot_product(engine.camera.dir))).normalize();
        let cam_right = cam_up.cross_product(engine.camera.dir).normalize();
        
        let mvel = [
            engine.camera.vel.dot_product(cam_right), 
            engine.camera.vel.dot_product(cam_up),
            engine.camera.vel.dot_product(cam_fwd),
            1.0
        ].scale_c(cspeed/FPS);
        engine.camera.pos = engine.camera.pos.add(mvel);
        let ew = engine.camera.window_width/2.0; let eh = engine.camera.window_height/2.0;
        
        let cam_pmat = world::point_at(engine.camera.pos, engine.camera.pos.add(engine.camera.dir), [0.0, 1.0, 0.0, 1.0]);
        let cam_mat = world::look_at(engine.camera.pos, engine.camera.pos.add(engine.camera.dir), [0.0, 1.0, 0.0, 1.0]);
        
        //in view space
       
        let r = engine.camera.fov.to_radians()*(engine.camera.window_height/engine.camera.window_width/2.0);
        let rsin = r.sin();
        let rcos = r.cos();

        let w_clip = [
            [[0.0, 0.0, engine.camera.render_distance, 1.0], [0.0, 0.0, -1.0, 1.0]],
            [[0.0, 0.0, engine.camera.clip_distance, 1.0], [0.0, 0.0, 1.0, 1.0]],
                        
            [[0.0, 0.0, 0.0, 1.0], [0.0, -rsin, rcos, 1.0]],
            [[0.0, 0.0, 0.0, 1.0], [0.0, rsin, rcos, 1.0]],
            
            [[0.0, 0.0, 0.0, 1.0], [-rsin, 0.0, rcos, 1.0]],
            [[0.0, 0.0, 0.0, 1.0], [rsin, 0.0, rcos, 1.0]],

        ];


        
        let tr = [ew, eh, 1.0, 1.0];
        
        //reset the buffers only when the camera or objects move
        if cam_moved(&engine.camera) || objs_moved(&engine.objects){
            engine.depth_buffer = vec![0.0; (player_cam.window_height*player_cam.window_width) as usize];
            engine.transparency_buffer = vec![(0.0, Color::WHITE); (player_cam.window_height*player_cam.window_width) as usize];
        }
        if objs_moved(&engine.objects){
            l_src.buf = vec![1.0; light::SHADOW_RESOLUTION.0*light::SHADOW_RESOLUTION.1];
        }

        for i in 0..engine.objects.len(){
            engine.objects[i] = engine.objects[i].upd(list_id_sc, engine.objects[i].vel.scale_c(1.0/FPS), engine.objects[i].rot_vel.scale_c(1.0/FPS), engine.objects[i].center());
            for j in 0..engine.objects[i].tris.len(){
                l_src.edit_shadow_buffer(engine.objects[i].tris[j]);
            }
        }
        
        for i in 0..engine.objects.len()*2{
            let obj = engine.objects[i%engine.objects.len()].multiply_mat(cam_mat);
            let otex : Surface = image::LoadSurface::from_file(Path::new(engine.objects[i%engine.objects.len()].tex.as_str())).unwrap();
            for j in 0..obj.tris.len(){
                
                let normal = obj.tris[j].normal();
                let c = obj.tris[j].center();
                if normal.dot_product(c) >= 0.0{
                    let draw = i >= engine.objects.len() || obj.tris[j].trs < 0.0;
                    let v = obj.tris[j];
                    let mut clipped = vec![v];
                    let trs = &mut [Tri3d::empty(), Tri3d::empty()];
                    for plane in &w_clip{
                        for _n in 0..clipped.len(){
                            let t_clipped = clip_tri(plane[0], plane[1], clipped[0], trs);
                            clipped.remove(0);
                            for b in 0..t_clipped{
                                clipped.push(trs[b]);
                            }
                        }
                    }
                    for tri in clipped{
                        if tri.trs != 1.0{
                            let off = [1.0, 1.0, 0.0, 0.0];
                            let mut t = tri.multiply_mat(mat3d);
                            let t03 = 1.0/t.ps[0][3]; let t13 = 1.0/t.ps[1][3]; let t23 = 1.0/t.ps[2][3];
                            t.uvs = tri.uvs;

                            t.uvs[0][1] *= t03;
                            t.uvs[1][1] *= t13;
                            t.uvs[2][1] *= t23;
                            
                            t.uvs[0][0] *= t03;
                            t.uvs[1][0] *= t13;
                            t.uvs[2][0] *= t23;
                            
                            t.uvs[0][2] = t03;
                            t.uvs[1][2] = t13;
                            t.uvs[2][2] = t23;
                            
                            t.ps[0] = t.ps[0].scale_c(t03).add(off).scale(tr);    
                            t.ps[1] = t.ps[1].scale_c(t13).add(off).scale(tr);
                            t.ps[2] = t.ps[2].scale_c(t23).add(off).scale(tr);
                            //for plane in &clip{
                            //    if v.len() > 0{
                            //        let cont = &mut [Tri3d::empty(), Tri3d::empty()];
                            //        let ntris = clip_tri(plane[0], plane[1], v[0], cont);
                            //        v.remove(0);
                            //        for b in 0..ntris{
                            //            v.push(cont[b]);
                            //        }
                            //    }
                            //}
                            
                            let mut etri = tri.multiply_mat(cam_pmat);
                            etri.ps[0] = etri.ps[0].scale_c(t03);
                            etri.ps[1] = etri.ps[1].scale_c(t13);
                            etri.ps[2] = etri.ps[2].scale_c(t23);

                            //let dp  = normal.dot_product([0.0, 0.0, -1.0, 1.0]);
                            //let darkness = (255.0*dp) as u8;
                            //canvas.fill_triangle(
                            //    o,     
                            //    g,
                            //    h,
                            //    Color::from((darkness, darkness, darkness))
                            //);
                            
                            
                            canvas.textured_triangle(
                                t.ps,
                                t.uvs,
                                otex.without_lock().unwrap(),
                                otex.pitch() as usize,
                                otex.width() as f32,
                                otex.height() as f32,
                                &mut engine,
                                etri,
                                &mut l_src,
                                etri.col.avg(Color::GRAY),
                                draw
                            );

                            //canvas.draw_triangle(
                            //    o,
                            //    g,
                            //    h,
                            //    Color::BLACK
                            //);
                        }           
                    }
                }    
            }
        }


        fps_manager.set_framerate(max_fps);
        let del = fps_manager.delay();
        if del != 0{
            fps_manager.set_framerate(1000/del);
        }
        canvas.string(
            5,
            5,
            &format!("FPS: {}", fps_manager.get_framerate()).to_string(),
            Color::WHITE
        );


        canvas.string(
            5,
            25,
            &format!("pos: (x: {}, y: {}, z: {})", engine.camera.pos[0], engine.camera.pos[1], engine.camera.pos[2]).to_string(),
            Color::WHITE
        );

        canvas.string(
            5,
            45,
            &format!("dir: (x: {}, y: {}, z: {})", engine.camera.dir[0], engine.camera.dir[1], engine.camera.dir[2]).to_string(),
            Color::WHITE
        );
        canvas.present();
    }
}
