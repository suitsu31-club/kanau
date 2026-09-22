//! Every message derive must honour type, lifetime and const generics.
//!
//! Cases compile as an external consumer of `kanau`, the same way users hit them.

#[test]
fn message_derives_support_generics() {
    let t = trybuild::TestCases::new();
    #[cfg(all(feature = "message", feature = "serde_json"))]
    t.pass("tests/generics/serde_json.rs");
    #[cfg(all(feature = "message", feature = "bincode"))]
    t.pass("tests/generics/bincode.rs");
    #[cfg(all(feature = "message", feature = "rkyv"))]
    t.pass("tests/generics/rkyv.rs");
    #[cfg(all(feature = "message", feature = "musli-wire"))]
    t.pass("tests/generics/musli_wire.rs");
    #[cfg(all(feature = "message", feature = "prost"))]
    t.pass("tests/generics/prost.rs");
    let _ = &t;
}
