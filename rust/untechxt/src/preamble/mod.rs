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
//! - The struct [`Profile`] is the set of chunks that one encoded value needs.
//!   A rule returns a profile together with the value it produces.
//! - The struct [`PreambleNeeds`] collects the chunks that a whole document
//!   needs, and writes the corresponding preamble.

mod profile;

use alloc::borrow::Cow;

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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Chunk {
    /// The identifier of the chunk: a short name. The name is the package's
    /// own name (`amssymb`), except where one package is loaded with different
    /// options under different identifiers (`fontenc-t2a`), or where the chunk
    /// is not a package at all (`bbold-alphabet`).
    pub id: Cow<'static, str>,
    /// The content of the chunk: a package to load, or the declarations to
    /// make. See the enum [`ChunkPreamble`].
    pub preamble: ChunkPreamble,
}

impl Chunk {
    /// The chunk that loads the package `name` with no options. The identifier
    /// of the chunk is the package name itself.
    ///
    /// ```
    /// use untechxt::preamble::{Chunk, ChunkPreamble};
    ///
    /// let chunk = Chunk::package("amssymb");
    /// assert_eq!(&*chunk.id, "amssymb");
    /// assert!(matches!(chunk.preamble, ChunkPreamble::Package(_)));
    /// ```
    pub const fn package(name: &'static str) -> Self {
        Chunk {
            id: Cow::Borrowed(name),
            preamble: ChunkPreamble::Package(Cow::Borrowed(name)),
        }
    }

    /// The chunk that loads the package `name` with `options`, under the
    /// identifier `id`. One package loaded with different options gives
    /// different chunks, so the identifier cannot be the package name. Give the
    /// same identifier to every chunk that loads this package with these
    /// options, so that a [`PreambleNeeds`] keeps one copy.
    pub const fn package_with_options(
        id: &'static str,
        name: &'static str,
        options: &'static str,
    ) -> Self {
        Chunk {
            id: Cow::Borrowed(id),
            preamble: ChunkPreamble::PackageWithOptions(
                Cow::Borrowed(name),
                Cow::Borrowed(options),
            ),
        }
    }

    /// The chunk that makes the declarations `text`, under the identifier `id`.
    /// The text is preamble LaTeX and must not end with a newline. The method
    /// [`PreambleNeeds::write_preamble`] adds a newline after each chunk.
    pub const fn snippet(id: &'static str, text: &'static str) -> Self {
        Chunk {
            id: Cow::Borrowed(id),
            preamble: ChunkPreamble::Snippet(Cow::Borrowed(text)),
        }
    }

    /// Whether the chunk loads a package, rather than making declarations of
    /// its own. A preamble loads every package before it makes any
    /// declaration, because a declaration may call into a package.
    pub const fn is_package(&self) -> bool {
        matches!(
            self.preamble,
            ChunkPreamble::Package(_) | ChunkPreamble::PackageWithOptions(..)
        )
    }
}

/// The content of a [`Chunk`]: a package to load, or declarations to make.
///
/// A program that assembles a preamble treats the two kinds differently. A
/// package is loaded once however many values need it, while declarations are
/// written after the packages they may call into. The package name and its
/// options are kept apart, so that a consumer with its own package machinery
/// can merge several requests for one package into a single `\usepackage`
/// line. The method [`PreambleNeeds::write_preamble`] itself does not merge.
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
