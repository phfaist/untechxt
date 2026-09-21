//! The [`Engine`] enum and the [`EngineSet`] struct: the LaTeX engines that a
//! piece of preamble is written for.

use core::fmt;
use core::ops::{BitOr, Not};

/// A LaTeX engine, which is the program that compiles the document.
///
/// The engines handle fonts and Unicode text differently, so the same encoded
/// LaTeX can need a different preamble under each engine. For example, the
/// command `\k` of the ogonek accent needs `\usepackage[T1]{fontenc}` under
/// pdfLaTeX. LuaLaTeX and XeLaTeX define the command `\k` themselves. Loading
/// the `T1` font encoding under these two engines would replace the Unicode
/// fonts of the document with 8-bit fonts, and the characters that are typed
/// directly into the document would then be lost without an error.
///
/// An [`Engine`] value names the one engine that a preamble is written for.
/// Pass it to the method [`Chunk::preamble_for`] or to the method
/// [`PreambleNeeds::write_preamble_for`]. To state which engines a piece of
/// preamble applies to, use an [`EngineSet`] instead.
///
/// A later version of this crate may add engines, which is why the enum is
/// non-exhaustive.
///
/// [`Chunk::preamble_for`]: crate::preamble::Chunk::preamble_for
/// [`PreambleNeeds::write_preamble_for`]:
///     crate::preamble::PreambleNeeds::write_preamble_for
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Engine {
    /// pdfLaTeX, which is the program `pdflatex`. The program `latex`, which
    /// writes a DVI file, runs the same engine and counts as pdfLaTeX too.
    /// LaTeX uses 8-bit font encodings such as `T1` under this engine.
    PdfLatex,
    /// LuaLaTeX, which is the program `lualatex`. LaTeX uses Unicode fonts
    /// and the Unicode font encoding `TU` under this engine.
    LuaLatex,
    /// XeLaTeX, which is the program `xelatex`. LaTeX uses Unicode fonts and
    /// the Unicode font encoding `TU` under this engine.
    XeLatex,
}

impl Engine {
    /// Every engine that this version of the crate knows, in the order of the
    /// variants.
    pub(crate) const KNOWN: [Engine; 3] =
        [Engine::PdfLatex, Engine::LuaLatex, Engine::XeLatex];

    /// The bit that stands for the engine in an [`EngineSet`].
    const fn bit(self) -> u8 {
        match self {
            Engine::PdfLatex => 1 << 0,
            Engine::LuaLatex => 1 << 1,
            Engine::XeLatex => 1 << 2,
        }
    }
}

/// A set of LaTeX engines, which states where a piece of preamble applies.
///
/// A [`ChunkCase`] holds an [`EngineSet`] beside its preamble. Build a set
/// from the constants of this struct. In a `const` or `static` item, combine
/// sets with the methods [`EngineSet::union`] and [`EngineSet::complement`].
/// Anywhere else you may also use the operators `|` and `!`, which work on
/// [`Engine`] values too.
///
/// ```
/// use untechxt::preamble::{Engine, EngineSet};
///
/// const EIGHT_BIT: EngineSet = EngineSet::UNICODE.complement();
/// assert!(EIGHT_BIT.contains(Engine::PdfLatex));
/// assert!(!EIGHT_BIT.contains(Engine::LuaLatex));
///
/// assert_eq!(Engine::LuaLatex | Engine::XeLatex, EngineSet::UNICODE);
/// ```
///
/// [`ChunkCase`]: crate::preamble::ChunkCase
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct EngineSet(u8);

impl EngineSet {
    /// Every engine, including any engine that a later version of this crate
    /// adds.
    pub const ALL: Self = EngineSet(u8::MAX);

    /// No engine at all.
    pub const EMPTY: Self = EngineSet(0);

    /// The engine [`Engine::PdfLatex`] alone.
    pub const PDFLATEX: Self = EngineSet::of(Engine::PdfLatex);

    /// The engine [`Engine::LuaLatex`] alone.
    pub const LUALATEX: Self = EngineSet::of(Engine::LuaLatex);

    /// The engine [`Engine::XeLatex`] alone.
    pub const XELATEX: Self = EngineSet::of(Engine::XeLatex);

    /// The engines that use Unicode fonts and the font encoding `TU`, which
    /// are LuaLaTeX and XeLaTeX. These two engines need the same preamble in
    /// most cases, and this constant states such a case once. The package
    /// `iftex` tests for the same set of engines with `\iftutex`.
    pub const UNICODE: Self = EngineSet::LUALATEX.union(EngineSet::XELATEX);

    /// The bits of the engines of [`Engine::KNOWN`]. Every other bit stands
    /// for an engine that a later version may add. Those other bits are
    /// always all set or all clear, because every set is built from the
    /// constants above with [`EngineSet::union`] and
    /// [`EngineSet::complement`].
    const KNOWN_BITS: u8 = 0b111;

    /// Returns the set that contains `engine` alone.
    pub const fn of(engine: Engine) -> Self {
        EngineSet(engine.bit())
    }

    /// Returns the union of the two sets. The [`BitOr`] operator (`a | b`)
    /// does the same thing outside a `const` context.
    pub const fn union(self, other: Self) -> Self {
        EngineSet(self.0 | other.0)
    }

    /// Returns the set of every engine that is not in this set, including
    /// any engine that a later version of this crate adds. The [`Not`]
    /// operator (`!a`) does the same thing outside a `const` context.
    pub const fn complement(self) -> Self {
        EngineSet(!self.0)
    }

    /// Returns whether `engine` is in the set.
    pub const fn contains(self, engine: Engine) -> bool {
        self.0 & engine.bit() != 0
    }

    /// Returns whether the set contains no engine at all.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl From<Engine> for EngineSet {
    fn from(engine: Engine) -> EngineSet {
        EngineSet::of(engine)
    }
}

impl BitOr for EngineSet {
    type Output = EngineSet;

    fn bitor(self, other: EngineSet) -> EngineSet {
        self.union(other)
    }
}

impl BitOr<Engine> for EngineSet {
    type Output = EngineSet;

    fn bitor(self, other: Engine) -> EngineSet {
        self.union(EngineSet::of(other))
    }
}

impl BitOr for Engine {
    type Output = EngineSet;

    fn bitor(self, other: Engine) -> EngineSet {
        EngineSet::of(self).union(EngineSet::of(other))
    }
}

impl BitOr<EngineSet> for Engine {
    type Output = EngineSet;

    fn bitor(self, other: EngineSet) -> EngineSet {
        EngineSet::of(self).union(other)
    }
}

impl Not for EngineSet {
    type Output = EngineSet;

    fn not(self) -> EngineSet {
        self.complement()
    }
}

impl Not for Engine {
    type Output = EngineSet;

    fn not(self) -> EngineSet {
        EngineSet::of(self).complement()
    }
}

impl fmt::Debug for EngineSet {
    /// Formats the set by naming its engines: `EngineSet(LuaLatex | XeLatex)`.
    /// A set that includes the engines of later versions is formatted as the
    /// complement of the engines it leaves out: `EngineSet(!(PdfLatex))`. The
    /// constants [`EngineSet::ALL`] and [`EngineSet::EMPTY`] format as their
    /// own names.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == EngineSet::ALL {
            return f.write_str("EngineSet::ALL");
        }
        if *self == EngineSet::EMPTY {
            return f.write_str("EngineSet::EMPTY");
        }
        let negated = self.0 & !EngineSet::KNOWN_BITS != 0;
        let named = if negated { self.complement() } else { *self };
        f.write_str(if negated { "EngineSet(!(" } else { "EngineSet(" })?;
        let mut first = true;
        for engine in Engine::KNOWN {
            if named.contains(engine) {
                if !first {
                    f.write_str(" | ")?;
                }
                write!(f, "{engine:?}")?;
                first = false;
            }
        }
        f.write_str(if negated { "))" } else { ")" })
    }
}
