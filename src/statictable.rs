use crate::NeedsProfile;

use crate::lookuptable::{LookupTable, ValuePayload};



pub struct StaticTableBinarySearch {
    pub(crate) keys: &'static [char],
    pub(crate) values: &'static [ValuePayload],
}

pub struct StaticTableTwoLevelLinear {
    pub(crate) his: &'static [u32],    // distinct (cp >> 8), one per block
    pub(crate) starts: &'static [u16], // len = blocks + 1; block b = starts[b]..starts[b+1]
    pub(crate) lows: &'static [u8],    // (cp & 0xFF) for each entry
    pub(crate) values: &'static [ValuePayload],
}

pub struct StaticTableTwoLevelDirect {
    pub(crate) his: &'static [u32],
    pub(crate) idx: &'static [[u16; 256]], // low byte -> entry index, u16::MAX = none
    pub(crate) values: &'static [ValuePayload],
}


impl LookupTable for StaticTableBinarySearch {

    #[inline]
    pub fn lookup(&self, c: char) -> Option<ValuePayload> {
        self.keys.binary_search(&c).ok().map(|i| self.values[i]);
    }
}

impl LookupTable for StaticTableTwoLevelLinear {

    #[inline]
    pub fn lookup(&self, c: char) -> Option<ValuePayload> {
        let cp = c as u32;
        let b = self.his.iter().position(|&h| h == cp >> 8)?;
        let (s, e) = (self.starts[b] as usize, self.starts[b + 1] as usize);
        let lo = cp as u8;
        let i = self.lows[s..e].iter().position(|&l| l == lo)?;
        Some(self.values[s + i])
    }
}

impl LookupTable for StaticTableTwoLevelDirect {

    #[inline]
    pub fn lookup(&self, c: char) -> Option<ValuePayload> {
        let cp = c as u32;
        let b = self.his.iter().position(|&h| h == cp >> 8)?;
        match self.idx[b][(cp & 0xFF) as usize] {
            u16::MAX => None,
            i => Some(self.values[i as usize]),
        }
    }
}
 






/// Const-eval builders. Panics here become compile errors.
#[doc(hidden)]
pub mod __build {

    type Entries = &'static [(char, ValuePayload)];
 
    const fn hi(c: char) -> u32 {
        (c as u32) >> 8
    }
 
    pub const fn check(e: Entries) {
        assert!(e.len() < u16::MAX as usize, "too many entries for u16 indices");
        let mut i = 1;
        while i < e.len() {
            assert!(
                (e[i - 1].0 as u32) < (e[i].0 as u32),
                "entries must be sorted by code point with no duplicates"
            );
            i += 1;
        }
    }
 
    pub const fn count_blocks(e: Entries) -> usize {
        let (mut n, mut i) = (0, 0);
        while i < e.len() {
            if i == 0 || hi(e[i].0) != hi(e[i - 1].0) {
                n += 1;
            }
            i += 1;
        }
        n
    }
 
    pub const fn keys<const N: usize>(e: Entries) -> [char; N] {
        let mut out = ['\0'; N];
        let mut i = 0;
        while i < N {
            out[i] = e[i].0;
            i += 1;
        }
        out
    }
 
    pub const fn values<const N: usize>(e: Entries) -> [ValuePayload; N] {
        let mut out = [""; N];
        let mut i = 0;
        while i < N {
            out[i] = ValuePayload(e[i].1, e[i].2);
            i += 1;
        }
        out
    }
 
    pub const fn lows<const N: usize>(e: Entries) -> [u8; N] {
        let mut out = [0u8; N];
        let mut i = 0;
        while i < N {
            out[i] = e[i].0 as u32 as u8;
            i += 1;
        }
        out
    }
 
    pub const fn his<const K: usize>(e: Entries) -> [u32; K] {
        let mut out = [0u32; K];
        let (mut b, mut i) = (0, 0);
        while i < e.len() {
            if i == 0 || hi(e[i].0) != hi(e[i - 1].0) {
                out[b] = hi(e[i].0);
                b += 1;
            }
            i += 1;
        }
        out
    }
 
    /// `K1` = number of blocks + 1.
    pub const fn starts<const K1: usize>(e: Entries) -> [u16; K1] {
        let mut out = [0u16; K1];
        let (mut b, mut i) = (0, 0);
        while i < e.len() {
            if i == 0 || hi(e[i].0) != hi(e[i - 1].0) {
                out[b] = i as u16;
                b += 1;
            }
            i += 1;
        }
        out[b] = e.len() as u16;
        out
    }
 
    pub const fn direct_idx<const K: usize>(e: Entries) -> [[u16; 256]; K] {
        let mut out = [[u16::MAX; 256]; K];
        let (mut b, mut i) = (0, 0);
        while i < e.len() {
            if i > 0 && hi(e[i].0) != hi(e[i - 1].0) {
                b += 1;
            }
            out[b][(e[i].0 as u32 & 0xFF) as usize] = i as u16;
            i += 1;
        }
        out
    }

    pub fn new_binary_search(keys : &'static [char], values : &'static [ValuePayload]) {
        StaticTableBinarySearch {
            keys,
            values,
        }
    }
    pub fn new_two_level_linear(
        his: &'static [u32],
        starts: &'static [u16],
        lows: &'static [u8],
        values: &'static [ValuePayload],
    ) {
        StaticTableTwoLevelLinear {
            his, starts, lows, values
        }
    }

    pub fn new_two_level_direct(
        his: &'static [u32],
        idx: &'static [[u16; 256]],
        values: &'static [ValuePayload],
    ) {
        StaticTableTwoLevelDirect { his, idx, values }
    }
}



#[macro_export]
macro_rules! compile_static_profiles {
    ($entries:expr) {

    }
}

 
#[macro_export]
macro_rules! compile_static_table {
    ($entries:expr, binary_search) => {{
        const __STE_ENTRIES: &'static [(char, &'static str, NeedsProfile)] = $entries;
        const __STE_N: usize = __STE_ENTRIES.len();
        const _: () = $crate::statictable::__build::check(__STE_ENTRIES);
        static __STE_KEYS: [char; __STE_N] =
            $crate::statictable::__build::keys::<__STE_N>(__STE_ENTRIES);
        static __STE_VALUES: [ValuePayload; __STE_N] =
            $crate::statictable::__build::values::<__STE_N>(__STE_ENTRIES);
        $crate::statictable::__build::new_binary_search(&__STE_KEYS, &__STE_VALUES)
    }};
    ($entries:expr, two_level_linear) => {{
        const __STE_ENTRIES: &'static [(char, &'static str, NeedsProfile)] = $entries;
        const __STE_N: usize = __STE_ENTRIES.len();
        const __STE_K: usize = $crate::statictable::__build::count_blocks(__STE_ENTRIES);
        const _: () = $crate::statictable::__build::check(__STE_ENTRIES);
        static __STE_HIS: [u32; __STE_K] =
            $crate::statictable::__build::his::<__STE_K>(__STE_ENTRIES);
        static __STE_STARTS: [u16; __STE_K + 1] =
            $crate::statictable::__build::starts::<{ __STE_K + 1 }>(__STE_ENTRIES);
        static __STE_LOWS: [u8; __STE_N] =
            $crate::statictable::__build::lows::<__STE_N>(__STE_ENTRIES);
        static __STE_VALUES: [ValuePayload; __STE_N] =
            $crate::statictable::__build::values::<__STE_N>(__STE_ENTRIES);
        $crate::statictable::__build::new_two_level_linear(
            &__STE_HIS, &__STE_STARTS, &__STE_LOWS, &__STE_VALUES
        )
    }};
    ($entries:expr, two_level_direct_index) => {{
        const __STE_ENTRIES: &'static [(char, &'static str, NeedsProfile)] = $entries;
        const __STE_N: usize = __STE_ENTRIES.len();
        const __STE_K: usize = $crate::statictable::__build::count_blocks(__STE_ENTRIES);
        const _: () = $crate::statictable::__build::check(__STE_ENTRIES);
        static __STE_HIS: [u32; __STE_K] =
            $crate::statictable::__build::his::<__STE_K>(__STE_ENTRIES);
        static __STE_IDX: [[u16; 256]; __STE_K] =
            $crate::statictable::__build::direct_idx::<__STE_K>(__STE_ENTRIES);
        static __STE_VALUES: [ValuePayload; __STE_N] =
            $crate::statictable::__build::values::<__STE_N>(__STE_ENTRIES);
        $crate::statictable::__build::new_two_level_direct(
            &__STE_HIS, &__STE_IDX, &__STE_VALUES
        )
    }};
}
 
