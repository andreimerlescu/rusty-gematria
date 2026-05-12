// src/matrix.rs
// indexes every dictionary word by its cipher values
// lookup: matrix.lookup(Cipher::English, 306) -> ["andrei", "michael", ...]

use std::collections::HashMap;
use crate::cipher;
use crate::dictionary::{self, Language};

// Cipher is an enum so call sites are explicit and compiler-checked
// no magic strings, no typos possible
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Cipher {
    English,
    Jewish,
    Simple,
    Mystery,
    Majestic,
    Eights,
}

// the full index
// outer key: which cipher
// inner key: the numeric value
// value: list of words that produce that value under that cipher
pub struct Matrix {
    index: HashMap<Cipher, HashMap<u64, Vec<String>>>,
}

impl Matrix {
    // builds the index from all dictionary words across all languages
    // called once at startup
    pub fn build() -> Matrix {
        let mut index: HashMap<Cipher, HashMap<u64, Vec<String>>> = HashMap::new();

        // initialize an empty bucket for each cipher
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

        // iterate every word across all four languages
        for lang in &[
            Language::English,
            Language::Spanish,
            Language::Romanian,
            Language::French,
        ] {
            for word in dictionary::words(lang) {
                // skip words with non-ascii characters for now
                // romanian has diacritics - we handle that in a later step
                if !word.is_ascii() {
                    continue;
                }

                let g = cipher::calculate(word);

                // insert word into the bucket for each cipher value
                // entry() is like Go's map[key] with a default value pattern
                // or_insert_with creates the Vec only if the key is missing
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

    // returns all words matching a cipher + value combination
    // returns empty slice if nothing matches
    pub fn lookup(&self, cipher: &Cipher, value: u64) -> &[String] {
        self.index
            .get(cipher)
            .and_then(|m| m.get(&value))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    // returns total number of words indexed
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

    #[test]
    fn matrix_builds_without_panic() {
        let m = Matrix::build();
        assert!(m.word_count() > 0, "matrix should contain words");
    }

    
    #[test]
    fn andrei_equals_306_english() {
        let result = calculate("andrei");
        assert_eq!(result.english, 306);
    }

    #[test]
    fn michael_found_at_306_english() {
        let m = Matrix::build();
        let words = m.lookup(&Cipher::English, 306);
        assert!(
            words.iter().any(|w| w == "michael"),
            "michael should be found at English value 306"
        );
    }

    #[test]
    fn empty_result_for_impossible_value() {
        let m = Matrix::build();
        // u64::MAX is impossible to reach by summing cipher values
        let words = m.lookup(&Cipher::English, u64::MAX);
        assert!(words.is_empty(), "impossible value should return empty");
    }
}
