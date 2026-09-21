//! Unicode-to-LaTeX encoding: the table and the encoder that spell a
//! non-ASCII character as LaTeX (`é` as `\'e`, `α` as `\ensuremath{\alpha}`,
//! `—` as `\textemdash`).
//!
//! LaTeX source is plain text, but a LaTeX document cannot always take a
//! character as it stands: an old installation reads no UTF-8, a font has no
//! glyph for the character, or the character is one of LaTeX's own special
//! characters (`%`, `&`, `#`). *Encoding* a string means replacing each such
//! character by its LaTeX spelling — the commands and characters that print
//! it — while leaving ordinary characters alone. This crate began as a port
//! of the `latexencode` module of pylatexenc, the Python library for working
//! with LaTeX code: the same algorithm and the same options, over a conversion
//! table copied from it, which is maintained here since — [`tables`] states its
//! provenance and lists every spelling that departs from pylatexenc's.
//!
//! Beyond pylatexenc, every spelling says what a document must load in order to
//! print it (`\mathds{1}` needs the package `dsfont`), so that a caller can
//! assemble the preamble its output needs: see *What a spelling needs*, below.
//!
//! This crate stands on its own: it depends on no other crate of the FLM
//! workspace, and it knows nothing of documents or of FLM markup. The library
//! `flm-core` depends on it and re-exports it as `flm_core::latexencode`; its
//! LaTeX backends use it when they write document text.
//!
//! # The one-line use
//!
//! [`encode`] encodes a string under the default options: the built-in table,
//! every character it knows replaced (the special characters included), each
//! spelling protected with braces where it needs to be, and an unknown
//! character kept as it is.
//!
//! ```
//! use flm_latexencode::encode;
//!
//! assert_eq!(encode("Café — naïve"), r#"Caf\'e {\textemdash} na\"ive"#);
//! assert_eq!(encode("100% & more"), r"100\% \& more");
//! ```
//!
//! # The encoder and its options
//!
//! An [`Encoder`] holds an [`Options`] value and encodes any number of strings
//! with it; it is cheap to build, and it can be shared between threads. The
//! options are, with pylatexenc's names where they differ:
//!
//! - [`non_ascii_only`](Options::non_ascii_only) — leave every ASCII character
//!   alone, so that `%` and `&` stay as typed and only characters outside ASCII
//!   are spelled.
//! - [`protection`](Options::protection) (`replacement_latex_protection` in
//!   pylatexenc) — what to put around a spelling so that it survives being
//!   pasted into arbitrary text: a spelling such as `\textemdash` followed
//!   directly by a letter would read as a longer command name. The default,
//!   [`Protection::Braces`], wraps exactly the spellings that end in a named
//!   command; the other modes are listed at [`Protection`].
//! - [`unknown_char_policy`](Options::unknown_char_policy) — what to write for
//!   a character outside ASCII that no rule knows: keep it, replace it by a
//!   marker, drop it, spell out its code point, or fail
//!   ([`UnknownCharPolicy`]).
//! - [`rules`](Options::rules) (`conversion_rules` in pylatexenc) — the list of
//!   [`Rule`]s tried in order at every position; the default list holds the
//!   built-in table alone. A rule placed before [`Rule::defaults`] overrides
//!   it, a rule placed after it fills in what the table lacks.
//!
//! ```
//! use flm_latexencode::{Encoder, Options, Protection, Rule, UnknownCharPolicy};
//!
//! // A dictionary rule ahead of the defaults overrides one spelling; a callable
//! // rule after them handles a sequence of characters the table cannot.
//! let ellipsis = |s: &str, pos: usize| {
//!     s[pos..].starts_with("...").then(|| (3, String::from(r"\ldots")))
//! };
//! let encoder = Encoder::new(Options {
//!     rules: vec![
//!         Rule::dict([('%', String::from(r"\textpercent"))]),
//!         Rule::defaults(),
//!         Rule::callable(ellipsis),
//!     ],
//!     protection: Protection::BracesAfterMacro,
//!     unknown_char_policy: UnknownCharPolicy::Unihex,
//!     ..Options::default()
//! });
//! assert_eq!(
//!     encoder.encode("100%... ธ").unwrap(),
//!     r"100\textpercent{}\ldots{} \ensuremath{\langle}\texttt{U+0E18}\ensuremath{\rangle}"
//! );
//! ```
//!
//! [`Encoder::encode_into`] appends to a string the caller owns and reports
//! back what the encoding needs and which characters no rule knew
//! ([`EncodeReport`]); [`Encoder::encode`] is the same with a fresh string and
//! without the report.
//!
//! # What a spelling needs
//!
//! A spelling is not always LaTeX that every document can print: `\mathds{1}`
//! needs the package `dsfont` loaded, and `\cyrya` the `T2A` font encoding.
//! Each entry of the table therefore carries a [`Profile`]: the set of
//! *preamble chunks* ([`Chunk`]) that the spelling needs. A chunk is a package
//! with the options it is loaded with, or a block of declarations no package
//! makes — a math alphabet read from a font family whose own package would
//! rename a command the document uses ([`ChunkPreamble`]). A [`PreambleNeeds`]
//! is the union of such sets, which is what a whole document needs; the
//! encoder returns one for every string it encodes, and
//! [`PreambleNeeds::preamble`] writes it out.
//!
//! ```
//! use flm_latexencode::Encoder;
//!
//! let mut out = String::new();
//! let report = Encoder::default().encode_into(&mut out, "𝟙 and ⅓").unwrap();
//! assert_eq!(out, r"\ensuremath{\mathds{1}} and \nicefrac{1}{3}");
//! assert_eq!(report.needs.preamble(), "\\usepackage{dsfont}\n\\usepackage{nicefrac}\n");
//! ```
//!
//! Only the built-in table's spellings are accounted for: a spelling a
//! [dictionary](Rule::dict) or [callable](Rule::callable) rule of the caller's
//! produces adds nothing to the set, since what such a spelling needs is the
//! caller's to know.
//!
//! # One character at a time
//!
//! [`spelling_of`] answers, for one character, the built-in table's spelling,
//! what kind of LaTeX it is ([`Spelling`], [`Mode`]) — text-mode commands such
//! as `\'e`, mathematics wrapped in `\ensuremath{…}`, or plain characters — and
//! the [`Profile`] of what it needs.
//! A caller that writes characters through as UTF-8 and declares their
//! spellings elsewhere — in a preamble, with `\newunicodechar` — uses this
//! instead of the encoder, and builds its own [`PreambleNeeds`] from the
//! profiles. Such a caller composes its text with [`composed`]
//! first, as the encoder does, so that it looks up the characters the table
//! knows; and it writes each spelling through [`Protection::apply`], as the
//! encoder does, so that the spelling cannot merge with the letters after it.
//!
//! # How a string is encoded
//!
//! The algorithm is pylatexenc's, step for step:
//!
//! 1. The input is composed with [`composed`] — normalized to Unicode's
//!    canonical composed form (NFC) — so that a letter followed by a combining
//!    accent becomes the single accented letter the table knows.
//! 2. At every position, from the start: with `non_ascii_only`, an ASCII
//!    character is copied and the rules are not consulted. Otherwise the rules
//!    are tried in order and the first that applies wins — no rule is tried
//!    after a match, and no longest match is sought. Its spelling is written
//!    through the protection mode (the rule's own, if it has one, else the
//!    encoder's) and the position advances by what the rule consumed.
//! 3. When no rule applies: a printable ASCII character, or a newline, carriage
//!    return or tab, is copied; any other character goes to the
//!    unknown-character policy, whose output is written *without* protection,
//!    and is recorded in the returned list.
//!
//! Positions — in a callable rule's arguments, in an [`UnknownChar`] and in an
//! [`EncodeError`] — are byte offsets into the composed input, the text
//! [`composed`] returns, which is the input itself whenever it was already in
//! NFC form.
//!
//! # Provenance and license
//!
//! The algorithm is pylatexenc's, and the table began as a copy of
//! pylatexenc's (<https://github.com/phfaist/pylatexenc>, commit `e4ddf2ba`,
//! the package `pylatexenc/latexencode/`), distributed under the MIT License,
//! Copyright (c) 2015-2023 Philippe Faist; the table's character map was in
//! turn adapted from latexcodec by Matthias C. M. Troffaes, under the same
//! license. Both license notices are reproduced in full in [`tables`], the file
//! that holds the data. What differs from pylatexenc:
//!
//! - The table is maintained here. It carries pylatexenc's spellings except
//!   where one of them was found to be LaTeX that does not compile; [`tables`]
//!   lists every such departure with its reason, so a few characters encode
//!   differently here. Each entry also carries what its spelling needs in the
//!   preamble ([`Profile`]), which pylatexenc records nowhere.
//!
//! - The built-in table is pylatexenc's `defaults` table alone. Its second
//!   table, `unicode-xml`, which spells mathematics without `\ensuremath` and
//!   carries a license of its own, is not ported.
//! - A rule is a dictionary or a callable ([`Rule`]). pylatexenc's
//!   regular-expression rule kind has no counterpart: the default rule set has
//!   none, and a [callable rule](Rule::callable) expresses any of them.
//! - A callable rule and a custom unknown-character policy receive no handle
//!   to the encoder (pylatexenc passes itself to a callable that declares a
//!   `u2lobj` argument).
//! - The encoder logs nothing: the characters no rule knew always come back in
//!   the [`EncodeReport`] that [`Encoder::encode_into`] returns, which is
//!   pylatexenc's unknown-character warning in the form of a value. The option
//!   that turns that warning off, `unknown_char_warning`, has no counterpart; a
//!   caller ignores the list instead.
//! - The output goes into a string the caller owns. pylatexenc's accumulator
//!   hook (`latex_string_class`), which let a caller collect the output chunk
//!   by chunk, has no counterpart.
//! - Positions are byte offsets into the composed text (pylatexenc counts
//!   code points).
//! - A callable rule's consumption must be at least one byte and must end on
//!   a character boundary within the input; anything else is
//!   [`EncodeError::InvalidConsumption`]. pylatexenc refuses a zero count the
//!   same way, but accepts a count past the end of the input and simply ends
//!   the string there.
//! - The test of [`Protection::Braces`] for a spelling that ends in letters
//!   uses Rust's notion of a letter, `char::is_alphabetic` (Python's
//!   `str.isalpha` in pylatexenc); the two agree on ASCII, which is all the
//!   built-in table contains.
//! - Not ported: `PartialLatexToLatexEncoder` (encoding text that already
//!   holds some LaTeX), the module-level encoder cache, and the pylatexenc-1
//!   compatibility function `utf8tolatex`.
//! - [`spelling_of`], [`Spelling`], [`Mode`], [`composed`], [`macro_names`],
//!   the preamble needs ([`Profile`], [`Chunk`],
//!   [`PreambleNeeds`]) and the public [`Protection::apply`] exist here only.
//!
//! Every entry of [`tables`] compiles and sets the glyph its character stands
//! for, given the preamble the entry itself asks for; the characters that
//! table has no spelling for — the ones no installed font has a command for,
//! which it lists — are reported to the caller rather than written. A caller
//! who wants another spelling for a character overrides the entry by placing a
//! [dictionary rule](Rule::dict) ahead of the defaults.

#[no_std]

mod needs;
pub mod tables;

pub use needs::{Chunk, ChunkPreamble, PreambleNeeds, Profile};

use core::borrow::Cow;
use core::collections::BTreeMap;
use core::fmt;
use core::sync::Arc;

use unicode_normalization::{IsNormalized, UnicodeNormalization, is_nfc_quick};

/// Encode a string under the default options.
///
/// The default options are [`Options::default`]: the built-in table as the
/// only rule, every character it knows replaced (ASCII special characters
/// included), [`Protection::Braces`], and an unknown character kept as it is.
/// Nothing can fail under them, so the result is the string itself. Build an
/// [`Encoder`] for any other configuration.
///
/// ```
/// use flm_latexencode::encode;
///
/// assert_eq!(encode("naïve α"), r#"na\"ive \ensuremath{\alpha}"#);
/// ```
pub fn encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    // Under the default options no rule is a callable and the policy is
    // `Keep`, so neither error can arise.
    let _ = Encoder::default().encode_into(&mut out, text);
    out
}

/// The text in Unicode's canonical composed form (NFC): a letter followed by
/// combining accents is replaced by the single accented letter wherever
/// Unicode defines one, so that `e` followed by a combining acute accent
/// becomes `é`, and a character with a composed equivalent is replaced by it,
/// so that the Angstrom sign becomes the letter `Å`.
///
/// This is the one definition of "composed" in this crate. The encoder
/// applies it to every string before any rule runs; [`spelling_of`] applies
/// it to a single character; and every position a [callable rule](Rule::callable),
/// an [`UnknownChar`] or an [`EncodeError`] carries is a byte offset into the
/// text this function returns. A caller that writes characters through
/// itself and declares their spellings elsewhere composes its text with this
/// function first, so that it looks up the same characters the encoder would
/// spell.
///
/// The input comes back borrowed, with no copy, when a quick check shows it
/// is already in composed form — the case for nearly all text — and as a new
/// string otherwise.
///
/// ```
/// use flm_latexencode::composed;
///
/// assert_eq!(composed("Cafe\u{301}"), "Caf\u{e9}");
/// assert_eq!(composed("Caf\u{e9}"), "Caf\u{e9}");
/// ```
pub fn composed(text: &str) -> Cow<'_, str> {
    match is_nfc_quick(text.chars()) {
        IsNormalized::Yes => Cow::Borrowed(text),
        IsNormalized::No | IsNormalized::Maybe => Cow::Owned(text.nfc().collect()),
    }
}

/// The built-in table's spelling of one character, with the kind of LaTeX it
/// is.
///
/// The spelling is looked up for the character as given and, when that fails
/// and the character's composed form (what [`composed`] makes of it) is a
/// single different character, for that form — the same normalization the
/// encoder applies to a whole string, so that `spelling_of` and
/// [`Encoder::encode`] agree on which characters are known. The spelling is
/// the table's text with no protection applied. `None` means the table has no
/// entry: an ASCII letter, a character from a script the table does not
/// cover, an unassigned code point.
///
/// This is what a caller uses when it writes characters through as they are
/// and declares their LaTeX spellings once elsewhere; [`Protection::apply`]
/// protects a spelling the way the encoder would.
///
/// ```
/// use flm_latexencode::{Mode, spelling_of};
///
/// let e = spelling_of('é').unwrap();
/// assert_eq!((e.latex, e.mode), (r"\'e", Mode::Text));
/// let alpha = spelling_of('α').unwrap();
/// assert_eq!((alpha.latex, alpha.mode), (r"\ensuremath{\alpha}", Mode::Math));
/// let ligature = spelling_of('ﬁ').unwrap();
/// assert_eq!((ligature.latex, ligature.mode), ("fi", Mode::Any));
/// assert_eq!(spelling_of('a'), None);
/// ```
pub fn spelling_of(ch: char) -> Option<Spelling> {
    if let Some(entry) = tables::lookup(ch) {
        return Some(Spelling::of(entry));
    }
    let mut buf = [0u8; 4];
    let form = composed(ch.encode_utf8(&mut buf));
    let mut chars = form.chars();
    let first = chars.next()?;
    if first == ch || chars.next().is_some() {
        return None;
    }
    tables::lookup(first).map(Spelling::of)
}

/// The LaTeX spelling of one character, as [`spelling_of`] answers it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Spelling {
    /// The LaTeX text that prints the character, exactly as the table holds
    /// it, with no protection around it.
    pub latex: &'static str,
    /// What kind of LaTeX the spelling is, read off its form.
    pub mode: Mode,
    /// What a document must load in order to print this spelling, as the
    /// [`Profile`] the table's entry carries. Add it to a [`PreambleNeeds`]
    /// with [`PreambleNeeds::include`]; [`Profile::BUILTINS`] means the LaTeX
    /// kernel prints it by itself.
    pub needs: Profile,
}

impl Spelling {
    fn of((latex, needs): (&'static str, Profile)) -> Spelling {
        Spelling { latex, mode: Mode::of(latex), needs }
    }
}

/// What kind of LaTeX a spelling is: text-mode commands, mathematics, or
/// plain characters.
///
/// The table records no mode; the mode is read off the spelling's form with
/// [`Mode::of`], and it tells a caller where the spelling may be written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    /// A text-mode spelling: it contains a command and is not wrapped in
    /// `\ensuremath`, so it is meant for ordinary text and may not work inside
    /// mathematics. `\'e`, `\textemdash`, `\c{c}`.
    Text,
    /// A mathematical spelling wrapped, as a whole, in `\ensuremath{…}`, which
    /// makes it work in text as well as inside mathematics.
    /// `\ensuremath{\alpha}`, `\ensuremath{\mathbb{1}}`.
    Math,
    /// Plain characters containing no command, usable anywhere: `ff`, `''`,
    /// `A`, or the empty spelling of an invisible character.
    Any,
}

impl Mode {
    /// The mode of a spelling, read off its form: [`Math`](Mode::Math) when
    /// the whole spelling is one `\ensuremath{…}` group, else
    /// [`Text`](Mode::Text) when it contains a backslash, else
    /// [`Any`](Mode::Any).
    ///
    /// A spelling that starts with `\ensuremath{` but continues past the
    /// group's closing brace (`\ensuremath{^\circ}F`) is text, as is one that
    /// wraps an `\ensuremath` group in a text command (`{\small\ensuremath{…}}`).
    ///
    /// ```
    /// use flm_latexencode::Mode;
    ///
    /// assert_eq!(Mode::of(r"\ensuremath{\alpha}"), Mode::Math);
    /// assert_eq!(Mode::of(r"\ensuremath{^\circ}F"), Mode::Text);
    /// assert_eq!(Mode::of(r"\'e"), Mode::Text);
    /// assert_eq!(Mode::of("ff"), Mode::Any);
    /// ```
    pub fn of(latex: &str) -> Mode {
        const ENSUREMATH: &str = "\\ensuremath{";
        if latex.starts_with(ENSUREMATH) {
            // The group opened by `\ensuremath{` must close at the very end;
            // `\{` and `\}` inside it are characters, not braces.
            let start = ENSUREMATH.len() - 1;
            let mut depth = 0usize;
            let mut escaped = false;
            let mut closes_at_end = false;
            for (index, ch) in latex[start..].char_indices() {
                if escaped {
                    escaped = false;
                    continue;
                }
                match ch {
                    '\\' => escaped = true,
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            closes_at_end = start + index + 1 == latex.len();
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if closes_at_end {
                return Mode::Math;
            }
        }
        if latex.contains('\\') { Mode::Text } else { Mode::Any }
    }
}

/// The command names a piece of LaTeX writes: every backslash followed by
/// ASCII letters, without the backslash, in order of occurrence, a name that
/// occurs twice answered twice.
///
/// The scan is the plain one: a name ends at the first character that is not an
/// ASCII letter, so that `\flmLabel@x` gives `flmLabel`, and a backslash
/// followed by anything but a letter (`\\`, `\%`) gives nothing. It is how
/// a caller writing LaTeX of its own beside the table's spellings reads the
/// commands out of its own text.
///
/// ```
/// use flm_latexencode::macro_names;
///
/// let names: Vec<&str> = macro_names(r"\ensuremath{\mathbb{1}} \'e \\ x\%").collect();
/// assert_eq!(names, ["ensuremath", "mathbb"]);
/// ```
pub fn macro_names(latex: &str) -> impl Iterator<Item = &str> + '_ {
    let mut rest = latex;
    std::iter::from_fn(move || {
        while let Some(at) = rest.find('\\') {
            let after = &rest[at + 1..];
            let end = after.find(|c: char| !c.is_ascii_alphabetic()).unwrap_or(after.len());
            let name = &after[..end];
            // Whatever the name was, the backslash and the letters are consumed,
            // so the scan always moves forward.
            rest = &after[end..];
            if !name.is_empty() {
                return Some(name);
            }
        }
        None
    })
}

/// How a spelling is protected from the text around it.
///
/// A spelling that ends in a named command — `\textemdash`, `\l`, `\^\i` —
/// cannot be followed directly by a letter: `\l` before `o` would read as the
/// command `\lo`. *Protection* is what the encoder adds around a spelling to
/// prevent that. The modes are pylatexenc's, under its names spelled in Rust;
/// a per-rule protection ([`Rule::with_protection`]) overrides the encoder's
/// for the spellings that rule produces. The output of an
/// [`UnknownCharPolicy`] is never protected.
#[derive(Clone, Default)]
pub enum Protection {
    /// Wrap the spelling in braces when its last backslash is followed by
    /// letters only, that is, when it ends in a named command: `\textemdash`
    /// becomes `{\textemdash}`, while `\'e`, `\c{c}` and `\ensuremath{\alpha}`
    /// stay as they are. pylatexenc's `'braces'`; the default.
    #[default]
    Braces,
    /// Wrap every spelling in braces, whatever its form — an empty spelling
    /// becomes `{}`. pylatexenc's `'braces-all'`.
    BracesAll,
    /// Wrap every spelling that starts with a backslash in braces, so `\'e`
    /// becomes `{\'e}` but `''` stays. pylatexenc's `'braces-almost-all'`,
    /// which reproduces the behavior of its version-1 encoder.
    BracesAlmostAll,
    /// Append `{}` to a spelling that ends in a named command, under the same
    /// test as [`Braces`](Protection::Braces): `\textemdash` becomes
    /// `\textemdash{}`. pylatexenc's `'braces-after-macro'`.
    BracesAfterMacro,
    /// Write every spelling as it is. Unsafe in general, since `\l` before a
    /// letter is then a different command. pylatexenc's `'none'`.
    None,
    /// A function of the caller's, given the spelling and returning what is
    /// written in its place; it is applied to every spelling, the empty one
    /// included.
    Custom(Arc<dyn Fn(&str) -> String + Send + Sync>),
}

impl Protection {
    /// Whether `latex` ends in a named command: its last backslash is followed
    /// by one or more letters and nothing else.
    fn ends_in_named_command(latex: &str) -> bool {
        match latex.rfind('\\') {
            Some(k) => {
                let name = &latex[k + 1..];
                !name.is_empty() && name.chars().all(char::is_alphabetic)
            }
            None => false,
        }
    }

    /// The spelling `latex` with this protection applied, as a new string.
    ///
    /// This is exactly what the encoder applies to every spelling a rule
    /// produces (under the rule's own protection if it has one, else the
    /// encoder's). A caller that writes spellings itself — from
    /// [`spelling_of`] or from a table of its own — protects them with it, so
    /// that they can no more merge with the letters after them than the
    /// encoder's output can. The encoder does not pass the output of an
    /// [`UnknownCharPolicy`] through it.
    ///
    /// # Panics
    ///
    /// Raises no panic of its own; a panic inside the function of a
    /// [`Protection::Custom`] is not caught.
    ///
    /// ```
    /// use flm_latexencode::Protection;
    ///
    /// assert_eq!(Protection::Braces.apply(r"\textemdash"), r"{\textemdash}");
    /// assert_eq!(Protection::Braces.apply(r"\'e"), r"\'e");
    /// assert_eq!(Protection::BracesAfterMacro.apply(r"\l"), r"\l{}");
    /// ```
    pub fn apply(&self, latex: &str) -> String {
        let mut out = String::with_capacity(latex.len() + 2);
        self.apply_into(&mut out, latex);
        out
    }

    /// Append the spelling `latex` to `out` with this protection applied: the
    /// same as [`apply`](Protection::apply), into a string the caller owns.
    ///
    /// # Panics
    ///
    /// As for [`apply`](Protection::apply).
    pub fn apply_into(&self, out: &mut String, latex: &str) {
        let braced = |out: &mut String| {
            out.push('{');
            out.push_str(latex);
            out.push('}');
        };
        match self {
            Protection::Braces if Protection::ends_in_named_command(latex) => braced(out),
            Protection::BracesAll => braced(out),
            Protection::BracesAlmostAll if latex.starts_with('\\') => braced(out),
            Protection::BracesAfterMacro => {
                out.push_str(latex);
                if Protection::ends_in_named_command(latex) {
                    out.push_str("{}");
                }
            }
            Protection::Custom(f) => out.push_str(&f(latex)),
            Protection::Braces | Protection::BracesAlmostAll | Protection::None => {
                out.push_str(latex);
            }
        }
    }
}

impl fmt::Debug for Protection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Protection::Braces => "Braces",
            Protection::BracesAll => "BracesAll",
            Protection::BracesAlmostAll => "BracesAlmostAll",
            Protection::BracesAfterMacro => "BracesAfterMacro",
            Protection::None => "None",
            Protection::Custom(_) => "Custom(..)",
        })
    }
}

/// What the encoder writes for a character outside ASCII that no rule knows.
///
/// Whatever the policy, the character is also reported in the list
/// [`Encoder::encode_into`] returns, and the policy's output is written
/// without [`Protection`]. The values are pylatexenc's.
#[derive(Clone, Default)]
pub enum UnknownCharPolicy {
    /// Write the character itself, as UTF-8. pylatexenc's `'keep'`; the
    /// default.
    #[default]
    Keep,
    /// Write a bold question mark, `{\bfseries ?}`. pylatexenc's `'replace'`.
    Replace,
    /// Write nothing. pylatexenc's `'ignore'`.
    Ignore,
    /// Stop with [`EncodeError::UnknownCharacter`]. pylatexenc's `'fail'`.
    Fail,
    /// Write the character's code point in typewriter type between angle
    /// brackets, `\ensuremath{\langle}\texttt{U+0E18}\ensuremath{\rangle}`.
    /// pylatexenc's `'unihex'`.
    Unihex,
    /// A function of the caller's, given the character and returning what to
    /// write.
    Custom(Arc<dyn Fn(char) -> String + Send + Sync>),
}

impl UnknownCharPolicy {
    /// Write what the policy says for `ch`, met at byte `position`.
    fn apply_into(&self, out: &mut String, ch: char, position: usize) -> Result<(), EncodeError> {
        match self {
            UnknownCharPolicy::Keep => out.push(ch),
            UnknownCharPolicy::Replace => out.push_str(r"{\bfseries ?}"),
            UnknownCharPolicy::Ignore => {}
            UnknownCharPolicy::Fail => {
                return Err(EncodeError::UnknownCharacter { ch, position });
            }
            UnknownCharPolicy::Unihex => {
                use fmt::Write as _;
                // Writing to a `String` cannot fail.
                let _ = write!(
                    out,
                    "\\ensuremath{{\\langle}}\\texttt{{U+{:04X}}}\\ensuremath{{\\rangle}}",
                    ch as u32
                );
            }
            UnknownCharPolicy::Custom(f) => out.push_str(&f(ch)),
        }
        Ok(())
    }
}

impl fmt::Debug for UnknownCharPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            UnknownCharPolicy::Keep => "Keep",
            UnknownCharPolicy::Replace => "Replace",
            UnknownCharPolicy::Ignore => "Ignore",
            UnknownCharPolicy::Fail => "Fail",
            UnknownCharPolicy::Unihex => "Unihex",
            UnknownCharPolicy::Custom(_) => "Custom(..)",
        })
    }
}

/// The function type of a callable rule: see [`Rule::callable`].
type RuleFn = dyn Fn(&str, usize) -> Option<(usize, String)> + Send + Sync;

/// One conversion rule: a way of spelling what stands at a position of the
/// input.
///
/// An encoder tries its rules in order at every position and the first that
/// applies wins. Two kinds exist: a *dictionary* rule spells one character
/// ([`Rule::dict`], and [`Rule::defaults`] for the built-in table), and a
/// *callable* rule is a function that may look at the whole input and consume
/// several characters ([`Rule::callable`]). A rule may carry a
/// [`Protection`] of its own ([`Rule::with_protection`]), which replaces the
/// encoder's for the spellings it produces.
#[derive(Clone)]
pub struct Rule {
    kind: RuleKind,
    protection: Option<Protection>,
}

#[derive(Clone)]
enum RuleKind {
    /// A sorted static table, searched by bisection: the built-in table.
    // Static data in a source file rather than a map built at run time.
    // See decision rationale at [§flm-dd-dr:latexencode-port].
    Table(&'static [(char, &'static str, Profile)]),
    /// A caller's dictionary.
    Dict(BTreeMap<char, String>),
    Callable(Arc<RuleFn>),
}

impl Rule {
    /// The built-in table, [`tables::DEFAULTS`], as a dictionary rule —
    /// pylatexenc's `'defaults'` rule set, which is its one built-in rule.
    pub fn defaults() -> Rule {
        Rule { kind: RuleKind::Table(tables::DEFAULTS), protection: None }
    }

    /// A dictionary rule: each entry maps one character to its spelling. The
    /// rule applies wherever the character at the position is a key, and
    /// consumes that one character. An empty spelling deletes the character.
    ///
    /// ```
    /// use flm_latexencode::{Encoder, Options, Rule};
    ///
    /// let encoder = Encoder::new(Options {
    ///     rules: vec![Rule::dict([('𝟙', String::from(r"\mathbb{1}"))]), Rule::defaults()],
    ///     ..Options::default()
    /// });
    /// assert_eq!(encoder.encode("𝟙 and 𝟚").unwrap(), r"\mathbb{1} and \ensuremath{\mathbbm{2}}");
    /// ```
    pub fn dict(entries: impl IntoIterator<Item = (char, String)>) -> Rule {
        Rule { kind: RuleKind::Dict(entries.into_iter().collect()), protection: None }
    }

    /// A callable rule. The function receives the whole normalized input and
    /// the byte position being encoded, and answers `None` when the rule does
    /// not apply there, or `Some((consumed, spelling))` with the number of
    /// bytes it consumed and the LaTeX to write for them.
    ///
    /// `consumed` must be at least one and must end on a character boundary
    /// of the input; anything else is reported as
    /// [`EncodeError::InvalidConsumption`] rather than applied, since a rule
    /// that consumes nothing would be applied at the same position forever.
    /// Consuming several characters at once is the point of this kind:
    /// spelling `...` as `\ldots`, or bracing a run of capitals.
    ///
    /// ```
    /// use flm_latexencode::{Encoder, Options, Rule};
    ///
    /// // Brace runs of two or more capital letters, keeping existing braces.
    /// let acronyms = |s: &str, pos: usize| {
    ///     let rest = &s[pos..];
    ///     if rest.starts_with(['{', '}']) {
    ///         return Some((1, rest[..1].to_string()));
    ///     }
    ///     let run: String = rest.chars().take_while(|c| c.is_ascii_uppercase()).collect();
    ///     (run.len() >= 2).then(|| (run.len(), format!("{{{run}}}")))
    /// };
    /// let encoder = Encoder::new(Options {
    ///     rules: vec![Rule::callable(acronyms), Rule::defaults()],
    ///     ..Options::default()
    /// });
    /// assert_eq!(
    ///     encoder.encode("Title with {Some} ABC acronyms").unwrap(),
    ///     "Title with {Some} {ABC} acronyms"
    /// );
    /// ```
    pub fn callable(
        f: impl Fn(&str, usize) -> Option<(usize, String)> + Send + Sync + 'static,
    ) -> Rule {
        Rule { kind: RuleKind::Callable(Arc::new(f)), protection: None }
    }

    /// The same rule with a [`Protection`] of its own, applied to the
    /// spellings this rule produces in place of the encoder's. A rule that
    /// guarantees its output is safe to paste anywhere sets
    /// [`Protection::None`] here to avoid braces it does not need.
    pub fn with_protection(mut self, protection: Protection) -> Rule {
        self.protection = Some(protection);
        self
    }

    /// The protection this rule carries, if any.
    pub fn protection(&self) -> Option<&Protection> {
        self.protection.as_ref()
    }

    /// Try the rule at byte `pos` of `s`, where `ch` is the character there:
    /// the bytes consumed and the spelling, or `None` when it does not apply.
    fn apply<'a>(
        &'a self,
        s: &str,
        pos: usize,
        ch: char,
    ) -> Result<Option<(usize, Cow<'a, str>, Profile)>, EncodeError> {
        match &self.kind {
            RuleKind::Table(table) => Ok(table
                .binary_search_by_key(&ch, |&(entry, _, _)| entry)
                .ok()
                .map(|index| (ch.len_utf8(), Cow::Borrowed(table[index].1), table[index].2))),
            RuleKind::Dict(map) => Ok(map
                .get(&ch)
                .map(|latex| (ch.len_utf8(), Cow::Borrowed(latex.as_str()), Profile::BUILTINS))),
            RuleKind::Callable(f) => match f(s, pos) {
                None => Ok(None),
                Some((consumed, latex)) => {
                    let end = pos.checked_add(consumed);
                    let valid = consumed > 0
                        && end.is_some_and(|end| end <= s.len() && s.is_char_boundary(end));
                    if !valid {
                        return Err(EncodeError::InvalidConsumption { position: pos, consumed });
                    }
                    Ok(Some((consumed, Cow::Owned(latex), Profile::BUILTINS)))
                }
            },
        }
    }
}

impl fmt::Debug for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("Rule");
        match &self.kind {
            RuleKind::Table(table) => d.field("table", &table.len()),
            RuleKind::Dict(map) => d.field("dict", &map.len()),
            RuleKind::Callable(_) => d.field("callable", &".."),
        };
        d.field("protection", &self.protection).finish()
    }
}

/// The options of an [`Encoder`]: pylatexenc's, under the names the crate
/// documentation lists.
///
/// `Options::default()` is pylatexenc's default configuration; a caller
/// usually names the fields it changes and takes the rest from it:
///
/// ```
/// use flm_latexencode::{Options, Protection};
///
/// let options = Options { non_ascii_only: true, protection: Protection::BracesAll, ..Options::default() };
/// assert_eq!(options.rules.len(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct Options {
    /// Leave every ASCII character as it is and consult no rule for it, so
    /// that only characters outside ASCII are spelled. Off by default, in
    /// which case the table's spellings of `"`, `#`, `$`, `%`, `&`, `<`, `>`,
    /// `\`, `^`, `_`, `{`, `}` and `~` apply.
    ///
    /// As in pylatexenc, "ASCII" here is every code point below 127: the
    /// delete character (U+007F) is not exempted, although the fallback for a
    /// character no rule knows copies it as ASCII.
    pub non_ascii_only: bool,
    /// The [`Protection`] added around every spelling a rule produces, unless
    /// the rule carries its own. [`Protection::Braces`] by default.
    /// pylatexenc's `replacement_latex_protection`.
    pub protection: Protection,
    /// What to write for a character outside ASCII that no rule knows.
    /// [`UnknownCharPolicy::Keep`] by default.
    pub unknown_char_policy: UnknownCharPolicy,
    /// The rules, tried in order at every position, the first match winning.
    /// By default the built-in table alone, `vec![Rule::defaults()]`.
    /// pylatexenc's `conversion_rules`.
    pub rules: Vec<Rule>,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            non_ascii_only: false,
            protection: Protection::default(),
            unknown_char_policy: UnknownCharPolicy::default(),
            rules: vec![Rule::defaults()],
        }
    }
}

/// An encoder: a fixed [`Options`] value and the two encoding entry points
/// over it.
///
/// Building one is cheap — the built-in table is static data — and an encoder
/// is `Send + Sync`, so one can serve every thread of a program. The crate
/// documentation describes the algorithm.
#[derive(Debug, Clone, Default)]
pub struct Encoder {
    options: Options,
}

impl Encoder {
    /// An encoder over the given options.
    pub fn new(options: Options) -> Encoder {
        Encoder { options }
    }

    /// The options this encoder was built with.
    pub fn options(&self) -> &Options {
        &self.options
    }

    /// Encode `text` into a new string.
    ///
    /// What the output needs in the preamble, and the characters no rule knew,
    /// are not reported: the unknown characters are handled by the
    /// unknown-character policy, and a caller that must know either one uses
    /// [`encode_into`](Encoder::encode_into), which returns both in an
    /// [`EncodeReport`]. The errors are those of `encode_into`.
    ///
    /// # Panics
    ///
    /// Raises no panic of its own; a panic inside a function the options carry
    /// — a [callable rule](Rule::callable), a [`Protection::Custom`] or an
    /// [`UnknownCharPolicy::Custom`] — is not caught.
    ///
    /// ```
    /// use flm_latexencode::Encoder;
    ///
    /// let encoder = Encoder::default();
    /// assert_eq!(encoder.encode("½ ≤ α").unwrap(), r"{\textonehalf} \ensuremath{\leq} \ensuremath{\alpha}");
    /// ```
    pub fn encode(&self, text: &str) -> Result<String, EncodeError> {
        let mut out = String::with_capacity(text.len());
        self.encode_into(&mut out, text)?;
        Ok(out)
    }

    /// Encode `text` and append the result to `out`, returning an
    /// [`EncodeReport`]: what the output needs in the preamble, and the
    /// characters outside ASCII that no rule knew.
    ///
    /// The list of unknown characters is pylatexenc's unknown-character warning
    /// in the form of a value: the policy has already written what it writes
    /// for each of them, and the caller decides whether to report them. It is
    /// empty whenever every character was handled by a rule or is ASCII.
    ///
    /// # Errors
    ///
    /// [`EncodeError::UnknownCharacter`] under [`UnknownCharPolicy::Fail`]
    /// at the first unknown character, and [`EncodeError::InvalidConsumption`]
    /// when a callable rule reports a consumption that is zero or does not end
    /// on a character boundary. In both cases `out` holds the output up to the
    /// position of the error.
    ///
    /// # Panics
    ///
    /// Raises no panic of its own; a panic inside a function the options carry
    /// — a [callable rule](Rule::callable), a [`Protection::Custom`] or an
    /// [`UnknownCharPolicy::Custom`] — is not caught.
    ///
    /// ```
    /// use flm_latexencode::{Encoder, UnknownChar};
    ///
    /// let encoder = Encoder::default();
    /// let mut out = String::from("% ");
    /// let report = encoder.encode_into(&mut out, "Thai: ธ").unwrap();
    /// assert_eq!(out, "% Thai: ธ");
    /// assert_eq!(report.unknown, vec![UnknownChar { ch: 'ธ', position: 6 }]);
    /// assert!(report.needs.is_empty());
    /// ```
    pub fn encode_into(
        &self,
        out: &mut String,
        text: &str,
    ) -> Result<EncodeReport, EncodeError> {
        let normalized = composed(text);
        let s: &str = &normalized;
        let mut report = EncodeReport::default();
        let mut pos = 0;
        while let Some(ch) = s[pos..].chars().next() {
            if self.options.non_ascii_only && (ch as u32) < 127 {
                out.push(ch);
                pos += ch.len_utf8();
                continue;
            }
            let mut applied = None;
            for rule in &self.options.rules {
                if let Some((consumed, latex, needs)) = rule.apply(s, pos, ch)? {
                    let protection = rule.protection.as_ref().unwrap_or(&self.options.protection);
                    protection.apply_into(out, &latex);
                    report.needs.include(needs);
                    applied = Some(consumed);
                    break;
                }
            }
            if let Some(consumed) = applied {
                pos += consumed;
                continue;
            }
            if (32..=127).contains(&(ch as u32)) || matches!(ch, '\n' | '\r' | '\t') {
                out.push(ch);
            } else {
                report.unknown.push(UnknownChar { ch, position: pos });
                self.options.unknown_char_policy.apply_into(out, ch, pos)?;
            }
            pos += ch.len_utf8();
        }
        Ok(report)
    }
}

/// What one encoding produced beyond the LaTeX text itself, as
/// [`Encoder::encode_into`] returns it: what the text needs in the preamble,
/// and the characters no rule knew.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EncodeReport {
    /// What a document holding this text must load in its preamble, from the
    /// built-in table's entries alone ([`PreambleNeeds`]).
    pub needs: PreambleNeeds,
    /// The characters outside ASCII that no rule knew, in order of occurrence,
    /// each with its byte position in the normalized input.
    pub unknown: Vec<UnknownChar>,
}

/// A character outside ASCII that no rule of the encoder knew, as
/// [`Encoder::encode_into`] reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnknownChar {
    /// The character.
    pub ch: char,
    /// Its byte position in the normalized input.
    pub position: usize,
}

/// The ways encoding can fail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    /// A character outside ASCII that no rule knew, met under
    /// [`UnknownCharPolicy::Fail`].
    UnknownCharacter {
        /// The character.
        ch: char,
        /// Its byte position in the normalized input.
        position: usize,
    },
    /// A callable rule reported a consumption that is not a valid advance:
    /// zero bytes, more than remain, or a count ending inside a character.
    InvalidConsumption {
        /// The byte position the rule was tried at.
        position: usize,
        /// The number of bytes it claimed to consume.
        consumed: usize,
    },
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodeError::UnknownCharacter { ch, position } => write!(
                f,
                "no known LaTeX representation for character U+{:04X} ‘{}’ at byte {}",
                *ch as u32, ch, position
            ),
            EncodeError::InvalidConsumption { position, consumed } => write!(
                f,
                "a callable rule reported consuming {consumed} bytes at byte {position}; \
                 a rule must consume at least one whole character, or answer None"
            ),
        }
    }
}

impl std::error::Error for EncodeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn braces_protection_wraps_exactly_the_named_command_endings() {
        let wrap = |latex: &str| {
            let mut out = String::new();
            Protection::Braces.apply_into(&mut out, latex);
            out
        };
        assert_eq!(wrap(r"\textemdash"), r"{\textemdash}");
        assert_eq!(wrap(r"\l"), r"{\l}");
        assert_eq!(wrap(r"\'e"), r"\'e");
        assert_eq!(wrap(r"\c{c}"), r"\c{c}");
        assert_eq!(wrap(r"\ensuremath{\alpha}"), r"\ensuremath{\alpha}");
        assert_eq!(wrap("''"), "''");
        assert_eq!(wrap(""), "");
        assert_eq!(wrap(r"\"), r"\");
    }

    #[test]
    fn braces_all_wraps_the_empty_spelling_too() {
        let mut out = String::new();
        Protection::BracesAll.apply_into(&mut out, "");
        assert_eq!(out, "{}");
    }

    #[test]
    fn apply_is_what_the_encoder_applies_to_every_spelling() {
        let modes = [
            Protection::Braces,
            Protection::BracesAll,
            Protection::BracesAlmostAll,
            Protection::BracesAfterMacro,
            Protection::None,
            Protection::Custom(Arc::new(|latex: &str| format!("<{latex}>"))),
        ];
        for protection in modes {
            let encoder = Encoder::new(Options { protection: protection.clone(), ..Options::default() });
            for ch in ['—', 'é', 'α', 'ł', 'ﬁ', '\u{2061}'] {
                let spelling = spelling_of(ch).unwrap().latex;
                assert_eq!(
                    encoder.encode(&ch.to_string()).unwrap(),
                    protection.apply(spelling),
                    "{protection:?} on U+{:04X}",
                    ch as u32
                );
            }
        }
        assert_eq!(Protection::BracesAll.apply(""), "{}");
        assert_eq!(Protection::None.apply(r"\l"), r"\l");
    }

    #[test]
    fn composed_borrows_composed_text_and_composes_the_rest() {
        assert!(matches!(composed("Caf\u{e9} \u{3b1}"), Cow::Borrowed(_)));
        assert!(matches!(composed(""), Cow::Borrowed(_)));
        let owned = composed("Cafe\u{301} A\u{30a}");
        assert!(matches!(owned, Cow::Owned(_)));
        assert_eq!(owned, "Caf\u{e9} \u{c5}");
        // The encoder and the single-character query see the same characters.
        assert_eq!(encode("e\u{301}"), r"\'e");
        assert_eq!(spelling_of('\u{212B}'), spelling_of('\u{c5}'));
    }

    #[test]
    fn mode_is_read_off_the_spelling_s_form() {
        assert_eq!(Mode::of(r"\ensuremath{\alpha}"), Mode::Math);
        assert_eq!(Mode::of(r"\ensuremath{\mathbb{1}}"), Mode::Math);
        assert_eq!(Mode::of(r"\ensuremath{^\circ}F"), Mode::Text);
        assert_eq!(Mode::of(r"\'{}\ensuremath{\Omega}"), Mode::Text);
        assert_eq!(Mode::of(r"{\small\ensuremath{\blacksquare}}"), Mode::Text);
        assert_eq!(Mode::of(r"\'e"), Mode::Text);
        assert_eq!(Mode::of("''"), Mode::Any);
        assert_eq!(Mode::of(""), Mode::Any);
        assert_eq!(Mode::of(r"\ensuremath{\{}"), Mode::Math);
        assert_eq!(Mode::of(r"\ensuremath{"), Mode::Text);
    }

    #[test]
    fn spelling_of_falls_back_to_the_composed_form() {
        // U+212B ANGSTROM SIGN composes to U+00C5, which the table knows.
        assert_eq!(spelling_of('\u{212B}').map(|s| s.latex), Some(r"\r{A}"));
        // A lone combining acute accent composes to nothing else.
        assert_eq!(spelling_of('\u{0301}'), None);
    }

    #[test]
    fn a_callable_consuming_past_the_end_or_inside_a_character_is_refused() {
        let past = Encoder::new(Options {
            rules: vec![Rule::callable(|_, _| Some((100, String::new())))],
            ..Options::default()
        });
        assert_eq!(
            past.encode("ab"),
            Err(EncodeError::InvalidConsumption { position: 0, consumed: 100 })
        );
        let inside = Encoder::new(Options {
            rules: vec![Rule::callable(|_, _| Some((1, String::new())))],
            ..Options::default()
        });
        assert_eq!(
            inside.encode("é"),
            Err(EncodeError::InvalidConsumption { position: 0, consumed: 1 })
        );
    }

    #[test]
    fn errors_display_their_position() {
        let e = EncodeError::UnknownCharacter { ch: 'ธ', position: 3 };
        assert_eq!(e.to_string(), "no known LaTeX representation for character U+0E18 ‘ธ’ at byte 3");
        let e = EncodeError::InvalidConsumption { position: 3, consumed: 0 };
        assert!(e.to_string().starts_with("a callable rule reported consuming 0 bytes at byte 3"));
    }

    #[test]
    fn debug_forms_name_the_kinds() {
        let rule = Rule::callable(|_, _| None).with_protection(Protection::None);
        assert_eq!(format!("{rule:?}"), "Rule { callable: \"..\", protection: Some(None) }");
        assert_eq!(format!("{:?}", Rule::defaults()), "Rule { table: 1549, protection: None }");
        assert_eq!(format!("{:?}", UnknownCharPolicy::Custom(Arc::new(|_| String::new()))), "Custom(..)");
    }
}
