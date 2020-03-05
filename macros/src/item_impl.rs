use proc_macro2::TokenStream;
use syn::spanned::Spanned;
use syn::ItemImpl;

use quote::quote;
use quote::quote_spanned;

use crate::gen_caster::generate_caster;

pub fn process(input: ItemImpl) -> TokenStream {
    let ItemImpl {
        ref self_ty,
        ref trait_,
        ..
    } = input;

    let generated = match trait_ {
        None => quote_spanned! {
            self_ty.span() => compile_error!("#[cast_to] should only be on an impl of a trait");
        },
        Some(trait_) => match trait_ {
            (Some(bang), _, _) => quote_spanned! {
                bang.span() => compile_error!("#[cast_to] is not for !Trait impl");
            },
            (None, path, _) => generate_caster(&self_ty, &path),
        },
    };

    quote! {
        #input
        #generated
    }
}
