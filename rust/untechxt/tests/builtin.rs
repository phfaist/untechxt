//! The builtin tables: that the data compiled, that the three tables
//! hold the parts of it they claim to hold, and that an encoder over them says
//! what a document needs. The suite ported from the initial port, with the
//! conformance golden, is in `latexencode.rs`.

use std::collections::BTreeSet;

use untechxt::builtin::default_table::ENTRIES;
use untechxt::builtin::needs_profiles::{AMSSYMB, PROFILES};
use untechxt::builtin::{
    BuiltinTable, DEFAULT_TABLE, DEFAULT_TABLE_ASCII_SPECIALS, DEFAULT_TABLE_NON_ASCII,
};
use untechxt::lookuptable::{DynTable, LookupTable};
use untechxt::preamble::{ChunkPreamble, Engine, Profile};
use untechxt::protection::{ReplacementProtectionHint as Hint, ValueTermination};
use untechxt::rule::{Rule, RuleChain};
use untechxt::statictable::ProfileIndex;
use untechxt::{default_rules, encode, DefaultRules, Encoder};

/// The 13 ASCII characters the builtin table has entries for.
const ASCII_ENTRIES: &str = "\"#$%&<>\\^_{}~";

#[test]
fn the_table_holds_every_entry() {
    assert_eq!(DEFAULT_TABLE.len(), 1549);
    assert_eq!(DEFAULT_TABLE.len(), ENTRIES.len());
    assert!(!DEFAULT_TABLE.is_empty());
}

#[test]
fn lookup_agrees_with_iter_over_the_whole_table() {
    let mut seen = 0;
    let mut previous = None;
    for (ch, entry) in DEFAULT_TABLE.iter() {
        assert_eq!(DEFAULT_TABLE.lookup(ch), Some(entry), "{ch:?}");
        assert!(previous < Some(ch), "the entries must ascend: {previous:?} then {ch:?}");
        previous = Some(ch);
        seen += 1;
    }
    assert_eq!(seen, ENTRIES.len());
}

#[test]
fn iter_yields_the_source_entries_in_order() {
    for ((ch, entry), &(source_ch, encoded, mode, profile)) in DEFAULT_TABLE.iter().zip(ENTRIES) {
        assert_eq!(ch, source_ch);
        assert_eq!(entry.encoded, encoded);
        // The mode is the source column, the termination is read off the
        // spelling, and the profile index is the position in `PROFILES` —
        // index 0 answering `None`.
        assert_eq!(
            entry.hint,
            Hint::Value { mode, termination: ValueTermination::inspect(encoded) },
            "{ch:?}"
        );
        match entry.needs {
            None => assert_eq!(profile, ProfileIndex::NONE, "{ch:?} names a profile but has none"),
            Some(needs) => assert!(
                std::ptr::eq(needs, &PROFILES[profile.0 as usize]),
                "{ch:?} resolves to a profile other than the one it names"
            ),
        }
    }
}

#[test]
fn the_ascii_specials_are_the_thirteen_and_nothing_else() {
    for ch in ASCII_ENTRIES.chars() {
        assert!(
            DEFAULT_TABLE_ASCII_SPECIALS.lookup(ch).is_some(),
            "{ch:?} must be an ASCII special"
        );
    }
    let specials: BTreeSet<char> = ASCII_ENTRIES.chars().collect();
    assert_eq!(specials.len(), 13);
    for byte in 0u8..128 {
        let ch = byte as char;
        assert_eq!(
            DEFAULT_TABLE_ASCII_SPECIALS.lookup(ch).is_some(),
            specials.contains(&ch),
            "{ch:?} is wrongly in or out of DEFAULT_TABLE_ASCII_SPECIALS"
        );
    }
    assert_eq!(DEFAULT_TABLE_ASCII_SPECIALS.lookup('\u{e9}'), None);
}

#[test]
fn the_ascii_specials_are_the_ascii_entries_of_the_full_table() {
    // A table compiled by itself, from the head of the same source list: it
    // must say what `DEFAULT_TABLE` says, profile reference included.
    assert_eq!(DEFAULT_TABLE_ASCII_SPECIALS.len(), 13);
    assert!(!DEFAULT_TABLE_ASCII_SPECIALS.is_empty());
    assert!(DEFAULT_TABLE_ASCII_SPECIALS
        .iter()
        .eq(DEFAULT_TABLE.iter().filter(|(ch, _)| ch.is_ascii())));
    for byte in 0u8..128 {
        let ch = byte as char;
        assert_eq!(DEFAULT_TABLE_ASCII_SPECIALS.lookup(ch), DEFAULT_TABLE.lookup(ch), "{ch:?}");
    }
    assert_eq!(DEFAULT_TABLE_ASCII_SPECIALS.ascii_keys(), DEFAULT_TABLE.ascii_keys());
    // Block 0 reaches up to U+00FF, and its upper half is not ASCII.
    for code_point in 0x80u32..0x100 {
        let ch = char::from_u32(code_point).unwrap();
        assert_eq!(DEFAULT_TABLE_ASCII_SPECIALS.lookup(ch), None, "{ch:?}");
    }
}

#[test]
fn the_non_ascii_table_answers_for_no_ascii_character() {
    for byte in 0u8..128 {
        assert_eq!(DEFAULT_TABLE_NON_ASCII.lookup(byte as char), None, "{byte:#04x}");
    }
    assert_eq!(DEFAULT_TABLE_NON_ASCII.lookup('\u{e9}').unwrap().encoded, r"\'e");
    assert!(DEFAULT_TABLE_NON_ASCII.ascii_keys().is_empty());
    assert!(DEFAULT_TABLE_NON_ASCII.ascii_triggers().is_empty());
}

#[test]
fn the_non_ascii_table_is_the_full_table_without_its_ascii_entries() {
    assert_eq!(
        DEFAULT_TABLE_NON_ASCII.len(),
        DEFAULT_TABLE.len() - DEFAULT_TABLE_ASCII_SPECIALS.len()
    );
    assert!(!DEFAULT_TABLE_NON_ASCII.is_empty());
    assert_eq!(DEFAULT_TABLE_NON_ASCII.iter().count(), DEFAULT_TABLE_NON_ASCII.len());
    let expected: Vec<_> = DEFAULT_TABLE.iter().filter(|(ch, _)| !ch.is_ascii()).collect();
    assert_eq!(DEFAULT_TABLE_NON_ASCII.iter().collect::<Vec<_>>(), expected);
    for (ch, entry) in DEFAULT_TABLE_NON_ASCII.iter() {
        assert_eq!(DEFAULT_TABLE_NON_ASCII.lookup(ch), Some(entry), "{ch:?}");
    }
    // As a rule, the table leaves the LaTeX code of the input unchanged.
    let encoder = Encoder::new(&DEFAULT_TABLE_NON_ASCII);
    assert_eq!(encoder.encode("\\emph{Caf\u{e9}} & 100%").unwrap(), r"\emph{Caf\'e} & 100%");
}

#[test]
fn the_three_tables_have_one_type_and_can_be_chosen_at_run_time() {
    let tables: [&'static BuiltinTable; 3] =
        [&DEFAULT_TABLE, &DEFAULT_TABLE_NON_ASCII, &DEFAULT_TABLE_ASCII_SPECIALS];
    let outputs: Vec<_> =
        tables.iter().map(|table| Encoder::new(*table).encode("\u{e9}&").unwrap()).collect();
    assert_eq!(outputs, [r"\'e\&", r"\'e&", "\u{e9}\\&"]);
}

#[test]
fn the_ascii_keys_of_the_full_table_are_the_thirteen() {
    let keys = DEFAULT_TABLE.ascii_keys();
    for byte in 0u8..128 {
        assert_eq!(
            keys.contains(byte),
            ASCII_ENTRIES.contains(byte as char),
            "{:?}",
            byte as char
        );
    }
}

#[test]
fn a_line_of_text_encodes_the_way_the_entries_and_the_protection_say() {
    let encoder = Encoder::new(&DEFAULT_TABLE);
    // `\'e` and `fi` terminate themselves; `\textemdash`, `\alpha`, `\leq`
    // and `\beta` end with a named macro, and the three math ones are wrapped
    // by `\ensuremath{…}`, which terminates them.
    assert_eq!(
        encoder.encode("Caf\u{e9} \u{2014} \u{3b1} \u{2264} \u{3b2}").unwrap(),
        r"Caf\'e {\textemdash} \ensuremath{\alpha} \ensuremath{\leq} \ensuremath{\beta}"
    );
}

#[test]
fn what_a_character_needs_ends_up_in_the_report() {
    let encoder = Encoder::new(&DEFAULT_TABLE);
    // U+2102 DOUBLE-STRUCK CAPITAL C is spelled `\mathbb{C}`, of `amssymb`.
    let (encoded, report) = encoder.encode_with_report("\u{2102}").unwrap();
    assert_eq!(encoded, r"\ensuremath{\mathbb{C}}");
    let ids: Vec<&str> = report.needs.chunks().map(|chunk| chunk.id()).collect();
    assert_eq!(ids, ["amssymb"]);

    let mut preamble = String::new();
    report.needs.write_preamble(&mut preamble).unwrap();
    assert_eq!(preamble, "\\usepackage{amssymb}\n");

    // And the entry's own profile is the one the constant names.
    let needs = DEFAULT_TABLE.lookup('\u{2102}').unwrap().needs.unwrap();
    assert!(std::ptr::eq(needs, &PROFILES[AMSSYMB.0 as usize]));
}

#[test]
fn a_snippet_profile_reports_its_package_first() {
    // U+0482 CYRILLIC THOUSANDS SIGN needs the `T2D` encoding and the symbol
    // declared out of it: the package comes before the declarations.
    let encoder = Encoder::new(&DEFAULT_TABLE);
    let (_, report) = encoder.encode_with_report("\u{0482}").unwrap();
    let ids: Vec<&str> = report.needs.chunks().map(|chunk| chunk.id()).collect();
    assert_eq!(ids, ["fontenc-t2d", "cyrillic-thousands"]);
}

#[test]
fn every_profile_index_the_entries_name_is_in_range() {
    // The macro checks this at compile time; the assertion here confirms that
    // it checked this array.
    for &(ch, _, _, profile) in ENTRIES {
        assert!(
            (profile.0 as usize) < PROFILES.len(),
            "{ch:?} names profile {} of {}",
            profile.0,
            PROFILES.len()
        );
    }
    assert_eq!(PROFILES.len(), 21);
    assert!(PROFILES[0].is_empty());
}

#[test]
fn no_two_builtin_profiles_hold_the_same_chunk_set() {
    let sets: Vec<BTreeSet<&str>> = PROFILES
        .iter()
        .map(|profile| profile.chunks().iter().map(|chunk| chunk.id()).collect())
        .collect();
    for (i, one) in sets.iter().enumerate() {
        for (j, other) in sets.iter().enumerate().skip(i + 1) {
            assert_ne!(one, other, "profiles {i} and {j} hold the same chunks");
        }
    }
}

#[test]
fn the_chunk_identifiers_are_distinct_and_name_one_chunk_each() {
    let mut chunks: Vec<(&str, String)> = Vec::new();
    for profile in &PROFILES {
        for chunk in profile.chunks() {
            let form = format!("{:?}", chunk.cases());
            match chunks.iter().find(|(id, _)| *id == chunk.id()) {
                Some((_, held)) => assert_eq!(held, &form, "{} means two things", chunk.id()),
                None => chunks.push((chunk.id(), form)),
            }
        }
    }
    assert_eq!(chunks.len(), 20);
}

#[test]
fn every_unxt_command_the_entries_write_is_declared_by_the_entry_s_own_profile() {
    /// The declarations of the snippet chunks of one profile under `engine`,
    /// run together.
    fn declarations(profile: Option<&Profile>, engine: Engine) -> String {
        profile.into_iter().flat_map(|profile| profile.chunks()).fold(
            String::new(),
            |mut all, chunk| {
                if let Some(ChunkPreamble::Snippet(text)) = chunk.preamble_for(engine) {
                    all.push_str(text);
                    all.push('\n');
                }
                all
            },
        )
    }

    let mut used: BTreeSet<String> = BTreeSet::new();
    for (ch, entry) in DEFAULT_TABLE.iter() {
        if !entry.encoded.contains("\\UnxT") {
            continue;
        }
        // The declaration must be there under every engine, since a chunk may
        // be nothing under some of them.
        for engine in [Engine::PdfLatex, Engine::LuaLatex, Engine::XeLatex] {
            let declared = declarations(entry.needs, engine);
            let mut rest = entry.encoded;
            while let Some(at) = rest.find("\\UnxT") {
                let tail = &rest[at + 1..];
                let end =
                    tail.find(|c: char| !c.is_ascii_alphabetic()).unwrap_or(tail.len());
                let command = &tail[..end];
                // The chunk that declares the command must be one the entry's
                // own profile holds: a profile that leaves it out is a silent
                // bug.
                assert!(
                    declared.contains(&format!("{{\\{command}}}")),
                    "{ch:?} writes \\{command}, which its profile does not declare \
                     under {engine:?}"
                );
                used.insert(command.to_string());
                rest = &tail[end..];
            }
        }
    }
    assert_eq!(used.len(), 36);
}

#[test]
fn the_debug_forms_name_the_tables() {
    assert!(format!("{DEFAULT_TABLE:?}").starts_with("BuiltinTable { len: 1549"));
    assert!(format!("{DEFAULT_TABLE_NON_ASCII:?}").starts_with("BuiltinTable { len: 1536"));
    assert!(format!("{DEFAULT_TABLE_ASCII_SPECIALS:?}").starts_with("BuiltinTable { len: 13"));
}

// ----------------------------------------------------------- default rules

#[test]
fn the_default_rules_encode_the_way_the_default_table_does() {
    let text = "Caf\u{e9} \u{2014} 100% & \u{3b1} \u{2264} \u{3b2} \u{1d7d9} \u{e18}";
    let with_rules = Encoder::new(default_rules()).encode_with_report(text).unwrap();
    let with_table = Encoder::new(&DEFAULT_TABLE).encode_with_report(text).unwrap();
    assert_eq!(with_rules.0, with_table.0);
    assert_eq!(
        with_rules.1.needs.chunks().collect::<Vec<_>>(),
        with_table.1.needs.chunks().collect::<Vec<_>>()
    );
    assert_eq!(with_rules.1.unknown_chars, with_table.1.unknown_chars);
    assert_eq!(encode(text), with_rules.0);
    assert_eq!(default_rules().ascii_triggers(), DEFAULT_TABLE.ascii_triggers());
}

#[test]
fn the_default_rules_are_an_opaque_copyable_thread_safe_rule() {
    fn assert_traits<T: Rule + Copy + Send + Sync + 'static>(_: T) {}
    assert_traits(default_rules());
    assert_eq!(format!("{:?}", default_rules()), "DefaultRules { .. }");
    assert!(std::mem::size_of::<DefaultRules>() <= std::mem::size_of::<usize>());
}

#[test]
fn an_encoder_type_without_arguments_is_the_default_rules_encoder() {
    struct Holder {
        encoder: Encoder,
    }
    let holder = Holder { encoder: Encoder::new(default_rules()) };
    let same: &Encoder<DefaultRules> = &holder.encoder;
    assert_eq!(same.encode("\u{e9}").unwrap(), r"\'e");
}

#[test]
fn the_default_rules_are_a_member_of_a_chain_like_any_rule() {
    let overrides =
        DynTable::new().with_entry('%', r"\textpercent", Hint::text_only(r"\textpercent"));
    let encoder = Encoder::new(RuleChain::new((overrides, default_rules())));
    assert_eq!(encoder.encode("100% \u{e9}").unwrap(), r"100{\textpercent} \'e");
}
