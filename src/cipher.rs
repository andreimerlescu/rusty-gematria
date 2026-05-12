// src/cipher.rs
// ported exactly from github.com/andreimerlescu/gematria

use std::collections::HashMap;

// --- simple array ciphers (a=index 0, z=index 25) ---

const ENGLISH: [u64; 26] = [
//  a    b    c    d    e    f    g    h    i    j    k    l    m
    6,  12,  18,  24,  30,  36,  42,  48,  54,  60,  66,  72,  78,
//  n    o    p    q    r    s    t    u    v    w    x    y    z
   84,  90,  96, 102, 108, 114, 120, 126, 132, 138, 144, 150, 156,
];

const SIMPLE: [u64; 26] = [
//  a   b   c   d   e   f   g   h   i   j   k   l   m
    1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13,
//  n   o   p   q   r   s   t   u   v   w   x   y   z
   14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
];

const MAJESTIC: [u64; 26] = [
//  a   b   c   d   e   f   g   h   i   j   k   l   m
    3,  6,  9, 12, 15, 18, 21, 24, 27, 30, 33, 36, 39,
//  n   o   p   q   r   s   t   u   v   w   x   y   z
   42, 45, 47, 51, 54, 57, 60, 63, 66, 69, 72, 75, 78,
];

const EIGHTS: [u64; 26] = [
//  a   b   c    d   e   f   g   h   i   j   k   l   m
    3,  5,  8,  16, 32, 40, 48, 54, 63, 69, 77, 80, 88,
//   n    o    p    q    r    s    t    u    v    w    x    y    z
   96, 104, 112, 120, 128, 136, 144, 152, 160, 168, 176, 184, 192,
];

// --- lookup table ciphers (non-sequential values) ---

fn jewish_table() -> HashMap<char, u64> {
    [
        ('a',1),   ('b',2),   ('c',3),   ('d',4),   ('e',5),   ('f',6),
        ('g',7),   ('h',8),   ('i',9),   ('j',600), ('k',10),  ('l',20),
        ('m',30),  ('n',40),  ('o',50),  ('p',60),  ('q',70),  ('r',80),
        ('s',90),  ('t',100), ('u',200), ('v',700), ('w',900), ('x',300),
        ('y',400), ('z',500),
    ]
    .iter()
    .cloned()
    .collect()
}

// mysteryCodes - numerology of Saint Andrei
fn mystery_table() -> HashMap<char, u64> {
    [
        ('a',369),  ('b',3),   ('c',144), ('d',6),   ('e',17),   ('f',9),
        ('g',22),   ('h',222), ('i',333), ('j',300), ('k',666),  ('l',600),
        ('m',963),  ('n',900), ('o',45),  ('p',47),  ('q',1776), ('r',639),
        ('s',1618), ('t',60),  ('u',999), ('v',434), ('w',99),   ('x',88),
        ('y',66),   ('z',33),
    ]
    .iter()
    .cloned()
    .collect()
}

// --- main struct, matches your Go Gematria struct exactly ---

#[derive(Debug)]
pub struct Gematria {
    pub original: String,
    pub english:  u64,
    pub jewish:   u64,
    pub simple:   u64,
    pub mystery:  u64,
    pub majestic: u64,
    pub eights:   u64,
}

fn normalize(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            'ă' | 'â' => 'a',
            'î' => 'i',
            'ș' | 'ş' => 's', // two versions exist in Unicode - both covered
            'ț' | 'ţ' => 't', // same here
            _ => c,
        })
        .collect()
}

fn extended_value(c: char) -> Option<u64> {
    match c {
        // romanian
        'ă' => Some(3),
        'â' => Some(6),
        'î' => Some(9),
        'ș' => Some(12),
        'ț' => Some(17),
        'ş' => Some(21), // cedilla variant - older texts
        'ţ' => Some(33), // cedilla variant - older texts

        // french
        'à' => Some(3),
        'æ' => Some(144),
        'ç' => Some(6),
        'è' => Some(9),
        'é' => Some(12),
        'ê' => Some(17),
        'ë' => Some(21),
        'ï' => Some(33),
        'ô' => Some(45),
        'œ' => Some(55),
        'ù' => Some(66),
        'û' => Some(77),
        'ü' => Some(88),

        // spanish
        'á' => Some(3),
        'í' => Some(6),
        'ó' => Some(9),
        'ú' => Some(12),
        'ñ' => Some(17),

        // german
        'ä' => Some(3),
        'ö' => Some(6),
        'ß' => Some(9),

        // italian
        'ì' => Some(17),
        'ò' => Some(76),

        // no match
        _ => None,
    }
}


// --- the calculator ---

pub fn calculate(input: &str) -> Gematria {
    let lower = input.to_lowercase();
    Gematria {
        original: input.to_string(),
        english:  compute_array(&lower, &ENGLISH),
        simple:   compute_array(&lower, &SIMPLE),
        majestic: compute_array(&lower, &MAJESTIC),
        eights:   compute_array(&lower, &EIGHTS),
        jewish:   compute_table(&lower, &jewish_table()),
        mystery:  compute_table(&lower, &mystery_table()),
    }
}

// handles array-based ciphers
fn compute_array(input: &str, table: &[u64; 26]) -> u64 {
    input
        .chars()
        .filter_map(|c| {
            if c.is_ascii_alphabetic() {
                Some(table[c as usize - 'a' as usize])
            } else {
                extended_value(c)
            }
        })
        .sum()
}

// handles HashMap-based ciphers
fn compute_table(input: &str, table: &HashMap<char, u64>) -> u64 {
    input
        .chars()
        .filter_map(|c| {
            if c.is_ascii_alphabetic() {
                Some(*table.get(&c).unwrap_or(&0))
            } else {
                extended_value(c)
            }
        })
        .sum()
}

// --- tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn andrei_equals_306_english() {
        let result = calculate("andrei");
        assert_eq!(result.english, 306);
    }

    #[test]
    fn michael_equals_306_english() {
        let result = calculate("michael");
        assert_eq!(result.english, 306);
    }
}
