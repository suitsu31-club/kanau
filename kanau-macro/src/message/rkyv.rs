use super::derive_impl;
use crate::krate::{codec, kanau};
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input, parse_quote};

// The higher-ranked lifetimes below use a reserved-looking name so they cannot
// shadow a lifetime parameter declared on the deriving type (E0496).

pub fn derive_rkyv_byte_des(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let kanau = kanau();
    let rkyv = codec(&kanau, "rkyv");
    derive_impl(
        &input,
        quote!(#kanau::message::MessageDe),
        [
            parse_quote!(Self: #rkyv::Archive),
            parse_quote!(
                #rkyv::Archived<Self>: for<'__kanau> #rkyv::bytecheck::CheckBytes<
                        #rkyv::api::high::HighValidator<'__kanau, #rkyv::rancor::Error>,
                    > + #rkyv::Deserialize<
                        Self,
                        #rkyv::api::high::HighDeserializer<#rkyv::rancor::Error>,
                    >
            ),
        ],
        quote! {
            type DeError = #rkyv::rancor::Error;

            fn from_bytes(bytes: &[u8]) -> ::core::result::Result<Self, Self::DeError>
            where
                Self: Sized
            {
                if bytes.as_ptr() as usize % #rkyv::util::AlignedVec::<16>::ALIGNMENT == 0 {
                    let archived =
                        #rkyv::access::<#rkyv::Archived<Self>, #rkyv::rancor::Error>(bytes)?;
                    #rkyv::deserialize(archived)
                } else {
                    let mut aligned =
                        #rkyv::util::AlignedVec::<16>::with_capacity(bytes.len());
                    aligned.extend_from_slice(bytes);
                    let archived = #rkyv::access::<#rkyv::Archived<Self>, #rkyv::rancor::Error>(
                        &aligned,
                    )?;
                    #rkyv::deserialize(archived)
                }
            }
        },
    )
    .into()
}

pub fn derive_rkyv_byte_ser(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let kanau = kanau();
    let rkyv = codec(&kanau, "rkyv");
    derive_impl(
        &input,
        quote!(#kanau::message::MessageSer),
        [parse_quote!(
            Self: for<'__kanau> #rkyv::Serialize<
                #rkyv::api::high::HighSerializer<
                    #rkyv::util::AlignedVec,
                    #rkyv::ser::allocator::ArenaHandle<'__kanau>,
                    #rkyv::rancor::Error,
                >,
            >
        )],
        quote! {
            type SerError = #rkyv::rancor::Error;

            fn to_bytes(self) -> ::core::result::Result<::std::boxed::Box<[u8]>, Self::SerError> {
                #rkyv::to_bytes::<#rkyv::rancor::Error>(&self)
                    .map(|bytes| bytes.into_boxed_slice())
            }
        },
    )
    .into()
}
