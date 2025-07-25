use std::ops::Mul;

use crate::{
    core::vec::Vector,
    prelude::{Mat3x3f, Vec3f},
};

pub type Quat = Quaternion;

#[derive(Debug, Clone, Copy)]
pub struct Quaternion {
    w: f32,
    i: f32,
    j: f32,
    k: f32,
}

impl Quaternion {
    pub fn new(wijk: [f32; 4]) -> Self {
        Self {
            w: wijk[0],
            i: wijk[1],
            j: wijk[2],
            k: wijk[3],
        }
    }

    pub fn wxyz(w: f32, x: f32, y: f32, z: f32) -> Self {
        Self {
            w,
            i: x,
            j: y,
            k: z,
        }
    }

    pub fn identity() -> Self {
        Self::wxyz(1., 0., 0., 0.)
    }

    /// rotate degree around axis
    pub fn angle_axis(degree: f32, axis: Vec3f) -> Self {
        let rad = degree.to_radians();
        let (cos, sin) = ((rad * 0.5).cos(), (rad * 0.5).sin());
        let v = axis.normalize() * sin;
        Self::wxyz(cos, v[0], v[1], v[2])
    }

    /// rotate around x,y,z axis
    pub fn euler(x: f32, y: f32, z: f32) -> Self {
        let mut q = if x == 0. {
            Self::angle_axis(x, Vec3f::vec([1., 0., 0.]))
        } else {
            Self::identity()
        };

        if y != 0. {
            let qy = Self::angle_axis(y, Vec3f::vec([0., 1., 0.]));
            q = qy * q;
        }

        if z != 0. {
            let qz = Self::angle_axis(z, Vec3f::vec([0., 0., 1.]));
            q = qz * q;
        }

        q
    }

    pub fn conjugate(&self) -> Self {
        Self::wxyz(self.w, -self.i, -self.j, -self.k)
    }


    #[rustfmt::skip]
    pub fn to_matrix(&self) -> Mat3x3f {
        let (r,i,j,k) = (self.w,self.i,self.j,self.k);
        Mat3x3f::mat([
           1. - 2. * (j * j + k * k), 2. * (i * j - r * k), 2.* (i * k + r * j),
           2. * (i * j + r * k), 1. - 2. * (i * i + k * k), 2. * (j * k - r * i),
           2. * (i * k - r * j), 2. * (j * k + r * i), 1. - 2. * (i * i + j * j),
        ], [3,3])
    }
}

impl Mul<Quaternion> for Quaternion {
    type Output = Quaternion;
    fn mul(self, rhs: Quaternion) -> Self::Output {
        let w = self.w * rhs.w - (self.i * rhs.i + self.j * rhs.j + self.k * rhs.k); //a1a2−(b1b2+c1c2+d1d2)
        let i = self.w * rhs.i + rhs.w * self.i + self.j * rhs.k - self.k * rhs.j; //a1b2+a2b1+c1d2−d1c2
        let j = self.w * rhs.j + rhs.w * self.j - self.i * rhs.k + self.k * rhs.i; //a1c2+a2c1−b1d2+d1b2
        let k = self.w * rhs.k + rhs.w * self.k + self.i * rhs.j - self.j * rhs.i; //a1d2+a2d1+b1c2−c1b2
        Self::wxyz(w, i, j, k)
    }
}
