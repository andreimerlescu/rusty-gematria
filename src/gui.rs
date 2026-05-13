// src/gui.rs
// native GUI using egui/eframe
// background thread handles heavy computation
// 369ms delay between input and recompute

use eframe::egui;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::cipher;
use crate::matrix::{Cipher, Matrix};
use crate::phrase::{self, TaggedWord};
use crate::textee;

struct ComputeResult {
    version: u64,
    results: Vec<cipher::Gematria>,
    matches: Vec<String>,
}

pub struct GematriaApp {
    input:         String,
    input_version: u64,
    results:       Vec<cipher::Gematria>,
    selected:      Option<usize>,
    matches:       Vec<String>,
    limit:         usize,
    last_typed:    Option<Instant>,
    tx:            mpsc::Sender<(u64, String)>,
    rx:            mpsc::Receiver<ComputeResult>,
}

impl GematriaApp {
    pub fn new(
        matrix:       Matrix,
        tagged_words: Vec<TaggedWord>,
        limit:        usize,
        initial_text: Option<String>,
    ) -> GematriaApp {
        let (work_tx, work_rx) = mpsc::channel::<(u64, String)>();
        let (result_tx, result_rx) = mpsc::channel::<ComputeResult>();

        let bg_matrix      = std::sync::Arc::new(matrix);
        let bg_tagged      = std::sync::Arc::new(tagged_words);
        let bg_limit       = limit;
        let bg_matrix_clone = std::sync::Arc::clone(&bg_matrix);
        let bg_tagged_clone = std::sync::Arc::clone(&bg_tagged);

        thread::spawn(move || {
            while let Ok((version, input)) = work_rx.recv() {
                let phrases = textee::extract(&input, bg_limit);
                let results: Vec<cipher::Gematria> = phrases
                    .iter()
                    .map(|p| cipher::calculate(&p.text))
                    .collect();

                let matches = if let Some(g) = results.first() {
                    let mut m = bg_matrix_clone.lookup(&Cipher::English, g.english).to_vec();
                    let generated = phrase::generate(
                        &bg_tagged_clone,
                        g.english,
                        &Cipher::English,
                        bg_limit,
                    );
                    m.extend(generated.into_iter().map(|p| format!("[{}]", p.text)));
                    m
                } else {
                    Vec::new()
                };

                let _ = result_tx.send(ComputeResult { version, results, matches });
            }
        });

        let mut app = GematriaApp {
            input:         initial_text.unwrap_or_default(),
            input_version: 0,
            results:       Vec::new(),
            selected:      None,
            matches:       Vec::new(),
            limit,
            last_typed:    None,
            tx:            work_tx,
            rx:            result_rx,
        };

        if !app.input.is_empty() {
            app.queue_recompute();
        }
        app
    }

    fn queue_recompute(&mut self) {
        self.input_version += 1;
        self.last_typed = Some(Instant::now());
    }

    fn tick(&mut self) {
        if let Some(typed_at) = self.last_typed {
            if typed_at.elapsed() >= Duration::from_millis(369) {
                let _ = self.tx.send((self.input_version, self.input.clone()));
                self.last_typed = None;
            }
        }

        while let Ok(result) = self.rx.try_recv() {
            if result.version == self.input_version {
                self.results = result.results;
                self.matches = result.matches;
                if !self.results.is_empty() {
                    self.selected = Some(0);
                }
            }
        }
    }
}

impl eframe::App for GematriaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.tick();

        // request repaint so tick fires regularly
        ctx.request_repaint_after(Duration::from_millis(50));

        egui::TopBottomPanel::top("input_panel").show(ctx, |ui| {
            ui.heading("RustyGematria");
            ui.horizontal(|ui| {
                ui.label("Input:");
                let response = ui.text_edit_singleline(&mut self.input);
                if response.changed() {
                    self.queue_recompute();
                }
            });
        });

        egui::TopBottomPanel::bottom("matches_panel")
            .min_height(120.0)
            .show(ctx, |ui| {
                let value_label = self.selected
                    .and_then(|i| self.results.get(i))
                    .map(|g| format!("Matches — English {}", g.english))
                    .unwrap_or_else(|| "Matches".to_string());

                ui.heading(value_label);

                egui::ScrollArea::vertical()
                    .id_salt("matches_scroll")
                    .show(ui, |ui| {
                        if self.matches.is_empty() {
                            ui.label("no matches");
                        } else {
                            ui.horizontal_wrapped(|ui| {
                                for m in &self.matches {
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

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Results");

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
                .id_salt("results_scroll")
                .show(ui, |ui| {
                    egui::Grid::new("results_grid")
                        .num_columns(7)
                        .striped(true)
                        .show(ui, |ui| {
                            let selected = self.selected;
                            let mut new_selected = selected;

                            for (i, g) in self.results.iter().enumerate() {
                                let is_selected = selected == Some(i);

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
                            }
                        });
                });
        });
    }
}

pub fn run(
    matrix:       Matrix,
    tagged_words: Vec<TaggedWord>,
    limit:        usize,
    _delay:       u64,
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
                initial_text,
            )))
        }),
    )
}
