use syn::parse::{Error, ParseStream, Result};
use syn::{parse_quote, Attribute, Path, Token};

// #[intertrait(crate = path::to::intertrait)]
pub(crate) fn intertrait_path(attrs: &mut Vec<Attribute>) -> Result<Path> {
    let mut intertrait_path = None;
    let mut errors: Option<Error> = None;

    attrs.retain(|attr| {
        if !attr.path.is_ident("intertrait") {
            return true;
        }
        match attr.parse_args_with(|input: ParseStream| {
            input.parse::<Token![crate]>()?;
            input.parse::<Token![=]>()?;
            input.call(Path::parse_mod_style)
        }) {
            Ok(path) => intertrait_path = Some(path),
            Err(err) => match &mut errors {
                None => errors = Some(err),
                Some(errors) => errors.combine(err),
            },
        }
        false
    });

    match errors {
        None => Ok(intertrait_path.unwrap_or_else(|| parse_quote!(::intertrait))),
        Some(errors) => Err(errors),
    }
}
