//! Unicode-to-LaTeX encoder.  This library turns text such as `Café — α ≤ β`
//! into the LaTeX source `Caf\'e {\textemdash} \ensuremath{\alpha}
//! \ensuremath{\leq} \ensuremath{\beta}`.
//!
//! LaTeX source typically only tolerates a restricted set of characters in its
//! input, and some of those characters have a special meaning (notably the
//! escape character `\\`). This library provides an *encoding* of an entire
//! input string, meaning that it replaces each character that LaTeX either
//! rejects or would take special action on, by some LaTeX code that displays
//! that character.
//!
//! Quick start: Head to the [`encode`] function, which runs the encoder with
//! some reasonable default settings and a built-in symbol encoding table.
//!
//! ```
//! use untechxt::encode;
//!
//! assert_eq!(encode("Café — naïve"), r#"Caf\'e {\textemdash} na\"ive"#);
//! assert_eq!(encode("100% & more"), r"100\% \& more");
//! ```
//!
//! The library is highly extensible and flexible: You may define custom rules
//! for encodings, including rules that match several characters of input.  The
//! output can be assembled as a string or directly written to an I/O
//! buffer. Input is unicode-normalized by default, but a custom normalization
//! step can replace the default behavior.
//!
//! The crate is `#![no_std]` with [`alloc`], and its one dependency is
//! `unicode-normalization`. Its modular design aims to yield small compiled
//! artifacts that only pull in the parts of the library that the user needs.
//!
//! # The encoder
//!
//! An [`Encoder`] holds a [`Rule`], a [`ReplacementProtection`] strategy, an
//! [`InputNormalizer`] and an [`UnknownCharPolicy`]. It is built once, encodes
//! any number of strings, is immutable, and can be shared across threads
//! provided the rules are thread-safe.
//!
//! - A *rule* specifies how an input character, or an input substring, is
//!   mapped to a LaTeX encoded value.  The [`default_rules`] function returns
//!   the default rules of the crate, which use a built-in symbol encoding
//!   table.  A special rule type, [`RuleChain`], tries several rules in order
//!   until the first match.  Use a [`RuleChain`] whenever the encoder should
//!   apply multiple rules.
//!
//! - *Replacement protection* refers to additional syntax applied to the LaTeX
//!   encoded symbol to ensure the generated LaTeX code is valid.  For instance,
//!   the encoder might replace `'~'` by `'\textasciitilde'`; if no further
//!   processing happened, the text `'~user'` would be encoded incorrectly as
//!   `'\textasciitildeuser'`. Standard replacement protection strategies ensure
//!   that such symbols are represented for instance as `{\textasciitilde}`,
//!   which composes correctly with surrounding strings.
//!
//! - Input normalization: Preprocessing applied to the string before applying
//!   the rules; by default, a unicode normalize step.
//!
//! - *Unknown char policy*: How to handle a non-ASCII character or
//!   non-printable character in the input for which the rule didn't apply (or
//!   for which none of the rules of a rule chain applied).  By default, the
//!   character is left in the output. Other possible behaviors include: Fail
//!   with an error, replace by a fixed string, provide a custom callback
//!   function.
//!
//! Beyond reporting the encoded string, the encoder can yield a *report*
//! associated with the encoding. The report contains information about which
//! LaTeX packages or preamble definitions should be included in a document that
//! uses the encoded string, to ensure the used LaTeX commands are properly
//! defined and/or relevant fonts are loaded. The report may also contain
//! information about which unknown characters were encountered.
//!
//! The encoder can be invoked in several ways:
//!
//! - The method [`Encoder::encode`] takes a string and returns a new string,
//!   with no side effects.
//!
//! - The method [`Encoder::encode_with_report`] works like [`Encoder::encode`],
//!   but it also returns an [`EncodeReport`] with information about, for
//!   instance, LaTeX packages needed and/or additional preamble definitions
//!   that are required.
//!
//! - The method [`Encoder::encode_into`] is the lower-level entry point. It
//!   takes a generic type for where to write the encoded pieces to (see
//!   [`OutBuffer`]), and a generic type for where to send information about
//!   required LaTeX packages/preamble definitions and encountered unknown
//!   characters (see [`EncodeReporter`]).
//!
//! ```
//! use untechxt::{default_rules, Encoder};
//!
//! let encoder = Encoder::new(default_rules());
//! let (body, report) = encoder.encode_with_report("𝟙 and ⅓").unwrap();
//! assert_eq!(body, r"\ensuremath{\mathds{1}} and \nicefrac{1}{3}");
//!
//! // `\mathds` is the package `dsfont`'s and `\nicefrac` the package
//! // `nicefrac`'s: the report knows, and writes the preamble that loads them.
//! let mut preamble = String::new();
//! report.needs.write_preamble(&mut preamble).unwrap();
//! assert_eq!(preamble, "\\usepackage{dsfont}\n\\usepackage{nicefrac}\n");
//! ```
//!
//! # Rules
//!
//! A [`Rule`] is offered a position in the input — a [`RuleInput`]: the whole
//! normalized text, the byte offset, and the character there — and answers
//! either with an [`EncodedReplacement`] (how many bytes it consumed, the
//! LaTeX that replaces them, how that LaTeX must be protected, and what it
//! needs in the preamble) or with "not mine". It may also fail, which stops
//! the encoding with [`EncodeError::Rule`]; that is how a rule driven from
//! another language passes an exception on.
//!
//! [`RuleChain`] is several rules as one. They are tried in order at every
//! position and the first match wins. A custom rule placed before a
//! lookup-table rule therefore overrides information from the lookup table,
//! while a custom rule placed after a lookup table can be used to fill in
//! entries the table lacks.  No further rule is tried after a match.  If a user
//! wants smarter matching, such as seeking the longest match, then the rules
//! must be reordered and/or written such that first-match-wins gives the
//! desired behavior. The members are a (static) tuple of up to twelve rules,
//! which the compiler may unroll and inline and which keeps each member's own
//! type; an array; or a [`Vec`](alloc::vec::Vec) — [`DynRuleChain`] and
//! [`LocalDynRuleChain`] are the chains of boxed rules that a configuration
//! file or a language binding assembles at run time.
//!
//! Rules come in three kinds.
//!
//! - **Tables.** A [`LookupTable`] answers for single characters. The
//!   [`default_rules`] function returns the default rules, which use the
//!   builtin lookup table [`DEFAULT_TABLE`] and provide reasonable encoding
//!   defaults. The [`builtin`] module contains further builtin tables: the
//!   table [`DEFAULT_TABLE_NON_ASCII`] only contains the non-ASCII section of
//!   the default table, and the table [`DEFAULT_TABLE_ASCII_SPECIALS`] is a
//!   small table that only encodes the few printable ASCII characters that
//!   have special meaning for LaTeX.  [`DynTable`] is a table built at run
//!   time; [`compile_static_table!`] compiles a table of your own at compile
//!   time into one of the layouts of the [`statictable`] module. The crate's
//!   own tables are rules already. Create your rule from your custom
//!   [`LookupTable`] with [`TableRule`].
//! - **Closures**, through [`rule_fn`]. A closure rule may hand out owned
//!   strings, literals, `&'static Profile`s and slices of the input, but it
//!   cannot lend out its own captures.
//! - **Anything else that implements [`Rule`]**: a struct that reads several
//!   characters ahead, that lends values from its own state, or that calls
//!   into another language. [`Rule`] requires only
//!   [`Debug`](core::fmt::Debug) — not `Send` or `Sync`, so that a rule
//!   holding a JavaScript callback or an [`Rc`](alloc::rc::Rc) is possible;
//!   an [`Encoder`] is `Send` and `Sync` exactly when its rule(s) are.
//!
//! ```
//! use untechxt::lookuptable::DynTable;
//! use untechxt::protection::ReplacementProtectionHint as Hint;
//! use untechxt::rule::{rule_fn, AsciiSet, RuleChain, RuleInput};
//! use untechxt::{default_rules, Encoder};
//!
//! // A table ahead of the defaults overrides one character; a closure after
//! // them reads a sequence of characters, which a table cannot.
//! let overrides =
//!     DynTable::new().with_entry('%', r"\textpercent", Hint::text_only(r"\textpercent"));
//! let ellipsis = rule_fn(|input: RuleInput<'_>| {
//!     Ok(input
//!         .rest()
//!         .starts_with("...")
//!         .then(|| input.replace_prefix(3, r"\ldots", Hint::any_mode(r"\ldots"))))
//! })
//! .with_ascii_triggers(AsciiSet::of("."));
//!
//! let encoder = Encoder::new(
//!     RuleChain::new((overrides, default_rules(), ellipsis))
//! );
//! assert_eq!(encoder.encode("100%... é").unwrap(), r"100{\textpercent}{\ldots} \'e");
//! ```
//!
//! # Protection, and LaTeX's two modes
//!
//! An encoded value cannot simply be pasted into the output: `\textemdash`
//! followed directly by a letter would read as a longer command name, and
//! `\alpha` is valid in mathematics alone. The rule therefore states what its
//! value is, through a required and authoritative
//! [`ReplacementProtectionHint`] — the encoder never inspects a value behind
//! its rule's back:
//!
//! - [`DoNotProtect`]: write the value exactly as it is, the rule vouching
//!   for it in its context. A rule that passes existing LaTeX through uses
//!   this.
//! - [`Value`]: the [`ValueMode`] the value is valid in ([`TextOnly`],
//!   [`MathOnly`] or [`AnyMode`]) and its [`ValueTermination`] — whether
//!   arbitrary text may follow it directly, or whether it ends with a named
//!   macro. A rule that does not know reads the termination off the value's
//!   form with [`ValueTermination::inspect`], which is what
//!   [`ReplacementProtectionHint::text_only`] and its two companions do, at
//!   compile time in a static table.
//!
//! The mode lives in the hint and not in the value: a table entry holds the
//! bare `\alpha` marked [`MathOnly`], not `\ensuremath{\alpha}`, so that math
//! output writes it as it is and text output wraps it once.
//!
//! What is written around the value is then the encoder's
//! [`ReplacementProtection`] strategy's business. Protection is stateless:
//! each value is protected as it comes, with no lookahead and no memory of
//! what was written before. [`StandardProtection`] is the strategy the crate
//! offers, and every one of its fields is public and settable at run time:
//!
//! - [`output_mode`], which of LaTeX's two modes the output is going into. A
//!   value whose mode does not match is wrapped by [`math_wrap`]
//!   (`\ensuremath{…}`) or by [`text_wrap`]; such a [`ModeWrapper`] may
//!   itself need something in the preamble, which is why the reporter reaches
//!   the strategy too.
//! - [`protect_names`], what is written around a value that ends with a named
//!   macro: [`BracesAround`] (`{\textemdash}`, the default and the safe choice
//!   in text mode), `BracesAfter` (`\textemdash{}`), `SpaceAfterMacroName`
//!   (`\pm `, the default of [`math_mode`], where spaces are ignored, and
//!   unsafe in text mode, where TeX would swallow a space of the input), or
//!   `NoProtection`.
//!
//! ```
//! use untechxt::protection::{BracesAroundAll, StandardProtection};
//! use untechxt::{default_rules, Encoder};
//!
//! // Text output: a math value is wrapped once, and a value that ends with a
//! // named macro is braced.
//! let text = Encoder::new(default_rules());
//! assert_eq!(text.encode("α≤β").unwrap(),
//!            r"\ensuremath{\alpha}\ensuremath{\leq}\ensuremath{\beta}");
//! assert_eq!(text.encode("—").unwrap(), r"{\textemdash}");
//!
//! // Math output: the same values go in bare, ended by a space; a text value
//! // is the one that has to be wrapped.
//! let math = Encoder::new(default_rules()).with_protection(
//!     StandardProtection::math_mode()
//! );
//! assert_eq!(math.encode("α≤β").unwrap(), r"\alpha \leq \beta ");
//! assert_eq!(math.encode("é").unwrap(), r"\textnormal{\'e}");
//!
//! // A strategy of its own: braces around every value.
//! let compat = Encoder::new(default_rules()).with_protection(
//!     BracesAroundAll(StandardProtection::text_mode())
//! );
//! assert_eq!(compat.encode("Café — α").unwrap(),
//!            r"Caf{\'e} {\textemdash} {\ensuremath{\alpha}}");
//! ```
//!
//! [`BracesAroundAll`] braces every value, whatever its hint says — this is
//! pylatexenc's `'braces-all'` mode. It is not a setting of
//! [`StandardProtection`] but a strategy of its own, written with public API
//! alone, and so it doubles as the example of how to write one: implement
//! [`ReplacementProtection`], read the value and its hint off the
//! [`ProtectInput`], and write to the [`OutBuffer`] you are handed.
//!
//! # Characters no rule knows
//!
//! A character is unknown when no rule matched it and it is not printable
//! ASCII (`0x20..=0x7E`) or one of `\n`, `\r` and `\t`. It is reported to the
//! [`EncodeReporter`] whatever happens next, and then the encoder's
//! [`UnknownCharPolicy`] says what is written for it:
//! [`Keep`](UnknownCharPolicy::Keep), the default, the character itself;
//! [`Ignore`](UnknownCharPolicy::Ignore) nothing;
//! [`Fail`](UnknownCharPolicy::Fail) stops with
//! [`EncodeError::UnknownChar`];
//! [`ReplaceWith`](UnknownCharPolicy::ReplaceWith) a fixed text; and
//! [`Callback`](UnknownCharPolicy::Callback) whatever a function answers —
//! [`unknown_unihex`] spells the code point out. None of that output is
//! protected and none of it carries preamble needs: a policy is a last
//! resort, and anything richer is a rule at the end of the chain.
//!
//! ```
//! use untechxt::{default_rules, unknown_unihex, Encoder, UnknownCharPolicy};
//!
//! // The Thai letter is in no builtin entry: it is kept, and reported.
//! let encoder = Encoder::new(default_rules());
//! let (out, report) = encoder.encode_with_report("ธ").unwrap();
//! assert_eq!(out, "ธ");
//! assert!(report.unknown_chars.contains(&'ธ'));
//!
//! let spelled_out = Encoder::new(default_rules()).with_unknown_chars(
//!     UnknownCharPolicy::callback(unknown_unihex)
//! );
//! assert_eq!(spelled_out.encode("ธ").unwrap(),
//!            r"\ensuremath{\langle}\texttt{U+0E18}\ensuremath{\rangle}");
//! ```
//!
//! # What the output needs in the preamble
//!
//! LaTeX such as `\mathds{1}` prints nothing unless the document loads the
//! package `dsfont`. A [`Chunk`] is one piece of preamble — a package with
//! the options it is loaded with, or a block of declarations no package makes
//! ([`ChunkPreamble`]) — under a stable identifier, and the same identifier
//! always means the same chunk. A [`Profile`] is the set of chunks that one
//! encoded value needs, which a rule hands out with its value
//! ([`EncodedReplacement::with_needs`]). There is no registry and no
//! identifier space to share out: a profile is a plain reference to something
//! the rule owns, so tables from unrelated crates cannot clash.
//!
//! The encoder passes each profile to an [`EncodeReporter`]. [`EncodeReport`]
//! keeps the needs and the unknown characters, [`PreambleNeeds`] the needs
//! alone, and [`NoReport`] nothing at all — under [`NoReport`] and a static
//! chain the compiler can remove the whole needs machinery from the loop. A
//! [`PreambleNeeds`] holds one copy of each distinct chunk, and writes the
//! preamble out with [`PreambleNeeds::write_preamble`], every package before
//! every block of declarations. It is a reporter itself, so the fragments of
//! one document can all report into one.
//!
//! ```
//! use untechxt::preamble::PreambleNeeds;
//! use untechxt::{default_rules, Encoder};
//!
//! let encoder = Encoder::new(default_rules());
//! let mut body = String::new();
//! let mut needs = PreambleNeeds::new();
//! for fragment in ["𝟙 ", "and ⅓"] {
//!     encoder.encode_into(fragment, &mut body, &mut needs).unwrap();
//! }
//! assert_eq!(body, r"\ensuremath{\mathds{1}} and \nicefrac{1}{3}");
//!
//! let mut preamble = String::new();
//! needs.write_preamble(&mut preamble).unwrap();
//! assert_eq!(preamble, "\\usepackage{dsfont}\n\\usepackage{nicefrac}\n");
//! ```
//!
//! ## LaTeX engines
//!
//! The encoded LaTeX is the same under every LaTeX engine, but the preamble
//! that it needs can differ between pdfLaTeX, LuaLaTeX and XeLaTeX, because
//! these engines handle fonts differently. For example, the Cyrillic letters
//! need the font encoding `T2A`. The package `fontenc` loads `T2A` beside the
//! font encoding of the document itself, which is `T1` under pdfLaTeX and `TU`
//! under LuaLaTeX and XeLaTeX. A preamble that names `T1` still compiles under
//! LuaLaTeX, but LuaLaTeX then drops the Unicode characters that are typed
//! directly into the document, and reports no error.
//!
//! A [`Chunk`] can therefore be a different piece of preamble under different
//! engines. It holds a list of cases (see [`ChunkCase`]), and each case applies
//! to a set of engines (see [`EngineSet`]). A chunk needs nothing under an
//! engine that none of its cases applies to. The builtin chunks already make
//! these distinctions, and custom chunks can make them too.
//!
//! You choose the engine when you write the preamble, in one of two ways:
//!
//! - The method [`PreambleNeeds::write_preamble`] takes no engine and writes
//!   a preamble that compiles under every engine. Where the engines need
//!   different things, the preamble loads the package `iftex` and tests which
//!   engine is running. Use this method when you do not know which engine
//!   will compile the document.
//! - The method [`PreambleNeeds::write_preamble_for`] takes an [`Engine`] and
//!   writes the preamble for that engine alone, without any test.
//!
//! ```
//! use untechxt::preamble::Engine;
//! use untechxt::{default_rules, Encoder};
//!
//! let encoder = Encoder::new(default_rules());
//! let (_, report) = encoder.encode_with_report("я").unwrap();
//! let needs = report.needs;
//!
//! let mut for_lualatex = String::new();
//! needs.write_preamble_for(Engine::LuaLatex, &mut for_lualatex).unwrap();
//! assert_eq!(for_lualatex, "\\usepackage[T2A,TU]{fontenc}\n");
//!
//! let mut for_every_engine = String::new();
//! needs.write_preamble(&mut for_every_engine).unwrap();
//! assert_eq!(for_every_engine, concat!(
//!     "\\usepackage{iftex}\n",
//!     "\\iftutex\n",
//!     "\\usepackage[T2A,TU]{fontenc}\n",
//!     "\\else\n",
//!     "\\usepackage[T2A,T1]{fontenc}\n",
//!     "\\fi\n",
//! ));
//! ```
//!
//! # Normalization
//!
//! The input goes through an [`InputNormalizer`] before any rule sees it. The
//! default is [`NormalizeNfc`], Unicode's canonical composed form, so that a
//! letter followed by a combining accent becomes the single accented
//! character the tables know; a quick check comes first, and text that it
//! finds composed already — nearly all text — is not copied.
//!
//! Positions, in an [`EncodeError`] as in
//! [`EncodeReporter::report_unknown_char`], are byte offsets into the
//! normalized text, which is not the caller's text where normalization
//! changed it. A caller that needs offsets into its own text normalizes it
//! once with [`nfc`] and encodes with [`NoNormalization`]. Under
//! [`NoNormalization`] nothing references the normalization tables and the
//! linker drops them.
//!
//! ```
//! use untechxt::normalizer::{nfc, NoNormalization};
//! use untechxt::{default_rules, encode, Encoder};
//!
//! // `e` followed by a combining acute accent, composed into `é` first.
//! assert_eq!(encode("Cafe\u{301}"), r"Caf\'e");
//!
//! let as_is = Encoder::new(default_rules()).with_normalizer(NoNormalization);
//! assert_eq!(as_is.encode("Cafe\u{301}").unwrap(), "Cafe\u{301}"); // unknown, kept
//! assert_eq!(as_is.encode(&nfc("Cafe\u{301}")).unwrap(), r"Caf\'e");
//! ```
//!
//! # Where the output goes
//!
//! [`Encoder::encode_into`] writes to any [`OutBuffer`]. [`String`] is one,
//! and costs nothing extra: the error check disappears when the compiler
//! inlines the call. [`FmtOut`] carries the output to a [`core::fmt::Write`],
//! and `IoOut` — with the crate feature `std` — to a `std::io::Write`: a
//! file, a socket, standard output, so that a long document never has to
//! exist as one string. The error type is the fixed [`BoxError`], so that no
//! signature of the crate carries an error parameter and an I/O error is
//! passed through as it is.
//!
//! ```
//! use untechxt::outbuffer::FmtOut;
//! use untechxt::report::NoReport;
//! use untechxt::{default_rules, Encoder};
//!
//! let mut out = FmtOut(String::new());
//! let encoder = Encoder::new(default_rules());
//! encoder.encode_into("Café", &mut out, &mut NoReport).unwrap();
//! assert_eq!(out.0, r"Caf\'e");
//! ```
//!
//! # The ASCII fast path
//!
//! Plain ASCII text is the common case, and it is copied in bulk. An
//! [`AsciiSet`] is a bitmap of ASCII characters in a `u128`, so a membership
//! test is a shift and a mask. [`Rule::ascii_triggers`] answers the set of
//! ASCII characters the rule may match at; [`Encoder::new`] asks once, unions
//! the answers with every non-printable ASCII character, and then copies runs
//! of input bytes outside that union with a single push, decoding no
//! character and calling no rule.
//!
//! That set is a contract a rule author must respect. It is a promise that
//! the rule never matches at an ASCII character outside it, and the answer
//! must be the same every time, since it is asked exactly once. It is a
//! promise in one direction only: the encoder may still consult the rule at
//! other characters, where a correct rule declines. The default is
//! [`AsciiSet::ALL`] — "I may match anywhere" — which is always correct and
//! only slower; [`AsciiSet::EMPTY`] is what a table of non-ASCII entries
//! answers. A table answers its own keys, and
//! [`RuleFn::with_ascii_triggers`] is where a closure rule states its own, as
//! the ellipsis rule above states its `.`.
//!
//! # Organization of the crate
//!
//! The items that most programs need are at the root of the crate: the
//! [`encode`] and [`default_rules`] functions, the [`Encoder`] struct with its
//! error type [`EncodeError`], and the [`UnknownCharPolicy`] enum. All other
//! items are in one module per concept, and each item has a single public
//! path:
//!
//! - The [`rule`] module contains the [`Rule`] trait, closure rules and rule
//!   chains.
//! - The [`lookuptable`] module contains the [`LookupTable`] trait and lookup
//!   tables that are built at run time.
//! - The [`statictable`] module contains lookup tables that are compiled at
//!   compile time.
//! - The [`builtin`] module contains the builtin lookup tables.
//! - The [`protection`] module contains the replacement protection hints and
//!   strategies.
//! - The [`preamble`] module contains the types that describe what the
//!   encoded LaTeX needs in the preamble of the document.
//! - The [`report`] module contains the [`EncodeReport`] struct and the
//!   [`EncodeReporter`] trait.
//! - The [`normalizer`] module contains the input normalizers.
//! - The [`outbuffer`] module contains the types that the output can be
//!   written to.
//!
//! # `no_std`, `alloc`, and the `std` feature
//!
//! The crate is `#![no_std]` and uses [`alloc`]: it allocates strings and
//! vectors and nothing else. Errors are [`core::error::Error`] throughout.
//! Its one dependency, `unicode-normalization`, is `no_std` as well.
//!
//! The default feature `std` adds a single item, the `IoOut` adapter for
//! `std::io::Write`; `cargo build --no-default-features` is the `no_std`
//! build.
//!
//! # Relation to pylatexenc
//!
//! The design follows the `latexencode` module of
//! [pylatexenc](https://github.com/phfaist/pylatexenc), the Python library
//! for working with LaTeX code, by the same author: an ordered list of rules,
//! the first match at each position winning, each value protected so that it
//! cannot merge with the text that follows, and a policy for the characters
//! no rule knows. Three things are this library's own — what an encoded value
//! needs in the preamble, LaTeX's two modes, and output that streams — beside
//! the rule model, which is a trait rather than a fixed set of rule kinds.
//!
//! The builtin data began as a copy of pylatexenc's `defaults` table, whose
//! character map came in turn from
//! [latexcodec](https://pypi.python.org/pypi/latexcodec). It is maintained
//! here by hand: every entry compiles and sets the glyph its character stands
//! for, and every departure from pylatexenc's spelling is listed with its
//! reason in [`builtin::default_table`], where the MIT notices that travel
//! with the data are reproduced in full. pylatexenc's second table,
//! `unicode-xml`, is not ported.
//!
//! What has no counterpart here: regular-expression rules (a closure rule
//! expresses them); per-rule protection overrides (a rule's only say is its
//! hint); the `non_ascii_only` flag, which silently disabled multi-character
//! rules that start at an ASCII character — leave
//! [`DEFAULT_TABLE_ASCII_SPECIALS`] out of the chain, or encode with
//! [`DEFAULT_TABLE_NON_ASCII`], instead; the
//! `'braces-almost-all'` protection mode ([`BracesAroundAll`] is
//! `'braces-all'`); and the `'unihex'` unknown-character mode, which is the
//! plain function [`unknown_unihex`] handed to
//! [`UnknownCharPolicy::callback`]. Positions are byte offsets, where
//! pylatexenc counts code points.
//!
//! [`AnyMode`]: protection::ValueMode::AnyMode
//! [`AsciiSet`]: rule::AsciiSet
//! [`AsciiSet::ALL`]: rule::AsciiSet::ALL
//! [`AsciiSet::EMPTY`]: rule::AsciiSet::EMPTY
//! [`BracesAround`]: protection::MacroNameProtection::BracesAround
//! [`BracesAroundAll`]: protection::BracesAroundAll
//! [`Chunk`]: preamble::Chunk
//! [`ChunkCase`]: preamble::ChunkCase
//! [`ChunkPreamble`]: preamble::ChunkPreamble
//! [`compile_static_table!`]: statictable::compile_static_table!
//! [`DEFAULT_TABLE`]: builtin::DEFAULT_TABLE
//! [`DEFAULT_TABLE_ASCII_SPECIALS`]: builtin::DEFAULT_TABLE_ASCII_SPECIALS
//! [`DEFAULT_TABLE_NON_ASCII`]: builtin::DEFAULT_TABLE_NON_ASCII
//! [`DoNotProtect`]: protection::ReplacementProtectionHint::DoNotProtect
//! [`DynRuleChain`]: rule::DynRuleChain
//! [`DynTable`]: lookuptable::DynTable
//! [`EncodedReplacement`]: rule::EncodedReplacement
//! [`EncodedReplacement::with_needs`]: rule::EncodedReplacement::with_needs
//! [`EncodeReport`]: report::EncodeReport
//! [`EncodeReporter`]: report::EncodeReporter
//! [`EncodeReporter::report_unknown_char`]:
//!     report::EncodeReporter::report_unknown_char
//! [`Engine`]: preamble::Engine
//! [`EngineSet`]: preamble::EngineSet
//! [`FmtOut`]: outbuffer::FmtOut
//! [`InputNormalizer`]: normalizer::InputNormalizer
//! [`LocalDynRuleChain`]: rule::LocalDynRuleChain
//! [`LookupTable`]: lookuptable::LookupTable
//! [`math_mode`]: protection::StandardProtection::math_mode
//! [`math_wrap`]: protection::StandardProtection::math_wrap
//! [`MathOnly`]: protection::ValueMode::MathOnly
//! [`ModeWrapper`]: protection::ModeWrapper
//! [`nfc`]: normalizer::nfc
//! [`NoNormalization`]: normalizer::NoNormalization
//! [`NoReport`]: report::NoReport
//! [`NormalizeNfc`]: normalizer::NormalizeNfc
//! [`OutBuffer`]: outbuffer::OutBuffer
//! [`output_mode`]: protection::StandardProtection::output_mode
//! [`PreambleNeeds`]: preamble::PreambleNeeds
//! [`PreambleNeeds::write_preamble`]: preamble::PreambleNeeds::write_preamble
//! [`PreambleNeeds::write_preamble_for`]:
//!     preamble::PreambleNeeds::write_preamble_for
//! [`Profile`]: preamble::Profile
//! [`protect_names`]: protection::StandardProtection::protect_names
//! [`ProtectInput`]: protection::ProtectInput
//! [`ReplacementProtection`]: protection::ReplacementProtection
//! [`ReplacementProtectionHint`]: protection::ReplacementProtectionHint
//! [`ReplacementProtectionHint::text_only`]:
//!     protection::ReplacementProtectionHint::text_only
//! [`Rule`]: rule::Rule
//! [`Rule::ascii_triggers`]: rule::Rule::ascii_triggers
//! [`rule_fn`]: rule::rule_fn
//! [`RuleChain`]: rule::RuleChain
//! [`RuleFn::with_ascii_triggers`]: rule::RuleFn::with_ascii_triggers
//! [`RuleInput`]: rule::RuleInput
//! [`StandardProtection`]: protection::StandardProtection
//! [`TableRule`]: lookuptable::TableRule
//! [`text_wrap`]: protection::StandardProtection::text_wrap
//! [`TextOnly`]: protection::ValueMode::TextOnly
//! [`Value`]: protection::ReplacementProtectionHint::Value
//! [`ValueMode`]: protection::ValueMode
//! [`ValueTermination`]: protection::ValueTermination
//! [`ValueTermination::inspect`]: protection::ValueTermination::inspect

#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod builtin;
mod defaults;
mod encoder;
pub mod lookuptable;
pub mod normalizer;
pub mod outbuffer;
pub mod preamble;
pub mod protection;
pub mod report;
pub mod rule;
pub mod statictable;
mod unknown_char;

pub use crate::defaults::{default_rules, DefaultRules};
pub use crate::encoder::{EncodeError, Encoder};
pub use crate::unknown_char::{unknown_unihex, UnknownCharPolicy};

use alloc::boxed::Box;
use alloc::string::String;

/// The error type that a rule, an output buffer or a protection strategy
/// returns when it fails. It is the boxed error type that is common in the
/// Rust ecosystem.
///
/// The error type is fixed rather than a type parameter, so that no signature
/// in this crate has an error type parameter. An error of any type converts
/// into a `BoxError` with `?` or `.into()`. For instance, an I/O error of an
/// output buffer is passed through unchanged, and a rule that calls a Python
/// or JavaScript callback can store the exception of the callback, converted
/// to a string if necessary.
pub type BoxError = Box<dyn core::error::Error + Send + Sync + 'static>;

/// The LaTeX for `text`, under every default setting.
///
/// Those settings are: the default rules (see [`default_rules`]) as the only
/// rule, [`StandardProtection::text_mode`] around every value,
/// [`NormalizeNfc`] over the input, [`UnknownCharPolicy::Keep`] for a
/// character the default rules do not know, [`NoReport`] — what the LaTeX
/// needs in the preamble is dropped — and a [`String`] to write into. None of
/// those can fail, which is why this answers the string itself rather than a
/// [`Result`].
///
/// Build an [`Encoder`] to change any of that, and in particular to learn
/// what the output needs in the document's preamble
/// ([`Encoder::encode_with_report`]). This function builds its encoder on
/// every call, which for a static table is a few instructions — there is no
/// cell to cache one in without `std` — but an encoder built once and kept is
/// still less work.
///
/// ```
/// use untechxt::encode;
///
/// assert_eq!(encode("Café — naïve"), r#"Caf\'e {\textemdash} na\"ive"#);
/// assert_eq!(encode("100% & more"), r"100\% \& more");
/// assert_eq!(encode("α ≤ β"), r"\ensuremath{\alpha} \ensuremath{\leq} \ensuremath{\beta}");
/// // A character the builtin table has no entry for is kept as it is.
/// assert_eq!(encode("ธ"), "ธ");
/// ```
///
/// [`NoReport`]: report::NoReport
/// [`NormalizeNfc`]: normalizer::NormalizeNfc
/// [`StandardProtection::text_mode`]: protection::StandardProtection::text_mode
pub fn encode(text: &str) -> String {
    Encoder::new(default_rules()).encode(text).expect(
        "encoding with the default rules under the default settings cannot fail: \
         the default rules never fail, an unknown character is kept rather than \
         refused, and a String never refuses what is written to it",
    )
}
