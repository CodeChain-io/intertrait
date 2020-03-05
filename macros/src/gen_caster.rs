use std::str::from_utf8_unchecked;

use proc_macro2::TokenStream;
use syn::Path;
use uuid::adapter::Simple;
use uuid::Uuid;

use quote::format_ident;
use quote::quote;
use quote::ToTokens;

pub fn generate_caster(ty: &impl ToTokens, trait_: &Path) -> TokenStream {
    let mut static_buf = [0u8; STATIC_BUF_LEN];
    let static_var = format_ident!("{}", generate_static(&mut static_buf));
    let mut fn_buf = [0u8; FN_BUF_LEN];
    let fn_ident = format_ident!("{}", generate_fn(&mut fn_buf));

    quote! {
        #[linkme::distributed_slice(intertrait::CASTERS)]
        static #static_var: fn() -> (std::any::TypeId, intertrait::BoxedCaster) = #fn_ident;

        fn #fn_ident() -> (std::any::TypeId, intertrait::BoxedCaster) {
            let type_id = std::any::TypeId::of::<#ty>();
            let caster = Box::new(intertrait::Caster::<dyn #trait_> {
                cast_ref: |from| from.downcast_ref::<#ty>().map(|c| c as &dyn #trait_),
                cast_mut: |from| from.downcast_mut::<#ty>().map(|c| c as &mut dyn #trait_),
                cast_box: |from| from.downcast::<#ty>().map(|c| c as Box<dyn #trait_>),
            });
            (type_id, caster)
        }
    }
}

const STATIC_PREFIX: &[u8] = b"__";
const STATIC_BUF_LEN: usize = STATIC_PREFIX.len() + Simple::LENGTH;

const FN_PREFIX: &[u8] = b"__";
const FN_BUF_LEN: usize = FN_PREFIX.len() + Simple::LENGTH;

fn generate_static(buf: &mut [u8]) -> &str {
    buf[..STATIC_PREFIX.len()].copy_from_slice(STATIC_PREFIX);
    Uuid::new_v4()
        .to_simple()
        .encode_upper(&mut buf[STATIC_PREFIX.len()..]);
    unsafe { from_utf8_unchecked(&buf[..STATIC_BUF_LEN]) }
}

fn generate_fn(buf: &mut [u8]) -> &str {
    buf[..FN_PREFIX.len()].copy_from_slice(FN_PREFIX);
    Uuid::new_v4()
        .to_simple()
        .encode_lower(&mut buf[FN_PREFIX.len()..]);
    unsafe { from_utf8_unchecked(&buf[..FN_BUF_LEN]) }
}
