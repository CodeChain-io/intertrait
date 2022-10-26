use std::collections::HashSet;

use proc_macro2::TokenStream;
use syn::spanned::Spanned;
use syn::{DeriveInput, Path};

use quote::{quote, quote_spanned};

use crate::args::Flag;
use crate::gen_caster::generate_caster;

pub fn process(flags: &HashSet<Flag>, paths: Vec<Path>, mut input: DeriveInput) -> TokenStream {
    let DeriveInput {
        ref mut attrs,
        ref ident,
        ref generics,
        ..
    } = input;

    let intertrait_path = crate::attr::intertrait_path(attrs).unwrap();

    let generated = if generics.lt_token.is_some() {
        quote_spanned! {
            generics.span() => compile_error!("#[cast_to(..)] can't be used on a generic type definition");
        }
    } else {
        paths
            .into_iter()
            .flat_map(|t| generate_caster(ident, &t, flags.contains(&Flag::Sync), &intertrait_path))
            .collect()
    };
    quote! {
        #input
        #generated
    }
}
