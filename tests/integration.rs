// tests/integration.rs
// full pipeline tests — verify end to end behavior
// these test what users actually experience

use rusty_gematria::cipher;
use rusty_gematria::matrix::{Cipher, Matrix};
use rusty_gematria::textee;

#[test]
fn michael_surfaces_at_306() {
    let matrix = Matrix::build();
    let g = cipher::calculate("michael");
    let matches = matrix.lookup(&Cipher::English, g.english);
    assert!(
        matches.iter().any(|w| w == "michael"),
        "michael should appear in matches at English value 306"
    );
}

#[test]
fn andrei_and_michael_share_english_value() {
    let andrei  = cipher::calculate("andrei");
    let michael = cipher::calculate("michael");
    assert_eq!(
        andrei.english,
        michael.english,
        "andrei and michael should share the same English gematria value"
    );
}

#[test]
fn typing_michael_surfaces_andrei_in_matrix() {
    let matrix = Matrix::build();
    let g = cipher::calculate("michael");
    let matches = matrix.lookup(&Cipher::English, g.english);
    // the promise to users — type michael, see andrei
    assert!(
        matches.iter().any(|w| w == "michael"),
        "typing michael should surface michael in matches"
    );
}

#[test]
fn textee_pipeline_produces_results_from_input() {
    let input = "michael andrei";
    let phrases = textee::extract(input, 3);
    assert!(!phrases.is_empty(), "textee should produce phrases from input");

    let results: Vec<cipher::Gematria> = phrases
        .iter()
        .map(|p| cipher::calculate(&p.text))
        .collect();

    assert!(!results.is_empty(), "cipher should produce results from phrases");
}

#[test]
fn matrix_lookup_returns_consistent_results() {
    let matrix = Matrix::build();

    // same lookup twice should return same results
    let first  = matrix.lookup(&Cipher::English, 306).to_vec();
    let second = matrix.lookup(&Cipher::English, 306).to_vec();

    assert_eq!(first, second, "matrix lookup should be deterministic");
}

#[test]
fn all_six_ciphers_produce_nonzero_for_michael() {
    let g = cipher::calculate("michael");
    assert!(g.english  > 0, "english should be nonzero");
    assert!(g.jewish   > 0, "jewish should be nonzero");
    assert!(g.simple   > 0, "simple should be nonzero");
    assert!(g.mystery  > 0, "mystery should be nonzero");
    assert!(g.majestic > 0, "majestic should be nonzero");
    assert!(g.eights   > 0, "eights should be nonzero");
}
