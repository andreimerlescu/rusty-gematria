// src/gui.rs
// native GUI using egui/eframe
// launched with --gui flag
// shares cipher, matrix, textee, phrase logic with tui.rs

use eframe::egui;
use crate::cipher;
use crate::matrix::{Cipher, Matrix};
use crate::phrase::{self, TaggedWord};
use crate::textee;

// application state — mirrors App in tui.rs
pub struct GematriaApp {
    input:        String,
    results:      Vec<cipher::Gematria>,
    selected:     Option<usize>,
    matches:      Vec<String>,
    matrix:       Matrix,
    tagged_words: Vec<TaggedWord>,
    limit:        usize,
    delay:        u64,
}

impl GematriaApp {
    pub fn new(
        matrix:       Matrix,
        tagged_words: Vec<TaggedWord>,
        limit:        usize,
        delay:        u64,
        initial_text: Option<String>,
    ) -> GematriaApp {
        let mut app = GematriaApp {
            input:        initial_text.unwrap_or_default(),
            results:      Vec::new(),
            selected:     None,
            matches:      Vec::new(),
            matrix,
            tagged_words,
            limit,
            delay,
        };
        if !app.input.is_empty() {
            app.recompute();
        }
        app
    }

    fn recompute(&mut self) {
        let phrases = textee::extract(&self.input, self.limit);
        self.results = phrases
            .iter()
            .map(|p| cipher::calculate(&p.text))
            .collect();
        if !self.results.is_empty() {
            self.selected = Some(0);
            self.update_matches();
        } else {
            self.selected = None;
            self.matches.clear();
        }
    }

    fn update_matches(&mut self) {
        if let Some(i) = self.selected {
            if let Some(g) = self.results.get(i) {
                let dict_matches = self.matrix
                    .lookup(&Cipher::English, g.english)
                    .to_vec();

                let generated = phrase::generate(
                    &self.tagged_words,
                    g.english,
                    &Cipher::English,
                    self.limit,
                );

                let gen_texts: Vec<String> = generated
                    .into_iter()
                    .map(|p| format!("[{}]", p.text))
                    .collect();

                self.matches = dict_matches;
                self.matches.extend(gen_texts);
            }
        }
    }
}

impl eframe::App for GematriaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // top panel — input
        egui::TopBottomPanel::top("input_panel").show(ctx, |ui| {
            ui.heading("RustyGematria");
            ui.horizontal(|ui| {
                ui.label("Input:");
                let response = ui.text_edit_singleline(&mut self.input);
                if response.changed() {
                    self.recompute();
                }
            });
        });

        // bottom panel — matches
        egui::TopBottomPanel::bottom("matches_panel")
            .min_height(120.0)
            .show(ctx, |ui| {
                let value_label = self.selected
                    .and_then(|i| self.results.get(i))
                    .map(|g| format!("Matches — English {}", g.english))
                    .unwrap_or_else(|| "Matches".to_string());

                ui.heading(value_label);

                egui::ScrollArea::vertical()
                    .id_source("matches_scroll")
                    .show(ui, |ui| {
                        if self.matches.is_empty() {
                            ui.label("no matches");
                        } else {
                            // display matches as wrapping text
                            ui.horizontal_wrapped(|ui| {
                                for m in &self.matches {
                                    // generated phrases in brackets styled differently
                                    if m.starts_with('[') {
                                        ui.colored_label(egui::Color32::GOLD, m);
                                    } else {
                                        ui.label(m);
                                    }
                                }
                            });
                        }
                    });
            });

        // central panel — results table
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Results");

            // column headers
            egui::Grid::new("results_header")
                .num_columns(7)
                .striped(false)
                .show(ui, |ui| {
                    ui.strong("Phrase");
                    ui.strong("Eng");
                    ui.strong("Jew");
                    ui.strong("Sim");
                    ui.strong("Mys");
                    ui.strong("Maj");
                    ui.strong("Eit");
                    ui.end_row();
                });

            ui.separator();

            egui::ScrollArea::vertical()
                .id_source("results_scroll")
                .show(ui, |ui| {
                    egui::Grid::new("results_grid")
                        .num_columns(7)
                        .striped(true)
                        .show(ui, |ui| {
                            let selected = self.selected;
                            let mut new_selected = selected;

                            for (i, g) in self.results.iter().enumerate() {
                                let is_selected = selected == Some(i);

                                // highlight selected row
                                let label_style = if is_selected {
                                    egui::RichText::new(&g.original)
                                        .color(egui::Color32::BLACK)
                                        .background_color(egui::Color32::GOLD)
                                } else {
                                    egui::RichText::new(&g.original)
                                };

                                if ui.label(label_style).clicked() {
                                    new_selected = Some(i);
                                }

                                ui.label(g.english.to_string());
                                ui.label(g.jewish.to_string());
                                ui.label(g.simple.to_string());
                                ui.label(g.mystery.to_string());
                                ui.label(g.majestic.to_string());
                                ui.label(g.eights.to_string());
                                ui.end_row();
                            }

                            if new_selected != selected {
                                self.selected = new_selected;
                                self.update_matches();
                            }
                        });
                });
        });
    }
}

// entry point called from main when --gui is passed
pub fn run(
    matrix:       Matrix,
    tagged_words: Vec<TaggedWord>,
    limit:        usize,
    delay:        u64,
    initial_text: Option<String>,
) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("RustyGematria")
            .with_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "RustyGematria",
        options,
        Box::new(|_cc| {
            Ok(Box::new(GematriaApp::new(
                matrix,
                tagged_words,
                limit,
                delay,
                initial_text,
            )))
        }),
    )
}
