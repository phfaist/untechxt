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
  another project (FLM), where it was the crate `flm-latexencode`. 
  It has no `Cargo.toml`, so it cannot be built or run: its behavior can only
  be read. It is reference material, and three parts of it are carried over
  into tracked files (steps 3 and 4), after which the repository no longer
  depends on it:
  - `src/tables.rs`: the data. 1549 hand-curated entries
    `(char, latex, PROFILE), // UNICODE NAME`, plus 20 preamble chunks, 21
    profiles, the provenance notes, a documented table of corrections over
    pylatexenc ("Departures from pylatexenc"), and two MIT license notices
    (pylatexenc and latexcodec) that must travel with the data.
  - `tests/latexencode.rs` and `tests/goldens/latexencode/` (the golden file
    `uni_chars_test_previous.txt` and a `README.md`): the behavioral spec.
    Each golden line is the **encoded** form of an input line built with the
    format string `"0x%04X %-50s    |%s|\n"`, where the `%-50s` field is
    `[UNICODE NAME]` and the last field is the character itself. So the
    stored lines look like `0x0023 [NUMBER SIGN]  ...  |{\#}|`. The old test
    recovers the code point and the name from each golden line, rebuilds the
    input line, encodes it, and compares byte for byte (`UPDATE_GOLDEN=1`
    rewrites the file). The golden was produced with NFC normalization,
    pylatexenc's `braces-almost-all` protection, and the `fail` unknown-char
    policy. 21 of its lines only encode because NFC maps the character onto
    one that has a table entry.
  - `src/lib.rs` and `src/needs.rs`: the old encoder loop and needs model
    (for reference), and documentation prose worth migrating.
- `src/`: an exploratory prototype of the new design. **It does not compile**
  and contains leftover pasted fragments. Treat it as a statement of intent
  and rewrite it: replace the files that the module layout names, delete
  prototype files it does not name. Files ending in `~` are git-ignored
  editor backups; leave them alone. Two things are kept as they are in spirit:
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
  `apply<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a>` where
  `RuleResult<'a> = Result<Option<EncodedReplacement<'a>>, BoxError>`:
  `Ok(None)` means "does not match here", `Err` aborts the encoding. The
  shared lifetime lets a result borrow from the rule **or from the input**
  (a rule that passes existing LaTeX through returns a slice of the input
  without allocating).
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
  or out-of-range `consumed` therefore cannot be constructed from the
  `RuleInput` the encoder handed to the rule, the encoder needs no per-match
  validation, and there is no "invalid consumption" error. A rule that
  fabricates a `RuleInput` over some other string (`RuleInput::new` is public
  for testing rules) is out of contract; the encoder then only checks the
  advanced position in debug builds (`debug_assert!` on the char boundary).
- `RuleInput<'s>` is a small `Copy` struct with private fields and accessors,
  so context can be added later without breaking every `Rule` impl.
- Closures become rules through `rule_fn(..)`. A blanket
  `impl<F: Fn(..)> Rule for F` is impossible: it overlaps with the `&R` and
  `Box<R>` forwarding impls, because `&F` and `Box<F>` are themselves `Fn`.
  A closure rule can return owned strings, literals, or slices of the input,
  but it cannot lend out its own captures; a rule that lends from its own
  state implements `Rule` on a struct.
- Rules are fallible so that foreign-language callbacks (Python, JS) can
  propagate exceptions. Error payloads are
  `BoxError = Box<dyn Error + Send + Sync>` (the ecosystem convention; a JS
  exception is stringified on the way in).

### Chains

- One generic wrapper `RuleChain<L>` implements `Rule`. A dedicated struct
  (instead of implementing `Rule` on bare tuples) leaves room for chain-level
  options later, such as "continue with the next rule if one fails". Do not
  implement such options now.
- `L: RuleList`, a sealed helper trait implemented for tuples of arity 0 to
  12 (static dispatch, unrolled; the empty tuple is the empty chain),
  `[R; N]`, and `Vec<R>`. Bare tuples and `Vec`s
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
- `ReplacementProtection` has a generic method and is therefore not
  dyn-compatible. That is accepted: `StandardProtection` is an ordinary
  struct whose fields can be set at run time (which is what language bindings
  need), and a fully custom strategy is a compile-time choice.
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
  the buffer and the report stays partially filled. `Display` names the
  position, and the two variants that wrap an error repeat its message and
  hand it out through `Error::source`.
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
  static chain. `report_needs` may be called any number of times for the
  same profile (the encoder only skips immediate repeats), so implementations
  must be idempotent; the trait docs say so.

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

- The default rules are obtained with the root function `default_rules()`,
  which returns the opaque type `DefaultRules` (a `Rule` that is `Copy`,
  `Send` and `Sync`, applied without dynamic dispatch). It holds the builtin
  table `DEFAULT_TABLE` today; it can become a `RuleChain` of several rules
  later with no API change. Every member of the default rules is also
  exported from `builtin::`, so that a variant can be assembled by hand.
  `DefaultRules` is the default of `Encoder`'s type parameter `R`.
- Builtin tables, in `builtin::`: `DEFAULT_TABLE` (all entries),
  `DEFAULT_TABLE_NON_ASCII`, `DEFAULT_TABLE_ASCII_SPECIALS` (the 13 entries
  `" # $ % & < > \ ^ _ { } ~`), from one source list.
  `DEFAULT_TABLE_NON_ASCII` shares the compiled data of `DEFAULT_TABLE` (its
  initializer copies the layout struct, which is a few `&'static` slices);
  `DEFAULT_TABLE_ASCII_SPECIALS` is a small table compiled by itself from the
  ASCII head of the list.
- All three builtin tables have the one opaque public type `BuiltinTable`
  (private fields: one of the layout structs, and a flag that makes the
  table ignore its ASCII entries), so the table layout stays an
  implementation detail and can change without breaking the API, and a
  program can choose a builtin table at run time. The three layout structs
  themselves are public for users' own tables. There are no public
  ASCII-filtering views of a user's table (`OnlyAscii` / `ExceptAscii` were
  removed in the API namespace review).
- No cargo features for binary size: unused statics are dropped at link
  time. The linker drops a static whole or not at all, so a table that
  refers to another static table links all of it; that is why
  `DEFAULT_TABLE_ASCII_SPECIALS` is compiled separately. More builtin tables
  are anticipated (see "Open items").
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
- `criterion` is added as a dev-dependency at step 6, together with
  `[[bench]] name = "encode", harness = false` and `[lib] bench = false` (so
  that `cargo bench` hands its options to criterion rather than to the
  library's own empty libtest harness), and an `exclude` that keeps the
  reference-only material out of the published package: `/initial-rust-port`,
  `/tools`, `/PLAN.md`, `/CLAUDE.md`. `tests/` with its golden, `benches/`,
  `examples/`, `README.md` and both license files stay in the package.
- `missing_docs = "deny"` also fires on public named struct fields and enum
  variants (not on tuple-struct fields): `Chunk`, `ModeWrapper`,
  `StandardProtection`, `TableEntry`, `EncodeReport` need per-field docs.
- The `Cargo.toml` changes (`name`, `[features]`) are part of step 1.
- Step 5 adds the publication metadata to `[package]`: `description`,
  `readme = "README.md"`, `keywords` (`latex`, `unicode`, `encoding`,
  `text-processing`, `no-std`) and `categories` (`text-processing`,
  `encoding`, `no-std`, all valid crates.io categories), beside the
  `license = "MIT OR Apache-2.0"` that was already there.

### Module layout

Every public item has exactly one public path, and no concept is split
between the crate root and a module:

- The crate root holds what most programs need: `encode()`,
  `default_rules()` with its type `DefaultRules`, `Encoder` with
  `EncodeError`, the cross-cutting `BoxError`, and `UnknownCharPolicy` with
  `unknown_unihex()`.
- Everything else lives in one public module per concept, which holds the
  trait, the shipped implementations and their supporting types together.
- An item is defined in a private module and `pub use`d at exactly one
  public place, or defined in the public module itself. `lib.rs` re-exports
  nothing from a public module.
- Implementation details are hidden: `BuiltinTable` and `DefaultRules` are
  opaque, and what `compile_static_table!` needs to name (`StaticEntry`, the
  `const fn` builders) sits in the `#[doc(hidden)]` module
  `statictable::__build`. The macro is exported at the root under the hidden
  name `__compile_static_table` (`#[macro_export]` forces the root) and is
  documented only as `statictable::compile_static_table!`.

```
src/
  lib.rs                     crate docs, BoxError, free fn encode(), root
                             re-exports of the private modules below
  defaults.rs     (private)  default_rules(), DefaultRules
  encoder.rs      (private)  Encoder<R, P, N>, EncodeError
  unknown_char.rs (private)  UnknownCharPolicy, unknown_unihex
  rule/
    mod.rs                   Rule, RuleInput, EncodedReplacement, RuleResult,
                             InvalidPrefixLength, rule_fn / RuleFn,
                             forwarding impls (&R, Box<R>, Option<R>)
    chain.rs      (private)  RuleChain<L>, sealed RuleList, DynRuleChain,
                             LocalDynRuleChain; re-exported from `rule`
    asciiset.rs   (private)  AsciiSet; re-exported from `rule`
  lookuptable.rs             LookupTable, TableEntry, TableRule, DynTable
  statictable.rs             static layouts, ProfileIndex,
                             compile_static_table!, hidden `__build`
  builtin/
    mod.rs                   BuiltinTable, DEFAULT_TABLE,
                             DEFAULT_TABLE_NON_ASCII,
                             DEFAULT_TABLE_ASCII_SPECIALS
    needs_profiles.rs        builtin chunks and profiles
    default_table.rs         the entries: hand-maintained source of truth,
                             with provenance notes and both MIT notices
  protection.rs              hint types, ReplacementProtection, ProtectInput,
                             StandardProtection, MacroNameProtection,
                             OutputMode, ModeWrapper, BracesAroundAll
  preamble/
    mod.rs                   Chunk, ChunkPreamble
    profile.rs    (private)  Profile, PreambleNeeds; re-exported from
                             `preamble`
  report.rs                  EncodeReporter, NoReport, EncodeReport
  normalizer.rs              InputNormalizer, NormalizeNfc, NoNormalization,
                             nfc()
  outbuffer.rs               OutBuffer, FmtOut, IoOut (feature "std")
tests/
  core.rs                    the core API: the rules, the chain, the
                             protection strategies, the encoder loop and what
                             it reports (step 1)
  statictable.rs             the three static layouts over one small table,
                             and `compile_static_table!` (step 2)
  builtin.rs                 the builtin tables, chunks and profiles (step 3)
  latexencode.rs             the ported pylatexenc suite and the golden
                             test (step 4)
  goldens/latexencode/       the conformance golden
                             `uni_chars_test_previous.txt`, and its `README.md`
tools/
  migrate_tables.py          one-off data migration script (step 3)
benches/
  encode.rs                  criterion benchmarks: five corpora against the
                             three table layouts, the report, the normalizer
examples/
  size_check.rs              the smallest program that uses the encoder, for
  size_check_nfc.rs          `cargo bloat` / `cargo asm`; the pair differs
                             only in the input normalizer
```

### Core API sketch

```rust
// lib.rs
pub type BoxError = Box<dyn core::error::Error + Send + Sync + 'static>;

// rule/mod.rs
pub type RuleResult<'a> = Result<Option<EncodedReplacement<'a>>, BoxError>;

pub trait Rule: Debug {
    // One lifetime for `self` and the input: the result may borrow from
    // either. (With `RuleInput<'_>` a rule could not return a slice of the
    // input; verified by compiling on Rust 1.86. This form stays
    // dyn-compatible.)
    fn apply<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a>;
    fn ascii_triggers(&self) -> AsciiSet { AsciiSet::ALL }
}

#[derive(Debug, Clone, Copy)]
pub struct RuleInput<'s> { /* full: &'s str, pos: usize, ch: char */ }
impl<'s> RuleInput<'s> {
    pub fn new(full: &'s str, pos: usize) -> Option<Self>;  // for testing rules
    pub const fn ch(&self) -> char;    // the char at pos, already decoded
    pub const fn pos(&self) -> usize;
    pub const fn full(&self) -> &'s str;
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
/// Display + Error; `?` boxes it into a `BoxError` by the alloc blanket impl.
pub struct InvalidPrefixLength {
    pub position: usize, pub n_bytes: usize, pub available: usize,
}

pub struct EncodedReplacement<'a> { /* consumed, encoded, hint, needs */ }
impl<'a> EncodedReplacement<'a> {
    pub fn with_needs(self, profile: &'a Profile) -> Self;
    pub const fn consumed(&self) -> usize;
    pub fn encoded(&self) -> &str;
    pub const fn hint(&self) -> ReplacementProtectionHint;
    pub const fn needs(&self) -> Option<&'a Profile>;
}

// A closure rule may return owned strings, literals, `&'static Profile`, or
// slices of the input. It cannot lend out its own captures; rules that lend
// from their own state implement `Rule` on a struct. (Verified on Rust 1.94:
// closure inference accepts the higher-ranked bound, unannotated closures
// included, so the `-> RuleResult<'static>` fallback was not needed.)
pub fn rule_fn<F>(f: F) -> RuleFn<F>
    where F: for<'s> Fn(RuleInput<'s>) -> RuleResult<'s>;
impl<F> RuleFn<F> { pub fn with_ascii_triggers(self, set: AsciiSet) -> Self; }
// RuleFn has a manual Debug impl that prints `RuleFn(..)`.

// rule/chain.rs
pub struct RuleChain<L> { /* rules: L; room for chain-level options */ }
impl<L: RuleList> RuleChain<L> {
    pub const fn new(rules: L) -> Self;
    pub fn rules(&self) -> &L;
}
impl<L: RuleList + Debug> Rule for RuleChain<L> { /* first match wins;
    ascii_triggers = union over the members */ }
// RuleList (sealed): tuples of arity 0 to 12 (the empty tuple never
// matches), [R; N], Vec<R>.
pub type DynRuleChain<'r>      = RuleChain<Vec<Box<dyn Rule + Send + Sync + 'r>>>;
pub type LocalDynRuleChain<'r> = RuleChain<Vec<Box<dyn Rule + 'r>>>;
// On both aliases: empty(), push(rule) (boxes for you), with_rule(rule) -> Self.

// protection.rs
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
pub struct ProtectInput<'v> { /* encoded: &'v str, hint */ }
impl<'v> ProtectInput<'v> {                  // accessors, and a constructor so
    pub const fn new(encoded: &'v str,       // that a caller can write a value
        hint: ReplacementProtectionHint) -> Self;   // of its own through a
    pub const fn encoded(&self) -> &'v str;         // strategy, or test one
    pub const fn hint(&self) -> ReplacementProtectionHint;
}
pub enum OutputMode { TextMode, MathMode }
pub enum MacroNameProtection { BracesAround, BracesAfter, SpaceAfterMacroName, NoProtection }
pub struct ModeWrapper {
    pub open: Cow<'static, str>,
    pub close: Cow<'static, str>,
    pub needs: Option<&'static Profile>,     // e.g. \text{..} needs amsmath
}
impl ModeWrapper {                           // const, for `text_mode()` & co.
    pub const fn new(open: &'static str, close: &'static str) -> Self;
    pub const fn with_needs(open: &'static str, close: &'static str,
                            needs: &'static Profile) -> Self;
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
    /// The wrapper this strategy applies to a value with this hint: `None`
    /// for `DoNotProtect` and for a value valid in the output mode. Public so
    /// that custom strategies (such as `BracesAroundAll`) can reuse the mode
    /// handling.
    pub fn mode_wrapper_for(&self, hint: ReplacementProtectionHint)
        -> Option<&ModeWrapper>;
}                                            // Default = text_mode()
pub struct BracesAroundAll(pub StandardProtection);

// report.rs
pub trait EncodeReporter {
    fn report_needs(&mut self, profile: &Profile) {}
    fn report_unknown_char(&mut self, ch: char, position: usize) {}
}
pub struct NoReport;
pub struct EncodeReport { pub needs: PreambleNeeds, pub unknown_chars: BTreeSet<char> }
// EncodeReport::new(); Default on both.

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

// rule/asciiset.rs
pub struct AsciiSet(u128);
impl AsciiSet {
    pub const ALL: Self; pub const EMPTY: Self;
    pub const fn of(chars: &str) -> Self;                  // panics on non-ASCII
    pub const fn range(r: RangeInclusive<u8>) -> Self;
    pub fn from_fn(f: impl Fn(u8) -> bool) -> Self;        // evaluated once
    pub const fn union(self, other: Self) -> Self;         // `|` also works,
                                                           // but not in const
    pub const fn contains(self, byte: u8) -> bool;         // false for >= 128
                                                           // (guard the shift)
    pub const fn is_empty(self) -> bool;
}
// Manual Debug: `AsciiSet::ALL`, `AsciiSet::EMPTY`, or `AsciiSet("#$%&")` with
// non-printable members as `\xNN`.

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
// Debug, not Clone: an UnknownCharPolicy may hold a boxed callback.
pub struct Encoder<R = DefaultRules, P = StandardProtection, N = NormalizeNfc> { .. }
impl<R: Rule> Encoder<R> { pub fn new(rule: R) -> Self; }
impl<R: Rule, P: ReplacementProtection, N: InputNormalizer> Encoder<R, P, N> {
    // The replacement is bound too, so that an encoder never holds a type
    // that is not a strategy or not a normalizer.
    pub fn with_protection<P2: ReplacementProtection>(self, p: P2)
        -> Encoder<R, P2, N>;
    pub fn with_normalizer<N2: InputNormalizer>(self, n: N2)
        -> Encoder<R, P, N2>;
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

// defaults.rs: the default rules, as an opaque `Rule` (Copy + Send + Sync).
pub const fn default_rules() -> DefaultRules;
pub struct DefaultRules { /* private: &'static BuiltinTable for now */ }

// lib.rs: default_rules() and all default settings. Cannot fail under those.
pub fn encode(text: &str) -> String;
```

`Encoder::new` is not `const` (it calls `ascii_triggers`); construction is
cheap, and an encoder is meant to be built once and reused. The free
`encode()` builds its encoder on every call; for a static chain that is a
few instructions (there is no `no_std` cell to cache it in).

`ReplacementProtectionHint` is `#[non_exhaustive]`, so a strategy written
outside the crate (including the golden test's) needs a wildcard match arm.

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
the mode wrapping of the inner `StandardProtection` (through
`mode_wrapper_for`; its `protect_names` is ignored) and is then wrapped in
`{` `}`, the empty string included (`{}`). It uses only public API, so it
serves as the example of a custom strategy.

**Unknown chars**: for each one, call `report.report_unknown_char(ch, pos)`
first, whatever the policy. Then `Keep` copies the char, `Ignore` writes
nothing, `Fail` returns `EncodeError::UnknownChar`, `ReplaceWith(s)` writes
`s`, `Callback(f)` writes `f(ch)`. None of this output is protected.

**`unknown_unihex(ch)`** returns
`\ensuremath{\langle}\texttt{U+XXXX}\ensuremath{\rangle}` with the code point
in uppercase hex, at least four digits.

**Needs**: `PreambleNeeds::chunks()` yields all package chunks first, then
all snippet chunks, each group in first-seen order (a snippet may call into
a package of its own profile). `write_preamble` writes one line per package
chunk (`\usepackage{name}` or `\usepackage[options]{name}`) and then the
snippets; it does **not** merge options. That is safe for the builtin chunks:
the only package loaded under several chunks is `fontenc`, which is written
to be loaded repeatedly without an option clash, and every builtin `fontenc`
chunk lists `T1` last, so the document's default encoding stays `T1`. Merging
is left to consumers that have their own package machinery.

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
// Chunk and ChunkPreamble derive Debug, Clone, PartialEq, Eq, Hash
// (`Cow<'static, [Chunk]>` requires `Chunk: Clone`).
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
    pub const fn is_package(&self) -> bool;  // a package chunk, not a snippet:
}                                            // what orders `chunks()`
pub struct Profile { /* Cow<'static, [Chunk]> */ }
impl Profile {
    pub const fn from_static(chunks: &'static [Chunk]) -> Self;
    pub fn new(chunks: Vec<Chunk>) -> Self;
    pub fn chunks(&self) -> &[Chunk];
    pub fn is_empty(&self) -> bool;
}
pub struct ProfileIndex(pub u8);   // in `statictable`: index into a table's own
                                   // profile array
impl ProfileIndex { pub const NONE: ProfileIndex; }   // the reserved 0
pub struct PreambleNeeds { /* two Vec<Chunk>, packages and snippets */ }
// new(), include(&Profile), merge(&PreambleNeeds), is_empty(), chunks(),
// write_preamble<O: OutBuffer + ?Sized>(&self, &mut O) -> Result<(), BoxError>,
// impl EncodeReporter, manual Debug (the chunk ids, in order).
// Distinctness is a linear scan over the ids: the sets are a few chunks long.
```

- The builtin chunks and profiles are the 20 chunks and 21 profiles of
  `initial-rust-port/src/tables.rs` (`CHUNKS`, `PROFILES`, and the constants
  below them), restructured: `\usepackage{amssymb}` becomes
  `Chunk::package("amssymb")`; `\usepackage[T2A,T1]{fontenc}` becomes
  `Chunk::package_with_options("fontenc-t2a", "fontenc", "T2A,T1")`; snippets
  keep their ids. Structured options let a consumer merge several `fontenc`
  requests into one `\usepackage` line.
- Profile index 0 is reserved for "no needs". `PROFILES[0]` is a real, empty
  `Profile`, so that indices equal array positions; a table lookup maps
  index 0 to `None` without consulting the array. The builtin profiles are
  `pub static PROFILES: [Profile; 21]` of `builtin/needs_profiles.rs`, with
  one `pub const NAME: ProfileIndex` per profile beside it; `BUILTINS` is
  index 0.
- `Cow` has drop glue, so `&[..]` literals of chunks are not promoted to
  statics on their own (confirmed: `static P: Profile =
  Profile::from_static(&[Chunk::package("x")]);` is rejected, and
  `static C: [Chunk; 1] = ..; static P: Profile = Profile::from_static(&C);`
  is accepted). Define the profiles through a small macro that also creates
  the backing statics for the chunk lists.

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
impl DynTable {                   // owns its strings and its profiles
    pub fn new() -> Self;         // also Default
    pub fn insert(&mut self, ch: char, encoded: impl Into<String>,
                  hint: ReplacementProtectionHint);          // replaces
    pub fn insert_with_needs(&mut self, /* same */, needs: Profile);
    pub fn with_entry(self, /* as insert */) -> Self;        // builder
    pub fn lookup(&self, ch: char) -> Option<TableEntry<'_>>; // via LookupTable
    pub fn iter(&self) -> impl Iterator<Item = (char, TableEntry<'_>)> + '_;
    pub fn len(&self) -> usize;  pub fn is_empty(&self) -> bool;
}
```

- The crate's own table types implement `Rule` directly (one lookup on
  `input.ch()`, `replace_char`, `ascii_triggers` = `ascii_keys`).
  `TableRule` is for user-defined `LookupTable` impls; a blanket impl would
  clash with the `&R` / `Box<R>` forwarding impls. `LookupTable` is also
  implemented for `&T`.
- The crate's table types offer `iter()` over `(char, TableEntry)` in key
  order (used by the tests, and useful for tooling).
- In `builtin::`: `pub static DEFAULT_TABLE: BuiltinTable`;
  `pub static DEFAULT_TABLE_NON_ASCII: BuiltinTable`;
  `pub static DEFAULT_TABLE_ASCII_SPECIALS: BuiltinTable`.
  `DEFAULT_TABLE_NON_ASCII` copies the layout struct of `DEFAULT_TABLE` in
  its initializer, so it shares the compiled data; its private `skip_ascii`
  flag makes `lookup` return `None` for ASCII characters and makes it report
  `AsciiSet::EMPTY` as its triggers. `DEFAULT_TABLE_ASCII_SPECIALS` is
  compiled a second time from the ASCII head of `ENTRIES`, which a `const fn`
  slices off (the list is sorted), so the source is not repeated. It was at
  first a filtering view that held a reference to the full table, and a
  program that used it alone then linked the whole of the full table:
  459,824 bytes, the same as with the full table itself,
  against 375,040 with a hand-written table of the 13 entries (the program of
  `examples/size_check.rs`, release, LTO). Its profile array is a one-element
  array of its own rather than `&PROFILES`, which would link every `\UnxT`
  snippet: all 13 entries name profile 0, and the macro's index check makes
  it a compile error if one ever names another. As compiled now the same
  program is 375,088 bytes, and one that links both tables pays 1,264 bytes
  for the second copy.
- Keep the approach of `src/statictable.rs`: `const fn` builders turn a
  sorted entry slice into one of three layouts, each its own public struct
  (`StaticTableBinarySearch`: binary search over a separate key array;
  `StaticTableTwoLevelLinear`: a block per distinct `code point >> 8`, with a
  linear scan inside the block; `StaticTableTwoLevelDirect`: the same blocks
  with a direct 256-slot index each), and a `macro_rules!` macro declares the
  backing statics. The macro is rewritten to take three arguments: the
  entries, the profiles as a `&'static [Profile]` expression (stored in the
  table, so that `lookup` can turn an entry's index into a
  `&'static Profile`), and the layout, with the prototype's arm names
  `binary_search`, `two_level_linear`, `two_level_direct_index`. Use
  `two_level_direct_index` for the builtin table until the benchmark decides.
  Each layout also carries the table's `ascii_keys` as a compiled `AsciiSet`,
  and offers `iter()`, `len()` and `is_empty()`. The compiled payload is the
  public `StaticEntry` of the `#[doc(hidden)]` module `statictable::__build`,
  which the macro has to name in the `static` items it declares.
- Macro input stays a plain `const` slice passed as an expression (a
  1549-entry token list would strain `macro_rules!`):

  ```rust
  const ENTRIES: &[(char, &str, ValueMode, ProfileIndex)] = &[
      ('\u{00E9}', r"\'e",    TEXT, BUILTINS), // LATIN SMALL LETTER E WITH ACUTE
      ('\u{03B1}', r"\alpha", MATH, BUILTINS), // GREEK SMALL LETTER ALPHA
  ];
  pub static DEFAULT_TABLE: BuiltinTable = BuiltinTable {
      layout: compile_static_table!(ENTRIES, &PROFILES, two_level_direct_index),
      skip_ascii: false,
  };
  ```
  The profile argument is a `&'static [Profile]`: a `const` of that type, or
  `&PROFILES` for a `static PROFILES: [Profile; N]`. A `const` cannot hold a
  reference to a `static`, so the macro reads only its length in a `const`
  item (the check that every index is in range) and passes the expression
  itself to the `static` items it declares.
  The builtin `ENTRIES` const is `#[doc(hidden)] pub`, so that the benchmarks
  can compile the same data in all three layouts.
- Compile-time checks (a violation is a compile error): strictly ascending
  keys, profile index in range, ASCII-only encoded strings, balanced braces
  (not counting `\{` and `\}`), no trailing lone backslash, entry count fits
  the index type. Const evaluation cannot format messages, so a violation
  reports which check failed but not which entry; do not spend time on that.
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
- Rename every `\flm` command prefix to `\UnxT`, leaving the rest of the name
  unchanged: `\flmBbold` becomes `\UnxTBbold`, `\flmsqint` becomes
  `\UnxTsqint` (68 entries use such a command; the snippets define them).
  Inside the snippets, rename the internal font identifiers that start with
  `flm` the same way (`flmstixcal` becomes `UnxTstixcal`, `flmwasy` becomes
  `UnxTwasy`, and so on).
- The script asserts, for every entry, that the text-mode rendering of the
  new entry (`\ensuremath{` + X + `}` for `MATH`, the string itself
  otherwise) equals the old string after the prefix rename.
- Copy `initial-rust-port/tests/goldens/latexencode/` (the golden file and
  its `README.md`) to `tests/goldens/latexencode/`, and update the README's
  wording for this crate. Apply the same prefix rename to the copied golden
  file (68 lines change) and review that diff. It is the only permitted
  change to the golden: **never regenerate the golden to make a test pass**;
  every changed line must be a change to the table that was meant.
- Carry over by hand into the module docs of `default_table.rs`: the
  provenance notes, the list of corrections over pylatexenc, and both MIT
  license notices. Replace references to FLM.

### Tests

- The tests live under `tests/`, as integration tests over the public API:
  `tests/core.rs` (step 1) covers the rules, the chain, the protection
  strategies, the encoder loop and what it reports; `tests/latexencode.rs`
  (step 4) is the ported suite, with the golden test.
- Port `initial-rust-port/tests/latexencode.rs` (39 tests) and the 9 unit
  tests at the end of `initial-rust-port/src/lib.rs` to the new API. Every
  old test has a disposition here; tests not named are ported as they are,
  with expected strings updated only for the `\UnxT` rename and for the
  `\ensuremath{..}` that moved from the spelling into the mode column (a
  user rule that overrides a math entry now writes `\ensuremath{..}` itself).
  The ported suite lives in `tests/latexencode.rs`; a few table-structure
  tests overlap with `tests/builtin.rs` (written in step 3), which is
  intended: the old names stay visible next to the old suite.
  - `basic_1_non_ascii_only_braces_all`,
    `basic_2c_ascii_specials_untouched_with_non_ascii_only`: use the
    `DEFAULT_TABLE_NON_ASCII` table instead of a `non_ascii_only` flag;
    `braces-all`
    becomes `BracesAroundAll`.
  - `basic_2_*`: `BracesAfter`. `basic_2b_protection_none`: `NoProtection`.
  - `basic_custom_protection_applies_to_every_spelling`: a custom
    `ReplacementProtection` impl.
  - `basic_3_unknown_kept_and_reported`: `EncodeReport` holds the char; the
    position (21) is checked through a small custom `EncodeReporter`.
  - `basic_3c_unknown_unihex`, `ignore_and_custom_policies`: through
    `UnknownCharPolicy::callback(..)`.
  - `rules_00_order_of_rules`, `issue_no21_acronyms_through_a_callable`,
    `rules_02_superscript_two`: `RuleChain`, `DynTable`, `rule_fn`.
  - `rules_callable_must_consume_at_least_one_char` and the unit test
    `a_callable_consuming_past_the_end_or_inside_a_character_is_refused`:
    replaced by tests that `replace_prefix` panics and `try_replace_prefix`
    returns an error for zero, past-the-end, and mid-character lengths.
  - `delete_character_at_the_ascii_boundary`: replaced by tests of the new
    unknown-char definition (DEL and control chars are unknown; a rule can
    match a control char).
  - `per_rule_protection_overrides_the_encoder_s`: becomes a `DoNotProtect`
    test.
  - `no_rules_means_everything_non_ascii_is_unknown`: `RuleChain::new(())`.
  - `the_conformance_golden_is_this_table_s_output`: see the golden test
    below. `the_conformance_golden_shows_the_departures`: ported as is.
  - `the_table_is_sorted_and_its_spellings_are_well_formed`,
    `every_entry_names_a_profile_that_exists`: become the compile-time
    checks; keep only the "`lookup` agrees with `iter()`" part as a test.
  - `every_named_table_entry_is_covered_by_the_fixture`,
    `astral_plane_entries_by_code_point`: ported, using `iter()`.
  - `modes_over_the_whole_table`: ported against the mode column (935
    `MathOnly`, 34 `AnyMode`, 580 `TextOnly`, the three mixed entries
    `TextOnly`); the parts that test `Mode::of` parsing are dropped.
  - `the_chunk_table_is_well_formed`: distinct ids only (no `docs` field, no
    64-chunk limit any more).
  - `a_snippet_chunk_never_appears_without_the_package_chunks_it_calls_into`:
    ported as a runtime test, reading the structured `PackageWithOptions`
    options instead of parsing a `\usepackage` line.
  - `every_profile_names_chunks_that_exist_and_no_set_appears_twice`: keep
    "no two builtin profiles hold the same chunk set".
  - `a_set_of_needs_unions_profiles_and_keeps_the_table_s_order`: adapted to
    the new order (packages first, then snippets, each in first-seen order);
    the expected lists change.
  - Dropped with their features: `the_commands_a_spelling_writes_are_readable`
    (`macro_names`), and the unit tests `mode_is_read_off_the_spelling_s_form`
    (`Mode::of`), `spelling_of_falls_back_to_the_composed_form`,
    `apply_is_what_the_encoder_applies_to_every_spelling`.
  - Unit tests `braces_protection_*` and `braces_all_*`: ported to
    `StandardProtection` / `BracesAroundAll`.
    `composed_borrows_composed_text_and_composes_the_rest`: ported to `nfc()`.
    `errors_display_their_position`, `debug_forms_name_the_kinds`: ported.
- Golden test: keep the golden under its original settings. Define, in the
  test file, a custom `ReplacementProtection` that reproduces
  `braces-almost-all`: `DoNotProtect` as is; otherwise compute the text-mode
  rendering (`mode_wrapper_for` on `StandardProtection::text_mode()`) and
  wrap it in braces if it is non-empty and starts with a backslash. Encode
  with `default_rules()`, `NormalizeNfc`, and `UnknownCharPolicy::Fail`,
  rebuilding
  each input line with `"0x%04X %-50s    |%s|\n"` exactly as the old test
  does. Output must be byte-identical to the golden. This validates the data
  migration, the mode wrapping, NFC, and the encoder loop in one go.
- New tests: `ascii_triggers` equivalence (same output as with `ALL` for
  every rule); math output mode; `SpaceAfterMacroName`; output equality
  between `NoReport` and `EncodeReport`; `Send + Sync` assertions for a
  static-chain encoder and for `DynRuleChain`; a `LocalDynRuleChain` holding
  an `Rc`; a run-time `Profile` from a user rule ending up in the report;
  streaming through `FmtOut`. (The `Send + Sync`, `Rc`, and `FmtOut` tests
  were delivered with step 1 in `tests/core.rs`; the rest in step 4.)
- Benchmarks (criterion, dev-only): ASCII-heavy text, accented Latin,
  Greek/math, Cyrillic, CJK under `Keep`; one run per table layout. Spot
  checks with `cargo asm` / `cargo bloat`: the needs logic is gone under
  `NoReport`, the NFC tables are gone under `NoNormalization`.
- LaTeX compile check (manual at first): one document that exercises every
  builtin profile, the `\UnxT` snippets, and the text-in-math wrapper
  candidates.

### Implementation order

1. Core that compiles, with everything the encoder loop touches:
   `Cargo.toml` fixes, `rule`, `chain`, hint types, `asciiset`, `outbuffer`,
   `report`, `preamble`, `profile`, `StandardProtection`, `BracesAroundAll`,
   `normalizer` (both normalizers), the whole `UnknownCharPolicy` with
   `unknown_unihex`, `LookupTable` / `TableEntry` / `TableRule` / `DynTable`,
   and the encoder. Unit tests against a tiny `DynTable`. This step settles
   the core signatures; update the plan where Rust disagrees.
2. The static layouts, `compile_static_table!` with its compile-time checks,
   and `iter()` on them. (`DynTable::iter()` was written in step 1, with
   the `OnlyAscii` / `ExceptAscii` views that step 7 removed.) Tests on a
   small table in each layout.
3. Data migration (script, builtin chunks and profiles, `BuiltinTable`,
   the three builtin tables — then named `DEFAULTS`, `NON_ASCII` and
   `ASCII_SPECIALS` — module docs with provenance and licenses, the golden
   directory).
4. Port the test suite and the golden test.
5. The free `encode()`, crate-level docs, `README.md`, and the crate's own
   license files `LICENSE-MIT` and `LICENSE-APACHE` (`Cargo.toml` declares
   `MIT OR Apache-2.0`).
6. Benchmarks, table layout decision, `cargo asm` / `cargo bloat` checks.
7. API namespace review: one public path per item and no concept split
   between the root and a module (see "Module layout"); `default_rules()` /
   `DefaultRules` as the way to name the defaults; the builtin tables renamed
   `DEFAULT_TABLE*` and moved to `builtin::`, all three of the one opaque
   type `BuiltinTable`; the `OnlyAscii` / `ExceptAscii` views removed;
   `StaticEntry` and the macro's root name hidden. A `rule_chain!` macro was
   considered and left out: it cannot flatten chains (a macro sees tokens,
   not types, and nesting is semantically the same as a flat chain), and
   string names for the defaults belong to run-time configuration.
8. Later, outside this plan: `unicode-xml` table generator; language
   bindings.

## Open items

- Default text-in-math wrapper (`\mbox`, `\text`, `\textnormal`): decide
  after compiling a test document. `\mbox` needs only the kernel but does not
  shrink in sub/superscripts; `\text` sizes correctly but needs amsmath;
  `\textnormal` is believed to behave like `\mbox` alone and like `\text`
  once amsmath is loaded (to be verified).
- **Decided (step 6): the builtin table keeps `two_level_direct_index`.**
  `benches/encode.rs` compiles the same 1549 entries in all three layouts and
  encodes five corpora with each — text mode, `NoReport`,
  `UnknownCharPolicy::Keep`. Numbers are the smallest of three criterion runs
  pinned to one core on an otherwise idle machine (single runs drift by up to
  25% there, which is why the minimum is taken), as time for the whole corpus
  and throughput in MiB/s; the last column is `DEFAULT_TABLE` itself (named
  `DEFAULTS` at the time), the same
  layout seen through `BuiltinTable`, and it agrees with the third column
  inside the noise:

  | corpus (bytes)      |  binary_search |  two_level_linear | two_level_direct_index |     `DEFAULTS` |
  |---------------------|---------------:|------------------:|-----------------------:|---------------:|
  | ascii_source (1233) |  3.74 µs / 315 |     3.59 µs / 327 |          2.36 µs / 498 |  2.48 µs / 475 |
  | accented (1144)     |  3.13 µs / 349 |     4.41 µs / 248 |          2.88 µs / 379 |  2.71 µs / 403 |
  | greek_math (1269)   | 12.63 µs /  96 |    19.19 µs /  63 |          9.18 µs / 132 |  9.25 µs / 131 |
  | cyrillic (1636)     | 18.21 µs /  86 |    32.07 µs /  49 |         15.00 µs / 104 | 13.10 µs / 119 |
  | cjk (1045)          |  7.61 µs / 131 |     7.77 µs / 128 |          7.11 µs / 140 |  7.13 µs / 140 |

  The direct index wins on every corpus: 1.07 to 1.58 times the binary
  search, 1.09 to 2.14 times the two-level linear layout. The worry above was
  half right — the linear block scan does cost — but it is the *inner* scan
  that decides: `two_level_linear` walks up to 256 low bytes inside a block
  and loses worst exactly where a block is densely used (Cyrillic, 32.07 µs
  against 15.00 µs). With one load inside the block, the same 22-block linear
  first level still beats bisection. The one case where the three layouts
  nearly tie is CJK, which the table does not cover at all: there the block
  scan runs to the end for every character. A direct first-level index would
  buy something there and nowhere else, so it is not implemented; it stays a
  possibility if a table with many more blocks is added.

  Payload packing (one string blob with offsets instead of one `&str` per
  entry) was **not** measured and stays open.

  Two side measurements from the same bench file. `EncodeReport` costs
  nothing measurable against `NoReport` — 2.40 µs against 2.66 µs on the
  accented corpus, 13.39 µs against 13.67 µs on the Cyrillic one, where every
  letter names a `fontenc` profile — because the encoder skips immediate
  repeats of a profile. `NormalizeNfc` over text that already is NFC costs
  half again the encoding itself: 2.46 µs against 1.62 µs with
  `NoNormalization` on the accented corpus, for the quick check alone.

  The two size claims of this plan were checked on `examples/size_check.rs`
  and `examples/size_check_nfc.rs` (release, LTO). Under `NoReport` with a
  static chain, `cargo asm` finds the whole encoder loop inlined into `main`
  with not one mention of the needs or report code, and `nm` finds no
  `PreambleNeeds`, `EncodeReport` or `report_needs` symbol; the profile
  *data* stays linked, since the table holds a reference to it. With
  `NoNormalization` the `unicode-normalization` tables are gone: 488,888
  bytes against 618,752, `.rodata` 56,752 against 169,816, and no
  `unicode_normalization` symbol left in the binary.
- `unicode-xml` table: pylatexenc's generated dict
  (`latexencode/_uni2latexmap_xml.py`, 2233 entries) has no mode column, no
  needs, and no license header, and it mixes bare math macros with text
  macros. It would have to be regenerated from the W3C `unicode.xml` source
  (check and carry its license), probably behind a cargo feature to spare
  everyone the compile time of its `const fn` build.
- Refine modes in the builtin table: some `TEXT` entries (`\#`, `\%`, `\&`,
  ..) are valid in math mode too and could become `ANY`.
