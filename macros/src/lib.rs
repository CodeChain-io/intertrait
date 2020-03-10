extern crate proc_macro;

use proc_macro::TokenStream;

use syn::parse_macro_input;
use syn::{DeriveInput, ItemImpl};

use cast_to_args::CastToArgs;
use castable_to_args::CastableToArgs;

use crate::gen_caster::generate_caster;

mod cast_to_args;
mod castable_to_args;
mod gen_caster;
mod item_impl;
mod item_type;

#[proc_macro_attribute]
pub fn cast_to(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as CastToArgs);
    let expanded = match args {
        CastToArgs::None => item_impl::process(parse_macro_input!(input as ItemImpl)),
        CastToArgs::Traits(paths) => {
            item_type::process(paths, parse_macro_input!(input as DeriveInput))
        }
    };
    expanded.into()
}

#[proc_macro]
pub fn castable_to(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as CastableToArgs);
    args.traits
        .iter()
        .flat_map(|t| generate_caster(&args.ty, t))
        .collect::<proc_macro2::TokenStream>()
        .into()
}
