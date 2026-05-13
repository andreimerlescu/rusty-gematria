// tests/threading.rs
// verify channel and threading behavior
// these test what unit tests cannot — timing and concurrency

use rusty_gematria::cipher;
use rusty_gematria::matrix::{Cipher, Matrix};
use rusty_gematria::textee;
use rusty_gematria::phrase;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn background_thread_sends_results() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let g = cipher::calculate("andrei");
        tx.send(g.english).unwrap();
    });

    let value = rx.recv_timeout(Duration::from_secs(1)).unwrap();
    assert_eq!(value, 306, "background thread should send correct english value");
}

#[test]
fn channel_delivers_results_within_reasonable_time() {
    let (tx, rx) = mpsc::channel();
    let start = Instant::now();

    thread::spawn(move || {
        let matrix = Matrix::build();
        let g = cipher::calculate("michael");
        let matches = matrix.lookup(&Cipher::English, g.english).to_vec();
        tx.send(matches).unwrap();
    });

    let matches = rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let elapsed = start.elapsed();

    assert!(!matches.is_empty(), "should receive matches from background thread");
    assert!(elapsed < Duration::from_secs(10), "should complete within 10 seconds");
}

#[test]
fn stale_results_are_identified_by_version() {
    let (tx, rx) = mpsc::channel::<(u64, u64)>();

    // simulate two work items sent — version 1 and version 2
    // only version 2 should be applied
    let current_version: u64 = 2;

    thread::spawn(move || {
        // stale result
        tx.send((1, cipher::calculate("old").english)).unwrap();
        // current result
        tx.send((2, cipher::calculate("michael").english)).unwrap();
    });

    let mut applied_value = 0u64;
    while let Ok((version, value)) = rx.recv_timeout(Duration::from_secs(1)) {
        if version == current_version {
            applied_value = value;
        }
    }

    assert_eq!(applied_value, 306, "only current version result should be applied");
}

#[test]
fn debounce_delay_is_respected() {
    // verify that 369ms is a meaningful delay
    // not testing the TUI directly — testing the timing concept
    let delay = Duration::from_millis(369);
    let start = Instant::now();

    thread::sleep(delay);

    let elapsed = start.elapsed();
    assert!(
        elapsed >= Duration::from_millis(360),
        "369ms delay should be at least 360ms"
    );
}

#[test]
fn multiple_threads_can_read_matrix_simultaneously() {
    use std::sync::Arc;

    let matrix = Arc::new(Matrix::build());
    let mut handles = Vec::new();

    for _ in 0..10 {
        let m = Arc::clone(&matrix);
        handles.push(thread::spawn(move || {
            let results = m.lookup(&Cipher::English, 306);
            assert!(!results.is_empty());
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}
