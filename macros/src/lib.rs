extern crate proc_macro;

use proc_macro::TokenStream;

use syn::parse_macro_input;
use syn::{DeriveInput, ItemImpl};

use args::Args;

mod args;
mod gen_caster;
mod item_impl;
mod item_type;

#[proc_macro_attribute]
pub fn cast_to(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as Args);
    let expanded = match args {
        Args::None => item_impl::process(parse_macro_input!(input as ItemImpl)),
        Args::Traits(paths) => item_type::process(paths, parse_macro_input!(input as DeriveInput)),
    };
    expanded.into()
}
