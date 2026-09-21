//! [`EncodeReporter`]: what the encoder tells the caller besides the LaTeX
//! itself, and the three implementations the crate offers.

use alloc::collections::BTreeSet;

use crate::profile::{PreambleNeeds, Profile};

/// What an encoding reports beyond the text it writes: what the output needs
/// in the document's preamble, and which characters no rule knew.
///
/// Both methods do nothing by default, so an implementation states only what
/// it is interested in. The encoder calls them as it goes; a report is passed
/// in rather than returned, because the usual workload is many fragments of
/// text feeding one preamble.
///
/// [`report_needs`](EncodeReporter::report_needs) may be called several times
/// for the same profile — the encoder only skips immediately repeated ones —
/// so an implementation must be idempotent.
pub trait EncodeReporter {
    /// A value that needs `profile` in the preamble was written.
    ///
    /// May be called any number of times with the same profile; the
    /// implementation makes sure that changes nothing.
    fn report_needs(&mut self, profile: &Profile) {
        let _ = profile;
    }

    /// A character that no rule knew was met at byte `position` of the
    /// normalized input, and the
    /// [`UnknownCharPolicy`](crate::UnknownCharPolicy) was applied to it.
    ///
    /// This is reported whatever the policy says, before the policy writes
    /// anything and before
    /// [`EncodeError::UnknownChar`](crate::EncodeError::UnknownChar) is
    /// raised.
    fn report_unknown_char(&mut self, ch: char, position: usize) {
        let _ = (ch, position);
    }
}

/// The reporter that keeps nothing. With a static rule chain the compiler can
/// then remove the whole needs machinery from the encoding loop.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct NoReport;

impl EncodeReporter for NoReport {}

/// What one or more encodings produced beyond the LaTeX text itself: what a
/// document holding the text must load, and which characters no rule knew.
///
/// Neither how often an unknown character occurred nor where it occurred is
/// kept; a caller who needs that implements [`EncodeReporter`] itself.
///
/// ```
/// use untechxt::EncodeReport;
///
/// let mut report = EncodeReport::new();
/// assert!(report.needs.is_empty() && report.unknown_chars.is_empty());
/// # let _ = &mut report;
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EncodeReport {
    /// What a document holding the encoded text must have in its preamble.
    pub needs: PreambleNeeds,
    /// The distinct characters that no rule knew, in code point order.
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
