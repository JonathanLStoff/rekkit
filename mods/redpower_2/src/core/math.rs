//! Orientation math: quaternions, 3x3 rotation matrices and the 24 block orientations.
//!
//! Ports `Quat`, `Matrix3` and `MathLib` of RedPower 2 core. An orientation is `down * 4 + rot`
//! with `down` in 0..6 (the face the part sits on) and `rot` in 0..4.

use super::spatial::Vector3;
use std::sync::OnceLock;

/// Quaternion with scalar part `s`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub s: f64,
}

impl Default for Quat {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Quat {
    pub const IDENTITY: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        s: 1.0,
    };

    pub const fn new(s: f64, x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z, s }
    }

    /// Rotation of `a` radians around the axis `(xi, yi, zi)`.
    pub fn around_axis(xi: f64, yi: f64, zi: f64, a: f64) -> Self {
        let h = a * 0.5;
        let sn = h.sin();
        Self::new(h.cos(), xi * sn, yi * sn, zi * sn)
    }

    /// `self = self * q`.
    pub fn multiply(&mut self, q: &Self) {
        let ts = self.s * q.s - self.x * q.x - self.y * q.y - self.z * q.z;
        let tx = self.s * q.x + self.x * q.s - self.y * q.z + self.z * q.y;
        let ty = self.s * q.y + self.x * q.z + self.y * q.s - self.z * q.x;
        let tz = self.s * q.z - self.x * q.y + self.y * q.x + self.z * q.s;
        *self = Self::new(ts, tx, ty, tz);
    }

    /// RedPower's `rightMultiply` (the product with the vector part's cross sign flipped).
    pub fn right_multiply(&mut self, q: &Self) {
        let ts = self.s * q.s - self.x * q.x - self.y * q.y - self.z * q.z;
        let tx = self.s * q.x + self.x * q.s + self.y * q.z - self.z * q.y;
        let ty = self.s * q.y - self.x * q.z + self.y * q.s + self.z * q.x;
        let tz = self.s * q.z + self.x * q.y - self.y * q.x + self.z * q.s;
        *self = Self::new(ts, tx, ty, tz);
    }

    pub fn mag(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.s * self.s).sqrt()
    }

    pub fn normalize(&mut self) {
        let d = self.mag();
        if d != 0.0 {
            let d = 1.0 / d;
            self.x *= d;
            self.y *= d;
            self.z *= d;
            self.s *= d;
        }
    }

    /// Rotate `v` in place (`q v q*`).
    pub fn rotate(&self, v: &mut Vector3) {
        let ts = -self.x * v.x - self.y * v.y - self.z * v.z;
        let tx = self.s * v.x + self.y * v.z - self.z * v.y;
        let ty = self.s * v.y - self.x * v.z + self.z * v.x;
        let tz = self.s * v.z + self.x * v.y - self.y * v.x;
        v.x = tx * self.s - ts * self.x - ty * self.z + tz * self.y;
        v.y = ty * self.s - ts * self.y + tx * self.z - tz * self.x;
        v.z = tz * self.s - ts * self.z - tx * self.y + ty * self.x;
    }
}

/// Row-major 3x3 matrix; vectors are rotated as row vectors (`v * M`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Matrix3 {
    pub xx: f64,
    pub xy: f64,
    pub xz: f64,
    pub yx: f64,
    pub yy: f64,
    pub yz: f64,
    pub zx: f64,
    pub zy: f64,
    pub zz: f64,
}

impl Matrix3 {
    pub const IDENTITY: Self = Self {
        xx: 1.0,
        xy: 0.0,
        xz: 0.0,
        yx: 0.0,
        yy: 1.0,
        yz: 0.0,
        zx: 0.0,
        zy: 0.0,
        zz: 1.0,
    };

    pub fn from_quat(q: &Quat) -> Self {
        Self {
            xx: q.s * q.s + q.x * q.x - q.z * q.z - q.y * q.y,
            xy: 2.0 * (q.s * q.z + q.x * q.y),
            xz: 2.0 * (q.x * q.z - q.s * q.y),
            yx: 2.0 * (q.x * q.y - q.s * q.z),
            yy: q.s * q.s + q.y * q.y - q.z * q.z - q.x * q.x,
            yz: 2.0 * (q.s * q.x + q.y * q.z),
            zx: 2.0 * (q.s * q.y + q.x * q.z),
            zy: 2.0 * (q.y * q.z - q.s * q.x),
            zz: q.s * q.s + q.z * q.z - q.y * q.y - q.x * q.x,
        }
    }

    pub fn multiply(&self, m: &Self) -> Self {
        Self {
            xx: self.xx * m.xx + self.xy * m.yx + self.xz * m.zx,
            xy: self.xx * m.xy + self.xy * m.yy + self.xz * m.zy,
            xz: self.xx * m.xz + self.xy * m.yz + self.xz * m.zz,
            yx: self.yx * m.xx + self.yy * m.yx + self.yz * m.zx,
            yy: self.yx * m.xy + self.yy * m.yy + self.yz * m.zy,
            yz: self.yx * m.xz + self.yy * m.yz + self.yz * m.zz,
            zx: self.zx * m.xx + self.zy * m.yx + self.zz * m.zx,
            zy: self.zx * m.xy + self.zy * m.yy + self.zz * m.zy,
            zz: self.zx * m.xz + self.zy * m.yz + self.zz * m.zz,
        }
    }

    /// Basis vector `n` (0 = x row, 1 = y row, otherwise z row).
    pub fn basis_vector(&self, n: usize) -> Vector3 {
        match n {
            0 => Vector3::new(self.xx, self.xy, self.xz),
            1 => Vector3::new(self.yx, self.yy, self.yz),
            _ => Vector3::new(self.zx, self.zy, self.zz),
        }
    }

    pub fn det(&self) -> f64 {
        self.xx * (self.yy * self.zz - self.yz * self.zy)
            - self.xy * (self.yx * self.zz - self.yz * self.zx)
            + self.xz * (self.yx * self.zy - self.yy * self.zx)
    }

    pub fn rotate(&self, v: &mut Vector3) {
        let tx = self.xx * v.x + self.yx * v.y + self.zx * v.z;
        let ty = self.xy * v.x + self.yy * v.y + self.zy * v.z;
        let tz = self.xz * v.x + self.yz * v.y + self.zz * v.z;
        v.x = tx;
        v.y = ty;
        v.z = tz;
    }
}

fn tables() -> &'static ([Quat; 24], [Matrix3; 24]) {
    static T: OnceLock<([Quat; 24], [Matrix3; 24])> = OnceLock::new();
    T.get_or_init(|| {
        use std::f64::consts::PI;
        let mut qs = [Quat::IDENTITY; 24];
        let flip = Quat::around_axis(1.0, 0.0, 0.0, PI);
        for j in 0..4 {
            let q1 = Quat::around_axis(0.0, 1.0, 0.0, (-PI / 2.0) * j as f64);
            qs[j] = q1;
            let mut q = q1;
            q.multiply(&flip);
            qs[j + 4] = q;
        }
        for i in 0..4usize {
            let k = ((i >> 1) | (i << 1)) & 3;
            let mut q2 = Quat::around_axis(0.0, 0.0, 1.0, PI / 2.0);
            q2.multiply(&Quat::around_axis(
                0.0,
                1.0,
                0.0,
                (PI / 2.0) * (k + 1) as f64,
            ));
            for j in 0..4 {
                let mut q1 = qs[j];
                q1.multiply(&q2);
                qs[8 + 4 * i + j] = q1;
            }
        }
        let mut ms = [Matrix3::IDENTITY; 24];
        for (m, q) in ms.iter_mut().zip(qs.iter()) {
            *m = Matrix3::from_quat(q);
        }
        (qs, ms)
    })
}

/// Matrix for orientation (`down` 0..6, `rot` 0..4). Out-of-range values wrap into 0..24.
pub fn orient_matrix(down: usize, rot: usize) -> Matrix3 {
    tables().1[(down * 4 + rot) % 24]
}

/// Quaternion for orientation (`down` 0..6, `rot` 0..4).
pub fn orient_quat(down: usize, rot: usize) -> Quat {
    tables().0[(down * 4 + rot) % 24]
}

/// `CoreLib.rotToSide`: side index (2..=5) a rotation 0..3 faces.
pub fn rot_to_side(r: i32) -> i32 {
    match r {
        0 => 5,
        1 => 3,
        2 => 4,
        _ => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_24_are_proper_rotations() {
        for d in 0..6 {
            for r in 0..4 {
                let m = orient_matrix(d, r);
                assert!((m.det() - 1.0).abs() < 1e-9);
                assert!((orient_quat(d, r).mag() - 1.0).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn all_24_are_distinct_and_axis_aligned() {
        let mut seen = Vec::new();
        for d in 0..6 {
            for r in 0..4 {
                let m = orient_matrix(d, r);
                for v in [m.xx, m.xy, m.xz, m.yx, m.yy, m.yz, m.zx, m.zy, m.zz] {
                    assert!(v.abs() < 1e-9 || (v.abs() - 1.0).abs() < 1e-9);
                }
                let key: Vec<i32> = [m.xx, m.xy, m.xz, m.yx, m.yy, m.yz, m.zx, m.zy, m.zz]
                    .iter()
                    .map(|v| v.round() as i32)
                    .collect();
                assert!(!seen.contains(&key));
                seen.push(key);
            }
        }
    }

    #[test]
    fn identity_orientation_and_flip() {
        assert_eq!(orient_matrix(0, 0).basis_vector(1).y.round(), 1.0);
        let mut v = Vector3::new(0.0, 1.0, 0.0);
        orient_matrix(1, 0).rotate(&mut v);
        assert!((v.y + 1.0).abs() < 1e-9);
    }

    #[test]
    fn quat_and_matrix_rotate_agree() {
        for d in 0..6 {
            for r in 0..4 {
                let mut a = Vector3::new(1.0, 2.0, 3.0);
                let mut b = a;
                orient_quat(d, r).rotate(&mut a);
                orient_matrix(d, r).rotate(&mut b);
                assert!(
                    (a.x - b.x).abs() < 1e-9
                        && (a.y - b.y).abs() < 1e-9
                        && (a.z - b.z).abs() < 1e-9
                );
            }
        }
    }

    #[test]
    fn matrix_multiply_identity_and_rot_to_side() {
        let m = orient_matrix(3, 2);
        assert_eq!(m.multiply(&Matrix3::IDENTITY), m);
        assert_eq!([0, 1, 2, 3].map(rot_to_side), [5, 3, 4, 2]);
    }
}
