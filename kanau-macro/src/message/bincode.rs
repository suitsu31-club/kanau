use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub fn derive_bincode_byte_des(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    quote! {
        impl kanau::message::MessageDe for #name {
            type DeError = bincode::error::DecodeError;

            fn from_bytes(bytes: &[u8]) -> Result<Self, Self::DeError>
            where
                Self: Sized
            {
                bincode::decode_from_slice(bytes, bincode::config::standard()).map(|(res, _)| res)
            }
        }
    }.into()
}

pub fn derive_bincode_byte_ser(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    quote! {
        impl kanau::message::MessageSer for #name {
            type SerError = bincode::error::EncodeError;

            fn to_bytes(self) -> Result<Box<[u8]>, Self::SerError> {
                bincode::encode_to_vec(&self, bincode::config::standard()).map(|v| v.into_boxed_slice())
            }
        }
    }.into()
}
