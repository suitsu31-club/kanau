use crate::krate::kanau;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

pub fn derive_rkyv_byte_des(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let kanau = kanau();
    quote! {
        #[automatically_derived]
        impl #kanau::message::MessageDe for #name {
            type DeError = ::rkyv::rancor::Error;

            fn from_bytes(bytes: &[u8]) -> ::core::result::Result<Self, Self::DeError>
            where
                Self: Sized
            {
                let archived =
                    ::rkyv::access::<::rkyv::Archived<Self>, ::rkyv::rancor::Error>(bytes)?;
                ::rkyv::deserialize(archived)
            }
        }
    }
    .into()
}

pub fn derive_rkyv_byte_ser(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let kanau = kanau();
    quote! {
        #[automatically_derived]
        impl #kanau::message::MessageSer for #name {
            type SerError = ::rkyv::rancor::Error;

            fn to_bytes(self) -> ::core::result::Result<::std::boxed::Box<[u8]>, Self::SerError> {
                ::rkyv::to_bytes::<::rkyv::rancor::Error>(&self)
                    .map(|bytes| bytes.into_boxed_slice())
            }
        }
    }
    .into()
}
