use crate::circle::minimum_enclosing_circle;
use crate::tensor::{dot, normalize, Vec3};
use crate::{Error, LoadingHistory};

#[derive(Debug, Clone, Copy)]
pub struct AngularConfig {
    /// Pas de discrétisation en degrés. Le défaut historique est un degré.
    pub step_degrees: f64,
    /// Coefficient matériau `alpha` dans `tau + alpha * p`.
    pub alpha: f64,
}

impl Default for AngularConfig {
    fn default() -> Self { Self { step_degrees: 1.0, alpha: 0.0 } }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AngularResult {
    pub pressure: Vec<f64>,
    /// Rayon du plus petit cercle englobant la trajectoire de cisaillement du plan critique.
    pub shear_amplitude: f64,
    pub critical_normal: Vec3,
    pub dang_van_values: Vec<f64>,
    pub max_dang_van: f64,
}

/// Calcule la méthode de recherche angulaire sur l'historique complet.
///
/// Chaque plan est gardé fixe sur tout l'historique. L'amplitude du cisaillement est le rayon
/// du plus petit cercle englobant les vecteurs tangentiels `tau(t)` (et non le maximum instantané
/// recalculé sur un plan différent à chaque instant).
pub fn analyze_angular(history: &LoadingHistory, config: AngularConfig) -> Result<AngularResult, Error> {
    if !config.step_degrees.is_finite() || config.step_degrees <= 0.0 || config.step_degrees > 90.0 {
        return Err(Error::InvalidAngularStep(config.step_degrees));
    }
    if !config.alpha.is_finite() { return Err(Error::NonFiniteValue("alpha")); }

    let pressure: Vec<f64> = history.samples.iter().map(|s| s.hydrostatic()).collect();
    let mut best_amp = -1.0;
    let mut best_normal = [0.0, 0.0, 1.0];
    let theta_count = (360.0 / config.step_degrees).ceil() as usize;
    let phi_count = (90.0 / config.step_degrees).ceil() as usize;

    for iphi in 0..=phi_count {
        let phi = ((iphi as f64) * config.step_degrees).min(90.0).to_radians();
        let (sin_phi, cos_phi) = phi.sin_cos();
        for itheta in 0..theta_count {
            let theta = (itheta as f64 * 360.0 / theta_count as f64).to_radians();
            let (sin_theta, cos_theta) = theta.sin_cos();
            let normal = normalize([cos_theta * sin_phi, sin_theta * sin_phi, cos_phi]);
            let (e1, e2) = tangent_basis(normal);
            let points: Vec<[f64; 2]> = history.samples.iter().map(|stress| {
                let shear = stress.shear_on_plane(normal);
                [dot(shear, e1), dot(shear, e2)]
            }).collect();
            let (_, amplitude) = minimum_enclosing_circle(&points);
            if amplitude > best_amp {
                best_amp = amplitude;
                best_normal = normal;
            }
        }
    }

    let dang_van_values: Vec<f64> = pressure.iter().map(|p| best_amp + config.alpha * p).collect();
    let max_dang_van = dang_van_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    Ok(AngularResult {
        pressure,
        shear_amplitude: best_amp.max(0.0),
        critical_normal: best_normal,
        dang_van_values,
        max_dang_van,
    })
}

fn tangent_basis(normal: Vec3) -> (Vec3, Vec3) {
    let reference = if normal[2].abs() < 0.9 { [0.0, 0.0, 1.0] } else { [1.0, 0.0, 0.0] };
    let first = normalize(cross(reference, normal));
    let second = normalize(cross(normal, first));
    (first, second)
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StressTensor;

    #[test]
    fn fully_reversed_uniaxial_gives_half_amplitude_shear() {
        let history = LoadingHistory::new(vec![
            StressTensor::new(100.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            StressTensor::new(-100.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        ]).unwrap();
        let result = analyze_angular(&history, AngularConfig { step_degrees: 5.0, alpha: 0.0 }).unwrap();
        assert!((result.shear_amplitude - 50.0).abs() < 1e-8, "{}", result.shear_amplitude);
    }

    #[test]
    fn hydrostatic_history_has_zero_shear() {
        let history = LoadingHistory::new(vec![
            StressTensor::new(12.0, 12.0, 12.0, 0.0, 0.0, 0.0),
            StressTensor::new(-8.0, -8.0, -8.0, 0.0, 0.0, 0.0),
        ]).unwrap();
        let result = analyze_angular(&history, AngularConfig { step_degrees: 15.0, alpha: 0.2 }).unwrap();
        assert!(result.shear_amplitude < 1e-10);
        assert!((result.max_dang_van - 2.4).abs() < 1e-10);
    }
}
