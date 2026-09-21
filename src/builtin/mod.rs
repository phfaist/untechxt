//! The builtin Unicode-to-LaTeX lookup tables.
//!
//! The function [`default_rules`](crate::default_rules) is the usual way to
//! encode with the builtin data. Use the tables of this module directly when
//! you need to look up single characters (see [`LookupTable`]), or when you
//! assemble a [`RuleChain`](crate::rule::RuleChain) that uses only a part of
//! the builtin data.
//!
//! The module contains three tables, which are compiled from the same list of
//! entries. All three tables have the type [`BuiltinTable`], and each of them
//! is a [`Rule`].
//!
//! - The table [`DEFAULT_TABLE`] contains every builtin entry.
//! - The table [`DEFAULT_TABLE_NON_ASCII`] contains the entries for non-ASCII
//!   characters only. Use this table when the input already contains LaTeX
//!   code, because the characters `\`, `{`, `%` and `&` of the input must then
//!   be kept unchanged. This table shares its compiled data with
//!   [`DEFAULT_TABLE`].
//! - The table [`DEFAULT_TABLE_ASCII_SPECIALS`] contains the entries for ASCII
//!   characters only. These are the few printable ASCII characters that have
//!   a special meaning for LaTeX, such as `%`, `&` and `\`. Use this table in
//!   a rule chain that handles all other characters with custom rules. This
//!   table is compiled separately, so that a program that uses no other
//!   builtin table does not link the non-ASCII entries.
//!
//! ```
//! use untechxt::builtin::DEFAULT_TABLE_NON_ASCII;
//! use untechxt::Encoder;
//!
//! // The input is LaTeX code already, and only `é` needs to be encoded.
//! let encoder = Encoder::new(&DEFAULT_TABLE_NON_ASCII);
//! assert_eq!(encoder.encode(r"\emph{Café} & more").unwrap(),
//!            r"\emph{Caf\'e} & more");
//! ```
//!
//! The entries themselves are listed in the [`default_table`] module, together
//! with their provenance and the license notices that apply to them. The
//! [`needs_profiles`] module lists what the entries need in the preamble of
//! the document.

pub mod default_table;
pub mod needs_profiles;

use core::fmt;

use crate::lookuptable::{apply_lookup, LookupTable, TableEntry};
use crate::preamble::Profile;
use crate::rule::{AsciiSet, Rule, RuleInput, RuleResult};
use crate::statictable::__build::Entries;
use crate::statictable::{compile_static_table, StaticTableTwoLevelBitmap};

use self::default_table::ENTRIES;
use self::needs_profiles::PROFILES;

/// The type of the builtin lookup tables. A `BuiltinTable` is a
/// [`LookupTable`] and a [`Rule`].
///
/// The type is opaque: how the table stores its data is an implementation
/// detail that may change. The values of this type are the statics
/// [`DEFAULT_TABLE`], [`DEFAULT_TABLE_NON_ASCII`] and
/// [`DEFAULT_TABLE_ASCII_SPECIALS`]. Because the three tables have the same
/// type, a program can choose one of them at run time:
///
/// ```
/// use untechxt::builtin::{
///     BuiltinTable, DEFAULT_TABLE, DEFAULT_TABLE_NON_ASCII,
/// };
/// use untechxt::Encoder;
///
/// let input_is_latex = true;
/// let table: &'static BuiltinTable =
///     if input_is_latex { &DEFAULT_TABLE_NON_ASCII } else { &DEFAULT_TABLE };
/// let encoder = Encoder::new(table);
/// assert_eq!(encoder.encode("Café & co").unwrap(), r"Caf\'e & co");
/// ```
pub struct BuiltinTable {
    /// The compiled entries. Which static layout contains them is not part of
    /// the API.
    layout: StaticTableTwoLevelBitmap,
    /// Whether the table ignores the ASCII entries of `layout`. This is how
    /// [`DEFAULT_TABLE_NON_ASCII`] shares the data of [`DEFAULT_TABLE`].
    skip_ascii: bool,
}

impl BuiltinTable {
    /// Returns the number of entries in the table.
    pub fn len(&self) -> usize {
        if self.skip_ascii {
            self.layout.len() - self.layout.ascii_keys().count()
        } else {
            self.layout.len()
        }
    }

    /// Returns whether the table contains no entry at all. No builtin table
    /// is empty; the method exists because [`len`](BuiltinTable::len) does.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the entries of the table, in ascending order
    /// of their character.
    ///
    /// ```
    /// use untechxt::builtin::DEFAULT_TABLE;
    ///
    /// let (first, entry) = DEFAULT_TABLE.iter().next().unwrap();
    /// assert_eq!(first, '"');
    /// assert_eq!(entry.encoded, "''");
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (char, TableEntry<'_>)> + '_ {
        let skip_ascii = self.skip_ascii;
        self.layout.iter().filter(move |(ch, _)| !(skip_ascii && ch.is_ascii()))
    }
}

impl LookupTable for BuiltinTable {
    fn lookup(&self, ch: char) -> Option<TableEntry<'_>> {
        if self.skip_ascii && ch.is_ascii() {
            return None;
        }
        self.layout.lookup(ch)
    }

    fn ascii_keys(&self) -> AsciiSet {
        if self.skip_ascii {
            AsciiSet::EMPTY
        } else {
            self.layout.ascii_keys()
        }
    }
}

impl Rule for BuiltinTable {
    fn apply<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a> {
        apply_lookup(self, input)
    }

    fn ascii_triggers(&self) -> AsciiSet {
        self.ascii_keys()
    }
}

impl fmt::Debug for BuiltinTable {
    /// Prints only the number of entries. There are too many entries to
    /// print, and how they are stored is not part of the API.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BuiltinTable").field("len", &self.len()).finish_non_exhaustive()
    }
}

/// The default builtin lookup table, with every builtin entry.
///
/// The function [`default_rules`](crate::default_rules) returns a rule that
/// uses this table, and it is the usual way to encode with the builtin data.
///
/// ```
/// use untechxt::builtin::DEFAULT_TABLE;
/// use untechxt::lookuptable::LookupTable;
///
/// assert_eq!(DEFAULT_TABLE.lookup('\u{2264}').unwrap().encoded, r"\leq");
/// assert_eq!(DEFAULT_TABLE.lookup('&').unwrap().encoded, r"\&");
/// ```
pub static DEFAULT_TABLE: BuiltinTable = BuiltinTable {
    layout: compile_static_table!(ENTRIES, &PROFILES, two_level_bitmap),
    skip_ascii: false,
};

/// The default builtin lookup table without its entries for ASCII characters.
///
/// Used as a [`Rule`], this table never matches an ASCII character. It
/// therefore keeps any LaTeX code that the input already contains unchanged,
/// which is why it suits an input that is already LaTeX. It replaces the
/// `non_ascii_only` flag of pylatexenc. The table shares the compiled data of
/// [`DEFAULT_TABLE`], so that a program that uses both tables contains one
/// copy of the data.
///
/// ```
/// use untechxt::builtin::DEFAULT_TABLE_NON_ASCII;
/// use untechxt::lookuptable::LookupTable;
///
/// let entry = DEFAULT_TABLE_NON_ASCII.lookup('\u{e9}').unwrap();
/// assert_eq!(entry.encoded, r"\'e");
/// assert_eq!(DEFAULT_TABLE_NON_ASCII.lookup('&'), None);
/// ```
pub static DEFAULT_TABLE_NON_ASCII: BuiltinTable =
    BuiltinTable { layout: DEFAULT_TABLE.layout, skip_ascii: true };

/// A small builtin lookup table for the few printable ASCII characters that
/// have a special meaning for LaTeX.
///
/// These are the characters that LaTeX either reserves for its own syntax,
/// such as `\`, `{`, `%` and `&`, or typesets differently from how they are
/// typed, such as `<` and `"`.
///
/// This table is compiled separately from [`DEFAULT_TABLE`], from the ASCII
/// entries at the start of the same list. A program that uses only this table
/// therefore links neither the non-ASCII entries nor the builtin profiles. A
/// program that links both tables contains a second copy of the ASCII
/// entries, which is small.
///
/// ```
/// use untechxt::builtin::{DEFAULT_TABLE, DEFAULT_TABLE_ASCII_SPECIALS};
/// use untechxt::lookuptable::LookupTable;
///
/// let entry = DEFAULT_TABLE_ASCII_SPECIALS.lookup('&').unwrap();
/// assert_eq!(entry.encoded, r"\&");
/// assert_eq!(Some(entry), DEFAULT_TABLE.lookup('&'));
/// assert_eq!(DEFAULT_TABLE_ASCII_SPECIALS.lookup('\u{e9}'), None);
/// ```
pub static DEFAULT_TABLE_ASCII_SPECIALS: BuiltinTable = BuiltinTable {
    layout: compile_static_table!(ascii_head(ENTRIES), &ASCII_PROFILES, two_level_bitmap),
    skip_ascii: false,
};

/// The profiles of [`DEFAULT_TABLE_ASCII_SPECIALS`]: index 0 alone, which
/// needs nothing. Every ASCII entry names that profile, so the small table has
/// no use for [`PROFILES`], and a reference to [`PROFILES`] would link all of
/// its snippets. An ASCII entry that came to name another profile would fail
/// the index check of [`compile_static_table!`], which is a compile error and
/// the point at which to pass `&PROFILES` to the small table instead.
static ASCII_PROFILES: [Profile; 1] = [Profile::from_static(&[])];

/// The entries of `entries` at ASCII characters. They are its head, because
/// the entries are sorted by character, which [`compile_static_table!`] checks
/// when it compiles [`DEFAULT_TABLE`].
const fn ascii_head(entries: Entries) -> Entries {
    let mut n = 0;
    while n < entries.len() && entries[n].0.is_ascii() {
        n += 1;
    }
    entries.split_at(n).0
}
