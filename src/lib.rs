#![deny(rustdoc::broken_intra_doc_links)]
#![warn(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![doc = include_str!("../README.md")]

pub mod processor;

/// Monadic flow control.
pub mod flow;

pub mod layer;

#[cfg(feature = "message")]
/// Message passing tool in MQ.
pub mod message;

pub mod chain;

#[cfg(test)]
mod tests;

/// Codec crates for the message derives' generated code. Not public API.
///
/// Generated impls name codecs through here rather than at the consumer's crate
/// root, so enabling a backend feature is sufficient on its own and the codec is
/// always the version `kanau` was built against.
#[cfg(feature = "message")]
#[doc(hidden)]
pub mod __private {
    #[cfg(feature = "bincode")]
    pub use bincode;
    #[cfg(feature = "musli-wire")]
    pub use musli;
    #[cfg(feature = "prost")]
    pub use prost;
    #[cfg(feature = "rkyv")]
    pub use rkyv;
    #[cfg(feature = "serde_json")]
    pub use serde;
    #[cfg(feature = "serde_json")]
    pub use serde_json;
}

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

#[cfg(all(feature = "musli-wire", feature = "message"))]
/// Musli wire message deserialization.
pub use kanau_macro::MusliWireMessageDe;

#[cfg(all(feature = "musli-wire", feature = "message"))]
/// Musli wire message serialization.
pub use kanau_macro::MusliWireMessageSer;
