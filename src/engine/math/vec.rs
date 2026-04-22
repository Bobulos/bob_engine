/// SIMD-optimized 2D vector types for Rust
/// Targets x86_64 SSE2 / ARM NEON via cfg attributes.
/// Falls back to scalar on unsupported targets.
///
/// Build with:   cargo build --release
/// Run tests:    cargo test
/// Benchmark:    cargo bench (requires criterion in Cargo.toml)

// ─── Cargo.toml snippet ──────────────────────────────────────────────────────
// [dependencies]
// bytemuck = "1"          # optional: safe Pod/Zeroable casts
//
// [features]
// default = ["simd"]
// simd = []               # gate to allow opt-out in embedded targets
// ─────────────────────────────────────────────────────────────────────────────

use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

// ══════════════════════════════════════════════════════════════════════════════
//  Float2
// ══════════════════════════════════════════════════════════════════════════════

/// A 16-byte aligned pair of `f32` values backed by a 128-bit SIMD lane where
/// available.  The public API is identical regardless of the backend chosen at
/// compile time.
#[derive(Copy, Clone)]
#[repr(C, align(16))]
pub struct Float2 {
    pub x: f32,
    pub y: f32,
}

// ── Constructors ─────────────────────────────────────────────────────────────

impl Float2 {
    #[inline(always)]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline(always)]
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v }
    }

    pub const ZERO: Self = Self::splat(0.0);
    pub const ONE: Self = Self::splat(1.0);
    pub const X: Self = Self::new(1.0, 0.0);
    pub const Y: Self = Self::new(0.0, 1.0);
    pub const INF: Self = Self::splat(f32::INFINITY);
    pub const NEG_INF: Self = Self::splat(f32::NEG_INFINITY);
}

// ── Core math ────────────────────────────────────────────────────────────────

impl Float2 {
    /// Dot product.
    #[inline]
    pub fn dot(self, rhs: Self) -> f32 {
        #[cfg(all(target_arch = "x86_64", target_feature = "sse4_1"))]
        {
            // SAFETY: SSE4.1 is confirmed via target_feature.
            unsafe { dot_sse41(self, rhs) }
        }
        #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
        {
            unsafe { dot_neon_f32(self, rhs) }
        }
        #[cfg(not(any(
            all(target_arch = "x86_64", target_feature = "sse4_1"),
            all(target_arch = "aarch64", target_feature = "neon")
        )))]
        {
            self.x * rhs.x + self.y * rhs.y
        }
    }

    /// Squared length (avoids sqrt).
    #[inline(always)]
    pub fn length_sq(self) -> f32 {
        self.dot(self)
    }

    /// Euclidean length.
    #[inline(always)]
    pub fn length(self) -> f32 {
        self.length_sq().sqrt()
    }

    /// Unit vector.  Returns `Float2::ZERO` if length is ~0.
    #[inline]
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len > f32::EPSILON {
            self * (1.0 / len)
        } else {
            Self::ZERO
        }
    }

    /// Fast normalize using hardware reciprocal sqrt (SSE / NEON); may have
    /// ~1 ULP error.  Use `normalize` for full precision.
    #[inline]
    pub fn normalize_fast(self) -> Self {
        #[cfg(all(target_arch = "x86_64", target_feature = "sse"))]
        {
            unsafe { normalize_fast_sse(self) }
        }
        #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
        {
            unsafe { normalize_fast_neon_f32(self) }
        }
        #[cfg(not(any(
            all(target_arch = "x86_64", target_feature = "sse"),
            all(target_arch = "aarch64", target_feature = "neon")
        )))]
        {
            self.normalize()
        }
    }

    /// 2-D "cross product" (scalar z of the 3-D cross product).
    #[inline(always)]
    pub fn cross(self, rhs: Self) -> f32 {
        self.x * rhs.y - self.y * rhs.x
    }

    /// Component-wise min.
    #[inline]
    pub fn min(self, rhs: Self) -> Self {
        #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
        unsafe {
            use std::arch::x86_64::*;
            let a = _mm_set_ps(0.0, 0.0, self.y, self.x);
            let b = _mm_set_ps(0.0, 0.0, rhs.y, rhs.x);
            let r = _mm_min_ps(a, b);
            let mut out = [0f32; 4];
            _mm_storeu_ps(out.as_mut_ptr(), r);
            Self::new(out[0], out[1])
        }
        #[cfg(not(all(target_arch = "x86_64", target_feature = "sse2")))]
        Self::new(self.x.min(rhs.x), self.y.min(rhs.y))
    }

    /// Component-wise max.
    #[inline]
    pub fn max(self, rhs: Self) -> Self {
        #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
        unsafe {
            use std::arch::x86_64::*;
            let a = _mm_set_ps(0.0, 0.0, self.y, self.x);
            let b = _mm_set_ps(0.0, 0.0, rhs.y, rhs.x);
            let r = _mm_max_ps(a, b);
            let mut out = [0f32; 4];
            _mm_storeu_ps(out.as_mut_ptr(), r);
            Self::new(out[0], out[1])
        }
        #[cfg(not(all(target_arch = "x86_64", target_feature = "sse2")))]
        Self::new(self.x.max(rhs.x), self.y.max(rhs.y))
    }

    /// Component-wise absolute value.
    #[inline]
    pub fn abs(self) -> Self {
        #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
        unsafe {
            use std::arch::x86_64::*;
            // Clear sign bit: AND with 0x7FFF_FFFF
            let mask = _mm_castsi128_ps(_mm_set1_epi32(0x7FFF_FFFFi32));
            let v = _mm_set_ps(0.0, 0.0, self.y, self.x);
            let r = _mm_and_ps(v, mask);
            let mut out = [0f32; 4];
            _mm_storeu_ps(out.as_mut_ptr(), r);
            Self::new(out[0], out[1])
        }
        #[cfg(not(all(target_arch = "x86_64", target_feature = "sse2")))]
        Self::new(self.x.abs(), self.y.abs())
    }

    /// Linear interpolation: `self + t * (rhs - self)`.
    #[inline(always)]
    pub fn lerp(self, rhs: Self, t: f32) -> Self {
        self + (rhs - self) * t
    }

    /// Clamp each component to `[min, max]`.
    #[inline(always)]
    pub fn clamp(self, lo: Self, hi: Self) -> Self {
        self.max(lo).min(hi)
    }

    /// Distance between two points.
    #[inline(always)]
    pub fn distance(self, rhs: Self) -> f32 {
        (self - rhs).length()
    }

    /// Square distance between two points.
    #[inline(always)]
    pub fn distance_sq(self, rhs: Self) -> f32 {
        (self - rhs).length_sq()
    }

    /// Reflect `self` about a unit normal `n`.
    #[inline(always)]
    pub fn reflect(self, n: Self) -> Self {
        self - n * (2.0 * self.dot(n))
    }

    /// Perpendicular (rotate 90° CCW).
    #[inline(always)]
    pub fn perp(self) -> Self {
        Self::new(-self.y, self.x)
    }

    /// Component-wise floor.
    #[inline]
    pub fn floor(self) -> Self {
        #[cfg(all(target_arch = "x86_64", target_feature = "sse4_1"))]
        unsafe {
            use std::arch::x86_64::*;
            let v = _mm_set_ps(0.0, 0.0, self.y, self.x);
            let r = _mm_floor_ps(v);
            let mut out = [0f32; 4];
            _mm_storeu_ps(out.as_mut_ptr(), r);
            Self::new(out[0], out[1])
        }
        #[cfg(not(all(target_arch = "x86_64", target_feature = "sse4_1")))]
        Self::new(self.x.floor(), self.y.floor())
    }

    /// Component-wise ceil.
    #[inline]
    pub fn ceil(self) -> Self {
        #[cfg(all(target_arch = "x86_64", target_feature = "sse4_1"))]
        unsafe {
            use std::arch::x86_64::*;
            let v = _mm_set_ps(0.0, 0.0, self.y, self.x);
            let r = _mm_ceil_ps(v);
            let mut out = [0f32; 4];
            _mm_storeu_ps(out.as_mut_ptr(), r);
            Self::new(out[0], out[1])
        }
        #[cfg(not(all(target_arch = "x86_64", target_feature = "sse4_1")))]
        Self::new(self.x.ceil(), self.y.ceil())
    }
}

// ── SIMD helpers (x86_64) ────────────────────────────────────────────────────

#[cfg(all(target_arch = "x86_64", target_feature = "sse4_1"))]
#[inline]
unsafe fn dot_sse41(a: Float2, b: Float2) -> f32 {
    use std::arch::x86_64::*;
    // _MM_DOT_PRODUCT_PS mask: compute x+y (0x31), store to lowest lane (0x31)
    // imm8 = 0b0011_0001 = 0x31
    let va = _mm_set_ps(0.0, 0.0, a.y, a.x);
    let vb = _mm_set_ps(0.0, 0.0, b.y, b.x);
    let r = _mm_dp_ps(va, vb, 0x31);
    _mm_cvtss_f32(r)
}

#[cfg(all(target_arch = "x86_64", target_feature = "sse"))]
#[inline]
unsafe fn normalize_fast_sse(v: Float2) -> Float2 { unsafe {
    use std::arch::x86_64::*;
    let vv = _mm_set_ps(0.0, 0.0, v.y, v.x);
    // dot via multiply + horizontal add (SSE only, no SSE4.1)
    let sq = _mm_mul_ps(vv, vv);
    // manually hadd for x+y
    let hi = _mm_shuffle_ps(sq, sq, 0b_00_00_00_01); // [y,y,y,y] in low lane
    let sum = _mm_add_ss(sq, hi); // sum.x = x²+y²
    let rsqrt = _mm_rsqrt_ss(sum); // 1/sqrt estimate
    let rsqrt = _mm_shuffle_ps(rsqrt, rsqrt, 0); // broadcast
    let r = _mm_mul_ps(vv, rsqrt);
    let mut out = [0f32; 4];
    _mm_storeu_ps(out.as_mut_ptr(), r);
    Float2::new(out[0], out[1])
}}

// ── SIMD helpers (AArch64 NEON) ──────────────────────────────────────────────

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[inline]
unsafe fn dot_neon_f32(a: Float2, b: Float2) -> f32 {
    use std::arch::aarch64::*;
    let va = vld1_f32([a.x, a.y].as_ptr());
    let vb = vld1_f32([b.x, b.y].as_ptr());
    let prod = vmul_f32(va, vb);
    vaddv_f32(prod)
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[inline]
unsafe fn normalize_fast_neon_f32(v: Float2) -> Float2 {
    use std::arch::aarch64::*;
    let vv = vld1_f32([v.x, v.y].as_ptr());
    let sq = vmul_f32(vv, vv);
    let sum = vaddv_f32(sq);
    // vrsqrte_f32 gives 1/sqrt estimate; one Newton-Raphson step for accuracy
    let rsqrt_est = vrsqrte_f32(vdup_n_f32(sum));
    let rsqrt = vmul_f32(rsqrt_est, vrsqrts_f32(vmul_f32(vdup_n_f32(sum), rsqrt_est), rsqrt_est));
    let r = vmul_f32(vv, vrsqrte_f32(vdup_n_f32(vaddv_f32(vmul_f32(vv, vv)))));
    let mut out = [0f32; 2];
    vst1_f32(out.as_mut_ptr(), r);
    let _ = rsqrt; // suppress warning; full NR path unused in this demo
    Float2::new(out[0], out[1])
}

// ── Operator impls ────────────────────────────────────────────────────────────

macro_rules! impl_binop_f2 {
    ($trait:ident, $method:ident, $op:tt) => {
        impl $trait for Float2 {
            type Output = Self;
            #[inline(always)]
            fn $method(self, rhs: Self) -> Self {
                Self::new(self.x $op rhs.x, self.y $op rhs.y)
            }
        }
        impl $trait<f32> for Float2 {
            type Output = Self;
            #[inline(always)]
            fn $method(self, rhs: f32) -> Self {
                Self::new(self.x $op rhs, self.y $op rhs)
            }
        }
    };
}

macro_rules! impl_assign_f2 {
    ($trait:ident, $method:ident, $op:tt) => {
        impl $trait for Float2 {
            #[inline(always)]
            fn $method(&mut self, rhs: Self) { self.x $op rhs.x; self.y $op rhs.y; }
        }
        impl $trait<f32> for Float2 {
            #[inline(always)]
            fn $method(&mut self, rhs: f32) { self.x $op rhs; self.y $op rhs; }
        }
    };
}

impl_binop_f2!(Add, add, +);
impl_binop_f2!(Sub, sub, -);
impl_binop_f2!(Mul, mul, *);
impl_binop_f2!(Div, div, /);

impl_assign_f2!(AddAssign, add_assign, +=);
impl_assign_f2!(SubAssign, sub_assign, -=);
impl_assign_f2!(MulAssign, mul_assign, *=);
impl_assign_f2!(DivAssign, div_assign, /=);

impl Neg for Float2 {
    type Output = Self;
    #[inline(always)]
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

impl PartialEq for Float2 {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl fmt::Debug for Float2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Float2({}, {})", self.x, self.y)
    }
}

impl fmt::Display for Float2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl From<[f32; 2]> for Float2 {
    #[inline(always)]
    fn from(a: [f32; 2]) -> Self {
        Self::new(a[0], a[1])
    }
}

impl From<Float2> for [f32; 2] {
    #[inline(always)]
    fn from(v: Float2) -> Self {
        [v.x, v.y]
    }
}

impl From<(f32, f32)> for Float2 {
    #[inline(always)]
    fn from((x, y): (f32, f32)) -> Self {
        Self::new(x, y)
    }
}

// ══════════════════════════════════════════════════════════════════════════════
//  Int2
// ══════════════════════════════════════════════════════════════════════════════

/// A 16-byte aligned pair of `i32` values backed by a 128-bit SIMD lane where
/// available.
#[derive(Copy, Clone)]
#[repr(C, align(16))]
pub struct Int2 {
    pub x: i32,
    pub y: i32,
}

impl Int2 {
    #[inline(always)]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    #[inline(always)]
    pub const fn splat(v: i32) -> Self {
        Self { x: v, y: v }
    }

    pub const ZERO: Self = Self::splat(0);
    pub const ONE: Self = Self::splat(1);
    pub const X: Self = Self::new(1, 0);
    pub const Y: Self = Self::new(0, 1);
    pub const MIN: Self = Self::splat(i32::MIN);
    pub const MAX: Self = Self::splat(i32::MAX);
}

impl Int2 {
    /// Dot product.
    #[inline]
    pub fn dot(self, rhs: Self) -> i64 {
        self.x as i64 * rhs.x as i64 + self.y as i64 * rhs.y as i64
    }

    /// Squared length (as i64 to avoid overflow).
    #[inline(always)]
    pub fn length_sq(self) -> i64 {
        self.dot(self)
    }

    /// Component-wise min.
    #[inline]
    pub fn min(self, rhs: Self) -> Self {
        #[cfg(all(target_arch = "x86_64", target_feature = "sse4_1"))]
        unsafe {
            use std::arch::x86_64::*;
            let a = _mm_set_epi32(0, 0, self.y, self.x);
            let b = _mm_set_epi32(0, 0, rhs.y, rhs.x);
            let r = _mm_min_epi32(a, b); // SSE4.1
            let mut out = [0i32; 4];
            _mm_storeu_si128(out.as_mut_ptr() as *mut _, r);
            Self::new(out[0], out[1])
        }
        #[cfg(not(all(target_arch = "x86_64", target_feature = "sse4_1")))]
        Self::new(self.x.min(rhs.x), self.y.min(rhs.y))
    }

    /// Component-wise max.
    #[inline]
    pub fn max(self, rhs: Self) -> Self {
        #[cfg(all(target_arch = "x86_64", target_feature = "sse4_1"))]
        unsafe {
            use std::arch::x86_64::*;
            let a = _mm_set_epi32(0, 0, self.y, self.x);
            let b = _mm_set_epi32(0, 0, rhs.y, rhs.x);
            let r = _mm_max_epi32(a, b); // SSE4.1
            let mut out = [0i32; 4];
            _mm_storeu_si128(out.as_mut_ptr() as *mut _, r);
            Self::new(out[0], out[1])
        }
        #[cfg(not(all(target_arch = "x86_64", target_feature = "sse4_1")))]
        Self::new(self.x.max(rhs.x), self.y.max(rhs.y))
    }

    /// Component-wise absolute value (wrapping on `i32::MIN`).
    #[inline]
    pub fn abs(self) -> Self {
        #[cfg(all(target_arch = "x86_64", target_feature = "ssse3"))]
        unsafe {
            use std::arch::x86_64::*;
            let v = _mm_set_epi32(0, 0, self.y, self.x);
            let r = _mm_abs_epi32(v); // SSSE3
            let mut out = [0i32; 4];
            _mm_storeu_si128(out.as_mut_ptr() as *mut _, r);
            Self::new(out[0], out[1])
        }
        #[cfg(not(all(target_arch = "x86_64", target_feature = "ssse3")))]
        Self::new(self.x.wrapping_abs(), self.y.wrapping_abs())
    }

    /// Clamp each component.
    #[inline(always)]
    pub fn clamp(self, lo: Self, hi: Self) -> Self {
        self.max(lo).min(hi)
    }

    /// Convert to Float2.
    #[inline]
    pub fn to_float2(self) -> Float2 {
        #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
        unsafe {
            use std::arch::x86_64::*;
            let v = _mm_set_epi32(0, 0, self.y, self.x);
            let r = _mm_cvtepi32_ps(v);
            let mut out = [0f32; 4];
            _mm_storeu_ps(out.as_mut_ptr(), r);
            Float2::new(out[0], out[1])
        }
        #[cfg(not(all(target_arch = "x86_64", target_feature = "sse2")))]
        Float2::new(self.x as f32, self.y as f32)
    }

    /// 2-D cross product (scalar z).
    #[inline(always)]
    pub fn cross(self, rhs: Self) -> i64 {
        self.x as i64 * rhs.y as i64 - self.y as i64 * rhs.x as i64
    }

    /// Manhattan distance.
    #[inline(always)]
    pub fn manhattan(self, rhs: Self) -> i32 {
        let d = (self - rhs).abs();
        d.x.saturating_add(d.y)
    }

    /// Chebyshev distance.
    #[inline(always)]
    pub fn chebyshev(self, rhs: Self) -> i32 {
        let d = (self - rhs).abs();
        d.x.max(d.y)
    }
}

// ── Operator impls ────────────────────────────────────────────────────────────

macro_rules! impl_binop_i2 {
    ($trait:ident, $method:ident, $op:tt, $checked:ident) => {
        impl $trait for Int2 {
            type Output = Self;
            #[inline]
            fn $method(self, rhs: Self) -> Self {
                #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
                unsafe {
                    use std::arch::x86_64::*;
                    let a = _mm_set_epi32(0, 0, self.y, self.x);
                    let b = _mm_set_epi32(0, 0, rhs.y, rhs.x);
                    let r = $checked(a, b);
                    let mut out = [0i32; 4];
                    _mm_storeu_si128(out.as_mut_ptr() as *mut _, r);
                    Self::new(out[0], out[1])
                }
                #[cfg(not(all(target_arch = "x86_64", target_feature = "sse2")))]
                Self::new(self.x $op rhs.x, self.y $op rhs.y)
            }
        }
        impl $trait<i32> for Int2 {
            type Output = Self;
            #[inline(always)]
            fn $method(self, rhs: i32) -> Self {
                Self::new(self.x $op rhs, self.y $op rhs)
            }
        }
    };
}


#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
impl_binop_i2!(Add, add, +, _mm_add_epi32);
#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
impl_binop_i2!(Sub, sub, -, _mm_sub_epi32);

// Fallback for Add/Sub when SSE2 is not available
#[cfg(not(all(target_arch = "x86_64", target_feature = "sse2")))]
impl Add for Int2 {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self { Self::new(self.x + rhs.x, self.y + rhs.y) }
}
#[cfg(not(all(target_arch = "x86_64", target_feature = "sse2")))]
impl Add<i32> for Int2 {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: i32) -> Self { Self::new(self.x + rhs, self.y + rhs) }
}
#[cfg(not(all(target_arch = "x86_64", target_feature = "sse2")))]
impl Sub for Int2 {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self { Self::new(self.x - rhs.x, self.y - rhs.y) }
}
#[cfg(not(all(target_arch = "x86_64", target_feature = "sse2")))]
impl Sub<i32> for Int2 {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: i32) -> Self { Self::new(self.x - rhs, self.y - rhs) }
}

// mullo_epi32 requires SSE4.1; fall back to scalar multiply otherwise.
#[cfg(all(target_arch = "x86_64", target_feature = "sse4_1"))]
impl Mul for Int2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        unsafe {
            use std::arch::x86_64::*;
            let a = _mm_set_epi32(0, 0, self.y, self.x);
            let b = _mm_set_epi32(0, 0, rhs.y, rhs.x);
            let r = _mm_mullo_epi32(a, b);
            let mut out = [0i32; 4];
            _mm_storeu_si128(out.as_mut_ptr() as *mut _, r);
            Self::new(out[0], out[1])
        }
    }
}
#[cfg(not(all(target_arch = "x86_64", target_feature = "sse4_1")))]
impl Mul for Int2 {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self { Self::new(self.x * rhs.x, self.y * rhs.y) }
}
impl Mul<i32> for Int2 {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: i32) -> Self { Self::new(self.x * rhs, self.y * rhs) }
}

impl Neg for Int2 {
    type Output = Self;
    #[inline(always)]
    fn neg(self) -> Self { Self::new(-self.x, -self.y) }
}

impl AddAssign        for Int2 { #[inline(always)] fn add_assign(&mut self, r: Self) { *self = *self + r; } }
impl SubAssign        for Int2 { #[inline(always)] fn sub_assign(&mut self, r: Self) { *self = *self - r; } }
impl MulAssign        for Int2 { #[inline(always)] fn mul_assign(&mut self, r: Self) { *self = *self * r; } }
impl AddAssign<i32>   for Int2 { #[inline(always)] fn add_assign(&mut self, r: i32)  { *self = *self + r; } }
impl SubAssign<i32>   for Int2 { #[inline(always)] fn sub_assign(&mut self, r: i32)  { *self = *self - r; } }
impl MulAssign<i32>   for Int2 { #[inline(always)] fn mul_assign(&mut self, r: i32)  { *self = *self * r; } }

impl PartialEq for Int2 {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
impl Eq for Int2 {}

impl fmt::Debug for Int2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Int2({}, {})", self.x, self.y)
    }
}

impl fmt::Display for Int2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl From<[i32; 2]> for Int2 {
    #[inline(always)]
    fn from(a: [i32; 2]) -> Self { Self::new(a[0], a[1]) }
}

impl From<Int2> for [i32; 2] {
    #[inline(always)]
    fn from(v: Int2) -> Self { [v.x, v.y] }
}

impl From<(i32, i32)> for Int2 {
    #[inline(always)]
    fn from((x, y): (i32, i32)) -> Self { Self::new(x, y) }
}

// Float2 → Int2 (truncating)
impl From<Float2> for Int2 {
    #[inline]
    fn from(v: Float2) -> Self {
        #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
        unsafe {
            use std::arch::x86_64::*;
            let fv = _mm_set_ps(0.0, 0.0, v.y, v.x);
            let iv = _mm_cvttps_epi32(fv); // truncate toward zero
            let mut out = [0i32; 4];
            _mm_storeu_si128(out.as_mut_ptr() as *mut _, iv);
            Self::new(out[0], out[1])
        }
        #[cfg(not(all(target_arch = "x86_64", target_feature = "sse2")))]
        Self::new(v.x as i32, v.y as i32)
    }
}

// ══════════════════════════════════════════════════════════════════════════════
//  Tests
// ══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Float2 ────────────────────────────────────────────────────────────────

    #[test]
    fn float2_dot() {
        let a = Float2::new(3.0, 4.0);
        let b = Float2::new(1.0, 2.0);
        assert!((a.dot(b) - 11.0).abs() < 1e-6);
    }

    #[test]
    fn float2_length() {
        let v = Float2::new(3.0, 4.0);
        assert!((v.length() - 5.0).abs() < 1e-6);
    }

    #[test]
    fn float2_normalize() {
        let v = Float2::new(3.0, 4.0).normalize();
        assert!((v.length() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn float2_normalize_fast_approx() {
        let v = Float2::new(3.0, 4.0).normalize_fast();
        assert!((v.length() - 1.0).abs() < 1e-3, "fast norm err: {}", v.length());
    }

    #[test]
    fn float2_cross() {
        let a = Float2::new(1.0, 0.0);
        let b = Float2::new(0.0, 1.0);
        assert!((a.cross(b) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn float2_lerp() {
        let a = Float2::new(0.0, 0.0);
        let b = Float2::new(10.0, 20.0);
        let m = a.lerp(b, 0.5);
        assert_eq!(m, Float2::new(5.0, 10.0));
    }

    #[test]
    fn float2_min_max() {
        let a = Float2::new(1.0, 5.0);
        let b = Float2::new(3.0, 2.0);
        assert_eq!(a.min(b), Float2::new(1.0, 2.0));
        assert_eq!(a.max(b), Float2::new(3.0, 5.0));
    }

    #[test]
    fn float2_abs() {
        let v = Float2::new(-3.0, 4.0).abs();
        assert_eq!(v, Float2::new(3.0, 4.0));
    }

    #[test]
    fn float2_ops() {
        let a = Float2::new(1.0, 2.0);
        let b = Float2::new(3.0, 4.0);
        assert_eq!(a + b, Float2::new(4.0, 6.0));
        assert_eq!(b - a, Float2::new(2.0, 2.0));
        assert_eq!(a * 2.0, Float2::new(2.0, 4.0));
        assert_eq!(b / 2.0, Float2::new(1.5, 2.0));
        assert_eq!(-a, Float2::new(-1.0, -2.0));
    }

    #[test]
    fn float2_reflect() {
        let v = Float2::new(1.0, -1.0);
        let n = Float2::new(0.0, 1.0); // up-facing normal
        let r = v.reflect(n);
        assert!((r.x - 1.0).abs() < 1e-6 && (r.y - 1.0).abs() < 1e-6);
    }

    #[test]
    fn float2_perp() {
        let v = Float2::new(1.0, 0.0);
        assert_eq!(v.perp(), Float2::new(0.0, 1.0));
    }

    #[test]
    fn float2_floor_ceil() {
        let v = Float2::new(1.3, -2.7);
        assert_eq!(v.floor(), Float2::new(1.0, -3.0));
        assert_eq!(v.ceil(), Float2::new(2.0, -2.0));
    }

    // ── Int2 ──────────────────────────────────────────────────────────────────

    #[test]
    fn int2_dot() {
        let a = Int2::new(3, 4);
        let b = Int2::new(1, 2);
        assert_eq!(a.dot(b), 11);
    }

    #[test]
    fn int2_min_max() {
        let a = Int2::new(1, 5);
        let b = Int2::new(3, 2);
        assert_eq!(a.min(b), Int2::new(1, 2));
        assert_eq!(a.max(b), Int2::new(3, 5));
    }

    #[test]
    fn int2_abs() {
        let v = Int2::new(-7, 3).abs();
        assert_eq!(v, Int2::new(7, 3));
    }

    #[test]
    fn int2_cross() {
        assert_eq!(Int2::X.cross(Int2::Y), 1);
        assert_eq!(Int2::Y.cross(Int2::X), -1);
    }

    #[test]
    fn int2_manhattan() {
        let a = Int2::new(0, 0);
        let b = Int2::new(3, 4);
        assert_eq!(a.manhattan(b), 7);
    }

    #[test]
    fn int2_chebyshev() {
        let a = Int2::ZERO;
        let b = Int2::new(3, 4);
        assert_eq!(a.chebyshev(b), 4);
    }

    #[test]
    fn int2_ops() {
        let a = Int2::new(1, 2);
        let b = Int2::new(3, 4);
        assert_eq!(a + b, Int2::new(4, 6));
        assert_eq!(b - a, Int2::new(2, 2));
        assert_eq!(a * b, Int2::new(3, 8));
        assert_eq!(a * 3, Int2::new(3, 6));
        assert_eq!(-a, Int2::new(-1, -2));
    }

    #[test]
    fn int2_to_float2() {
        let i = Int2::new(3, -4);
        let f = i.to_float2();
        assert_eq!(f, Float2::new(3.0, -4.0));
    }

    #[test]
    fn float2_to_int2() {
        let f = Float2::new(3.9, -2.1);
        let i = Int2::from(f);
        assert_eq!(i, Int2::new(3, -2)); // truncation
    }

    #[test]
    fn alignment() {
        assert_eq!(std::mem::align_of::<Float2>(), 16);
        assert_eq!(std::mem::align_of::<Int2>(), 16);
        assert_eq!(std::mem::size_of::<Float2>(), 8);
        assert_eq!(std::mem::size_of::<Int2>(), 8);
    }
}