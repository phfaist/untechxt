
use super::statictable::StaticProfiles;

use super::ChunkPreamble;
use super::statictable_def::{compile_static_needs_profiles,};


const CHUNKS : [(&str, ChunkPreamble)] = &[
    "amsmath",
    ChunkPreamble::Package("amsmath"),
]


pub const PROFILES = compile_static_needs_profiles!(CHUNKS);
