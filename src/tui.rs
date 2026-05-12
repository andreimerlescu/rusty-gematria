// src/tui.rs
// terminal UI for RustyGematria
// three panes: Input, Results, Matches
// vim keybindings throughout

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

use crate::cipher;
use crate::matrix::{Cipher, Matrix};
use crate::textee;

// which pane has focus
#[derive(Debug, Clone, PartialEq)]
enum Pane {
    Input,
    Results,
    Matches,
}

// vim modal editing state
#[derive(Debug, Clone, PartialEq)]
enum Mode {
    Normal,
    Insert,
}

// full application state
pub struct App {
    mode:          Mode,
    active_pane:   Pane,
    input:         String,
    // textee phrases computed from input
    phrases:       Vec<textee::Phrase>,
    // cipher results per phrase
    results:       Vec<cipher::Gematria>,
    results_state: TableState,
    // matrix matches for selected result
    matches:       Vec<String>,
    // the built matrix
    matrix:        Matrix,
    // sliding window limit from --limit flag
    limit:         usize,
    // signals main loop to exit
    should_quit:   bool,
    // gg double-press detection
    last_key_g:    bool,
}

impl App {
    pub fn new(matrix: Matrix, limit: usize) -> App {
        App {
            mode:          Mode::Normal,
            active_pane:   Pane::Input,
            input:         String::new(),
            phrases:       Vec::new(),
            results:       Vec::new(),
            results_state: TableState::default(),
            matches:       Vec::new(),
            matrix,
            limit,
            should_quit:   false,
            last_key_g:    false,
        }
    }

    // recompute phrases and cipher results from current input
    fn recompute(&mut self) {
        self.phrases = textee::extract(&self.input, self.limit);
        self.results = self.phrases
            .iter()
            .map(|p| cipher::calculate(&p.text))
            .collect();
        // reset selection
        if !self.results.is_empty() {
            self.results_state.select(Some(0));
            self.update_matches();
        } else {
            self.results_state.select(None);
            self.matches.clear();
        }
    }

    // populate Matches pane from currently selected Results row
    fn update_matches(&mut self) {
        if let Some(i) = self.results_state.selected() {
            if let Some(g) = self.results.get(i) {
                self.matches = self.matrix
                    .lookup(&Cipher::English, g.english)
                    .to_vec();
            }
        }
    }

    // handle a keypress in normal mode
    fn handle_normal(&mut self, key: KeyCode) {
        match key {
            // enter insert mode
            KeyCode::Char('i') => {
                self.mode = Mode::Insert;
                self.last_key_g = false;
            }

            // quit
            KeyCode::Char('q') => {
                self.should_quit = true;
            }

            // cycle panes with Tab
            KeyCode::Tab => {
                self.active_pane = match self.active_pane {
                    Pane::Input   => Pane::Results,
                    Pane::Results => Pane::Matches,
                    Pane::Matches => Pane::Input,
                };
                self.last_key_g = false;
            }

            // vim navigation in Results pane
            KeyCode::Char('j') => {
                if self.active_pane == Pane::Results {
                    let i = match self.results_state.selected() {
                        Some(i) => (i + 1).min(self.results.len().saturating_sub(1)),
                        None    => 0,
                    };
                    self.results_state.select(Some(i));
                    self.update_matches();
                }
                self.last_key_g = false;
            }

            KeyCode::Char('k') => {
                if self.active_pane == Pane::Results {
                    let i = match self.results_state.selected() {
                        Some(i) => i.saturating_sub(1),
                        None    => 0,
                    };
                    self.results_state.select(Some(i));
                    self.update_matches();
                }
                self.last_key_g = false;
            }

            // G jumps to bottom
            KeyCode::Char('G') => {
                if self.active_pane == Pane::Results && !self.results.is_empty() {
                    self.results_state.select(Some(self.results.len() - 1));
                    self.update_matches();
                }
                self.last_key_g = false;
            }

            // gg jumps to top
            KeyCode::Char('g') => {
                if self.last_key_g {
                    if self.active_pane == Pane::Results && !self.results.is_empty() {
                        self.results_state.select(Some(0));
                        self.update_matches();
                    }
                    self.last_key_g = false;
                } else {
                    self.last_key_g = true;
                }
            }

            // Enter selects and updates matches
            KeyCode::Enter => {
                if self.active_pane == Pane::Results {
                    self.update_matches();
                    self.active_pane = Pane::Matches;
                }
                self.last_key_g = false;
            }

            _ => {
                self.last_key_g = false;
            }
        }
    }

    // handle a keypress in insert mode
    fn handle_insert(&mut self, key: KeyCode) {
        match key {
            // exit insert mode
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }

            // backspace
            KeyCode::Backspace => {
                self.input.pop();
                self.recompute();
            }

            // any printable character
            KeyCode::Char(c) => {
                self.input.push(c);
                self.recompute();
            }

            _ => {}
        }
    }
}

// entry point called from main
pub fn run(matrix: Matrix, limit: usize) -> io::Result<()> {
    // set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(matrix, limit);

    loop {
        // draw frame
        terminal.draw(|frame| {
            let area = frame.area();
            draw(&mut app, frame, area);
        })?;

        // handle events
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                // ctrl-c always quits regardless of mode
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && key.code == KeyCode::Char('c')
                {
                    app.should_quit = true;
                } else {
                    match app.mode {
                        Mode::Normal => app.handle_normal(key.code),
                        Mode::Insert => app.handle_insert(key.code),
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

// draws the full UI
fn draw(app: &mut App, frame: &mut ratatui::Frame, area: Rect) {
    // split vertically into input, results, matches, statusbar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // input
            Constraint::Min(8),     // results
            Constraint::Length(5),  // matches
            Constraint::Length(1),  // status bar
        ])
        .split(area);

    draw_input(app, frame, chunks[0]);
    draw_results(app, frame, chunks[1]);
    draw_matches(app, frame, chunks[2]);
    draw_statusbar(app, frame, chunks[3]);
}

fn draw_input(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let border_style = if app.active_pane == Pane::Input {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let mode_indicator = match app.mode {
        Mode::Insert => " [INSERT]",
        Mode::Normal => "",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(format!("Input{}", mode_indicator));

    let text = Paragraph::new(app.input.as_str()).block(block);
    frame.render_widget(text, area);
}

fn draw_results(app: &mut App, frame: &mut ratatui::Frame, area: Rect) {
    let border_style = if app.active_pane == Pane::Results {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let header = Row::new(vec![
        Cell::from("Phrase"),
        Cell::from("Eng"),
        Cell::from("Jew"),
        Cell::from("Sim"),
        Cell::from("Mys"),
        Cell::from("Maj"),
        Cell::from("Eit"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = app.results.iter().map(|g| {
        Row::new(vec![
            Cell::from(g.original.clone()),
            Cell::from(g.english.to_string()),
            Cell::from(g.jewish.to_string()),
            Cell::from(g.simple.to_string()),
            Cell::from(g.mystery.to_string()),
            Cell::from(g.majestic.to_string()),
            Cell::from(g.eights.to_string()),
        ])
    }).collect();

    let table = Table::new(
        rows,
        [
            Constraint::Min(20),    // phrase
            Constraint::Length(6),  // eng
            Constraint::Length(6),  // jew
            Constraint::Length(6),  // sim
            Constraint::Length(8),  // mys
            Constraint::Length(6),  // maj
            Constraint::Length(6),  // eit
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title("Results"),
    )
    .row_highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_stateful_widget(table, area, &mut app.results_state);
}

fn draw_matches(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let border_style = if app.active_pane == Pane::Matches {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let selected_value = app.results_state
        .selected()
        .and_then(|i| app.results.get(i))
        .map(|g| format!(" — English {}", g.english))
        .unwrap_or_default();

    let content = if app.matches.is_empty() {
        Line::from(Span::raw("no matches"))
    } else {
        Line::from(app.matches.join("  "))
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(format!("Matches{}", selected_value));

    let text = Paragraph::new(content)
        .block(block)
        .wrap(ratatui::widgets::Wrap { trim: true });

    frame.render_widget(text, area);
}

fn draw_statusbar(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let mode = match app.mode {
        Mode::Normal => "NORMAL",
        Mode::Insert => "INSERT",
    };

    let text = Line::from(vec![
        Span::styled(
            format!(" {} ", mode),
            Style::default().fg(Color::Black).bg(Color::Yellow),
        ),
        Span::raw("  i=insert  Esc=normal  Tab=pane  j/k=navigate  G=bottom  gg=top  q=quit"),
    ]);

    frame.render_widget(Paragraph::new(text), area);
}
