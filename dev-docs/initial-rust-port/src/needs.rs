//! What a spelling needs in a document's preamble: the chunk, the profile and
//! the set. The data these types describe lives in [`super::tables`]; the
//! types are re-exported from the crate root, which is where their
//! documentation is read.

use std::fmt;

use super::tables::{CHUNKS, PROFILES};

/// One **preamble chunk**: a piece of preamble — a package to load, or
/// declarations to make — that some spellings of the Unicode-to-LaTeX table
/// need.
///
/// A spelling such as `\ensuremath{\mathds{1}}` prints nothing on its own: the
/// command `\mathds` exists only once the document has loaded the package
/// `dsfont`. Some spellings need something no package provides —
/// `\ensuremath{\flmBbold{0}}` needs a math alphabet declared from a font
/// family, because the package that has that font would redefine `\mathbb`
/// for the whole document — and for those the chunk is the declaration
/// itself ([`ChunkPreamble`]). Either way a chunk is named by a short
/// identifier, so that a program can recognize it without parsing its text.
/// The chunks are the static table [`CHUNKS`](super::tables::CHUNKS); a
/// spelling names a set of them through its [`Profile`], and a document
/// accumulates the sets of the spellings it wrote into a [`PreambleNeeds`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Chunk {
    /// The chunk's identifier: a short name, unique in
    /// [`CHUNKS`](super::tables::CHUNKS), which is the package's own name
    /// (`amssymb`) except where one package is loaded with different options
    /// under different identifiers (`fontenc-t2a`), or where the chunk is
    /// not a package at all (`bbold-alphabet`).
    pub id: &'static str,
    /// What the document's preamble must hold: a package to load, or the
    /// declarations to make.
    pub preamble: ChunkPreamble,
    /// One sentence saying what the chunk provides and which spellings need
    /// it.
    pub docs: &'static str,
}

impl Chunk {
    /// The chunk's LaTeX, whichever kind it is: the `\usepackage` line, or
    /// the declarations. No newline at the end of a one-line chunk; a
    /// several-line chunk ends its last line without one too.
    pub fn latex(&self) -> &'static str {
        match self.preamble {
            ChunkPreamble::Package(line) => line,
            ChunkPreamble::Snippet(latex) => latex,
        }
    }
}

/// What a [`Chunk`] asks a preamble for: a package, or declarations of its
/// own.
///
/// The two are told apart because a program that assembles a preamble treats
/// them differently — a package is requested once however many things ask
/// for it, declarations are written where the packages they call into are
/// already loaded. The LaTeX backends resolve a `Package` chunk to a package
/// request and a `Snippet` chunk to a definition piece of the style file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkPreamble {
    /// One LaTeX package with the options it is loaded with, as the line that
    /// loads it: `\usepackage{amssymb}`, `\usepackage[T2A,T1]{fontenc}`.
    Package(&'static str),
    /// Declarations no package makes, as the lines that make them: a math
    /// alphabet (`\DeclareMathAlphabet{\flmBbold}{U}{bbold}{m}{n}`), a symbol
    /// font with the symbols read from it. Every command such a chunk defines
    /// is named `\flm…`, which the LaTeX backends reserve.
    Snippet(&'static str),
}

/// A **profile**: the set of [`Chunk`]s one spelling of the Unicode-to-LaTeX
/// table needs, held as one byte.
///
/// Every entry of the table carries a profile, so that a program that writes
/// a spelling learns what the document must load without looking at the
/// spelling's text. The byte is an index into the static table
/// [`PROFILES`](super::tables::PROFILES), which lists the chunks of each
/// profile; the value itself is not meaningful outside this module, and the
/// only profile with a name is [`Profile::BUILTINS`]. Add a profile to a
/// [`PreambleNeeds`] with [`PreambleNeeds::include`] to learn its chunks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Profile(u8);

impl Profile {
    /// The profile of a spelling that needs nothing beyond the LaTeX kernel —
    /// the commands every document has without loading a package. It is the
    /// profile of most of the table's entries, and the default.
    pub const BUILTINS: Profile = Profile(0);

    /// The profile numbered `index` in [`PROFILES`](super::tables::PROFILES).
    /// The table names its profiles with constants built this way.
    pub(super) const fn at(index: u8) -> Profile {
        Profile(index)
    }

    /// The chunk indices of this profile, or nothing when the index is not a
    /// profile of the table — which the table's structural test excludes.
    fn chunk_indices(self) -> &'static [usize] {
        PROFILES.get(self.0 as usize).copied().unwrap_or(&[])
    }
}

/// What a document must have in its preamble because of the LaTeX spellings
/// written into it: a set of [`Chunk`]s, accumulated as spellings are written.
///
/// The set starts empty ([`new`](PreambleNeeds::new)), grows by one spelling's
/// [`Profile`] ([`include`](PreambleNeeds::include)) or by another set
/// ([`include_all`](PreambleNeeds::include_all)), and is read either as its
/// chunks ([`chunks`](PreambleNeeds::chunks), in the order of
/// [`CHUNKS`](super::tables::CHUNKS)) or directly as the `\usepackage` lines
/// they stand for ([`preamble`](PreambleNeeds::preamble)). How the set is held
/// is not part of what it is: a caller reads it only through these methods, so
/// that chunks can be added to the table without changing anything a caller
/// wrote.
///
/// The encoder returns one for every string it encodes
/// ([`Encoder::encode_into`](super::Encoder::encode_into)); a caller that
/// writes characters through itself builds one from the profiles
/// [`spelling_of`](super::spelling_of) answers.
///
/// ```
/// use flm_latexencode::{PreambleNeeds, spelling_of};
///
/// let mut needs = PreambleNeeds::new();
/// assert!(needs.is_empty());
/// needs.include(spelling_of('𝟙').unwrap().needs);
/// assert_eq!(needs.preamble(), "\\usepackage{dsfont}\n");
/// // A spelling the LaTeX kernel can print adds nothing.
/// needs.include(spelling_of('é').unwrap().needs);
/// assert_eq!(needs.chunks().map(|c| c.id).collect::<Vec<_>>(), ["dsfont"]);
/// ```
#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct PreambleNeeds {
    /// One bit per chunk of `CHUNKS`, by index. Never exposed: the set is read
    /// through its methods alone.
    mask: u64,
}

impl PreambleNeeds {
    /// The empty set: a document needing nothing beyond the LaTeX kernel.
    pub fn new() -> PreambleNeeds {
        PreambleNeeds::default()
    }

    /// Adds the chunks of `profile`, which is a spelling's
    /// [`needs`](super::Spelling::needs).
    pub fn include(&mut self, profile: Profile) {
        for &index in profile.chunk_indices() {
            // An index outside the table contributes nothing; the table's
            // structural test forbids one.
            self.mask |= 1u64.checked_shl(index as u32).unwrap_or(0);
        }
    }

    /// Adds every chunk of `other`.
    pub fn include_all(&mut self, other: &PreambleNeeds) {
        self.mask |= other.mask;
    }

    /// Whether the set holds no chunk at all — the case for a document whose
    /// spellings the LaTeX kernel prints by itself.
    pub fn is_empty(&self) -> bool {
        self.mask == 0
    }

    /// The chunks of the set, in the order of
    /// [`CHUNKS`](super::tables::CHUNKS), which is the order a preamble loads
    /// them in.
    pub fn chunks(&self) -> impl Iterator<Item = &'static Chunk> + '_ {
        CHUNKS
            .iter()
            .enumerate()
            .filter(|(index, _)| self.mask & 1u64.checked_shl(*index as u32).unwrap_or(0) != 0)
            .map(|(_, chunk)| chunk)
    }

    /// The preamble the set asks for: the LaTeX of its chunks
    /// ([`Chunk::latex`]) in the order of [`chunks`](PreambleNeeds::chunks),
    /// each ended by a newline; the empty string when the set is empty.
    ///
    /// The lines are in chunk-table order, which loads every package before
    /// any declaration that calls into one.
    ///
    /// ```
    /// use flm_latexencode::{Encoder, PreambleNeeds};
    ///
    /// let mut out = String::new();
    /// let report = Encoder::default().encode_into(&mut out, "я ⅓").unwrap();
    /// assert_eq!(
    ///     report.needs.preamble(),
    ///     "\\usepackage{nicefrac}\n\\usepackage[T2A,T1]{fontenc}\n"
    /// );
    /// ```
    pub fn preamble(&self) -> String {
        let mut out = String::new();
        for chunk in self.chunks() {
            out.push_str(chunk.latex());
            out.push('\n');
        }
        out
    }
}

impl fmt::Debug for PreambleNeeds {
    /// The chunk identifiers of the set, in their table order.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.chunks().map(|chunk| chunk.id)).finish()
    }
}
