//! [`RuleChain`]: several rules tried in order, as one rule.

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::asciiset::AsciiSet;
use crate::rule::{Rule, RuleInput, RuleResult};

/// Several rules as one: they are tried in order at every position and the
/// first match wins.
///
/// The members are held in an [`RuleList`]: a tuple of up to twelve rules,
/// which the compiler unrolls and inlines and which keeps every rule's own
/// type; an array; or a [`Vec`], for a chain assembled at run time. The empty
/// tuple is the chain that never matches.
///
/// A chain is itself a [`Rule`], so chains nest. It is a struct of its own
/// rather than an implementation of [`Rule`] on bare tuples, which leaves
/// room for chain-level options later and leaves exactly one way to build a
/// chain.
///
/// ```
/// use untechxt::{DynTable, Encoder, ReplacementProtectionHint, RuleChain};
///
/// let mut overrides = DynTable::new();
/// overrides.insert('%', r"\textpercent", ReplacementProtectionHint::text_only(r"\textpercent"));
/// let mut fallback = DynTable::new();
/// fallback.insert('%', r"\%", ReplacementProtectionHint::text_only(r"\%"));
/// // The first rule of the chain wins.
/// let encoder = Encoder::new(RuleChain::new((overrides, fallback)));
/// assert_eq!(encoder.encode("50%").unwrap(), r"50{\textpercent}");
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct RuleChain<L> {
    /// The members, in the order they are tried. Private: room is kept here
    /// for chain-level options.
    rules: L,
}

impl<L: RuleList> RuleChain<L> {
    /// The chain of the rules of `rules`.
    pub const fn new(rules: L) -> Self {
        RuleChain { rules }
    }

    /// The members of the chain.
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

/// What a [`RuleChain`] can be built from: a tuple of up to twelve rules, an
/// array of rules, or a [`Vec`] of rules.
///
/// The trait is sealed: it describes the shapes the crate knows how to walk,
/// and a user's own collection of rules becomes a chain by way of a `Vec` or
/// by implementing [`Rule`] directly.
pub trait RuleList: sealed::Sealed {
    /// Tries the members in order and answers the first match, or `None` when
    /// none of them matched.
    ///
    /// # Errors
    ///
    /// The error of the first member that failed; the members after it are
    /// not tried.
    fn apply_first<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a>;

    /// The union of the members' [`Rule::ascii_triggers`].
    fn ascii_triggers_union(&self) -> AsciiSet;
}

mod sealed {
    /// Keeps [`RuleList`](super::RuleList) closed to the shapes the crate
    /// implements it for.
    pub trait Sealed {}
}

impl sealed::Sealed for () {}

/// The empty chain: it never matches, and it triggers on no ASCII character
/// at all.
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

/// The first match among `rules`, tried in order.
fn apply_first_of<'a, R: Rule>(rules: &'a [R], input: RuleInput<'a>) -> RuleResult<'a> {
    for rule in rules {
        if let Some(replacement) = rule.apply(input)? {
            return Ok(Some(replacement));
        }
    }
    Ok(None)
}

/// The union of the triggers of `rules`.
fn triggers_union_of<R: Rule>(rules: &[R]) -> AsciiSet {
    let mut set = AsciiSet::EMPTY;
    for rule in rules {
        set = set.union(rule.ascii_triggers());
    }
    set
}

/// A chain of rules assembled at run time, each boxed, that can be sent
/// between threads and shared: the chain of a Python binding, or of an
/// encoder built from a configuration file.
///
/// The lifetime is that of the rules it holds; `DynRuleChain<'static>` holds
/// rules that borrow nothing.
///
/// ```
/// use untechxt::{rule_fn, DynRuleChain, Encoder, ReplacementProtectionHint};
///
/// let mut chain = DynRuleChain::empty();
/// chain.push(rule_fn(|input: untechxt::RuleInput<'_>| {
///     Ok((input.ch() == '&').then(|| {
///         input.replace_char(r"\&", ReplacementProtectionHint::text_only(r"\&"))
///     }))
/// }));
/// let encoder = Encoder::new(chain);
/// assert_eq!(encoder.encode("you & me").unwrap(), r"you \& me");
/// ```
pub type DynRuleChain<'r> = RuleChain<Vec<Box<dyn Rule + Send + Sync + 'r>>>;

/// A chain of rules assembled at run time that stays on one thread: the chain
/// of a JavaScript binding, or one holding an [`Rc`](alloc::rc::Rc) or a
/// [`RefCell`](core::cell::RefCell).
///
/// The same as [`DynRuleChain`] without `Send + Sync`.
pub type LocalDynRuleChain<'r> = RuleChain<Vec<Box<dyn Rule + 'r>>>;

impl<'r> DynRuleChain<'r> {
    /// The chain with no rules in it, which never matches.
    pub fn empty() -> Self {
        RuleChain { rules: Vec::new() }
    }

    /// Adds `rule` at the end of the chain, boxing it.
    pub fn push<R: Rule + Send + Sync + 'r>(&mut self, rule: R) {
        self.rules.push(Box::new(rule));
    }

    /// The chain with `rule` added at its end, for building one in a single
    /// expression.
    #[must_use]
    pub fn with_rule<R: Rule + Send + Sync + 'r>(mut self, rule: R) -> Self {
        self.push(rule);
        self
    }
}

impl<'r> LocalDynRuleChain<'r> {
    /// The chain with no rules in it, which never matches.
    pub fn empty() -> Self {
        RuleChain { rules: Vec::new() }
    }

    /// Adds `rule` at the end of the chain, boxing it.
    pub fn push<R: Rule + 'r>(&mut self, rule: R) {
        self.rules.push(Box::new(rule));
    }

    /// The chain with `rule` added at its end, for building one in a single
    /// expression.
    #[must_use]
    pub fn with_rule<R: Rule + 'r>(mut self, rule: R) -> Self {
        self.push(rule);
        self
    }
}
