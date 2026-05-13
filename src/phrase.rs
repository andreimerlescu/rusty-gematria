// src/phrase.rs
// generative sentence engine
// phrase generation is parallel via rayon

use std::collections::HashMap;
use rayon::prelude::*;
use crate::cipher::{self, Gematria};
use crate::dictionary::{self, Language};
use crate::matrix::Cipher;

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

#[derive(Debug, Clone)]
pub struct Template {
    pub slots: Vec<POS>,
}

pub fn templates() -> Vec<Template> {
    vec![
        Template { slots: vec![POS::Article,    POS::Noun,        POS::Verb] },
        Template { slots: vec![POS::ProperNoun,  POS::Verb,        POS::Adverb] },
        Template { slots: vec![POS::Article,    POS::Adjective,   POS::Noun,        POS::Verb] },
        Template { slots: vec![POS::Article,    POS::Noun,        POS::Verb,        POS::Adverb] },
        Template { slots: vec![POS::Pronoun,    POS::Verb,        POS::Article,     POS::Noun] },
        Template { slots: vec![POS::Adjective,  POS::Noun,        POS::Verb,        POS::Adjective,   POS::Noun] },
        Template { slots: vec![POS::ProperNoun,  POS::Verb,       POS::Preposition, POS::Article,     POS::Noun] },
        Template { slots: vec![POS::Article,    POS::Noun,        POS::Verb,        POS::Conjunction, POS::Verb] },
        Template { slots: vec![POS::Article,    POS::Noun,        POS::Verb,        POS::Preposition, POS::Article, POS::Noun] },
    ]
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TaggedWord {
    pub text:     String,
    pub pos:      POS,
    pub gematria: Gematria,
}

fn guess_pos(word: &str) -> POS {
    let w = word.to_lowercase();

    if matches!(w.as_str(), "a" | "an" | "the") {
        return POS::Article;
    }
    if matches!(w.as_str(), "i" | "you" | "he" | "she" | "it" | "we" | "they" | "me" | "him" | "her" | "us" | "them") {
        return POS::Pronoun;
    }
    if matches!(w.as_str(), "and" | "but" | "or" | "nor" | "so" | "yet" | "for") {
        return POS::Conjunction;
    }
    if matches!(w.as_str(), "in" | "on" | "at" | "by" | "for" | "with" | "about" | "against"
        | "between" | "through" | "during" | "before" | "after" | "above" | "below"
        | "from" | "to" | "of" | "into" | "over" | "under") {
        return POS::Preposition;
    }
    if w.ends_with("ly") && w.len() > 4 {
        return POS::Adverb;
    }
    if w.ends_with("ing") && w.len() > 5 {
        return POS::Verb;
    }
    if w.ends_with("ed") && w.len() > 4 {
        return POS::Verb;
    }
    if w.ends_with("ful") || w.ends_with("less") || w.ends_with("ous")
        || w.ends_with("ive") || w.ends_with("al") && w.len() > 4 {
        return POS::Adjective;
    }
    if word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
        return POS::ProperNoun;
    }

    POS::Noun
}

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

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GeneratedPhrase {
    pub words:   Vec<String>,
    pub text:    String,
    pub english: u64,
}

pub fn generate(
    tagged_words: &[TaggedWord],
    target:       u64,
    _cipher:      &Cipher,
    limit:        usize,
) -> Vec<GeneratedPhrase> {
    // build pos index with owned Strings so rayon can safely share across threads
    let mut pos_index: HashMap<String, Vec<(String, u64)>> = HashMap::new();
    for tw in tagged_words {
        let key = format!("{:?}", tw.pos);
        pos_index
            .entry(key)
            .or_insert_with(Vec::new)
            .push((tw.text.clone(), tw.gematria.english));
    }

    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as usize;

    let tmpls = templates();

    // each template processed in parallel — owned data, no lifetime issues
    tmpls
        .par_iter()
        .filter(|template| template.slots.len() <= limit)
        .flat_map(|template| {
            let slot_candidates: Vec<Vec<(String, u64)>> = template.slots.iter()
                .map(|pos| {
                    let key = format!("{:?}", pos);
                    pos_index.get(&key)
                        .cloned()
                        .unwrap_or_default()
                })
                .collect();

            if slot_candidates.iter().any(|c| c.is_empty()) {
                return vec![];
            }

            let mut results: Vec<GeneratedPhrase> = Vec::new();

            for i in 0..500 {
                let chosen: Vec<&(String, u64)> = slot_candidates.iter()
                    .enumerate()
                    .map(|(slot_idx, candidates)| {
                        let idx = (seed.wrapping_add(i * 31).wrapping_add(slot_idx * 17)) % candidates.len();
                        &candidates[idx]
                    })
                    .collect();

                let total: u64 = chosen.iter().map(|(_, v)| v).sum();

                if total == target {
                    let words: Vec<String> = chosen.iter().map(|(w, _)| w.clone()).collect();
                    let text = words.join(" ");
                    if !results.iter().any(|r| r.text == text) {
                        results.push(GeneratedPhrase {
                            words,
                            text,
                            english: total,
                        });
                    }
                }
            }
            results
        })
        .collect()
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
