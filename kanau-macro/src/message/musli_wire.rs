use super::derive_impl;
use crate::krate::kanau;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input, parse_quote};

pub fn derive_musli_wire_byte_des(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let kanau = kanau();
    derive_impl(
        &input,
        quote!(#kanau::message::MessageDe),
        // Reserved-looking lifetime: must not shadow one declared on the type (E0496).
        [parse_quote!(
            Self: for<'__kanau> ::musli::Decode<
                '__kanau,
                ::musli::mode::Binary,
                ::musli::alloc::Global,
            >
        )],
        quote! {
            type DeError = ::musli::wire::Error;

            fn from_bytes(bytes: &[u8]) -> ::core::result::Result<Self, Self::DeError>
            where
                Self: Sized
            {
                ::musli::wire::from_slice(bytes)
            }
        },
    )
    .into()
}

pub fn derive_musli_wire_byte_ser(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let kanau = kanau();
    derive_impl(
        &input,
        quote!(#kanau::message::MessageSer),
        [parse_quote!(Self: ::musli::Encode<::musli::mode::Binary>)],
        quote! {
            type SerError = ::musli::wire::Error;

            fn to_bytes(self) -> ::core::result::Result<::std::boxed::Box<[u8]>, Self::SerError> {
                ::musli::wire::to_vec(&self).map(|v| v.into_boxed_slice())
            }
        },
    )
    .into()
}
