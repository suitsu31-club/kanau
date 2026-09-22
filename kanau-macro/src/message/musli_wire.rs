use crate::krate::kanau;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

pub fn derive_musli_wire_byte_des(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let kanau = kanau();
    quote! {
        #[automatically_derived]
        impl #kanau::message::MessageDe for #name {
            type DeError = ::musli::wire::Error;

            fn from_bytes(bytes: &[u8]) -> ::core::result::Result<Self, Self::DeError>
            where
                Self: Sized
            {
                ::musli::wire::from_slice(bytes)
            }
        }
    }
    .into()
}

pub fn derive_musli_wire_byte_ser(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let kanau = kanau();
    quote! {
        #[automatically_derived]
        impl #kanau::message::MessageSer for #name {
            type SerError = ::musli::wire::Error;

            fn to_bytes(self) -> ::core::result::Result<::std::boxed::Box<[u8]>, Self::SerError> {
                ::musli::wire::to_vec(&self).map(|v| v.into_boxed_slice())
            }
        }
    }
    .into()
}
