use crate::{Engine};
use sdl2::pixels::Color;

#[derive(Copy, Clone)]
pub struct Vec3{
    x:f32,
    y:f32,
    z:f32,
    w:f32
}
impl std::ops::Index<usize> for Vec3{
    type Output = f32;
    fn index(&self, index:usize)->&Self::Output{
        match index{
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.w
        }
    }
}
impl std::ops::Add<Vec3> for Vec3{
    type Output = Self;
    fn add(self, a:Self)->Self::Output{
        return Self{x:self[0]+a[0], y:self[1]+a[1], z:self[2]+a[2], w:1.0}
    }
}
impl std::ops::Add<f32> for Vec3{
    type Output = Self;
    fn add(self, a:f32)->Self::Output{
        return Self{x:self[0]+a, y:self[1]+a, z:self[2]+a, w:1.0}
    }
}

impl std::ops::Sub<Vec3> for Vec3{
    type Output = Self;
    fn sub(self, a:Self)->Self::Output{
        return Self{x:self[0]-a[0], y:self[1]-a[1], z:self[2]-a[2], w:1.0}
    }
}
impl std::ops::Sub<f32> for Vec3{
    type Output = Self;
    fn sub(self, a:f32)->Self::Output{
        return Self{x:self[0]-a, y:self[1]-a, z:self[2]-a, w:1.0}
    }
}
impl std::ops::Neg for Vec3{
    type Output = Self;
    fn neg(self)->Self::Output{
        return Self{x:-self[0], y:-self[1], z:-self[2], w:1.0}
    }
}

impl std::ops::Mul<Vec3> for Vec3{
    type Output = Self;
    fn mul(self, a:Self)->Self::Output{
        return Self{x:self[0]*a[0], y:self[1]*a[1], z:self[2]*a[2], w:1.0}
    }
}
impl std::ops::Mul<f32> for Vec3{
    type Output = Self;
    fn mul(self, a:f32)->Self::Output{
        return Self{x:self[0]*a, y:self[1]*a, z:self[2]*a, w:1.0}
    }
}
impl std::ops::Mul<[[f32;4];4]> for Vec3{
    type Output = Self;
    fn mul(self, a:[[f32;4];4])->Self::Output{
        Self {
            x:self[0] * a[0][0] + self[1] * a[1][0] + self[2] * a[2][0] + self[3] * a[3][0],
            y:self[0] * a[0][1] + self[1] * a[1][1] + self[2] * a[2][1] + self[3] * a[3][1],
            z:self[0] * a[0][2] + self[1] * a[1][2] + self[2] * a[2][2] + self[3] * a[3][2],
            w:self[0] * a[0][3] + self[1] * a[1][3] + self[2] * a[2][3] + self[3] * a[3][3]
        }
    }
}
impl Vec3{
    pub fn empty()->Self{
        return Vec3{x:0.0, y:0.0, z:0.0, w:1.0};
    }

    pub fn magnitude(&self)->f32{
        return (self[0].powi(2) + self[1].powi(2) + self[2].powi(2)).sqrt();
    }
    pub fn normalize(&self)->Self{
        let m = self.magnitude();
        return Vec3{x:self[0]/m, y:self[1]/m, z:self[2]/m, w:self[3]};
    }
    pub fn cross_product(&self, c: Self)->Self{
        return Self{x:-self[1]*c[2] + c[1]*self[2], y:-self[2]*c[0] + c[2]*self[0], z:-self[0]*c[1] + c[0]*self[1], w:1.0};
    }
    pub fn dot_product(&self, d: Self)->f32{
        return self[0]*d[0] + self[1]*d[1] + self[2]*d[2];
    }
    pub fn rot(self, r:[f32;4])->Self{
        return self
            *(Engine::z_rot(r[2]))
            *(Engine::y_rot(r[1]))
            *(Engine::x_rot(r[0]));
    }

}


pub struct Tri3d{
    pub ps : [Vec3;3], 
    pub uvs : [[f32;3];3], 
    pub ns : [Vec3;3],
    pub col : Color,
    pub rfl : f32
}
impl Tri3d{
    pub fn new(ps:[Vec3;3], uvs:[[f32;3];3], ns:[Vec3;3], col:Color, rfl : f32)->Self{
        return Self{ps, uvs, ns, col, rfl};
    }
    pub fn empty()->Self{
        return Tri3d{
            ps:[Vec3::empty(),Vec3::empty(),Vec3::empty()],
            uvs:[[0.0, 0.0, 0.0],[0.0, 0.0, 0.0],[0.0, 0.0, 0.0]],
            ns:[Vec3::empty(),Vec3::empty(),Vec3::empty()],
            col:Color::WHITE,
            rfl:0.0
        }
    }
    pub fn normal(&self)->Vec3{
        return (self.ps[2]-self.ps[0]).cross_product(self.ps[1]-self.ps[0]).normalize();//sheeeesh
    }
    pub fn translate(&self, t : Vec3)->Self{
        return Self::new([self.ps[0]+t, self.ps[1]+t, self.ps[2]+t], self.uvs, self.ns, self.col, self.rfl);
    }
    pub fn scale(&self, t : Vec3)->Self{
        return Self::new([self.ps[0]*t, self.ps[1]*t, self.ps[2]*t], self.uvs, self.ns, self.col, self.rfl);
    }
    pub fn center(&self)->Vec3{
        return (self.ps[0]+self.ps[1]+self.ps[2])*(1.0/3.0)
    }
    pub fn multiply_mat(&self, m:[[f32;4];4])->Self{
        return Self::new(
            [
                self.ps[0]*m,
                self.ps[1]*m,
                self.ps[2]*m
            ],
            self.uvs,
            [
                self.ns[0]*m,
                self.ns[1]*m,
                self.ns[2]*m
            ],
            self.col,
            self.rfl
        );
    }
    pub fn upd(&self, scalar : Vec3, trans : Vec3, rot : Vec3, rot_point : Vec3, center : Vec3)->Self{
        let mut t = *self;
        if scalar[0] != 0.0 || scalar[1] != 0.0 || scalar[2] != 0.0{
            t = t.translate(-center).scale(scalar).translate(center);
        }
        if rot[0] != 0.0 || rot[1] != 0.0 || rot[2] != 0.0{
            t = t.translate(-rot_point);
            if rot[2] != 0.0{
                t = t.multiply_mat(Engine::z_rot(rot[2]));
            }
            if rot[1] != 0.0{
                t = t.multiply_mat(Engine::y_rot(rot[1]));
            }
            if rot[0] != 0.0{
                t = t.multiply_mat(Engine::x_rot(rot[0]));
            }
            t = t.translate(rot_point);
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
pub fn max(n1:f32, n2:f32)->f32{
    return if n1>n2{
        n1
    } else{
        n2
    }
}
pub fn min(n1:f32, n2:f32)->f32{
    return if n1<n2{
        n1
    } else{
        n2
    }
}
pub fn clamp(val:f32, min_val:f32, max_val:f32)->f32{
    return if val > max_val{
        max_val
    } else if val < min_val{
        min_val
    } else {
        val
    }
}
