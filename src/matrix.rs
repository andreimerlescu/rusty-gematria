// src/matrix.rs
// indexes every dictionary word by its cipher values
// the matrix is prebuilt at compile time by build.rs
// and embedded as a binary blob - startup is instant

use std::collections::HashMap;

// embed the prebuilt matrix binary at compile time
// concat! and env! are macros that resolve at compile time
// OUT_DIR is where build.rs wrote matrix.bin
static MATRIX_BYTES: &[u8] = include_bytes!(
    concat!(env!("OUT_DIR"), "/matrix.bin")
);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Cipher {
    English,
    Jewish,
    Simple,
    Mystery,
    Majestic,
    Eights,
}

impl Cipher {
    // converts enum to the string key used in the prebuilt matrix
    fn key(&self) -> &'static str {
        match self {
            Cipher::English  => "english",
            Cipher::Jewish   => "jewish",
            Cipher::Simple   => "simple",
            Cipher::Mystery  => "mystery",
            Cipher::Majestic => "majestic",
            Cipher::Eights   => "eights",
        }
    }
}

// the full index deserialized from the embedded binary
type MatrixIndex = HashMap<String, HashMap<u64, Vec<String>>>;

pub struct Matrix {
    index: MatrixIndex,
}

impl Matrix {
    // deserializes the prebuilt matrix from the embedded bytes
    // called once at startup - takes microseconds not seconds
    pub fn build() -> Matrix {
        let index: MatrixIndex = bincode::deserialize(MATRIX_BYTES)
            .expect("failed to deserialize prebuilt matrix");
        Matrix { index }
    }

    pub fn lookup(&self, cipher: &Cipher, value: u64) -> &[String] {
        self.index
            .get(cipher.key())
            .and_then(|m| m.get(&value))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    #[allow(dead_code)]
    pub fn word_count(&self) -> usize {
        self.index
            .get("english")
            .map(|m| m.values().map(|v| v.len()).sum())
            .unwrap_or(0)
    }
}

// --- tests ---

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;

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
