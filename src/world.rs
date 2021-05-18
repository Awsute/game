use std::fs::{File, read_to_string};
use std::io::{Read, BufReader, BufRead};

use crate::Vec3;
use crate::Tri3d;
use sdl2::surface::{Surface};
use crate::ops::operations4x4;


#[derive(Copy, Clone)]
pub struct Camera{
    pub fov : f32,
    pub pos : [f32;4],
    pub dir : [f32;4],
    pub vel : [f32;4],
    pub rot_vel : [f32;4],
    pub clip_distance : f32,
    pub render_distance : f32,
    pub window_height : f32,
    pub window_width : f32,
}

pub struct Engine{
    pub camera : Camera,
    pub objects : Vec<Mesh>,
    pub depth_buffer : Vec<f32>
}
pub fn matrix3d_perspective(fov : f32, render_distance : f32, clip_distance : f32, window_width : f32, window_height : f32)->[[f32;4];4]{
    let t = ((fov/2.0)*(std::f32::consts::PI/180.0)).tan();
    let zratio = render_distance/(render_distance-clip_distance);
    return [
        [-1.0/(t*window_width/window_height), 0.0, 0.0, 0.0],
        [0.0, -1.0/t, 0.0, 0.0],
        [0.0, 0.0, zratio, 1.0],
        [0.0, 0.0, -clip_distance*zratio, 0.0]
    ];
}
pub fn matrix3d_ortho(r:f32, l:f32, t:f32, b:f32, near:f32, far:f32)->[[f32;4];4]{
    return [
        [2.0/(r-l), 0.0, 0.0, 0.0],
        [0.0, 2.0/(t-b), 0.0, 0.0],
        [0.0, 0.0, 2.0/(far-near), -near/(far-near)],
        [0.0, 0.0, 0.0, 1.0]
    ]
}
impl Engine{


    pub fn x_rot(angle : f32)->[[f32;4];4]{
        return [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, angle.cos(), angle.sin(), 0.0],
            [0.0, -angle.sin(), angle.cos(), 0.0],
            [0.0, 0.0, 0.0, 1.0]
        ];
    }
    pub fn y_rot(angle : f32)->[[f32;4];4]{
        return [
            [angle.cos(), 0.0, -angle.sin(), 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [angle.sin(), 0.0, angle.cos(), 0.0],
            [0.0, 0.0, 0.0, 1.0]
        ];
    }
    pub fn z_rot(angle : f32)->[[f32;4];4]{
        return [
            [angle.cos(), -angle.sin(), 0.0, 0.0],
            [angle.sin(), angle.cos(), 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0]
        ];
    }

}
pub struct Mesh{
    pub tris : Vec<Tri3d>,
    pub rot : [f32;4],
    pub vel : [f32;4],
    pub rot_vel : [f32;4],
}

impl Mesh{
    pub fn new(tris:Vec<Tri3d>, rot:[f32;4], t_coords : Vec<[[f32;3];3]>, texture : Surface)->Self{
        return Mesh{tris, rot, vel : [0.0, 0.0, 0.0, 0.0], rot_vel : [0.0, 0.0, 0.0, 0.0]};
    }
    #[inline]
    pub fn center(&self)->[f32;4]{
        let mut c = [0.0, 0.0, 0.0, 1.0];
        let n = self.tris.len() as f32;
        for tri in &self.tris{
            c = c.add(tri.center());
        }
        return c.scale([1.0/n, 1.0/n, 1.0/n, 1.0])
    }

    pub fn load_obj_file(file_path:String)->Self{
        let file = File::open(file_path).unwrap();
        let reader = BufReader::new(file);
        let mut ts : Vec<Tri3d> = Vec::new();
        let mut t_n : Vec<[f32;4]> = Vec::new();
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
                    let p1 : Vec<&str> = vals[1].split("/").collect();
                    let p2 : Vec<&str> = vals[2].split("/").collect();
                    let p3 : Vec<&str> = vals[3].split("/").collect();
                    if p1.len() == 2{

                        
                        ts.push(
                            Tri3d::new(
                                [
                                    points[p1[0].parse::<usize>().unwrap()-1],
                                    points[p2[0].parse::<usize>().unwrap()-1],
                                    points[p3[0].parse::<usize>().unwrap()-1]
                                ],
                                [
                                    t_c[p1[1].parse::<usize>().unwrap()-1],
                                    t_c[p2[1].parse::<usize>().unwrap()-1],
                                    t_c[p3[1].parse::<usize>().unwrap()-1]
                                ],
                                [
                                    [0.0, 0.0, 0.0, 1.0],
                                    [0.0, 0.0, 0.0, 1.0],
                                    [0.0, 0.0, 0.0, 1.0]
                                ],
                            )
                        );

                    } else if p1.len() == 1 {
                        ts.push(
                            Tri3d::new(
                                [
                                    points[vals[1].parse::<usize>().unwrap()-1],
                                    points[vals[2].parse::<usize>().unwrap()-1],
                                    points[vals[3].parse::<usize>().unwrap()-1]
                                ],
                                [
                                    [0.0, 0.0, 0.0],
                                    [1.0, 0.0, 0.0],
                                    [1.0, 1.0, 0.0]
                                ],
                                [
                                    [0.0, 0.0, 0.0, 1.0],
                                    [0.0, 0.0, 0.0, 1.0],
                                    [0.0, 0.0, 0.0, 1.0]
                                ],
                            )
                        );
                    } else if p1.len() == 3{
                        ts.push(
                            Tri3d::new(
                                [
                                    points[p1[0].parse::<usize>().unwrap()-1],
                                    points[p2[0].parse::<usize>().unwrap()-1],
                                    points[p3[0].parse::<usize>().unwrap()-1]
                                ],
                                [
                                    t_c[p1[1].parse::<usize>().unwrap()-1],
                                    t_c[p2[1].parse::<usize>().unwrap()-1],
                                    t_c[p3[1].parse::<usize>().unwrap()-1]
                                ],
                                [
                                    t_n[p1[2].parse::<usize>().unwrap()-1],
                                    t_n[p2[2].parse::<usize>().unwrap()-1],
                                    t_n[p3[2].parse::<usize>().unwrap()-1]
                                ]
                            )
                        );
                    }
                } else if vals[0].to_string() == "vt".to_string(){
                    t_c.push(
                        [
                            1.0-vals[1].parse::<f32>().unwrap(), 
                            1.0-vals[2].parse::<f32>().unwrap(), 
                            1.0
                        ]
                    );
                } else if vals[0].to_string() == "vn".to_string(){
                    t_n.push(
                        [
                            vals[1].parse::<f32>().unwrap(), 
                            vals[2].parse::<f32>().unwrap(), 
                            vals[3].parse::<f32>().unwrap(), 
                            1.0
                        ].normalize()
                    )
                }
            }
        }
        return Mesh{tris:ts, rot:[0.0, 0.0, 0.0, 0.0], vel:[0.0, 0.0, 0.0, 0.0], rot_vel:[0.0, 0.0, 0.0, 0.0]};
    }
    pub fn translate(&self, t : [f32;4])->Self{
        let mut s = Vec::new();
        for i in &self.tris{
            s.push(i.translate(t));
        }
        return Mesh{tris:s, rot:self.rot, vel:self.vel, rot_vel:self.rot_vel};
    }
    pub fn scale(&self, t : [f32;4])->Self{
        let mut s = Vec::new();
        for i in &self.tris{
            s.push(i.scale(t));
        }
        return Mesh{tris:s, rot:self.rot, vel:self.vel, rot_vel:self.rot_vel};
    }
    pub fn rotate_point(&self, deg : [f32;4], point : [f32;4])->Self{
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
        return Mesh{tris:ts, rot:self.rot.add(deg), vel:self.vel, rot_vel:self.rot_vel};
    }
    #[inline]
    pub fn upd(&self, scalar : [f32;4], trans : [f32;4], rot : [f32;4], rot_point : [f32;4])->Self{
        let center = self.center();
        
        let ts = self.tris.iter().map(|&i|{
            return i.upd(scalar, trans, rot, rot_point, center);
        }).collect::<Vec<Tri3d>>();
        return Mesh{tris:ts, rot:self.rot.add(rot), vel:self.vel, rot_vel:self.rot_vel};
    }
    pub fn multiply_mat(&self, mat:[[f32;4];4])->Self{
        let ts = self.tris.iter().map(|&i|{
            return i.multiply_mat(mat)
        }).collect::<Vec<Tri3d>>();
        return Mesh{tris:ts, rot:self.rot, vel:self.vel, rot_vel:self.rot_vel};
    }

}



pub fn point_at(pos : [f32;4], target : [f32;4], up : [f32;4])->[[f32;4];4]{
    let forward = target.subtract(pos).normalize();
    
    let up = up.subtract(forward.scale_c(up.dot_product(forward))).normalize();
    
    let right = up.cross_product(forward).normalize();
    
    return [
        [right[0], right[1], right[2], 0.0],
        [up[0], up[1], up[2], 0.0],
        [forward[0], forward[1], forward[2], 0.0],
        [pos[0], pos[1], pos[2], 1.0]
    ];
}
fn quick_inv(m:[[f32;4];4])->[[f32;4];4]{
//    mat4x4 matrix;
//    matrix.m[0][0] = m.m[0][0]; matrix.m[0][1] = m.m[1][0]; matrix.m[0][2] = m.m[2][0]; matrix.m[0][3] = 0.0f;
//    matrix.m[1][0] = m.m[0][1]; matrix.m[1][1] = m.m[1][1]; matrix.m[1][2] = m.m[2][1]; matrix.m[1][3] = 0.0f;
//    matrix.m[2][0] = m.m[0][2]; matrix.m[2][1] = m.m[1][2]; matrix.m[2][2] = m.m[2][2]; matrix.m[2][3] = 0.0f;
//    matrix.m[3][0] = -(m.m[3][0] * matrix.m[0][0] + m.m[3][1] * matrix.m[1][0] + m.m[3][2] * matrix.m[2][0]);
//    matrix.m[3][1] = -(m.m[3][0] * matrix.m[0][1] + m.m[3][1] * matrix.m[1][1] + m.m[3][2] * matrix.m[2][1]);
//    matrix.m[3][2] = -(m.m[3][0] * matrix.m[0][2] + m.m[3][1] * matrix.m[1][2] + m.m[3][2] * matrix.m[2][2]);
//    matrix.m[3][3] = 1.0f;
//yes i copied olc what are you gonna do abt it
    return [
        [m[0][0], m[1][0], m[2][0], 0.0],
        [m[0][1], m[1][1], m[2][1], 0.0],
        [m[0][2], m[1][2], m[2][2], 0.0],
        [
            -(m[3][0] * m[0][0] + m[3][1] * m[0][1] + m[3][2] * m[0][2]), 
            -(m[3][0] * m[1][0] + m[3][1] * m[1][1] + m[3][2] * m[1][2]), 
            -(m[3][0] * m[2][0] + m[3][1] * m[2][1] + m[3][2] * m[2][2]), 
            1.0
        ]
    ]
}
pub fn look_at(pos : [f32;4], target : [f32;4], up : [f32;4])->[[f32;4];4]{
    return quick_inv(point_at(pos, target, up));
}
pub fn world_transform(rot:[f32;4], t:[f32;4])->[[f32;4];4]{
    return Engine::z_rot(rot[2]).multiply(Engine::y_rot(rot[1])).multiply(Engine::x_rot(rot[0])).multiply([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [t[0], t[1], t[2], 1.0]
    ]);
    
}