//! The [`Encoder`] struct, which turns Unicode text into LaTeX, and its
//! error type [`EncodeError`].

use alloc::string::String;
use core::fmt;

use crate::defaults::DefaultRules;
use crate::normalizer::{InputNormalizer, NormalizeNfc};
use crate::outbuffer::OutBuffer;
use crate::preamble::Profile;
use crate::protection::{ProtectInput, ReplacementProtection, StandardProtection};
use crate::report::{EncodeReport, EncodeReporter, NoReport};
use crate::rule::{AsciiSet, Rule, RuleInput};
use crate::unknown_char::UnknownCharPolicy;
use crate::BoxError;

/// The ASCII characters the scan always stops at, whatever the rule's
/// triggers are: every non-printable ASCII character (`0x00..=0x1F` and
/// `0x7F`). Stopping at them lets each one reach the rule and then the
/// unknown-character policy.
const ALWAYS_STOP: AsciiSet = AsciiSet::range(0x00..=0x1f).union(AsciiSet::range(0x7f..=0x7f));

/// A Unicode-to-LaTeX encoder, built from a rule, a replacement protection
/// strategy, an input normalizer and an unknown-character policy.
///
/// An encoder is built once with [`Encoder::new`] and used to encode any
/// number of strings. It is immutable, so it can be shared across threads
/// provided its rule is `Send` and `Sync`. [`Rule`] requires neither `Send`
/// nor `Sync`, so that a rule holding a JavaScript callback or an
/// [`Rc`](alloc::rc::Rc) is possible, and an encoder that holds such a rule
/// is not `Send`.
///
/// Each of the four parts has a default and is set through a builder method:
///
/// - The *rule* maps an input character, or an input substring, to the
///   LaTeX that replaces it. It is given to [`Encoder::new`] and has no
///   default. Pass [`default_rules`](crate::default_rules) for the crate's
///   builtin symbol encoding table, or a
///   [`RuleChain`](crate::rule::RuleChain) to apply several rules in order.
/// - The *replacement protection strategy* writes the syntax around each
///   value that keeps the surrounding LaTeX valid. The default is
///   [`StandardProtection::text_mode`], and
///   [`with_protection`](Encoder::with_protection) sets another.
/// - The *input normalizer* preprocesses the input before any rule sees it.
///   The default is [`NormalizeNfc`], Unicode's canonical composed form, and
///   [`with_normalizer`](Encoder::with_normalizer) sets another.
/// - The *unknown-character policy* determines what is written for a
///   character that no rule matched and that is not printable ASCII. The
///   default is [`UnknownCharPolicy::Keep`], which keeps the character, and
///   [`with_unknown_chars`](Encoder::with_unknown_chars) sets another.
///
/// ```
/// use untechxt::lookuptable::DynTable;
/// use untechxt::protection::ReplacementProtectionHint as Hint;
/// use untechxt::Encoder;
///
/// let mut table = DynTable::new();
/// table.insert('\u{e9}', r"\'e", Hint::text_only(r"\'e"));
/// table.insert('\u{2014}', r"\textemdash", Hint::text_only(r"\textemdash"));
/// let encoder = Encoder::new(table);
///
/// assert_eq!(
///     encoder.encode("Caf\u{e9} \u{2014} ok").unwrap(),
///     r"Caf\'e {\textemdash} ok",
/// );
/// ```
///
/// # Encoding a string
///
/// An encoder has three encoding methods, from the most convenient to the
/// most flexible:
///
/// - [`encode`](Encoder::encode) takes a string and returns the LaTeX as a
///   new string, with no side effects. Use it when what the output needs in
///   the preamble does not matter.
/// - [`encode_with_report`](Encoder::encode_with_report) takes a string and
///   returns the LaTeX together with an [`EncodeReport`], which records what
///   the output needs in the document's preamble and which characters no
///   rule matched. Use it to encode one string and learn its preamble needs.
/// - [`encode_into`](Encoder::encode_into) takes a string, an [`OutBuffer`]
///   to append the LaTeX to, and an [`EncodeReporter`] to report to. Use it
///   to stream the output to a formatter or a file, and to encode the
///   fragments of one document into a shared report so that they share a
///   preamble.
///
/// # How a string is encoded
///
/// 1. The input goes through the normalizer, [`NormalizeNfc`] by default, so
///    that a letter followed by a combining accent becomes the single
///    accented character the tables contain.
/// 2. The normalized text is scanned byte by byte. A printable ASCII byte
///    that no rule can match at (see [`Rule::ascii_triggers`]) extends the
///    current run, which is copied to the output in one piece.
/// 3. At any other position the character is decoded and the rules are tried
///    in order. The first match wins, no rule is tried after it, and no
///    longest match is sought. The value of the matching rule is written
///    through the protection strategy, what that value needs is reported,
///    and the position advances by the number of bytes the rule consumed.
/// 4. Where no rule matched, a printable ASCII character, or a newline,
///    carriage return or tab, is copied as it is. Any other character is an
///    unknown character: it is reported, and then the [`UnknownCharPolicy`]
///    decides what is written for it, without protection.
///
/// Every position, in an [`EncodeError`] as in
/// [`EncodeReporter::report_unknown_char`], is a byte offset into the
/// normalized text.
///
/// The type parameters have defaults. The type `Encoder`, written with no
/// type arguments, is the type of the encoder that
/// `Encoder::new(default_rules())` returns (see
/// [`default_rules`](crate::default_rules)).
///
/// The encoder does not implement [`Clone`], because an
/// [`UnknownCharPolicy`] may hold a boxed callback, and a boxed callback
/// cannot be cloned.
#[derive(Debug)]
pub struct Encoder<R = DefaultRules, P = StandardProtection, N = NormalizeNfc> {
    /// The rule tried at every position; a
    /// [`RuleChain`](crate::rule::RuleChain) for more than one.
    rule: R,
    /// What is written around each value.
    protection: P,
    /// What the input goes through before any rule sees it.
    normalizer: N,
    /// The policy applied to a character that no rule matched.
    unknown_chars: UnknownCharPolicy,
    /// The ASCII characters the scan stops at: the rule's triggers together
    /// with every non-printable ASCII character. Computed once, here.
    stop_set: AsciiSet,
}

impl<R: Rule> Encoder<R> {
    /// Creates an encoder that applies `rule`, with the default settings: it
    /// protects each value for text-mode output
    /// ([`StandardProtection::text_mode`]), normalizes the input to NFC
    /// ([`NormalizeNfc`]), and keeps a character that no rule matched
    /// ([`UnknownCharPolicy::Keep`]). Change any of these with
    /// [`with_protection`](Encoder::with_protection),
    /// [`with_normalizer`](Encoder::with_normalizer) and
    /// [`with_unknown_chars`](Encoder::with_unknown_chars).
    ///
    /// This asks `rule` for its ASCII triggers
    /// ([`Rule::ascii_triggers`]) once and stores them, which is why it is
    /// not a `const fn`. Building an encoder is cheap all the same, so build
    /// one and keep it rather than building a new one for each string.
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
    /// Returns this encoder with `protection` as its replacement protection
    /// strategy, in place of the default [`StandardProtection::text_mode`].
    /// The strategy writes the syntax around each value that keeps the
    /// surrounding LaTeX valid. See the [`protection`](crate::protection)
    /// module for the available strategies.
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

    /// Returns this encoder with `normalizer` as its input normalizer, in
    /// place of the default [`NormalizeNfc`]. Pass
    /// [`NoNormalization`](crate::normalizer::NoNormalization) to encode the
    /// input exactly as it is. See the [`normalizer`](crate::normalizer)
    /// module for the available normalizers.
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

    /// Returns this encoder with `policy` for the characters that no rule
    /// matched, in place of the default [`UnknownCharPolicy::Keep`]. See
    /// [`UnknownCharPolicy`] for the available policies.
    #[must_use]
    pub fn with_unknown_chars(mut self, policy: UnknownCharPolicy) -> Self {
        self.unknown_chars = policy;
        self
    }

    /// Returns a reference to the encoder's replacement protection strategy.
    pub fn protection(&self) -> &P {
        &self.protection
    }

    /// Encodes `text` and returns the LaTeX as a new string.
    ///
    /// This reports nothing about what the output needs in the preamble. To
    /// also learn the preamble needs, use
    /// [`encode_with_report`](Encoder::encode_with_report).
    ///
    /// # Errors
    ///
    /// [`EncodeError`]: a character that no rule matched under
    /// [`UnknownCharPolicy::Fail`], a rule that failed, or an output that
    /// refused what was written to it.
    pub fn encode(&self, text: &str) -> Result<String, EncodeError> {
        let mut out = String::with_capacity(text.len());
        self.encode_into(text, &mut out, &mut NoReport)?;
        Ok(out)
    }

    /// Encodes `text` and returns the LaTeX together with an
    /// [`EncodeReport`].
    ///
    /// The report records what the output needs in the document's preamble
    /// and which characters no rule matched. To encode several fragments of
    /// one document into a single shared report, use
    /// [`encode_into`](Encoder::encode_into) with one report instead.
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

    /// Encodes `text`, appending the LaTeX to `out` and reporting to
    /// `report` what the output needs in the preamble and which characters
    /// no rule matched.
    ///
    /// This is the lower-level entry point. It writes to any [`OutBuffer`],
    /// so the output can stream to a formatter or a file instead of being
    /// assembled as one string, and it reports to any [`EncodeReporter`].
    /// Both `out` and `report` are appended to, so the fragments of one
    /// document can be encoded into a shared report and share a single
    /// preamble.
    ///
    /// # Errors
    ///
    /// As for [`encode`](Encoder::encode). On an error, the output written
    /// so far stays in `out` and the report stays partly filled.
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

/// The error returned by the encode methods of [`Encoder`] when encoding
/// fails. Each variant below says what triggers it.
///
/// ```
/// use untechxt::rule::RuleChain;
/// use untechxt::{EncodeError, Encoder, UnknownCharPolicy};
///
/// let encoder = Encoder::new(RuleChain::new(()))
///     .with_unknown_chars(UnknownCharPolicy::Fail);
/// let error = encoder.encode("ab \u{e9}").unwrap_err();
/// assert!(matches!(
///     error,
///     EncodeError::UnknownChar { ch: '\u{e9}', position: 3 }
/// ));
/// ```
#[derive(Debug)]
#[non_exhaustive]
pub enum EncodeError {
    /// A character that no rule matched, encountered under
    /// [`UnknownCharPolicy::Fail`]. Only that policy returns an error for
    /// such a character. The other policies write something for the character
    /// and encoding continues.
    UnknownChar {
        /// The character that no rule matched.
        ch: char,
        /// The byte position of the character in the normalized input.
        position: usize,
    },
    /// A rule failed with an error of its own, and encoding stopped. A rule
    /// can fail this way when, for instance, it wraps a callback written in
    /// another language and that callback raised an exception.
    Rule {
        /// The byte position in the normalized input where the rule was tried.
        position: usize,
        /// The error the rule returned.
        source: BoxError,
    },
    /// The output buffer, or the protection strategy writing to it, returned
    /// an error. For an output buffer over I/O, this carries the I/O error
    /// through unchanged.
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
