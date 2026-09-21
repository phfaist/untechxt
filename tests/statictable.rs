//! The static table layouts (step 2): one small table compiled in each of the
//! three layouts, checked the same way in all three.

use untechxt::lookuptable::LookupTable;
use untechxt::preamble::{Chunk, Profile};
use untechxt::protection::{
    ReplacementProtectionHint as Hint, StandardProtection, ValueMode, ValueTermination,
};
use untechxt::report::EncodeReport;
use untechxt::rule::{AsciiSet, Rule};
use untechxt::statictable::{
    compile_static_table, ProfileIndex, StaticTableBinarySearch, StaticTableTwoLevelDirect,
    StaticTableTwoLevelLinear,
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
layout_tests!(two_level_direct, StaticTableTwoLevelDirect, two_level_direct_index);

// -------------------------------------------------- across the three layouts

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

/// The three layouts are three ways of holding one table: every lookup and
/// every iteration must agree.
#[test]
fn the_three_layouts_answer_alike() {
    static BINARY: StaticTableBinarySearch =
        compile_static_table!(ENTRIES, &PROFILES, binary_search);
    static LINEAR: StaticTableTwoLevelLinear =
        compile_static_table!(ENTRIES, &PROFILES, two_level_linear);
    static DIRECT: StaticTableTwoLevelDirect =
        compile_static_table!(ENTRIES, &PROFILES, two_level_direct_index);

    let binary: Vec<_> = BINARY.iter().collect();
    let linear: Vec<_> = LINEAR.iter().collect();
    let direct: Vec<_> = DIRECT.iter().collect();
    assert_eq!(binary, linear);
    assert_eq!(binary, direct);

    for ch in ENTRIES.iter().map(|&(ch, ..)| ch).chain(MISSES.iter().copied()) {
        assert_eq!(BINARY.lookup(ch), LINEAR.lookup(ch));
        assert_eq!(BINARY.lookup(ch), DIRECT.lookup(ch));
    }
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
