use rkyv::{access, deserialize, rancor, Archive, Archived, Deserialize, Serialize};
use crate::message::{MessageDe, MessageSer};

#[derive(Debug, PartialEq, Clone, Archive, Serialize, Deserialize)]
struct ExampleUser {
    user_id: u64,
    username: String,
    email: Option<String>,
    user_age: u8,
    is_active: bool,
}

impl MessageDe for ExampleUser {
    type DeError = rancor::Error;

    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::DeError>
    where
        Self: Sized,
    {
        let archived = access::<Archived<Self>, rancor::Error>(bytes)?;
        let de = deserialize(archived)?;
        Ok(de)
    }
}

impl MessageSer for ExampleUser {
    type SerError = rancor::Error;

    fn to_bytes(self) -> Result<Box<[u8]>, Self::SerError> {
        let bytes = rkyv::to_bytes::<rancor::Error>(&self)?;
        Ok(bytes.into_boxed_slice())
    }
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
