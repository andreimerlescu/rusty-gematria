// src/main.rs

mod cipher;
mod dictionary;
mod matrix;
mod phrase;
mod textee;
mod tui;
mod gui;

use clap::Parser;
use dictionary::Language;
use matrix::Matrix;
use phrase::build_tagged_words;

#[derive(Parser, Debug)]
#[command(
    name    = "rusty-gematria",
    version = "0.0.1",
    about   = "Gematria explorer with TUI and GUI interface",
    author  = "Andrei Merlescu"
)]
struct Args {
    #[arg(short, long, help = "Path to a text file to analyze")]
    file: Option<String>,

    #[arg(short, long, help = "Language dictionary to use: en, es, ro, fr")]
    lang: Option<String>,

    #[arg(short = 'L', long, default_value_t = 3, help = "Maximum phrase length in words (default: 3)")]
    limit: usize,

    #[arg(short, long, default_value_t = 369, help = "Phrase suggestion rotation delay in ms (default: 369)")]
    delay: u64,

    // launch native GUI instead of terminal TUI
    #[arg(short, long, help = "Launch graphical interface instead of TUI")]
    gui: bool,
}

fn main() {
    let args = Args::parse();

    let initial_text: Option<String> = if let Some(path) = &args.file {
        match std::fs::read_to_string(path) {
            Ok(contents) => Some(contents),
            Err(e) => {
                eprintln!("Error reading file '{}': {}", path, e);
                std::process::exit(1);
            }
        }
    } else if let Some(lang_code) = &args.lang {
        match Language::from_flag(lang_code) {
            Some(lang) => {
                let words = dictionary::words(&lang);
                Some(words.join(" "))
            }
            None => {
                eprintln!("Unknown language '{}'. Supported: en, es, ro, fr", lang_code);
                std::process::exit(1);
            }
        }
    } else {
        None
    };

    let matrix = Matrix::build();
    let tagged_words = build_tagged_words();

    if args.gui {
        if let Err(e) = gui::run(matrix, tagged_words, args.limit, args.delay, initial_text) {
            eprintln!("GUI Error: {}", e);
            std::process::exit(1);
        }
    } else {
        if let Err(e) = tui::run(matrix, tagged_words, args.limit, args.delay, initial_text) {
            eprintln!("TUI Error: {}", e);
            std::process::exit(1);
        }
    }
}
