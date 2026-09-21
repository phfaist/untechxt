//! What a value needs in the preamble: the [`Profile`] one encoded value
//! carries, the [`ProfileIndex`] a static table stores, and the
//! [`PreambleNeeds`] a whole document accumulates.

use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::fmt;

use crate::outbuffer::OutBuffer;
use crate::preamble::{Chunk, ChunkPreamble};
use crate::report::EncodeReporter;
use crate::rule::BoxError;

/// A **profile**: the set of [`Chunk`]s that one encoded value needs in the
/// preamble.
///
/// A rule hands one out with the value it produces
/// ([`EncodedReplacement::with_needs`](crate::EncodedReplacement::with_needs)),
/// as a plain reference to a profile the rule owns. There is no registry and
/// no identifier space: tables from unrelated crates cannot clash, and a rule
/// built at run time returns a reference to a profile it built itself. Same
/// chunk identifier always means the same chunk, which is how
/// [`PreambleNeeds`] unions profiles of different origins.
///
/// A profile borrows a static chunk list ([`from_static`](Profile::from_static),
/// which static tables use) or owns one ([`new`](Profile::new)).
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Profile {
    chunks: Cow<'static, [Chunk]>,
}

impl Profile {
    /// The profile of the chunks of a static list, without a copy. This is
    /// how the builtin profiles and any user's static table state their
    /// needs.
    ///
    /// A [`Chunk`] has drop glue, so a slice literal of chunks is not
    /// promoted to a static on its own: give the list a static of its own and
    /// point at it.
    ///
    /// ```
    /// use untechxt::{Chunk, Profile};
    ///
    /// static DSFONT_CHUNKS: [Chunk; 1] = [Chunk::package("dsfont")];
    /// static DSFONT: Profile = Profile::from_static(&DSFONT_CHUNKS);
    /// assert_eq!(DSFONT.chunks().len(), 1);
    /// ```
    pub const fn from_static(chunks: &'static [Chunk]) -> Self {
        Profile { chunks: Cow::Borrowed(chunks) }
    }

    /// The profile of the chunks of `chunks`, which it takes over. This is
    /// how a rule built at run time states its needs.
    pub fn new(chunks: Vec<Chunk>) -> Self {
        Profile { chunks: Cow::Owned(chunks) }
    }

    /// The chunks of the profile, in the order they were given in.
    pub fn chunks(&self) -> &[Chunk] {
        &self.chunks
    }

    /// Whether the profile asks for nothing — the profile of a value the
    /// LaTeX kernel prints by itself.
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
}

/// The number of a profile in a table's own profile array.
///
/// A static table stores one byte per entry rather than a profile, and turns
/// it into a `&'static Profile` on lookup. The index is meaningful only
/// together with the array it indexes; index 0 is reserved for "needs
/// nothing", so that a lookup answers `None` for it without consulting the
/// array.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct ProfileIndex(pub u8);

impl ProfileIndex {
    /// The reserved index of the profile that needs nothing: 0.
    pub const NONE: ProfileIndex = ProfileIndex(0);
}

/// What a document must have in its preamble because of the LaTeX written
/// into it: a set of [`Chunk`]s, accumulated as values are encoded.
///
/// The set starts empty ([`new`](PreambleNeeds::new)), grows by one value's
/// [`Profile`] ([`include`](PreambleNeeds::include)) or by another set
/// ([`merge`](PreambleNeeds::merge)), and is read as its chunks
/// ([`chunks`](PreambleNeeds::chunks)) or written out directly
/// ([`write_preamble`](PreambleNeeds::write_preamble)). It owns a copy of
/// each distinct chunk it has seen, so that it carries no lifetime and can
/// outlive the rules that reported them.
///
/// It is itself an [`EncodeReporter`], and so it can be passed to
/// [`Encoder::encode_into`](crate::Encoder::encode_into) as the report of a
/// caller that wants the needs alone.
///
/// ```
/// use untechxt::{Chunk, PreambleNeeds, Profile};
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
    /// The empty set: a document needing nothing beyond the LaTeX kernel.
    pub fn new() -> Self {
        PreambleNeeds::default()
    }

    /// Adds every chunk of `profile` that the set does not hold yet, keyed by
    /// [`Chunk::id`](Chunk::id). Calling this again with the same profile
    /// changes nothing.
    pub fn include(&mut self, profile: &Profile) {
        for chunk in profile.chunks() {
            self.include_chunk(chunk);
        }
    }

    /// Adds every chunk of `other` that the set does not hold yet.
    pub fn merge(&mut self, other: &PreambleNeeds) {
        for chunk in other.chunks() {
            self.include_chunk(chunk);
        }
    }

    /// Whether the set holds no chunk at all.
    pub fn is_empty(&self) -> bool {
        self.packages.is_empty() && self.snippets.is_empty()
    }

    /// The chunks of the set: every package chunk first, then every
    /// declaration chunk, each group in first-seen order.
    ///
    /// Packages come first because a declaration may call into a package of
    /// its own profile, and a preamble must load it first.
    pub fn chunks(&self) -> impl Iterator<Item = &Chunk> + '_ {
        self.packages.iter().chain(self.snippets.iter())
    }

    /// Writes the preamble the set asks for: one `\usepackage` line per
    /// package chunk, then the declarations of each snippet chunk, in the
    /// order of [`chunks`](PreambleNeeds::chunks), each ended by a newline.
    /// Nothing at all for an empty set.
    ///
    /// Several chunks that load the same package under different options are
    /// written as several lines: this does **not** merge options. That is
    /// safe for the builtin chunks — the only package they load more than
    /// once is `fontenc`, which may be loaded repeatedly, and every builtin
    /// `fontenc` chunk names `T1` last, so the document's default encoding
    /// stays `T1`. A consumer with its own package machinery reads
    /// [`ChunkPreamble::PackageWithOptions`] and merges as it sees fit.
    ///
    /// # Errors
    ///
    /// Whatever `out` reports.
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
    /// The chunk identifiers of the set, in the order of
    /// [`chunks`](PreambleNeeds::chunks).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.chunks().map(|chunk| &*chunk.id)).finish()
    }
}
