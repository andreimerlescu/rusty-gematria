// build.rs
// runs at compile time before the main program is compiled
// reads dictionary files, computes all cipher values,
// serializes the result into a binary blob embedded in the binary
// this means Matrix::build() at runtime is just a deserialization
// instead of iterating millions of words

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

// we cannot use crate:: here - build.rs is a separate program
// so we inline the cipher logic directly

// same values as src/cipher.rs
const ENGLISH: [u64; 26] = [
    6,  12,  18,  24,  30,  36,  42,  48,  54,  60,  66,  72,  78,
   84,  90,  96, 102, 108, 114, 120, 126, 132, 138, 144, 150, 156,
];

const SIMPLE: [u64; 26] = [
    1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13,
   14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
];

const MAJESTIC: [u64; 26] = [
    3,  6,  9, 12, 15, 18, 21, 24, 27, 30, 33, 36, 39,
   42, 45, 47, 51, 54, 57, 60, 63, 66, 69, 72, 75, 78,
];

const EIGHTS: [u64; 26] = [
    3,  5,  8,  16, 32, 40, 48, 54, 63, 69, 77, 80, 88,
   96, 104, 112, 120, 128, 136, 144, 152, 160, 168, 176, 184, 192,
];

fn jewish_table() -> HashMap<char, u64> {
    [
        ('a',1),   ('b',2),   ('c',3),   ('d',4),   ('e',5),   ('f',6),
        ('g',7),   ('h',8),   ('i',9),   ('j',600), ('k',10),  ('l',20),
        ('m',30),  ('n',40),  ('o',50),  ('p',60),  ('q',70),  ('r',80),
        ('s',90),  ('t',100), ('u',200), ('v',700), ('w',900), ('x',300),
        ('y',400), ('z',500),
    ].iter().cloned().collect()
}

fn mystery_table() -> HashMap<char, u64> {
    [
        ('a',369),  ('b',3),   ('c',144), ('d',6),   ('e',17),   ('f',9),
        ('g',22),   ('h',222), ('i',333), ('j',300), ('k',666),  ('l',600),
        ('m',963),  ('n',900), ('o',45),  ('p',47),  ('q',1776), ('r',639),
        ('s',1618), ('t',60),  ('u',999), ('v',434), ('w',99),   ('x',88),
        ('y',66),   ('z',33),
    ].iter().cloned().collect()
}

fn extended_value(c: char) -> Option<u64> {
    match c {
        'ă' => Some(3),  'â' => Some(6),   'î' => Some(9),
        'ș' => Some(12), 'ț' => Some(17),  'ş' => Some(21),
        'ţ' => Some(33), 'à' => Some(3),   'æ' => Some(144),
        'ç' => Some(6),  'è' => Some(9),   'é' => Some(12),
        'ê' => Some(17), 'ë' => Some(21),  'ï' => Some(33),
        'ô' => Some(45), 'œ' => Some(55),  'ù' => Some(66),
        'û' => Some(77), 'ü' => Some(88),  'á' => Some(3),
        'í' => Some(6),  'ó' => Some(9),   'ú' => Some(12),
        'ñ' => Some(17), 'ä' => Some(3),   'ö' => Some(6),
        'ß' => Some(9),  'ì' => Some(17),  'ò' => Some(76),
        _ => None,
    }
}

fn compute_array(input: &str, table: &[u64; 26]) -> u64 {
    input.chars().filter_map(|c| {
        if c.is_ascii_alphabetic() {
            Some(table[c as usize - 'a' as usize])
        } else {
            extended_value(c)
        }
    }).sum()
}

fn compute_table(input: &str, table: &HashMap<char, u64>) -> u64 {
    input.chars().filter_map(|c| {
        if c.is_ascii_alphabetic() {
            Some(*table.get(&c).unwrap_or(&0))
        } else {
            extended_value(c)
        }
    }).sum()
}

// the serializable matrix entry
// serde derives allow bincode to serialize this struct
#[allow(dead_code)]
#[derive(serde::Serialize, serde::Deserialize)]
struct MatrixEntry {
    english:  u64,
    jewish:   u64,
    simple:   u64,
    mystery:  u64,
    majestic: u64,
    eights:   u64,
}

// the full prebuilt matrix
// HashMap<cipher_name, HashMap<value, Vec<word>>>
type PrebuiltMatrix = HashMap<String, HashMap<u64, Vec<String>>>;

fn main() {
    // tell cargo to rerun build.rs if any dictionary file changes
    println!("cargo:rerun-if-changed=data/en.txt");
    println!("cargo:rerun-if-changed=data/es.txt");
    println!("cargo:rerun-if-changed=data/ro.txt");
    println!("cargo:rerun-if-changed=data/fr.txt");

    let jewish  = jewish_table();
    let mystery = mystery_table();

    let mut matrix: PrebuiltMatrix = HashMap::new();
    for cipher in &["english","jewish","simple","mystery","majestic","eights"] {
        matrix.insert(cipher.to_string(), HashMap::new());
    }

    // read each dictionary file relative to the project root
    // env::var("CARGO_MANIFEST_DIR") gives us the project root path
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let data_dir = Path::new(&manifest_dir).join("data");

    for filename in &["en.txt", "es.txt", "ro.txt", "fr.txt"] {
        let path = data_dir.join(filename);
        let content = match fs::read_to_string(&path) {
            Ok(c)  => c,
            Err(e) => {
                eprintln!("build.rs: could not read {:?}: {}", path, e);
                continue;
            }
        };

        for word in content.lines().map(|l| l.trim()).filter(|l| !l.is_empty()) {
            let lower = word.to_lowercase();

            let english  = compute_array(&lower, &ENGLISH);
            let jewish_v = compute_table(&lower, &jewish);
            let simple   = compute_array(&lower, &SIMPLE);
            let mystery_v= compute_table(&lower, &mystery);
            let majestic = compute_array(&lower, &MAJESTIC);
            let eights   = compute_array(&lower, &EIGHTS);

            let pairs = [
                ("english",  english),
                ("jewish",   jewish_v),
                ("simple",   simple),
                ("mystery",  mystery_v),
                ("majestic", majestic),
                ("eights",   eights),
            ];

            for (cipher_name, value) in &pairs {
                matrix
                    .get_mut(*cipher_name)
                    .unwrap()
                    .entry(*value)
                    .or_insert_with(Vec::new)
                    .push(word.to_string());
            }
        }
    }

    // serialize and write to OUT_DIR
    // OUT_DIR is a special directory Cargo provides for build output
    // it is unique per build configuration and target
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("matrix.bin");
    let encoded = bincode::serialize(&matrix).expect("build.rs: failed to serialize matrix");
    fs::write(&out_path, &encoded).expect("build.rs: failed to write matrix.bin");

    println!("cargo:rerun-if-changed=build.rs");
}
