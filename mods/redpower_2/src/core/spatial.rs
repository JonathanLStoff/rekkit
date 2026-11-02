//! Core spatial mathematics: 3D vector operations and block coordinate spatial navigation.
//!
//! Directly ports:
//! - `com.eloraam.redpower.core.Vector3`: 3D floating-point vector arithmetic, dot/cross products, normalization.
//! - `com.eloraam.redpower.core.WorldCoord`: 3D integer coordinate navigation, directional steps (0..5: -Y, +Y, -Z, +Z, -X, +X),
//!   indirect stepping (`indStep`), square distance, and directional sort comparator.

use rekkit_core::grid::BlockPos;
use rekkit_core::side::Direction;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

// ============================================================================
// Vector3: 3D Float Vector Arithmetic
// ============================================================================

/// 3D floating-point vector for spatial transforms, lighting, and ray casting.
///
/// Port of `com.eloraam.redpower.core.Vector3`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    /// Zero vector `<0.0, 0.0, 0.0>`.
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    /// Unit X vector `<1.0, 0.0, 0.0>`.
    pub const UNIT_X: Self = Self::new(1.0, 0.0, 0.0);

    /// Unit Y vector `<0.0, 1.0, 0.0>`.
    pub const UNIT_Y: Self = Self::new(0.0, 1.0, 0.0);

    /// Unit Z vector `<0.0, 0.0, 1.0>`.
    pub const UNIT_Z: Self = Self::new(0.0, 0.0, 1.0);

    /// Construct a new vector with components `(x, y, z)`.
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Construct a zero vector.
    pub const fn zero() -> Self {
        Self::ZERO
    }

    /// Set coordinates to `(xi, yi, zi)`.
    pub fn set(&mut self, xi: f64, yi: f64, zi: f64) {
        self.x = xi;
        self.y = yi;
        self.z = zi;
    }

    /// Set coordinates from another vector.
    pub fn set_vector(&mut self, v: &Self) {
        self.x = v.x;
        self.y = v.y;
        self.z = v.z;
    }

    /// Dot product with another vector: `v.x * self.x + v.y * self.y + v.z * self.z`.
    pub fn dot_product(&self, v: &Self) -> f64 {
        v.x * self.x + v.y * self.y + v.z * self.z
    }

    /// Dot product with raw coordinate components.
    pub fn dot_product_coords(&self, xi: f64, yi: f64, zi: f64) -> f64 {
        xi * self.x + yi * self.y + zi * self.z
    }

    /// In-place cross product: `self = self × v`.
    ///
    /// Matches original `Vector3#crossProduct`:
    /// ```text
    /// tx = this.y * v.z - this.z * v.y
    /// ty = this.z * v.x - this.x * v.z
    /// tz = this.x * v.y - this.y * v.x
    /// ```
    pub fn cross_product(&mut self, v: &Self) {
        let tx = self.y * v.z - self.z * v.y;
        let ty = self.z * v.x - self.x * v.z;
        let tz = self.x * v.y - self.y * v.x;
        self.x = tx;
        self.y = ty;
        self.z = tz;
    }

    /// Functional cross product returning a new vector: `self × v`.
    pub fn cross(&self, v: &Self) -> Self {
        Self {
            x: self.y * v.z - self.z * v.y,
            y: self.z * v.x - self.x * v.z,
            z: self.x * v.y - self.y * v.x,
        }
    }

    /// In-place addition of coordinate deltas.
    pub fn add_coords(&mut self, xi: f64, yi: f64, zi: f64) {
        self.x += xi;
        self.y += yi;
        self.z += zi;
    }

    /// In-place vector addition (`self += v`).
    pub fn add_vec(&mut self, v: Self) {
        self.x += v.x;
        self.y += v.y;
        self.z += v.z;
    }

    /// In-place vector subtraction (`self -= v`).
    pub fn sub_vec(&mut self, v: Self) {
        self.x -= v.x;
        self.y -= v.y;
        self.z -= v.z;
    }

    /// In-place scalar multiplication (`self *= d`).
    pub fn multiply(&mut self, d: f64) {
        self.x *= d;
        self.y *= d;
        self.z *= d;
    }

    /// Euclidean length / magnitude: `sqrt(x^2 + y^2 + z^2)`.
    pub fn mag(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Squared magnitude: `x^2 + y^2 + z^2`.
    pub fn mag_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// In-place normalization to unit length. No-op if magnitude is zero.
    pub fn normalize(&mut self) {
        let d = self.mag();
        if d != 0.0 {
            self.multiply(1.0 / d);
        }
    }

    /// Functional normalization returning a unit vector.
    pub fn normalized(&self) -> Self {
        let mut v = *self;
        v.normalize();
        v
    }

    /// Manhattan distance between the floored integer coordinates of two vectors.
    /// Matches `FractalLib.mdist`.
    pub fn mdist(&self, other: &Self) -> i32 {
        ((self.x.floor() - other.x.floor()).abs()
            + (self.y.floor() - other.y.floor()).abs()
            + (self.z.floor() - other.z.floor()).abs()) as i32
    }

    /// Convert floored coordinates to [`WorldCoord`].
    pub fn to_world_coord(&self) -> WorldCoord {
        WorldCoord::new(
            self.x.floor() as i32,
            self.y.floor() as i32,
            self.z.floor() as i32,
        )
    }

    /// Convert floored coordinates to [`BlockPos`].
    pub fn to_block_pos(&self) -> BlockPos {
        BlockPos::new(
            self.x.floor() as i32,
            self.y.floor() as i32,
            self.z.floor() as i32,
        )
    }
}

impl fmt::Display for Vector3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Vector:\n  < {:.6} {:.6} {:.6} >\n",
            self.x, self.y, self.z
        )
    }
}

impl Add for Vector3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl AddAssign for Vector3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for Vector3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl SubAssign for Vector3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Mul<f64> for Vector3 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl MulAssign<f64> for Vector3 {
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Div<f64> for Vector3 {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl DivAssign<f64> for Vector3 {
    fn div_assign(&mut self, rhs: f64) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl Neg for Vector3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl From<(f64, f64, f64)> for Vector3 {
    fn from((x, y, z): (f64, f64, f64)) -> Self {
        Self::new(x, y, z)
    }
}

impl From<[f64; 3]> for Vector3 {
    fn from(arr: [f64; 3]) -> Self {
        Self::new(arr[0], arr[1], arr[2])
    }
}

impl From<Vector3> for (f64, f64, f64) {
    fn from(v: Vector3) -> Self {
        (v.x, v.y, v.z)
    }
}

impl From<Vector3> for [f64; 3] {
    fn from(v: Vector3) -> Self {
        [v.x, v.y, v.z]
    }
}

impl From<WorldCoord> for Vector3 {
    fn from(wc: WorldCoord) -> Self {
        Self::new(wc.x as f64, wc.y as f64, wc.z as f64)
    }
}

impl From<BlockPos> for Vector3 {
    fn from(pos: BlockPos) -> Self {
        Self::new(pos.0 as f64, pos.1 as f64, pos.2 as f64)
    }
}

// ============================================================================
// WorldCoord: Block Coordinate Spatial Navigation
// ============================================================================

/// 3D integer block coordinate in world space.
///
/// Port of `com.eloraam.redpower.core.WorldCoord`.
/// Uses the canonical RedPower orientation mapping `0..=5`:
/// - `0`: `-Y` (Down)
/// - `1`: `+Y` (Up)
/// - `2`: `-Z` (North)
/// - `3`: `+Z` (South)
/// - `4`: `-X` (West)
/// - `5`: `+X` (East)
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct WorldCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl WorldCoord {
    /// Origin coordinate `(0, 0, 0)`.
    pub const ZERO: Self = Self::new(0, 0, 0);

    /// Construct a new coordinate at `(x, y, z)`.
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Set coordinates from another [`WorldCoord`].
    pub fn set(&mut self, wc: &Self) {
        self.x = wc.x;
        self.y = wc.y;
        self.z = wc.z;
    }

    /// Set raw coordinate values.
    pub fn set_coords(&mut self, x: i32, y: i32, z: i32) {
        self.x = x;
        self.y = y;
        self.z = z;
    }

    /// Return a copy stepped 1 block in direction `dir` (`0..5`).
    ///
    /// Matches `WorldCoord#coordStep`:
    /// - `0`: `(x, y - 1, z)`
    /// - `1`: `(x, y + 1, z)`
    /// - `2`: `(x, y, z - 1)`
    /// - `3`: `(x, y, z + 1)`
    /// - `4`: `(x - 1, y, z)`
    /// - default (`5`): `(x + 1, y, z)`
    pub const fn coord_step(&self, dir: i32) -> Self {
        match dir {
            0 => Self::new(self.x, self.y - 1, self.z),
            1 => Self::new(self.x, self.y + 1, self.z),
            2 => Self::new(self.x, self.y, self.z - 1),
            3 => Self::new(self.x, self.y, self.z + 1),
            4 => Self::new(self.x - 1, self.y, self.z),
            _ => Self::new(self.x + 1, self.y, self.z),
        }
    }

    /// Return a copy stepped 1 block along [`Direction`].
    pub const fn coord_step_dir(&self, dir: Direction) -> Self {
        self.coord_step(dir as i32)
    }

    /// Return a copy stepped `dist` blocks in direction `dir` (`0..5`).
    pub const fn coord_step_dist(&self, dir: i32, dist: i32) -> Self {
        match dir {
            0 => Self::new(self.x, self.y - dist, self.z),
            1 => Self::new(self.x, self.y + dist, self.z),
            2 => Self::new(self.x, self.y, self.z - dist),
            3 => Self::new(self.x, self.y, self.z + dist),
            4 => Self::new(self.x - dist, self.y, self.z),
            _ => Self::new(self.x + dist, self.y, self.z),
        }
    }

    /// Return a copy stepped `dist` blocks along [`Direction`].
    pub const fn coord_step_dir_dist(&self, dir: Direction, dist: i32) -> Self {
        self.coord_step_dist(dir as i32, dist)
    }

    /// In-place step 1 block in direction `dir` (`0..5`).
    ///
    /// Matches `WorldCoord#step(int dir)`:
    /// - `0`: `y--`
    /// - `1`: `y++`
    /// - `2`: `z--`
    /// - `3`: `z++`
    /// - `4`: `x--`
    /// - default: `x++`
    pub fn step(&mut self, dir: i32) {
        match dir {
            0 => self.y -= 1,
            1 => self.y += 1,
            2 => self.z -= 1,
            3 => self.z += 1,
            4 => self.x -= 1,
            _ => self.x += 1,
        }
    }

    /// In-place step 1 block along [`Direction`].
    pub fn step_dir(&mut self, dir: Direction) {
        self.step(dir as i32);
    }

    /// In-place step `dist` blocks in direction `dir` (`0..5`).
    ///
    /// Matches `WorldCoord#step(int dir, int dist)`.
    pub fn step_dist(&mut self, dir: i32, dist: i32) {
        match dir {
            0 => self.y -= dist,
            1 => self.y += dist,
            2 => self.z -= dist,
            3 => self.z += dist,
            4 => self.x -= dist,
            _ => self.x += dist,
        }
    }

    /// In-place step `dist` blocks along [`Direction`].
    pub fn step_dir_dist(&mut self, dir: Direction, dist: i32) {
        self.step_dist(dir as i32, dist);
    }

    /// Euclidean distance squared to raw target coordinates:
    /// `(xi - x)^2 + (yi - y)^2 + (zi - z)^2`.
    ///
    /// Matches `WorldCoord#squareDist`.
    pub const fn square_dist(&self, xi: i32, yi: i32, zi: i32) -> i32 {
        let dx = xi - self.x;
        let dy = yi - self.y;
        let dz = zi - self.z;
        dx * dx + dy * dy + dz * dz
    }

    /// Euclidean distance squared to another [`WorldCoord`].
    pub const fn square_dist_coord(&self, other: Self) -> i32 {
        self.square_dist(other.x, other.y, other.z)
    }

    /// 90-degree right rotation for horizontal directions.
    ///
    /// Matches `WorldCoord#getRightDir`:
    /// - `< 2`: returns `dir` (`0 -> 0`, `1 -> 1`)
    /// - `2` (North / -Z): `4` (West / -X)
    /// - `3` (South / +Z): `5` (East / +X)
    /// - `4` (West / -X): `3` (South / +Z)
    /// - default (`5`, East / +X): `2` (North / -Z)
    pub const fn get_right_dir(dir: i32) -> i32 {
        if dir < 2 {
            return dir;
        }
        match dir {
            0 => 0,
            1 => 1,
            2 => 4,
            3 => 5,
            4 => 3,
            _ => 2,
        }
    }

    /// 90-degree right rotation for [`Direction`].
    pub const fn get_right_direction(dir: Direction) -> Direction {
        match dir {
            Direction::Down => Direction::Down,
            Direction::Up => Direction::Up,
            Direction::North => Direction::West,
            Direction::South => Direction::East,
            Direction::West => Direction::South,
            Direction::East => Direction::North,
        }
    }

    /// Map primary direction `d1` (`0..5`) and corner index `d2` (`0..3`) to an orthogonal step direction.
    ///
    /// Matches `WorldCoord#getIndStepDir`:
    /// - `d1 in (0, 1)`: `d2 + 2` (steps across horizontal directions 2, 3, 4, 5)
    /// - `d1 in (2, 3)`: `d2 + (d2 & 2)` (steps across 0, 1, 4, 5)
    /// - `d1 in (4, 5)`: `d2` (steps across 0, 1, 2, 3)
    pub const fn get_ind_step_dir(d1: i32, d2: i32) -> i32 {
        match d1 {
            0 | 1 => d2 + 2,
            2 | 3 => d2 + (d2 & 2),
            _ => d2,
        }
    }

    /// Indirect corner step: steps along primary direction `d1`, then steps along orthogonal `getIndStepDir(d1, d2)`.
    ///
    /// Matches `WorldCoord#indStep`. Used in wire wrapping and tube pathfinding.
    pub fn ind_step(&mut self, d1: i32, d2: i32) {
        self.step(d1);
        self.step(Self::get_ind_step_dir(d1, d2));
    }

    /// Non-mutating indirect corner step.
    pub const fn ind_step_coord(&self, d1: i32, d2: i32) -> Self {
        let first = self.coord_step(d1);
        first.coord_step(Self::get_ind_step_dir(d1, d2))
    }

    /// Directional ordering comparison between two coordinates along axis `dir`.
    ///
    /// Matches `WorldCoord.WCComparator#compare`:
    /// - `0` (-Y): `a.y.cmp(&b.y)` (ascending Y)
    /// - `1` (+Y): `b.y.cmp(&a.y)` (descending Y)
    /// - `2` (-Z): `a.z.cmp(&b.z)` (ascending Z)
    /// - `3` (+Z): `b.z.cmp(&a.z)` (descending Z)
    /// - `4` (-X): `a.x.cmp(&b.x)` (ascending X)
    /// - default (`5`, +X): `b.x.cmp(&a.x)` (descending X)
    pub fn compare_dir(dir: i32, a: &Self, b: &Self) -> Ordering {
        match dir {
            0 => a.y.cmp(&b.y),
            1 => b.y.cmp(&a.y),
            2 => a.z.cmp(&b.z),
            3 => b.z.cmp(&a.z),
            4 => a.x.cmp(&b.x),
            _ => b.x.cmp(&a.x),
        }
    }

    /// Total directional comparison: compares along axis `dir`, tie-breaking with natural `Ord`.
    pub fn compare_dir_total(dir: i32, a: &Self, b: &Self) -> Ordering {
        Self::compare_dir(dir, a, b).then_with(|| a.cmp(b))
    }

    /// Return a closure comparator suitable for `slice::sort_by`.
    pub fn dir_comparator(dir: i32) -> impl Fn(&Self, &Self) -> Ordering {
        move |a, b| Self::compare_dir(dir, a, b)
    }

    /// Java-compatible hash code calculation: `x + 31 * (y + 31 * z)`.
    pub fn java_hash_code(&self) -> i32 {
        self.x
            .wrapping_add(31i32.wrapping_mul(self.y.wrapping_add(31i32.wrapping_mul(self.z))))
    }

    /// Convert to [`BlockPos`].
    pub const fn to_block_pos(&self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }

    /// Convert from [`BlockPos`].
    pub const fn from_block_pos(pos: BlockPos) -> Self {
        Self::new(pos.0, pos.1, pos.2)
    }

    /// Convert to [`Vector3`] pointing to block corner.
    pub fn to_vector3(&self) -> Vector3 {
        Vector3::new(self.x as f64, self.y as f64, self.z as f64)
    }

    /// Convert to [`Vector3`] pointing to block center `(x + 0.5, y + 0.5, z + 0.5)`.
    pub fn to_vector3_centered(&self) -> Vector3 {
        Vector3::new(
            self.x as f64 + 0.5,
            self.y as f64 + 0.5,
            self.z as f64 + 0.5,
        )
    }
}

impl fmt::Display for WorldCoord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WorldCoord({}, {}, {})", self.x, self.y, self.z)
    }
}

impl Add for WorldCoord {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl AddAssign for WorldCoord {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for WorldCoord {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl SubAssign for WorldCoord {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl From<(i32, i32, i32)> for WorldCoord {
    fn from((x, y, z): (i32, i32, i32)) -> Self {
        Self::new(x, y, z)
    }
}

impl From<[i32; 3]> for WorldCoord {
    fn from(arr: [i32; 3]) -> Self {
        Self::new(arr[0], arr[1], arr[2])
    }
}

impl From<WorldCoord> for (i32, i32, i32) {
    fn from(wc: WorldCoord) -> Self {
        (wc.x, wc.y, wc.z)
    }
}

impl From<WorldCoord> for [i32; 3] {
    fn from(wc: WorldCoord) -> Self {
        [wc.x, wc.y, wc.z]
    }
}

impl From<BlockPos> for WorldCoord {
    fn from(pos: BlockPos) -> Self {
        Self::new(pos.0, pos.1, pos.2)
    }
}

impl From<WorldCoord> for BlockPos {
    fn from(wc: WorldCoord) -> Self {
        BlockPos::new(wc.x, wc.y, wc.z)
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector3_basic_arithmetic() {
        let mut v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::new(4.0, 5.0, 6.0);

        // Dot product: 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
        assert_eq!(v1.dot_product(&v2), 32.0);
        assert_eq!(v1.dot_product_coords(4.0, 5.0, 6.0), 32.0);

        // Addition
        v1.add_vec(v2);
        assert_eq!(v1, Vector3::new(5.0, 7.0, 9.0));

        // Subtraction
        v1.sub_vec(v2);
        assert_eq!(v1, Vector3::new(1.0, 2.0, 3.0));

        // AddAssign / SubAssign
        v1 += v2;
        assert_eq!(v1, Vector3::new(5.0, 7.0, 9.0));
        v1 -= v2;
        assert_eq!(v1, Vector3::new(1.0, 2.0, 3.0));

        // Multiplication
        v1.multiply(2.0);
        assert_eq!(v1, Vector3::new(2.0, 4.0, 6.0));

        // Magnitude
        assert_eq!(v1.mag_squared(), 4.0 + 16.0 + 36.0); // 56.0
        assert!((v1.mag() - 56.0f64.sqrt()).abs() < 1e-9);

        // Operator traits
        let sum = v1 + v2;
        assert_eq!(sum, Vector3::new(6.0, 9.0, 12.0));
        let diff = sum - v2;
        assert_eq!(diff, v1);
        let scaled = v1 * 0.5;
        assert_eq!(scaled, Vector3::new(1.0, 2.0, 3.0));
        let div = scaled / 2.0;
        assert_eq!(div, Vector3::new(0.5, 1.0, 1.5));
        let neg = -v1;
        assert_eq!(neg, Vector3::new(-2.0, -4.0, -6.0));
    }

    #[test]
    fn test_vector3_cross_product() {
        let x = Vector3::UNIT_X;
        let y = Vector3::UNIT_Y;

        // X x Y = Z
        let z = x.cross(&y);
        assert_eq!(z, Vector3::UNIT_Z);

        // In-place cross product
        let mut v = x;
        v.cross_product(&y);
        assert_eq!(v, Vector3::UNIT_Z);

        // Y x Z = X
        assert_eq!(y.cross(&z), Vector3::UNIT_X);

        // Z x X = Y
        assert_eq!(z.cross(&x), Vector3::UNIT_Y);

        // Y x X = -Z
        assert_eq!(y.cross(&x), -Vector3::UNIT_Z);
    }

    #[test]
    fn test_vector3_normalization() {
        let mut v = Vector3::new(3.0, 0.0, 4.0);
        assert_eq!(v.mag(), 5.0);
        v.normalize();
        assert!((v.x - 0.6).abs() < 1e-9);
        assert_eq!(v.y, 0.0);
        assert!((v.z - 0.8).abs() < 1e-9);
        assert!((v.mag() - 1.0).abs() < 1e-9);

        // Normalizing zero vector is a safe no-op
        let mut zero = Vector3::ZERO;
        zero.normalize();
        assert_eq!(zero, Vector3::ZERO);
    }

    #[test]
    fn test_vector3_mdist_and_formatting() {
        let a = Vector3::new(1.2, 3.8, -2.1);
        let b = Vector3::new(4.1, 1.2, 0.5);
        // a floored: (1, 3, -3)
        // b floored: (4, 1, 0)
        // mdist: |1-4| + |3-1| + |-3-0| = 3 + 2 + 3 = 8
        assert_eq!(a.mdist(&b), 8);

        let s = format!("{}", Vector3::new(1.0, 2.0, 3.0));
        assert!(s.contains("Vector:\n  < 1.000000 2.000000 3.000000 >"));
    }

    #[test]
    fn test_worldcoord_stepping() {
        let origin = WorldCoord::new(10, 20, 30);

        // Step in each canonical direction:
        // 0: -Y
        assert_eq!(origin.coord_step(0), WorldCoord::new(10, 19, 30));
        // 1: +Y
        assert_eq!(origin.coord_step(1), WorldCoord::new(10, 21, 30));
        // 2: -Z
        assert_eq!(origin.coord_step(2), WorldCoord::new(10, 20, 29));
        // 3: +Z
        assert_eq!(origin.coord_step(3), WorldCoord::new(10, 20, 31));
        // 4: -X
        assert_eq!(origin.coord_step(4), WorldCoord::new(9, 20, 30));
        // 5: +X
        assert_eq!(origin.coord_step(5), WorldCoord::new(11, 20, 30));

        // Step dist
        assert_eq!(origin.coord_step_dist(0, 5), WorldCoord::new(10, 15, 30));
        assert_eq!(origin.coord_step_dist(5, 5), WorldCoord::new(15, 20, 30));

        // In-place steps
        let mut wc = origin;
        wc.step(1);
        wc.step(3);
        wc.step(5);
        assert_eq!(wc, WorldCoord::new(11, 21, 31));

        wc.step_dist(0, 2);
        wc.step_dist(2, 2);
        wc.step_dist(4, 2);
        assert_eq!(wc, WorldCoord::new(9, 19, 29));
    }

    #[test]
    fn test_worldcoord_direction_enum_stepping() {
        let c = WorldCoord::new(0, 10, 0);
        assert_eq!(c.coord_step_dir(Direction::Down), WorldCoord::new(0, 9, 0));
        assert_eq!(c.coord_step_dir(Direction::Up), WorldCoord::new(0, 11, 0));
        assert_eq!(c.coord_step_dir(Direction::North), WorldCoord::new(0, 10, -1));
        assert_eq!(c.coord_step_dir(Direction::South), WorldCoord::new(0, 10, 1));
        assert_eq!(c.coord_step_dir(Direction::West), WorldCoord::new(-1, 10, 0));
        assert_eq!(c.coord_step_dir(Direction::East), WorldCoord::new(1, 10, 0));
    }

    #[test]
    fn test_worldcoord_square_dist() {
        let a = WorldCoord::new(1, 2, 3);
        let b = WorldCoord::new(4, 6, 3);
        // (4-1)^2 + (6-2)^2 + (3-3)^2 = 9 + 16 + 0 = 25
        assert_eq!(a.square_dist(4, 6, 3), 25);
        assert_eq!(a.square_dist_coord(b), 25);
    }

    #[test]
    fn test_worldcoord_right_dir() {
        // Vertical returns unmodified
        assert_eq!(WorldCoord::get_right_dir(0), 0);
        assert_eq!(WorldCoord::get_right_dir(1), 1);

        // Horizontal rotation:
        // 2 (North) -> 4 (West)
        assert_eq!(WorldCoord::get_right_dir(2), 4);
        // 4 (West) -> 3 (South)
        assert_eq!(WorldCoord::get_right_dir(4), 3);
        // 3 (South) -> 5 (East)
        assert_eq!(WorldCoord::get_right_dir(3), 5);
        // 5 (East) -> 2 (North)
        assert_eq!(WorldCoord::get_right_dir(5), 2);
    }

    #[test]
    fn test_worldcoord_ind_step() {
        // Primary direction 0 (-Y / Down):
        // d2 in 0..3 maps to d2 + 2 = 2, 3, 4, 5
        assert_eq!(WorldCoord::get_ind_step_dir(0, 0), 2); // North
        assert_eq!(WorldCoord::get_ind_step_dir(0, 1), 3); // South
        assert_eq!(WorldCoord::get_ind_step_dir(0, 2), 4); // West
        assert_eq!(WorldCoord::get_ind_step_dir(0, 3), 5); // East

        // Primary direction 2 (-Z / North):
        // d2 + (d2 & 2):
        // 0 -> 0 (Down)
        // 1 -> 1 (Up)
        // 2 -> 4 (West)
        // 3 -> 5 (East)
        assert_eq!(WorldCoord::get_ind_step_dir(2, 0), 0);
        assert_eq!(WorldCoord::get_ind_step_dir(2, 1), 1);
        assert_eq!(WorldCoord::get_ind_step_dir(2, 2), 4);
        assert_eq!(WorldCoord::get_ind_step_dir(2, 3), 5);

        // Primary direction 4 (-X / West):
        // returns d2: 0, 1, 2, 3
        assert_eq!(WorldCoord::get_ind_step_dir(4, 0), 0);
        assert_eq!(WorldCoord::get_ind_step_dir(4, 1), 1);
        assert_eq!(WorldCoord::get_ind_step_dir(4, 2), 2);
        assert_eq!(WorldCoord::get_ind_step_dir(4, 3), 3);

        // Full ind_step simulation from origin (0, 0, 0):
        // Step Down (0), then East (d2 = 3 -> 5):
        let stepped = WorldCoord::ZERO.ind_step_coord(0, 3);
        assert_eq!(stepped, WorldCoord::new(1, -1, 0));
    }

    #[test]
    fn test_worldcoord_sorting_and_comparator() {
        let coords = [
            WorldCoord::new(5, 10, 2),
            WorldCoord::new(2, 20, 8),
            WorldCoord::new(8, 5, 4),
        ];

        // Sort dir 0 (-Y, ascending Y): 5, then 10, then 20
        let mut sorted = coords;
        sorted.sort_by(WorldCoord::dir_comparator(0));
        assert_eq!(sorted[0].y, 5);
        assert_eq!(sorted[1].y, 10);
        assert_eq!(sorted[2].y, 20);

        // Sort dir 1 (+Y, descending Y): 20, then 10, then 5
        sorted.sort_by(WorldCoord::dir_comparator(1));
        assert_eq!(sorted[0].y, 20);
        assert_eq!(sorted[1].y, 10);
        assert_eq!(sorted[2].y, 5);

        // Sort dir 4 (-X, ascending X): 2, then 5, then 8
        sorted.sort_by(WorldCoord::dir_comparator(4));
        assert_eq!(sorted[0].x, 2);
        assert_eq!(sorted[1].x, 5);
        assert_eq!(sorted[2].x, 8);

        // Sort dir 5 (+X, descending X): 8, then 5, then 2
        sorted.sort_by(WorldCoord::dir_comparator(5));
        assert_eq!(sorted[0].x, 8);
        assert_eq!(sorted[1].x, 5);
        assert_eq!(sorted[2].x, 2);
    }

    #[test]
    fn test_worldcoord_conversions_and_hash() {
        let wc = WorldCoord::new(1, 2, 3);
        let bp = wc.to_block_pos();
        assert_eq!(bp, BlockPos::new(1, 2, 3));
        assert_eq!(WorldCoord::from_block_pos(bp), wc);

        let v3 = wc.to_vector3();
        assert_eq!(v3, Vector3::new(1.0, 2.0, 3.0));

        let v3_center = wc.to_vector3_centered();
        assert_eq!(v3_center, Vector3::new(1.5, 2.5, 3.5));

        // Java hash code: c1 + 31 * (c2 + 31 * c3) = 1 + 31 * (2 + 31 * 3) = 1 + 31 * (95) = 2946
        assert_eq!(wc.java_hash_code(), 1 + 31 * (2 + 31 * 3));
    }
}
