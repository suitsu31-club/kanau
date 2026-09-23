//! The reported case: `serde` is a direct dependency (its derive needs it), `serde_json` is not.

use kanau::message::{MessageDe, MessageSer};
use kanau::{JsonMessageDe, JsonMessageSer};

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, JsonMessageDe, JsonMessageSer)]
struct Msg {
    a: u32,
}

fn main() {
    let bytes = Msg { a: 7 }.to_bytes().unwrap();
    assert_eq!(&*bytes, br#"{"a":7}"#);
    assert_eq!(Msg::from_bytes(&bytes).unwrap(), Msg { a: 7 });
}
