//! Encodes its argument with the default rules and no input normalization.
//!
//! This exists to be measured, not to be read: it is the smallest program
//! that pulls in the encoder, so `cargo bloat` and `cargo asm` over it show
//! what a user's binary actually pays for. Its twin,
//! `examples/size_check_nfc.rs`, is the same program with the default
//! `NormalizeNfc` normalizer, and the difference between the two is what the
//! `unicode-normalization` tables cost.
//!
//! ```text
//! cargo bloat --release --example size_check -n 20
//! cargo bloat --release --example size_check_nfc -n 20
//! cargo asm --release --example size_check --everything
//! ```
//!
//! The input comes from the command line so that nothing can be folded away
//! at compile time.

use untechxt::normalizer::NoNormalization;
use untechxt::{default_rules, Encoder};

fn main() {
    let text = std::env::args().nth(1).unwrap_or_else(|| "Caf\u{e9} \u{3b1} \u{2264} \u{3b2}".into());
    // `encode` reports into `NoReport`, so the needs logic has nothing to do.
    let encoder = Encoder::new(default_rules()).with_normalizer(NoNormalization);
    let encoded = encoder.encode(&text).unwrap();
    println!("{encoded}");
}
