//! Static lookup tables: a table of entries known at compile time, compiled
//! by [`compile_static_table!`] into one of three layouts, with nothing left
//! to do at run time and nothing to allocate.
//!
//! A table is written as a plain `const` slice of tuples — the character, its
//! LaTeX, the [`ValueMode`] that LaTeX is valid in, and the
//! [`ProfileIndex`] of what it needs in the preamble — beside an array of the
//! [`Profile`]s those indices name. The macro turns the two into one of
//! [`StaticTableBinarySearch`], [`StaticTableTwoLevelLinear`] or
//! [`StaticTableTwoLevelDirect`], each of which is a [`LookupTable`] and a
//! [`Rule`] in its own right:
//!
//! ```
//! use untechxt::lookuptable::LookupTable;
//! use untechxt::preamble::{Chunk, Profile};
//! use untechxt::protection::ValueMode;
//! use untechxt::statictable::{
//!     compile_static_table, ProfileIndex, StaticTableTwoLevelDirect,
//! };
//! use untechxt::Encoder;
//!
//! static AMSSYMB_CHUNKS: [Chunk; 1] = [Chunk::package("amssymb")];
//! static PROFILES: [Profile; 2] = [
//!     Profile::from_static(&[]),                  // index 0: needs nothing
//!     Profile::from_static(&AMSSYMB_CHUNKS),      // index 1
//! ];
//! const NONE: ProfileIndex = ProfileIndex::NONE;
//! const AMSSYMB: ProfileIndex = ProfileIndex(1);
//!
//! const ENTRIES: &[(char, &str, ValueMode, ProfileIndex)] = &[
//!     ('\u{e9}', r"\'e", ValueMode::TextOnly, NONE),
//!     ('\u{2102}', r"\mathbb{C}", ValueMode::MathOnly, AMSSYMB),
//! ];
//!
//! static TABLE: StaticTableTwoLevelDirect =
//!     compile_static_table!(ENTRIES, &PROFILES, two_level_direct_index);
//!
//! assert_eq!(TABLE.lookup('\u{e9}').unwrap().encoded, r"\'e");
//! assert_eq!(TABLE.lookup('x'), None);
//! assert_eq!(Encoder::new(&TABLE).encode("Caf\u{e9}").unwrap(), r"Caf\'e");
//! ```
//!
//! # The three layouts
//!
//! All three answer the same lookups; they differ in how they find the entry
//! and in how much data they carry. Pick one by measuring, and keep the
//! choice private to your crate — it is an implementation detail, as
//! `BuiltinTable` keeps it for the builtin data.
//!
//! - [`StaticTableBinarySearch`]: a sorted array of the characters, searched
//!   by bisection. No index of its own: four bytes per entry beyond the
//!   payload, and nothing per block.
//! - [`StaticTableTwoLevelLinear`]: one block per distinct `code point >> 8`,
//!   found by a linear scan over the blocks, then a linear scan over the low
//!   bytes inside the block. One byte per entry beyond the payload, and six
//!   per block — the most compact of the three unless the entries are spread
//!   thinly over very many blocks.
//! - [`StaticTableTwoLevelDirect`]: the same blocks, each with a 256-slot
//!   index from the low byte straight to the entry. 512 bytes per block, and
//!   a lookup inside a block is one load.
//!
//! # What the compile-time checks reject
//!
//! The macro runs [`__build::check`] as a `const` item, so a table that
//! breaks any of these rules is a compile error rather than a run-time
//! surprise. Constant evaluation cannot format a message, so the error names
//! the rule that was broken but not the entry that broke it.
//!
//! - The characters must ascend strictly: sorted, with no duplicate. (Both
//!   two-level layouts and the bisection depend on it.)
//! - Every [`ProfileIndex`] must be a position of the profile array.
//! - Every LaTeX value must be ASCII.
//! - Its braces must balance, not counting the escaped `\{` and `\}`.
//! - It must not end with a lone backslash, which would take the first
//!   character of whatever follows into a command name.
//! - The table must hold fewer than `u16::MAX` entries, since an entry is
//!   named by a `u16` inside the compiled layouts.

use core::fmt;

use crate::lookuptable::{apply_lookup, LookupTable, TableEntry};
use crate::preamble::Profile;
use crate::protection::{ReplacementProtectionHint, ValueMode, ValueTermination};
use crate::rule::{AsciiSet, Rule, RuleInput, RuleResult};

use self::__build::StaticEntry;

#[doc(inline)]
pub use crate::__compile_static_table as compile_static_table;

/// The number of a profile in a table's own profile array.
///
/// A static table stores one byte per entry rather than a profile, and turns
/// it into a `&'static Profile` on lookup. The index is meaningful only
/// together with the array it indexes; index 0 is reserved for "needs
/// nothing", so that a lookup answers `None` for it without consulting the
/// array.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct ProfileIndex(pub u8);

impl ProfileIndex {
    /// The reserved index of the profile that needs nothing: 0.
    pub const NONE: ProfileIndex = ProfileIndex(0);
}

/// The slot value of the direct index that means "no entry here".
const NO_ENTRY: u16 = u16::MAX;

/// A static table that finds its entry by bisecting a sorted array of the
/// characters.
///
/// Build one with [`compile_static_table!`] and the layout name
/// `binary_search`. It is the layout with no index of its own: the
/// characters, the payloads, and nothing else.
#[derive(Clone, Copy)]
pub struct StaticTableBinarySearch {
    /// The characters of the entries, strictly ascending.
    keys: &'static [char],
    /// The payloads, in the order of `keys`.
    values: &'static [StaticEntry],
    /// The profiles an entry's index names.
    profiles: &'static [Profile],
    /// The ASCII characters the table has entries for.
    ascii_keys: AsciiSet,
}

/// A static table of one block per distinct `code point >> 8`, each scanned
/// linearly.
///
/// Build one with [`compile_static_table!`] and the layout name
/// `two_level_linear`. A lookup scans the blocks for the character's high
/// bits, then the block's low bytes for the rest; it costs one byte per entry
/// and a few per block beyond the payloads.
#[derive(Clone, Copy)]
pub struct StaticTableTwoLevelLinear {
    /// The distinct `code point >> 8` of the entries, strictly ascending.
    blocks: &'static [u32],
    /// Where each block starts in `lows` and `values`, with the number of
    /// entries at the end: block `b` is `starts[b]..starts[b + 1]`.
    starts: &'static [u16],
    /// The low byte of each entry's character, in the order of `values`.
    lows: &'static [u8],
    /// The payloads, in ascending order of their character.
    values: &'static [StaticEntry],
    /// The profiles an entry's index names.
    profiles: &'static [Profile],
    /// The ASCII characters the table has entries for.
    ascii_keys: AsciiSet,
}

/// A static table of one block per distinct `code point >> 8`, each with a
/// direct 256-slot index.
///
/// Build one with [`compile_static_table!`] and the layout name
/// `two_level_direct_index`. A lookup scans the blocks for the character's
/// high bits and then reads the entry's position straight out of the block's
/// index. It costs 512 bytes per block, whatever the block holds.
#[derive(Clone, Copy)]
pub struct StaticTableTwoLevelDirect {
    /// The distinct `code point >> 8` of the entries, strictly ascending.
    blocks: &'static [u32],
    /// One 256-slot index per block: the low byte of a character to the
    /// position of its entry in `values`, or [`NO_ENTRY`].
    index: &'static [[u16; 256]],
    /// The payloads, in ascending order of their character.
    values: &'static [StaticEntry],
    /// The profiles an entry's index names.
    profiles: &'static [Profile],
    /// The ASCII characters the table has entries for.
    ascii_keys: AsciiSet,
}

impl StaticTableBinarySearch {
    /// The number of entries in the table.
    pub const fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether the table has no entry at all.
    pub const fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// The entries of the table, in ascending order of their character.
    pub fn iter(&self) -> impl Iterator<Item = (char, TableEntry<'_>)> + '_ {
        let profiles = self.profiles;
        self.keys
            .iter()
            .zip(self.values.iter())
            .map(move |(&ch, value)| (ch, value.view(profiles)))
    }
}

impl StaticTableTwoLevelLinear {
    /// The number of entries in the table.
    pub const fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether the table has no entry at all.
    pub const fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// The entries of the table, in ascending order of their character.
    pub fn iter(&self) -> impl Iterator<Item = (char, TableEntry<'_>)> + '_ {
        let (starts, lows, values, profiles) =
            (self.starts, self.lows, self.values, self.profiles);
        self.blocks.iter().enumerate().flat_map(move |(block, &high)| {
            let start = block_bound(starts, block);
            let end = block_bound(starts, block + 1);
            (start..end).filter_map(move |i| {
                let low = *lows.get(i)? as u32;
                let ch = char::from_u32((high << 8) | low)?;
                Some((ch, values.get(i)?.view(profiles)))
            })
        })
    }
}

impl StaticTableTwoLevelDirect {
    /// The number of entries in the table.
    pub const fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether the table has no entry at all.
    pub const fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// The entries of the table, in ascending order of their character.
    pub fn iter(&self) -> impl Iterator<Item = (char, TableEntry<'_>)> + '_ {
        let (values, profiles) = (self.values, self.profiles);
        self.blocks.iter().zip(self.index.iter()).flat_map(move |(&high, slots)| {
            slots.iter().enumerate().filter_map(move |(low, &slot)| {
                if slot == NO_ENTRY {
                    return None;
                }
                let ch = char::from_u32((high << 8) | low as u32)?;
                Some((ch, values.get(slot as usize)?.view(profiles)))
            })
        })
    }
}

/// Where block `block` begins in the entry arrays, panic-free.
fn block_bound(starts: &[u16], block: usize) -> usize {
    match starts.get(block) {
        Some(&start) => start as usize,
        None => 0,
    }
}

impl LookupTable for StaticTableBinarySearch {
    fn lookup(&self, ch: char) -> Option<TableEntry<'_>> {
        let index = self.keys.binary_search(&ch).ok()?;
        Some(self.values.get(index)?.view(self.profiles))
    }

    fn ascii_keys(&self) -> AsciiSet {
        self.ascii_keys
    }
}

impl LookupTable for StaticTableTwoLevelLinear {
    fn lookup(&self, ch: char) -> Option<TableEntry<'_>> {
        let code_point = ch as u32;
        let block = self.blocks.iter().position(|&high| high == code_point >> 8)?;
        let start = block_bound(self.starts, block);
        let end = block_bound(self.starts, block + 1);
        let low = code_point as u8;
        let offset = self.lows.get(start..end)?.iter().position(|&held| held == low)?;
        Some(self.values.get(start + offset)?.view(self.profiles))
    }

    fn ascii_keys(&self) -> AsciiSet {
        self.ascii_keys
    }
}

impl LookupTable for StaticTableTwoLevelDirect {
    fn lookup(&self, ch: char) -> Option<TableEntry<'_>> {
        let code_point = ch as u32;
        let block = self.blocks.iter().position(|&high| high == code_point >> 8)?;
        let slot = self.index.get(block)?[(code_point & 0xFF) as usize];
        if slot == NO_ENTRY {
            return None;
        }
        Some(self.values.get(slot as usize)?.view(self.profiles))
    }

    fn ascii_keys(&self) -> AsciiSet {
        self.ascii_keys
    }
}

impl Rule for StaticTableBinarySearch {
    fn apply<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a> {
        apply_lookup(self, input)
    }

    fn ascii_triggers(&self) -> AsciiSet {
        self.ascii_keys
    }
}

impl Rule for StaticTableTwoLevelLinear {
    fn apply<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a> {
        apply_lookup(self, input)
    }

    fn ascii_triggers(&self) -> AsciiSet {
        self.ascii_keys
    }
}

impl Rule for StaticTableTwoLevelDirect {
    fn apply<'a>(&'a self, input: RuleInput<'a>) -> RuleResult<'a> {
        apply_lookup(self, input)
    }

    fn ascii_triggers(&self) -> AsciiSet {
        self.ascii_keys
    }
}

impl fmt::Debug for StaticTableBinarySearch {
    /// The layout and the number of entries: the data itself is far too long
    /// to print.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StaticTableBinarySearch").field("len", &self.len()).finish_non_exhaustive()
    }
}

impl fmt::Debug for StaticTableTwoLevelLinear {
    /// The layout, the number of entries and the number of blocks.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StaticTableTwoLevelLinear")
            .field("len", &self.len())
            .field("blocks", &self.blocks.len())
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for StaticTableTwoLevelDirect {
    /// The layout, the number of entries and the number of blocks.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StaticTableTwoLevelDirect")
            .field("len", &self.len())
            .field("blocks", &self.blocks.len())
            .finish_non_exhaustive()
    }
}

/// The constant evaluation behind [`compile_static_table!`]: the checks, the
/// arrays each layout is made of, and the constructors of the three layout
/// structs.
///
/// Every function here runs at compile time, where a failed `assert!` is a
/// compile error. It is public because the macro expands to calls of it
/// outside this crate; nothing here is part of the stable API, and it is of
/// no use to call any of it by hand.
#[doc(hidden)]
pub mod __build {
    use super::{
        AsciiSet, Profile, ProfileIndex, ReplacementProtectionHint, StaticTableBinarySearch,
        StaticTableTwoLevelDirect, StaticTableTwoLevelLinear, TableEntry, ValueMode,
        ValueTermination, NO_ENTRY,
    };

    /// The flag bit of [`StaticEntry::flags`] that marks a value ending with a
    /// named macro.
    const ENDS_WITH_NAMED_MACRO: u8 = 0b1000_0000;

    /// The bits of [`StaticEntry::flags`] that hold the [`ValueMode`].
    const MODE_MASK: u8 = 0b0000_0011;

    /// [`ValueMode::TextOnly`], as [`StaticEntry::flags`] holds it.
    const MODE_TEXT: u8 = 0;
    /// [`ValueMode::MathOnly`], as [`StaticEntry::flags`] holds it.
    const MODE_MATH: u8 = 1;
    /// [`ValueMode::AnyMode`], as [`StaticEntry::flags`] holds it.
    const MODE_ANY: u8 = 2;

    /// One entry of a static table, as `compile_static_table!` compiles it: the
    /// LaTeX, one byte of flags (the mode and the termination) and one byte of
    /// profile index.
    ///
    /// The macro names this type in the `static` items it declares, which is
    /// why it is public; the packing itself is an implementation detail and is
    /// not part of the stable API.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct StaticEntry {
        /// The LaTeX that prints the character, with nothing around it.
        encoded: &'static str,
        /// The [`ValueMode`] in [`MODE_MASK`], plus [`ENDS_WITH_NAMED_MACRO`].
        flags: u8,
        /// The entry's position in the table's profile array; 0 means "needs
        /// nothing", and the array is then never consulted.
        profile: u8,
    }

    impl StaticEntry {
        /// The filler an array of entries is built from before it is written.
        const FILLER: StaticEntry = StaticEntry { encoded: "", flags: MODE_TEXT, profile: 0 };

        /// The compiled form of one source entry. The termination is read off
        /// `encoded` with [`ValueTermination::inspect`], at compile time.
        const fn new(encoded: &'static str, mode: ValueMode, profile: ProfileIndex) -> Self {
            let mode_bits = match mode {
                ValueMode::TextOnly => MODE_TEXT,
                ValueMode::MathOnly => MODE_MATH,
                ValueMode::AnyMode => MODE_ANY,
            };
            let termination_bit = match ValueTermination::inspect(encoded) {
                ValueTermination::ValueIsSelfTerminating => 0,
                ValueTermination::ValueEndsWithNamedMacro => ENDS_WITH_NAMED_MACRO,
            };
            StaticEntry { encoded, flags: mode_bits | termination_bit, profile: profile.0 }
        }

        /// The hint the two flag bytes stand for.
        const fn hint(&self) -> ReplacementProtectionHint {
            let mode = match self.flags & MODE_MASK {
                MODE_MATH => ValueMode::MathOnly,
                MODE_ANY => ValueMode::AnyMode,
                _ => ValueMode::TextOnly,
            };
            let termination = if self.flags & ENDS_WITH_NAMED_MACRO != 0 {
                ValueTermination::ValueEndsWithNamedMacro
            } else {
                ValueTermination::ValueIsSelfTerminating
            };
            ReplacementProtectionHint::Value { mode, termination }
        }

        /// The entry as a `LookupTable` answers it, resolving the profile
        /// index against `profiles`. Index 0 is `None` without consulting the
        /// array, and an index out of its range — which the compile-time
        /// check rules out — is `None` too rather than a panic.
        pub(super) fn view(&self, profiles: &'static [Profile]) -> TableEntry<'static> {
            TableEntry {
                encoded: self.encoded,
                hint: self.hint(),
                needs: if self.profile == 0 { None } else { profiles.get(self.profile as usize) },
            }
        }
    }

    /// The source form of a table, as the macro receives it.
    pub type Entries = &'static [(char, &'static str, ValueMode, ProfileIndex)];

    /// The block a character belongs to: its code point without the low byte.
    const fn high(ch: char) -> u32 {
        (ch as u32) >> 8
    }

    /// Checks everything a compiled table must hold, panicking — that is,
    /// failing to compile — on the first violation. See the module
    /// documentation for the list.
    ///
    /// # Panics
    ///
    /// When the entries are not strictly ascending, when an entry names a
    /// profile outside `n_profiles`, when a value is not ASCII, has
    /// unbalanced braces or ends with a lone backslash, or when there are too
    /// many entries for a `u16` position.
    pub const fn check(entries: Entries, n_profiles: usize) {
        assert!(
            entries.len() < u16::MAX as usize,
            "compile_static_table!: too many entries to number with a u16"
        );
        let mut i = 0;
        while i < entries.len() {
            assert!(
                i == 0 || (entries[i - 1].0 as u32) < (entries[i].0 as u32),
                "compile_static_table!: the entries must be sorted by character, \
                 strictly ascending, with no repeated character"
            );
            assert!(
                (entries[i].3 .0 as usize) < n_profiles,
                "compile_static_table!: an entry names a profile index that the \
                 profile array does not have"
            );
            check_value(entries[i].1);
            i += 1;
        }
    }

    /// Checks one encoded value: ASCII, balanced braces, no lone backslash at
    /// its end.
    ///
    /// # Panics
    ///
    /// On any of those three.
    const fn check_value(encoded: &str) {
        let bytes = encoded.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            assert!(
                bytes[i] < 128,
                "compile_static_table!: an encoded value holds a character outside ASCII"
            );
            i += 1;
        }
        // Braces, with `\{`, `\}` and `\\` read as the escapes they are.
        let mut depth: usize = 0;
        i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'\\' => {
                    assert!(
                        i + 1 < bytes.len(),
                        "compile_static_table!: an encoded value ends with a lone backslash"
                    );
                    i += 1;
                }
                b'{' => depth += 1,
                b'}' => {
                    assert!(
                        depth > 0,
                        "compile_static_table!: an encoded value closes a brace it never opened"
                    );
                    depth -= 1;
                }
                _ => {}
            }
            i += 1;
        }
        assert!(depth == 0, "compile_static_table!: an encoded value leaves a brace open");
    }

    /// The number of distinct blocks the entries fall into.
    pub const fn count_blocks(entries: Entries) -> usize {
        let mut blocks = 0;
        let mut i = 0;
        while i < entries.len() {
            if i == 0 || high(entries[i].0) != high(entries[i - 1].0) {
                blocks += 1;
            }
            i += 1;
        }
        blocks
    }

    /// The ASCII characters the entries cover: the table's
    /// [`ascii_keys`](super::LookupTable::ascii_keys).
    pub const fn ascii_keys(entries: Entries) -> AsciiSet {
        let mut set = AsciiSet::EMPTY;
        let mut i = 0;
        while i < entries.len() {
            let code_point = entries[i].0 as u32;
            if code_point < 128 {
                set = set.with(code_point as u8);
            }
            i += 1;
        }
        set
    }

    /// The characters of the entries, in order. `N` is the number of entries.
    pub const fn keys<const N: usize>(entries: Entries) -> [char; N] {
        let mut out = ['\0'; N];
        let mut i = 0;
        while i < N {
            out[i] = entries[i].0;
            i += 1;
        }
        out
    }

    /// The compiled payloads of the entries, in order. `N` is the number of
    /// entries.
    pub const fn values<const N: usize>(entries: Entries) -> [StaticEntry; N] {
        let mut out = [StaticEntry::FILLER; N];
        let mut i = 0;
        while i < N {
            out[i] = StaticEntry::new(entries[i].1, entries[i].2, entries[i].3);
            i += 1;
        }
        out
    }

    /// The low byte of each entry's character, in order. `N` is the number of
    /// entries.
    pub const fn lows<const N: usize>(entries: Entries) -> [u8; N] {
        let mut out = [0u8; N];
        let mut i = 0;
        while i < N {
            out[i] = entries[i].0 as u32 as u8;
            i += 1;
        }
        out
    }

    /// The distinct blocks of the entries, ascending. `K` is
    /// [`count_blocks`].
    pub const fn blocks<const K: usize>(entries: Entries) -> [u32; K] {
        let mut out = [0u32; K];
        let mut block = 0;
        let mut i = 0;
        while i < entries.len() {
            if i == 0 || high(entries[i].0) != high(entries[i - 1].0) {
                out[block] = high(entries[i].0);
                block += 1;
            }
            i += 1;
        }
        out
    }

    /// Where each block begins, with the number of entries at the end. `K1`
    /// is [`count_blocks`] plus one.
    pub const fn starts<const K1: usize>(entries: Entries) -> [u16; K1] {
        let mut out = [0u16; K1];
        let mut block = 0;
        let mut i = 0;
        while i < entries.len() {
            if i == 0 || high(entries[i].0) != high(entries[i - 1].0) {
                out[block] = i as u16;
                block += 1;
            }
            i += 1;
        }
        out[block] = entries.len() as u16;
        out
    }

    /// One 256-slot index per block, from the low byte of a character to the
    /// position of its entry, or `u16::MAX` where the block has no entry for
    /// that byte. `K` is [`count_blocks`].
    pub const fn direct_index<const K: usize>(entries: Entries) -> [[u16; 256]; K] {
        let mut out = [[NO_ENTRY; 256]; K];
        let mut block = 0;
        let mut i = 0;
        while i < entries.len() {
            if i > 0 && high(entries[i].0) != high(entries[i - 1].0) {
                block += 1;
            }
            out[block][(entries[i].0 as u32 & 0xFF) as usize] = i as u16;
            i += 1;
        }
        out
    }

    /// The `binary_search` layout over the arrays the macro declared.
    pub const fn new_binary_search(
        keys: &'static [char],
        values: &'static [StaticEntry],
        profiles: &'static [Profile],
        ascii_keys: AsciiSet,
    ) -> StaticTableBinarySearch {
        StaticTableBinarySearch { keys, values, profiles, ascii_keys }
    }

    /// The `two_level_linear` layout over the arrays the macro declared.
    pub const fn new_two_level_linear(
        blocks: &'static [u32],
        starts: &'static [u16],
        lows: &'static [u8],
        values: &'static [StaticEntry],
        profiles: &'static [Profile],
        ascii_keys: AsciiSet,
    ) -> StaticTableTwoLevelLinear {
        StaticTableTwoLevelLinear { blocks, starts, lows, values, profiles, ascii_keys }
    }

    /// The `two_level_direct_index` layout over the arrays the macro
    /// declared.
    pub const fn new_two_level_direct(
        blocks: &'static [u32],
        index: &'static [[u16; 256]],
        values: &'static [StaticEntry],
        profiles: &'static [Profile],
        ascii_keys: AsciiSet,
    ) -> StaticTableTwoLevelDirect {
        StaticTableTwoLevelDirect { blocks, index, values, profiles, ascii_keys }
    }
}

/// Compiles a constant list of entries and an array of profiles into a static
/// lookup table.
///
/// ```text
/// compile_static_table!(ENTRIES, PROFILES, LAYOUT)
/// ```
///
/// - `ENTRIES` is a constant expression of type
///   `&'static [(char, &'static str, ValueMode, ProfileIndex)]` — normally a
///   `const` item, since a list of this shape written out in the macro call
///   would be a very long token stream. The characters must ascend strictly.
/// - `PROFILES` is a constant or static expression of type
///   `&'static [Profile]`: the profiles an entry's [`ProfileIndex`] names.
///   Position 0 is never consulted — index 0 means "needs nothing" — but it
///   must exist, so that an index is a position of the array. Write
///   `&PROFILES` for a `static PROFILES: [Profile; N]`.
/// - `LAYOUT` is one of the three bare words `binary_search`,
///   `two_level_linear` and `two_level_direct_index`, and it decides which of
///   [`StaticTableBinarySearch`], [`StaticTableTwoLevelLinear`] and
///   [`StaticTableTwoLevelDirect`] the macro evaluates to.
///
/// The macro declares the `static` arrays the layout is made of and evaluates
/// to the layout struct itself, so that it can initialize a `static` of your
/// own. Everything it computes it computes at compile time, and the checks
/// listed in the [module documentation](self) are compile errors.
///
/// ```
/// use untechxt::lookuptable::LookupTable;
/// use untechxt::preamble::Profile;
/// use untechxt::protection::ValueMode;
/// use untechxt::statictable::{compile_static_table, ProfileIndex};
///
/// static PROFILES: [Profile; 1] = [Profile::from_static(&[])];
/// const ENTRIES: &[(char, &str, ValueMode, ProfileIndex)] =
///     &[('\u{2014}', r"\textemdash", ValueMode::TextOnly, ProfileIndex::NONE)];
///
/// static TABLE: untechxt::statictable::StaticTableBinarySearch =
///     compile_static_table!(ENTRIES, &PROFILES, binary_search);
/// assert_eq!(TABLE.lookup('\u{2014}').unwrap().encoded, r"\textemdash");
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __compile_static_table {
    ($entries:expr, $profiles:expr, binary_search) => {{
        const __UNTECHXT_ENTRIES: $crate::statictable::__build::Entries = $entries;
        const __UNTECHXT_N: usize = __UNTECHXT_ENTRIES.len();
        const __UNTECHXT_PROFILE_COUNT: usize = $profiles.len();
        const _: () =
            $crate::statictable::__build::check(__UNTECHXT_ENTRIES, __UNTECHXT_PROFILE_COUNT);
        static __UNTECHXT_KEYS: [char; __UNTECHXT_N] =
            $crate::statictable::__build::keys::<__UNTECHXT_N>(__UNTECHXT_ENTRIES);
        static __UNTECHXT_VALUES: [$crate::statictable::__build::StaticEntry; __UNTECHXT_N] =
            $crate::statictable::__build::values::<__UNTECHXT_N>(__UNTECHXT_ENTRIES);
        $crate::statictable::__build::new_binary_search(
            &__UNTECHXT_KEYS,
            &__UNTECHXT_VALUES,
            $profiles,
            $crate::statictable::__build::ascii_keys(__UNTECHXT_ENTRIES),
        )
    }};
    ($entries:expr, $profiles:expr, two_level_linear) => {{
        const __UNTECHXT_ENTRIES: $crate::statictable::__build::Entries = $entries;
        const __UNTECHXT_N: usize = __UNTECHXT_ENTRIES.len();
        const __UNTECHXT_K: usize =
            $crate::statictable::__build::count_blocks(__UNTECHXT_ENTRIES);
        const __UNTECHXT_PROFILE_COUNT: usize = $profiles.len();
        const _: () =
            $crate::statictable::__build::check(__UNTECHXT_ENTRIES, __UNTECHXT_PROFILE_COUNT);
        static __UNTECHXT_BLOCKS: [u32; __UNTECHXT_K] =
            $crate::statictable::__build::blocks::<__UNTECHXT_K>(__UNTECHXT_ENTRIES);
        static __UNTECHXT_STARTS: [u16; __UNTECHXT_K + 1] =
            $crate::statictable::__build::starts::<{ __UNTECHXT_K + 1 }>(__UNTECHXT_ENTRIES);
        static __UNTECHXT_LOWS: [u8; __UNTECHXT_N] =
            $crate::statictable::__build::lows::<__UNTECHXT_N>(__UNTECHXT_ENTRIES);
        static __UNTECHXT_VALUES: [$crate::statictable::__build::StaticEntry; __UNTECHXT_N] =
            $crate::statictable::__build::values::<__UNTECHXT_N>(__UNTECHXT_ENTRIES);
        $crate::statictable::__build::new_two_level_linear(
            &__UNTECHXT_BLOCKS,
            &__UNTECHXT_STARTS,
            &__UNTECHXT_LOWS,
            &__UNTECHXT_VALUES,
            $profiles,
            $crate::statictable::__build::ascii_keys(__UNTECHXT_ENTRIES),
        )
    }};
    ($entries:expr, $profiles:expr, two_level_direct_index) => {{
        const __UNTECHXT_ENTRIES: $crate::statictable::__build::Entries = $entries;
        const __UNTECHXT_N: usize = __UNTECHXT_ENTRIES.len();
        const __UNTECHXT_K: usize =
            $crate::statictable::__build::count_blocks(__UNTECHXT_ENTRIES);
        const __UNTECHXT_PROFILE_COUNT: usize = $profiles.len();
        const _: () =
            $crate::statictable::__build::check(__UNTECHXT_ENTRIES, __UNTECHXT_PROFILE_COUNT);
        static __UNTECHXT_BLOCKS: [u32; __UNTECHXT_K] =
            $crate::statictable::__build::blocks::<__UNTECHXT_K>(__UNTECHXT_ENTRIES);
        static __UNTECHXT_INDEX: [[u16; 256]; __UNTECHXT_K] =
            $crate::statictable::__build::direct_index::<__UNTECHXT_K>(__UNTECHXT_ENTRIES);
        static __UNTECHXT_VALUES: [$crate::statictable::__build::StaticEntry; __UNTECHXT_N] =
            $crate::statictable::__build::values::<__UNTECHXT_N>(__UNTECHXT_ENTRIES);
        $crate::statictable::__build::new_two_level_direct(
            &__UNTECHXT_BLOCKS,
            &__UNTECHXT_INDEX,
            &__UNTECHXT_VALUES,
            $profiles,
            $crate::statictable::__build::ascii_keys(__UNTECHXT_ENTRIES),
        )
    }};
}
