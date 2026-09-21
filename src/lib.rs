//! Unicode-to-LaTeX encoding: turn text such as `Café — α ≤ β` into LaTeX
//! source such as `Caf\'e {\textemdash} \ensuremath{\alpha}
//! \ensuremath{\leq} \ensuremath{\beta}`, and learn what the document's
//! preamble must hold for it to print.
//!
//! LaTeX source is plain text, but a LaTeX document cannot always take a
//! character as it stands: an old installation reads no UTF-8, a font has no
//! glyph for the character, or the character is one of LaTeX's own special
//! characters (`%`, `&`, `#`). *Encoding* a string means replacing each such
//! character by the LaTeX that prints it, while leaving ordinary characters
//! alone. The design follows the `latexencode` module of
//! [pylatexenc](https://github.com/phfaist/pylatexenc), by the same author.
//!
//! The crate is `#![no_std]` (with [`alloc`]), and everything it does is
//! open: the rules, the protection strategy, the output sink, the report
//! sink, the input normalizer.
//!
//! # The encoder
//!
//! An [`Encoder`] holds a [`Rule`], a [`ReplacementProtection`] strategy, an
//! [`InputNormalizer`] and an [`UnknownCharPolicy`]; it encodes any number of
//! strings, and it is immutable and shareable. [`Encoder::encode`] answers a
//! new string, [`Encoder::encode_with_report`] adds what the output needs in
//! the preamble, and [`Encoder::encode_into`] appends to an output and a
//! report the caller owns — which is what a document made of many fragments
//! uses.
//!
//! ```
//! use untechxt::{DynTable, Encoder, ReplacementProtectionHint as Hint};
//!
//! let mut table = DynTable::new();
//! table.insert('\u{e9}', r"\'e", Hint::text_only(r"\'e"));
//! table.insert('\u{3b1}', r"\alpha", Hint::math_only(r"\alpha"));
//! let encoder = Encoder::new(table);
//! assert_eq!(encoder.encode("Caf\u{e9} \u{3b1}").unwrap(), r"Caf\'e \ensuremath{\alpha}");
//! ```
//!
//! # Rules
//!
//! A [`Rule`] is offered a position in the input and answers with the LaTeX
//! that replaces what it consumed there, or with "not mine". A
//! [`RuleChain`] tries several in order, and the first match wins. A lookup
//! table ([`LookupTable`], [`DynTable`]) is a rule; so is a closure, through
//! [`rule_fn`]; so is anything else that implements the trait. A rule placed
//! before a table overrides it, a rule placed after it fills in what the
//! table lacks.
//!
//! # Protection, and the two LaTeX modes
//!
//! A value cannot simply be pasted into the output: `\textemdash` followed
//! directly by a letter would read as a longer command name, and `\alpha` is
//! valid in mathematics alone. A rule therefore states what its value is —
//! its [`ReplacementProtectionHint`] — and the encoder's
//! [`ReplacementProtection`] strategy decides what to write around it.
//! [`StandardProtection`] wraps a math value in `\ensuremath{…}` when the
//! output is text (and a text value in `\textnormal{…}` when the output is
//! mathematics), and protects a value that ends with a named macro according
//! to [`MacroNameProtection`].
//!
//! # What the output needs
//!
//! LaTeX such as `\mathds{1}` prints nothing unless the document loads the
//! package `dsfont`. A rule may therefore hand out a [`Profile`]: the set of
//! preamble [`Chunk`]s its value needs. The encoder reports them to an
//! [`EncodeReporter`] — [`EncodeReport`] or [`PreambleNeeds`] keeps them,
//! [`NoReport`] drops them — and
//! [`PreambleNeeds::write_preamble`] writes the preamble out.

#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod asciiset;
pub mod chain;
pub mod encoder;
pub mod lookuptable;
pub mod normalizer;
pub mod outbuffer;
pub mod preamble;
pub mod profile;
pub mod replacement_protection;
pub mod report;
pub mod rule;
pub mod statictable;
pub mod unknown_char;

pub use crate::asciiset::AsciiSet;
pub use crate::chain::{DynRuleChain, LocalDynRuleChain, RuleChain, RuleList};
pub use crate::encoder::{EncodeError, Encoder};
pub use crate::lookuptable::{DynTable, ExceptAscii, LookupTable, OnlyAscii, TableEntry, TableRule};
pub use crate::normalizer::{nfc, InputNormalizer, NoNormalization, NormalizeNfc};
#[cfg(feature = "std")]
pub use crate::outbuffer::IoOut;
pub use crate::outbuffer::{FmtOut, OutBuffer};
pub use crate::preamble::{Chunk, ChunkPreamble};
pub use crate::profile::{PreambleNeeds, Profile, ProfileIndex};
pub use crate::replacement_protection::{
    BracesAroundAll, MacroNameProtection, ModeWrapper, OutputMode, ProtectInput,
    ReplacementProtection, ReplacementProtectionHint, StandardProtection, ValueMode,
    ValueTermination,
};
pub use crate::report::{EncodeReport, EncodeReporter, NoReport};
pub use crate::rule::{
    rule_fn, BoxError, EncodedReplacement, InvalidPrefixLength, Rule, RuleFn, RuleInput,
    RuleResult,
};
pub use crate::statictable::{
    StaticTableBinarySearch, StaticTableTwoLevelDirect, StaticTableTwoLevelLinear,
};
pub use crate::unknown_char::{unknown_unihex, UnknownCharPolicy};
