use crate::{Engine};

pub trait Vec3{
    fn scale(&self, scalar : [f32;4])->Self;
    fn scale_c(&self, scalar : f32)->Self;
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
    fn scale_c(&self, scalar : f32)->Self{
        return [self[0]*scalar, self[1]*scalar, self[2]*scalar, 1.0];
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


#[derive(Copy, Clone)]
pub struct Tri3d{
    pub ps : [[f32;4];3], 
    pub uvs : [[f32;3];3], 
    pub ns : [[f32;4];3]
}
impl Tri3d{
    pub fn new(ps:[[f32;4];3], uvs:[[f32;3];3], ns:[[f32;4];3])->Self{
        return Self{ps, uvs, ns};
    }
    pub fn normal(&self)->[f32;4]{
        return self.ps[2].subtract(self.ps[0]).cross_product(self.ps[1].subtract(self.ps[0])).normalize();//sheeeesh
    }
    pub fn translate(&self, t : [f32;4])->Self{
        return Self::new([self.ps[0].add(t), self.ps[1].add(t), self.ps[2].add(t)], self.uvs, self.ns);
    }
    pub fn scale(&self, t : [f32;4])->Self{
        return Self::new([self.ps[0].scale(t), self.ps[1].scale(t), self.ps[2].scale(t)], self.uvs, self.ns);
    }
    pub fn center(&self)->[f32;4]{
        return self.ps[0].add(self.ps[1]).add(self.ps[2]).scale([1.0/3.0, 1.0/3.0, 1.0/3.0, 1.0])
    }
    pub fn multiply_mat(&self, m:[[f32;4];4])->Self{
        return Self::new(
            [
                self.ps[0].multiply_mat(m),
                self.ps[1].multiply_mat(m),
                self.ps[2].multiply_mat(m)
            ],
            self.uvs,
            [
                self.ns[0].multiply_mat(m),
                self.ns[1].multiply_mat(m),
                self.ns[2].multiply_mat(m)
            ],
        );
    }
    pub fn upd(&self, scalar : [f32;4], trans : [f32;4], rot : [f32;4], rot_point : [f32;4], center : [f32;4])->Self{
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


pub trait operations4x4{
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