//! Lookup tables: one encoded value per character, and how such a table
//! becomes a [`Rule`].

use alloc::string::String;
use alloc::vec::Vec;

use crate::asciiset::AsciiSet;
use crate::profile::Profile;
use crate::replacement_protection::ReplacementProtectionHint;
use crate::rule::{Rule, RuleInput, RuleResult};

/// A table that answers, for one character, the LaTeX that prints it.
///
/// This is what the crate's own tables are, and what a user's table
/// implements to become a rule through [`TableRule`]. The crate's own tables
/// implement [`Rule`] directly, which is why there is no blanket
/// implementation here: one would clash with the `&R` and `Box<R>` forwarding
/// implementations of [`Rule`].
pub trait LookupTable: core::fmt::Debug {
    /// The table's entry for `ch`, or `None` when it has none.
    fn lookup(&self, ch: char) -> Option<TableEntry<'_>>;

    /// The ASCII characters the table has entries for. The encoder uses it as
    /// the table's [`Rule::ascii_triggers`], and so it must never leave out a
    /// character the table answers for.
    ///
    /// The default is [`AsciiSet::ALL`].
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

/// What a [`LookupTable`] holds for one character.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableEntry<'t> {
    /// The LaTeX that prints the character, with nothing around it.
    pub encoded: &'t str,
    /// What the protection strategy must know about that LaTeX.
    pub hint: ReplacementProtectionHint,
    /// What the LaTeX needs in the document's preamble, if anything.
    pub needs: Option<&'t Profile>,
}

/// The rule that looks every character up in the [`LookupTable`] it holds.
///
/// The crate's own tables are rules already; this is for a table of the
/// user's.
///
/// ```
/// use untechxt::{DynTable, Encoder, ReplacementProtectionHint, TableRule};
///
/// let mut table = DynTable::new();
/// table.insert('\u{2014}', r"\textemdash", ReplacementProtectionHint::text_only(r"\textemdash"));
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

/// A view of a table that answers for ASCII characters alone.
///
/// The builtin `ASCII_SPECIALS` table is this view of the full builtin table,
/// so that both share one copy of the data.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct OnlyAscii<T>(pub T);

impl<T: LookupTable> LookupTable for OnlyAscii<T> {
    fn lookup(&self, ch: char) -> Option<TableEntry<'_>> {
        ch.is_ascii().then(|| self.0.lookup(ch)).flatten()
    }

    fn ascii_keys(&self) -> AsciiSet {
        self.0.ascii_keys()
    }
}

impl<T: LookupTable> Rule for OnlyAscii<T> {
    fn apply<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a> {
        apply_lookup(self, input)
    }

    fn ascii_triggers(&self) -> AsciiSet {
        self.0.ascii_keys()
    }
}

/// A view of a table that answers for characters outside ASCII alone.
///
/// This is what replaces pylatexenc's `non_ascii_only` flag, which silently
/// disabled multi-character rules that start at an ASCII character: leave the
/// ASCII entries out of the chain instead of changing how the encoder scans.
/// It triggers on no ASCII character at all, and so the encoder copies ASCII
/// runs without consulting it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ExceptAscii<T>(pub T);

impl<T: LookupTable> LookupTable for ExceptAscii<T> {
    fn lookup(&self, ch: char) -> Option<TableEntry<'_>> {
        (!ch.is_ascii()).then(|| self.0.lookup(ch)).flatten()
    }

    fn ascii_keys(&self) -> AsciiSet {
        AsciiSet::EMPTY
    }
}

impl<T: LookupTable> Rule for ExceptAscii<T> {
    fn apply<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a> {
        apply_lookup(self, input)
    }

    fn ascii_triggers(&self) -> AsciiSet {
        AsciiSet::EMPTY
    }
}

/// A lookup table built at run time: a sorted list of entries, searched by
/// bisection.
///
/// This is the table of an encoder configured from a file, from a language
/// binding, or from a caller who overrides a few characters of a static
/// table. It owns its LaTeX and its profiles, and it is a [`Rule`] itself.
///
/// ```
/// use untechxt::{DynTable, Encoder, ReplacementProtectionHint};
///
/// let mut table = DynTable::new();
/// table.insert('\u{3b1}', r"\alpha", ReplacementProtectionHint::math_only(r"\alpha"));
/// let encoder = Encoder::new(table);
/// assert_eq!(encoder.encode("\u{3b1}").unwrap(), r"\ensuremath{\alpha}");
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DynTable {
    /// The entries, in ascending order of their character.
    entries: Vec<DynEntry>,
    /// The ASCII characters the entries cover.
    ascii_keys: AsciiSet,
}

/// One entry of a [`DynTable`], which owns everything it holds.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DynEntry {
    /// The character the entry is for.
    ch: char,
    /// The LaTeX that prints it.
    encoded: String,
    /// What the protection strategy must know about that LaTeX.
    hint: ReplacementProtectionHint,
    /// What the LaTeX needs in the preamble, if anything.
    needs: Option<Profile>,
}

impl DynTable {
    /// The empty table, which answers for no character at all.
    pub fn new() -> Self {
        DynTable::default()
    }

    /// Adds the entry for `ch`, replacing the table's entry for it if it had
    /// one. The value needs nothing in the preamble.
    pub fn insert(
        &mut self,
        ch: char,
        encoded: impl Into<String>,
        hint: ReplacementProtectionHint,
    ) {
        self.put(ch, encoded.into(), hint, None);
    }

    /// Adds the entry for `ch`, whose value needs `needs` in the document's
    /// preamble, replacing the table's entry for it if it had one.
    pub fn insert_with_needs(
        &mut self,
        ch: char,
        encoded: impl Into<String>,
        hint: ReplacementProtectionHint,
        needs: Profile,
    ) {
        self.put(ch, encoded.into(), hint, Some(needs));
    }

    /// The table with the entry for `ch` added, for building one in a single
    /// expression.
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

    /// The number of entries in the table.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the table has no entry at all.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The entries of the table, in ascending order of their character.
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
    /// The entry as a [`LookupTable`] answers it.
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

/// The replacement a table lookup at `input` gives: the entry's LaTeX for the
/// character there, with the entry's hint and needs.
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
