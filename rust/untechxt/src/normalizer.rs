//! Input normalization: the [`InputNormalizer`] trait, its default
//! implementation [`NormalizeNfc`], the pass-through [`NoNormalization`], and
//! the [`nfc`] function.

use alloc::borrow::Cow;
use alloc::string::String;

use unicode_normalization::{is_nfc_quick, IsNormalized, UnicodeNormalization};

/// A transformation the encoder applies to the input before any rule is
/// tried. The default is [`NormalizeNfc`], and [`NoNormalization`] leaves
/// the input unchanged.
///
/// Implement this trait to preprocess the input in a custom way, for example
/// to apply a different Unicode normalization form. The
/// [`normalize`](InputNormalizer::normalize) method takes the input text and
/// returns the text the rules will see.
///
/// The positions in an [`EncodeError`](crate::EncodeError) and in
/// [`EncodeReporter::report_unknown_char`] are byte offsets into the text
/// the normalizer returned, which is not the caller's text when the
/// normalizer changed it. A caller who needs offsets into its own text
/// normalizes it once with [`nfc`] and encodes with [`NoNormalization`].
///
/// [`EncodeReporter::report_unknown_char`]:
///     crate::report::EncodeReporter::report_unknown_char
pub trait InputNormalizer: core::fmt::Debug {
    /// Returns the text the rules will see. Return `text` itself, borrowed,
    /// when the normalizer has nothing to change, and a new string
    /// otherwise.
    fn normalize<'t>(&self, text: &'t str) -> Cow<'t, str>;
}

/// The input normalizer that puts the input in Unicode's canonical composed
/// form (NFC) before encoding. This is the default normalizer of
/// [`Encoder`](crate::Encoder).
///
/// Without it, a decomposed `e` followed by a combining acute accent would
/// be two characters, neither of which any table knows, instead of the `é`
/// that encodes as `\'e`. See [`nfc`] for what canonical composition does.
///
/// ```
/// use untechxt::normalizer::{InputNormalizer, NormalizeNfc};
/// use untechxt::rule::RuleChain;
/// use untechxt::Encoder;
///
/// assert_eq!(NormalizeNfc.normalize("Cafe\u{301}"), "Caf\u{e9}");
/// let _ = Encoder::new(RuleChain::new(())).with_normalizer(NormalizeNfc);
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct NormalizeNfc;

impl InputNormalizer for NormalizeNfc {
    fn normalize<'t>(&self, text: &'t str) -> Cow<'t, str> {
        nfc(text)
    }
}

/// The input normalizer that leaves the input unchanged, so the rules see
/// the caller's text exactly as it is.
///
/// Positions then refer to the caller's own text. Nothing in the crate then
/// references the normalization tables, which the linker can drop.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct NoNormalization;

impl InputNormalizer for NoNormalization {
    fn normalize<'t>(&self, text: &'t str) -> Cow<'t, str> {
        Cow::Borrowed(text)
    }
}

/// Returns `text` in Unicode's canonical composed form (NFC). A letter
/// followed by combining accents is replaced by the single accented letter
/// wherever Unicode defines one, so that `e` followed by a combining acute
/// accent becomes `é`. A character with a composed equivalent is replaced by
/// it, so that the Angstrom sign becomes the letter `Å`.
///
/// The input comes back borrowed, with no copy, when a quick check shows
/// that it is already in composed form, which is the case for nearly all
/// text. It comes back as a new string otherwise.
///
/// This is the normalization that [`NormalizeNfc`] applies. A caller that
/// needs byte positions into a text of its own normalizes it once with this
/// function and encodes the result with [`NoNormalization`].
///
/// ```
/// use untechxt::normalizer::nfc;
///
/// assert_eq!(nfc("Cafe\u{301}"), "Caf\u{e9}");
/// assert_eq!(nfc("Caf\u{e9}"), "Caf\u{e9}");
/// ```
pub fn nfc(text: &str) -> Cow<'_, str> {
    match is_nfc_quick(text.chars()) {
        IsNormalized::Yes => Cow::Borrowed(text),
        IsNormalized::No | IsNormalized::Maybe => Cow::Owned(text.nfc().collect::<String>()),
    }
}
