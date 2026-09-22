use kanau::message::{MessageDe, MessageSer};
use kanau::{JsonMessageDe, JsonMessageSer};
use std::borrow::Cow;
use std::marker::PhantomData;

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, JsonMessageDe, JsonMessageSer)]
struct Wrapper<T>(T);

#[derive(serde::Serialize, JsonMessageSer)]
struct Borrowed<'a> {
    s: &'a str,
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, JsonMessageDe, JsonMessageSer)]
struct Named<'a> {
    name: Cow<'a, str>,
}

// serde has no const-generic array impls, so the parameter rides alongside the data.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, JsonMessageDe, JsonMessageSer)]
struct Sized_<const N: usize> {
    xs: Vec<u8>,
}

/// Not serializable: the derive must not demand `T: Serialize` for a skipped marker.
#[derive(Debug, PartialEq)]
struct Opaque;

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, JsonMessageDe, JsonMessageSer)]
struct Tagged<T> {
    id: u64,
    #[serde(skip)]
    _marker: PhantomData<T>,
}

fn main() {
    let bytes = Wrapper(7u32).to_bytes().unwrap();
    assert_eq!(Wrapper::<u32>::from_bytes(&bytes).unwrap(), Wrapper(7));

    let bytes = Borrowed { s: "hi" }.to_bytes().unwrap();
    assert_eq!(&*bytes, br#"{"s":"hi"}"#);

    let bytes = Named { name: Cow::Borrowed("hi") }.to_bytes().unwrap();
    assert_eq!(Named::from_bytes(&bytes).unwrap(), Named { name: Cow::Borrowed("hi") });

    let bytes = Sized_::<3> { xs: vec![1, 2, 3] }.to_bytes().unwrap();
    assert_eq!(Sized_::<3>::from_bytes(&bytes).unwrap(), Sized_ { xs: vec![1, 2, 3] });

    let tagged = Tagged::<Opaque> { id: 1, _marker: PhantomData };
    let bytes = tagged.to_bytes().unwrap();
    assert_eq!(
        Tagged::<Opaque>::from_bytes(&bytes).unwrap(),
        Tagged { id: 1, _marker: PhantomData }
    );
}
