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

/// The result of applying a rule at one position. It is one of three
/// outcomes:
///
/// - `Ok(Some(replacement))`, the rule matched and returns the
///   [`EncodedReplacement`] for what it consumed;
/// - `Ok(None)`, the rule did not match at this position; or
/// - `Err(error)`, the rule encountered an error, which stops the encoding
///   with [`EncodeError::Rule`](crate::EncodeError::Rule).
pub type RuleResult<'a> = Result<Option<EncodedReplacement<'a>>, BoxError>;

/// One rule of an encoder. A rule takes a position in the input, through a
/// [`RuleInput`], and returns the LaTeX that replaces the input it consumes
/// at that position, or reports that it does not match there.
///
/// The encoder tries its rules in order at every position, and the first
/// match wins: no rule is tried after a match, and no longest match is
/// sought. Combine several rules into one with the [`RuleChain`] struct. For
/// matching that this fixed order cannot express, order the rules or write a
/// single rule so that the first match is the one you want.
///
/// There are three ways to build a rule:
///
/// - the [`rule_fn`] function, which builds a rule from a closure;
/// - the [`TableRule`](crate::lookuptable::TableRule) struct, which builds a
///   rule from a custom
///   [`LookupTable`](crate::lookuptable::LookupTable); and
/// - a custom type that implements [`Rule`] directly, for a rule that reads
///   several characters ahead or borrows from its own state.
///
/// The crate's own lookup tables implement [`Rule`] already. A shared
/// reference `&R`, a [`Box<R>`](alloc::boxed::Box), and an `Option<R>` are
/// also rules whenever `R` is a rule. An `Option<R>` that is `None` never
/// matches, which switches a rule off without changing the type of the chain
/// it sits in.
///
/// The only supertrait is [`Debug`](core::fmt::Debug). This trait does not
/// require `Send` or `Sync`. Those are auto traits, so an
/// [`Encoder`](crate::Encoder) is `Send` and `Sync` exactly when its rules
/// are, and requiring them here would rule out a rule that holds a
/// JavaScript callback, an [`Rc`](alloc::rc::Rc), or a
/// [`RefCell`](core::cell::RefCell).
///
/// ```
/// use untechxt::protection::ReplacementProtectionHint as Hint;
/// use untechxt::rule::{rule_fn, Rule, RuleInput};
///
/// // A rule that encodes an ellipsis typed as three periods.
/// let ellipsis = rule_fn(|input: RuleInput<'_>| {
///     Ok(if input.rest().starts_with("...") {
///         Some(input.replace_prefix(3, r"\ldots", Hint::any_mode(r"\ldots")))
///     } else {
///         None
///     })
/// });
/// let input = RuleInput::new("wait...", 4).unwrap();
/// assert_eq!(ellipsis.apply(input).unwrap().unwrap().encoded(), r"\ldots");
/// ```
pub trait Rule: fmt::Debug {
    /// Tries the rule at the position that `input` names, and returns a
    /// [`RuleResult`]: `Ok(Some(_))` for a match, `Ok(None)` for no match,
    /// or `Err(_)` for an error.
    ///
    /// The single lifetime `'a` covers both the rule and the input, so the
    /// returned value may borrow from either without allocating. A lookup
    /// table returns its own static string, and a rule that passes existing
    /// LaTeX through returns a slice of the input.
    ///
    /// # Errors
    ///
    /// Returns an error when the rule cannot do its work, for example when
    /// a foreign-language callback raised an exception. The error stops the
    /// encoding with [`EncodeError::Rule`](crate::EncodeError::Rule).
    fn apply<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a>;

    /// Returns the set of ASCII characters that this rule may match at. The
    /// set is a promise that the rule never matches at an ASCII character
    /// outside it.
    ///
    /// The encoder calls this method once, in
    /// [`Encoder::new`](crate::Encoder::new), takes the union of its rules'
    /// sets, and copies runs of ASCII input outside that union without
    /// decoding characters or calling any rule. The returned set must
    /// therefore be the same on every call. The promise holds in one
    /// direction only: the encoder may still try the rule at other
    /// characters, where a correct rule returns `Ok(None)`.
    ///
    /// The default implementation returns [`AsciiSet::ALL`], meaning that
    /// the rule may match at any ASCII character. This default is always
    /// correct and only makes the scan slower. A rule that matches at a
    /// known, smaller set of ASCII characters should override it, and a rule
    /// that never matches at an ASCII character, such as a lookup table of
    /// non-ASCII entries, returns [`AsciiSet::EMPTY`].
    fn ascii_triggers(&self) -> AsciiSet {
        AsciiSet::ALL
    }
}

/// The position at which a rule is being tried: the whole input, the byte
/// offset into it, and the character at that offset.
///
/// A [`RuleInput`] is a small [`Copy`] struct. It exposes its parts through
/// accessor methods rather than public fields, so that later versions can
/// give rules more context without breaking existing rules. It is also the
/// only way to build an [`EncodedReplacement`], through the methods
/// [`replace_char`](RuleInput::replace_char),
/// [`replace_prefix`](RuleInput::replace_prefix), and
/// [`try_replace_prefix`](RuleInput::try_replace_prefix). This keeps a rule
/// from reporting a byte count that is not a valid advance over the input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuleInput<'s> {
    /// The whole input the encoder is encoding, after normalization.
    full: &'s str,
    /// The byte offset into `full` at which the rule is tried. It always
    /// lies on a character boundary.
    pos: usize,
    /// The character at `pos`, decoded once by the encoder.
    ch: char,
}

impl<'s> RuleInput<'s> {
    /// Creates a [`RuleInput`] at byte `pos` of `full`, or returns `None`
    /// when `pos` is not the start of a character in `full`. The encoder
    /// builds its own inputs, so this constructor is for testing a rule or
    /// calling one directly.
    pub fn new(full: &'s str, pos: usize) -> Option<Self> {
        let ch = full.get(pos..)?.chars().next()?;
        Some(RuleInput { full, pos, ch })
    }

    /// Creates a [`RuleInput`] at byte `pos` of `full`, where `ch` is
    /// already known to be the character at that offset. This is what the
    /// encoder builds, having decoded `ch` in its scan.
    pub(crate) const fn at(full: &'s str, pos: usize, ch: char) -> Self {
        RuleInput { full, pos, ch }
    }

    /// Returns the character at the position, already decoded.
    pub const fn ch(&self) -> char {
        self.ch
    }

    /// Returns the byte offset of the position in the normalized input.
    pub const fn pos(&self) -> usize {
        self.pos
    }

    /// Returns the whole normalized input.
    pub const fn full(&self) -> &'s str {
        self.full
    }

    /// Returns the input from the position onward, `&full()[pos()..]`, for
    /// lookahead. Its first character is the one that the method
    /// [`ch`](RuleInput::ch) returns.
    pub fn rest(&self) -> &'s str {
        &self.full[self.pos..]
    }

    /// Returns the input before the position, `&full()[..pos()]`, for
    /// lookbehind.
    pub fn before(&self) -> &'s str {
        &self.full[..self.pos]
    }

    /// Returns an [`EncodedReplacement`] that consumes exactly the character
    /// at the position and replaces it with `encoded`, described to the
    /// protection strategy by `hint`. This replacement is always valid,
    /// whatever the input.
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

    /// Returns an [`EncodedReplacement`] that consumes the first `n_bytes`
    /// bytes of [`rest`](RuleInput::rest) and replaces them with `encoded`,
    /// described to the protection strategy by `hint`.
    ///
    /// # Panics
    ///
    /// Panics unless `n_bytes` is non-zero, no larger than the length of
    /// [`rest`](RuleInput::rest), and on a character boundary. These are the
    /// same requirements as slicing the input, so a violation is a bug in the
    /// rule. Use [`try_replace_prefix`](RuleInput::try_replace_prefix) to
    /// report an invalid length instead of panicking.
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

    /// Works like the method [`replace_prefix`](RuleInput::replace_prefix),
    /// but returns an error for an invalid length instead of panicking. The
    /// `?` operator turns that error into a rule error. Use this method when
    /// `n_bytes` comes from outside the rule, such as from a rule driven by
    /// another language, where the rule author cannot check the length.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidPrefixLength`] when `n_bytes` is zero, reaches past
    /// the end of the input, or ends inside a character.
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

/// The result of a rule that matched: how much input the rule consumed, the
/// LaTeX that replaces that input, how that LaTeX must be protected, and
/// what that LaTeX needs in the preamble.
///
/// The fields are private. The only way to build an [`EncodedReplacement`]
/// is through a [`RuleInput`], with the method
/// [`replace_char`](RuleInput::replace_char),
/// [`replace_prefix`](RuleInput::replace_prefix), or
/// [`try_replace_prefix`](RuleInput::try_replace_prefix). A byte count that
/// is not a valid advance over the input therefore cannot be built, and the
/// encoder needs no check of its own.
///
/// The lifetime `'a` covers the encoded text and the profile alike. Each of
/// them may belong to the rule, borrow from the input, or be `'static`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedReplacement<'a> {
    /// The number of input bytes consumed. It is never zero, and it always
    /// ends on a character boundary within the input.
    consumed: usize,
    /// The LaTeX that replaces the consumed input.
    encoded: Cow<'a, str>,
    /// What the protection strategy must know about that LaTeX.
    hint: ReplacementProtectionHint,
    /// What the LaTeX needs in the preamble, if anything.
    needs: Option<&'a Profile>,
}

impl<'a> EncodedReplacement<'a> {
    /// Returns the same replacement, recording that its LaTeX needs
    /// `profile` in the document's preamble.
    #[must_use]
    pub fn with_needs(mut self, profile: &'a Profile) -> Self {
        self.needs = Some(profile);
        self
    }

    /// Returns the number of input bytes the rule consumed.
    pub const fn consumed(&self) -> usize {
        self.consumed
    }

    /// Returns the LaTeX that replaces the consumed input, with no
    /// protection applied.
    pub fn encoded(&self) -> &str {
        &self.encoded
    }

    /// Returns the [`ReplacementProtectionHint`] the rule gave for that
    /// LaTeX.
    pub const fn hint(&self) -> ReplacementProtectionHint {
        self.hint
    }

    /// Returns what the LaTeX needs in the document's preamble, or `None`.
    pub const fn needs(&self) -> Option<&'a Profile> {
        self.needs
    }
}

/// The error returned when a rule asks to consume a number of bytes that is
/// not a valid advance over the input: zero bytes, more bytes than the input
/// holds, or a count that ends inside a character.
///
/// The method [`try_replace_prefix`](RuleInput::try_replace_prefix) returns
/// this error, and the method [`replace_prefix`](RuleInput::replace_prefix)
/// panics with it instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InvalidPrefixLength {
    /// The byte offset into the input at which the rule was tried.
    pub position: usize,
    /// The number of bytes the rule asked to consume.
    pub n_bytes: usize,
    /// The number of bytes the input holds from `position` onward.
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

/// A rule built from the closure or function `f`.
///
/// Build a [`RuleFn`] with the [`rule_fn`] function, which describes the
/// requirements on `f`. By default the rule may match at any ASCII
/// character; restrict that with the method
/// [`with_ascii_triggers`](RuleFn::with_ascii_triggers).
pub struct RuleFn<F> {
    f: F,
    ascii_triggers: AsciiSet,
}

impl<F> RuleFn<F> {
    /// Returns the same rule with its ASCII trigger set replaced by `set`,
    /// promising that the rule never matches at an ASCII character outside
    /// `set`. This lets the encoder copy runs of the other ASCII characters
    /// without calling the rule. See the method [`Rule::ascii_triggers`] for
    /// the contract that `set` must satisfy.
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

/// Builds a [`RuleFn`] that applies the closure or function `f` at every
/// position.
///
/// The closure takes a [`RuleInput`] and returns a [`RuleResult`]: a
/// replacement built through that input for a match, `Ok(None)` where the
/// rule does not apply, or an error that stops the encoding.
///
/// The closure may return owned strings, string literals, `&'static
/// Profile` references, and slices of the input, but it cannot return a
/// value borrowed from its own captures. For a rule that borrows from its
/// own state, implement the [`Rule`] trait on a custom struct instead. The
/// higher-ranked lifetime bound on `f` states this: the returned value may
/// borrow from the input, which is a different input at each position.
///
/// By default the rule may match at any ASCII character. Restrict that with
/// the method [`RuleFn::with_ascii_triggers`].
///
/// ```
/// use untechxt::protection::ReplacementProtectionHint as Hint;
/// use untechxt::rule::{rule_fn, RuleInput};
/// use untechxt::Encoder;
///
/// // Pass LaTeX that is already in the input through untouched.
/// let verbatim = rule_fn(|input: RuleInput<'_>| {
///     let rest = input.rest();
///     Ok(rest.strip_prefix("[[").and_then(|rest| {
///         rest.find("]]").map(|end| {
///             input.replace_prefix(end + 4, &rest[..end], Hint::DoNotProtect)
///         })
///     }))
/// });
/// let encoder = Encoder::new(verbatim);
/// assert_eq!(
///     encoder.encode(r"a [[\textbf{b}]] c").unwrap(),
///     r"a \textbf{b} c",
/// );
/// ```
pub fn rule_fn<F>(f: F) -> RuleFn<F>
where
    F: for<'s> Fn(RuleInput<'s>) -> RuleResult<'s>,
{
    RuleFn { f, ascii_triggers: AsciiSet::ALL }
}

/// A shared reference to a rule is itself a rule, and forwards each call to
/// the rule it refers to.
impl<R: Rule + ?Sized> Rule for &R {
    fn apply<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a> {
        (**self).apply(input)
    }

    fn ascii_triggers(&self) -> AsciiSet {
        (**self).ascii_triggers()
    }
}

/// A boxed rule is itself a rule, and forwards each call to the rule in the
/// box.
impl<R: Rule + ?Sized> Rule for Box<R> {
    fn apply<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a> {
        (**self).apply(input)
    }

    fn ascii_triggers(&self) -> AsciiSet {
        (**self).ascii_triggers()
    }
}

/// An optional rule is itself a rule, which switches a rule off without
/// changing the type of the chain it sits in. `Some(rule)` forwards each
/// call to `rule`, and `None` never matches and triggers on no ASCII
/// character.
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
