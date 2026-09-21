//! Unit tests for the core of the crate: the rules, the chain, the
//! protection strategies, the encoder loop and what it reports. The full
//! suite ported from the initial port is in `latexencode.rs`.

use std::cell::RefCell;
use std::fmt::Write as _;
use std::rc::Rc;

use untechxt::lookuptable::{DynTable, LookupTable, TableRule};
use untechxt::normalizer::{nfc, NoNormalization};
use untechxt::outbuffer::{FmtOut, OutBuffer};
use untechxt::preamble::{Chunk, ChunkPreamble, PreambleNeeds, Profile};
use untechxt::protection::{
    BracesAroundAll, MacroNameProtection, ModeWrapper, OutputMode, ProtectInput,
    ReplacementProtection, ReplacementProtectionHint as Hint, StandardProtection, ValueTermination,
};
use untechxt::report::{EncodeReport, EncodeReporter, NoReport};
use untechxt::rule::{
    rule_fn, AsciiSet, DynRuleChain, LocalDynRuleChain, Rule, RuleChain, RuleInput, RuleResult,
};
use untechxt::{unknown_unihex, BoxError, EncodeError, Encoder, UnknownCharPolicy};

// ---------------------------------------------------------------- fixtures

static AMSMATH_CHUNKS: [Chunk; 1] = [Chunk::package("amsmath")];
static AMSMATH: Profile = Profile::from_static(&AMSMATH_CHUNKS);

static BBOLD_CHUNKS: [Chunk; 2] = [
    Chunk::snippet("bbold-alphabet", r"\DeclareMathAlphabet{\UnxTBbold}{U}{bbold}{m}{n}"),
    Chunk::package("amssymb"),
];
static BBOLD: Profile = Profile::from_static(&BBOLD_CHUNKS);

static FONTENC_CHUNKS: [Chunk; 1] = [Chunk::package_with_options("fontenc-t2a", "fontenc", "T2A,T1")];
static FONTENC: Profile = Profile::from_static(&FONTENC_CHUNKS);

/// A small table: an accented letter, a dash, a Greek letter, a ligature.
fn small_table() -> DynTable {
    DynTable::new()
        .with_entry('\u{e9}', r"\'e", Hint::text_only(r"\'e"))
        .with_entry('\u{2014}', r"\textemdash", Hint::text_only(r"\textemdash"))
        .with_entry('\u{3b1}', r"\alpha", Hint::math_only(r"\alpha"))
        .with_entry('\u{fb01}', "fi", Hint::any_mode("fi"))
}

/// A reporter that keeps every unknown character with its position.
#[derive(Debug, Default)]
struct PositionReport {
    unknown: Vec<(char, usize)>,
    needs: PreambleNeeds,
}

impl EncodeReporter for PositionReport {
    fn report_needs(&mut self, profile: &Profile) {
        self.needs.include(profile);
    }

    fn report_unknown_char(&mut self, ch: char, position: usize) {
        self.unknown.push((ch, position));
    }
}

// -------------------------------------------------------- rules and chains

#[test]
fn a_table_rule_matches_its_entries_and_nothing_else() {
    let encoder = Encoder::new(small_table());
    assert_eq!(encoder.encode("Caf\u{e9}").unwrap(), r"Caf\'e");
    assert_eq!(encoder.encode("plain ASCII").unwrap(), "plain ASCII");
    // An entry the table has no key for is left to the unknown-char policy.
    assert_eq!(encoder.encode("\u{3b2}").unwrap(), "\u{3b2}");
}

#[test]
fn the_first_rule_of_a_chain_that_matches_wins() {
    let mut over = DynTable::new();
    over.insert('%', r"\textpercent", Hint::text_only(r"\textpercent"));
    let mut under = DynTable::new();
    under.insert('%', r"\%", Hint::text_only(r"\%"));
    under.insert('&', r"\&", Hint::text_only(r"\&"));

    let chain = RuleChain::new((over, under));
    let encoder = Encoder::new(chain);
    assert_eq!(encoder.encode("% &").unwrap(), r"{\textpercent} \&");
}

#[test]
fn an_optional_rule_is_switched_off_without_changing_the_chain_s_type() {
    let build = |overrides: Option<DynTable>| {
        let mut base = DynTable::new();
        base.insert('%', r"\%", Hint::text_only(r"\%"));
        Encoder::new(RuleChain::new((overrides, base)))
    };
    let mut over = DynTable::new();
    over.insert('%', r"\textpercent", Hint::text_only(r"\textpercent"));

    assert_eq!(build(Some(over)).encode("%").unwrap(), r"{\textpercent}");
    assert_eq!(build(None).encode("%").unwrap(), r"\%");
}

#[test]
fn the_empty_chain_never_matches() {
    let encoder = Encoder::new(RuleChain::new(()));
    assert_eq!(encoder.encode("a\u{e9}").unwrap(), "a\u{e9}");
}

#[test]
fn a_rule_reached_through_a_reference_a_box_or_an_array_works_the_same() {
    let table = small_table();
    let expected = r"Caf\'e";
    assert_eq!(Encoder::new(&table).encode("Caf\u{e9}").unwrap(), expected);
    assert_eq!(
        Encoder::new(Box::new(small_table())).encode("Caf\u{e9}").unwrap(),
        expected
    );
    assert_eq!(
        Encoder::new(RuleChain::new([small_table(), DynTable::new()]))
            .encode("Caf\u{e9}")
            .unwrap(),
        expected
    );
}

#[test]
fn a_closure_rule_may_consume_several_characters_and_lend_from_the_input() {
    let ellipsis = rule_fn(|input: RuleInput<'_>| {
        Ok(input
            .rest()
            .starts_with("...")
            .then(|| input.replace_prefix(3, r"\ldots", Hint::any_mode(r"\ldots"))))
    });
    // A rule that hands back a slice of the input, protected by nothing.
    let verbatim = rule_fn(|input: RuleInput<'_>| {
        let rest = input.rest();
        Ok(rest.strip_prefix('`').and_then(|rest| {
            rest.find('`')
                .map(|end| input.replace_prefix(end + 2, &rest[..end], Hint::DoNotProtect))
        }))
    });
    let encoder = Encoder::new(RuleChain::new((ellipsis, verbatim)));
    assert_eq!(
        encoder.encode(r"wait... `\textbf{now}`").unwrap(),
        r"wait{\ldots} \textbf{now}"
    );
}

#[test]
fn a_rule_error_stops_the_encoding_and_carries_the_position() {
    let failing = rule_fn(|input: RuleInput<'_>| -> RuleResult<'_> {
        if input.ch() == '\u{e9}' {
            Err("no LaTeX for this one".into())
        } else {
            Ok(None)
        }
    });
    let error = Encoder::new(failing).encode("ab\u{e9}").unwrap_err();
    match error {
        EncodeError::Rule { position, ref source } => {
            assert_eq!(position, 2);
            assert_eq!(source.to_string(), "no LaTeX for this one");
        }
        other => panic!("{other:?}"),
    }
    assert!(error.to_string().starts_with("a rule failed at byte 2: "));
}

#[test]
fn a_dyn_chain_is_send_and_sync_and_so_is_a_static_one() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<DynRuleChain<'static>>();
    assert_send_sync::<Encoder<RuleChain<(DynTable, TableRule<DynTable>)>>>();
    assert_send_sync::<Encoder<DynRuleChain<'static>>>();

    let chain = DynRuleChain::empty().with_rule(small_table()).with_rule(rule_fn(
        |input: RuleInput<'_>| {
            Ok((input.ch() == '&').then(|| input.replace_char(r"\&", Hint::text_only(r"\&"))))
        },
    ));
    let encoder = Encoder::new(chain);
    assert_eq!(encoder.encode("\u{e9} & \u{2014}").unwrap(), r"\'e \& {\textemdash}");
}

#[test]
fn a_local_dyn_chain_may_hold_an_rc() {
    let shared: Rc<RefCell<Vec<char>>> = Rc::new(RefCell::new(Vec::new()));
    let seen = Rc::clone(&shared);
    let mut chain = LocalDynRuleChain::empty();
    chain.push(rule_fn(move |input: RuleInput<'_>| {
        if input.ch() == '\u{e9}' {
            seen.borrow_mut().push(input.ch());
            Ok(Some(input.replace_char(r"\'e", Hint::text_only(r"\'e"))))
        } else {
            Ok(None)
        }
    }));
    let encoder = Encoder::new(chain);
    assert_eq!(encoder.encode("Caf\u{e9} \u{e9}").unwrap(), r"Caf\'e \'e");
    assert_eq!(&*shared.borrow(), &['\u{e9}', '\u{e9}']);
}

// ------------------------------------------------------ replacement lengths

#[test]
fn try_replace_prefix_refuses_lengths_that_are_not_an_advance() {
    let input = RuleInput::new("\u{e9}b", 0).unwrap();
    // Zero bytes.
    let err = input.try_replace_prefix(0, "x", Hint::DoNotProtect).unwrap_err();
    assert_eq!((err.position, err.n_bytes, err.available), (0, 0, 3));
    // Past the end.
    assert!(input.try_replace_prefix(4, "x", Hint::DoNotProtect).is_err());
    // Inside a character: `é` is two bytes.
    assert!(input.try_replace_prefix(1, "x", Hint::DoNotProtect).is_err());
    // The whole input is fine.
    assert_eq!(
        input.try_replace_prefix(3, "x", Hint::DoNotProtect).unwrap().consumed(),
        3
    );
    assert!(err.to_string().contains("at byte 0"));
}

#[test]
#[should_panic(expected = "a rule asked to consume 0 bytes at byte 0")]
fn replace_prefix_panics_on_zero() {
    let input = RuleInput::new("abc", 0).unwrap();
    let _ = input.replace_prefix(0, "x", Hint::DoNotProtect);
}

#[test]
#[should_panic(expected = "consume 9 bytes at byte 0, of the 3 that remain")]
fn replace_prefix_panics_past_the_end() {
    let input = RuleInput::new("abc", 0).unwrap();
    let _ = input.replace_prefix(9, "x", Hint::DoNotProtect);
}

#[test]
#[should_panic(expected = "a rule asked to consume 1 bytes")]
fn replace_prefix_panics_inside_a_character() {
    let input = RuleInput::new("\u{e9}", 0).unwrap();
    let _ = input.replace_prefix(1, "x", Hint::DoNotProtect);
}

#[test]
fn an_invalid_length_becomes_a_rule_error_through_the_question_mark() {
    // What a rule driven from another language does: it cannot check the
    // length itself, so `?` turns a bad one into a rule error.
    let greedy = rule_fn(|input: RuleInput<'_>| -> RuleResult<'_> {
        if input.ch() == '\u{e9}' {
            Ok(Some(input.try_replace_prefix(10, "x", Hint::DoNotProtect)?))
        } else {
            Ok(None)
        }
    });
    let error = Encoder::new(greedy).encode("a\u{e9}").unwrap_err();
    match error {
        EncodeError::Rule { position, ref source } => {
            assert_eq!(position, 1);
            assert!(source.to_string().contains("consume 10 bytes at byte 1"));
        }
        other => panic!("{other:?}"),
    }
}

// --------------------------------------------------------------- protection

/// The value `encoded`, protected by `protection` under the hint `hint`.
fn protect<P: ReplacementProtection>(protection: &P, encoded: &str, hint: Hint) -> String {
    let mut out = String::new();
    protection
        .write_protected(&mut out, &mut NoReport, ProtectInput::new(encoded, hint))
        .unwrap();
    out
}

#[test]
fn termination_is_read_off_the_form_of_the_value() {
    use ValueTermination::{ValueEndsWithNamedMacro as Named, ValueIsSelfTerminating as Self_};
    assert_eq!(ValueTermination::inspect(r"\textemdash"), Named);
    assert_eq!(ValueTermination::inspect(r"\hat\i"), Named);
    assert_eq!(ValueTermination::inspect(r"\l"), Named);
    assert_eq!(ValueTermination::inspect(r"\'e"), Self_);
    assert_eq!(ValueTermination::inspect(r"\c{c}"), Self_);
    assert_eq!(ValueTermination::inspect(r"\ensuremath{\alpha}"), Self_);
    assert_eq!(ValueTermination::inspect("''"), Self_);
    assert_eq!(ValueTermination::inspect(""), Self_);
    assert_eq!(ValueTermination::inspect(r"\"), Self_);
}

#[test]
fn each_macro_name_protection_writes_its_own_form() {
    let named = Hint::text_only(r"\textemdash");
    let plain = Hint::text_only(r"\'e");
    for (protect_names, expected) in [
        (MacroNameProtection::BracesAround, r"{\textemdash}"),
        (MacroNameProtection::BracesAfter, r"\textemdash{}"),
        (MacroNameProtection::SpaceAfterMacroName, "\\textemdash "),
        (MacroNameProtection::NoProtection, r"\textemdash"),
    ] {
        let protection = StandardProtection { protect_names, ..StandardProtection::text_mode() };
        assert_eq!(protect(&protection, r"\textemdash", named), expected);
        // A self-terminating value is never touched.
        assert_eq!(protect(&protection, r"\'e", plain), r"\'e");
    }
}

#[test]
fn do_not_protect_is_written_as_it_is() {
    let text = StandardProtection::text_mode();
    assert_eq!(protect(&text, r"\textbf{a}", Hint::DoNotProtect), r"\textbf{a}");
    // Even a value that ends with a named macro, and even in the wrong mode.
    assert_eq!(protect(&text, r"\alpha", Hint::DoNotProtect), r"\alpha");
    let braces = BracesAroundAll(StandardProtection::text_mode());
    assert_eq!(protect(&braces, r"\alpha", Hint::DoNotProtect), r"\alpha");
}

#[test]
fn a_value_of_the_wrong_mode_is_wrapped_in_either_output_mode() {
    let text = StandardProtection::text_mode();
    assert_eq!(protect(&text, r"\alpha", Hint::math_only(r"\alpha")), r"\ensuremath{\alpha}");
    assert_eq!(protect(&text, "fi", Hint::any_mode("fi")), "fi");

    let math = StandardProtection::math_mode();
    assert_eq!(protect(&math, r"\alpha", Hint::math_only(r"\alpha")), "\\alpha ");
    assert_eq!(
        protect(&math, r"\textemdash", Hint::text_only(r"\textemdash")),
        r"\textnormal{\textemdash}"
    );
    assert_eq!(protect(&math, "fi", Hint::any_mode("fi")), "fi");
}

#[test]
fn a_mode_wrapper_reports_what_it_needs_itself() {
    let protection = StandardProtection {
        text_wrap: ModeWrapper::with_needs(r"\text{", "}", &AMSMATH),
        ..StandardProtection::math_mode()
    };
    let mut table = DynTable::new();
    table.insert('\u{2014}', r"\textemdash", Hint::text_only(r"\textemdash"));
    let encoder = Encoder::new(table).with_protection(protection);

    let (out, report) = encoder.encode_with_report("\u{2014}").unwrap();
    assert_eq!(out, r"\text{\textemdash}");
    assert_eq!(report.needs.chunks().map(|c| &*c.id).collect::<Vec<_>>(), ["amsmath"]);
}

#[test]
fn braces_around_all_wraps_every_value_including_the_empty_one() {
    let braces = BracesAroundAll(StandardProtection::text_mode());
    assert_eq!(protect(&braces, r"\'e", Hint::text_only(r"\'e")), r"{\'e}");
    assert_eq!(protect(&braces, "''", Hint::any_mode("''")), "{''}");
    assert_eq!(protect(&braces, "", Hint::any_mode("")), "{}");
    // The mode wrapping of the inner strategy happens inside the braces.
    assert_eq!(
        protect(&braces, r"\alpha", Hint::math_only(r"\alpha")),
        r"{\ensuremath{\alpha}}"
    );
}

#[test]
fn the_output_mode_reaches_the_encoder() {
    let encoder = Encoder::new(small_table()).with_protection(StandardProtection::math_mode());
    assert_eq!(encoder.protection().output_mode, OutputMode::MathMode);
    assert_eq!(
        encoder.encode("\u{3b1} \u{2014} \u{e9}").unwrap(),
        "\\alpha  \\textnormal{\\textemdash} \\textnormal{\\'e}"
    );
}

#[test]
fn a_protection_strategy_can_be_written_outside_the_crate() {
    /// pylatexenc's `braces-almost-all`, written with the public API alone:
    /// the mode rendering of a [`StandardProtection`], in braces when it
    /// starts with a backslash.
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

    let protection = BracesAlmostAll(StandardProtection::text_mode());
    assert_eq!(protect(&protection, r"\'e", Hint::text_only(r"\'e")), r"{\'e}");
    assert_eq!(
        protect(&protection, r"\alpha", Hint::math_only(r"\alpha")),
        r"{\ensuremath{\alpha}}"
    );
    assert_eq!(protect(&protection, "fi", Hint::any_mode("fi")), "fi");
    assert_eq!(protect(&protection, r"\textbf{a}", Hint::DoNotProtect), r"\textbf{a}");

    let encoder = Encoder::new(small_table()).with_protection(protection);
    assert_eq!(
        encoder.encode("Caf\u{e9} \u{3b1}").unwrap(),
        r"Caf{\'e} {\ensuremath{\alpha}}"
    );
}

// ---------------------------------------------------------- unknown chars

#[test]
fn every_unknown_char_policy_writes_its_own_form() {
    let encoder = |policy| Encoder::new(small_table()).with_unknown_chars(policy);
    let text = "a\u{3b2}b";
    assert_eq!(encoder(UnknownCharPolicy::Keep).encode(text).unwrap(), "a\u{3b2}b");
    assert_eq!(encoder(UnknownCharPolicy::Ignore).encode(text).unwrap(), "ab");
    assert_eq!(
        encoder(UnknownCharPolicy::ReplaceWith(r"{\bfseries ?}".into())).encode(text).unwrap(),
        r"a{\bfseries ?}b"
    );
    assert_eq!(
        encoder(UnknownCharPolicy::callback(unknown_unihex)).encode(text).unwrap(),
        r"a\ensuremath{\langle}\texttt{U+03B2}\ensuremath{\rangle}b"
    );
    assert_eq!(
        encoder(UnknownCharPolicy::callback(|ch| format!("<{}>", ch as u32)))
            .encode(text)
            .unwrap(),
        "a<946>b"
    );
}

#[test]
fn failing_on_an_unknown_char_reports_its_position_in_the_normalized_text() {
    let encoder = Encoder::new(small_table()).with_unknown_chars(UnknownCharPolicy::Fail);
    // "Caf" + é (two bytes once composed) + " " + β
    let error = encoder.encode("Cafe\u{301} \u{3b2}").unwrap_err();
    match error {
        EncodeError::UnknownChar { ch, position } => {
            assert_eq!(ch, '\u{3b2}');
            // `\'e` replaces two bytes of normalized input at byte 3.
            assert_eq!(position, 6);
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(
        error.to_string(),
        "no known LaTeX representation for character U+03B2 '\u{3b2}' at byte 6"
    );
}

#[test]
fn unknown_chars_are_reported_whatever_the_policy_says() {
    let encoder = Encoder::new(small_table()).with_unknown_chars(UnknownCharPolicy::Ignore);
    let mut out = String::new();
    let mut report = PositionReport::default();
    encoder.encode_into("\u{3b2}x\u{3b3}", &mut out, &mut report).unwrap();
    assert_eq!(out, "x");
    assert_eq!(report.unknown, [('\u{3b2}', 0), ('\u{3b3}', 3)]);

    let (_, report) = encoder.encode_with_report("\u{3b2}\u{3b2}\u{3b3}").unwrap();
    assert_eq!(report.unknown_chars.iter().copied().collect::<Vec<_>>(), ['\u{3b2}', '\u{3b3}']);
}

#[test]
fn control_characters_are_unknown_but_a_rule_may_still_match_them() {
    let encoder = Encoder::new(RuleChain::new(()));
    // Printable ASCII, and the three white-space characters, are copied.
    assert_eq!(encoder.encode("a b\tc\nd\re").unwrap(), "a b\tc\nd\re");
    // DEL and the other control characters are not.
    let encoder = encoder.with_unknown_chars(UnknownCharPolicy::Ignore);
    assert_eq!(encoder.encode("a\u{7f}b\u{1}c").unwrap(), "abc");

    let mut table = DynTable::new();
    table.insert('\u{7f}', r"\textdelete", Hint::text_only(r"\textdelete"));
    let encoder = Encoder::new(table);
    assert_eq!(encoder.encode("a\u{7f}").unwrap(), r"a{\textdelete}");
}

// -------------------------------------------------------------- normalizers

#[test]
fn nfc_composes_and_borrows_what_is_composed_already() {
    assert!(matches!(nfc("Caf\u{e9} \u{3b1}"), std::borrow::Cow::Borrowed(_)));
    assert!(matches!(nfc(""), std::borrow::Cow::Borrowed(_)));
    let owned = nfc("Cafe\u{301} A\u{30a}");
    assert!(matches!(owned, std::borrow::Cow::Owned(_)));
    assert_eq!(owned, "Caf\u{e9} \u{c5}");
}

#[test]
fn without_normalization_a_decomposed_letter_is_two_characters() {
    let composed = Encoder::new(small_table());
    assert_eq!(composed.encode("e\u{301}").unwrap(), r"\'e");

    let as_typed = Encoder::new(small_table()).with_normalizer(NoNormalization);
    // The combining accent is left to the unknown-char policy.
    assert_eq!(as_typed.encode("e\u{301}").unwrap(), "e\u{301}");
    let as_typed = as_typed.with_unknown_chars(UnknownCharPolicy::Fail);
    assert!(matches!(
        as_typed.encode("e\u{301}").unwrap_err(),
        EncodeError::UnknownChar { ch: '\u{301}', position: 1 }
    ));
}

// ---------------------------------------------------------- the ASCII path

#[test]
fn a_multi_character_rule_starting_at_an_ascii_character_still_fires() {
    let ellipsis = rule_fn(|input: RuleInput<'_>| {
        Ok(input
            .rest()
            .starts_with("...")
            .then(|| input.replace_prefix(3, r"\ldots", Hint::any_mode(r"\ldots"))))
    })
    .with_ascii_triggers(AsciiSet::of("."));
    let encoder = Encoder::new(RuleChain::new((ellipsis, small_table())));
    assert_eq!(encoder.encode("a...b").unwrap(), r"a{\ldots}b");
    assert_eq!(encoder.encode("a. b").unwrap(), "a. b");
}

#[test]
fn a_rule_is_not_consulted_outside_the_ascii_characters_it_declares() {
    // The rule would match every ASCII letter, but promises to match only at
    // `%`; the encoder takes it at its word for the others.
    let greedy = rule_fn(|input: RuleInput<'_>| {
        Ok(input.ch().is_ascii().then(|| input.replace_char("!", Hint::any_mode("!"))))
    })
    .with_ascii_triggers(AsciiSet::of("%"));
    let encoder = Encoder::new(greedy);
    assert_eq!(encoder.encode("ab%cd").unwrap(), "ab!cd");
}

#[test]
fn a_table_of_non_ascii_entries_triggers_on_no_ascii_character() {
    let table = small_table();
    assert_eq!(table.ascii_keys(), AsciiSet::EMPTY);
    let mut with_ascii = DynTable::new();
    with_ascii.insert('%', r"\%", Hint::text_only(r"\%"));
    assert_eq!(with_ascii.ascii_keys(), AsciiSet::of("%"));
}

#[test]
fn the_ascii_fast_path_gives_the_same_answer_as_a_rule_that_triggers_everywhere() {
    let text = "a%b\u{e9}\t\u{2014} ...";
    let rule = || {
        rule_fn(|input: RuleInput<'_>| {
            Ok(match input.ch() {
                '%' => Some(input.replace_char(r"\%", Hint::text_only(r"\%"))),
                '\u{e9}' => Some(input.replace_char(r"\'e", Hint::text_only(r"\'e"))),
                '\u{2014}' => {
                    Some(input.replace_char(r"\textemdash", Hint::text_only(r"\textemdash")))
                }
                _ => None,
            })
        })
    };
    let narrow = Encoder::new(rule().with_ascii_triggers(AsciiSet::of("%")));
    let wide = Encoder::new(rule().with_ascii_triggers(AsciiSet::ALL));
    assert_eq!(narrow.encode(text).unwrap(), wide.encode(text).unwrap());
    assert_eq!(narrow.encode(text).unwrap(), "a\\%b\\'e\t{\\textemdash} ...");
}

#[test]
fn ascii_sets_are_built_from_characters_ranges_and_predicates() {
    let lower = AsciiSet::from_fn(|byte| byte.is_ascii_lowercase());
    assert!(lower.contains(b'a') && !lower.contains(b'A'));
    // Nothing outside ASCII is ever in a set: the shift is guarded.
    assert!(!lower.contains(0x80) && !AsciiSet::ALL.contains(0xff));
    assert_eq!(AsciiSet::range(b'a'..=b'c'), AsciiSet::of("abc"));
    assert_eq!(AsciiSet::of("ab") | AsciiSet::of("bc"), AsciiSet::of("abc"));
    assert_eq!(AsciiSet::of("ab").union(AsciiSet::EMPTY), AsciiSet::of("ab"));
    assert!(AsciiSet::EMPTY.is_empty() && !AsciiSet::ALL.is_empty());
    // An empty range gives the empty set rather than panicking.
    let (start, end) = (5u8, 4u8);
    assert!(AsciiSet::range(start..=end).is_empty());
}

// ------------------------------------------------------------------- needs

#[test]
fn the_needs_of_a_value_reach_the_report_once() {
    let mut table = DynTable::new();
    table.insert_with_needs(
        '\u{1d7d9}',
        r"\UnxTBbold{1}",
        Hint::math_only(r"\UnxTBbold{1}"),
        BBOLD.clone(),
    );
    table.insert_with_needs('\u{44f}', r"\cyrya", Hint::text_only(r"\cyrya"), FONTENC.clone());
    let encoder = Encoder::new(table);

    let (out, report) = encoder.encode_with_report("\u{1d7d9}\u{1d7d9} \u{44f}").unwrap();
    assert_eq!(out, r"\ensuremath{\UnxTBbold{1}}\ensuremath{\UnxTBbold{1}} {\cyrya}");
    // Packages first, then snippets, each in first-seen order.
    assert_eq!(
        report.needs.chunks().map(|chunk| &*chunk.id).collect::<Vec<_>>(),
        ["amssymb", "fontenc-t2a", "bbold-alphabet"]
    );
}

#[test]
fn a_profile_built_at_run_time_reaches_the_report() {
    /// A rule that lends both its value and its profile from its own state,
    /// which a closure rule cannot do.
    #[derive(Debug)]
    struct Fractions {
        encoded: String,
        profile: Profile,
    }
    impl Rule for Fractions {
        fn apply<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a> {
            Ok((input.ch() == '\u{2153}').then(|| {
                input
                    .replace_char(&*self.encoded, Hint::text_only(r"\nicefrac{1}{3}"))
                    .with_needs(&self.profile)
            }))
        }

        fn ascii_triggers(&self) -> AsciiSet {
            AsciiSet::EMPTY
        }
    }

    let rule = Fractions {
        encoded: String::from(r"\nicefrac{1}{3}"),
        profile: Profile::new(vec![Chunk::package("nicefrac")]),
    };
    let (out, report) = Encoder::new(rule).encode_with_report("\u{2153}").unwrap();
    assert_eq!(out, r"\nicefrac{1}{3}");
    assert_eq!(report.needs.chunks().map(|c| &*c.id).collect::<Vec<_>>(), ["nicefrac"]);
}

#[test]
fn a_preamble_is_written_packages_first() {
    let mut needs = PreambleNeeds::new();
    assert!(needs.is_empty());
    needs.include(&BBOLD);
    needs.include(&BBOLD);
    needs.include(&FONTENC);
    assert!(!needs.is_empty());

    let mut preamble = String::new();
    needs.write_preamble(&mut preamble).unwrap();
    assert_eq!(
        preamble,
        concat!(
            "\\usepackage{amssymb}\n",
            "\\usepackage[T2A,T1]{fontenc}\n",
            "\\DeclareMathAlphabet{\\UnxTBbold}{U}{bbold}{m}{n}\n",
        )
    );
    assert_eq!(format!("{needs:?}"), r#"["amssymb", "fontenc-t2a", "bbold-alphabet"]"#);

    let mut other = PreambleNeeds::new();
    other.include(&AMSMATH);
    other.merge(&needs);
    assert_eq!(
        other.chunks().map(|c| &*c.id).collect::<Vec<_>>(),
        ["amsmath", "amssymb", "fontenc-t2a", "bbold-alphabet"]
    );
}

#[test]
fn a_chunk_states_what_it_is() {
    assert!(Chunk::package("amssymb").is_package());
    assert!(Chunk::package_with_options("fontenc-t2a", "fontenc", "T2A,T1").is_package());
    assert!(!Chunk::snippet("id", "text").is_package());
    assert_eq!(
        Chunk::package_with_options("fontenc-t2a", "fontenc", "T2A,T1").preamble,
        ChunkPreamble::PackageWithOptions("fontenc".into(), "T2A,T1".into())
    );
}

#[test]
fn a_preamble_needs_is_itself_a_reporter() {
    let mut table = DynTable::new();
    table.insert_with_needs('\u{44f}', r"\cyrya", Hint::text_only(r"\cyrya"), FONTENC.clone());
    let mut out = String::new();
    let mut needs = PreambleNeeds::new();
    Encoder::new(table).encode_into("\u{44f}", &mut out, &mut needs).unwrap();
    assert_eq!(needs.chunks().map(|c| &*c.id).collect::<Vec<_>>(), ["fontenc-t2a"]);
}

#[test]
fn a_profile_is_reported_again_only_when_it_is_not_the_one_last_reported() {
    /// A reporter that keeps every call, so that repeats show up.
    #[derive(Debug, Default)]
    struct EveryCall(Vec<String>);
    impl EncodeReporter for EveryCall {
        fn report_needs(&mut self, profile: &Profile) {
            self.0.push(profile.chunks()[0].id.to_string());
        }
    }

    let mut table = DynTable::new();
    table.insert_with_needs('\u{44f}', r"\cyrya", Hint::text_only(r"\cyrya"), FONTENC.clone());
    table.insert_with_needs('\u{2135}', r"\aleph", Hint::math_only(r"\aleph"), AMSMATH.clone());
    table.insert('\u{2014}', r"\textemdash", Hint::text_only(r"\textemdash"));
    let encoder = Encoder::new(table);

    // An immediate repeat is skipped; the same profile after another one is
    // reported again.
    let mut report = EveryCall::default();
    encoder
        .encode_into("\u{44f}\u{44f}\u{2135}\u{44f}", &mut String::new(), &mut report)
        .unwrap();
    assert_eq!(report.0, ["fontenc-t2a", "amsmath", "fontenc-t2a"]);

    // A value that needs nothing does not report, and so does not clear the
    // shortcut either.
    let mut report = EveryCall::default();
    encoder.encode_into("\u{44f}\u{2014}\u{44f}", &mut String::new(), &mut report).unwrap();
    assert_eq!(report.0, ["fontenc-t2a"]);

    // The shortcut is a local of one call: the next call reports afresh.
    let mut report = EveryCall::default();
    encoder.encode_into("\u{44f}", &mut String::new(), &mut report).unwrap();
    encoder.encode_into("\u{44f}", &mut String::new(), &mut report).unwrap();
    assert_eq!(report.0, ["fontenc-t2a", "fontenc-t2a"]);
}

// ------------------------------------------------------------------ output

#[test]
fn the_output_can_be_streamed_through_a_formatter() {
    /// A `fmt::Write` that keeps every piece it was given.
    #[derive(Default)]
    struct Pieces(Vec<String>);
    impl std::fmt::Write for Pieces {
        fn write_str(&mut self, s: &str) -> std::fmt::Result {
            self.0.push(s.to_string());
            Ok(())
        }
    }

    let mut out = FmtOut(Pieces::default());
    Encoder::new(small_table())
        .encode_into("Caf\u{e9} \u{2014}!", &mut out, &mut NoReport)
        .unwrap();
    assert_eq!(out.0 .0.concat(), r"Caf\'e {\textemdash}!");
    // The ASCII run is written in one piece, not character by character.
    assert_eq!(out.0 .0[0], "Caf");

    let mut plain = FmtOut(String::new());
    Encoder::new(small_table()).encode_into("\u{e9}", &mut plain, &mut NoReport).unwrap();
    assert_eq!(plain.0, r"\'e");
}

#[test]
fn an_output_that_fails_stops_the_encoding() {
    /// An output that refuses everything after the first piece.
    struct OnePiece(usize);
    impl OutBuffer for OnePiece {
        fn push_str(&mut self, _: &str) -> Result<(), untechxt::BoxError> {
            self.0 += 1;
            if self.0 > 1 {
                return Err("the output is full".into());
            }
            Ok(())
        }
    }
    let error = Encoder::new(small_table())
        .encode_into("ab\u{e9}", &mut OnePiece(0), &mut NoReport)
        .unwrap_err();
    assert!(matches!(error, EncodeError::Output(_)));
    assert_eq!(error.to_string(), "writing the output failed: the output is full");
}

#[test]
fn encoding_appends_to_the_output_and_the_report() {
    let encoder = Encoder::new(small_table());
    let mut out = String::from("already there: ");
    let mut report = EncodeReport::new();
    encoder.encode_into("\u{e9}", &mut out, &mut report).unwrap();
    encoder.encode_into(" \u{3b1}", &mut out, &mut report).unwrap();
    assert_eq!(out, r"already there: \'e \ensuremath{\alpha}");
}

#[test]
fn what_is_reported_does_not_change_what_is_written() {
    let text = "Caf\u{e9} \u{3b1} \u{3b2} \u{2014}";
    let encoder = Encoder::new(small_table());
    let (with_report, _) = encoder.encode_with_report(text).unwrap();
    assert_eq!(encoder.encode(text).unwrap(), with_report);
}

#[test]
fn an_error_hands_out_the_error_it_wraps() {
    use std::error::Error as _;

    /// An output that refuses everything.
    struct Refusing;
    impl OutBuffer for Refusing {
        fn push_str(&mut self, _: &str) -> Result<(), BoxError> {
            Err("the output is closed".into())
        }
    }

    let failing = rule_fn(|input: RuleInput<'_>| -> RuleResult<'_> {
        if input.ch() == '\u{e9}' {
            Err("the callback raised".into())
        } else {
            Ok(None)
        }
    });
    let error = Encoder::new(failing).encode("\u{e9}").unwrap_err();
    assert_eq!(error.source().unwrap().to_string(), "the callback raised");

    let error = Encoder::new(small_table())
        .encode_into("\u{e9}", &mut Refusing, &mut NoReport)
        .unwrap_err();
    assert_eq!(error.source().unwrap().to_string(), "the output is closed");

    // An unknown character wraps nothing.
    let error = Encoder::new(small_table())
        .with_unknown_chars(UnknownCharPolicy::Fail)
        .encode("\u{3b2}")
        .unwrap_err();
    assert!(error.source().is_none());
}

// ------------------------------------------------------------------ tables

#[test]
fn a_table_lookup_agrees_with_its_iteration_order() {
    let table = small_table();
    let listed: Vec<char> = table.iter().map(|(ch, _)| ch).collect();
    assert_eq!(listed, ['\u{e9}', '\u{3b1}', '\u{2014}', '\u{fb01}']);
    for (ch, entry) in table.iter() {
        assert_eq!(table.lookup(ch).unwrap().encoded, entry.encoded);
    }
    assert_eq!(table.len(), 4);
    assert!(!table.is_empty());
    assert!(table.lookup('a').is_none());
}

#[test]
fn inserting_the_same_character_twice_replaces_its_entry() {
    let mut table = DynTable::new();
    table.insert('%', r"\%", Hint::text_only(r"\%"));
    table.insert('%', r"\textpercent", Hint::text_only(r"\textpercent"));
    assert_eq!(table.len(), 1);
    assert_eq!(table.lookup('%').unwrap().encoded, r"\textpercent");
}

#[test]
fn a_user_table_becomes_a_rule_through_table_rule() {
    /// A table of one entry, of a user's own making.
    #[derive(Debug)]
    struct OneEntry;
    impl LookupTable for OneEntry {
        fn lookup(&self, ch: char) -> Option<untechxt::lookuptable::TableEntry<'_>> {
            (ch == '\u{2014}').then(|| untechxt::lookuptable::TableEntry {
                encoded: r"\textemdash",
                hint: Hint::text_only(r"\textemdash"),
                needs: Some(&AMSMATH),
            })
        }

        fn ascii_keys(&self) -> AsciiSet {
            AsciiSet::EMPTY
        }
    }
    let (out, report) =
        Encoder::new(TableRule(OneEntry)).encode_with_report("a\u{2014}").unwrap();
    assert_eq!(out, r"a{\textemdash}");
    assert_eq!(report.needs.chunks().map(|c| &*c.id).collect::<Vec<_>>(), ["amsmath"]);
}

// ------------------------------------------------------------------- debug

#[test]
fn debug_forms_name_what_they_are() {
    assert_eq!(format!("{:?}", rule_fn(|_: RuleInput<'_>| Ok(None))), "RuleFn(..)");
    assert_eq!(format!("{:?}", UnknownCharPolicy::Keep), "Keep");
    assert_eq!(format!("{:?}", UnknownCharPolicy::callback(unknown_unihex)), "Callback(..)");
    assert_eq!(format!("{:?}", AsciiSet::ALL), "AsciiSet::ALL");
    assert_eq!(format!("{:?}", AsciiSet::EMPTY), "AsciiSet::EMPTY");
    assert_eq!(format!("{:?}", AsciiSet::of("%&")), r#"AsciiSet("%&")"#);
    let mut shown = String::new();
    write!(shown, "{:?}", AsciiSet::range(9..=10)).unwrap();
    assert_eq!(shown, r#"AsciiSet("\x09\x0a")"#);
}

#[cfg(feature = "std")]
#[test]
fn the_output_can_be_streamed_to_an_io_writer() {
    use untechxt::outbuffer::IoOut;

    let mut out = IoOut(Vec::new());
    Encoder::new(small_table())
        .encode_into("Caf\u{e9}", &mut out, &mut NoReport)
        .unwrap();
    assert_eq!(String::from_utf8(out.0).unwrap(), r"Caf\'e");
}
