//! The [`AsciiSet`] struct: a set of ASCII characters held as a bitmap.

use core::fmt;
use core::ops::{BitOr, RangeInclusive};

/// A set of ASCII characters, one bit per character, held in a `u128`.
///
/// A membership test is a shift and a mask, which is what keeps the
/// encoder's scan loop cheap. A rule reports the ASCII characters it may
/// match at through the method [`Rule::ascii_triggers`], the encoder takes
/// the union of those sets once, and every input byte below `0x80` outside
/// the union is copied without decoding a character or calling a rule.
///
/// A byte of `0x80` or greater belongs to no set, because an [`AsciiSet`]
/// describes ASCII characters alone.
///
/// [`Rule::ascii_triggers`]: crate::rule::Rule::ascii_triggers
///
/// ```
/// use untechxt::rule::AsciiSet;
///
/// const SPECIALS: AsciiSet = AsciiSet::of(r"#$%&");
/// assert!(SPECIALS.contains(b'%'));
/// assert!(!SPECIALS.contains(b'a'));
/// assert!(!AsciiSet::ALL.contains(0x80));
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AsciiSet(u128);

impl AsciiSet {
    /// Every ASCII character, `\0` through `\x7F`. The value that the method
    /// [`Rule::ascii_triggers`](crate::rule::Rule::ascii_triggers) returns
    /// by default, meaning that the rule may match at any ASCII character.
    pub const ALL: Self = AsciiSet(u128::MAX);

    /// No character at all. The set for a rule that never matches at an
    /// ASCII character, such as a lookup table whose entries are all
    /// non-ASCII.
    pub const EMPTY: Self = AsciiSet(0);

    /// Returns the set of the characters in `chars`.
    ///
    /// # Panics
    ///
    /// Panics if `chars` contains a character outside ASCII. In a `const`
    /// item that panic is a compile error instead.
    pub const fn of(chars: &str) -> Self {
        let bytes = chars.as_bytes();
        let mut bits = 0u128;
        let mut i = 0;
        while i < bytes.len() {
            let byte = bytes[i];
            assert!(byte < 128, "AsciiSet::of: the string holds a non-ASCII character");
            bits |= 1u128 << byte;
            i += 1;
        }
        AsciiSet(bits)
    }

    /// Returns the set of the bytes in `range`, both ends included. An empty
    /// range, where `start > end`, returns the empty set.
    ///
    /// # Panics
    ///
    /// Panics if the range contains a byte of `0x80` or greater.
    pub const fn range(range: RangeInclusive<u8>) -> Self {
        let (start, end) = (*range.start(), *range.end());
        if start > end {
            return AsciiSet::EMPTY;
        }
        assert!(end < 128, "AsciiSet::range: the range holds a non-ASCII byte");
        let mut bits = 0u128;
        let mut byte = start;
        loop {
            bits |= 1u128 << byte;
            if byte == end {
                break;
            }
            byte += 1;
        }
        AsciiSet(bits)
    }

    /// Returns the set of the ASCII bytes for which `f` returns `true`. This
    /// method calls `f` once for each of the 128 ASCII bytes and never again.
    pub fn from_fn(f: impl Fn(u8) -> bool) -> Self {
        let mut bits = 0u128;
        for byte in 0u8..128 {
            if f(byte) {
                bits |= 1u128 << byte;
            }
        }
        AsciiSet(bits)
    }

    /// Returns the union of the two sets. The [`BitOr`] operator (`a | b`)
    /// does the same thing outside a `const` context.
    pub const fn union(self, other: Self) -> Self {
        AsciiSet(self.0 | other.0)
    }

    /// Returns whether `byte` is in the set. Returns `false` for any byte of
    /// `0x80` or greater.
    pub const fn contains(self, byte: u8) -> bool {
        byte < 128 && (self.0 >> byte) & 1 != 0
    }

    /// Returns whether the set contains no character at all.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns the number of characters in the set.
    pub(crate) const fn count(self) -> usize {
        self.0.count_ones() as usize
    }

    /// Returns the set with `byte` added.
    ///
    /// # Panics
    ///
    /// Panics if `byte` is `0x80` or greater.
    pub(crate) const fn with(self, byte: u8) -> Self {
        assert!(byte < 128, "AsciiSet::with: non-ASCII byte");
        AsciiSet(self.0 | 1u128 << byte)
    }
}

impl BitOr for AsciiSet {
    type Output = AsciiSet;

    fn bitor(self, other: AsciiSet) -> AsciiSet {
        self.union(other)
    }
}

impl fmt::Debug for AsciiSet {
    /// Formats the set by writing out its members: `AsciiSet("#$%&")`, with
    /// each non-printable character shown as `\xNN`. The constants
    /// [`AsciiSet::ALL`] and [`AsciiSet::EMPTY`] format as their own names.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == AsciiSet::ALL {
            return f.write_str("AsciiSet::ALL");
        }
        if *self == AsciiSet::EMPTY {
            return f.write_str("AsciiSet::EMPTY");
        }
        f.write_str("AsciiSet(\"")?;
        for byte in 0u8..128 {
            if self.contains(byte) {
                if (0x20..0x7f).contains(&byte) {
                    write!(f, "{}", byte as char)?;
                } else {
                    write!(f, "\\x{byte:02x}")?;
                }
            }
        }
        f.write_str("\")")
    }
}
