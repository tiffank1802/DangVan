use dangvan::{
    analyze_angular, analyze_deviatoric, AngularConfig, AngularResult, Centering,
    DeviatoricConfig, DeviatoricResult, LoadingHistory, Metric, StressTensor,
};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoadingCase { Uniaxial, Torsion, Multiaxial }

impl LoadingCase {
    fn label(self) -> &'static str {
        match self {
            Self::Uniaxial => "Traction-compression",
            Self::Torsion => "Torsion pure",
            Self::Multiaxial => "Multiaxial",
        }
    }
}

pub struct DangVanApp {
    loading_case: Option<LoadingCase>,
    amplitude: f64,
    alpha: f64,
    beta: f64,
    angular_step: f64,
    metric_j2: bool,
    angular_result: Option<AngularResult>,
    deviatoric_result: Option<DeviatoricResult>,
    error: Option<String>,
}

impl DangVanApp {
    pub fn new() -> Self {
        Self {
            loading_case: Some(LoadingCase::Uniaxial),
            amplitude: 100.0,
            alpha: 0.23,
            beta: 50.0,
            angular_step: 5.0,
            metric_j2: false,
            angular_result: None,
            deviatoric_result: None,
            error: None,
        }
    }

    fn make_history(&self) -> Result<LoadingHistory, String> {
        let case = self.loading_case.unwrap_or(LoadingCase::Uniaxial);
        let samples = (0..=120).map(|i| {
            let phase = std::f64::consts::TAU * i as f64 / 120.0;
            let c = phase.cos();
            let s = phase.sin();
            match case {
                LoadingCase::Uniaxial => StressTensor::new(self.amplitude * c, 0.0, 0.0, 0.0, 0.0, 0.0),
                LoadingCase::Torsion => StressTensor::new(0.0, 0.0, 0.0, self.amplitude * c, 0.0, 0.0),
                LoadingCase::Multiaxial => StressTensor::new(
                    self.amplitude * c,
                    0.25 * self.amplitude * (2.0 * phase).cos(),
                    0.0,
                    -0.35 * self.amplitude * s,
                    0.0,
                    0.2 * self.amplitude * (2.0 * phase).sin(),
                ),
            }
        }).collect();
        LoadingHistory::new(samples).map_err(|e| e.to_string())
    }

    fn calculate(&mut self, angular: bool, deviatoric: bool) {
        self.angular_result = None;
        self.deviatoric_result = None;
        self.error = None;
        let result = self.make_history().and_then(|history| {
            if angular {
                self.angular_result = Some(analyze_angular(
                    &history,
                    AngularConfig { step_degrees: self.angular_step, alpha: self.alpha },
                ).map_err(|e| e.to_string())?);
            }
            if deviatoric {
                let metric = if self.metric_j2 { Metric::J2 } else { Metric::Tresca };
                self.deviatoric_result = Some(analyze_deviatoric(
                    &history,
                    DeviatoricConfig { metric, centering: Centering::FarthestPairMidpoint },
                ).map_err(|e| e.to_string())?);
            }
            Ok(())
        });
        if let Err(message) = result { self.error = Some(message); }
    }
}

impl Default for DangVanApp {
    fn default() -> Self { Self::new() }
}

impl eframe::App for DangVanApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.heading("Critère de fatigue multiaxiale de Dang Van");
            ui.label("Calcul Rust — méthodes angulaire et déviatorique");
        });

        egui::SidePanel::left("controls").min_width(265.0).show(ctx, |ui| {
            ui.heading("Chargement");
            egui::ComboBox::from_label("Cas de charge")
                .selected_text(self.loading_case.unwrap_or(LoadingCase::Uniaxial).label())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.loading_case, Some(LoadingCase::Uniaxial), LoadingCase::Uniaxial.label());
                    ui.selectable_value(&mut self.loading_case, Some(LoadingCase::Torsion), LoadingCase::Torsion.label());
                    ui.selectable_value(&mut self.loading_case, Some(LoadingCase::Multiaxial), LoadingCase::Multiaxial.label());
                });
            ui.add(egui::Slider::new(&mut self.amplitude, 1.0..=300.0).text("Amplitude (MPa)"));
            ui.add(egui::Slider::new(&mut self.alpha, -1.0..=1.0).text("Coefficient α"));
            ui.add(egui::Slider::new(&mut self.beta, 1.0..=200.0).text("Limite matériau β (MPa)"));
            ui.add(egui::Slider::new(&mut self.angular_step, 2.0..=30.0).text("Pas angulaire (°)"));
            ui.checkbox(&mut self.metric_j2, "Métrique J2 (sinon Tresca)");
            ui.separator();
            if ui.button("Calculer les deux méthodes").clicked() { self.calculate(true, true); }
            ui.horizontal(|ui| {
                if ui.button("Angulaire seule").clicked() { self.calculate(true, false); }
                if ui.button("Déviatorique seule").clicked() { self.calculate(false, true); }
            });
            ui.small("L’historique synthétique contient 121 instants sur un cycle.");
            if let Some(message) = &self.error {
                ui.colored_label(Color32::LIGHT_RED, format!("Erreur : {message}"));
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Résultats");
                if self.angular_result.is_none() && self.deviatoric_result.is_none() {
                    ui.label("Choisissez un chargement puis lancez un calcul.");
                }
                if let Some(result) = &self.angular_result {
                    ui.group(|ui| {
                        ui.label("Méthode angulaire — balayage des plans");
                        ui.label(format!("Amplitude de cisaillement critique : {:.4} MPa", result.shear_amplitude));
                        let max_dang_van = result.pressure.iter()
                            .map(|pressure| result.shear_amplitude + self.alpha * pressure)
                            .fold(f64::NEG_INFINITY, f64::max);
                        ui.label(format!("Maximum τ + αp : {:.4} MPa (limite β = {:.2} MPa)", max_dang_van, self.beta));
                        let resists = max_dang_van <= self.beta;
                        ui.colored_label(
                            if resists { Color32::LIGHT_GREEN } else { Color32::LIGHT_RED },
                            if resists { "Résiste : le critère est respecté" } else { "Dépassement : rupture prédite" },
                        );
                        ui.label(format!("Normale critique : [{:.3}, {:.3}, {:.3}]", result.critical_normal[0], result.critical_normal[1], result.critical_normal[2]));
                    });
                    let points: Vec<[f64; 2]> = result.pressure.iter().map(|p| [*p, result.shear_amplitude]).collect();
                    scatter_plot(ui, "Diagramme de Dang Van — angulaire", "Pression hydrostatique p (MPa)", "Cisaillement τ (MPa)", &points, Color32::from_rgb(40, 120, 230), self.alpha, self.beta);
                }
                if let Some(result) = &self.deviatoric_result {
                    if let Ok(history) = self.make_history() {
                        let max_dang_van = history.samples.iter().zip(&result.equivalent_by_sample)
                            .map(|(stress, shear)| *shear + self.alpha * stress.hydrostatic())
                            .fold(f64::NEG_INFINITY, f64::max);
                        ui.group(|ui| {
                            ui.label("Méthode déviatorique — recentrage par paire extrême (heuristique)");
                            ui.label(format!("Maximum recentré : {:.4} MPa", result.max_equivalent));
                            ui.label(format!("Maximum τ + αp : {:.4} MPa (limite β = {:.2} MPa)", max_dang_van, self.beta));
                            let resists = max_dang_van <= self.beta;
                            ui.colored_label(
                                if resists { Color32::LIGHT_GREEN } else { Color32::LIGHT_RED },
                                if resists { "Résiste : le critère est respecté" } else { "Dépassement : rupture prédite" },
                            );
                            ui.label(format!("Distance de la paire extrême : {:.4} MPa", result.farthest_pair_distance));
                            ui.label(format!("Centre [xx, yy, zz, xy, xz, yz] : {:?}", result.center.components().map(|v| (v * 1000.0).round() / 1000.0)));
                        });
                        let points: Vec<[f64; 2]> = history.samples.iter().zip(&result.equivalent_by_sample)
                            .map(|(stress, shear)| [stress.hydrostatic(), *shear]).collect();
                        scatter_plot(ui, "Nuage recentré — déviatorique", "Pression hydrostatique p (MPa)", "Cisaillement τ (MPa)", &points, Color32::from_rgb(220, 100, 50), self.alpha, self.beta);
                    }
                }
                ui.add_space(8.0);
                ui.label("Note : la méthode déviatorique utilise le milieu de la paire la plus éloignée; elle n’est pas la solution exacte de la plus petite sphère englobante.");
            });
        });
    }
}

fn scatter_plot(
    ui: &mut egui::Ui,
    title: &str,
    x_label: &str,
    y_label: &str,
    points: &[[f64; 2]],
    color: Color32,
    alpha: f64,
    beta: f64,
) {
    ui.label(title);
    if points.is_empty() {
        ui.label("Aucune donnée à afficher.");
        return;
    }

    let width = ui.available_width().max(320.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 270.0), Sense::hover());
    let plot = Rect::from_min_max(rect.min + Vec2::new(62.0, 12.0), rect.max - Vec2::new(12.0, 48.0));
    let painter = ui.painter().with_clip_rect(plot);
    painter.rect_stroke(plot, 0.0, Stroke::new(1.0_f32, Color32::GRAY));

    let (mut xmin, mut xmax, mut ymin, mut ymax) = (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
    for point in points {
        xmin = xmin.min(point[0]);
        xmax = xmax.max(point[0]);
        ymin = ymin.min(point[1]);
        ymax = ymax.max(point[1]);
    }
    let xpad = ((xmax - xmin) * 0.08).max(1.0);
    xmin -= xpad;
    xmax += xpad;
    let limit_at_min = beta - alpha * xmin;
    let limit_at_max = beta - alpha * xmax;
    ymin = ymin.min(limit_at_min).min(limit_at_max);
    ymax = ymax.max(limit_at_min).max(limit_at_max);
    let ypad = ((ymax - ymin) * 0.12).max(1.0);
    ymin -= ypad;
    ymax += ypad;

    let x_to_screen = |x: f64| plot.left() + ((x - xmin) / (xmax - xmin)) as f32 * plot.width();
    let y_to_screen = |y: f64| plot.bottom() - ((y - ymin) / (ymax - ymin)) as f32 * plot.height();
    let text_color = Color32::LIGHT_GRAY;
    let x_step = (xmax - xmin) / 4.0;
    let y_step = (ymax - ymin) / 4.0;

    // Grille et graduations numériques sur les deux axes.
    for i in 0..=4 {
        let fraction = i as f64 / 4.0;
        let x_value = xmin + fraction * (xmax - xmin);
        let y_value = ymin + fraction * (ymax - ymin);
        let x = x_to_screen(x_value);
        let y = y_to_screen(y_value);
        painter.line_segment(
            [Pos2::new(x, plot.top()), Pos2::new(x, plot.bottom())],
            Stroke::new(0.5_f32, Color32::DARK_GRAY),
        );
        painter.line_segment(
            [Pos2::new(plot.left(), y), Pos2::new(plot.right(), y)],
            Stroke::new(0.5_f32, Color32::DARK_GRAY),
        );
        ui.painter().text(
            Pos2::new(x, plot.bottom() + 4.0),
            Align2::CENTER_TOP,
            tick_label(x_value, x_step),
            FontId::proportional(10.0),
            text_color,
        );
        ui.painter().text(
            Pos2::new(plot.left() - 5.0, y),
            Align2::RIGHT_CENTER,
            tick_label(y_value, y_step),
            FontId::proportional(10.0),
            text_color,
        );
    }

    // Limite matériau : tau + alpha*p = beta, soit tau = beta - alpha*p.
    painter.line_segment(
        [
            Pos2::new(x_to_screen(xmin), y_to_screen(limit_at_min)),
            Pos2::new(x_to_screen(xmax), y_to_screen(limit_at_max)),
        ],
        Stroke::new(2.0_f32, Color32::from_rgb(245, 75, 75)),
    );
    for point in points {
        painter.circle_filled(Pos2::new(x_to_screen(point[0]), y_to_screen(point[1])), 2.5, color);
    }

    ui.painter().text(
        Pos2::new(plot.center().x, rect.bottom() - 14.0),
        Align2::CENTER_CENTER,
        x_label,
        FontId::proportional(12.0),
        text_color,
    );
    ui.painter().text(
        Pos2::new(rect.left() + 5.0, rect.top() + 5.0),
        Align2::LEFT_TOP,
        y_label,
        FontId::proportional(11.0),
        text_color,
    );
    let (legend_rect, _) = ui.allocate_exact_size(Vec2::new(width, 20.0), Sense::hover());
    ui.painter().line_segment(
        [legend_rect.min + Vec2::new(4.0, 10.0), legend_rect.min + Vec2::new(28.0, 10.0)],
        Stroke::new(2.0_f32, Color32::from_rgb(245, 75, 75)),
    );
    ui.painter().text(
        legend_rect.min + Vec2::new(34.0, 10.0),
        Align2::LEFT_CENTER,
        format!("Limite matériau : τ = β − αp (α = {:.2}, β = {:.1} MPa)", alpha, beta),
        FontId::proportional(11.0),
        text_color,
    );
}

fn tick_label(value: f64, step: f64) -> String {
    let decimals = if step >= 10.0 { 0 } else if step >= 1.0 { 1 } else { 2 };
    format!("{value:.decimals$}")
}
