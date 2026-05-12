mod cipher;
mod dictionary;
mod matrix;
mod phrase;
mod textee;
mod tui;

use matrix::Matrix;

fn main() {
    let limit = 3;
    let matrix = Matrix::build();

    if let Err(e) = tui::run(matrix, limit) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
