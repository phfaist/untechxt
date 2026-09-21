# Untechxt Build Plan

## Context: What we're building

`untechxt` is a Unicode-to-LaTeX encoder library in Rust. It turns a Unicode
string such as `Café — α ≤ β` into LaTeX source such as
`Caf\'e {\textemdash} \ensuremath{\alpha} \ensuremath{\leq} \ensuremath{\beta}`,
and it reports which LaTeX packages or declarations the output needs in the
document preamble. Requirements:

- **Fast.** Plain ASCII text is copied in bulk; static lookup tables are
  compiled at build time into a form that is cheap to search; a statically
  typed rule chain lets the compiler inline and unroll everything.
- **Lightweight.** `#![no_std]` + `alloc`. One dependency
  (`unicode-normalization`).
- **Very extensible.** Users supply their own rules (tables, callbacks,
  anything implementing a trait), their own protection strategy, output sink,
  report sink, input normalizer, and preamble needs.

The design is modeled on the `latexencode` module of the Python package
pylatexenc (<https://github.com/phfaist/pylatexenc>), by the same author.
In pylatexenc, an encoder holds an ordered list of rules. At each position of
the input, the rules are tried in order and the first match wins. A rule
reports how much input it consumed and the replacement LaTeX. The encoder
then "protects" the replacement so that it cannot merge with the text that
follows (`\textemdash` + `more` must not become `\textemdashmore`), and
appends it to the output. Characters that no rule matches go through an
"unknown character policy". pylatexenc has no notion of preamble needs, of
math versus text mode, or of streaming output; this library adds all three.

### Material in this repository

- `CLAUDE.md`: working conventions. Read it first and follow it.
- `initial-rust-port/`: a quick-and-dirty earlier Rust port, extracted from
  another project (FLM), where it was the crate `flm-latexencode`. It is
  **git-ignored and present only locally**. It has no `Cargo.toml` and is not
  built. It is reference material, and three parts of it are carried over:
  - `src/tables.rs`: the data. 1549 hand-curated entries
    `(char, latex, PROFILE), // UNICODE NAME`, plus 20 preamble chunks, 21
    profiles, the provenance notes, a documented list of 16 corrections over
    pylatexenc, and two MIT license notices (pylatexenc and latexcodec) that
    must travel with the data.
  - `tests/latexencode.rs` and
    `tests/goldens/latexencode/uni_chars_test_previous.txt`: the behavioral
    spec. The golden file has one line per character, in the form
    `0x%04X [UNICODE NAME]   |<char>|`; the test rebuilds each input line
    from the golden itself, encodes it, and compares byte for byte
    (`UPDATE_GOLDEN=1` rewrites the file). The golden was produced under
    pylatexenc's `braces-almost-all` protection and the `fail` unknown-char
    policy.
  - `src/lib.rs` and `src/needs.rs`: the old encoder loop and needs model
    (for reference), and documentation prose worth migrating.
- `src/`: an exploratory prototype of the new design. **It does not compile**
  and contains leftover pasted fragments. Treat it as a statement of intent
  and rewrite it. Two things in it are kept as they are in spirit:
  `src/statictable.rs` (three static table layouts built by `const fn`s and a
  `macro_rules!` macro) and `src/builtin/default_table.rs` (a verbatim copy of
  the initial port's 1549 entries, not yet migrated).
- `Cargo.toml`: edition 2021, `rust-version = "1.86"`, lints
  `missing_docs = "deny"` (every public item needs a doc comment), release
  profile with LTO. It currently **lacks a `name` field**; see "Crate setup".
- A local pylatexenc install (`pip show pylatexenc`) can be consulted for the
  original semantics: `latexencode/_unicode_to_latex_encoder.py`.

### How to execute this plan

- Work through "Implementation order" step by step. Each step ends with all
  of these clean: `cargo build`, `cargo build --no-default-features`,
  `cargo test`, `cargo clippy --all-targets`, `cargo doc`.
- The signatures in this plan are sketches that were never compiled. Where
  Rust rejects one, change it as little as possible, keep the intent of the
  decision behind it, and update this file so that it stays the single source
  of truth.
- Do not reopen the decisions below without asking. Each one was weighed
  against alternatives; the short "why" is given to prevent well-meant
  "fixes".
- Commit and push at regular stages.

## Naming conventions

- A rule's output string is called `encoded` (not `latex`).
- What a matching rule returns is an `EncodedReplacement` (not `Replacement`).
- The protection strategy trait is `ReplacementProtection` (not `Protection`).
- Macro-name protection styles: `MacroNameProtection::{BracesAround,
  BracesAfter, SpaceAfterMacroName, NoProtection}` (field `protect_names` of
  `StandardProtection`). Names must say what they do; a name that disables a
  safety net must say so.
- Input normalizers: `NormalizeNfc`, `NoNormalization`.
- Commands defined by builtin snippets use the `\UnxT` prefix (was `\flm`).

## Design decisions

### Rules

- One dyn-compatible trait, `Rule: Debug`, with
  `apply(&self, input: RuleInput<'_>) -> RuleResult<'a>` where
  `RuleResult<'a> = Result<Option<EncodedReplacement<'a>>, BoxError>`:
  `Ok(None)` means "does not match here", `Err` aborts the encoding.
  First match wins; there is no longest-match logic. All positions and
  lengths are byte offsets.
- `Debug` is the only supertrait. `Send + Sync` are deliberately **not**
  supertraits: they are auto traits, so `Encoder<R>` is `Send + Sync` exactly
  when `R` is, and requiring them would forbid rules that hold JS callbacks
  (wasm-bindgen values are neither `Send` nor `Sync`), `Rc`, or `RefCell`.
- `EncodedReplacement<'a>` holds `consumed`, `encoded: Cow<'a, str>`, a
  protection hint, and preamble needs. A fixed `Cow` was chosen over an
  associated output type: an associated type that can borrow from `self`
  must be a generic associated type, which makes the trait not
  dyn-compatible and forces chains to unify output types. Constructors take
  `impl Into<Cow<'a, str>>`, so rule authors pass `&'static str`, `String`,
  or `&self.field` without conversions. Tables stay allocation-free.
- `EncodedReplacement` has private fields and is built only through
  `RuleInput` (`replace_char`, `replace_prefix`, `try_replace_prefix`). A zero
  or out-of-range `consumed` therefore cannot be constructed, the encoder
  needs no per-match validation, and there is no "invalid consumption" error.
- `RuleInput<'s>` is a small `Copy` struct with private fields and accessors,
  so context can be added later without breaking every `Rule` impl.
- Closures become rules through `rule_fn(..)`. A blanket
  `impl<F: Fn(..)> Rule for F` is impossible: it overlaps with the `&R` and
  `Box<R>` forwarding impls, because `&F` and `Box<F>` are themselves `Fn`.
- Rules are fallible so that foreign-language callbacks (Python, JS) can
  propagate exceptions. Error payloads are
  `BoxError = Box<dyn Error + Send + Sync>` (the ecosystem convention; a JS
  exception is stringified on the way in).

### Chains

- One generic wrapper `RuleChain<L>` implements `Rule`. A dedicated struct
  (instead of implementing `Rule` on bare tuples) leaves room for chain-level
  options later, such as "continue with the next rule if one fails". Do not
  implement such options now.
- `L: RuleList`, a sealed helper trait implemented for tuples up to arity 12
  (static dispatch, unrolled), `[R; N]`, and `Vec<R>`. Bare tuples and `Vec`s
  are not rules themselves, so there is exactly one way to build a chain.
- `&R`, `Box<R>`, and `Option<R>` forward `Rule` (`Option` gives a rule that
  can be switched off without changing the chain's type).
- Aliases: `DynRuleChain<'r> = RuleChain<Vec<Box<dyn Rule + Send + Sync + 'r>>>`
  for threads and Python bindings; `LocalDynRuleChain<'r>` without
  `Send + Sync` for JS callbacks and `Rc`.

### Protection

- Protection is stateless: each encoded string is protected as it comes,
  with no lookahead and no memory of what was written before. Seeing the
  next output would be needed to decide `{..}` wrapping exactly, and that
  decision chain can be unbounded; it was judged not worth the machinery.
- The rule states what its value needs through a required, authoritative
  hint. There is no "inspect it for me" default and no per-rule protection
  mode override (pylatexenc's mechanism). A hint is either
  - `DoNotProtect`: written verbatim; the rule vouches for the value in its
    context (example: a rule that passes existing LaTeX such as `\textbf`
    through unchanged), or
  - `Value { mode, termination }` with
    `ValueMode::{TextOnly, MathOnly, AnyMode}` and
    `ValueTermination::{ValueIsSelfTerminating, ValueEndsWithNamedMacro}`.
  Public `const fn` helpers inspect the termination for rules that do not
  know it; the table macro runs the same code at compile time.
- Math/text mode lives in the hint, not in the table strings: entries store
  bare `\alpha` marked `MathOnly`, not `\ensuremath{\alpha}`. A value that
  contains its own `\ensuremath{..}` is valid in both modes (`AnyMode`), or
  `TextOnly` if other parts of it are text-only.
- The standard strategy is a small struct, `StandardProtection { output_mode,
  protect_names, math_wrap, text_wrap }`; see "Behavior specifications" for
  what it writes. Macro-name protection depends on the output mode:
  - Text output: braces (`BracesAround`, the default, or `BracesAfter`).
    `SpaceAfterMacroName` is unsafe there: TeX skips all spaces after a
    control word, so a real space that follows in the input would be lost,
    and a stateless strategy cannot know whether one follows.
  - Math output: `SpaceAfterMacroName` is the default, because spaces are
    ignored in math mode. `\pm{}` adds an empty atom that detaches a
    following super/subscript and changes spacing in edge cases, and
    `{\pm}` turns an operator into an ordinary symbol.
- `write_protected` receives the reporter, because a mode wrapper may itself
  need a package (`\text{..}` needs amsmath).
- pylatexenc's `braces-all` and `braces-almost-all` modes (version-1
  compatibility) are not standard options. `BracesAroundAll` exists as a
  separate strategy object and doubles as the example of a custom strategy.

### Unknown characters

- A char is unknown when no rule matched it and it is not printable ASCII
  (`0x20..=0x7E`) or one of `\n`, `\r`, `\t`. Non-printable ASCII is offered
  to the rules first like any other char. (pylatexenc's inconsistent handling
  of DEL is not reproduced.)
- `UnknownCharPolicy::{Keep, Ignore, Fail, ReplaceWith, Callback}`. The
  callback is a plain `Fn(char) -> String + Send + Sync`; its output gets no
  protection, hints, or profiles. Anything richer is a rule at the end of the
  chain. There is no `Unihex` variant: `unknown_unihex` is a plain function
  passed to `UnknownCharPolicy::callback(..)`, a constructor that boxes. The
  `Send + Sync` bound keeps the mere presence of this variant from making
  every encoder single-threaded.

### Preamble needs

- A **chunk** is one piece of preamble: one package load or one snippet of
  declarations, with a stable string id. A **profile** is the set of chunks
  one encoded string needs. Same chunk id means same chunk.
- Static tables store a `u8` profile index per entry, but what a rule hands
  out is a plain reference, `Option<&'a Profile>`. There are no runtime id
  spaces and no registries, so tables from independent crates cannot clash
  and user rules (static or built at run time) return a reference to a
  profile they own.
- `Chunk` and `Profile` have no lifetime parameter (`Cow<'static, ..>`
  inside: borrowed for static data, owned for run-time data).
- The needs accumulator owns a copy of each distinct chunk it has seen, so
  reports have no lifetime and can live in the same struct as the encoder.

### Output, errors, reporting

- `OutBuffer`: own trait so that output can stream (for example to I/O)
  without an intermediate string. `push_str` is required, `push_char` has a
  default. The error type is the fixed `BoxError` (no error type parameter
  spreading through every signature; I/O errors are preserved, boxed only on
  the error path; for `String` the check disappears after inlining).
  `core::fmt::Write` was rejected as the sink because its error carries no
  information.
- `EncodeError::{UnknownChar { ch, position }, Rule { position, source },
  Output(source)}`, `#[non_exhaustive]`. On error, partial output stays in
  the buffer and the report stays partially filled.
- Two API tiers. Accumulating primitive:
  `encode_into(text, &mut out, &mut report)` appends to both. Pure
  conveniences: `encode(text) -> String` and
  `encode_with_report(text) -> (String, EncodeReport)`. Reports are passed in
  (not returned) because the main workload is many text fragments feeding
  one preamble. State inside the encoder was rejected: the encoder is
  immutable and shareable across threads.
- The report is abstracted behind `EncodeReporter`, with defaulted no-op
  methods. Impls: `NoReport`, `EncodeReport` (needs + distinct unknown chars;
  no counts, no positions), `PreambleNeeds` (needs only). `encode()` uses
  `NoReport`. Table lookups resolve the profile panic-free (`.get(idx)`), so
  the compiler can remove the needs logic entirely under `NoReport` with a
  static chain.

### Input normalization

- `InputNormalizer` trait; `NormalizeNfc` is the default type parameter of
  the encoder, `NoNormalization` the alternative. Without NFC, a decomposed
  `e` + combining acute would be an unknown char instead of `\'e`.
- Positions in errors and in `report_unknown_char` refer to the normalized
  text. The report does not hold the normalized input. A caller who needs
  exact positions normalizes once with the public `nfc()` helper and uses
  `NoNormalization`.
- `unicode-normalization` stays an unconditional dependency. With
  `NoNormalization` nothing references its tables and the linker drops them.

### ASCII fast path

- `AsciiSet(u128)`: a bitmap, one bit per ASCII char; a membership test is a
  shift and a mask.
- `Rule::ascii_triggers() -> AsciiSet` (default `ALL`) is a promise: "I never
  match at an ASCII char outside this set." It is called once in
  `Encoder::new` and must always return the same answer. The encoder may
  still consult a rule at other chars; a correct rule declines there.
- The scan loop's stop set is the chain's union plus all non-printable ASCII
  (so that those reach the rules and then the unknown-char policy). Runs of
  ASCII bytes outside the stop set are copied with a single `push_str`,
  without decoding chars or calling rules. Scanning bytes is sound because in
  UTF-8 every byte below `0x80` is a complete ASCII char.
- This replaces pylatexenc's `non_ascii_only` flag, which silently disabled
  multi-char rules that start on an ASCII char: leave the ASCII-specials
  table out of the chain instead.

### Tables

- Builtin tables: `DEFAULTS` (all entries), `NON_ASCII`, `ASCII_SPECIALS`
  (the 13 entries `" # $ % & < > \ ^ _ { } ~`), from one source list. The
  latter two are filtered views that share the compiled data of `DEFAULTS`.
- No cargo features for binary size: unused statics are dropped at link
  time. More builtin tables are anticipated (see "Open items").
- There is no per-char lookup on the encoder (normalization and protection
  make it fragile); callers use the `encode` family on a one-char string.
  Tables offer their own `lookup(ch)`.

### Deliberately not included

Regex rules; per-rule protection mode overrides; lookahead or stateful
protection; a `non_ascii_only` flag; `Encoder::lookup_char`; the initial
port's `Mode::of`, `spelling_of`, and `macro_names`; an `Unihex` policy
variant; occurrence counts or positions in the report; the normalized input
in the report; cargo features to trim tables; an "invalid consumption" error;
a runtime `Chunk::docs` string (source comments instead).

## Blueprint

### Crate setup

- Add `name = "untechxt"` to `[package]` in `Cargo.toml`.
- `lib.rs` starts with `#![no_std]`, `extern crate alloc;`, and
  `#[cfg(feature = "std")] extern crate std;`.
- Features: `default = ["std"]`, `std = []`. The `std` feature only adds the
  `io::Write` adapter. `cargo build --no-default-features` is the `no_std`
  check.
- `core::error::Error` is used throughout (stable since Rust 1.81).
- `criterion` is added as a dev-dependency at step 6.

### Module layout

Existing file names are kept where they fit.

```
src/
  lib.rs                     crate docs, re-exports, free fn encode()
  rule.rs                    Rule, RuleInput, EncodedReplacement, RuleResult,
                             BoxError, InvalidPrefixLength, rule_fn / RuleFn,
                             forwarding impls (&R, Box<R>, Option<R>)
  chain.rs                   RuleChain<L>, sealed RuleList, DynRuleChain,
                             LocalDynRuleChain
  replacement_protection.rs  hint types, ReplacementProtection, ProtectInput,
                             StandardProtection, MacroNameProtection,
                             OutputMode, ModeWrapper, BracesAroundAll
  preamble.rs                Chunk, ChunkPreamble
  profile.rs                 Profile, ProfileIndex, PreambleNeeds
  report.rs                  EncodeReporter, NoReport, EncodeReport
  outbuffer.rs               OutBuffer, FmtOut, IoOut (feature "std")
  normalizer.rs              InputNormalizer, NormalizeNfc, NoNormalization,
                             nfc()
  asciiset.rs                AsciiSet
  unknown_char.rs            UnknownCharPolicy, unknown_unihex
  encoder.rs                 Encoder<R, P, N>, EncodeError
  lookuptable.rs             LookupTable, TableEntry, TableRule, DynTable,
                             OnlyAscii / ExceptAscii views
  statictable.rs             static layouts, const-fn builders,
                             compile_static_table!
  builtin/
    mod.rs                   DEFAULTS, NON_ASCII, ASCII_SPECIALS
    needs_profiles.rs        builtin chunks and profiles
    default_table.rs         the entries: hand-maintained source of truth,
                             with provenance notes and both MIT notices
tools/
  migrate_tables.py          one-off data migration script (step 3)
```

### Core API sketch

```rust
// rule.rs
pub type BoxError = Box<dyn core::error::Error + Send + Sync + 'static>;
pub type RuleResult<'a> = Result<Option<EncodedReplacement<'a>>, BoxError>;

pub trait Rule: Debug {
    fn apply<'a>(&'a self, input: RuleInput<'_>) -> RuleResult<'a>;
    fn ascii_triggers(&self) -> AsciiSet { AsciiSet::ALL }
}

#[derive(Debug, Clone, Copy)]
pub struct RuleInput<'s> { /* full: &'s str, pos: usize, ch: char */ }
impl<'s> RuleInput<'s> {
    pub fn new(full: &'s str, pos: usize) -> Option<Self>;  // for testing rules
    pub fn ch(&self) -> char;          // the char at pos, already decoded
    pub fn pos(&self) -> usize;
    pub fn full(&self) -> &'s str;
    pub fn rest(&self) -> &'s str;     // &full[pos..], for lookahead
    pub fn before(&self) -> &'s str;   // &full[..pos], for lookbehind

    /// Consumes exactly the current char. Always valid.
    pub fn replace_char<'a>(&self, encoded: impl Into<Cow<'a, str>>,
        hint: ReplacementProtectionHint) -> EncodedReplacement<'a>;
    /// Consumes `n_bytes` of `rest()`. Panics unless `n_bytes` is non-zero,
    /// within `rest()`, and on a char boundary (like slice indexing).
    pub fn replace_prefix<'a>(&self, n_bytes: usize,
        encoded: impl Into<Cow<'a, str>>,
        hint: ReplacementProtectionHint) -> EncodedReplacement<'a>;
    /// Same, returning an error instead of panicking. `?` turns it into a
    /// rule error. For callbacks from foreign languages.
    pub fn try_replace_prefix<'a>(&self, n_bytes: usize, /* same */)
        -> Result<EncodedReplacement<'a>, InvalidPrefixLength>;
}

pub struct EncodedReplacement<'a> { /* consumed, encoded, hint, needs */ }
impl<'a> EncodedReplacement<'a> {
    pub fn with_needs(self, profile: &'a Profile) -> Self;
    pub fn consumed(&self) -> usize;
    pub fn encoded(&self) -> &str;
    pub fn hint(&self) -> ReplacementProtectionHint;
    pub fn needs(&self) -> Option<&'a Profile>;
}

// A closure cannot lend out its captures, so closure rules return 'static
// data (owned strings, literals, `&'static Profile`). Rules that lend from
// their own state implement `Rule` on a struct.
pub fn rule_fn<F>(f: F) -> RuleFn<F>
    where F: Fn(RuleInput<'_>) -> RuleResult<'static>;
impl<F> RuleFn<F> { pub fn with_ascii_triggers(self, set: AsciiSet) -> Self; }
// RuleFn has a manual Debug impl that prints `RuleFn(..)`.

// chain.rs
pub struct RuleChain<L> { /* rules: L; room for chain-level options */ }
impl<L: RuleList> RuleChain<L> { pub const fn new(rules: L) -> Self; }
impl<L: RuleList + Debug> Rule for RuleChain<L> { /* first match wins;
    ascii_triggers = union over the members */ }
pub type DynRuleChain<'r>      = RuleChain<Vec<Box<dyn Rule + Send + Sync + 'r>>>;
pub type LocalDynRuleChain<'r> = RuleChain<Vec<Box<dyn Rule + 'r>>>;
// On both aliases: empty(), push(rule) (boxes for you), with_rule(rule) -> Self.

// replacement_protection.rs
pub enum ValueMode { TextOnly, MathOnly, AnyMode }
pub enum ValueTermination { ValueIsSelfTerminating, ValueEndsWithNamedMacro }
impl ValueTermination { pub const fn inspect(encoded: &str) -> Self; }
#[non_exhaustive]
pub enum ReplacementProtectionHint {
    DoNotProtect,
    Value { mode: ValueMode, termination: ValueTermination },
}
impl ReplacementProtectionHint {            // each inspects the termination
    pub const fn text_only(encoded: &str) -> Self;
    pub const fn math_only(encoded: &str) -> Self;
    pub const fn any_mode(encoded: &str) -> Self;
}

pub trait ReplacementProtection: Debug {
    fn write_protected<O: OutBuffer, Rep: EncodeReporter>(
        &self, out: &mut O, report: &mut Rep, item: ProtectInput<'_>,
    ) -> Result<(), BoxError>;
}
pub struct ProtectInput<'v> { /* encoded: &'v str, hint */ }  // accessors only
pub enum OutputMode { TextMode, MathMode }
pub enum MacroNameProtection { BracesAround, BracesAfter, SpaceAfterMacroName, NoProtection }
pub struct ModeWrapper {
    pub open: Cow<'static, str>,
    pub close: Cow<'static, str>,
    pub needs: Option<&'static Profile>,     // e.g. \text{..} needs amsmath
}
pub struct StandardProtection {
    pub output_mode: OutputMode,
    pub protect_names: MacroNameProtection,
    pub math_wrap: ModeWrapper,              // math value in text output
    pub text_wrap: ModeWrapper,              // text value in math output
}
impl StandardProtection {
    pub const fn text_mode() -> Self;        // BracesAround
    pub const fn math_mode() -> Self;        // SpaceAfterMacroName
}                                            // Default = text_mode()
pub struct BracesAroundAll(pub StandardProtection);

// report.rs
pub trait EncodeReporter {
    fn report_needs(&mut self, profile: &Profile) {}
    fn report_unknown_char(&mut self, ch: char, position: usize) {}
}
pub struct NoReport;
pub struct EncodeReport { pub needs: PreambleNeeds, pub unknown_chars: BTreeSet<char> }

// outbuffer.rs
pub trait OutBuffer {
    fn push_str(&mut self, s: &str) -> Result<(), BoxError>;
    fn push_char(&mut self, c: char) -> Result<(), BoxError> { /* via push_str */ }
}
impl OutBuffer for String { .. }
pub struct FmtOut<W: core::fmt::Write>(pub W);
#[cfg(feature = "std")] pub struct IoOut<W: std::io::Write>(pub W);

// normalizer.rs
pub trait InputNormalizer: Debug {
    fn normalize<'t>(&self, text: &'t str) -> Cow<'t, str>;
}
pub struct NormalizeNfc;      // quick check first; borrows when already NFC
pub struct NoNormalization;   // always borrows
pub fn nfc(text: &str) -> Cow<'_, str>;

// asciiset.rs
pub struct AsciiSet(u128);
impl AsciiSet {
    pub const ALL: Self; pub const EMPTY: Self;
    pub const fn of(chars: &str) -> Self;                  // panics on non-ASCII
    pub const fn range(r: RangeInclusive<u8>) -> Self;
    pub fn from_fn(f: impl Fn(u8) -> bool) -> Self;        // evaluated once
    pub const fn union(self, other: Self) -> Self;         // also `|`
    pub const fn contains(self, byte: u8) -> bool;
}

// unknown_char.rs
pub enum UnknownCharPolicy {                  // Default = Keep; manual Debug
    Keep, Ignore, Fail,
    ReplaceWith(Cow<'static, str>),
    Callback(Box<dyn Fn(char) -> String + Send + Sync>),
}
impl UnknownCharPolicy {
    pub fn callback(f: impl Fn(char) -> String + Send + Sync + 'static) -> Self;
}
pub fn unknown_unihex(ch: char) -> String;

// encoder.rs
pub struct Encoder<R, P = StandardProtection, N = NormalizeNfc> { .. }
impl<R: Rule> Encoder<R> { pub fn new(rule: R) -> Self; }
impl<R: Rule, P: ReplacementProtection, N: InputNormalizer> Encoder<R, P, N> {
    pub fn with_protection<P2>(self, p: P2) -> Encoder<R, P2, N>;
    pub fn with_normalizer<N2>(self, n: N2) -> Encoder<R, P, N2>;
    pub fn with_unknown_chars(self, policy: UnknownCharPolicy) -> Self;
    pub fn protection(&self) -> &P;

    pub fn encode(&self, text: &str) -> Result<String, EncodeError>;
    pub fn encode_with_report(&self, text: &str)
        -> Result<(String, EncodeReport), EncodeError>;
    pub fn encode_into<O: OutBuffer, Rep: EncodeReporter>(
        &self, text: &str, out: &mut O, report: &mut Rep,
    ) -> Result<(), EncodeError>;
}
#[non_exhaustive]
pub enum EncodeError {                        // Display + core::error::Error
    UnknownChar { ch: char, position: usize },
    Rule { position: usize, source: BoxError },
    Output(BoxError),                         // out buffer or protection failed
}

// lib.rs: DEFAULTS and all default settings. Cannot fail under those.
pub fn encode(text: &str) -> String;
```

`Encoder::new` is not `const` (it calls `ascii_triggers`); construction is
cheap, and an encoder is meant to be built once and reused.

### Behavior specifications

**Termination inspection** (`ValueTermination::inspect`): the value ends with
a named macro when it ends in a run of one or more ASCII letters that is
immediately preceded by a backslash (`\textemdash`, `\hat\i`). Everything
else is self-terminating (`\'e`, `\r{A}`, `\#`, `\ensuremath{\alpha}`, `fi`,
the empty string). Use ASCII letters only, not Unicode `is_alphabetic`.

**`StandardProtection`**, for a value with hint `h`:

- `DoNotProtect`: write `encoded` as is.
- The value's mode is valid in the output mode (`AnyMode`; `TextOnly` in
  `TextMode`; `MathOnly` in `MathMode`): if the termination is
  `ValueEndsWithNamedMacro`, apply `protect_names`; otherwise write `encoded`
  as is.

  | `protect_names`       | written           |
  |-----------------------|-------------------|
  | `BracesAround`        | `{` encoded `}`   |
  | `BracesAfter`         | encoded `{}`      |
  | `SpaceAfterMacroName` | encoded + a space |
  | `NoProtection`        | encoded           |

- Otherwise (mode mismatch): write `wrap.open`, `encoded`, `wrap.close`,
  using `math_wrap` for a math value in text output and `text_wrap` for a
  text value in math output. If `wrap.needs` is set, call
  `report.report_needs`. No macro-name protection is added: a wrapped value
  is self-terminating.
- Defaults: `text_mode()` = `TextMode`, `BracesAround`, `math_wrap` =
  `\ensuremath{` .. `}`; `math_mode()` = `MathMode`, `SpaceAfterMacroName`.
  The default `text_wrap` is an open item; use `\textnormal{` .. `}` with no
  needs until it is decided.

**`BracesAroundAll`**: `DoNotProtect` is written as is. Any other value gets
the mode wrapping of the inner `StandardProtection` (its `protect_names` is
ignored) and is then wrapped in `{` `}`, the empty string included (`{}`).

**Unknown chars**: for each one, call `report.report_unknown_char(ch, pos)`
first, whatever the policy. Then `Keep` copies the char, `Ignore` writes
nothing, `Fail` returns `EncodeError::UnknownChar`, `ReplaceWith(s)` writes
`s`, `Callback(f)` writes `f(ch)`. None of this output is protected.

**`unknown_unihex(ch)`** returns
`\ensuremath{\langle}\texttt{U+XXXX}\ensuremath{\rangle}` with the code point
in uppercase hex, at least four digits.

**Needs**: `PreambleNeeds::chunks()` yields all package chunks first, then
all snippet chunks, each group in first-seen order (a snippet may call into
a package of its own profile).

### Encoder loop

1. `text = normalizer.normalize(text)`.
2. Scan bytes. An ASCII byte outside the stop set extends the current run.
3. At a stop (a non-ASCII char or an ASCII char in the stop set): decode the
   char and consult the rule. A rule error becomes
   `EncodeError::Rule { position, .. }`.
4. On a match: flush the run, write the value through the protection
   strategy, report the needs if the profile is not the one last reported in
   this call (compare references with `ptr::eq`; the shortcut is a local
   variable, reset for every call), advance by `consumed`.
5. No match: printable ASCII and `\n`, `\r`, `\t` stay in the run; anything
   else is an unknown char.
6. Flush the last run.

### Needs data model

```rust
pub struct Chunk { pub id: Cow<'static, str>, pub preamble: ChunkPreamble }
pub enum ChunkPreamble {
    Package(Cow<'static, str>),                                // name
    PackageWithOptions(Cow<'static, str>, Cow<'static, str>),  // name, options
    Snippet(Cow<'static, str>),                                // raw preamble lines
}
impl Chunk {   // const constructors for static data
    pub const fn package(name: &'static str) -> Self;          // id = name
    pub const fn package_with_options(id: &'static str, name: &'static str,
                                      options: &'static str) -> Self;
    pub const fn snippet(id: &'static str, text: &'static str) -> Self;
}
pub struct Profile { /* Cow<'static, [Chunk]> */ }
impl Profile {
    pub const fn from_static(chunks: &'static [Chunk]) -> Self;
    pub fn new(chunks: Vec<Chunk>) -> Self;
    pub fn chunks(&self) -> &[Chunk];
}
pub struct ProfileIndex(pub u8);   // index into a table's own profile array
pub struct PreambleNeeds { /* distinct chunks, keyed by id */ }
// new(), include(&Profile), merge(&PreambleNeeds), is_empty(), chunks(),
// write_preamble(&mut impl OutBuffer), impl EncodeReporter
```

- The builtin chunks and profiles are the 20 chunks and 21 profiles of
  `initial-rust-port/src/tables.rs` (`CHUNKS`, `PROFILES`, and the constants
  below them), restructured: `\usepackage{amssymb}` becomes
  `Chunk::package("amssymb")`; `\usepackage[T2A,T1]{fontenc}` becomes
  `Chunk::package_with_options("fontenc-t2a", "fontenc", "T2A,T1")`; snippets
  keep their ids. Structured options let a consumer merge several `fontenc`
  requests into one `\usepackage` line.
- Profile index 0 is reserved for "no needs" and resolves to `None`.
- `Cow` has drop glue, so `&[..]` literals of chunks are not promoted to
  statics on their own. Define the profiles through a small macro that also
  creates the backing statics for the chunk lists.

### Tables

```rust
pub trait LookupTable: Debug {
    fn lookup(&self, ch: char) -> Option<TableEntry<'_>>;
    fn ascii_keys(&self) -> AsciiSet { AsciiSet::ALL }
}
pub struct TableEntry<'t> {
    pub encoded: &'t str,
    pub hint: ReplacementProtectionHint,
    pub needs: Option<&'t Profile>,
}
pub struct TableRule<T>(pub T);   // makes any user LookupTable a Rule
pub struct DynTable { .. }        // built at run time; sorted Vec + binary search
pub struct OnlyAscii<T>(pub T);   // view: answers only for ASCII chars
pub struct ExceptAscii<T>(pub T); // view: answers only for non-ASCII chars
```

- The crate's own table types implement `Rule` directly (one lookup on
  `input.ch()`, `replace_char`, `ascii_triggers` = `ascii_keys`).
  `TableRule` is for user-defined `LookupTable` impls; a blanket impl would
  clash with the `&R` / `Box<R>` forwarding impls.
- `NON_ASCII` and `ASCII_SPECIALS` are `ExceptAscii` / `OnlyAscii` views
  holding a reference to the `DEFAULTS` static, so all three share one copy
  of the data. `ExceptAscii` reports `AsciiSet::EMPTY` as its triggers.
- Keep the approach of `src/statictable.rs`: `const fn` builders turn a
  sorted entry slice into one of three layouts (binary search over a
  separate key array; two-level with a linear scan inside a 256-code-point
  block; two-level with a direct 256-slot index per block), and a
  `macro_rules!` macro declares the backing statics. The layout is an
  argument of the macro; use the two-level direct index for the builtin
  table until the benchmark decides.
- Macro input stays a plain `const` slice passed as an expression (a
  1549-entry token list would strain `macro_rules!`):

  ```rust
  const ENTRIES: &[(char, &str, ValueMode, ProfileIndex)] = &[
      ('\u{00E9}', r"\'e",    TEXT, BUILTINS), // LATIN SMALL LETTER E WITH ACUTE
      ('\u{03B1}', r"\alpha", MATH, BUILTINS), // GREEK SMALL LETTER ALPHA
  ];
  pub static DEFAULTS: .. = compile_static_table!(ENTRIES, PROFILES, two_level_direct);
  ```
- Compile-time checks (a violation is a compile error): strictly ascending
  keys, profile index in range, ASCII-only encoded strings, balanced braces
  (not counting `\{` and `\}`), no trailing lone backslash, entry count fits
  the index type.
- Compiled per entry: the `&'static str` plus two bytes (termination bit,
  mode bits, profile index). The termination and the table's `ascii_keys`
  are computed by `const fn` at compile time.

### Data migration

`tools/migrate_tables.py` (Python 3, one-off) reads
`initial-rust-port/src/tables.rs` and writes the `ENTRIES` list of
`src/builtin/default_table.rs`. After the migration, that Rust file is the
hand-maintained source of truth.

- Entry format in the source:
  `('\u{XXXX}', r#"..."#, PROFILE), // UNICODE NAME` (1549 entries). Keep the
  name comments.
- Mode assignment: a value of the form `\ensuremath{X}` whose opening brace
  closes at the very end becomes `X` with `MATH` (935 entries). A value with
  no backslash at all is `ANY` (34 entries, among them the empty string for
  U+2061). Everything else is `TEXT` (580 entries). Three mixed entries keep
  their inner `\ensuremath` and are `TEXT`: U+038F `\'{}\ensuremath{\Omega}`,
  U+2109 `\ensuremath{^\circ}F`, U+25AA `{\small\ensuremath{\blacksquare}}`.
- Rename every `\flm` command prefix to `\UnxT` (68 entries use one; the
  snippets define them). Inside the snippets, also rename the internal font
  identifiers that start with `flm` (for example `flmstixcal`, `flmwasy`).
- The script asserts, for every entry, that the text-mode rendering of the
  new entry (`\ensuremath{` + X + `}` for `MATH`, the string itself
  otherwise) equals the old string after the prefix rename.
- Apply the same prefix rename to the copied golden file and review that
  diff; it is the only expected change to the golden.
- Carry over by hand into the module docs of `default_table.rs`: the
  provenance notes, the list of corrections over pylatexenc, and both MIT
  license notices. Replace references to FLM.

### Tests

- Port `initial-rust-port/tests/latexencode.rs` to the new API; copy the
  golden file to `tests/goldens/`. Adaptations:
  - `braces-all` / `braces-almost-all` are no longer modes. One test covers
    `BracesAroundAll`.
  - The per-rule protection override test becomes a `DoNotProtect` test.
  - The DEL quirk test is replaced by tests of the new unknown-char
    definition (including: a rule can match a control char).
  - `unihex` is tested through `UnknownCharPolicy::callback(unknown_unihex)`.
  - Zero consumption: `replace_prefix(0, ..)` panics and
    `try_replace_prefix(0, ..)` returns an error.
- Golden test: keep the golden under its original protection mode by
  defining, in the test file, a custom `ReplacementProtection` that
  reproduces `braces-almost-all` (compute the text-mode rendering as above;
  wrap it in braces if it starts with a backslash), with
  `UnknownCharPolicy::Fail`. Output must be byte-identical to the golden.
  This validates the data migration, the mode wrapping, and the encoder loop
  in one go, without reviewing 1600 changed lines by eye.
- The structural table tests become the compile-time checks above. The test
  that every snippet's profile also loads the `fontenc` encoding the snippet
  refers to stays a runtime test.
- New tests: `ascii_triggers` equivalence (same output as with `ALL` for
  every rule); math output mode; `SpaceAfterMacroName`; output equality
  between `NoReport` and `EncodeReport`; `Send + Sync` assertions for a
  static-chain encoder and for `DynRuleChain`; a `LocalDynRuleChain` holding
  an `Rc`; a run-time `Profile` from a user rule ending up in the report;
  streaming through `FmtOut`.
- Benchmarks (criterion, dev-only): ASCII-heavy text, accented Latin,
  Greek/math, Cyrillic, CJK under `Keep`; one run per table layout. Spot
  checks with `cargo asm` / `cargo bloat`: the needs logic is gone under
  `NoReport`, the NFC tables are gone under `NoNormalization`.
- LaTeX compile check (manual at first): one document that exercises every
  builtin profile, the `\UnxT` snippets, and the text-in-math wrapper
  candidates.

### Implementation order

1. Core skeleton that compiles: `rule`, `chain`, hint types, `asciiset`,
   `outbuffer`, `report`, `preamble`, `profile`, `StandardProtection`,
   `BracesAroundAll`, the encoder loop with `NoNormalization` only. Unit
   tests against a tiny `DynTable`. This step validates every signature in
   this plan; update the plan where Rust disagrees.
2. `LookupTable`, the static layouts, `compile_static_table!` with its
   compile-time checks, the `OnlyAscii` / `ExceptAscii` views.
3. Data migration (script, builtin chunks and profiles, `DEFAULTS`,
   `NON_ASCII`, `ASCII_SPECIALS`, module docs with provenance and licenses).
4. Port the test suite and the golden file.
5. `InputNormalizer` with `NormalizeNfc` as the default, `UnknownCharPolicy`
   callback, `unknown_unihex`, the free `encode()`, crate-level docs.
6. Benchmarks, table layout decision, `cargo asm` / `cargo bloat` checks.
7. Later, outside this plan: `unicode-xml` table generator; language
   bindings.

## Open items

- Default text-in-math wrapper (`\mbox`, `\text`, `\textnormal`): decide
  after compiling a test document. `\mbox` needs only the kernel but does not
  shrink in sub/superscripts; `\text` sizes correctly but needs amsmath;
  `\textnormal` is believed to behave like `\mbox` alone and like `\text`
  once amsmath is loaded (to be verified).
- Static table layout and payload packing (for example one string blob with
  offsets instead of one `&str` per entry): decide by benchmark.
- `unicode-xml` table: pylatexenc's generated dict
  (`latexencode/_uni2latexmap_xml.py`, 2233 entries) has no mode column, no
  needs, and no license header, and it mixes bare math macros with text
  macros. It would have to be regenerated from the W3C `unicode.xml` source
  (check and carry its license), probably behind a cargo feature to spare
  everyone the compile time of its `const fn` build.
- Refine modes in the builtin table: some `TEXT` entries (`\#`, `\%`, `\&`,
  ..) are valid in math mode too and could become `ANY`.
