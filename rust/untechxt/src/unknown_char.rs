//! The [`UnknownCharPolicy`] enum, which decides what the encoder writes for a
//! character that no rule matched, and the [`unknown_unihex`] helper that
//! spells such a character out as its code point.

use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use core::fmt;

use crate::encoder::EncodeError;
use crate::outbuffer::OutBuffer;

/// How the encoder handles a character that no rule matched.
///
/// Such a character is an *unknown character* when it is also not printable
/// ASCII (`0x20..=0x7E`) and not one of `\n`, `\r` and `\t`. The encoder
/// reports every unknown character to the
/// [`EncodeReporter`](crate::report::EncodeReporter), whichever policy is in
/// force, and the policy then determines what, if anything, is written for
/// the character.
///
/// Whatever a policy writes goes to the output as it is, with no protection,
/// no mode hint and no preamble needs. A policy is a last resort. For output
/// richer than a fixed substitution, add a rule at the end of the rule chain
/// instead of using a policy.
///
/// The default is [`Keep`](UnknownCharPolicy::Keep). Set another policy on an
/// encoder with
/// [`Encoder::with_unknown_chars`](crate::Encoder::with_unknown_chars).
///
/// ```
/// use untechxt::rule::RuleChain;
/// use untechxt::{Encoder, UnknownCharPolicy};
///
/// let encoder = Encoder::new(RuleChain::new(()))
///     .with_unknown_chars(UnknownCharPolicy::Ignore);
/// assert_eq!(encoder.encode("a\u{e9}b").unwrap(), "ab");
/// ```
#[derive(Default)]
pub enum UnknownCharPolicy {
    /// Keeps the character itself, encoded as UTF-8. This is the default.
    /// Pick it when the output is consumed by a LaTeX setup that accepts the
    /// character directly, such as a document with a matching input encoding
    /// and fonts.
    #[default]
    Keep,
    /// Writes nothing for the character, dropping it from the output. Pick it
    /// to silently discard any character that the output cannot represent.
    Ignore,
    /// Stops encoding and returns [`EncodeError::UnknownChar`]. Pick it to
    /// treat an unknown character as an error instead of encoding the text
    /// incompletely.
    Fail,
    /// Writes this fixed text for every unknown character, whatever the
    /// character is. Pick it for a single placeholder such as `"?"`. The text
    /// is a [`Cow`](alloc::borrow::Cow), so a `&'static str` costs no
    /// allocation.
    ReplaceWith(Cow<'static, str>),
    /// Writes what this function returns for the character. The function
    /// receives the character and returns the text to write for it. Pick it
    /// to produce output that depends on the character, for instance
    /// [`unknown_unihex`], which spells the character out as its code point.
    ///
    /// The bound `Send + Sync` keeps the mere presence of this variant from
    /// making every encoder single-threaded. Build this variant with
    /// [`callback`](UnknownCharPolicy::callback), which boxes the function.
    Callback(Box<dyn Fn(char) -> String + Send + Sync>),
}

impl UnknownCharPolicy {
    /// Returns a [`Callback`](UnknownCharPolicy::Callback) policy that writes
    /// what `f` returns for each unknown character. This boxes `f`. Pass
    /// [`unknown_unihex`] to spell the code point out, or a function of your
    /// own.
    ///
    /// ```
    /// use untechxt::rule::RuleChain;
    /// use untechxt::{unknown_unihex, Encoder, UnknownCharPolicy};
    ///
    /// let encoder = Encoder::new(RuleChain::new(()))
    ///     .with_unknown_chars(UnknownCharPolicy::callback(unknown_unihex));
    /// assert_eq!(
    ///     encoder.encode("\u{0e18}").unwrap(),
    ///     r"\ensuremath{\langle}\texttt{U+0E18}\ensuremath{\rangle}"
    /// );
    /// ```
    pub fn callback(f: impl Fn(char) -> String + Send + Sync + 'static) -> Self {
        UnknownCharPolicy::Callback(Box::new(f))
    }

    /// Applies the policy to the unknown character `ch`, met at byte
    /// `position`, and writes any resulting text to `out`.
    pub(crate) fn apply_into<O: OutBuffer>(
        &self,
        out: &mut O,
        ch: char,
        position: usize,
    ) -> Result<(), EncodeError> {
        match self {
            UnknownCharPolicy::Keep => out.push_char(ch).map_err(EncodeError::Output),
            UnknownCharPolicy::Ignore => Ok(()),
            UnknownCharPolicy::Fail => Err(EncodeError::UnknownChar { ch, position }),
            UnknownCharPolicy::ReplaceWith(text) => {
                out.push_str(text).map_err(EncodeError::Output)
            }
            UnknownCharPolicy::Callback(f) => {
                out.push_str(&f(ch)).map_err(EncodeError::Output)
            }
        }
    }
}

impl fmt::Debug for UnknownCharPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnknownCharPolicy::Keep => f.write_str("Keep"),
            UnknownCharPolicy::Ignore => f.write_str("Ignore"),
            UnknownCharPolicy::Fail => f.write_str("Fail"),
            UnknownCharPolicy::ReplaceWith(text) => {
                f.debug_tuple("ReplaceWith").field(text).finish()
            }
            UnknownCharPolicy::Callback(_) => f.write_str("Callback(..)"),
        }
    }
}

/// Spells `ch` out as its Unicode code point, in typewriter type between angle
/// brackets. The code point is written in uppercase hexadecimal, with at least
/// four digits.
///
/// This is pylatexenc's `'unihex'` unknown-character mode. Pass it to the
/// [`UnknownCharPolicy::callback`] constructor to use it.
///
/// ```
/// use untechxt::unknown_unihex;
///
/// assert_eq!(
///     unknown_unihex('\u{0e18}'),
///     r"\ensuremath{\langle}\texttt{U+0E18}\ensuremath{\rangle}"
/// );
/// ```
pub fn unknown_unihex(ch: char) -> String {
    format!("\\ensuremath{{\\langle}}\\texttt{{U+{:04X}}}\\ensuremath{{\\rangle}}", ch as u32)
}
