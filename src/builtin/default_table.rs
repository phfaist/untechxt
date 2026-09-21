//! The builtin Unicode-to-LaTeX entries: one LaTeX spelling per character,
//! with the mode that spelling is valid in and the preamble chunks a document
//! needs in order to print it.
//!
//! **This file is maintained by hand.** It originated in a copy of
//! pylatexenc's table (see *Provenance* below) and is a source file of this
//! library like any other since then: a spelling that is wrong is corrected
//! here, and the correction is listed under *Departures from pylatexenc*. No
//! generator produces this file, and no second copy of the data exists
//! anywhere.
//!
//! The one-off script `tools/migrate_tables.py` produced the `ENTRIES` block
//! below once, out of the earlier port's `initial-rust-port/src/tables.rs`: it
//! split the `\ensuremath{…}` wrapper off into the mode column and renamed the
//! `\flm` command prefix to `\UnxT`. That script has done its work; this file
//! is the source of truth now, and the script is kept only as a record of how
//! the data got here. Do not run it again over an edited table.
//!
//! Three pieces make up the builtin table, and they are read together:
//!
//! - `ENTRIES`, below — a character, its LaTeX spelling, the [`ValueMode`] of
//!   that spelling and the [`ProfileIndex`] of the preamble it needs. Sorted
//!   by character. `mod.rs` compiles it into the static table `DEFAULTS`, of
//!   which `NON_ASCII` is a view, and compiles its ASCII head once more into
//!   the small table `ASCII_SPECIALS`.
//! - The **preamble chunks** of `needs_profiles.rs`, which some spellings
//!   need: each a LaTeX package with the options it is loaded with, or a block
//!   of declarations no package makes.
//! - The **profiles** of `needs_profiles.rs`: each a set of chunks, named by
//!   its position, which is the index an entry carries. Profile 0, `BUILTINS`,
//!   is the empty set — a spelling the LaTeX kernel prints by itself.
//!
//! # How the table is maintained
//!
//! An entry is a line of `ENTRIES`, in character order, with a comment naming
//! the character. Its profile is the name of a profile constant of
//! `needs_profiles.rs`, chosen by reading the spelling: which commands it
//! writes, and which package or declaration defines each of them. A spelling
//! that needs something no chunk covers takes a new chunk there and, where no
//! existing profile is that chunk's set, a new profile with a constant beside
//! it.
//!
//! Its mode says where the spelling may be written: `MATH` for a bare
//! mathematical spelling ([`ValueMode::MathOnly`] — the encoder wraps it in
//! `\ensuremath{…}` when the output is text), `TEXT` for a text-mode spelling
//! ([`ValueMode::TextOnly`]), and `ANY` for one that holds in both
//! ([`ValueMode::AnyMode`]): a plain character, or a spelling that carries its
//! own `\ensuremath{…}` throughout.
//!
//! The `compile_static_table!` macro checks the rest at compile time: that the
//! keys ascend strictly, that every profile index names a profile that exists,
//! and that every spelling is ASCII, has balanced braces (not counting `\{`
//! and `\}`) and does not end in a lone backslash.
//!
//! # Provenance
//!
//! The table originated as a copy of pylatexenc's `defaults` conversion
//! table: <https://github.com/phfaist/pylatexenc>, commit `e4ddf2ba`, file
//! `pylatexenc/latexencode/_uni2latexmap.py` (the dictionary `uni2latex`,
//! 1553 entries, U+0022 to U+1D7FF), whose character map was in turn adapted
//! from latexcodec. Both license notices are reproduced below and stay with
//! the table. It has been maintained by hand since 2026-09-14; the entries are
//! pylatexenc's except for the departures listed next, and the profiles are
//! this library's own — pylatexenc records nothing about packages.
//!
//! # Departures from pylatexenc
//!
//! Every entry of this table compiles and sets the glyph the character stands
//! for, on the floor this library states (a LaTeX system of 2022 or later with
//! the chunks of `needs_profiles.rs` installed). Reaching that took the
//! departures listed here. They are of three kinds: a spelling whose command
//! no installed package defines is replaced by one that works; a spelling
//! whose command is right but whose font has no glyph for the letter is moved
//! to a font that has it; and a character no installed font has at all loses
//! its entry, so that a caller reports it as one no spelling is known for and
//! the recomposed LaTeX still compiles.
//!
//! | Characters | pylatexenc's spelling | here | why |
//! |---|---|---|---|
//! | U+1D7D9 `𝟙` | `\ensuremath{\mathbb{1}}` | `\mathds{1}` | `\mathbb`, of `amssymb`, is defined for the capital letters alone. `dsfont`'s `\mathds` has the digit `1` — the one of this range a document is likely to hold |
//! | U+1D7DA `𝟚` and U+1D552 … U+1D56B, the double-struck lowercase letters `𝕒`…`𝕫` | `\ensuremath{\mathbb{a}}` … | `\mathbbm{a}` … | `bbm`'s `\mathbbm` has the 26 lowercase letters and the digits `1` and `2`; `dsfont`'s font has only `a`, `h` and `k` of them |
//! | U+1D7D8 `𝟘` and U+1D7DB … U+1D7E1 `𝟛`…`𝟡` | `\ensuremath{\mathbb{0}}` … | `\UnxTBbold{0}` … | no font a package offers under a name of its own has these digits: `bbm10` has no glyph for `0` or for `3`–`9`, and `bbold`, which has them all, offers them *as* `\mathbb`, which would change what the double-struck capitals print. The `bbold-alphabet` chunk declares the `bbold` family as the math alphabet `\UnxTBbold` without loading that package |
//! | U+1D4B6 … U+1D4CF, the script lowercase letters `𝒶`…`𝓏` | `\ensuremath{\mathscr{a}}` … | `\UnxTScr{a}` … | the `rsfs` font `mathrsfs` loads has the 26 capitals alone. The `script-alphabet` chunk declares the `dutchcal` family, which has both cases, as the math alphabet `\UnxTScr` without loading `dutchcal`, which would make that family the document's own `\mathcal`. The capitals keep `\mathscr` |
//! | U+210A `ℊ`, U+212F `ℯ` and U+2134 `ℴ`, the script small letters of the Letterlike Symbols block | `\ensuremath{g}`, `\ensuremath{e}`, `\ensuremath{o}` | `\UnxTScr{g}`, `\UnxTScr{e}`, `\UnxTScr{o}` | a math-italic letter is not the script letter the character stands for. These three are the script lowercase letters Unicode keeps outside the Mathematical Alphanumeric block, and they are spelled with the same `\UnxTScr` alphabet as the 23 of the row above, so that one spelling prints the whole script lowercase |
//! | U+2153 … U+215E, the vulgar fractions `⅓`…`⅞` | `\textfrac{1}{3}` … | `\nicefrac{1}{3}` … | no package defines `\textfrac`; `\nicefrac`, of `nicefrac`, prints the same fraction |
//! | the 242 Cyrillic entries, `Ѐ` … `ӿ` | `\cyrya`, `\cyrchar\cyrksi` | `{\fontencoding{T2A}\selectfont\cyrya}`, `{\fontencoding{T2D}\selectfont\cyrksi}` | a Cyrillic letter command is declared in a font encoding, and stops the compilation ("Command \cyrya unavailable in encoding T1") while another encoding is selected, so the spelling selects that encoding around itself and leaves the rest of the document in the document's own. Which encoding differs by letter: `T2A` for the 167 letters of the modern languages, `X2` for 40 more, `T2B` for 4, `T2C` for 4, `OT2` for fita, and `T2D` for the 25 Old Church Slavonic entries (24 letters and the thousands sign of the row below). The `\cyrchar` prefix of pylatexenc's spellings is the old `ucs` package's and is dropped: the letter command of the encoding is what the encoding declares |
//! | U+0482 `҂`, the Cyrillic thousands sign | `\cyrchar\cyrthousands` | `{\fontencoding{T2D}\selectfont\UnxTCyrThousands}` | `T2D` declares its glyph as a *text accent*, which sets the sign over its argument; the `cyrillic-thousands` chunk reads the same slot as a text symbol, which sets it on its own |
//! | 32 mathematical symbols, U+219C … U+2A16 | `\ensuremath{\allequal}` … | `\UnxTbackcong` … | the names are Unicode's, by way of latexcodec, and no package of a current installation defines them. The STIX fonts have every one of these glyphs; the `stix-symbols` chunk reads them from the STIX families as `\UnxT…` symbols of its own, without loading `stix`, which would make the STIX fonts the document's whole mathematics. The `\UnxT…` names are `unicode-math`'s names for the same characters |
//! | U+2315 `⌕`, the telephone recorder | `\ensuremath{\recorder}` | `\UnxTrecorder` | `\recorder` is `wasysym`'s, and that package redefines `\int` and other symbols the document may be using; the `wasy-recorder` chunk reads the glyph from the same `wasy` font as a symbol of its own |
//! | U+019E `ƞ` | `\textnrleg` | `\textnrleg`, with the `tipx` chunk | the command is real but is `tipx`'s, not `tipa`'s; only the profile was wrong |
//! | U+0149 `ŉ` | `\nument{149}` | `\textquoteright n` | nothing defines `\nument`. The character is the apostrophe and the letter, which is what `xunicode` writes for it too |
//! | U+025B, U+0390, U+03D2, U+266D, U+266E, U+266F | `\varepsilon`, `\flat`, … | `\varepsilon` and `\flat` marked `MATH`, … | six mathematical spellings were written without the `\ensuremath{…}` wrapper every other one of the table carried, and stopped with "Missing $ inserted" in ordinary text. The wrapper is the mode column's business now, and these six carry `MATH` like their neighbors |
//! | U+0307 and U+0308, the combining dot above and combining diaeresis | `\ensuremath{\dot{}}`, `\ensuremath{\ddot{}}` | no entry | an accent command over an empty group sets the mark *after* the letter, not on it; with no entry the character is one no spelling is known for, which a caller reports. A combining mark that follows a letter Unicode has a composed form for never reaches the table: NFC normalization has already replaced the two by the accented letter |
//! | U+0488 and U+0489, the combining Cyrillic hundred thousands and millions signs | `\cyrchar\cyrhundredthousands`, `\cyrchar\cyrmillions` | no entry | the same, and no encoding of any installed font declares either sign, under that name or another |
//!
//! The `here` column shows the spelling as this table stores it: the bare
//! mathematical form, with the `\ensuremath{…}` of pylatexenc's spelling now
//! carried by the `MATH` mode instead.
//!
//! # Spellings that are still to correct
//!
//! None: every entry compiles and sets its glyph, and the entries that could
//! not are listed as removed in the departures above. A spelling found wrong
//! is corrected here, its departure recorded in that table, and — where it
//! needs something the chunks do not yet offer — a chunk added.
//!
//! # License
//!
//! pylatexenc is distributed under the MIT License:
//!
//! > The MIT License (MIT)
//! >
//! > Copyright (c) 2015 Philippe Faist
//! >
//! > Permission is hereby granted, free of charge, to any person obtaining a
//! > copy of this software and associated documentation files (the
//! > "Software"), to deal in the Software without restriction, including
//! > without limitation the rights to use, copy, modify, merge, publish,
//! > distribute, sublicense, and/or sell copies of the Software, and to permit
//! > persons to whom the Software is furnished to do so, subject to the
//! > following conditions:
//! >
//! > The above copyright notice and this permission notice shall be included
//! > in all copies or substantial portions of the Software.
//! >
//! > THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
//! > OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
//! > MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN
//! > NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
//! > DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
//! > OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE
//! > USE OR OTHER DEALINGS IN THE SOFTWARE.
//!
//! The character map in pylatexenc was itself adapted from latexcodec 0.2, by
//! Peter Troeger (<https://pypi.python.org/pypi/latexcodec>), whose notice
//! pylatexenc keeps and which is reproduced here for the same reason:
//!
//! > latexcodec is a lexer and codec to work with LaTeX code in Python
//! >
//! > Copyright (c) 2011-2014 by Matthias C. M. Troffaes
//! >
//! > Permission is hereby granted, free of charge, to any person obtaining a
//! > copy of this software and associated documentation files (the
//! > "Software"), to deal in the Software without restriction, including
//! > without limitation the rights to use, copy, modify, merge, publish,
//! > distribute, sublicense, and/or sell copies of the Software, and to permit
//! > persons to whom the Software is furnished to do so, subject to the
//! > following conditions:
//! >
//! > The above copyright notice and this permission notice shall be included
//! > in all copies or substantial portions of the Software.
//! >
//! > THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
//! > OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
//! > MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN
//! > NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
//! > DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
//! > OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE
//! > USE OR OTHER DEALINGS IN THE SOFTWARE.

use crate::profile::ProfileIndex;
use crate::replacement_protection::ValueMode;

// The `ProfileIndex` constants the entries name: BUILTINS, FONTENC_T1, ...
use super::needs_profiles::*;

// Short names for the mode column, so that an entry line stays one line.
const TEXT: ValueMode = ValueMode::TextOnly;
const MATH: ValueMode = ValueMode::MathOnly;
const ANY: ValueMode = ValueMode::AnyMode;

/// The builtin entries: 1549 characters, each with its LaTeX spelling, the
/// [`ValueMode`] that spelling is valid in and the [`ProfileIndex`] of the
/// preamble it needs, sorted by character.
///
/// Public and hidden so that the benchmarks can compile the same data in every
/// static table layout. Use the `DEFAULTS`, `NON_ASCII` and `ASCII_SPECIALS`
/// tables instead; this slice is not part of the stable API.
#[doc(hidden)]
pub const ENTRIES: &[(char, &str, ValueMode, ProfileIndex)] = &[
    // BEGIN ENTRIES (generated by tools/migrate_tables.py)
    ('\u{0022}', r#"''"#, ANY, BUILTINS), // QUOTATION MARK
    ('\u{0023}', r#"\#"#, TEXT, BUILTINS), // NUMBER SIGN
    ('\u{0024}', r#"\$"#, TEXT, BUILTINS), // DOLLAR SIGN
    ('\u{0025}', r#"\%"#, TEXT, BUILTINS), // PERCENT SIGN
    ('\u{0026}', r#"\&"#, TEXT, BUILTINS), // AMPERSAND
    ('\u{003C}', r#"<"#, MATH, BUILTINS), // LESS-THAN SIGN
    ('\u{003E}', r#">"#, MATH, BUILTINS), // GREATER-THAN SIGN
    ('\u{005C}', r#"\textbackslash"#, TEXT, BUILTINS), // REVERSE SOLIDUS
    ('\u{005E}', r#"\textasciicircum"#, TEXT, BUILTINS), // CIRCUMFLEX ACCENT
    ('\u{005F}', r#"\_"#, TEXT, BUILTINS), // LOW LINE
    ('\u{007B}', r#"\{"#, TEXT, BUILTINS), // LEFT CURLY BRACKET
    ('\u{007D}', r#"\}"#, TEXT, BUILTINS), // RIGHT CURLY BRACKET
    ('\u{007E}', r#"\textasciitilde"#, TEXT, BUILTINS), // TILDE
    ('\u{00A0}', r#"~"#, ANY, BUILTINS), // NO-BREAK SPACE
    ('\u{00A1}', r#"\textexclamdown"#, TEXT, BUILTINS), // INVERTED EXCLAMATION MARK
    ('\u{00A2}', r#"\textcent"#, TEXT, BUILTINS), // CENT SIGN
    ('\u{00A3}', r#"\textsterling"#, TEXT, BUILTINS), // POUND SIGN
    ('\u{00A4}', r#"\textcurrency"#, TEXT, BUILTINS), // CURRENCY SIGN
    ('\u{00A5}', r#"\textyen"#, TEXT, BUILTINS), // YEN SIGN
    ('\u{00A6}', r#"\textbrokenbar"#, TEXT, BUILTINS), // BROKEN BAR
    ('\u{00A7}', r#"\textsection"#, TEXT, BUILTINS), // SECTION SIGN
    ('\u{00A8}', r#"\textasciidieresis"#, TEXT, BUILTINS), // DIAERESIS
    ('\u{00A9}', r#"\textcopyright"#, TEXT, BUILTINS), // COPYRIGHT SIGN
    ('\u{00AA}', r#"\textordfeminine"#, TEXT, BUILTINS), // FEMININE ORDINAL INDICATOR
    ('\u{00AB}', r#"\guillemotleft"#, TEXT, FONTENC_T1), // LEFT-POINTING DOUBLE ANGLE QUOTATION MARK
    ('\u{00AC}', r#"\textlnot"#, TEXT, BUILTINS), // NOT SIGN
    ('\u{00AD}', r#"\-"#, TEXT, BUILTINS), // SOFT HYPHEN
    ('\u{00AE}', r#"\textregistered"#, TEXT, BUILTINS), // REGISTERED SIGN
    ('\u{00AF}', r#"\textasciimacron"#, TEXT, BUILTINS), // MACRON
    ('\u{00B0}', r#"\textdegree"#, TEXT, BUILTINS), // DEGREE SIGN
    ('\u{00B1}', r#"\pm"#, MATH, BUILTINS), // PLUS-MINUS SIGN
    ('\u{00B2}', r#"\texttwosuperior"#, TEXT, BUILTINS), // SUPERSCRIPT TWO
    ('\u{00B3}', r#"\textthreesuperior"#, TEXT, BUILTINS), // SUPERSCRIPT THREE
    ('\u{00B4}', r#"\textasciiacute"#, TEXT, BUILTINS), // ACUTE ACCENT
    ('\u{00B5}', r#"\textmu"#, TEXT, BUILTINS), // MICRO SIGN
    ('\u{00B6}', r#"\textparagraph"#, TEXT, BUILTINS), // PILCROW SIGN
    ('\u{00B7}', r#"\textperiodcentered"#, TEXT, BUILTINS), // MIDDLE DOT
    ('\u{00B9}', r#"\textonesuperior"#, TEXT, BUILTINS), // SUPERSCRIPT ONE
    ('\u{00BA}', r#"\textordmasculine"#, TEXT, BUILTINS), // MASCULINE ORDINAL INDICATOR
    ('\u{00BB}', r#"\guillemotright"#, TEXT, FONTENC_T1), // RIGHT-POINTING DOUBLE ANGLE QUOTATION MARK
    ('\u{00BC}', r#"\textonequarter"#, TEXT, BUILTINS), // VULGAR FRACTION ONE QUARTER
    ('\u{00BD}', r#"\textonehalf"#, TEXT, BUILTINS), // VULGAR FRACTION ONE HALF
    ('\u{00BE}', r#"\textthreequarters"#, TEXT, BUILTINS), // VULGAR FRACTION THREE QUARTERS
    ('\u{00BF}', r#"\textquestiondown"#, TEXT, BUILTINS), // INVERTED QUESTION MARK
    ('\u{00C0}', r#"\`A"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER A WITH GRAVE
    ('\u{00C1}', r#"\'A"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER A WITH ACUTE
    ('\u{00C2}', r#"\^A"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER A WITH CIRCUMFLEX
    ('\u{00C3}', r#"\~A"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER A WITH TILDE
    ('\u{00C4}', r#"\"A"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER A WITH DIAERESIS
    ('\u{00C5}', r#"\r{A}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER A WITH RING ABOVE
    ('\u{00C6}', r#"\AE"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER AE
    ('\u{00C7}', r#"\c{C}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER C WITH CEDILLA
    ('\u{00C8}', r#"\`E"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER E WITH GRAVE
    ('\u{00C9}', r#"\'E"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER E WITH ACUTE
    ('\u{00CA}', r#"\^E"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER E WITH CIRCUMFLEX
    ('\u{00CB}', r#"\"E"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER E WITH DIAERESIS
    ('\u{00CC}', r#"\`I"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER I WITH GRAVE
    ('\u{00CD}', r#"\'I"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER I WITH ACUTE
    ('\u{00CE}', r#"\^I"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER I WITH CIRCUMFLEX
    ('\u{00CF}', r#"\"I"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER I WITH DIAERESIS
    ('\u{00D0}', r#"\DH"#, TEXT, FONTENC_T1), // LATIN CAPITAL LETTER ETH
    ('\u{00D1}', r#"\~N"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER N WITH TILDE
    ('\u{00D2}', r#"\`O"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER O WITH GRAVE
    ('\u{00D3}', r#"\'O"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER O WITH ACUTE
    ('\u{00D4}', r#"\^O"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER O WITH CIRCUMFLEX
    ('\u{00D5}', r#"\~O"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER O WITH TILDE
    ('\u{00D6}', r#"\"O"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER O WITH DIAERESIS
    ('\u{00D7}', r#"\texttimes"#, TEXT, BUILTINS), // MULTIPLICATION SIGN
    ('\u{00D8}', r#"\O"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER O WITH STROKE
    ('\u{00D9}', r#"\`U"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER U WITH GRAVE
    ('\u{00DA}', r#"\'U"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER U WITH ACUTE
    ('\u{00DB}', r#"\^U"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER U WITH CIRCUMFLEX
    ('\u{00DC}', r#"\"U"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER U WITH DIAERESIS
    ('\u{00DD}', r#"\'Y"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER Y WITH ACUTE
    ('\u{00DE}', r#"\TH"#, TEXT, FONTENC_T1), // LATIN CAPITAL LETTER THORN
    ('\u{00DF}', r#"\ss"#, TEXT, BUILTINS), // LATIN SMALL LETTER SHARP S
    ('\u{00E0}', r#"\`a"#, TEXT, BUILTINS), // LATIN SMALL LETTER A WITH GRAVE
    ('\u{00E1}', r#"\'a"#, TEXT, BUILTINS), // LATIN SMALL LETTER A WITH ACUTE
    ('\u{00E2}', r#"\^a"#, TEXT, BUILTINS), // LATIN SMALL LETTER A WITH CIRCUMFLEX
    ('\u{00E3}', r#"\~a"#, TEXT, BUILTINS), // LATIN SMALL LETTER A WITH TILDE
    ('\u{00E4}', r#"\"a"#, TEXT, BUILTINS), // LATIN SMALL LETTER A WITH DIAERESIS
    ('\u{00E5}', r#"\r{a}"#, TEXT, BUILTINS), // LATIN SMALL LETTER A WITH RING ABOVE
    ('\u{00E6}', r#"\ae"#, TEXT, BUILTINS), // LATIN SMALL LETTER AE
    ('\u{00E7}', r#"\c{c}"#, TEXT, BUILTINS), // LATIN SMALL LETTER C WITH CEDILLA
    ('\u{00E8}', r#"\`e"#, TEXT, BUILTINS), // LATIN SMALL LETTER E WITH GRAVE
    ('\u{00E9}', r#"\'e"#, TEXT, BUILTINS), // LATIN SMALL LETTER E WITH ACUTE
    ('\u{00EA}', r#"\^e"#, TEXT, BUILTINS), // LATIN SMALL LETTER E WITH CIRCUMFLEX
    ('\u{00EB}', r#"\"e"#, TEXT, BUILTINS), // LATIN SMALL LETTER E WITH DIAERESIS
    ('\u{00EC}', r#"\`i"#, TEXT, BUILTINS), // LATIN SMALL LETTER I WITH GRAVE
    ('\u{00ED}', r#"\'i"#, TEXT, BUILTINS), // LATIN SMALL LETTER I WITH ACUTE
    ('\u{00EE}', r#"\^i"#, TEXT, BUILTINS), // LATIN SMALL LETTER I WITH CIRCUMFLEX
    ('\u{00EF}', r#"\"i"#, TEXT, BUILTINS), // LATIN SMALL LETTER I WITH DIAERESIS
    ('\u{00F0}', r#"\dh"#, TEXT, FONTENC_T1), // LATIN SMALL LETTER ETH
    ('\u{00F1}', r#"\~n"#, TEXT, BUILTINS), // LATIN SMALL LETTER N WITH TILDE
    ('\u{00F2}', r#"\`o"#, TEXT, BUILTINS), // LATIN SMALL LETTER O WITH GRAVE
    ('\u{00F3}', r#"\'o"#, TEXT, BUILTINS), // LATIN SMALL LETTER O WITH ACUTE
    ('\u{00F4}', r#"\^o"#, TEXT, BUILTINS), // LATIN SMALL LETTER O WITH CIRCUMFLEX
    ('\u{00F5}', r#"\~o"#, TEXT, BUILTINS), // LATIN SMALL LETTER O WITH TILDE
    ('\u{00F6}', r#"\"o"#, TEXT, BUILTINS), // LATIN SMALL LETTER O WITH DIAERESIS
    ('\u{00F7}', r#"\textdiv"#, TEXT, BUILTINS), // DIVISION SIGN
    ('\u{00F8}', r#"\o"#, TEXT, BUILTINS), // LATIN SMALL LETTER O WITH STROKE
    ('\u{00F9}', r#"\`u"#, TEXT, BUILTINS), // LATIN SMALL LETTER U WITH GRAVE
    ('\u{00FA}', r#"\'u"#, TEXT, BUILTINS), // LATIN SMALL LETTER U WITH ACUTE
    ('\u{00FB}', r#"\^u"#, TEXT, BUILTINS), // LATIN SMALL LETTER U WITH CIRCUMFLEX
    ('\u{00FC}', r#"\"u"#, TEXT, BUILTINS), // LATIN SMALL LETTER U WITH DIAERESIS
    ('\u{00FD}', r#"\'y"#, TEXT, BUILTINS), // LATIN SMALL LETTER Y WITH ACUTE
    ('\u{00FE}', r#"\th"#, TEXT, FONTENC_T1), // LATIN SMALL LETTER THORN
    ('\u{00FF}', r#"\"y"#, TEXT, BUILTINS), // LATIN SMALL LETTER Y WITH DIAERESIS
    ('\u{0100}', r#"\={A}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER A WITH MACRON
    ('\u{0101}', r#"\={a}"#, TEXT, BUILTINS), // LATIN SMALL LETTER A WITH MACRON
    ('\u{0102}', r#"\u{A}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER A WITH BREVE
    ('\u{0103}', r#"\u{a}"#, TEXT, BUILTINS), // LATIN SMALL LETTER A WITH BREVE
    ('\u{0104}', r#"\k{A}"#, TEXT, FONTENC_T1), // LATIN CAPITAL LETTER A WITH OGONEK
    ('\u{0105}', r#"\k{a}"#, TEXT, FONTENC_T1), // LATIN SMALL LETTER A WITH OGONEK
    ('\u{0106}', r#"\'C"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER C WITH ACUTE
    ('\u{0107}', r#"\'c"#, TEXT, BUILTINS), // LATIN SMALL LETTER C WITH ACUTE
    ('\u{0108}', r#"\^{C}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER C WITH CIRCUMFLEX
    ('\u{0109}', r#"\^{c}"#, TEXT, BUILTINS), // LATIN SMALL LETTER C WITH CIRCUMFLEX
    ('\u{010A}', r#"\.{C}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER C WITH DOT ABOVE
    ('\u{010B}', r#"\.{c}"#, TEXT, BUILTINS), // LATIN SMALL LETTER C WITH DOT ABOVE
    ('\u{010C}', r#"\v{C}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER C WITH CARON
    ('\u{010D}', r#"\v{c}"#, TEXT, BUILTINS), // LATIN SMALL LETTER C WITH CARON
    ('\u{010E}', r#"\v{D}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER D WITH CARON
    ('\u{010F}', r#"\v{d}"#, TEXT, BUILTINS), // LATIN SMALL LETTER D WITH CARON
    ('\u{0110}', r#"\DJ"#, TEXT, FONTENC_T1), // LATIN CAPITAL LETTER D WITH STROKE
    ('\u{0111}', r#"\dj"#, TEXT, FONTENC_T1), // LATIN SMALL LETTER D WITH STROKE
    ('\u{0112}', r#"\={E}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER E WITH MACRON
    ('\u{0113}', r#"\={e}"#, TEXT, BUILTINS), // LATIN SMALL LETTER E WITH MACRON
    ('\u{0114}', r#"\u{E}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER E WITH BREVE
    ('\u{0115}', r#"\u{e}"#, TEXT, BUILTINS), // LATIN SMALL LETTER E WITH BREVE
    ('\u{0116}', r#"\.{E}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER E WITH DOT ABOVE
    ('\u{0117}', r#"\.{e}"#, TEXT, BUILTINS), // LATIN SMALL LETTER E WITH DOT ABOVE
    ('\u{0118}', r#"\k{E}"#, TEXT, FONTENC_T1), // LATIN CAPITAL LETTER E WITH OGONEK
    ('\u{0119}', r#"\k{e}"#, TEXT, FONTENC_T1), // LATIN SMALL LETTER E WITH OGONEK
    ('\u{011A}', r#"\v{E}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER E WITH CARON
    ('\u{011B}', r#"\v{e}"#, TEXT, BUILTINS), // LATIN SMALL LETTER E WITH CARON
    ('\u{011C}', r#"\^{G}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER G WITH CIRCUMFLEX
    ('\u{011D}', r#"\^{g}"#, TEXT, BUILTINS), // LATIN SMALL LETTER G WITH CIRCUMFLEX
    ('\u{011E}', r#"\u{G}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER G WITH BREVE
    ('\u{011F}', r#"\u{g}"#, TEXT, BUILTINS), // LATIN SMALL LETTER G WITH BREVE
    ('\u{0120}', r#"\.{G}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER G WITH DOT ABOVE
    ('\u{0121}', r#"\.{g}"#, TEXT, BUILTINS), // LATIN SMALL LETTER G WITH DOT ABOVE
    ('\u{0122}', r#"\c{G}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER G WITH CEDILLA
    ('\u{0123}', r#"\c{g}"#, TEXT, BUILTINS), // LATIN SMALL LETTER G WITH CEDILLA
    ('\u{0124}', r#"\^{H}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER H WITH CIRCUMFLEX
    ('\u{0125}', r#"\^{h}"#, TEXT, BUILTINS), // LATIN SMALL LETTER H WITH CIRCUMFLEX
    ('\u{0126}', r#"\={H}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER H WITH STROKE
    ('\u{0127}', r#"\={h}"#, TEXT, BUILTINS), // LATIN SMALL LETTER H WITH STROKE
    ('\u{0128}', r#"\~{I}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER I WITH TILDE
    ('\u{0129}', r#"\~{i}"#, TEXT, BUILTINS), // LATIN SMALL LETTER I WITH TILDE
    ('\u{012A}', r#"\={I}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER I WITH MACRON
    ('\u{012B}', r#"\={i}"#, TEXT, BUILTINS), // LATIN SMALL LETTER I WITH MACRON
    ('\u{012C}', r#"\u{I}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER I WITH BREVE
    ('\u{012D}', r#"\u{i}"#, TEXT, BUILTINS), // LATIN SMALL LETTER I WITH BREVE
    ('\u{012E}', r#"\k{I}"#, TEXT, FONTENC_T1), // LATIN CAPITAL LETTER I WITH OGONEK
    ('\u{012F}', r#"\k{i}"#, TEXT, FONTENC_T1), // LATIN SMALL LETTER I WITH OGONEK
    ('\u{0130}', r#"\.I"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER I WITH DOT ABOVE
    ('\u{0131}', r#"\i"#, TEXT, BUILTINS), // LATIN SMALL LETTER DOTLESS I
    ('\u{0132}', r#"\IJ"#, TEXT, BUILTINS), // LATIN CAPITAL LIGATURE IJ
    ('\u{0133}', r#"\ij"#, TEXT, BUILTINS), // LATIN SMALL LIGATURE IJ
    ('\u{0134}', r#"\^{J}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER J WITH CIRCUMFLEX
    ('\u{0135}', r#"\^{j}"#, TEXT, BUILTINS), // LATIN SMALL LETTER J WITH CIRCUMFLEX
    ('\u{0136}', r#"\c{K}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER K WITH CEDILLA
    ('\u{0137}', r#"\c{k}"#, TEXT, BUILTINS), // LATIN SMALL LETTER K WITH CEDILLA
    ('\u{0138}', r#"\textsc{k}"#, TEXT, BUILTINS), // LATIN SMALL LETTER KRA
    ('\u{0139}', r#"\'L"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER L WITH ACUTE
    ('\u{013A}', r#"\'l"#, TEXT, BUILTINS), // LATIN SMALL LETTER L WITH ACUTE
    ('\u{013B}', r#"\c{L}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER L WITH CEDILLA
    ('\u{013C}', r#"\c{l}"#, TEXT, BUILTINS), // LATIN SMALL LETTER L WITH CEDILLA
    ('\u{013D}', r#"\v{L}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER L WITH CARON
    ('\u{013E}', r#"\v{l}"#, TEXT, BUILTINS), // LATIN SMALL LETTER L WITH CARON
    ('\u{013F}', r#"\.{L}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER L WITH MIDDLE DOT
    ('\u{0140}', r#"\.{l}"#, TEXT, BUILTINS), // LATIN SMALL LETTER L WITH MIDDLE DOT
    ('\u{0141}', r#"\L"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER L WITH STROKE
    ('\u{0142}', r#"\l"#, TEXT, BUILTINS), // LATIN SMALL LETTER L WITH STROKE
    ('\u{0143}', r#"\'N"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER N WITH ACUTE
    ('\u{0144}', r#"\'n"#, TEXT, BUILTINS), // LATIN SMALL LETTER N WITH ACUTE
    ('\u{0145}', r#"\c{N}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER N WITH CEDILLA
    ('\u{0146}', r#"\c{n}"#, TEXT, BUILTINS), // LATIN SMALL LETTER N WITH CEDILLA
    ('\u{0147}', r#"\v{N}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER N WITH CARON
    ('\u{0148}', r#"\v{n}"#, TEXT, BUILTINS), // LATIN SMALL LETTER N WITH CARON
    ('\u{0149}', r#"\textquoteright n"#, TEXT, BUILTINS), // LATIN SMALL LETTER N PRECEDED BY APOSTROPHE
    ('\u{014A}', r#"\NG"#, TEXT, FONTENC_T1), // LATIN CAPITAL LETTER ENG
    ('\u{014B}', r#"\ng"#, TEXT, FONTENC_T1), // LATIN SMALL LETTER ENG
    ('\u{014C}', r#"\={O}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER O WITH MACRON
    ('\u{014D}', r#"\={o}"#, TEXT, BUILTINS), // LATIN SMALL LETTER O WITH MACRON
    ('\u{014E}', r#"\u{O}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER O WITH BREVE
    ('\u{014F}', r#"\u{o}"#, TEXT, BUILTINS), // LATIN SMALL LETTER O WITH BREVE
    ('\u{0150}', r#"\H{O}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER O WITH DOUBLE ACUTE
    ('\u{0151}', r#"\H{o}"#, TEXT, BUILTINS), // LATIN SMALL LETTER O WITH DOUBLE ACUTE
    ('\u{0152}', r#"\OE"#, TEXT, BUILTINS), // LATIN CAPITAL LIGATURE OE
    ('\u{0153}', r#"\oe"#, TEXT, BUILTINS), // LATIN SMALL LIGATURE OE
    ('\u{0154}', r#"\'R"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER R WITH ACUTE
    ('\u{0155}', r#"\'r"#, TEXT, BUILTINS), // LATIN SMALL LETTER R WITH ACUTE
    ('\u{0156}', r#"\c{R}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER R WITH CEDILLA
    ('\u{0157}', r#"\c{r}"#, TEXT, BUILTINS), // LATIN SMALL LETTER R WITH CEDILLA
    ('\u{0158}', r#"\v{R}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER R WITH CARON
    ('\u{0159}', r#"\v{r}"#, TEXT, BUILTINS), // LATIN SMALL LETTER R WITH CARON
    ('\u{015A}', r#"\'S"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER S WITH ACUTE
    ('\u{015B}', r#"\'s"#, TEXT, BUILTINS), // LATIN SMALL LETTER S WITH ACUTE
    ('\u{015C}', r#"\^{S}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER S WITH CIRCUMFLEX
    ('\u{015D}', r#"\^{s}"#, TEXT, BUILTINS), // LATIN SMALL LETTER S WITH CIRCUMFLEX
    ('\u{015E}', r#"\c{S}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER S WITH CEDILLA
    ('\u{015F}', r#"\c{s}"#, TEXT, BUILTINS), // LATIN SMALL LETTER S WITH CEDILLA
    ('\u{0160}', r#"\v{S}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER S WITH CARON
    ('\u{0161}', r#"\v{s}"#, TEXT, BUILTINS), // LATIN SMALL LETTER S WITH CARON
    ('\u{0162}', r#"\c{T}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER T WITH CEDILLA
    ('\u{0163}', r#"\c{t}"#, TEXT, BUILTINS), // LATIN SMALL LETTER T WITH CEDILLA
    ('\u{0164}', r#"\v{T}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER T WITH CARON
    ('\u{0165}', r#"\v{t}"#, TEXT, BUILTINS), // LATIN SMALL LETTER T WITH CARON
    ('\u{0166}', r#"\={T}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER T WITH STROKE
    ('\u{0167}', r#"\={t}"#, TEXT, BUILTINS), // LATIN SMALL LETTER T WITH STROKE
    ('\u{0168}', r#"\~{U}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER U WITH TILDE
    ('\u{0169}', r#"\~{u}"#, TEXT, BUILTINS), // LATIN SMALL LETTER U WITH TILDE
    ('\u{016A}', r#"\={U}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER U WITH MACRON
    ('\u{016B}', r#"\={u}"#, TEXT, BUILTINS), // LATIN SMALL LETTER U WITH MACRON
    ('\u{016C}', r#"\u{U}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER U WITH BREVE
    ('\u{016D}', r#"\u{u}"#, TEXT, BUILTINS), // LATIN SMALL LETTER U WITH BREVE
    ('\u{016E}', r#"\r{U}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER U WITH RING ABOVE
    ('\u{016F}', r#"\r{u}"#, TEXT, BUILTINS), // LATIN SMALL LETTER U WITH RING ABOVE
    ('\u{0170}', r#"\'{U}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER U WITH DOUBLE ACUTE
    ('\u{0171}', r#"\'{u}"#, TEXT, BUILTINS), // LATIN SMALL LETTER U WITH DOUBLE ACUTE
    ('\u{0172}', r#"\k{U}"#, TEXT, FONTENC_T1), // LATIN CAPITAL LETTER U WITH OGONEK
    ('\u{0173}', r#"\k{u}"#, TEXT, FONTENC_T1), // LATIN SMALL LETTER U WITH OGONEK
    ('\u{0174}', r#"\^{W}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER W WITH CIRCUMFLEX
    ('\u{0175}', r#"\^{w}"#, TEXT, BUILTINS), // LATIN SMALL LETTER W WITH CIRCUMFLEX
    ('\u{0176}', r#"\^{Y}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER Y WITH CIRCUMFLEX
    ('\u{0177}', r#"\^{y}"#, TEXT, BUILTINS), // LATIN SMALL LETTER Y WITH CIRCUMFLEX
    ('\u{0178}', r#"\"Y"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER Y WITH DIAERESIS
    ('\u{0179}', r#"\'Z"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER Z WITH ACUTE
    ('\u{017A}', r#"\'z"#, TEXT, BUILTINS), // LATIN SMALL LETTER Z WITH ACUTE
    ('\u{017B}', r#"\.Z"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER Z WITH DOT ABOVE
    ('\u{017C}', r#"\.z"#, TEXT, BUILTINS), // LATIN SMALL LETTER Z WITH DOT ABOVE
    ('\u{017D}', r#"\v{Z}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER Z WITH CARON
    ('\u{017E}', r#"\v{z}"#, TEXT, BUILTINS), // LATIN SMALL LETTER Z WITH CARON
    ('\u{0192}', r#"\textflorin"#, TEXT, BUILTINS), // LATIN SMALL LETTER F WITH HOOK
    ('\u{0195}', r#"\texthvlig"#, TEXT, TIPA), // LATIN SMALL LETTER HV
    ('\u{019E}', r#"\textnrleg"#, TEXT, TIPX), // LATIN SMALL LETTER N WITH LONG RIGHT LEG
    ('\u{01E7}', r#"\v{g}"#, TEXT, BUILTINS), // LATIN SMALL LETTER G WITH CARON
    ('\u{01F5}', r#"\'{g}"#, TEXT, BUILTINS), // LATIN SMALL LETTER G WITH ACUTE
    ('\u{0228}', r#"\c{E}"#, TEXT, BUILTINS), // LATIN CAPITAL LETTER E WITH CEDILLA
    ('\u{0229}', r#"\c{e}"#, TEXT, BUILTINS), // LATIN SMALL LETTER E WITH CEDILLA
    ('\u{0259}', r#"\textschwa"#, TEXT, TIPA), // LATIN SMALL LETTER SCHWA
    ('\u{025B}', r#"\varepsilon"#, MATH, BUILTINS), // LATIN SMALL LETTER OPEN E
    ('\u{0278}', r#"\textphi"#, TEXT, TIPA), // LATIN SMALL LETTER PHI
    ('\u{0294}', r#"\textglotstop"#, TEXT, TIPA), // LATIN LETTER GLOTTAL STOP
    ('\u{029E}', r#"\textturnk"#, TEXT, TIPA), // LATIN SMALL LETTER TURNED K
    ('\u{02B7}', r#"\textsuperscript{w}"#, TEXT, BUILTINS), // MODIFIER LETTER SMALL W
    ('\u{02BC}', r#"'"#, ANY, BUILTINS), // MODIFIER LETTER APOSTROPHE
    ('\u{02C6}', r#"\textasciicircum"#, TEXT, BUILTINS), // MODIFIER LETTER CIRCUMFLEX ACCENT
    ('\u{02C7}', r#"\textasciicaron"#, TEXT, BUILTINS), // CARON
    ('\u{02D8}', r#"\textasciibreve"#, TEXT, BUILTINS), // BREVE
    ('\u{02D9}', r#"\textperiodcentered"#, TEXT, BUILTINS), // DOT ABOVE
    ('\u{02DA}', r#"\r{}"#, TEXT, BUILTINS), // RING ABOVE
    ('\u{02DB}', r#"\k{}"#, TEXT, FONTENC_T1), // OGONEK
    ('\u{02DC}', r#"\textasciitilde"#, TEXT, BUILTINS), // SMALL TILDE
    ('\u{02DD}', r#"\textacutedbl"#, TEXT, BUILTINS), // DOUBLE ACUTE ACCENT
    ('\u{0386}', r#"\'{}A"#, TEXT, BUILTINS), // GREEK CAPITAL LETTER ALPHA WITH TONOS
    ('\u{0388}', r#"\'{}E"#, TEXT, BUILTINS), // GREEK CAPITAL LETTER EPSILON WITH TONOS
    ('\u{0389}', r#"\'{}H"#, TEXT, BUILTINS), // GREEK CAPITAL LETTER ETA WITH TONOS
    ('\u{038A}', r#"\'{}I"#, TEXT, BUILTINS), // GREEK CAPITAL LETTER IOTA WITH TONOS
    ('\u{038C}', r#"\'{}O"#, TEXT, BUILTINS), // GREEK CAPITAL LETTER OMICRON WITH TONOS
    ('\u{038E}', r#"\'{}Y"#, TEXT, BUILTINS), // GREEK CAPITAL LETTER UPSILON WITH TONOS
    ('\u{038F}', r#"\'{}\ensuremath{\Omega}"#, TEXT, BUILTINS), // GREEK CAPITAL LETTER OMEGA WITH TONOS
    ('\u{0390}', r#"\acute{\ddot{\iota}}"#, MATH, BUILTINS), // GREEK SMALL LETTER IOTA WITH DIALYTIKA AND TONOS
    ('\u{0391}', r#"A"#, ANY, BUILTINS), // GREEK CAPITAL LETTER ALPHA
    ('\u{0392}', r#"B"#, ANY, BUILTINS), // GREEK CAPITAL LETTER BETA
    ('\u{0393}', r#"\Gamma"#, MATH, BUILTINS), // GREEK CAPITAL LETTER GAMMA
    ('\u{0394}', r#"\Delta"#, MATH, BUILTINS), // GREEK CAPITAL LETTER DELTA
    ('\u{0395}', r#"E"#, ANY, BUILTINS), // GREEK CAPITAL LETTER EPSILON
    ('\u{0396}', r#"Z"#, ANY, BUILTINS), // GREEK CAPITAL LETTER ZETA
    ('\u{0397}', r#"H"#, ANY, BUILTINS), // GREEK CAPITAL LETTER ETA
    ('\u{0398}', r#"\Theta"#, MATH, BUILTINS), // GREEK CAPITAL LETTER THETA
    ('\u{0399}', r#"I"#, ANY, BUILTINS), // GREEK CAPITAL LETTER IOTA
    ('\u{039A}', r#"K"#, ANY, BUILTINS), // GREEK CAPITAL LETTER KAPPA
    ('\u{039B}', r#"\Lambda"#, MATH, BUILTINS), // GREEK CAPITAL LETTER LAMDA
    ('\u{039C}', r#"M"#, ANY, BUILTINS), // GREEK CAPITAL LETTER MU
    ('\u{039D}', r#"N"#, ANY, BUILTINS), // GREEK CAPITAL LETTER NU
    ('\u{039E}', r#"\Xi"#, MATH, BUILTINS), // GREEK CAPITAL LETTER XI
    ('\u{039F}', r#"O"#, ANY, BUILTINS), // GREEK CAPITAL LETTER OMICRON
    ('\u{03A0}', r#"\Pi"#, MATH, BUILTINS), // GREEK CAPITAL LETTER PI
    ('\u{03A1}', r#"P"#, ANY, BUILTINS), // GREEK CAPITAL LETTER RHO
    ('\u{03A3}', r#"\Sigma"#, MATH, BUILTINS), // GREEK CAPITAL LETTER SIGMA
    ('\u{03A4}', r#"T"#, ANY, BUILTINS), // GREEK CAPITAL LETTER TAU
    ('\u{03A5}', r#"\Upsilon"#, MATH, BUILTINS), // GREEK CAPITAL LETTER UPSILON
    ('\u{03A6}', r#"\Phi"#, MATH, BUILTINS), // GREEK CAPITAL LETTER PHI
    ('\u{03A7}', r#"X"#, ANY, BUILTINS), // GREEK CAPITAL LETTER CHI
    ('\u{03A8}', r#"\Psi"#, MATH, BUILTINS), // GREEK CAPITAL LETTER PSI
    ('\u{03A9}', r#"\Omega"#, MATH, BUILTINS), // GREEK CAPITAL LETTER OMEGA
    ('\u{03AA}', r#"\ddot{I}"#, MATH, BUILTINS), // GREEK CAPITAL LETTER IOTA WITH DIALYTIKA
    ('\u{03AB}', r#"\ddot{Y}"#, MATH, BUILTINS), // GREEK CAPITAL LETTER UPSILON WITH DIALYTIKA
    ('\u{03AC}', r#"\acute\alpha"#, MATH, BUILTINS), // GREEK SMALL LETTER ALPHA WITH TONOS
    ('\u{03AD}', r#"\acute\epsilon"#, MATH, BUILTINS), // GREEK SMALL LETTER EPSILON WITH TONOS
    ('\u{03AE}', r#"\acute\eta"#, MATH, BUILTINS), // GREEK SMALL LETTER ETA WITH TONOS
    ('\u{03AF}', r#"\acute\iota"#, MATH, BUILTINS), // GREEK SMALL LETTER IOTA WITH TONOS
    ('\u{03B0}', r#"\acute{\ddot{\upsilon}}"#, MATH, BUILTINS), // GREEK SMALL LETTER UPSILON WITH DIALYTIKA AND TONOS
    ('\u{03B1}', r#"\alpha"#, MATH, BUILTINS), // GREEK SMALL LETTER ALPHA
    ('\u{03B2}', r#"\beta"#, MATH, BUILTINS), // GREEK SMALL LETTER BETA
    ('\u{03B3}', r#"\gamma"#, MATH, BUILTINS), // GREEK SMALL LETTER GAMMA
    ('\u{03B4}', r#"\delta"#, MATH, BUILTINS), // GREEK SMALL LETTER DELTA
    ('\u{03B5}', r#"\varepsilon"#, MATH, BUILTINS), // GREEK SMALL LETTER EPSILON
    ('\u{03B6}', r#"\zeta"#, MATH, BUILTINS), // GREEK SMALL LETTER ZETA
    ('\u{03B7}', r#"\eta"#, MATH, BUILTINS), // GREEK SMALL LETTER ETA
    ('\u{03B8}', r#"\theta"#, MATH, BUILTINS), // GREEK SMALL LETTER THETA
    ('\u{03B9}', r#"\iota"#, MATH, BUILTINS), // GREEK SMALL LETTER IOTA
    ('\u{03BA}', r#"\kappa"#, MATH, BUILTINS), // GREEK SMALL LETTER KAPPA
    ('\u{03BB}', r#"\lambda"#, MATH, BUILTINS), // GREEK SMALL LETTER LAMDA
    ('\u{03BC}', r#"\mu"#, MATH, BUILTINS), // GREEK SMALL LETTER MU
    ('\u{03BD}', r#"\nu"#, MATH, BUILTINS), // GREEK SMALL LETTER NU
    ('\u{03BE}', r#"\xi"#, MATH, BUILTINS), // GREEK SMALL LETTER XI
    ('\u{03BF}', r#"o"#, ANY, BUILTINS), // GREEK SMALL LETTER OMICRON
    ('\u{03C0}', r#"\pi"#, MATH, BUILTINS), // GREEK SMALL LETTER PI
    ('\u{03C1}', r#"\rho"#, MATH, BUILTINS), // GREEK SMALL LETTER RHO
    ('\u{03C2}', r#"\varsigma"#, MATH, BUILTINS), // GREEK SMALL LETTER FINAL SIGMA
    ('\u{03C3}', r#"\sigma"#, MATH, BUILTINS), // GREEK SMALL LETTER SIGMA
    ('\u{03C4}', r#"\tau"#, MATH, BUILTINS), // GREEK SMALL LETTER TAU
    ('\u{03C5}', r#"\upsilon"#, MATH, BUILTINS), // GREEK SMALL LETTER UPSILON
    ('\u{03C6}', r#"\varphi"#, MATH, BUILTINS), // GREEK SMALL LETTER PHI
    ('\u{03C7}', r#"\chi"#, MATH, BUILTINS), // GREEK SMALL LETTER CHI
    ('\u{03C8}', r#"\psi"#, MATH, BUILTINS), // GREEK SMALL LETTER PSI
    ('\u{03C9}', r#"\omega"#, MATH, BUILTINS), // GREEK SMALL LETTER OMEGA
    ('\u{03CA}', r#"\ddot\iota"#, MATH, BUILTINS), // GREEK SMALL LETTER IOTA WITH DIALYTIKA
    ('\u{03CB}', r#"\ddot{\upsilon}"#, MATH, BUILTINS), // GREEK SMALL LETTER UPSILON WITH DIALYTIKA
    ('\u{03CC}', r#"\'{o}"#, TEXT, BUILTINS), // GREEK SMALL LETTER OMICRON WITH TONOS
    ('\u{03CD}', r#"\acute\upsilon"#, MATH, BUILTINS), // GREEK SMALL LETTER UPSILON WITH TONOS
    ('\u{03CE}', r#"\acute\omega"#, MATH, BUILTINS), // GREEK SMALL LETTER OMEGA WITH TONOS
    ('\u{03D1}', r#"\vartheta"#, MATH, BUILTINS), // GREEK THETA SYMBOL
    ('\u{03D2}', r#"\Upsilon"#, MATH, BUILTINS), // GREEK UPSILON WITH HOOK SYMBOL
    ('\u{03D5}', r#"\phi"#, MATH, BUILTINS), // GREEK PHI SYMBOL
    ('\u{03D6}', r#"\varpi"#, MATH, BUILTINS), // GREEK PI SYMBOL
    ('\u{03F0}', r#"\varkappa"#, MATH, AMSSYMB), // GREEK KAPPA SYMBOL
    ('\u{03F1}', r#"\varrho"#, MATH, BUILTINS), // GREEK RHO SYMBOL
    ('\u{03F5}', r#"\epsilon"#, MATH, BUILTINS), // GREEK LUNATE EPSILON SYMBOL
    ('\u{03F6}', r#"\backepsilon"#, MATH, AMSSYMB), // GREEK REVERSED LUNATE EPSILON SYMBOL
    ('\u{0400}', r#"{\fontencoding{T2A}\selectfont\`\CYRE}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER IE WITH GRAVE
    ('\u{0401}', r#"{\fontencoding{T2A}\selectfont\CYRYO}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER IO
    ('\u{0402}', r#"{\fontencoding{T2A}\selectfont\CYRDJE}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER DJE
    ('\u{0403}', r#"{\fontencoding{T2A}\selectfont\`\CYRG}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER GJE
    ('\u{0404}', r#"{\fontencoding{T2A}\selectfont\CYRIE}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER UKRAINIAN IE
    ('\u{0405}', r#"{\fontencoding{T2A}\selectfont\CYRDZE}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER DZE
    ('\u{0406}', r#"{\fontencoding{T2A}\selectfont\CYRII}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER BYELORUSSIAN-UKRAINIAN I
    ('\u{0407}', r#"{\fontencoding{T2A}\selectfont\CYRYI}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER YI
    ('\u{0408}', r#"{\fontencoding{T2A}\selectfont\CYRJE}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER JE
    ('\u{0409}', r#"{\fontencoding{T2A}\selectfont\CYRLJE}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER LJE
    ('\u{040A}', r#"{\fontencoding{T2A}\selectfont\CYRNJE}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER NJE
    ('\u{040B}', r#"{\fontencoding{T2A}\selectfont\CYRTSHE}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER TSHE
    ('\u{040C}', r#"{\fontencoding{T2A}\selectfont\`\CYRK}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER KJE
    ('\u{040D}', r#"{\fontencoding{T2A}\selectfont\`\CYRI}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER I WITH GRAVE
    ('\u{040E}', r#"{\fontencoding{T2A}\selectfont\CYRUSHRT}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER SHORT U
    ('\u{040F}', r#"{\fontencoding{T2A}\selectfont\CYRDZHE}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER DZHE
    ('\u{0410}', r#"{\fontencoding{T2A}\selectfont\CYRA}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER A
    ('\u{0411}', r#"{\fontencoding{T2A}\selectfont\CYRB}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER BE
    ('\u{0412}', r#"{\fontencoding{T2A}\selectfont\CYRV}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER VE
    ('\u{0413}', r#"{\fontencoding{T2A}\selectfont\CYRG}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER GHE
    ('\u{0414}', r#"{\fontencoding{T2A}\selectfont\CYRD}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER DE
    ('\u{0415}', r#"{\fontencoding{T2A}\selectfont\CYRE}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER IE
    ('\u{0416}', r#"{\fontencoding{T2A}\selectfont\CYRZH}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER ZHE
    ('\u{0417}', r#"{\fontencoding{T2A}\selectfont\CYRZ}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER ZE
    ('\u{0418}', r#"{\fontencoding{T2A}\selectfont\CYRI}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER I
    ('\u{0419}', r#"{\fontencoding{T2A}\selectfont\CYRISHRT}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER SHORT I
    ('\u{041A}', r#"{\fontencoding{T2A}\selectfont\CYRK}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER KA
    ('\u{041B}', r#"{\fontencoding{T2A}\selectfont\CYRL}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER EL
    ('\u{041C}', r#"{\fontencoding{T2A}\selectfont\CYRM}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER EM
    ('\u{041D}', r#"{\fontencoding{T2A}\selectfont\CYRN}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER EN
    ('\u{041E}', r#"{\fontencoding{T2A}\selectfont\CYRO}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER O
    ('\u{041F}', r#"{\fontencoding{T2A}\selectfont\CYRP}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER PE
    ('\u{0420}', r#"{\fontencoding{T2A}\selectfont\CYRR}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER ER
    ('\u{0421}', r#"{\fontencoding{T2A}\selectfont\CYRS}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER ES
    ('\u{0422}', r#"{\fontencoding{T2A}\selectfont\CYRT}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER TE
    ('\u{0423}', r#"{\fontencoding{T2A}\selectfont\CYRU}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER U
    ('\u{0424}', r#"{\fontencoding{T2A}\selectfont\CYRF}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER EF
    ('\u{0425}', r#"{\fontencoding{T2A}\selectfont\CYRH}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER HA
    ('\u{0426}', r#"{\fontencoding{T2A}\selectfont\CYRC}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER TSE
    ('\u{0427}', r#"{\fontencoding{T2A}\selectfont\CYRCH}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER CHE
    ('\u{0428}', r#"{\fontencoding{T2A}\selectfont\CYRSH}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER SHA
    ('\u{0429}', r#"{\fontencoding{T2A}\selectfont\CYRSHCH}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER SHCHA
    ('\u{042A}', r#"{\fontencoding{T2A}\selectfont\CYRHRDSN}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER HARD SIGN
    ('\u{042B}', r#"{\fontencoding{T2A}\selectfont\CYRERY}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER YERU
    ('\u{042C}', r#"{\fontencoding{T2A}\selectfont\CYRSFTSN}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER SOFT SIGN
    ('\u{042D}', r#"{\fontencoding{T2A}\selectfont\CYREREV}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER E
    ('\u{042E}', r#"{\fontencoding{T2A}\selectfont\CYRYU}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER YU
    ('\u{042F}', r#"{\fontencoding{T2A}\selectfont\CYRYA}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER YA
    ('\u{0430}', r#"{\fontencoding{T2A}\selectfont\cyra}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER A
    ('\u{0431}', r#"{\fontencoding{T2A}\selectfont\cyrb}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER BE
    ('\u{0432}', r#"{\fontencoding{T2A}\selectfont\cyrv}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER VE
    ('\u{0433}', r#"{\fontencoding{T2A}\selectfont\cyrg}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER GHE
    ('\u{0434}', r#"{\fontencoding{T2A}\selectfont\cyrd}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER DE
    ('\u{0435}', r#"{\fontencoding{T2A}\selectfont\cyre}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER IE
    ('\u{0436}', r#"{\fontencoding{T2A}\selectfont\cyrzh}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER ZHE
    ('\u{0437}', r#"{\fontencoding{T2A}\selectfont\cyrz}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER ZE
    ('\u{0438}', r#"{\fontencoding{T2A}\selectfont\cyri}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER I
    ('\u{0439}', r#"{\fontencoding{T2A}\selectfont\cyrishrt}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER SHORT I
    ('\u{043A}', r#"{\fontencoding{T2A}\selectfont\cyrk}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER KA
    ('\u{043B}', r#"{\fontencoding{T2A}\selectfont\cyrl}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER EL
    ('\u{043C}', r#"{\fontencoding{T2A}\selectfont\cyrm}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER EM
    ('\u{043D}', r#"{\fontencoding{T2A}\selectfont\cyrn}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER EN
    ('\u{043E}', r#"{\fontencoding{T2A}\selectfont\cyro}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER O
    ('\u{043F}', r#"{\fontencoding{T2A}\selectfont\cyrp}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER PE
    ('\u{0440}', r#"{\fontencoding{T2A}\selectfont\cyrr}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER ER
    ('\u{0441}', r#"{\fontencoding{T2A}\selectfont\cyrs}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER ES
    ('\u{0442}', r#"{\fontencoding{T2A}\selectfont\cyrt}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER TE
    ('\u{0443}', r#"{\fontencoding{T2A}\selectfont\cyru}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER U
    ('\u{0444}', r#"{\fontencoding{T2A}\selectfont\cyrf}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER EF
    ('\u{0445}', r#"{\fontencoding{T2A}\selectfont\cyrh}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER HA
    ('\u{0446}', r#"{\fontencoding{T2A}\selectfont\cyrc}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER TSE
    ('\u{0447}', r#"{\fontencoding{T2A}\selectfont\cyrch}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER CHE
    ('\u{0448}', r#"{\fontencoding{T2A}\selectfont\cyrsh}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER SHA
    ('\u{0449}', r#"{\fontencoding{T2A}\selectfont\cyrshch}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER SHCHA
    ('\u{044A}', r#"{\fontencoding{T2A}\selectfont\cyrhrdsn}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER HARD SIGN
    ('\u{044B}', r#"{\fontencoding{T2A}\selectfont\cyrery}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER YERU
    ('\u{044C}', r#"{\fontencoding{T2A}\selectfont\cyrsftsn}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER SOFT SIGN
    ('\u{044D}', r#"{\fontencoding{T2A}\selectfont\cyrerev}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER E
    ('\u{044E}', r#"{\fontencoding{T2A}\selectfont\cyryu}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER YU
    ('\u{044F}', r#"{\fontencoding{T2A}\selectfont\cyrya}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER YA
    ('\u{0450}', r#"{\fontencoding{T2A}\selectfont\`\cyre}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER IE WITH GRAVE
    ('\u{0451}', r#"{\fontencoding{T2A}\selectfont\cyryo}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER IO
    ('\u{0452}', r#"{\fontencoding{T2A}\selectfont\cyrdje}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER DJE
    ('\u{0453}', r#"{\fontencoding{T2A}\selectfont\`\cyrg}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER GJE
    ('\u{0454}', r#"{\fontencoding{T2A}\selectfont\cyrie}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER UKRAINIAN IE
    ('\u{0455}', r#"{\fontencoding{T2A}\selectfont\cyrdze}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER DZE
    ('\u{0456}', r#"{\fontencoding{T2A}\selectfont\cyrii}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER BYELORUSSIAN-UKRAINIAN I
    ('\u{0457}', r#"{\fontencoding{T2A}\selectfont\cyryi}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER YI
    ('\u{0458}', r#"{\fontencoding{T2A}\selectfont\cyrje}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER JE
    ('\u{0459}', r#"{\fontencoding{T2A}\selectfont\cyrlje}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER LJE
    ('\u{045A}', r#"{\fontencoding{T2A}\selectfont\cyrnje}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER NJE
    ('\u{045B}', r#"{\fontencoding{T2A}\selectfont\cyrtshe}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER TSHE
    ('\u{045C}', r#"{\fontencoding{T2A}\selectfont\`\cyrk}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER KJE
    ('\u{045D}', r#"{\fontencoding{T2A}\selectfont\`\cyri}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER I WITH GRAVE
    ('\u{045E}', r#"{\fontencoding{T2A}\selectfont\cyrushrt}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER SHORT U
    ('\u{045F}', r#"{\fontencoding{T2A}\selectfont\cyrdzhe}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER DZHE
    ('\u{0460}', r#"{\fontencoding{T2D}\selectfont\CYROMGA}"#, TEXT, FONTENC_T2D), // CYRILLIC CAPITAL LETTER OMEGA
    ('\u{0461}', r#"{\fontencoding{T2D}\selectfont\cyromga}"#, TEXT, FONTENC_T2D), // CYRILLIC SMALL LETTER OMEGA
    ('\u{0462}', r#"{\fontencoding{X2}\selectfont\CYRYAT}"#, TEXT, FONTENC_X2), // CYRILLIC CAPITAL LETTER YAT
    ('\u{0463}', r#"{\fontencoding{X2}\selectfont\cyryat}"#, TEXT, FONTENC_X2), // CYRILLIC SMALL LETTER YAT
    ('\u{0464}', r#"{\fontencoding{T2D}\selectfont\CYRIOTEST}"#, TEXT, FONTENC_T2D), // CYRILLIC CAPITAL LETTER IOTIFIED E
    ('\u{0465}', r#"{\fontencoding{T2D}\selectfont\cyriotest}"#, TEXT, FONTENC_T2D), // CYRILLIC SMALL LETTER IOTIFIED E
    ('\u{0466}', r#"{\fontencoding{T2D}\selectfont\CYRLYUS}"#, TEXT, FONTENC_T2D), // CYRILLIC CAPITAL LETTER LITTLE YUS
    ('\u{0467}', r#"{\fontencoding{T2D}\selectfont\cyrlyus}"#, TEXT, FONTENC_T2D), // CYRILLIC SMALL LETTER LITTLE YUS
    ('\u{0468}', r#"{\fontencoding{T2D}\selectfont\CYRIOTLYUS}"#, TEXT, FONTENC_T2D), // CYRILLIC CAPITAL LETTER IOTIFIED LITTLE YUS
    ('\u{0469}', r#"{\fontencoding{T2D}\selectfont\cyriotlyus}"#, TEXT, FONTENC_T2D), // CYRILLIC SMALL LETTER IOTIFIED LITTLE YUS
    ('\u{046A}', r#"{\fontencoding{X2}\selectfont\CYRBYUS}"#, TEXT, FONTENC_X2), // CYRILLIC CAPITAL LETTER BIG YUS
    ('\u{046B}', r#"{\fontencoding{X2}\selectfont\cyrbyus}"#, TEXT, FONTENC_X2), // CYRILLIC SMALL LETTER BIG YUS
    ('\u{046C}', r#"{\fontencoding{T2D}\selectfont\CYRIOTBYUS}"#, TEXT, FONTENC_T2D), // CYRILLIC CAPITAL LETTER IOTIFIED BIG YUS
    ('\u{046D}', r#"{\fontencoding{T2D}\selectfont\cyriotbyus}"#, TEXT, FONTENC_T2D), // CYRILLIC SMALL LETTER IOTIFIED BIG YUS
    ('\u{046E}', r#"{\fontencoding{T2D}\selectfont\CYRKSI}"#, TEXT, FONTENC_T2D), // CYRILLIC CAPITAL LETTER KSI
    ('\u{046F}', r#"{\fontencoding{T2D}\selectfont\cyrksi}"#, TEXT, FONTENC_T2D), // CYRILLIC SMALL LETTER KSI
    ('\u{0470}', r#"{\fontencoding{T2D}\selectfont\CYRPSI}"#, TEXT, FONTENC_T2D), // CYRILLIC CAPITAL LETTER PSI
    ('\u{0471}', r#"{\fontencoding{T2D}\selectfont\cyrpsi}"#, TEXT, FONTENC_T2D), // CYRILLIC SMALL LETTER PSI
    ('\u{0472}', r#"{\fontencoding{OT2}\selectfont\CYRFITA}"#, TEXT, FONTENC_OT2), // CYRILLIC CAPITAL LETTER FITA
    ('\u{0473}', r#"{\fontencoding{OT2}\selectfont\cyrfita}"#, TEXT, FONTENC_OT2), // CYRILLIC SMALL LETTER FITA
    ('\u{0474}', r#"{\fontencoding{X2}\selectfont\CYRIZH}"#, TEXT, FONTENC_X2), // CYRILLIC CAPITAL LETTER IZHITSA
    ('\u{0475}', r#"{\fontencoding{X2}\selectfont\cyrizh}"#, TEXT, FONTENC_X2), // CYRILLIC SMALL LETTER IZHITSA
    ('\u{0476}', r#"{\fontencoding{X2}\selectfont\C\CYRIZH}"#, TEXT, FONTENC_X2), // CYRILLIC CAPITAL LETTER IZHITSA WITH DOUBLE GRAVE ACCENT
    ('\u{0477}', r#"{\fontencoding{X2}\selectfont\C\cyrizh}"#, TEXT, FONTENC_X2), // CYRILLIC SMALL LETTER IZHITSA WITH DOUBLE GRAVE ACCENT
    ('\u{0478}', r#"{\fontencoding{T2D}\selectfont\CYRUK}"#, TEXT, FONTENC_T2D), // CYRILLIC CAPITAL LETTER UK
    ('\u{0479}', r#"{\fontencoding{T2D}\selectfont\cyruk}"#, TEXT, FONTENC_T2D), // CYRILLIC SMALL LETTER UK
    ('\u{047A}', r#"{\fontencoding{T2D}\selectfont\CYROMRND}"#, TEXT, FONTENC_T2D), // CYRILLIC CAPITAL LETTER ROUND OMEGA
    ('\u{047B}', r#"{\fontencoding{T2D}\selectfont\cyromrnd}"#, TEXT, FONTENC_T2D), // CYRILLIC SMALL LETTER ROUND OMEGA
    ('\u{047C}', r#"{\fontencoding{T2D}\selectfont\CYROMTLO}"#, TEXT, FONTENC_T2D), // CYRILLIC CAPITAL LETTER OMEGA WITH TITLO
    ('\u{047D}', r#"{\fontencoding{T2D}\selectfont\cyromtlo}"#, TEXT, FONTENC_T2D), // CYRILLIC SMALL LETTER OMEGA WITH TITLO
    ('\u{047E}', r#"{\fontencoding{T2D}\selectfont\CYROT}"#, TEXT, FONTENC_T2D), // CYRILLIC CAPITAL LETTER OT
    ('\u{047F}', r#"{\fontencoding{T2D}\selectfont\cyrot}"#, TEXT, FONTENC_T2D), // CYRILLIC SMALL LETTER OT
    ('\u{0480}', r#"{\fontencoding{T2D}\selectfont\CYRKOPPA}"#, TEXT, FONTENC_T2D), // CYRILLIC CAPITAL LETTER KOPPA
    ('\u{0481}', r#"{\fontencoding{T2D}\selectfont\cyrkoppa}"#, TEXT, FONTENC_T2D), // CYRILLIC SMALL LETTER KOPPA
    ('\u{0482}', r#"{\fontencoding{T2D}\selectfont\UnxTCyrThousands}"#, TEXT, CYR_THOUSANDS), // CYRILLIC THOUSANDS SIGN
    ('\u{048C}', r#"{\fontencoding{T2C}\selectfont\CYRSEMISFTSN}"#, TEXT, FONTENC_T2C), // CYRILLIC CAPITAL LETTER SEMISOFT SIGN
    ('\u{048D}', r#"{\fontencoding{T2C}\selectfont\cyrsemisftsn}"#, TEXT, FONTENC_T2C), // CYRILLIC SMALL LETTER SEMISOFT SIGN
    ('\u{048E}', r#"{\fontencoding{T2C}\selectfont\CYRRTICK}"#, TEXT, FONTENC_T2C), // CYRILLIC CAPITAL LETTER ER WITH TICK
    ('\u{048F}', r#"{\fontencoding{T2C}\selectfont\cyrrtick}"#, TEXT, FONTENC_T2C), // CYRILLIC SMALL LETTER ER WITH TICK
    ('\u{0490}', r#"{\fontencoding{T2A}\selectfont\CYRGUP}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER GHE WITH UPTURN
    ('\u{0491}', r#"{\fontencoding{T2A}\selectfont\cyrgup}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER GHE WITH UPTURN
    ('\u{0492}', r#"{\fontencoding{T2A}\selectfont\CYRGHCRS}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER GHE WITH STROKE
    ('\u{0493}', r#"{\fontencoding{T2A}\selectfont\cyrghcrs}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER GHE WITH STROKE
    ('\u{0494}', r#"{\fontencoding{X2}\selectfont\CYRGHK}"#, TEXT, FONTENC_X2), // CYRILLIC CAPITAL LETTER GHE WITH MIDDLE HOOK
    ('\u{0495}', r#"{\fontencoding{X2}\selectfont\cyrghk}"#, TEXT, FONTENC_X2), // CYRILLIC SMALL LETTER GHE WITH MIDDLE HOOK
    ('\u{0496}', r#"{\fontencoding{T2A}\selectfont\CYRZHDSC}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER ZHE WITH DESCENDER
    ('\u{0497}', r#"{\fontencoding{T2A}\selectfont\cyrzhdsc}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER ZHE WITH DESCENDER
    ('\u{0498}', r#"{\fontencoding{T2A}\selectfont\CYRZDSC}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER ZE WITH DESCENDER
    ('\u{0499}', r#"{\fontencoding{T2A}\selectfont\cyrzdsc}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER ZE WITH DESCENDER
    ('\u{049A}', r#"{\fontencoding{T2A}\selectfont\CYRKDSC}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER KA WITH DESCENDER
    ('\u{049B}', r#"{\fontencoding{T2A}\selectfont\cyrkdsc}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER KA WITH DESCENDER
    ('\u{049C}', r#"{\fontencoding{T2A}\selectfont\CYRKVCRS}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER KA WITH VERTICAL STROKE
    ('\u{049D}', r#"{\fontencoding{T2A}\selectfont\cyrkvcrs}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER KA WITH VERTICAL STROKE
    ('\u{049E}', r#"{\fontencoding{X2}\selectfont\CYRKHCRS}"#, TEXT, FONTENC_X2), // CYRILLIC CAPITAL LETTER KA WITH STROKE
    ('\u{049F}', r#"{\fontencoding{X2}\selectfont\cyrkhcrs}"#, TEXT, FONTENC_X2), // CYRILLIC SMALL LETTER KA WITH STROKE
    ('\u{04A0}', r#"{\fontencoding{T2A}\selectfont\CYRKBEAK}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER BASHKIR KA
    ('\u{04A1}', r#"{\fontencoding{T2A}\selectfont\cyrkbeak}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER BASHKIR KA
    ('\u{04A2}', r#"{\fontencoding{T2A}\selectfont\CYRNDSC}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER EN WITH DESCENDER
    ('\u{04A3}', r#"{\fontencoding{T2A}\selectfont\cyrndsc}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER EN WITH DESCENDER
    ('\u{04A4}', r#"{\fontencoding{T2A}\selectfont\CYRNG}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LIGATURE EN GHE
    ('\u{04A5}', r#"{\fontencoding{T2A}\selectfont\cyrng}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LIGATURE EN GHE
    ('\u{04A6}', r#"{\fontencoding{X2}\selectfont\CYRPHK}"#, TEXT, FONTENC_X2), // CYRILLIC CAPITAL LETTER PE WITH MIDDLE HOOK
    ('\u{04A7}', r#"{\fontencoding{X2}\selectfont\cyrphk}"#, TEXT, FONTENC_X2), // CYRILLIC SMALL LETTER PE WITH MIDDLE HOOK
    ('\u{04A8}', r#"{\fontencoding{X2}\selectfont\CYRABHHA}"#, TEXT, FONTENC_X2), // CYRILLIC CAPITAL LETTER ABKHASIAN HA
    ('\u{04A9}', r#"{\fontencoding{X2}\selectfont\cyrabhha}"#, TEXT, FONTENC_X2), // CYRILLIC SMALL LETTER ABKHASIAN HA
    ('\u{04AA}', r#"{\fontencoding{T2A}\selectfont\CYRSDSC}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER ES WITH DESCENDER
    ('\u{04AB}', r#"{\fontencoding{T2A}\selectfont\cyrsdsc}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER ES WITH DESCENDER
    ('\u{04AC}', r#"{\fontencoding{X2}\selectfont\CYRTDSC}"#, TEXT, FONTENC_X2), // CYRILLIC CAPITAL LETTER TE WITH DESCENDER
    ('\u{04AD}', r#"{\fontencoding{X2}\selectfont\cyrtdsc}"#, TEXT, FONTENC_X2), // CYRILLIC SMALL LETTER TE WITH DESCENDER
    ('\u{04AE}', r#"{\fontencoding{T2A}\selectfont\CYRY}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER STRAIGHT U
    ('\u{04AF}', r#"{\fontencoding{T2A}\selectfont\cyry}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER STRAIGHT U
    ('\u{04B0}', r#"{\fontencoding{T2A}\selectfont\CYRYHCRS}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER STRAIGHT U WITH STROKE
    ('\u{04B1}', r#"{\fontencoding{T2A}\selectfont\cyryhcrs}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER STRAIGHT U WITH STROKE
    ('\u{04B2}', r#"{\fontencoding{T2A}\selectfont\CYRHDSC}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER HA WITH DESCENDER
    ('\u{04B3}', r#"{\fontencoding{T2A}\selectfont\cyrhdsc}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER HA WITH DESCENDER
    ('\u{04B4}', r#"{\fontencoding{X2}\selectfont\CYRTETSE}"#, TEXT, FONTENC_X2), // CYRILLIC CAPITAL LIGATURE TE TSE
    ('\u{04B5}', r#"{\fontencoding{X2}\selectfont\cyrtetse}"#, TEXT, FONTENC_X2), // CYRILLIC SMALL LIGATURE TE TSE
    ('\u{04B6}', r#"{\fontencoding{T2A}\selectfont\CYRCHRDSC}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER CHE WITH DESCENDER
    ('\u{04B7}', r#"{\fontencoding{T2A}\selectfont\cyrchrdsc}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER CHE WITH DESCENDER
    ('\u{04B8}', r#"{\fontencoding{T2A}\selectfont\CYRCHVCRS}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER CHE WITH VERTICAL STROKE
    ('\u{04B9}', r#"{\fontencoding{T2A}\selectfont\cyrchvcrs}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER CHE WITH VERTICAL STROKE
    ('\u{04BA}', r#"{\fontencoding{T2A}\selectfont\CYRSHHA}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER SHHA
    ('\u{04BB}', r#"{\fontencoding{T2A}\selectfont\cyrshha}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER SHHA
    ('\u{04BC}', r#"{\fontencoding{X2}\selectfont\CYRABHCH}"#, TEXT, FONTENC_X2), // CYRILLIC CAPITAL LETTER ABKHASIAN CHE
    ('\u{04BD}', r#"{\fontencoding{X2}\selectfont\cyrabhch}"#, TEXT, FONTENC_X2), // CYRILLIC SMALL LETTER ABKHASIAN CHE
    ('\u{04BE}', r#"{\fontencoding{X2}\selectfont\CYRABHCHDSC}"#, TEXT, FONTENC_X2), // CYRILLIC CAPITAL LETTER ABKHASIAN CHE WITH DESCENDER
    ('\u{04BF}', r#"{\fontencoding{X2}\selectfont\cyrabhchdsc}"#, TEXT, FONTENC_X2), // CYRILLIC SMALL LETTER ABKHASIAN CHE WITH DESCENDER
    ('\u{04C0}', r#"{\fontencoding{T2A}\selectfont\CYRpalochka}"#, TEXT, FONTENC_T2A), // CYRILLIC LETTER PALOCHKA
    ('\u{04C1}', r#"{\fontencoding{T2A}\selectfont\U\CYRZH}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER ZHE WITH BREVE
    ('\u{04C2}', r#"{\fontencoding{T2A}\selectfont\U\cyrzh}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER ZHE WITH BREVE
    ('\u{04C3}', r#"{\fontencoding{X2}\selectfont\CYRKHK}"#, TEXT, FONTENC_X2), // CYRILLIC CAPITAL LETTER KA WITH HOOK
    ('\u{04C4}', r#"{\fontencoding{X2}\selectfont\cyrkhk}"#, TEXT, FONTENC_X2), // CYRILLIC SMALL LETTER KA WITH HOOK
    ('\u{04C5}', r#"{\fontencoding{X2}\selectfont\CYRLDSC}"#, TEXT, FONTENC_X2), // CYRILLIC CAPITAL LETTER EL WITH TAIL
    ('\u{04C6}', r#"{\fontencoding{X2}\selectfont\cyrldsc}"#, TEXT, FONTENC_X2), // CYRILLIC SMALL LETTER EL WITH TAIL
    ('\u{04C7}', r#"{\fontencoding{X2}\selectfont\CYRNHK}"#, TEXT, FONTENC_X2), // CYRILLIC CAPITAL LETTER EN WITH HOOK
    ('\u{04C8}', r#"{\fontencoding{X2}\selectfont\cyrnhk}"#, TEXT, FONTENC_X2), // CYRILLIC SMALL LETTER EN WITH HOOK
    ('\u{04CB}', r#"{\fontencoding{X2}\selectfont\CYRCHLDSC}"#, TEXT, FONTENC_X2), // CYRILLIC CAPITAL LETTER KHAKASSIAN CHE
    ('\u{04CC}', r#"{\fontencoding{X2}\selectfont\cyrchldsc}"#, TEXT, FONTENC_X2), // CYRILLIC SMALL LETTER KHAKASSIAN CHE
    ('\u{04CD}', r#"{\fontencoding{X2}\selectfont\CYRMDSC}"#, TEXT, FONTENC_X2), // CYRILLIC CAPITAL LETTER EM WITH TAIL
    ('\u{04CE}', r#"{\fontencoding{X2}\selectfont\cyrmdsc}"#, TEXT, FONTENC_X2), // CYRILLIC SMALL LETTER EM WITH TAIL
    ('\u{04D0}', r#"{\fontencoding{T2A}\selectfont\U\CYRA}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER A WITH BREVE
    ('\u{04D1}', r#"{\fontencoding{T2A}\selectfont\U\cyra}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER A WITH BREVE
    ('\u{04D2}', r#"{\fontencoding{T2A}\selectfont\"\CYRA}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER A WITH DIAERESIS
    ('\u{04D3}', r#"{\fontencoding{T2A}\selectfont\"\cyra}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER A WITH DIAERESIS
    ('\u{04D4}', r#"{\fontencoding{T2A}\selectfont\CYRAE}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LIGATURE A IE
    ('\u{04D5}', r#"{\fontencoding{T2A}\selectfont\cyrae}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LIGATURE A IE
    ('\u{04D6}', r#"{\fontencoding{T2A}\selectfont\U\CYRE}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER IE WITH BREVE
    ('\u{04D7}', r#"{\fontencoding{T2A}\selectfont\U\cyre}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER IE WITH BREVE
    ('\u{04D8}', r#"{\fontencoding{T2A}\selectfont\CYRSCHWA}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER SCHWA
    ('\u{04D9}', r#"{\fontencoding{T2A}\selectfont\cyrschwa}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER SCHWA
    ('\u{04DA}', r#"{\fontencoding{T2A}\selectfont\"\CYRSCHWA}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER SCHWA WITH DIAERESIS
    ('\u{04DB}', r#"{\fontencoding{T2A}\selectfont\"\cyrschwa}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER SCHWA WITH DIAERESIS
    ('\u{04DC}', r#"{\fontencoding{T2A}\selectfont\"\CYRZH}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER ZHE WITH DIAERESIS
    ('\u{04DD}', r#"{\fontencoding{T2A}\selectfont\"\cyrzh}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER ZHE WITH DIAERESIS
    ('\u{04DE}', r#"{\fontencoding{T2A}\selectfont\"\CYRZ}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER ZE WITH DIAERESIS
    ('\u{04DF}', r#"{\fontencoding{T2A}\selectfont\"\cyrz}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER ZE WITH DIAERESIS
    ('\u{04E0}', r#"{\fontencoding{X2}\selectfont\CYRABHDZE}"#, TEXT, FONTENC_X2), // CYRILLIC CAPITAL LETTER ABKHASIAN DZE
    ('\u{04E1}', r#"{\fontencoding{X2}\selectfont\cyrabhdze}"#, TEXT, FONTENC_X2), // CYRILLIC SMALL LETTER ABKHASIAN DZE
    ('\u{04E2}', r#"{\fontencoding{T2A}\selectfont\=\CYRI}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER I WITH MACRON
    ('\u{04E3}', r#"{\fontencoding{T2A}\selectfont\=\cyri}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER I WITH MACRON
    ('\u{04E4}', r#"{\fontencoding{T2A}\selectfont\"\CYRI}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER I WITH DIAERESIS
    ('\u{04E5}', r#"{\fontencoding{T2A}\selectfont\"\cyri}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER I WITH DIAERESIS
    ('\u{04E6}', r#"{\fontencoding{T2A}\selectfont\"\CYRO}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER O WITH DIAERESIS
    ('\u{04E7}', r#"{\fontencoding{T2A}\selectfont\"\cyro}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER O WITH DIAERESIS
    ('\u{04E8}', r#"{\fontencoding{T2A}\selectfont\CYROTLD}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER BARRED O
    ('\u{04E9}', r#"{\fontencoding{T2A}\selectfont\cyrotld}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER BARRED O
    ('\u{04EC}', r#"{\fontencoding{T2A}\selectfont\"\CYREREV}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER E WITH DIAERESIS
    ('\u{04ED}', r#"{\fontencoding{T2A}\selectfont\"\cyrerev}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER E WITH DIAERESIS
    ('\u{04EE}', r#"{\fontencoding{T2A}\selectfont\=\CYRU}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER U WITH MACRON
    ('\u{04EF}', r#"{\fontencoding{T2A}\selectfont\=\cyru}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER U WITH MACRON
    ('\u{04F0}', r#"{\fontencoding{T2A}\selectfont\"\CYRU}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER U WITH DIAERESIS
    ('\u{04F1}', r#"{\fontencoding{T2A}\selectfont\"\cyru}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER U WITH DIAERESIS
    ('\u{04F2}', r#"{\fontencoding{T2A}\selectfont\H\CYRU}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER U WITH DOUBLE ACUTE
    ('\u{04F3}', r#"{\fontencoding{T2A}\selectfont\H\cyru}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER U WITH DOUBLE ACUTE
    ('\u{04F4}', r#"{\fontencoding{T2A}\selectfont\"\CYRCH}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER CHE WITH DIAERESIS
    ('\u{04F5}', r#"{\fontencoding{T2A}\selectfont\"\cyrch}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER CHE WITH DIAERESIS
    ('\u{04F6}', r#"{\fontencoding{X2}\selectfont\CYRGDSC}"#, TEXT, FONTENC_X2), // CYRILLIC CAPITAL LETTER GHE WITH DESCENDER
    ('\u{04F7}', r#"{\fontencoding{X2}\selectfont\cyrgdsc}"#, TEXT, FONTENC_X2), // CYRILLIC SMALL LETTER GHE WITH DESCENDER
    ('\u{04F8}', r#"{\fontencoding{T2A}\selectfont\"\CYRERY}"#, TEXT, FONTENC_T2A), // CYRILLIC CAPITAL LETTER YERU WITH DIAERESIS
    ('\u{04F9}', r#"{\fontencoding{T2A}\selectfont\"\cyrery}"#, TEXT, FONTENC_T2A), // CYRILLIC SMALL LETTER YERU WITH DIAERESIS
    ('\u{04FA}', r#"{\fontencoding{T2B}\selectfont\CYRGDSCHCRS}"#, TEXT, FONTENC_T2B), // CYRILLIC CAPITAL LETTER GHE WITH STROKE AND HOOK
    ('\u{04FB}', r#"{\fontencoding{T2B}\selectfont\cyrgdschcrs}"#, TEXT, FONTENC_T2B), // CYRILLIC SMALL LETTER GHE WITH STROKE AND HOOK
    ('\u{04FC}', r#"{\fontencoding{X2}\selectfont\CYRHHK}"#, TEXT, FONTENC_X2), // CYRILLIC CAPITAL LETTER HA WITH HOOK
    ('\u{04FD}', r#"{\fontencoding{X2}\selectfont\cyrhhk}"#, TEXT, FONTENC_X2), // CYRILLIC SMALL LETTER HA WITH HOOK
    ('\u{04FE}', r#"{\fontencoding{T2B}\selectfont\CYRHHCRS}"#, TEXT, FONTENC_T2B), // CYRILLIC CAPITAL LETTER HA WITH STROKE
    ('\u{04FF}', r#"{\fontencoding{T2B}\selectfont\cyrhhcrs}"#, TEXT, FONTENC_T2B), // CYRILLIC SMALL LETTER HA WITH STROKE
    ('\u{0E3F}', r#"\textbaht"#, TEXT, BUILTINS), // THAI CURRENCY SYMBOL BAHT
    ('\u{2000}', r#"\enskip"#, TEXT, BUILTINS), // EN QUAD
    ('\u{2001}', r#"\quad"#, TEXT, BUILTINS), // EM QUAD
    ('\u{2002}', r#"\enskip"#, TEXT, BUILTINS), // EN SPACE
    ('\u{2003}', r#"\quad"#, TEXT, BUILTINS), // EM SPACE
    ('\u{2004}', r#"\hspace{0.33em}"#, TEXT, BUILTINS), // THREE-PER-EM SPACE
    ('\u{2005}', r#"\hspace{0.25em}"#, TEXT, BUILTINS), // FOUR-PER-EM SPACE
    ('\u{2006}', r#"\hspace{0.167em}"#, TEXT, BUILTINS), // SIX-PER-EM SPACE
    ('\u{2007}', r#"~"#, ANY, BUILTINS), // FIGURE SPACE
    ('\u{2008}', r#"\;"#, TEXT, BUILTINS), // PUNCTUATION SPACE
    ('\u{2009}', r#"\,"#, TEXT, BUILTINS), // THIN SPACE
    ('\u{200A}', r#"\hspace{1pt}"#, TEXT, BUILTINS), // HAIR SPACE
    ('\u{200C}', r#"\textcompwordmark"#, TEXT, BUILTINS), // ZERO WIDTH NON-JOINER
    ('\u{2010}', r#"-"#, ANY, BUILTINS), // HYPHEN
    ('\u{2011}', r#"\nobreakdash-"#, TEXT, AMSMATH), // NON-BREAKING HYPHEN
    ('\u{2012}', r#"-"#, ANY, BUILTINS), // FIGURE DASH
    ('\u{2013}', r#"\textendash"#, TEXT, BUILTINS), // EN DASH
    ('\u{2014}', r#"\textemdash"#, TEXT, BUILTINS), // EM DASH
    ('\u{2015}', r#"\textemdash"#, TEXT, BUILTINS), // HORIZONTAL BAR
    ('\u{2016}', r#"\Vert"#, MATH, BUILTINS), // DOUBLE VERTICAL LINE
    ('\u{2018}', r#"\textquoteleft"#, TEXT, BUILTINS), // LEFT SINGLE QUOTATION MARK
    ('\u{2019}', r#"\textquoteright"#, TEXT, BUILTINS), // RIGHT SINGLE QUOTATION MARK
    ('\u{201A}', r#"\quotesinglbase"#, TEXT, FONTENC_T1), // SINGLE LOW-9 QUOTATION MARK
    ('\u{201C}', r#"\textquotedblleft"#, TEXT, BUILTINS), // LEFT DOUBLE QUOTATION MARK
    ('\u{201D}', r#"\textquotedblright"#, TEXT, BUILTINS), // RIGHT DOUBLE QUOTATION MARK
    ('\u{201E}', r#"\quotedblbase"#, TEXT, FONTENC_T1), // DOUBLE LOW-9 QUOTATION MARK
    ('\u{2020}', r#"\textdagger"#, TEXT, BUILTINS), // DAGGER
    ('\u{2021}', r#"\textdaggerdbl"#, TEXT, BUILTINS), // DOUBLE DAGGER
    ('\u{2022}', r#"\textbullet"#, TEXT, BUILTINS), // BULLET
    ('\u{2024}', r#"."#, ANY, BUILTINS), // ONE DOT LEADER
    ('\u{2025}', r#".."#, ANY, BUILTINS), // TWO DOT LEADER
    ('\u{2026}', r#"\textellipsis"#, TEXT, BUILTINS), // HORIZONTAL ELLIPSIS
    ('\u{2030}', r#"\textperthousand"#, TEXT, BUILTINS), // PER MILLE SIGN
    ('\u{2031}', r#"\textpertenthousand"#, TEXT, BUILTINS), // PER TEN THOUSAND SIGN
    ('\u{2032}', r#"'"#, ANY, BUILTINS), // PRIME
    ('\u{2033}', r#"''"#, ANY, BUILTINS), // DOUBLE PRIME
    ('\u{2034}', r#"'''"#, ANY, BUILTINS), // TRIPLE PRIME
    ('\u{2035}', r#"\backprime"#, MATH, AMSSYMB), // REVERSED PRIME
    ('\u{2039}', r#"\guilsinglleft"#, TEXT, FONTENC_T1), // SINGLE LEFT-POINTING ANGLE QUOTATION MARK
    ('\u{203A}', r#"\guilsinglright"#, TEXT, FONTENC_T1), // SINGLE RIGHT-POINTING ANGLE QUOTATION MARK
    ('\u{203B}', r#"\textreferencemark"#, TEXT, BUILTINS), // REFERENCE MARK
    ('\u{203D}', r#"\textinterrobang"#, TEXT, BUILTINS), // INTERROBANG
    ('\u{2044}', r#"\textfractionsolidus"#, TEXT, BUILTINS), // FRACTION SLASH
    ('\u{204E}', r#"\textasteriskcentered"#, TEXT, BUILTINS), // LOW ASTERISK
    ('\u{2052}', r#"\textdiscount"#, TEXT, BUILTINS), // COMMERCIAL MINUS SIGN
    ('\u{2057}', r#"''''"#, ANY, BUILTINS), // QUADRUPLE PRIME
    ('\u{205F}', r#"\hspace{0.22em}"#, TEXT, BUILTINS), // MEDIUM MATHEMATICAL SPACE
    ('\u{2060}', r#"\nolinebreak"#, TEXT, BUILTINS), // WORD JOINER
    ('\u{2061}', r#""#, ANY, BUILTINS), // FUNCTION APPLICATION
    ('\u{2070}', r#"^0"#, MATH, BUILTINS), // SUPERSCRIPT ZERO
    ('\u{2071}', r#"^i"#, MATH, BUILTINS), // SUPERSCRIPT LATIN SMALL LETTER I
    ('\u{2074}', r#"^4"#, MATH, BUILTINS), // SUPERSCRIPT FOUR
    ('\u{2075}', r#"^5"#, MATH, BUILTINS), // SUPERSCRIPT FIVE
    ('\u{2076}', r#"^6"#, MATH, BUILTINS), // SUPERSCRIPT SIX
    ('\u{2077}', r#"^7"#, MATH, BUILTINS), // SUPERSCRIPT SEVEN
    ('\u{2078}', r#"^8"#, MATH, BUILTINS), // SUPERSCRIPT EIGHT
    ('\u{2079}', r#"^9"#, MATH, BUILTINS), // SUPERSCRIPT NINE
    ('\u{207A}', r#"^+"#, MATH, BUILTINS), // SUPERSCRIPT PLUS SIGN
    ('\u{207B}', r#"^-"#, MATH, BUILTINS), // SUPERSCRIPT MINUS
    ('\u{207C}', r#"^="#, MATH, BUILTINS), // SUPERSCRIPT EQUALS SIGN
    ('\u{207D}', r#"^("#, MATH, BUILTINS), // SUPERSCRIPT LEFT PARENTHESIS
    ('\u{207E}', r#"^)"#, MATH, BUILTINS), // SUPERSCRIPT RIGHT PARENTHESIS
    ('\u{207F}', r#"^n"#, MATH, BUILTINS), // SUPERSCRIPT LATIN SMALL LETTER N
    ('\u{2080}', r#"_0"#, MATH, BUILTINS), // SUBSCRIPT ZERO
    ('\u{2081}', r#"_1"#, MATH, BUILTINS), // SUBSCRIPT ONE
    ('\u{2082}', r#"_2"#, MATH, BUILTINS), // SUBSCRIPT TWO
    ('\u{2083}', r#"_3"#, MATH, BUILTINS), // SUBSCRIPT THREE
    ('\u{2084}', r#"_4"#, MATH, BUILTINS), // SUBSCRIPT FOUR
    ('\u{2085}', r#"_5"#, MATH, BUILTINS), // SUBSCRIPT FIVE
    ('\u{2086}', r#"_6"#, MATH, BUILTINS), // SUBSCRIPT SIX
    ('\u{2087}', r#"_7"#, MATH, BUILTINS), // SUBSCRIPT SEVEN
    ('\u{2088}', r#"_8"#, MATH, BUILTINS), // SUBSCRIPT EIGHT
    ('\u{2089}', r#"_9"#, MATH, BUILTINS), // SUBSCRIPT NINE
    ('\u{208A}', r#"_+"#, MATH, BUILTINS), // SUBSCRIPT PLUS SIGN
    ('\u{208B}', r#"_-"#, MATH, BUILTINS), // SUBSCRIPT MINUS
    ('\u{208C}', r#"_="#, MATH, BUILTINS), // SUBSCRIPT EQUALS SIGN
    ('\u{208D}', r#"_("#, MATH, BUILTINS), // SUBSCRIPT LEFT PARENTHESIS
    ('\u{208E}', r#"_)"#, MATH, BUILTINS), // SUBSCRIPT RIGHT PARENTHESIS
    ('\u{2090}', r#"_a"#, MATH, BUILTINS), // LATIN SUBSCRIPT SMALL LETTER A
    ('\u{2091}', r#"_e"#, MATH, BUILTINS), // LATIN SUBSCRIPT SMALL LETTER E
    ('\u{2092}', r#"_o"#, MATH, BUILTINS), // LATIN SUBSCRIPT SMALL LETTER O
    ('\u{2093}', r#"_x"#, MATH, BUILTINS), // LATIN SUBSCRIPT SMALL LETTER X
    ('\u{2095}', r#"_h"#, MATH, BUILTINS), // LATIN SUBSCRIPT SMALL LETTER H
    ('\u{2096}', r#"_k"#, MATH, BUILTINS), // LATIN SUBSCRIPT SMALL LETTER K
    ('\u{2097}', r#"_l"#, MATH, BUILTINS), // LATIN SUBSCRIPT SMALL LETTER L
    ('\u{2098}', r#"_m"#, MATH, BUILTINS), // LATIN SUBSCRIPT SMALL LETTER M
    ('\u{2099}', r#"_n"#, MATH, BUILTINS), // LATIN SUBSCRIPT SMALL LETTER N
    ('\u{209A}', r#"_p"#, MATH, BUILTINS), // LATIN SUBSCRIPT SMALL LETTER P
    ('\u{209B}', r#"_s"#, MATH, BUILTINS), // LATIN SUBSCRIPT SMALL LETTER S
    ('\u{209C}', r#"_t"#, MATH, BUILTINS), // LATIN SUBSCRIPT SMALL LETTER T
    ('\u{20A1}', r#"\textcolonmonetary"#, TEXT, BUILTINS), // COLON SIGN
    ('\u{20A4}', r#"\textlira"#, TEXT, BUILTINS), // LIRA SIGN
    ('\u{20A6}', r#"\textnaira"#, TEXT, BUILTINS), // NAIRA SIGN
    ('\u{20A9}', r#"\textwon"#, TEXT, BUILTINS), // WON SIGN
    ('\u{20AB}', r#"\textdong"#, TEXT, BUILTINS), // DONG SIGN
    ('\u{20AC}', r#"\texteuro"#, TEXT, BUILTINS), // EURO SIGN
    ('\u{20B1}', r#"\textpeso"#, TEXT, BUILTINS), // PESO SIGN
    ('\u{2102}', r#"\mathbb{C}"#, MATH, AMSSYMB), // DOUBLE-STRUCK CAPITAL C
    ('\u{2103}', r#"\textcelsius"#, TEXT, BUILTINS), // DEGREE CELSIUS
    ('\u{2109}', r#"\ensuremath{^\circ}F"#, TEXT, BUILTINS), // DEGREE FAHRENHEIT
    ('\u{210A}', r#"\UnxTScr{g}"#, MATH, SCRIPT), // SCRIPT SMALL G
    ('\u{210B}', r#"\mathscr{H}"#, MATH, MATHRSFS), // SCRIPT CAPITAL H
    ('\u{210C}', r#"\mathfrak{H}"#, MATH, AMSSYMB), // BLACK-LETTER CAPITAL H
    ('\u{210D}', r#"\mathbb{H}"#, MATH, AMSSYMB), // DOUBLE-STRUCK CAPITAL H
    ('\u{210E}', r#"h"#, MATH, BUILTINS), // PLANCK CONSTANT
    ('\u{210F}', r#"\hbar"#, MATH, BUILTINS), // PLANCK CONSTANT OVER TWO PI
    ('\u{2110}', r#"\mathscr{I}"#, MATH, MATHRSFS), // SCRIPT CAPITAL I
    ('\u{2111}', r#"\mathfrak{I}"#, MATH, AMSSYMB), // BLACK-LETTER CAPITAL I
    ('\u{2112}', r#"\mathscr{L}"#, MATH, MATHRSFS), // SCRIPT CAPITAL L
    ('\u{2113}', r#"\ell"#, MATH, BUILTINS), // SCRIPT SMALL L
    ('\u{2115}', r#"\mathbb{N}"#, MATH, AMSSYMB), // DOUBLE-STRUCK CAPITAL N
    ('\u{2116}', r#"\textnumero"#, TEXT, BUILTINS), // NUMERO SIGN
    ('\u{2117}', r#"\textcircledP"#, TEXT, BUILTINS), // SOUND RECORDING COPYRIGHT
    ('\u{2118}', r#"\wp"#, MATH, BUILTINS), // SCRIPT CAPITAL P
    ('\u{2119}', r#"\mathbb{P}"#, MATH, AMSSYMB), // DOUBLE-STRUCK CAPITAL P
    ('\u{211A}', r#"\mathbb{Q}"#, MATH, AMSSYMB), // DOUBLE-STRUCK CAPITAL Q
    ('\u{211B}', r#"\mathscr{R}"#, MATH, MATHRSFS), // SCRIPT CAPITAL R
    ('\u{211C}', r#"\mathfrak{R}"#, MATH, AMSSYMB), // BLACK-LETTER CAPITAL R
    ('\u{211D}', r#"\mathbb{R}"#, MATH, AMSSYMB), // DOUBLE-STRUCK CAPITAL R
    ('\u{211E}', r#"\textrecipe"#, TEXT, BUILTINS), // PRESCRIPTION TAKE
    ('\u{2120}', r#"\textservicemark"#, TEXT, BUILTINS), // SERVICE MARK
    ('\u{2122}', r#"\texttrademark"#, TEXT, BUILTINS), // TRADE MARK SIGN
    ('\u{2124}', r#"\mathbb{Z}"#, MATH, AMSSYMB), // DOUBLE-STRUCK CAPITAL Z
    ('\u{2126}', r#"\textohm"#, TEXT, BUILTINS), // OHM SIGN
    ('\u{2127}', r#"\textmho"#, TEXT, BUILTINS), // INVERTED OHM SIGN
    ('\u{2128}', r#"\mathfrak{Z}"#, MATH, AMSSYMB), // BLACK-LETTER CAPITAL Z
    ('\u{212A}', r#"K"#, ANY, BUILTINS), // KELVIN SIGN
    ('\u{212B}', r#"\r{A}"#, TEXT, BUILTINS), // ANGSTROM SIGN
    ('\u{212C}', r#"\mathscr{B}"#, MATH, MATHRSFS), // SCRIPT CAPITAL B
    ('\u{212D}', r#"\mathfrak{C}"#, MATH, AMSSYMB), // BLACK-LETTER CAPITAL C
    ('\u{212E}', r#"\textestimated"#, TEXT, BUILTINS), // ESTIMATED SYMBOL
    ('\u{212F}', r#"\UnxTScr{e}"#, MATH, SCRIPT), // SCRIPT SMALL E
    ('\u{2130}', r#"\mathscr{E}"#, MATH, MATHRSFS), // SCRIPT CAPITAL E
    ('\u{2131}', r#"\mathscr{F}"#, MATH, MATHRSFS), // SCRIPT CAPITAL F
    ('\u{2133}', r#"\mathscr{M}"#, MATH, MATHRSFS), // SCRIPT CAPITAL M
    ('\u{2134}', r#"\UnxTScr{o}"#, MATH, SCRIPT), // SCRIPT SMALL O
    ('\u{2135}', r#"\aleph"#, MATH, BUILTINS), // ALEF SYMBOL
    ('\u{2136}', r#"\beth"#, MATH, AMSSYMB), // BET SYMBOL
    ('\u{2137}', r#"\gimel"#, MATH, AMSSYMB), // GIMEL SYMBOL
    ('\u{2138}', r#"\daleth"#, MATH, AMSSYMB), // DALET SYMBOL
    ('\u{2153}', r#"\nicefrac{1}{3}"#, TEXT, NICEFRAC), // VULGAR FRACTION ONE THIRD
    ('\u{2154}', r#"\nicefrac{2}{3}"#, TEXT, NICEFRAC), // VULGAR FRACTION TWO THIRDS
    ('\u{2155}', r#"\nicefrac{1}{5}"#, TEXT, NICEFRAC), // VULGAR FRACTION ONE FIFTH
    ('\u{2156}', r#"\nicefrac{2}{5}"#, TEXT, NICEFRAC), // VULGAR FRACTION TWO FIFTHS
    ('\u{2157}', r#"\nicefrac{3}{5}"#, TEXT, NICEFRAC), // VULGAR FRACTION THREE FIFTHS
    ('\u{2158}', r#"\nicefrac{4}{5}"#, TEXT, NICEFRAC), // VULGAR FRACTION FOUR FIFTHS
    ('\u{2159}', r#"\nicefrac{1}{6}"#, TEXT, NICEFRAC), // VULGAR FRACTION ONE SIXTH
    ('\u{215A}', r#"\nicefrac{5}{6}"#, TEXT, NICEFRAC), // VULGAR FRACTION FIVE SIXTHS
    ('\u{215B}', r#"\nicefrac{1}{8}"#, TEXT, NICEFRAC), // VULGAR FRACTION ONE EIGHTH
    ('\u{215C}', r#"\nicefrac{3}{8}"#, TEXT, NICEFRAC), // VULGAR FRACTION THREE EIGHTHS
    ('\u{215D}', r#"\nicefrac{5}{8}"#, TEXT, NICEFRAC), // VULGAR FRACTION FIVE EIGHTHS
    ('\u{215E}', r#"\nicefrac{7}{8}"#, TEXT, NICEFRAC), // VULGAR FRACTION SEVEN EIGHTHS
    ('\u{2190}', r#"\textleftarrow"#, TEXT, BUILTINS), // LEFTWARDS ARROW
    ('\u{2191}', r#"\textuparrow"#, TEXT, BUILTINS), // UPWARDS ARROW
    ('\u{2192}', r#"\textrightarrow"#, TEXT, BUILTINS), // RIGHTWARDS ARROW
    ('\u{2193}', r#"\textdownarrow"#, TEXT, BUILTINS), // DOWNWARDS ARROW
    ('\u{2194}', r#"\leftrightarrow"#, MATH, BUILTINS), // LEFT RIGHT ARROW
    ('\u{2195}', r#"\updownarrow"#, MATH, BUILTINS), // UP DOWN ARROW
    ('\u{2196}', r#"\nwarrow"#, MATH, BUILTINS), // NORTH WEST ARROW
    ('\u{2197}', r#"\nearrow"#, MATH, BUILTINS), // NORTH EAST ARROW
    ('\u{2198}', r#"\searrow"#, MATH, BUILTINS), // SOUTH EAST ARROW
    ('\u{2199}', r#"\swarrow"#, MATH, BUILTINS), // SOUTH WEST ARROW
    ('\u{219A}', r#"\nleftarrow"#, MATH, AMSSYMB), // LEFTWARDS ARROW WITH STROKE
    ('\u{219B}', r#"\nrightarrow"#, MATH, AMSSYMB), // RIGHTWARDS ARROW WITH STROKE
    ('\u{219C}', r#"\UnxTleftwavearrow"#, MATH, STIX), // LEFTWARDS WAVE ARROW
    ('\u{219D}', r#"\UnxTrightwavearrow"#, MATH, STIX), // RIGHTWARDS WAVE ARROW
    ('\u{219E}', r#"\twoheadleftarrow"#, MATH, AMSSYMB), // LEFTWARDS TWO HEADED ARROW
    ('\u{21A0}', r#"\twoheadrightarrow"#, MATH, AMSSYMB), // RIGHTWARDS TWO HEADED ARROW
    ('\u{21A2}', r#"\leftarrowtail"#, MATH, AMSSYMB), // LEFTWARDS ARROW WITH TAIL
    ('\u{21A3}', r#"\rightarrowtail"#, MATH, AMSSYMB), // RIGHTWARDS ARROW WITH TAIL
    ('\u{21A6}', r#"\mapsto"#, MATH, BUILTINS), // RIGHTWARDS ARROW FROM BAR
    ('\u{21A9}', r#"\hookleftarrow"#, MATH, BUILTINS), // LEFTWARDS ARROW WITH HOOK
    ('\u{21AA}', r#"\hookrightarrow"#, MATH, BUILTINS), // RIGHTWARDS ARROW WITH HOOK
    ('\u{21AB}', r#"\looparrowleft"#, MATH, AMSSYMB), // LEFTWARDS ARROW WITH LOOP
    ('\u{21AC}', r#"\looparrowright"#, MATH, AMSSYMB), // RIGHTWARDS ARROW WITH LOOP
    ('\u{21AD}', r#"\leftrightsquigarrow"#, MATH, AMSSYMB), // LEFT RIGHT WAVE ARROW
    ('\u{21AE}', r#"\nleftrightarrow"#, MATH, AMSSYMB), // LEFT RIGHT ARROW WITH STROKE
    ('\u{21B0}', r#"\Lsh"#, MATH, AMSSYMB), // UPWARDS ARROW WITH TIP LEFTWARDS
    ('\u{21B1}', r#"\Rsh"#, MATH, AMSSYMB), // UPWARDS ARROW WITH TIP RIGHTWARDS
    ('\u{21B6}', r#"\curvearrowleft"#, MATH, AMSSYMB), // ANTICLOCKWISE TOP SEMICIRCLE ARROW
    ('\u{21B7}', r#"\curvearrowright"#, MATH, AMSSYMB), // CLOCKWISE TOP SEMICIRCLE ARROW
    ('\u{21BA}', r#"\circlearrowleft"#, MATH, AMSSYMB), // ANTICLOCKWISE OPEN CIRCLE ARROW
    ('\u{21BB}', r#"\circlearrowright"#, MATH, AMSSYMB), // CLOCKWISE OPEN CIRCLE ARROW
    ('\u{21BC}', r#"\leftharpoonup"#, MATH, BUILTINS), // LEFTWARDS HARPOON WITH BARB UPWARDS
    ('\u{21BD}', r#"\leftharpoondown"#, MATH, BUILTINS), // LEFTWARDS HARPOON WITH BARB DOWNWARDS
    ('\u{21BE}', r#"\upharpoonright"#, MATH, AMSSYMB), // UPWARDS HARPOON WITH BARB RIGHTWARDS
    ('\u{21BF}', r#"\upharpoonleft"#, MATH, AMSSYMB), // UPWARDS HARPOON WITH BARB LEFTWARDS
    ('\u{21C0}', r#"\rightharpoonup"#, MATH, BUILTINS), // RIGHTWARDS HARPOON WITH BARB UPWARDS
    ('\u{21C1}', r#"\rightharpoondown"#, MATH, BUILTINS), // RIGHTWARDS HARPOON WITH BARB DOWNWARDS
    ('\u{21C2}', r#"\downharpoonright"#, MATH, AMSSYMB), // DOWNWARDS HARPOON WITH BARB RIGHTWARDS
    ('\u{21C3}', r#"\downharpoonleft"#, MATH, AMSSYMB), // DOWNWARDS HARPOON WITH BARB LEFTWARDS
    ('\u{21C4}', r#"\rightleftarrows"#, MATH, AMSSYMB), // RIGHTWARDS ARROW OVER LEFTWARDS ARROW
    ('\u{21C5}', r#"\UnxTupdownarrows"#, MATH, STIX), // UPWARDS ARROW LEFTWARDS OF DOWNWARDS ARROW
    ('\u{21C6}', r#"\leftrightarrows"#, MATH, AMSSYMB), // LEFTWARDS ARROW OVER RIGHTWARDS ARROW
    ('\u{21C7}', r#"\leftleftarrows"#, MATH, AMSSYMB), // LEFTWARDS PAIRED ARROWS
    ('\u{21C8}', r#"\upuparrows"#, MATH, AMSSYMB), // UPWARDS PAIRED ARROWS
    ('\u{21C9}', r#"\rightrightarrows"#, MATH, AMSSYMB), // RIGHTWARDS PAIRED ARROWS
    ('\u{21CA}', r#"\downdownarrows"#, MATH, AMSSYMB), // DOWNWARDS PAIRED ARROWS
    ('\u{21CB}', r#"\leftrightharpoons"#, MATH, AMSSYMB), // LEFTWARDS HARPOON OVER RIGHTWARDS HARPOON
    ('\u{21CC}', r#"\rightleftharpoons"#, MATH, BUILTINS), // RIGHTWARDS HARPOON OVER LEFTWARDS HARPOON
    ('\u{21CD}', r#"\nLeftarrow"#, MATH, AMSSYMB), // LEFTWARDS DOUBLE ARROW WITH STROKE
    ('\u{21CE}', r#"\nLeftrightarrow"#, MATH, AMSSYMB), // LEFT RIGHT DOUBLE ARROW WITH STROKE
    ('\u{21CF}', r#"\nRightarrow"#, MATH, AMSSYMB), // RIGHTWARDS DOUBLE ARROW WITH STROKE
    ('\u{21D0}', r#"\Leftarrow"#, MATH, BUILTINS), // LEFTWARDS DOUBLE ARROW
    ('\u{21D1}', r#"\Uparrow"#, MATH, BUILTINS), // UPWARDS DOUBLE ARROW
    ('\u{21D2}', r#"\Rightarrow"#, MATH, BUILTINS), // RIGHTWARDS DOUBLE ARROW
    ('\u{21D3}', r#"\Downarrow"#, MATH, BUILTINS), // DOWNWARDS DOUBLE ARROW
    ('\u{21D4}', r#"\Leftrightarrow"#, MATH, BUILTINS), // LEFT RIGHT DOUBLE ARROW
    ('\u{21D5}', r#"\Updownarrow"#, MATH, BUILTINS), // UP DOWN DOUBLE ARROW
    ('\u{21DA}', r#"\Lleftarrow"#, MATH, AMSSYMB), // LEFTWARDS TRIPLE ARROW
    ('\u{21DB}', r#"\Rrightarrow"#, MATH, AMSSYMB), // RIGHTWARDS TRIPLE ARROW
    ('\u{21DD}', r#"\rightsquigarrow"#, MATH, AMSSYMB), // RIGHTWARDS SQUIGGLE ARROW
    ('\u{21F5}', r#"\UnxTdownuparrows"#, MATH, STIX), // DOWNWARDS ARROW LEFTWARDS OF UPWARDS ARROW
    ('\u{2200}', r#"\forall"#, MATH, BUILTINS), // FOR ALL
    ('\u{2201}', r#"\complement"#, MATH, AMSSYMB), // COMPLEMENT
    ('\u{2202}', r#"\partial"#, MATH, BUILTINS), // PARTIAL DIFFERENTIAL
    ('\u{2203}', r#"\exists"#, MATH, BUILTINS), // THERE EXISTS
    ('\u{2204}', r#"\nexists"#, MATH, AMSSYMB), // THERE DOES NOT EXIST
    ('\u{2205}', r#"\varnothing"#, MATH, AMSSYMB), // EMPTY SET
    ('\u{2206}', r#"\Delta"#, MATH, BUILTINS), // INCREMENT
    ('\u{2207}', r#"\nabla"#, MATH, BUILTINS), // NABLA
    ('\u{2208}', r#"\in"#, MATH, BUILTINS), // ELEMENT OF
    ('\u{2209}', r#"\notin"#, MATH, BUILTINS), // NOT AN ELEMENT OF
    ('\u{220A}', r#"\in"#, MATH, BUILTINS), // SMALL ELEMENT OF
    ('\u{220B}', r#"\ni"#, MATH, BUILTINS), // CONTAINS AS MEMBER
    ('\u{220C}', r#"\not\ni"#, MATH, BUILTINS), // DOES NOT CONTAIN AS MEMBER
    ('\u{220D}', r#"\ni"#, MATH, BUILTINS), // SMALL CONTAINS AS MEMBER
    ('\u{220E}', r#"\blacksquare"#, MATH, AMSSYMB), // END OF PROOF
    ('\u{220F}', r#"\prod"#, MATH, BUILTINS), // N-ARY PRODUCT
    ('\u{2210}', r#"\coprod"#, MATH, BUILTINS), // N-ARY COPRODUCT
    ('\u{2211}', r#"\sum"#, MATH, BUILTINS), // N-ARY SUMMATION
    ('\u{2212}', r#"-"#, MATH, BUILTINS), // MINUS SIGN
    ('\u{2213}', r#"\mp"#, MATH, BUILTINS), // MINUS-OR-PLUS SIGN
    ('\u{2214}', r#"\dotplus"#, MATH, AMSSYMB), // DOT PLUS
    ('\u{2215}', r#"/"#, MATH, BUILTINS), // DIVISION SLASH
    ('\u{2216}', r#"\smallsetminus"#, MATH, AMSSYMB), // SET MINUS
    ('\u{2217}', r#"*"#, MATH, BUILTINS), // ASTERISK OPERATOR
    ('\u{2218}', r#"\circ"#, MATH, BUILTINS), // RING OPERATOR
    ('\u{2219}', r#"\bullet"#, MATH, BUILTINS), // BULLET OPERATOR
    ('\u{221A}', r#"\sqrt{}"#, MATH, BUILTINS), // SQUARE ROOT
    ('\u{221B}', r#"\sqrt[3]{}"#, MATH, BUILTINS), // CUBE ROOT
    ('\u{221C}', r#"\sqrt[4]{}"#, MATH, BUILTINS), // FOURTH ROOT
    ('\u{221D}', r#"\propto"#, MATH, BUILTINS), // PROPORTIONAL TO
    ('\u{221E}', r#"\infty"#, MATH, BUILTINS), // INFINITY
    ('\u{221F}', r#"\UnxTrightangle"#, MATH, STIX), // RIGHT ANGLE
    ('\u{2220}', r#"\angle"#, MATH, BUILTINS), // ANGLE
    ('\u{2221}', r#"\measuredangle"#, MATH, AMSSYMB), // MEASURED ANGLE
    ('\u{2222}', r#"\sphericalangle"#, MATH, AMSSYMB), // SPHERICAL ANGLE
    ('\u{2223}', r#"\mid"#, MATH, BUILTINS), // DIVIDES
    ('\u{2224}', r#"\nmid"#, MATH, AMSSYMB), // DOES NOT DIVIDE
    ('\u{2225}', r#"\parallel"#, MATH, BUILTINS), // PARALLEL TO
    ('\u{2226}', r#"\nparallel"#, MATH, AMSSYMB), // NOT PARALLEL TO
    ('\u{2227}', r#"\wedge"#, MATH, BUILTINS), // LOGICAL AND
    ('\u{2228}', r#"\vee"#, MATH, BUILTINS), // LOGICAL OR
    ('\u{2229}', r#"\cap"#, MATH, BUILTINS), // INTERSECTION
    ('\u{222A}', r#"\cup"#, MATH, BUILTINS), // UNION
    ('\u{222B}', r#"\int"#, MATH, BUILTINS), // INTEGRAL
    ('\u{222C}', r#"\iint"#, MATH, AMSMATH), // DOUBLE INTEGRAL
    ('\u{222D}', r#"\iiint"#, MATH, AMSMATH), // TRIPLE INTEGRAL
    ('\u{222E}', r#"\oint"#, MATH, BUILTINS), // CONTOUR INTEGRAL
    ('\u{222F}', r#"\UnxToiint"#, MATH, STIX), // SURFACE INTEGRAL
    ('\u{2230}', r#"\UnxToiiint"#, MATH, STIX), // VOLUME INTEGRAL
    ('\u{2231}', r#"\UnxTintclockwise"#, MATH, STIX), // CLOCKWISE INTEGRAL
    ('\u{2234}', r#"\therefore"#, MATH, AMSSYMB), // THEREFORE
    ('\u{2235}', r#"\because"#, MATH, AMSSYMB), // BECAUSE
    ('\u{2236}', r#":"#, MATH, BUILTINS), // RATIO
    ('\u{2237}', r#"::"#, MATH, BUILTINS), // PROPORTION
    ('\u{223A}', r#"\mathbin{{:}\!\!{-}\!\!{:}}"#, MATH, BUILTINS), // GEOMETRIC PROPORTION
    ('\u{223B}', r#"\UnxTkernelcontraction"#, MATH, STIX), // HOMOTHETIC
    ('\u{223C}', r#"\sim"#, MATH, BUILTINS), // TILDE OPERATOR
    ('\u{223D}', r#"\backsim"#, MATH, AMSSYMB), // REVERSED TILDE
    ('\u{223E}', r#"\UnxTinvlazys"#, MATH, STIX), // INVERTED LAZY S
    ('\u{2240}', r#"\wr"#, MATH, BUILTINS), // WREATH PRODUCT
    ('\u{2241}', r#"\not\sim"#, MATH, BUILTINS), // NOT TILDE
    ('\u{2243}', r#"\simeq"#, MATH, BUILTINS), // ASYMPTOTICALLY EQUAL TO
    ('\u{2244}', r#"\not\simeq"#, MATH, BUILTINS), // NOT ASYMPTOTICALLY EQUAL TO
    ('\u{2245}', r#"\cong"#, MATH, BUILTINS), // APPROXIMATELY EQUAL TO
    ('\u{2246}', r#"\UnxTsimneqq"#, MATH, STIX), // APPROXIMATELY BUT NOT ACTUALLY EQUAL TO
    ('\u{2247}', r#"\not\cong"#, MATH, BUILTINS), // NEITHER APPROXIMATELY NOR ACTUALLY EQUAL TO
    ('\u{2248}', r#"\approx"#, MATH, BUILTINS), // ALMOST EQUAL TO
    ('\u{2249}', r#"\not\approx"#, MATH, BUILTINS), // NOT ALMOST EQUAL TO
    ('\u{224A}', r#"\approxeq"#, MATH, AMSSYMB), // ALMOST EQUAL OR EQUAL TO
    ('\u{224B}', r#"\UnxTapproxident"#, MATH, STIX), // TRIPLE TILDE
    ('\u{224C}', r#"\UnxTbackcong"#, MATH, STIX), // ALL EQUAL TO
    ('\u{224D}', r#"\asymp"#, MATH, BUILTINS), // EQUIVALENT TO
    ('\u{224E}', r#"\Bumpeq"#, MATH, AMSSYMB), // GEOMETRICALLY EQUIVALENT TO
    ('\u{224F}', r#"\bumpeq"#, MATH, AMSSYMB), // DIFFERENCE BETWEEN
    ('\u{2250}', r#"\doteq"#, MATH, BUILTINS), // APPROACHES THE LIMIT
    ('\u{2251}', r#"\doteqdot"#, MATH, AMSSYMB), // GEOMETRICALLY EQUAL TO
    ('\u{2252}', r#"\fallingdotseq"#, MATH, AMSSYMB), // APPROXIMATELY EQUAL TO OR THE IMAGE OF
    ('\u{2253}', r#"\risingdotseq"#, MATH, AMSSYMB), // IMAGE OF OR APPROXIMATELY EQUAL TO
    ('\u{2254}', r#":="#, MATH, BUILTINS), // COLON EQUALS
    ('\u{2255}', r#"=:"#, MATH, BUILTINS), // EQUALS COLON
    ('\u{2256}', r#"\eqcirc"#, MATH, AMSSYMB), // RING IN EQUAL TO
    ('\u{2257}', r#"\circeq"#, MATH, AMSSYMB), // RING EQUAL TO
    ('\u{2259}', r#"\UnxTwedgeq"#, MATH, STIX), // ESTIMATES
    ('\u{225B}', r#"\UnxTstareq"#, MATH, STIX), // STAR EQUALS
    ('\u{225C}', r#"\triangleq"#, MATH, AMSSYMB), // DELTA EQUAL TO
    ('\u{2260}', r#"\neq"#, MATH, BUILTINS), // NOT EQUAL TO
    ('\u{2261}', r#"\equiv"#, MATH, BUILTINS), // IDENTICAL TO
    ('\u{2262}', r#"\not\equiv"#, MATH, BUILTINS), // NOT IDENTICAL TO
    ('\u{2264}', r#"\leq"#, MATH, BUILTINS), // LESS-THAN OR EQUAL TO
    ('\u{2265}', r#"\geq"#, MATH, BUILTINS), // GREATER-THAN OR EQUAL TO
    ('\u{2266}', r#"\leqq"#, MATH, AMSSYMB), // LESS-THAN OVER EQUAL TO
    ('\u{2267}', r#"\geqq"#, MATH, AMSSYMB), // GREATER-THAN OVER EQUAL TO
    ('\u{2268}', r#"\lneqq"#, MATH, AMSSYMB), // LESS-THAN BUT NOT EQUAL TO
    ('\u{2269}', r#"\gneqq"#, MATH, AMSSYMB), // GREATER-THAN BUT NOT EQUAL TO
    ('\u{226A}', r#"\ll"#, MATH, BUILTINS), // MUCH LESS-THAN
    ('\u{226B}', r#"\gg"#, MATH, BUILTINS), // MUCH GREATER-THAN
    ('\u{226C}', r#"\between"#, MATH, AMSSYMB), // BETWEEN
    ('\u{226D}', r#"\not\kern-0.3em\times"#, MATH, BUILTINS), // NOT EQUIVALENT TO
    ('\u{226E}', r#"\nless"#, MATH, AMSSYMB), // NOT LESS-THAN
    ('\u{226F}', r#"\ngtr"#, MATH, AMSSYMB), // NOT GREATER-THAN
    ('\u{2270}', r#"\nleq"#, MATH, AMSSYMB), // NEITHER LESS-THAN NOR EQUAL TO
    ('\u{2271}', r#"\ngeq"#, MATH, AMSSYMB), // NEITHER GREATER-THAN NOR EQUAL TO
    ('\u{2272}', r#"\lesssim"#, MATH, AMSSYMB), // LESS-THAN OR EQUIVALENT TO
    ('\u{2273}', r#"\gtrsim"#, MATH, AMSSYMB), // GREATER-THAN OR EQUIVALENT TO
    ('\u{2274}', r#"\not\lesssim"#, MATH, AMSSYMB), // NEITHER LESS-THAN NOR EQUIVALENT TO
    ('\u{2275}', r#"\not\gtrsim"#, MATH, AMSSYMB), // NEITHER GREATER-THAN NOR EQUIVALENT TO
    ('\u{2276}', r#"\lessgtr"#, MATH, AMSSYMB), // LESS-THAN OR GREATER-THAN
    ('\u{2277}', r#"\gtrless"#, MATH, AMSSYMB), // GREATER-THAN OR LESS-THAN
    ('\u{2278}', r#"\UnxTnlessgtr"#, MATH, STIX), // NEITHER LESS-THAN NOR GREATER-THAN
    ('\u{2279}', r#"\UnxTngtrless"#, MATH, STIX), // NEITHER GREATER-THAN NOR LESS-THAN
    ('\u{227A}', r#"\prec"#, MATH, BUILTINS), // PRECEDES
    ('\u{227B}', r#"\succ"#, MATH, BUILTINS), // SUCCEEDS
    ('\u{227C}', r#"\preceq"#, MATH, BUILTINS), // PRECEDES OR EQUAL TO
    ('\u{227D}', r#"\succeq"#, MATH, BUILTINS), // SUCCEEDS OR EQUAL TO
    ('\u{227E}', r#"\precsim"#, MATH, AMSSYMB), // PRECEDES OR EQUIVALENT TO
    ('\u{227F}', r#"\succsim"#, MATH, AMSSYMB), // SUCCEEDS OR EQUIVALENT TO
    ('\u{2280}', r#"\nprec"#, MATH, AMSSYMB), // DOES NOT PRECEDE
    ('\u{2281}', r#"\nsucc"#, MATH, AMSSYMB), // DOES NOT SUCCEED
    ('\u{2282}', r#"\subset"#, MATH, BUILTINS), // SUBSET OF
    ('\u{2283}', r#"\supset"#, MATH, BUILTINS), // SUPERSET OF
    ('\u{2284}', r#"\not\subset"#, MATH, BUILTINS), // NOT A SUBSET OF
    ('\u{2285}', r#"\not\supset"#, MATH, BUILTINS), // NOT A SUPERSET OF
    ('\u{2286}', r#"\subseteq"#, MATH, BUILTINS), // SUBSET OF OR EQUAL TO
    ('\u{2287}', r#"\supseteq"#, MATH, BUILTINS), // SUPERSET OF OR EQUAL TO
    ('\u{2288}', r#"\nsubseteq"#, MATH, AMSSYMB), // NEITHER A SUBSET OF NOR EQUAL TO
    ('\u{2289}', r#"\nsupseteq"#, MATH, AMSSYMB), // NEITHER A SUPERSET OF NOR EQUAL TO
    ('\u{228A}', r#"\subsetneq"#, MATH, AMSSYMB), // SUBSET OF WITH NOT EQUAL TO
    ('\u{228B}', r#"\supsetneq"#, MATH, AMSSYMB), // SUPERSET OF WITH NOT EQUAL TO
    ('\u{228E}', r#"\uplus"#, MATH, BUILTINS), // MULTISET UNION
    ('\u{228F}', r#"\sqsubset"#, MATH, AMSSYMB), // SQUARE IMAGE OF
    ('\u{2290}', r#"\sqsupset"#, MATH, AMSSYMB), // SQUARE ORIGINAL OF
    ('\u{2291}', r#"\sqsubseteq"#, MATH, BUILTINS), // SQUARE IMAGE OF OR EQUAL TO
    ('\u{2292}', r#"\sqsupseteq"#, MATH, BUILTINS), // SQUARE ORIGINAL OF OR EQUAL TO
    ('\u{2293}', r#"\sqcap"#, MATH, BUILTINS), // SQUARE CAP
    ('\u{2294}', r#"\sqcup"#, MATH, BUILTINS), // SQUARE CUP
    ('\u{2295}', r#"\oplus"#, MATH, BUILTINS), // CIRCLED PLUS
    ('\u{2296}', r#"\ominus"#, MATH, BUILTINS), // CIRCLED MINUS
    ('\u{2297}', r#"\otimes"#, MATH, BUILTINS), // CIRCLED TIMES
    ('\u{2298}', r#"\oslash"#, MATH, BUILTINS), // CIRCLED DIVISION SLASH
    ('\u{2299}', r#"\odot"#, MATH, BUILTINS), // CIRCLED DOT OPERATOR
    ('\u{229A}', r#"\circledcirc"#, MATH, AMSSYMB), // CIRCLED RING OPERATOR
    ('\u{229B}', r#"\circledast"#, MATH, AMSSYMB), // CIRCLED ASTERISK OPERATOR
    ('\u{229D}', r#"\circleddash"#, MATH, AMSSYMB), // CIRCLED DASH
    ('\u{229E}', r#"\boxplus"#, MATH, AMSSYMB), // SQUARED PLUS
    ('\u{229F}', r#"\boxminus"#, MATH, AMSSYMB), // SQUARED MINUS
    ('\u{22A0}', r#"\boxtimes"#, MATH, AMSSYMB), // SQUARED TIMES
    ('\u{22A1}', r#"\boxdot"#, MATH, AMSSYMB), // SQUARED DOT OPERATOR
    ('\u{22A2}', r#"\vdash"#, MATH, BUILTINS), // RIGHT TACK
    ('\u{22A3}', r#"\dashv"#, MATH, BUILTINS), // LEFT TACK
    ('\u{22A4}', r#"\top"#, MATH, BUILTINS), // DOWN TACK
    ('\u{22A5}', r#"\perp"#, MATH, BUILTINS), // UP TACK
    ('\u{22A7}', r#"\UnxTmodels"#, MATH, STIX), // MODELS
    ('\u{22A8}', r#"\UnxTvDash"#, MATH, STIX), // TRUE
    ('\u{22A9}', r#"\Vdash"#, MATH, AMSSYMB), // FORCES
    ('\u{22AA}', r#"\Vvdash"#, MATH, AMSSYMB), // TRIPLE VERTICAL BAR RIGHT TURNSTILE
    ('\u{22AB}', r#"\UnxTVDash"#, MATH, STIX), // DOUBLE VERTICAL BAR DOUBLE RIGHT TURNSTILE
    ('\u{22AC}', r#"\nvdash"#, MATH, AMSSYMB), // DOES NOT PROVE
    ('\u{22AD}', r#"\nvDash"#, MATH, AMSSYMB), // NOT TRUE
    ('\u{22AE}', r#"\nVdash"#, MATH, AMSSYMB), // DOES NOT FORCE
    ('\u{22AF}', r#"\nVDash"#, MATH, AMSSYMB), // NEGATED DOUBLE VERTICAL BAR DOUBLE RIGHT TURNSTILE
    ('\u{22B2}', r#"\vartriangleleft"#, MATH, AMSSYMB), // NORMAL SUBGROUP OF
    ('\u{22B3}', r#"\vartriangleright"#, MATH, AMSSYMB), // CONTAINS AS NORMAL SUBGROUP
    ('\u{22B4}', r#"\trianglelefteq"#, MATH, AMSSYMB), // NORMAL SUBGROUP OF OR EQUAL TO
    ('\u{22B5}', r#"\trianglerighteq"#, MATH, AMSSYMB), // CONTAINS AS NORMAL SUBGROUP OR EQUAL TO
    ('\u{22B6}', r#"\UnxTorigof"#, MATH, STIX), // ORIGINAL OF
    ('\u{22B7}', r#"\UnxTimageof"#, MATH, STIX), // IMAGE OF
    ('\u{22B8}', r#"\multimap"#, MATH, AMSSYMB), // MULTIMAP
    ('\u{22B9}', r#"\UnxThermitmatrix"#, MATH, STIX), // HERMITIAN CONJUGATE MATRIX
    ('\u{22BA}', r#"\intercal"#, MATH, AMSSYMB), // INTERCALATE
    ('\u{22BB}', r#"\veebar"#, MATH, AMSSYMB), // XOR
    ('\u{22BE}', r#"\UnxTmeasuredrightangle"#, MATH, STIX), // RIGHT ANGLE WITH ARC
    ('\u{22C0}', r#"\bigwedge"#, MATH, BUILTINS), // N-ARY LOGICAL AND
    ('\u{22C1}', r#"\bigvee"#, MATH, BUILTINS), // N-ARY LOGICAL OR
    ('\u{22C2}', r#"\bigcap"#, MATH, BUILTINS), // N-ARY INTERSECTION
    ('\u{22C3}', r#"\bigcup"#, MATH, BUILTINS), // N-ARY UNION
    ('\u{22C4}', r#"\diamond"#, MATH, BUILTINS), // DIAMOND OPERATOR
    ('\u{22C5}', r#"\cdot"#, MATH, BUILTINS), // DOT OPERATOR
    ('\u{22C6}', r#"\star"#, MATH, BUILTINS), // STAR OPERATOR
    ('\u{22C7}', r#"\divideontimes"#, MATH, AMSSYMB), // DIVISION TIMES
    ('\u{22C8}', r#"\bowtie"#, MATH, BUILTINS), // BOWTIE
    ('\u{22C9}', r#"\ltimes"#, MATH, AMSSYMB), // LEFT NORMAL FACTOR SEMIDIRECT PRODUCT
    ('\u{22CA}', r#"\rtimes"#, MATH, AMSSYMB), // RIGHT NORMAL FACTOR SEMIDIRECT PRODUCT
    ('\u{22CB}', r#"\leftthreetimes"#, MATH, AMSSYMB), // LEFT SEMIDIRECT PRODUCT
    ('\u{22CC}', r#"\rightthreetimes"#, MATH, AMSSYMB), // RIGHT SEMIDIRECT PRODUCT
    ('\u{22CD}', r#"\backsimeq"#, MATH, AMSSYMB), // REVERSED TILDE EQUALS
    ('\u{22CE}', r#"\curlyvee"#, MATH, AMSSYMB), // CURLY LOGICAL OR
    ('\u{22CF}', r#"\curlywedge"#, MATH, AMSSYMB), // CURLY LOGICAL AND
    ('\u{22D0}', r#"\Subset"#, MATH, AMSSYMB), // DOUBLE SUBSET
    ('\u{22D1}', r#"\Supset"#, MATH, AMSSYMB), // DOUBLE SUPERSET
    ('\u{22D2}', r#"\Cap"#, MATH, AMSSYMB), // DOUBLE INTERSECTION
    ('\u{22D3}', r#"\Cup"#, MATH, AMSSYMB), // DOUBLE UNION
    ('\u{22D4}', r#"\pitchfork"#, MATH, AMSSYMB), // PITCHFORK
    ('\u{22D6}', r#"\lessdot"#, MATH, AMSSYMB), // LESS-THAN WITH DOT
    ('\u{22D7}', r#"\gtrdot"#, MATH, AMSSYMB), // GREATER-THAN WITH DOT
    ('\u{22D8}', r#"\UnxTlll"#, MATH, STIX), // VERY MUCH LESS-THAN
    ('\u{22D9}', r#"\UnxTggg"#, MATH, STIX), // VERY MUCH GREATER-THAN
    ('\u{22DA}', r#"\lesseqgtr"#, MATH, AMSSYMB), // LESS-THAN EQUAL TO OR GREATER-THAN
    ('\u{22DB}', r#"\gtreqless"#, MATH, AMSSYMB), // GREATER-THAN EQUAL TO OR LESS-THAN
    ('\u{22DE}', r#"\curlyeqprec"#, MATH, AMSSYMB), // EQUAL TO OR PRECEDES
    ('\u{22DF}', r#"\curlyeqsucc"#, MATH, AMSSYMB), // EQUAL TO OR SUCCEEDS
    ('\u{22E2}', r#"\not\sqsubseteq"#, MATH, BUILTINS), // NOT SQUARE IMAGE OF OR EQUAL TO
    ('\u{22E3}', r#"\not\sqsupseteq"#, MATH, BUILTINS), // NOT SQUARE ORIGINAL OF OR EQUAL TO
    ('\u{22E6}', r#"\lnsim"#, MATH, AMSSYMB), // LESS-THAN BUT NOT EQUIVALENT TO
    ('\u{22E7}', r#"\gnsim"#, MATH, AMSSYMB), // GREATER-THAN BUT NOT EQUIVALENT TO
    ('\u{22E8}', r#"\UnxTprecnsim"#, MATH, STIX), // PRECEDES BUT NOT EQUIVALENT TO
    ('\u{22E9}', r#"\succnsim"#, MATH, AMSSYMB), // SUCCEEDS BUT NOT EQUIVALENT TO
    ('\u{22EA}', r#"\ntriangleleft"#, MATH, AMSSYMB), // NOT NORMAL SUBGROUP OF
    ('\u{22EB}', r#"\ntriangleright"#, MATH, AMSSYMB), // DOES NOT CONTAIN AS NORMAL SUBGROUP
    ('\u{22EC}', r#"\ntrianglelefteq"#, MATH, AMSSYMB), // NOT NORMAL SUBGROUP OF OR EQUAL TO
    ('\u{22ED}', r#"\ntrianglerighteq"#, MATH, AMSSYMB), // DOES NOT CONTAIN AS NORMAL SUBGROUP OR EQUAL
    ('\u{22EE}', r#"\vdots"#, MATH, BUILTINS), // VERTICAL ELLIPSIS
    ('\u{22EF}', r#"\cdots"#, MATH, BUILTINS), // MIDLINE HORIZONTAL ELLIPSIS
    ('\u{22F0}', r#"\UnxTadots"#, MATH, STIX), // UP RIGHT DIAGONAL ELLIPSIS
    ('\u{22F1}', r#"\ddots"#, MATH, BUILTINS), // DOWN RIGHT DIAGONAL ELLIPSIS
    ('\u{2305}', r#"\barwedge"#, MATH, AMSSYMB), // PROJECTIVE
    ('\u{2306}', r#"\UnxTvardoublebarwedge"#, MATH, STIX), // PERSPECTIVE
    ('\u{2308}', r#"\lceil"#, MATH, BUILTINS), // LEFT CEILING
    ('\u{2309}', r#"\rceil"#, MATH, BUILTINS), // RIGHT CEILING
    ('\u{230A}', r#"\lfloor"#, MATH, BUILTINS), // LEFT FLOOR
    ('\u{230B}', r#"\rfloor"#, MATH, BUILTINS), // RIGHT FLOOR
    ('\u{2315}', r#"\UnxTrecorder"#, MATH, WASY), // TELEPHONE RECORDER
    ('\u{2316}', r#"\mathchar"2208"#, MATH, BUILTINS), // POSITION INDICATOR
    ('\u{231C}', r#"\ulcorner"#, MATH, AMSSYMB), // TOP LEFT CORNER
    ('\u{231D}', r#"\urcorner"#, MATH, AMSSYMB), // TOP RIGHT CORNER
    ('\u{231E}', r#"\llcorner"#, MATH, AMSSYMB), // BOTTOM LEFT CORNER
    ('\u{231F}', r#"\lrcorner"#, MATH, AMSSYMB), // BOTTOM RIGHT CORNER
    ('\u{2322}', r#"\frown"#, MATH, BUILTINS), // FROWN
    ('\u{2323}', r#"\smile"#, MATH, BUILTINS), // SMILE
    ('\u{2329}', r#"\textlangle"#, TEXT, BUILTINS), // LEFT-POINTING ANGLE BRACKET
    ('\u{232A}', r#"\textrangle"#, TEXT, BUILTINS), // RIGHT-POINTING ANGLE BRACKET
    ('\u{23B0}', r#"\lmoustache"#, MATH, BUILTINS), // UPPER LEFT OR LOWER RIGHT CURLY BRACKET SECTION
    ('\u{23B1}', r#"\rmoustache"#, MATH, BUILTINS), // UPPER RIGHT OR LOWER LEFT CURLY BRACKET SECTION
    ('\u{2422}', r#"\textblank"#, TEXT, BUILTINS), // BLANK SYMBOL
    ('\u{2423}', r#"\textvisiblespace"#, TEXT, BUILTINS), // OPEN BOX
    ('\u{25A0}', r#"\blacksquare"#, MATH, AMSSYMB), // BLACK SQUARE
    ('\u{25A1}', r#"\square"#, MATH, AMSSYMB), // WHITE SQUARE
    ('\u{25AA}', r#"{\small\ensuremath{\blacksquare}}"#, TEXT, AMSSYMB), // BLACK SMALL SQUARE
    ('\u{25AD}', r#"\fbox{~~}"#, TEXT, BUILTINS), // WHITE RECTANGLE
    ('\u{25B3}', r#"\bigtriangleup"#, MATH, BUILTINS), // WHITE UP-POINTING TRIANGLE
    ('\u{25B4}', r#"\blacktriangle"#, MATH, AMSSYMB), // BLACK UP-POINTING SMALL TRIANGLE
    ('\u{25B5}', r#"\vartriangle"#, MATH, AMSSYMB), // WHITE UP-POINTING SMALL TRIANGLE
    ('\u{25B8}', r#"\blacktriangleright"#, MATH, AMSSYMB), // BLACK RIGHT-POINTING SMALL TRIANGLE
    ('\u{25B9}', r#"\triangleright"#, MATH, BUILTINS), // WHITE RIGHT-POINTING SMALL TRIANGLE
    ('\u{25BD}', r#"\bigtriangledown"#, MATH, BUILTINS), // WHITE DOWN-POINTING TRIANGLE
    ('\u{25BE}', r#"\blacktriangledown"#, MATH, AMSSYMB), // BLACK DOWN-POINTING SMALL TRIANGLE
    ('\u{25BF}', r#"\triangledown"#, MATH, AMSSYMB), // WHITE DOWN-POINTING SMALL TRIANGLE
    ('\u{25C2}', r#"\blacktriangleleft"#, MATH, AMSSYMB), // BLACK LEFT-POINTING SMALL TRIANGLE
    ('\u{25C3}', r#"\triangleleft"#, MATH, BUILTINS), // WHITE LEFT-POINTING SMALL TRIANGLE
    ('\u{25CA}', r#"\lozenge"#, MATH, AMSSYMB), // LOZENGE
    ('\u{25CB}', r#"\bigcirc"#, MATH, BUILTINS), // WHITE CIRCLE
    ('\u{25E6}', r#"\textopenbullet"#, TEXT, BUILTINS), // WHITE BULLET
    ('\u{25EF}', r#"\textbigcircle"#, TEXT, BUILTINS), // LARGE CIRCLE
    ('\u{2662}', r#"\diamond"#, MATH, BUILTINS), // WHITE DIAMOND SUIT
    ('\u{2669}', r#"\UnxTquarternote"#, MATH, STIX), // QUARTER NOTE
    ('\u{266A}', r#"\textmusicalnote"#, TEXT, BUILTINS), // EIGHTH NOTE
    ('\u{266D}', r#"\flat"#, MATH, BUILTINS), // MUSIC FLAT SIGN
    ('\u{266E}', r#"\natural"#, MATH, BUILTINS), // MUSIC NATURAL SIGN
    ('\u{266F}', r#"\sharp"#, MATH, BUILTINS), // MUSIC SHARP SIGN
    ('\u{27E8}', r#"\langle"#, MATH, BUILTINS), // MATHEMATICAL LEFT ANGLE BRACKET
    ('\u{27E9}', r#"\rangle"#, MATH, BUILTINS), // MATHEMATICAL RIGHT ANGLE BRACKET
    ('\u{27F5}', r#"\longleftarrow"#, MATH, BUILTINS), // LONG LEFTWARDS ARROW
    ('\u{27F6}', r#"\longrightarrow"#, MATH, BUILTINS), // LONG RIGHTWARDS ARROW
    ('\u{27F7}', r#"\longleftrightarrow"#, MATH, BUILTINS), // LONG LEFT RIGHT ARROW
    ('\u{27F8}', r#"\Longleftarrow"#, MATH, BUILTINS), // LONG LEFTWARDS DOUBLE ARROW
    ('\u{27F9}', r#"\Longrightarrow"#, MATH, BUILTINS), // LONG RIGHTWARDS DOUBLE ARROW
    ('\u{27FA}', r#"\Longleftrightarrow"#, MATH, BUILTINS), // LONG LEFT RIGHT DOUBLE ARROW
    ('\u{27FC}', r#"\longmapsto"#, MATH, BUILTINS), // LONG RIGHTWARDS ARROW FROM BAR
    ('\u{27FF}', r#"\sim\joinrel\leadsto"#, MATH, AMSSYMB), // LONG RIGHTWARDS SQUIGGLE ARROW
    ('\u{2993}', r#"<\kern-0.58em("#, MATH, BUILTINS), // LEFT ARC LESS-THAN BRACKET
    ('\u{29EB}', r#"\blacklozenge"#, MATH, AMSSYMB), // BLACK LOZENGE
    ('\u{2A0F}', r#"\UnxTfint"#, MATH, STIX), // INTEGRAL AVERAGE WITH SLASH
    ('\u{2A16}', r#"\UnxTsqint"#, MATH, STIX), // QUATERNION INTEGRAL OPERATOR
    ('\u{2A3F}', r#"\amalg"#, MATH, BUILTINS), // AMALGAMATION OR COPRODUCT
    ('\u{2A6E}', r#"\stackrel{*}{=}"#, MATH, BUILTINS), // EQUALS WITH ASTERISK
    ('\u{2A75}', r#"=="#, ANY, BUILTINS), // TWO CONSECUTIVE EQUALS SIGNS
    ('\u{2A7D}', r#"\leqslant"#, MATH, AMSSYMB), // LESS-THAN OR SLANTED EQUAL TO
    ('\u{2A7E}', r#"\geqslant"#, MATH, AMSSYMB), // GREATER-THAN OR SLANTED EQUAL TO
    ('\u{2A85}', r#"\lessapprox"#, MATH, AMSSYMB), // LESS-THAN OR APPROXIMATE
    ('\u{2A86}', r#"\gtrapprox"#, MATH, AMSSYMB), // GREATER-THAN OR APPROXIMATE
    ('\u{2A87}', r#"\lneq"#, MATH, AMSSYMB), // LESS-THAN AND SINGLE-LINE NOT EQUAL TO
    ('\u{2A88}', r#"\gneq"#, MATH, AMSSYMB), // GREATER-THAN AND SINGLE-LINE NOT EQUAL TO
    ('\u{2A89}', r#"\lnapprox"#, MATH, AMSSYMB), // LESS-THAN AND NOT APPROXIMATE
    ('\u{2A8A}', r#"\gnapprox"#, MATH, AMSSYMB), // GREATER-THAN AND NOT APPROXIMATE
    ('\u{2A8B}', r#"\lesseqqgtr"#, MATH, AMSSYMB), // LESS-THAN ABOVE DOUBLE-LINE EQUAL ABOVE GREATER-THAN
    ('\u{2A8C}', r#"\gtreqqless"#, MATH, AMSSYMB), // GREATER-THAN ABOVE DOUBLE-LINE EQUAL ABOVE LESS-THAN
    ('\u{2A95}', r#"\eqslantless"#, MATH, AMSSYMB), // SLANTED EQUAL TO OR LESS-THAN
    ('\u{2A96}', r#"\eqslantgtr"#, MATH, AMSSYMB), // SLANTED EQUAL TO OR GREATER-THAN
    ('\u{2AAF}', r#"\preceq"#, MATH, BUILTINS), // PRECEDES ABOVE SINGLE-LINE EQUALS SIGN
    ('\u{2AB0}', r#"\succeq"#, MATH, BUILTINS), // SUCCEEDS ABOVE SINGLE-LINE EQUALS SIGN
    ('\u{2AB5}', r#"\precneqq"#, MATH, AMSSYMB), // PRECEDES ABOVE NOT EQUAL TO
    ('\u{2AB6}', r#"\succneqq"#, MATH, AMSSYMB), // SUCCEEDS ABOVE NOT EQUAL TO
    ('\u{2AB7}', r#"\precapprox"#, MATH, AMSSYMB), // PRECEDES ABOVE ALMOST EQUAL TO
    ('\u{2AB8}', r#"\succapprox"#, MATH, AMSSYMB), // SUCCEEDS ABOVE ALMOST EQUAL TO
    ('\u{2AB9}', r#"\precnapprox"#, MATH, AMSSYMB), // PRECEDES ABOVE NOT ALMOST EQUAL TO
    ('\u{2ABA}', r#"\succnapprox"#, MATH, AMSSYMB), // SUCCEEDS ABOVE NOT ALMOST EQUAL TO
    ('\u{2AC5}', r#"\subseteqq"#, MATH, AMSSYMB), // SUBSET OF ABOVE EQUALS SIGN
    ('\u{2AC6}', r#"\supseteqq"#, MATH, AMSSYMB), // SUPERSET OF ABOVE EQUALS SIGN
    ('\u{2ACB}', r#"\subsetneqq"#, MATH, AMSSYMB), // SUBSET OF ABOVE NOT EQUAL TO
    ('\u{2ACC}', r#"\supsetneqq"#, MATH, AMSSYMB), // SUPERSET OF ABOVE NOT EQUAL TO
    ('\u{2AFD}', r#"{{/}\!\!{/}}"#, MATH, BUILTINS), // DOUBLE SOLIDUS OPERATOR
    ('\u{3008}', r#"\langle"#, MATH, BUILTINS), // LEFT ANGLE BRACKET
    ('\u{3009}', r#"\rangle"#, MATH, BUILTINS), // RIGHT ANGLE BRACKET
    ('\u{FB00}', r#"ff"#, ANY, BUILTINS), // LATIN SMALL LIGATURE FF
    ('\u{FB01}', r#"fi"#, ANY, BUILTINS), // LATIN SMALL LIGATURE FI
    ('\u{FB02}', r#"fl"#, ANY, BUILTINS), // LATIN SMALL LIGATURE FL
    ('\u{FB03}', r#"ffi"#, ANY, BUILTINS), // LATIN SMALL LIGATURE FFI
    ('\u{FB04}', r#"ffl"#, ANY, BUILTINS), // LATIN SMALL LIGATURE FFL
    ('\u{1D400}', r#"\mathbf{A}"#, MATH, BUILTINS), // MATHEMATICAL BOLD CAPITAL A
    ('\u{1D401}', r#"\mathbf{B}"#, MATH, BUILTINS), // MATHEMATICAL BOLD CAPITAL B
    ('\u{1D402}', r#"\mathbf{C}"#, MATH, BUILTINS), // MATHEMATICAL BOLD CAPITAL C
    ('\u{1D403}', r#"\mathbf{D}"#, MATH, BUILTINS), // MATHEMATICAL BOLD CAPITAL D
    ('\u{1D404}', r#"\mathbf{E}"#, MATH, BUILTINS), // MATHEMATICAL BOLD CAPITAL E
    ('\u{1D405}', r#"\mathbf{F}"#, MATH, BUILTINS), // MATHEMATICAL BOLD CAPITAL F
    ('\u{1D406}', r#"\mathbf{G}"#, MATH, BUILTINS), // MATHEMATICAL BOLD CAPITAL G
    ('\u{1D407}', r#"\mathbf{H}"#, MATH, BUILTINS), // MATHEMATICAL BOLD CAPITAL H
    ('\u{1D408}', r#"\mathbf{I}"#, MATH, BUILTINS), // MATHEMATICAL BOLD CAPITAL I
    ('\u{1D409}', r#"\mathbf{J}"#, MATH, BUILTINS), // MATHEMATICAL BOLD CAPITAL J
    ('\u{1D40A}', r#"\mathbf{K}"#, MATH, BUILTINS), // MATHEMATICAL BOLD CAPITAL K
    ('\u{1D40B}', r#"\mathbf{L}"#, MATH, BUILTINS), // MATHEMATICAL BOLD CAPITAL L
    ('\u{1D40C}', r#"\mathbf{M}"#, MATH, BUILTINS), // MATHEMATICAL BOLD CAPITAL M
    ('\u{1D40D}', r#"\mathbf{N}"#, MATH, BUILTINS), // MATHEMATICAL BOLD CAPITAL N
    ('\u{1D40E}', r#"\mathbf{O}"#, MATH, BUILTINS), // MATHEMATICAL BOLD CAPITAL O
    ('\u{1D40F}', r#"\mathbf{P}"#, MATH, BUILTINS), // MATHEMATICAL BOLD CAPITAL P
    ('\u{1D410}', r#"\mathbf{Q}"#, MATH, BUILTINS), // MATHEMATICAL BOLD CAPITAL Q
    ('\u{1D411}', r#"\mathbf{R}"#, MATH, BUILTINS), // MATHEMATICAL BOLD CAPITAL R
    ('\u{1D412}', r#"\mathbf{S}"#, MATH, BUILTINS), // MATHEMATICAL BOLD CAPITAL S
    ('\u{1D413}', r#"\mathbf{T}"#, MATH, BUILTINS), // MATHEMATICAL BOLD CAPITAL T
    ('\u{1D414}', r#"\mathbf{U}"#, MATH, BUILTINS), // MATHEMATICAL BOLD CAPITAL U
    ('\u{1D415}', r#"\mathbf{V}"#, MATH, BUILTINS), // MATHEMATICAL BOLD CAPITAL V
    ('\u{1D416}', r#"\mathbf{W}"#, MATH, BUILTINS), // MATHEMATICAL BOLD CAPITAL W
    ('\u{1D417}', r#"\mathbf{X}"#, MATH, BUILTINS), // MATHEMATICAL BOLD CAPITAL X
    ('\u{1D418}', r#"\mathbf{Y}"#, MATH, BUILTINS), // MATHEMATICAL BOLD CAPITAL Y
    ('\u{1D419}', r#"\mathbf{Z}"#, MATH, BUILTINS), // MATHEMATICAL BOLD CAPITAL Z
    ('\u{1D41A}', r#"\mathbf{a}"#, MATH, BUILTINS), // MATHEMATICAL BOLD SMALL A
    ('\u{1D41B}', r#"\mathbf{b}"#, MATH, BUILTINS), // MATHEMATICAL BOLD SMALL B
    ('\u{1D41C}', r#"\mathbf{c}"#, MATH, BUILTINS), // MATHEMATICAL BOLD SMALL C
    ('\u{1D41D}', r#"\mathbf{d}"#, MATH, BUILTINS), // MATHEMATICAL BOLD SMALL D
    ('\u{1D41E}', r#"\mathbf{e}"#, MATH, BUILTINS), // MATHEMATICAL BOLD SMALL E
    ('\u{1D41F}', r#"\mathbf{f}"#, MATH, BUILTINS), // MATHEMATICAL BOLD SMALL F
    ('\u{1D420}', r#"\mathbf{g}"#, MATH, BUILTINS), // MATHEMATICAL BOLD SMALL G
    ('\u{1D421}', r#"\mathbf{h}"#, MATH, BUILTINS), // MATHEMATICAL BOLD SMALL H
    ('\u{1D422}', r#"\mathbf{i}"#, MATH, BUILTINS), // MATHEMATICAL BOLD SMALL I
    ('\u{1D423}', r#"\mathbf{j}"#, MATH, BUILTINS), // MATHEMATICAL BOLD SMALL J
    ('\u{1D424}', r#"\mathbf{k}"#, MATH, BUILTINS), // MATHEMATICAL BOLD SMALL K
    ('\u{1D425}', r#"\mathbf{l}"#, MATH, BUILTINS), // MATHEMATICAL BOLD SMALL L
    ('\u{1D426}', r#"\mathbf{m}"#, MATH, BUILTINS), // MATHEMATICAL BOLD SMALL M
    ('\u{1D427}', r#"\mathbf{n}"#, MATH, BUILTINS), // MATHEMATICAL BOLD SMALL N
    ('\u{1D428}', r#"\mathbf{o}"#, MATH, BUILTINS), // MATHEMATICAL BOLD SMALL O
    ('\u{1D429}', r#"\mathbf{p}"#, MATH, BUILTINS), // MATHEMATICAL BOLD SMALL P
    ('\u{1D42A}', r#"\mathbf{q}"#, MATH, BUILTINS), // MATHEMATICAL BOLD SMALL Q
    ('\u{1D42B}', r#"\mathbf{r}"#, MATH, BUILTINS), // MATHEMATICAL BOLD SMALL R
    ('\u{1D42C}', r#"\mathbf{s}"#, MATH, BUILTINS), // MATHEMATICAL BOLD SMALL S
    ('\u{1D42D}', r#"\mathbf{t}"#, MATH, BUILTINS), // MATHEMATICAL BOLD SMALL T
    ('\u{1D42E}', r#"\mathbf{u}"#, MATH, BUILTINS), // MATHEMATICAL BOLD SMALL U
    ('\u{1D42F}', r#"\mathbf{v}"#, MATH, BUILTINS), // MATHEMATICAL BOLD SMALL V
    ('\u{1D430}', r#"\mathbf{w}"#, MATH, BUILTINS), // MATHEMATICAL BOLD SMALL W
    ('\u{1D431}', r#"\mathbf{x}"#, MATH, BUILTINS), // MATHEMATICAL BOLD SMALL X
    ('\u{1D432}', r#"\mathbf{y}"#, MATH, BUILTINS), // MATHEMATICAL BOLD SMALL Y
    ('\u{1D433}', r#"\mathbf{z}"#, MATH, BUILTINS), // MATHEMATICAL BOLD SMALL Z
    ('\u{1D434}', r#"\mathit{A}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC CAPITAL A
    ('\u{1D435}', r#"\mathit{B}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC CAPITAL B
    ('\u{1D436}', r#"\mathit{C}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC CAPITAL C
    ('\u{1D437}', r#"\mathit{D}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC CAPITAL D
    ('\u{1D438}', r#"\mathit{E}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC CAPITAL E
    ('\u{1D439}', r#"\mathit{F}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC CAPITAL F
    ('\u{1D43A}', r#"\mathit{G}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC CAPITAL G
    ('\u{1D43B}', r#"\mathit{H}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC CAPITAL H
    ('\u{1D43C}', r#"\mathit{I}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC CAPITAL I
    ('\u{1D43D}', r#"\mathit{J}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC CAPITAL J
    ('\u{1D43E}', r#"\mathit{K}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC CAPITAL K
    ('\u{1D43F}', r#"\mathit{L}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC CAPITAL L
    ('\u{1D440}', r#"\mathit{M}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC CAPITAL M
    ('\u{1D441}', r#"\mathit{N}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC CAPITAL N
    ('\u{1D442}', r#"\mathit{O}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC CAPITAL O
    ('\u{1D443}', r#"\mathit{P}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC CAPITAL P
    ('\u{1D444}', r#"\mathit{Q}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC CAPITAL Q
    ('\u{1D445}', r#"\mathit{R}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC CAPITAL R
    ('\u{1D446}', r#"\mathit{S}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC CAPITAL S
    ('\u{1D447}', r#"\mathit{T}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC CAPITAL T
    ('\u{1D448}', r#"\mathit{U}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC CAPITAL U
    ('\u{1D449}', r#"\mathit{V}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC CAPITAL V
    ('\u{1D44A}', r#"\mathit{W}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC CAPITAL W
    ('\u{1D44B}', r#"\mathit{X}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC CAPITAL X
    ('\u{1D44C}', r#"\mathit{Y}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC CAPITAL Y
    ('\u{1D44D}', r#"\mathit{Z}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC CAPITAL Z
    ('\u{1D44E}', r#"\mathit{a}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC SMALL A
    ('\u{1D44F}', r#"\mathit{b}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC SMALL B
    ('\u{1D450}', r#"\mathit{c}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC SMALL C
    ('\u{1D451}', r#"\mathit{d}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC SMALL D
    ('\u{1D452}', r#"\mathit{e}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC SMALL E
    ('\u{1D453}', r#"\mathit{f}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC SMALL F
    ('\u{1D454}', r#"\mathit{g}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC SMALL G
    ('\u{1D455}', r#"\mathit{h}"#, MATH, BUILTINS), // (unnamed code point)
    ('\u{1D456}', r#"\mathit{i}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC SMALL I
    ('\u{1D457}', r#"\mathit{j}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC SMALL J
    ('\u{1D458}', r#"\mathit{k}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC SMALL K
    ('\u{1D459}', r#"\mathit{l}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC SMALL L
    ('\u{1D45A}', r#"\mathit{m}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC SMALL M
    ('\u{1D45B}', r#"\mathit{n}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC SMALL N
    ('\u{1D45C}', r#"\mathit{o}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC SMALL O
    ('\u{1D45D}', r#"\mathit{p}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC SMALL P
    ('\u{1D45E}', r#"\mathit{q}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC SMALL Q
    ('\u{1D45F}', r#"\mathit{r}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC SMALL R
    ('\u{1D460}', r#"\mathit{s}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC SMALL S
    ('\u{1D461}', r#"\mathit{t}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC SMALL T
    ('\u{1D462}', r#"\mathit{u}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC SMALL U
    ('\u{1D463}', r#"\mathit{v}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC SMALL V
    ('\u{1D464}', r#"\mathit{w}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC SMALL W
    ('\u{1D465}', r#"\mathit{x}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC SMALL X
    ('\u{1D466}', r#"\mathit{y}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC SMALL Y
    ('\u{1D467}', r#"\mathit{z}"#, MATH, BUILTINS), // MATHEMATICAL ITALIC SMALL Z
    ('\u{1D468}', r#"\boldsymbol{\mathit{A}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL A
    ('\u{1D469}', r#"\boldsymbol{\mathit{B}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL B
    ('\u{1D46A}', r#"\boldsymbol{\mathit{C}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL C
    ('\u{1D46B}', r#"\boldsymbol{\mathit{D}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL D
    ('\u{1D46C}', r#"\boldsymbol{\mathit{E}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL E
    ('\u{1D46D}', r#"\boldsymbol{\mathit{F}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL F
    ('\u{1D46E}', r#"\boldsymbol{\mathit{G}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL G
    ('\u{1D46F}', r#"\boldsymbol{\mathit{H}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL H
    ('\u{1D470}', r#"\boldsymbol{\mathit{I}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL I
    ('\u{1D471}', r#"\boldsymbol{\mathit{J}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL J
    ('\u{1D472}', r#"\boldsymbol{\mathit{K}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL K
    ('\u{1D473}', r#"\boldsymbol{\mathit{L}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL L
    ('\u{1D474}', r#"\boldsymbol{\mathit{M}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL M
    ('\u{1D475}', r#"\boldsymbol{\mathit{N}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL N
    ('\u{1D476}', r#"\boldsymbol{\mathit{O}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL O
    ('\u{1D477}', r#"\boldsymbol{\mathit{P}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL P
    ('\u{1D478}', r#"\boldsymbol{\mathit{Q}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL Q
    ('\u{1D479}', r#"\boldsymbol{\mathit{R}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL R
    ('\u{1D47A}', r#"\boldsymbol{\mathit{S}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL S
    ('\u{1D47B}', r#"\boldsymbol{\mathit{T}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL T
    ('\u{1D47C}', r#"\boldsymbol{\mathit{U}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL U
    ('\u{1D47D}', r#"\boldsymbol{\mathit{V}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL V
    ('\u{1D47E}', r#"\boldsymbol{\mathit{W}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL W
    ('\u{1D47F}', r#"\boldsymbol{\mathit{X}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL X
    ('\u{1D480}', r#"\boldsymbol{\mathit{Y}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL Y
    ('\u{1D481}', r#"\boldsymbol{\mathit{Z}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL Z
    ('\u{1D482}', r#"\boldsymbol{\mathit{a}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL A
    ('\u{1D483}', r#"\boldsymbol{\mathit{b}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL B
    ('\u{1D484}', r#"\boldsymbol{\mathit{c}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL C
    ('\u{1D485}', r#"\boldsymbol{\mathit{d}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL D
    ('\u{1D486}', r#"\boldsymbol{\mathit{e}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL E
    ('\u{1D487}', r#"\boldsymbol{\mathit{f}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL F
    ('\u{1D488}', r#"\boldsymbol{\mathit{g}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL G
    ('\u{1D489}', r#"\boldsymbol{\mathit{h}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL H
    ('\u{1D48A}', r#"\boldsymbol{\mathit{i}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL I
    ('\u{1D48B}', r#"\boldsymbol{\mathit{j}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL J
    ('\u{1D48C}', r#"\boldsymbol{\mathit{k}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL K
    ('\u{1D48D}', r#"\boldsymbol{\mathit{l}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL L
    ('\u{1D48E}', r#"\boldsymbol{\mathit{m}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL M
    ('\u{1D48F}', r#"\boldsymbol{\mathit{n}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL N
    ('\u{1D490}', r#"\boldsymbol{\mathit{o}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL O
    ('\u{1D491}', r#"\boldsymbol{\mathit{p}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL P
    ('\u{1D492}', r#"\boldsymbol{\mathit{q}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL Q
    ('\u{1D493}', r#"\boldsymbol{\mathit{r}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL R
    ('\u{1D494}', r#"\boldsymbol{\mathit{s}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL S
    ('\u{1D495}', r#"\boldsymbol{\mathit{t}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL T
    ('\u{1D496}', r#"\boldsymbol{\mathit{u}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL U
    ('\u{1D497}', r#"\boldsymbol{\mathit{v}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL V
    ('\u{1D498}', r#"\boldsymbol{\mathit{w}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL W
    ('\u{1D499}', r#"\boldsymbol{\mathit{x}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL X
    ('\u{1D49A}', r#"\boldsymbol{\mathit{y}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL Y
    ('\u{1D49B}', r#"\boldsymbol{\mathit{z}}"#, MATH, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL Z
    ('\u{1D49C}', r#"\mathscr{A}"#, MATH, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL A
    ('\u{1D49D}', r#"\mathscr{B}"#, MATH, MATHRSFS), // (unnamed code point)
    ('\u{1D49E}', r#"\mathscr{C}"#, MATH, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL C
    ('\u{1D49F}', r#"\mathscr{D}"#, MATH, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL D
    ('\u{1D4A0}', r#"\mathscr{E}"#, MATH, MATHRSFS), // (unnamed code point)
    ('\u{1D4A1}', r#"\mathscr{F}"#, MATH, MATHRSFS), // (unnamed code point)
    ('\u{1D4A2}', r#"\mathscr{G}"#, MATH, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL G
    ('\u{1D4A3}', r#"\mathscr{H}"#, MATH, MATHRSFS), // (unnamed code point)
    ('\u{1D4A4}', r#"\mathscr{I}"#, MATH, MATHRSFS), // (unnamed code point)
    ('\u{1D4A5}', r#"\mathscr{J}"#, MATH, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL J
    ('\u{1D4A6}', r#"\mathscr{K}"#, MATH, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL K
    ('\u{1D4A7}', r#"\mathscr{L}"#, MATH, MATHRSFS), // (unnamed code point)
    ('\u{1D4A8}', r#"\mathscr{M}"#, MATH, MATHRSFS), // (unnamed code point)
    ('\u{1D4A9}', r#"\mathscr{N}"#, MATH, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL N
    ('\u{1D4AA}', r#"\mathscr{O}"#, MATH, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL O
    ('\u{1D4AB}', r#"\mathscr{P}"#, MATH, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL P
    ('\u{1D4AC}', r#"\mathscr{Q}"#, MATH, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL Q
    ('\u{1D4AD}', r#"\mathscr{R}"#, MATH, MATHRSFS), // (unnamed code point)
    ('\u{1D4AE}', r#"\mathscr{S}"#, MATH, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL S
    ('\u{1D4AF}', r#"\mathscr{T}"#, MATH, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL T
    ('\u{1D4B0}', r#"\mathscr{U}"#, MATH, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL U
    ('\u{1D4B1}', r#"\mathscr{V}"#, MATH, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL V
    ('\u{1D4B2}', r#"\mathscr{W}"#, MATH, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL W
    ('\u{1D4B3}', r#"\mathscr{X}"#, MATH, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL X
    ('\u{1D4B4}', r#"\mathscr{Y}"#, MATH, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL Y
    ('\u{1D4B5}', r#"\mathscr{Z}"#, MATH, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL Z
    ('\u{1D4B6}', r#"\UnxTScr{a}"#, MATH, SCRIPT), // MATHEMATICAL SCRIPT SMALL A
    ('\u{1D4B7}', r#"\UnxTScr{b}"#, MATH, SCRIPT), // MATHEMATICAL SCRIPT SMALL B
    ('\u{1D4B8}', r#"\UnxTScr{c}"#, MATH, SCRIPT), // MATHEMATICAL SCRIPT SMALL C
    ('\u{1D4B9}', r#"\UnxTScr{d}"#, MATH, SCRIPT), // MATHEMATICAL SCRIPT SMALL D
    ('\u{1D4BB}', r#"\UnxTScr{f}"#, MATH, SCRIPT), // MATHEMATICAL SCRIPT SMALL F
    ('\u{1D4BD}', r#"\UnxTScr{h}"#, MATH, SCRIPT), // MATHEMATICAL SCRIPT SMALL H
    ('\u{1D4BE}', r#"\UnxTScr{i}"#, MATH, SCRIPT), // MATHEMATICAL SCRIPT SMALL I
    ('\u{1D4BF}', r#"\UnxTScr{j}"#, MATH, SCRIPT), // MATHEMATICAL SCRIPT SMALL J
    ('\u{1D4C0}', r#"\UnxTScr{k}"#, MATH, SCRIPT), // MATHEMATICAL SCRIPT SMALL K
    ('\u{1D4C1}', r#"\UnxTScr{l}"#, MATH, SCRIPT), // MATHEMATICAL SCRIPT SMALL L
    ('\u{1D4C2}', r#"\UnxTScr{m}"#, MATH, SCRIPT), // MATHEMATICAL SCRIPT SMALL M
    ('\u{1D4C3}', r#"\UnxTScr{n}"#, MATH, SCRIPT), // MATHEMATICAL SCRIPT SMALL N
    ('\u{1D4C5}', r#"\UnxTScr{p}"#, MATH, SCRIPT), // MATHEMATICAL SCRIPT SMALL P
    ('\u{1D4C6}', r#"\UnxTScr{q}"#, MATH, SCRIPT), // MATHEMATICAL SCRIPT SMALL Q
    ('\u{1D4C7}', r#"\UnxTScr{r}"#, MATH, SCRIPT), // MATHEMATICAL SCRIPT SMALL R
    ('\u{1D4C8}', r#"\UnxTScr{s}"#, MATH, SCRIPT), // MATHEMATICAL SCRIPT SMALL S
    ('\u{1D4C9}', r#"\UnxTScr{t}"#, MATH, SCRIPT), // MATHEMATICAL SCRIPT SMALL T
    ('\u{1D4CA}', r#"\UnxTScr{u}"#, MATH, SCRIPT), // MATHEMATICAL SCRIPT SMALL U
    ('\u{1D4CB}', r#"\UnxTScr{v}"#, MATH, SCRIPT), // MATHEMATICAL SCRIPT SMALL V
    ('\u{1D4CC}', r#"\UnxTScr{w}"#, MATH, SCRIPT), // MATHEMATICAL SCRIPT SMALL W
    ('\u{1D4CD}', r#"\UnxTScr{x}"#, MATH, SCRIPT), // MATHEMATICAL SCRIPT SMALL X
    ('\u{1D4CE}', r#"\UnxTScr{y}"#, MATH, SCRIPT), // MATHEMATICAL SCRIPT SMALL Y
    ('\u{1D4CF}', r#"\UnxTScr{z}"#, MATH, SCRIPT), // MATHEMATICAL SCRIPT SMALL Z
    ('\u{1D504}', r#"\mathfrak{A}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL A
    ('\u{1D505}', r#"\mathfrak{B}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL B
    ('\u{1D506}', r#"\mathfrak{C}"#, MATH, AMSSYMB), // (unnamed code point)
    ('\u{1D507}', r#"\mathfrak{D}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL D
    ('\u{1D508}', r#"\mathfrak{E}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL E
    ('\u{1D509}', r#"\mathfrak{F}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL F
    ('\u{1D50A}', r#"\mathfrak{G}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL G
    ('\u{1D50B}', r#"\mathfrak{H}"#, MATH, AMSSYMB), // (unnamed code point)
    ('\u{1D50C}', r#"\mathfrak{I}"#, MATH, AMSSYMB), // (unnamed code point)
    ('\u{1D50D}', r#"\mathfrak{J}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL J
    ('\u{1D50E}', r#"\mathfrak{K}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL K
    ('\u{1D50F}', r#"\mathfrak{L}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL L
    ('\u{1D510}', r#"\mathfrak{M}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL M
    ('\u{1D511}', r#"\mathfrak{N}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL N
    ('\u{1D512}', r#"\mathfrak{O}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL O
    ('\u{1D513}', r#"\mathfrak{P}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL P
    ('\u{1D514}', r#"\mathfrak{Q}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL Q
    ('\u{1D515}', r#"\mathfrak{R}"#, MATH, AMSSYMB), // (unnamed code point)
    ('\u{1D516}', r#"\mathfrak{S}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL S
    ('\u{1D517}', r#"\mathfrak{T}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL T
    ('\u{1D518}', r#"\mathfrak{U}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL U
    ('\u{1D519}', r#"\mathfrak{V}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL V
    ('\u{1D51A}', r#"\mathfrak{W}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL W
    ('\u{1D51B}', r#"\mathfrak{X}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL X
    ('\u{1D51C}', r#"\mathfrak{Y}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL Y
    ('\u{1D51D}', r#"\mathfrak{Z}"#, MATH, AMSSYMB), // (unnamed code point)
    ('\u{1D51E}', r#"\mathfrak{a}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL A
    ('\u{1D51F}', r#"\mathfrak{b}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL B
    ('\u{1D520}', r#"\mathfrak{c}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL C
    ('\u{1D521}', r#"\mathfrak{d}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL D
    ('\u{1D522}', r#"\mathfrak{e}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL E
    ('\u{1D523}', r#"\mathfrak{f}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL F
    ('\u{1D524}', r#"\mathfrak{g}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL G
    ('\u{1D525}', r#"\mathfrak{h}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL H
    ('\u{1D526}', r#"\mathfrak{i}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL I
    ('\u{1D527}', r#"\mathfrak{j}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL J
    ('\u{1D528}', r#"\mathfrak{k}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL K
    ('\u{1D529}', r#"\mathfrak{l}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL L
    ('\u{1D52A}', r#"\mathfrak{m}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL M
    ('\u{1D52B}', r#"\mathfrak{n}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL N
    ('\u{1D52C}', r#"\mathfrak{o}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL O
    ('\u{1D52D}', r#"\mathfrak{p}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL P
    ('\u{1D52E}', r#"\mathfrak{q}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL Q
    ('\u{1D52F}', r#"\mathfrak{r}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL R
    ('\u{1D530}', r#"\mathfrak{s}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL S
    ('\u{1D531}', r#"\mathfrak{t}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL T
    ('\u{1D532}', r#"\mathfrak{u}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL U
    ('\u{1D533}', r#"\mathfrak{v}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL V
    ('\u{1D534}', r#"\mathfrak{w}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL W
    ('\u{1D535}', r#"\mathfrak{x}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL X
    ('\u{1D536}', r#"\mathfrak{y}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL Y
    ('\u{1D537}', r#"\mathfrak{z}"#, MATH, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL Z
    ('\u{1D538}', r#"\mathbb{A}"#, MATH, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL A
    ('\u{1D539}', r#"\mathbb{B}"#, MATH, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL B
    ('\u{1D53A}', r#"\mathbb{C}"#, MATH, AMSSYMB), // (unnamed code point)
    ('\u{1D53B}', r#"\mathbb{D}"#, MATH, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL D
    ('\u{1D53C}', r#"\mathbb{E}"#, MATH, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL E
    ('\u{1D53D}', r#"\mathbb{F}"#, MATH, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL F
    ('\u{1D53E}', r#"\mathbb{G}"#, MATH, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL G
    ('\u{1D53F}', r#"\mathbb{H}"#, MATH, AMSSYMB), // (unnamed code point)
    ('\u{1D540}', r#"\mathbb{I}"#, MATH, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL I
    ('\u{1D541}', r#"\mathbb{J}"#, MATH, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL J
    ('\u{1D542}', r#"\mathbb{K}"#, MATH, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL K
    ('\u{1D543}', r#"\mathbb{L}"#, MATH, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL L
    ('\u{1D544}', r#"\mathbb{M}"#, MATH, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL M
    ('\u{1D545}', r#"\mathbb{N}"#, MATH, AMSSYMB), // (unnamed code point)
    ('\u{1D546}', r#"\mathbb{O}"#, MATH, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL O
    ('\u{1D547}', r#"\mathbb{P}"#, MATH, AMSSYMB), // (unnamed code point)
    ('\u{1D548}', r#"\mathbb{Q}"#, MATH, AMSSYMB), // (unnamed code point)
    ('\u{1D549}', r#"\mathbb{R}"#, MATH, AMSSYMB), // (unnamed code point)
    ('\u{1D54A}', r#"\mathbb{S}"#, MATH, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL S
    ('\u{1D54B}', r#"\mathbb{T}"#, MATH, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL T
    ('\u{1D54C}', r#"\mathbb{U}"#, MATH, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL U
    ('\u{1D54D}', r#"\mathbb{V}"#, MATH, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL V
    ('\u{1D54E}', r#"\mathbb{W}"#, MATH, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL W
    ('\u{1D54F}', r#"\mathbb{X}"#, MATH, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL X
    ('\u{1D550}', r#"\mathbb{Y}"#, MATH, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL Y
    ('\u{1D551}', r#"\mathbb{Z}"#, MATH, AMSSYMB), // (unnamed code point)
    ('\u{1D552}', r#"\mathbbm{a}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL A
    ('\u{1D553}', r#"\mathbbm{b}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL B
    ('\u{1D554}', r#"\mathbbm{c}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL C
    ('\u{1D555}', r#"\mathbbm{d}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL D
    ('\u{1D556}', r#"\mathbbm{e}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL E
    ('\u{1D557}', r#"\mathbbm{f}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL F
    ('\u{1D558}', r#"\mathbbm{g}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL G
    ('\u{1D559}', r#"\mathbbm{h}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL H
    ('\u{1D55A}', r#"\mathbbm{i}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL I
    ('\u{1D55B}', r#"\mathbbm{j}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL J
    ('\u{1D55C}', r#"\mathbbm{k}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL K
    ('\u{1D55D}', r#"\mathbbm{l}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL L
    ('\u{1D55E}', r#"\mathbbm{m}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL M
    ('\u{1D55F}', r#"\mathbbm{n}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL N
    ('\u{1D560}', r#"\mathbbm{o}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL O
    ('\u{1D561}', r#"\mathbbm{p}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL P
    ('\u{1D562}', r#"\mathbbm{q}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL Q
    ('\u{1D563}', r#"\mathbbm{r}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL R
    ('\u{1D564}', r#"\mathbbm{s}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL S
    ('\u{1D565}', r#"\mathbbm{t}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL T
    ('\u{1D566}', r#"\mathbbm{u}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL U
    ('\u{1D567}', r#"\mathbbm{v}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL V
    ('\u{1D568}', r#"\mathbbm{w}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL W
    ('\u{1D569}', r#"\mathbbm{x}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL X
    ('\u{1D56A}', r#"\mathbbm{y}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL Y
    ('\u{1D56B}', r#"\mathbbm{z}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL Z
    ('\u{1D5A0}', r#"\mathsf{A}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL A
    ('\u{1D5A1}', r#"\mathsf{B}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL B
    ('\u{1D5A2}', r#"\mathsf{C}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL C
    ('\u{1D5A3}', r#"\mathsf{D}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL D
    ('\u{1D5A4}', r#"\mathsf{E}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL E
    ('\u{1D5A5}', r#"\mathsf{F}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL F
    ('\u{1D5A6}', r#"\mathsf{G}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL G
    ('\u{1D5A7}', r#"\mathsf{H}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL H
    ('\u{1D5A8}', r#"\mathsf{I}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL I
    ('\u{1D5A9}', r#"\mathsf{J}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL J
    ('\u{1D5AA}', r#"\mathsf{K}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL K
    ('\u{1D5AB}', r#"\mathsf{L}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL L
    ('\u{1D5AC}', r#"\mathsf{M}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL M
    ('\u{1D5AD}', r#"\mathsf{N}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL N
    ('\u{1D5AE}', r#"\mathsf{O}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL O
    ('\u{1D5AF}', r#"\mathsf{P}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL P
    ('\u{1D5B0}', r#"\mathsf{Q}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL Q
    ('\u{1D5B1}', r#"\mathsf{R}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL R
    ('\u{1D5B2}', r#"\mathsf{S}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL S
    ('\u{1D5B3}', r#"\mathsf{T}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL T
    ('\u{1D5B4}', r#"\mathsf{U}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL U
    ('\u{1D5B5}', r#"\mathsf{V}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL V
    ('\u{1D5B6}', r#"\mathsf{W}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL W
    ('\u{1D5B7}', r#"\mathsf{X}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL X
    ('\u{1D5B8}', r#"\mathsf{Y}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL Y
    ('\u{1D5B9}', r#"\mathsf{Z}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL Z
    ('\u{1D5BA}', r#"\mathsf{a}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL A
    ('\u{1D5BB}', r#"\mathsf{b}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL B
    ('\u{1D5BC}', r#"\mathsf{c}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL C
    ('\u{1D5BD}', r#"\mathsf{d}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL D
    ('\u{1D5BE}', r#"\mathsf{e}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL E
    ('\u{1D5BF}', r#"\mathsf{f}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL F
    ('\u{1D5C0}', r#"\mathsf{g}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL G
    ('\u{1D5C1}', r#"\mathsf{h}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL H
    ('\u{1D5C2}', r#"\mathsf{i}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL I
    ('\u{1D5C3}', r#"\mathsf{j}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL J
    ('\u{1D5C4}', r#"\mathsf{k}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL K
    ('\u{1D5C5}', r#"\mathsf{l}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL L
    ('\u{1D5C6}', r#"\mathsf{m}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL M
    ('\u{1D5C7}', r#"\mathsf{n}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL N
    ('\u{1D5C8}', r#"\mathsf{o}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL O
    ('\u{1D5C9}', r#"\mathsf{p}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL P
    ('\u{1D5CA}', r#"\mathsf{q}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL Q
    ('\u{1D5CB}', r#"\mathsf{r}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL R
    ('\u{1D5CC}', r#"\mathsf{s}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL S
    ('\u{1D5CD}', r#"\mathsf{t}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL T
    ('\u{1D5CE}', r#"\mathsf{u}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL U
    ('\u{1D5CF}', r#"\mathsf{v}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL V
    ('\u{1D5D0}', r#"\mathsf{w}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL W
    ('\u{1D5D1}', r#"\mathsf{x}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL X
    ('\u{1D5D2}', r#"\mathsf{y}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL Y
    ('\u{1D5D3}', r#"\mathsf{z}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL Z
    ('\u{1D670}', r#"\mathtt{A}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL A
    ('\u{1D671}', r#"\mathtt{B}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL B
    ('\u{1D672}', r#"\mathtt{C}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL C
    ('\u{1D673}', r#"\mathtt{D}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL D
    ('\u{1D674}', r#"\mathtt{E}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL E
    ('\u{1D675}', r#"\mathtt{F}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL F
    ('\u{1D676}', r#"\mathtt{G}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL G
    ('\u{1D677}', r#"\mathtt{H}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL H
    ('\u{1D678}', r#"\mathtt{I}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL I
    ('\u{1D679}', r#"\mathtt{J}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL J
    ('\u{1D67A}', r#"\mathtt{K}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL K
    ('\u{1D67B}', r#"\mathtt{L}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL L
    ('\u{1D67C}', r#"\mathtt{M}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL M
    ('\u{1D67D}', r#"\mathtt{N}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL N
    ('\u{1D67E}', r#"\mathtt{O}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL O
    ('\u{1D67F}', r#"\mathtt{P}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL P
    ('\u{1D680}', r#"\mathtt{Q}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL Q
    ('\u{1D681}', r#"\mathtt{R}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL R
    ('\u{1D682}', r#"\mathtt{S}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL S
    ('\u{1D683}', r#"\mathtt{T}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL T
    ('\u{1D684}', r#"\mathtt{U}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL U
    ('\u{1D685}', r#"\mathtt{V}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL V
    ('\u{1D686}', r#"\mathtt{W}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL W
    ('\u{1D687}', r#"\mathtt{X}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL X
    ('\u{1D688}', r#"\mathtt{Y}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL Y
    ('\u{1D689}', r#"\mathtt{Z}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL Z
    ('\u{1D68A}', r#"\mathtt{a}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE SMALL A
    ('\u{1D68B}', r#"\mathtt{b}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE SMALL B
    ('\u{1D68C}', r#"\mathtt{c}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE SMALL C
    ('\u{1D68D}', r#"\mathtt{d}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE SMALL D
    ('\u{1D68E}', r#"\mathtt{e}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE SMALL E
    ('\u{1D68F}', r#"\mathtt{f}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE SMALL F
    ('\u{1D690}', r#"\mathtt{g}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE SMALL G
    ('\u{1D691}', r#"\mathtt{h}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE SMALL H
    ('\u{1D692}', r#"\mathtt{i}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE SMALL I
    ('\u{1D693}', r#"\mathtt{j}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE SMALL J
    ('\u{1D694}', r#"\mathtt{k}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE SMALL K
    ('\u{1D695}', r#"\mathtt{l}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE SMALL L
    ('\u{1D696}', r#"\mathtt{m}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE SMALL M
    ('\u{1D697}', r#"\mathtt{n}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE SMALL N
    ('\u{1D698}', r#"\mathtt{o}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE SMALL O
    ('\u{1D699}', r#"\mathtt{p}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE SMALL P
    ('\u{1D69A}', r#"\mathtt{q}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE SMALL Q
    ('\u{1D69B}', r#"\mathtt{r}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE SMALL R
    ('\u{1D69C}', r#"\mathtt{s}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE SMALL S
    ('\u{1D69D}', r#"\mathtt{t}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE SMALL T
    ('\u{1D69E}', r#"\mathtt{u}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE SMALL U
    ('\u{1D69F}', r#"\mathtt{v}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE SMALL V
    ('\u{1D6A0}', r#"\mathtt{w}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE SMALL W
    ('\u{1D6A1}', r#"\mathtt{x}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE SMALL X
    ('\u{1D6A2}', r#"\mathtt{y}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE SMALL Y
    ('\u{1D6A3}', r#"\mathtt{z}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE SMALL Z
    ('\u{1D7CE}', r#"\mathbf{0}"#, MATH, BUILTINS), // MATHEMATICAL BOLD DIGIT ZERO
    ('\u{1D7CF}', r#"\mathbf{1}"#, MATH, BUILTINS), // MATHEMATICAL BOLD DIGIT ONE
    ('\u{1D7D0}', r#"\mathbf{2}"#, MATH, BUILTINS), // MATHEMATICAL BOLD DIGIT TWO
    ('\u{1D7D1}', r#"\mathbf{3}"#, MATH, BUILTINS), // MATHEMATICAL BOLD DIGIT THREE
    ('\u{1D7D2}', r#"\mathbf{4}"#, MATH, BUILTINS), // MATHEMATICAL BOLD DIGIT FOUR
    ('\u{1D7D3}', r#"\mathbf{5}"#, MATH, BUILTINS), // MATHEMATICAL BOLD DIGIT FIVE
    ('\u{1D7D4}', r#"\mathbf{6}"#, MATH, BUILTINS), // MATHEMATICAL BOLD DIGIT SIX
    ('\u{1D7D5}', r#"\mathbf{7}"#, MATH, BUILTINS), // MATHEMATICAL BOLD DIGIT SEVEN
    ('\u{1D7D6}', r#"\mathbf{8}"#, MATH, BUILTINS), // MATHEMATICAL BOLD DIGIT EIGHT
    ('\u{1D7D7}', r#"\mathbf{9}"#, MATH, BUILTINS), // MATHEMATICAL BOLD DIGIT NINE
    ('\u{1D7D8}', r#"\UnxTBbold{0}"#, MATH, BBOLD), // MATHEMATICAL DOUBLE-STRUCK DIGIT ZERO
    ('\u{1D7D9}', r#"\mathds{1}"#, MATH, DSFONT), // MATHEMATICAL DOUBLE-STRUCK DIGIT ONE
    ('\u{1D7DA}', r#"\mathbbm{2}"#, MATH, BBM), // MATHEMATICAL DOUBLE-STRUCK DIGIT TWO
    ('\u{1D7DB}', r#"\UnxTBbold{3}"#, MATH, BBOLD), // MATHEMATICAL DOUBLE-STRUCK DIGIT THREE
    ('\u{1D7DC}', r#"\UnxTBbold{4}"#, MATH, BBOLD), // MATHEMATICAL DOUBLE-STRUCK DIGIT FOUR
    ('\u{1D7DD}', r#"\UnxTBbold{5}"#, MATH, BBOLD), // MATHEMATICAL DOUBLE-STRUCK DIGIT FIVE
    ('\u{1D7DE}', r#"\UnxTBbold{6}"#, MATH, BBOLD), // MATHEMATICAL DOUBLE-STRUCK DIGIT SIX
    ('\u{1D7DF}', r#"\UnxTBbold{7}"#, MATH, BBOLD), // MATHEMATICAL DOUBLE-STRUCK DIGIT SEVEN
    ('\u{1D7E0}', r#"\UnxTBbold{8}"#, MATH, BBOLD), // MATHEMATICAL DOUBLE-STRUCK DIGIT EIGHT
    ('\u{1D7E1}', r#"\UnxTBbold{9}"#, MATH, BBOLD), // MATHEMATICAL DOUBLE-STRUCK DIGIT NINE
    ('\u{1D7E2}', r#"\mathsf{0}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF DIGIT ZERO
    ('\u{1D7E3}', r#"\mathsf{1}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF DIGIT ONE
    ('\u{1D7E4}', r#"\mathsf{2}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF DIGIT TWO
    ('\u{1D7E5}', r#"\mathsf{3}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF DIGIT THREE
    ('\u{1D7E6}', r#"\mathsf{4}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF DIGIT FOUR
    ('\u{1D7E7}', r#"\mathsf{5}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF DIGIT FIVE
    ('\u{1D7E8}', r#"\mathsf{6}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF DIGIT SIX
    ('\u{1D7E9}', r#"\mathsf{7}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF DIGIT SEVEN
    ('\u{1D7EA}', r#"\mathsf{8}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF DIGIT EIGHT
    ('\u{1D7EB}', r#"\mathsf{9}"#, MATH, BUILTINS), // MATHEMATICAL SANS-SERIF DIGIT NINE
    ('\u{1D7F6}', r#"\mathtt{0}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE DIGIT ZERO
    ('\u{1D7F7}', r#"\mathtt{1}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE DIGIT ONE
    ('\u{1D7F8}', r#"\mathtt{2}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE DIGIT TWO
    ('\u{1D7F9}', r#"\mathtt{3}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE DIGIT THREE
    ('\u{1D7FA}', r#"\mathtt{4}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE DIGIT FOUR
    ('\u{1D7FB}', r#"\mathtt{5}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE DIGIT FIVE
    ('\u{1D7FC}', r#"\mathtt{6}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE DIGIT SIX
    ('\u{1D7FD}', r#"\mathtt{7}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE DIGIT SEVEN
    ('\u{1D7FE}', r#"\mathtt{8}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE DIGIT EIGHT
    ('\u{1D7FF}', r#"\mathtt{9}"#, MATH, BUILTINS), // MATHEMATICAL MONOSPACE DIGIT NINE
    // END ENTRIES
];
