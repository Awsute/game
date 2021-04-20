pub trait Vec3{
    fn scale(&self, scalar : Self)->Self;
    fn add(&self, a : Self)->Self;
    fn subtract(&self, a : Self)->Self;
    fn magnitude(&self)->f32;
    fn normalize(&self)->Self;
    fn negative(&self)->Self;
    fn cross_product(&self, c : Self)->Self;
    fn dot_product(&self, d : Self)->f32;
    fn multiply_mat(&self, mat : [[f32;4];4])->Self;
}
impl Vec3 for [f32;4]{
    fn scale(&self, scalar : [f32;4])->[f32;4]{
        return [self[0]*scalar[0], self[1]*scalar[1], self[2]*scalar[2], self[3]];
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
pub trait Tri3d{
    fn normal(&self)->[f32;4];
    fn translate(&self, t:[f32;4])->Self;
    fn scale(&self, t:[f32;4])->Self;
    fn center(&self)->[f32;4];
    fn multiply_mat(&self, m:[[f32;4];4])->Self;
    fn upd(&self, scalar : [f32;4], trans : [f32;4], rot : [f32;4], rot_point : [f32;4])->Self;
}
impl Tri3d for [[f32;4];3]{
    fn normal(&self)->[f32;4]{
        return self[2].subtract(self[0]).cross_product(self[1].subtract(self[0])).normalize();//sheeeesh
    }
    fn translate(&self, t : [f32;4])->Self{
        return [self[0].add(t), self[1].add(t), self[2].add(t)];
    }
    fn scale(&self, t : [f32;4])->Self{
        return [self[0].scale(t), self[1].scale(t), self[2].scale(t)];
    }
    fn center(&self)->[f32;4]{
        return self[0].add(self[1]).add(self[2]).scale([1.0/3.0, 1.0/3.0, 1.0/3.0, 1.0])
    }
    fn multiply_mat(&self, m:[[f32;4];4])->Self{
        return [
            self[0].multiply_mat(m),
            self[1].multiply_mat(m),
            self[2].multiply_mat(m)
        ];
    }
    fn upd(&self, scalar : [f32;4], trans : [f32;4], rot : [f32;4], rot_point : [f32;4], center : [f32;4])->Self{
        let mut t = i;
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