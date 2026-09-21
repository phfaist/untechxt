//! Tests of `flm_latexencode`, the port of pylatexenc's `latexencode`.
//!
//! Three groups:
//!
//! - **The ported tests**: pylatexenc's own encoder tests
//!   (`test/test_latexencode.py` at commit `e4ddf2ba`), each rewritten from
//!   scratch against this API with the same input and the same expected
//!   output, and a one-line note naming its origin. The tests that exercise
//!   the regular-expression rule kind use a callable rule to the same effect;
//!   those of the `unicode-xml` table, of the partial encoder and of the
//!   pylatexenc-1 function are not ported, since none of those exist here.
//! - **The conformance golden**: `tests/goldens/latexencode/uni_chars_test_previous.txt`
//!   holds one line per code point — a code point, the character's Unicode name
//!   and the character, all encoded under the `braces-almost-all` protection
//!   and the `fail` policy. It began as pylatexenc's file of the same name and
//!   now records this table's output (its `README.md` states the provenance and
//!   how it is refreshed). The test rebuilds every line from the golden itself,
//!   encodes it, and compares; `UPDATE_GOLDEN=1` rewrites the file. It is the
//!   acceptance test of the whole table and the composition step.
//! - **Properties of the table**: well-formed spellings, the profiles and the
//!   chunks they name, coverage by the golden, the entries above the Basic
//!   Multilingual Plane addressed by code point.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

use flm_latexencode::{
    ChunkPreamble, EncodeError, Encoder, Mode, Options, PreambleNeeds, Profile, Protection, Rule,
    UnknownChar, UnknownCharPolicy, encode, macro_names, spelling_of, tables,
};

/// The sentence most of pylatexenc's tests encode.
const SANTE: &str = "\"À votre santé!\" s'exclama le maître de maison à 100%.";

fn encoder(options: Options) -> Encoder {
    Encoder::new(options)
}

// ---------------------------------------------------------------------------
// The ported tests
// ---------------------------------------------------------------------------

/// pylatexenc `test_basic_0` / `test_basic_0b`: the default encoder.
#[test]
fn basic_0_default_encoder() {
    let expected = "''\\`A votre sant\\'e!'' s'exclama le ma\\^itre de maison \\`a 100\\%.";
    assert_eq!(Encoder::default().encode(SANTE).unwrap(), expected);
    assert_eq!(encode(SANTE), expected);
}

/// pylatexenc `test_basic_1`: `non_ascii_only` with `braces-all`.
#[test]
fn basic_1_non_ascii_only_braces_all() {
    let u = encoder(Options {
        non_ascii_only: true,
        protection: Protection::BracesAll,
        ..Options::default()
    });
    assert_eq!(
        u.encode(SANTE).unwrap(),
        "\"{\\`A} votre sant{\\'e}!\" s'exclama le ma{\\^i}tre de maison {\\`a} 100%."
    );
}

/// pylatexenc `test_basic_2`: `braces-after-macro` changes nothing here.
#[test]
fn basic_2_braces_after_macro() {
    let u = encoder(Options { protection: Protection::BracesAfterMacro, ..Options::default() });
    assert_eq!(
        u.encode(SANTE).unwrap(),
        "''\\`A votre sant\\'e!'' s'exclama le ma\\^itre de maison \\`a 100\\%."
    );
}

/// pylatexenc `test_basic_2a` (its issue 44): `braces-after-macro` appends
/// `{}` after `\l` and `\textasciitilde` but not after `\c{c}` or `\ensuremath{…}`.
#[test]
fn basic_2a_braces_after_macro_issue_44() {
    let u = encoder(Options { protection: Protection::BracesAfterMacro, ..Options::default() });
    assert_eq!(
        u.encode("Jabłoński, François, ⟨.⟩, ~").unwrap(),
        "Jab\\l{}o\\'nski, Fran\\c{c}ois, \\ensuremath{\\langle}.\\ensuremath{\\rangle}, \\textasciitilde{}"
    );
}

/// pylatexenc `test_basic_2b`: no protection.
#[test]
fn basic_2b_protection_none() {
    let u = encoder(Options { protection: Protection::None, ..Options::default() });
    assert_eq!(
        u.encode(SANTE).unwrap(),
        "''\\`A votre sant\\'e!'' s'exclama le ma\\^itre de maison \\`a 100\\%."
    );
}

/// pylatexenc `test_basic_2c`: with `non_ascii_only`, the ASCII specials pass.
#[test]
fn basic_2c_ascii_specials_untouched_with_non_ascii_only() {
    let u = encoder(Options { non_ascii_only: true, ..Options::default() });
    let ascii = " \" # $ % & \\ _ { } ~ ";
    assert_eq!(u.encode(ascii).unwrap(), ascii);
}

/// pylatexenc `test_basic_2d`: without it, every special is spelled.
#[test]
fn basic_2d_ascii_specials_spelled() {
    let u = encoder(Options { non_ascii_only: false, ..Options::default() });
    assert_eq!(
        u.encode(" \" # $ % & \\ _ { } ~ ").unwrap(),
        " '' \\# \\$ \\% \\& {\\textbackslash} \\_ \\{ \\} {\\textasciitilde} "
    );
}

/// pylatexenc `test_basic_callable_replacement_latex_protection`: a custom
/// protection function is applied to every spelling.
#[test]
fn basic_custom_protection_applies_to_every_spelling() {
    let u = encoder(Options {
        protection: Protection::Custom(Arc::new(|s| format!("{{***{{{s}}}***}}"))),
        ..Options::default()
    });
    assert_eq!(
        u.encode(SANTE).unwrap(),
        "{***{''}***}{***{\\`A}***} votre sant{***{\\'e}***}!{***{''}***} s'exclama le ma{***{\\^i}***}tre de maison {***{\\`a}***} 100{***{\\%}***}."
    );
}

/// pylatexenc `test_basic_3`: the `keep` policy keeps the character; the
/// warning pylatexenc logs is the returned list here.
#[test]
fn basic_3_unknown_kept_and_reported() {
    let text = "A unicode character: ธ";
    let u = encoder(Options { unknown_char_policy: UnknownCharPolicy::Keep, ..Options::default() });
    let mut out = String::new();
    let report = u.encode_into(&mut out, text).unwrap();
    assert_eq!(out, text);
    assert_eq!(report.unknown, vec![UnknownChar { ch: 'ธ', position: 21 }]);
}

/// pylatexenc `test_basic_3b`: the `replace` policy.
#[test]
fn basic_3b_unknown_replaced() {
    let u =
        encoder(Options { unknown_char_policy: UnknownCharPolicy::Replace, ..Options::default() });
    assert_eq!(u.encode("A unicode character: ธ").unwrap(), "A unicode character: {\\bfseries ?}");
}

/// pylatexenc `test_basic_3c`: the `unihex` policy.
#[test]
fn basic_3c_unknown_unihex() {
    let u =
        encoder(Options { unknown_char_policy: UnknownCharPolicy::Unihex, ..Options::default() });
    assert_eq!(
        u.encode("A unicode character: ธ").unwrap(),
        "A unicode character: \\ensuremath{\\langle}\\texttt{U+0E18}\\ensuremath{\\rangle}"
    );
}

/// The three regular-expression rules of pylatexenc's `test_rules_00`, as a
/// callable: `v(otre)` → `notre`, `s'exclama` (any case) → `s'exprima`,
/// `î` → `{\^i}`.
fn rules_00_regex_stand_in(s: &str, pos: usize) -> Option<(usize, String)> {
    let rest = &s[pos..];
    if rest.starts_with("votre") {
        return Some((5, "notre".to_string()));
    }
    let exclama = "s'exclama";
    if rest.len() >= exclama.len()
        && rest.is_char_boundary(exclama.len())
        && rest[..exclama.len()].eq_ignore_ascii_case(exclama)
    {
        return Some((exclama.len(), "s'exprima".to_string()));
    }
    if rest.starts_with('î') {
        return Some(('î'.len_utf8(), "{\\^i}".to_string()));
    }
    None
}

/// The trailing callable of pylatexenc's `test_rules_00`: `é` and `...`.
fn rules_00_callable(s: &str, pos: usize) -> Option<(usize, String)> {
    let rest = &s[pos..];
    if rest.starts_with('é') {
        return Some(('é'.len_utf8(), "{\\'{e}}".to_string()));
    }
    if rest.starts_with("...") {
        return Some((3, "\\ldots".to_string()));
    }
    None
}

/// pylatexenc `test_rules_00` / `_00b` / `_00c`: rule order. A dictionary and
/// a callable (for the regular expressions) before the defaults win over
/// them; a callable after them fires only where the table did not (`é` keeps
/// the table's `\'e`; `...` gets `{\ldots}`).
#[test]
fn rules_00_order_of_rules() {
    let u = encoder(Options {
        rules: vec![
            Rule::dict([('À', "{{\\`{A}}}".to_string()), ('%', "\\textpercent".to_string())]),
            Rule::callable(rules_00_regex_stand_in),
            Rule::defaults(),
            Rule::callable(rules_00_callable),
        ],
        ..Options::default()
    });
    let input = "\"À votre santé!\" s'exclama le maître de maison ... à 100%.";
    assert_eq!(
        u.encode(input).unwrap(),
        "''{{\\`{A}}} notre sant\\'e!'' s'exprima le ma{\\^i}tre de maison {\\ldots} \\`a 100{\\textpercent}."
    );
}

/// pylatexenc `test_rules_callable_must_consume_at_least_one_char`: a rule
/// that consumes nothing is an error, not an endless loop.
#[test]
fn rules_callable_must_consume_at_least_one_char() {
    let u = encoder(Options {
        rules: vec![Rule::callable(|_, _| Some((0, "\\ldots".to_string())))],
        ..Options::default()
    });
    assert_eq!(u.encode("santé"), Err(EncodeError::InvalidConsumption { position: 0, consumed: 0 }));
}

/// pylatexenc `test_rules_02` / `_02b`: the superscript two.
#[test]
fn rules_02_superscript_two() {
    let u = encoder(Options { rules: vec![Rule::defaults()], ..Options::default() });
    assert_eq!(
        u.encode("* \"À votre santé!\" s'exclama² le maître de maison à 100%.").unwrap(),
        "* ''\\`A votre sant\\'e!'' s'exclama{\\texttwosuperior} le ma\\^itre de maison \\`a 100\\%."
    );
}

/// pylatexenc `test_issue_no21`: bracing acronyms through a callable rule
/// that also preserves existing braces (the regular-expression form of the
/// same test is the same callable here).
#[test]
fn issue_no21_acronyms_through_a_callable() {
    fn capitalize_acronyms(s: &str, pos: usize) -> Option<(usize, String)> {
        let rest = &s[pos..];
        if rest.starts_with(['{', '}']) {
            return Some((1, rest[..1].to_string()));
        }
        // `\b[A-Z]{2,}\w*\b`: at a word start, two capitals then word characters.
        let at_word_start = pos == 0 || !s[..pos].ends_with(|c: char| c.is_alphanumeric() || c == '_');
        if !at_word_start {
            return None;
        }
        let capitals = rest.chars().take_while(|c| c.is_ascii_uppercase()).count();
        if capitals < 2 {
            return None;
        }
        let word: String =
            rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
        Some((word.len(), format!("{{{word}}}")))
    }
    let u = encoder(Options {
        rules: vec![Rule::callable(capitalize_acronyms), Rule::defaults()],
        ..Options::default()
    });
    assert_eq!(
        u.encode("Title with {Some} ABC acronyms LIKe this.").unwrap(),
        "Title with {Some} {ABC} acronyms {LIKe} this."
    );
    assert_eq!(
        u.encode("Title 2 with {Some} ABC acronyms LIKe this.").unwrap(),
        "Title 2 with {Some} {ABC} acronyms {LIKe} this."
    );
}

// pylatexenc `test_latex_string_class` is not ported: the accumulator hook it
// exercises has no counterpart here (the module documentation says so).

// ---------------------------------------------------------------------------
// Behavior pinned beyond pylatexenc's own tests
// ---------------------------------------------------------------------------

/// The probe string of the reference study, under the default options.
#[test]
fn the_study_s_probe_string() {
    assert_eq!(
        encode("Café — naïve “quotes” ½ ≤ α"),
        "Caf\\'e {\\textemdash} na\\\"ive {\\textquotedblleft}quotes{\\textquotedblright} {\\textonehalf} \\ensuremath{\\leq} \\ensuremath{\\alpha}"
    );
}

/// The double-struck one is spelled with `\mathds`, which `dsfont` defines —
/// one of the table's departures from pylatexenc, whose `\mathbb{1}` no
/// package defines.
#[test]
fn the_double_struck_one_is_spelled_with_mathds_and_needs_dsfont() {
    let s = spelling_of('𝟙').unwrap();
    assert_eq!(s.latex, "\\ensuremath{\\mathds{1}}");
    assert_eq!(s.mode, Mode::Math);
    let mut needs = PreambleNeeds::new();
    needs.include(s.needs);
    assert_eq!(needs.preamble(), "\\usepackage{dsfont}\n");
}

/// A base letter followed by a combining accent is composed before lookup.
#[test]
fn input_is_composed_before_lookup() {
    assert_eq!(encode("e\u{0301}"), "\\'e");
    assert_eq!(encode("Cafe\u{0301} A\u{030A}"), "Caf\\'e \\r{A}");
    // Positions are byte offsets into the composed text.
    let mut out = String::new();
    let report = Encoder::default().encode_into(&mut out, "e\u{0301}ธ").unwrap();
    assert_eq!(out, "\\'eธ");
    assert_eq!(report.unknown, vec![UnknownChar { ch: 'ธ', position: 2 }]);
}

/// The `fail` policy stops at the first unknown character, with the output up
/// to it left in the buffer.
#[test]
fn fail_policy_reports_the_first_unknown_character() {
    let u = encoder(Options { unknown_char_policy: UnknownCharPolicy::Fail, ..Options::default() });
    let mut out = String::new();
    let err = u.encode_into(&mut out, "ab ธ ธ").unwrap_err();
    assert_eq!(err, EncodeError::UnknownCharacter { ch: 'ธ', position: 3 });
    assert_eq!(out, "ab ");
}

/// The `ignore` policy and a custom policy; neither output is protected.
#[test]
fn ignore_and_custom_policies() {
    let u = encoder(Options { unknown_char_policy: UnknownCharPolicy::Ignore, ..Options::default() });
    assert_eq!(u.encode("aธb").unwrap(), "ab");
    let u = encoder(Options {
        unknown_char_policy: UnknownCharPolicy::Custom(Arc::new(|c| format!("\\unknown{{{}}}", c as u32))),
        protection: Protection::BracesAll,
        ..Options::default()
    });
    assert_eq!(u.encode("aธb").unwrap(), "a\\unknown{3608}b");
}

/// pylatexenc's boundary quirk, kept: with `non_ascii_only`, the delete
/// character (U+007F) is not exempted from the rules, and with no rule for
/// it, it is copied as ASCII by the fallback.
#[test]
fn delete_character_at_the_ascii_boundary() {
    let u = encoder(Options {
        non_ascii_only: true,
        rules: vec![Rule::dict([('\u{7F}', "DEL".to_string())])],
        ..Options::default()
    });
    assert_eq!(u.encode("a\u{7F}b").unwrap(), "aDELb");
    let u = encoder(Options { unknown_char_policy: UnknownCharPolicy::Fail, ..Options::default() });
    assert_eq!(u.encode("a\u{7F}b\n\r\t").unwrap(), "a\u{7F}b\n\r\t");
}

/// A rule's own protection replaces the encoder's for its spellings only.
#[test]
fn per_rule_protection_overrides_the_encoder_s() {
    let u = encoder(Options {
        rules: vec![
            Rule::dict([('—', "\\textemdash".to_string())]).with_protection(Protection::None),
            Rule::defaults(),
        ],
        protection: Protection::BracesAll,
        ..Options::default()
    });
    assert_eq!(u.encode("—é").unwrap(), "\\textemdash{\\'e}");
    assert!(matches!(u.options().rules[0].protection(), Some(Protection::None)));
    assert!(u.options().rules[1].protection().is_none());
}

/// An empty spelling deletes the character, and `braces-all` turns it into `{}`.
#[test]
fn the_empty_spelling_of_function_application() {
    assert_eq!(encode("f\u{2061}x"), "fx");
    let u = encoder(Options { protection: Protection::BracesAll, ..Options::default() });
    assert_eq!(u.encode("f\u{2061}x").unwrap(), "f{}x");
}

/// An empty rule list makes every non-ASCII character unknown.
#[test]
fn no_rules_means_everything_non_ascii_is_unknown() {
    let u = encoder(Options { rules: vec![], ..Options::default() });
    let mut out = String::new();
    let report = u.encode_into(&mut out, "a%é").unwrap();
    assert_eq!(out, "a%é");
    assert_eq!(report.unknown, vec![UnknownChar { ch: 'é', position: 2 }]);
}

/// An encoder can be shared between threads.
#[test]
fn an_encoder_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>(_: &T) {}
    let u = encoder(Options {
        rules: vec![Rule::callable(|_, _| None), Rule::defaults()],
        protection: Protection::Custom(Arc::new(|s| s.to_string())),
        unknown_char_policy: UnknownCharPolicy::Custom(Arc::new(|c| c.to_string())),
        ..Options::default()
    });
    assert_send_sync(&u);
    let shared = Arc::new(u);
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let shared = Arc::clone(&shared);
            std::thread::spawn(move || shared.encode("é").unwrap())
        })
        .collect();
    for handle in handles {
        assert_eq!(handle.join().unwrap(), "\\'e");
    }
}

// ---------------------------------------------------------------------------
// The conformance fixture
// ---------------------------------------------------------------------------

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/goldens/latexencode/uni_chars_test_previous.txt")
}

/// Compares `actual` against the golden file at `path`, byte for byte, and
/// rewrites the file instead when `UPDATE_GOLDEN` is set to exactly `"1"`.
///
/// `flm-core` has the same helper as public interface (`dump::golden`), but
/// this crate does not depend on `flm-core` and must not: `flm-core` depends on
/// this crate. The few lines are copied rather than borrowed.
fn assert_text_golden(path: PathBuf, actual: &str) {
    if std::env::var("UPDATE_GOLDEN").ok().as_deref() == Some("1") {
        std::fs::write(&path, actual).expect("the golden is writable");
        return;
    }
    match std::fs::read_to_string(&path) {
        Ok(expected) if expected == actual => {}
        Ok(expected) => panic!(
            "golden {} mismatch\n(run with UPDATE_GOLDEN=1 to rewrite it)\n\
             --- expected ---\n{expected}\n--- actual ---\n{actual}",
            path.display()
        ),
        Err(error) => panic!("cannot read golden {}: {error}", path.display()),
    }
}

/// One line of the fixture, read back: the code point and the character's
/// name, from which pylatexenc's input line is rebuilt.
fn parse_fixture_line(line: &str) -> (u32, &str) {
    let (hex, rest) = line.split_once(' ').expect("a fixture line starts with `0x…`");
    let cp = u32::from_str_radix(hex.strip_prefix("0x").expect("a fixture line starts with `0x`"), 16)
        .expect("a fixture line's code point is hexadecimal");
    let name_start = rest.find('[').expect("a fixture line names the character in brackets");
    let name_end = rest.find(']').expect("a fixture line names the character in brackets");
    (cp, &rest[name_start + 1..name_end])
}

/// pylatexenc's input line for one code point: `"0x%04X %-50s    |%s|\n"`.
fn fixture_input_line(cp: u32, name: &str) -> String {
    let ch = char::from_u32(cp).expect("a fixture code point is a character");
    format!("0x{cp:04X} {:<50}    |{ch}|\n", format!("[{name}]"))
}

/// The fixture's encoder: `braces-almost-all` protection, the `fail` policy.
fn fixture_encoder() -> Encoder {
    encoder(Options {
        protection: Protection::BracesAlmostAll,
        unknown_char_policy: UnknownCharPolicy::Fail,
        ..Options::default()
    })
}

/// Every line of the golden, rebuilt as its input line and encoded, is the
/// golden's line.
///
/// The code points and the Unicode names come from the golden itself — this
/// crate has no Unicode name database — so the file's set of code points never
/// grows on its own; the spellings come from the table. A line whose character
/// the table no longer spells drops out, as it would from pylatexenc's
/// generator, which kept the lines that encode under the `fail` policy.
#[test]
fn the_conformance_golden_is_this_table_s_output() {
    let fixture = std::fs::read_to_string(fixture_path()).expect("the golden is readable");
    let u = fixture_encoder();
    let mut output = String::with_capacity(fixture.len());
    for (cp, name) in fixture.lines().map(parse_fixture_line) {
        let input = fixture_input_line(cp, name);
        // The `fail` policy: a character the table does not spell has no line.
        if let Ok(encoded) = u.encode(&input) {
            output.push_str(&encoded);
        }
    }
    assert_text_golden(fixture_path(), &output);
}

/// The golden shows the table's departures from pylatexenc, which are the only
/// lines that differ from the file pylatexenc generated.
#[test]
fn the_conformance_golden_shows_the_departures() {
    let fixture = std::fs::read_to_string(fixture_path()).expect("the golden is readable");
    let line_of = |cp: u32| {
        fixture
            .lines()
            .find(|line| parse_fixture_line(line).0 == cp)
            .map(|line| line[line.find('|').unwrap()..].to_string())
    };
    // The double-struck range is spelled from three fonts, `\mathds` for `𝟙`
    // alone, and the fractions from `\nicefrac`.
    assert_eq!(line_of(0x1D7D9).as_deref(), Some(r"|{\ensuremath{\mathds{1}}}|"));
    assert_eq!(line_of(0x1D7D8).as_deref(), Some(r"|{\ensuremath{\flmBbold{0}}}|"));
    assert_eq!(line_of(0x1D552).as_deref(), Some(r"|{\ensuremath{\mathbbm{a}}}|"));
    assert_eq!(line_of(0x2153).as_deref(), Some(r"|{\nicefrac{1}{3}}|"));
    // The lowercase script letters come from the `\flmScr` alphabet, the
    // capitals from `\mathscr`.
    assert_eq!(line_of(0x1D4B6).as_deref(), Some(r"|{\ensuremath{\flmScr{a}}}|"));
    assert_eq!(line_of(0x210A).as_deref(), Some(r"|{\ensuremath{\flmScr{g}}}|"));
    assert_eq!(line_of(0x1D49C).as_deref(), Some(r"|{\ensuremath{\mathscr{A}}}|"));
    // A Cyrillic letter carries the switch to the font encoding that declares
    // its command, which is not the same encoding for every letter.
    assert_eq!(line_of(0x044F).as_deref(), Some(r"|{\fontencoding{T2A}\selectfont\cyrya}|"));
    assert_eq!(line_of(0x0463).as_deref(), Some(r"|{\fontencoding{X2}\selectfont\cyryat}|"));
    assert_eq!(line_of(0x046F).as_deref(), Some(r"|{\fontencoding{T2D}\selectfont\cyrksi}|"));
    // A mathematical symbol no package declares is spelled with the `\flm…`
    // command the style file declares from a STIX font.
    assert_eq!(line_of(0x22AB).as_deref(), Some(r"|{\ensuremath{\flmVDash}}|"));
    // The two combining marks, and the two combining Cyrillic number signs no
    // installed font has, have no entry, so they have no line.
    assert_eq!(line_of(0x0307), None);
    assert_eq!(line_of(0x0308), None);
    assert_eq!(line_of(0x0488), None);
    assert_eq!(line_of(0x0489), None);
    // The uppercase double-struck letters keep pylatexenc's spelling.
    assert_eq!(line_of(0x2102).as_deref(), Some(r"|{\ensuremath{\mathbb{C}}}|"));
}

/// The code points the fixture covers.
fn fixture_code_points() -> BTreeSet<u32> {
    let fixture = std::fs::read_to_string(fixture_path()).expect("the fixture is readable");
    fixture.lines().map(|line| parse_fixture_line(line).0).collect()
}

/// Every table entry is exercised by the fixture, except the 21 code points
/// of the Mathematical Alphanumeric Symbols block that Unicode leaves
/// permanently reserved (their letters live elsewhere: U+1D455 is the hole
/// for the Planck constant, U+210E); those have no Unicode name, so the
/// fixture generator never wrote a line for them.
#[test]
fn every_named_table_entry_is_covered_by_the_fixture() {
    let covered = fixture_code_points();
    let reserved_holes: BTreeSet<u32> = [
        0x1D455, 0x1D49D, 0x1D4A0, 0x1D4A1, 0x1D4A3, 0x1D4A4, 0x1D4A7, 0x1D4A8, 0x1D4AD,
        0x1D506, 0x1D50B, 0x1D50C, 0x1D515, 0x1D51D, 0x1D53A, 0x1D53F, 0x1D545, 0x1D547,
        0x1D548, 0x1D549, 0x1D551,
    ]
    .into_iter()
    .collect();
    let uncovered: BTreeSet<u32> =
        tables::DEFAULTS.iter().map(|&(ch, _, _)| ch as u32).filter(|cp| !covered.contains(cp)).collect();
    assert_eq!(uncovered, reserved_holes);
    // The fixture also holds the printable ASCII characters and the code
    // points whose composed form is in the table (U+212B ANGSTROM SIGN).
    assert!(covered.contains(&0x0041));
    assert!(covered.contains(&0x212B));
}

// ---------------------------------------------------------------------------
// Properties of the table
// ---------------------------------------------------------------------------

/// The table is sorted, without duplicates, ASCII-valued, and every spelling
/// is well formed: braces balance (`\{` and `\}` being characters, not
/// braces) and no spelling ends in a lone backslash.
#[test]
fn the_table_is_sorted_and_its_spellings_are_well_formed() {
    assert_eq!(tables::DEFAULTS.len(), 1549);
    for pair in tables::DEFAULTS.windows(2) {
        assert!(pair[0].0 < pair[1].0, "unsorted or duplicate entry at U+{:04X}", pair[1].0 as u32);
    }
    for &(ch, latex, _) in tables::DEFAULTS {
        assert!(latex.is_ascii(), "U+{:04X}: non-ASCII spelling {latex:?}", ch as u32);
        let mut depth = 0i32;
        let mut escaped = false;
        for c in latex.chars() {
            if escaped {
                escaped = false;
                continue;
            }
            match c {
                '\\' => escaped = true,
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
            assert!(depth >= 0, "U+{:04X}: a brace closes before it opens in {latex:?}", ch as u32);
        }
        assert_eq!(depth, 0, "U+{:04X}: unbalanced braces in {latex:?}", ch as u32);
        assert!(!escaped, "U+{:04X}: a lone trailing backslash in {latex:?}", ch as u32);
        assert_eq!(tables::lookup(ch).map(|(latex, _)| latex), Some(latex));
    }
    assert_eq!(tables::lookup('a'), None);
    assert_eq!(tables::lookup('\u{4E00}'), None);
}

/// The entries above the Basic Multilingual Plane (U+1D400 and up) are
/// addressed by code point: one character of four UTF-8 bytes, never a pair
/// of UTF-16 units.
#[test]
fn astral_plane_entries_by_code_point() {
    assert_eq!(spelling_of('\u{1D400}').map(|s| s.latex), Some("\\ensuremath{\\mathbf{A}}"));
    assert_eq!(spelling_of('\u{1D49C}').map(|s| s.latex), Some("\\ensuremath{\\mathscr{A}}"));
    assert_eq!(spelling_of('\u{1D538}').map(|s| s.latex), Some("\\ensuremath{\\mathbb{A}}"));
    assert_eq!(spelling_of('\u{1D552}').map(|s| s.latex), Some("\\ensuremath{\\mathbbm{a}}"));
    assert_eq!(spelling_of('\u{1D7FF}').map(|s| s.latex), Some("\\ensuremath{\\mathtt{9}}"));
    // The reserved hole has an entry too, reproduced as pylatexenc has it.
    assert!(spelling_of('\u{1D455}').is_some());
    assert_eq!(
        encode("𝐀𝟙x"),
        "\\ensuremath{\\mathbf{A}}\\ensuremath{\\mathds{1}}x"
    );
    let mut out = String::new();
    let report = Encoder::default().encode_into(&mut out, "𝐀\u{1F600}").unwrap();
    assert_eq!(report.unknown, vec![UnknownChar { ch: '\u{1F600}', position: 4 }]);
    let astral = tables::DEFAULTS.iter().filter(|&&(ch, _, _)| ch as u32 >= 0x1D400).count();
    assert_eq!(astral, 453);
}

/// The modes read off the table: every `\ensuremath{…}`-wrapped spelling is
/// mathematics, the rest is text or plain characters, and the three known
/// exceptions are text.
#[test]
fn modes_over_the_whole_table() {
    let mut math = 0;
    let mut text = 0;
    let mut any = 0;
    for &(ch, latex, _) in tables::DEFAULTS {
        match spelling_of(ch).map(|s| s.mode) {
            Some(Mode::Math) => {
                math += 1;
                assert!(latex.starts_with("\\ensuremath{") && latex.ends_with('}'));
            }
            Some(Mode::Text) => {
                text += 1;
                assert!(latex.contains('\\'));
            }
            Some(Mode::Any) => {
                any += 1;
                assert!(!latex.contains('\\'));
            }
            None => panic!("U+{:04X} is in the table but has no spelling", ch as u32),
        }
    }
    assert_eq!(math + text + any, 1549);
    assert_eq!(any, 34);
    assert_eq!(spelling_of('\u{2109}').map(|s| s.mode), Some(Mode::Text));
    assert_eq!(spelling_of('\u{038F}').map(|s| s.mode), Some(Mode::Text));
    assert_eq!(spelling_of('\u{25AA}').map(|s| s.mode), Some(Mode::Text));
    assert_eq!(math, 935);
}

// ---------------------------------------------------------------------------
// The chunks, the profiles and what a document needs
// ---------------------------------------------------------------------------

/// The chunk table is well formed: distinct identifiers, LaTeX of the kind
/// each chunk says it is, a sentence of documentation, and few enough chunks
/// to be held in one machine word, which is how a set of them is held.
#[test]
fn the_chunk_table_is_well_formed() {
    let ids: BTreeSet<&str> = tables::CHUNKS.iter().map(|chunk| chunk.id).collect();
    assert_eq!(ids.len(), tables::CHUNKS.len(), "the chunk identifiers are distinct");
    assert!(tables::CHUNKS.len() <= 64, "a set of chunks is held in 64 bits");
    let mut seen_snippet = false;
    for chunk in tables::CHUNKS {
        assert!(!chunk.id.is_empty());
        assert!(chunk.docs.ends_with('.'), "{}: the documentation is a sentence", chunk.id);
        match chunk.preamble {
            ChunkPreamble::Package(line) => {
                assert!(line.starts_with("\\usepackage"), "{}", chunk.id);
                assert!(line.ends_with('}'), "{}", chunk.id);
                assert!(
                    !seen_snippet,
                    "{}: the package chunks come before the snippet chunks, so that a \
                     preamble written in chunk order loads every package before any \
                     declaration that calls into one",
                    chunk.id
                );
            }
            ChunkPreamble::Snippet(latex) => {
                seen_snippet = true;
                assert!(latex.starts_with('\\'), "{}: the snippet is LaTeX", chunk.id);
                assert!(!latex.ends_with('\n'), "{}: no newline at the end", chunk.id);
            }
        }
        assert_eq!(chunk.latex(), match chunk.preamble {
            ChunkPreamble::Package(line) => line,
            ChunkPreamble::Snippet(latex) => latex,
        });
    }
}

/// A snippet chunk's declarations may call into what a package chunk loads —
/// a text symbol read from a font encoding that `fontenc` declares — and
/// nothing in the chunk itself says so: what puts the package beside the
/// declarations is the profile naming both, with the package chunks before
/// the snippet chunks. This is what pins that arrangement: for every font
/// encoding a snippet declares something in, either the LaTeX kernel declares
/// the encoding itself, or the snippet does, or every profile naming the
/// snippet also names a `fontenc` chunk loaded with that encoding — so that a
/// snippet added with its package forgotten fails here, not in LaTeX.
#[test]
fn a_snippet_chunk_never_appears_without_the_package_chunks_it_calls_into() {
    // The font encodings LaTeX declares by itself, in `fonttext.ltx` and
    // `fontmath.ltx`.
    const KERNEL_ENCODINGS: &[&str] = &["OT1", "T1", "TS1", "TU", "OML", "OMS", "OMX", "U"];
    // The declarations whose second brace group is a font encoding.
    const DECLARING: &[&str] = &[
        "\\DeclareTextSymbol{",
        "\\DeclareTextAccent{",
        "\\DeclareSymbolFont{",
        "\\DeclareMathAlphabet{",
    ];
    fn second_group(line: &str) -> Option<&str> {
        let rest = DECLARING.iter().find_map(|prefix| line.strip_prefix(prefix))?;
        rest.split('}').nth(1).map(|group| group.trim_start_matches('{'))
    }
    let mut dependent: Vec<&str> = Vec::new();
    for (index, chunk) in tables::CHUNKS.iter().enumerate() {
        let ChunkPreamble::Snippet(latex) = chunk.preamble else { continue };
        let own: BTreeSet<&str> = latex
            .lines()
            .filter_map(|line| line.strip_prefix("\\DeclareFontEncoding{"))
            .filter_map(|rest| rest.split('}').next())
            .collect();
        let needed: BTreeSet<&str> = latex
            .lines()
            .filter_map(second_group)
            .filter(|enc| !KERNEL_ENCODINGS.contains(enc) && !own.contains(enc))
            .collect();
        if needed.is_empty() {
            continue;
        }
        dependent.push(chunk.id);
        for (number, chunks) in tables::PROFILES.iter().enumerate() {
            if !chunks.contains(&index) {
                continue;
            }
            for enc in &needed {
                let loaded = chunks.iter().any(|&i| match tables::CHUNKS[i].preamble {
                    ChunkPreamble::Package(line) => {
                        line.ends_with("{fontenc}")
                            && line
                                .trim_start_matches("\\usepackage[")
                                .split(']')
                                .next()
                                .is_some_and(|options| options.split(',').any(|o| o == *enc))
                    }
                    ChunkPreamble::Snippet(_) => false,
                });
                assert!(
                    loaded,
                    "profile {number} names the snippet chunk {} but no `fontenc` chunk \
                     loaded with the `{enc}` encoding its declarations read from",
                    chunk.id
                );
            }
        }
    }
    // Today exactly one snippet chunk calls into a package chunk: the Cyrillic
    // thousands sign, read from a slot of the `T2D` encoding.
    assert_eq!(dependent, ["cyrillic-thousands"]);
}

/// Every profile names chunks that exist, in increasing order and without
/// repetition; profile 0 is the empty set; and no two profiles are the same
/// set, so that one set of chunks has one profile.
#[test]
fn every_profile_names_chunks_that_exist_and_no_set_appears_twice() {
    assert_eq!(tables::PROFILES.first().copied(), Some(&[][..]), "profile 0 is the empty set");
    let mut seen: BTreeSet<&[usize]> = BTreeSet::new();
    for (number, chunks) in tables::PROFILES.iter().enumerate() {
        for pair in chunks.windows(2) {
            assert!(pair[0] < pair[1], "profile {number}: the chunk indices are sorted");
        }
        for &index in *chunks {
            assert!(index < tables::CHUNKS.len(), "profile {number}: no chunk {index}");
        }
        assert!(seen.insert(chunks), "profile {number} is a set another profile already has");
    }
    assert!(tables::PROFILES.len() <= 256, "a profile is one byte");
}

/// Every entry of the table names a profile the table has.
#[test]
fn every_entry_names_a_profile_that_exists() {
    for &(ch, latex, profile) in tables::DEFAULTS {
        let mut needs = PreambleNeeds::new();
        needs.include(profile);
        // A profile outside `PROFILES` would answer the empty set here, so the
        // check is that the entry's own lookup agrees with the table's row.
        assert_eq!(
            tables::lookup(ch).map(|(_, p)| p),
            Some(profile),
            "U+{:04X} {latex}",
            ch as u32
        );
        assert!(
            profile == Profile::BUILTINS || !needs.is_empty(),
            "U+{:04X} {latex}: a profile that is not `BUILTINS` names at least one chunk",
            ch as u32
        );
    }
}

/// Every profile of the table is pinned here by a character that needs it:
/// the corrections the table carries name the chunks that make them work, and
/// one character of every other profile names the chunk it must resolve to.
///
/// The profile constants of `tables.rs` name their rows of `PROFILES` by
/// number, so a mistyped number would hand a character another profile's
/// chunks with nothing else noticing — the structural tests check only that a
/// profile exists and names chunks that exist. This test is what catches
/// that, and its last assertion fails when a profile is added with no
/// character to pin it.
#[test]
fn the_corrected_entries_name_their_chunks() {
    let chunks_of = |ch: char| {
        let mut needs = PreambleNeeds::new();
        needs.include(spelling_of(ch).unwrap().needs);
        needs.chunks().map(|chunk| chunk.id).collect::<Vec<_>>()
    };
    // Characters of every profile the table has, each with the chunks its
    // spelling needs and nothing else.
    let pinned: &[(&[char], &[&str])] = &[
        (&['é', 'ℓ'], &[]),
        // The double-struck range is spelled from three fonts: `\mathds` for
        // `𝟙` alone, `\mathbbm` for the letters and `𝟚`, and the `\flmBbold`
        // alphabet for the digits `bbm`'s font has no glyph for.
        (&['𝟙'], &["dsfont"]),
        (&['𝟚', '𝕒', '𝕫'], &["bbm"]),
        (&['𝟘', '𝟛', '𝟡'], &["bbold-alphabet"]),
        (&['⅓', '⅞'], &["nicefrac"]),
        (&['Я', 'я', 'ё'], &["fontenc-t2a"]),
        (&['ą'], &["fontenc-t1"]),
        (&['ə'], &["tipa"]),
        (&['ƞ'], &["tipx"]),
        (&['ℂ'], &["amssymb"]),
        (&['∬'], &["amsmath"]),
        // `\mathscr` prints the capitals; the lowercase letters come from the
        // `\flmScr` alphabet, whose font has them.
        (&['𝒜'], &["mathrsfs"]),
        // … including the three script lowercase letters Unicode keeps in
        // the Letterlike Symbols block.
        (&['𝒶', '𝓏', 'ℊ', 'ℯ', 'ℴ'], &["script-alphabet"]),
        // The Cyrillic letters no `T2A` font has, each in the encoding that
        // declares its command.
        (&['Ѣ', 'ѣ'], &["fontenc-x2"]),
        (&['Ӿ', 'ӻ'], &["fontenc-t2b"]),
        (&['Ҍ'], &["fontenc-t2c"]),
        (&['Ѳ'], &["fontenc-ot2"]),
        (&['Ѡ', 'ѯ'], &["fontenc-t2d"]),
        (&['҂'], &["fontenc-t2d", "cyrillic-thousands"]),
        // The mathematical symbols no package declares, and the one the
        // `wasy` font has.
        (&['⊫', '≌', '⨏'], &["stix-symbols"]),
        (&['⌕'], &["wasy-recorder"]),
    ];
    for (chars, chunks) in pinned {
        for &ch in *chars {
            assert_eq!(&chunks_of(ch)[..], *chunks, "U+{:04X} {ch}", ch as u32);
        }
    }
    // `\ell`, a letterlike symbol the kernel spells by itself, is not swept
    // into a correction just because it sits among the blackboard-bold and
    // black-letter capitals that need `amssymb`.
    assert_eq!(spelling_of('ℓ').unwrap().needs, Profile::BUILTINS);
    // Every profile the table has is pinned above: one added with no character
    // to pin it fails here.
    let pinned_sets: BTreeSet<Vec<&str>> =
        pinned.iter().map(|(_, chunks)| chunks.to_vec()).collect();
    let all_sets: BTreeSet<Vec<&str>> = tables::PROFILES
        .iter()
        .map(|chunks| chunks.iter().map(|&index| tables::CHUNKS[index].id).collect())
        .collect();
    assert_eq!(pinned_sets, all_sets, "every profile of the table is pinned by a character here");
}

/// A set of needs unions the profiles put into it, lists its chunks in the
/// chunk table's order whatever the order they went in, and writes them as
/// `\usepackage` lines.
#[test]
fn a_set_of_needs_unions_profiles_and_keeps_the_table_s_order() {
    let mut needs = PreambleNeeds::new();
    assert!(needs.is_empty());
    assert_eq!(needs, PreambleNeeds::default());
    assert_eq!(needs.preamble(), "");
    // In the reverse of the chunk table's order.
    needs.include(spelling_of('я').unwrap().needs);
    needs.include(spelling_of('⅓').unwrap().needs);
    needs.include(spelling_of('𝟙').unwrap().needs);
    // Twice over changes nothing, and `BUILTINS` adds nothing.
    needs.include(spelling_of('𝟙').unwrap().needs);
    needs.include(Profile::BUILTINS);
    assert!(!needs.is_empty());
    assert_eq!(needs.chunks().map(|chunk| chunk.id).collect::<Vec<_>>(), [
        "dsfont",
        "nicefrac",
        "fontenc-t2a"
    ]);
    assert_eq!(
        needs.preamble(),
        "\\usepackage{dsfont}\n\\usepackage{nicefrac}\n\\usepackage[T2A,T1]{fontenc}\n"
    );
    assert_eq!(format!("{needs:?}"), r#"["dsfont", "nicefrac", "fontenc-t2a"]"#);

    let mut other = PreambleNeeds::new();
    other.include(spelling_of('𝟙').unwrap().needs);
    let mut joined = PreambleNeeds::new();
    joined.include_all(&other);
    assert_ne!(joined, needs);
    joined.include_all(&needs);
    assert_eq!(joined, needs);
}

/// The encoder reports what the string it encoded needs; a spelling of a rule
/// of the caller's contributes nothing.
#[test]
fn the_encoder_reports_what_the_text_needs() {
    let mut out = String::new();
    let report = Encoder::default().encode_into(&mut out, "Café").unwrap();
    assert!(report.needs.is_empty());

    let mut out = String::new();
    let report = Encoder::default().encode_into(&mut out, "𝟙 ⅓ я").unwrap();
    assert_eq!(report.needs.chunks().map(|chunk| chunk.id).collect::<Vec<_>>(), [
        "dsfont",
        "nicefrac",
        "fontenc-t2a"
    ]);
    assert!(report.unknown.is_empty());

    // A dictionary rule ahead of the table takes the character over, and what
    // its spelling needs is the caller's to know.
    let u = encoder(Options {
        rules: vec![Rule::dict([('𝟙', String::from("\\mathbbm{1}"))]), Rule::defaults()],
        ..Options::default()
    });
    let mut out = String::new();
    let report = u.encode_into(&mut out, "𝟙").unwrap();
    assert_eq!(out, "\\mathbbm{1}");
    assert!(report.needs.is_empty());
}

/// The command names of a piece of LaTeX.
#[test]
fn the_commands_a_spelling_writes_are_readable() {
    let names: Vec<&str> = macro_names(r"\ensuremath{\mathds{1}} \'e \\ x\% \flmLabel@x").collect();
    assert_eq!(names, ["ensuremath", "mathds", "flmLabel"]);
    assert!(macro_names("plain").next().is_none());
    assert!(macro_names("\\").next().is_none());
    // Every spelling of the table is readable this way.
    for &(_, latex, _) in tables::DEFAULTS {
        for name in macro_names(latex) {
            assert!(!name.is_empty());
            assert!(name.chars().all(|c| c.is_ascii_alphabetic()), "\\{name}");
        }
    }
}
