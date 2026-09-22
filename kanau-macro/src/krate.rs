use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

/// Path to the `kanau` crate as seen from the call site.
///
/// Resolves through `Cargo.toml`, so a consumer that renames the dependency
/// (`kanau_renamed = { package = "kanau", .. }`) still gets a valid path, and
/// `kanau`'s own tests resolve to `crate`.
pub(crate) fn kanau() -> TokenStream {
    match crate_name("kanau") {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
        // Not declared as a dependency at all. `::kanau` yields the clearest
        // diagnostic ("use of unresolved crate `kanau`") at the call site.
        Err(_) => quote!(::kanau),
    }
}
