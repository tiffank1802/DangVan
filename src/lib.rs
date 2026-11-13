//! Outils de calcul du critère de fatigue multiaxiale de Dang Van.
//!
//! Les composantes d'un tenseur sont stockées dans l'ordre `[xx, yy, zz, xy, xz, yz]`.
//! Les cisaillements sont les composantes tensorielle usuelles (aucun facteur 2).

mod circle;
pub mod csv_io;
pub mod deviatoric;
pub mod angular;
pub mod tensor;

use std::error::Error as StdError;
use std::fmt;

pub use angular::{analyze_angular, AngularConfig, AngularResult};
pub use deviatoric::{analyze_deviatoric, Centering, DeviatoricConfig, DeviatoricResult, Metric};
pub use tensor::{StressTensor, Vec3};

/// Erreur commune de validation et de lecture.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    EmptyHistory,
    NonFiniteValue(&'static str),
    InvalidAngularStep(f64),
    InvalidCsv(String),
    Io(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyHistory => write!(f, "l'historique de contraintes est vide"),
            Self::NonFiniteValue(where_) => write!(f, "valeur non finie dans {where_}"),
            Self::InvalidAngularStep(step) => write!(f, "pas angulaire invalide : {step}° (attendu : 0 < pas <= 90)"),
            Self::InvalidCsv(message) => write!(f, "CSV invalide : {message}"),
            Self::Io(message) => write!(f, "erreur d'entrée/sortie : {message}"),
        }
    }
}

impl StdError for Error {}

/// Historique discret des contraintes macroscopiques.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadingHistory {
    pub samples: Vec<StressTensor>,
}

impl LoadingHistory {
    pub fn new(samples: Vec<StressTensor>) -> Result<Self, Error> {
        if samples.is_empty() {
            return Err(Error::EmptyHistory);
        }
        if samples.iter().any(|sample| !sample.is_finite()) {
            return Err(Error::NonFiniteValue("l'historique"));
        }
        Ok(Self { samples })
    }
}
