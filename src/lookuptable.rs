
use crate::{NeedsProfile, ReplacementProtectionHints};


pub struct ValuePayload(& 'static str, NeedsProfile);


pub trait LookupTable {

    #[inline]
    pub fn lookup(&self, c: char) -> Option<ValuePayload>;
}





pub struct LookupTableBinarySearch {
    pub entries : Vec<(char, &'static str, NeedsProfile)>;
}

impl LookupTable for LookupTableBinarySearch {

    #[inline]
    pub fn lookup(&self, c: char) -> Option<ValuePayload> {
        self.entries
            .binary_search_by_key(&c, |&(entry_char, _, _)| entry_char)
            .ok()
            .map(|index| ValuePayload(self.entries[index].1, self.entries.[index].2))
    }

}
