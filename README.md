# RustyGematria

A gematria exploration tool built in Rust — for those who prefer correctness over simplicity.

Compute, explore, and discover gematria cipher values across six ciphers and four languages,
with a full terminal UI, vim bindings, and generative phrase matching powered by embedded
multilingual dictionaries.

---

## What is Gematria

Gematria is the practice of assigning numerical values to letters and words, rooted in
Jewish mysticism and used across centuries of religious, philosophical, and esoteric study.
This tool implements six cipher systems:

| Cipher   | Description |
|----------|-------------|
| English  | A=6, B=12 ... Z=156 |
| Jewish   | Classical Hebrew-derived values including J=600, V=700, W=900 |
| Simple   | A=1, B=2 ... Z=26 |
| Mystery  | A personal cipher — Life is a mystery |
| Majestic | A=3, B=6 ... following the 3+6=9 pattern |
| Eights   | Based on the numerology theory of 3+5=8 |

---

## Features

- Six cipher engines computed simultaneously per word or phrase
- Four embedded languages — English, Spanish, Romanian, French
- Sliding window phrase decomposition via the textee algorithm
- Compile-time matrix — all cipher values prebuilt at cargo build, zero startup latency
- Generative phrase engine — produces novel phrases matching your target value each session
- Full terminal UI with vim keybindings
- Three input modes — file, dictionary corpus, or freetext editor
- Single static binary — no runtime dependencies, no installation required

---

## Installation

### From source

    git clone https://github.com/andreimerlescu/rusty-gematria
    cd rusty-gematria
    cargo build --release
    ./target/release/rusty-gematria

### Requirements

- Rust 1.95+ (install via rustup — https://rustup.rs)

---

## Usage

    rusty-gematria [OPTIONS]

    Options:
      -f, --file <FILE>     Path to a text file to analyze
      -l, --lang <LANG>     Language dictionary corpus: en, es, ro, fr
      -L, --limit <INT>     Maximum phrase length in words [default: 3]
      -d, --delay <MS>      Phrase suggestion rotation delay in ms [default: 369]
      -h, --help            Print help
      -V, --version         Print version

### Examples

    # freetext mode — opens TUI, type to explore
    rusty-gematria

    # analyze a document
    rusty-gematria --file document.txt

    # explore the Romanian dictionary
    rusty-gematria --lang ro

    # deeper phrase analysis
    rusty-gematria --file document.txt --limit 6

---

## TUI Keybindings

    i         enter insert mode in Input pane
    Esc       exit insert mode, return to normal mode
    Tab       cycle between panes (Input → Results → Matches)
    j / k     scroll down / up in Results
    G         jump to bottom of Results
    gg        jump to top of Results
    Enter     select row, populate Matches pane
    q         quit
    Ctrl-C    force quit

---

## How It Works

**Input** — type freely, paste text, or load a file. Every keystroke updates the Results pane.

**Results** — every sliding window phrase extracted from your input, with all six cipher
values displayed per row. Navigate with j/k.

**Matches** — when you select a phrase, this pane shows every dictionary word that shares
its English gematria value, plus generated phrases from the phrase engine. Generated phrases
appear in [brackets].

The phrase engine samples from POS-tagged dictionary words to construct grammatically
structured sentences whose combined cipher value matches your target. Every session produces
different results from the same corpus — entropy-driven but deterministic in what is possible.

---

## Architecture

    build.rs          — compile-time matrix builder, runs once during cargo build
    src/cipher.rs     — six cipher engines, extended character support for 4 languages
    src/dictionary.rs — four embedded language dictionaries (en, es, ro, fr)
    src/matrix.rs     — prebuilt value index, O(1) lookup by cipher + value
    src/textee.rs     — sliding window phrase decomposition
    src/phrase.rs     — POS tagger and generative phrase engine
    src/tui.rs        — ratatui terminal UI with vim bindings
    src/main.rs       — clap CLI, startup wiring

The matrix is built at compile time by build.rs and embedded as a binary blob via
include_bytes!. At runtime Matrix::build() is a deserialization — microseconds, not seconds.

---

## Languages

The extended cipher table assigns individual values to non-ASCII characters across all
supported languages. Each symbol carries its own cipher value distinct from its base letter.

| Language | Characters            |
|----------|-----------------------|
| Romanian | ă â î ș ț ş ţ         |
| French   | à æ ç è é ê ë ï ô œ ù û ü |
| Spanish  | á í ó ú ñ             |
| German   | ä ö ü ß               |
| Italian  | ì ò                   |

---

## Related Projects

- [cli-gematria](https://github.com/andreimerlescu/cli-gematria) — the Go version this replaces
- [gematria](https://github.com/andreimerlescu/gematria) — the Go cipher library
- [textee](https://github.com/andreimerlescu/textee) — the sliding window algorithm
- [genwordpass](https://github.com/andreimerlescu/genwordpass) — the dictionary source
