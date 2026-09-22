pub(crate) mod bincode;
pub(crate) mod musli_wire;
pub(crate) mod prost;
pub(crate) mod rkyv;
pub(crate) mod serde_json;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, WherePredicate};

/// Emits `impl #trait_path for #ident` carrying the input's generics.
///
/// `bounds` are appended to the input's where clause. Backends bound on `Self`
/// (e.g. `Self: ::serde::Serialize`) rather than on each type parameter: that
/// defers to whatever bounds the codec's own derive chose, so `PhantomData`
/// fields and custom `#[serde(bound = "...")]` are not over-constrained.
pub(crate) fn derive_impl(
    input: &DeriveInput,
    trait_path: TokenStream,
    bounds: impl IntoIterator<Item = WherePredicate>,
    body: TokenStream,
) -> TokenStream {
    let name = &input.ident;
    let mut generics = input.generics.clone();
    generics.make_where_clause().predicates.extend(bounds);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote! {
        #[automatically_derived]
        impl #impl_generics #trait_path for #name #ty_generics #where_clause {
            #body
        }
    }
}
