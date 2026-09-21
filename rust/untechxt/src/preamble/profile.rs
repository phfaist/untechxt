//! The [`Profile`] that one encoded value needs in the preamble, and the
//! [`PreambleNeeds`] that a whole document collects.

use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::fmt;

use crate::outbuffer::OutBuffer;
use crate::preamble::{Chunk, ChunkPreamble};
use crate::report::EncodeReporter;
use crate::BoxError;

/// The set of [`Chunk`]s that one encoded value needs in the preamble.
///
/// A rule returns a profile with the value it produces (see the method
/// [`EncodedReplacement::with_needs`]), as a plain reference to a profile the
/// rule owns. There is no registry and no shared identifier space, so tables
/// from unrelated crates cannot clash, and a rule built at run time returns a
/// reference to a profile it built itself. The same chunk identifier always
/// means the same chunk, which is how the struct [`PreambleNeeds`] combines
/// profiles of different origins.
///
/// A profile either borrows a static list of chunks or owns one. Use the
/// method [`Profile::from_static`] for a static list, which is what static
/// tables use. Use the method [`Profile::new`] to own a list built at run
/// time.
///
/// [`EncodedReplacement::with_needs`]:
///     crate::rule::EncodedReplacement::with_needs
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Profile {
    chunks: Cow<'static, [Chunk]>,
}

impl Profile {
    /// The profile that borrows the static list `chunks`, without copying it.
    /// This is how the builtin profiles, and any user's static table, state
    /// their needs.
    ///
    /// A [`Chunk`] has drop glue, so a slice literal of chunks is not promoted
    /// to a static on its own. Declare the list as a static of its own and
    /// pass a reference to it, as the example does.
    ///
    /// ```
    /// use untechxt::preamble::{Chunk, Profile};
    ///
    /// static DSFONT_CHUNKS: [Chunk; 1] = [Chunk::package("dsfont")];
    /// static DSFONT: Profile = Profile::from_static(&DSFONT_CHUNKS);
    /// assert_eq!(DSFONT.chunks().len(), 1);
    /// ```
    pub const fn from_static(chunks: &'static [Chunk]) -> Self {
        Profile { chunks: Cow::Borrowed(chunks) }
    }

    /// The profile that owns `chunks`. This is how a rule built at run time
    /// states its needs.
    pub fn new(chunks: Vec<Chunk>) -> Self {
        Profile { chunks: Cow::Owned(chunks) }
    }

    /// The chunks of the profile, in the order they were given in.
    pub fn chunks(&self) -> &[Chunk] {
        &self.chunks
    }

    /// Whether the profile has no chunks. A value that the LaTeX kernel prints
    /// by itself needs an empty profile.
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
}

/// The set of [`Chunk`]s that a whole document needs in its preamble,
/// collected as values are encoded.
///
/// The set starts empty. Add one value's [`Profile`] with the method
/// [`PreambleNeeds::include`], or another set with the method
/// [`PreambleNeeds::merge`]. Read the collected chunks with the method
/// [`PreambleNeeds::chunks`], or write the preamble directly with the method
/// [`PreambleNeeds::write_preamble`]. The set owns a copy of each distinct
/// chunk it has seen, so that it has no lifetime and can outlive the rules
/// that reported the chunks.
///
/// The struct [`PreambleNeeds`] is itself an [`EncodeReporter`], so a caller
/// that wants the preamble needs and nothing else can pass it to the method
/// [`Encoder::encode_into`](crate::Encoder::encode_into) as the report. The
/// fragments of one document then report into one set.
///
/// ```
/// use untechxt::preamble::{Chunk, PreambleNeeds, Profile};
///
/// static AMSSYMB_CHUNKS: [Chunk; 1] = [Chunk::package("amssymb")];
/// static AMSSYMB: Profile = Profile::from_static(&AMSSYMB_CHUNKS);
/// let mut needs = PreambleNeeds::new();
/// assert!(needs.is_empty());
/// needs.include(&AMSSYMB);
/// needs.include(&AMSSYMB); // the same chunk is kept once
/// let mut preamble = String::new();
/// needs.write_preamble(&mut preamble).unwrap();
/// assert_eq!(preamble, "\\usepackage{amssymb}\n");
/// ```
#[derive(Clone, Default, PartialEq, Eq)]
pub struct PreambleNeeds {
    /// The package chunks, in first-seen order.
    packages: Vec<Chunk>,
    /// The declaration chunks, in first-seen order.
    snippets: Vec<Chunk>,
}

impl PreambleNeeds {
    /// An empty set, for a document that needs nothing beyond the LaTeX
    /// kernel.
    pub fn new() -> Self {
        PreambleNeeds::default()
    }

    /// Adds every chunk of `profile` that the set does not already hold,
    /// identified by [`Chunk::id`]. Calling this again with the same profile
    /// changes nothing.
    pub fn include(&mut self, profile: &Profile) {
        for chunk in profile.chunks() {
            self.include_chunk(chunk);
        }
    }

    /// Adds every chunk of `other` that the set does not already hold. Use
    /// this to combine the needs of separately encoded fragments into one set.
    pub fn merge(&mut self, other: &PreambleNeeds) {
        for chunk in other.chunks() {
            self.include_chunk(chunk);
        }
    }

    /// Whether the set holds no chunk.
    pub fn is_empty(&self) -> bool {
        self.packages.is_empty() && self.snippets.is_empty()
    }

    /// The chunks of the set, as an iterator: every package chunk first, then
    /// every declaration chunk, each group in first-seen order.
    ///
    /// Packages come first because a declaration may call into a package of
    /// its own profile, and a preamble must load the package first.
    pub fn chunks(&self) -> impl Iterator<Item = &Chunk> + '_ {
        self.packages.iter().chain(self.snippets.iter())
    }

    /// Writes the preamble that the set describes to `out`: one `\usepackage`
    /// line per package chunk, then the declarations of each snippet chunk, in
    /// the order of the method [`PreambleNeeds::chunks`], each line ended by a
    /// newline. An empty set writes nothing.
    ///
    /// Several chunks that load the same package under different options are
    /// written as several lines. This method does not merge options. That is
    /// safe for the builtin chunks. The only package they load more than once
    /// is `fontenc`, which may be loaded repeatedly, and every builtin
    /// `fontenc` chunk lists `T1` last, so the document's default font encoding
    /// stays `T1`. A consumer with its own package machinery can read the
    /// variant [`ChunkPreamble::PackageWithOptions`] and merge the options
    /// itself.
    ///
    /// # Errors
    ///
    /// Returns whatever `out` returns when a write fails.
    pub fn write_preamble<O: OutBuffer + ?Sized>(&self, out: &mut O) -> Result<(), BoxError> {
        for chunk in self.chunks() {
            match &chunk.preamble {
                ChunkPreamble::Package(name) => {
                    out.push_str("\\usepackage{")?;
                    out.push_str(name)?;
                    out.push_str("}")?;
                }
                ChunkPreamble::PackageWithOptions(name, options) => {
                    out.push_str("\\usepackage[")?;
                    out.push_str(options)?;
                    out.push_str("]{")?;
                    out.push_str(name)?;
                    out.push_str("}")?;
                }
                ChunkPreamble::Snippet(text) => out.push_str(text)?,
            }
            out.push_str("\n")?;
        }
        Ok(())
    }

    /// Adds one chunk to the list its kind belongs in, if no chunk of that
    /// identifier is there already.
    fn include_chunk(&mut self, chunk: &Chunk) {
        // Same id means same chunk, whichever list holds it.
        if self.chunks().any(|held| held.id == chunk.id) {
            return;
        }
        let list = if chunk.is_package() { &mut self.packages } else { &mut self.snippets };
        list.push(chunk.clone());
    }
}

impl EncodeReporter for PreambleNeeds {
    fn report_needs(&mut self, profile: &Profile) {
        self.include(profile);
    }
}

impl fmt::Debug for PreambleNeeds {
    /// Formats the set as the list of its chunk identifiers, in the order of
    /// the method [`PreambleNeeds::chunks`].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.chunks().map(|chunk| &*chunk.id)).finish()
    }
}
