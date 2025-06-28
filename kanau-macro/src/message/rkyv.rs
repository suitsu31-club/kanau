use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub fn derive_rkyv_byte_des(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    quote! {
        impl kanau::message::MessageDe for #name {
            type DeError = rkyv::rancor::Error;

            fn from_bytes(bytes: &[u8]) -> Result<Self, Self::DeError>
            where
                Self: Sized
            {
                use rkyv::{Archive, Deserialize};
                let archived = rkyv::access::<rkyv::Archived<Self>, rkyv::rancor::Error>(bytes)?;
                let de = rkyv::deserialize(archived)?;
                Ok(de)
            }
        }
    }.into()
}

pub fn derive_rkyv_byte_ser(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    quote! {
        impl kanau::message::MessageSer for #name {
            type SerError = rkyv::rancor::Error;

            fn to_bytes(self) -> Result<Box<[u8]>, Self::SerError> {
                use rkyv::{Archive, Serialize};
                let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&self)?;
                Ok(bytes.into_boxed_slice())
            }
        }
    }.into()
}
