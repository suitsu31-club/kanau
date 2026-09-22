use crate::message::{MessageDe, MessageSer};

#[derive(Debug, PartialEq, Clone, musli::Encode, musli::Decode)]
struct ExampleUser {
    pub user_id: u64,
    pub username: String,
    pub email: Option<String>,
    pub user_age: u8,
    pub is_active: bool,
}

impl MessageDe for ExampleUser {
    type DeError = musli::wire::Error;

    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::DeError>
    where
        Self: Sized,
    {
        musli::wire::from_slice(bytes)
    }
}

impl MessageSer for ExampleUser {
    type SerError = musli::wire::Error;

    fn to_bytes(self) -> Result<Box<[u8]>, Self::SerError> {
        musli::wire::to_vec(&self).map(|v| v.into_boxed_slice())
    }
}

#[test]
fn test_musli_wire_message() {
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
