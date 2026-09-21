//! Unicode-to-LaTeX encoding: turn text such as `Café — α ≤ β` into LaTeX
//! source such as `Caf\'e {\textemdash} \ensuremath{\alpha}
//! \ensuremath{\leq} \ensuremath{\beta}`, and learn what the document's
//! preamble must hold for it to print.
//!
//! LaTeX source is plain text, but a LaTeX document cannot always take a
//! character as it stands: an old installation reads no UTF-8, a font has no
//! glyph for the character, or the character is one of LaTeX's own special
//! characters (`%`, `&`, `#`). *Encoding* a string means replacing each such
//! character by the LaTeX that prints it — the **encoded** value — while
//! leaving ordinary characters alone.
//!
//! [`encode`] does that under every default setting: the builtin table of
//! 1549 characters, each encoded value protected so that it cannot merge
//! with the text that follows it, and a character no rule knows kept as it
//! is.
//!
//! ```
//! use untechxt::encode;
//!
//! assert_eq!(encode("Café — naïve"), r#"Caf\'e {\textemdash} na\"ive"#);
//! assert_eq!(encode("100% & more"), r"100\% \& more");
//! ```
//!
//! Everything in between is open: the rules that match the input, the
//! protection written around a value, the policy for a character no rule
//! knows, the normalization of the input, the sink the LaTeX is written to,
//! and the report of what it needs. The crate is `#![no_std]` with
//! [`alloc`], and its one dependency is `unicode-normalization`.
//!
//! # The encoder
//!
//! An [`Encoder`] holds a [`Rule`], a [`ReplacementProtection`] strategy, an
//! [`InputNormalizer`] and an [`UnknownCharPolicy`]. It is built once,
//! encodes any number of strings, and is immutable — and so shareable
//! between threads whenever its rules are.
//!
//! [`Encoder::encode`] answers a new string.
//! [`Encoder::encode_with_report`] answers that string together with an
//! [`EncodeReport`]: what the LaTeX needs in the document's preamble, and
//! which characters no rule knew. [`Encoder::encode_into`] is the primitive
//! the other two are written with — it appends to an output and a report the
//! caller owns, which is what a document made of many fragments uses.
//!
//! ```
//! use untechxt::{Encoder, DEFAULTS};
//!
//! let encoder = Encoder::new(&DEFAULTS);
//! let (body, report) = encoder.encode_with_report("𝟙 and ⅓").unwrap();
//! assert_eq!(body, r"\ensuremath{\mathds{1}} and \nicefrac{1}{3}");
//!
//! // `\mathds` is the package `dsfont`'s and `\nicefrac` the package
//! // `nicefrac`'s: the report knows, and writes the preamble that loads them.
//! let mut preamble = String::new();
//! report.needs.write_preamble(&mut preamble).unwrap();
//! assert_eq!(preamble, "\\usepackage{dsfont}\n\\usepackage{nicefrac}\n");
//! ```
//!
//! [`encode`] is that encoder's [`encode`](Encoder::encode) with the report
//! dropped.
//!
//! # Rules
//!
//! A [`Rule`] is offered a position in the input — a [`RuleInput`]: the whole
//! normalized text, the byte offset, and the character there — and answers
//! either with an [`EncodedReplacement`] (how many bytes it consumed, the
//! LaTeX that replaces them, how that LaTeX must be protected, and what it
//! needs in the preamble) or with "not mine". It may also fail, which stops
//! the encoding with [`EncodeError::Rule`]; that is how a rule driven from
//! another language passes an exception on.
//!
//! [`RuleChain`] is several rules as one. They are tried in order at every
//! position and **the first match wins**: no rule is tried after a match, and
//! no longest match is sought. A rule placed before a table therefore
//! overrides it, and a rule placed after it fills in what the table lacks.
//! The members are a tuple of up to twelve rules, which the compiler unrolls
//! and inlines and which keeps each member's own type; an array; or a
//! [`Vec`](alloc::vec::Vec) — [`DynRuleChain`] and [`LocalDynRuleChain`] are
//! the chains of boxed rules that a configuration file or a language binding
//! assembles at run time.
//!
//! Rules come in three kinds.
//!
//! - **Tables.** A [`LookupTable`] answers for single characters. The builtin
//!   data is [`DEFAULTS`], with the views [`NON_ASCII`] (no ASCII entry, for
//!   input that already holds LaTeX) and [`ASCII_SPECIALS`] (the 13 ASCII
//!   entries alone). [`DynTable`] is a table built at run time;
//!   [`compile_static_table!`] compiles a table of your own at compile time
//!   into one of the layouts of [`statictable`]. The crate's own tables are
//!   rules already, and a [`LookupTable`] of your own becomes one through
//!   [`TableRule`].
//! - **Closures**, through [`rule_fn`]. A closure rule may hand out owned
//!   strings, literals, `&'static Profile`s and slices of the input, but it
//!   cannot lend out its own captures.
//! - **Anything else that implements [`Rule`]**: a struct that reads several
//!   characters ahead, that lends values from its own state, or that calls
//!   into another language. [`Rule`] requires only
//!   [`Debug`](core::fmt::Debug) — not `Send` or `Sync`, so that a rule
//!   holding a JavaScript callback or an [`Rc`](alloc::rc::Rc) is possible;
//!   an [`Encoder`] is `Send` and `Sync` exactly when its rules are.
//!
//! ```
//! use untechxt::{
//!     rule_fn, AsciiSet, DynTable, Encoder, ReplacementProtectionHint as Hint,
//!     RuleChain, RuleInput, DEFAULTS,
//! };
//!
//! // A table ahead of the defaults overrides one character; a closure after
//! // them reads a sequence of characters, which a table cannot.
//! let overrides =
//!     DynTable::new().with_entry('%', r"\textpercent", Hint::text_only(r"\textpercent"));
//! let ellipsis = rule_fn(|input: RuleInput<'_>| {
//!     Ok(input
//!         .rest()
//!         .starts_with("...")
//!         .then(|| input.replace_prefix(3, r"\ldots", Hint::any_mode(r"\ldots"))))
//! })
//! .with_ascii_triggers(AsciiSet::of("."));
//!
//! let encoder = Encoder::new(RuleChain::new((overrides, &DEFAULTS, ellipsis)));
//! assert_eq!(encoder.encode("100%... é").unwrap(), r"100{\textpercent}{\ldots} \'e");
//! ```
//!
//! # Protection, and LaTeX's two modes
//!
//! An encoded value cannot simply be pasted into the output: `\textemdash`
//! followed directly by a letter would read as a longer command name, and
//! `\alpha` is valid in mathematics alone. The rule therefore states what its
//! value is, through a required and authoritative
//! [`ReplacementProtectionHint`] — the encoder never inspects a value behind
//! its rule's back:
//!
//! - [`DoNotProtect`](ReplacementProtectionHint::DoNotProtect): write the
//!   value exactly as it is, the rule vouching for it in its context. A rule
//!   that passes existing LaTeX through uses this.
//! - [`Value`](ReplacementProtectionHint::Value): the [`ValueMode`] the value
//!   is valid in ([`TextOnly`](ValueMode::TextOnly),
//!   [`MathOnly`](ValueMode::MathOnly) or [`AnyMode`](ValueMode::AnyMode)) and
//!   its [`ValueTermination`] — whether arbitrary text may follow it directly,
//!   or whether it ends with a named macro. A rule that does not know reads
//!   the termination off the value's form with [`ValueTermination::inspect`],
//!   which is what [`ReplacementProtectionHint::text_only`] and its two
//!   companions do, at compile time in a static table.
//!
//! The mode lives in the hint and not in the value: a table entry holds the
//! bare `\alpha` marked [`MathOnly`](ValueMode::MathOnly), not
//! `\ensuremath{\alpha}`, so that math output writes it as it is and text
//! output wraps it once.
//!
//! What is written around the value is then the encoder's
//! [`ReplacementProtection`] strategy's business. Protection is stateless:
//! each value is protected as it comes, with no lookahead and no memory of
//! what was written before. [`StandardProtection`] is the strategy the crate
//! offers, and every one of its fields is public and settable at run time:
//!
//! - [`output_mode`](StandardProtection::output_mode), which of LaTeX's two
//!   modes the output is going into. A value whose mode does not match is
//!   wrapped by [`math_wrap`](StandardProtection::math_wrap)
//!   (`\ensuremath{…}`) or by [`text_wrap`](StandardProtection::text_wrap);
//!   such a [`ModeWrapper`] may itself need something in the preamble, which
//!   is why the reporter reaches the strategy too.
//! - [`protect_names`](StandardProtection::protect_names), what is written
//!   around a value that ends with a named macro:
//!   [`BracesAround`](MacroNameProtection::BracesAround) (`{\textemdash}`,
//!   the default and the safe choice in text mode), `BracesAfter`
//!   (`\textemdash{}`), `SpaceAfterMacroName` (`\pm `, the default of
//!   [`math_mode`](StandardProtection::math_mode), where spaces are ignored,
//!   and unsafe in text mode, where TeX would swallow a space of the input),
//!   or `NoProtection`.
//!
//! ```
//! use untechxt::{BracesAroundAll, Encoder, StandardProtection, DEFAULTS};
//!
//! // Text output: a math value is wrapped once, and a value that ends with a
//! // named macro is braced.
//! let text = Encoder::new(&DEFAULTS);
//! assert_eq!(text.encode("α≤β").unwrap(),
//!            r"\ensuremath{\alpha}\ensuremath{\leq}\ensuremath{\beta}");
//! assert_eq!(text.encode("—").unwrap(), r"{\textemdash}");
//!
//! // Math output: the same values go in bare, ended by a space; a text value
//! // is the one that has to be wrapped.
//! let math = Encoder::new(&DEFAULTS).with_protection(StandardProtection::math_mode());
//! assert_eq!(math.encode("α≤β").unwrap(), r"\alpha \leq \beta ");
//! assert_eq!(math.encode("é").unwrap(), r"\textnormal{\'e}");
//!
//! // A strategy of its own: braces around every value.
//! let compat =
//!     Encoder::new(&DEFAULTS).with_protection(BracesAroundAll(StandardProtection::text_mode()));
//! assert_eq!(compat.encode("Café — α").unwrap(),
//!            r"Caf{\'e} {\textemdash} {\ensuremath{\alpha}}");
//! ```
//!
//! [`BracesAroundAll`] braces every value, whatever its hint says — this is
//! pylatexenc's `'braces-all'` mode. It is not a setting of
//! [`StandardProtection`] but a strategy of its own, written with public API
//! alone, and so it doubles as the example of how to write one: implement
//! [`ReplacementProtection`], read the value and its hint off the
//! [`ProtectInput`], and write to the [`OutBuffer`] you are handed.
//!
//! # Characters no rule knows
//!
//! A character is unknown when no rule matched it and it is not printable
//! ASCII (`0x20..=0x7E`) or one of `\n`, `\r` and `\t`. It is reported to the
//! [`EncodeReporter`] whatever happens next, and then the encoder's
//! [`UnknownCharPolicy`] says what is written for it:
//! [`Keep`](UnknownCharPolicy::Keep), the default, the character itself;
//! [`Ignore`](UnknownCharPolicy::Ignore) nothing;
//! [`Fail`](UnknownCharPolicy::Fail) stops with
//! [`EncodeError::UnknownChar`];
//! [`ReplaceWith`](UnknownCharPolicy::ReplaceWith) a fixed text; and
//! [`Callback`](UnknownCharPolicy::Callback) whatever a function answers —
//! [`unknown_unihex`] spells the code point out. None of that output is
//! protected and none of it carries preamble needs: a policy is a last
//! resort, and anything richer is a rule at the end of the chain.
//!
//! ```
//! use untechxt::{unknown_unihex, Encoder, UnknownCharPolicy, DEFAULTS};
//!
//! // The Thai letter is in no builtin entry: it is kept, and reported.
//! let (out, report) = Encoder::new(&DEFAULTS).encode_with_report("ธ").unwrap();
//! assert_eq!(out, "ธ");
//! assert!(report.unknown_chars.contains(&'ธ'));
//!
//! let spelled_out =
//!     Encoder::new(&DEFAULTS).with_unknown_chars(UnknownCharPolicy::callback(unknown_unihex));
//! assert_eq!(spelled_out.encode("ธ").unwrap(),
//!            r"\ensuremath{\langle}\texttt{U+0E18}\ensuremath{\rangle}");
//! ```
//!
//! # What the output needs in the preamble
//!
//! LaTeX such as `\mathds{1}` prints nothing unless the document loads the
//! package `dsfont`. A [`Chunk`] is one piece of preamble — a package with
//! the options it is loaded with, or a block of declarations no package makes
//! ([`ChunkPreamble`]) — under a stable identifier, and the same identifier
//! always means the same chunk. A [`Profile`] is the set of chunks that one
//! encoded value needs, which a rule hands out with its value
//! ([`EncodedReplacement::with_needs`]). There is no registry and no
//! identifier space to share out: a profile is a plain reference to something
//! the rule owns, so tables from unrelated crates cannot clash.
//!
//! The encoder passes each profile to an [`EncodeReporter`]. [`EncodeReport`]
//! keeps the needs and the unknown characters, [`PreambleNeeds`] the needs
//! alone, and [`NoReport`] nothing at all — under [`NoReport`] and a static
//! chain the compiler can remove the whole needs machinery from the loop. A
//! [`PreambleNeeds`] holds one copy of each distinct chunk, orders every
//! package before every block of declarations, and writes the preamble out
//! with [`PreambleNeeds::write_preamble`]. It is a reporter itself, so the
//! fragments of one document can all report into one.
//!
//! ```
//! use untechxt::{Encoder, PreambleNeeds, DEFAULTS};
//!
//! let encoder = Encoder::new(&DEFAULTS);
//! let mut body = String::new();
//! let mut needs = PreambleNeeds::new();
//! for fragment in ["𝟙 ", "and ⅓"] {
//!     encoder.encode_into(fragment, &mut body, &mut needs).unwrap();
//! }
//! assert_eq!(body, r"\ensuremath{\mathds{1}} and \nicefrac{1}{3}");
//!
//! let mut preamble = String::new();
//! needs.write_preamble(&mut preamble).unwrap();
//! assert_eq!(preamble, "\\usepackage{dsfont}\n\\usepackage{nicefrac}\n");
//! ```
//!
//! # Normalization
//!
//! The input goes through an [`InputNormalizer`] before any rule sees it. The
//! default is [`NormalizeNfc`], Unicode's canonical composed form, so that a
//! letter followed by a combining accent becomes the single accented
//! character the tables know; a quick check comes first, and text that it
//! finds composed already — nearly all text — is not copied.
//!
//! Positions, in an [`EncodeError`] as in
//! [`EncodeReporter::report_unknown_char`], are byte offsets into the
//! normalized text, which is not the caller's text where normalization
//! changed it. A caller that needs offsets into its own text normalizes it
//! once with [`nfc`] and encodes with [`NoNormalization`]. Under
//! [`NoNormalization`] nothing references the normalization tables and the
//! linker drops them.
//!
//! ```
//! use untechxt::{encode, nfc, Encoder, NoNormalization, DEFAULTS};
//!
//! // `e` followed by a combining acute accent, composed into `é` first.
//! assert_eq!(encode("Cafe\u{301}"), r"Caf\'e");
//!
//! let as_is = Encoder::new(&DEFAULTS).with_normalizer(NoNormalization);
//! assert_eq!(as_is.encode("Cafe\u{301}").unwrap(), "Cafe\u{301}"); // unknown, kept
//! assert_eq!(as_is.encode(&nfc("Cafe\u{301}")).unwrap(), r"Caf\'e");
//! ```
//!
//! # Where the output goes
//!
//! [`Encoder::encode_into`] writes to any [`OutBuffer`]. [`String`] is one,
//! and costs nothing extra: the error check disappears when the compiler
//! inlines the call. [`FmtOut`] carries the output to a [`core::fmt::Write`],
//! and `IoOut` — with the crate feature `std` — to a `std::io::Write`: a
//! file, a socket, standard output, so that a long document never has to
//! exist as one string. The error type is the fixed [`BoxError`], so that no
//! signature of the crate carries an error parameter and an I/O error is
//! passed through as it is.
//!
//! ```
//! use untechxt::{Encoder, FmtOut, NoReport, DEFAULTS};
//!
//! let mut out = FmtOut(String::new());
//! Encoder::new(&DEFAULTS).encode_into("Café", &mut out, &mut NoReport).unwrap();
//! assert_eq!(out.0, r"Caf\'e");
//! ```
//!
//! # The ASCII fast path
//!
//! Plain ASCII text is the common case, and it is copied in bulk. An
//! [`AsciiSet`] is a bitmap of ASCII characters in a `u128`, so a membership
//! test is a shift and a mask. [`Rule::ascii_triggers`] answers the set of
//! ASCII characters the rule may match at; [`Encoder::new`] asks once, unions
//! the answers with every non-printable ASCII character, and then copies runs
//! of input bytes outside that union with a single push, decoding no
//! character and calling no rule.
//!
//! That set is a contract a rule author must respect. It is a promise that
//! the rule never matches at an ASCII character outside it, and the answer
//! must be the same every time, since it is asked exactly once. It is a
//! promise in one direction only: the encoder may still consult the rule at
//! other characters, where a correct rule declines. The default is
//! [`AsciiSet::ALL`] — "I may match anywhere" — which is always correct and
//! only slower; [`AsciiSet::EMPTY`] is what a table of non-ASCII entries
//! answers. A table answers its own keys, and
//! [`RuleFn::with_ascii_triggers`] is where a closure rule states its own, as
//! the ellipsis rule above states its `.`.
//!
//! # `no_std`, `alloc`, and the `std` feature
//!
//! The crate is `#![no_std]` and uses [`alloc`]: it allocates strings and
//! vectors and nothing else. Errors are [`core::error::Error`] throughout.
//! Its one dependency, `unicode-normalization`, is `no_std` as well.
//!
//! The default feature `std` adds a single item, the `IoOut` adapter for
//! `std::io::Write`; `cargo build --no-default-features` is the `no_std`
//! build.
//!
//! # Relation to pylatexenc
//!
//! The design follows the `latexencode` module of
//! [pylatexenc](https://github.com/phfaist/pylatexenc), the Python library
//! for working with LaTeX code, by the same author: an ordered list of rules,
//! the first match at each position winning, each value protected so that it
//! cannot merge with the text that follows, and a policy for the characters
//! no rule knows. Three things are this library's own — what an encoded value
//! needs in the preamble, LaTeX's two modes, and output that streams — beside
//! the rule model, which is a trait rather than a fixed set of rule kinds.
//!
//! The builtin data began as a copy of pylatexenc's `defaults` table, whose
//! character map came in turn from
//! [latexcodec](https://pypi.python.org/pypi/latexcodec). It is maintained
//! here by hand: every entry compiles and sets the glyph its character stands
//! for, and every departure from pylatexenc's spelling is listed with its
//! reason in [`builtin::default_table`], where the MIT notices that travel
//! with the data are reproduced in full. pylatexenc's second table,
//! `unicode-xml`, is not ported.
//!
//! What has no counterpart here: regular-expression rules (a closure rule
//! expresses them); per-rule protection overrides (a rule's only say is its
//! hint); the `non_ascii_only` flag, which silently disabled multi-character
//! rules that start at an ASCII character — leave [`ASCII_SPECIALS`] out of
//! the chain, or encode with [`NON_ASCII`], instead; the
//! `'braces-almost-all'` protection mode ([`BracesAroundAll`] is
//! `'braces-all'`); and the `'unihex'` unknown-character mode, which is the
//! plain function [`unknown_unihex`] handed to
//! [`UnknownCharPolicy::callback`]. Positions are byte offsets, where
//! pylatexenc counts code points.
#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod asciiset;
pub mod builtin;
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
pub use crate::builtin::{BuiltinTable, ASCII_SPECIALS, DEFAULTS, NON_ASCII};
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

use alloc::string::String;

/// The LaTeX for `text`, under every default setting.
///
/// Those settings are: the builtin table [`DEFAULTS`] as the only rule,
/// [`StandardProtection::text_mode`] around every value, [`NormalizeNfc`]
/// over the input, [`UnknownCharPolicy::Keep`] for a character the table does
/// not know, [`NoReport`] — what the LaTeX needs in the preamble is dropped —
/// and a [`String`] to write into. None of those can fail, which is why this
/// answers the string itself rather than a [`Result`].
///
/// Build an [`Encoder`] to change any of that, and in particular to learn
/// what the output needs in the document's preamble
/// ([`Encoder::encode_with_report`]). This function builds its encoder on
/// every call, which for a static table is a few instructions — there is no
/// cell to cache one in without `std` — but an encoder built once and kept is
/// still less work.
///
/// ```
/// use untechxt::encode;
///
/// assert_eq!(encode("Café — naïve"), r#"Caf\'e {\textemdash} na\"ive"#);
/// assert_eq!(encode("100% & more"), r"100\% \& more");
/// assert_eq!(encode("α ≤ β"), r"\ensuremath{\alpha} \ensuremath{\leq} \ensuremath{\beta}");
/// // A character the builtin table has no entry for is kept as it is.
/// assert_eq!(encode("ธ"), "ธ");
/// ```
pub fn encode(text: &str) -> String {
    Encoder::new(&DEFAULTS).encode(text).expect(
        "encoding with the builtin table under the default settings cannot fail: \
         the table's lookup never fails, an unknown character is kept rather than \
         refused, and a String never refuses what is written to it",
    )
}
