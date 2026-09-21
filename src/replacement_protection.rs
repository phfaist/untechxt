//! Protection: what is written around an encoded value so that it cannot
//! merge with the text that follows it, and so that it is valid in the mode
//! the output is in.
//!
//! A rule states what its value is through a
//! [`ReplacementProtectionHint`]; a [`ReplacementProtection`] strategy —
//! [`StandardProtection`] unless the caller says otherwise — turns the value
//! and its hint into the text that is actually written.

use alloc::borrow::Cow;

use crate::outbuffer::OutBuffer;
use crate::profile::Profile;
use crate::report::EncodeReporter;
use crate::rule::BoxError;

/// In which of LaTeX's two modes an encoded value may be used.
///
/// The mode lives here rather than in the value: a table entry holds the bare
/// `\alpha` marked [`MathOnly`](ValueMode::MathOnly), not
/// `\ensuremath{\alpha}`, so that a math output mode can write it as it is
/// and a text output mode can wrap it once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueMode {
    /// LaTeX that is valid in text mode alone, such as `\'e` or
    /// `\textemdash`.
    TextOnly,
    /// LaTeX that is valid in math mode alone, such as `\alpha` or `\leq`.
    MathOnly,
    /// LaTeX that is valid in either mode: plain characters such as `fi`, or
    /// a value that carries its own `\ensuremath{…}`.
    AnyMode,
}

/// Whether an encoded value can be followed directly by arbitrary text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueTermination {
    /// The encoded value, under a standard document parsing state (catcode
    /// setting), will always parse correctly, whatever string succeeds it.
    ValueIsSelfTerminating,
    /// The encoded value ends with a macro name (for example `r"\hat\i"`). If
    /// it is immediately followed by more content, the macro will fail to
    /// parse correctly (`r"\hat\i"` + `"more text"` = `r"\hat\imore text"`,
    /// incorrect). The value needs to be protected by a suitable protection
    /// strategy, for instance by enclosing it in braces (`r"{\hat\i}"`).
    ValueEndsWithNamedMacro,
    // We might add further variants in the future...
    //
    // /// The value will cause any text that follows to be interpreted as part
    // /// of a comment. Needs a newline to restore correct content parsing.
    // ValueEndsInAComment,
}

impl ValueTermination {
    /// Reads the termination off the form of `encoded`: it
    /// [ends with a named macro](ValueTermination::ValueEndsWithNamedMacro)
    /// when it ends in a run of one or more ASCII letters that is immediately
    /// preceded by a backslash, and is
    /// [self-terminating](ValueTermination::ValueIsSelfTerminating)
    /// otherwise.
    ///
    /// A rule that knows the answer states it directly; this is for rules
    /// that do not, and it is what the static table builders run at compile
    /// time.
    ///
    /// ```
    /// use untechxt::ValueTermination::{self, ValueEndsWithNamedMacro, ValueIsSelfTerminating};
    ///
    /// assert_eq!(ValueTermination::inspect(r"\textemdash"), ValueEndsWithNamedMacro);
    /// assert_eq!(ValueTermination::inspect(r"\hat\i"), ValueEndsWithNamedMacro);
    /// assert_eq!(ValueTermination::inspect(r"\'e"), ValueIsSelfTerminating);
    /// assert_eq!(ValueTermination::inspect(r"\r{A}"), ValueIsSelfTerminating);
    /// assert_eq!(ValueTermination::inspect("fi"), ValueIsSelfTerminating);
    /// assert_eq!(ValueTermination::inspect(""), ValueIsSelfTerminating);
    /// ```
    pub const fn inspect(encoded: &str) -> Self {
        let bytes = encoded.as_bytes();
        let mut start = bytes.len();
        while start > 0 && bytes[start - 1].is_ascii_alphabetic() {
            start -= 1;
        }
        if start < bytes.len() && start > 0 && bytes[start - 1] == b'\\' {
            ValueTermination::ValueEndsWithNamedMacro
        } else {
            ValueTermination::ValueIsSelfTerminating
        }
    }
}

/// What a rule says about the value it produces, so that the protection
/// strategy knows what to write around it.
///
/// The hint is required and authoritative: the encoder never inspects a value
/// itself. A rule that does not know the termination of its value reads it
/// off the value's form with one of the constructors here, which is what
/// [`ValueTermination::inspect`] does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ReplacementProtectionHint {
    /// Write the value exactly as it is, with nothing around it: the rule
    /// vouches for it in its context. This is what a rule that passes
    /// existing LaTeX through (`\textbf{…}`) uses.
    DoNotProtect,
    /// A value that stands for the input it replaced, to be protected
    /// according to the mode it is valid in and the way it ends.
    Value {
        /// The LaTeX mode the value is valid in.
        mode: ValueMode,
        /// Whether the value can be followed directly by arbitrary text.
        termination: ValueTermination,
    },
}

impl ReplacementProtectionHint {
    /// The hint of a text-mode value, with its termination read off
    /// `encoded`.
    pub const fn text_only(encoded: &str) -> Self {
        ReplacementProtectionHint::Value {
            mode: ValueMode::TextOnly,
            termination: ValueTermination::inspect(encoded),
        }
    }

    /// The hint of a math-mode value, with its termination read off
    /// `encoded`.
    pub const fn math_only(encoded: &str) -> Self {
        ReplacementProtectionHint::Value {
            mode: ValueMode::MathOnly,
            termination: ValueTermination::inspect(encoded),
        }
    }

    /// The hint of a value that is valid in either mode, with its termination
    /// read off `encoded`.
    pub const fn any_mode(encoded: &str) -> Self {
        ReplacementProtectionHint::Value {
            mode: ValueMode::AnyMode,
            termination: ValueTermination::inspect(encoded),
        }
    }
}

/// One encoded value on its way to the output, as a
/// [`ReplacementProtection`] receives it.
///
/// It is a small `Copy` struct with accessors rather than public fields, so
/// that more context can be given to strategies later without breaking the
/// ones that exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProtectInput<'v> {
    encoded: &'v str,
    hint: ReplacementProtectionHint,
}

impl<'v> ProtectInput<'v> {
    /// The value `encoded` with the hint its rule gave. The encoder builds
    /// one for every match; a caller builds one to write a value of its own
    /// through a strategy, or to test a strategy.
    pub const fn new(encoded: &'v str, hint: ReplacementProtectionHint) -> Self {
        ProtectInput { encoded, hint }
    }

    /// The LaTeX the rule produced, with nothing around it.
    pub const fn encoded(&self) -> &'v str {
        self.encoded
    }

    /// What the rule said about the value.
    pub const fn hint(&self) -> ReplacementProtectionHint {
        self.hint
    }
}

/// How an encoded value is written to the output: what is put around it so
/// that it survives being pasted into arbitrary text, and so that it is valid
/// in the mode the output is in.
///
/// Protection is stateless: each value is protected as it comes, with no
/// lookahead and no memory of what was written before.
///
/// The reporter is passed in because a mode wrapper may itself need something
/// in the preamble — `\text{…}` needs amsmath.
///
/// The trait has a generic method, and so it is not dyn-compatible: a fully
/// custom strategy is a compile-time choice. [`StandardProtection`] is an
/// ordinary struct whose fields can be set at run time, which is what
/// language bindings need.
pub trait ReplacementProtection: core::fmt::Debug {
    /// Writes `item` to `out`, with whatever protection the strategy applies.
    ///
    /// # Errors
    ///
    /// Whatever `out` reports; the encoder passes it on as
    /// [`EncodeError::Output`](crate::EncodeError::Output).
    fn write_protected<O: OutBuffer, Rep: EncodeReporter>(
        &self,
        out: &mut O,
        report: &mut Rep,
        item: ProtectInput<'_>,
    ) -> Result<(), BoxError>;
}

/// Which of LaTeX's two modes the encoded output is going into.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OutputMode {
    /// Ordinary document text. The default.
    #[default]
    TextMode,
    /// Inside mathematics, between `$…$` or in a display.
    MathMode,
}

/// What is written around a value that ends with a named macro, so that the
/// text that follows cannot become part of the macro's name.
///
/// Which one is safe depends on the output mode, which is why this is a field
/// of [`StandardProtection`] beside
/// [`output_mode`](StandardProtection::output_mode) rather than a choice made
/// per value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MacroNameProtection {
    /// Wrap the value in braces: `\textemdash` becomes `{\textemdash}`. The
    /// default, and the safe choice in text mode.
    #[default]
    BracesAround,
    /// Append an empty group: `\textemdash` becomes `\textemdash{}`.
    BracesAfter,
    /// Append a space: `\pm` becomes `\pm `. Safe in math mode, where spaces
    /// are ignored, and the default there. **Unsafe in text mode**: TeX skips
    /// every space after a control word, so a real space that follows in the
    /// input would be swallowed, and a stateless strategy cannot know whether
    /// one follows.
    SpaceAfterMacroName,
    /// Write the value as it is, with nothing appended. Unsafe in general:
    /// `\l` before a letter is a different command. Use it when the caller
    /// knows what follows every value.
    NoProtection,
}

/// What is written around a value whose mode is not the output's mode:
/// `\ensuremath{` … `}` around a math value in text output, `\text{` … `}` or
/// the like around a text value in math output.
///
/// A wrapper may itself need something in the preamble — `\text{…}` needs
/// amsmath — which is what [`needs`](ModeWrapper::needs) states.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModeWrapper {
    /// What is written before the value.
    pub open: Cow<'static, str>,
    /// What is written after the value.
    pub close: Cow<'static, str>,
    /// What the wrapper itself needs in the preamble, reported to the
    /// [`EncodeReporter`] whenever the wrapper is used.
    pub needs: Option<&'static Profile>,
}

impl ModeWrapper {
    /// The wrapper `open` … `close`, needing nothing in the preamble.
    pub const fn new(open: &'static str, close: &'static str) -> Self {
        ModeWrapper { open: Cow::Borrowed(open), close: Cow::Borrowed(close), needs: None }
    }

    /// The wrapper `open` … `close`, which itself needs `needs` in the
    /// preamble.
    pub const fn with_needs(
        open: &'static str,
        close: &'static str,
        needs: &'static Profile,
    ) -> Self {
        ModeWrapper {
            open: Cow::Borrowed(open),
            close: Cow::Borrowed(close),
            needs: Some(needs),
        }
    }
}

/// The standard protection strategy: mode wrapping where the value's mode is
/// not the output's, and macro-name protection where the value ends with a
/// named macro.
///
/// Build it with [`text_mode`](StandardProtection::text_mode) or
/// [`math_mode`](StandardProtection::math_mode) and change the fields as
/// needed; every field is public and can be set at run time.
///
/// ```
/// use untechxt::{
///     NoReport, ProtectInput, ReplacementProtection, ReplacementProtectionHint,
///     StandardProtection,
/// };
///
/// let protection = StandardProtection::text_mode();
/// let write = |encoded: &str, hint| {
///     let mut out = String::new();
///     protection
///         .write_protected(&mut out, &mut NoReport, ProtectInput::new(encoded, hint))
///         .unwrap();
///     out
/// };
/// assert_eq!(write(r"\textemdash", ReplacementProtectionHint::text_only(r"\textemdash")),
///            r"{\textemdash}");
/// assert_eq!(write(r"\'e", ReplacementProtectionHint::text_only(r"\'e")), r"\'e");
/// assert_eq!(write(r"\alpha", ReplacementProtectionHint::math_only(r"\alpha")),
///            r"\ensuremath{\alpha}");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StandardProtection {
    /// Which mode the encoded output is going into.
    pub output_mode: OutputMode,
    /// What to write around a value that ends with a named macro.
    pub protect_names: MacroNameProtection,
    /// What to write around a math value in text output.
    pub math_wrap: ModeWrapper,
    /// What to write around a text value in math output.
    pub text_wrap: ModeWrapper,
}

impl StandardProtection {
    /// The strategy for output that goes into ordinary document text: math
    /// values are wrapped in `\ensuremath{…}`, and a value ending with a
    /// named macro is wrapped in braces.
    pub const fn text_mode() -> Self {
        StandardProtection {
            output_mode: OutputMode::TextMode,
            protect_names: MacroNameProtection::BracesAround,
            math_wrap: ModeWrapper::new("\\ensuremath{", "}"),
            text_wrap: ModeWrapper::new("\\textnormal{", "}"),
        }
    }

    /// The strategy for output that goes inside mathematics: text values are
    /// wrapped in [`text_wrap`](StandardProtection::text_wrap), and a value
    /// ending with a named macro is followed by a space, which math mode
    /// ignores.
    pub const fn math_mode() -> Self {
        StandardProtection {
            output_mode: OutputMode::MathMode,
            protect_names: MacroNameProtection::SpaceAfterMacroName,
            math_wrap: ModeWrapper::new("\\ensuremath{", "}"),
            text_wrap: ModeWrapper::new("\\textnormal{", "}"),
        }
    }

    /// The wrapper this strategy applies to a value with this hint: `None`
    /// for [`DoNotProtect`](ReplacementProtectionHint::DoNotProtect) and for
    /// a value whose mode is valid in the output mode.
    ///
    /// It is public so that a custom strategy — [`BracesAroundAll`] is the
    /// example — can reuse the mode handling.
    pub fn mode_wrapper_for(&self, hint: ReplacementProtectionHint) -> Option<&ModeWrapper> {
        match hint {
            ReplacementProtectionHint::DoNotProtect => None,
            ReplacementProtectionHint::Value { mode, .. } => match (self.output_mode, mode) {
                (_, ValueMode::AnyMode)
                | (OutputMode::TextMode, ValueMode::TextOnly)
                | (OutputMode::MathMode, ValueMode::MathOnly) => None,
                (OutputMode::TextMode, ValueMode::MathOnly) => Some(&self.math_wrap),
                (OutputMode::MathMode, ValueMode::TextOnly) => Some(&self.text_wrap),
            },
        }
    }

    /// Writes `encoded` with the macro-name protection of this strategy
    /// applied, whatever the value's mode. A wrapped value does not need it.
    fn write_name_protected<O: OutBuffer>(
        &self,
        out: &mut O,
        encoded: &str,
        termination: ValueTermination,
    ) -> Result<(), BoxError> {
        if termination == ValueTermination::ValueIsSelfTerminating {
            return out.push_str(encoded);
        }
        match self.protect_names {
            MacroNameProtection::BracesAround => {
                out.push_str("{")?;
                out.push_str(encoded)?;
                out.push_str("}")
            }
            MacroNameProtection::BracesAfter => {
                out.push_str(encoded)?;
                out.push_str("{}")
            }
            MacroNameProtection::SpaceAfterMacroName => {
                out.push_str(encoded)?;
                out.push_str(" ")
            }
            MacroNameProtection::NoProtection => out.push_str(encoded),
        }
    }
}

impl Default for StandardProtection {
    /// [`text_mode`](StandardProtection::text_mode).
    fn default() -> Self {
        StandardProtection::text_mode()
    }
}

impl ReplacementProtection for StandardProtection {
    fn write_protected<O: OutBuffer, Rep: EncodeReporter>(
        &self,
        out: &mut O,
        report: &mut Rep,
        item: ProtectInput<'_>,
    ) -> Result<(), BoxError> {
        let hint = item.hint();
        match hint {
            ReplacementProtectionHint::DoNotProtect => out.push_str(item.encoded()),
            ReplacementProtectionHint::Value { termination, .. } => {
                match self.mode_wrapper_for(hint) {
                    // A wrapped value is self-terminating: the wrapper closes
                    // it, so no macro-name protection is added.
                    Some(wrap) => {
                        if let Some(needs) = wrap.needs {
                            report.report_needs(needs);
                        }
                        out.push_str(&wrap.open)?;
                        out.push_str(item.encoded())?;
                        out.push_str(&wrap.close)
                    }
                    None => self.write_name_protected(out, item.encoded(), termination),
                }
            }
        }
    }
}

/// A protection strategy that wraps **every** value in braces, the empty one
/// included, after the mode wrapping of the [`StandardProtection`] it holds.
///
/// This is pylatexenc's `'braces-all'` mode, kept for compatibility with its
/// version-1 encoder. It is not a setting of [`StandardProtection`] but a
/// strategy of its own, and since it is written with public API alone it is
/// also the example of how to write one.
///
/// ```
/// use untechxt::{
///     BracesAroundAll, NoReport, ProtectInput, ReplacementProtection,
///     ReplacementProtectionHint, StandardProtection,
/// };
///
/// let protection = BracesAroundAll(StandardProtection::text_mode());
/// let mut out = String::new();
/// protection
///     .write_protected(
///         &mut out,
///         &mut NoReport,
///         ProtectInput::new(r"\'e", ReplacementProtectionHint::text_only(r"\'e")),
///     )
///     .unwrap();
/// assert_eq!(out, r"{\'e}");
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct BracesAroundAll(pub StandardProtection);

impl ReplacementProtection for BracesAroundAll {
    fn write_protected<O: OutBuffer, Rep: EncodeReporter>(
        &self,
        out: &mut O,
        report: &mut Rep,
        item: ProtectInput<'_>,
    ) -> Result<(), BoxError> {
        if item.hint() == ReplacementProtectionHint::DoNotProtect {
            return out.push_str(item.encoded());
        }
        let wrap = self.0.mode_wrapper_for(item.hint());
        if let Some(needs) = wrap.and_then(|wrap| wrap.needs) {
            report.report_needs(needs);
        }
        out.push_str("{")?;
        if let Some(wrap) = wrap {
            out.push_str(&wrap.open)?;
        }
        out.push_str(item.encoded())?;
        if let Some(wrap) = wrap {
            out.push_str(&wrap.close)?;
        }
        out.push_str("}")
    }
}
