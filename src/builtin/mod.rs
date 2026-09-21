//! The builtin Unicode-to-LaTeX tables: 1549 characters with the LaTeX that
//! prints each one, and what that LaTeX needs in the document's preamble.
//!
//! Three tables share one compiled copy of the data:
//!
//! - [`DEFAULTS`], every entry, which is what an encoder built with no
//!   arguments uses;
//! - [`NON_ASCII`], the entries outside ASCII alone, for output that already
//!   is LaTeX and must keep its own `\`, `{`, `%` and `&`;
//! - [`ASCII_SPECIALS`], the 13 ASCII entries alone — `"`, `#`, `$`, `%`,
//!   `&`, `<`, `>`, `\`, `^`, `_`, `{`, `}` and `~` — for a chain that
//!   handles the rest of the characters itself.
//!
//! ```
//! use untechxt::{Encoder, DEFAULTS};
//!
//! let encoder = Encoder::new(&DEFAULTS);
//! assert_eq!(encoder.encode("Caf\u{e9}").unwrap(), r"Caf\'e");
//! ```
//!
//! The data itself is [`default_table::ENTRIES`], with its provenance and the
//! license notices that travel with it; what the entries need in the preamble
//! is [`needs_profiles`].

pub mod default_table;
pub mod needs_profiles;

use core::fmt;

use crate::asciiset::AsciiSet;
use crate::compile_static_table;
use crate::lookuptable::{apply_lookup, ExceptAscii, LookupTable, OnlyAscii, TableEntry};
use crate::rule::{Rule, RuleInput, RuleResult};
use crate::statictable::StaticTableTwoLevelDirect;

use self::default_table::ENTRIES;
use self::needs_profiles::PROFILES;

/// The type of the builtin tables: a [`LookupTable`] and a [`Rule`] over the
/// compiled builtin data.
///
/// Which static layout holds the data is an implementation detail and may
/// change, which is why this is an opaque type of its own rather than one of
/// the [`statictable`](crate::statictable) layouts. There is one value of it,
/// [`DEFAULTS`]; [`NON_ASCII`] and [`ASCII_SPECIALS`] are views of that one.
pub struct BuiltinTable(StaticTableTwoLevelDirect);

impl BuiltinTable {
    /// The number of entries in the table.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the table has no entry at all. It never has none, but the
    /// method is here because [`len`](BuiltinTable::len) is.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The entries of the table, in ascending order of their character.
    ///
    /// ```
    /// use untechxt::DEFAULTS;
    ///
    /// let (first, entry) = DEFAULTS.iter().next().unwrap();
    /// assert_eq!(first, '"');
    /// assert_eq!(entry.encoded, "''");
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (char, TableEntry<'_>)> + '_ {
        self.0.iter()
    }
}

impl LookupTable for BuiltinTable {
    fn lookup(&self, ch: char) -> Option<TableEntry<'_>> {
        self.0.lookup(ch)
    }

    fn ascii_keys(&self) -> AsciiSet {
        self.0.ascii_keys()
    }
}

impl Rule for BuiltinTable {
    fn apply<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a> {
        apply_lookup(self, input)
    }

    fn ascii_triggers(&self) -> AsciiSet {
        self.0.ascii_keys()
    }
}

impl fmt::Debug for BuiltinTable {
    /// The number of entries alone: the data is far too long to print, and
    /// the layout that holds it is not part of the API.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BuiltinTable").field("len", &self.len()).finish_non_exhaustive()
    }
}

/// The builtin table: every one of the 1549 entries.
///
/// ```
/// use untechxt::{DEFAULTS, LookupTable};
///
/// assert_eq!(DEFAULTS.len(), 1549);
/// assert_eq!(DEFAULTS.lookup('\u{2264}').unwrap().encoded, r"\leq");
/// ```
pub static DEFAULTS: BuiltinTable =
    BuiltinTable(compile_static_table!(ENTRIES, &PROFILES, two_level_direct_index));

/// The builtin table without its ASCII entries: a rule that never matches at
/// an ASCII character, and so never touches LaTeX that is already in the
/// input.
///
/// This is what replaces pylatexenc's `non_ascii_only` flag. It is a view of
/// [`DEFAULTS`], so it costs no second copy of the data.
///
/// ```
/// use untechxt::{LookupTable, NON_ASCII};
///
/// assert_eq!(NON_ASCII.lookup('\u{e9}').unwrap().encoded, r"\'e");
/// assert_eq!(NON_ASCII.lookup('&'), None);
/// ```
pub static NON_ASCII: ExceptAscii<&'static BuiltinTable> = ExceptAscii(&DEFAULTS);

/// The ASCII entries of the builtin table alone: the 13 characters `"`, `#`,
/// `$`, `%`, `&`, `<`, `>`, `\`, `^`, `_`, `{`, `}` and `~`, which LaTeX
/// either reserves for itself or reads differently from the way they are
/// typed.
///
/// It is a view of [`DEFAULTS`], so it costs no second copy of the data.
///
/// ```
/// use untechxt::{ASCII_SPECIALS, LookupTable};
///
/// assert_eq!(ASCII_SPECIALS.lookup('&').unwrap().encoded, r"\&");
/// assert_eq!(ASCII_SPECIALS.lookup('\u{e9}'), None);
/// ```
pub static ASCII_SPECIALS: OnlyAscii<&'static BuiltinTable> = OnlyAscii(&DEFAULTS);
