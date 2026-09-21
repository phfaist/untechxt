//! [`UnknownCharPolicy`]: what is written for a character no rule knew.

use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use core::fmt;

use crate::encoder::EncodeError;
use crate::outbuffer::OutBuffer;

/// What the encoder writes for a character that no rule matched and that is
/// not printable ASCII (`0x20..=0x7E`) or one of `\n`, `\r`, `\t`.
///
/// Whatever the policy says is written without protection, without a hint and
/// without preamble needs: a policy is a last resort, and anything richer is
/// a rule at the end of the chain. The character is reported to the
/// [`EncodeReporter`](crate::EncodeReporter) whichever policy is in force.
///
/// The default is [`Keep`](UnknownCharPolicy::Keep).
///
/// ```
/// use untechxt::{Encoder, RuleChain, UnknownCharPolicy};
///
/// let encoder =
///     Encoder::new(RuleChain::new(())).with_unknown_chars(UnknownCharPolicy::Ignore);
/// assert_eq!(encoder.encode("a\u{e9}b").unwrap(), "ab");
/// ```
#[derive(Default)]
pub enum UnknownCharPolicy {
    /// Keep the character itself, as UTF-8. The default.
    #[default]
    Keep,
    /// Write nothing at all for it.
    Ignore,
    /// Stop with [`EncodeError::UnknownChar`].
    Fail,
    /// Write this fixed text for any unknown character.
    ReplaceWith(Cow<'static, str>),
    /// Write what this function answers for the character.
    ///
    /// The bound `Send + Sync` keeps the mere existence of this variant from
    /// making every encoder single-threaded; build one with
    /// [`callback`](UnknownCharPolicy::callback).
    Callback(Box<dyn Fn(char) -> String + Send + Sync>),
}

impl UnknownCharPolicy {
    /// The policy that writes what `f` answers for the character.
    ///
    /// ```
    /// use untechxt::{unknown_unihex, Encoder, RuleChain, UnknownCharPolicy};
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

    /// Writes what the policy says for `ch`, met at byte `position`.
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

/// The character's code point in typewriter type between angle brackets:
/// `\ensuremath{\langle}\texttt{U+0E18}\ensuremath{\rangle}`, with the code
/// point in uppercase hexadecimal, at least four digits.
///
/// This is pylatexenc's `'unihex'` unknown-character policy; pass it to
/// [`UnknownCharPolicy::callback`].
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
