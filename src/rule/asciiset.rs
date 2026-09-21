//! [`AsciiSet`]: a set of ASCII characters held as a bitmap.

use core::fmt;
use core::ops::{BitOr, RangeInclusive};

/// A set of ASCII characters, one bit per character, held in a `u128`.
///
/// Membership is a shift and a mask, which is what makes the encoder's scan
/// loop cheap: a rule announces through [`Rule::ascii_triggers`] the ASCII
/// characters it can match at, the encoder unions those sets once, and every
/// input byte below `0x80` outside the union is copied without decoding a
/// character or calling a rule.
///
/// A byte of `0x80` or more is in no set: the type describes ASCII alone.
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
    /// Every ASCII character, `\0` through `\x7F`. The default answer of
    /// [`Rule::ascii_triggers`](crate::rule::Rule::ascii_triggers): "I may
    /// match anywhere".
    pub const ALL: Self = AsciiSet(u128::MAX);

    /// No character at all. The triggers of a rule that never matches at an
    /// ASCII character, such as a table of non-ASCII entries.
    pub const EMPTY: Self = AsciiSet(0);

    /// The set of the characters of `chars`.
    ///
    /// # Panics
    ///
    /// If `chars` holds a character outside ASCII. In a `const` item that is
    /// a compile error.
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

    /// The set of the bytes of `range`, both ends included. An empty range
    /// (`start > end`) gives the empty set.
    ///
    /// # Panics
    ///
    /// If the range holds a byte of `0x80` or more.
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

    /// The set of the ASCII bytes that `f` answers `true` for. `f` is called
    /// once for each of the 128 bytes, here and never again.
    pub fn from_fn(f: impl Fn(u8) -> bool) -> Self {
        let mut bits = 0u128;
        for byte in 0u8..128 {
            if f(byte) {
                bits |= 1u128 << byte;
            }
        }
        AsciiSet(bits)
    }

    /// The union of the two sets. [`BitOr`] (`a | b`) does the same outside a
    /// `const` context.
    pub const fn union(self, other: Self) -> Self {
        AsciiSet(self.0 | other.0)
    }

    /// Whether `byte` is in the set. Always `false` for a byte of `0x80` or
    /// more.
    pub const fn contains(self, byte: u8) -> bool {
        byte < 128 && (self.0 >> byte) & 1 != 0
    }

    /// Whether the set holds no character at all.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The number of characters in the set.
    pub(crate) const fn count(self) -> usize {
        self.0.count_ones() as usize
    }

    /// The set with `byte` added.
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
    /// The members, written out: `AsciiSet("#$%&")`, with non-printable
    /// characters as `\xNN`. The two constants print as their names.
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
