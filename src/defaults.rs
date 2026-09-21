//! The default rules: the function [`default_rules`] and its return type
//! [`DefaultRules`].

use core::fmt;

use crate::builtin::{BuiltinTable, DEFAULT_TABLE};
use crate::rule::{AsciiSet, Rule, RuleInput, RuleResult};

/// The rule that a [`DefaultRules`] applies. To add a rule to the default
/// rules, change this type to a `RuleChain` of the member rules, update
/// [`default_rules`] accordingly, and export the new member rule from the
/// `builtin` module.
type Inner = &'static BuiltinTable;

/// Returns the default rules of the crate. The default rules are a [`Rule`]
/// that encodes every character that the builtin data of the crate contains.
///
/// Pass the default rules to [`Encoder::new`](crate::Encoder::new) to create
/// an encoder. The [`encode`](crate::encode) function uses the same rules.
///
/// ```
/// use untechxt::{default_rules, Encoder};
///
/// let encoder = Encoder::new(default_rules());
/// assert_eq!(encoder.encode("Café — 100%").unwrap(),
///            r"Caf\'e {\textemdash} 100\%");
/// ```
///
/// The default rules are a rule like any other, so they can be a member of a
/// [`RuleChain`](crate::rule::RuleChain). A custom rule that is placed before
/// the default rules overrides them, and a custom rule that is placed after
/// the default rules handles input that the default rules do not match.
///
/// ```
/// use untechxt::lookuptable::DynTable;
/// use untechxt::protection::ReplacementProtectionHint as Hint;
/// use untechxt::rule::RuleChain;
/// use untechxt::{default_rules, Encoder};
///
/// let overrides = DynTable::new().with_entry(
///     '%', r"\textpercent", Hint::text_only(r"\textpercent")
/// );
/// let encoder = Encoder::new(RuleChain::new((overrides, default_rules())));
/// assert_eq!(encoder.encode("100% é").unwrap(), r"100{\textpercent} \'e");
/// ```
///
/// # What the default rules contain
///
/// The default rules currently consist of the builtin lookup table
/// [`DEFAULT_TABLE`]. Future versions of the crate may add further rules to
/// the default rules, which is why this function returns the opaque type
/// [`DefaultRules`].
///
/// Every rule that is part of the default rules is also available on its own
/// in the [`builtin`](crate::builtin) module. Use the items of that module to
/// assemble a variant of the default rules. For instance, encode with the
/// table
/// [`DEFAULT_TABLE_NON_ASCII`](crate::builtin::DEFAULT_TABLE_NON_ASCII) when
/// the input already contains LaTeX code that must be kept unchanged.
pub const fn default_rules() -> DefaultRules {
    DefaultRules { inner: &DEFAULT_TABLE }
}

/// The type of the default rules, which the [`default_rules`] function
/// returns.
///
/// The type is opaque: which rules it consists of may change in future
/// versions of the crate (see [`default_rules`]). It implements [`Rule`],
/// [`Debug`](core::fmt::Debug), [`Clone`] and [`Copy`], and it is `Send` and
/// `Sync`. It is small, and it applies its rules without dynamic dispatch.
///
/// `DefaultRules` is the default value of the type parameter `R` of
/// [`Encoder`](crate::Encoder). The type `Encoder`, written with no type
/// arguments, is therefore the type of `Encoder::new(default_rules())`:
///
/// ```
/// use untechxt::{default_rules, Encoder};
///
/// struct Exporter {
///     encoder: Encoder,
/// }
///
/// let exporter = Exporter { encoder: Encoder::new(default_rules()) };
/// assert_eq!(exporter.encoder.encode("α").unwrap(), r"\ensuremath{\alpha}");
/// ```
#[derive(Clone, Copy)]
pub struct DefaultRules {
    /// The rules themselves.
    inner: Inner,
}

impl Rule for DefaultRules {
    fn apply<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a> {
        self.inner.apply(input)
    }

    fn ascii_triggers(&self) -> AsciiSet {
        self.inner.ascii_triggers()
    }
}

impl fmt::Debug for DefaultRules {
    /// Prints the name of the type only, because what the default rules
    /// consist of is not part of the API.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DefaultRules").finish_non_exhaustive()
    }
}
