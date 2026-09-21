//! The builtin preamble chunks and profiles: what a document must hold for
//! the spellings of [`default_table`](super::default_table) to print.
//!
//! A spelling such as `\mathds{1}` prints nothing on its own: the command
//! `\mathds` exists only once the document has loaded the package `dsfont`.
//! Some spellings need something no package provides — `\UnxTBbold{0}` needs
//! a math alphabet declared from a font family, because the package that has
//! that font would redefine `\mathbb` for the whole document — and for those
//! the [`Chunk`] is the declaration itself.
//!
//! Every entry of the builtin table carries a [`ProfileIndex`]: the position
//! in [`PROFILES`] of the set of chunks its spelling needs. The constants
//! below name those positions, and the entries are written with the names,
//! not the numbers. Index 0, [`BUILTINS`], is the empty set — a spelling the
//! LaTeX kernel prints by itself — and a table lookup answers `None` for it
//! without reading the array at all.
//!
//! # Maintaining these
//!
//! A chunk exists here only because some entry needs it; a spelling that
//! needs something no chunk covers adds one. A profile is added at the end,
//! never reordered, since the index is what the compiled table stores; adding
//! one means adding its constant here and naming that constant in the
//! entries.
//!
//! Every command a snippet chunk defines is named `\UnxT…`, a prefix this
//! crate reserves, and so is every font identifier a snippet declares. A
//! document may therefore load whatever packages it likes beside these
//! chunks: nothing here redefines a command of the kernel or of a package.

use crate::preamble::Chunk;
use crate::profile::{Profile, ProfileIndex};

// ------------------------------------------------------------------ chunks
//
// One constant per distinct piece of preamble, with what it provides and
// which spellings need it. The profiles below are built out of these.

/// The American Mathematical Society's mathematics package: `\boldsymbol`,
/// `\iint`, `\iiint`, `\nobreakdash`.
const AMSMATH_CHUNK: Chunk = Chunk::package("amsmath");

/// The American Mathematical Society's symbol package: `\mathbb`, `\mathfrak`
/// and several hundred mathematical symbols the LaTeX kernel does not have.
const AMSSYMB_CHUNK: Chunk = Chunk::package("amssymb");

/// The script alphabet `\mathscr`, which is the capital letters.
const MATHRSFS_CHUNK: Chunk = Chunk::package("mathrsfs");

/// The double-struck alphabet `\mathds`, which has digits and lowercase
/// letters where `amssymb`'s `\mathbb` has capital letters alone.
const DSFONT_CHUNK: Chunk = Chunk::package("dsfont");

/// `\nicefrac`, the slanted fraction the vulgar fractions are spelled with.
const NICEFRAC_CHUNK: Chunk = Chunk::package("nicefrac");

/// The phonetic alphabet: `\textschwa`, `\textglotstop` and the other
/// phonetic letters.
const TIPA_CHUNK: Chunk = Chunk::package("tipa");

/// The phonetic letters `tipa` moved out of its own encoding, `\textnrleg`
/// among them.
const TIPX_CHUNK: Chunk = Chunk::package("tipx");

/// The double-struck alphabet `\mathbbm`, which has the 26 lowercase letters
/// and the digits `1` and `2`; it defines nothing else, so the document's own
/// fonts are untouched.
const BBM_CHUNK: Chunk = Chunk::package("bbm");

/// The `T1` font encoding, in which the letters of the European languages
/// that `OT1` has no place for are declared: the ogonek accent `\k`, the
/// guillemets, `\DH`, `\TH`, `\NG`, `\DJ` and the low quotation marks.
const FONTENC_T1_CHUNK: Chunk = Chunk::package_with_options("fontenc-t1", "fontenc", "T1");

/// The `T2A` Cyrillic font encoding, in which the Cyrillic letter commands of
/// the modern languages are declared, beside `T1`, which stays the document's
/// own encoding so that Latin text keeps its usual fonts.
const FONTENC_T2A_CHUNK: Chunk = Chunk::package_with_options("fontenc-t2a", "fontenc", "T2A,T1");

/// The `X2` font encoding, which holds every Cyrillic letter of the `T2`
/// encodings together, and the letters of the minority languages none of them
/// has; beside `T1`, which stays the document's own encoding.
const FONTENC_X2_CHUNK: Chunk = Chunk::package_with_options("fontenc-x2", "fontenc", "X2,T1");

/// The `T2B` Cyrillic font encoding, in which the letters of the Caucasian
/// and Siberian languages `X2` has no place for are declared; beside `T1`.
const FONTENC_T2B_CHUNK: Chunk = Chunk::package_with_options("fontenc-t2b", "fontenc", "T2B,T1");

/// The `T2C` Cyrillic font encoding, in which the older Slavonic letters —
/// the semisoft sign, er with tick — are declared; beside `T1`.
const FONTENC_T2C_CHUNK: Chunk = Chunk::package_with_options("fontenc-t2c", "fontenc", "T2C,T1");

/// The `OT2` Cyrillic font encoding, the seven-bit one, which is where fita
/// is declared; beside `T1`.
const FONTENC_OT2_CHUNK: Chunk = Chunk::package_with_options("fontenc-ot2", "fontenc", "OT2,T1");

/// The `T2D` Old Church Slavonic font encoding, in which the letters of the
/// Slavonic alphabet that no modern encoding has — omega, ksi, psi, koppa,
/// the yuses — are declared; beside `T1`.
const FONTENC_T2D_CHUNK: Chunk = Chunk::package_with_options("fontenc-t2d", "fontenc", "T2D,T1");

/// The math alphabet `\UnxTBbold`, declared from the `bbold` font family, for
/// the double-struck digits `bbm`'s font has no glyph for. The package
/// `bbold` is not loaded: it would give those glyphs the name `\mathbb`,
/// which is `amssymb`'s and is what the double-struck capitals are spelled
/// with.
const BBOLD_CHUNK: Chunk =
    Chunk::snippet("bbold-alphabet", r"\DeclareMathAlphabet{\UnxTBbold}{U}{bbold}{m}{n}");

/// The math alphabet `\UnxTScr`, declared from the `dutchcal` font family,
/// for the lowercase script letters the `rsfs` font `mathrsfs` loads has no
/// glyph for. The package `dutchcal` is not loaded: it would make that family
/// the whole document's `\mathcal`.
const SCRIPT_CHUNK: Chunk =
    Chunk::snippet("script-alphabet", r"\DeclareMathAlphabet{\UnxTScr}{U}{dutchcal}{m}{n}");

/// The text symbol `\UnxTCyrThousands`, the Cyrillic thousands sign, read
/// from its slot in the `T2D` encoding, which declares it as an accent — a
/// form that sets the sign over its argument rather than beside it.
const CYR_THOUSANDS_CHUNK: Chunk = Chunk::snippet(
    "cyrillic-thousands",
    r#"\DeclareTextSymbol{\UnxTCyrThousands}{T2D}{"9E}"#,
);

/// The mathematical symbols of the STIX fonts that no package of a current
/// installation declares: six symbol fonts read from the STIX families, and
/// one `\UnxT…` command per symbol. The package `stix` is not loaded: it
/// would make the STIX fonts the whole document's mathematics.
const STIX_CHUNK: Chunk = Chunk::snippet(
    "stix-symbols",
    r#"\DeclareFontEncoding{LS1}{}{}
\DeclareFontSubstitution{LS1}{stix}{m}{n}
\DeclareFontEncoding{LS2}{}{}
\DeclareFontSubstitution{LS2}{stix}{m}{n}
\DeclareSymbolFont{UnxTstixsf}{LS1}{stixsf}{m}{n}
\DeclareSymbolFont{UnxTstixtt}{LS2}{stixtt}{m}{n}
\DeclareSymbolFont{UnxTstixcal}{LS2}{stixcal}{m}{n}
\DeclareSymbolFont{UnxTstixit}{LS1}{stix}{m}{it}
\DeclareSymbolFont{UnxTstixrm}{LS1}{stix}{m}{n}
\DeclareSymbolFont{UnxTstixscr}{LS1}{stixscr}{m}{n}
\DeclareMathSymbol{\UnxTleftwavearrow}{\mathrel}{UnxTstixsf}{"A3}
\DeclareMathSymbol{\UnxTrightwavearrow}{\mathrel}{UnxTstixsf}{"A4}
\DeclareMathSymbol{\UnxTupdownarrows}{\mathrel}{UnxTstixsf}{"CC}
\DeclareMathSymbol{\UnxTdownuparrows}{\mathrel}{UnxTstixsf}{"F3}
\DeclareMathSymbol{\UnxTrightangle}{\mathord}{UnxTstixrm}{"D9}
\DeclareMathSymbol{\UnxToiint}{\mathop}{UnxTstixcal}{"04}
\DeclareMathSymbol{\UnxToiiint}{\mathop}{UnxTstixcal}{"05}
\DeclareMathSymbol{\UnxTintclockwise}{\mathop}{UnxTstixcal}{"06}
\DeclareMathSymbol{\UnxTkernelcontraction}{\mathrel}{UnxTstixrm}{"EC}
\DeclareMathSymbol{\UnxTinvlazys}{\mathbin}{UnxTstixrm}{"EF}
\DeclareMathSymbol{\UnxTsimneqq}{\mathrel}{UnxTstixrm}{"F7}
\DeclareMathSymbol{\UnxTapproxident}{\mathrel}{UnxTstixrm}{"FC}
\DeclareMathSymbol{\UnxTbackcong}{\mathrel}{UnxTstixrm}{"FD}
\DeclareMathSymbol{\UnxTwedgeq}{\mathrel}{UnxTstixcal}{"8A}
\DeclareMathSymbol{\UnxTstareq}{\mathrel}{UnxTstixcal}{"8C}
\DeclareMathSymbol{\UnxTnlessgtr}{\mathrel}{UnxTstixit}{"C5}
\DeclareMathSymbol{\UnxTngtrless}{\mathrel}{UnxTstixit}{"C6}
\DeclareMathSymbol{\UnxTmodels}{\mathrel}{UnxTstixit}{"F4}
\DeclareMathSymbol{\UnxTvDash}{\mathrel}{UnxTstixit}{"F5}
\DeclareMathSymbol{\UnxTVDash}{\mathrel}{UnxTstixit}{"F8}
\DeclareMathSymbol{\UnxTorigof}{\mathrel}{UnxTstixscr}{"01}
\DeclareMathSymbol{\UnxTimageof}{\mathrel}{UnxTstixscr}{"02}
\DeclareMathSymbol{\UnxThermitmatrix}{\mathord}{UnxTstixscr}{"04}
\DeclareMathSymbol{\UnxTmeasuredrightangle}{\mathord}{UnxTstixscr}{"09}
\DeclareMathSymbol{\UnxTlll}{\mathrel}{UnxTstixscr}{"1E}
\DeclareMathSymbol{\UnxTggg}{\mathrel}{UnxTstixscr}{"1F}
\DeclareMathSymbol{\UnxTprecnsim}{\mathrel}{UnxTstixscr}{"2E}
\DeclareMathSymbol{\UnxTadots}{\mathrel}{UnxTstixscr}{"36}
\DeclareMathSymbol{\UnxTvardoublebarwedge}{\mathbin}{UnxTstixscr}{"9A}
\DeclareMathSymbol{\UnxTquarternote}{\mathord}{UnxTstixtt}{"2F}
\DeclareMathSymbol{\UnxTfint}{\mathop}{UnxTstixcal}{"0D}
\DeclareMathSymbol{\UnxTsqint}{\mathop}{UnxTstixcal}{"14}"#,
);

/// The symbol `\UnxTrecorder`, the telephone recorder, read from its slot in
/// the `wasy` font. The package `wasysym` is not loaded: it redefines `\int`
/// and several symbols the document may be using.
const WASY_CHUNK: Chunk = Chunk::snippet(
    "wasy-recorder",
    r"\DeclareSymbolFont{UnxTwasy}{U}{wasy}{m}{n}
\DeclareMathSymbol{\UnxTrecorder}{\mathord}{UnxTwasy}{6}",
);

// ---------------------------------------------------------------- profiles

/// The number of token trees given, as a `usize` constant.
macro_rules! count {
    (@one $item:tt) => {
        ()
    };
    ($($item:tt)*) => {
        <[()]>::len(&[$(count!(@one $item)),*])
    };
}

/// Declares the builtin profiles: one `static` array of [`Chunk`]s per
/// profile, the [`PROFILES`] array over those, and one public
/// [`ProfileIndex`] constant per profile, numbered by the order they are
/// written in.
///
/// Each profile is written as `INDEX_NAME, CHUNKS_NAME = [chunks…];`, with
/// its documentation above it. The chunk lists need arrays of their own
/// because a [`Chunk`] has drop glue, so a slice literal of chunks is not
/// promoted to a static on its own and [`Profile::from_static`] would have
/// nothing to borrow.
macro_rules! builtin_profiles {
    (
        $(
            $(#[$doc:meta])*
            $name:ident, $chunks:ident = [ $($chunk:expr),* $(,)? ];
        )*
    ) => {
        $(
            static $chunks: [Chunk; count!($($chunk)*)] = [$($chunk),*];
        )*

        /// The builtin profiles, one per position: the set of preamble
        /// [`Chunk`]s that the spellings carrying that position need.
        ///
        /// Position 0 is the empty profile, so that a
        /// [`ProfileIndex`](crate::ProfileIndex) is always a position of this
        /// array; a table lookup answers `None` for it without reading the
        /// array. The constants of this module name the positions.
        ///
        /// ```
        /// use untechxt::builtin::needs_profiles::{AMSSYMB, PROFILES};
        ///
        /// let chunks = PROFILES[AMSSYMB.0 as usize].chunks();
        /// assert_eq!(chunks.len(), 1);
        /// assert_eq!(&*chunks[0].id, "amssymb");
        /// ```
        pub static PROFILES: [Profile; count!($($name)*)] =
            [$(Profile::from_static(&$chunks)),*];

        builtin_profile_indices!(0u8; $($(#[$doc])* $name)*);
    };
}

/// Numbers the profile constants of [`builtin_profiles!`] from `$index`
/// upward, one per name, keeping each one's documentation.
macro_rules! builtin_profile_indices {
    ($index:expr;) => {};
    (
        $index:expr;
        $(#[$doc:meta])* $name:ident $($rest:tt)*
    ) => {
        $(#[$doc])*
        pub const $name: ProfileIndex = ProfileIndex($index);
        builtin_profile_indices!($index + 1; $($rest)*);
    };
}

builtin_profiles! {
    /// Nothing beyond the LaTeX kernel — the commands every document has
    /// without loading a package. It is the profile of most of the builtin
    /// entries, and it is the reserved index 0, which a table lookup answers
    /// `None` for.
    BUILTINS, BUILTINS_CHUNKS = [];

    /// `dsfont`, for the `\mathds` spellings.
    DSFONT, DSFONT_CHUNKS = [DSFONT_CHUNK];

    /// `nicefrac`, for the `\nicefrac` spellings.
    NICEFRAC, NICEFRAC_CHUNKS = [NICEFRAC_CHUNK];

    /// The `T2A` font encoding, for the Cyrillic spellings of the modern
    /// languages.
    FONTENC_T2A, FONTENC_T2A_CHUNKS = [FONTENC_T2A_CHUNK];

    /// The `T1` font encoding, for the letters and punctuation `OT1` has no
    /// place for: the ogonek accent `\k`, `\DH`, `\TH`, `\dh`, `\th`, `\DJ`,
    /// `\dj`, `\NG`, `\ng`, the guillemets and the low quotation marks.
    FONTENC_T1, FONTENC_T1_CHUNKS = [FONTENC_T1_CHUNK];

    /// `tipa`, for the phonetic letters the `\text…` spellings name that no
    /// other package defines.
    TIPA, TIPA_CHUNKS = [TIPA_CHUNK];

    /// `amssymb`, for `\mathbb`, `\mathfrak` and the American Mathematical
    /// Society's symbols.
    AMSSYMB, AMSSYMB_CHUNKS = [AMSSYMB_CHUNK];

    /// `amsmath`, for `\boldsymbol`, `\iint`, `\iiint` and `\nobreakdash`.
    AMSMATH, AMSMATH_CHUNKS = [AMSMATH_CHUNK];

    /// `mathrsfs`, for the script alphabet `\mathscr`, which is the capitals.
    MATHRSFS, MATHRSFS_CHUNKS = [MATHRSFS_CHUNK];

    /// `bbm`, for the double-struck lowercase letters and the digit `2`.
    BBM, BBM_CHUNKS = [BBM_CHUNK];

    /// The `\UnxTBbold` alphabet, for the double-struck digits `bbm` has no
    /// glyph for.
    BBOLD, BBOLD_CHUNKS = [BBOLD_CHUNK];

    /// The `\UnxTScr` alphabet, for the lowercase script letters `rsfs` has
    /// no glyph for.
    SCRIPT, SCRIPT_CHUNKS = [SCRIPT_CHUNK];

    /// The `X2` font encoding, for the Cyrillic letters `T2A` has no place
    /// for.
    FONTENC_X2, FONTENC_X2_CHUNKS = [FONTENC_X2_CHUNK];

    /// The `T2B` font encoding, for the Caucasian and Siberian letters.
    FONTENC_T2B, FONTENC_T2B_CHUNKS = [FONTENC_T2B_CHUNK];

    /// The `T2C` font encoding, for the older Slavonic letters.
    FONTENC_T2C, FONTENC_T2C_CHUNKS = [FONTENC_T2C_CHUNK];

    /// The `OT2` font encoding, for fita.
    FONTENC_OT2, FONTENC_OT2_CHUNKS = [FONTENC_OT2_CHUNK];

    /// The `T2D` font encoding, for the Old Church Slavonic letters.
    FONTENC_T2D, FONTENC_T2D_CHUNKS = [FONTENC_T2D_CHUNK];

    /// The `T2D` font encoding and the `\UnxTCyrThousands` symbol read from
    /// it.
    CYR_THOUSANDS, CYR_THOUSANDS_CHUNKS = [FONTENC_T2D_CHUNK, CYR_THOUSANDS_CHUNK];

    /// The STIX symbol fonts and their `\UnxT…` symbols.
    STIX, STIX_CHUNKS = [STIX_CHUNK];

    /// The `\UnxTrecorder` symbol read from the `wasy` font.
    WASY, WASY_CHUNKS = [WASY_CHUNK];

    /// `tipx`, for `\textnrleg`.
    TIPX, TIPX_CHUNKS = [TIPX_CHUNK];
}
