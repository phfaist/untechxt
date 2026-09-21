//! The suite ported from the initial Rust port (`initial-rust-port/`), which
//! had itself been ported from pylatexenc's `test/test_latexencode.py`, plus
//! the conformance golden.
//!
//! Three groups:
//!
//! - **The ported tests**: pylatexenc's own encoder tests, rewritten against
//!   this crate's API with the same input and the same expected output. Each
//!   keeps the name it had in `initial-rust-port/tests/latexencode.rs`, so
//!   that the correspondence to the old suite stays visible; a test whose
//!   expected output changed with the new design says so in a comment. The
//!   tests of features this design dropped — the `non_ascii_only` flag, the
//!   per-rule protection override, `macro_names`, `Mode::of` — are replaced by
//!   tests of what took their place, or are gone with the feature.
//! - **The conformance golden**:
//!   `tests/goldens/latexencode/uni_chars_test_previous.txt` holds one line per
//!   code point — the code point, the character's Unicode name and the
//!   character — all encoded under pylatexenc's `braces-almost-all` protection
//!   and the `fail` unknown-character policy. The test rebuilds every line from
//!   the golden itself, encodes it, and compares byte for byte; `UPDATE_GOLDEN=1`
//!   rewrites the file, which is for a deliberate change to the table alone
//!   (see the golden's `README.md`). It is the acceptance test of the whole
//!   table, of the mode wrapping, of NFC and of the encoder loop at once.
//! - **Properties of the builtin table**: the modes, the profiles and the
//!   chunks they name, coverage by the golden, the entries above the Basic
//!   Multilingual Plane addressed by code point.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

use untechxt::builtin::default_table::ENTRIES;
use untechxt::builtin::needs_profiles::PROFILES;
use untechxt::{
    nfc, rule_fn, unknown_unihex, AsciiSet, BoxError, BracesAroundAll, Chunk, ChunkPreamble,
    DynTable, EncodeError, EncodeReport, EncodeReporter, Encoder, LookupTable,
    MacroNameProtection, NoReport, OutBuffer, PreambleNeeds, Profile, ProfileIndex, ProtectInput,
    ReplacementProtection, ReplacementProtectionHint as Hint, Rule, RuleChain, RuleInput,
    RuleResult, StandardProtection, UnknownCharPolicy, ValueMode, DEFAULTS, NON_ASCII,
};

/// The sentence most of pylatexenc's tests encode.
const SANTE: &str = "\"À votre santé!\" s'exclama le maître de maison à 100%.";

/// The encoder of the builtin table under this crate's defaults: text-mode
/// protection, NFC normalization, unknown characters kept. This is what the
/// old suite's `Encoder::default()` and free `encode()` were.
fn defaults() -> Encoder<&'static untechxt::BuiltinTable> {
    Encoder::new(&DEFAULTS)
}

/// A reporter that keeps every unknown character with the position it was met
/// at, which [`EncodeReport`] deliberately does not.
#[derive(Debug, Default)]
struct PositionReport {
    /// The unknown characters, in the order they were met.
    unknown: Vec<(char, usize)>,
}

impl EncodeReporter for PositionReport {
    fn report_unknown_char(&mut self, ch: char, position: usize) {
        self.unknown.push((ch, position));
    }
}

/// The value `encoded` under the hint `hint`, as `protection` writes it.
fn protect<P: ReplacementProtection>(protection: &P, encoded: &str, hint: Hint) -> String {
    let mut out = String::new();
    protection.write_protected(&mut out, &mut NoReport, ProtectInput::new(encoded, hint)).unwrap();
    out
}

// ---------------------------------------------------------------------------
// The ported tests
// ---------------------------------------------------------------------------

/// pylatexenc `test_basic_0` / `test_basic_0b`: the default encoder.
#[test]
fn basic_0_default_encoder() {
    let expected = "''\\`A votre sant\\'e!'' s'exclama le ma\\^itre de maison \\`a 100\\%.";
    assert_eq!(defaults().encode(SANTE).unwrap(), expected);
}

/// pylatexenc `test_basic_1`: `non_ascii_only` with `braces-all`. The flag is
/// gone: the `NON_ASCII` view of the builtin table is what leaves the ASCII
/// characters alone, and `braces-all` is the `BracesAroundAll` strategy.
#[test]
fn basic_1_non_ascii_only_braces_all() {
    let u = Encoder::new(&NON_ASCII)
        .with_protection(BracesAroundAll(StandardProtection::text_mode()));
    assert_eq!(
        u.encode(SANTE).unwrap(),
        "\"{\\`A} votre sant{\\'e}!\" s'exclama le ma{\\^i}tre de maison {\\`a} 100%."
    );
}

/// pylatexenc `test_basic_2`: `braces-after-macro`, which changes nothing
/// here, every value of the sentence terminating itself.
#[test]
fn basic_2_braces_after_macro() {
    let u = defaults().with_protection(StandardProtection {
        protect_names: MacroNameProtection::BracesAfter,
        ..StandardProtection::text_mode()
    });
    assert_eq!(
        u.encode(SANTE).unwrap(),
        "''\\`A votre sant\\'e!'' s'exclama le ma\\^itre de maison \\`a 100\\%."
    );
}

/// pylatexenc `test_basic_2a` (its issue 44): `braces-after-macro` appends
/// `{}` after `\l` and `\textasciitilde` but not after `\c{c}`, and a math
/// value that the mode wrapper closed needs nothing appended either.
#[test]
fn basic_2a_braces_after_macro_issue_44() {
    let u = defaults().with_protection(StandardProtection {
        protect_names: MacroNameProtection::BracesAfter,
        ..StandardProtection::text_mode()
    });
    assert_eq!(
        u.encode("Jabłoński, François, ⟨.⟩, ~").unwrap(),
        "Jab\\l{}o\\'nski, Fran\\c{c}ois, \\ensuremath{\\langle}.\\ensuremath{\\rangle}, \\textasciitilde{}"
    );
}

/// pylatexenc `test_basic_2b`: no macro-name protection at all.
#[test]
fn basic_2b_protection_none() {
    let u = defaults().with_protection(StandardProtection {
        protect_names: MacroNameProtection::NoProtection,
        ..StandardProtection::text_mode()
    });
    assert_eq!(
        u.encode(SANTE).unwrap(),
        "''\\`A votre sant\\'e!'' s'exclama le ma\\^itre de maison \\`a 100\\%."
    );
}

/// pylatexenc `test_basic_2c`: with the `NON_ASCII` table in place of the
/// `non_ascii_only` flag, the ASCII specials pass through untouched.
#[test]
fn basic_2c_ascii_specials_untouched_with_non_ascii_only() {
    let u = Encoder::new(&NON_ASCII);
    let ascii = " \" # $ % & \\ _ { } ~ ";
    assert_eq!(u.encode(ascii).unwrap(), ascii);
}

/// pylatexenc `test_basic_2d`: with the whole table, every special is spelled.
#[test]
fn basic_2d_ascii_specials_spelled() {
    assert_eq!(
        defaults().encode(" \" # $ % & \\ _ { } ~ ").unwrap(),
        " '' \\# \\$ \\% \\& {\\textbackslash} \\_ \\{ \\} {\\textasciitilde} "
    );
}

/// pylatexenc `test_basic_callable_replacement_latex_protection`: a protection
/// strategy of the caller's is applied to every value. Here it is a
/// `ReplacementProtection` implementation rather than a callable.
#[test]
fn basic_custom_protection_applies_to_every_spelling() {
    /// Writes `{***{` … `}***}` around the text-mode rendering of every value.
    #[derive(Debug)]
    struct Stars(StandardProtection);

    impl ReplacementProtection for Stars {
        fn write_protected<O: OutBuffer, Rep: EncodeReporter>(
            &self,
            out: &mut O,
            report: &mut Rep,
            item: ProtectInput<'_>,
        ) -> Result<(), BoxError> {
            if item.hint() == Hint::DoNotProtect {
                return out.push_str(item.encoded());
            }
            out.push_str("{***{")?;
            match self.0.mode_wrapper_for(item.hint()) {
                Some(wrap) => {
                    if let Some(needs) = wrap.needs {
                        report.report_needs(needs);
                    }
                    out.push_str(&wrap.open)?;
                    out.push_str(item.encoded())?;
                    out.push_str(&wrap.close)?;
                }
                None => out.push_str(item.encoded())?,
            }
            out.push_str("}***}")
        }
    }

    let u = defaults().with_protection(Stars(StandardProtection::text_mode()));
    assert_eq!(
        u.encode(SANTE).unwrap(),
        "{***{''}***}{***{\\`A}***} votre sant{***{\\'e}***}!{***{''}***} s'exclama le ma{***{\\^i}***}tre de maison {***{\\`a}***} 100{***{\\%}***}."
    );
}

/// pylatexenc `test_basic_3`: the `keep` policy keeps the character, and the
/// character is reported. `EncodeReport` holds the character; the position
/// comes from a reporter of the caller's.
#[test]
fn basic_3_unknown_kept_and_reported() {
    let text = "A unicode character: ธ";
    let u = defaults().with_unknown_chars(UnknownCharPolicy::Keep);

    let (out, report) = u.encode_with_report(text).unwrap();
    assert_eq!(out, text);
    assert_eq!(report.unknown_chars.iter().copied().collect::<Vec<_>>(), ['ธ']);

    let mut out = String::new();
    let mut positions = PositionReport::default();
    u.encode_into(text, &mut out, &mut positions).unwrap();
    assert_eq!(positions.unknown, [('ธ', 21)]);
}

/// pylatexenc `test_basic_3b`: the `replace` policy.
#[test]
fn basic_3b_unknown_replaced() {
    let u = defaults()
        .with_unknown_chars(UnknownCharPolicy::ReplaceWith("{\\bfseries ?}".into()));
    assert_eq!(u.encode("A unicode character: ธ").unwrap(), "A unicode character: {\\bfseries ?}");
}

/// pylatexenc `test_basic_3c`: the `unihex` policy, which is now the plain
/// function `unknown_unihex` passed to `UnknownCharPolicy::callback`.
#[test]
fn basic_3c_unknown_unihex() {
    let u = defaults().with_unknown_chars(UnknownCharPolicy::callback(unknown_unihex));
    assert_eq!(
        u.encode("A unicode character: ธ").unwrap(),
        "A unicode character: \\ensuremath{\\langle}\\texttt{U+0E18}\\ensuremath{\\rangle}"
    );
}

/// The three regular-expression rules of pylatexenc's `test_rules_00`, as a
/// closure rule: `v(otre)` → `notre`, `s'exclama` (any case) → `s'exprima`,
/// `î` → `{\^i}`.
fn rules_00_regex_stand_in(input: RuleInput<'_>) -> RuleResult<'_> {
    let rest = input.rest();
    if rest.starts_with("votre") {
        return Ok(Some(input.replace_prefix(5, "notre", Hint::any_mode("notre"))));
    }
    let exclama = "s'exclama";
    if rest.len() >= exclama.len()
        && rest.is_char_boundary(exclama.len())
        && rest[..exclama.len()].eq_ignore_ascii_case(exclama)
    {
        return Ok(Some(input.replace_prefix(
            exclama.len(),
            "s'exprima",
            Hint::any_mode("s'exprima"),
        )));
    }
    if input.ch() == 'î' {
        return Ok(Some(input.replace_char(r"{\^i}", Hint::text_only(r"{\^i}"))));
    }
    Ok(None)
}

/// The trailing callable of pylatexenc's `test_rules_00`: `é` and `...`.
fn rules_00_callable(input: RuleInput<'_>) -> RuleResult<'_> {
    if input.ch() == 'é' {
        return Ok(Some(input.replace_char(r"{\'{e}}", Hint::text_only(r"{\'{e}}"))));
    }
    if input.rest().starts_with("...") {
        return Ok(Some(input.replace_prefix(3, r"\ldots", Hint::any_mode(r"\ldots"))));
    }
    Ok(None)
}

/// pylatexenc `test_rules_00` / `_00b` / `_00c`: rule order. A table and a
/// closure before the builtin table win over it; a closure after it fires only
/// where the table did not (`é` keeps the table's `\'e`; `...` gets
/// `{\ldots}`).
#[test]
fn rules_00_order_of_rules() {
    let mut overrides = DynTable::new();
    overrides.insert('À', r"{{\`{A}}}", Hint::text_only(r"{{\`{A}}}"));
    overrides.insert('%', r"\textpercent", Hint::text_only(r"\textpercent"));
    let chain = RuleChain::new((
        overrides,
        rule_fn(rules_00_regex_stand_in),
        &DEFAULTS,
        rule_fn(rules_00_callable),
    ));
    let u = Encoder::new(chain);
    let input = "\"À votre santé!\" s'exclama le maître de maison ... à 100%.";
    assert_eq!(
        u.encode(input).unwrap(),
        "''{{\\`{A}}} notre sant\\'e!'' s'exprima le ma{\\^i}tre de maison {\\ldots} \\`a 100{\\textpercent}."
    );
}

/// pylatexenc `test_rules_callable_must_consume_at_least_one_char`, and the
/// initial port's unit test of a consumption past the end or inside a
/// character. The encoder has no "invalid consumption" error any more: a
/// length that is not an advance is refused where the replacement is built,
/// and `?` turns the refusal into a rule error.
#[test]
fn rules_callable_must_consume_at_least_one_char() {
    // At the `é` of `santé`: zero bytes, past the end, and inside the
    // character are all refused.
    let input = RuleInput::new("santé", 4).unwrap();
    for n_bytes in [0, 100, 1] {
        let err = input.try_replace_prefix(n_bytes, r"\ldots", Hint::any_mode(r"\ldots"));
        let err = err.expect_err("a length that is not an advance is refused");
        assert_eq!(err.position, 4);
        assert_eq!(err.n_bytes, n_bytes);
        assert_eq!(err.available, 2);
    }
    // One whole character is the least a rule may consume, and it is accepted.
    let ok = input.try_replace_prefix(2, "e", Hint::any_mode("e")).unwrap();
    assert_eq!(ok.consumed(), 2);

    // Through a rule, the refusal stops the encoding at the position.
    let u = Encoder::new(rule_fn(|input: RuleInput<'_>| {
        Ok(Some(input.try_replace_prefix(0, r"\ldots", Hint::any_mode(r"\ldots"))?))
    }));
    let err = u.encode("santé").unwrap_err();
    assert!(matches!(err, EncodeError::Rule { position: 0, .. }), "{err:?}");
}

/// The initial port's unit test
/// `a_callable_consuming_past_the_end_or_inside_a_character_is_refused`: the
/// panicking form of the same refusal.
#[test]
#[should_panic = "a rule must consume at least one whole character"]
fn a_callable_consuming_past_the_end_or_inside_a_character_is_refused() {
    let input = RuleInput::new("é", 0).unwrap();
    let _ = input.replace_prefix(1, "", Hint::any_mode(""));
}

/// pylatexenc `test_rules_02` / `_02b`: the superscript two.
#[test]
fn rules_02_superscript_two() {
    assert_eq!(
        defaults()
            .encode("* \"À votre santé!\" s'exclama² le maître de maison à 100%.")
            .unwrap(),
        "* ''\\`A votre sant\\'e!'' s'exclama{\\texttwosuperior} le ma\\^itre de maison \\`a 100\\%."
    );
}

/// pylatexenc `test_issue_no21`: bracing acronyms through a closure rule that
/// also passes existing braces through (the regular-expression form of the
/// same test is the same closure here).
#[test]
fn issue_no21_acronyms_through_a_callable() {
    fn capitalize_acronyms(input: RuleInput<'_>) -> RuleResult<'_> {
        let rest = input.rest();
        if matches!(input.ch(), '{' | '}') {
            // Existing LaTeX, passed through: the rule vouches for it.
            return Ok(Some(input.replace_char(&rest[..1], Hint::DoNotProtect)));
        }
        // `\b[A-Z]{2,}\w*\b`: at a word start, two capitals then word characters.
        let at_word_start =
            !input.before().ends_with(|c: char| c.is_alphanumeric() || c == '_');
        if !at_word_start {
            return Ok(None);
        }
        if rest.chars().take_while(char::is_ascii_uppercase).count() < 2 {
            return Ok(None);
        }
        let word: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
        let n_bytes = word.len();
        let braced = format!("{{{word}}}");
        let hint = Hint::any_mode(&braced);
        Ok(Some(input.replace_prefix(n_bytes, braced, hint)))
    }

    let u = Encoder::new(RuleChain::new((rule_fn(capitalize_acronyms), &DEFAULTS)));
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
// exercises has no counterpart here.

// ---------------------------------------------------------------------------
// Behavior pinned beyond pylatexenc's own tests
// ---------------------------------------------------------------------------

/// The probe string of the reference study, under the default settings.
#[test]
fn the_study_s_probe_string() {
    assert_eq!(
        defaults().encode("Café — naïve “quotes” ½ ≤ α").unwrap(),
        "Caf\\'e {\\textemdash} na\\\"ive {\\textquotedblleft}quotes{\\textquotedblright} {\\textonehalf} \\ensuremath{\\leq} \\ensuremath{\\alpha}"
    );
}

/// The double-struck one is spelled with `\mathds`, which `dsfont` defines —
/// one of the table's departures from pylatexenc, whose `\mathbb{1}` no
/// package defines. The `\ensuremath{…}` of the old spelling now lives in the
/// entry's mode, which is what the encoder writes it with.
#[test]
fn the_double_struck_one_is_spelled_with_mathds_and_needs_dsfont() {
    let entry = DEFAULTS.lookup('𝟙').unwrap();
    assert_eq!(entry.encoded, r"\mathds{1}");
    assert_eq!(entry.hint, Hint::math_only(r"\mathds{1}"));
    assert_eq!(defaults().encode("𝟙").unwrap(), r"\ensuremath{\mathds{1}}");

    let mut needs = PreambleNeeds::new();
    needs.include(entry.needs.expect("the entry needs a package"));
    let mut preamble = String::new();
    needs.write_preamble(&mut preamble).unwrap();
    assert_eq!(preamble, "\\usepackage{dsfont}\n");
}

/// A base letter followed by a combining accent is composed before lookup.
#[test]
fn input_is_composed_before_lookup() {
    assert_eq!(defaults().encode("e\u{0301}").unwrap(), "\\'e");
    assert_eq!(defaults().encode("Cafe\u{0301} A\u{030A}").unwrap(), "Caf\\'e \\r{A}");
    // Positions are byte offsets into the composed text.
    let mut out = String::new();
    let mut report = PositionReport::default();
    defaults().encode_into("e\u{0301}ธ", &mut out, &mut report).unwrap();
    assert_eq!(out, "\\'eธ");
    assert_eq!(report.unknown, [('ธ', 2)]);
}

/// The `fail` policy stops at the first unknown character, with the output up
/// to it left in the buffer.
#[test]
fn fail_policy_reports_the_first_unknown_character() {
    let u = defaults().with_unknown_chars(UnknownCharPolicy::Fail);
    let mut out = String::new();
    let err = u.encode_into("ab ธ ธ", &mut out, &mut NoReport).unwrap_err();
    assert!(matches!(err, EncodeError::UnknownChar { ch: 'ธ', position: 3 }), "{err:?}");
    assert_eq!(out, "ab ");
}

/// The `ignore` policy and a policy of the caller's; neither output is
/// protected, even under a strategy that braces everything.
#[test]
fn ignore_and_custom_policies() {
    let u = defaults().with_unknown_chars(UnknownCharPolicy::Ignore);
    assert_eq!(u.encode("aธb").unwrap(), "ab");

    let u = defaults()
        .with_protection(BracesAroundAll(StandardProtection::text_mode()))
        .with_unknown_chars(UnknownCharPolicy::callback(|c| format!("\\unknown{{{}}}", c as u32)));
    assert_eq!(u.encode("aธb").unwrap(), "a\\unknown{3608}b");
}

/// pylatexenc's boundary quirk is not reproduced: the delete character and the
/// other non-printable ASCII characters are offered to the rules like any
/// other character, and are unknown characters when no rule takes them.
#[test]
fn delete_character_at_the_ascii_boundary() {
    // A rule may match a control character, whatever else the chain holds.
    let mut controls = DynTable::new();
    controls.insert('\u{7f}', "DEL", Hint::any_mode("DEL"));
    let u = Encoder::new(RuleChain::new((controls, &DEFAULTS)));
    assert_eq!(u.encode("a\u{7f}b").unwrap(), "aDELb");

    // Without such a rule it is an unknown character, and so is `\0`, while
    // `\n`, `\r` and `\t` are copied through.
    let u = defaults().with_unknown_chars(UnknownCharPolicy::Fail);
    let err = u.encode("a\u{7f}b").unwrap_err();
    assert!(matches!(err, EncodeError::UnknownChar { ch: '\u{7f}', position: 1 }), "{err:?}");
    let err = u.encode("a\u{0}b").unwrap_err();
    assert!(matches!(err, EncodeError::UnknownChar { ch: '\u{0}', position: 1 }), "{err:?}");
    assert_eq!(u.encode("ab\n\r\t").unwrap(), "ab\n\r\t");
}

/// A rule that vouches for its value has it written verbatim, whatever the
/// encoder's strategy would otherwise do. This is what replaces the per-rule
/// protection override of pylatexenc and of the initial port.
#[test]
fn per_rule_protection_overrides_the_encoder_s() {
    let verbatim = rule_fn(|input: RuleInput<'_>| {
        Ok((input.ch() == '—')
            .then(|| input.replace_char(r"\textemdash", Hint::DoNotProtect)))
    });
    let u = Encoder::new(RuleChain::new((verbatim, &DEFAULTS)))
        .with_protection(BracesAroundAll(StandardProtection::text_mode()));
    assert_eq!(u.encode("—é").unwrap(), r"\textemdash{\'e}");
    // The same value through the table instead is braced like everything else.
    let u = defaults().with_protection(BracesAroundAll(StandardProtection::text_mode()));
    assert_eq!(u.encode("—é").unwrap(), r"{\textemdash}{\'e}");
}

/// An empty spelling deletes the character, and `braces-all` turns it into
/// `{}`. (The initial port's unit test `braces_all_wraps_the_empty_spelling_too`
/// is the second half of this one.)
#[test]
fn the_empty_spelling_of_function_application() {
    assert_eq!(defaults().encode("f\u{2061}x").unwrap(), "fx");
    let u = defaults().with_protection(BracesAroundAll(StandardProtection::text_mode()));
    assert_eq!(u.encode("f\u{2061}x").unwrap(), "f{}x");
}

/// An empty chain makes every non-ASCII character unknown.
#[test]
fn no_rules_means_everything_non_ascii_is_unknown() {
    let u = Encoder::new(RuleChain::new(()));
    let mut out = String::new();
    let mut report = PositionReport::default();
    u.encode_into("a%é", &mut out, &mut report).unwrap();
    assert_eq!(out, "a%é");
    assert_eq!(report.unknown, [('é', 2)]);
}

/// An encoder over the builtin table can be shared between threads.
#[test]
fn an_encoder_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>(_: &T) {}

    let u = Encoder::new(RuleChain::new((rule_fn(|_: RuleInput<'_>| Ok(None)), &DEFAULTS)))
        .with_unknown_chars(UnknownCharPolicy::callback(|c| c.to_string()));
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
// The conformance golden
// ---------------------------------------------------------------------------

/// pylatexenc's `braces-almost-all` protection, which the golden was produced
/// with and which is not a standard option of this crate: the text-mode
/// rendering of the value, in braces when it starts with a backslash.
///
/// It is written with the public API alone, and so it doubles as the worked
/// example of a protection strategy of the caller's.
#[derive(Debug)]
struct BracesAlmostAll(StandardProtection);

impl ReplacementProtection for BracesAlmostAll {
    fn write_protected<O: OutBuffer, Rep: EncodeReporter>(
        &self,
        out: &mut O,
        report: &mut Rep,
        item: ProtectInput<'_>,
    ) -> Result<(), BoxError> {
        let rendered = match item.hint() {
            Hint::DoNotProtect => return out.push_str(item.encoded()),
            // The wildcard arm is what `#[non_exhaustive]` asks for.
            _ => match self.0.mode_wrapper_for(item.hint()) {
                Some(wrap) => {
                    if let Some(needs) = wrap.needs {
                        report.report_needs(needs);
                    }
                    format!("{}{}{}", wrap.open, item.encoded(), wrap.close)
                }
                None => item.encoded().to_string(),
            },
        };
        if rendered.starts_with('\\') {
            out.push_str("{")?;
            out.push_str(&rendered)?;
            out.push_str("}")
        } else {
            out.push_str(&rendered)
        }
    }
}

/// The golden's encoder: the builtin table, NFC, `braces-almost-all` and the
/// `fail` unknown-character policy.
fn golden_encoder() -> Encoder<&'static untechxt::BuiltinTable, BracesAlmostAll> {
    defaults()
        .with_protection(BracesAlmostAll(StandardProtection::text_mode()))
        .with_unknown_chars(UnknownCharPolicy::Fail)
}

/// Where the golden lives.
fn golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/goldens/latexencode/uni_chars_test_previous.txt")
}

/// The golden's text.
fn golden_text() -> String {
    std::fs::read_to_string(golden_path()).expect("the golden is readable")
}

/// Compares `actual` against the golden file at `path`, byte for byte, and
/// rewrites the file instead when `UPDATE_GOLDEN` is set to exactly `"1"`.
///
/// Rewriting is for a change to the table that was meant, and the diff is read
/// line by line: the golden is never regenerated to make this test pass.
fn assert_text_golden(path: PathBuf, actual: &str) {
    if std::env::var("UPDATE_GOLDEN").ok().as_deref() == Some("1") {
        std::fs::write(&path, actual).expect("the golden is writable");
        return;
    }
    match std::fs::read_to_string(&path) {
        Ok(expected) if expected == actual => {}
        Ok(expected) => {
            let first_difference = expected
                .lines()
                .zip(actual.lines())
                .position(|(a, b)| a != b)
                .map(|n| {
                    format!(
                        "first differing line {}:\n  expected: {}\n  actual:   {}",
                        n + 1,
                        expected.lines().nth(n).unwrap_or(""),
                        actual.lines().nth(n).unwrap_or("")
                    )
                })
                .unwrap_or_else(|| {
                    format!(
                        "the same {} lines, then {} against {}",
                        expected.lines().count().min(actual.lines().count()),
                        expected.lines().count(),
                        actual.lines().count()
                    )
                });
            panic!(
                "golden {} mismatch (the golden is right; the code or the data is wrong)\n{}",
                path.display(),
                first_difference
            );
        }
        Err(error) => panic!("cannot read golden {}: {error}", path.display()),
    }
}

/// One line of the golden, read back: the code point and the character's name,
/// from which pylatexenc's input line is rebuilt.
fn parse_golden_line(line: &str) -> (u32, &str) {
    let (hex, rest) = line.split_once(' ').expect("a golden line starts with `0x…`");
    let hex = hex.strip_prefix("0x").expect("a golden line starts with `0x`");
    let cp = u32::from_str_radix(hex, 16).expect("a golden line's code point is hexadecimal");
    let name_start = rest.find('[').expect("a golden line names the character in brackets");
    let name_end = rest.find(']').expect("a golden line names the character in brackets");
    (cp, &rest[name_start + 1..name_end])
}

/// pylatexenc's input line for one code point: `"0x%04X %-50s    |%s|\n"`.
fn golden_input_line(cp: u32, name: &str) -> String {
    let ch = char::from_u32(cp).expect("a golden code point is a character");
    let named = format!("[{name}]");
    format!("0x{cp:04X} {named:<50}    |{ch}|\n")
}

/// The input lines of the whole golden, in order.
fn golden_input_lines(golden: &str) -> Vec<String> {
    golden.lines().map(parse_golden_line).map(|(cp, name)| golden_input_line(cp, name)).collect()
}

/// Every line of the golden, rebuilt as its input line and encoded, is the
/// golden's line.
///
/// The code points and the Unicode names come from the golden itself — this
/// crate has no Unicode name database — so the file's set of code points never
/// grows on its own; the spellings come from the builtin table. A line whose
/// character the table no longer spells drops out, as it would from
/// pylatexenc's generator, which kept the lines that encode under the `fail`
/// policy.
#[test]
fn the_conformance_golden_is_this_table_s_output() {
    let golden = golden_text();
    let u = golden_encoder();
    let mut output = String::with_capacity(golden.len());
    for input in golden_input_lines(&golden) {
        // The `fail` policy: a character the table does not spell has no line.
        if let Ok(encoded) = u.encode(&input) {
            output.push_str(&encoded);
        }
    }
    assert_text_golden(golden_path(), &output);
}

/// The golden shows the table's departures from pylatexenc, which are the only
/// lines that differ from the file pylatexenc generated.
#[test]
fn the_conformance_golden_shows_the_departures() {
    let golden = golden_text();
    let line_of = |cp: u32| {
        golden
            .lines()
            .find(|line| parse_golden_line(line).0 == cp)
            .map(|line| line[line.find('|').unwrap()..].to_string())
    };
    // The double-struck range is spelled from three fonts, `\mathds` for `𝟙`
    // alone, and the fractions from `\nicefrac`.
    assert_eq!(line_of(0x1D7D9).as_deref(), Some(r"|{\ensuremath{\mathds{1}}}|"));
    assert_eq!(line_of(0x1D7D8).as_deref(), Some(r"|{\ensuremath{\UnxTBbold{0}}}|"));
    assert_eq!(line_of(0x1D552).as_deref(), Some(r"|{\ensuremath{\mathbbm{a}}}|"));
    assert_eq!(line_of(0x2153).as_deref(), Some(r"|{\nicefrac{1}{3}}|"));
    // The lowercase script letters come from the `\UnxTScr` alphabet, the
    // capitals from `\mathscr`.
    assert_eq!(line_of(0x1D4B6).as_deref(), Some(r"|{\ensuremath{\UnxTScr{a}}}|"));
    assert_eq!(line_of(0x210A).as_deref(), Some(r"|{\ensuremath{\UnxTScr{g}}}|"));
    assert_eq!(line_of(0x1D49C).as_deref(), Some(r"|{\ensuremath{\mathscr{A}}}|"));
    // A Cyrillic letter carries the switch to the font encoding that declares
    // its command, which is not the same encoding for every letter.
    assert_eq!(line_of(0x044F).as_deref(), Some(r"|{\fontencoding{T2A}\selectfont\cyrya}|"));
    assert_eq!(line_of(0x0463).as_deref(), Some(r"|{\fontencoding{X2}\selectfont\cyryat}|"));
    assert_eq!(line_of(0x046F).as_deref(), Some(r"|{\fontencoding{T2D}\selectfont\cyrksi}|"));
    // A mathematical symbol no package declares is spelled with the `\UnxT…`
    // command a snippet chunk declares from a STIX font.
    assert_eq!(line_of(0x22AB).as_deref(), Some(r"|{\ensuremath{\UnxTVDash}}|"));
    // The two combining marks, and the two combining Cyrillic number signs no
    // installed font has, have no entry, so they have no line.
    assert_eq!(line_of(0x0307), None);
    assert_eq!(line_of(0x0308), None);
    assert_eq!(line_of(0x0488), None);
    assert_eq!(line_of(0x0489), None);
    // The uppercase double-struck letters keep pylatexenc's spelling.
    assert_eq!(line_of(0x2102).as_deref(), Some(r"|{\ensuremath{\mathbb{C}}}|"));
}

/// The code points the golden covers.
fn golden_code_points() -> BTreeSet<u32> {
    golden_text().lines().map(|line| parse_golden_line(line).0).collect()
}

/// Every table entry is exercised by the golden, except the 21 code points of
/// the Mathematical Alphanumeric Symbols block that Unicode leaves permanently
/// reserved (their letters live elsewhere: U+1D455 is the hole for the Planck
/// constant, U+210E); those have no Unicode name, so the golden's generator
/// never wrote a line for them.
#[test]
fn every_named_table_entry_is_covered_by_the_fixture() {
    let covered = golden_code_points();
    let reserved_holes: BTreeSet<u32> = [
        0x1D455, 0x1D49D, 0x1D4A0, 0x1D4A1, 0x1D4A3, 0x1D4A4, 0x1D4A7, 0x1D4A8, 0x1D4AD, 0x1D506,
        0x1D50B, 0x1D50C, 0x1D515, 0x1D51D, 0x1D53A, 0x1D53F, 0x1D545, 0x1D547, 0x1D548, 0x1D549,
        0x1D551,
    ]
    .into_iter()
    .collect();
    let uncovered: BTreeSet<u32> =
        DEFAULTS.iter().map(|(ch, _)| ch as u32).filter(|cp| !covered.contains(cp)).collect();
    assert_eq!(uncovered, reserved_holes);
    // The golden also holds the printable ASCII characters and the code points
    // whose composed form is in the table (U+212B ANGSTROM SIGN).
    assert!(covered.contains(&0x0041));
    assert!(covered.contains(&0x212B));
}

// ---------------------------------------------------------------------------
// Properties of the builtin table
// ---------------------------------------------------------------------------

/// The keys ascend, and `lookup` agrees with `iter()` over every entry.
///
/// What the old test checked besides — ASCII-only spellings, balanced braces,
/// no trailing lone backslash, no duplicate key — is now checked by
/// `compile_static_table!` at compile time, so a violation is a compile error
/// rather than a failing test.
#[test]
fn the_table_is_sorted_and_its_spellings_are_well_formed() {
    assert_eq!(DEFAULTS.len(), 1549);
    let mut previous: Option<char> = None;
    for (ch, entry) in DEFAULTS.iter() {
        assert!(previous < Some(ch), "unsorted or duplicate entry at U+{:04X}", ch as u32);
        previous = Some(ch);
        assert_eq!(DEFAULTS.lookup(ch), Some(entry), "U+{:04X}", ch as u32);
    }
    assert_eq!(DEFAULTS.lookup('a'), None);
    assert_eq!(DEFAULTS.lookup('\u{4E00}'), None);
}

/// Every entry's lookup hands out the very profile its index names, and a
/// profile that is not the empty one names at least one chunk.
///
/// That every index is in range is a compile-time check of
/// `compile_static_table!`; what is left for a test is that the lookup resolves
/// the index to the right profile.
#[test]
fn every_entry_names_a_profile_that_exists() {
    for &(ch, encoded, _, index) in ENTRIES {
        let entry = DEFAULTS.lookup(ch).unwrap_or_else(|| panic!("U+{:04X}", ch as u32));
        assert_eq!(entry.encoded, encoded, "U+{:04X}", ch as u32);
        match entry.needs {
            None => assert_eq!(index, ProfileIndex::NONE, "U+{:04X} {encoded}", ch as u32),
            Some(profile) => {
                assert!(
                    std::ptr::eq(profile, &PROFILES[index.0 as usize]),
                    "U+{:04X} {encoded}: the lookup resolves another profile",
                    ch as u32
                );
                assert!(
                    !profile.is_empty(),
                    "U+{:04X} {encoded}: a profile that is not `BUILTINS` names at least one chunk",
                    ch as u32
                );
            }
        }
    }
}

/// The entries above the Basic Multilingual Plane (U+1D400 and up) are
/// addressed by code point: one character of four UTF-8 bytes, never a pair of
/// UTF-16 units. The spellings no longer carry their own `\ensuremath{…}`,
/// which is the entry's mode now.
#[test]
fn astral_plane_entries_by_code_point() {
    let spelling = |ch: char| DEFAULTS.lookup(ch).map(|entry| entry.encoded);
    assert_eq!(spelling('\u{1D400}'), Some(r"\mathbf{A}"));
    assert_eq!(spelling('\u{1D49C}'), Some(r"\mathscr{A}"));
    assert_eq!(spelling('\u{1D538}'), Some(r"\mathbb{A}"));
    assert_eq!(spelling('\u{1D552}'), Some(r"\mathbbm{a}"));
    assert_eq!(spelling('\u{1D7FF}'), Some(r"\mathtt{9}"));
    // The reserved hole has an entry too, reproduced as pylatexenc has it.
    assert!(spelling('\u{1D455}').is_some());
    assert_eq!(
        defaults().encode("𝐀𝟙x").unwrap(),
        r"\ensuremath{\mathbf{A}}\ensuremath{\mathds{1}}x"
    );

    let mut out = String::new();
    let mut report = PositionReport::default();
    defaults().encode_into("𝐀\u{1F600}", &mut out, &mut report).unwrap();
    assert_eq!(report.unknown, [('\u{1F600}', 4)]);

    let astral = DEFAULTS.iter().filter(|(ch, _)| *ch as u32 >= 0x1D400).count();
    assert_eq!(astral, 453);
}

/// The modes over the whole table: the entries that were wrapped in
/// `\ensuremath{…}` are the mathematical ones, a spelling with no backslash at
/// all holds in either mode, and everything else is text — the three spellings
/// that mix the two included.
#[test]
fn modes_over_the_whole_table() {
    let mut math = 0;
    let mut text = 0;
    let mut any = 0;
    for &(ch, encoded, mode, _) in ENTRIES {
        let entry = DEFAULTS.lookup(ch).unwrap_or_else(|| panic!("U+{:04X}", ch as u32));
        let compiled = match entry.hint {
            Hint::Value { mode, .. } => mode,
            other => panic!("U+{:04X}: a table entry is a value, not {other:?}", ch as u32),
        };
        assert_eq!(compiled, mode, "U+{:04X} {encoded}", ch as u32);
        match mode {
            ValueMode::MathOnly => {
                math += 1;
                // The `\ensuremath{…}` the old spelling carried is written by
                // the encoder now, from the mode.
                assert_eq!(
                    defaults().encode(&ch.to_string()).unwrap(),
                    format!("\\ensuremath{{{encoded}}}"),
                    "U+{:04X}",
                    ch as u32
                );
            }
            ValueMode::TextOnly => {
                text += 1;
                assert!(encoded.contains('\\'), "U+{:04X} {encoded}", ch as u32);
            }
            ValueMode::AnyMode => {
                any += 1;
                assert!(!encoded.contains('\\'), "U+{:04X} {encoded}", ch as u32);
            }
        }
    }
    assert_eq!(math + text + any, 1549);
    assert_eq!((math, any, text), (935, 34, 580));
    // The three spellings that mix text and mathematics keep their own
    // `\ensuremath{…}` and are text.
    for ch in ['\u{2109}', '\u{038F}', '\u{25AA}'] {
        let entry = DEFAULTS.lookup(ch).unwrap();
        assert_eq!(entry.hint, Hint::text_only(entry.encoded), "U+{:04X}", ch as u32);
        assert!(entry.encoded.contains("\\ensuremath{"), "U+{:04X}", ch as u32);
    }
}

// ---------------------------------------------------------------------------
// The chunks, the profiles and what a document needs
// ---------------------------------------------------------------------------

/// Every distinct chunk the builtin profiles name, in the order a walk of the
/// profiles first meets it.
fn builtin_chunks() -> Vec<&'static Chunk> {
    let mut chunks: Vec<&'static Chunk> = Vec::new();
    for profile in &PROFILES {
        for chunk in profile.chunks() {
            if !chunks.iter().any(|held| held.id == chunk.id) {
                chunks.push(chunk);
            }
        }
    }
    chunks
}

/// The chunk table is well formed: distinct identifiers, and LaTeX of the kind
/// each chunk says it is. (There is no `docs` field and no 64-chunk limit any
/// more: a chunk is documented where it is defined, and a set of chunks is a
/// list, not a bit field.)
#[test]
fn the_chunk_table_is_well_formed() {
    let chunks = builtin_chunks();
    let ids: BTreeSet<&str> = chunks.iter().map(|chunk| &*chunk.id).collect();
    assert_eq!(ids.len(), chunks.len(), "the chunk identifiers are distinct");
    assert_eq!(chunks.len(), 20);
    for chunk in chunks {
        assert!(!chunk.id.is_empty());
        match &chunk.preamble {
            ChunkPreamble::Package(name) => {
                assert!(chunk.is_package());
                assert_eq!(&*chunk.id, &**name, "a plain package chunk is named after it");
                assert!(!name.contains(['{', '}', '\\', '[', ']']), "{}", chunk.id);
            }
            ChunkPreamble::PackageWithOptions(name, options) => {
                assert!(chunk.is_package());
                assert!(!name.is_empty() && !options.is_empty(), "{}", chunk.id);
                assert!(!name.contains(['{', '}', '\\', '[', ']']), "{}", chunk.id);
            }
            ChunkPreamble::Snippet(latex) => {
                assert!(!chunk.is_package());
                assert!(latex.starts_with('\\'), "{}: the snippet is LaTeX", chunk.id);
                assert!(!latex.ends_with('\n'), "{}: no newline at the end", chunk.id);
            }
        }
    }
}

/// A snippet chunk's declarations may call into what a package chunk loads — a
/// text symbol read from a font encoding that `fontenc` declares — and nothing
/// in the chunk itself says so: what puts the package beside the declarations
/// is the profile naming both, and `PreambleNeeds` then writes the packages
/// first. This is what pins that arrangement: for every font encoding a snippet
/// declares something in, either the LaTeX kernel declares the encoding itself,
/// or the snippet does, or every profile naming the snippet also names a
/// `fontenc` chunk loaded with that encoding — so that a snippet added with its
/// package forgotten fails here, not in LaTeX.
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
    for chunk in builtin_chunks() {
        let ChunkPreamble::Snippet(latex) = &chunk.preamble else { continue };
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
        dependent.push(&chunk.id);
        for (number, profile) in PROFILES.iter().enumerate() {
            if !profile.chunks().iter().any(|held| held.id == chunk.id) {
                continue;
            }
            for enc in &needed {
                let loaded = profile.chunks().iter().any(|held| match &held.preamble {
                    ChunkPreamble::PackageWithOptions(name, options) => {
                        &**name == "fontenc" && options.split(',').any(|option| option == *enc)
                    }
                    ChunkPreamble::Package(_) | ChunkPreamble::Snippet(_) => false,
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

/// Profile 0 is the empty set, a profile index still fits the byte a compiled
/// table stores, and no two profiles are the same set of chunks, so that one
/// set of chunks has one profile.
///
/// That every profile names chunks that exist is no longer a question: a
/// profile holds the chunks themselves, not indices into a chunk table.
#[test]
fn every_profile_names_chunks_that_exist_and_no_set_appears_twice() {
    assert!(PROFILES[0].is_empty(), "profile 0 is the empty set");
    assert!(PROFILES.len() <= 256, "a profile index is one byte");
    let mut seen: BTreeSet<Vec<&str>> = BTreeSet::new();
    for (number, profile) in PROFILES.iter().enumerate() {
        let ids: Vec<&str> = profile.chunks().iter().map(|chunk| &*chunk.id).collect();
        assert!(seen.insert(ids), "profile {number} is a set another profile already has");
    }
}

/// The identifiers of the chunks that `ch`'s spelling needs, in the order a
/// preamble would write them.
fn chunk_ids_of(ch: char) -> Vec<String> {
    let entry = DEFAULTS.lookup(ch).unwrap_or_else(|| panic!("U+{:04X} is not in the table", ch as u32));
    let mut needs = PreambleNeeds::new();
    if let Some(profile) = entry.needs {
        needs.include(profile);
    }
    needs.chunks().map(|chunk| chunk.id.to_string()).collect()
}

/// Every profile of the table is pinned here by a character that needs it: the
/// corrections the table carries name the chunks that make them work, and one
/// character of every other profile names the chunk it must resolve to.
///
/// The entries name their profiles by constant, and a mistyped constant would
/// hand a character another profile's chunks with nothing else noticing. This
/// test is what catches that, and its last assertion fails when a profile is
/// added with no character to pin it.
#[test]
fn the_corrected_entries_name_their_chunks() {
    // Characters of every profile the table has, each with the chunks its
    // spelling needs and nothing else.
    let pinned: &[(&[char], &[&str])] = &[
        (&['é', 'ℓ'], &[]),
        // The double-struck range is spelled from three fonts: `\mathds` for
        // `𝟙` alone, `\mathbbm` for the letters and `𝟚`, and the `\UnxTBbold`
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
        // `\UnxTScr` alphabet, whose font has them.
        (&['𝒜'], &["mathrsfs"]),
        // … including the three script lowercase letters Unicode keeps in the
        // Letterlike Symbols block.
        (&['𝒶', '𝓏', 'ℊ', 'ℯ', 'ℴ'], &["script-alphabet"]),
        // The Cyrillic letters no `T2A` font has, each in the encoding that
        // declares its command.
        (&['Ѣ', 'ѣ'], &["fontenc-x2"]),
        (&['Ӿ', 'ӻ'], &["fontenc-t2b"]),
        (&['Ҍ'], &["fontenc-t2c"]),
        (&['Ѳ'], &["fontenc-ot2"]),
        (&['Ѡ', 'ѯ'], &["fontenc-t2d"]),
        (&['҂'], &["fontenc-t2d", "cyrillic-thousands"]),
        // The mathematical symbols no package declares, and the one the `wasy`
        // font has.
        (&['⊫', '≌', '⨏'], &["stix-symbols"]),
        (&['⌕'], &["wasy-recorder"]),
    ];
    for (chars, chunks) in pinned {
        let expected: Vec<String> = chunks.iter().map(|id| (*id).to_string()).collect();
        for &ch in *chars {
            assert_eq!(chunk_ids_of(ch), expected, "U+{:04X} {ch}", ch as u32);
        }
    }
    // `\ell`, a letterlike symbol the kernel spells by itself, is not swept
    // into a correction just because it sits among the blackboard-bold and
    // black-letter capitals that need `amssymb`.
    assert!(DEFAULTS.lookup('ℓ').unwrap().needs.is_none());
    // Every profile the table has is pinned above: one added with no character
    // to pin it fails here.
    let pinned_sets: BTreeSet<Vec<&str>> =
        pinned.iter().map(|(_, chunks)| chunks.to_vec()).collect();
    let all_sets: BTreeSet<Vec<&str>> = PROFILES
        .iter()
        .map(|profile| profile.chunks().iter().map(|chunk| &*chunk.id).collect())
        .collect();
    assert_eq!(pinned_sets, all_sets, "every profile of the table is pinned by a character here");
}

/// A set of needs unions the profiles put into it, lists its chunks with the
/// packages before the snippets and each group in first-seen order — the old
/// order was the chunk table's — and writes them as `\usepackage` lines.
#[test]
fn a_set_of_needs_unions_profiles_and_keeps_the_table_s_order() {
    let profile_of = |ch: char| DEFAULTS.lookup(ch).unwrap().needs.unwrap();

    let mut needs = PreambleNeeds::new();
    assert!(needs.is_empty());
    assert_eq!(needs, PreambleNeeds::default());
    let mut preamble = String::new();
    needs.write_preamble(&mut preamble).unwrap();
    assert_eq!(preamble, "");

    needs.include(profile_of('я'));
    needs.include(profile_of('⅓'));
    needs.include(profile_of('𝟙'));
    // Twice over changes nothing, and the empty profile adds nothing.
    needs.include(profile_of('𝟙'));
    needs.include(&PROFILES[0]);
    assert!(!needs.is_empty());
    assert_eq!(
        needs.chunks().map(|chunk| &*chunk.id).collect::<Vec<_>>(),
        ["fontenc-t2a", "nicefrac", "dsfont"]
    );
    let mut preamble = String::new();
    needs.write_preamble(&mut preamble).unwrap();
    assert_eq!(
        preamble,
        "\\usepackage[T2A,T1]{fontenc}\n\\usepackage{nicefrac}\n\\usepackage{dsfont}\n"
    );
    assert_eq!(format!("{needs:?}"), r#"["fontenc-t2a", "nicefrac", "dsfont"]"#);

    // A snippet chunk is written after every package, whatever order it went in.
    let mut mixed = PreambleNeeds::new();
    mixed.include(profile_of('҂'));
    mixed.include(profile_of('⅓'));
    assert_eq!(
        mixed.chunks().map(|chunk| &*chunk.id).collect::<Vec<_>>(),
        ["fontenc-t2d", "nicefrac", "cyrillic-thousands"]
    );

    let mut other = PreambleNeeds::new();
    other.include(profile_of('𝟙'));
    let mut joined = PreambleNeeds::new();
    joined.merge(&other);
    assert_ne!(joined, needs);
    joined.merge(&needs);
    assert_eq!(joined.chunks().map(|chunk| &*chunk.id).collect::<Vec<_>>(), [
        "dsfont",
        "fontenc-t2a",
        "nicefrac"
    ]);
}

/// The encoder reports what the string it encoded needs; a value of a rule of
/// the caller's contributes what that rule says, which may be nothing.
#[test]
fn the_encoder_reports_what_the_text_needs() {
    let (_, report) = defaults().encode_with_report("Café").unwrap();
    assert!(report.needs.is_empty());

    let (_, report) = defaults().encode_with_report("𝟙 ⅓ я").unwrap();
    assert_eq!(report.needs.chunks().map(|chunk| &*chunk.id).collect::<Vec<_>>(), [
        "dsfont",
        "nicefrac",
        "fontenc-t2a"
    ]);
    assert!(report.unknown_chars.is_empty());

    // A table of the caller's ahead of the builtin one takes the character
    // over, and what its value needs is the caller's to state. The spelling no
    // longer carries its own `\ensuremath{…}`: the encoder writes it from the
    // hint.
    let mut overrides = DynTable::new();
    overrides.insert('𝟙', r"\mathbbm{1}", Hint::math_only(r"\mathbbm{1}"));
    let u = Encoder::new(RuleChain::new((overrides, &DEFAULTS)));
    let (out, report) = u.encode_with_report("𝟙").unwrap();
    assert_eq!(out, r"\ensuremath{\mathbbm{1}}");
    assert!(report.needs.is_empty());
}

// ---------------------------------------------------------------------------
// The initial port's unit tests
// ---------------------------------------------------------------------------

/// Braces go around exactly the values that end with a named command.
#[test]
fn braces_protection_wraps_exactly_the_named_command_endings() {
    let text = StandardProtection::text_mode();
    let wrap = |encoded: &str| protect(&text, encoded, Hint::text_only(encoded));
    assert_eq!(wrap(r"\textemdash"), r"{\textemdash}");
    assert_eq!(wrap(r"\l"), r"{\l}");
    assert_eq!(wrap(r"\'e"), r"\'e");
    assert_eq!(wrap(r"\c{c}"), r"\c{c}");
    assert_eq!(wrap(r"\ensuremath{\alpha}"), r"\ensuremath{\alpha}");
    assert_eq!(wrap("''"), "''");
    assert_eq!(wrap(""), "");
    assert_eq!(wrap(r"\"), r"\");
}

/// `nfc` borrows text that is composed already and composes the rest.
#[test]
fn composed_borrows_composed_text_and_composes_the_rest() {
    assert!(matches!(nfc("Caf\u{e9} \u{3b1}"), std::borrow::Cow::Borrowed(_)));
    assert!(matches!(nfc(""), std::borrow::Cow::Borrowed(_)));
    let owned = nfc("Cafe\u{301} A\u{30a}");
    assert!(matches!(owned, std::borrow::Cow::Owned(_)));
    assert_eq!(owned, "Caf\u{e9} \u{c5}");
    // The encoder sees those same characters: U+212B ANGSTROM SIGN composes to
    // U+00C5, which the table knows, and a lone combining accent composes to
    // nothing else and stays unknown.
    assert_eq!(defaults().encode("e\u{301}").unwrap(), r"\'e");
    assert_eq!(defaults().encode("\u{212B}").unwrap(), defaults().encode("\u{c5}").unwrap());
    assert_eq!(DEFAULTS.lookup('\u{0301}'), None);
}

/// An error names the position it happened at, and repeats what it wraps.
#[test]
fn errors_display_their_position() {
    let u = defaults().with_unknown_chars(UnknownCharPolicy::Fail);
    let err = u.encode("ab ธ").unwrap_err();
    assert_eq!(
        err.to_string(),
        "no known LaTeX representation for character U+0E18 'ธ' at byte 3"
    );

    let failing = rule_fn(|_: RuleInput<'_>| -> RuleResult<'_> { Err("the callback raised".into()) });
    let err = Encoder::new(failing).encode("ab").unwrap_err();
    assert_eq!(err.to_string(), "a rule failed at byte 0: the callback raised");
}

/// The debug forms name what they are without printing the data.
#[test]
fn debug_forms_name_the_kinds() {
    let chain = RuleChain::new((rule_fn(|_: RuleInput<'_>| Ok(None)), &DEFAULTS));
    let shown = format!("{chain:?}");
    assert!(shown.starts_with("RuleChain { rules: (RuleFn(..), BuiltinTable"), "{shown}");
    assert_eq!(format!("{:?}", UnknownCharPolicy::callback(unknown_unihex)), "Callback(..)");
}

// ---------------------------------------------------------------------------
// New tests
// ---------------------------------------------------------------------------

/// A rule that hides another rule's [`Rule::ascii_triggers`] behind
/// [`AsciiSet::ALL`], so that the encoder stops at every ASCII character and
/// consults it there.
#[derive(Debug)]
struct TriggersEverywhere<R>(R);

impl<R: Rule> Rule for TriggersEverywhere<R> {
    fn apply<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a> {
        self.0.apply(input)
    }

    fn ascii_triggers(&self) -> AsciiSet {
        AsciiSet::ALL
    }
}

/// The ASCII fast path is exactly that: over the whole golden, the builtin
/// table and the golden's chain write the same output whether the encoder
/// trusts their triggers or stops at every ASCII character.
#[test]
fn the_ascii_triggers_of_the_builtin_table_change_nothing_but_the_speed() {
    let corpus = golden_input_lines(&golden_text()).concat();
    assert!(!DEFAULTS.ascii_keys().is_empty());

    let trusted = defaults();
    let everywhere = Encoder::new(TriggersEverywhere(&DEFAULTS));
    assert_eq!(trusted.encode(&corpus).unwrap(), everywhere.encode(&corpus).unwrap());

    // And under the golden's own protection and chain.
    let trusted = golden_encoder().with_unknown_chars(UnknownCharPolicy::Keep);
    let everywhere = Encoder::new(TriggersEverywhere(&DEFAULTS))
        .with_protection(BracesAlmostAll(StandardProtection::text_mode()));
    assert_eq!(trusted.encode(&corpus).unwrap(), everywhere.encode(&corpus).unwrap());

    // The `NON_ASCII` view promises the empty set and keeps its promise.
    assert_eq!(NON_ASCII.ascii_keys(), AsciiSet::EMPTY);
    let trusted = Encoder::new(&NON_ASCII);
    let everywhere = Encoder::new(TriggersEverywhere(&NON_ASCII));
    assert_eq!(trusted.encode(&corpus).unwrap(), everywhere.encode(&corpus).unwrap());
}

/// Output going into mathematics: the builtin table's math values are written
/// as they are, its text values are wrapped, and a named macro is ended by a
/// space, which math mode ignores.
#[test]
fn math_output_mode_over_the_builtin_table() {
    let u = defaults().with_protection(StandardProtection::math_mode());
    assert_eq!(u.encode("\u{3b1}\u{2264}\u{3b2}").unwrap(), "\\alpha \\leq \\beta ");
    // `<` is a mathematical spelling with no macro name to protect.
    assert_eq!(u.encode("a<b").unwrap(), "a<b");
    // A text value is wrapped, and the wrapper terminates it.
    assert_eq!(u.encode("\u{e9}\u{2014}").unwrap(), r"\textnormal{\'e}\textnormal{\textemdash}");
    // The empty spelling stays empty in either mode.
    assert_eq!(u.encode("f\u{2061}x").unwrap(), "fx");
}

/// `SpaceAfterMacroName` appends a space instead of braces, which is what the
/// math-mode strategy does by default and what text mode must not do.
#[test]
fn space_after_macro_name_ends_a_named_macro_with_a_space() {
    assert_eq!(StandardProtection::math_mode().protect_names, MacroNameProtection::SpaceAfterMacroName);

    let spaced = StandardProtection {
        protect_names: MacroNameProtection::SpaceAfterMacroName,
        ..StandardProtection::text_mode()
    };
    assert_eq!(protect(&spaced, r"\textemdash", Hint::text_only(r"\textemdash")), "\\textemdash ");
    // A self-terminating value gets nothing appended.
    assert_eq!(protect(&spaced, r"\'e", Hint::text_only(r"\'e")), r"\'e");
    // Through the encoder, over the builtin table.
    assert_eq!(defaults().with_protection(spaced).encode("a\u{2014}b").unwrap(), "a\\textemdash b");
}

/// What is reported never changes what is written: over every input line of
/// the golden, `NoReport` and `EncodeReport` give the same output.
#[test]
fn a_report_does_not_change_the_output_over_the_whole_golden() {
    let u = defaults();
    let mut silent = String::new();
    let mut reported = String::new();
    let mut report = EncodeReport::new();
    for input in golden_input_lines(&golden_text()) {
        u.encode_into(&input, &mut silent, &mut NoReport).unwrap();
        u.encode_into(&input, &mut reported, &mut report).unwrap();
    }
    assert_eq!(silent, reported);
    // And the whole table's needs came out of it.
    assert!(report.needs.chunks().count() >= 20);
    assert!(report.unknown_chars.is_empty());
}

/// A profile a rule of the caller's built at run time reaches the report
/// beside the builtin ones, and a preamble is written from both.
#[test]
fn a_run_time_profile_from_a_user_rule_reaches_the_report() {
    /// A rule that lends its value and its own profile, which a closure rule
    /// cannot do.
    #[derive(Debug)]
    struct Emoji {
        /// What replaces the character.
        encoded: String,
        /// What that value needs in the preamble.
        profile: Profile,
    }

    impl Rule for Emoji {
        fn apply<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a> {
            Ok((input.ch() == '\u{1F600}').then(|| {
                input
                    .replace_char(self.encoded.as_str(), Hint::text_only(&self.encoded))
                    .with_needs(&self.profile)
            }))
        }

        fn ascii_triggers(&self) -> AsciiSet {
            AsciiSet::EMPTY
        }
    }

    let emoji = Emoji {
        encoded: String::from(r"\emoji{grinning-face}"),
        profile: Profile::new(vec![Chunk::package("emoji")]),
    };
    let u = Encoder::new(RuleChain::new((emoji, &DEFAULTS)));
    let (out, report) = u.encode_with_report("\u{1F600} \u{2102}").unwrap();
    assert_eq!(out, r"\emoji{grinning-face} \ensuremath{\mathbb{C}}");
    assert_eq!(report.needs.chunks().map(|chunk| &*chunk.id).collect::<Vec<_>>(), [
        "emoji", "amssymb"
    ]);
    let mut preamble = String::new();
    report.needs.write_preamble(&mut preamble).unwrap();
    assert_eq!(preamble, "\\usepackage{emoji}\n\\usepackage{amssymb}\n");
}
