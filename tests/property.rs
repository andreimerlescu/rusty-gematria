// tests/property.rs
// property tests — verify invariants hold across many inputs
// add proptest to Cargo.toml to enable these

use rusty_gematria::cipher;
use rusty_gematria::textee;

// manual property tests without proptest crate
// these verify invariants across a range of inputs

#[test]
fn english_value_always_positive_for_alphabetic_input() {
    let words = vec![
        "a", "z", "hello", "world", "andrei", "michael",
        "rust", "gematria", "yeshua", "israel", "romania",
    ];
    for word in words {
        let g = cipher::calculate(word);
        assert!(g.english > 0, "english value should be positive for '{}'", word);
    }
}

#[test]
fn simple_value_equals_sum_of_positions() {
    // simple cipher is A=1 B=2 ... Z=26
    // verify manually for known word
    // a=1 n=14 d=4 r=18 e=5 i=9 = 51
    let g = cipher::calculate("andrei");
    assert_eq!(g.simple, 51, "simple value for andrei should be 51");
}

#[test]
fn english_value_is_multiple_of_six() {
    // english cipher is A=6 B=12 ... Z=156 — all multiples of 6
    // therefore any english value must be divisible by 6
    let words = vec!["andrei", "michael", "rust", "hello", "world"];
    for word in words {
        let g = cipher::calculate(word);
        assert_eq!(
            g.english % 6, 0,
            "english value for '{}' should be divisible by 6", word
        );
    }
}

#[test]
fn textee_limit_is_always_respected() {
    let input = "this is a long sentence with many words in it";
    for limit in 1..=6 {
        let phrases = textee::extract(input, limit);
        for phrase in &phrases {
            assert!(
                phrase.word_count <= limit,
                "phrase word count {} should not exceed limit {}",
                phrase.word_count, limit
            );
        }
    }
}

#[test]
fn textee_produces_single_words() {
    let input = "andrei michael";
    let phrases = textee::extract(input, 3);
    let texts: Vec<&str> = phrases.iter().map(|p| p.text.as_str()).collect();
    assert!(texts.contains(&"andrei"), "should contain single word andrei");
    assert!(texts.contains(&"michael"), "should contain single word michael");
}

#[test]
fn cipher_is_case_insensitive() {
    let lower = cipher::calculate("andrei");
    let upper = cipher::calculate("ANDREI");
    let mixed = cipher::calculate("AnDrEi");
    assert_eq!(lower.english, upper.english, "case should not affect english value");
    assert_eq!(lower.english, mixed.english, "case should not affect english value");
}

#[test]
fn empty_input_produces_zero_values() {
    let g = cipher::calculate("");
    assert_eq!(g.english,  0, "empty input should produce zero english");
    assert_eq!(g.simple,   0, "empty input should produce zero simple");
    assert_eq!(g.jewish,   0, "empty input should produce zero jewish");
    assert_eq!(g.mystery,  0, "empty input should produce zero mystery");
    assert_eq!(g.majestic, 0, "empty input should produce zero majestic");
    assert_eq!(g.eights,   0, "empty input should produce zero eights");
}

#[test]
fn spaces_do_not_affect_cipher_values() {
    let with_spaces    = cipher::calculate("and rei");
    let without_spaces = cipher::calculate("andrei");
    assert_eq!(
        with_spaces.english,
        without_spaces.english,
        "spaces should not affect english value"
    );
}
