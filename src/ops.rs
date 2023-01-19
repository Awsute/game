use crate::Engine;
use sdl2::pixels::Color;
use std::cmp::PartialOrd;

#[inline]
fn fisqrt(x: f32)-> f32{
    if x == 0.0{
        return 0.0
    }
    let x2: f32 = x * 0.5f32;
    let mut i: u32 = unsafe { std::mem::transmute(x) }; // evil floating point bit level hacking
    i = 0x5f375a86 - (i >> 1);                        // what the fuck?
    let y: f32 = unsafe { std::mem::transmute(i) };
    let y  = y * ( 1.5 - ( x2 * y * y ) );     // 1st iteration
    //let y  = y * ( 1.5 - ( x2 * y * y ) );       // 2nd iteration, this can be removed

    return y;
}

pub trait Vec3 {
    fn scale(&self, scalar: [f32; 4]) -> Self;
    fn scale_c(&self, scalar: f32) -> Self;
    fn add(&self, a: [f32; 4]) -> Self;
    fn subtract(&self, a: [f32; 4]) -> Self;
    fn magnitude(&self) -> f32;
    fn normalize(&self) -> Self;
    fn negative(&self) -> Self;
    fn cross_product(&self, c: [f32; 4]) -> Self;
    fn dot_product(&self, d: [f32; 4]) -> f32;
    fn multiply_mat(&self, mat: [[f32; 4]; 4]) -> Self;
}
impl Vec3 for [f32; 4] {
    #[inline]
    fn scale(&self, scalar: [f32; 4]) -> [f32; 4] {
        [
            self[0] * scalar[0],
            self[1] * scalar[1],
            self[2] * scalar[2],
            self[3],
        ]
    }
    #[inline]
    fn scale_c(&self, scalar: f32) -> Self {
        [self[0] * scalar, self[1] * scalar, self[2] * scalar, self[3]]
    }
    #[inline]
    fn add(&self, a: [f32; 4]) -> [f32; 4] {
        [self[0] + a[0], self[1] + a[1], self[2] + a[2], self[3]]
    }
    #[inline]
    fn subtract(&self, a: [f32; 4]) -> [f32; 4] {
        [self[0] - a[0], self[1] - a[1], self[2] - a[2], self[3]]
    }
    #[inline]
    fn magnitude(&self) -> f32 {
        (self[0]*self[0] + self[1]*self[1] + self[2]*self[2]).sqrt()
    }
    #[inline]
    fn normalize(&self) -> [f32; 4] {
        let m = fisqrt(self[0]*self[0]+self[1]*self[1]+self[2]*self[2]);
        [self[0] * m, self[1] * m, self[2] * m, self[3]]
    }
    
    fn negative(&self) -> [f32; 4] {
        [-self[0], -self[1], -self[2], 1.0]
    }
    #[inline]
    fn cross_product(&self, c: [f32; 4]) -> [f32; 4] {
        [
            -self[1] * c[2] + c[1] * self[2],
            -self[2] * c[0] + c[2] * self[0],
            -self[0] * c[1] + c[0] * self[1],
            1.0,
        ]
    }
    #[inline]
    fn dot_product(&self, d: Self) -> f32 {
        self[0] * d[0] + self[1] * d[1] + self[2] * d[2]
    }
    #[inline]
    fn multiply_mat(&self, m: [[f32; 4]; 4]) -> [f32; 4] {
        [
            self[0] * m[0][0] + self[1] * m[1][0] + self[2] * m[2][0] + self[3] * m[3][0],
            self[0] * m[0][1] + self[1] * m[1][1] + self[2] * m[2][1] + self[3] * m[3][1],
            self[0] * m[0][2] + self[1] * m[1][2] + self[2] * m[2][2] + self[3] * m[3][2],
            self[0] * m[0][3] + self[1] * m[1][3] + self[2] * m[2][3] + self[3] * m[3][3],
        ]
    }

}

#[derive(Copy,Clone)]
pub struct Tri3d {
    pub ps: [[f32; 4]; 3],
    pub uvs: [[f32; 3]; 3],
    pub ns: [[f32; 4]; 3],
    pub col: Color,
    pub rfl: f32,
    pub trs: f32,
}
impl Tri3d {
    pub fn new(
        ps: [[f32; 4]; 3],
        uvs: [[f32; 3]; 3],
        ns: [[f32; 4]; 3],
        col: Color,
        rfl: f32,
        trs: f32,
    ) -> Self {
        Self {
            ps,
            uvs,
            ns,
            col,
            rfl,
            trs,
        }
    }
    pub fn empty() -> Self {
        Tri3d {
            ps: [
                [0.0, 0.0, 0.0, 1.0],
                [0.0, 0.0, 0.0, 1.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            uvs: [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]],
            ns: [
                [0.0, 0.0, 0.0, 1.0],
                [0.0, 0.0, 0.0, 1.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            col: Color::WHITE,
            rfl: 0.0,
            trs: 0.0,
        }
    }
    pub fn normal(&self) -> [f32; 4] {
        self.ps[2]
            .subtract(self.ps[0])
            .cross_product(self.ps[1].subtract(self.ps[0]))
            .normalize() //sheeeesh
    }
    pub fn translate(&self, t: [f32; 4]) -> Self {
        Self::new(
            [self.ps[0].add(t), self.ps[1].add(t), self.ps[2].add(t)],
            self.uvs,
            self.ns,
            self.col,
            self.rfl,
            self.trs,
        )
    }
    pub fn scale(&self, t: [f32; 4]) -> Self {
        Self::new(
            [
                self.ps[0].scale(t),
                self.ps[1].scale(t),
                self.ps[2].scale(t),
            ],
            self.uvs,
            self.ns,
            self.col,
            self.rfl,
            self.trs,
        )
    }
    pub fn center(&self) -> [f32; 4] {
        self.ps[0]
            .add(self.ps[1])
            .add(self.ps[2])
            .scale([0.333, 0.333, 0.333, 0.333])
    }
    #[inline]
    pub fn multiply_mat(&self, m: [[f32; 4]; 4]) -> Self {
        Self {
            ps: [
                self.ps[0].multiply_mat(m),
                self.ps[1].multiply_mat(m),
                self.ps[2].multiply_mat(m),
            ],
            uvs: self.uvs,
            ns: [
                self.ns[0].multiply_mat(m),
                self.ns[1].multiply_mat(m),
                self.ns[2].multiply_mat(m),
            ],
            col: self.col,
            rfl: self.rfl,
            trs: self.trs,
        }
    }
    pub fn upd(
        &self,
        trans: [f32; 4],
        rot: [f32; 4],
        rot_point: [f32; 4],
    ) -> Self {
        self.translate(rot_point.negative()).multiply_mat(Engine::z_rot(rot[2])).multiply_mat(Engine::y_rot(rot[1])).multiply_mat(Engine::x_rot(rot[0])).translate(rot_point).translate(trans)
    }
}

pub fn clamp<T:PartialOrd>(val: T, min_val: T, max_val: T) -> T{
    if val > max_val {
        max_val
    } else if val < min_val {
        min_val
    } else {
        val
    }
}
pub fn inverse4x4(mat: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut inv = [[0.0; 4]; 4];
    let mut det = 0.0;

    for i in 0..4 {
        for j in 0..4 {
            let mut submat = [[0.0; 3]; 3];
            let mut k = 0;
            let mut l = 0;
            for p in 0..4 {
                for q in 0..4 {
                    if p != i && q != j {
                        submat[k][l] = mat[p][q];
                        l += 1;
                        if l == 3 {
                            k += 1;
                            l = 0;
                        }
                    }
                }
            }
            let subdet = submat[0][0]*(submat[1][1]*submat[2][2]-submat[1][2]*submat[2][1]) - submat[0][1]*(submat[1][0]*submat[2][2]-submat[1][2]*submat[2][0]) + submat[0][2]*(submat[1][0]*submat[2][1]-submat[1][1]*submat[2][0]);
            det += ((i+j) % 2 == 0) as i32 as f32 * subdet * mat[i][j];
            inv[j][i] = ((((i+j) % 2) == 0) as i32 as f32 * subdet) / det;
        }
    }
    inv
}
