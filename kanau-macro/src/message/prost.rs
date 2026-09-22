use super::derive_impl;
use crate::krate::kanau;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input, parse_quote};

pub fn derive_proto_des(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let kanau = kanau();
    derive_impl(
        &input,
        quote!(#kanau::message::MessageDe),
        [parse_quote!(Self: ::prost::Message + ::core::default::Default)],
        quote! {
            type DeError = ::prost::DecodeError;

            fn from_bytes(bytes: &[u8]) -> ::core::result::Result<Self, Self::DeError>
            where
                Self: Sized
            {
                <Self as ::prost::Message>::decode(bytes)
            }
        },
    )
    .into()
}

pub fn derive_proto_ser(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let kanau = kanau();
    derive_impl(
        &input,
        quote!(#kanau::message::MessageSer),
        [parse_quote!(Self: ::prost::Message)],
        quote! {
            type SerError = ::prost::EncodeError;

            fn to_bytes(self) -> ::core::result::Result<::std::boxed::Box<[u8]>, Self::SerError> {
                let mut buf = ::std::vec::Vec::new();
                <Self as ::prost::Message>::encode(&self, &mut buf)?;
                ::core::result::Result::Ok(buf.into_boxed_slice())
            }
        },
    )
    .into()
}
