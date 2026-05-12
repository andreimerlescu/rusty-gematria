// src/main.rs

mod cipher;
mod dictionary;
mod matrix;
mod phrase;
mod textee;
mod tui;

use clap::Parser;
use dictionary::Language;
use matrix::Matrix;
use phrase::build_tagged_words;

// clap derive macro generates the CLI argument parser
// same concept as figtree in your Go projects
#[derive(Parser, Debug)]
#[command(
    name    = "rusty-gematria",
    version = "0.0.1",
    about   = "Gematria explorer with TUI interface",
    author  = "Andrei Merlescu"
)]
struct Args {
    // --file <path> - ingest a text file
    #[arg(short, long, help = "Path to a text file to analyze")]
    file: Option<String>,

    // --lang <code> - load a dictionary as the corpus
    #[arg(short, long, help = "Language dictionary to use: en, es, ro, fr")]
    lang: Option<String>,

    // --limit <int> - sliding window depth, default 3
    #[arg(short = 'L', long, default_value_t = 3, help = "Maximum phrase length in words (default: 3)")]
    limit: usize,

    // --delay <ms> - phrase suggestion rotation rate
    #[arg(short, long, default_value_t = 369, help = "Phrase suggestion rotation delay in ms (default: 369)")]
    delay: u64,
}

fn main() {
    let args = Args::parse();

    // resolve input text from --file, --lang, or freetext mode
    let initial_text: Option<String> = if let Some(path) = &args.file {
        // --file mode: read the file contents
        match std::fs::read_to_string(path) {
            Ok(contents) => Some(contents),
            Err(e) => {
                eprintln!("Error reading file '{}': {}", path, e);
                std::process::exit(1);
            }
        }
    } else if let Some(lang_code) = &args.lang {
        // --lang mode: load all words from the dictionary as a corpus
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
        // freetext mode: no initial text, user types in TUI
        None
    };

    // build the matrix from the prebuilt binary blob
    let matrix = Matrix::build();

    // build the tagged word list for the phrase engine
    let tagged_words = build_tagged_words();

    // run the TUI
    if let Err(e) = tui::run(matrix, tagged_words, args.limit, args.delay, initial_text) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
