//! Regression guard for macro hygiene.
//!
//! The derives must not depend on `Result`, `Box`, `Vec` or `Ok` resolving to their
//! prelude meanings at the call site. Shadowing them here is not contrived: a
//! consumer module doing `use std::io::Result;` hits the same failure.
//!
//! Note this module deliberately does *not* alias `crate` to `kanau`. That, plus the
//! other test modules no longer needing `use crate as kanau;`, is what covers the
//! call-site crate-path resolution.
#![cfg(all(feature = "message", feature = "serde_json", feature = "bincode"))]
#![allow(dead_code)]

use crate::message::{MessageDe, MessageSer};

#[allow(clippy::upper_case_acronyms)]
struct Result;
struct Box;
struct Vec;
struct Ok;

#[derive(
    Debug, PartialEq, serde::Serialize, serde::Deserialize, kanau_macro::JsonMessageDe,
    kanau_macro::JsonMessageSer,
)]
struct Json {
    a: u32,
}

#[derive(
    Debug, PartialEq, bincode::Encode, bincode::Decode, kanau_macro::BincodeMessageDe,
    kanau_macro::BincodeMessageSer,
)]
struct Bin {
    a: u32,
}

#[test]
fn derives_roundtrip_with_prelude_names_shadowed() {
    let json = Json { a: 7 }.to_bytes().unwrap();
    assert_eq!(Json::from_bytes(&json).unwrap(), Json { a: 7 });

    let bin = Bin { a: 7 }.to_bytes().unwrap();
    assert_eq!(Bin::from_bytes(&bin).unwrap(), Bin { a: 7 });
}
