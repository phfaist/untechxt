//! The [`Profile`] that one encoded value needs in the preamble, and the
//! [`PreambleNeeds`] that a whole document collects.

use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::fmt;

use crate::outbuffer::OutBuffer;
use crate::preamble::{Chunk, ChunkPreamble, Engine};
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
/// [`PreambleNeeds::merge`]. The set owns a copy of each distinct chunk it has
/// seen, so that it has no lifetime and can outlive the rules that reported
/// the chunks.
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
///
/// # Choosing the LaTeX engine
///
/// A chunk can be a different piece of preamble under different LaTeX engines
/// (see the struct [`Chunk`]). The set itself does not depend on an engine,
/// because it holds each chunk with all of its cases. The engine is chosen
/// when the set is read, in one of the following ways.
///
/// - The method [`PreambleNeeds::write_preamble`] takes no engine and writes
///   a preamble that compiles under every engine. Use it when you do not
///   know which engine will compile the document.
/// - The method [`PreambleNeeds::write_preamble_for`] takes an [`Engine`] and
///   writes the preamble for that engine alone.
/// - The method [`PreambleNeeds::preambles_for`] takes an [`Engine`] and
///   returns the pieces of preamble for that engine. Use it in a program that
///   assembles the preamble itself.
///
/// The builtin chunks for the Cyrillic letters are an example. They load the
/// font encoding `T2A` beside `T1` under pdfLaTeX, and beside `TU` under
/// LuaLaTeX and XeLaTeX:
///
/// ```
/// use untechxt::preamble::{Engine, PreambleNeeds};
/// use untechxt::{default_rules, Encoder};
///
/// let encoder = Encoder::new(default_rules());
/// let mut body = String::new();
/// let mut needs = PreambleNeeds::new();
/// encoder.encode_into("я", &mut body, &mut needs).unwrap();
///
/// let mut for_lualatex = String::new();
/// needs.write_preamble_for(Engine::LuaLatex, &mut for_lualatex).unwrap();
/// assert_eq!(for_lualatex, "\\usepackage[T2A,TU]{fontenc}\n");
///
/// let mut for_every_engine = String::new();
/// needs.write_preamble(&mut for_every_engine).unwrap();
/// assert_eq!(for_every_engine, concat!(
///     "\\usepackage{iftex}\n",
///     "\\iftutex\n",
///     "\\usepackage[T2A,TU]{fontenc}\n",
///     "\\else\n",
///     "\\usepackage[T2A,T1]{fontenc}\n",
///     "\\fi\n",
/// ));
/// ```
#[derive(Clone, Default, PartialEq, Eq)]
pub struct PreambleNeeds {
    /// The chunks, in first-seen order.
    chunks: Vec<Chunk>,
}

impl PreambleNeeds {
    /// An empty set, for a document that needs nothing beyond the LaTeX
    /// kernel.
    pub fn new() -> Self {
        PreambleNeeds::default()
    }

    /// Adds every chunk of `profile` that the set does not already hold,
    /// identified by the method [`Chunk::id`]. Calling this again with the
    /// same profile changes nothing.
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

    /// Whether the set holds no chunk. A set that holds chunks can still need
    /// nothing under one particular engine, when none of its chunks has a
    /// case that applies to that engine.
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    /// The chunks of the set, as an iterator, in first-seen order.
    ///
    /// This order is not the order that a preamble is written in, because
    /// whether a chunk is a package or a block of declarations can depend on
    /// the engine. Use the method [`PreambleNeeds::preambles_for`] for the
    /// pieces of preamble of one engine in the order to write them in.
    pub fn chunks(&self) -> impl Iterator<Item = &Chunk> + '_ {
        self.chunks.iter()
    }

    /// What the set needs under `engine`, as an iterator over each chunk that
    /// needs something under `engine` together with what that chunk is under
    /// `engine`. A chunk that needs nothing under `engine` is left out.
    ///
    /// Every package comes first, then every block of declarations, each
    /// group in first-seen order. Packages come first because a declaration
    /// may call into a package of its own profile, and a preamble must load
    /// the package first.
    ///
    /// ```
    /// use untechxt::preamble::{Engine, PreambleNeeds};
    /// use untechxt::{default_rules, Encoder};
    ///
    /// let encoder = Encoder::new(default_rules());
    /// let mut body = String::new();
    /// let mut needs = PreambleNeeds::new();
    /// encoder.encode_into("ą", &mut body, &mut needs).unwrap();
    ///
    /// // The ogonek accent needs the `T1` font encoding under pdfLaTeX, and
    /// // nothing under LuaLaTeX, which defines the accent itself.
    /// let ids = |engine| -> Vec<&str> {
    ///     needs.preambles_for(engine).map(|(chunk, _)| chunk.id()).collect()
    /// };
    /// assert_eq!(ids(Engine::PdfLatex), ["fontenc-t1"]);
    /// assert!(ids(Engine::LuaLatex).is_empty());
    /// ```
    pub fn preambles_for(
        &self,
        engine: Engine,
    ) -> impl Iterator<Item = (&Chunk, &ChunkPreamble)> + '_ {
        let of_kind = move |packages: bool| {
            self.chunks
                .iter()
                .filter_map(move |chunk| Some((chunk, chunk.preamble_for(engine)?)))
                .filter(move |(_, preamble)| preamble.is_package() == packages)
        };
        of_kind(true).chain(of_kind(false))
    }

    /// Writes to `out` a preamble that compiles under every LaTeX engine: one
    /// `\usepackage` line per package, then the declarations of each snippet,
    /// each line ended by a newline. An empty set writes nothing.
    ///
    /// A chunk that is the same under every engine is written as it is. A
    /// chunk that differs between engines is written inside a test of the
    /// package `iftex`, such as `\iftutex … \else … \fi`, so that each engine
    /// reads its own case. The preamble then starts with the line
    /// `\usepackage{iftex}`. Neighboring chunks that differ in the same way
    /// share one test. An engine that this crate does not know reads what
    /// pdfLaTeX reads. A set in which no chunk differs between engines is
    /// written with no test and without the package `iftex`.
    ///
    /// The lines are in the order of the method
    /// [`PreambleNeeds::preambles_for`]: packages first, then declarations.
    ///
    /// Several chunks that load the same package under different options are
    /// written as several lines. This method does not merge options. That is
    /// safe for the builtin chunks. The only package they load more than once
    /// is `fontenc`, which may be loaded repeatedly, and every builtin
    /// `fontenc` chunk lists the encoding of the document last, so that the
    /// encoding of the document stays `T1` under pdfLaTeX and `TU` under
    /// LuaLaTeX and XeLaTeX. A consumer with its own package machinery can
    /// read the variant [`ChunkPreamble::PackageWithOptions`] from the method
    /// [`PreambleNeeds::preambles_for`] and merge the options itself.
    ///
    /// TeX skips the cases of the other engines without running them. A
    /// custom snippet that applies to some engines alone must therefore not
    /// declare a conditional with `\newif` and use that conditional in the
    /// same snippet, because TeX cannot skip such text correctly. If you need
    /// such a snippet, write the preamble for one engine with the method
    /// [`PreambleNeeds::write_preamble_for`] instead.
    ///
    /// # Errors
    ///
    /// Returns whatever `out` returns when a write fails.
    pub fn write_preamble<O: OutBuffer + ?Sized>(&self, out: &mut O) -> Result<(), BoxError> {
        let resolved: Vec<Resolved<'_>> = self
            .chunks
            .iter()
            .map(|chunk| Engine::KNOWN.map(|engine| chunk.preamble_for(engine)))
            .collect();
        if resolved.iter().any(|chunk| Split::of(chunk) != Split::Same) {
            out.push_str("\\usepackage{iftex}\n")?;
        }
        for packages in [true, false] {
            // What each chunk writes in this group, which is nothing for a
            // chunk that is of the other kind under every engine.
            let group: Vec<Resolved<'_>> = resolved
                .iter()
                .map(|chunk| chunk.map(|p| p.filter(|p| p.is_package() == packages)))
                .filter(|chunk| chunk.iter().any(Option::is_some))
                .collect();
            for run in group.chunk_by(|a, b| Split::of(a) == Split::of(b)) {
                match Split::of(&run[0]) {
                    Split::Same => write_branch(out, run, PDFLATEX)?,
                    Split::Test(test, engine) => {
                        write_conditional(out, run, &[(test, engine)])?;
                    }
                    Split::Each => write_conditional(
                        out,
                        run,
                        &[("\\ifluatex", LUALATEX), ("\\ifxetex", XELATEX)],
                    )?,
                }
            }
        }
        Ok(())
    }

    /// Writes to `out` the preamble that the set needs under `engine`: one
    /// `\usepackage` line per package, then the declarations of each snippet,
    /// in the order of the method [`PreambleNeeds::preambles_for`], each line
    /// ended by a newline. Nothing is written for a chunk that needs nothing
    /// under `engine`, and the preamble contains no test of the engine.
    ///
    /// This method does not merge the options of several chunks that load the
    /// same package. See the method [`PreambleNeeds::write_preamble`].
    ///
    /// # Errors
    ///
    /// Returns whatever `out` returns when a write fails.
    pub fn write_preamble_for<O: OutBuffer + ?Sized>(
        &self,
        engine: Engine,
        out: &mut O,
    ) -> Result<(), BoxError> {
        for (_, preamble) in self.preambles_for(engine) {
            write_one(out, preamble)?;
        }
        Ok(())
    }

    /// Adds one chunk, if no chunk of that identifier is there already.
    fn include_chunk(&mut self, chunk: &Chunk) {
        // Same id means same chunk.
        if !self.chunks.iter().any(|held| held.id() == chunk.id()) {
            self.chunks.push(chunk.clone());
        }
    }
}

/// What one chunk is under each engine of [`Engine::KNOWN`], in that order.
type Resolved<'a> = [Option<&'a ChunkPreamble>; 3];

/// The position of pdfLaTeX in a [`Resolved`].
const PDFLATEX: usize = 0;
/// The position of LuaLaTeX in a [`Resolved`].
const LUALATEX: usize = 1;
/// The position of XeLaTeX in a [`Resolved`].
const XELATEX: usize = 2;

/// How the known engines divide over what one chunk is under each of them,
/// which decides the test that the chunk is written inside.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Split {
    /// The chunk is the same under every engine, and needs no test.
    Same,
    /// One test of the package `iftex` is enough. The engines for which the
    /// test is true share what the chunk is under the engine at the given
    /// position of a [`Resolved`], and every other engine shares what the
    /// chunk is under pdfLaTeX.
    Test(&'static str, usize),
    /// The chunk is different under each engine.
    Each,
}

impl Split {
    /// The split of one chunk.
    fn of(chunk: &Resolved<'_>) -> Split {
        let [pdflatex, lualatex, xelatex] = chunk;
        match (pdflatex == lualatex, lualatex == xelatex, pdflatex == xelatex) {
            (true, true, _) => Split::Same,
            (false, true, _) => Split::Test("\\iftutex", LUALATEX),
            (true, false, _) => Split::Test("\\ifxetex", XELATEX),
            (false, false, true) => Split::Test("\\ifluatex", LUALATEX),
            (false, false, false) => Split::Each,
        }
    }
}

/// Writes the chunks of `run` inside the tests of `tests`, each of which is a
/// test of the package `iftex` and the position in a [`Resolved`] of what to
/// write where the test is true. What the chunks are under pdfLaTeX is written
/// where every test is false, so that an engine this crate does not know reads
/// what pdfLaTeX reads.
fn write_conditional<O: OutBuffer + ?Sized>(
    out: &mut O,
    run: &[Resolved<'_>],
    tests: &[(&str, usize)],
) -> Result<(), BoxError> {
    for (number, (test, engine)) in tests.iter().enumerate() {
        if number > 0 {
            out.push_str("\\else")?;
        }
        out.push_str(test)?;
        out.push_str("\n")?;
        write_branch(out, run, *engine)?;
    }
    if run.iter().any(|chunk| chunk[PDFLATEX].is_some()) {
        out.push_str("\\else\n")?;
        write_branch(out, run, PDFLATEX)?;
    }
    for _ in tests {
        out.push_str("\\fi")?;
    }
    out.push_str("\n")
}

/// Writes what each chunk of `run` is under the engine at the position
/// `engine` of a [`Resolved`].
fn write_branch<O: OutBuffer + ?Sized>(
    out: &mut O,
    run: &[Resolved<'_>],
    engine: usize,
) -> Result<(), BoxError> {
    for preamble in run.iter().filter_map(|chunk| chunk[engine]) {
        write_one(out, preamble)?;
    }
    Ok(())
}

/// Writes one piece of preamble and the newline that ends it.
fn write_one<O: OutBuffer + ?Sized>(
    out: &mut O,
    preamble: &ChunkPreamble,
) -> Result<(), BoxError> {
    match preamble {
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
    out.push_str("\n")
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
        f.debug_list().entries(self.chunks().map(Chunk::id)).finish()
    }
}
