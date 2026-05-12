// src/dictionary.rs
// loads word lists embedded at compile time from the data/ directory

// include_str! is like //go:embed but for a single file
// it reads the file at compile time and stores it as a &str inside the binary
// the path is relative to the project root, not to this source file
const ENGLISH_RAW:  &str = include_str!("../data/english.txt");
const FRENCH_RAW:   &str = include_str!("../data/french.txt");
const GERMAN_RAW:   &str = include_str!("../data/german.txt");
const ITALIAN_RAW:  &str = include_str!("../data/italian.txt");
const SPANISH_RAW:  &str = include_str!("../data/spanish.txt");

// Language is an enum - like Go's iota const block
// it represents all valid dictionary choices
#[derive(Debug, Clone, PartialEq)]
pub enum Language {
    English,
    French,
    German,
    Italian,
    Spanish,
}

// lets you convert a &str like "en" into a Language
// same pattern as a switch statement in Go
impl Language {
    pub fn from_flag(flag: &str) -> Option<Language> {
        match flag {
            "en" | "english" => Some(Language::English),
            "fr" | "french"  => Some(Language::French),
            "de" | "german"  => Some(Language::German),
            "it" | "italian" => Some(Language::Italian),
            "es" | "spanish" => Some(Language::Spanish),
            // _ is the default case, same as Go's default: in a switch
            _                => None,
        }
    }
}

// returns all words for a given language as a Vec of &str
// &'static str means the string lives for the entire program lifetime
// because it is embedded in the binary - it never gets freed
pub fn words(language: &Language) -> Vec<&'static str> {
    let raw = match language {
        Language::English => ENGLISH_RAW,
        Language::French  => FRENCH_RAW,
        Language::German  => GERMAN_RAW,
        Language::Italian => ITALIAN_RAW,
        Language::Spanish => SPANISH_RAW,
    };

    // split the raw text into lines, trim whitespace, drop empty lines
    // same as strings.Split() + a loop with strings.TrimSpace() in Go
    raw.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect()
}

// returns all words across all languages combined
pub fn all_words() -> Vec<&'static str> {
    let mut combined = Vec::new();
    for lang in &[
        Language::English,
        Language::French,
        Language::German,
        Language::Italian,
        Language::Spanish,
    ] {
        combined.extend(words(lang));
    }
    combined
}

// --- tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_dictionary_loads() {
        let w = words(&Language::English);
        assert!(!w.is_empty(), "english dictionary should not be empty");
    }

    #[test]
    fn all_languages_load() {
        let w = all_words();
        assert!(!w.is_empty(), "combined dictionary should not be empty");
    }

    #[test]
    fn language_from_flag() {
        assert_eq!(Language::from_flag("en"), Some(Language::English));
        assert_eq!(Language::from_flag("fr"), Some(Language::French));
        assert_eq!(Language::from_flag("xx"), None);
    }
}
