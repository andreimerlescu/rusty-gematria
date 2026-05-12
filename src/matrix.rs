// src/matrix.rs
// indexes every dictionary word by its cipher values
// lookup: matrix.lookup(Cipher::English, 306) -> ["michael", ...]

use std::collections::HashMap;
use crate::cipher;
use crate::dictionary::{self, Language};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Cipher {
    English,
    Jewish,
    Simple,
    Mystery,
    Majestic,
    Eights,
}

pub struct Matrix {
    index: HashMap<Cipher, HashMap<u64, Vec<String>>>,
}

impl Matrix {
    pub fn build() -> Matrix {
        let mut index: HashMap<Cipher, HashMap<u64, Vec<String>>> = HashMap::new();

        for cipher in &[
            Cipher::English,
            Cipher::Jewish,
            Cipher::Simple,
            Cipher::Mystery,
            Cipher::Majestic,
            Cipher::Eights,
        ] {
            index.insert(cipher.clone(), HashMap::new());
        }

        for lang in &[
            Language::English,
            Language::Spanish,
            Language::Romanian,
            Language::French,
        ] {
            for word in dictionary::words(lang) {
                let g = cipher::calculate(word);

                let pairs = [
                    (Cipher::English,  g.english),
                    (Cipher::Jewish,   g.jewish),
                    (Cipher::Simple,   g.simple),
                    (Cipher::Mystery,  g.mystery),
                    (Cipher::Majestic, g.majestic),
                    (Cipher::Eights,   g.eights),
                ];

                for (cipher_key, value) in pairs {
                    index
                        .get_mut(&cipher_key)
                        .unwrap()
                        .entry(value)
                        .or_insert_with(Vec::new)
                        .push(word.to_string());
                }
            }
        }

        Matrix { index }
    }

    pub fn lookup(&self, cipher: &Cipher, value: u64) -> &[String] {
        self.index
            .get(cipher)
            .and_then(|m| m.get(&value))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn word_count(&self) -> usize {
        self.index
            .get(&Cipher::English)
            .map(|m| m.values().map(|v| v.len()).sum())
            .unwrap_or(0)
    }
}

// --- tests ---

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;

    // builds the matrix once and reuses it across all tests in this module
    // OnceLock is Rust's built-in "initialize exactly once" type
    // same concept as a singleton in Go using sync.Once
    fn shared_matrix() -> &'static Matrix {
        static MATRIX: OnceLock<Matrix> = OnceLock::new();
        MATRIX.get_or_init(Matrix::build)
    }

    #[test]
    fn matrix_builds_without_panic() {
        let m = shared_matrix();
        assert!(m.word_count() > 0, "matrix should contain words");
    }

    #[test]
    fn michael_found_at_306_english() {
        let m = shared_matrix();
        let words = m.lookup(&Cipher::English, 306);
        assert!(
            words.iter().any(|w| w == "michael"),
            "michael should be found at English value 306"
        );
    }

    #[test]
    fn empty_result_for_impossible_value() {
        let m = shared_matrix();
        let words = m.lookup(&Cipher::English, u64::MAX);
        assert!(words.is_empty(), "impossible value should return empty");
    }
}

