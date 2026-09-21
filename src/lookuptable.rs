//! Lookup tables, each holding one encoded value per character.
//!
//! A lookup table maps a single character to the LaTeX that prints it. This
//! module contains the [`LookupTable`] trait that every such table
//! implements, the [`TableEntry`] that a lookup returns, and two ways to
//! build a table:
//!
//! - [`DynTable`] is a table built at run time, one entry at a time. Use it
//!   for an encoder configured from a file, from a language binding, or from a
//!   caller who overrides a few entries of another table.
//! - The [`statictable`](crate::statictable) module compiles a table of your
//!   own at compile time, with nothing to allocate at run time, and the
//!   [`builtin`](crate::builtin) module holds the crate's own tables, which
//!   are compiled that way.
//!
//! A lookup table is not a [`Rule`] on its own. The crate's own tables and
//! [`DynTable`] implement [`Rule`] directly. Create a rule from a custom
//! [`LookupTable`] with the [`TableRule`] wrapper.

use alloc::string::String;
use alloc::vec::Vec;

use crate::preamble::Profile;
use crate::protection::ReplacementProtectionHint;
use crate::rule::{AsciiSet, Rule, RuleInput, RuleResult};

/// A lookup table: one encoded value per character.
///
/// A lookup table returns, for a single character, the LaTeX that prints that
/// character, together with the protection hint and the preamble needs that
/// go with it (see [`TableEntry`]). The crate's own tables implement this
/// trait, and so does [`DynTable`]. Create a rule from a custom
/// [`LookupTable`] with the [`TableRule`] wrapper.
///
/// This trait does not provide a blanket implementation of [`Rule`], because
/// one would clash with the `&R` and `Box<R>` forwarding implementations of
/// [`Rule`]. The crate's own tables therefore implement [`Rule`] directly.
pub trait LookupTable: core::fmt::Debug {
    /// Returns the table's entry for `ch`, or `None` if the table has no entry
    /// for it.
    fn lookup(&self, ch: char) -> Option<TableEntry<'_>>;

    /// Returns the ASCII characters the table has entries for. The encoder
    /// uses this set as the table's [`Rule::ascii_triggers`], so it must
    /// include every ASCII character the table has an entry for. If it left
    /// one out, the encoder's ASCII fast path would copy that character
    /// unchanged instead of encoding it.
    ///
    /// The default is [`AsciiSet::ALL`], which is always correct and never
    /// skips a character, at the cost of consulting the table at every ASCII
    /// character.
    fn ascii_keys(&self) -> AsciiSet {
        AsciiSet::ALL
    }
}

impl<T: LookupTable + ?Sized> LookupTable for &T {
    fn lookup(&self, ch: char) -> Option<TableEntry<'_>> {
        (**self).lookup(ch)
    }

    fn ascii_keys(&self) -> AsciiSet {
        (**self).ascii_keys()
    }
}

/// The entry a [`LookupTable`] returns for one character: the LaTeX, the
/// protection hint that goes with it, and its preamble needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableEntry<'t> {
    /// The LaTeX that prints the character, with nothing around it.
    pub encoded: &'t str,
    /// The [`ReplacementProtectionHint`] the protection strategy uses to wrap
    /// the value: the LaTeX mode the value is valid in and whether it ends
    /// with a named macro.
    pub hint: ReplacementProtectionHint,
    /// The [`Profile`] of what the value needs in the document's preamble, or
    /// `None` when it needs nothing beyond the LaTeX kernel.
    pub needs: Option<&'t Profile>,
}

/// A rule that looks each character up in the [`LookupTable`] it holds.
///
/// The crate's own tables are rules already, and [`DynTable`] is a rule too.
/// Use [`TableRule`] to make a rule from a custom [`LookupTable`].
///
/// ```
/// use untechxt::lookuptable::{DynTable, TableRule};
/// use untechxt::protection::ReplacementProtectionHint;
/// use untechxt::Encoder;
///
/// let mut table = DynTable::new();
/// table.insert(
///     '\u{2014}',
///     r"\textemdash",
///     ReplacementProtectionHint::text_only(r"\textemdash"),
/// );
/// let encoder = Encoder::new(TableRule(&table));
/// assert_eq!(encoder.encode("a \u{2014} b").unwrap(), r"a {\textemdash} b");
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct TableRule<T>(pub T);

impl<T: LookupTable> Rule for TableRule<T> {
    fn apply<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a> {
        apply_lookup(&self.0, input)
    }

    fn ascii_triggers(&self) -> AsciiSet {
        self.0.ascii_keys()
    }
}

/// A lookup table built at run time: a sorted list of entries, searched by
/// bisection.
///
/// A [`DynTable`] owns the LaTeX and the profiles of its entries, and it is a
/// [`Rule`] itself. Use it for an encoder configured from a file, from a
/// language binding, or from a caller who overrides a few entries of a static
/// table. Add entries with [`insert`](DynTable::insert), or with
/// [`with_entry`](DynTable::with_entry) when building a table in a single
/// expression.
///
/// ```
/// use untechxt::lookuptable::DynTable;
/// use untechxt::protection::ReplacementProtectionHint;
/// use untechxt::Encoder;
///
/// let mut table = DynTable::new();
/// table.insert(
///     '\u{3b1}',
///     r"\alpha",
///     ReplacementProtectionHint::math_only(r"\alpha"),
/// );
/// let encoder = Encoder::new(table);
/// assert_eq!(encoder.encode("\u{3b1}").unwrap(), r"\ensuremath{\alpha}");
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DynTable {
    /// The entries, in ascending order of their character.
    entries: Vec<DynEntry>,
    /// The ASCII characters the table has entries for.
    ascii_keys: AsciiSet,
}

/// One entry of a [`DynTable`], which owns everything it holds.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DynEntry {
    /// The character the entry is for.
    ch: char,
    /// The LaTeX that prints the character, with nothing around it.
    encoded: String,
    /// The hint the protection strategy uses to wrap the value.
    hint: ReplacementProtectionHint,
    /// The profile of what the value needs in the preamble, or `None` when it
    /// needs nothing.
    needs: Option<Profile>,
}

impl DynTable {
    /// Creates an empty table, which has no entry for any character.
    pub fn new() -> Self {
        DynTable::default()
    }

    /// Adds an entry for `ch`. If the table already has an entry for `ch`, the
    /// new entry replaces it. The value needs nothing in the document's
    /// preamble; use [`insert_with_needs`](DynTable::insert_with_needs) for a
    /// value that does.
    pub fn insert(
        &mut self,
        ch: char,
        encoded: impl Into<String>,
        hint: ReplacementProtectionHint,
    ) {
        self.put(ch, encoded.into(), hint, None);
    }

    /// Adds an entry for `ch` whose value needs `needs` in the document's
    /// preamble. If the table already has an entry for `ch`, the new entry
    /// replaces it.
    pub fn insert_with_needs(
        &mut self,
        ch: char,
        encoded: impl Into<String>,
        hint: ReplacementProtectionHint,
        needs: Profile,
    ) {
        self.put(ch, encoded.into(), hint, Some(needs));
    }

    /// Returns the table with an entry for `ch` added, for building a table in
    /// a single expression.
    #[must_use]
    pub fn with_entry(
        mut self,
        ch: char,
        encoded: impl Into<String>,
        hint: ReplacementProtectionHint,
    ) -> Self {
        self.insert(ch, encoded, hint);
        self
    }

    /// Returns the number of entries in the table.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the table has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns an iterator over the table's entries, in ascending order of
    /// their character.
    pub fn iter(&self) -> impl Iterator<Item = (char, TableEntry<'_>)> + '_ {
        self.entries.iter().map(|entry| (entry.ch, entry.view()))
    }

    /// Puts one entry in place, keeping the list sorted and the ASCII key set
    /// up to date.
    fn put(
        &mut self,
        ch: char,
        encoded: String,
        hint: ReplacementProtectionHint,
        needs: Option<Profile>,
    ) {
        let entry = DynEntry { ch, encoded, hint, needs };
        match self.entries.binary_search_by_key(&ch, |held| held.ch) {
            Ok(index) => self.entries[index] = entry,
            Err(index) => self.entries.insert(index, entry),
        }
        if ch.is_ascii() {
            self.ascii_keys = self.ascii_keys.with(ch as u8);
        }
    }
}

impl DynEntry {
    /// Returns the entry as a borrowed [`TableEntry`].
    fn view(&self) -> TableEntry<'_> {
        TableEntry { encoded: &self.encoded, hint: self.hint, needs: self.needs.as_ref() }
    }
}

impl LookupTable for DynTable {
    fn lookup(&self, ch: char) -> Option<TableEntry<'_>> {
        let index = self.entries.binary_search_by_key(&ch, |entry| entry.ch).ok()?;
        Some(self.entries[index].view())
    }

    fn ascii_keys(&self) -> AsciiSet {
        self.ascii_keys
    }
}

impl Rule for DynTable {
    fn apply<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a> {
        apply_lookup(self, input)
    }

    fn ascii_triggers(&self) -> AsciiSet {
        self.ascii_keys
    }
}

/// Looks the character at `input` up in `table` and returns the matching
/// replacement, or `Ok(None)` when the table has no entry for that character.
///
/// This is the shared body of the [`Rule::apply`] implementation of every
/// table type in the crate.
pub(crate) fn apply_lookup<'a, T: LookupTable + ?Sized>(
    table: &'a T,
    input: RuleInput<'a>,
) -> RuleResult<'a> {
    Ok(table.lookup(input.ch()).map(|entry| {
        let replacement = input.replace_char(entry.encoded, entry.hint);
        match entry.needs {
            Some(profile) => replacement.with_needs(profile),
            None => replacement,
        }
    }))
}
