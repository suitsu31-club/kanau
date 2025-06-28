
#[cfg(all(feature = "serde_json", feature = "message"))]
mod serde_json_macro_inner;

#[cfg(all(feature = "bincode", feature = "message"))]
mod bincode_macro_inner;

#[cfg(all(feature = "rkyv", feature = "message"))]
mod rkyv_macro_inner;

#[cfg(all(feature = "bincode", feature = "message"))]
mod bincode_macro;
mod rkyv_macro;
mod serde_json_macro;
