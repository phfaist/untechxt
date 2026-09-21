//! What an encoding reports beyond the LaTeX text: what the output needs in
//! the document's preamble, and which characters no rule matched.
//!
//! An encoder can report more than the encoded text. The [`EncodeReporter`]
//! trait is what the encoder reports through: the encoder calls a reporter's
//! methods as it works, once for each value that needs something in the
//! preamble and once for each character no rule matched. A reporter is passed
//! in rather than returned, because one preamble is usually built from many
//! fragments of encoded text.
//!
//! The crate offers these reporters:
//!
//! - The [`EncodeReport`] struct keeps both the preamble needs and the
//!   unknown characters.
//! - The [`PreambleNeeds`] struct keeps the preamble needs alone.
//! - The [`NoReport`] struct keeps nothing. Under [`NoReport`] and a static
//!   rule chain the compiler can remove the whole reporting machinery from
//!   the encoding loop.

use alloc::collections::BTreeSet;

use crate::preamble::{PreambleNeeds, Profile};

/// The trait the encoder reports through as it works: it is given each value's
/// preamble needs and each character that no rule matched.
///
/// Both methods do nothing by default, so an implementation overrides only the
/// one it cares about. The encoder calls them as it encodes. A reporter is
/// passed in rather than returned, because one preamble is usually built from
/// many fragments of text.
///
/// The [`report_needs`](EncodeReporter::report_needs) method may be called
/// more than once with the same profile, because the encoder skips only
/// immediately repeated ones, so an implementation of it must be idempotent.
pub trait EncodeReporter {
    /// Reports that a value needing `profile` in the preamble was written.
    ///
    /// The encoder may call this method any number of times with the same
    /// profile, so an implementation must make repeated calls with one profile
    /// have the same effect as a single call.
    fn report_needs(&mut self, profile: &Profile) {
        let _ = profile;
    }

    /// Reports that a character no rule matched was encountered at byte
    /// `position` of the normalized input, where the
    /// [`UnknownCharPolicy`](crate::UnknownCharPolicy) was then applied to it.
    ///
    /// The encoder calls this method whatever the policy is, before the policy
    /// writes anything and before
    /// [`EncodeError::UnknownChar`](crate::EncodeError::UnknownChar) is
    /// returned.
    fn report_unknown_char(&mut self, ch: char, position: usize) {
        let _ = (ch, position);
    }
}

/// The [`EncodeReporter`] that keeps nothing. Under [`NoReport`] and a static
/// rule chain the compiler can remove the whole reporting machinery from the
/// encoding loop.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct NoReport;

impl EncodeReporter for NoReport {}

/// What one or more encodings produced beyond the LaTeX text: what a document
/// holding the text must load in its preamble, and which characters no rule
/// matched.
///
/// The [`Encoder::encode_with_report`](crate::Encoder::encode_with_report)
/// method returns one of these along with the encoded text, and
/// [`Encoder::encode_into`](crate::Encoder::encode_into) fills one passed to
/// it. An [`EncodeReport`] is itself an [`EncodeReporter`], so the fragments
/// of one document can all report into the same one.
///
/// Neither how often an unknown character occurred nor where it occurred is
/// kept. A caller that needs either one implements [`EncodeReporter`] itself.
///
/// ```
/// use untechxt::{default_rules, Encoder};
///
/// let encoder = Encoder::new(default_rules());
///
/// // `\mathds{1}` needs the package `dsfont` and `\nicefrac{1}{3}` the
/// // package `nicefrac`; the report collected both.
/// let (body, report) = encoder.encode_with_report("𝟙 and ⅓").unwrap();
/// assert_eq!(body, r"\ensuremath{\mathds{1}} and \nicefrac{1}{3}");
/// let mut preamble = String::new();
/// report.needs.write_preamble(&mut preamble).unwrap();
/// assert_eq!(preamble, "\\usepackage{dsfont}\n\\usepackage{nicefrac}\n");
///
/// // The Thai letter is in no builtin entry: it is kept, and reported.
/// let (_, report) = encoder.encode_with_report("ธ").unwrap();
/// assert!(report.unknown_chars.contains(&'ธ'));
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EncodeReport {
    /// What a document holding the encoded text needs in its preamble. See the
    /// struct [`PreambleNeeds`].
    pub needs: PreambleNeeds,
    /// The distinct characters that no rule matched, in code point order.
    pub unknown_chars: BTreeSet<char>,
}

impl EncodeReport {
    /// An empty report.
    pub fn new() -> Self {
        EncodeReport::default()
    }
}

impl EncodeReporter for EncodeReport {
    fn report_needs(&mut self, profile: &Profile) {
        self.needs.include(profile);
    }

    fn report_unknown_char(&mut self, ch: char, position: usize) {
        let _ = position;
        self.unknown_chars.insert(ch);
    }
}
