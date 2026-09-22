use kanau::message::{MessageDe, MessageSer};
use kanau::{BincodeMessageDe, BincodeMessageSer};

#[derive(Debug, PartialEq, bincode::Encode, bincode::Decode, BincodeMessageDe, BincodeMessageSer)]
struct Wrapper<T>(T);

#[derive(bincode::Encode, BincodeMessageSer)]
struct Borrowed<'a> {
    s: &'a str,
}

#[derive(Debug, PartialEq, bincode::Encode, bincode::Decode, BincodeMessageDe, BincodeMessageSer)]
struct Arr<const N: usize> {
    xs: [u8; N],
}

fn main() {
    let bytes = Wrapper(7u32).to_bytes().unwrap();
    assert_eq!(Wrapper::<u32>::from_bytes(&bytes).unwrap(), Wrapper(7));

    let bytes = Borrowed { s: "hi" }.to_bytes().unwrap();
    assert!(!bytes.is_empty());

    let bytes = Arr { xs: [1u8, 2, 3] }.to_bytes().unwrap();
    assert_eq!(Arr::<3>::from_bytes(&bytes).unwrap(), Arr { xs: [1, 2, 3] });
}
