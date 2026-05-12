// src/dictionary.rs

const ENGLISH_RAW:  &str = include_str!("../data/english.txt");
const SPANISH_RAW:  &str = include_str!("../data/spanish.txt");
const ROMANIAN_RAW: &str = include_str!("../data/romanian.txt");
const FRENCH_RAW:   &str = include_str!("../data/french.txt");

#[derive(Debug, Clone, PartialEq)]
pub enum Language {
    English,
    Spanish,
    Romanian,
    French,
}

impl Language {
    pub fn from_flag(flag: &str) -> Option<Language> {
        match flag {
            "en" | "english"  => Some(Language::English),
            "es" | "spanish"  => Some(Language::Spanish),
            "ro" | "romanian" => Some(Language::Romanian),
            "fr" | "french"   => Some(Language::French),
            _                 => None,
        }
    }
}

pub fn words(language: &Language) -> Vec<&'static str> {
    let raw = match language {
        Language::English  => ENGLISH_RAW,
        Language::Spanish  => SPANISH_RAW,
        Language::Romanian => ROMANIAN_RAW,
        Language::French   => FRENCH_RAW,
    };

    raw.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect()
}

pub fn all_words() -> Vec<&'static str> {
    let mut combined = Vec::new();
    for lang in &[
        Language::English,
        Language::Spanish,
        Language::Romanian,
        Language::French,
    ] {
        combined.extend(words(lang));
    }
    combined
}

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
        assert_eq!(Language::from_flag("ro"), Some(Language::Romanian));
        assert_eq!(Language::from_flag("xx"), None);
    }
}
