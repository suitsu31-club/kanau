use crate::krate::kanau;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

pub fn derive_proto_des(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let kanau = kanau();
    quote! {
        #[automatically_derived]
        impl #kanau::message::MessageDe for #name {
            type DeError = ::prost::DecodeError;

            fn from_bytes(bytes: &[u8]) -> ::core::result::Result<Self, Self::DeError>
            where
                Self: Sized
            {
                <Self as ::prost::Message>::decode(bytes)
            }
        }
    }
    .into()
}

pub fn derive_proto_ser(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let kanau = kanau();
    quote! {
        #[automatically_derived]
        impl #kanau::message::MessageSer for #name {
            type SerError = ::prost::EncodeError;

            fn to_bytes(self) -> ::core::result::Result<::std::boxed::Box<[u8]>, Self::SerError> {
                let mut buf = ::std::vec::Vec::new();
                <Self as ::prost::Message>::encode(&self, &mut buf)?;
                ::core::result::Result::Ok(buf.into_boxed_slice())
            }
        }
    }
    .into()
}
