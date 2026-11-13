//! Lecture minimale, sans dépendance externe, des CSV de tenseurs du dépôt.
use crate::{Error, LoadingHistory, StressTensor};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

const COLUMNS: [&str; 6] = ["sigma_xx", "sigma_yy", "sigma_zz", "sigma_xy", "sigma_xz", "sigma_yz"];

pub fn load_csv_path(path: impl AsRef<Path>) -> Result<LoadingHistory, Error> {
    let file = File::open(path).map_err(|e| Error::Io(e.to_string()))?;
    load_csv(file)
}

/// Charge un CSV à en-tête nommé. L'ordre des colonnes peut varier; les six colonnes
/// `sigma_xx`, `sigma_yy`, `sigma_zz`, `sigma_xy`, `sigma_xz`, `sigma_yz` sont requises.
pub fn load_csv(reader: impl Read) -> Result<LoadingHistory, Error> {
    let mut lines = BufReader::new(reader).lines();
    let header_line = lines.next().ok_or_else(|| Error::InvalidCsv("en-tête absent".into()))?
        .map_err(|e| Error::Io(e.to_string()))?;
    let header_line = header_line.trim_start_matches('\u{feff}');
    let headers: Vec<&str> = header_line.split(',').map(str::trim).collect();
    let indices: [usize; 6] = COLUMNS.map(|name| {
        headers.iter().position(|header| *header == name)
            .ok_or_else(|| Error::InvalidCsv(format!("colonne obligatoire absente : {name}")))
    }).into_iter().collect::<Result<Vec<_>, _>>()?.try_into()
        .map_err(|_| Error::InvalidCsv("en-tête incomplet".into()))?;

    let mut samples = Vec::new();
    for (zero_index, line) in lines.enumerate() {
        let line_number = zero_index + 2;
        let line = line.map_err(|e| Error::Io(e.to_string()))?;
        if line.trim().is_empty() { continue; }
        let fields: Vec<&str> = line.split(',').map(str::trim).collect();
        let values = indices.map(|index| {
            fields.get(index).ok_or_else(|| Error::InvalidCsv(format!("ligne {line_number}: colonne manquante")))?
                .parse::<f64>().map_err(|_| Error::InvalidCsv(format!("ligne {line_number}: valeur numérique invalide")))
        });
        let components: [f64; 6] = values.into_iter().collect::<Result<Vec<_>, _>>()?.try_into()
            .map_err(|_| Error::InvalidCsv(format!("ligne {line_number}: nombre de colonnes invalide")))?;
        let tensor = StressTensor::from_components(components);
        if !tensor.is_finite() { return Err(Error::InvalidCsv(format!("ligne {line_number}: valeur non finie"))); }
        samples.push(tensor);
    }
    LoadingHistory::new(samples)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn reads_reordered_named_columns() {
        let data = "sigma_xy,sigma_zz,sigma_xx,sigma_yz,sigma_yy,sigma_xz\n4,3,1,6,2,5\n";
        let history = load_csv(Cursor::new(data)).unwrap();
        assert_eq!(history.samples[0].components(), [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn rejects_non_finite_values() {
        let data = "sigma_xx,sigma_yy,sigma_zz,sigma_xy,sigma_xz,sigma_yz\nNaN,0,0,0,0,0\n";
        assert!(load_csv(Cursor::new(data)).is_err());
    }
}
