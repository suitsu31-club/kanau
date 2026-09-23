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

/// Path to a codec crate re-exported by `kanau`, given the path from [`kanau`].
///
/// Generated code must not name the codec at the consumer's crate root: the
/// consumer enabled a `kanau` feature, not necessarily a direct dependency, and
/// a direct dependency may be a different major version than the one `kanau`
/// implements its error conversions for.
pub(crate) fn codec(kanau: &TokenStream, name: &str) -> TokenStream {
    let ident = Ident::new(name, Span::call_site());
    quote!(#kanau::__private::#ident)
}
