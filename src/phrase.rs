// src/phrase.rs
// generative sentence engine
// builds phrases from POS-tagged dictionary words whose
// combined cipher value matches a target value
// entropy ensures different sentences each session

use std::collections::HashMap;
use crate::cipher::{self, Gematria};
use crate::dictionary::{self, Language};
use crate::matrix::{Cipher, Matrix};

// part of speech tags
// each dictionary word gets one tag
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum POS {
    Article,
    Noun,
    Verb,
    Adjective,
    Adverb,
    Pronoun,
    Preposition,
    Conjunction,
    ProperNoun,
}

// a sentence template — ordered list of POS slots
#[derive(Debug, Clone)]
pub struct Template {
    pub slots: Vec<POS>,
}

// all supported templates from the PROMPT.html spec
pub fn templates() -> Vec<Template> {
    vec![
        // 3 word templates
        Template { slots: vec![POS::Article,   POS::Noun,        POS::Verb] },
        Template { slots: vec![POS::ProperNoun, POS::Verb,        POS::Adverb] },
        // 4 word templates
        Template { slots: vec![POS::Article,   POS::Adjective,   POS::Noun,        POS::Verb] },
        Template { slots: vec![POS::Article,   POS::Noun,        POS::Verb,        POS::Adverb] },
        Template { slots: vec![POS::Pronoun,   POS::Verb,        POS::Article,     POS::Noun] },
        // 5 word templates
        Template { slots: vec![POS::Adjective, POS::Noun,        POS::Verb,        POS::Adjective,   POS::Noun] },
        Template { slots: vec![POS::ProperNoun, POS::Verb,       POS::Preposition, POS::Article,     POS::Noun] },
        Template { slots: vec![POS::Article,   POS::Noun,        POS::Verb,        POS::Conjunction, POS::Verb] },
        // 6 word templates
        Template { slots: vec![POS::Article,   POS::Noun,        POS::Verb,        POS::Preposition, POS::Article, POS::Noun] },
    ]
}

// a word with its assigned POS tag and precomputed cipher values
#[derive(Debug, Clone)]
pub struct TaggedWord {
    pub text:     String,
    pub pos:      POS,
    pub gematria: Gematria,
}

// simple heuristic POS tagger
// not perfect — good enough for generative phrase matching
// real POS tagging requires a trained model; this uses word shape rules
fn guess_pos(word: &str) -> POS {
    let w = word.to_lowercase();

    // articles
    if matches!(w.as_str(), "a" | "an" | "the") {
        return POS::Article;
    }

    // pronouns
    if matches!(w.as_str(), "i" | "you" | "he" | "she" | "it" | "we" | "they" | "me" | "him" | "her" | "us" | "them") {
        return POS::Pronoun;
    }

    // conjunctions
    if matches!(w.as_str(), "and" | "but" | "or" | "nor" | "so" | "yet" | "for") {
        return POS::Conjunction;
    }

    // prepositions
    if matches!(w.as_str(), "in" | "on" | "at" | "by" | "for" | "with" | "about" | "against"
        | "between" | "through" | "during" | "before" | "after" | "above" | "below"
        | "from" | "to" | "of" | "into" | "over" | "under") {
        return POS::Preposition;
    }

    // adverbs — common ly-suffix heuristic
    if w.ends_with("ly") && w.len() > 4 {
        return POS::Adverb;
    }

    // verb heuristics — ing, ed, common verb endings
    if w.ends_with("ing") && w.len() > 5 {
        return POS::Verb;
    }
    if w.ends_with("ed") && w.len() > 4 {
        return POS::Verb;
    }

    // adjective heuristics — er, est, ful, less, ous, ive, al
    if w.ends_with("ful") || w.ends_with("less") || w.ends_with("ous")
        || w.ends_with("ive") || w.ends_with("al") && w.len() > 4 {
        return POS::Adjective;
    }

    // proper noun heuristic — starts with uppercase in original
    if word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
        return POS::ProperNoun;
    }

    // default to noun
    POS::Noun
}

// builds a tagged word list from all dictionary words
// called once at startup alongside Matrix::build()
pub fn build_tagged_words() -> Vec<TaggedWord> {
    let mut tagged = Vec::new();

    for lang in &[
        Language::English,
        Language::Spanish,
        Language::Romanian,
        Language::French,
    ] {
        for word in dictionary::words(lang) {
            let pos = guess_pos(word);
            let gematria = cipher::calculate(word);
            tagged.push(TaggedWord {
                text: word.to_string(),
                pos,
                gematria,
            });
        }
    }

    tagged
}

// a generated phrase with its combined cipher values
#[derive(Debug, Clone)]
pub struct GeneratedPhrase {
    pub words:    Vec<String>,
    pub text:     String,
    pub english:  u64,
}

// generates candidate phrases from a template whose English value
// matches the target — uses entropy for variety each session
// delay_ms controls rotation speed (reserved for future use)
pub fn generate(
    tagged_words: &[TaggedWord],
    target:       u64,
    cipher:       &Cipher,
    limit:        usize,
) -> Vec<GeneratedPhrase> {
    // build a POS index for fast lookup
    // pos -> list of words with that tag
    let mut pos_index: HashMap<&POS, Vec<&TaggedWord>> = HashMap::new();
    for tw in tagged_words {
        pos_index.entry(&tw.pos).or_insert_with(Vec::new).push(tw);
    }

    let tmpls = templates();
    let mut results = Vec::new();

    // use a simple entropy seed from current time
    // not cryptographic — just enough for session variety
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as usize;

    for template in &tmpls {
        if template.slots.len() > limit {
            continue;
        }

        // get candidate words for each slot
        let slot_candidates: Vec<Vec<&TaggedWord>> = template.slots.iter()
            .map(|pos| {
                pos_index.get(pos)
                    .map(|v| v.as_slice())
                    .unwrap_or(&[])
                    .to_vec()
            })
            .collect();

        // skip if any slot has no candidates
        if slot_candidates.iter().any(|c| c.is_empty()) {
            continue;
        }

        // try a limited number of random combinations
        // full combinatorial search would be too slow
        let attempts = 500;
        for i in 0..attempts {
            // pick one word per slot using entropy offset
            let chosen: Vec<&TaggedWord> = slot_candidates.iter()
                .enumerate()
                .map(|(slot_idx, candidates)| {
                    let idx = (seed.wrapping_add(i * 31).wrapping_add(slot_idx * 17)) % candidates.len();
                    candidates[idx]
                })
                .collect();

            // compute combined English value
            let total: u64 = chosen.iter().map(|tw| tw.gematria.english).sum();

            if total == target {
                let words: Vec<String> = chosen.iter().map(|tw| tw.text.clone()).collect();
                let text = words.join(" ");
                // avoid duplicates
                if !results.iter().any(|r: &GeneratedPhrase| r.text == text) {
                    results.push(GeneratedPhrase {
                        words,
                        text,
                        english: total,
                    });
                }
            }
        }
    }

    results
}

// --- tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pos_tagger_articles() {
        assert_eq!(guess_pos("the"), POS::Article);
        assert_eq!(guess_pos("a"),   POS::Article);
        assert_eq!(guess_pos("an"),  POS::Article);
    }

    #[test]
    fn pos_tagger_adverbs() {
        assert_eq!(guess_pos("quickly"), POS::Adverb);
        assert_eq!(guess_pos("slowly"),  POS::Adverb);
    }

    #[test]
    fn pos_tagger_pronouns() {
        assert_eq!(guess_pos("he"),   POS::Pronoun);
        assert_eq!(guess_pos("they"), POS::Pronoun);
    }

    #[test]
    fn templates_are_nonempty() {
        assert!(!templates().is_empty());
    }

    #[test]
    fn build_tagged_words_nonempty() {
        let words = build_tagged_words();
        assert!(!words.is_empty());
    }
}
