//! The static table layouts (step 2): one small table compiled in each of the
//! layouts, checked the same way in all of them.

use untechxt::lookuptable::LookupTable;
use untechxt::preamble::{Chunk, Profile};
use untechxt::protection::{
    ReplacementProtectionHint as Hint, StandardProtection, ValueMode, ValueTermination,
};
use untechxt::report::EncodeReport;
use untechxt::rule::{AsciiSet, Rule};
use untechxt::statictable::{
    compile_static_table, ProfileIndex, StaticTableBinarySearch, StaticTableTwoLevelBitmap,
    StaticTableTwoLevelDirect, StaticTableTwoLevelLinear,
};
use untechxt::Encoder;

// ---------------------------------------------------------------- fixtures

static AMSSYMB_CHUNKS: [Chunk; 1] = [Chunk::package("amssymb")];
static MATHRSFS_CHUNKS: [Chunk; 1] = [Chunk::package("mathrsfs")];
static DSFONT_CHUNKS: [Chunk; 1] = [Chunk::package("dsfont")];

/// The profile array the entries index into. Position 0 is the empty profile,
/// so that an index is a position of this array.
static PROFILES: [Profile; 4] = [
    Profile::from_static(&[]),
    Profile::from_static(&AMSSYMB_CHUNKS),
    Profile::from_static(&MATHRSFS_CHUNKS),
    Profile::from_static(&DSFONT_CHUNKS),
];

const NONE: ProfileIndex = ProfileIndex::NONE;
const AMSSYMB: ProfileIndex = ProfileIndex(1);
const MATHRSFS: ProfileIndex = ProfileIndex(2);
const DSFONT: ProfileIndex = ProfileIndex(3);

const TEXT: ValueMode = ValueMode::TextOnly;
const MATH: ValueMode = ValueMode::MathOnly;
const ANY: ValueMode = ValueMode::AnyMode;

/// Ten entries over six distinct `code point >> 8` blocks (`0x00`, `0x03`,
/// `0x20`, `0x21`, `0xFB`, `0x1D7`): two ASCII characters, values of all
/// three modes, values that end with a named macro and values that terminate
/// themselves, and both profile index 0 and three non-zero ones.
const ENTRIES: &[(char, &str, ValueMode, ProfileIndex)] = &[
    ('\u{0023}', r"\#", TEXT, NONE),                  // NUMBER SIGN
    ('\u{0026}', r"\&", TEXT, NONE),                  // AMPERSAND
    ('\u{00E9}', r"\'e", TEXT, NONE),                 // e WITH ACUTE
    ('\u{03B1}', r"\alpha", MATH, NONE),              // GREEK SMALL LETTER ALPHA
    ('\u{03B2}', r"\beta", MATH, NONE),               // GREEK SMALL LETTER BETA
    ('\u{2014}', r"\textemdash", TEXT, NONE),         // EM DASH
    ('\u{2102}', r"\mathbb{C}", MATH, AMSSYMB),       // DOUBLE-STRUCK CAPITAL C
    ('\u{2133}', r"\mathscr{M}", MATH, MATHRSFS),     // SCRIPT CAPITAL M
    ('\u{FB01}', "fi", ANY, NONE),                    // LATIN SMALL LIGATURE FI
    ('\u{1D7D9}', r"\mathds{1}", MATH, DSFONT),       // DOUBLE-STRUCK DIGIT ONE
];

/// Characters no entry covers: two in blocks the table has (`0x00`, `0x03`,
/// `0x21`) and two in blocks it has not (`0x04`, `0x4E`).
const MISSES: &[char] = &['x', '\u{00E8}', '\u{03B3}', '\u{2135}', '\u{0416}', '\u{4E00}'];

/// The hint an entry of `ENTRIES` must answer with.
fn expected_hint(encoded: &str, mode: ValueMode) -> Hint {
    Hint::Value { mode, termination: ValueTermination::inspect(encoded) }
}

/// The text a table encodes in text output, and what it must come to.
const TEXT_INPUT: &str = "a#\u{00E9}\u{03B1}\u{2014}\u{2102}\u{FB01}";
const TEXT_OUTPUT: &str = "a\\#\\'e\\ensuremath{\\alpha}{\\textemdash}\\ensuremath{\\mathbb{C}}fi";

/// The same in math output: a math value keeps its bare form and is closed by
/// a space, a text value is wrapped, a value of either mode is written as it
/// is.
const MATH_INPUT: &str = "\u{03B1}\u{00E9}#\u{FB01}";
const MATH_OUTPUT: &str = "\\alpha \\textnormal{\\'e}\\textnormal{\\#}fi";

/// The same battery of tests for one layout: the table `TABLE` is the same
/// data compiled by `$layout`, and everything below must hold whichever
/// layout that is.
macro_rules! layout_tests {
    ($name:ident, $struct:ident, $layout:ident) => {
        mod $name {
            use super::*;

            static TABLE: $struct = compile_static_table!(ENTRIES, &PROFILES, $layout);

            #[test]
            fn the_table_holds_the_entries() {
                assert_eq!(TABLE.len(), ENTRIES.len());
                assert!(!TABLE.is_empty());
            }

            #[test]
            fn every_entry_is_looked_up_with_its_value_its_hint_and_its_needs() {
                for &(ch, encoded, mode, profile) in ENTRIES {
                    let entry = TABLE.lookup(ch).expect("the entry is in the table");
                    assert_eq!(entry.encoded, encoded);
                    assert_eq!(entry.hint, expected_hint(encoded, mode));
                    match profile {
                        ProfileIndex(0) => assert!(entry.needs.is_none()),
                        ProfileIndex(index) => {
                            let needs = entry.needs.expect("the entry names a profile");
                            assert!(std::ptr::eq(needs, &PROFILES[index as usize]));
                        }
                    }
                }
            }

            #[test]
            fn a_character_outside_the_table_is_a_miss() {
                for &ch in MISSES {
                    assert_eq!(TABLE.lookup(ch), None, "{ch:?} must not be in the table");
                }
            }

            #[test]
            fn iter_yields_the_entries_in_key_order() {
                let held: Vec<char> = TABLE.iter().map(|(ch, _)| ch).collect();
                let expected: Vec<char> = ENTRIES.iter().map(|&(ch, ..)| ch).collect();
                assert_eq!(held, expected);
                for ((_, entry), &(_, encoded, mode, _)) in TABLE.iter().zip(ENTRIES) {
                    assert_eq!(entry.encoded, encoded);
                    assert_eq!(entry.hint, expected_hint(encoded, mode));
                }
            }

            #[test]
            fn lookup_agrees_with_iter() {
                for (ch, entry) in TABLE.iter() {
                    assert_eq!(TABLE.lookup(ch), Some(entry));
                }
            }

            #[test]
            fn the_ascii_keys_are_the_ascii_entries() {
                assert_eq!(TABLE.ascii_keys(), AsciiSet::of("#&"));
                assert_eq!(TABLE.ascii_triggers(), TABLE.ascii_keys());
            }

            #[test]
            fn the_table_is_a_rule_in_text_output() {
                let encoder = Encoder::new(&TABLE);
                assert_eq!(encoder.encode(TEXT_INPUT).unwrap(), TEXT_OUTPUT);
            }

            #[test]
            fn the_table_is_a_rule_in_math_output() {
                let encoder =
                    Encoder::new(&TABLE).with_protection(StandardProtection::math_mode());
                assert_eq!(encoder.encode(MATH_INPUT).unwrap(), MATH_OUTPUT);
            }

            #[test]
            fn what_an_entry_needs_reaches_the_report() {
                let encoder = Encoder::new(&TABLE);
                let (_, report) = encoder.encode_with_report("\u{2102}\u{2133}\u{03B1}").unwrap();
                let ids: Vec<&str> =
                    report.needs.chunks().map(|chunk| &*chunk.id).collect();
                assert_eq!(ids, ["amssymb", "mathrsfs"]);
            }

            #[test]
            fn the_debug_form_names_the_layout_and_the_size() {
                let debug = format!("{:?}", TABLE);
                assert!(debug.starts_with(stringify!($struct)), "{debug}");
                assert!(debug.contains("len: 10"), "{debug}");
            }
        }
    };
}

layout_tests!(binary_search, StaticTableBinarySearch, binary_search);
layout_tests!(two_level_linear, StaticTableTwoLevelLinear, two_level_linear);
layout_tests!(two_level_bitmap, StaticTableTwoLevelBitmap, two_level_bitmap);
layout_tests!(two_level_direct, StaticTableTwoLevelDirect, two_level_direct_index);

// -------------------------------------------------------- across the layouts

/// An empty table is legal, answers nothing, and triggers on no ASCII
/// character at all.
#[test]
fn an_empty_table_answers_nothing() {
    const NO_ENTRIES: &[(char, &str, ValueMode, ProfileIndex)] = &[];
    static EMPTY: StaticTableTwoLevelDirect =
        compile_static_table!(NO_ENTRIES, &PROFILES, two_level_direct_index);

    assert_eq!(EMPTY.len(), 0);
    assert!(EMPTY.is_empty());
    assert_eq!(EMPTY.lookup('a'), None);
    assert_eq!(EMPTY.ascii_keys(), AsciiSet::EMPTY);
    assert_eq!(EMPTY.iter().count(), 0);
}

/// The layouts are different ways of holding one table: every lookup and
/// every iteration must agree.
#[test]
fn the_layouts_answer_alike() {
    static BINARY: StaticTableBinarySearch =
        compile_static_table!(ENTRIES, &PROFILES, binary_search);
    static LINEAR: StaticTableTwoLevelLinear =
        compile_static_table!(ENTRIES, &PROFILES, two_level_linear);
    static BITMAP: StaticTableTwoLevelBitmap =
        compile_static_table!(ENTRIES, &PROFILES, two_level_bitmap);
    static DIRECT: StaticTableTwoLevelDirect =
        compile_static_table!(ENTRIES, &PROFILES, two_level_direct_index);

    let binary: Vec<_> = BINARY.iter().collect();
    let linear: Vec<_> = LINEAR.iter().collect();
    let bitmap: Vec<_> = BITMAP.iter().collect();
    let direct: Vec<_> = DIRECT.iter().collect();
    assert_eq!(binary, linear);
    assert_eq!(binary, bitmap);
    assert_eq!(binary, direct);

    for ch in ENTRIES.iter().map(|&(ch, ..)| ch).chain(MISSES.iter().copied()) {
        assert_eq!(BINARY.lookup(ch), LINEAR.lookup(ch));
        assert_eq!(BINARY.lookup(ch), BITMAP.lookup(ch));
        assert_eq!(BINARY.lookup(ch), DIRECT.lookup(ch));
    }
}

/// The number of entries of [`DENSE`]: a full block and eight more.
const DENSE_LEN: usize = 256 + 8;

/// A table with a full block followed by a sparse one, which `ENTRIES` has
/// neither of. Block `0x04` has an entry at every low byte. Block `0x05` has
/// one at the first and at the last bit of each of the four words of a
/// 256-bit bitmap, so that a position counted wrongly across a word boundary
/// names the wrong entry.
///
/// The entries of the full block are generated, so they all have the same
/// value; their mode and their profile cycle with the periods 3 and 4, which
/// tells an entry from every neighbor closer than 12 positions. The entries
/// of the second block have a value each.
const DENSE: [(char, &str, ValueMode, ProfileIndex); DENSE_LEN] = {
    let mut out = [('\0', "x", TEXT, NONE); DENSE_LEN];
    let mut i = 0;
    while i < 256 {
        out[i].0 = match char::from_u32(0x0400 + i as u32) {
            Some(ch) => ch,
            None => panic!("every code point of the block is a character"),
        };
        out[i].2 = match i % 3 {
            0 => TEXT,
            1 => MATH,
            _ => ANY,
        };
        out[i].3 = ProfileIndex((i % 4) as u8);
        i += 1;
    }
    out[256] = ('\u{0500}', "w0-first", TEXT, NONE);
    out[257] = ('\u{053F}', "w0-last", TEXT, NONE);
    out[258] = ('\u{0540}', "w1-first", TEXT, NONE);
    out[259] = ('\u{057F}', "w1-last", TEXT, NONE);
    out[260] = ('\u{0580}', "w2-first", TEXT, NONE);
    out[261] = ('\u{05BF}', "w2-last", TEXT, NONE);
    out[262] = ('\u{05C0}', "w3-first", TEXT, NONE);
    out[263] = ('\u{05FF}', "w3-last", TEXT, NONE);
    out
};

/// Full blocks and entries on either side of a bitmap word boundary: every
/// layout finds each entry of [`DENSE`] under its own character, and nothing
/// at the characters between the entries of the sparse block.
#[test]
fn a_full_block_and_the_word_boundaries_of_a_bitmap() {
    static BINARY: StaticTableBinarySearch =
        compile_static_table!(&DENSE, &PROFILES, binary_search);
    static LINEAR: StaticTableTwoLevelLinear =
        compile_static_table!(&DENSE, &PROFILES, two_level_linear);
    static BITMAP: StaticTableTwoLevelBitmap =
        compile_static_table!(&DENSE, &PROFILES, two_level_bitmap);
    static DIRECT: StaticTableTwoLevelDirect =
        compile_static_table!(&DENSE, &PROFILES, two_level_direct_index);

    let tables: [(&str, &dyn LookupTable); 4] = [
        ("binary_search", &BINARY),
        ("two_level_linear", &LINEAR),
        ("two_level_bitmap", &BITMAP),
        ("two_level_direct_index", &DIRECT),
    ];
    for (name, table) in tables {
        for &(ch, encoded, mode, profile) in &DENSE {
            let entry = table.lookup(ch).unwrap_or_else(|| panic!("{name}: {ch:?} is missing"));
            assert_eq!(entry.encoded, encoded, "{name}: {ch:?}");
            assert_eq!(entry.hint, expected_hint(encoded, mode), "{name}: {ch:?}");
            match profile {
                ProfileIndex(0) => assert!(entry.needs.is_none(), "{name}: {ch:?}"),
                ProfileIndex(index) => {
                    let needs = entry.needs.expect("the entry names a profile");
                    assert!(std::ptr::eq(needs, &PROFILES[index as usize]), "{name}: {ch:?}");
                }
            }
        }
        for miss in ['\u{0501}', '\u{053E}', '\u{0541}', '\u{05BE}', '\u{05FE}', '\u{0600}'] {
            assert_eq!(table.lookup(miss), None, "{name}: {miss:?} must not be in the table");
        }
    }

    let expected: Vec<char> = DENSE.iter().map(|&(ch, ..)| ch).collect();
    assert_eq!(BINARY.iter().map(|(ch, _)| ch).collect::<Vec<_>>(), expected);
    assert_eq!(LINEAR.iter().map(|(ch, _)| ch).collect::<Vec<_>>(), expected);
    assert_eq!(BITMAP.iter().map(|(ch, _)| ch).collect::<Vec<_>>(), expected);
    assert_eq!(DIRECT.iter().map(|(ch, _)| ch).collect::<Vec<_>>(), expected);
    assert!(BITMAP.iter().eq(BINARY.iter()));
}

/// A report gathered from a static table is the same whichever layout held
/// the data.
#[test]
fn the_report_is_the_same_whichever_layout() {
    static BINARY: StaticTableBinarySearch =
        compile_static_table!(ENTRIES, &PROFILES, binary_search);
    static DIRECT: StaticTableTwoLevelDirect =
        compile_static_table!(ENTRIES, &PROFILES, two_level_direct_index);

    let text = "\u{1D7D9} \u{2102} \u{2133}";
    let mut one = EncodeReport::new();
    let mut two = EncodeReport::new();
    let mut first = String::new();
    let mut second = String::new();
    Encoder::new(&BINARY).encode_into(text, &mut first, &mut one).unwrap();
    Encoder::new(&DIRECT).encode_into(text, &mut second, &mut two).unwrap();

    assert_eq!(first, second);
    assert_eq!(format!("{one:?}"), format!("{two:?}"));
    assert_eq!(format!("{:?}", one.needs), r#"["dsfont", "amssymb", "mathrsfs"]"#);
}
