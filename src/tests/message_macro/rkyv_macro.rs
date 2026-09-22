#![cfg(all(feature = "rkyv", feature = "message"))]

use crate::message::{MessageDe, MessageSer};
use kanau_macro::{RkyvMessageDe, RkyvMessageSer};

#[derive(
    Debug,
    PartialEq,
    Clone,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    RkyvMessageDe,
    RkyvMessageSer,
)]
struct ExampleUser {
    pub user_id: u64,
    pub username: String,
    pub email: Option<String>,
    pub user_age: u8,
    pub is_active: bool,
}

#[test]
fn test_rkyv_message() {
    let user = ExampleUser {
        user_id: 1,
        username: "John".to_string(),
        email: Some("john@example.com".to_string()),
        user_age: 30,
        is_active: true,
    };

    let user_clone = user.clone();

    let bytes = user.to_bytes().unwrap();
    let user2 = ExampleUser::from_bytes(&bytes).unwrap();

    assert_eq!(user_clone, user2);
}

#[test]
fn test_rkyv_message_misaligned_buffer() {
    let user = ExampleUser {
        user_id: 1,
        username: "John".to_string(),
        email: Some("john@example.com".to_string()),
        user_age: 30,
        is_active: true,
    };
    let bytes = user.clone().to_bytes().unwrap();

    // Place the payload at offset 1 of a 16-aligned buffer, so the slice is
    // misaligned regardless of what the allocator hands out.
    let mut buf = rkyv::util::AlignedVec::<16>::new();
    buf.push(0);
    buf.extend_from_slice(&bytes);
    let misaligned = &buf[1..];
    assert_eq!(misaligned.as_ptr() as usize % 8, 1);

    assert_eq!(ExampleUser::from_bytes(misaligned).unwrap(), user);
}
