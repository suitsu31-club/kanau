use super::derive_impl;
use crate::krate::kanau;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input, parse_quote};

pub fn derive_serde_json_byte_des(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let kanau = kanau();
    derive_impl(
        &input,
        quote!(#kanau::message::MessageDe),
        [parse_quote!(Self: ::serde::de::DeserializeOwned)],
        quote! {
            type DeError = ::serde_json::Error;

            fn from_bytes(bytes: &[u8]) -> ::core::result::Result<Self, Self::DeError>
            where
                Self: Sized
            {
                ::serde_json::from_slice(bytes)
            }
        },
    )
    .into()
}

pub fn derive_serde_json_byte_ser(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let kanau = kanau();
    derive_impl(
        &input,
        quote!(#kanau::message::MessageSer),
        [parse_quote!(Self: ::serde::Serialize)],
        quote! {
            type SerError = ::serde_json::Error;

            fn to_bytes(self) -> ::core::result::Result<::std::boxed::Box<[u8]>, Self::SerError> {
                ::serde_json::to_vec(&self).map(::std::vec::Vec::into_boxed_slice)
            }
        },
    )
    .into()
}
