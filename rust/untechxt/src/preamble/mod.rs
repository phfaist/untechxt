//! Types that describe what the encoded LaTeX needs in the document preamble.
//!
//! LaTeX code such as `\mathds{1}` prints nothing unless the document loads
//! the package `dsfont`. The encoder tracks such requirements so that the
//! caller can build a preamble that defines every command the encoded text
//! uses. This module contains the types that describe those requirements.
//!
//! - The struct [`Chunk`] is one piece of preamble under a stable identifier.
//!   A chunk is either a package to load or a block of declarations (see the
//!   enum [`ChunkPreamble`]).
//! - A chunk can be a different piece of preamble under different LaTeX
//!   engines, because pdfLaTeX, LuaLaTeX and XeLaTeX handle fonts
//!   differently. The enum [`Engine`] names one engine, the struct
//!   [`EngineSet`] is a set of engines, and the struct [`ChunkCase`] is what
//!   a chunk is under one set of engines.
//! - The struct [`Profile`] is the set of chunks that one encoded value needs.
//!   A rule returns a profile together with the value it produces.
//! - The struct [`PreambleNeeds`] collects the chunks that a whole document
//!   needs, and writes the corresponding preamble, either for one engine or
//!   in a form that compiles under every engine.

mod engine;
mod profile;

use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::hash::{Hash, Hasher};

pub use self::engine::{Engine, EngineSet};
pub use self::profile::{PreambleNeeds, Profile};

/// One piece of preamble that some encoded values need: a package to load, or
/// a block of declarations to make.
///
/// A value such as `\mathds{1}` prints nothing on its own, because the command
/// `\mathds` exists only once the document has loaded the package `dsfont`.
/// For such a value the chunk is the package to load. Some values need
/// something that no package provides. The value `\UnxTBbold{0}` needs a math
/// alphabet declared from a font family, because the package that provides
/// that font would redefine `\mathbb` for the whole document. For such a value
/// the chunk is the declaration itself (see the variant
/// [`ChunkPreamble::Snippet`]).
///
/// Either way a chunk has a short identifier, so that a program can recognize
/// it without parsing its text. The same identifier always means the same
/// chunk, which is how the struct [`PreambleNeeds`] keeps one copy of each. A
/// rule states the chunks of one value through a [`Profile`].
///
/// # Chunks that depend on the LaTeX engine
///
/// Most chunks are the same piece of preamble under every LaTeX engine. Create
/// such a chunk with the method [`Chunk::package`], the method
/// [`Chunk::package_with_options`] or the method [`Chunk::snippet`].
///
/// Some chunks must differ between engines. For example, the Cyrillic letters
/// need the font encoding `T2A`, which the package `fontenc` loads. The
/// package `fontenc` makes the last encoding of its option list the encoding
/// of the whole document. Under pdfLaTeX that last encoding should be `T1`.
/// Under LuaLaTeX and XeLaTeX it must be `TU`, because a document in the
/// encoding `T1` loses the Unicode characters that are typed directly into it.
///
/// A chunk therefore holds a list of cases (see the struct [`ChunkCase`]).
/// Each case is one piece of preamble together with the set of engines that
/// the case applies to. The method [`Chunk::preamble_for`] returns the
/// preamble of the first case that applies to a given engine. The order of
/// the cases therefore matters: write the specific cases first and a case
/// made with the method [`ChunkCase::otherwise`] last. The last case is
/// optional. When no case applies to an engine, the chunk needs nothing under
/// that engine, and nothing is written for the chunk.
///
/// There are three ways to create a chunk that depends on the engine:
///
/// - The method [`Chunk::for_engines`] takes one preamble and the set of
///   engines that need it. Use it for a chunk that is needed under some
///   engines and not at all under the others.
/// - The method [`Chunk::from_static`] takes a static list of cases. Use it
///   for static data with two or more cases.
/// - The method [`Chunk::new`] takes an owned list of cases. Use it for a
///   chunk that is built at run time.
///
/// ```
/// use untechxt::preamble::{
///     Chunk, ChunkCase, ChunkPreamble, Engine, EngineSet,
/// };
///
/// static FONTENC_T2A_CASES: [ChunkCase; 2] = [
///     ChunkCase::for_engines(
///         EngineSet::UNICODE,
///         ChunkPreamble::package_with_options("fontenc", "T2A,TU"),
///     ),
///     ChunkCase::otherwise(
///         ChunkPreamble::package_with_options("fontenc", "T2A,T1"),
///     ),
/// ];
/// const FONTENC_T2A: Chunk =
///     Chunk::from_static("fontenc-t2a", &FONTENC_T2A_CASES);
///
/// assert_eq!(
///     FONTENC_T2A.preamble_for(Engine::LuaLatex),
///     Some(&ChunkPreamble::package_with_options("fontenc", "T2A,TU")),
/// );
/// assert_eq!(
///     FONTENC_T2A.preamble_for(Engine::PdfLatex),
///     Some(&ChunkPreamble::package_with_options("fontenc", "T2A,T1")),
/// );
/// ```
#[derive(Debug, Clone)]
pub struct Chunk {
    /// The identifier of the chunk. See the method [`Chunk::id`].
    id: Cow<'static, str>,
    /// The cases of the chunk. See the method [`Chunk::cases`].
    cases: Cases,
}

/// The cases of a [`Chunk`]. A single case is held in the chunk itself, so
/// that a `const` constructor can create the chunk from its arguments without
/// a static list to borrow.
#[derive(Debug, Clone)]
enum Cases {
    /// One case.
    One(ChunkCase),
    /// A list of cases, borrowed from a static or owned.
    Many(Cow<'static, [ChunkCase]>),
}

impl Chunk {
    /// The chunk that loads the package `name` with no options, under every
    /// engine. The identifier of the chunk is the package name itself.
    ///
    /// ```
    /// use untechxt::preamble::{Chunk, ChunkPreamble, Engine};
    ///
    /// let chunk = Chunk::package("amssymb");
    /// assert_eq!(chunk.id(), "amssymb");
    /// assert_eq!(
    ///     chunk.preamble_for(Engine::PdfLatex),
    ///     Some(&ChunkPreamble::package("amssymb")),
    /// );
    /// ```
    pub const fn package(name: &'static str) -> Self {
        Chunk::for_engines(name, EngineSet::ALL, ChunkPreamble::package(name))
    }

    /// The chunk that loads the package `name` with `options` under every
    /// engine, under the identifier `id`. One package loaded with different
    /// options gives different chunks, so the identifier cannot be the package
    /// name. Give the same identifier to every chunk that loads this package
    /// with these options, so that a [`PreambleNeeds`] keeps one copy.
    pub const fn package_with_options(
        id: &'static str,
        name: &'static str,
        options: &'static str,
    ) -> Self {
        Chunk::for_engines(
            id,
            EngineSet::ALL,
            ChunkPreamble::package_with_options(name, options),
        )
    }

    /// The chunk that makes the declarations `text` under every engine, under
    /// the identifier `id`. The text is preamble LaTeX and must not end with a
    /// newline, because a newline is written after each chunk.
    pub const fn snippet(id: &'static str, text: &'static str) -> Self {
        Chunk::for_engines(id, EngineSet::ALL, ChunkPreamble::snippet(text))
    }

    /// The chunk that is `preamble` under the engines of `engines` and that
    /// needs nothing under every other engine, under the identifier `id`.
    ///
    /// ```
    /// use untechxt::preamble::{Chunk, ChunkPreamble, Engine, EngineSet};
    ///
    /// // LuaLaTeX and XeLaTeX define the commands of the `T1` font encoding
    /// // themselves, so that only the other engines load the encoding.
    /// const FONTENC_T1: Chunk = Chunk::for_engines(
    ///     "fontenc-t1",
    ///     EngineSet::UNICODE.complement(),
    ///     ChunkPreamble::package_with_options("fontenc", "T1"),
    /// );
    /// assert!(FONTENC_T1.preamble_for(Engine::PdfLatex).is_some());
    /// assert_eq!(FONTENC_T1.preamble_for(Engine::LuaLatex), None);
    /// ```
    pub const fn for_engines(
        id: &'static str,
        engines: EngineSet,
        preamble: ChunkPreamble,
    ) -> Self {
        Chunk {
            id: Cow::Borrowed(id),
            cases: Cases::One(ChunkCase::for_engines(engines, preamble)),
        }
    }

    /// The chunk that borrows the static list `cases`, under the identifier
    /// `id`. This is how static data states a chunk with two or more cases.
    ///
    /// A [`ChunkCase`] has drop glue, so a slice literal of cases is not
    /// promoted to a static on its own. Declare the list as a static of its
    /// own and pass a reference to it, as the example of the struct [`Chunk`]
    /// does.
    pub const fn from_static(id: &'static str, cases: &'static [ChunkCase]) -> Self {
        Chunk { id: Cow::Borrowed(id), cases: Cases::Many(Cow::Borrowed(cases)) }
    }

    /// The chunk that owns `cases`, under the identifier `id`. This is how a
    /// rule built at run time states a chunk.
    ///
    /// ```
    /// use untechxt::preamble::{Chunk, ChunkCase, ChunkPreamble, Engine};
    ///
    /// let package = String::from("fontspec");
    /// let chunk = Chunk::new(String::from("fonts"), vec![
    ///     ChunkCase::for_engines(
    ///         Engine::LuaLatex | Engine::XeLatex,
    ///         ChunkPreamble::Package(package.into()),
    ///     ),
    /// ]);
    /// assert!(chunk.preamble_for(Engine::XeLatex).is_some());
    /// assert_eq!(chunk.preamble_for(Engine::PdfLatex), None);
    /// ```
    pub fn new(id: impl Into<Cow<'static, str>>, cases: Vec<ChunkCase>) -> Self {
        Chunk { id: id.into(), cases: Cases::Many(Cow::Owned(cases)) }
    }

    /// The identifier of the chunk: a short name. The name is the package's
    /// own name (`amssymb`), except where one package is loaded with different
    /// options under different identifiers (`fontenc-t2a`), or where the chunk
    /// is not a package at all (`bbold-alphabet`).
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The cases of the chunk, in the order they were given in. A chunk that
    /// is the same under every engine has one case, which applies to the set
    /// [`EngineSet::ALL`].
    pub fn cases(&self) -> &[ChunkCase] {
        match &self.cases {
            Cases::One(case) => core::slice::from_ref(case),
            Cases::Many(cases) => cases,
        }
    }

    /// Returns what the chunk is under `engine`: the preamble of the first
    /// case that applies to `engine`. Returns `None` when no case applies,
    /// which means that the chunk needs nothing under `engine`.
    pub fn preamble_for(&self, engine: Engine) -> Option<&ChunkPreamble> {
        self.cases()
            .iter()
            .find(|case| case.engines.contains(engine))
            .map(|case| &case.preamble)
    }
}

/// Two chunks are equal when they have the same identifier and the same list
/// of cases. How the chunk was created does not matter.
impl PartialEq for Chunk {
    fn eq(&self, other: &Chunk) -> bool {
        self.id == other.id && self.cases() == other.cases()
    }
}

impl Eq for Chunk {}

impl Hash for Chunk {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.cases().hash(state);
    }
}

/// What a [`Chunk`] is under one set of LaTeX engines: a piece of preamble,
/// together with the engines that it applies to.
///
/// A chunk holds a list of cases, and the first case that applies to an engine
/// is what the chunk is under that engine. See the section "Chunks that depend
/// on the LaTeX engine" of the struct [`Chunk`].
///
/// - The method [`ChunkCase::for_engines`] creates a case that applies to a
///   given set of engines.
/// - The method [`ChunkCase::otherwise`] creates a case that applies to every
///   engine. Write it last in a list, so that it applies to the engines that
///   no earlier case names.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChunkCase {
    /// The engines that the case applies to.
    engines: EngineSet,
    /// What the chunk is under those engines.
    preamble: ChunkPreamble,
}

impl ChunkCase {
    /// The case that a chunk is `preamble` under the engines of `engines`.
    ///
    /// In a `const` or `static` item, build `engines` from the constants of
    /// the struct [`EngineSet`]. Anywhere else you may also write
    /// `Engine::LuaLatex | Engine::XeLatex`, or `Engine::LuaLatex.into()` for
    /// one engine.
    pub const fn for_engines(engines: EngineSet, preamble: ChunkPreamble) -> Self {
        ChunkCase { engines, preamble }
    }

    /// The case that a chunk is `preamble` under every engine. In a list of
    /// cases this case goes last, and it then applies to every engine that no
    /// earlier case applies to. A list does not need such a case: without it,
    /// the chunk needs nothing under the engines that no case applies to.
    pub const fn otherwise(preamble: ChunkPreamble) -> Self {
        ChunkCase { engines: EngineSet::ALL, preamble }
    }

    /// The engines that the case applies to.
    pub const fn engines(&self) -> EngineSet {
        self.engines
    }

    /// What the chunk is under the engines of the case.
    pub const fn preamble(&self) -> &ChunkPreamble {
        &self.preamble
    }
}

/// A piece of preamble: a package to load, or declarations to make. This is
/// what a [`Chunk`] is under one LaTeX engine (see the method
/// [`Chunk::preamble_for`]).
///
/// A program that assembles a preamble treats the two kinds differently. A
/// package is loaded once however many values need it, while declarations are
/// written after the packages they may call into. The package name and its
/// options are kept apart, so that a consumer with its own package machinery
/// can merge several requests for one package into a single `\usepackage`
/// line. The struct [`PreambleNeeds`] itself does not merge.
///
/// The variants hold a [`Cow`], so that static data borrows its text and a
/// value built at run time owns its text. For static data, the methods
/// [`ChunkPreamble::package`], [`ChunkPreamble::package_with_options`] and
/// [`ChunkPreamble::snippet`] create each variant from string literals.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChunkPreamble {
    /// A package loaded with no options. Contains the package name, so that
    /// `"amssymb"` gives `\usepackage{amssymb}`.
    Package(Cow<'static, str>),
    /// A package loaded with options. Contains the package name and the option
    /// text, so that `("fontenc", "T2A,T1")` gives
    /// `\usepackage[T2A,T1]{fontenc}`.
    PackageWithOptions(Cow<'static, str>, Cow<'static, str>),
    /// Declarations that no package makes. Contains the preamble LaTeX that
    /// makes them, such as a math alphabet declared from a font family
    /// (`\DeclareMathAlphabet{\UnxTBbold}{U}{bbold}{m}{n}`). Every command that
    /// a builtin snippet defines has the prefix `\UnxT`, which this crate
    /// reserves.
    Snippet(Cow<'static, str>),
}

impl ChunkPreamble {
    /// The variant [`ChunkPreamble::Package`] for the package `name`.
    pub const fn package(name: &'static str) -> Self {
        ChunkPreamble::Package(Cow::Borrowed(name))
    }

    /// The variant [`ChunkPreamble::PackageWithOptions`] for the package
    /// `name` loaded with `options`.
    pub const fn package_with_options(name: &'static str, options: &'static str) -> Self {
        ChunkPreamble::PackageWithOptions(Cow::Borrowed(name), Cow::Borrowed(options))
    }

    /// The variant [`ChunkPreamble::Snippet`] for the declarations `text`. The
    /// text is preamble LaTeX and must not end with a newline.
    pub const fn snippet(text: &'static str) -> Self {
        ChunkPreamble::Snippet(Cow::Borrowed(text))
    }

    /// Whether the preamble loads a package, rather than making declarations
    /// of its own. A preamble loads every package before it makes any
    /// declaration, because a declaration may call into a package.
    pub const fn is_package(&self) -> bool {
        matches!(self, ChunkPreamble::Package(_) | ChunkPreamble::PackageWithOptions(..))
    }
}
