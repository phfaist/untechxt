//! Encodes its argument with the default rules and the default NFC
//! normalizer.
//!
//! The twin of `examples/size_check.rs`, which is the same program with
//! `NoNormalization`; see that file for what the pair is for and how to
//! measure them.

use untechxt::{default_rules, Encoder};

fn main() {
    let text = std::env::args().nth(1).unwrap_or_else(|| "Caf\u{e9} \u{3b1} \u{2264} \u{3b2}".into());
    // `encode` reports into `NoReport`, so the needs logic has nothing to do.
    let encoded = Encoder::new(default_rules()).encode(&text).unwrap();
    println!("{encoded}");
}
