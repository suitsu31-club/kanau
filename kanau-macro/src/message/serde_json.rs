use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub fn derive_serde_json_byte_des(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    quote! {
        impl kanau::message::MessageDe for #name {
            type DeError = serde_json::Error;

            fn from_bytes(bytes: &[u8]) -> Result<Self, Self::DeError>
            where
                Self: Sized
            {
                serde_json::from_slice(bytes)
            }
        }
    }.into()
}

pub fn derive_serde_json_byte_ser(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    quote! {
        impl kanau::message::MessageSer for #name {
            type SerError = serde_json::Error;

            fn to_bytes(self) -> Result<Box<[u8]>, Self::SerError> {
                serde_json::to_vec(&self).map(|v| v.into_boxed_slice())
            }
        }
    }.into()
}
