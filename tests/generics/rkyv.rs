use kanau::message::{MessageDe, MessageSer};
use kanau::{RkyvMessageDe, RkyvMessageSer};

#[derive(
    Debug, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, RkyvMessageDe, RkyvMessageSer,
)]
struct Wrapper<T>(T);

#[derive(
    Debug, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, RkyvMessageDe, RkyvMessageSer,
)]
struct Arr<const N: usize> {
    xs: [u8; N],
}

// A lifetime that must not collide with the derive's own higher-ranked lifetime.
#[derive(rkyv::Archive, rkyv::Serialize, RkyvMessageSer)]
struct Borrowed<'a> {
    #[rkyv(with = rkyv::with::InlineAsBox)]
    s: &'a str,
}

fn main() {
    let bytes = Wrapper(7u32).to_bytes().unwrap();
    assert_eq!(Wrapper::<u32>::from_bytes(&bytes).unwrap(), Wrapper(7));

    let bytes = Arr { xs: [1u8, 2, 3] }.to_bytes().unwrap();
    assert_eq!(Arr::<3>::from_bytes(&bytes).unwrap(), Arr { xs: [1, 2, 3] });

    let bytes = Borrowed { s: "hi" }.to_bytes().unwrap();
    assert!(!bytes.is_empty());
}
