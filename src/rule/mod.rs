//! Rules: the [`Rule`] trait, the types a rule takes and returns, and the
//! ways to build a rule.
//!
//! A rule maps an input character, or an input substring, to its LaTeX
//! encoding. This module contains everything that is specific to rules:
//!
//! - The [`Rule`] trait itself. A rule takes a [`RuleInput`] and returns a
//!   [`RuleResult`], which contains an [`EncodedReplacement`] when the rule
//!   matched.
//! - The function [`rule_fn`], which creates a rule from a closure (see
//!   [`RuleFn`]).
//! - The struct [`RuleChain`], which combines several rules into a single
//!   rule. The type aliases [`DynRuleChain`] and [`LocalDynRuleChain`] are
//!   chains of boxed rules that are assembled at run time.
//! - The struct [`AsciiSet`], a set of ASCII characters. A rule uses an
//!   [`AsciiSet`] to tell the encoder at which ASCII characters the rule may
//!   match (see [`Rule::ascii_triggers`]).
//!
//! Lookup tables are rules as well. See the [`lookuptable`](crate::lookuptable)
//! module for tables built at run time, the
//! [`statictable`](crate::statictable) module for tables compiled at compile
//! time, and the [`builtin`](crate::builtin) module for the tables that come
//! with the crate.

mod asciiset;
mod chain;

use alloc::borrow::Cow;
use alloc::boxed::Box;
use core::fmt;

use crate::preamble::Profile;
use crate::protection::ReplacementProtectionHint;
use crate::BoxError;

pub use self::asciiset::AsciiSet;
pub use self::chain::{DynRuleChain, LocalDynRuleChain, RuleChain, RuleList};

/// What a rule call reports. The rule may report (i) that it was successfully
/// applied, returning the associated success data (`Ok(Some(…))`, see
/// [`EncodedReplacement`]); (ii) that it did not match at this position
/// (`Ok(None)`); or (iii) that it would normally apply but encountered a
/// critical error that should be reported (`Err(…)`), which stops the
/// encoding.
pub type RuleResult<'a> = Result<Option<EncodedReplacement<'a>>, BoxError>;

/// One rule of an encoder: it is offered a position in the input and answers
/// with the LaTeX that replaces what it consumed there, or with "not mine".
///
/// The encoder tries its rules in order at every position and the first match
/// wins: no rule is tried after a match, and no longest match is sought.
/// Several rules are combined into one with a
/// [`RuleChain`].
///
/// [`Debug`](core::fmt::Debug) is the only supertrait. `Send` and `Sync` are
/// deliberately **not** required: they are auto traits, so an
/// [`Encoder`](crate::Encoder) is `Send`/`Sync` exactly when its rules are,
/// and requiring them would forbid rules that hold a JavaScript callback, an
/// [`Rc`](alloc::rc::Rc) or a [`RefCell`](core::cell::RefCell).
///
/// A closure becomes a rule through [`rule_fn`]; a lookup table of the user's
/// through [`TableRule`](crate::lookuptable::TableRule). `&R`, `Box<R>` and
/// `Option<R>` are rules whenever `R` is.
///
/// ```
/// use untechxt::protection::ReplacementProtectionHint;
/// use untechxt::rule::{rule_fn, Rule, RuleInput};
///
/// // A rule that spells out an ellipsis, however it was typed.
/// let ellipsis = rule_fn(|input: RuleInput<'_>| {
///     Ok(if input.rest().starts_with("...") {
///         Some(input.replace_prefix(3, r"\ldots", ReplacementProtectionHint::any_mode(r"\ldots")))
///     } else {
///         None
///     })
/// });
/// let input = RuleInput::new("wait...", 4).unwrap();
/// assert_eq!(ellipsis.apply(input).unwrap().unwrap().encoded(), r"\ldots");
/// ```
pub trait Rule: fmt::Debug {
    /// Tries the rule at the position `input` names.
    ///
    /// The one lifetime `'a` covers both the rule and the input, so that the
    /// value returned may borrow from either: a table lends out its own
    /// static string, a rule that passes existing LaTeX through lends out a
    /// slice of the input, and neither allocates.
    ///
    /// # Errors
    ///
    /// A rule that cannot do its work — a foreign-language callback that
    /// raised — reports it here, and the encoding stops with
    /// [`EncodeError::Rule`](crate::EncodeError::Rule).
    fn apply<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a>;

    /// The ASCII characters this rule may match at: a promise that it never
    /// matches at an ASCII character outside this set.
    ///
    /// The encoder asks once, in [`Encoder::new`](crate::Encoder::new), and
    /// copies runs of ASCII outside the union of its rules' sets without
    /// decoding characters or calling anyone. The answer must therefore
    /// always be the same. It is a promise in one direction only: the encoder
    /// may still consult the rule at other characters, where a correct rule
    /// declines.
    ///
    /// The default is [`AsciiSet::ALL`]: "I may match anywhere".
    fn ascii_triggers(&self) -> AsciiSet {
        AsciiSet::ALL
    }
}

/// Where a rule is being tried: the whole input, the byte position in it, and
/// the character there.
///
/// It is a small `Copy` struct with accessors rather than public fields, so
/// that more context can be given to rules later without breaking the ones
/// that exist. It is also the only way to build an [`EncodedReplacement`],
/// which is how a rule cannot report a consumption that is not a valid
/// advance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuleInput<'s> {
    /// The whole input the encoder is working on, already normalized.
    full: &'s str,
    /// The byte offset in `full` the rule is tried at; always a character
    /// boundary.
    pos: usize,
    /// The character at `pos`, decoded once by the encoder.
    ch: char,
}

impl<'s> RuleInput<'s> {
    /// The input at byte `pos` of `full`, or `None` when `pos` is not the
    /// start of a character of `full`. The encoder builds its own; this is
    /// for testing a rule and for calling one directly.
    pub fn new(full: &'s str, pos: usize) -> Option<Self> {
        let ch = full.get(pos..)?.chars().next()?;
        Some(RuleInput { full, pos, ch })
    }

    /// The input at byte `pos` of `full`, where `ch` is known to be the
    /// character there. What the encoder builds, having decoded `ch` already.
    pub(crate) const fn at(full: &'s str, pos: usize, ch: char) -> Self {
        RuleInput { full, pos, ch }
    }

    /// The character at the position, already decoded.
    pub const fn ch(&self) -> char {
        self.ch
    }

    /// The byte offset of the position in the normalized input.
    pub const fn pos(&self) -> usize {
        self.pos
    }

    /// The whole normalized input.
    pub const fn full(&self) -> &'s str {
        self.full
    }

    /// The input from the position on, `&full()[pos()..]`, for lookahead. Its
    /// first character is [`ch`](RuleInput::ch).
    pub fn rest(&self) -> &'s str {
        &self.full[self.pos..]
    }

    /// The input before the position, `&full()[..pos()]`, for lookbehind.
    pub fn before(&self) -> &'s str {
        &self.full[..self.pos]
    }

    /// A replacement that consumes exactly the character at the position.
    /// Always valid, whatever the input.
    pub fn replace_char<'a>(
        &self,
        encoded: impl Into<Cow<'a, str>>,
        hint: ReplacementProtectionHint,
    ) -> EncodedReplacement<'a> {
        EncodedReplacement {
            consumed: self.ch.len_utf8(),
            encoded: encoded.into(),
            hint,
            needs: None,
        }
    }

    /// A replacement that consumes the first `n_bytes` bytes of
    /// [`rest`](RuleInput::rest).
    ///
    /// # Panics
    ///
    /// Unless `n_bytes` is non-zero, no more than the length of
    /// [`rest`](RuleInput::rest), and on a character boundary — the same
    /// requirements as slicing the input, and a bug in the rule.
    /// [`try_replace_prefix`](RuleInput::try_replace_prefix) reports them
    /// instead.
    pub fn replace_prefix<'a>(
        &self,
        n_bytes: usize,
        encoded: impl Into<Cow<'a, str>>,
        hint: ReplacementProtectionHint,
    ) -> EncodedReplacement<'a> {
        match self.try_replace_prefix(n_bytes, encoded, hint) {
            Ok(replacement) => replacement,
            Err(err) => panic!("{err}"),
        }
    }

    /// The same as [`replace_prefix`](RuleInput::replace_prefix), reporting
    /// an invalid length instead of panicking; `?` turns the error into a
    /// rule error. This is what a rule driven from another language uses,
    /// where the length is not the rule author's to check.
    ///
    /// # Errors
    ///
    /// [`InvalidPrefixLength`] when `n_bytes` is zero, reaches past the end
    /// of the input, or ends inside a character.
    pub fn try_replace_prefix<'a>(
        &self,
        n_bytes: usize,
        encoded: impl Into<Cow<'a, str>>,
        hint: ReplacementProtectionHint,
    ) -> Result<EncodedReplacement<'a>, InvalidPrefixLength> {
        let rest = self.rest();
        if n_bytes == 0 || n_bytes > rest.len() || !rest.is_char_boundary(n_bytes) {
            return Err(InvalidPrefixLength {
                position: self.pos,
                n_bytes,
                available: rest.len(),
            });
        }
        Ok(EncodedReplacement { consumed: n_bytes, encoded: encoded.into(), hint, needs: None })
    }
}

/// What a matching rule hands back: how much input it consumed, the LaTeX
/// that replaces it, how that LaTeX must be protected, and what it needs in
/// the preamble.
///
/// The fields are private and the only way to build one is through
/// [`RuleInput`] ([`replace_char`](RuleInput::replace_char),
/// [`replace_prefix`](RuleInput::replace_prefix),
/// [`try_replace_prefix`](RuleInput::try_replace_prefix)), so that a
/// consumption that is not a valid advance cannot be constructed and the
/// encoder needs no check of its own.
///
/// The lifetime covers the encoded text and the profile alike: both may
/// belong to the rule, or to the input, or be `'static`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedReplacement<'a> {
    /// The number of input bytes consumed; never zero, always ending on a
    /// character boundary within the input.
    consumed: usize,
    /// The LaTeX that replaces them.
    encoded: Cow<'a, str>,
    /// What the protection strategy must know about that LaTeX.
    hint: ReplacementProtectionHint,
    /// What the LaTeX needs in the preamble, if anything.
    needs: Option<&'a Profile>,
}

impl<'a> EncodedReplacement<'a> {
    /// The same replacement, stating that its LaTeX needs `profile` in the
    /// document's preamble.
    #[must_use]
    pub fn with_needs(mut self, profile: &'a Profile) -> Self {
        self.needs = Some(profile);
        self
    }

    /// The number of input bytes the rule consumed.
    pub const fn consumed(&self) -> usize {
        self.consumed
    }

    /// The LaTeX that replaces them, with no protection applied.
    pub fn encoded(&self) -> &str {
        &self.encoded
    }

    /// What the rule said about that LaTeX.
    pub const fn hint(&self) -> ReplacementProtectionHint {
        self.hint
    }

    /// What the LaTeX needs in the document's preamble, if anything.
    pub const fn needs(&self) -> Option<&'a Profile> {
        self.needs
    }
}

/// A rule asked to consume a number of bytes that is not a valid advance:
/// zero bytes, more than the input holds, or a count ending inside a
/// character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InvalidPrefixLength {
    /// The byte position in the input the rule was tried at.
    pub position: usize,
    /// The number of bytes it asked to consume.
    pub n_bytes: usize,
    /// The number of bytes the input holds from `position` on.
    pub available: usize,
}

impl fmt::Display for InvalidPrefixLength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "a rule asked to consume {} bytes at byte {}, of the {} that remain; \
             a rule must consume at least one whole character",
            self.n_bytes, self.position, self.available
        )
    }
}

impl core::error::Error for InvalidPrefixLength {}

/// A rule made of the closure or function `f`.
///
/// Build one with [`rule_fn`], which is where the requirements on `f` are
/// described.
pub struct RuleFn<F> {
    f: F,
    ascii_triggers: AsciiSet,
}

impl<F> RuleFn<F> {
    /// The same rule, promising that it never matches at an ASCII character
    /// outside `set` — which lets the encoder copy runs of the other ASCII
    /// characters without calling it. See
    /// [`Rule::ascii_triggers`].
    #[must_use]
    pub fn with_ascii_triggers(mut self, set: AsciiSet) -> Self {
        self.ascii_triggers = set;
        self
    }
}

impl<F> fmt::Debug for RuleFn<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RuleFn(..)")
    }
}

impl<F> Rule for RuleFn<F>
where
    F: for<'s> Fn(RuleInput<'s>) -> RuleResult<'s>,
{
    fn apply<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a> {
        (self.f)(input)
    }

    fn ascii_triggers(&self) -> AsciiSet {
        self.ascii_triggers
    }
}

/// The rule that applies the closure or function `f` at every position.
///
/// The closure is given a [`RuleInput`] and answers a [`RuleResult`]: a
/// replacement built through the input, `None` where it does not apply, or an
/// error that stops the encoding.
///
/// It may hand out owned strings, string literals, `&'static Profile`s and
/// slices of the input, but it cannot lend out its own captures — a rule that
/// lends from its own state implements [`Rule`] on a struct instead. (This is
/// what the higher-ranked bound says: the answer may borrow from the input,
/// which is a different input every time.)
///
/// ```
/// use untechxt::protection::ReplacementProtectionHint;
/// use untechxt::rule::rule_fn;
/// use untechxt::Encoder;
///
/// // Pass LaTeX that is already in the input through untouched.
/// let verbatim = rule_fn(|input: untechxt::rule::RuleInput<'_>| {
///     let rest = input.rest();
///     Ok(rest.strip_prefix("[[").and_then(|rest| rest.find("]]").map(|end| {
///         input.replace_prefix(
///             end + 4,
///             &rest[..end],
///             ReplacementProtectionHint::DoNotProtect,
///         )
///     })))
/// });
/// let encoder = Encoder::new(verbatim);
/// assert_eq!(encoder.encode(r"a [[\textbf{b}]] c").unwrap(), r"a \textbf{b} c");
/// ```
pub fn rule_fn<F>(f: F) -> RuleFn<F>
where
    F: for<'s> Fn(RuleInput<'s>) -> RuleResult<'s>,
{
    RuleFn { f, ascii_triggers: AsciiSet::ALL }
}

impl<R: Rule + ?Sized> Rule for &R {
    fn apply<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a> {
        (**self).apply(input)
    }

    fn ascii_triggers(&self) -> AsciiSet {
        (**self).ascii_triggers()
    }
}

impl<R: Rule + ?Sized> Rule for Box<R> {
    fn apply<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a> {
        (**self).apply(input)
    }

    fn ascii_triggers(&self) -> AsciiSet {
        (**self).ascii_triggers()
    }
}

/// A rule that can be switched off without changing the type of the chain it
/// is in: `None` never matches.
impl<R: Rule> Rule for Option<R> {
    fn apply<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a> {
        match self {
            Some(rule) => rule.apply(input),
            None => Ok(None),
        }
    }

    fn ascii_triggers(&self) -> AsciiSet {
        match self {
            Some(rule) => rule.ascii_triggers(),
            None => AsciiSet::EMPTY,
        }
    }
}
