use proc_macro2::TokenStream;
use syn::spanned::Spanned;
use syn::{DeriveInput, Path};

use quote::quote;
use quote::quote_spanned;

use crate::gen_caster::generate_caster;

pub fn process(paths: Vec<Path>, input: DeriveInput) -> TokenStream {
    let DeriveInput {
        ref ident,
        ref generics,
        ..
    } = input;
    if generics.lt_token.is_some() {
        let error = quote_spanned! {
            generics.span() => compile_error!("#[cast_to(..)] can't be used on a generic type definition");
        };
        quote! {
            #error
            #input
        }
    } else {
        let generated: TokenStream = paths
            .into_iter()
            .flat_map(|t| generate_caster(ident, &t))
            .collect();
        quote! {
            #input
            #generated
        }
    }
}
