#![cfg(all(feature = "musli-wire", feature = "message"))]

use crate::message::{MessageDe, MessageSer};
use kanau_macro::{MusliWireMessageDe, MusliWireMessageSer};

#[derive(
    Debug,
    PartialEq,
    Clone,
    musli::Encode,
    musli::Decode,
    MusliWireMessageDe,
    MusliWireMessageSer,
)]
struct ExampleUser {
    pub user_id: u64,
    pub username: String,
    pub email: Option<String>,
    pub user_age: u8,
    pub is_active: bool,
}

/// A newer revision of [`ExampleUser`] adding a defaulted field, used to prove the
/// wire format stays upgrade stable across schema evolution.
#[derive(
    Debug,
    PartialEq,
    Clone,
    musli::Encode,
    musli::Decode,
    MusliWireMessageDe,
    MusliWireMessageSer,
)]
struct ExampleUserV2 {
    pub user_id: u64,
    pub username: String,
    pub email: Option<String>,
    pub user_age: u8,
    pub is_active: bool,
    #[musli(default)]
    pub nickname: Option<String>,
}

fn sample() -> ExampleUser {
    ExampleUser {
        user_id: 1,
        username: "John".to_string(),
        email: Some("john@example.com".to_string()),
        user_age: 30,
        is_active: true,
    }
}

#[test]
fn test_musli_wire_message() {
    let user = sample();
    let user_clone = user.clone();

    let bytes = user.to_bytes().unwrap();
    let user2 = ExampleUser::from_bytes(&bytes).unwrap();

    assert_eq!(user_clone, user2);
}

#[test]
fn test_musli_wire_message_is_upgrade_stable() {
    // An old peer's payload must decode on a newer schema, with the added field defaulted.
    let bytes = sample().to_bytes().unwrap();
    let upgraded = ExampleUserV2::from_bytes(&bytes).unwrap();
    assert_eq!(upgraded.username, "John");
    assert_eq!(upgraded.nickname, None);

    // A new peer's payload must decode on the old schema, skipping the unknown field.
    let bytes = ExampleUserV2 {
        user_id: 1,
        username: "John".to_string(),
        email: Some("john@example.com".to_string()),
        user_age: 30,
        is_active: true,
        nickname: Some("Johnny".to_string()),
    }
    .to_bytes()
    .unwrap();
    let downgraded = ExampleUser::from_bytes(&bytes).unwrap();
    assert_eq!(downgraded, sample());
}

#[test]
fn test_musli_wire_decode_error_converts() {
    let err = ExampleUser::from_bytes(&[0xff, 0xff, 0xff, 0xff]).unwrap_err();
    let _: crate::message::DeserializeError = err.into();
}
