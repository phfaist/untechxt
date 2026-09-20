

pub trait ProfileCollection {
    pub fn get(id : ProfileId) -> ChunkPreamble;
}




pub struct NeedsProfile {
    collection : & ProfileCollection,
}








 
#[macro_export]
macro_rules! compile_static_table {
    ($entries:expr) => {{
        
    }}

}
