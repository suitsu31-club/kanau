use kanau::message::{MessageDe, MessageSer};
use kanau::{ProstMessageDe, ProstMessageSer};

#[derive(Clone, PartialEq, prost::Message, ProstMessageDe, ProstMessageSer)]
// prost's derive emits a `Debug` impl that does not inherit the bound from `Message`.
struct Wrapper<T: prost::Message + Default + std::fmt::Debug> {
    #[prost(message, optional, tag = "1")]
    inner: Option<T>,
}

#[derive(Clone, PartialEq, prost::Message)]
struct Leaf {
    #[prost(uint32, tag = "1")]
    value: u32,
}

fn main() {
    let msg = Wrapper { inner: Some(Leaf { value: 7 }) };
    let bytes = msg.clone().to_bytes().unwrap();
    assert_eq!(Wrapper::<Leaf>::from_bytes(&bytes).unwrap(), msg);
}
