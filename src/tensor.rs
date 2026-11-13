use crate::Error;
use std::ops::{Add, Div, Mul, Sub};

pub type Vec3 = [f64; 3];

/// Tenseur des contraintes symétrique, composantes `[xx, yy, zz, xy, xz, yz]`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct StressTensor {
    pub xx: f64,
    pub yy: f64,
    pub zz: f64,
    pub xy: f64,
    pub xz: f64,
    pub yz: f64,
}

impl StressTensor {
    pub const ZERO: Self = Self { xx: 0.0, yy: 0.0, zz: 0.0, xy: 0.0, xz: 0.0, yz: 0.0 };

    pub const fn new(xx: f64, yy: f64, zz: f64, xy: f64, xz: f64, yz: f64) -> Self {
        Self { xx, yy, zz, xy, xz, yz }
    }

    pub fn from_components(values: [f64; 6]) -> Self {
        Self::new(values[0], values[1], values[2], values[3], values[4], values[5])
    }

    pub fn components(self) -> [f64; 6] {
        [self.xx, self.yy, self.zz, self.xy, self.xz, self.yz]
    }

    pub fn matrix(self) -> [[f64; 3]; 3] {
        [[self.xx, self.xy, self.xz], [self.xy, self.yy, self.yz], [self.xz, self.yz, self.zz]]
    }

    pub fn from_matrix(m: [[f64; 3]; 3]) -> Self {
        Self::new(m[0][0], m[1][1], m[2][2], 0.5 * (m[0][1] + m[1][0]),
                  0.5 * (m[0][2] + m[2][0]), 0.5 * (m[1][2] + m[2][1]))
    }

    pub fn is_finite(self) -> bool {
        self.components().iter().all(|v| v.is_finite())
    }

    pub fn hydrostatic(self) -> f64 {
        (self.xx + self.yy + self.zz) / 3.0
    }

    pub fn deviator(self) -> Self {
        let p = self.hydrostatic();
        Self::new(self.xx - p, self.yy - p, self.zz - p, self.xy, self.xz, self.yz)
    }

    /// Vecteur de traction tangentielle `σ n - (n · σ n)n` (normalisée en interne).
    pub fn shear_on_plane(self, normal: Vec3) -> Vec3 {
        let n = normalize(normal);
        let traction = mat_vec(self.matrix(), n);
        let normal_traction = dot(traction, n);
        sub(traction, scale(n, normal_traction))
    }

    /// Différence des valeurs propres extrêmes, convention Tresca du projet.
    pub fn tresca(self) -> f64 {
        let eigenvalues = self.eigenvalues_symmetric();
        eigenvalues[2] - eigenvalues[0]
    }

    /// Norme équivalente `sqrt(3 J2)`, avec `J2 = s:s/2` pour un déviateur.
    pub fn j2_norm(self) -> f64 {
        let s = self.deviator();
        let double_contract = s.xx * s.xx + s.yy * s.yy + s.zz * s.zz
            + 2.0 * (s.xy * s.xy + s.xz * s.xz + s.yz * s.yz);
        (1.5 * double_contract).sqrt()
    }

    /// Valeurs propres par itérations de Jacobi, adaptées aux matrices réelles symétriques 3×3.
    pub fn eigenvalues_symmetric(self) -> [f64; 3] {
        let mut a = self.matrix();
        for _ in 0..32 {
            let mut p = 0;
            let mut q = 1;
            if a[0][2].abs() > a[p][q].abs() { p = 0; q = 2; }
            if a[1][2].abs() > a[p][q].abs() { p = 1; q = 2; }
            let apq = a[p][q];
            let matrix_scale = a.iter().flatten().fold(0.0_f64, |scale, value| scale.max(value.abs()));
            if apq.abs() <= 1e-14 * matrix_scale { break; }
            let tau = (a[q][q] - a[p][p]) / (2.0 * apq);
            let t = if tau >= 0.0 { 1.0 / (tau + (1.0 + tau * tau).sqrt()) }
                    else { -1.0 / (-tau + (1.0 + tau * tau).sqrt()) };
            let c = 1.0 / (1.0 + t * t).sqrt();
            let s = t * c;
            let app = a[p][p];
            let aqq = a[q][q];
            a[p][p] = app - t * apq;
            a[q][q] = aqq + t * apq;
            a[p][q] = 0.0;
            a[q][p] = 0.0;
            for r in 0..3 {
                if r != p && r != q {
                    let arp = a[r][p];
                    let arq = a[r][q];
                    a[r][p] = c * arp - s * arq;
                    a[p][r] = a[r][p];
                    a[r][q] = s * arp + c * arq;
                    a[q][r] = a[r][q];
                }
            }
        }
        let mut values = [a[0][0], a[1][1], a[2][2]];
        values.sort_by(f64::total_cmp);
        values
    }
}

impl Add for StressTensor { type Output = Self; fn add(self, r: Self) -> Self { Self::new(self.xx+r.xx,self.yy+r.yy,self.zz+r.zz,self.xy+r.xy,self.xz+r.xz,self.yz+r.yz) } }
impl Sub for StressTensor { type Output = Self; fn sub(self, r: Self) -> Self { Self::new(self.xx-r.xx,self.yy-r.yy,self.zz-r.zz,self.xy-r.xy,self.xz-r.xz,self.yz-r.yz) } }
impl Mul<f64> for StressTensor { type Output = Self; fn mul(self, k: f64) -> Self { Self::new(self.xx*k,self.yy*k,self.zz*k,self.xy*k,self.xz*k,self.yz*k) } }
impl Div<f64> for StressTensor { type Output = Self; fn div(self, k: f64) -> Self { self * (1.0/k) } }

pub(crate) fn dot(a: Vec3, b: Vec3) -> f64 { a[0]*b[0] + a[1]*b[1] + a[2]*b[2] }
pub(crate) fn norm(a: Vec3) -> f64 { dot(a, a).sqrt() }
pub(crate) fn scale(a: Vec3, k: f64) -> Vec3 { [a[0]*k, a[1]*k, a[2]*k] }
pub(crate) fn sub(a: Vec3, b: Vec3) -> Vec3 { [a[0]-b[0], a[1]-b[1], a[2]-b[2]] }
pub(crate) fn normalize(a: Vec3) -> Vec3 { let l = norm(a); if l > 0.0 { scale(a, 1.0/l) } else { [0.0, 0.0, 1.0] } }
pub(crate) fn mat_vec(a: [[f64; 3]; 3], v: Vec3) -> Vec3 {
    [dot(a[0], v), dot(a[1], v), dot(a[2], v)]
}

impl TryFrom<[f64; 6]> for StressTensor {
    type Error = Error;
    fn try_from(value: [f64; 6]) -> Result<Self, Self::Error> {
        let tensor = Self::from_components(value);
        if tensor.is_finite() { Ok(tensor) } else { Err(Error::NonFiniteValue("tenseur")) }
    }
}
