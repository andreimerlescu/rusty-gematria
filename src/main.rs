mod cipher;
mod dictionary;
mod matrix;
mod textee;
mod tui;

use matrix::Matrix;

fn main() {
    // default limit for textee sliding window
    let limit = 3;

    // build the matrix once at startup
    let matrix = Matrix::build();

    // run the TUI
    if let Err(e) = tui::run(matrix, limit) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
