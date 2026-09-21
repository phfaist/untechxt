//! [`InputNormalizer`]: what the encoder does to the input before any rule
//! sees it.

use alloc::borrow::Cow;
use alloc::string::String;

use unicode_normalization::{is_nfc_quick, IsNormalized, UnicodeNormalization};

/// What the encoder puts the input through before any rule is tried.
///
/// The positions in an [`EncodeError`](crate::EncodeError) and in
/// [`EncodeReporter::report_unknown_char`](crate::EncodeReporter::report_unknown_char)
/// are byte offsets into the text the normalizer returned, which is not the
/// caller's text when the normalizer changed it. A caller who needs offsets
/// into its own text normalizes it once with [`nfc`] and encodes with
/// [`NoNormalization`].
pub trait InputNormalizer: core::fmt::Debug {
    /// The text the rules will see. Returning `text` itself, borrowed, is
    /// what a normalizer does when it has nothing to change.
    fn normalize<'t>(&self, text: &'t str) -> Cow<'t, str>;
}

/// Put the input in Unicode's canonical composed form (NFC) before encoding
/// it. The default of [`Encoder`](crate::Encoder).
///
/// Without it, a decomposed `e` followed by a combining acute accent would be
/// two characters, neither of which any table knows, instead of the `é` that
/// encodes as `\'e`.
///
/// ```
/// use untechxt::{Encoder, InputNormalizer, NormalizeNfc, RuleChain};
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

/// Encode the input exactly as it is.
///
/// Positions then refer to the caller's own text, and nothing in the crate
/// references the normalization tables, which the linker drops.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct NoNormalization;

impl InputNormalizer for NoNormalization {
    fn normalize<'t>(&self, text: &'t str) -> Cow<'t, str> {
        Cow::Borrowed(text)
    }
}

/// The text in Unicode's canonical composed form (NFC): a letter followed by
/// combining accents is replaced by the single accented letter wherever
/// Unicode defines one, so that `e` followed by a combining acute accent
/// becomes `é`, and a character with a composed equivalent is replaced by it,
/// so that the Angstrom sign becomes the letter `Å`.
///
/// The input comes back borrowed, with no copy, when a quick check shows that
/// it is already in composed form — the case for nearly all text — and as a
/// new string otherwise.
///
/// This is what [`NormalizeNfc`] applies. A caller that needs byte positions
/// into a text of its own normalizes it once with this function and encodes
/// the result with [`NoNormalization`].
///
/// ```
/// use untechxt::nfc;
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
