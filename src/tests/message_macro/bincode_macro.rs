use kanau_macro::{BincodeMessageDe, BincodeMessageSer};
use crate as kanau;
use crate::message::{MessageDe, MessageSer};

#[derive(Debug, PartialEq, Clone, bincode::Encode, bincode::Decode, BincodeMessageDe, BincodeMessageSer)]
struct ExampleUser {
    pub user_id: u64,
    pub username: String,
    pub email: Option<String>,
    pub user_age: u8,
    pub is_active: bool,
}

#[test]
fn test_bincode_message() {
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
