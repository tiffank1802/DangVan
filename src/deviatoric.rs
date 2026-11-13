use crate::{Error, LoadingHistory, StressTensor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Metric {
    /// Différence des contraintes principales extrêmes (convention Tresca du projet).
    Tresca,
    /// `sqrt(3 J2)` pour le déviateur.
    J2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Centering {
    /// Approximation historique : milieu de la paire de déviateurs la plus éloignée.
    FarthestPairMidpoint,
}

#[derive(Debug, Clone, Copy)]
pub struct DeviatoricConfig {
    pub metric: Metric,
    pub centering: Centering,
}

impl Default for DeviatoricConfig {
    fn default() -> Self { Self { metric: Metric::Tresca, centering: Centering::FarthestPairMidpoint } }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeviatoricResult {
    pub center: StressTensor,
    pub centered_deviators: Vec<StressTensor>,
    /// Valeur de la métrique après recentrage, pour chaque pas temporel.
    pub equivalent_by_sample: Vec<f64>,
    pub max_equivalent: f64,
    pub farthest_pair_distance: f64,
    pub farthest_pair_indices: (usize, usize),
}

/// Applique le recentrage heuristique de l'approche déviatorique du dépôt.
///
/// Important : le milieu de la paire la plus éloignée n'est pas nécessairement le centre de la
/// plus petite sphère englobante en dimension cinq. Cette API le nomme explicitement afin de ne
/// pas le présenter comme une solution exacte.
pub fn analyze_deviatoric(history: &LoadingHistory, config: DeviatoricConfig) -> Result<DeviatoricResult, Error> {
    if history.samples.is_empty() { return Err(Error::EmptyHistory); }
    let deviators: Vec<StressTensor> = history.samples.iter().map(|s| s.deviator()).collect();
    let mut farthest_distance = 0.0;
    let mut pair = (0, 0);
    for i in 0..deviators.len() {
        for j in (i + 1)..deviators.len() {
            let distance = metric_norm(deviators[i] - deviators[j], config.metric);
            if distance > farthest_distance {
                farthest_distance = distance;
                pair = (i, j);
            }
        }
    }
    let center = match config.centering {
        Centering::FarthestPairMidpoint => (deviators[pair.0] + deviators[pair.1]) * 0.5,
    };
    let centered_deviators: Vec<StressTensor> = deviators.iter().map(|s| *s - center).collect();
    let equivalent_by_sample: Vec<f64> = centered_deviators.iter().map(|s| metric_norm(*s, config.metric)).collect();
    let max_equivalent = equivalent_by_sample.iter().copied().fold(0.0, f64::max);
    Ok(DeviatoricResult {
        center,
        centered_deviators,
        equivalent_by_sample,
        max_equivalent,
        farthest_pair_distance: farthest_distance,
        farthest_pair_indices: pair,
    })
}

fn metric_norm(tensor: StressTensor, metric: Metric) -> f64 {
    match metric {
        Metric::Tresca => tensor.tresca(),
        Metric::J2 => tensor.j2_norm(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LoadingHistory;

    #[test]
    fn uniaxial_deviator_has_expected_tresca() {
        let history = LoadingHistory::new(vec![
            StressTensor::new(100.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            StressTensor::new(50.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        ]).unwrap();
        let result = analyze_deviatoric(&history, DeviatoricConfig::default()).unwrap();
        assert!((result.farthest_pair_distance - 50.0).abs() < 1e-10);
        assert!((result.max_equivalent - 25.0).abs() < 1e-10);
        assert!((result.center.xx - 50.0).abs() < 1e-10);
        assert!((result.center.yy + 25.0).abs() < 1e-10);
    }

    #[test]
    fn one_sample_centers_on_its_deviator() {
        let history = LoadingHistory::new(vec![StressTensor::new(9.0, 0.0, 0.0, 0.0, 0.0, 0.0)]).unwrap();
        let result = analyze_deviatoric(&history, DeviatoricConfig::default()).unwrap();
        assert_eq!(result.max_equivalent, 0.0);
        assert_eq!(result.farthest_pair_indices, (0, 0));
    }
}
