//! The [`RuleChain`] struct: several rules combined into one rule, tried
//! in order. The [`RuleList`] trait describes the collections that hold a
//! chain's member rules.

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::rule::{AsciiSet, Rule, RuleInput, RuleResult};

/// Several rules combined into one rule: the member rules are tried in
/// order at every position, and the first match wins.
///
/// The type parameter `L` is the list of member rules, which implements the
/// [`RuleList`] trait. It is one of:
///
/// - a tuple of up to twelve rules, which keeps each member rule's own type
///   and which the compiler may unroll and inline;
/// - an array `[R; N]` of rules that share one type; or
/// - a [`Vec`] of rules, for a chain whose members are assembled at run
///   time.
///
/// The empty tuple `()` is the chain that never matches. For a chain of
/// boxed rules assembled at run time, see the type aliases [`DynRuleChain`]
/// and [`LocalDynRuleChain`].
///
/// A [`RuleChain`] is itself a [`Rule`], so one chain can be a member rule
/// of another. It is a struct rather than an implementation of [`Rule`] on
/// bare tuples, which keeps exactly one way to build a chain and leaves room
/// for chain-level options in the future.
///
/// Because the first match wins, a rule placed earlier in the chain
/// overrides a rule placed later for the same input. For matching that a
/// fixed order cannot express, such as always taking the longest match,
/// order the member rules or write a single [`Rule`] so that the first match
/// is the desired one.
///
/// ```
/// use untechxt::lookuptable::DynTable;
/// use untechxt::protection::ReplacementProtectionHint as Hint;
/// use untechxt::rule::RuleChain;
/// use untechxt::Encoder;
///
/// let mut overrides = DynTable::new();
/// overrides.insert('%', r"\textpercent", Hint::text_only(r"\textpercent"));
/// let mut fallback = DynTable::new();
/// fallback.insert('%', r"\%", Hint::text_only(r"\%"));
/// // The first matching rule of the chain wins.
/// let encoder = Encoder::new(RuleChain::new((overrides, fallback)));
/// assert_eq!(encoder.encode("50%").unwrap(), r"50{\textpercent}");
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct RuleChain<L> {
    /// The member rules, in the order they are tried. This field is private
    /// so that chain-level options can be added around it later.
    rules: L,
}

impl<L: RuleList> RuleChain<L> {
    /// Creates a chain from `rules`, a [`RuleList`] such as a tuple, an
    /// array, or a [`Vec`] of rules.
    pub const fn new(rules: L) -> Self {
        RuleChain { rules }
    }

    /// Returns a reference to the member rules of the chain.
    pub fn rules(&self) -> &L {
        &self.rules
    }
}

impl<L: RuleList + core::fmt::Debug> Rule for RuleChain<L> {
    fn apply<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a> {
        self.rules.apply_first(input)
    }

    fn ascii_triggers(&self) -> AsciiSet {
        self.rules.ascii_triggers_union()
    }
}

/// The list of member rules that a [`RuleChain`] holds. It is implemented
/// for a tuple of up to twelve rules, an array of rules, and a [`Vec`] of
/// rules.
///
/// This trait is sealed, so it cannot be implemented outside the crate: it
/// covers the fixed set of shapes the crate knows how to iterate over. To
/// build a chain from a collection of rules of your own, collect the rules
/// into a [`Vec`], or implement the [`Rule`] trait on a custom type
/// directly.
pub trait RuleList: sealed::Sealed {
    /// Tries the member rules in order and returns the first match, or
    /// `None` when none of them matched.
    ///
    /// # Errors
    ///
    /// Returns the error of the first member rule that failed. The member
    /// rules after it are not tried.
    fn apply_first<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a>;

    /// Returns the union of the member rules' [`Rule::ascii_triggers`] sets.
    fn ascii_triggers_union(&self) -> AsciiSet;
}

mod sealed {
    /// Restricts [`RuleList`](super::RuleList) to the shapes the crate
    /// implements it for, so that it cannot be implemented elsewhere.
    pub trait Sealed {}
}

impl sealed::Sealed for () {}

/// The empty tuple `()` is the chain that never matches. Its
/// [`Rule::ascii_triggers`] set is empty, so it triggers on no ASCII
/// character.
impl RuleList for () {
    fn apply_first<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a> {
        let _ = input;
        Ok(None)
    }

    fn ascii_triggers_union(&self) -> AsciiSet {
        AsciiSet::EMPTY
    }
}

macro_rules! tuple_rule_list {
    ($($name:ident),+) => {
        impl<$($name: Rule),+> sealed::Sealed for ($($name,)+) {}

        impl<$($name: Rule),+> RuleList for ($($name,)+) {
            #[allow(non_snake_case)]
            fn apply_first<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a> {
                let ($($name,)+) = self;
                $(
                    if let Some(replacement) = $name.apply(input)? {
                        return Ok(Some(replacement));
                    }
                )+
                Ok(None)
            }

            #[allow(non_snake_case)]
            fn ascii_triggers_union(&self) -> AsciiSet {
                let ($($name,)+) = self;
                let mut set = AsciiSet::EMPTY;
                $( set = set.union($name.ascii_triggers()); )+
                set
            }
        }
    };
}

tuple_rule_list!(A);
tuple_rule_list!(A, B);
tuple_rule_list!(A, B, C);
tuple_rule_list!(A, B, C, D);
tuple_rule_list!(A, B, C, D, E);
tuple_rule_list!(A, B, C, D, E, F);
tuple_rule_list!(A, B, C, D, E, F, G);
tuple_rule_list!(A, B, C, D, E, F, G, H);
tuple_rule_list!(A, B, C, D, E, F, G, H, I);
tuple_rule_list!(A, B, C, D, E, F, G, H, I, J);
tuple_rule_list!(A, B, C, D, E, F, G, H, I, J, K);
tuple_rule_list!(A, B, C, D, E, F, G, H, I, J, K, L);

impl<R: Rule, const N: usize> sealed::Sealed for [R; N] {}

impl<R: Rule, const N: usize> RuleList for [R; N] {
    fn apply_first<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a> {
        apply_first_of(self, input)
    }

    fn ascii_triggers_union(&self) -> AsciiSet {
        triggers_union_of(self)
    }
}

impl<R: Rule> sealed::Sealed for Vec<R> {}

impl<R: Rule> RuleList for Vec<R> {
    fn apply_first<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a> {
        apply_first_of(self, input)
    }

    fn ascii_triggers_union(&self) -> AsciiSet {
        triggers_union_of(self)
    }
}

/// Returns the first match among `rules`, tried in order, or `None`.
fn apply_first_of<'a, R: Rule>(rules: &'a [R], input: RuleInput<'a>) -> RuleResult<'a> {
    for rule in rules {
        if let Some(replacement) = rule.apply(input)? {
            return Ok(Some(replacement));
        }
    }
    Ok(None)
}

/// Returns the union of the ASCII trigger sets of `rules`.
fn triggers_union_of<R: Rule>(rules: &[R]) -> AsciiSet {
    let mut set = AsciiSet::EMPTY;
    for rule in rules {
        set = set.union(rule.ascii_triggers());
    }
    set
}

/// A chain of boxed rules assembled at run time that can be sent between
/// threads and shared, because each boxed rule is `Send + Sync`.
///
/// Use this type alias for a chain whose member rules are chosen at run
/// time, such as the chain of a Python binding, or of an encoder built from
/// a configuration file. Start from `DynRuleChain::empty`, then add rules
/// with `push` or `with_rule`.
///
/// The lifetime `'r` is the lifetime of the rules the chain holds. A
/// `DynRuleChain<'static>` holds rules that borrow nothing.
///
/// ```
/// use untechxt::protection::ReplacementProtectionHint as Hint;
/// use untechxt::rule::{rule_fn, DynRuleChain, RuleInput};
/// use untechxt::Encoder;
///
/// let mut chain = DynRuleChain::empty();
/// chain.push(rule_fn(|input: RuleInput<'_>| {
///     Ok((input.ch() == '&')
///         .then(|| input.replace_char(r"\&", Hint::text_only(r"\&"))))
/// }));
/// let encoder = Encoder::new(chain);
/// assert_eq!(encoder.encode("you & me").unwrap(), r"you \& me");
/// ```
pub type DynRuleChain<'r> = RuleChain<Vec<Box<dyn Rule + Send + Sync + 'r>>>;

/// A chain of boxed rules assembled at run time that stays on one thread,
/// because its boxed rules are not required to be `Send + Sync`.
///
/// This type alias is the same as [`DynRuleChain`] without the `Send + Sync`
/// bound. Use it for the chain of a JavaScript binding, or for a chain of a
/// rule that is not thread-safe, such as one that captures an
/// [`Rc`](alloc::rc::Rc) or a [`RefCell`](core::cell::RefCell).
pub type LocalDynRuleChain<'r> = RuleChain<Vec<Box<dyn Rule + 'r>>>;

impl<'r> DynRuleChain<'r> {
    /// Creates a chain with no rules, which never matches. Add rules to it
    /// with `push` or `with_rule`.
    pub fn empty() -> Self {
        RuleChain { rules: Vec::new() }
    }

    /// Boxes `rule` and adds it at the end of the chain. Because the chain
    /// is a [`DynRuleChain`], `rule` must be `Send + Sync`.
    pub fn push<R: Rule + Send + Sync + 'r>(&mut self, rule: R) {
        self.rules.push(Box::new(rule));
    }

    /// Boxes `rule`, adds it at the end of the chain, and returns the
    /// chain, for building a chain in a single expression.
    #[must_use]
    pub fn with_rule<R: Rule + Send + Sync + 'r>(mut self, rule: R) -> Self {
        self.push(rule);
        self
    }
}

impl<'r> LocalDynRuleChain<'r> {
    /// Creates a chain with no rules, which never matches. Add rules to it
    /// with `push` or `with_rule`.
    pub fn empty() -> Self {
        RuleChain { rules: Vec::new() }
    }

    /// Boxes `rule` and adds it at the end of the chain. Because the chain
    /// is a [`LocalDynRuleChain`], `rule` need not be `Send + Sync`.
    pub fn push<R: Rule + 'r>(&mut self, rule: R) {
        self.rules.push(Box::new(rule));
    }

    /// Boxes `rule`, adds it at the end of the chain, and returns the
    /// chain, for building a chain in a single expression.
    #[must_use]
    pub fn with_rule<R: Rule + 'r>(mut self, rule: R) -> Self {
        self.push(rule);
        self
    }
}
