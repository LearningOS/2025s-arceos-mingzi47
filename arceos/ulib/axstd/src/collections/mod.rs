pub mod hashmap;

#[cfg(feature = "alloc")]
pub use hashmap::HashMap as HashMap;
