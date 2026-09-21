//! The command line: what `untechxt --help` prints, and which value of the
//! library each argument stands for.
//!
//! Every argument here selects one rule, one replacement protection strategy,
//! one unknown-character policy or one LaTeX engine, and the methods of
//! [`Cli`] are that mapping. The mapping lives in this module, and not in `main`, so that it is
//! stated once and tested here.
//!
//! The enumerated values are restated as local [`ValueEnum`] types rather than
//! derived on the types of the library, because the library must not depend on
//! clap.
//!
//! The types [`Rules`] and [`Protection`] have a second purpose. The rule and
//! the replacement protection strategy are type parameters of the encoder, so
//! a program that picks one of two of them at run time needs a single type
//! that stands for both. Each of these two enums is such a type, and
//! implements the trait of the library by passing each call on to the value it
//! holds.

use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use untechxt::builtin::{BuiltinTable, DEFAULT_TABLE_NON_ASCII};
use untechxt::outbuffer::OutBuffer;
use untechxt::preamble::Engine;
use untechxt::protection::{
    BracesAroundAll, MacroNameProtection, ProtectInput, ReplacementProtection,
    StandardProtection,
};
use untechxt::report::EncodeReporter;
use untechxt::rule::{AsciiSet, Rule, RuleInput, RuleResult};
use untechxt::{
    default_rules, unknown_unihex, BoxError, DefaultRules, UnknownCharPolicy,
};

/// The text that `--unknown-char-policy replace` writes for a character that
/// has no known LaTeX representation. It is the text that pylatexenc writes,
/// and it sets a bold question mark, which stands out in the typeset document.
pub const REPLACEMENT_TEXT: &str = r"{\bfseries ?}";

// `about` and `version` take their text from the manifest
// (`CARGO_PKG_DESCRIPTION` and `CARGO_PKG_VERSION`), so that the help banner
// cannot drift from what was built. clap derives the rest of the help text
// from the doc comments below, which is why implementation notes go in
// ordinary `//` comments like this one.
/// Encode Unicode text as LaTeX source.
///
/// Reads each FILE in turn, or standard input when no file is given, and
/// writes the LaTeX to standard output or to the file named by --output. A
/// report on standard error lists the characters that have no known LaTeX
/// representation and what the output needs in the document preamble.
///
/// Exit codes: 0 when the input was encoded, 1 when the input could not be
/// encoded, and 2 when a file could not be read or written, an input was not
/// UTF-8, or the command line was wrong.
#[derive(Debug, Parser)]
#[command(name = "untechxt", version, about)]
pub struct Cli {
    /// The files to encode, in order. Reads standard input when no file is
    /// given, and for a file named -.
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    /// Write the LaTeX here instead of to standard output.
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    pub output: Option<PathBuf>,

    // The flag and its negation override each other, so the last one on the
    // command line wins. That is what lets a flag in an alias or a wrapper
    // script be turned off by hand.
    /// Encode the non-ASCII characters only, and keep every ASCII character,
    /// including the LaTeX special characters \ % { } $ & # _ ^ ~, unchanged.
    /// Use this for input that already contains LaTeX code.
    #[arg(long, overrides_with = "no_non_ascii_only")]
    pub non_ascii_only: bool,

    /// Encode the LaTeX special characters as well as the non-ASCII
    /// characters. This is the default.
    #[arg(long, overrides_with = "non_ascii_only")]
    pub no_non_ascii_only: bool,

    /// How a replacement that ends with a macro name is kept apart from the
    /// text that follows it. Without this option, braces in text mode and a
    /// space in math mode.
    #[arg(long, value_enum, value_name = "STRATEGY")]
    pub replacement_protection: Option<ProtectionArg>,

    /// The output is going inside LaTeX math rather than into document text.
    /// Symbols such as \alpha are then written as they are, and a value that
    /// is valid in text mode alone is wrapped in \textnormal{}.
    #[arg(long)]
    pub math_mode: bool,

    /// What to write for a character that has no known LaTeX representation.
    #[arg(
        long,
        value_enum,
        default_value_t = UnknownCharArg::Keep,
        value_name = "POLICY"
    )]
    pub unknown_char_policy: UnknownCharArg,

    /// Write what the output needs in the document preamble here, as
    /// \usepackage lines and definitions. The file is written even when
    /// nothing is needed, so that a document can \input it unconditionally.
    #[arg(long, value_name = "FILE")]
    pub preamble: Option<PathBuf>,

    /// The LaTeX engine that the preamble is written for, both in the file of
    /// --preamble and in the report on standard error. The LaTeX itself is
    /// the same for every engine.
    #[arg(long, value_enum, default_value_t = EngineArg::Any, value_name = "ENGINE")]
    pub engine: EngineArg,

    /// Print no report on standard error. A failure is still reported.
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,
}

impl Cli {
    /// Returns the rule that these arguments encode the input with: the
    /// default rules of the library, or the builtin table of the non-ASCII
    /// entries under `--non-ascii-only`.
    pub fn rules(&self) -> Rules {
        if self.non_ascii_only {
            Rules::NonAscii(&DEFAULT_TABLE_NON_ASCII)
        } else {
            Rules::Default(default_rules())
        }
    }

    /// Returns the replacement protection strategy that these arguments
    /// describe.
    ///
    /// The `--math-mode` flag selects the output mode, and the output mode
    /// decides two things: what is written around a value that is valid in
    /// the other mode alone, and, when `--replacement-protection` is absent,
    /// what is written around a value that ends with a macro name.
    pub fn protection(&self) -> Protection {
        let mut standard = if self.math_mode {
            StandardProtection::math_mode()
        } else {
            StandardProtection::text_mode()
        };
        // Four of the strategies name one value of `MacroNameProtection`.
        // `braces-all` names none of them, because braces around every
        // replacement are a strategy of their own rather than a setting of
        // `StandardProtection`; it therefore keeps the macro-name protection
        // of the mode, as does the absent option.
        let protect_names = match self.replacement_protection {
            Some(ProtectionArg::Braces) => Some(MacroNameProtection::BracesAround),
            Some(ProtectionArg::BracesAfter) => Some(MacroNameProtection::BracesAfter),
            Some(ProtectionArg::Space) => {
                Some(MacroNameProtection::SpaceAfterMacroName)
            }
            Some(ProtectionArg::None) => Some(MacroNameProtection::NoProtection),
            Some(ProtectionArg::BracesAll) | None => None,
        };
        if let Some(protect_names) = protect_names {
            standard.protect_names = protect_names;
        }
        if self.replacement_protection == Some(ProtectionArg::BracesAll) {
            Protection::BracesAll(BracesAroundAll(standard))
        } else {
            Protection::Standard(standard)
        }
    }

    /// Returns the LaTeX engine that these arguments write the preamble for,
    /// or `None` for a preamble that compiles under every engine, which is
    /// the default.
    pub fn engine(&self) -> Option<Engine> {
        match self.engine {
            EngineArg::Any => None,
            EngineArg::Pdflatex => Some(Engine::PdfLatex),
            EngineArg::Lualatex => Some(Engine::LuaLatex),
            EngineArg::Xelatex => Some(Engine::XeLatex),
        }
    }

    /// Returns the policy that these arguments apply to a character that has
    /// no known LaTeX representation.
    pub fn unknown_chars(&self) -> UnknownCharPolicy {
        match self.unknown_char_policy {
            UnknownCharArg::Keep => UnknownCharPolicy::Keep,
            UnknownCharArg::Ignore => UnknownCharPolicy::Ignore,
            UnknownCharArg::Fail => UnknownCharPolicy::Fail,
            UnknownCharArg::Replace => {
                UnknownCharPolicy::ReplaceWith(REPLACEMENT_TEXT.into())
            }
            UnknownCharArg::Unihex => UnknownCharPolicy::callback(unknown_unihex),
        }
    }
}

/// The values of the `--replacement-protection` option. Each of them names
/// one way of keeping a replacement apart from the text that follows it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ProtectionArg {
    /// Braces around the replacement: {\textemdash}.
    Braces,
    /// An empty group after the replacement: \textemdash{}.
    BracesAfter,
    /// Braces around every replacement, including the replacements that need
    /// no protection at all: {\'e}.
    BracesAll,
    /// A space after the replacement, which writes \textemdash followed by a
    /// space. This is only safe in math mode. TeX skips the spaces that
    /// follow a control word, so a space of the text itself would be lost in
    /// text mode.
    Space,
    /// Nothing at all. This can produce broken LaTeX, such as the single
    /// unknown command \textemdashuser.
    None,
}

/// The values of the `--unknown-char-policy` option. Each of them names one
/// thing to write for a character that has no known LaTeX representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum UnknownCharArg {
    /// Write the character itself, as UTF-8.
    Keep,
    /// Write nothing for the character.
    Ignore,
    /// Stop, and report the character and its position.
    Fail,
    /// Write a bold question mark for the character.
    Replace,
    /// Write the code point of the character, as \texttt{U+0E18} between
    /// angle brackets.
    Unihex,
}

/// The values of the `--engine` option. Each of them names what compiles the
/// document, which decides what the preamble contains. For example, the
/// Cyrillic letters need the font encoding T2A beside T1 under pdfLaTeX, and
/// beside TU under LuaLaTeX and XeLaTeX.
// The variants are single words so that clap names the values `pdflatex`,
// `lualatex` and `xelatex`, which are the names of the programs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum EngineArg {
    /// Every engine. Where the engines need different things, the preamble
    /// loads the package iftex and tests which engine is running.
    Any,
    /// pdfLaTeX, which includes LaTeX writing a DVI file.
    Pdflatex,
    /// LuaLaTeX.
    Lualatex,
    /// XeLaTeX.
    Xelatex,
}

/// The rule that the command line selected, which is the encoding table that
/// the encoder consults at every position of the input.
///
/// The default rules and the builtin table of non-ASCII entries have different
/// types, and the rule is a type parameter of
/// [`Encoder`](untechxt::Encoder), so a choice made at run time needs one type
/// that stands for both rules. This enum is that type: it implements [`Rule`]
/// by passing each call on to the rule it holds.
#[derive(Clone, Copy, Debug)]
pub enum Rules {
    /// The default rules of the library, which encode every character that
    /// the builtin data contains. This is the default.
    Default(DefaultRules),
    /// The builtin table of the non-ASCII entries, under `--non-ascii-only`.
    NonAscii(&'static BuiltinTable),
}

impl Rule for Rules {
    fn apply<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a> {
        match self {
            Rules::Default(rules) => rules.apply(input),
            Rules::NonAscii(table) => table.apply(input),
        }
    }

    fn ascii_triggers(&self) -> AsciiSet {
        match self {
            Rules::Default(rules) => rules.ascii_triggers(),
            Rules::NonAscii(table) => table.ascii_triggers(),
        }
    }
}

/// The replacement protection strategy that the command line selected, which
/// the encoder writes every replacement through.
///
/// The two strategies of the library have different types, and the strategy is
/// a type parameter of [`Encoder`](untechxt::Encoder), so a choice made at run
/// time needs one type that stands for both strategies. This enum is that
/// type: it implements [`ReplacementProtection`] by passing each call on to
/// the strategy it holds.
#[derive(Clone, Debug)]
pub enum Protection {
    /// The standard strategy of the library, under `braces`, `braces-after`,
    /// `space` and `none`, and when `--replacement-protection` is absent.
    Standard(StandardProtection),
    /// Braces around every replacement, under `braces-all`.
    BracesAll(BracesAroundAll),
}

impl ReplacementProtection for Protection {
    fn write_protected<O: OutBuffer, Rep: EncodeReporter>(
        &self,
        out: &mut O,
        report: &mut Rep,
        item: ProtectInput<'_>,
    ) -> Result<(), BoxError> {
        match self {
            Protection::Standard(protection) => {
                protection.write_protected(out, report, item)
            }
            Protection::BracesAll(protection) => {
                protection.write_protected(out, report, item)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use untechxt::protection::OutputMode;

    /// Parses a command line that starts with the name of the program.
    fn parse(args: &[&str]) -> Cli {
        let mut argv = vec!["untechxt"];
        argv.extend_from_slice(args);
        Cli::parse_from(argv)
    }

    /// The [`StandardProtection`] of a [`Protection::Standard`], which is what
    /// every test but the one about `braces-all` expects.
    fn standard(protection: &Protection) -> StandardProtection {
        match protection {
            Protection::Standard(standard) => standard.clone(),
            Protection::BracesAll(_) => {
                panic!("the standard strategy was expected, not braces-all")
            }
        }
    }

    #[test]
    fn the_command_line_definition_is_well_formed() {
        // The consistency check of clap itself: duplicate names, impossible
        // defaults, and so on. It panics on a mistake, which is why it belongs
        // in a test rather than in `main`.
        Cli::command().debug_assert();
    }

    #[test]
    fn the_defaults_are_the_defaults_of_the_library() {
        let cli = parse(&[]);
        assert!(matches!(cli.rules(), Rules::Default(_)));
        assert_eq!(standard(&cli.protection()), StandardProtection::text_mode());
        assert!(matches!(cli.unknown_chars(), UnknownCharPolicy::Keep));
        assert!(cli.files.is_empty());
        assert_eq!(cli.output, None);
        assert_eq!(cli.preamble, None);
        assert_eq!(cli.engine(), None);
        assert!(!cli.quiet);
    }

    #[test]
    fn the_non_ascii_only_flag_selects_the_non_ascii_table() {
        assert!(matches!(parse(&["--non-ascii-only"]).rules(), Rules::NonAscii(_)));
        assert!(matches!(
            parse(&["--no-non-ascii-only"]).rules(),
            Rules::Default(_)
        ));
        // The flag and its negation override each other, so the last one on
        // the command line is the one that counts.
        assert!(matches!(
            parse(&["--non-ascii-only", "--no-non-ascii-only"]).rules(),
            Rules::Default(_)
        ));
        assert!(matches!(
            parse(&["--no-non-ascii-only", "--non-ascii-only"]).rules(),
            Rules::NonAscii(_)
        ));
    }

    #[test]
    fn the_math_mode_flag_selects_the_output_mode() {
        let text = standard(&parse(&[]).protection());
        assert_eq!(text.output_mode, OutputMode::TextMode);
        assert_eq!(text.protect_names, MacroNameProtection::BracesAround);

        // Both of the defaults of `StandardProtection::math_mode`, the output
        // mode and the macro-name protection, come along with the flag.
        let math = standard(&parse(&["--math-mode"]).protection());
        assert_eq!(math.output_mode, OutputMode::MathMode);
        assert_eq!(math.protect_names, MacroNameProtection::SpaceAfterMacroName);
    }

    #[test]
    fn every_replacement_protection_reaches_its_strategy() {
        let protect_names = |args: &[&str]| standard(&parse(args).protection()).protect_names;
        assert_eq!(
            protect_names(&["--replacement-protection", "braces"]),
            MacroNameProtection::BracesAround
        );
        assert_eq!(
            protect_names(&["--replacement-protection", "braces-after"]),
            MacroNameProtection::BracesAfter
        );
        assert_eq!(
            protect_names(&["--replacement-protection", "space"]),
            MacroNameProtection::SpaceAfterMacroName
        );
        assert_eq!(
            protect_names(&["--replacement-protection", "none"]),
            MacroNameProtection::NoProtection
        );
        // An explicit strategy overrides the default of the output mode.
        assert_eq!(
            protect_names(&["--math-mode", "--replacement-protection", "braces"]),
            MacroNameProtection::BracesAround
        );

        // `braces-all` is the one strategy of its own, and it wraps the
        // standard strategy of the output mode unchanged.
        let all = parse(&["--math-mode", "--replacement-protection", "braces-all"])
            .protection();
        match all {
            Protection::BracesAll(BracesAroundAll(inner)) => {
                assert_eq!(inner, StandardProtection::math_mode());
            }
            Protection::Standard(_) => panic!("braces-all is a strategy of its own"),
        }
    }

    #[test]
    fn every_unknown_char_policy_reaches_its_policy() {
        let policy = |value: &str| parse(&["--unknown-char-policy", value]).unknown_chars();
        assert!(matches!(policy("keep"), UnknownCharPolicy::Keep));
        assert!(matches!(policy("ignore"), UnknownCharPolicy::Ignore));
        assert!(matches!(policy("fail"), UnknownCharPolicy::Fail));
        match policy("replace") {
            UnknownCharPolicy::ReplaceWith(text) => assert_eq!(text, REPLACEMENT_TEXT),
            other => panic!("replace is a fixed text, not {other:?}"),
        }
        match policy("unihex") {
            // Calling the callback is the only way to tell which function it
            // holds, so the test compares what it writes for one character
            // with what `unknown_unihex` writes for that character.
            UnknownCharPolicy::Callback(write) => {
                assert_eq!(write('\u{0e18}'), unknown_unihex('\u{0e18}'));
            }
            other => panic!("unihex is a callback, not {other:?}"),
        }
    }

    #[test]
    fn every_engine_reaches_its_engine() {
        let engine = |value: &str| parse(&["--engine", value]).engine();
        assert_eq!(engine("any"), None);
        assert_eq!(engine("pdflatex"), Some(Engine::PdfLatex));
        assert_eq!(engine("lualatex"), Some(Engine::LuaLatex));
        assert_eq!(engine("xelatex"), Some(Engine::XeLatex));
    }

    #[test]
    fn an_unknown_value_is_a_usage_error() {
        assert!(Cli::try_parse_from(["untechxt", "--unknown-char-policy", "shrug"]).is_err());
        assert!(
            Cli::try_parse_from(["untechxt", "--replacement-protection", "curly"]).is_err()
        );
        assert!(Cli::try_parse_from(["untechxt", "--engine", "tex"]).is_err());
    }
}
