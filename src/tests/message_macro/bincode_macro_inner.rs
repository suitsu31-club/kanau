use crate::message::{MessageDe, MessageSer};

#[derive(Debug, PartialEq, Clone, bincode::Encode, bincode::Decode)]
struct ExampleUser {
    pub user_id: u64,
    pub username: String,
    pub email: Option<String>,
    pub user_age: u8,
    pub is_active: bool,
}

impl MessageDe for ExampleUser {
    type DeError = bincode::error::DecodeError;

    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::DeError>
    where
        Self: Sized
    {
        bincode::decode_from_slice(bytes, bincode::config::standard()).map(|(res, _)| res)
    }
}

impl MessageSer for ExampleUser {
    type SerError = bincode::error::EncodeError;

    fn to_bytes(self) -> Result<Box<[u8]>, Self::SerError> {
        bincode::encode_to_vec(&self, bincode::config::standard()).map(|v| v.into_boxed_slice())
    }
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
