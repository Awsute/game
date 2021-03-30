pub trait Vec3{
    fn scale(&self, scalar : Self)->Self;
    fn add(&self, a : Self)->Self;
    fn subtract(&self, a : Self)->Self;
    fn magnitude(&self)->f32;
    fn normalize(&self)->Self;
    fn cross_product(&self, c : Self)->Self;
    fn dot_product(&self, d : Self)->f32;
    fn multiply_mat(&self, mat : [[f32;3];3])->Self;
}
impl Vec3 for [f32;3]{
    fn scale(&self, scalar : [f32;3])->[f32;3]{
        return [self[0]*scalar[0], self[1]*scalar[1], self[2]*scalar[2]];
    }
    fn add(&self, a : [f32;3])->[f32;3]{
        return [self[0]*a[0], self[1]*a[1], self[2]*a[2]];
    }
    fn subtract(&self, a : [f32;3])->[f32;3]{
        return [self[0]-a[0], self[1]-a[1], self[2]-a[2]];
    }
    fn magnitude(&self)->f32{
        return (self[0].powi(2) + self[1].powi(2) + self[2].powi(2)).sqrt();
    }
    fn normalize(&self)->[f32;3]{
        let m = self.magnitude();
        return [self[0]/m, self[1]/m, self[2]/m];
    }
    fn cross_product(&self, c: [f32;3])->[f32;3]{
        return [self[1]*c[2] - c[1]*self[2], -self[2]*c[0] + c[2]*self[0], self[0]*c[1] - c[0]*self[1]];
    }
    fn dot_product(&self, d: Self)->f32{
        return self[0]*d[0] + self[1]*d[1] + self[2]*d[2];
    }
    fn multiply_mat(&self, m : [[f32;3];3])->[f32;3]{
        return [
            self[0] * m[0][0] + self[1] * m[1][0] + self[2] * m[2][0],
            self[0] * m[0][1] + self[1] * m[1][1] + self[2] * m[2][1],
            self[0] * m[0][2] + self[1] * m[1][2] + self[2] * m[2][2] 
        ];
    }
}

trait Tri3d{
    fn normal(&self)->[f32;3];
}
impl Tri3d for [[f32;3];3]{
    fn normal(&self)->[f32;3]{
        return self[1].subtract(self[0]).cross_product(self[2].subtract(self[0])).normalize();//sheeeesh
    }
}