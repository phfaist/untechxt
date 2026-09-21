//! What the encoded LaTeX needs in the preamble of the document.
//!
//! LaTeX code such as `\mathds{1}` only works when the document loads the
//! package `dsfont`. This module contains the types that describe such
//! requirements:
//!
//! - A [`Chunk`] is one piece of preamble under a stable identifier. It is
//!   either a package to load or a block of declarations (see
//!   [`ChunkPreamble`]).
//! - A [`Profile`] is the set of chunks that one encoded value needs. A rule
//!   returns a profile together with its encoded value.
//! - A [`PreambleNeeds`] accumulates the chunks that a whole document needs,
//!   and writes the corresponding preamble.

mod profile;

use alloc::borrow::Cow;

pub use self::profile::{PreambleNeeds, Profile};

/// One **preamble chunk**: a piece of preamble — a package to load, or
/// declarations to make — that some encoded values need.
///
/// A value such as `\mathds{1}` prints nothing on its own: the command
/// `\mathds` exists only once the document has loaded the package `dsfont`.
/// Some values need something no package provides — `\UnxTBbold{0}` needs a
/// math alphabet declared from a font family, because the package that has
/// that font would redefine `\mathbb` for the whole document — and for those
/// the chunk is the declaration itself ([`ChunkPreamble::Snippet`]).
///
/// Either way a chunk is named by a short identifier, so that a program can
/// recognize it without parsing its text. The same identifier always means
/// the same chunk: that is how a [`PreambleNeeds`] keeps one copy of each. A
/// rule names the chunks of one value through a [`Profile`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Chunk {
    /// The chunk's identifier: a short name, which is the package's own name
    /// (`amssymb`) except where one package is loaded with different options
    /// under different identifiers (`fontenc-t2a`), or where the chunk is not
    /// a package at all (`bbold-alphabet`).
    pub id: Cow<'static, str>,
    /// What the document's preamble must hold: a package to load, or the
    /// declarations to make.
    pub preamble: ChunkPreamble,
}

impl Chunk {
    /// The chunk that loads the package `name` with no options. Its
    /// identifier is the package name itself.
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
    /// identifier `id` — one package loaded with different options is
    /// different chunks, so the identifier cannot be the package name.
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

    /// The chunk that makes the declarations `text`, under the identifier
    /// `id`. The text is preamble LaTeX, without a newline at its end.
    pub const fn snippet(id: &'static str, text: &'static str) -> Self {
        Chunk {
            id: Cow::Borrowed(id),
            preamble: ChunkPreamble::Snippet(Cow::Borrowed(text)),
        }
    }

    /// Whether the chunk loads a package, rather than making declarations of
    /// its own. A preamble loads every package before it makes any
    /// declaration, since a declaration may call into a package.
    pub const fn is_package(&self) -> bool {
        matches!(
            self.preamble,
            ChunkPreamble::Package(_) | ChunkPreamble::PackageWithOptions(..)
        )
    }
}

/// What a [`Chunk`] asks a preamble for: a package, or declarations of its
/// own.
///
/// The kinds are told apart because a program that assembles a preamble
/// treats them differently — a package is requested once however many things
/// ask for it, declarations are written where the packages they call into are
/// already loaded. The package name and its options are kept apart, so that a
/// consumer with its own package machinery can merge several requests for one
/// package into a single `\usepackage` line;
/// [`PreambleNeeds::write_preamble`] itself does not merge.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChunkPreamble {
    /// One LaTeX package, loaded with no options: `\usepackage{amssymb}`.
    Package(Cow<'static, str>),
    /// One LaTeX package with the options it is loaded with, as the package
    /// name and the option text: `("fontenc", "T2A,T1")` for
    /// `\usepackage[T2A,T1]{fontenc}`.
    PackageWithOptions(Cow<'static, str>, Cow<'static, str>),
    /// Declarations no package makes, as the lines that make them: a math
    /// alphabet (`\DeclareMathAlphabet{\UnxTBbold}{U}{bbold}{m}{n}`), a symbol
    /// font with the symbols read from it. Every command a builtin snippet
    /// defines is named `\UnxT…`, a prefix this crate reserves.
    Snippet(Cow<'static, str>),
}
