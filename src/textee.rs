// src/textee.rs
// word-level sliding window phrase decomposition
// ported from github.com/andreimerlescu/textee
// --limit controls maximum phrase length in words, default 3

// a single phrase extracted from the input with its position
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Phrase {
    pub text:       String,
    pub word_start: usize, // index of first word in this phrase
    pub word_count: usize, // how many words this phrase contains
}

// extracts all sliding window phrases up to limit words wide
// limit = 3 means phrases of 1, 2, and 3 words
pub fn extract(input: &str, limit: usize) -> Vec<Phrase> {
    // split on whitespace, filter empty tokens
    let words: Vec<&str> = input
        .split_whitespace()
        .collect();

    if words.is_empty() || limit == 0 {
        return Vec::new();
    }

    let mut phrases = Vec::new();

    // for each starting position in the word list
    for start in 0..words.len() {
        // for each window size from 1 up to limit
        // but never exceed remaining words
        let max_len = limit.min(words.len() - start);
        for len in 1..=max_len {
            let text = words[start..start + len].join(" ");
            phrases.push(Phrase {
                text,
                word_start: start,
                word_count: len,
            });
        }
    }

    phrases
}

// --- tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_single_words() {
        let phrases = extract("this is a", 3);
        let texts: Vec<&str> = phrases.iter().map(|p| p.text.as_str()).collect();
        assert!(texts.contains(&"this"));
        assert!(texts.contains(&"is"));
        assert!(texts.contains(&"a"));
    }

    #[test]
    fn extracts_two_word_phrases() {
        let phrases = extract("this is a", 3);
        let texts: Vec<&str> = phrases.iter().map(|p| p.text.as_str()).collect();
        assert!(texts.contains(&"this is"));
        assert!(texts.contains(&"is a"));
    }

    #[test]
    fn extracts_three_word_phrases() {
        let phrases = extract("this is a", 3);
        let texts: Vec<&str> = phrases.iter().map(|p| p.text.as_str()).collect();
        assert!(texts.contains(&"this is a"));
    }

    #[test]
    fn limit_controls_max_phrase_length() {
        let phrases = extract("this is a long", 2);
        // with limit 2, no phrase should have more than 2 words
        assert!(phrases.iter().all(|p| p.word_count <= 2));
    }

    #[test]
    fn empty_input_returns_empty() {
        let phrases = extract("", 3);
        assert!(phrases.is_empty());
    }

    #[test]
    fn limit_zero_returns_empty() {
        let phrases = extract("this is a", 0);
        assert!(phrases.is_empty());
    }
}
