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

/// Attached on an `impl` item or type definition, registers traits as targets for casting.
///
/// If on an `impl` item, no argument is allowed. But on a type definition, the target traits
/// must be listed explicitly.
///
/// # Examples
/// ## On a trait impl
/// ```
/// struct Data;
///
/// trait Greet {
///     fn greet(&self);
/// }
///
/// // Greet can be cast into from any CastFrom sub-trait object backed by Data.
/// #[cast_to]
/// impl Greet for Data {
///     fn greet(&self) {
///         println!("Hello");
///     }
/// }
/// ```
///
/// ## On a type definition
/// Use when a target trait is derived or implemented in an external crate.
/// ```
/// // Debug can be cast into from any CastFrom sub-trait object backed by Data.
/// #[cast_to(std::fmt::Debug)]
/// #[derive(std::fmt::Debug)]
/// struct Data;
/// ```
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

/// Declare target traits for casting implemented by a type.
///
/// This macro is for registering both a concrete type and its traits to be targets for casting.
/// Useful when the type definition and the trait implementations are in an external crate.
///
/// **Note**: this macro cannot be used in an expression or statement due to
/// [the current limitation](https://github.com/rust-lang/rust/pull/68717) in the stable Rust.
///
/// # Examples
/// ```
/// #[derive(std::fmt::Debug)]
/// enum Data {
///     A, B, C
/// }
/// trait Greet {
///     fn greet(&self);
/// }
/// impl Greet for Data {
///     fn greet(&self) {
///         println!("Hello");
///     }
/// }
/// castable_to! { Data: std::fmt::Debug, Greet }
/// ```
#[proc_macro]
pub fn castable_to(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as CastableToArgs);
    args.traits
        .iter()
        .flat_map(|t| generate_caster(&args.ty, t))
        .collect::<proc_macro2::TokenStream>()
        .into()
}
