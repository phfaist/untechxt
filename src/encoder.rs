//! [`Encoder`]: the loop that turns Unicode text into LaTeX, and the ways it
//! can fail.

use alloc::string::String;
use core::fmt;

use crate::asciiset::AsciiSet;
use crate::normalizer::{InputNormalizer, NormalizeNfc};
use crate::outbuffer::OutBuffer;
use crate::profile::Profile;
use crate::replacement_protection::{ProtectInput, ReplacementProtection, StandardProtection};
use crate::report::{EncodeReport, EncodeReporter, NoReport};
use crate::rule::{BoxError, Rule, RuleInput};
use crate::unknown_char::UnknownCharPolicy;

/// The ASCII characters the encoder always stops at, whatever its rules say:
/// every non-printable one, so that they reach the rules and then the
/// unknown-character policy.
const ALWAYS_STOP: AsciiSet = AsciiSet::range(0x00..=0x1f).union(AsciiSet::range(0x7f..=0x7f));

/// What turns Unicode text into LaTeX: a [`Rule`], a
/// [`ReplacementProtection`] strategy, an [`InputNormalizer`] and an
/// [`UnknownCharPolicy`].
///
/// An encoder is built once and used for any number of strings. It is
/// immutable, so it can be shared between threads whenever its rules can
/// ([`Rule`] requires neither `Send` nor `Sync`, so that rules holding a
/// JavaScript callback or an [`Rc`](alloc::rc::Rc) are possible; an encoder
/// holding those simply is not `Send`).
///
/// # How a string is encoded
///
/// 1. The input goes through the normalizer — by default
///    [`NormalizeNfc`], so that a letter followed by a
///    combining accent becomes the single accented character the tables know.
/// 2. The text is scanned byte by byte. An ASCII byte that no rule can match
///    at (see [`Rule::ascii_triggers`]) and that is printable extends the
///    current run, which is copied to the output in one piece.
/// 3. Anywhere else, the character is decoded and the rules are tried in
///    order; the first match wins, no rule is tried after it, and no longest
///    match is sought. Its value is written through the protection strategy,
///    what it needs is reported, and the position advances by what the rule
///    consumed.
/// 4. Where no rule matched: a printable ASCII character, or a newline,
///    carriage return or tab, is copied as it is. Any other character is an
///    unknown character: it is reported, and the
///    [`UnknownCharPolicy`] decides what is written for it — without
///    protection.
///
/// Every position — in an [`EncodeError`], in
/// [`EncodeReporter::report_unknown_char`] — is a byte offset into the
/// normalized text.
///
/// ```
/// use untechxt::{DynTable, Encoder, ReplacementProtectionHint as Hint};
///
/// let mut table = DynTable::new();
/// table.insert('\u{e9}', r"\'e", Hint::text_only(r"\'e"));
/// table.insert('\u{2014}', r"\textemdash", Hint::text_only(r"\textemdash"));
/// let encoder = Encoder::new(table);
///
/// assert_eq!(encoder.encode("Caf\u{e9} \u{2014} ok").unwrap(), r"Caf\'e {\textemdash} ok");
/// ```
///
/// The encoder is not [`Clone`]: an [`UnknownCharPolicy`] may hold a boxed
/// callback, which cannot be cloned.
#[derive(Debug)]
pub struct Encoder<R, P = StandardProtection, N = NormalizeNfc> {
    /// The rule tried at every position; a [`RuleChain`](crate::RuleChain)
    /// for more than one.
    rule: R,
    /// What is written around each value.
    protection: P,
    /// What the input goes through before any rule sees it.
    normalizer: N,
    /// What is written for a character no rule knew.
    unknown_chars: UnknownCharPolicy,
    /// The ASCII characters the scan stops at: the rule's triggers together
    /// with every non-printable ASCII character. Computed once, here.
    stop_set: AsciiSet,
}

impl<R: Rule> Encoder<R> {
    /// The encoder that applies `rule`, protects its values for text-mode
    /// output ([`StandardProtection::text_mode`]), normalizes its input to
    /// NFC ([`NormalizeNfc`]) and keeps unknown characters
    /// ([`UnknownCharPolicy::Keep`]).
    ///
    /// This asks the rule for its [`ascii_triggers`](Rule::ascii_triggers)
    /// once and remembers them, which is why it is not a `const fn`. Building
    /// an encoder is cheap all the same; build one and keep it.
    pub fn new(rule: R) -> Self {
        let stop_set = rule.ascii_triggers().union(ALWAYS_STOP);
        Encoder {
            rule,
            protection: StandardProtection::text_mode(),
            normalizer: NormalizeNfc,
            unknown_chars: UnknownCharPolicy::Keep,
            stop_set,
        }
    }
}

impl<R: Rule, P: ReplacementProtection, N: InputNormalizer> Encoder<R, P, N> {
    /// The same encoder with `protection` as its protection strategy.
    #[must_use]
    pub fn with_protection<P2: ReplacementProtection>(self, protection: P2) -> Encoder<R, P2, N> {
        Encoder {
            rule: self.rule,
            protection,
            normalizer: self.normalizer,
            unknown_chars: self.unknown_chars,
            stop_set: self.stop_set,
        }
    }

    /// The same encoder with `normalizer` as its input normalizer.
    #[must_use]
    pub fn with_normalizer<N2: InputNormalizer>(self, normalizer: N2) -> Encoder<R, P, N2> {
        Encoder {
            rule: self.rule,
            protection: self.protection,
            normalizer,
            unknown_chars: self.unknown_chars,
            stop_set: self.stop_set,
        }
    }

    /// The same encoder with `policy` for the characters no rule knows.
    #[must_use]
    pub fn with_unknown_chars(mut self, policy: UnknownCharPolicy) -> Self {
        self.unknown_chars = policy;
        self
    }

    /// The encoder's protection strategy.
    pub fn protection(&self) -> &P {
        &self.protection
    }

    /// The LaTeX for `text`, as a new string.
    ///
    /// # Errors
    ///
    /// [`EncodeError`]: a character no rule knew under
    /// [`UnknownCharPolicy::Fail`], a rule that failed, or an output that
    /// refused what was written to it.
    pub fn encode(&self, text: &str) -> Result<String, EncodeError> {
        let mut out = String::with_capacity(text.len());
        self.encode_into(text, &mut out, &mut NoReport)?;
        Ok(out)
    }

    /// The LaTeX for `text` and what it needs in the document's preamble.
    ///
    /// Encode the fragments of one document with
    /// [`encode_into`](Encoder::encode_into) and one shared report instead,
    /// where there are several.
    ///
    /// # Errors
    ///
    /// As for [`encode`](Encoder::encode).
    pub fn encode_with_report(
        &self,
        text: &str,
    ) -> Result<(String, EncodeReport), EncodeError> {
        let mut out = String::with_capacity(text.len());
        let mut report = EncodeReport::new();
        self.encode_into(text, &mut out, &mut report)?;
        Ok((out, report))
    }

    /// Encodes `text`, appending the LaTeX to `out` and telling `report` what
    /// the output needs and which characters no rule knew.
    ///
    /// This is the primitive the other two are written with: both `out` and
    /// `report` are appended to, so that the fragments of one document can
    /// share a preamble.
    ///
    /// # Errors
    ///
    /// As for [`encode`](Encoder::encode). On an error the output written so
    /// far stays in `out` and the report stays partly filled.
    pub fn encode_into<O: OutBuffer, Rep: EncodeReporter>(
        &self,
        text: &str,
        out: &mut O,
        report: &mut Rep,
    ) -> Result<(), EncodeError> {
        let normalized = self.normalizer.normalize(text);
        let text: &str = &normalized;
        let bytes = text.as_bytes();
        // The start of the run of input bytes that will be copied as it is.
        let mut run_start = 0usize;
        let mut pos = 0usize;
        // The profile reported last, to skip immediate repeats; a local, so
        // that it starts afresh at every call.
        let mut last_needs: Option<&Profile> = None;

        while pos < bytes.len() {
            let byte = bytes[pos];
            if byte < 0x80 && !self.stop_set.contains(byte) {
                pos += 1;
                continue;
            }
            // `pos` is at a character boundary, so the character is there.
            let Some(ch) = text[pos..].chars().next() else { break };
            let input = RuleInput::at(text, pos, ch);
            let matched = self
                .rule
                .apply(input)
                .map_err(|source| EncodeError::Rule { position: pos, source })?;
            match matched {
                Some(replacement) => {
                    if run_start < pos {
                        out.push_str(&text[run_start..pos]).map_err(EncodeError::Output)?;
                    }
                    let item = ProtectInput::new(replacement.encoded(), replacement.hint());
                    self.protection
                        .write_protected(out, report, item)
                        .map_err(EncodeError::Output)?;
                    if let Some(profile) = replacement.needs() {
                        let repeated = matches!(last_needs, Some(last) if core::ptr::eq(last, profile));
                        if !repeated {
                            report.report_needs(profile);
                            last_needs = Some(profile);
                        }
                    }
                    pos += replacement.consumed();
                    // A replacement built from the `RuleInput` the rule was
                    // handed cannot land here; only a rule that fabricated a
                    // `RuleInput` over another string can (out of contract).
                    debug_assert!(
                        text.is_char_boundary(pos),
                        "rule consumed past the end or into the middle of a char"
                    );
                    run_start = pos;
                }
                // Printable ASCII, and the three white-space characters, stay
                // in the run and are copied with it.
                None if is_copied_as_is(ch) => pos += ch.len_utf8(),
                None => {
                    if run_start < pos {
                        out.push_str(&text[run_start..pos]).map_err(EncodeError::Output)?;
                    }
                    report.report_unknown_char(ch, pos);
                    self.unknown_chars.apply_into(out, ch, pos)?;
                    pos += ch.len_utf8();
                    run_start = pos;
                }
            }
        }
        if run_start < text.len() {
            out.push_str(&text[run_start..]).map_err(EncodeError::Output)?;
        }
        Ok(())
    }
}

/// Whether a character that no rule matched is written through as it is,
/// rather than being an unknown character: printable ASCII (`0x20..=0x7E`)
/// and the three white-space characters `\n`, `\r` and `\t`.
const fn is_copied_as_is(ch: char) -> bool {
    matches!(ch, ' '..='~' | '\n' | '\r' | '\t')
}

/// The ways encoding can fail.
///
/// ```
/// use untechxt::{Encoder, EncodeError, RuleChain, UnknownCharPolicy};
///
/// let encoder = Encoder::new(RuleChain::new(()))
///     .with_unknown_chars(UnknownCharPolicy::Fail);
/// let error = encoder.encode("ab \u{e9}").unwrap_err();
/// assert!(matches!(error, EncodeError::UnknownChar { ch: '\u{e9}', position: 3 }));
/// ```
#[derive(Debug)]
#[non_exhaustive]
pub enum EncodeError {
    /// A character that no rule knew, met under
    /// [`UnknownCharPolicy::Fail`].
    UnknownChar {
        /// The character.
        ch: char,
        /// Its byte position in the normalized input.
        position: usize,
    },
    /// A rule reported an error of its own, and the encoding stopped.
    Rule {
        /// The byte position in the normalized input the rule was tried at.
        position: usize,
        /// What the rule reported.
        source: BoxError,
    },
    /// The output buffer, or the protection strategy writing to it, reported
    /// an error.
    Output(BoxError),
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodeError::UnknownChar { ch, position } => write!(
                f,
                "no known LaTeX representation for character U+{:04X} '{}' at byte {}",
                *ch as u32, ch, position
            ),
            EncodeError::Rule { position, source } => {
                write!(f, "a rule failed at byte {position}: {source}")
            }
            EncodeError::Output(source) => write!(f, "writing the output failed: {source}"),
        }
    }
}

impl core::error::Error for EncodeError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            EncodeError::UnknownChar { .. } => None,
            EncodeError::Rule { source, .. } | EncodeError::Output(source) => Some(&**source),
        }
    }
}
