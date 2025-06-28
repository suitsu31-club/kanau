#![deny(rustdoc::broken_intra_doc_links)]
#![warn(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![doc = include_str!("../README.md")]

/// Async closure `Fn(I) -> Future<Output = O>` as a trait.
pub mod processor;

/// Monadic flow control.
pub mod flow;

/// Something that wraps around a processor.
pub mod layer;

#[cfg(feature = "message")]
/// Message passing tool in MQ.
pub mod message;

#[cfg(test)]
mod tests;

#[cfg(all(feature = "bincode", feature = "message"))]
/// Bincode message deserialization.
pub use kanau_macro::BincodeMessageDe;

#[cfg(all(feature = "bincode", feature = "message"))]
/// Bincode message serialization.
pub use kanau_macro::BincodeMessageSer;

#[cfg(all(feature = "serde_json", feature = "message"))]
/// Serde json message deserialization.
pub use kanau_macro::JsonMessageDe;

#[cfg(all(feature = "serde_json", feature = "message"))]
/// Serde json message serialization.   
pub use kanau_macro::JsonMessageSer;

#[cfg(all(feature = "rkyv", feature = "message"))]
/// Rkyv message deserialization.
pub use kanau_macro::RkyvMessageDe;

#[cfg(all(feature = "rkyv", feature = "message"))]
/// Rkyv message serialization.
pub use kanau_macro::RkyvMessageSer;

#[cfg(all(feature = "prost", feature = "message"))]
/// Prost message deserialization.
pub use kanau_macro::ProstMessageDe;

#[cfg(all(feature = "prost", feature = "message"))]
/// Prost message serialization.
pub use kanau_macro::ProstMessageSer;
