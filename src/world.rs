use std::fs::{read_to_string, File};
use std::io::{BufRead, BufReader, Read};

use crate::ops::operations4x4;
use crate::Tri3d;
use crate::Vec3;
use sdl2::pixels::Color;
use sdl2::surface::{Surface, SurfaceContext, SurfaceRef};
pub struct Camera {
    pub fov: f32,
    pub pos: [f32; 4],
    pub dir: [f32; 4],
    pub vel: [f32; 4],
    pub rot_vel: [f32; 4],
    pub clip_distance: f32,
    pub render_distance: f32,
    pub window_height: f32,
    pub window_width: f32,
}

pub struct Engine {
    pub camera: Camera,
    pub objects: Vec<Mesh>,
    pub depth_buffer: Vec<f32>,
    pub transparency_buffer: Vec<(f32, Color)>,
    pub lights: Vec<crate::light::Light>,
    pub ambient: Color,
}
pub fn matrix3d_perspective(
    fov: f32,
    render_distance: f32,
    clip_distance: f32,
    window_width: f32,
    window_height: f32,
) -> [[f32; 4]; 4] {
    let t = (fov.to_radians() * 0.5).tan();
    let zratio = render_distance / (render_distance - clip_distance);
    [
        [-window_height / (t * window_width), 0.0, 0.0, 0.0],
        [0.0, -1.0 / t, 0.0, 0.0],
        [0.0, 0.0, zratio, 1.0],
        [0.0, 0.0, -clip_distance * zratio, 0.0],
    ]
}
pub fn matrix3d_ortho(r: f32, t: f32, n: f32, f: f32) -> [[f32; 4]; 4] {
    [
        [-1.0 / r, 0.0, 0.0, 0.0],
        [0.0, -1.0 / t, 0.0, 0.0],
        [0.0, 0.0, 1.0 / (f - n), 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}
impl Engine {
    pub fn x_rot(angle: f32) -> [[f32; 4]; 4] {
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, angle.cos(), angle.sin(), 0.0],
            [0.0, -angle.sin(), angle.cos(), 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }
    pub fn y_rot(angle: f32) -> [[f32; 4]; 4] {
        [
            [angle.cos(), 0.0, -angle.sin(), 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [angle.sin(), 0.0, angle.cos(), 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }
    pub fn z_rot(angle: f32) -> [[f32; 4]; 4] {
        [
            [angle.cos(), -angle.sin(), 0.0, 0.0],
            [angle.sin(), angle.cos(), 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }
    pub fn sort_objs(&mut self) {
        let cpos = self.camera.pos;
        self.objects.sort_by(|a, b| {
            a.center()
                .subtract(cpos)
                .magnitude()
                .partial_cmp(&b.center().subtract(cpos).magnitude())
                .unwrap()
        })
    }
}
pub struct Mesh {
    pub tris: Vec<Tri3d>,
    pub vel: [f32; 4],
    pub rot_vel: [f32; 4],
    pub tex: String,
}

impl Mesh {
    pub fn new(tris: Vec<Tri3d>, tex: String) -> Self {
        Mesh {
            tris,
            vel: [0.0, 0.0, 0.0, 0.0],
            rot_vel: [0.0, 0.0, 0.0, 0.0],
            tex,
        }
    }
    #[inline]
    pub fn center(&self) -> [f32; 4] {
        let mut c = [0.0, 0.0, 0.0, 1.0];
        let n = 1.0/self.tris.len() as f32;
        for tri in &self.tris {
            c = c.add(tri.center());
        }
        c.scale([1.0 * n, 1.0 * n, 1.0 * n, 1.0])
    }

    pub fn load_obj_file(file_path: String, tex: String, col: Color, rfl: f32, trs: f32) -> Self {
        let file = File::open(file_path).unwrap();
        let reader = BufReader::new(file);
        let mut ts: Vec<Tri3d> = Vec::new();
        let mut t_n: Vec<[f32; 4]> = Vec::new();
        let mut points: Vec<[f32; 4]> = Vec::new();
        let mut t_c: Vec<[f32; 3]> = Vec::new();
        let obj_key: [&str; 4] = ["v", "f", "vt", "vn"];

        for line in reader.lines() {
            let ln = Box::leak(line.unwrap().into_boxed_str());
            let vals: Vec<&str> = ln.split_whitespace().collect();
            if !vals.is_empty() {
                if *vals[0] == *obj_key[0] {
                    points.push([
                        vals[1].parse::<f32>().unwrap(),
                        vals[2].parse::<f32>().unwrap(),
                        vals[3].parse::<f32>().unwrap(),
                        1.0,
                    ]);
                } else if *vals[0] == *obj_key[1] {
                    let p1: Vec<&str> = vals[1].split('/').collect();
                    let p2: Vec<&str> = vals[2].split('/').collect();
                    let p3: Vec<&str> = vals[3].split('/').collect();
                    if p1.len() == 2 {
                        ts.push(Tri3d::new(
                            [
                                points[p1[0].parse::<usize>().unwrap() - 1],
                                points[p2[0].parse::<usize>().unwrap() - 1],
                                points[p3[0].parse::<usize>().unwrap() - 1],
                            ],
                            [
                                t_c[p1[1].parse::<usize>().unwrap() - 1],
                                t_c[p2[1].parse::<usize>().unwrap() - 1],
                                t_c[p3[1].parse::<usize>().unwrap() - 1],
                            ],
                            [
                                [0.0, 0.0, 0.0, 1.0],
                                [0.0, 0.0, 0.0, 1.0],
                                [0.0, 0.0, 0.0, 1.0],
                            ],
                            col,
                            rfl,
                            trs,
                        ));
                    } else if p1.len() == 1 {
                        ts.push(Tri3d::new(
                            [
                                points[vals[1].parse::<usize>().unwrap() - 1],
                                points[vals[2].parse::<usize>().unwrap() - 1],
                                points[vals[3].parse::<usize>().unwrap() - 1],
                            ],
                            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]],
                            [
                                [0.0, 0.0, 0.0, 1.0],
                                [0.0, 0.0, 0.0, 1.0],
                                [0.0, 0.0, 0.0, 1.0],
                            ],
                            col,
                            rfl,
                            trs,
                        ));
                    } else if p1.len() == 3 {
                        ts.push(Tri3d::new(
                            [
                                points[p1[0].parse::<usize>().unwrap() - 1],
                                points[p2[0].parse::<usize>().unwrap() - 1],
                                points[p3[0].parse::<usize>().unwrap() - 1],
                            ],
                            [
                                t_c[p1[1].parse::<usize>().unwrap() - 1],
                                t_c[p2[1].parse::<usize>().unwrap() - 1],
                                t_c[p3[1].parse::<usize>().unwrap() - 1],
                            ],
                            [
                                t_n[p1[2].parse::<usize>().unwrap() - 1],
                                t_n[p2[2].parse::<usize>().unwrap() - 1],
                                t_n[p3[2].parse::<usize>().unwrap() - 1],
                            ],
                            col,
                            rfl,
                            trs,
                        ));
                    }
                } else if *vals[0] == *obj_key[2] {
                    t_c.push([
                        1.0 - vals[1].parse::<f32>().unwrap(),
                        1.0 - vals[2].parse::<f32>().unwrap(),
                        1.0,
                    ]);
                } else if *vals[0] == *obj_key[3] {
                    t_n.push(
                        [
                            vals[1].parse::<f32>().unwrap(),
                            vals[2].parse::<f32>().unwrap(),
                            vals[3].parse::<f32>().unwrap(),
                            1.0,
                        ]
                        .normalize(),
                    )
                }
            }
        }
        Mesh {
            tris: ts,
            vel: [0.0, 0.0, 0.0, 0.0],
            rot_vel: [0.0, 0.0, 0.0, 0.0],
            tex,
        }
    }
    pub fn translate(&self, t: [f32; 4]) -> Self {
        let mut s = Vec::new();
        for i in &self.tris {
            s.push(i.translate(t));
        }
        Mesh {
            tris: s,
            vel: self.vel,
            rot_vel: self.rot_vel,
            tex: self.tex.as_str().to_string(),
        }
    }
    pub fn scale(&self, t: [f32; 4]) -> Self {
        let mut s = Vec::new();
        for i in &self.tris {
            s.push(i.scale(t));
        }
        Mesh {
            tris: s,
            vel: self.vel,
            rot_vel: self.rot_vel,
            tex: self.tex.as_str().to_string(),
        }
    }
    pub fn rotate_point(&self, deg: [f32; 4], point: [f32; 4]) -> Self {
        let mut ts = Vec::new();
        for i in 0..ts.len() {
            ts.push(self.tris[i].translate(point.negative()));
            if deg[2] != 0.0 {
                ts[i] = ts[i].multiply_mat(Engine::z_rot(deg[2]));
            }
            if deg[1] != 0.0 {
                ts[i] = ts[i].multiply_mat(Engine::y_rot(deg[1]));
            }
            if deg[0] != 0.0 {
                ts[i] = ts[i].multiply_mat(Engine::x_rot(deg[0]));
            }
            ts[i] = ts[i].translate(point);
        }
        Mesh {
            tris: ts,
            vel: self.vel,
            rot_vel: self.rot_vel,
            tex: self.tex.as_str().to_string(),
        }
    }
    #[inline]
    pub fn upd(
        &self,
        trans: [f32; 4],
        rot: [f32; 4],
        rot_point: [f32; 4],
    ) -> Self {

        let mut ts = vec![];
        for i in &self.tris {
            ts.push(i.upd(trans, rot, rot_point))
        }
        Mesh {
            tris: ts,
            vel: self.vel,
            rot_vel: self.rot_vel,
            tex: self.tex.as_str().to_string(),
        }
    }
    pub fn multiply_mat(&self, mat: [[f32; 4]; 4]) -> Self {
        let mut ts = vec![];
        for i in &self.tris {
            ts.push(i.multiply_mat(mat));
        }
        Mesh {
            tris: ts,
            vel: self.vel,
            rot_vel: self.rot_vel,
            tex: self.tex.as_str().to_string(),
        }
    }
}

pub fn point_at(pos: [f32; 4], target: [f32; 4], up: [f32; 4]) -> [[f32; 4]; 4] {
    let forward = target.subtract(pos).normalize();

    let up = up
        .subtract(forward.scale_c(up.dot_product(forward)))
        .normalize();

    let right = up.cross_product(forward).normalize();

    [
        [right[0], right[1], right[2], 0.0],
        [up[0], up[1], up[2], 0.0],
        [forward[0], forward[1], forward[2], 0.0],
        pos,
    ]
}
pub fn quick_inv(m: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    //    mat4x4 matrix;
    //    matrix.m[0][0] = m.m[0][0]; matrix.m[0][1] = m.m[1][0]; matrix.m[0][2] = m.m[2][0]; matrix.m[0][3] = 0.0f;
    //    matrix.m[1][0] = m.m[0][1]; matrix.m[1][1] = m.m[1][1]; matrix.m[1][2] = m.m[2][1]; matrix.m[1][3] = 0.0f;
    //    matrix.m[2][0] = m.m[0][2]; matrix.m[2][1] = m.m[1][2]; matrix.m[2][2] = m.m[2][2]; matrix.m[2][3] = 0.0f;
    //    matrix.m[3][0] = -(m.m[3][0] * matrix.m[0][0] + m.m[3][1] * matrix.m[1][0] + m.m[3][2] * matrix.m[2][0]);
    //    matrix.m[3][1] = -(m.m[3][0] * matrix.m[0][1] + m.m[3][1] * matrix.m[1][1] + m.m[3][2] * matrix.m[2][1]);
    //    matrix.m[3][2] = -(m.m[3][0] * matrix.m[0][2] + m.m[3][1] * matrix.m[1][2] + m.m[3][2] * matrix.m[2][2]);
    //    matrix.m[3][3] = 1.0f;
    //yes i copied olc what are you gonna do abt it
    [
        [m[0][0], m[1][0], m[2][0], 0.0],
        [m[0][1], m[1][1], m[2][1], 0.0],
        [m[0][2], m[1][2], m[2][2], 0.0],
        [
            -(m[3][0] * m[0][0] + m[3][1] * m[0][1] + m[3][2] * m[0][2]),
            -(m[3][0] * m[1][0] + m[3][1] * m[1][1] + m[3][2] * m[1][2]),
            -(m[3][0] * m[2][0] + m[3][1] * m[2][1] + m[3][2] * m[2][2]),
            1.0,
        ],
    ]
}

pub fn vec_intersect_plane(
    plane_p: [f32; 4],
    plane_n: [f32; 4],
    line_s: [f32; 4],
    line_e: [f32; 4],
) -> ([f32; 4], f32) {
    let plane_n = plane_n.normalize();
    let plane_d = -plane_p.dot_product(plane_n);
    let ad = line_s.dot_product(plane_n);
    let bd = line_e.dot_product(plane_n);
    let t = (-plane_d - ad) / (bd - ad);
    (line_s.add(line_e.subtract(line_s).scale_c(t)), t)
}

pub fn clip_tri(
    mat3d: [[f32;4];4],
    plane_p: [f32; 4],
    plane_n: [f32; 4],
    in_tri: Tri3d,
    out_tris: &mut [Tri3d; 2],
) -> usize {
    let plane_n = plane_n;

    let dist = |p: [f32; 4]| -> f32 { p.dot_product(plane_n) - plane_n.dot_product(plane_p) };
    
    let in_tri2d = in_tri.multiply_mat(mat3d);


    
    let mut in_points = Vec::new();
    let mut in_points2d = Vec::new();

    let mut out_points = Vec::new();
    let mut out_points2d = Vec::new();

    let mut in_uvs = Vec::new();
    let mut out_uvs = Vec::new();

    let mut in_ns = Vec::new();
    let mut out_ns = Vec::new();

    let d0 = dist(in_tri2d.ps[0]);
    let d1 = dist(in_tri2d.ps[1]);
    let d2 = dist(in_tri2d.ps[2]);

    if d0 >= 0.0 {
        in_points.push(in_tri.ps[0]);
        in_points2d.push(in_tri2d.ps[0]);

        in_uvs.push(in_tri.uvs[0]);
        in_ns.push(in_tri.ns[0]);
    } else {
        out_points.push(in_tri.ps[0]);

        out_points2d.push(in_tri2d.ps[0]);
        out_uvs.push(in_tri.uvs[0]);
        out_ns.push(in_tri.ns[0]);
    }

    if d1 >= 0.0 {
        in_points.push(in_tri.ps[1]);
        in_points2d.push(in_tri2d.ps[1]);
        in_uvs.push(in_tri.uvs[1]);
        in_ns.push(in_tri.ns[1]);
    } else {
        out_points.push(in_tri.ps[1]);
        out_points2d.push(in_tri2d.ps[1]);

        out_uvs.push(in_tri.uvs[1]);
        out_ns.push(in_tri.ns[1]);
    }

    if d2 >= 0.0 {
        in_points.push(in_tri.ps[2]);
        in_points2d.push(in_tri2d.ps[2]);

        in_uvs.push(in_tri.uvs[2]);
        in_ns.push(in_tri.ns[2]);
    } else {
        out_points.push(in_tri.ps[2]);
        out_points2d.push(in_tri2d.ps[2]);

        out_uvs.push(in_tri.uvs[2]);
        out_ns.push(in_tri.ns[2]);
    }

    if in_points.len() == 3 {
        out_tris[0] = in_tri;
        return 1;
    } else if in_points.is_empty() {
        return 0;
    } else if in_points.len() == 1 {
        out_tris[0] = in_tri;

        let ab = vec_intersect_plane(plane_p, plane_n, in_points2d[0], out_points2d[0]);
        let ac = vec_intersect_plane(plane_p, plane_n, in_points2d[0], out_points2d[1]);
        out_tris[0].ps[0] = in_points[0];
        out_tris[0].ps[1] = in_points[0].add(out_points[0].subtract(in_points[0]).scale_c(ab.1));
        out_tris[0].ps[2] = in_points[0].add(out_points[1].subtract(in_points[0]).scale_c(ac.1));

        let tab = ab.1;

        let tac = ac.1;

        out_tris[0].uvs[0] = in_uvs[0];
        out_tris[0].uvs[1] = [
            tab * (out_uvs[0][0] - in_uvs[0][0]) + in_uvs[0][0],
            tab * (out_uvs[0][1] - in_uvs[0][1]) + in_uvs[0][1],
            tab * (out_uvs[0][2] - in_uvs[0][2]) + in_uvs[0][2],
        ];
        out_tris[0].uvs[2] = [
            tac * (out_uvs[1][0] - in_uvs[0][0]) + in_uvs[0][0],
            tac * (out_uvs[1][1] - in_uvs[0][1]) + in_uvs[0][1],
            tac * (out_uvs[1][2] - in_uvs[0][2]) + in_uvs[0][2],
        ];

        out_tris[0].ns[0] = in_ns[0];
        out_tris[0].ns[1] = out_ns[0].subtract(in_ns[0]).scale_c(tab).add(in_ns[0]);
        out_tris[0].ns[2] = out_ns[1].subtract(in_ns[0]).scale_c(tac).add(in_ns[0]);

        //out_tris[0].col = Color::RED;
        return 1;
    } else if in_points.len() == 2 {
        out_tris[0] = in_tri;
        out_tris[1] = in_tri;

        //out_tris[0].col = Color::BLUE;
        //out_tris[1].col = Color::GREEN;

        let ab = vec_intersect_plane(plane_p, plane_n, in_points2d[1], out_points2d[0]);
        let ac = vec_intersect_plane(plane_p, plane_n, in_points2d[0], out_points2d[0]);
        let tac = ac.1;

        out_tris[0].ps[0] = in_points[0];
        out_tris[0].ps[1] = in_points[1];
        out_tris[0].ps[2] = in_points[0].add(out_points[0].subtract(in_points[0]).scale_c(tac));

        out_tris[0].uvs[0] = in_uvs[0];
        out_tris[0].uvs[1] = in_uvs[1];
        out_tris[0].uvs[2] = [
            tac * (out_uvs[0][0] - in_uvs[0][0]) + in_uvs[0][0],
            tac * (out_uvs[0][1] - in_uvs[0][1]) + in_uvs[0][1],
            tac * (out_uvs[0][2] - in_uvs[0][2]) + in_uvs[0][2],
        ];

        out_tris[0].ns[0] = in_ns[0];
        out_tris[0].ns[1] = in_ns[1];
        out_tris[0].ns[2] = out_ns[0].subtract(in_ns[0]).scale_c(tac).add(in_ns[0]);

        let tab = ab.1;

        out_tris[1].ps[0] = in_points[1];
        out_tris[1].ps[1] = out_tris[0].ps[2];
        out_tris[1].ps[2] = in_points[1].add(out_points[0].subtract(in_points[1]).scale_c(tab));

        out_tris[1].uvs[0] = in_uvs[1];
        out_tris[1].uvs[1] = out_tris[0].uvs[2];
        out_tris[1].uvs[2] = [
            tab * (out_uvs[0][0] - in_uvs[1][0]) + in_uvs[1][0],
            tab * (out_uvs[0][1] - in_uvs[1][1]) + in_uvs[1][1],
            tab * (out_uvs[0][2] - in_uvs[1][2]) + in_uvs[1][2],
        ];

        out_tris[1].ns[0] = in_ns[1];
        out_tris[1].ns[1] = out_tris[0].ns[2];
        out_tris[1].ns[2] = out_ns[0].subtract(in_ns[1]).scale_c(tab).add(in_ns[1]);

        return 2;
    }
    0
}
//VERY SLOW AND SHOULD ONLY BE USED ONCE PER OBJECT
pub fn estimate_normals(mesh: &mut Mesh) {
    for i in 0..mesh.tris.len() {
        let tri = mesh.tris[i];
        for j in 0..3 {
            let mut norm = tri.normal();
            let point = tri.ps[j];
            for i1 in 0..mesh.tris.len() {
                if i1 != i {
                    let tri1 = mesh.tris[i1];
                    let mut c = false;
                    for j1 in 0..3 {
                        let point1 = tri1.ps[j1];
                        if (point[0] - point1[0]).abs() < f32::EPSILON
                            && (point[1] - point1[1]).abs() < f32::EPSILON
                            && (point[2] - point1[2]).abs() < f32::EPSILON
                        {
                            c = true;
                        }
                    }
                    if c {
                        norm = norm.add(tri1.normal());
                    }
                }
            }
            mesh.tris[i].ns[j] = norm.normalize();
        }
    }
}

pub const POISSON_DISK: [[f32; 2]; 16] = [
    [-0.942_016, -0.399_062],
    [0.945_586, -0.768_907],
    [-0.094_184, -0.929_388],
    [0.344_959, 0.293_877],
    [-0.915_885, 0.457_714],
    [-0.815_442, -0.879_124],
    [-0.382_775, 0.276_768],
    [0.974_843, 0.756_483],
    [0.443_233, -0.975_115],
    [0.537_429, -0.473_734],
    [-0.264_969, -0.418_930],
    [0.791_975, 0.190_901],
    [-0.241_888, 0.997_065],
    [-0.814_099, 0.914_375],
    [0.199_841, 0.786_413],
    [0.143_831, -0.141_007],
];
