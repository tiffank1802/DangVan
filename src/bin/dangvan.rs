use dangvan::{analyze_angular, analyze_deviatoric, csv_io::load_csv_path, AngularConfig, Centering, DeviatoricConfig, Metric};
use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => { eprintln!("Erreur : {message}\n\nUsage : dangvan <fichier.csv> [angular|deviatoric] [--step-deg N] [--alpha A] [--metric tresca|j2]"); ExitCode::FAILURE }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let path = args.next().ok_or_else(|| "fichier CSV requis".to_string())?;
    let mut method = "angular".to_string();
    let mut step = 1.0;
    let mut alpha = 0.0;
    let mut metric = Metric::Tresca;
    let rest: Vec<String> = args.collect();
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "angular" | "deviatoric" => method = rest[i].clone(),
            "--step-deg" => { i += 1; step = rest.get(i).ok_or("valeur manquante pour --step-deg")?.parse().map_err(|_| "pas angulaire invalide")?; }
            "--alpha" => { i += 1; alpha = rest.get(i).ok_or("valeur manquante pour --alpha")?.parse().map_err(|_| "alpha invalide")?; }
            "--metric" => {
                i += 1;
                metric = match rest.get(i).map(String::as_str) {
                    Some("tresca") => Metric::Tresca,
                    Some("j2") => Metric::J2,
                    _ => return Err("la métrique doit être tresca ou j2".into()),
                };
            }
            other => return Err(format!("option ou méthode inconnue : {other}")),
        }
        i += 1;
    }

    let history = load_csv_path(&path).map_err(|e| e.to_string())?;
    match method.as_str() {
        "angular" => {
            let result = analyze_angular(&history, AngularConfig { step_degrees: step, alpha }).map_err(|e| e.to_string())?;
            println!("Méthode : balayage angulaire (pas {:.4}°)", step);
            println!("Échantillons : {}", history.samples.len());
            println!("Plan critique (normale) : [{:.8}, {:.8}, {:.8}]", result.critical_normal[0], result.critical_normal[1], result.critical_normal[2]);
            println!("Amplitude de cisaillement : {:.8}", result.shear_amplitude);
            println!("Maximum de tau + alpha*p : {:.8}", result.max_dang_van);
        }
        "deviatoric" => {
            let result = analyze_deviatoric(&history, DeviatoricConfig { metric, centering: Centering::FarthestPairMidpoint }).map_err(|e| e.to_string())?;
            println!("Méthode : déviatorique, milieu de la paire extrême (heuristique)");
            println!("Métrique : {:?}", metric);
            println!("Échantillons : {}", history.samples.len());
            println!("Paire extrême : {:?}, distance {:.8}", result.farthest_pair_indices, result.farthest_pair_distance);
            println!("Centre déviatorique [xx, yy, zz, xy, xz, yz] : {:?}", result.center.components());
            println!("Maximum recentré : {:.8}", result.max_equivalent);
        }
        _ => unreachable!(),
    }
    Ok(())
}
