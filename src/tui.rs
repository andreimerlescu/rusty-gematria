// src/tui.rs
// terminal UI for RustyGematria
// three panes: Input, Results, Matches
// vim keybindings throughout
// background thread handles heavy computation
// 369ms delay between input and recompute

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::cipher;
use crate::matrix::{Cipher, Matrix};
use crate::phrase::{self, TaggedWord};
use crate::textee;

#[derive(Debug, Clone, PartialEq)]
enum Pane {
    Input,
    Results,
    Matches,
}

#[derive(Debug, Clone, PartialEq)]
enum Mode {
    Normal,
    Insert,
}

// message from background thread to main thread
struct ComputeResult {
    version:  u64,
    results:  Vec<cipher::Gematria>,
    matches:  Vec<String>,
}

pub struct App {
    mode:          Mode,
    active_pane:   Pane,
    input:         String,
    input_version: u64,  // increments on every keystroke
    results:       Vec<cipher::Gematria>,
    results_state: TableState,
    matches:       Vec<String>,
    matrix:        Matrix,
    tagged_words:  Vec<TaggedWord>,
    limit:         usize,
    should_quit:   bool,
    last_key_g:    bool,
    last_typed:    Option<Instant>,  // when the last keystroke happened
    tx:            mpsc::Sender<(u64, String)>,
    rx:            mpsc::Receiver<ComputeResult>,
}

impl App {
    pub fn new(
        matrix:       Matrix,
        tagged_words: Vec<TaggedWord>,
        limit:        usize,
        initial_text: Option<String>,
    ) -> App {
        // channel from main thread to background thread — sends (version, input)
        let (work_tx, work_rx) = mpsc::channel::<(u64, String)>();
        // channel from background thread to main thread — sends results
        let (result_tx, result_rx) = mpsc::channel::<ComputeResult>();

        // clone what the background thread needs
        let bg_matrix = std::sync::Arc::new(matrix);
        let bg_tagged = std::sync::Arc::new(tagged_words.clone());
        let bg_limit = limit;
        let bg_matrix_clone = std::sync::Arc::clone(&bg_matrix);
        let bg_tagged_clone = std::sync::Arc::clone(&bg_tagged);

        // background thread — processes compute requests
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

                // send results back — main thread discards stale versions
                let _ = result_tx.send(ComputeResult { version, results, matches });
            }
        });

        // extract matrix and tagged_words back from Arc for App storage
        let matrix = std::sync::Arc::try_unwrap(bg_matrix).unwrap_or_else(|a| (*a).clone());

        let mut app = App {
            mode:          Mode::Normal,
            active_pane:   Pane::Input,
            input:         initial_text.unwrap_or_default(),
            input_version: 0,
            results:       Vec::new(),
            results_state: TableState::default(),
            matches:       Vec::new(),
            matrix,
            tagged_words,
            limit,
            should_quit:   false,
            last_key_g:    false,
            last_typed:    None,
            tx:            work_tx,
            rx:            result_rx,
        };

        if !app.input.is_empty() {
            app.queue_recompute();
        }
        app
    }

    // queues work to background thread — does not block main thread
    fn queue_recompute(&mut self) {
        self.input_version += 1;
        self.last_typed = Some(Instant::now());
        // do not send yet — wait for 369ms delay
    }

    // called each frame — sends work if 369ms has elapsed since last keystroke
    fn tick(&mut self) {
        // check if 369ms has passed since last keystroke
        if let Some(typed_at) = self.last_typed {
            if typed_at.elapsed() >= Duration::from_millis(369) {
                let _ = self.tx.send((self.input_version, self.input.clone()));
                self.last_typed = None;
            }
        }

        // drain results from background thread
        while let Ok(result) = self.rx.try_recv() {
            // only apply if version matches current input
            if result.version == self.input_version {
                self.results = result.results;
                self.matches = result.matches;
                if !self.results.is_empty() {
                    self.results_state.select(Some(0));
                }
            }
            // stale results are silently discarded
        }
    }

    fn update​​​​​​​​​​​​​​​​
