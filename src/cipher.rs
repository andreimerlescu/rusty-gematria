// src/cipher.rs

// no import needed - arrays and functions are stdlib

// same as Go's const block - evaluated entirely at compile time
// (usize is "unsigned integer, size decided by your CPU" - fine for indexes)
const ENGLISH: [u32; 26] = [
//   a   b   c   d   e   f   g   h   i   j   k   l   m
     6, 12, 18, 24, 30, 36, 42, 48, 54, 60, 66, 72, 78,
//   n   o   p   q   r   s   t   u   v   w   x   y   z
    84, 90, 96,102,108,114,120,126,132,138,144,150,156
];

// a struct is the same as Go - groups related values together
pub struct Gematria {
    pub english:  u32,
    pub jewish:   u32,
    pub simple:   u32,
    pub mystery:  u32,
    pub majestic: u32,
    pub eights:   u32,
}

// pub fn = public function, same as an exported func in Go (capital letter)
// &str = you are borrowing a string read-only, not owning it
// -> Gematria = returns a Gematria struct
pub fn calculate(input: &str) -> Gematria {
    Gematria {
        english:  compute(input, &ENGLISH),
        jewish:   0, // we fill these in next
        simple:   0,
        mystery:  0,
        majestic: 0,
        eights:   0,
    }
}

// private function - no pub, same as lowercase in Go
fn compute(input: &str, table: &[u32; 26]) -> u32 {
    input
        .to_lowercase()        // same as strings.ToLower()
        .chars()               // iterate over characters
        .filter_map(|c| {      // like Go's range but filters and maps in one step
            if c.is_ascii_alphabetic() {
                // 'a' as usize gives us 97, subtracting gives us 0-25 as index
                Some(table[c as usize - 'a' as usize])
            } else {
                None           // skip non-letters, like spaces and punctuation
            }
        })
        .sum()                 // add them all up
}
