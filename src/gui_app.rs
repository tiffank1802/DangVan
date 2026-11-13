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
    school_logo: Option<egui::TextureHandle>,
    glass_theme_applied: bool,
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
            school_logo: None,
            glass_theme_applied: false,
        }
    }

    fn load_school_logo(&mut self, ctx: &egui::Context) {
        if self.school_logo.is_none() {
            let bytes = include_bytes!("../images/branding/logo_centrale.png");
            let rgba = image::load_from_memory(bytes)
                .expect("Le logo Centrale Lyon doit être un PNG valide")
                .to_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            let image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
            self.school_logo = Some(ctx.load_texture(
                "centrale-lyon-logo",
                image,
                egui::TextureOptions::LINEAR,
            ));
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
        if !self.glass_theme_applied {
            apply_glass_theme(ctx);
            self.glass_theme_applied = true;
        }
        self.load_school_logo(ctx);
        paint_glass_backdrop(ctx);

        egui::TopBottomPanel::top("header")
            .frame(glass_frame().inner_margin(egui::Margin::symmetric(18.0, 12.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some(logo) = &self.school_logo {
                        let (logo_rect, _) = ui.allocate_exact_size(Vec2::new(190.0, 48.0), Sense::hover());
                        ui.painter().rect_filled(logo_rect, 7.0, Color32::WHITE);
                        ui.painter().image(
                            logo.id(),
                            logo_rect.shrink(4.0),
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    }
                    ui.separator();
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("DANG VAN · LABORATOIRE DE FATIGUE").strong().size(18.0).color(Color32::WHITE));
                        ui.label(egui::RichText::new("Analyse multiaxiale · Centrale Lyon ENISE").size(12.0).color(Color32::from_rgb(158, 179, 202)));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("●  ESPACE DE TRAVAIL").small().color(Color32::from_rgb(94, 234, 212)));
                    });
                });
            });

        egui::SidePanel::left("controls")
            .default_width(310.0)
            .min_width(280.0)
            .resizable(true)
            .frame(glass_frame())
            .show(ctx, |ui| {
            ui.label(egui::RichText::new("CONFIGURATION DU CALCUL").small().strong().color(Color32::from_rgb(94, 234, 212)));
            ui.heading("Chargement");
            ui.add_space(4.0);
            egui::ComboBox::from_label("Scénario de fatigue")
                .selected_text(self.loading_case.unwrap_or(LoadingCase::Uniaxial).label())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.loading_case, Some(LoadingCase::Uniaxial), LoadingCase::Uniaxial.label());
                    ui.selectable_value(&mut self.loading_case, Some(LoadingCase::Torsion), LoadingCase::Torsion.label());
                    ui.selectable_value(&mut self.loading_case, Some(LoadingCase::Multiaxial), LoadingCase::Multiaxial.label());
                });
            ui.add(egui::Slider::new(&mut self.amplitude, 1.0..=300.0).text("Amplitude (MPa)"));
            ui.separator();
            ui.label(egui::RichText::new("PROPRIÉTÉS MATÉRIAU").small().strong().color(Color32::from_rgb(94, 234, 212)));
            ui.add(egui::Slider::new(&mut self.alpha, -1.0..=1.0).text("Coefficient α"));
            ui.add(egui::Slider::new(&mut self.beta, 1.0..=200.0).text("Limite β (MPa)"));
            ui.separator();
            ui.label(egui::RichText::new("RÉSOLUTION NUMÉRIQUE").small().strong().color(Color32::from_rgb(94, 234, 212)));
            ui.add(egui::Slider::new(&mut self.angular_step, 2.0..=30.0).text("Pas angulaire (°)"));
            ui.checkbox(&mut self.metric_j2, "Utiliser la métrique J2");
            ui.small("Désactivé : métrique de Tresca.");
            ui.separator();
            let calculate_button = egui::Button::new(egui::RichText::new("▶  Lancer l’analyse complète").strong())
                .fill(Color32::from_rgb(13, 148, 136));
            let button_width = ui.available_width();
            if ui.add_sized(Vec2::new(button_width, 42.0), calculate_button).clicked() {
                self.calculate(true, true);
            }
            ui.horizontal(|ui| {
                if ui.button("Angulaire").clicked() { self.calculate(true, false); }
                if ui.button("Déviatorique").clicked() { self.calculate(false, true); }
            });
            ui.small("L’historique synthétique contient 121 instants sur un cycle.");
            if let Some(message) = &self.error {
                ui.colored_label(Color32::LIGHT_RED, format!("Erreur : {message}"));
            }
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label(egui::RichText::new("ESPACE D’ANALYSE").small().strong().color(Color32::from_rgb(94, 234, 212)));
                ui.heading("Résultats de fatigue");
                ui.label(egui::RichText::new("Visualisez les contraintes et comparez-les à la limite du matériau.").color(Color32::from_rgb(158, 179, 202)));
                if self.angular_result.is_none() && self.deviatoric_result.is_none() {
                    ui.label("Choisissez un chargement puis lancez un calcul.");
                }
                if let Some(result) = &self.angular_result {
                    glass_card(ui, |ui| {
                        ui.label(egui::RichText::new("MÉTHODE ANGULAIRE").strong().color(Color32::from_rgb(94, 234, 212)));
                        ui.label("Balayage angulaire des plans critiques");
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
                        glass_card(ui, |ui| {
                            ui.label(egui::RichText::new("MÉTHODE DÉVIATORIQUE").strong().color(Color32::from_rgb(94, 234, 212)));
                            ui.label("Recentrage par paire extrême · approximation");
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

fn apply_glass_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = Color32::from_rgba_unmultiplied(11, 18, 32, 210);
    visuals.window_fill = Color32::from_rgba_unmultiplied(20, 31, 50, 238);
    visuals.extreme_bg_color = Color32::from_rgb(7, 12, 23);
    visuals.faint_bg_color = Color32::from_rgba_unmultiplied(41, 58, 80, 110);
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgba_unmultiplied(21, 34, 54, 185);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(224, 234, 245));
    visuals.widgets.inactive.bg_fill = Color32::from_rgba_unmultiplied(35, 50, 72, 190);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(224, 234, 245));
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(22, 126, 136);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    visuals.widgets.active.bg_fill = Color32::from_rgb(13, 148, 136);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    visuals.selection.bg_fill = Color32::from_rgb(13, 148, 136);
    visuals.selection.stroke = Stroke::new(1.0_f32, Color32::WHITE);
    visuals.hyperlink_color = Color32::from_rgb(94, 234, 212);

    let mut style = (*ctx.style()).clone();
    style.visuals = visuals;
    style.spacing.item_spacing = Vec2::new(12.0, 12.0);
    style.spacing.button_padding = Vec2::new(12.0, 8.0);
    ctx.set_style(style);
}

fn paint_glass_backdrop(ctx: &egui::Context) {
    let screen = ctx.screen_rect();
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Background,
        egui::Id::new("liquid-glass-backdrop"),
    ));
    painter.rect_filled(screen, 0.0, Color32::from_rgb(7, 12, 23));
    paint_glow(
        &painter,
        Pos2::new(screen.right() - 70.0, screen.top() + 100.0),
        [42, 90, 190],
    );
    paint_glow(
        &painter,
        Pos2::new(screen.left() + 60.0, screen.bottom() - 80.0),
        [13, 148, 136],
    );
    paint_glow(
        &painter,
        Pos2::new(screen.center().x + 30.0, screen.center().y),
        [94, 65, 150],
    );
}

fn paint_glow(painter: &egui::Painter, center: Pos2, rgb: [u8; 3]) {
    for layer in (0..8).rev() {
        let radius = 72.0 + layer as f32 * 20.0;
        let alpha = (3 + (7 - layer) * 2) as u8;
        painter.circle_filled(
            center,
            radius,
            Color32::from_rgba_unmultiplied(rgb[0], rgb[1], rgb[2], alpha),
        );
    }
}

fn glass_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(Color32::from_rgba_unmultiplied(20, 31, 50, 202))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(170, 205, 228, 74)))
        .rounding(egui::Rounding::same(18.0))
        .inner_margin(egui::Margin::same(16.0))
}

fn glass_card<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<R> {
    egui::Frame::none()
        .fill(Color32::from_rgba_unmultiplied(29, 43, 64, 176))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(190, 220, 238, 58)))
        .rounding(egui::Rounding::same(14.0))
        .inner_margin(egui::Margin::same(15.0))
        .show(ui, add_contents)
}
