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

